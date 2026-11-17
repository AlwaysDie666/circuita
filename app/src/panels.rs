//! Chrome around the canvas: the top transport bar, the component library, the
//! inspector and the status strip.

use crate::app::{App, Selection, Tool, LEFT_W, RIGHT_W, STATUS_H, TOP_H};
use crate::model::{Category, Kind, PinRef, PinRole};
use crate::sim::Level;
use crate::symbols::{self, DrawOpts};
use crate::theme;
use crate::theme::*;
use crate::ui_kit::{self, Icon};
use eframe::egui::{
    self, pos2, vec2, Align2, Color32, CursorIcon, FontId, Margin, Pos2, Rect, Sense, Stroke,
    StrokeKind, Ui, Vec2,
};

pub fn top_bar(app: &mut App, ctx: &egui::Context) {
    egui::TopBottomPanel::top("topbar")
        .exact_height(TOP_H)
        .frame(egui::Frame::new().fill(PANEL))
        .show(ctx, |ui| {
            let rect = ui.max_rect();
            let cy = rect.center().y;
            let p = ui.painter().clone();

            // Brand.
            let logo_rect = Rect::from_center_size(pos2(rect.left() + 32.0, cy), Vec2::splat(30.0));
            ui_kit::logo(&p, logo_rect);
            p.text(
                pos2(logo_rect.right() + 12.0, cy - 7.0),
                Align2::LEFT_CENTER,
                "CIRCUITA",
                FontId::new(14.0, egui::FontFamily::Name("bold".into())),
                TEXT,
            );
            p.text(
                pos2(logo_rect.right() + 12.5, cy + 8.0),
                Align2::LEFT_CENTER,
                "SCHEMATIC STUDIO",
                FontId::new(8.0, egui::FontFamily::Name("medium".into())),
                TEXT_FAINT,
            );

            let mut x = logo_rect.right() + 116.0;
            p.line_segment(
                [pos2(x, cy - 14.0), pos2(x, cy + 14.0)],
                Stroke::new(1.0, BORDER),
            );
            x += 18.0;

            // History.
            let undo = ui.interact(
                Rect::from_center_size(pos2(x + 15.0, cy), Vec2::splat(30.0)),
                ui.id().with("undo"),
                Sense::click(),
            );
            let hl = undo.hovered();
            if hl {
                p.rect_filled(undo.rect, egui::CornerRadius::same(8), ELEV_2);
            }
            ui_kit::paint_icon(
                &p,
                Rect::from_center_size(undo.rect.center(), Vec2::splat(18.0)),
                Icon::Undo,
                if hl { TEXT } else { TEXT_FAINT },
                1.0,
            );
            if undo.clicked() {
                app.undo();
            }
            let undo = undo.on_hover_text("Undo  (Ctrl+Z)");
            let _ = undo;
            x += 38.0;
            let redo = ui.interact(
                Rect::from_center_size(pos2(x + 15.0, cy), Vec2::splat(30.0)),
                ui.id().with("redo"),
                Sense::click(),
            );
            let hl = redo.hovered();
            if hl {
                p.rect_filled(redo.rect, egui::CornerRadius::same(8), ELEV_2);
            }
            ui_kit::paint_icon(
                &p,
                Rect::from_center_size(redo.rect.center(), Vec2::splat(18.0)),
                Icon::Redo,
                if hl { TEXT } else { TEXT_FAINT },
                1.0,
            );
            if redo.clicked() {
                app.redo();
            }
            let _ = redo.on_hover_text("Redo  (Ctrl+Shift+Z)");

            // Transport, centred.
            let tw = 300.0f32;
            let tr = Rect::from_center_size(
                pos2(rect.center().x, cy),
                vec2(tw, 38.0),
            );
            p.rect(
                tr,
                egui::CornerRadius::same(10),
                Color32::from_rgb(0x19, 0x19, 0x19),
                Stroke::new(1.0, BORDER),
                StrokeKind::Inside,
            );
            let by = tr.center().y;
            let stop_r = Rect::from_center_size(pos2(tr.left() + 24.0, by), Vec2::splat(30.0));
            let stop = ui.interact(stop_r, ui.id().with("stop"), Sense::click());
            if stop.hovered() {
                p.rect_filled(stop_r, egui::CornerRadius::same(7), ELEV_2);
            }
            ui_kit::paint_icon(
                &p,
                Rect::from_center_size(stop_r.center(), Vec2::splat(16.0)),
                Icon::Stop,
                if stop.hovered() { TEXT } else { TEXT_DIM },
                1.0,
            );
            if stop.clicked() {
                app.stop();
            }
            let _ = stop.on_hover_text("Stop and reset  (Space toggles play)");

            let play_r = Rect::from_center_size(pos2(tr.left() + 62.0, by), Vec2::splat(34.0));
            let play = ui.interact(play_r, ui.id().with("play"), Sense::click());
            p.circle_filled(
                play_r.center(),
                17.0,
                if play.hovered() { ACCENT_HI } else { ACCENT },
            );
            ui_kit::paint_icon(
                &p,
                Rect::from_center_size(play_r.center() + vec2(1.0, 0.0), Vec2::splat(20.0)),
                if app.running { Icon::Pause } else { Icon::Play },
                Color32::from_rgb(0x16, 0x0b, 0x00),
                1.1,
            );
            if play.clicked() {
                app.play_pause();
            }
            let _ = play.on_hover_text(if app.running { "Pause  (Space)" } else { "Play  (Space)" });

            let step_r = Rect::from_center_size(pos2(tr.left() + 100.0, by), Vec2::splat(30.0));
            let stepb = ui.interact(step_r, ui.id().with("step"), Sense::click());
            if stepb.hovered() {
                p.rect_filled(step_r, egui::CornerRadius::same(7), ELEV_2);
            }
            ui_kit::paint_icon(
                &p,
                Rect::from_center_size(step_r.center(), Vec2::splat(17.0)),
                Icon::Step,
                if stepb.hovered() { TEXT } else { TEXT_DIM },
                1.0,
            );
            if stepb.clicked() {
                app.step();
            }
            let _ = stepb.on_hover_text("Advance one step");

            p.line_segment(
                [pos2(tr.left() + 122.0, by - 12.0), pos2(tr.left() + 122.0, by + 12.0)],
                Stroke::new(1.0, BORDER),
            );
            // Running indicator.
            let dot = pos2(tr.left() + 140.0, by);
            let status_col = if app.running {
                OK
            } else if app.time > 0.0 {
                MID
            } else {
                TEXT_FAINT
            };
            if app.running {
                p.circle_filled(dot, 4.5, theme::with_alpha(status_col, 0.22));
            }
            p.circle_filled(dot, 2.6, status_col);
            p.text(
                pos2(dot.x + 10.0, by - 6.0),
                Align2::LEFT_CENTER,
                format!("{:.3} s", app.time),
                FontId::new(11.5, egui::FontFamily::Monospace),
                TEXT,
            );
            p.text(
                pos2(dot.x + 10.0, by + 7.5),
                Align2::LEFT_CENTER,
                if app.running { "RUNNING" } else { "PAUSED" },
                FontId::new(7.5, egui::FontFamily::Name("medium".into())),
                status_col,
            );

            let speed_r = Rect::from_center_size(pos2(tr.right() - 36.0, by), vec2(56.0, 26.0));
            let speed = ui.interact(speed_r, ui.id().with("speed"), Sense::click());
            p.rect(
                speed_r,
                egui::CornerRadius::same(7),
                if speed.hovered() { ELEV_2 } else { ELEV },
                Stroke::new(1.0, BORDER),
                StrokeKind::Inside,
            );
            p.text(
                speed_r.center(),
                Align2::CENTER_CENTER,
                format!("{:.1}×", app.speed),
                FontId::new(10.5, egui::FontFamily::Name("medium".into())),
                TEXT_DIM,
            );
            if speed.clicked() {
                app.speed = match app.speed {
                    s if s < 0.75 => 1.0,
                    s if s < 1.5 => 2.0,
                    s if s < 2.5 => 4.0,
                    _ => 0.5,
                };
            }
            let _ = speed.on_hover_text("Simulation speed");

            // Right cluster: instruments and zoom.
            let mut rx = rect.right() - 14.0;
            let mut instrument_btn = |ui: &mut Ui, p: &egui::Painter, rx: &mut f32, icon: Icon, on: bool, name: &str| {
                *rx -= 30.0;
                let r = Rect::from_center_size(pos2(*rx + 15.0, cy), Vec2::splat(28.0));
                let resp = ui.interact(r, ui.id().with(("inst", name)), Sense::click());
                if on {
                    p.rect_filled(r, egui::CornerRadius::same(7), theme::with_alpha(ACCENT, 0.14));
                } else if resp.hovered() {
                    p.rect_filled(r, egui::CornerRadius::same(7), ELEV_2);
                }
                ui_kit::paint_icon(
                    p,
                    Rect::from_center_size(r.center(), Vec2::splat(16.0)),
                    icon,
                    if on { ACCENT_HI } else if resp.hovered() { TEXT } else { TEXT_FAINT },
                    1.0,
                );
                *rx -= 4.0;
                resp
            };
            let scope_on = app.instruments.scope_open;
            let probe_on = app.instruments.probe_open;
            let amp_on = app.instruments.amp_open;
            let volt_on = app.instruments.volt_open;
            let r4 = instrument_btn(ui, &p, &mut rx, Icon::Scope, scope_on, "scope");
            let r3 = instrument_btn(ui, &p, &mut rx, Icon::Probe, probe_on, "probe");
            let r2 = instrument_btn(ui, &p, &mut rx, Icon::Ammeter, amp_on, "amp");
            let r1 = instrument_btn(ui, &p, &mut rx, Icon::Voltmeter, volt_on, "volt");
            if r1.clicked() {
                app.instruments.toggle(0);
            }
            if r2.clicked() {
                app.instruments.toggle(1);
            }
            if r3.clicked() {
                app.instruments.toggle(2);
            }
            if r4.clicked() {
                app.instruments.toggle(3);
            }
            let _ = r1.on_hover_text("Voltmeter");
            let _ = r2.on_hover_text("Ammeter");
            let _ = r3.on_hover_text("Logic probe");
            let _ = r4.on_hover_text("Oscilloscope");

            rx -= 6.0;
            p.line_segment(
                [pos2(rx, cy - 12.0), pos2(rx, cy + 12.0)],
                Stroke::new(1.0, BORDER),
            );
            rx -= 10.0;

            // Zoom controls.
            let fit_r = Rect::from_center_size(pos2(rx - 13.0, cy), Vec2::splat(28.0));
            let fit = ui.interact(fit_r, ui.id().with("fit"), Sense::click());
            if fit.hovered() {
                p.rect_filled(fit_r, egui::CornerRadius::same(7), ELEV_2);
            }
            ui_kit::paint_icon(
                &p,
                Rect::from_center_size(fit_r.center(), Vec2::splat(16.0)),
                Icon::Fit,
                if fit.hovered() { TEXT } else { TEXT_DIM },
                1.0,
            );
            if fit.clicked() {
                app.fit_view(app.canvas_rect);
            }
            let _ = fit.on_hover_text("Fit circuit  (F)");
            rx -= 34.0;
            p.text(
                pos2(rx - 30.0, cy),
                Align2::CENTER_CENTER,
                format!("{}%", (app.zoom * 100.0).round() as i32),
                FontId::new(10.0, egui::FontFamily::Monospace),
                TEXT_DIM,
            );
            let zin = ui.interact(
                Rect::from_center_size(pos2(rx - 8.0, cy), Vec2::splat(26.0)),
                ui.id().with("zin"),
                Sense::click(),
            );
            if zin.clicked() {
                app.zoom = (app.zoom * 1.15).min(3.0);
            }
            let _ = zin.on_hover_text("Zoom in");
            rx -= 26.0;
            let zout = ui.interact(
                Rect::from_center_size(pos2(rx - 8.0, cy), Vec2::splat(26.0)),
                ui.id().with("zout"),
                Sense::click(),
            );
            if zout.clicked() {
                app.zoom = (app.zoom / 1.15).max(0.25);
            }
            let _ = zout.on_hover_text("Zoom out");
            let _ = ctx;

            p.line_segment(
                [pos2(rect.left(), rect.bottom() - 0.5), pos2(rect.right(), rect.bottom() - 0.5)],
                Stroke::new(1.0, BORDER_SOFT),
            );
        });
}

