use eframe::egui::{self, Align, Color32, Frame, Layout, Margin, ProgressBar, RichText, Rounding, ScrollArea, Stroke, Ui, Vec2};
use crate::state::AppState;
use crate::theme::Theme;

const MAX_WIDTH: f32 = 820.0;

pub fn render_models_view(
    ui: &mut Ui,
    state: &mut AppState,
    on_select_model: &mut Option<String>,
    on_start_download: &mut Option<(String, String)>,
    on_refresh_models: &mut bool,
) {
    let available_width = ui.available_width();
    let side_padding = ((available_width - MAX_WIDTH) / 2.0).max(12.0);

    ui.horizontal(|ui| {
        ui.add_space(side_padding);

        ui.vertical(|ui| {
            ui.set_max_width(MAX_WIDTH);

            ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(16.0);

                // Header
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Models & Local Runtimes")
                            .font(egui::FontId::proportional(22.0))
                            .strong()
                            .color(Theme::TEXT_PARCHMENT),
                    );

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .button(
                                RichText::new("Refresh")
                                    .font(egui::FontId::proportional(12.0))
                                    .color(Theme::CLAUDE_TERRACOTTA),
                            )
                            .clicked()
                        {
                            *on_refresh_models = true;
                        }
                    });
                });

                ui.add_space(4.0);
                ui.label(
                    RichText::new("Fast, lightweight models optimized to run 100% offline within your 4 GB GPU VRAM")
                        .font(egui::FontId::proportional(13.5))
                        .color(Theme::TEXT_SECONDARY),
                );

                ui.add_space(20.0);

                // Download Progress if active
                if let Some(ref progress) = state.active_download {
                    Frame::none()
                        .fill(Theme::BG_CARD)
                        .stroke(Stroke::new(1.0_f32, Theme::CLAUDE_TERRACOTTA))
                        .rounding(Rounding::same(12.0))
                        .inner_margin(Margin::same(14.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(format!("Downloading {}", progress.file_name))
                                        .font(egui::FontId::proportional(13.5))
                                        .strong()
                                        .color(Theme::CLAUDE_TERRACOTTA),
                                );
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.label(
                                        RichText::new(format!("{:.1} MB/s | {:.1}%", progress.speed_mbps, progress.percent))
                                            .font(egui::FontId::monospace(12.0))
                                            .color(Theme::TEXT_PARCHMENT),
                                    );
                                });
                            });
                            ui.add_space(8.0);
                            ui.add(
                                ProgressBar::new(progress.percent / 100.0)
                                    .desired_height(6.0)
                                    .rounding(Rounding::same(3.0))
                                    .fill(Theme::CLAUDE_TERRACOTTA),
                            );
                        });
                    ui.add_space(20.0);
                }

                // Section 1: Installed Models
                ui.label(
                    RichText::new("Active & Installed Models")
                        .font(egui::FontId::proportional(16.0))
                        .strong()
                        .color(Theme::TEXT_PARCHMENT),
                );
                ui.add_space(10.0);

                if state.local_models.is_empty() {
                    Frame::none()
                        .fill(Theme::BG_CARD)
                        .stroke(Stroke::new(1.0_f32, Theme::BORDER_SUBTLE))
                        .rounding(Rounding::same(10.0))
                        .inner_margin(Margin::same(16.0))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new("No local models detected. Choose one of the curated models below to download.")
                                    .font(egui::FontId::proportional(13.0))
                                    .color(Theme::TEXT_MUTED),
                            );
                        });
                } else {
                    for model in &state.local_models {
                        let is_active = state.selected_model.ends_with(&model.file_name)
                            || model.file_path.to_string_lossy() == state.selected_model;

                        Frame::none()
                            .fill(if is_active {
                                Color32::from_rgb(44, 40, 36)
                            } else {
                                Theme::BG_CARD
                            })
                            .stroke(Stroke::new(
                                1.0_f32,
                                if is_active {
                                    Theme::CLAUDE_TERRACOTTA
                                } else {
                                    Theme::BORDER_SUBTLE
                                },
                            ))
                            .rounding(Rounding::same(10.0))
                            .inner_margin(Margin::same(14.0))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                RichText::new(&model.file_name)
                                                    .font(egui::FontId::proportional(14.5))
                                                    .strong()
                                                    .color(Theme::TEXT_PARCHMENT),
                                            );

                                            if is_active {
                                                ui.label(
                                                    RichText::new("ACTIVE")
                                                        .font(egui::FontId::monospace(10.0))
                                                        .color(Color32::BLACK)
                                                        .background_color(Theme::CLAUDE_TERRACOTTA),
                                                );
                                            }

                                            if model.fits_gtx_1650 {
                                                ui.label(
                                                    RichText::new("100% VRAM")
                                                        .font(egui::FontId::monospace(10.0))
                                                        .color(Theme::ACCENT_SAGE),
                                                );
                                            }
                                        });

                                        ui.add_space(3.0);
                                        ui.label(
                                            RichText::new(format!(
                                                "{:.0} MB | Arch: {} | Quant: {} | Context: {} tokens",
                                                model.file_size_mb,
                                                model.architecture.as_deref().unwrap_or("qwen2"),
                                                model.quantization.as_deref().unwrap_or("Q4_K_M"),
                                                model.context_length.unwrap_or(2048)
                                            ))
                                            .font(egui::FontId::proportional(12.0))
                                            .color(Theme::TEXT_SECONDARY),
                                        );
                                    });

                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        if !is_active {
                                            let select_btn = egui::Button::new(
                                                RichText::new("Activate")
                                                    .font(egui::FontId::proportional(12.0))
                                                    .color(Theme::CLAUDE_TERRACOTTA),
                                            )
                                            .fill(Color32::from_rgb(46, 44, 40))
                                            .rounding(Rounding::same(6.0));

                                            if ui.add(select_btn).clicked() {
                                                *on_select_model = Some(model.file_path.to_string_lossy().to_string());
                                            }
                                        }
                                    });
                                });
                            });
                        ui.add_space(8.0);
                    }
                }

                ui.add_space(24.0);

                // Section 2: Curated Catalog
                ui.label(
                    RichText::new("Curated High-Performance Models")
                        .font(egui::FontId::proportional(16.0))
                        .strong()
                        .color(Theme::TEXT_PARCHMENT),
                );
                ui.add_space(10.0);

                for curated in state.curated_models {
                    let is_already_downloaded = state
                        .local_models
                        .iter()
                        .any(|m| m.file_name == curated.default_filename);

                    Frame::none()
                        .fill(Theme::BG_CARD)
                        .stroke(Stroke::new(1.0_f32, Theme::BORDER_SUBTLE))
                        .rounding(Rounding::same(10.0))
                        .inner_margin(Margin::same(14.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new(curated.name)
                                                .font(egui::FontId::proportional(14.5))
                                                .strong()
                                                .color(Theme::TEXT_PARCHMENT),
                                        );
                                        ui.label(
                                            RichText::new(curated.expected_tps)
                                                .font(egui::FontId::monospace(11.0))
                                                .color(Theme::CLAUDE_TERRACOTTA),
                                        );
                                        ui.label(
                                            RichText::new(format!("{:.0} MB", curated.file_size_mb))
                                                .font(egui::FontId::proportional(11.0))
                                                .color(Theme::TEXT_MUTED),
                                        );
                                    });

                                    ui.add_space(3.0);
                                    ui.label(
                                        RichText::new(curated.description)
                                            .font(egui::FontId::proportional(12.5))
                                            .color(Theme::TEXT_SECONDARY),
                                    );
                                });

                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if is_already_downloaded {
                                        ui.label(
                                            RichText::new("Installed")
                                                .font(egui::FontId::proportional(12.0))
                                                .color(Theme::ACCENT_SAGE),
                                        );
                                    } else {
                                        let download_btn = egui::Button::new(
                                            RichText::new("Download")
                                                .font(egui::FontId::proportional(12.0))
                                                .strong()
                                                .color(Color32::WHITE),
                                        )
                                        .fill(Theme::CLAUDE_TERRACOTTA)
                                        .rounding(Rounding::same(6.0))
                                        .min_size(Vec2::new(88.0, 28.0));

                                        if ui.add(download_btn).clicked() {
                                            *on_start_download = Some((
                                                curated.download_url.to_string(),
                                                curated.default_filename.to_string(),
                                            ));
                                        }
                                    }
                                });
                            });
                        });
                    ui.add_space(8.0);
                }
            });
        });

        ui.add_space(side_padding);
    });
}
