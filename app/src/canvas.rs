//! The interactive schematic canvas: dot grid, pan and zoom, component
//! placement, orthogonal wiring, route editing and the live signal animation.

use crate::app::{App, DragKind, DragState, Selection, Tool};
use crate::model::{snap, Dir, Kind, PinRef, PinRole, GRID};
use crate::routing;
use crate::sim::Level;
use crate::symbols::{self, DrawOpts};
use crate::theme;
use crate::theme::*;
use crate::ui_kit::{self, Icon};
use eframe::egui::{
    self, pos2, vec2, Align2, Color32, CursorIcon, FontId, Mesh, Painter, Pos2, Rect, Response,
    Sense, Shape, Stroke, StrokeKind, Ui, Vec2,
};

struct Hit {
    pin: Option<PinRef>,
    wire: Option<usize>,
    seg: Option<(usize, usize)>,
    comp: Option<usize>,
}

impl Hit {
    fn empty() -> Self {
        Hit {
            pin: None,
            wire: None,
            seg: None,
            comp: None,
        }
    }
}

pub fn show(app: &mut App, ctx: &egui::Context) {
    egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(CANVAS))
        .show(ctx, |ui| {
            let rect = ui.max_rect();
            app.canvas_rect = rect;
            let response = ui.allocate_rect(rect, Sense::click_and_drag());

            let overlays = overlay_rects(rect);
            let hover_screen = response.hover_pos();
            app.last_pointer = hover_screen;
            let over_ui = hover_screen
                .map(|p| overlays.iter().any(|r| r.contains(p)))
                .unwrap_or(false);

            if let Some(scroll) = wheel_input(ctx, &response, over_ui) {
                if let Some(p) = hover_screen {
                    let before = to_world(app, rect, p);
                    app.zoom = (app.zoom * (1.0 + scroll * 0.0016)).clamp(0.25, 3.0);
                    let after = rect.center() + (before - app.view_center) * app.zoom;
                    app.view_center += (p - after) / app.zoom;
                }
            }

            // Hover state.
            app.hover_comp = None;
            app.hover_pin = None;
            app.hover_seg = None;
            if let Some(p) = hover_screen.filter(|_| !over_ui) {
                let hit = hit_test(app, rect, p);
                app.hover_pin = hit.pin;
                app.hover_comp = hit.comp;
                app.hover_seg = hit.seg;
                ui.ctx().set_cursor_icon(match hit.pin {
                    Some(_) => CursorIcon::Crosshair,
                    None => match hit.comp {
                        Some(_) => CursorIcon::Grab,
                        None => match hit.seg {
                            Some(_) => CursorIcon::ResizeHorizontal,
                            None => CursorIcon::Default,
                        },
                    },
                });
            }

            // Wire rubber band follows the pointer.
            if app.wire_start.is_some() {
                app.wire_cursor = hover_screen.map(|p| snap(to_world(app, rect, p), app.snap));
            } else {
                app.wire_cursor = None;
            }

            // Middle mouse panning works anywhere on the canvas.
            let middle_down = ctx.input(|i| i.pointer.middle_down());
            if middle_down && response.hovered() && !over_ui {
                let d = ctx.input(|i| i.pointer.delta());
                app.view_center -= d / app.zoom;
            }

            // Drag start.
            if response.drag_started() {
                let origin = ctx
                    .input(|i| i.pointer.press_origin())
                    .unwrap_or_else(|| rect.center());
                if !overlays.iter().any(|r| r.contains(origin)) {
                    start_drag(app, ctx, rect, origin, &response);
                }
            }

            // Drag update.
            if response.dragged() {
                update_drag(app, ctx, rect);
            }

            if response.drag_stopped() {
                finish_drag(app, ctx, rect, hover_screen);
            }

            if response.clicked() {
                if let Some(p) = response.interact_pointer_pos() {
                    if !overlays.iter().any(|r| r.contains(p)) {
                        handle_click(app, ctx, rect, p);
                    }
                }
            }
            if response.double_clicked() {
                if let Some(p) = response.interact_pointer_pos() {
                    if !overlays.iter().any(|r| r.contains(p)) {
                        let hit = hit_test(app, rect, p);
                        if let Some(cid) = hit.comp {
                            app.selection = Some(Selection::Comp(cid));
                            app.rotate(1);
                        }
                    }
                }
            }

            paint(app, ui, rect);

            // Floating toolbar and small overlays.
            toolbar(app, ui, overlays[0]);
            zoom_pill(app, ui, overlays[1]);
            legend(app, ui, overlays[2]);
            sheet_chip(app, ui, overlays[3]);
            hint_chip(app, ui, rect, overlays);
            let _ = ctx;
        });
}

