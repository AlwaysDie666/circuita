//! Small drawing kit: vector icons and the little pills, keycaps and buttons the
//! rest of the interface reuses.

use crate::theme;
use crate::theme::*;
use eframe::egui::{
    self, pos2, vec2, Align2, Color32, FontId, Painter, Pos2, Rect, Response, Sense, Shape, Stroke,
    Ui, Vec2,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Icon {
    Play,
    Pause,
    Stop,
    Step,
    Cursor,
    Wire,
    Move,
    ZoomIn,
    ZoomOut,
    Fit,
    Grid,
    Magnet,
    Rotate,
    Mirror,
    Trash,
    Search,
    Scope,
    Probe,
    Voltmeter,
    Ammeter,
    Close,
    ChevronDown,
    Undo,
    Redo,
    Plus,
    Minus,
    Chip,
    Check,
    Layers,
    Spark,
    Save,
    Folder,
    Info,
}

pub fn paint_icon(painter: &Painter, rect: Rect, icon: Icon, color: Color32, weight: f32) {
    let c = rect.center();
    let s = rect.width().min(rect.height()) * 0.5;
    let w = (1.55 * weight).max(1.0);
    let st = Stroke::new(w, color);
    let line = |a: (f32, f32), b: (f32, f32)| {
        painter.line_segment(
            [pos2(c.x + a.0 * s, c.y + a.1 * s), pos2(c.x + b.0 * s, c.y + b.1 * s)],
            st,
        );
    };
    let poly = |pts: Vec<(f32, f32)>, fill: bool| {
        let v: Vec<Pos2> = pts
            .iter()
            .map(|(x, y)| pos2(c.x + x * s, c.y + y * s))
            .collect();
        if fill {
            painter.add(Shape::convex_polygon(v, color, Stroke::NONE));
        } else {
            painter.add(Shape::line(v, st));
        }
    };
    match icon {
        Icon::Play => {
            poly(vec![(-0.42, -0.62), (0.62, 0.0), (-0.42, 0.62)], true);
        }
        Icon::Pause => {
            painter.rect_filled(
                Rect::from_center_size(pos2(c.x - 0.26 * s, c.y), vec2(0.22 * s, 1.1 * s)),
                1.0,
                color,
            );
            painter.rect_filled(
                Rect::from_center_size(pos2(c.x + 0.26 * s, c.y), vec2(0.22 * s, 1.1 * s)),
                1.0,
                color,
            );
        }
        Icon::Stop => {
            painter.rect_filled(
                Rect::from_center_size(c, Vec2::splat(1.0 * s)),
                2.0,
                color,
            );
        }
        Icon::Step => {
            poly(vec![(-0.55, -0.58), (0.28, 0.0), (-0.55, 0.58)], true);
            line((0.5, -0.6), (0.5, 0.6));
        }
        Icon::Cursor => {
            poly(
                vec![
                    (-0.42, -0.62),
                    (-0.42, 0.5),
                    (-0.12, 0.2),
                    (0.08, 0.62),
                    (0.3, 0.5),
                    (0.1, 0.1),
                    (0.48, 0.05),
                ],
                true,
            );
        }
        Icon::Wire => {
            poly(
                vec![(-0.55, 0.42), (-0.1, 0.42), (-0.1, -0.32), (0.55, -0.32)],
                false,
            );
            painter.circle_filled(pos2(c.x - 0.55 * s, c.y + 0.42 * s), 2.2, color);
            painter.circle_filled(pos2(c.x + 0.55 * s, c.y - 0.32 * s), 2.2, color);
        }
        Icon::Move => {
            line((-0.6, 0.0), (0.6, 0.0));
            line((0.0, -0.6), (0.0, 0.6));
            poly(vec![(0.6, 0.0), (0.35, -0.16), (0.35, 0.16)], true);
            poly(vec![(-0.6, 0.0), (-0.35, -0.16), (-0.35, 0.16)], true);
            poly(vec![(0.0, -0.6), (-0.16, -0.35), (0.16, -0.35)], true);
            poly(vec![(0.0, 0.6), (-0.16, 0.35), (0.16, 0.35)], true);
        }
        Icon::ZoomIn | Icon::ZoomOut => {
            painter.circle_stroke(
                pos2(c.x - 0.12 * s, c.y - 0.12 * s),
                0.4 * s,
                st,
            );
            line((0.16, 0.16), (0.55, 0.55));
            line((-0.3, -0.12), (0.06, -0.12));
            if icon == Icon::ZoomIn {
                line((-0.12, -0.3), (-0.12, 0.06));
            }
        }
        Icon::Fit => {
            for (sx, sy) in [(-1.0f32, -1.0f32), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
                line((sx * 0.55, sy * 0.2), (sx * 0.55, sy * 0.55));
                line((sx * 0.2, sy * 0.55), (sx * 0.55, sy * 0.55));
            }
        }
        Icon::Grid => {
            for gx in [-0.45f32, 0.0, 0.45] {
                for gy in [-0.45f32, 0.0, 0.45] {
                    painter.circle_filled(pos2(c.x + gx * s, c.y + gy * s), 1.35, color);
                }
            }
        }
        Icon::Magnet => {
            let pts: Vec<Pos2> = (0..=16)
                .map(|i| {
                    let a = std::f32::consts::PI * (i as f32 / 16.0);
                    pos2(
                        c.x - a.cos() * 0.42 * s,
                        c.y - 0.1 * s + a.sin() * 0.42 * s,
                    )
                })
                .collect();
            painter.add(Shape::line(pts, st));
            line((-0.42, -0.1), (-0.42, 0.45));
            line((0.42, -0.1), (0.42, 0.45));
        }
        Icon::Rotate => {
            let mut pts = Vec::new();
            for i in 0..=18 {
                let a = -2.3 + (i as f32 / 18.0) * 4.4;
                pts.push(pos2(c.x + a.cos() * 0.46 * s, c.y + a.sin() * 0.46 * s));
            }
            painter.add(Shape::line(pts, st));
            poly(vec![(0.46, -0.55), (0.2, -0.28), (0.56, -0.16)], true);
        }
        Icon::Mirror => {
            painter.add(Shape::convex_polygon(
                vec![
                    pos2(c.x - 0.55 * s, c.y + 0.5 * s),
                    pos2(c.x - 0.55 * s, c.y - 0.5 * s),
                    pos2(c.x - 0.08 * s, c.y),
                ],
                theme::with_alpha(color, 0.75),
                Stroke::NONE,
            ));
            painter.add(Shape::convex_polygon(
                vec![
                    pos2(c.x + 0.55 * s, c.y + 0.5 * s),
                    pos2(c.x + 0.55 * s, c.y - 0.5 * s),
                    pos2(c.x + 0.08 * s, c.y),
                ],
                Color32::TRANSPARENT,
                st,
            ));
            for i in 0..4 {
                let y = c.y - 0.42 * s + i as f32 * 0.28 * s;
                painter.line_segment(
                    [pos2(c.x, y), pos2(c.x, y + 0.12 * s)],
                    Stroke::new(1.0, color),
                );
            }
        }
        Icon::Trash => {
            line((-0.5, -0.38), (0.5, -0.38));
            line((-0.15, -0.55), (0.15, -0.55));
            line((-0.36, -0.38), (-0.3, 0.55));
            line((0.36, -0.38), (0.3, 0.55));
            line((-0.3, 0.55), (0.3, 0.55));
        }
        Icon::Search => {
            painter.circle_stroke(pos2(c.x - 0.1 * s, c.y - 0.1 * s), 0.4 * s, st);
            line((0.18, 0.18), (0.56, 0.56));
        }
        Icon::Scope => {
            painter.rect_stroke(
                Rect::from_center_size(c, Vec2::splat(1.05 * s)),
                2.0,
                Stroke::new(1.2, color),
                egui::StrokeKind::Inside,
            );
            poly(
                vec![
                    (-0.42, 0.2),
                    (-0.18, 0.2),
                    (-0.18, -0.28),
                    (0.06, -0.28),
                    (0.06, 0.2),
                    (0.42, 0.2),
                ],
                false,
            );
        }
        Icon::Probe => {
            line((-0.5, 0.5), (0.28, -0.28));
            line((0.28, -0.28), (0.5, -0.5));
            painter.circle_filled(pos2(c.x - 0.5 * s, c.y + 0.5 * s), 2.6, color);
            painter.circle_stroke(pos2(c.x + 0.1 * s, c.y - 0.1 * s), 0.3 * s, st);
        }
        Icon::Voltmeter | Icon::Ammeter => {
            painter.circle_stroke(c, 0.52 * s, st);
            painter.text(
                c,
                Align2::CENTER_CENTER,
                if icon == Icon::Voltmeter { "V" } else { "A" },
                FontId::new(0.78 * s, egui::FontFamily::Name("semibold".into())),
                color,
            );
        }
        Icon::Close => {
            line((-0.42, -0.42), (0.42, 0.42));
            line((0.42, -0.42), (-0.42, 0.42));
        }
        Icon::ChevronDown => {
            poly(vec![(-0.45, -0.18), (0.0, 0.3), (0.45, -0.18)], false);
        }
        Icon::Undo | Icon::Redo => {
            let dir = if icon == Icon::Undo { 1.0 } else { -1.0 };
            let mut pts = Vec::new();
            for i in 0..=16 {
                let a = 0.4 + (i as f32 / 16.0) * 3.6;
                pts.push(pos2(
                    c.x + dir * a.cos() * 0.42 * s,
                    c.y + 0.12 * s + a.sin() * 0.42 * s,
                ));
            }
            painter.add(Shape::line(pts, st));
            let tip = (dir * 0.42f32, -0.3f32);
            poly(
                vec![
                    (tip.0, tip.1 - 0.22),
                    (tip.0 - dir * 0.2, tip.1 + 0.12),
                    (tip.0 + dir * 0.2, tip.1 + 0.12),
                ],
                true,
            );
        }
        Icon::Plus => {
            line((-0.5, 0.0), (0.5, 0.0));
            line((0.0, -0.5), (0.0, 0.5));
        }
        Icon::Minus => line((-0.5, 0.0), (0.5, 0.0)),
        Icon::Chip => {
            painter.rect_stroke(
                Rect::from_center_size(c, Vec2::splat(0.7 * s)),
                2.0,
                st,
                egui::StrokeKind::Inside,
            );
            for i in 0..3 {
                let off = -0.25 + i as f32 * 0.25;
                line((off, -0.62), (off, -0.36));
                line((off, 0.36), (off, 0.62));
                line((-0.62, off), (-0.36, off));
                line((0.36, off), (0.62, off));
            }
        }
        Icon::Check => poly(vec![(-0.5, 0.02), (-0.12, 0.4), (0.5, -0.35)], false),
        Icon::Layers => {
            for i in 0..3 {
                let dy = -0.3 + i as f32 * 0.3;
                poly(
                    vec![(0.0, dy - 0.2), (0.55, dy), (0.0, dy + 0.2), (-0.55, dy)],
                    false,
                );
            }
        }
        Icon::Spark => {
            poly(
                vec![
                    (0.0, -0.6),
                    (0.16, -0.16),
                    (0.6, 0.0),
                    (0.16, 0.16),
                    (0.0, 0.6),
                    (-0.16, 0.16),
                    (-0.6, 0.0),
                    (-0.16, -0.16),
                ],
                true,
            );
        }
        Icon::Save | Icon::Folder => {
            painter.rect_stroke(
                Rect::from_center_size(c, Vec2::splat(0.85 * s)),
                2.0,
                st,
                egui::StrokeKind::Inside,
            );
            line((-0.3, -0.42), (-0.3, 0.0));
            line((-0.3, 0.0), (0.3, 0.0));
        }
        Icon::Info => {
            painter.circle_stroke(c, 0.5 * s, st);
            painter.circle_filled(pos2(c.x, c.y - 0.26 * s), 1.3, color);
            line((0.0, -0.04), (0.0, 0.34));
        }
    }
}

pub fn icon_button(ui: &mut Ui, icon: Icon, size: f32, active: bool) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(size, size), Sense::click());
    let hovered = resp.hovered();
    let bg = if active {
        theme::with_alpha(ACCENT, 0.16)
    } else if hovered {
        theme::ELEV_2
    } else {
        Color32::TRANSPARENT
    };
    if bg.a() > 0 {
        ui.painter().rect_filled(
            rect.shrink(1.0),
            egui::CornerRadius::same(8),
            bg,
        );
    }
    if active {
        ui.painter().rect_stroke(
            rect.shrink(1.0),
            egui::CornerRadius::same(8),
            Stroke::new(1.0, theme::with_alpha(ACCENT, 0.45)),
            egui::StrokeKind::Inside,
        );
    }
    let color = if active {
        ACCENT_HI
    } else if hovered {
        Color32::WHITE
    } else {
        TEXT_DIM
    };
    paint_icon(
        ui.painter(),
        Rect::from_center_size(rect.center(), Vec2::splat(size * 0.56)),
        icon,
        color,
        1.0,
    );
    resp
}

