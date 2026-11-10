use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::mpsc::Receiver;
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

use eframe::egui::{self, Align, CentralPanel, Color32, Frame, Layout, Margin, RichText, Rounding, ScrollArea, SidePanel, Stroke, TopBottomPanel, Vec2};
use velocity_core::{Conversation, InferenceConfig, Message, Role};
use velocity_models::ModelManager;
use velocity_storage::Storage;

use crate::state::{ActiveTab, AppState, EngineCommand, EngineEvent};
use crate::theme::Theme;
use crate::views::{
    benchmarks::render_benchmarks_view,
    chat::{render_chat_messages, render_claude_input_capsule},
    models::render_models_view,
    settings::render_settings_view,
};

pub struct VelocityApp {
    pub state: AppState,
    pub cmd_tx: UnboundedSender<EngineCommand>,
    pub event_rx: Receiver<EngineEvent>,
    pub storage: Option<Storage>,
    pub model_mgr: ModelManager,
}

impl VelocityApp {
    pub fn new(
        cmd_tx: UnboundedSender<EngineCommand>,
        event_rx: Receiver<EngineEvent>,
        storage: Option<Storage>,
        model_mgr: ModelManager,
    ) -> Self {
        let mut state = AppState::default();

        // Load local models
        if let Ok(models) = model_mgr.list_models() {
            state.local_models = models;
        }

        // Load conversations from SQLite
        if let Some(ref store) = storage {
            if let Ok(convs) = store.list_conversations() {
                state.conversations = convs;
            }
        }

        // Start with a new conversation if none exist
        if state.conversations.is_empty() {
            let new_conv = Conversation::new("New conversation", "qwen2.5-0.5b");
            if let Some(ref store) = storage {
                let _ = store.save_conversation(&new_conv);
            }
            state.current_conv_id = Some(new_conv.id);
            state.conversations.push(new_conv);
        } else {
            state.current_conv_id = Some(state.conversations[0].id);
            if let Some(ref store) = storage {
                if let Ok(msgs) = store.list_messages(state.conversations[0].id) {
                    state.messages = msgs;
                }
            }
        }

        // Verify active model path
        let candidate_model = Path::new(&state.selected_model);
        if !candidate_model.exists() {
            if let Some(first_local) = state.local_models.first() {
                state.selected_model = first_local.file_path.to_string_lossy().to_string();
            }
        }

        // Use detected hardware recommendation
        if let Ok(hw) = velocity_hardware::detect_hardware() {
            state.active_backend = hw.recommended_backend;
            state.threads = hw.recommended_threads;
            state.gpu_layers = hw.recommended_gpu_layers;
            state.hardware = Some(hw);
        }

        // Start engine in background
        let start_cfg = InferenceConfig {
            model_path: state.selected_model.clone(),
            backend: state.active_backend,
            threads: state.threads,
            gpu_layers: state.gpu_layers,
            context_size: state.context_size,
            batch_size: state.batch_size,
            temperature: state.temperature,
            top_p: state.top_p,
            max_tokens: 2048,
        };
        let _ = cmd_tx.send(EngineCommand::StartEngine { config: start_cfg });

        Self {
            state,
            cmd_tx,
            event_rx,
            storage,
            model_mgr,
        }
    }