fn overlay_rects(rect: Rect) -> [Rect; 4] {
    let toolbar = Rect::from_min_size(
        pos2(rect.left() + 14.0, rect.center().y - 104.0),
        vec2(46.0, 208.0),
    );
    let zoom = Rect::from_min_size(
        pos2(rect.right() - 196.0, rect.bottom() - 50.0),
        vec2(182.0, 36.0),
    );
    let legend = Rect::from_min_size(
        pos2(rect.left() + 14.0, rect.bottom() - 38.0),
        vec2(226.0, 26.0),
    );
    let sheet = Rect::from_min_size(pos2(rect.left() + 14.0, rect.top() + 12.0), vec2(330.0, 34.0));
    [toolbar, zoom, legend, sheet]
}

fn wheel_input(ctx: &egui::Context, response: &Response, over_ui: bool) -> Option<f32> {
    if !response.hovered() || over_ui {
        return None;
    }
    ctx.input(|i| {
        let mut v = i.smooth_scroll_delta.y;
        if v == 0.0 {
            v = i.raw_scroll_delta.y;
        }
        if i.zoom_delta() != 1.0 {
            v += (i.zoom_delta() - 1.0) * 400.0;
        }
        (v != 0.0).then_some(v)
    })
}

pub fn to_screen(app: &App, rect: Rect, p: Pos2) -> Pos2 {
    rect.center() + (p - app.view_center) * app.zoom
}

pub fn to_world(app: &App, rect: Rect, p: Pos2) -> Pos2 {
    app.view_center + (p - rect.center()) / app.zoom
}

fn hit_test(app: &App, rect: Rect, p: Pos2) -> Hit {
    let mut hit = Hit::empty();
    let mut best_pin = 9.0f32;
    for comp in &app.project.comps {
        for pin in 0..comp.pins().len() {
            let sp = to_screen(app, rect, comp.pin_pos(pin));
            let d = sp.distance(p);
            if d < best_pin {
                best_pin = d;
                hit.pin = Some(PinRef {
                    comp: comp.id,
                    pin,
                });
            }
        }
    }
    let mut best_seg = 6.0f32;
    for wire in &app.project.wires {
        let pts: Vec<Pos2> = wire.pts.iter().map(|w| to_screen(app, rect, *w)).collect();
        for (i, seg) in pts.windows(2).enumerate() {
            let d = dist_to_seg(p, seg[0], seg[1]);
            if d < best_seg {
                best_seg = d;
                hit.wire = Some(wire.id);
                hit.seg = Some((wire.id, i));
            }
        }
    }
    for comp in app.project.comps.iter().rev() {
        let b = comp.bounds().expand(6.0 / app.zoom);
        if b.contains(to_world(app, rect, p)) {
            hit.comp = Some(comp.id);
            break;
        }
    }
    hit
}

fn dist_to_seg(p: Pos2, a: Pos2, b: Pos2) -> f32 {
    let ab = b - a;
    let len2 = ab.length_sq();
    if len2 < 1e-6 {
        return p.distance(a);
    }
    let t = ((p - a).dot(ab) / len2).clamp(0.0, 1.0);
    p.distance(a + ab * t)
}

fn start_drag(app: &mut App, ctx: &egui::Context, rect: Rect, origin: Pos2, response: &Response) {
    let hit = hit_test(app, rect, origin);
    let world = to_world(app, rect, origin);
    let delta = ctx.input(|i| i.pointer.delta());
    let _ = delta;

    if let Some(pin) = hit.pin {
        app.wire_start = Some(pin);
        app.wire_pressed_at = Some(pin);
        app.selection = Some(Selection::Comp(pin.comp));
        app.drag = None;
        return;
    }
    if app.tool == Tool::Pan || response.dragged_by(egui::PointerButton::Secondary) {
        app.drag = Some(DragState {
            kind: DragKind::Pan,
            start_screen: origin,
            start_world: world,
            orig_pts: Vec::new(),
            orig_pos: app.view_center,
            moved: false,
        });
        return;
    }
    if let Some((wire, seg)) = hit.seg {
        if app.tool == Tool::Select {
            app.selection = Some(Selection::Wire(wire));
            let orig = app
                .project
                .wire(wire)
                .map(|w| w.pts.clone())
                .unwrap_or_default();
            app.drag = Some(DragState {
                kind: DragKind::Segment { wire, seg },
                start_screen: origin,
                start_world: world,
                orig_pts: orig,
                orig_pos: Pos2::ZERO,
                moved: false,
            });
            app.snapshot();
            return;
        }
    }
    if let Some(cid) = hit.comp {
        app.selection = Some(Selection::Comp(cid));
        if let Some(c) = app.project.comp(cid) {
            app.drag = Some(DragState {
                kind: DragKind::Comp,
                start_screen: origin,
                start_world: world,
                orig_pts: Vec::new(),
                orig_pos: c.pos,
                moved: false,
            });
            app.snapshot();
        }
        return;
    }
    app.selection = None;
    app.drag = Some(DragState {
        kind: DragKind::Pan,
        start_screen: origin,
        start_world: world,
        orig_pts: Vec::new(),
        orig_pos: app.view_center,
        moved: false,
    });
}

