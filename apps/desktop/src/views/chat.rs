use eframe::egui::{self, Align, Color32, Frame, Layout, Margin, RichText, Rounding, ScrollArea, Stroke, TextEdit, Ui, Vec2};
use uuid::Uuid;
use velocity_core::{Message, Role};

use crate::state::AppState;
use crate::theme::Theme;

pub fn render_chat_view(ui: &mut Ui, state: &mut AppState, on_send: &mut Option<String>, on_stop: &mut bool) {
    ui.vertical(|ui| {
        // Chat messages scroll area
        let available_height = ui.available_height() - 90.0;
        ScrollArea::vertical()
            .stick_to_bottom(true)
            .auto_shrink([false, false])
            .max_height(available_height)
            .show(ui, |ui| {
                ui.add_space(8.0);

                if state.messages.is_empty() && state.streaming_text.is_empty() {
                    render_welcome_screen(ui, on_send);
                } else {
                    for msg in &state.messages {
                        if msg.role != Role::System {
                            render_message_bubble(ui, msg);
                            ui.add_space(8.0);
                        }
                    }

                    // Render active streaming message if generating
                    if state.is_generating && !state.streaming_text.is_empty() {
                        let stream_msg = Message::new(
                            state.current_conv_id.unwrap_or_else(Uuid::nil),
                            Role::Assistant,
                            format!("{} ❚", state.streaming_text),
                        );
                        render_message_bubble(ui, &stream_msg);
                    }
                }
            });

        ui.add_space(6.0);

        // Input dock
        render_input_dock(ui, state, on_send, on_stop);
    });
}

fn render_welcome_screen(ui: &mut Ui, on_send: &mut Option<String>) {
    ui.vertical_centered(|ui| {
        ui.add_space(40.0);
        ui.label(
            RichText::new("⚡ VELOCITY AI")
                .font(egui::FontId::proportional(28.0))
                .strong()
                .color(Theme::ACCENT_CYAN),
        );
        ui.add_space(6.0);
        ui.label(
            RichText::new("Ultra-fast offline inference powered by Rust, Vulkan & CUDA")
                .font(egui::FontId::proportional(14.0))
                .color(Theme::TEXT_SECONDARY),
        );
        ui.add_space(28.0);

        ui.label(
            RichText::new("Quick Starters")
                .font(egui::FontId::proportional(13.0))
                .color(Theme::TEXT_MUTED),
        );
        ui.add_space(10.0);

        let prompts = [
            "Explain how Vulkan accelerates LLM inference on an NVIDIA GTX 1650.",
            "Write a high-performance concurrent token streaming client in Rust.",
            "Why is 4 CPU threads optimal for an Intel i5-9300H over 8 threads?",
            "How does speculative decoding achieve 2x speedup on small models?",
        ];

        for prompt in prompts {
            let button = egui::Button::new(
                RichText::new(prompt)
                    .font(egui::FontId::proportional(13.0))
                    .color(Theme::TEXT_PRIMARY),
            )
            .fill(Theme::BG_CARD)
            .stroke(Stroke::new(1.0_f32, Theme::BORDER_SUBTLE))
            .rounding(Rounding::same(6.0))
            .min_size(Vec2::new(540.0, 36.0));

            if ui.add(button).clicked() {
                *on_send = Some(prompt.to_string());
            }
            ui.add_space(6.0);
        }
    });
}

fn render_message_bubble(ui: &mut Ui, msg: &Message) {
    let is_user = msg.role == Role::User;

    let fill_color = if is_user {
        Theme::BG_USER_BUBBLE
    } else {
        Theme::BG_CARD
    };

    let border_color = if is_user {
        Color32::from_rgb(45, 55, 72)
    } else {
        Theme::BORDER_SUBTLE
    };

    Frame::none()
        .fill(fill_color)
        .stroke(Stroke::new(1.0_f32, border_color))
        .rounding(Rounding::same(8.0))
        .inner_margin(Margin::symmetric(14.0, 10.0))
        .show(ui, |ui| {
            ui.set_width(ui.available_width() - 8.0);

            // Message Header (Role + Copy Action)
            ui.horizontal(|ui| {
                if is_user {
                    ui.label(
                        RichText::new("You")
                            .font(egui::FontId::proportional(12.0))
                            .strong()
                            .color(Theme::ACCENT_CYAN),
                    );
                } else {
                    ui.label(
                        RichText::new("VELOCITY AI")
                            .font(egui::FontId::proportional(12.0))
                            .strong()
                            .color(Theme::ACCENT_GREEN),
                    );
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui
                        .button(
                            RichText::new("📋 Copy")
                                .font(egui::FontId::proportional(11.0))
                                .color(Theme::TEXT_MUTED),
                        )
                        .clicked()
                    {
                        ui.ctx().copy_text(msg.content.clone());
                    }
                });
            });

            ui.add_space(4.0);

            // Content with code block extraction
            render_formatted_content(ui, &msg.content);
        });
}

