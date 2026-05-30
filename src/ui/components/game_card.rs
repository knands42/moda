use super::super::style;

pub fn render(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    is_hovered: bool,
    icon: &str,
    name: &str,
    description: &str,
) {
    let bg = if is_hovered {
        style::CARD_HOVER
    } else {
        style::CARD_BG
    };

    ui.painter().rect_filled(rect, 12.0, bg);
    ui.painter().rect_stroke(
        rect,
        egui::CornerRadius::same(12),
        egui::Stroke::new(1.0, style::BORDER),
        egui::StrokeKind::Outside,
    );

    if is_hovered {
        ui.painter().rect_stroke(
            rect,
            egui::CornerRadius::same(12),
            egui::Stroke::new(1.5, style::ACCENT),
            egui::StrokeKind::Outside,
        );
    }

    ui.painter().text(
        egui::Pos2::new(rect.center().x, rect.top() + 40.0),
        egui::Align2::CENTER_CENTER,
        icon,
        egui::FontId::proportional(36.0),
        style::TEXT,
    );

    ui.painter().text(
        egui::Pos2::new(rect.center().x, rect.top() + 80.0),
        egui::Align2::CENTER_CENTER,
        name,
        egui::FontId::proportional(18.0),
        style::TEXT,
    );

    let wrap_width = rect.width() - 24.0;
    let mut job = egui::text::LayoutJob::default();
    job.append(
        description,
        0.0,
        egui::text::TextFormat {
            font_id: egui::FontId::proportional(12.0),
            color: style::TEXT_MUTED,
            ..Default::default()
        },
    );
    job.wrap = egui::text::TextWrapping {
        max_width: wrap_width.max(0.0),
        max_rows: 4,
        break_anywhere: false,
        overflow_character: Some('…'),
    };
    let galley = ui.painter().layout_job(job);
    let pos = egui::Pos2::new(
        rect.center().x - galley.rect.width() / 2.0,
        rect.top() + 140.0,
    );
    ui.painter().galley(pos, galley, egui::Color32::PLACEHOLDER);
}
