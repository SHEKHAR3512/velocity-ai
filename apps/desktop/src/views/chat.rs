use eframe::egui::{self, Align, Color32, Frame, Layout, Margin, RichText, Rounding, ScrollArea, Stroke, TextEdit, Ui, Vec2};
use uuid::Uuid;
use velocity_core::{Message, Role};

use crate::state::AppState;
use crate::theme::Theme;

const MAX_CHAT_WIDTH: f32 = 780.0;

pub fn render_chat_view(ui: &mut Ui, state: &mut AppState, on_send: &mut Option<String>, on_stop: &mut bool) {
    let full_rect = ui.available_rect_before_wrap();
    let available_width = full_rect.width();
    let content_width = available_width.min(MAX_CHAT_WIDTH);
    let margin_x = ((available_width - content_width) / 2.0).max(0.0);

    let chat_rect = egui::Rect::from_min_size(
        egui::pos2(full_rect.min.x + margin_x, full_rect.min.y),
        egui::vec2(content_width, full_rect.height()),
    );

    ui.allocate_new_ui(
        egui::UiBuilder::new()
            .max_rect(chat_rect)
            .layout(Layout::bottom_up(Align::Center)),
        |ui| {
            ui.add_space(8.0);

            // 1. PINNED BOTTOM INPUT CAPSULE (always at the very bottom, like ChatGPT & Claude)
            render_claude_input_capsule(ui, state, on_send, on_stop);

            ui.add_space(10.0);

            // 2. CHAT MESSAGES SCROLL AREA (Takes all remaining vertical space up to the top)
            ScrollArea::vertical()
                .stick_to_bottom(true)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.with_layout(Layout::top_down(Align::Min), |ui| {
                        ui.set_width(ui.available_width());
                        ui.add_space(12.0);

                        if state.messages.is_empty() && state.streaming_text.is_empty() {
                            render_claude_welcome(ui, on_send);
                        } else {
                            for msg in &state.messages {
                                if msg.role != Role::System {
                                    render_claude_message(ui, msg);
                                    ui.add_space(18.0);
                                }
                            }

                            // Render active streaming message if generating
                            if state.is_generating && !state.streaming_text.is_empty() {
                                let stream_msg = Message::new(
                                    state.current_conv_id.unwrap_or_else(Uuid::nil),
                                    Role::Assistant,
                                    format!("{} |", state.streaming_text),
                                );
                                render_claude_message(ui, &stream_msg);
                                ui.add_space(18.0);
                            }
                        }
                        ui.add_space(16.0);
                    });
                });
        },
    );
}

fn render_claude_welcome(ui: &mut Ui, on_send: &mut Option<String>) {
    ui.vertical_centered(|ui| {
        ui.add_space(30.0);

        // Claude-style terracotta header
        ui.label(
            RichText::new("VELOCITY AI")
                .font(egui::FontId::proportional(26.0))
                .strong()
                .color(Theme::CLAUDE_TERRACOTTA),
        );
        ui.add_space(8.0);

        ui.label(
            RichText::new("How can I help you today?")
                .font(egui::FontId::proportional(22.0))
                .strong()
                .color(Theme::TEXT_PARCHMENT),
        );
        ui.add_space(6.0);

        ui.label(
            RichText::new("Ultra-fast offline intelligence running locally on your laptop")
                .font(egui::FontId::proportional(13.5))
                .color(Theme::TEXT_SECONDARY),
        );
        ui.add_space(32.0);

        // 2x2 Clean Claude Suggestion Tiles
        let starters = [
            (
                "Hardware Acceleration",
                "How Vulkan & CUDA unlock 160+ tok/s on an NVIDIA GTX 1650 laptop.",
                "Explain how Vulkan compute and CUDA accelerate LLM inference on an NVIDIA GTX 1650 laptop."
            ),
            (
                "Concurrent Rust Client",
                "Build an asynchronous token streaming client using Tokio MPSC channels.",
                "Write a high-performance concurrent token streaming client in Rust using Tokio MPSC channels."
            ),
            (
                "L3 Cache & Thread Tuning",
                "Why 4 physical cores outperform 8 hyperthreads in GEMM matrix math.",
                "Explain why 4 physical CPU threads beat 8 hyperthreads on an Intel i5-9300H for LLM inference."
            ),
            (
                "Speculative Decoding",
                "How small draft models achieve 2x generation speedups on consumer laptops.",
                "How does speculative decoding work and how can small draft models double inference speed on laptops?"
            ),
        ];

        egui::Grid::new("claude_starters_grid")
            .num_columns(2)
            .spacing([14.0, 14.0])
            .show(ui, |ui| {
                for (i, (title, desc, prompt)) in starters.iter().enumerate() {
                    let card_frame = Frame::none()
                        .fill(Theme::BG_CARD)
                        .stroke(Stroke::new(1.0_f32, Theme::BORDER_SUBTLE))
                        .rounding(Rounding::same(12.0))
                        .inner_margin(Margin::same(14.0));

                    let response = card_frame.show(ui, |ui| {
                        ui.set_width(360.0);
                        ui.set_height(70.0);
                        ui.vertical(|ui| {
                            ui.label(
                                RichText::new(*title)
                                    .font(egui::FontId::proportional(14.0))
                                    .strong()
                                    .color(Theme::TEXT_PARCHMENT),
                            );
                            ui.add_space(4.0);
                            ui.label(
                                RichText::new(*desc)
                                    .font(egui::FontId::proportional(12.0))
                                    .color(Theme::TEXT_MUTED),
                            );
                        });
                    });

                    if response.response.interact(egui::Sense::click()).clicked() {
                        *on_send = Some(prompt.to_string());
                    }

                    if i % 2 == 1 {
                        ui.end_row();
                    }
                }
            });
    });
}

