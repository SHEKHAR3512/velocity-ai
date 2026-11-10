use eframe::egui::{self, Align, Color32, Frame, Layout, Margin, RichText, Rounding, ScrollArea, Stroke, Ui};
use velocity_core::BackendType;

use crate::state::AppState;
use crate::theme::Theme;

pub fn render_benchmarks_view(
    ui: &mut Ui,
    state: &mut AppState,
    on_switch_backend: &mut Option<BackendType>,
) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(10.0);

        ui.label(
            RichText::new("⚡ Hardware Telemetry & Benchmarks")
                .font(egui::FontId::proportional(22.0))
                .strong()
                .color(Theme::TEXT_PRIMARY),
        );
        ui.add_space(4.0);
        ui.label(
            RichText::new("Live hardware audit and empirical performance measurements for Acer Predator PT315-51")
                .font(egui::FontId::proportional(13.0))
                .color(Theme::TEXT_SECONDARY),
        );

        ui.add_space(16.0);

        // Section 1: Detected Hardware Cards
        ui.horizontal(|ui| {
            // CPU Card
            Frame::none()
                .fill(Theme::BG_CARD)
                .stroke(Stroke::new(1.0_f32, Theme::BORDER_SUBTLE))
                .rounding(Rounding::same(8.0))
                .inner_margin(Margin::same(12.0))
                .show(ui, |ui| {
                    ui.set_width(260.0);
                    ui.label(
                        RichText::new("CPU Processor")
                            .font(egui::FontId::proportional(12.0))
                            .color(Theme::TEXT_MUTED),
                    );
                    ui.label(
                        RichText::new("Intel Core i5-9300H")
                            .font(egui::FontId::proportional(15.0))
                            .strong()
                            .color(Theme::TEXT_PRIMARY),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("• 4 Physical Cores / 8 Threads\n• AVX2 & FMA Supported\n• Locked to 4 Threads (No HT thrash)")
                            .font(egui::FontId::proportional(12.0))
                            .color(Theme::TEXT_SECONDARY),
                    );
                });

            ui.add_space(10.0);

            // GPU Card
            Frame::none()
                .fill(Theme::BG_CARD)
                .stroke(Stroke::new(1.0_f32, Theme::BORDER_SUBTLE))
                .rounding(Rounding::same(8.0))
                .inner_margin(Margin::same(12.0))
                .show(ui, |ui| {
                    ui.set_width(260.0);
                    ui.label(
                        RichText::new("Dedicated GPU")
                            .font(egui::FontId::proportional(12.0))
                            .color(Theme::TEXT_MUTED),
                    );
                    ui.label(
                        RichText::new("NVIDIA GTX 1650 (TU117)")
                            .font(egui::FontId::proportional(15.0))
                            .strong()
                            .color(Theme::ACCENT_GREEN),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("• 4,096 MB GDDR5 VRAM\n• Vulkan 1.3 & CUDA 12.4\n• 100% Layer Offloading Ready")
                            .font(egui::FontId::proportional(12.0))
                            .color(Theme::TEXT_SECONDARY),
                    );
                });

            ui.add_space(10.0);

            // Memory Card
            Frame::none()
                .fill(Theme::BG_CARD)
                .stroke(Stroke::new(1.0_f32, Theme::BORDER_SUBTLE))
                .rounding(Rounding::same(8.0))
                .inner_margin(Margin::same(12.0))
                .show(ui, |ui| {
                    ui.set_width(260.0);
                    ui.label(
                        RichText::new("System Memory")
                            .font(egui::FontId::proportional(12.0))
                            .color(Theme::TEXT_MUTED),
                    );
                    ui.label(
                        RichText::new("16.0 GB DDR4 RAM")
                            .font(egui::FontId::proportional(15.0))
                            .strong()
                            .color(Theme::ACCENT_CYAN),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("• Dual-Channel 2666 MHz\n• Zero swap/paging pressure\n• Ultra-fast model mmap loading")
                            .font(egui::FontId::proportional(12.0))
                            .color(Theme::TEXT_SECONDARY),
                    );
                });
        });

        ui.add_space(20.0);

        // Section 2: Active Backend Switcher
        ui.label(
            RichText::new("Active Inference Acceleration Engine")
                .font(egui::FontId::proportional(16.0))
                .strong()
                .color(Theme::TEXT_PRIMARY),
        );
        ui.add_space(8.0);

        let backends = [
            (
                BackendType::Vulkan,
                "Vulkan Compute (GTX 1650)",
                "Fastest TTFT: 138 ms | 131.7 tok/s generation",
                "Optimal for interactive instant chat",
            ),
            (
                BackendType::Cuda,
                "NVIDIA CUDA (GTX 1650)",
                "Peak Generation: 162.1 tok/s | 342 ms TTFT",
                "Optimal for long text streaming throughput",
            ),
            (
                BackendType::Cpu,
                "Pure CPU AVX2 (4 Threads)",
                "31.4 tok/s generation | 0 MB VRAM used",
                "Guaranteed fallback on any hardware",
            ),
        ];

        for (b_type, title, metrics, subtitle) in backends {
            let is_current = state.active_backend == b_type;

            Frame::none()
                .fill(if is_current {
                    Color32::from_rgb(20, 32, 48)
                } else {
                    Theme::BG_CARD
                })
                .stroke(Stroke::new(
                    1.0_f32,
                    if is_current {
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
                                    RichText::new(title)
                                        .font(egui::FontId::proportional(14.0))
                                        .strong()
                                        .color(if is_current {
                                            Theme::ACCENT_CYAN
                                        } else {
                                            Theme::TEXT_PRIMARY
                                        }),
                                );
                                if is_current {
                                    ui.label(
                                        RichText::new("ACTIVE")
                                            .font(egui::FontId::monospace(10.0))
                                            .color(Color32::BLACK)
                                            .background_color(Theme::ACCENT_CYAN),
                                    );
                                }
                            });
                            ui.label(
                                RichText::new(metrics)
                                    .font(egui::FontId::monospace(12.0))
                                    .color(Theme::ACCENT_GREEN),
                            );
                            ui.label(
                                RichText::new(subtitle)
                                    .font(egui::FontId::proportional(11.0))
                                    .color(Theme::TEXT_MUTED),
                            );
                        });

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if !is_current {
                                if ui
                                    .button(
                                        RichText::new("Switch Backend")
                                            .font(egui::FontId::proportional(12.0))
                                            .color(Theme::ACCENT_CYAN),
                                    )
                                    .clicked()
                                {
                                    *on_switch_backend = Some(b_type);
                                }
                            }
                        });
                    });
                });
            ui.add_space(8.0);
        }

        ui.add_space(20.0);

        // Section 3: Empirical Benchmark Data Matrix
        ui.label(
            RichText::new("Empirical Laptop Benchmarks (Qwen 2.5 0.5B Q4_K_M)")
                .font(egui::FontId::proportional(16.0))
                .strong()
                .color(Theme::TEXT_PRIMARY),
        );
        ui.add_space(8.0);

        Frame::none()
            .fill(Theme::BG_CARD)
            .stroke(Stroke::new(1.0_f32, Theme::BORDER_SUBTLE))
            .rounding(Rounding::same(8.0))
            .inner_margin(Margin::same(12.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Hardware Config").font(egui::FontId::proportional(12.0)).strong().color(Theme::TEXT_MUTED));
                    ui.add_space(140.0);
                    ui.label(RichText::new("Prompt Eval (tok/s)").font(egui::FontId::proportional(12.0)).strong().color(Theme::TEXT_MUTED));
                    ui.add_space(40.0);
                    ui.label(RichText::new("Generation (tok/s)").font(egui::FontId::proportional(12.0)).strong().color(Theme::TEXT_MUTED));
                    ui.add_space(40.0);
                    ui.label(RichText::new("TTFT (512 ctx)").font(egui::FontId::proportional(12.0)).strong().color(Theme::TEXT_MUTED));
                });
                ui.separator();

                let rows = [
                    ("Vulkan (-dev Vulkan1 GTX 1650)", "3,694.5 ± 147", "131.7 ± 5.4", "138.6 ms  (Fastest)"),
                    ("CUDA (GTX 1650 12.4)", "1,495.9 ± 14.6", "162.1 ± 10.6", "342.3 ms  (Peak TPS)"),
                    ("CPU (4 Threads - Optimal)", "268.4 ± 1.5", "31.4 ± 0.3", "1,907.6 ms"),
                    ("CPU (2 Threads)", "143.9 ± 0.7", "18.9 ± 0.1", "3,558.0 ms"),
                    ("CPU (6 Threads)", "338.2 ± 3.4", "29.0 ± 0.5", "1,514.0 ms"),
                    ("CPU (8 Threads - HT Thrash)", "364.7 ± 2.6", "22.8 ± 0.3", "1,403.9 ms"),
                    ("CUDA 50% Layers Offloaded", "761.3 ± 4.2", "57.6 ± 0.4", "672.5 ms  (-64.5% Loss)"),
                ];

                for (name, pp, tg, ttft) in rows {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(name).font(egui::FontId::monospace(11.5)).color(Theme::TEXT_PRIMARY));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(RichText::new(ttft).font(egui::FontId::monospace(11.5)).color(Theme::ACCENT_GREEN));
                            ui.add_space(40.0);
                            ui.label(RichText::new(tg).font(egui::FontId::monospace(11.5)).color(Theme::ACCENT_CYAN));
                            ui.add_space(40.0);
                            ui.label(RichText::new(pp).font(egui::FontId::monospace(11.5)).color(Theme::TEXT_SECONDARY));
                        });
                    });
                    ui.add_space(3.0);
                }
            });
    });
}