pub fn pill(ui: &mut Ui, text: &str, color: Color32) {
    let font = FontId::new(9.0, egui::FontFamily::Name("medium".into()));
    let galley = ui.painter().layout_no_wrap(text.to_string(), font.clone(), color);
    let size = vec2(galley.size().x + 12.0, 18.0);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    ui.painter().rect_filled(
        rect,
        egui::CornerRadius::same(9),
        theme::with_alpha(color, 0.12),
    );
    ui.painter().rect_stroke(
        rect,
        egui::CornerRadius::same(9),
        Stroke::new(1.0, theme::with_alpha(color, 0.25)),
        egui::StrokeKind::Inside,
    );
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        text,
        font,
        color,
    );
}

pub fn keycap(ui: &mut Ui, text: &str) {
    let font = FontId::new(9.0, egui::FontFamily::Monospace);
    let galley = ui
        .painter()
        .layout_no_wrap(text.to_string(), font.clone(), TEXT_FAINT);
    let size = vec2(galley.size().x + 8.0, 16.0);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    ui.painter().rect_filled(
        rect,
        egui::CornerRadius::same(4),
        theme::ELEV,
    );
    ui.painter().rect_stroke(
        rect,
        egui::CornerRadius::same(4),
        Stroke::new(1.0, BORDER),
        egui::StrokeKind::Inside,
    );
    ui.painter()
        .text(rect.center(), Align2::CENTER_CENTER, text, font, TEXT_FAINT);
}