fn render_claude_message(ui: &mut Ui, msg: &Message) {
    if msg.role == Role::User {
        // User message: Clean right-aligned pill
        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
            Frame::none()
                .fill(Theme::BG_USER_BUBBLE)
                .stroke(Stroke::new(1.0_f32, Theme::BORDER_WARM))
                .rounding(Rounding::same(16.0))
                .inner_margin(Margin::symmetric(16.0, 12.0))
                .show(ui, |ui| {
                    ui.set_max_width(MAX_CHAT_WIDTH * 0.82);
                    ui.label(
                        RichText::new(&msg.content)
                            .font(egui::FontId::proportional(14.5))
                            .color(Theme::TEXT_PARCHMENT),
                    );
                });
        });
    } else {
        // Assistant message: Editorial natural prose directly on canvas (No harsh box!)
        ui.vertical(|ui| {
            // Header
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Velocity Assistant")
                        .font(egui::FontId::proportional(13.0))
                        .strong()
                        .color(Theme::CLAUDE_TERRACOTTA),
                );

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui
                        .button(
                            RichText::new("Copy")
                                .font(egui::FontId::proportional(11.0))
                                .color(Theme::TEXT_MUTED),
                        )
                        .clicked()
                    {
                        ui.ctx().copy_text(msg.content.clone());
                    }
                });
            });

            ui.add_space(6.0);

            // Rich Editorial Content
            render_editorial_content(ui, &msg.content);
        });
    }
}

fn render_editorial_content(ui: &mut Ui, content: &str) {
    let mut in_code_block = false;
    let mut code_buffer = String::new();
    let mut code_lang = String::new();
    let mut text_buffer = String::new();

    for line in content.lines() {
        if line.starts_with("```") {
            if in_code_block {
                // End of code block
                render_claude_code_block(ui, &code_lang, &code_buffer);
                code_buffer.clear();
                code_lang.clear();
                in_code_block = false;
            } else {
                // Start of code block
                if !text_buffer.is_empty() {
                    render_prose_lines(ui, &text_buffer);
                    text_buffer.clear();
                }
                code_lang = line.trim_start_matches("```").trim().to_string();
                in_code_block = true;
            }
        } else if in_code_block {
            code_buffer.push_str(line);
            code_buffer.push('\n');
        } else {
            text_buffer.push_str(line);
            text_buffer.push('\n');
        }
    }

    if in_code_block && !code_buffer.is_empty() {
        render_claude_code_block(ui, &code_lang, &code_buffer);
    } else if !text_buffer.is_empty() {
        render_prose_lines(ui, &text_buffer);
    }
}

fn render_prose_lines(ui: &mut Ui, text: &str) {
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            ui.add_space(6.0);
            continue;
        }

        if let Some(h1) = trimmed.strip_prefix("# ") {
            ui.add_space(10.0);
            ui.label(
                RichText::new(h1)
                    .font(egui::FontId::proportional(20.0))
                    .strong()
                    .color(Theme::TEXT_PARCHMENT),
            );
            ui.add_space(4.0);
        } else if let Some(h2) = trimmed.strip_prefix("## ") {
            ui.add_space(8.0);
            ui.label(
                RichText::new(h2)
                    .font(egui::FontId::proportional(17.0))
                    .strong()
                    .color(Theme::TEXT_PARCHMENT),
            );
            ui.add_space(3.0);
        } else if let Some(h3) = trimmed.strip_prefix("### ") {
            ui.add_space(6.0);
            ui.label(
                RichText::new(h3)
                    .font(egui::FontId::proportional(15.0))
                    .strong()
                    .color(Theme::CLAUDE_TERRACOTTA),
            );
            ui.add_space(2.0);
        } else if let Some(quote) = trimmed.strip_prefix("> ") {
            // Blockquote with warm terracotta border
            Frame::none()
                .stroke(Stroke::new(2.5_f32, Theme::CLAUDE_TERRACOTTA))
                .inner_margin(Margin { left: 12.0, right: 0.0, top: 2.0, bottom: 2.0 })
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(quote)
                            .font(egui::FontId::proportional(14.0))
                            .italics()
                            .color(Theme::TEXT_SECONDARY),
                    );
                });
            ui.add_space(4.0);
        } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            let bullet_text = &trimmed[2..];
            ui.horizontal(|ui| {
                ui.label(RichText::new("-").font(egui::FontId::proportional(14.0)).color(Theme::CLAUDE_TERRACOTTA));
                ui.label(
                    RichText::new(bullet_text)
                        .font(egui::FontId::proportional(14.5))
                        .color(Theme::TEXT_PARCHMENT),
                );
            });
            ui.add_space(2.0);
        } else {
            ui.label(
                RichText::new(trimmed)
                    .font(egui::FontId::proportional(14.5))
                    .color(Theme::TEXT_PARCHMENT),
            );
            ui.add_space(2.0);
        }
    }
}