pub fn status_bar(app: &mut App, ctx: &egui::Context) {
    egui::TopBottomPanel::bottom("status")
        .exact_height(STATUS_H)
        .frame(egui::Frame::new().fill(PANEL))
        .show(ctx, |ui| {
            let rect = ui.max_rect();
            let p = ui.painter().clone();
            p.line_segment(
                [
                    pos2(rect.left(), rect.top() + 0.5),
                    pos2(rect.right(), rect.top() + 0.5),
                ],
                Stroke::new(1.0, BORDER_SOFT),
            );
            let mut x = rect.left() + 14.0;
            let cy = rect.center().y;

            let (dot_col, label) = if let Some((msg, _)) = &app.toast {
                (ACCENT, msg.clone())
            } else if app.result.short {
                (HIGH, "Short circuit detected".to_string())
            } else {
                (OK, "Ready".to_string())
            };
            p.circle_filled(pos2(x + 3.0, cy), 3.0, dot_col);
            x += 12.0;
            p.text(
                pos2(x, cy),
                Align2::LEFT_CENTER,
                label,
                FontId::new(10.0, egui::FontFamily::Name("medium".into())),
                TEXT_DIM,
            );

            // Keyboard hints, right aligned before the coordinates.
            let hints: [(&str, &str); 5] = [
                ("R", "rotate"),
                ("M", "mirror"),
                ("W", "wire"),
                ("Del", "delete"),
                ("Space", "run"),
            ];
            let mut hx = rect.left() + 220.0;
            for (key, what) in hints {
                let key_w = key.len() as f32 * 5.6 + 8.0;
                if hx + key_w + 44.0 > rect.right() - 250.0 {
                    break;
                }
                let (r, _) = ui.allocate_exact_size(vec2(key_w, 15.0), Sense::hover());
                let key_rect = Rect::from_min_size(pos2(hx, cy - 7.5), r.size());
                p.rect_filled(key_rect, egui::CornerRadius::same(3), ELEV);
                p.rect_stroke(
                    key_rect,
                    egui::CornerRadius::same(3),
                    Stroke::new(1.0, BORDER),
                    StrokeKind::Inside,
                );
                p.text(
                    key_rect.center(),
                    Align2::CENTER_CENTER,
                    key,
                    FontId::new(8.5, egui::FontFamily::Monospace),
                    TEXT_FAINT,
                );
                p.text(
                    pos2(key_rect.right() + 4.0, cy),
                    Align2::LEFT_CENTER,
                    what,
                    FontId::new(9.0, egui::FontFamily::Proportional),
                    TEXT_FAINT,
                );
                hx = key_rect.right() + 6.0 + what.len() as f32 * 4.4 + 12.0;
            }

            let cursor = app
                .canvas_rect
                .contains(ctx.pointer_hover_pos().unwrap_or(Pos2::ZERO))
                .then(|| ctx.pointer_hover_pos())
                .flatten()
                .map(|p| {
                    let w = app.canvas_rect.center()
                        + (p - app.canvas_rect.center()) / app.zoom
                        + (app.view_center.to_vec2() - app.canvas_rect.center().to_vec2()) * 0.0;
                    let _ = w;
                    let world = app.view_center + (p - app.canvas_rect.center()) / app.zoom;
                    (world.x, world.y)
                });
            let right_text = match cursor {
                Some((x, y)) => format!("X {:.0}   Y {:.0}", x, y),
                None => format!("{} components  ·  {} wires", app.project.comps.len(), app.project.wires.len()),
            };
            p.text(
                pos2(rect.right() - 14.0, cy),
                Align2::RIGHT_CENTER,
                right_text,
                FontId::new(9.5, egui::FontFamily::Monospace),
                TEXT_FAINT,
            );
            p.text(
                pos2(rect.right() - 168.0, cy),
                Align2::RIGHT_CENTER,
                format!("GRID {:.0}", crate::model::GRID),
                FontId::new(9.5, egui::FontFamily::Monospace),
                if app.snap { ACCENT } else { TEXT_FAINT },
            );
        });
}

