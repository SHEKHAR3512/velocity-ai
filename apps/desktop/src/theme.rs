use eframe::egui::{self, Color32, Rounding, Stroke, Visuals};

pub struct Theme;

impl Theme {
    // Claude Warm Dark Palette
    pub const BG_CANVAS: Color32 = Color32::from_rgb(31, 30, 29);         // #1F1E1D (Warm Charcoal)
    pub const BG_SIDEBAR: Color32 = Color32::from_rgb(25, 24, 23);        // #191817 (Deeper Charcoal)
    pub const BG_CARD: Color32 = Color32::from_rgb(38, 37, 34);           // #262522 (Warm Stone)
    pub const BG_CARD_HOVER: Color32 = Color32::from_rgb(46, 44, 40);     // #2E2C28
    pub const BG_INPUT: Color32 = Color32::from_rgb(38, 37, 34);          // #262522
    pub const BG_USER_BUBBLE: Color32 = Color32::from_rgb(46, 44, 41);    // #2E2C29 (Warm Slate)
    pub const BG_CODE: Color32 = Color32::from_rgb(22, 21, 20);           // #161514 (Deep Code Block)

    // Warm Borders
    pub const BORDER_SUBTLE: Color32 = Color32::from_rgb(53, 51, 47);     // #35332F
    pub const BORDER_WARM: Color32 = Color32::from_rgb(66, 64, 59);       // #42403B
    pub const BORDER_FOCUS: Color32 = Color32::from_rgb(217, 119, 87);    // Claude Terracotta

    // Warm Parchment Typography
    pub const TEXT_PARCHMENT: Color32 = Color32::from_rgb(238, 236, 232); // #EEECE8 (Warm Off-white)
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(175, 172, 166); // #AFACA6 (Warm Grey)
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(120, 118, 112);     // #787670

    // Claude Terracotta / Accents
    pub const CLAUDE_TERRACOTTA: Color32 = Color32::from_rgb(217, 119, 87); // #D97757 (Signature Coral/Terracotta)
    pub const CLAUDE_HOVER: Color32 = Color32::from_rgb(230, 137, 107);     // #E6896B
    pub const ACCENT_SAGE: Color32 = Color32::from_rgb(104, 166, 125);      // #68A67D (Online / Success)
    pub const ACCENT_AMBER: Color32 = Color32::from_rgb(229, 168, 105);     // #E5A869
    pub const ACCENT_RED: Color32 = Color32::from_rgb(217, 83, 79);         // Stop

    pub fn apply(ctx: &egui::Context) {
        let mut visuals = Visuals::dark();
        visuals.panel_fill = Self::BG_CANVAS;
        visuals.window_fill = Self::BG_CANVAS;
        visuals.extreme_bg_color = Self::BG_CODE;
        visuals.faint_bg_color = Self::BG_CARD;

        visuals.widgets.noninteractive.bg_fill = Self::BG_CARD;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, Self::BORDER_SUBTLE);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, Self::TEXT_PARCHMENT);
        visuals.widgets.noninteractive.rounding = Rounding::same(8.0);

        visuals.widgets.inactive.bg_fill = Self::BG_CARD;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, Self::BORDER_SUBTLE);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, Self::TEXT_PARCHMENT);
        visuals.widgets.inactive.rounding = Rounding::same(8.0);

        visuals.widgets.hovered.bg_fill = Self::BG_CARD_HOVER;
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, Self::CLAUDE_TERRACOTTA);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, Self::TEXT_PARCHMENT);
        visuals.widgets.hovered.rounding = Rounding::same(8.0);

        visuals.widgets.active.bg_fill = Color32::from_rgb(52, 50, 46);
        visuals.widgets.active.bg_stroke = Stroke::new(1.5_f32, Self::CLAUDE_TERRACOTTA);
        visuals.widgets.active.fg_stroke = Stroke::new(1.5_f32, Color32::WHITE);
        visuals.widgets.active.rounding = Rounding::same(8.0);

        visuals.selection.bg_fill = Color32::from_rgba_premultiplied(217, 119, 87, 60);
        visuals.selection.stroke = Stroke::new(1.0_f32, Self::CLAUDE_TERRACOTTA);

        ctx.set_visuals(visuals);
    }
}