    fn poll_events(&mut self, ctx: &egui::Context) {
        while let Ok(event) = self.event_rx.try_recv() {
            match event {
                EngineEvent::EngineReady { backend, port } => {
                    self.state.is_engine_ready = true;
                    self.state.active_backend = backend;
                    self.state.active_port = Some(port);
                    self.state.status_message = Some((format!("Inference engine online on port {}", port), false));
                    ctx.request_repaint();
                }
                EngineEvent::EngineStopped => {
                    self.state.is_engine_ready = false;
                    self.state.active_port = None;
                    ctx.request_repaint();
                }
                EngineEvent::EngineError(err) => {
                    self.state.is_engine_ready = false;
                    self.state.status_message = Some((format!("Engine error: {}", err), true));
                    ctx.request_repaint();
                }
                EngineEvent::StreamToken(delta) => {
                    self.state.streaming_text.push_str(&delta);
                    ctx.request_repaint();
                }
                EngineEvent::StreamFinished(metrics) => {
                    self.state.is_generating = false;
                    self.state.latest_metrics = Some(metrics);

                    let conv_id = self.state.current_conv_id.unwrap_or_else(Uuid::nil);
                    let final_text = self.state.streaming_text.clone();
                    self.state.streaming_text.clear();

                    let assistant_msg = Message::new(conv_id, Role::Assistant, final_text);
                    if let Some(ref store) = self.storage {
                        let _ = store.save_message(&assistant_msg);
                    }
                    self.state.messages.push(assistant_msg);
                    ctx.request_repaint();
                }
                EngineEvent::StreamError(err) => {
                    self.state.is_generating = false;
                    self.state.status_message = Some((format!("Streaming error: {}", err), true));
                    ctx.request_repaint();
                }
                EngineEvent::DownloadUpdate(progress) => {
                    self.state.active_download = Some(progress);
                    ctx.request_repaint();
                }
                EngineEvent::DownloadDone(model_meta) => {
                    self.state.active_download = None;
                    self.state.local_models.push(model_meta);
                    self.state.status_message = Some(("Download completed successfully!".to_string(), false));
                    ctx.request_repaint();
                }
                EngineEvent::DownloadError(err) => {
                    self.state.active_download = None;
                    self.state.status_message = Some((format!("Download failed: {}", err), true));
                    ctx.request_repaint();
                }
            }
        }
    }

    fn start_new_chat(&mut self) {
        let count = self.state.conversations.len() + 1;
        let new_conv = Conversation::new(format!("Conversation {}", count), "qwen2.5-0.5b");
        if let Some(ref store) = self.storage {
            let _ = store.save_conversation(&new_conv);
        }
        self.state.current_conv_id = Some(new_conv.id);
        self.state.messages.clear();
        self.state.conversations.insert(0, new_conv);
        self.state.active_tab = ActiveTab::Chat;
    }

    fn select_conversation(&mut self, conv_id: Uuid) {
        self.state.current_conv_id = Some(conv_id);
        if let Some(ref store) = self.storage {
            if let Ok(msgs) = store.list_messages(conv_id) {
                self.state.messages = msgs;
            }
        }
        self.state.active_tab = ActiveTab::Chat;
    }

    fn send_user_message(&mut self, text: String) {
        if text.trim().is_empty() || self.state.is_generating {
            return;
        }

        let conv_id = self.state.current_conv_id.unwrap_or_else(Uuid::nil);
        let user_msg = Message::new(conv_id, Role::User, text);

        if let Some(ref store) = self.storage {
            let _ = store.save_message(&user_msg);
        }
        self.state.messages.push(user_msg);

        // Build messages payload with system prompt
        let mut full_messages = vec![Message::new(
            conv_id,
            Role::System,
            self.state.system_prompt.clone(),
        )];
        full_messages.extend(self.state.messages.clone());

        let cfg = InferenceConfig {
            model_path: self.state.selected_model.clone(),
            backend: self.state.active_backend,
            threads: self.state.threads,
            gpu_layers: self.state.gpu_layers,
            context_size: self.state.context_size,
            batch_size: self.state.batch_size,
            temperature: self.state.temperature,
            top_p: self.state.top_p,
            max_tokens: 2048,
        };

        self.state.is_generating = true;
        self.state.streaming_text.clear();
        self.state.abort_flag.store(false, Ordering::SeqCst);

        let _ = self.cmd_tx.send(EngineCommand::StreamChat {
            messages: full_messages,
            config: cfg,
        });
    }

    fn stop_generation(&mut self) {
        self.state.abort_flag.store(true, Ordering::SeqCst);
        let _ = self.cmd_tx.send(EngineCommand::AbortInference);
    }
}