pub fn library(app: &mut App, ctx: &egui::Context) {
    egui::SidePanel::left("library")
        .exact_width(LEFT_W)
        .resizable(false)
        .frame(
            egui::Frame::new()
                .fill(PANEL)
                .inner_margin(Margin::symmetric(14, 12)),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Component library")
                        .font(display(14.0))
                        .color(TEXT),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui_kit::pill(ui, &format!("{} parts", Kind::ALL.len()), TEXT_FAINT);
                });
            });
            ui.add_space(10.0);

            // Search field.
            let (search_rect, _) =
                ui.allocate_exact_size(vec2(ui.available_width(), 36.0), Sense::hover());
            ui.painter().rect(
                search_rect,
                egui::CornerRadius::same(9),
                Color32::from_rgb(0x0f, 0x0f, 0x0f),
                Stroke::new(1.0, BORDER),
                StrokeKind::Inside,
            );
            let icon_rect = Rect::from_center_size(
                pos2(search_rect.left() + 16.0, search_rect.center().y),
                Vec2::splat(14.0),
            );
            ui_kit::paint_icon(ui.painter(), icon_rect, Icon::Search, TEXT_FAINT, 1.0);
            let edit_rect = Rect::from_min_max(
                pos2(search_rect.left() + 30.0, search_rect.top() + 6.0),
                pos2(search_rect.right() - 8.0, search_rect.bottom() - 6.0),
            );
            let mut child = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(edit_rect)
                    .layout(egui::Layout::left_to_right(egui::Align::Center)),
            );
            child.add(
                egui::TextEdit::singleline(&mut app.search)
                    .hint_text(
                        egui::RichText::new("Search resistor, LED, gate…")
                            .font(body(11.5))
                            .color(TEXT_FAINT),
                    )
                    .desired_width(edit_rect.width())
                    .frame(false)
                    .font(body(11.5)),
            );
            ui.add_space(10.0);

            // Category chips.
            let mut chips: Vec<(Category, usize)> = Vec::new();
            for cat in Category::ALL {
                chips.push((
                    cat,
                    Kind::ALL.iter().filter(|k| k.spec().category == cat).count(),
                ));
            }
            ui.horizontal_wrapped(|ui| {
                if chip(ui, "All", Kind::ALL.len(), app.category.is_none()).clicked() {
                    app.category = None;
                }
                for (cat, count) in chips {
                    let active = app.category == Some(cat);
                    if chip(ui, cat.label(), count, active).clicked() {
                        app.category = if active { None } else { Some(cat) };
                    }
                }
            });
            ui.add_space(6.0);

            // List.
            let query = app.search.to_lowercase();
            let items: Vec<Kind> = Kind::ALL
                .iter()
                .copied()
                .filter(|k| app.category.map(|c| k.spec().category == c).unwrap_or(true))
                .filter(|k| {
                    if query.is_empty() {
                        return true;
                    }
                    let s = k.spec();
                    s.name.to_lowercase().contains(&query)
                        || s.short.to_lowercase().contains(&query)
                        || s.desc.to_lowercase().contains(&query)
                        || s.value.to_lowercase().contains(&query)
                })
                .collect();

            let preview_h = 132.0;
            let list_h = (ui.available_height() - preview_h - 18.0).max(120.0);
            let mut clicked: Option<Kind> = None;
            let mut hovered_kind: Option<Kind> = None;
            egui::ScrollArea::vertical()
                .max_height(list_h)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing = vec2(0.0, 2.0);
                    if items.is_empty() {
                        ui.add_space(20.0);
                        ui.vertical_centered(|ui| {
                            ui.label(
                                egui::RichText::new("No parts match that search")
                                    .font(body(11.0))
                                    .color(TEXT_FAINT),
                            );
                        });
                    }
                    for kind in &items {
                        let (row, resp) = ui.allocate_exact_size(
                            vec2(ui.available_width(), 46.0),
                            Sense::click(),
                        );
                        if resp.hovered() {
                            hovered_kind = Some(*kind);
                        }
                        let armed = app.armed == Some(*kind);
                        let bg = if armed {
                            theme::with_alpha(ACCENT, 0.10)
                        } else if resp.hovered() {
                            ELEV
                        } else {
                            Color32::TRANSPARENT
                        };
                        if bg.a() > 0 {
                            ui.painter().rect_filled(
                                row.shrink2(vec2(0.0, 1.0)),
                                egui::CornerRadius::same(8),
                                bg,
                            );
                        }
                        if armed {
                            ui.painter().rect_stroke(
                                row.shrink2(vec2(0.0, 1.0)),
                                egui::CornerRadius::same(8),
                                Stroke::new(1.0, theme::with_alpha(ACCENT, 0.4)),
                                StrokeKind::Inside,
                            );
                        }
                        let thumb = Rect::from_min_size(
                            pos2(row.left() + 6.0, row.top() + 5.0),
                            vec2(48.0, 36.0),
                        );
                        ui.painter().rect(
                            thumb,
                            egui::CornerRadius::same(7),
                            Color32::from_rgb(0x0d, 0x0d, 0x0d),
                            Stroke::new(1.0, BORDER_SOFT),
                            StrokeKind::Inside,
                        );
                        draw_symbol_preview(ui.painter(), thumb.shrink(2.0), *kind);
                        let tx = thumb.right() + 10.0;
                        ui.painter().text(
                            pos2(tx, row.center().y - 7.0),
                            Align2::LEFT_CENTER,
                            kind.spec().name,
                            FontId::new(12.0, egui::FontFamily::Name("medium".into())),
                            if armed || resp.hovered() { TEXT } else { TEXT_DIM },
                        );
                        ui.painter().text(
                            pos2(tx, row.center().y + 8.0),
                            Align2::LEFT_CENTER,
                            if *kind == Kind::AcSource {
                                "Sine supply"
                            } else if *kind == Kind::Vcc {
                                "Power rail"
                            } else {
                                kind.spec().category.label()
                            },
                            FontId::new(9.0, egui::FontFamily::Proportional),
                            TEXT_FAINT,
                        );
                        // Add affordance.
                        if resp.hovered() || armed {
                            let plus = Rect::from_center_size(
                                pos2(row.right() - 18.0, row.center().y),
                                Vec2::splat(18.0),
                            );
                            ui.painter().circle_filled(
                                plus.center(),
                                9.0,
                                theme::with_alpha(ACCENT, if armed { 0.9 } else { 0.15 }),
                            );
                            ui_kit::paint_icon(
                                ui.painter(),
                                Rect::from_center_size(plus.center(), Vec2::splat(10.0)),
                                Icon::Plus,
                                if armed { Color32::from_rgb(0x16, 0x0b, 0x00) } else { ACCENT },
                                1.0,
                            );
                        }
                        if resp.clicked() {
                            clicked = Some(*kind);
                        }
                        resp.on_hover_cursor(CursorIcon::PointingHand);
                    }
                });

            if let Some(kind) = clicked {
                if app.armed == Some(kind) {
                    app.armed = None;
                } else {
                    app.armed = Some(kind);
                    app.tool = Tool::Select;
                    app.lib_preview = Some(kind);
                    app.toast(
                        format!("{} armed — click the canvas", kind.spec().name),
                        app.clock,
                    );
                }
            }
            if let Some(kind) = hovered_kind {
                app.lib_hover = Some(kind);
            }

            ui.add_space(8.0);
            // Preview card.
            let preview_kind = app
                .lib_hover
                .or(app.armed)
                .or(app.lib_preview)
                .unwrap_or(Kind::Resistor);
            let spec = preview_kind.spec();
            let (card, _) = ui.allocate_exact_size(
                vec2(ui.available_width(), preview_h),
                Sense::hover(),
            );
            ui.painter().rect(
                card,
                egui::CornerRadius::same(10),
                Color32::from_rgb(0x11, 0x11, 0x11),
                Stroke::new(1.0, BORDER),
                StrokeKind::Inside,
            );
            let box_rect = Rect::from_min_size(
                pos2(card.left() + 12.0, card.top() + 12.0),
                vec2(96.0, preview_h - 24.0),
            );
            ui.painter().rect(
                box_rect,
                egui::CornerRadius::same(8),
                Color32::from_rgb(0x0b, 0x0b, 0x0b),
                Stroke::new(1.0, BORDER_SOFT),
                StrokeKind::Inside,
            );
            draw_symbol_preview(
                ui.painter(),
                box_rect.shrink2(vec2(8.0, 14.0)),
                preview_kind,
            );
            let tx = box_rect.right() + 12.0;
            ui.painter().text(
                pos2(tx, card.top() + 24.0),
                Align2::LEFT_CENTER,
                spec.name,
                FontId::new(12.5, egui::FontFamily::Name("semibold".into())),
                TEXT,
            );
            ui.painter().text(
                pos2(tx, card.top() + 42.0),
                Align2::LEFT_CENTER,
                format!("{}  ·  {}", spec.short, spec.category.label()),
                FontId::new(9.0, egui::FontFamily::Monospace),
                ACCENT,
            );
            let desc = ui.painter().layout(
                spec.desc.to_string(),
                FontId::new(10.0, egui::FontFamily::Proportional),
                TEXT_DIM,
                (card.right() - tx - 12.0).max(40.0),
            );
            ui.painter().galley(
                pos2(tx, card.top() + 52.0),
                desc,
                TEXT_DIM,
            );
            ui.painter().text(
                pos2(tx, card.bottom() - 18.0),
                Align2::LEFT_CENTER,
                if app.armed == Some(preview_kind) {
                    "Click the canvas to place"
                } else {
                    "Click a part, then the canvas"
                },
                FontId::new(9.5, egui::FontFamily::Proportional),
                TEXT_FAINT,
            );
        });
}

