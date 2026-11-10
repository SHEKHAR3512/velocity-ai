use eframe::egui::{self, Color32, Rounding, Stroke, Visuals};

#[allow(dead_code)]
pub struct Theme;

impl Theme {
    // Background colors
    pub const BG_PRIMARY: Color32 = Color32::from_rgb(17, 18, 22);      // #111216
    pub const BG_SIDEBAR: Color32 = Color32::from_rgb(22, 23, 28);      // #16171C
    pub const BG_CARD: Color32 = Color32::from_rgb(27, 29, 36);         // #1B1D24
    pub const BG_INPUT: Color32 = Color32::from_rgb(24, 26, 32);        // #181A20
    pub const BG_CODE: Color32 = Color32::from_rgb(13, 14, 18);         // #0D0E12
    pub const BG_USER_BUBBLE: Color32 = Color32::from_rgb(29, 36, 51);  // #1D2433

    // Borders
    pub const BORDER_SUBTLE: Color32 = Color32::from_rgb(38, 41, 51);   // #262933
    pub const BORDER_FOCUS: Color32 = Color32::from_rgb(0, 210, 255);    // #00D2FF

    // Text colors
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(243, 244, 246); // #F3F4F6
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(156, 163, 175); // #9CA3AF
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(107, 114, 128);   // #6B7280

    // Accents
    pub const ACCENT_CYAN: Color32 = Color32::from_rgb(0, 210, 255);    // Velocity Electric Cyan
    pub const ACCENT_BLUE: Color32 = Color32::from_rgb(59, 130, 246);   // #3B82F6
    pub const ACCENT_GREEN: Color32 = Color32::from_rgb(16, 185, 129);  // #10B981 (Success / Online)
    pub const ACCENT_AMBER: Color32 = Color32::from_rgb(245, 158, 11);  // #F59E0B (Warning)
    pub const ACCENT_RED: Color32 = Color32::from_rgb(239, 68, 68);     // #EF4444 (Stop / Error)

    pub fn apply(ctx: &egui::Context) {
        let mut visuals = Visuals::dark();
        visuals.panel_fill = Self::BG_PRIMARY;
        visuals.window_fill = Self::BG_PRIMARY;
        visuals.extreme_bg_color = Self::BG_CODE;
        visuals.faint_bg_color = Self::BG_CARD;

        visuals.widgets.noninteractive.bg_fill = Self::BG_CARD;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, Self::BORDER_SUBTLE);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, Self::TEXT_PRIMARY);
        visuals.widgets.noninteractive.rounding = Rounding::same(6.0);

        visuals.widgets.inactive.bg_fill = Self::BG_CARD;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, Self::BORDER_SUBTLE);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, Self::TEXT_PRIMARY);
        visuals.widgets.inactive.rounding = Rounding::same(6.0);

        visuals.widgets.hovered.bg_fill = Color32::from_rgb(35, 38, 48);
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, Self::ACCENT_CYAN);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);
        visuals.widgets.hovered.rounding = Rounding::same(6.0);

        visuals.widgets.active.bg_fill = Color32::from_rgb(42, 46, 58);
        visuals.widgets.active.bg_stroke = Stroke::new(1.5_f32, Self::ACCENT_CYAN);
        visuals.widgets.active.fg_stroke = Stroke::new(1.5_f32, Color32::WHITE);
        visuals.widgets.active.rounding = Rounding::same(6.0);

        visuals.selection.bg_fill = Color32::from_rgba_premultiplied(0, 210, 255, 60);
        visuals.selection.stroke = Stroke::new(1.0_f32, Self::ACCENT_CYAN);

        ctx.set_visuals(visuals);
    }
}
