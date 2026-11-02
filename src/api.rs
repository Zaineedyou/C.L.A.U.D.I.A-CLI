use std::{env, time::Duration};

use anyhow::{bail, Context, Result};
use futures_util::StreamExt;
use reqwest::{Client, RequestBuilder, Response};
use serde_json::{json, Value};
use tokio::task::JoinHandle;

use crate::{
    chat::{Message, Role},
    event::{Ev, Tx},
    tools,
};

/// Semua yang dibutuhkan untuk bicara ke satu provider.
#[derive(Clone)]
pub struct Target {
    pub id: &'static str,
    pub base_url: String,
    pub key: Option<String>,
    pub model: String,
    /// Tawarkan alat baca file ke model pada permintaan ini.
    pub tools: bool,
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

/// Batas putaran model -> alat -> model dalam satu balasan.
const MAX_ROUNDS: usize = 8;

#[derive(Default)]
struct ToolCall {
    id: String,
    name: String,
    args: String,
}

async fn stream_chat(client: &Client, t: &Target, history: &[Message], tx: &Tx) -> Result<()> {
    if t.id == "hfspace" {
        return stream_space(client, t, history, tx).await;
    }
    let mut messages: Vec<Value> = history.iter().map(Message::to_json).collect();
    let mut use_tools = t.tools;
    let url = format!("{}/chat/completions", t.base_url);

    for _ in 0..MAX_ROUNDS {
        let mut body = json!({ "model": t.model, "messages": messages, "stream": true });
        if use_tools {
            body["tools"] = tools::definitions();
        }
        let resp = authed(client.post(&url), t).json(&body).send().await.context("gagal konek")?;
        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            let lower = text.to_lowercase();
            let code = status.as_u16();
            if use_tools && matches!(code, 400 | 404 | 422) && (lower.contains("tool") || lower.contains("function")) {
                // Provider atau model ini tidak mengerti alat: lanjut sebagai chat biasa.
                use_tools = false;
                let _ = tx.send(Ev::Tool("model ini nggak mendukung alat file, lanjut tanpa baca file".to_string()));
                continue;
            }
            bail!("{status}: {}", text.chars().take(160).collect::<String>());
        }

        let (text, calls) = read_stream(resp, tx).await?;
        if calls.is_empty() {
            return Ok(());
        }

        let calls_json: Vec<Value> = calls
            .iter()
            .map(|c| json!({ "id": c.id, "type": "function", "function": { "name": c.name, "arguments": c.args } }))
            .collect();
        let content = if text.is_empty() { Value::Null } else { Value::String(text) };
        messages.push(json!({ "role": "assistant", "content": content, "tool_calls": calls_json }));

        for call in calls {
            let args: Value = serde_json::from_str(&call.args).unwrap_or(Value::Null);
            let _ = tx.send(Ev::Tool(tools::label(&call.name, &args)));
            let name = call.name.clone();
            let result = tokio::task::spawn_blocking(move || tools::run(&name, &args))
                .await
                .unwrap_or_else(|e| format!("Error: {e}"));
            messages.push(json!({ "role": "tool", "tool_call_id": call.id, "content": result }));
        }
    }
    let _ = tx.send(Ev::Chunk("\n\n(berhenti: terlalu banyak langkah baca file)".to_string()));
    Ok(())
}

/// Baca satu balasan streaming: teks dikirim ke layar, panggilan alat dikumpulkan buat dijalankan.
async fn read_stream(resp: Response, tx: &Tx) -> Result<(String, Vec<ToolCall>)> {
    let mut stream = resp.bytes_stream();
    let mut buf: Vec<u8> = Vec::new();
    let mut text = String::new();
    let mut calls: Vec<ToolCall> = Vec::new();
    while let Some(chunk) = stream.next().await {
        buf.extend_from_slice(&chunk?);
        while let Some(pos) = buf.iter().position(|&b| b == b'\n') {
            let raw: Vec<u8> = buf.drain(..=pos).collect();
            let line = String::from_utf8_lossy(&raw);
            let Some(data) = line.trim().strip_prefix("data:").map(str::trim) else {
                continue;
            };
            if data == "[DONE]" {
                return Ok(finish_calls(text, calls));
            }
            let Ok(value) = serde_json::from_str::<Value>(data) else {
                continue;
            };
            let delta = &value["choices"][0]["delta"];
            if let Some(piece) = delta["content"].as_str().filter(|p| !p.is_empty()) {
                text.push_str(piece);
                let _ = tx.send(Ev::Chunk(piece.to_string()));
            }
            if let Some(items) = delta["tool_calls"].as_array() {
                for item in items {
                    let has_id = item["id"].as_str().map_or(false, |s| !s.is_empty());
                    let index = match item["index"].as_u64() {
                        Some(i) => i as usize,
                        None if has_id => calls.len(),
                        None => calls.len().saturating_sub(1),
                    };
                    while calls.len() <= index {
                        calls.push(ToolCall::default());
                    }
                    let call = &mut calls[index];
                    if let Some(id) = item["id"].as_str().filter(|s| !s.is_empty()) {
                        call.id = id.to_string();
                    }
                    if let Some(name) = item["function"]["name"].as_str().filter(|s| !s.is_empty()) {
                        call.name = name.to_string();
                    }
                    if let Some(args) = item["function"]["arguments"].as_str() {
                        call.args.push_str(args);
                    }
                }
            }
        }
    }
    Ok(finish_calls(text, calls))
}

fn finish_calls(text: String, mut calls: Vec<ToolCall>) -> (String, Vec<ToolCall>) {
    calls.retain(|c| !c.name.is_empty());
    for (i, call) in calls.iter_mut().enumerate() {
        if call.id.is_empty() {
            call.id = format!("call_{i}");
        }
    }
    (text, calls)
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
    let turns: Vec<&Message> = history.iter().filter(|m| m.role != Role::System && !m.full().trim().is_empty()).collect();
    match turns.as_slice() {
        [] => String::new(),
        [only] => only.full(),
        _ => {
            let mut out = String::from("Riwayat percakapan sejauh ini:\n\n");
            for m in &turns[..turns.len() - 1] {
                let who = if m.role == Role::User { "User" } else { "Asisten" };
                out.push_str(&format!("{who}: {}\n\n", m.full().trim()));
            }
            out.push_str(&format!("Pesan terbaru dari user (jawab yang ini):\n{}", turns[turns.len() - 1].full().trim()));
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