fn chip(
    ui: &mut Ui,
    label: &str,
    count: usize,
    active: bool,
) -> egui::Response {
    let font = FontId::new(9.5, egui::FontFamily::Name("medium".into()));
    let text = format!("{label} {count}");
    let galley = ui
        .painter()
        .layout_no_wrap(text.clone(), font.clone(), TEXT);
    let size = vec2(galley.size().x + 16.0, 23.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let bg = if active {
        ACCENT
    } else if resp.hovered() {
        ELEV_2
    } else {
        Color32::from_rgb(0x18, 0x18, 0x18)
    };
    ui.painter()
        .rect_filled(rect, egui::CornerRadius::same(11), bg);
    if !active {
        ui.painter().rect_stroke(
            rect,
            egui::CornerRadius::same(11),
            Stroke::new(1.0, BORDER_SOFT),
            StrokeKind::Inside,
        );
    }
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        text,
        font,
        if active { Color32::from_rgb(0x16, 0x0b, 0x00) } else { TEXT_DIM },
    );
    resp.on_hover_cursor(CursorIcon::PointingHand)
}

pub fn draw_symbol_preview(painter: &egui::Painter, rect: Rect, kind: Kind) {
    let spec = kind.spec();
    let mut comp = crate::model::Component::new(0, kind, Pos2::ZERO, String::new());
    comp.value = spec.value.to_string();
    let mut ex = spec.size.0 * 0.5 + 4.0;
    let mut ey = spec.size.1 * 0.5 + 4.0;
    for pin in spec.pins {
        ex = ex.max(pin.at.0.abs() + 4.0);
        ey = ey.max(pin.at.1.abs() + 10.0);
    }
    let zoom = (rect.width() / (2.0 * ex)).min(rect.height() / (2.0 * ey));
    let painter = painter.with_clip_rect(rect);
    let opts = DrawOpts {
        ghost: false,
        labels: false,
        pin_colors: None,
        ..Default::default()
    };
    // Pin colours omitted so previews stay neutral.
    symbols::draw(&painter, &comp, rect.center(), zoom.max(0.2), &opts);
}