pub fn section_label(ui: &mut Ui, text: &str, right: Option<&str>) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(text)
                .font(medium(9.5))
                .color(TEXT_FAINT),
        );
        if let Some(right) = right {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(right)
                        .font(mono(9.0))
                        .color(TEXT_FAINT),
                );
            });
        }
    });
}

pub fn hairline(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
    ui.painter()
        .rect_filled(rect, 0, theme::BORDER_SOFT);
}

pub fn logo(painter: &Painter, rect: Rect) {
    painter.rect_filled(rect, egui::CornerRadius::same(10), ACCENT);
    let c = rect.center();
    let s = rect.width();
    let ink = Color32::from_rgb(0x16, 0x0b, 0x00);
    let st = Stroke::new(1.6, ink);
    painter.line_segment(
        [
            pos2(c.x - 0.24 * s, c.y - 0.12 * s),
            pos2(c.x + 0.02 * s, c.y - 0.12 * s),
        ],
        st,
    );
    painter.line_segment(
        [
            pos2(c.x + 0.02 * s, c.y - 0.12 * s),
            pos2(c.x + 0.02 * s, c.y + 0.14 * s),
        ],
        st,
    );
    painter.line_segment(
        [
            pos2(c.x + 0.02 * s, c.y + 0.14 * s),
            pos2(c.x + 0.26 * s, c.y + 0.14 * s),
        ],
        st,
    );
    painter.circle_filled(pos2(c.x - 0.24 * s, c.y - 0.12 * s), 2.4, ink);
    painter.circle_filled(pos2(c.x + 0.26 * s, c.y + 0.14 * s), 2.4, ink);
    painter.circle_filled(pos2(c.x + 0.02 * s, c.y - 0.02 * s), 1.7, ink);
}