fn render_claude_code_block(ui: &mut Ui, lang: &str, code: &str) {
    let trimmed = code.trim_end();
    let display_lang = if lang.is_empty() { "code" } else { lang };

    Frame::none()
        .fill(Theme::BG_CODE)
        .stroke(Stroke::new(1.0_f32, Theme::BORDER_SUBTLE))
        .rounding(Rounding::same(8.0))
        .inner_margin(Margin::same(12.0))
        .show(ui, |ui| {
            // Header bar
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(display_lang.to_uppercase())
                        .font(egui::FontId::monospace(10.5))
                        .color(Theme::TEXT_MUTED),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui
                        .button(
                            RichText::new("Copy code")
                                .font(egui::FontId::proportional(11.0))
                                .color(Theme::CLAUDE_TERRACOTTA),
                        )
                        .clicked()
                    {
                        ui.ctx().copy_text(trimmed.to_string());
                    }
                });
            });

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(6.0);

            // Monospace code
            ui.label(
                RichText::new(trimmed)
                    .font(egui::FontId::monospace(12.5))
                    .color(Color32::from_rgb(230, 228, 222)),
            );
        });
    ui.add_space(8.0);
}

fn render_claude_input_capsule(ui: &mut Ui, state: &mut AppState, on_send: &mut Option<String>, on_stop: &mut bool) {
    let capsule_width = ui.available_width();
    Frame::none()
        .fill(Theme::BG_INPUT)
        .stroke(Stroke::new(1.0_f32, Theme::BORDER_WARM))
        .rounding(Rounding::same(16.0))
        .inner_margin(Margin::symmetric(16.0, 10.0))
        .show(ui, |ui| {
            ui.set_width(capsule_width);
            ui.vertical(|ui| {
                // Text input
                let response = ui.add(
                    TextEdit::multiline(&mut state.input_text)
                        .hint_text("Reply to Velocity... (Enter to send, Shift+Enter for new line)")
                        .desired_rows(2)
                        .desired_width(ui.available_width())
                        .font(egui::FontId::proportional(14.5))
                        .frame(false),
                );

                // Handle Enter key for send (without Shift)
                if response.has_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift)
                {
                    let trimmed = state.input_text.trim().to_string();
                    if !trimmed.is_empty() && !state.is_generating {
                        *on_send = Some(trimmed);
                        state.input_text.clear();
                    }
                }

                ui.add_space(6.0);

                // Capsule bottom action row
                ui.horizontal(|ui| {
                    // Model badge pill
                    ui.label(
                        RichText::new("Velocity Instant | Qwen 0.5B")
                            .font(egui::FontId::proportional(11.5))
                            .color(Theme::CLAUDE_TERRACOTTA),
                    );

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if state.is_generating {
                            // Pulsing stop button
                            let stop_btn = egui::Button::new(
                                RichText::new("Stop")
                                    .font(egui::FontId::proportional(12.0))
                                    .strong()
                                    .color(Color32::WHITE),
                            )
                            .fill(Theme::ACCENT_RED)
                            .rounding(Rounding::same(14.0))
                            .min_size(Vec2::new(72.0, 28.0));

                            if ui.add(stop_btn).clicked() {
                                *on_stop = true;
                            }
                        } else {
                            let can_send = !state.input_text.trim().is_empty();
                            let send_btn = egui::Button::new(
                                RichText::new("Send")
                                    .font(egui::FontId::proportional(12.5))
                                    .strong()
                                    .color(if can_send {
                                        Color32::WHITE
                                    } else {
                                        Theme::TEXT_MUTED
                                    }),
                            )
                            .fill(if can_send {
                                Theme::CLAUDE_TERRACOTTA
                            } else {
                                Color32::from_rgb(46, 44, 40)
                            })
                            .rounding(Rounding::same(14.0))
                            .min_size(Vec2::new(72.0, 28.0));

                            if ui.add_enabled(can_send, send_btn).clicked() {
                                let trimmed = state.input_text.trim().to_string();
                                if !trimmed.is_empty() {
                                    *on_send = Some(trimmed);
                                    state.input_text.clear();
                                }
                            }
                        }
                    });
                });
            });
        });
}