fn render_formatted_content(ui: &mut Ui, content: &str) {
    let mut in_code_block = false;
    let mut code_buffer = String::new();
    let mut text_buffer = String::new();

    for line in content.lines() {
        if line.starts_with("```") {
            if in_code_block {
                // End code block
                render_code_block(ui, &code_buffer);
                code_buffer.clear();
                in_code_block = false;
            } else {
                // Start code block
                if !text_buffer.is_empty() {
                    ui.label(
                        RichText::new(&text_buffer)
                            .font(egui::FontId::proportional(14.0))
                            .color(Theme::TEXT_PRIMARY),
                    );
                    text_buffer.clear();
                }
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
        render_code_block(ui, &code_buffer);
    } else if !text_buffer.is_empty() {
        ui.label(
            RichText::new(text_buffer.trim_end())
                .font(egui::FontId::proportional(14.0))
                .color(Theme::TEXT_PRIMARY),
        );
    }
}

fn render_code_block(ui: &mut Ui, code: &str) {
    let trimmed = code.trim_end();
    Frame::none()
        .fill(Theme::BG_CODE)
        .stroke(Stroke::new(1.0_f32, Theme::BORDER_SUBTLE))
        .rounding(Rounding::same(6.0))
        .inner_margin(Margin::symmetric(10.0, 8.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Code")
                        .font(egui::FontId::monospace(10.0))
                        .color(Theme::TEXT_MUTED),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui
                        .button(
                            RichText::new("Copy")
                                .font(egui::FontId::monospace(10.0))
                                .color(Theme::ACCENT_CYAN),
                        )
                        .clicked()
                    {
                        ui.ctx().copy_text(trimmed.to_string());
                    }
                });
            });
            ui.add_space(2.0);
            ui.label(
                RichText::new(trimmed)
                    .font(egui::FontId::monospace(12.5))
                    .color(Color32::from_rgb(226, 232, 240)),
            );
        });
    ui.add_space(4.0);
}

fn render_input_dock(ui: &mut Ui, state: &mut AppState, on_send: &mut Option<String>, on_stop: &mut bool) {
    Frame::none()
        .fill(Theme::BG_INPUT)
        .stroke(Stroke::new(1.0_f32, Theme::BORDER_SUBTLE))
        .rounding(Rounding::same(10.0))
        .inner_margin(Margin::symmetric(12.0, 8.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let response = ui.add(
                    TextEdit::multiline(&mut state.input_text)
                        .hint_text("Ask VELOCITY AI anything... (Enter to send, Shift+Enter for new line)")
                        .desired_rows(2)
                        .desired_width(ui.available_width() - 110.0)
                        .font(egui::FontId::proportional(14.0))
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

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if state.is_generating {
                        let stop_btn = egui::Button::new(
                            RichText::new("⏹ Stop")
                                .font(egui::FontId::proportional(13.0))
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .fill(Theme::ACCENT_RED)
                        .rounding(Rounding::same(6.0))
                        .min_size(Vec2::new(80.0, 34.0));

                        if ui.add(stop_btn).clicked() {
                            *on_stop = true;
                        }
                    } else {
                        let can_send = !state.input_text.trim().is_empty();
                        let send_btn = egui::Button::new(
                            RichText::new("⚡ Send")
                                .font(egui::FontId::proportional(13.0))
                                .strong()
                                .color(if can_send {
                                    Color32::BLACK
                                } else {
                                    Theme::TEXT_MUTED
                                }),
                        )
                        .fill(if can_send {
                            Theme::ACCENT_CYAN
                        } else {
                            Theme::BG_CARD
                        })
                        .rounding(Rounding::same(6.0))
                        .min_size(Vec2::new(80.0, 34.0));

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
}
