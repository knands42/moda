use egui::*;

pub const SURFACE: Color32 = Color32::from_rgb(18, 18, 24);
pub const CARD_BG: Color32 = Color32::from_rgb(28, 28, 38);
pub const CARD_HOVER: Color32 = Color32::from_rgb(38, 38, 50);
pub const ACCENT: Color32 = Color32::from_rgb(99, 102, 241);
pub const ACCENT_HOVER: Color32 = Color32::from_rgb(129, 132, 255);
pub const SUCCESS: Color32 = Color32::from_rgb(52, 211, 153);
pub const WARNING: Color32 = Color32::from_rgb(251, 191, 36);
pub const ERROR: Color32 = Color32::from_rgb(248, 113, 113);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(148, 148, 165);
pub const TEXT: Color32 = Color32::from_rgb(226, 226, 241);
pub const BORDER: Color32 = Color32::from_rgb(40, 40, 54);

pub fn configure_visuals() -> egui::Visuals {
    egui::Visuals {
        dark_mode: true,
        override_text_color: Some(TEXT),
        widgets: egui::style::Widgets {
            noninteractive: egui::style::WidgetVisuals {
                bg_fill: CARD_BG,
                weak_bg_fill: SURFACE,
                bg_stroke: egui::Stroke::new(1.0, BORDER),
                corner_radius: egui::CornerRadius::same(6),
                fg_stroke: egui::Stroke::new(1.0, TEXT_MUTED),
                expansion: 0.0,
            },
            inactive: egui::style::WidgetVisuals {
                bg_fill: CARD_BG,
                weak_bg_fill: SURFACE,
                bg_stroke: egui::Stroke::new(1.0, BORDER),
                corner_radius: egui::CornerRadius::same(6),
                fg_stroke: egui::Stroke::new(1.5, TEXT),
                expansion: 0.0,
            },
            hovered: egui::style::WidgetVisuals {
                bg_fill: CARD_HOVER,
                weak_bg_fill: SURFACE,
                bg_stroke: egui::Stroke::new(1.0, ACCENT),
                corner_radius: egui::CornerRadius::same(6),
                fg_stroke: egui::Stroke::new(2.0, TEXT),
                expansion: 1.0,
            },
            active: egui::style::WidgetVisuals {
                bg_fill: ACCENT,
                weak_bg_fill: SURFACE,
                bg_stroke: egui::Stroke::new(1.0, ACCENT),
                corner_radius: egui::CornerRadius::same(6),
                fg_stroke: egui::Stroke::new(2.0, TEXT),
                expansion: 1.0,
            },
            open: egui::style::WidgetVisuals {
                bg_fill: CARD_BG,
                weak_bg_fill: SURFACE,
                bg_stroke: egui::Stroke::new(1.0, ACCENT),
                corner_radius: egui::CornerRadius::same(6),
                fg_stroke: egui::Stroke::new(1.5, TEXT),
                expansion: 0.0,
            },
        },
        selection: egui::style::Selection {
            bg_fill: ACCENT.linear_multiply(0.3),
            stroke: egui::Stroke::new(1.0, ACCENT),
        },
        hyperlink_color: ACCENT,
        faint_bg_color: SURFACE,
        extreme_bg_color: SURFACE,
        code_bg_color: CARD_BG,
        warn_fg_color: WARNING,
        error_fg_color: ERROR,
        ..Default::default()
    }
}
