#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod state;
mod theme;
mod views;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;
use std::thread;

use eframe::egui::Vec2;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};
use velocity_core::{init_tracing, InferenceMetrics};
use velocity_engine::{InferenceBackend, LlamaEngine};
use velocity_hardware::detect_hardware;
use velocity_models::ModelManager;
use velocity_storage::Storage;

use app::VelocityApp;
use state::{EngineCommand, EngineEvent};
use theme::Theme;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    // 1. Detect Hardware Profile
    let _hw = detect_hardware().ok();

    // 2. Initialize Models Directory
    let models_dir = {
        let mut candidates = vec![
            PathBuf::from("models"),
            PathBuf::from("../../models"),
            PathBuf::from("../../../models"),
        ];
        if let Ok(exe) = std::env::current_exe() {
            if let Some(parent) = exe.parent() {
                candidates.push(parent.join("models"));
                if let Some(gp) = parent.parent() {
                    candidates.push(gp.join("models"));
                    if let Some(ggp) = gp.parent() {
                        candidates.push(ggp.join("models"));
                    }
                }
            }
        }
        candidates
            .into_iter()
            .find(|p| p.exists())
            .unwrap_or_else(|| PathBuf::from("models"))
    };
    let model_mgr = ModelManager::new(&models_dir);

    // 3. Initialize SQLite Storage
    let storage_path = PathBuf::from("velocity_chat.db");
    let storage = Storage::open(storage_path).ok();

    // 4. Background Tokio Engine Worker
    let (cmd_tx, cmd_rx) = unbounded_channel::<EngineCommand>();
    let (event_tx, event_rx) = channel::<EngineEvent>();

    let worker_models_dir = models_dir.clone();
    thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to build Tokio runtime for inference worker");

        rt.block_on(async move {
            run_engine_worker(cmd_rx, event_tx, worker_models_dir).await;
        });
    });

    // 5. Native Window Options
    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("VELOCITY AI — High Performance Desktop Chatbot")
            .with_inner_size(Vec2::new(1150.0, 750.0))
            .with_min_inner_size(Vec2::new(850.0, 580.0))
            .with_active(true),
        ..Default::default()
    };

    // 6. Launch eframe GUI
    eframe::run_native(
        "VELOCITY AI",
        native_options,
        Box::new(|cc| {
            Theme::apply(&cc.egui_ctx);
            Ok(Box::new(VelocityApp::new(cmd_tx, event_rx, storage, model_mgr)))
        }),
    )?;

    Ok(())
}

async fn run_engine_worker(
    mut cmd_rx: UnboundedReceiver<EngineCommand>,
    event_tx: Sender<EngineEvent>,
    models_dir: PathBuf,
) {
    let mut engine = LlamaEngine::new();
    let abort_flag = Arc::new(AtomicBool::new(false));

    while let Some(cmd) = cmd_rx.recv().await {
        match cmd {
            EngineCommand::StartEngine { config } => {
                tracing::info!("Worker: Starting engine for model: {}", config.model_path);
                match engine.start(&config).await {
                    Ok(()) => {
                        let _ = event_tx.send(EngineEvent::EngineReady {
                            backend: config.backend,
                            port: engine.port(),
                        });
                    }
                    Err(e) => {
                        let _ = event_tx.send(EngineEvent::EngineError(e.to_string()));
                    }
                }
            }
            EngineCommand::StopEngine => {
                let _ = engine.stop().await;
                let _ = event_tx.send(EngineEvent::EngineStopped);
            }
            EngineCommand::StreamChat { messages, config } => {
                abort_flag.store(false, Ordering::SeqCst);
                let flag = Arc::clone(&abort_flag);
                let tx = event_tx.clone();

                if !engine.is_running() {
                    if let Err(e) = engine.start(&config).await {
                        let _ = tx.send(EngineEvent::StreamError(format!("Failed to start engine: {}", e)));
                        continue;
                    }
                }

                match engine.stream_chat(&messages, &config).await {
                    Ok(mut rx) => {
                        tokio::spawn(async move {
                            let mut metrics_captured = None;

                            while let Some(chunk) = rx.recv().await {
                                if flag.load(Ordering::SeqCst) {
                                    // User aborted
                                    break;
                                }

                                if !chunk.delta.is_empty() {
                                    let _ = tx.send(EngineEvent::StreamToken(chunk.delta));
                                }

                                if chunk.is_finished {
                                    metrics_captured = chunk.metrics;
                                    break;
                                }
                            }

                            let default_m = metrics_captured.unwrap_or(InferenceMetrics {
                                prompt_tokens: 0,
                                completion_tokens: 0,
                                time_to_first_token_ms: 0.0,
                                tokens_per_second: 0.0,
                                total_duration_ms: 0.0,
                            });
                            let _ = tx.send(EngineEvent::StreamFinished(default_m));
                        });
                    }
                    Err(e) => {
                        let _ = tx.send(EngineEvent::StreamError(e.to_string()));
                    }
                }
            }
            EngineCommand::AbortInference => {
                abort_flag.store(true, Ordering::SeqCst);
            }
            EngineCommand::StartDownload { url, filename } => {
                let tx = event_tx.clone();
                let mgr = ModelManager::new(&models_dir);

                tokio::spawn(async move {
                    let (prog_tx, mut prog_rx) = tokio::sync::mpsc::channel(32);

                    let tx_for_prog = tx.clone();
                    tokio::spawn(async move {
                        while let Some(prog) = prog_rx.recv().await {
                            let _ = tx_for_prog.send(EngineEvent::DownloadUpdate(prog));
                        }
                    });

                    match mgr.download_model(&url, &filename, Some(prog_tx)).await {
                        Ok(meta) => {
                            let _ = tx.send(EngineEvent::DownloadDone(meta));
                        }
                        Err(e) => {
                            let _ = tx.send(EngineEvent::DownloadError(e.to_string()));
                        }
                    }
                });
            }
        }
    }
}