pub fn inspector(app: &mut App, ctx: &egui::Context) {
    egui::SidePanel::right("inspector")
        .exact_width(RIGHT_W)
        .resizable(false)
        .frame(
            egui::Frame::new()
                .fill(PANEL)
                .inner_margin(Margin::symmetric(14, 12)),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Inspector")
                        .font(display(14.0))
                        .color(TEXT),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui_kit::pill(
                        ui,
                        if app.result.short { "SHORT" } else { "LIVE" },
                        if app.result.short { HIGH } else { OK },
                    );
                });
            });
            ui.add_space(10.0);
            // Segmented tabs.
            let (tab_rect, _) =
                ui.allocate_exact_size(vec2(ui.available_width(), 30.0), Sense::hover());
            ui.painter().rect_filled(tab_rect, egui::CornerRadius::same(8), ELEV);
            for i in 0..2 {
                let r = Rect::from_min_size(
                    pos2(tab_rect.left() + i as f32 * tab_rect.width() / 2.0, tab_rect.top()),
                    vec2(tab_rect.width() / 2.0, tab_rect.height()),
                );
                let active = app.inspector_tab == i;
                if active {
                    ui.painter().rect_filled(
                        r.shrink(2.0),
                        egui::CornerRadius::same(6),
                        Color32::from_rgb(0x25, 0x25, 0x25),
                    );
                }
                let resp = ui.interact(
                    r,
                    ui.id().with(("tab", i)),
                    Sense::click(),
                );
                ui.painter().text(
                    r.center(),
                    Align2::CENTER_CENTER,
                    if i == 0 { "PROPERTIES" } else { "NETS" },
                    FontId::new(9.0, egui::FontFamily::Name("medium".into())),
                    if active { TEXT } else { TEXT_FAINT },
                );
                if resp.clicked() {
                    app.inspector_tab = i;
                }
            }
            ui.add_space(12.0);

            if app.inspector_tab == 0 {
                properties_tab(app, ui);
            } else {
                nets_tab(app, ui);
            }
        });
}

