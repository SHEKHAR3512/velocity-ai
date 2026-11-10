use eframe::egui::{self, Align, Color32, Frame, Layout, Margin, ProgressBar, RichText, Rounding, ScrollArea, Stroke, Ui, Vec2};
use crate::state::AppState;
use crate::theme::Theme;

pub fn render_models_view(
    ui: &mut Ui,
    state: &mut AppState,
    on_select_model: &mut Option<String>,
    on_start_download: &mut Option<(String, String)>,
    on_refresh_models: &mut bool,
) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(10.0);

        // Header
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("📦 Model Manager")
                    .font(egui::FontId::proportional(22.0))
                    .strong()
                    .color(Theme::TEXT_PRIMARY),
            );

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .button(
                        RichText::new("🔄 Refresh List")
                            .font(egui::FontId::proportional(12.0))
                            .color(Theme::ACCENT_CYAN),
                    )
                    .clicked()
                {
                    *on_refresh_models = true;
                }
            });
        });

        ui.add_space(4.0);
        ui.label(
            RichText::new("Curated lightweight models and offline GGUF files optimized for NVIDIA GTX 1650 (4 GB VRAM)")
                .font(egui::FontId::proportional(13.0))
                .color(Theme::TEXT_SECONDARY),
        );

        ui.add_space(16.0);

        // Download Progress Bar if active
        if let Some(ref progress) = state.active_download {
            Frame::none()
                .fill(Theme::BG_CARD)
                .stroke(Stroke::new(1.0_f32, Theme::ACCENT_CYAN))
                .rounding(Rounding::same(8.0))
                .inner_margin(Margin::same(12.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("Downloading {}", progress.file_name))
                                .font(egui::FontId::proportional(13.0))
                                .strong()
                                .color(Theme::ACCENT_CYAN),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(
                                RichText::new(format!("{:.1} MB/s | {:.1}%", progress.speed_mbps, progress.percent))
                                    .font(egui::FontId::monospace(12.0))
                                    .color(Theme::TEXT_PRIMARY),
                            );
                        });
                    });
                    ui.add_space(6.0);
                    ui.add(
                        ProgressBar::new(progress.percent / 100.0)
                            .desired_height(8.0)
                            .rounding(Rounding::same(4.0))
                            .fill(Theme::ACCENT_CYAN),
                    );
                });
            ui.add_space(16.0);
        }

        // Section 1: Installed Local Models
        ui.label(
            RichText::new("Installed Local Models")
                .font(egui::FontId::proportional(16.0))
                .strong()
                .color(Theme::TEXT_PRIMARY),
        );
        ui.add_space(8.0);

        if state.local_models.is_empty() {
            Frame::none()
                .fill(Theme::BG_CARD)
                .rounding(Rounding::same(6.0))
                .inner_margin(Margin::same(14.0))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new("No GGUF models detected in the models directory yet. Download one below or place .gguf files in models/.")
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
                        Color32::from_rgb(26, 32, 44)
                    } else {
                        Theme::BG_CARD
                    })
                    .stroke(Stroke::new(
                        1.0_f32,
                        if is_active {
                            Theme::ACCENT_CYAN
                        } else {
                            Theme::BORDER_SUBTLE
                        },
                    ))
                    .rounding(Rounding::same(8.0))
                    .inner_margin(Margin::same(12.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(&model.file_name)
                                            .font(egui::FontId::proportional(14.0))
                                            .strong()
                                            .color(Theme::TEXT_PRIMARY),
                                    );

                                    if is_active {
                                        ui.label(
                                            RichText::new("ACTIVE")
                                                .font(egui::FontId::monospace(10.0))
                                                .color(Color32::BLACK)
                                                .background_color(Theme::ACCENT_GREEN),
                                        );
                                    }

                                    if model.fits_gtx_1650 {
                                        ui.label(
                                            RichText::new("100% GPU VRAM")
                                                .font(egui::FontId::monospace(10.0))
                                                .color(Theme::ACCENT_GREEN),
                                        );
                                    } else {
                                        ui.label(
                                            RichText::new("CPU/Hybrid")
                                                .font(egui::FontId::monospace(10.0))
                                                .color(Theme::ACCENT_AMBER),
                                        );
                                    }
                                });

                                ui.add_space(2.0);
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(format!(
                                            "Size: {:.1} MB | Arch: {} | Quant: {} | Ctx: {}",
                                            model.file_size_mb,
                                            model.architecture.as_deref().unwrap_or("unknown"),
                                            model.quantization.as_deref().unwrap_or("auto"),
                                            model.context_length.unwrap_or(2048)
                                        ))
                                        .font(egui::FontId::proportional(12.0))
                                        .color(Theme::TEXT_SECONDARY),
                                    );
                                });
                            });

                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if !is_active {
                                    if ui
                                        .button(
                                            RichText::new("Activate")
                                                .font(egui::FontId::proportional(12.0))
                                                .color(Theme::ACCENT_CYAN),
                                        )
                                        .clicked()
                                    {
                                        *on_select_model = Some(model.file_path.to_string_lossy().to_string());
                                    }
                                }
                            });
                        });
                    });
                ui.add_space(8.0);
            }
        }

        ui.add_space(20.0);

        // Section 2: Curated High-Speed Models Catalog
        ui.label(
            RichText::new("Curated High-Speed Models (Empirically Verified)")
                .font(egui::FontId::proportional(16.0))
                .strong()
                .color(Theme::TEXT_PRIMARY),
        );
        ui.add_space(8.0);

        for curated in state.curated_models {
            let is_already_downloaded = state
                .local_models
                .iter()
                .any(|m| m.file_name == curated.default_filename);

            Frame::none()
                .fill(Theme::BG_CARD)
                .stroke(Stroke::new(1.0_f32, Theme::BORDER_SUBTLE))
                .rounding(Rounding::same(8.0))
                .inner_margin(Margin::same(12.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(curated.name)
                                        .font(egui::FontId::proportional(14.0))
                                        .strong()
                                        .color(Theme::TEXT_PRIMARY),
                                );
                                ui.label(
                                    RichText::new(format!("⚡ {}", curated.expected_tps))
                                        .font(egui::FontId::monospace(11.0))
                                        .color(Theme::ACCENT_CYAN),
                                );
                                ui.label(
                                    RichText::new(format!("{:.0} MB", curated.file_size_mb))
                                        .font(egui::FontId::proportional(11.0))
                                        .color(Theme::TEXT_MUTED),
                                );
                            });

                            ui.add_space(2.0);
                            ui.label(
                                RichText::new(curated.description)
                                    .font(egui::FontId::proportional(12.0))
                                    .color(Theme::TEXT_SECONDARY),
                            );
                        });

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if is_already_downloaded {
                                ui.label(
                                    RichText::new("✓ Installed")
                                        .font(egui::FontId::proportional(12.0))
                                        .color(Theme::ACCENT_GREEN),
                                );
                            } else {
                                let download_btn = egui::Button::new(
                                    RichText::new("⬇ Download")
                                        .font(egui::FontId::proportional(12.0))
                                        .strong()
                                        .color(Color32::BLACK),
                                )
                                .fill(Theme::ACCENT_CYAN)
                                .rounding(Rounding::same(5.0))
                                .min_size(Vec2::new(95.0, 30.0));

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
}