fn update_drag(app: &mut App, ctx: &egui::Context, rect: Rect) {
    let Some(drag) = app.drag.clone() else { return };
    let pointer = ctx.input(|i| i.pointer.interact_pos()).unwrap_or(drag.start_screen);
    let world = to_world(app, rect, pointer);
    if pointer.distance(drag.start_screen) > 2.0 {
        if let Some(d) = app.drag.as_mut() {
            d.moved = true;
        }
    }
    match drag.kind {
        DragKind::Pan => {
            let delta = ctx.input(|i| i.pointer.delta());
            app.view_center -= delta / app.zoom;
        }
        DragKind::Comp => {
            let id = match app.selection {
                Some(Selection::Comp(id)) => id,
                _ => return,
            };
            let target = drag.orig_pos + (world - drag.start_world);
            let target = snap(target, app.snap);
            if let Some(c) = app.project.comp_mut(id) {
                c.pos = target;
            }
            app.project.reroute_all();
        }
        DragKind::Segment { wire, seg } => {
            let pts = apply_segment_drag(&drag.orig_pts, seg, world, app.snap);
            if let Some(w) = app.project.wires.iter_mut().find(|w| w.id == wire) {
                w.pts = pts;
                w.manual = true;
            }
        }
        DragKind::Marquee => {}
    }
}

fn finish_drag(app: &mut App, ctx: &egui::Context, rect: Rect, hover: Option<Pos2>) {
    let drag = app.drag.take();
    if let Some(drag) = drag {
        if drag.moved {
            if let DragKind::Comp = drag.kind {
                app.project.reroute_all();
                app.toast("Component moved", app.clock);
            }
            if let DragKind::Segment { .. } = drag.kind {
                app.toast("Wire reshaped", app.clock);
            }
            if let DragKind::Pan = drag.kind {
                // nothing to record
            }
        }
    }
    // Drag-wiring: releasing over a second pin connects them.
    if let Some(start) = app.wire_start.take() {
        let release = hover.or_else(|| ctx.input(|i| i.pointer.interact_pos()));
        if let Some(p) = release {
            let hit = hit_test(app, rect, p);
            if let Some(pin) = hit.pin {
                if pin != start {
                    app.add_wire(start, pin);
                }
            }
        }
        app.wire_pressed_at = None;
        app.wire_cursor = None;
    }
}

fn handle_click(app: &mut App, ctx: &egui::Context, rect: Rect, p: Pos2) {
    let _ = ctx;
    let hit = hit_test(app, rect, p);
    if let Some(start) = app.wire_start {
        if let Some(pin) = hit.pin {
            if pin != start {
                app.add_wire(start, pin);
            }
        }
        app.wire_start = None;
        app.wire_cursor = None;
        return;
    }
    if let Some(kind) = app.armed {
        let world = snap(to_world(app, rect, p), app.snap);
        app.place(kind, world);
        if !ctx.input(|i| i.modifiers.shift) {
            app.armed = None;
        }
        return;
    }
    if let Some(seg) = hit.seg {
        if app.tool == Tool::Select {
            app.selection = Some(Selection::Wire(seg.0));
            return;
        }
    }
    if let Some(pin) = hit.pin {
        app.selection = Some(Selection::Comp(pin.comp));
        app.wire_start = Some(pin);
        app.wire_pressed_at = Some(pin);
        return;
    }
    if let Some(cid) = hit.comp {
        app.selection = Some(Selection::Comp(cid));
        app.lib_preview = app.project.comp(cid).map(|c| c.kind);
        return;
    }
    app.selection = None;
}

