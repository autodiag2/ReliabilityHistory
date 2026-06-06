use egui::*;

use crate::model::{DaySummary, EventKind};

pub fn draw(
    ui: &mut Ui,
    days: &[DaySummary],
    selected: &mut usize,
) {
    let desired = vec2(ui.available_width(), 340.0);

    let (rect, response) =
        ui.allocate_exact_size(desired, Sense::click());

    let painter = ui.painter();

    painter.rect_filled(rect, 0.0, Color32::WHITE);

    let rows = 5.0;

    let graph_height = 180.0;

    let column_w =
        rect.width() / (days.len().max(1) as f32);

    for r in 0..=10 {
        let y =
            rect.top() + graph_height * (r as f32 / 10.0);

        painter.line_segment(
            [pos2(rect.left(), y), pos2(rect.right(), y)],
            Stroke::new(1.0, Color32::LIGHT_GRAY),
        );
    }

    let mut previous = None;

    for (i, day) in days.iter().enumerate() {
        let x = rect.left() + column_w * i as f32;

        painter.line_segment(
            [
                pos2(x, rect.top()),
                pos2(x, rect.bottom()),
            ],
            Stroke::new(1.0, Color32::LIGHT_GRAY),
        );

        let score_y =
            rect.top() + graph_height * (1.0 - day.score / 10.0);

        let p = pos2(x + column_w / 2.0, score_y);

        if let Some(prev) = previous {
            painter.line_segment(
                [prev, p],
                Stroke::new(2.0, Color32::BLUE),
            );
        }

        painter.circle_filled(
            p,
            3.0,
            Color32::from_rgb(0, 120, 215),
        );

        previous = Some(p);

        for ev in &day.events {
            let row = match ev.kind {
                EventKind::ApplicationFailure => 0,
                EventKind::SystemFailure => 1,
                EventKind::Warning => 2,
                EventKind::Information => 3,
            };

            let cy =
                rect.top() + graph_height + 30.0 + row as f32 * 28.0;

            let color = match ev.kind {
                EventKind::ApplicationFailure => Color32::RED,
                EventKind::SystemFailure => Color32::DARK_RED,
                EventKind::Warning => Color32::YELLOW,
                EventKind::Information => Color32::BLUE,
            };

            painter.circle_filled(
                pos2(x + column_w / 2.0, cy),
                6.0,
                color,
            );
        }

        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                if pos.x >= x && pos.x < x + column_w {
                    *selected = i;
                }
            }
        }
    }

    painter.text(
        pos2(rect.left() + 8.0, rect.top() + 8.0),
        Align2::LEFT_TOP,
        "Stability Index",
        FontId::proportional(14.0),
        Color32::BLACK,
    );
}