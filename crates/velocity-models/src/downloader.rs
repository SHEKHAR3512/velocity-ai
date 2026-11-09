use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tokio::sync::mpsc::Sender;
use velocity_core::{VelocityError, VelocityResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub file_name: String,
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
    pub percent: f32,
    pub speed_mbps: f64,
    pub is_finished: bool,
}

pub struct ResumableDownloader {
    client: reqwest::Client,
}

impl ResumableDownloader {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .tcp_nodelay(true)
                .build()
                .unwrap_or_default(),
        }
    }

    /// Queries the remote file size via HTTP HEAD or range GET without downloading the full body.
    pub async fn query_file_size(&self, url: &str) -> VelocityResult<Option<u64>> {
        let resp = self
            .client
            .head(url)
            .send()
            .await
            .map_err(|e| VelocityError::Model(format!("Failed to query remote file metadata: {}", e)))?;

        if resp.status().is_success() {
            if let Some(cl) = resp.headers().get(reqwest::header::CONTENT_LENGTH) {
                if let Ok(len_str) = cl.to_str() {
                    if let Ok(bytes) = len_str.parse::<u64>() {
                        return Ok(Some(bytes));
                    }
                }
            }
        }

        Ok(None)
    }

    /// Downloads a file with resumption support using the HTTP `Range` header.
    /// Writes to `<target_path>.part` during download and atomically renames on completion.
    pub async fn download(
        &self,
        url: &str,
        target_path: &Path,
        progress_tx: Option<Sender<DownloadProgress>>,
    ) -> VelocityResult<PathBuf> {
        let file_name = target_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("model.gguf")
            .to_string();

        let part_path = target_path.with_extension("gguf.part");

        // Determine existing bytes on disk
        let existing_bytes = if part_path.exists() {
            std::fs::metadata(&part_path).map(|m| m.len()).unwrap_or(0)
        } else {
            0
        };

        let mut req = self.client.get(url);
        if existing_bytes > 0 {
            tracing::info!(
                "Resuming download of {} from byte offset {}",
                file_name,
                existing_bytes
            );
            req = req.header(reqwest::header::RANGE, format!("bytes={}-", existing_bytes));
        }

        let resp = req
            .send()
            .await
            .map_err(|e| VelocityError::Model(format!("HTTP request failed: {}", e)))?;

        let status = resp.status();
        let (is_resume, total_bytes) = if status == reqwest::StatusCode::PARTIAL_CONTENT {
            // Server honored Range request
            let content_len = resp
                .headers()
                .get(reqwest::header::CONTENT_LENGTH)
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok());
            (true, content_len.map(|cl| cl + existing_bytes))
        } else if status == reqwest::StatusCode::OK {
            // Full download
            let content_len = resp
                .headers()
                .get(reqwest::header::CONTENT_LENGTH)
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok());
            (false, content_len)
        } else if status == reqwest::StatusCode::RANGE_NOT_SATISFIABLE {
            // File might already be completely downloaded
            tracing::info!("Range not satisfiable; checking if file is already complete");
            (false, Some(existing_bytes))
        } else {
            return Err(VelocityError::Model(format!(
                "Server returned unexpected HTTP status: {}",
                status
            )));
        };

        let mut file = if is_resume {
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(&part_path)
                .map_err(|e| VelocityError::Model(format!("Failed to open part file for append: {}", e)))?
        } else {
            OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&part_path)
                .map_err(|e| VelocityError::Model(format!("Failed to create part file: {}", e)))?
        };

        let mut downloaded = if is_resume { existing_bytes } else { 0 };
        let mut stream = resp.bytes_stream();
        let start_time = Instant::now();
        let mut last_progress_time = Instant::now();

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result
                .map_err(|e| VelocityError::Model(format!("Error reading stream chunk: {}", e)))?;
            file.write_all(&chunk)
                .map_err(|e| VelocityError::Model(format!("Error writing to disk: {}", e)))?;
            downloaded += chunk.len() as u64;

            // Report progress at most every 200ms
            if last_progress_time.elapsed().as_millis() >= 200 {
                last_progress_time = Instant::now();
                let elapsed_sec = start_time.elapsed().as_secs_f64();
                let speed_mbps = if elapsed_sec > 0.0 {
                    let newly_downloaded = downloaded.saturating_sub(if is_resume { existing_bytes } else { 0 });
                    (newly_downloaded as f64) / (1024.0 * 1024.0 * elapsed_sec)
                } else {
                    0.0
                };

                let percent = if let Some(tot) = total_bytes {
                    if tot > 0 {
                        ((downloaded as f64 / tot as f64) * 100.0) as f32
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };

                if let Some(ref tx) = progress_tx {
                    let _ = tx
                        .send(DownloadProgress {
                            file_name: file_name.clone(),
                            downloaded_bytes: downloaded,
                            total_bytes,
                            percent,
                            speed_mbps,
                            is_finished: false,
                        })
                        .await;
                }
            }
        }

        file.flush()
            .map_err(|e| VelocityError::Model(format!("Failed flushing part file: {}", e)))?;
        drop(file);

        // Atomic rename from .part to final target
        if target_path.exists() {
            let _ = std::fs::remove_file(target_path);
        }
        std::fs::rename(&part_path, target_path)
            .map_err(|e| VelocityError::Model(format!("Failed to rename .part file to final: {}", e)))?;

        // Send final progress message
        if let Some(ref tx) = progress_tx {
            let _ = tx
                .send(DownloadProgress {
                    file_name: file_name.clone(),
                    downloaded_bytes: downloaded,
                    total_bytes: Some(downloaded),
                    percent: 100.0,
                    speed_mbps: 0.0,
                    is_finished: true,
                })
                .await;
        }

        tracing::info!(
            "Download completed successfully: {} ({} bytes)",
            target_path.display(),
            downloaded
        );
        Ok(target_path.to_path_buf())
    }
}

impl Default for ResumableDownloader {
    fn default() -> Self {
        Self::new()
    }
}