fn apply_segment_drag(orig: &[Pos2], seg: usize, mouse: Pos2, snap_on: bool) -> Vec<Pos2> {
    let mut pts = orig.to_vec();
    if pts.len() < 3 || seg + 1 >= pts.len() {
        return pts;
    }
    let a = pts[seg];
    let b = pts[seg + 1];
    let horizontal = (a.y - b.y).abs() < (a.x - b.x).abs();
    let mut value = if horizontal { mouse.y } else { mouse.x };
    if snap_on {
        value = (value / GRID).round() * GRID;
    }
    let n = pts.len();
    if seg > 0 && seg + 1 < n - 1 {
        // Interior segment: keep the two neighbouring legs pointing the right way.
        let p_prev = pts[seg - 1];
        let p_next = pts[seg + 2];
        if horizontal {
            let mut lo = f32::NEG_INFINITY;
            let mut hi = f32::INFINITY;
            if a.y >= p_prev.y {
                lo = lo.max(p_prev.y + 8.0);
            } else {
                hi = hi.min(p_prev.y - 8.0);
            }
            if b.y >= p_next.y {
                lo = lo.max(p_next.y + 8.0);
            } else {
                hi = hi.min(p_next.y - 8.0);
            }
            value = value.clamp(lo, hi);
            pts[seg].y = value;
            pts[seg + 1].y = value;
        } else {
            let mut lo = f32::NEG_INFINITY;
            let mut hi = f32::INFINITY;
            if a.x >= p_prev.x {
                lo = lo.max(p_prev.x + 8.0);
            } else {
                hi = hi.min(p_prev.x - 8.0);
            }
            if b.x >= p_next.x {
                lo = lo.max(p_next.x + 8.0);
            } else {
                hi = hi.min(p_next.x - 8.0);
            }
            value = value.clamp(lo, hi);
            pts[seg].x = value;
            pts[seg + 1].x = value;
        }
    } else if seg == 0 {
        let p0 = pts[0];
        let p1 = pts[1];
        let mut insert = if horizontal {
            vec![p0, pos2(p0.x, value), pos2(p1.x, value)]
        } else {
            vec![p0, pos2(value, p0.y), pos2(value, p1.y)]
        };
        insert.extend_from_slice(&pts[1..]);
        pts = insert;
    } else {
        let last = pts[n - 1];
        let prev = pts[n - 2];
        let mut insert = if horizontal {
            vec![prev, pos2(prev.x, value), pos2(last.x, value)]
        } else {
            vec![prev, pos2(value, prev.y), pos2(value, last.y)]
        };
        insert.push(last);
        let mut newpts = pts[..n - 2].to_vec();
        newpts.extend(insert);
        pts = newpts;
    }
    routing::simplify(pts)
}

fn paint(app: &mut App, ui: &mut Ui, rect: Rect) {
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0, CANVAS);
    if app.show_grid {
        paint_grid(&painter, rect, app);
    }
    paint_wires(&painter, rect, app);
    paint_components(&painter, rect, app);
    paint_selection(&painter, rect, app);
    paint_preview(&painter, rect, app);
    paint_overlays(&painter, rect, app);
}

fn paint_grid(painter: &Painter, rect: Rect, app: &App) {
    let mut world_step = 20.0f32;
    while world_step * app.zoom < 7.0 {
        world_step *= 2.0;
    }
    let step = world_step * app.zoom;
    let mut mesh = Mesh::default();
    let kx0 = (((rect.left() - rect.center().x) / app.zoom + app.view_center.x) / world_step)
        .floor() as i64;
    let kx1 = (((rect.right() - rect.center().x) / app.zoom + app.view_center.x) / world_step)
        .ceil() as i64;
    let ky0 = (((rect.top() - rect.center().y) / app.zoom + app.view_center.y) / world_step)
        .floor() as i64;
    let ky1 = (((rect.bottom() - rect.center().y) / app.zoom + app.view_center.y) / world_step)
        .ceil() as i64;
    let _ = step;
    for kx in kx0..=kx1 {
        for ky in ky0..=ky1 {
            let x = rect.center().x + (kx as f32 * world_step - app.view_center.x) * app.zoom;
            let y = rect.center().y + (ky as f32 * world_step - app.view_center.y) * app.zoom;
            if x < rect.left() || x > rect.right() || y < rect.top() || y > rect.bottom() {
                continue;
            }
            let major = kx % 5 == 0 && ky % 5 == 0;
            let r = if major { 1.4 } else { 0.9 };
            mesh.add_colored_rect(
                Rect::from_center_size(pos2(x, y), Vec2::splat(r * 2.0)),
                if major { GRID_DOT_HI } else { GRID_DOT },
            );
        }
    }
    painter.add(Shape::mesh(mesh));
}

fn wire_color(app: &App, wire: &crate::model::Wire) -> Color32 {
    let net = app.netlist.wire_net.get(&wire.id).copied().unwrap_or(0);
    let level = app.result.level(net);
    theme::mix(WIRE, symbols::level_color(level), 0.82)
}

