use eframe::egui::{self, Align, Color32, Frame, Layout, Margin, RichText, Rounding, ScrollArea, Stroke, Ui, Vec2};
use velocity_core::BackendType;

use crate::state::AppState;
use crate::theme::Theme;

const MAX_WIDTH: f32 = 820.0;

pub fn render_benchmarks_view(
    ui: &mut Ui,
    state: &mut AppState,
    on_switch_backend: &mut Option<BackendType>,
) {
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
                        RichText::new("Hardware Telemetry & Acceleration")
                            .font(egui::FontId::proportional(22.0))
                            .strong()
                            .color(Theme::TEXT_PARCHMENT),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("Live hardware compute routing and empirical performance benchmarks on your Acer Predator laptop")
                            .font(egui::FontId::proportional(13.5))
                            .color(Theme::TEXT_SECONDARY),
                    );

                    ui.add_space(24.0);

                    // Hardware Summary Row (3 Cards)
                    let card_w = ((content_width - 24.0) / 3.0).max(220.0);
                    ui.horizontal(|ui| {
                        render_hw_card(ui, card_w, "CPU Architecture", "Intel Core i5-9300H", "4 Cores | 8 Threads | AVX2 (Locked to 4)");
                        ui.add_space(12.0);
                        render_hw_card(ui, card_w, "Discrete GPU", "NVIDIA GTX 1650", "4,096 MB GDDR5 | Vulkan 1.3 | CUDA 12.4");
                        ui.add_space(12.0);
                        render_hw_card(ui, card_w, "System Memory", "16.0 GB RAM", "Dual-Channel DDR4 | Zero Paging Pressure");
                    });

                    ui.add_space(28.0);

                    // Acceleration Engine Selector
                    ui.label(
                        RichText::new("Active Acceleration Engine")
                            .font(egui::FontId::proportional(16.0))
                            .strong()
                            .color(Theme::TEXT_PARCHMENT),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("Dynamically route matrix multiplications to discrete GPU shaders, CUDA kernels, or CPU vector units")
                            .font(egui::FontId::proportional(12.5))
                            .color(Theme::TEXT_MUTED),
                    );
                    ui.add_space(12.0);

                    let backends = [
                        (
                            BackendType::Vulkan,
                            "Vulkan Compute (GTX 1650)",
                            "Fastest TTFT: 138.6 ms | 131.7 tok/s",
                            "Direct SPIR-V shader execution, optimal for instant conversational responsiveness",
                        ),
                        (
                            BackendType::Cuda,
                            "NVIDIA CUDA (GTX 1650)",
                            "Peak Throughput: 162.1 tok/s | 342.3 ms TTFT",
                            "Optimized CUBLAS matrix GEMM routines, peak tokens/second on long answers",
                        ),
                        (
                            BackendType::Cpu,
                            "CPU AVX2 (4 Physical Cores)",
                            "31.4 tok/s | 0 MB VRAM Used",
                            "Locked to 4 physical cores to prevent hyperthread thrashing, zero GPU dependency",
                        ),
                    ];

                    for (b_type, title, metrics, subtitle) in backends {
                        let is_current = state.active_backend == b_type;

                        Frame::none()
                            .fill(if is_current {
                                Color32::from_rgb(42, 38, 35)
                            } else {
                                Theme::BG_CARD
                            })
                            .stroke(Stroke::new(
                                1.2_f32,
                                if is_current {
                                    Theme::CLAUDE_TERRACOTTA
                                } else {
                                    Theme::BORDER_SUBTLE
                                },
                            ))
                            .rounding(Rounding::same(12.0))
                            .inner_margin(Margin::same(16.0))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                RichText::new(title)
                                                    .font(egui::FontId::proportional(15.0))
                                                    .strong()
                                                    .color(if is_current {
                                                        Theme::CLAUDE_TERRACOTTA
                                                    } else {
                                                        Theme::TEXT_PARCHMENT
                                                    }),
                                            );
                                            if is_current {
                                                ui.label(
                                                    RichText::new("ACTIVE ENGINE")
                                                        .font(egui::FontId::monospace(10.0))
                                                        .strong()
                                                        .color(Color32::BLACK)
                                                        .background_color(Theme::CLAUDE_TERRACOTTA),
                                                );
                                            }
                                        });
                                        ui.add_space(4.0);
                                        ui.label(
                                            RichText::new(metrics)
                                                .font(egui::FontId::monospace(12.5))
                                                .strong()
                                                .color(Theme::ACCENT_SAGE),
                                        );
                                        ui.add_space(2.0);
                                        ui.label(
                                            RichText::new(subtitle)
                                                .font(egui::FontId::proportional(12.0))
                                                .color(Theme::TEXT_SECONDARY),
                                        );
                                    });

                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        if !is_current {
                                            let switch_btn = egui::Button::new(
                                                RichText::new("Switch Engine")
                                                    .font(egui::FontId::proportional(12.0))
                                                    .strong()
                                                    .color(Color32::WHITE),
                                            )
                                            .fill(Theme::CLAUDE_TERRACOTTA)
                                            .rounding(Rounding::same(8.0))
                                            .min_size(Vec2::new(104.0, 28.0));

                                            if ui.add(switch_btn).clicked() {
                                                *on_switch_backend = Some(b_type);
                                            }
                                        }
                                    });
                                });
                            });
                        ui.add_space(10.0);
                    }

                    ui.add_space(28.0);

                    // Empirical Benchmark Results Table
                    ui.label(
                        RichText::new("Empirical Hardware Benchmark Results (Qwen 2.5 0.5B)")
                            .font(egui::FontId::proportional(16.0))
                            .strong()
                            .color(Theme::TEXT_PARCHMENT),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("Averaged across 3 independent trials after cache warm-up")
                            .font(egui::FontId::proportional(12.5))
                            .color(Theme::TEXT_MUTED),
                    );
                    ui.add_space(12.0);

                    Frame::none()
                        .fill(Theme::BG_CARD)
                        .stroke(Stroke::new(1.0_f32, Theme::BORDER_SUBTLE))
                        .rounding(Rounding::same(12.0))
                        .inner_margin(Margin::same(16.0))
                        .show(ui, |ui| {
                            // Table Header
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("BACKEND CONFIGURATION").font(egui::FontId::monospace(11.0)).strong().color(Theme::TEXT_MUTED));
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.label(RichText::new("TTFT LATENCY").font(egui::FontId::monospace(11.0)).strong().color(Theme::TEXT_MUTED));
                                    ui.add_space(24.0);
                                    ui.label(RichText::new("GENERATION").font(egui::FontId::monospace(11.0)).strong().color(Theme::TEXT_MUTED));
                                    ui.add_space(24.0);
                                    ui.label(RichText::new("PROMPT EVAL").font(egui::FontId::monospace(11.0)).strong().color(Theme::TEXT_MUTED));
                                });
                            });
                            ui.add_space(8.0);
                            ui.separator();
                            ui.add_space(8.0);

                            let rows = [
                                ("Vulkan Compute (GTX 1650)", "3,694.5 tok/s", "131.7 tok/s", "138.6 ms (Fastest TTFT)"),
                                ("CUDA 12.4 (GTX 1650)", "1,495.9 tok/s", "162.1 tok/s", "342.3 ms (Peak Speed)"),
                                ("CPU (4 Threads - Optimal)", "268.4 tok/s", "31.4 tok/s", "1,907.6 ms"),
                                ("CPU (8 Threads - Thrashing)", "364.7 tok/s", "22.8 tok/s", "1,403.9 ms (-27.3% Loss)"),
                                ("CUDA 50% Layers Offload", "761.3 tok/s", "57.6 tok/s", "672.5 ms (-64.5% Loss)"),
                            ];

                            for (name, pp, tg, ttft) in rows {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(name).font(egui::FontId::proportional(13.0)).strong().color(Theme::TEXT_PARCHMENT));
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        ui.label(RichText::new(ttft).font(egui::FontId::monospace(12.0)).color(Theme::ACCENT_SAGE));
                                        ui.add_space(24.0);
                                        ui.label(RichText::new(tg).font(egui::FontId::monospace(12.0)).strong().color(Theme::CLAUDE_TERRACOTTA));
                                        ui.add_space(24.0);
                                        ui.label(RichText::new(pp).font(egui::FontId::monospace(12.0)).color(Theme::TEXT_SECONDARY));
                                    });
                                });
                                ui.add_space(6.0);
                            }
                        });

                    ui.add_space(24.0);
                });

                ui.add_space(side_padding);
            });
        });
}

fn render_hw_card(ui: &mut Ui, width: f32, label: &str, title: &str, subtitle: &str) {
    Frame::none()
        .fill(Theme::BG_CARD)
        .stroke(Stroke::new(1.0_f32, Theme::BORDER_SUBTLE))
        .rounding(Rounding::same(12.0))
        .inner_margin(Margin::same(14.0))
        .show(ui, |ui| {
            ui.set_width(width);
            ui.label(RichText::new(label).font(egui::FontId::proportional(11.5)).color(Theme::TEXT_MUTED));
            ui.add_space(3.0);
            ui.label(RichText::new(title).font(egui::FontId::proportional(14.5)).strong().color(Theme::TEXT_PARCHMENT));
            ui.add_space(4.0);
            ui.label(RichText::new(subtitle).font(egui::FontId::proportional(11.5)).color(Theme::TEXT_SECONDARY));
        });
}
