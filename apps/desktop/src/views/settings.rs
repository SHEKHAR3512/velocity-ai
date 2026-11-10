use eframe::egui::{self, Frame, Margin, RichText, Rounding, ScrollArea, Stroke, TextEdit, Ui};
use crate::state::AppState;
use crate::theme::Theme;

const MAX_WIDTH: f32 = 820.0;

pub fn render_settings_view(ui: &mut Ui, state: &mut AppState) {
    let available_width = ui.available_width();
    let side_padding = ((available_width - MAX_WIDTH) / 2.0).max(12.0);

    ui.horizontal(|ui| {
        ui.add_space(side_padding);

        ui.vertical(|ui| {
            ui.set_max_width(MAX_WIDTH);

            ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(16.0);
                ui.label(
                    RichText::new("✦ Inference Settings")
                        .font(egui::FontId::proportional(22.0))
                        .strong()
                        .color(Theme::TEXT_PARCHMENT),
                );
                ui.add_space(4.0);
                ui.label(
                    RichText::new("Fine-tune sampling parameters, context length, system instructions, and thread allocation")
                        .font(egui::FontId::proportional(13.5))
                        .color(Theme::TEXT_SECONDARY),
                );

                ui.add_space(20.0);

                Frame::none()
                    .fill(Theme::BG_CARD)
                    .stroke(Stroke::new(1.0_f32, Theme::BORDER_SUBTLE))
                    .rounding(Rounding::same(12.0))
                    .inner_margin(Margin::same(18.0))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new("System Instructions")
                                .font(egui::FontId::proportional(14.0))
                                .strong()
                                .color(Theme::TEXT_PARCHMENT),
                        );
                        ui.add_space(6.0);
                        ui.add(
                            TextEdit::multiline(&mut state.system_prompt)
                                .desired_rows(3)
                                .desired_width(ui.available_width())
                                .font(egui::FontId::proportional(13.5)),
                        );

                        ui.add_space(18.0);
                        ui.separator();
                        ui.add_space(14.0);

                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("Context Size (tokens):")
                                    .font(egui::FontId::proportional(13.5))
                                    .color(Theme::TEXT_PARCHMENT),
                            );
                            ui.add(egui::Slider::new(&mut state.context_size, 512..=8192).step_by(512.0));
                        });

                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("CPU Threads (4 Optimal):")
                                    .font(egui::FontId::proportional(13.5))
                                    .color(Theme::TEXT_PARCHMENT),
                            );
                            ui.add(egui::Slider::new(&mut state.threads, 1..=8));
                        });

                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("Temperature:")
                                    .font(egui::FontId::proportional(13.5))
                                    .color(Theme::TEXT_PARCHMENT),
                            );
                            ui.add(egui::Slider::new(&mut state.temperature, 0.0..=1.5).step_by(0.05));
                        });

                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("Top-P:")
                                    .font(egui::FontId::proportional(13.5))
                                    .color(Theme::TEXT_PARCHMENT),
                            );
                            ui.add(egui::Slider::new(&mut state.top_p, 0.1..=1.0).step_by(0.05));
                        });
                    });
            });
        });

        ui.add_space(side_padding);
    });
}