fn paint_wires(painter: &Painter, rect: Rect, app: &App) {
    for wire in &app.project.wires {
        let pts: Vec<Pos2> = wire.pts.iter().map(|p| to_screen(app, rect, *p)).collect();
        if pts.len() < 2 {
            continue;
        }
        let selected = app.selection == Some(Selection::Wire(wire.id));
        let hovered = app
            .hover_seg
            .map(|(w, _)| w == wire.id)
            .unwrap_or(false);
        let color = wire_color(app, wire);
        if selected || hovered {
            painter.add(Shape::line(
                pts.clone(),
                Stroke::new(7.0, theme::with_alpha(ACCENT, 0.10)),
            ));
        }
        painter.add(Shape::line(
            pts.clone(),
            Stroke::new(
                if selected { 2.4 } else { 1.8 },
                if selected {
                    ACCENT_HI
                } else if hovered {
                    theme::mix(color, Color32::WHITE, 0.35)
                } else {
                    color
                },
            ),
        ));

        // Animated current flow.
        let flow = app.result.wire_flow.get(&wire.id).copied().unwrap_or(0.0);
        if app.running && flow.abs() > 2e-5 {
            let speed = (18.0 + (flow.abs() * 900.0).min(90.0)) as f32;
            let spacing = 26.0f32;
            let phase = (app.clock as f32 * speed) % spacing;
            let total = routing::polyline_length(&pts);
            let dir = if flow >= 0.0 { 1.0 } else { -1.0 };
            let mut d = if dir > 0.0 { phase } else { spacing - phase };
            while d < total {
                let at = if dir > 0.0 { d } else { total - d };
                if let Some(p) = routing::point_along(&pts, at) {
                    painter.circle_filled(
                        p,
                        2.6,
                        theme::with_alpha(theme::mix(color, Color32::WHITE, 0.55), 0.95),
                    );
                }
                d += spacing;
            }
        }
    }

    // Junction dots where two or more wires share a pin.
    for comp in &app.project.comps {
        for pin in 0..comp.pins().len() {
            let r = PinRef {
                comp: comp.id,
                pin,
            };
            if app.project.pin_wires(r) >= 2 {
                let net = app.netlist.pin_net.get(&r).copied().unwrap_or(0);
                let color = symbols::level_color(app.result.level(net));
                let p = to_screen(app, rect, comp.pin_pos(pin));
                painter.circle_filled(p, 3.4, theme::mix(color, Color32::WHITE, 0.1));
                painter.circle_stroke(p, 5.2, Stroke::new(1.0, theme::with_alpha(color, 0.25)));
            }
        }
    }
}

fn pin_colors(app: &App, comp: &crate::model::Component) -> Vec<Color32> {
    (0..comp.pins().len())
        .map(|pin| {
            let net = app
                .netlist
                .pin_net
                .get(&PinRef {
                    comp: comp.id,
                    pin,
                })
                .copied()
                .unwrap_or(0);
            theme::mix(
                symbols::level_color(app.result.level(net)),
                Color32::WHITE,
                0.05,
            )
        })
        .collect()
}

fn paint_components(painter: &Painter, rect: Rect, app: &App) {
    for comp in &app.project.comps {
        let center = to_screen(app, rect, comp.pos);
        let selected = app.selection == Some(Selection::Comp(comp.id));
        let hovered = app.hover_comp == Some(comp.id);
        let opts = DrawOpts {
            selected,
            hovered,
            ghost: false,
            labels: true,
            lit_override: None,
            sim: Some(&app.result),
            pin_colors: Some(pin_colors(app, comp)),
            highlight_pin: app
                .hover_pin
                .filter(|p| p.comp == comp.id)
                .map(|p| p.pin),
            dim: false,
        };
        if selected {
            let b = comp.bounds().expand(7.0);
            let r = Rect::from_min_max(to_screen(app, rect, b.min), to_screen(app, rect, b.max));
            painter.rect(
                r,
                egui::CornerRadius::same(8),
                Color32::TRANSPARENT,
                Stroke::new(1.2, ACCENT),
                StrokeKind::Inside,
            );
            for c in [
                r.left_top(),
                r.right_top(),
                r.left_bottom(),
                r.right_bottom(),
            ] {
                painter.rect_filled(
                    Rect::from_center_size(c, Vec2::splat(4.5)),
                    egui::CornerRadius::same(1),
                    ACCENT,
                );
            }
        }
        symbols::draw(painter, comp, center, app.zoom, &opts);
    }
}