fn properties_tab(app: &mut App, ui: &mut Ui) {
    match app.selection {
        Some(Selection::Comp(id)) => {
            let Some(comp) = app.project.comp(id).cloned() else {
                return;
            };
            let spec = comp.kind.spec();
            // Header card.
            let (card, _) =
                ui.allocate_exact_size(vec2(ui.available_width(), 74.0), Sense::hover());
            ui.painter().rect(
                card,
                egui::CornerRadius::same(10),
                Color32::from_rgb(0x11, 0x11, 0x11),
                Stroke::new(1.0, BORDER),
                StrokeKind::Inside,
            );
            let boxr = Rect::from_min_size(
                pos2(card.left() + 10.0, card.top() + 10.0),
                vec2(54.0, 54.0),
            );
            ui.painter().rect(
                boxr,
                egui::CornerRadius::same(8),
                Color32::from_rgb(0x0b, 0x0b, 0x0b),
                Stroke::new(1.0, BORDER_SOFT),
                StrokeKind::Inside,
            );
            draw_symbol_preview(ui.painter(), boxr.shrink(6.0), comp.kind);
            ui.painter().text(
                pos2(boxr.right() + 12.0, card.top() + 24.0),
                Align2::LEFT_CENTER,
                &comp.label,
                FontId::new(12.5, egui::FontFamily::Name("semibold".into())),
                TEXT,
            );
            ui.painter().text(
                pos2(boxr.right() + 12.0, card.top() + 41.0),
                Align2::LEFT_CENTER,
                spec.name,
                FontId::new(10.0, egui::FontFamily::Proportional),
                TEXT_DIM,
            );
            ui.painter().text(
                pos2(boxr.right() + 12.0, card.top() + 57.0),
                Align2::LEFT_CENTER,
                format!("{} · {}°{}", spec.short, comp.rot * 90, if comp.mirror { " · mirrored" } else { "" }),
                FontId::new(8.5, egui::FontFamily::Monospace),
                ACCENT,
            );
            ui.add_space(12.0);

            ui_kit::section_label(ui, "REFERENCE", None);
            let mut label = comp.label.clone();
            if ui
                .add(
                    egui::TextEdit::singleline(&mut label)
                        .desired_width(f32::INFINITY)
                        .font(mono(12.0)),
                )
                .changed()
            {
                if let Some(c) = app.project.comp_mut(id) {
                    c.label = label;
                }
            }
            ui.add_space(8.0);
            ui_kit::section_label(ui, spec.value_label.to_uppercase().as_str(), None);
            let mut value = comp.value.clone();
            if ui
                .add(
                    egui::TextEdit::singleline(&mut value)
                        .desired_width(f32::INFINITY)
                        .font(mono(12.0)),
                )
                .changed()
            {
                if let Some(c) = app.project.comp_mut(id) {
                    c.value = value;
                }
            }

            // Kind specific controls.
            match comp.kind {
                Kind::Potentiometer => {
                    ui.add_space(8.0);
                    ui_kit::section_label(
                        ui,
                        "WIPER",
                        Some(&format!("{:.0}%", comp.pot * 100.0)),
                    );
                    let mut pot = comp.pot;
                    if ui
                        .add(
                            egui::Slider::new(&mut pot, 0.0..=1.0)
                                .show_value(false)
                                .trailing_fill(true),
                        )
                        .changed()
                    {
                        if let Some(c) = app.project.comp_mut(id) {
                            c.pot = pot;
                        }
                    }
                }
                Kind::AcSource => {
                    ui.add_space(8.0);
                    ui_kit::section_label(ui, "FREQUENCY", Some("Hz"));
                    let mut f = comp.freq;
                    if ui
                        .add(
                            egui::DragValue::new(&mut f)
                                .speed(0.1)
                                .range(0.1..=1000.0),
                        )
                        .changed()
                    {
                        if let Some(c) = app.project.comp_mut(id) {
                            c.freq = f;
                        }
                    }
                }
                Kind::SwitchSpst => {
                    ui.add_space(8.0);
                    let closed = comp.state;
                    let txt = if closed { "Switch closed" } else { "Switch open" };
                    let mut button = egui::Button::new(
                        egui::RichText::new(txt).font(medium(11.5)),
                    )
                    .fill(if closed {
                        theme::with_alpha(ACCENT, 0.18)
                    } else {
                        ELEV
                    })
                    .stroke(Stroke::new(
                        1.0,
                        if closed { ACCENT } else { BORDER },
                    ));
                    button = button.min_size(vec2(ui.available_width(), 30.0));
                    if ui.add(button).clicked() {
                        app.snapshot();
                        if let Some(c) = app.project.comp_mut(id) {
                            c.state = !c.state;
                        }
                    }
                }
                Kind::UnoBoard | Kind::Atmega | Kind::Esp32 => {
                    ui.add_space(10.0);
                    ui_kit::section_label(ui, "FIRMWARE", None);
                    let (r, _) = ui.allocate_exact_size(
                        vec2(ui.available_width(), 42.0),
                        Sense::hover(),
                    );
                    ui.painter().rect(
                        r,
                        egui::CornerRadius::same(8),
                        Color32::from_rgb(0x0f, 0x0f, 0x0f),
                        Stroke::new(1.0, BORDER_SOFT),
                        StrokeKind::Inside,
                    );
                    ui.painter().text(
                        pos2(r.left() + 10.0, r.top() + 14.0),
                        Align2::LEFT_CENTER,
                        "blink.ino",
                        FontId::new(10.0, egui::FontFamily::Monospace),
                        TEXT_DIM,
                    );
                    ui.painter().text(
                        pos2(r.left() + 10.0, r.top() + 29.0),
                        Align2::LEFT_CENTER,
                        "D13 toggles every 500 ms",
                        FontId::new(9.0, egui::FontFamily::Proportional),
                        TEXT_FAINT,
                    );
                    let blink_on = (app.time * 2.0).rem_euclid(2.0) < 1.0;
                    ui.painter().circle_filled(
                        pos2(r.right() - 16.0, r.center().y),
                        4.0,
                        if blink_on { HIGH } else { ELEV_2 },
                    );
                }
                _ => {}
            }

            ui.add_space(12.0);
            ui_kit::section_label(ui, "ORIENTATION", None);
            ui.horizontal(|ui| {
                if ui_kit::icon_button(ui, Icon::Rotate, 30.0, false)
                    .on_hover_text("Rotate 90° (R)")
                    .clicked()
                {
                    app.rotate(1);
                }
                if ui_kit::icon_button(ui, Icon::Mirror, 30.0, comp.mirror)
                    .on_hover_text("Mirror (M)")
                    .clicked()
                {
                    app.mirror();
                }
                let (r, resp) = ui.allocate_exact_size(vec2(30.0, 30.0), Sense::click());
                ui.painter().rect(
                    r,
                    egui::CornerRadius::same(8),
                    if resp.hovered() {
                        theme::with_alpha(HIGH, 0.12)
                    } else {
                        ELEV
                    },
                    Stroke::new(1.0, BORDER),
                    StrokeKind::Inside,
                );
                ui_kit::paint_icon(
                    ui.painter(),
                    Rect::from_center_size(r.center(), Vec2::splat(16.0)),
                    Icon::Trash,
                    if resp.hovered() { HIGH } else { TEXT_DIM },
                    1.0,
                );
                if resp.clicked() {
                    app.delete_selection();
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(format!("{:.0}, {:.0}", comp.pos.x, comp.pos.y))
                            .font(mono(9.5))
                            .color(TEXT_FAINT),
                    );
                });
            });

            ui.add_space(14.0);
            ui_kit::section_label(ui, "LIVE READOUT", None);
            let current = app.result.comp_amp(id);
            let pins = comp.pins();
            let (va, vb) = if pins.len() >= 2 {
                let na = app
                    .netlist
                    .pin_net
                    .get(&PinRef { comp: id, pin: 0 })
                    .copied()
                    .unwrap_or(0);
                let nb = app
                    .netlist
                    .pin_net
                    .get(&PinRef { comp: id, pin: 1 })
                    .copied()
                    .unwrap_or(0);
                (app.result.volt(na), app.result.volt(nb))
            } else {
                (0.0, 0.0)
            };
            egui::Grid::new("readout")
                .num_columns(2)
                .spacing(vec2(8.0, 7.0))
                .show(ui, |ui| {
                    row_value(ui, "Voltage across", &format!("{:.2} V", va - vb), TEXT);
                    ui.end_row();
                    if comp.term_count() >= 2 {
                        row_value(
                            ui,
                            "Current",
                            &if current.abs() < 1.0 {
                                format!("{:.2} mA", current.abs() * 1000.0)
                            } else {
                                format!("{:.2} A", current.abs())
                            },
                            if current.abs() > 0.03 {
                                Color32::from_rgb(0xff, 0xb2, 0x5c)
                            } else {
                                TEXT_DIM
                            },
                        );
                        ui.end_row();
                    }
                    let level = app
                        .netlist
                        .pin_net
                        .get(&PinRef { comp: id, pin: 0 })
                        .map(|n| app.result.level(*n))
                        .unwrap_or(Level::Low);
                    ui.label(egui::RichText::new("Net level").font(body(10.0)).color(TEXT_FAINT));
                    ui.label(
                        egui::RichText::new(match level {
                            Level::High => "HIGH",
                            Level::Low => "LOW",
                            Level::Mid => "ANALOG",
                        })
                        .font(mono(10.0))
                        .color(symbols::level_color(level)),
                    );
                    ui.end_row();
                });
        }
        Some(Selection::Wire(id)) => {
            let Some(wire) = app.project.wire(id).cloned() else {
                return;
            };
            let al = wire
                .a
                .comp
                .to_string();
            let _ = al;
            let an = app
                .project
                .comp(wire.a.comp)
                .map(|c| format!("{}.{}", c.label, c.pins()[wire.a.pin].name))
                .unwrap_or_default();
            let bn = app
                .project
                .comp(wire.b.comp)
                .map(|c| format!("{}.{}", c.label, c.pins()[wire.b.pin].name))
                .unwrap_or_default();
            ui_kit::section_label(ui, "CONNECTION", None);
            let (card, _) = ui.allocate_exact_size(vec2(ui.available_width(), 72.0), Sense::hover());
            ui.painter().rect(
                card,
                egui::CornerRadius::same(10),
                Color32::from_rgb(0x11, 0x11, 0x11),
                Stroke::new(1.0, BORDER),
                StrokeKind::Inside,
            );
            ui.painter().text(
                pos2(card.left() + 12.0, card.top() + 22.0),
                Align2::LEFT_CENTER,
                &an,
                FontId::new(11.0, egui::FontFamily::Monospace),
                TEXT,
            );
            ui.painter().text(
                pos2(card.left() + 12.0, card.top() + 42.0),
                Align2::LEFT_CENTER,
                &bn,
                FontId::new(11.0, egui::FontFamily::Monospace),
                TEXT,
            );
            ui.painter().line_segment(
                [
                    pos2(card.left() + 12.0, card.top() + 32.0),
                    pos2(card.left() + 24.0, card.top() + 32.0),
                ],
                Stroke::new(1.0, ACCENT),
            );
            let net = app.netlet_wire_net(wire.id);
            let level = app.result.level(net);
            ui.painter().circle_filled(
                pos2(card.right() - 18.0, card.center().y),
                5.0,
                symbols::level_color(level),
            );
            ui.add_space(10.0);
            egui::Grid::new("wire_readout")
                .num_columns(2)
                .spacing(vec2(8.0, 7.0))
                .show(ui, |ui| {
                    row_value(ui, "Net", &app.netlet_name(net), TEXT);
                    ui.end_row();
                    row_value(ui, "Length", &format!("{:.0} u", wire.length()), TEXT_DIM);
                    ui.end_row();
                    let flow = app.result.wire_flow.get(&wire.id).copied().unwrap_or(0.0);
                    row_value(
                        ui,
                        "Current",
                        &if flow.abs() < 1.0 {
                            format!("{:.2} mA", flow.abs() * 1000.0)
                        } else {
                            format!("{:.2} A", flow.abs())
                        },
                        TEXT_DIM,
                    );
                    ui.end_row();
                    row_value(
                        ui,
                        "Routing",
                        if wire.manual { "Manual" } else { "Auto" },
                        TEXT_DIM,
                    );
                    ui.end_row();
                });
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui
                    .add(
                        egui::Button::new(egui::RichText::new("Re-route").font(medium(11.0)))
                            .min_size(vec2(96.0, 28.0)),
                    )
                    .clicked()
                {
                    app.snapshot();
                    if let Some(w) = app.project.wires.iter_mut().find(|w| w.id == id) {
                        w.manual = false;
                    }
                    app.project.reroute_all();
                    app.toast("Route recalculated", app.clock);
                }
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new("Delete wire").font(medium(11.0)).color(HIGH),
                        )
                        .min_size(vec2(110.0, 28.0))
                        .fill(Color32::TRANSPARENT)
                        .stroke(Stroke::new(1.0, theme::with_alpha(HIGH, 0.35))),
                    )
                    .clicked()
                {
                    app.delete_selection();
                }
            });
            ui.add_space(10.0);
            ui.label(
                egui::RichText::new("Drag any segment to reshape the route. Corners stay square.")
                    .font(body(10.0))
                    .color(TEXT_FAINT),
            );
        }
        None => {
            ui_kit::section_label(ui, "NOTHING SELECTED", None);
            let (card, _) = ui.allocate_exact_size(vec2(ui.available_width(), 118.0), Sense::hover());
            ui.painter().rect(
                card,
                egui::CornerRadius::same(10),
                Color32::from_rgb(0x11, 0x11, 0x11),
                Stroke::new(1.0, BORDER),
                StrokeKind::Inside,
            );
            ui.painter().text(
                pos2(card.left() + 14.0, card.top() + 26.0),
                Align2::LEFT_CENTER,
                "Pick a part on the canvas",
                FontId::new(12.0, egui::FontFamily::Name("medium".into())),
                TEXT_DIM,
            );
            ui.painter().text(
                pos2(card.left() + 14.0, card.top() + 48.0),
                Align2::LEFT_CENTER,
                "Click a symbol to edit its value,",
                FontId::new(10.0, egui::FontFamily::Proportional),
                TEXT_FAINT,
            );
            ui.painter().text(
                pos2(card.left() + 14.0, card.top() + 64.0),
                Align2::LEFT_CENTER,
                "or grab a pin to start a wire.",
                FontId::new(10.0, egui::FontFamily::Proportional),
                TEXT_FAINT,
            );
            ui.painter().text(
                pos2(card.left() + 14.0, card.top() + 92.0),
                Align2::LEFT_CENTER,
                "Double-click a part to rotate it",
                FontId::new(9.5, egui::FontFamily::Proportional),
                ACCENT,
            );
            ui.add_space(14.0);
            ui_kit::section_label(ui, "CIRCUIT", None);
            egui::Grid::new("circuit_stats")
                .num_columns(2)
                .spacing(vec2(8.0, 7.0))
                .show(ui, |ui| {
                    row_value(
                        ui,
                        "Components",
                        &app.project.comps.len().to_string(),
                        TEXT_DIM,
                    );
                    ui.end_row();
                    row_value(ui, "Wires", &app.project.wires.len().to_string(), TEXT_DIM);
                    ui.end_row();
                    row_value(ui, "Nets", &app.netlist.nets.len().to_string(), TEXT_DIM);
                    ui.end_row();
                    row_value(
                        ui,
                        "Solver nodes",
                        &app.result.active_nodes.to_string(),
                        TEXT_DIM,
                    );
                    ui.end_row();
                });
        }
    }
}