impl eframe::App for VelocityApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_events(ctx);

        let mut on_send = None;
        let mut on_stop = false;
        let mut on_select_model = None;
        let mut on_start_download = None;
        let mut on_switch_backend = None;
        let mut on_refresh_models = false;

        // 1. Bottom Claude Minimal Status Bar
        TopBottomPanel::bottom("status_bar")
            .frame(
                Frame::none()
                    .fill(Theme::BG_SIDEBAR)
                    .stroke(Stroke::new(1.0_f32, Theme::BORDER_SUBTLE))
                    .inner_margin(Margin::symmetric(18.0, 7.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if self.state.is_engine_ready {
                        ui.label(
                            RichText::new("Online")
                                .font(egui::FontId::proportional(11.5))
                                .strong()
                                .color(Theme::ACCENT_SAGE),
                        );
                    } else {
                        ui.label(
                            RichText::new("Initializing...")
                                .font(egui::FontId::proportional(11.5))
                                .color(Theme::ACCENT_AMBER),
                        );
                    }

                    ui.label(RichText::new("|").font(egui::FontId::proportional(11.0)).color(Theme::TEXT_MUTED));

                    let model_name = Path::new(&self.state.selected_model)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(&self.state.selected_model);
                    ui.label(
                        RichText::new(model_name)
                            .font(egui::FontId::proportional(11.5))
                            .color(Theme::TEXT_SECONDARY),
                    );

                    ui.label(RichText::new("|").font(egui::FontId::proportional(11.0)).color(Theme::TEXT_MUTED));

                    ui.label(
                        RichText::new(format!("{}", self.state.active_backend))
                            .font(egui::FontId::proportional(11.5))
                            .color(Theme::CLAUDE_TERRACOTTA),
                    );

                    if let Some(ref m) = self.state.latest_metrics {
                        ui.label(RichText::new("|").font(egui::FontId::proportional(11.0)).color(Theme::TEXT_MUTED));
                        ui.label(
                            RichText::new(format!(
                                "{:.1} tok/s | {:.0} ms TTFT",
                                m.tokens_per_second, m.time_to_first_token_ms
                            ))
                            .font(egui::FontId::monospace(11.0))
                            .color(Theme::ACCENT_SAGE),
                        );
                    }

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(
                            RichText::new("NVIDIA GTX 1650 | 4 Cores")
                                .font(egui::FontId::proportional(11.0))
                                .color(Theme::TEXT_MUTED),
                        );
                    });
                });
            });

        // 2. Left Claude Minimal Sidebar
        SidePanel::left("left_sidebar")
            .resizable(false)
            .default_width(230.0)
            .frame(
                Frame::none()
                    .fill(Theme::BG_SIDEBAR)
                    .stroke(Stroke::new(1.0_f32, Theme::BORDER_SUBTLE))
                    .inner_margin(Margin::same(14.0)),
            )
            .show(ctx, |ui| {
                // Brand Header
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Velocity AI")
                            .font(egui::FontId::proportional(17.0))
                            .strong()
                            .color(Theme::CLAUDE_TERRACOTTA),
                    );
                });
                ui.add_space(14.0);

                // "+ New chat" Button
                let new_chat_btn = egui::Button::new(
                    RichText::new("+ New chat")
                        .font(egui::FontId::proportional(13.5))
                        .color(Theme::TEXT_PARCHMENT),
                )
                .fill(Color32::from_rgb(38, 36, 33))
                .stroke(Stroke::new(1.0_f32, Theme::BORDER_WARM))
                .rounding(Rounding::same(8.0))
                .min_size(Vec2::new(ui.available_width(), 34.0));

                if ui.add(new_chat_btn).clicked() {
                    self.start_new_chat();
                }

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(10.0);

                // Navigation Tabs (Clean ASCII labels, no missing font boxes)
                let tabs = [
                    (ActiveTab::Chat, "Chat"),
                    (ActiveTab::Models, "Models"),
                    (ActiveTab::Hardware, "Telemetry"),
                    (ActiveTab::Settings, "Settings"),
                ];

                for (tab, label) in tabs {
                    let is_active = self.state.active_tab == tab;
                    let tab_btn = egui::Button::new(
                        RichText::new(label)
                            .font(egui::FontId::proportional(13.0))
                            .color(if is_active {
                                Theme::CLAUDE_TERRACOTTA
                            } else {
                                Theme::TEXT_SECONDARY
                            }),
                    )
                    .fill(if is_active {
                        Color32::from_rgb(38, 36, 33)
                    } else {
                        Color32::TRANSPARENT
                    })
                    .rounding(Rounding::same(6.0))
                    .min_size(Vec2::new(ui.available_width(), 28.0));

                    if ui.add(tab_btn).clicked() {
                        self.state.active_tab = tab;
                    }
                    ui.add_space(2.0);
                }

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(10.0);

                // Recent Conversations Section
                ui.label(
                    RichText::new("Recent")
                        .font(egui::FontId::proportional(11.5))
                        .strong()
                        .color(Theme::TEXT_MUTED),
                );
                ui.add_space(6.0);

                ScrollArea::vertical().show(ui, |ui| {
                    let mut conv_to_select = None;
                    for conv in &self.state.conversations {
                        let is_selected = self.state.current_conv_id == Some(conv.id);
                        let conv_btn = egui::Button::new(
                            RichText::new(&conv.title)
                                .font(egui::FontId::proportional(12.5))
                                .color(if is_selected {
                                    Theme::TEXT_PARCHMENT
                                } else {
                                    Theme::TEXT_MUTED
                                }),
                        )
                        .fill(if is_selected {
                            Color32::from_rgb(38, 36, 33)
                        } else {
                            Color32::TRANSPARENT
                        })
                        .rounding(Rounding::same(6.0))
                        .min_size(Vec2::new(ui.available_width(), 28.0));

                        if ui.add(conv_btn).clicked() {
                            conv_to_select = Some(conv.id);
                        }
                        ui.add_space(2.0);
                    }
                    if let Some(id) = conv_to_select {
                        self.select_conversation(id);
                    }
                });
            });

        // 2b. Sleek Top Header Bar (ChatGPT style - persistent across all views)
        TopBottomPanel::top("top_header_bar")
            .frame(
                Frame::none()
                    .fill(Theme::BG_CANVAS)
                    .stroke(Stroke::new(1.0_f32, Theme::BORDER_SUBTLE))
                    .inner_margin(Margin::symmetric(24.0, 9.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| match self.state.active_tab {
                    ActiveTab::Chat => {
                        let conv_title = self
                            .state
                            .current_conv_id
                            .and_then(|id| self.state.conversations.iter().find(|c| c.id == id))
                            .map(|c| c.title.as_str())
                            .unwrap_or("Velocity Chat");

                        ui.label(
                            RichText::new(conv_title)
                                .font(egui::FontId::proportional(15.0))
                                .strong()
                                .color(Theme::TEXT_PARCHMENT),
                        );

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            Frame::none()
                                .fill(Theme::BG_CARD)
                                .stroke(Stroke::new(1.0_f32, Theme::BORDER_WARM))
                                .rounding(Rounding::same(10.0))
                                .inner_margin(Margin::symmetric(10.0, 3.0))
                                .show(ui, |ui| {
                                    ui.label(
                                        RichText::new(format!("{}", self.state.active_backend))
                                            .font(egui::FontId::proportional(11.5))
                                            .strong()
                                            .color(Theme::CLAUDE_TERRACOTTA),
                                    );
                                });

                            ui.add_space(8.0);

                            let model_name = Path::new(&self.state.selected_model)
                                .file_stem()
                                .and_then(|n| n.to_str())
                                .unwrap_or(&self.state.selected_model);

                            ui.label(
                                RichText::new(model_name)
                                    .font(egui::FontId::proportional(11.5))
                                    .color(Theme::TEXT_SECONDARY),
                            );
                        });
                    }
                    ActiveTab::Models => {
                        ui.label(
                            RichText::new("Model Library & Runtimes")
                                .font(egui::FontId::proportional(15.0))
                                .strong()
                                .color(Theme::TEXT_PARCHMENT),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(
                                RichText::new("4,096 MB VRAM Allocated | 100% Offline")
                                    .font(egui::FontId::proportional(11.5))
                                    .color(Theme::ACCENT_SAGE),
                            );
                        });
                    }
                    ActiveTab::Hardware => {
                        ui.label(
                            RichText::new("Hardware Telemetry & Benchmarks")
                                .font(egui::FontId::proportional(15.0))
                                .strong()
                                .color(Theme::TEXT_PARCHMENT),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(
                                RichText::new("NVIDIA GTX 1650 | Intel i5-9300H (4 Cores)")
                                    .font(egui::FontId::proportional(11.5))
                                    .color(Theme::CLAUDE_TERRACOTTA),
                            );
                        });
                    }
                    ActiveTab::Settings => {
                        ui.label(
                            RichText::new("Inference Configuration & Tuning")
                                .font(egui::FontId::proportional(15.0))
                                .strong()
                                .color(Theme::TEXT_PARCHMENT),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(
                                RichText::new("Runtime Tuning | Zero Paging Pressure")
                                    .font(egui::FontId::proportional(11.5))
                                    .color(Theme::TEXT_MUTED),
                            );
                        });
                    }
                });
            });

        // 3. Pinned Bottom Chat Input Dock (Rendered ONLY on Chat tab, directly above status bar)
        if self.state.active_tab == ActiveTab::Chat {
            TopBottomPanel::bottom("chat_input_dock")
                .frame(
                    Frame::none()
                        .fill(Theme::BG_CANVAS)
                        .inner_margin(Margin {
                            left: 12.0,
                            right: 12.0,
                            top: 4.0,
                            bottom: 8.0,
                        }),
                )
                .show(ctx, |ui| {
                    render_claude_input_capsule(ui, &mut self.state, &mut on_send, &mut on_stop);
                });
        }

        // 4. Central Canvas (Messages scroll area takes all remaining vertical space!)
        CentralPanel::default()
            .frame(Frame::none().fill(Theme::BG_CANVAS).inner_margin(Margin::same(0.0)))
            .show(ctx, |ui| match self.state.active_tab {
                ActiveTab::Chat => render_chat_messages(ui, &mut self.state, &mut on_send),
                ActiveTab::Models => render_models_view(
                    ui,
                    &mut self.state,
                    &mut on_select_model,
                    &mut on_start_download,
                    &mut on_refresh_models,
                ),
                ActiveTab::Hardware => render_benchmarks_view(ui, &mut self.state, &mut on_switch_backend),
                ActiveTab::Settings => render_settings_view(ui, &mut self.state),
            });

        // Handle dispatched actions
        if let Some(text) = on_send {
            self.send_user_message(text);
        }
        if on_stop {
            self.stop_generation();
        }
        if let Some(model_path) = on_select_model {
            self.state.selected_model = model_path.clone();
            let cfg = InferenceConfig {
                model_path,
                backend: self.state.active_backend,
                threads: self.state.threads,
                gpu_layers: self.state.gpu_layers,
                context_size: self.state.context_size,
                batch_size: self.state.batch_size,
                temperature: self.state.temperature,
                top_p: self.state.top_p,
                max_tokens: 2048,
            };
            let _ = self.cmd_tx.send(EngineCommand::StartEngine { config: cfg });
        }
        if let Some(backend) = on_switch_backend {
            self.state.active_backend = backend;
            let cfg = InferenceConfig {
                model_path: self.state.selected_model.clone(),
                backend,
                threads: self.state.threads,
                gpu_layers: self.state.gpu_layers,
                context_size: self.state.context_size,
                batch_size: self.state.batch_size,
                temperature: self.state.temperature,
                top_p: self.state.top_p,
                max_tokens: 2048,
            };
            let _ = self.cmd_tx.send(EngineCommand::StartEngine { config: cfg });
        }
        if let Some((url, filename)) = on_start_download {
            let _ = self.cmd_tx.send(EngineCommand::StartDownload { url, filename });
        }
        if on_refresh_models {
            if let Ok(models) = self.model_mgr.list_models() {
                self.state.local_models = models;
            }
        }
    }
}