fn paint_selection(painter: &Painter, rect: Rect, app: &App) {
    // Wire segment handle for the selected wire.
    if let Some(Selection::Wire(id)) = app.selection {
        if let Some(wire) = app.project.wire(id) {
            let pts: Vec<Pos2> = wire.pts.iter().map(|p| to_screen(app, rect, *p)).collect();
            for i in 1..pts.len().saturating_sub(1) {
                painter.circle_filled(pts[i], 3.0, Color32::from_rgb(0x16, 0x0b, 0x00));
                painter.circle_stroke(pts[i], 4.2, Stroke::new(1.4, ACCENT));
            }
        }
    }
}

fn paint_preview(painter: &Painter, rect: Rect, app: &App) {
    // Placing a new part.
    if let Some(kind) = app.armed {
        if let Some(hover) = app.last_pointer {
            let pos = snap(to_world(app, rect, hover), app.snap);
            let mut comp = crate::model::Component::new(0, kind, pos, String::new());
            comp.value = kind.spec().value.to_string();
            let center = to_screen(app, rect, pos);
            let opts = DrawOpts {
                ghost: true,
                labels: false,
                ..Default::default()
            };
            symbols::draw(painter, &comp, center, app.zoom, &opts);
            let b = comp.bounds().expand(8.0);
            let r = Rect::from_min_max(to_screen(app, rect, b.min), to_screen(app, rect, b.max));
            painter.rect(
                r,
                egui::CornerRadius::same(6),
                Color32::TRANSPARENT,
                Stroke::new(1.0, theme::with_alpha(ACCENT, 0.35)),
                StrokeKind::Inside,
            );
        }
    }

    // Rubber band wire.
    if let (Some(start), Some(cursor)) = (app.wire_start, app.wire_cursor) {
        let (Some(pa), Some(da)) = (app.project.pin_pos(start), app.project.pin_dir(start)) else {
            return;
        };
        let hover_pin = app.hover_pin.filter(|p| *p != start);
        let (path, target_ok) = if let Some(target) = hover_pin {
            let (Some(pb), Some(db)) = (app.project.pin_pos(target), app.project.pin_dir(target))
            else {
                return;
            };
            let obstacles = app.project.obstacles(start.comp, target.comp);
            let wires = app.project.wire_obstacles(0);
            (routing::route(pa, da, pb, db, &obstacles, &wires), true)
        } else {
            let s = pa + da.vec() * 14.0;
            let path = if (s.x - cursor.x).abs() < 1.0 || (s.y - cursor.y).abs() < 1.0 {
                vec![pa, s, cursor]
            } else if (cursor.x - s.x).abs() > (cursor.y - s.y).abs() {
                vec![pa, s, pos2(cursor.x, s.y), cursor]
            } else {
                vec![pa, s, pos2(s.x, cursor.y), cursor]
            };
            (path, false)
        };
        let pts: Vec<Pos2> = path.iter().map(|p| to_screen(app, rect, *p)).collect();
        let tip = pts.last().copied().unwrap_or(Pos2::ZERO);
        painter.add(Shape::line(
            pts,
            Stroke::new(
                1.8,
                if target_ok { ACCENT_HI } else { theme::with_alpha(ACCENT, 0.7) },
            ),
        ));
        painter.circle_filled(
            tip,
            4.0,
            if target_ok { ACCENT_HI } else { theme::with_alpha(ACCENT, 0.5) },
        );
        if let Some(target) = hover_pin {
            let p = to_screen(app, rect, app.project.pin_pos(target).unwrap());
            painter.circle_stroke(p, 8.0, Stroke::new(1.4, ACCENT_HI));
        }
    }
}

fn paint_overlays(painter: &Painter, rect: Rect, app: &App) {
    // Floating net label next to the cursor when hovering a pin.
    if let Some(pin) = app.hover_pin {
        if let (Some(pos), Some(comp)) = (
            app.project.pin_pos(pin),
            app.project.comp(pin.comp),
        ) {
            let net = app.netlist.pin_net.get(&pin).copied().unwrap_or(0);
            let name = app
                .netlist
                .nets
                .get(net)
                .map(|n| n.name.clone())
                .unwrap_or_default();
            let v = app.result.volt(net);
            let text = format!("{}  ·  {:.2} V", name, v);
            let sp = to_screen(app, rect, pos) + vec2(14.0, -26.0);
            let font = FontId::new(9.5, egui::FontFamily::Monospace);
            let galley = painter.layout_no_wrap(text, font, TEXT);
            let size = galley.size() + vec2(14.0, 8.0);
            let boxr = Rect::from_min_size(sp, size);
            painter.rect(
                boxr,
                egui::CornerRadius::same(6),
                Color32::from_rgb(0x0b, 0x0b, 0x0b),
                Stroke::new(1.0, BORDER_HI),
                StrokeKind::Inside,
            );
            painter.galley(boxr.min + vec2(7.0, 4.0), galley, TEXT_DIM);
            let _ = comp;
        }
    }
    let _ = rect;
}

