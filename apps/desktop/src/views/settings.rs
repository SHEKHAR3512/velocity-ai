use eframe::egui::{self, Align, Color32, Frame, Layout, Margin, RichText, Rounding, ScrollArea, Stroke, TextEdit, Ui, Vec2};
use crate::state::AppState;
use crate::theme::Theme;

const MAX_WIDTH: f32 = 820.0;

pub fn render_settings_view(ui: &mut Ui, state: &mut AppState) {
    let available_width = ui.available_width();
    let content_width = available_width.min(MAX_WIDTH);
    let side_padding = ((available_width - content_width) / 2.0).max(0.0);

    ScrollArea::vertical()
        .stick_to_bottom(false)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(side_padding);

                ui.vertical(|ui| {
                    ui.set_width(content_width);
                    ui.set_max_width(content_width);
                    ui.add_space(20.0);

                    // Hero Header
                    ui.label(
                        RichText::new("Inference Configuration & Tuning")
                            .font(egui::FontId::proportional(22.0))
                            .strong()
                            .color(Theme::TEXT_PARCHMENT),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("Fine-tune sampling hyperparameters, context memory window, thread limits, and system persona")
                            .font(egui::FontId::proportional(13.5))
                            .color(Theme::TEXT_SECONDARY),
                    );

                    ui.add_space(24.0);

                    // Card 1: System Persona & Instructions
                    ui.label(
                        RichText::new("System Persona & Instructions")
                            .font(egui::FontId::proportional(16.0))
                            .strong()
                            .color(Theme::TEXT_PARCHMENT),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("Prepended as the system prompt to guide tone, role, and formatting of assistant replies")
                            .font(egui::FontId::proportional(12.5))
                            .color(Theme::TEXT_MUTED),
                    );
                    ui.add_space(10.0);

                    Frame::none()
                        .fill(Theme::BG_CARD)
                        .stroke(Stroke::new(1.0_f32, Theme::BORDER_SUBTLE))
                        .rounding(Rounding::same(12.0))
                        .inner_margin(Margin::same(16.0))
                        .show(ui, |ui| {
                            ui.add(
                                TextEdit::multiline(&mut state.system_prompt)
                                    .hint_text("Enter system persona instructions here...")
                                    .desired_rows(4)
                                    .desired_width(content_width)
                                    .font(egui::FontId::proportional(13.5))
                                    .frame(false),
                            );
                        });

                    ui.add_space(28.0);

                    // Card 2: Compute & Memory Tuning
                    ui.label(
                        RichText::new("Hardware & Memory Allocation")
                            .font(egui::FontId::proportional(16.0))
                            .strong()
                            .color(Theme::TEXT_PARCHMENT),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("Tuned for dual-channel RAM, Intel 8MB L3 cache, and GTX 1650 GDDR5 VRAM")
                            .font(egui::FontId::proportional(12.5))
                            .color(Theme::TEXT_MUTED),
                    );
                    ui.add_space(10.0);

                    Frame::none()
                        .fill(Theme::BG_CARD)
                        .stroke(Stroke::new(1.0_f32, Theme::BORDER_SUBTLE))
                        .rounding(Rounding::same(12.0))
                        .inner_margin(Margin::same(18.0))
                        .show(ui, |ui| {
                            // Context Window Size
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(
                                        RichText::new("Context Size (Tokens)")
                                            .font(egui::FontId::proportional(14.0))
                                            .strong()
                                            .color(Theme::TEXT_PARCHMENT),
                                    );
                                    ui.label(
                                        RichText::new("Max tokens retained in memory before sliding window truncates")
                                            .font(egui::FontId::proportional(12.0))
                                            .color(Theme::TEXT_MUTED),
                                    );
                                });

                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.add(egui::Slider::new(&mut state.context_size, 512..=8192).step_by(512.0));
                                });
                            });

                            ui.add_space(14.0);
                            ui.separator();
                            ui.add_space(14.0);

                            // CPU Threads
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new("CPU Worker Threads")
                                                .font(egui::FontId::proportional(14.0))
                                                .strong()
                                                .color(Theme::TEXT_PARCHMENT),
                                        );
                                        ui.label(
                                            RichText::new("4 RECOMMENDED")
                                                .font(egui::FontId::monospace(10.0))
                                                .strong()
                                                .color(Theme::ACCENT_SAGE),
                                        );
                                    });
                                    ui.label(
                                        RichText::new("4 physical cores matches L3 cache; 8 threads triggers hyperthread thrashing")
                                            .font(egui::FontId::proportional(12.0))
                                            .color(Theme::TEXT_MUTED),
                                    );
                                });

                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.add(egui::Slider::new(&mut state.threads, 1..=8));
                                });
                            });

                            ui.add_space(14.0);
                            ui.separator();
                            ui.add_space(14.0);

                            // GPU Layers Offload
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new("GPU Layer Offload")
                                                .font(egui::FontId::proportional(14.0))
                                                .strong()
                                                .color(Theme::TEXT_PARCHMENT),
                                        );
                                        ui.label(
                                            RichText::new("33 = 100% VRAM")
                                                .font(egui::FontId::monospace(10.0))
                                                .strong()
                                                .color(Theme::CLAUDE_TERRACOTTA),
                                        );
                                    });
                                    ui.label(
                                        RichText::new("Offload transformer layers into 4 GB GDDR5 discrete memory")
                                            .font(egui::FontId::proportional(12.0))
                                            .color(Theme::TEXT_MUTED),
                                    );
                                });

                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.add(egui::Slider::new(&mut state.gpu_layers, 0..=33));
                                });
                            });
                        });

                    ui.add_space(28.0);

                    // Card 3: Sampling & Generation
                    ui.label(
                        RichText::new("Sampling & Generation Hyperparameters")
                            .font(egui::FontId::proportional(16.0))
                            .strong()
                            .color(Theme::TEXT_PARCHMENT),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("Control creativity, randomness, nucleus filtering, and prompt batch processing")
                            .font(egui::FontId::proportional(12.5))
                            .color(Theme::TEXT_MUTED),
                    );
                    ui.add_space(10.0);

                    Frame::none()
                        .fill(Theme::BG_CARD)
                        .stroke(Stroke::new(1.0_f32, Theme::BORDER_SUBTLE))
                        .rounding(Rounding::same(12.0))
                        .inner_margin(Margin::same(18.0))
                        .show(ui, |ui| {
                            // Temperature
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(
                                        RichText::new("Temperature")
                                            .font(egui::FontId::proportional(14.0))
                                            .strong()
                                            .color(Theme::TEXT_PARCHMENT),
                                    );
                                    ui.label(
                                        RichText::new("Higher = more creative and varied, Lower = more deterministic and focused")
                                            .font(egui::FontId::proportional(12.0))
                                            .color(Theme::TEXT_MUTED),
                                    );
                                });

                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.add(egui::Slider::new(&mut state.temperature, 0.0..=1.5).step_by(0.05));
                                });
                            });

                            ui.add_space(14.0);
                            ui.separator();
                            ui.add_space(14.0);

                            // Top-P
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(
                                        RichText::new("Top-P (Nucleus Sampling)")
                                            .font(egui::FontId::proportional(14.0))
                                            .strong()
                                            .color(Theme::TEXT_PARCHMENT),
                                    );
                                    ui.label(
                                        RichText::new("Limits token selection to cumulative probability threshold")
                                            .font(egui::FontId::proportional(12.0))
                                            .color(Theme::TEXT_MUTED),
                                    );
                                });

                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.add(egui::Slider::new(&mut state.top_p, 0.1..=1.0).step_by(0.05));
                                });
                            });

                            ui.add_space(14.0);
                            ui.separator();
                            ui.add_space(14.0);

                            // Batch Size
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(
                                        RichText::new("Batch Processing Size")
                                            .font(egui::FontId::proportional(14.0))
                                            .strong()
                                            .color(Theme::TEXT_PARCHMENT),
                                    );
                                    ui.label(
                                        RichText::new("Prompt tokens evaluated per GEMM compute kernel dispatch")
                                            .font(egui::FontId::proportional(12.0))
                                            .color(Theme::TEXT_MUTED),
                                    );
                                });

                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.add(egui::Slider::new(&mut state.batch_size, 128..=2048).step_by(128.0));
                                });
                            });
                        });

                    ui.add_space(20.0);

                    // Reset to Defaults Button
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let reset_btn = egui::Button::new(
                            RichText::new("Reset to Optimized Hardware Defaults")
                                .font(egui::FontId::proportional(12.5))
                                .strong()
                                .color(Theme::CLAUDE_TERRACOTTA),
                        )
                        .fill(Color32::from_rgb(38, 36, 33))
                        .stroke(Stroke::new(1.0_f32, Theme::BORDER_WARM))
                        .rounding(Rounding::same(8.0))
                        .min_size(Vec2::new(220.0, 32.0));

                        if ui.add(reset_btn).clicked() {
                            state.threads = 4;
                            state.gpu_layers = 33;
                            state.context_size = 2048;
                            state.batch_size = 512;
                            state.temperature = 0.7;
                            state.top_p = 0.9;
                            state.system_prompt = "You are Velocity, a high-performance local AI assistant running offline on pure Rust and hardware acceleration.".to_string();
                        }
                    });

                    ui.add_space(30.0);
                });

                ui.add_space(side_padding);
            });
        });
}
