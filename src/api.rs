use std::time::Duration;

use anyhow::{bail, Context, Result};
use futures_util::StreamExt;
use reqwest::{Client, RequestBuilder, Response};
use serde_json::{json, Value};
use tokio::task::JoinHandle;

use crate::{
    chat::Message,
    event::{Ev, Tx},
};

/// Semua yang dibutuhkan untuk bicara ke satu provider.
#[derive(Clone)]
pub struct Target {
    pub id: &'static str,
    pub base_url: String,
    pub key: Option<String>,
    pub model: String,
}

/// Jalankan chat streaming di background; hasilnya datang lewat `Ev`.
pub fn spawn_chat(client: Client, target: Target, history: Vec<Message>, tx: Tx) -> JoinHandle<()> {
    tokio::spawn(async move {
        let ev = match stream_chat(&client, &target, &history, &tx).await {
            Ok(()) => Ev::Done,
            Err(e) => Ev::Failed(format!("{e:#}")),
        };
        let _ = tx.send(ev);
    })
}

/// Ambil daftar model asli dari provider di background.
pub fn spawn_models(client: Client, target: Target, tx: Tx) {
    tokio::spawn(async move {
        let result = fetch_models(&client, &target).await.map_err(|e| format!("{e:#}"));
        let _ = tx.send(Ev::Models { provider: target.id, result });
    });
}

fn authed(req: RequestBuilder, t: &Target) -> RequestBuilder {
    match &t.key {
        // Endpoint /models milik Anthropic butuh header native.
        Some(key) if t.id == "anthropic" => req
            .bearer_auth(key)
            .header("x-api-key", key)
            .header("anthropic-version", "2023-06-01"),
        Some(key) => req.bearer_auth(key),
        None => req,
    }
}

async fn ensure_ok(resp: Response) -> Result<Response> {
    let status = resp.status();
    if status.is_success() {
        return Ok(resp);
    }
    let body = resp.text().await.unwrap_or_default();
    bail!("{status}: {}", body.chars().take(160).collect::<String>())
}

async fn stream_chat(client: &Client, t: &Target, history: &[Message], tx: &Tx) -> Result<()> {
    let messages: Vec<Value> = history.iter().map(Message::to_json).collect();
    let body = json!({ "model": t.model, "messages": messages, "stream": true });
    let url = format!("{}/chat/completions", t.base_url);
    let resp = authed(client.post(url), t)
        .json(&body)
        .send()
        .await
        .context("gagal konek")?;

    let mut stream = ensure_ok(resp).await?.bytes_stream();
    let mut buf: Vec<u8> = Vec::new();
    while let Some(chunk) = stream.next().await {
        buf.extend_from_slice(&chunk?);
        while let Some(pos) = buf.iter().position(|&b| b == b'\n') {
            let raw: Vec<u8> = buf.drain(..=pos).collect();
            let line = String::from_utf8_lossy(&raw);
            let Some(data) = line.trim().strip_prefix("data:").map(str::trim) else {
                continue;
            };
            if data == "[DONE]" {
                return Ok(());
            }
            let text = serde_json::from_str::<Value>(data)
                .ok()
                .and_then(|v| v["choices"][0]["delta"]["content"].as_str().map(String::from));
            if let Some(text) = text.filter(|t| !t.is_empty()) {
                let _ = tx.send(Ev::Chunk(text));
            }
        }
    }
    Ok(())
}

const NON_CHAT: &[&str] = &["embed", "whisper", "tts", "dall-e", "moderation", "rerank", "guard", "transcri"];

fn is_chat_model(id: &str) -> bool {
    let id = id.to_lowercase();
    !NON_CHAT.iter().any(|k| id.contains(k))
}

async fn fetch_models(client: &Client, t: &Target) -> Result<Vec<String>> {
    let resp = authed(client.get(format!("{}/models", t.base_url)), t)
        .timeout(Duration::from_secs(20))
        .send()
        .await
        .context("gagal konek")?;
    let json: Value = ensure_ok(resp).await?.json().await?;
    let list = json["data"]
        .as_array()
        .or_else(|| json["models"].as_array())
        .context("format daftar model nggak dikenal")?;

    let mut ids: Vec<String> = list
        .iter()
        .filter_map(|m| m["id"].as_str().or_else(|| m["name"].as_str()))
        .map(|id| id.trim_start_matches("models/").to_string())
        .filter(|id| is_chat_model(id))
        .collect();
    ids.sort_unstable();
    ids.dedup();
    Ok(ids)
}
