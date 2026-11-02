use std::{env, time::Duration};

use anyhow::{bail, Context, Result};
use futures_util::StreamExt;
use reqwest::{Client, RequestBuilder, Response};
use serde_json::{json, Value};
use tokio::task::JoinHandle;

use crate::{
    chat::{Message, Role},
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
    if t.id == "hfspace" {
        return stream_space(client, t, history, tx).await;
    }
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

async fn fetch_models(client: &Client, t: &Target) -> Result<Vec<(String, String)>> {
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

    // (id, nama tampilan): nama ikut dicari di picker, jadi "Uncensored" menemukan id yang slug-nya beda.
    let mut models: Vec<(String, String)> = list
        .iter()
        .filter_map(|m| {
            let id = m["id"].as_str().or_else(|| m["name"].as_str())?.trim_start_matches("models/").to_string();
            if !is_chat_model(&id) {
                return None;
            }
            let name = m["name"].as_str().filter(|n| *n != id).unwrap_or("").to_string();
            Some((id, name))
        })
        .collect();
    models.sort_unstable();
    models.dedup_by(|a, b| a.0 == b.0);
    Ok(models)
}

// ── Hugging Face Space (Gradio) ───────────────────────────

/// "owner/nama" jadi https://owner-nama.hf.space; URL lengkap dipakai apa adanya.
fn space_base(space: &str) -> String {
    let s = space.trim().trim_end_matches('/');
    if s.starts_with("http") {
        return s.to_string();
    }
    let slug: String = s.to_lowercase().chars().map(|c| if matches!(c, '/' | '_' | '.') { '-' } else { c }).collect();
    format!("https://{slug}.hf.space")
}

/// Space chat lewat API stateless, jadi riwayat dilipat jadi satu pesan kalau sudah lebih dari satu giliran.
fn space_prompt(history: &[Message]) -> String {
    let turns: Vec<&Message> = history.iter().filter(|m| m.role != Role::System && !m.content.trim().is_empty()).collect();
    match turns.as_slice() {
        [] => String::new(),
        [only] => only.content.clone(),
        _ => {
            let mut out = String::from("Riwayat percakapan sejauh ini:\n\n");
            for m in &turns[..turns.len() - 1] {
                let who = if m.role == Role::User { "User" } else { "Asisten" };
                out.push_str(&format!("{who}: {}\n\n", m.content.trim()));
            }
            out.push_str(&format!("Pesan terbaru dari user (jawab yang ini):\n{}", turns[turns.len() - 1].content.trim()));
            out
        }
    }
}

/// Teks dari output Gradio: string, `{content}`/`{value}`, atau elemen terakhir sebuah list.
fn gradio_text(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Array(items) => items.last().and_then(gradio_text),
        Value::Object(map) => map.get("content").or_else(|| map.get("value")).and_then(gradio_text),
        _ => None,
    }
}

/// Skema API Space (`/info`); dicoba dengan dan tanpa awalan `gradio_api` (Gradio 5 vs 4).
async fn space_info(client: &Client, t: &Target, base: &str) -> Option<(&'static str, Value)> {
    for prefix in ["gradio_api", ""] {
        let url = if prefix.is_empty() { format!("{base}/info") } else { format!("{base}/{prefix}/info") };
        let Ok(resp) = authed(client.get(&url), t).timeout(Duration::from_secs(20)).send().await else {
            continue;
        };
        if !resp.status().is_success() {
            continue;
        }
        if let Ok(info) = resp.json::<Value>().await {
            return Some((prefix, info));
        }
    }
    None
}

async fn stream_space(client: &Client, t: &Target, history: &[Message], tx: &Tx) -> Result<()> {
    if t.model.trim().is_empty() {
        bail!("belum ada Space: isi id-nya lewat /model owner/nama (atau env HF_SPACE)");
    }
    let base = space_base(&t.model);
    let mut api = env::var("HF_SPACE_API").ok().map(|a| a.trim().trim_start_matches('/').to_string()).filter(|a| !a.is_empty());

    // Parameter tambahan (slider, system message, dst.) diisi nilai bawaan dari skema API Space.
    let mut data: Vec<Value> = vec![Value::String(space_prompt(history))];
    let mut prefix = "gradio_api";
    if let Some((found, info)) = space_info(client, t, &base).await {
        prefix = found;
        if let Some(endpoints) = info["named_endpoints"].as_object() {
            let wanted = format!("/{}", api.as_deref().unwrap_or("chat"));
            let key = if endpoints.contains_key(&wanted) { Some(wanted) } else { endpoints.keys().next().cloned() };
            if let Some(key) = key {
                api = Some(key.trim_start_matches('/').to_string());
                if let Some(params) = endpoints[&key]["parameters"].as_array() {
                    for param in params.iter().skip(1) {
                        let has_default = param["parameter_has_default"].as_bool().unwrap_or(false);
                        data.push(if has_default { param["parameter_default"].clone() } else { Value::Null });
                    }
                }
            }
        }
    }
    // HF_SPACE_ARGS (JSON list) menggantikan parameter tambahan hasil tebakan di atas.
    if let Some(Value::Array(extra)) = env::var("HF_SPACE_ARGS").ok().and_then(|raw| serde_json::from_str::<Value>(&raw).ok()) {
        data.truncate(1);
        data.extend(extra);
    }
    let api = api.unwrap_or_else(|| "chat".to_string());

    let call = if prefix.is_empty() { format!("{base}/call/{api}") } else { format!("{base}/{prefix}/call/{api}") };
    let resp = authed(client.post(&call), t).json(&json!({ "data": data })).send().await.context("gagal konek ke Space")?;
    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        bail!("Space atau endpoint /{api} nggak ketemu (cek id Space, atau set HF_SPACE_API)");
    }
    let started: Value = ensure_ok(resp).await?.json().await?;
    let event_id = started["event_id"].as_str().context("Space nggak ngasih event_id")?.to_string();

    let resp = authed(client.get(format!("{call}/{event_id}")), t).send().await.context("gagal konek ke Space")?;
    let mut stream = ensure_ok(resp).await?.bytes_stream();
    let mut buf: Vec<u8> = Vec::new();
    let mut event = String::new();
    let mut shown = String::new();
    while let Some(chunk) = stream.next().await {
        buf.extend_from_slice(&chunk?);
        while let Some(pos) = buf.iter().position(|&b| b == b'\n') {
            let raw: Vec<u8> = buf.drain(..=pos).collect();
            let line = String::from_utf8_lossy(&raw);
            let line = line.trim();
            if let Some(name) = line.strip_prefix("event:") {
                event = name.trim().to_string();
                continue;
            }
            let Some(payload) = line.strip_prefix("data:").map(str::trim) else {
                continue;
            };
            match event.as_str() {
                "generating" | "complete" => {
                    let text = serde_json::from_str::<Value>(payload).ok().as_ref().and_then(gradio_text);
                    if let Some(text) = text {
                        // Gradio mengirim teks kumulatif; yang dikirim ke layar cuma bagian barunya.
                        let rest = text.strip_prefix(shown.as_str()).map(str::to_string);
                        match rest {
                            Some(rest) => {
                                if !rest.is_empty() {
                                    let _ = tx.send(Ev::Chunk(rest));
                                }
                                shown = text;
                            }
                            None => {
                                let _ = tx.send(Ev::Chunk(text.clone()));
                                shown.push_str(&text);
                            }
                        }
                    }
                    if event == "complete" {
                        return Ok(());
                    }
                }
                "error" => bail!("Space error: {}", payload.chars().take(160).collect::<String>()),
                _ => {}
            }
        }
    }
    Ok(())
}