fn toolbar(app: &mut App, ui: &mut Ui, rect: Rect) {
    let p = ui.painter().clone();
    p.rect(
        rect,
        egui::CornerRadius::same(11),
        Color32::from_rgb(0x16, 0x16, 0x16),
        Stroke::new(1.0, BORDER),
        StrokeKind::Inside,
    );
    let buttons: [(Icon, Tool, &str); 3] = [
        (Icon::Cursor, Tool::Select, "Select  (V)"),
        (Icon::Wire, Tool::Wire, "Wire  (W)"),
        (Icon::Move, Tool::Pan, "Pan  (H)"),
    ];
    let mut y = rect.top() + 8.0;
    for (icon, tool, tip) in buttons {
        let r = Rect::from_min_size(pos2(rect.left() + 7.0, y), vec2(32.0, 32.0));
        let resp = ui.interact(r, ui.id().with(("tool", tip)), Sense::click());
        let active = app.tool == tool && app.armed.is_none();
        if active {
            p.rect_filled(r, egui::CornerRadius::same(8), theme::with_alpha(ACCENT, 0.16));
            p.rect_stroke(
                r,
                egui::CornerRadius::same(8),
                Stroke::new(1.0, theme::with_alpha(ACCENT, 0.5)),
                StrokeKind::Inside,
            );
        } else if resp.hovered() {
            p.rect_filled(r, egui::CornerRadius::same(8), ELEV_2);
        }
        ui_kit::paint_icon(
            &p,
            Rect::from_center_size(r.center(), Vec2::splat(17.0)),
            icon,
            if active { ACCENT_HI } else if resp.hovered() { TEXT } else { TEXT_DIM },
            1.0,
        );
        if resp.clicked() {
            app.tool = tool;
            app.armed = None;
            app.wire_start = None;
        }
        let _ = resp.on_hover_text(tip);
        y += 38.0;
    }
    p.line_segment(
        [pos2(rect.left() + 11.0, y + 2.0), pos2(rect.right() - 11.0, y + 2.0)],
        Stroke::new(1.0, BORDER_SOFT),
    );
    y += 10.0;
    let toggles: [(Icon, bool, &str); 2] = [
        (Icon::Grid, app.show_grid, "Dot grid"),
        (Icon::Magnet, app.snap, "Snap to grid  (G)"),
    ];
    for (icon, on, tip) in toggles {
        let r = Rect::from_min_size(pos2(rect.left() + 7.0, y), vec2(32.0, 32.0));
        let resp = ui.interact(r, ui.id().with(("tgl", tip)), Sense::click());
        if on {
            p.rect_filled(r, egui::CornerRadius::same(8), theme::with_alpha(ACCENT, 0.14));
        } else if resp.hovered() {
            p.rect_filled(r, egui::CornerRadius::same(8), ELEV_2);
        }
        ui_kit::paint_icon(
            &p,
            Rect::from_center_size(r.center(), Vec2::splat(16.0)),
            icon,
            if on { ACCENT_HI } else { TEXT_FAINT },
            1.0,
        );
        if resp.clicked() {
            match icon {
                Icon::Grid => app.show_grid = !app.show_grid,
                _ => app.snap = !app.snap,
            }
        }
        let _ = resp.on_hover_text(tip);
        y += 38.0;
    }
}

fn zoom_pill(app: &mut App, ui: &mut Ui, rect: Rect) {
    let p = ui.painter().clone();
    p.rect(
        rect,
        egui::CornerRadius::same(9),
        Color32::from_rgb(0x16, 0x16, 0x16),
        Stroke::new(1.0, BORDER),
        StrokeKind::Inside,
    );
    let minus = Rect::from_center_size(pos2(rect.left() + 22.0, rect.center().y), Vec2::splat(28.0));
    let plus = Rect::from_center_size(pos2(rect.left() + 70.0, rect.center().y), Vec2::splat(28.0));
    let fit = Rect::from_center_size(pos2(rect.right() - 22.0, rect.center().y), Vec2::splat(28.0));
    for (r, icon, action) in [
        (minus, Icon::Minus, 0),
        (plus, Icon::Plus, 1),
        (fit, Icon::Fit, 2),
    ] {
        let resp = ui.interact(r, ui.id().with(("zoom", action)), Sense::click());
        if resp.hovered() {
            p.rect_filled(r, egui::CornerRadius::same(7), ELEV_2);
        }
        ui_kit::paint_icon(
            &p,
            Rect::from_center_size(r.center(), Vec2::splat(15.0)),
            icon,
            if resp.hovered() { TEXT } else { TEXT_DIM },
            1.0,
        );
        if resp.clicked() {
            match action {
                0 => app.zoom = (app.zoom / 1.15).max(0.25),
                1 => app.zoom = (app.zoom * 1.15).min(3.0),
                _ => app.fit_view(app.canvas_rect),
            }
        }
    }
    p.text(
        pos2(rect.left() + 100.0, rect.center().y),
        Align2::CENTER_CENTER,
        format!("{:.0}%", app.zoom * 100.0),
        FontId::new(10.0, egui::FontFamily::Monospace),
        TEXT_DIM,
    );
}