fn nets_tab(app: &mut App, ui: &mut Ui) {
    ui_kit::section_label(
        ui,
        "NETS",
        Some(&format!("{} total", app.netlist.nets.len())),
    );
    let nets: Vec<(usize, String, f64, usize, Level)> = app
        .netlist
        .nets
        .iter()
        .map(|n| {
            (
                n.id,
                n.name.clone(),
                app.result.volt(n.id),
                n.pins.len(),
                app.result.level(n.id),
            )
        })
        .collect();
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for (id, name, volts, pins, level) in nets {
                let (row, resp) =
                    ui.allocate_exact_size(vec2(ui.available_width(), 40.0), Sense::click());
                let active = app.instruments.probe_net == id;
                if resp.hovered() || active {
                    ui.painter().rect_filled(
                        row.shrink(1.0),
                        egui::CornerRadius::same(8),
                        if active { theme::with_alpha(ACCENT, 0.10) } else { ELEV },
                    );
                }
                let col = symbols::level_color(level);
                ui.painter().circle_filled(
                    pos2(row.left() + 16.0, row.center().y),
                    4.5,
                    col,
                );
                ui.painter().text(
                    pos2(row.left() + 30.0, row.center().y - 6.0),
                    Align2::LEFT_CENTER,
                    &name,
                    FontId::new(11.0, egui::FontFamily::Name("medium".into())),
                    TEXT,
                );
                ui.painter().text(
                    pos2(row.left() + 30.0, row.center().y + 8.0),
                    Align2::LEFT_CENTER,
                    format!("{pins} pins"),
                    FontId::new(8.5, egui::FontFamily::Proportional),
                    TEXT_FAINT,
                );
                ui.painter().text(
                    pos2(row.right() - 12.0, row.center().y),
                    Align2::RIGHT_CENTER,
                    if volts.abs() < 0.005 {
                        "0.00 V".to_string()
                    } else {
                        format!("{volts:.2} V")
                    },
                    FontId::new(9.5, egui::FontFamily::Monospace),
                    if level == Level::High { HIGH } else { TEXT_DIM },
                );
                if resp.clicked() {
                    app.instruments.probe_net = id;
                    app.instruments.probe_open = true;
                }
            }
        });
}

fn row_value(ui: &mut Ui, label: &str, value: &str, color: Color32) {
    ui.label(
        egui::RichText::new(label)
            .font(body(10.0))
            .color(TEXT_FAINT),
    );
    ui.label(
        egui::RichText::new(value)
            .font(mono(10.5))
            .color(color),
    );
}
