use futures_util::StreamExt;
use serde_json::json;
use std::time::Instant;
use tokio::sync::mpsc::{channel, Receiver};
use velocity_core::{InferenceConfig, InferenceMetrics, Message, Role, StreamChunk, VelocityResult};

pub struct LlamaHttpClient {
    client: reqwest::Client,
}

impl LlamaHttpClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .tcp_nodelay(true)
                .build()
                .unwrap_or_default(),
        }
    }

    pub async fn stream_chat(
        &self,
        port: u16,
        messages: &[Message],
        config: &InferenceConfig,
    ) -> VelocityResult<Receiver<StreamChunk>> {
        let (tx, rx) = channel::<StreamChunk>(256);
        let url = format!("http://127.0.0.1:{}/v1/chat/completions", port);

        let formatted_msgs: Vec<serde_json::Value> = messages
            .iter()
            .map(|m| {
                let role = match m.role {
                    Role::System => "system",
                    Role::User => "user",
                    Role::Assistant => "assistant",
                };
                json!({
                    "role": role,
                    "content": m.content,
                })
            })
            .collect();

        let payload = json!({
            "model": "local",
            "messages": formatted_msgs,
            "stream": true,
            "temperature": config.temperature,
            "top_p": config.top_p,
            "max_tokens": config.max_tokens,
        });

        let client = self.client.clone();

        tokio::spawn(async move {
            let start_time = Instant::now();
            let mut first_token_time = None;
            let mut token_count = 0usize;

            let response = match client.post(&url).json(&payload).send().await {
                Ok(r) => r,
                Err(e) => {
                    let _ = tx
                        .send(StreamChunk {
                            delta: format!("\n[Error connecting to inference runtime: {}]", e),
                            is_finished: true,
                            metrics: None,
                        })
                        .await;
                    return;
                }
            };

            if !response.status().is_success() {
                let status = response.status();
                let err_text = response.text().await.unwrap_or_default();
                let _ = tx
                    .send(StreamChunk {
                        delta: format!("\n[Inference error {}: {}]", status, err_text),
                        is_finished: true,
                        metrics: None,
                    })
                    .await;
                return;
            }

            let mut stream = response.bytes_stream();
            let mut buffer = String::new();

            while let Some(chunk_result) = stream.next().await {
                let bytes = match chunk_result {
                    Ok(b) => b,
                    Err(e) => {
                        let _ = tx
                            .send(StreamChunk {
                                delta: format!("\n[Stream read error: {}]", e),
                                is_finished: true,
                                metrics: None,
                            })
                            .await;
                        break;
                    }
                };

                buffer.push_str(&String::from_utf8_lossy(&bytes));

                while let Some(newline_pos) = buffer.find('\n') {
                    let line = buffer[..newline_pos].trim().to_string();
                    buffer = buffer[newline_pos + 1..].to_string();

                    if line.is_empty() || line.starts_with(':') {
                        continue;
                    }

                    if let Some(data_str) = line.strip_prefix("data: ") {
                        if data_str.trim() == "[DONE]" {
                            let end_time = Instant::now();
                            let total_duration = end_time.duration_since(start_time).as_secs_f64() * 1000.0;
                            let ttft_ms = first_token_time
                                .map(|t: Instant| t.duration_since(start_time).as_secs_f64() * 1000.0)
                                .unwrap_or(total_duration);

                            let tps = if let Some(ft) = first_token_time {
                                let gen_sec = end_time.duration_since(ft).as_secs_f64();
                                if gen_sec > 0.005 && token_count > 1 {
                                    (token_count.saturating_sub(1) as f64) / gen_sec
                                } else {
                                    (token_count as f64) / (total_duration / 1000.0).max(0.001)
                                }
                            } else {
                                0.0
                            };

                            let metrics = InferenceMetrics {
                                prompt_tokens: 0,
                                completion_tokens: token_count,
                                time_to_first_token_ms: ttft_ms,
                                tokens_per_second: tps,
                                total_duration_ms: total_duration,
                            };

                            let _ = tx
                                .send(StreamChunk {
                                    delta: String::new(),
                                    is_finished: true,
                                    metrics: Some(metrics),
                                })
                                .await;
                            return;
                        }

                        if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(data_str) {
                            if let Some(delta) = json_val["choices"][0]["delta"]["content"].as_str() {
                                if !delta.is_empty() {
                                    if first_token_time.is_none() {
                                        first_token_time = Some(Instant::now());
                                    }
                                    token_count += 1;

                                    if tx
                                        .send(StreamChunk {
                                            delta: delta.to_string(),
                                            is_finished: false,
                                            metrics: None,
                                        })
                                        .await
                                        .is_err()
                                    {
                                        // Receiver dropped, stop streaming
                                        return;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        Ok(rx)
    }
}