fn legend(app: &mut App, ui: &mut Ui, rect: Rect) {
    let p = ui.painter().clone();
    p.rect(
        rect,
        egui::CornerRadius::same(8),
        theme::with_alpha(Color32::from_rgb(0x16, 0x16, 0x16), 0.94),
        Stroke::new(1.0, BORDER),
        StrokeKind::Inside,
    );
    let items = [
        (HIGH, "HIGH"),
        (LOW, "LOW"),
        (MID, "ANALOG"),
    ];
    let mut x = rect.left() + 12.0;
    for (color, label) in items {
        p.circle_filled(pos2(x + 3.0, rect.center().y), 3.2, color);
        p.text(
            pos2(x + 11.0, rect.center().y),
            Align2::LEFT_CENTER,
            label,
            FontId::new(8.0, egui::FontFamily::Name("medium".into())),
            TEXT_FAINT,
        );
        x += 12.0 + label.len() as f32 * 5.0 + 12.0;
    }
    p.line_segment(
        [
            pos2(rect.right() - 74.0, rect.center().y - 7.0),
            pos2(rect.right() - 74.0, rect.center().y + 7.0),
        ],
        Stroke::new(1.0, BORDER_SOFT),
    );
    p.text(
        pos2(rect.right() - 14.0, rect.center().y),
        Align2::RIGHT_CENTER,
        if app.running { "LIVE" } else { "HELD" },
        FontId::new(8.0, egui::FontFamily::Name("medium".into())),
        if app.running { OK } else { TEXT_FAINT },
    );
}

fn sheet_chip(app: &mut App, ui: &mut Ui, rect: Rect) {
    let p = ui.painter().clone();
    p.text(
        pos2(rect.left(), rect.center().y - 6.0),
        Align2::LEFT_CENTER,
        "WORKSHEET",
        FontId::new(7.5, egui::FontFamily::Name("medium".into())),
        TEXT_FAINT,
    );
    p.text(
        pos2(rect.left(), rect.center().y + 8.0),
        Align2::LEFT_CENTER,
        &app.project.name,
        FontId::new(12.0, egui::FontFamily::Name("semibold".into())),
        TEXT_DIM,
    );
    p.text(
        pos2(rect.left() + 240.0, rect.center().y + 8.0),
        Align2::LEFT_CENTER,
        "SHEET 1 / 1",
        FontId::new(8.0, egui::FontFamily::Monospace),
        TEXT_FAINT,
    );
}

fn hint_chip(app: &mut App, ui: &mut Ui, rect: Rect, overlays: [Rect; 4]) {
    let text = if app.armed.is_some() {
        "Click to place  ·  Esc to cancel"
    } else if app.wire_start.is_some() {
        "Pick the second pin  ·  Esc to cancel"
    } else if let Some(Selection::Wire(_)) = app.selection {
        "Drag a wire segment to reshape it  ·  Del removes it"
    } else {
        ""
    };
    if text.is_empty() {
        return;
    }
    let p = ui.painter().clone();
    let font = FontId::new(10.0, egui::FontFamily::Name("medium".into()));
    let galley = p.layout_no_wrap(text.to_string(), font, Color32::from_rgb(0x16, 0x0b, 0x00));
    let size = galley.size() + vec2(24.0, 12.0);
    let center = pos2(rect.center().x, rect.bottom() - 58.0);
    let r = Rect::from_center_size(center, size);
    if overlays.iter().any(|o| o.intersects(r)) {
        return;
    }
    p.rect_filled(r, egui::CornerRadius::same(9), ACCENT);
    p.galley(r.min + vec2(12.0, 6.0), galley, Color32::from_rgb(0x16, 0x0b, 0x00));
}
