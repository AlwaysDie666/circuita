//! Hand drawn schematic symbols. Everything is authored in a local coordinate
//! space, then mirrored/rotated into the world so one drawing works for every
//! orientation.

use crate::model::{Component, Dir, Kind, PinRole};
use crate::sim::{Level, SimResult};
use crate::theme::{self, *};
use eframe::egui::{
    pos2, vec2, Align2, Color32, FontId, Painter, Pos2, Rect, Shape, Stroke, StrokeKind, Vec2,
};

pub struct DrawOpts<'a> {
    pub selected: bool,
    pub hovered: bool,
    pub ghost: bool,
    pub labels: bool,
    pub lit_override: Option<f32>,
    pub sim: Option<&'a SimResult>,
    pub pin_colors: Option<Vec<Color32>>,
    pub highlight_pin: Option<usize>,
    pub dim: bool,
}

impl Default for DrawOpts<'_> {
    fn default() -> Self {
        DrawOpts {
            selected: false,
            hovered: false,
            ghost: false,
            labels: true,
            lit_override: None,
            sim: None,
            pin_colors: None,
            highlight_pin: None,
            dim: false,
        }
    }
}

struct Sym<'a> {
    p: &'a Painter,
    comp: &'a Component,
    c: Pos2,
    z: f32,
    tint: Color32,
    alpha: f32,
}

impl<'a> Sym<'a> {
    fn lp(&self, x: f32, y: f32) -> Pos2 {
        let mut v = vec2(x, y);
        if self.comp.mirror {
            v.x = -v.x;
        }
        for _ in 0..(self.comp.rot.rem_euclid(4)) {
            v = vec2(-v.y, v.x);
        }
        self.c + v * self.z
    }

    fn lv(&self, x: f32, y: f32) -> Vec2 {
        let mut v = vec2(x, y);
        if self.comp.mirror {
            v.x = -v.x;
        }
        for _ in 0..(self.comp.rot.rem_euclid(4)) {
            v = vec2(-v.y, v.x);
        }
        v * self.z
    }

    fn col(&self, c: Color32) -> Color32 {
        if self.alpha >= 0.999 {
            c
        } else {
            with_alpha(c, self.alpha)
        }
    }

    fn sw(&self, w: f32) -> Stroke {
        Stroke::new(
            (w * self.z).clamp(1.05, 2.6),
            self.col(self.tint),
        )
    }

    fn swc(&self, w: f32, c: Color32) -> Stroke {
        Stroke::new((w * self.z).clamp(1.05, 2.8), self.col(c))
    }

    fn line(&self, a: (f32, f32), b: (f32, f32), w: f32) {
        self.p
            .line_segment([self.lp(a.0, a.1), self.lp(b.0, b.1)], self.sw(w));
    }

    fn line_c(&self, a: (f32, f32), b: (f32, f32), w: f32, c: Color32) {
        self.p
            .line_segment([self.lp(a.0, a.1), self.lp(b.0, b.1)], self.swc(w, c));
    }

    fn poly(&self, pts: &[(f32, f32)], w: f32) {
        let v: Vec<Pos2> = pts.iter().map(|(x, y)| self.lp(*x, *y)).collect();
        self.p.add(Shape::line(v, self.sw(w)));
    }

    fn poly_closed(&self, pts: &[(f32, f32)], fill: Color32, w: f32) {
        let v: Vec<Pos2> = pts.iter().map(|(x, y)| self.lp(*x, *y)).collect();
        self.p.add(Shape::convex_polygon(
            v,
            self.col(fill),
            self.sw(w),
        ));
    }

    fn circle(&self, x: f32, y: f32, r: f32, fill: Option<Color32>, w: f32) {
        let c = self.lp(x, y);
        let rr = r * self.z;
        if let Some(f) = fill {
            self.p.circle(c, rr, self.col(f), self.sw(w));
        } else {
            self.p.circle_stroke(c, rr, self.sw(w));
        }
    }

    fn circle_c(&self, x: f32, y: f32, r: f32, fill: Color32, w: f32, c: Color32) {
        self.p.circle(
            self.lp(x, y),
            r * self.z,
            self.col(fill),
            self.swc(w, c),
        );
    }

    fn rect(&self, x0: f32, y0: f32, x1: f32, y1: f32, fill: Option<Color32>, w: f32) {
        let a = self.lp(x0, y0);
        let b = self.lp(x1, y1);
        let r = Rect::from_two_pos(a, b);
        let radius = (5.0 * self.z).clamp(1.0, 10.0) as u8;
        match fill {
            Some(f) => {
                self.p.rect(
                    r,
                    egui::CornerRadius::same(radius),
                    self.col(f),
                    self.sw(w),
                    StrokeKind::Inside,
                );
            }
            None => {
                self.p.rect_stroke(
                    r,
                    egui::CornerRadius::same(radius),
                    self.sw(w),
                    StrokeKind::Inside,
                );
            }
        }
    }

    fn text(&self, x: f32, y: f32, text: &str, align: Align2, font: FontId, color: Color32) {
        self.p.text(
            self.lp(x, y),
            align,
            text,
            font,
            self.col(color),
        );
    }

    fn arc(&self, cx: f32, cy: f32, r: f32, a0: f32, a1: f32, w: f32, steps: usize) {
        let mut pts = Vec::new();
        for i in 0..=steps {
            let t = a0 + (a1 - a0) * (i as f32 / steps as f32);
            pts.push(self.lp(cx + r * t.cos(), cy + r * t.sin()));
        }
        self.p.add(Shape::line(pts, self.sw(w)));
    }

    fn bezier(&self, p0: (f32, f32), p1: (f32, f32), p2: (f32, f32), p3: (f32, f32), steps: usize) -> Vec<(f32, f32)> {
        let mut out = Vec::with_capacity(steps + 1);
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let mt = 1.0 - t;
            let x = mt * mt * mt * p0.0
                + 3.0 * mt * mt * t * p1.0
                + 3.0 * mt * t * t * p2.0
                + t * t * t * p3.0;
            let y = mt * mt * mt * p0.1
                + 3.0 * mt * mt * t * p1.1
                + 3.0 * mt * t * t * p2.1
                + t * t * t * p3.1;
            out.push((x, y));
        }
        out
    }

    fn arrow_head(&self, tip: (f32, f32), from: (f32, f32), size: f32) {
        let dx = tip.0 - from.0;
        let dy = tip.1 - from.1;
        let len = (dx * dx + dy * dy).sqrt().max(0.001);
        let ux = dx / len;
        let uy = dy / len;
        let px = -uy;
        let py = ux;
        let base = (tip.0 - ux * size, tip.1 - uy * size);
        let w = size * 0.55;
        self.poly_closed(
            &[
                tip,
                (base.0 + px * w, base.1 + py * w),
                (base.0 - px * w, base.1 - py * w),
            ],
            self.tint,
            0.8,
        );
    }
}

pub fn draw(painter: &Painter, comp: &Component, center: Pos2, zoom: f32, opts: &DrawOpts) {
    let lit = opts
        .lit_override
        .or_else(|| opts.sim.and_then(|s| s.led.get(&comp.id).copied()));
    let tint = if opts.ghost {
        theme::with_alpha(ACCENT, 0.85)
    } else if opts.selected {
        theme::mix(SYMBOL, ACCENT, 0.55)
    } else if opts.hovered {
        Color32::WHITE
    } else {
        SYMBOL
    };
    let alpha = if opts.ghost { 0.7 } else { 1.0 };
    let sym = Sym {
        p: painter,
        comp,
        c: center,
        z: zoom,
        tint: if opts.dim {
            theme::with_alpha(tint, 0.5)
        } else {
            tint
        },
        alpha,
    };

    match comp.kind {
        Kind::Resistor => draw_resistor(&sym),
        Kind::Capacitor => draw_capacitor(&sym),
        Kind::Inductor => draw_inductor(&sym),
        Kind::Potentiometer => draw_potentiometer(&sym),
        Kind::DcSource => draw_dc(&sym, comp),
        Kind::AcSource => draw_ac(&sym),
        Kind::Vcc => draw_vcc(&sym),
        Kind::Ground => draw_ground(&sym),
        Kind::Led => draw_led(&sym, lit.unwrap_or(0.0), opts.ghost),
        Kind::SwitchSpst => draw_switch(&sym, comp.state),
        Kind::GateAnd => draw_and(&sym, false),
        Kind::GateNand => draw_and(&sym, true),
        Kind::GateOr => draw_or(&sym, false),
        Kind::GateXor => draw_or(&sym, true),
        Kind::GateNot => draw_not(&sym),
        Kind::UnoBoard => draw_uno(&sym),
        Kind::Atmega => draw_atmega(&sym),
        Kind::Esp32 => draw_esp32(&sym),
        Kind::NanoBoard => draw_nano(&sym),
        Kind::Mpu6050 => draw_mpu(&sym),
        Kind::Ina219 => draw_ina(&sym),
        Kind::Dht22 => draw_dht(&sym),
        Kind::Lcd1602 => draw_lcd(&sym),
        Kind::PushButton => draw_button(&sym, comp.state),
        Kind::Custom => draw_custom(&sym, comp),
    }

    draw_pins(painter, comp, center, zoom, opts);
    if opts.labels {
        draw_labels(painter, comp, center, zoom, opts);
    }
}

fn body_rect(comp: &Component, center: Pos2, zoom: f32) -> Rect {
    let size = comp.size();
    Rect::from_center_size(center, size * zoom)
}

fn draw_labels(painter: &Painter, comp: &Component, center: Pos2, zoom: f32, opts: &DrawOpts) {
    let b = body_rect(comp, center, zoom);
    let color = if opts.selected {
        TEXT
    } else {
        TEXT_DIM
    };
    let color = if opts.ghost {
        theme::with_alpha(ACCENT, 0.8)
    } else {
        color
    };
    let ref_font = FontId::new(10.5, egui::FontFamily::Name("medium".into()));
    let val_font = FontId::new(9.5, egui::FontFamily::Proportional);
    let val_color = if opts.selected { TEXT_DIM } else { TEXT_FAINT };

    // Rails and grounds get their annotation beside or above the glyph so the
    // label never sits on the pin stub or the wire leaving it.
    if comp.kind == Kind::Vcc {
        painter.text(
            pos2(b.center().x, b.top() - 5.0),
            Align2::CENTER_BOTTOM,
            &comp.label,
            ref_font,
            color,
        );
        painter.text(
            pos2(b.center().x, b.top() - 17.0),
            Align2::CENTER_BOTTOM,
            &comp.value,
            val_font,
            val_color,
        );
        return;
    }
    if comp.kind == Kind::Ground {
        painter.text(
            pos2(b.right() + 6.0, b.center().y + 2.0),
            Align2::LEFT_CENTER,
            &comp.label,
            ref_font,
            color,
        );
        return;
    }

    let value = if comp.kind == Kind::SwitchSpst {
        if comp.state { "CLOSED".into() } else { "OPEN".into() }
    } else if comp.kind == Kind::AcSource {
        format!("{} · {}Hz", comp.value, comp.freq as i32)
    } else {
        comp.value.clone()
    };

    if comp.rot.rem_euclid(2) == 1 {
        // Vertical parts read best with the text stacked to their right.
        painter.text(
            pos2(b.right() + 6.0, b.center().y - 2.0),
            Align2::LEFT_BOTTOM,
            &comp.label,
            ref_font,
            color,
        );
        if !value.is_empty() {
            painter.text(
                pos2(b.right() + 6.0, b.center().y + 2.0),
                Align2::LEFT_TOP,
                value,
                val_font,
                val_color,
            );
        }
        return;
    }

    painter.text(
        pos2(b.left(), b.top() - 5.0),
        Align2::LEFT_BOTTOM,
        &comp.label,
        ref_font,
        color,
    );
    if !value.is_empty() {
        painter.text(
            pos2(b.left(), b.bottom() + 5.0),
            Align2::LEFT_TOP,
            value,
            val_font,
            val_color,
        );
    }
}

pub fn pin_screen(comp: &Component, pin: usize, center: Pos2, zoom: f32) -> Pos2 {
    center + comp.pin_offset(pin) * zoom
}

pub fn pin_dir_vec(comp: &Component, pin: usize) -> Vec2 {
    comp.pin_dir(pin).vec()
}

fn draw_pins(painter: &Painter, comp: &Component, center: Pos2, zoom: f32, opts: &DrawOpts) {
    for (i, pin) in comp.pins().iter().enumerate() {
        let pos = pin_screen(comp, i, center, zoom);
        let color = opts
            .pin_colors
            .as_ref()
            .and_then(|colors| colors.get(i).copied())
            .unwrap_or(Color32::from_rgb(0x8c, 0x8c, 0x8c));
        let r = if opts.highlight_pin == Some(i) {
            4.2
        } else if opts.selected || opts.hovered {
            3.2
        } else {
            2.6
        };
        if opts.highlight_pin == Some(i) {
            painter.circle_filled(pos, r + 3.5, theme::accent_glow(50));
        }
        painter.circle(
            pos,
            r,
            if opts.ghost {
                Color32::TRANSPARENT
            } else {
                theme::BG
            },
            Stroke::new(1.6, if opts.ghost { theme::with_alpha(ACCENT, 0.8) } else { color }),
        );

        if matches!(pin.role, PinRole::Power | PinRole::Ground | PinRole::Input | PinRole::Output)
            && !matches!(comp.kind, Kind::Vcc | Kind::Ground)
            || comp.kind.is_mcu()
        {
            let d = pin_dir_vec(comp, i);
            let anchor = pos + d * 7.0;
            let (align, off) = if d.x < -0.5 {
                (Align2::RIGHT_CENTER, vec2(-2.0, 0.0))
            } else if d.x > 0.5 {
                (Align2::LEFT_CENTER, vec2(2.0, 0.0))
            } else if d.y < -0.5 {
                (Align2::CENTER_BOTTOM, vec2(0.0, -2.0))
            } else {
                (Align2::CENTER_TOP, vec2(0.0, 2.0))
            };
            painter.text(
                anchor + off,
                align,
                pin.name,
                FontId::new(8.5, egui::FontFamily::Monospace),
                if opts.ghost {
                    theme::with_alpha(ACCENT, 0.7)
                } else {
                    TEXT_FAINT
                },
            );
        }
    }
}

fn stub(sym: &Sym, pin: (f32, f32), dir: Dir, len: f32, w: f32) {
    let v = dir.vec();
    sym.line(pin, (pin.0 + v.x * len, pin.1 + v.y * len), w);
}

fn draw_resistor(sym: &Sym) {
    stub(sym, (-40.0, 0.0), Dir::Right, 17.0, 1.7);
    stub(sym, (40.0, 0.0), Dir::Left, 17.0, 1.7);
    sym.rect(-23.0, -9.0, 23.0, 9.0, Some(SYMBOL_FILL), 1.7);
    // Three subtle calibration ticks keep the body from looking empty.
    for x in [-8.0, 0.0, 8.0] {
        sym.line((x, -4.0), (x, 4.0), 1.0);
    }
}

fn draw_capacitor(sym: &Sym) {
    sym.line((-40.0, 0.0), (-7.0, 0.0), 1.7);
    sym.line((40.0, 0.0), (7.0, 0.0), 1.7);
    sym.line_c((-7.0, -13.0), (-7.0, 13.0), 2.4, SYMBOL);
    sym.line_c((7.0, -13.0), (7.0, 13.0), 2.4, SYMBOL);
}

fn draw_inductor(sym: &Sym) {
    sym.line((-40.0, 0.0), (-18.0, 0.0), 1.7);
    sym.line((18.0, 0.0), (40.0, 0.0), 1.7);
    for i in 0..4 {
        let cx = -13.5 + i as f32 * 9.0;
        sym.arc(cx, 0.0, 4.6, std::f32::consts::PI, std::f32::consts::TAU, 1.7, 12);
    }
}

fn draw_potentiometer(sym: &Sym) {
    sym.line((-44.0, 0.0), (-23.0, 0.0), 1.7);
    sym.line((44.0, 0.0), (23.0, 0.0), 1.7);
    sym.rect(-23.0, -9.0, 23.0, 9.0, Some(SYMBOL_FILL), 1.7);
    sym.line((0.0, -34.0), (0.0, -13.0), 1.7);
    sym.arrow_head((0.0, -9.0), (0.0, -18.0), 6.0);
    sym.circle_c(0.0, -34.0, 2.4, BG, 1.4, SYMBOL);
}

fn draw_dc(sym: &Sym, comp: &Component) {
    sym.line((-40.0, 0.0), (-19.0, 0.0), 1.7);
    sym.line((19.0, 0.0), (40.0, 0.0), 1.7);
    sym.circle(0.0, 0.0, 19.0, Some(SYMBOL_FILL), 1.7);
    // Battery plates.
    sym.line_c((-5.0, -8.0), (-5.0, 8.0), 2.2, SYMBOL);
    sym.line_c((5.0, -4.0), (5.0, 4.0), 2.2, SYMBOL);
    sym.text(
        -14.0,
        -13.0,
        "+",
        Align2::CENTER_CENTER,
        FontId::new(10.0, egui::FontFamily::Monospace),
        ACCENT,
    );
    sym.text(
        14.0,
        13.0,
        "−",
        Align2::CENTER_CENTER,
        FontId::new(10.0, egui::FontFamily::Monospace),
        TEXT_DIM,
    );
    let _ = comp;
}

fn draw_ac(sym: &Sym) {
    sym.line((-40.0, 0.0), (-19.0, 0.0), 1.7);
    sym.line((19.0, 0.0), (40.0, 0.0), 1.7);
    sym.circle(0.0, 0.0, 19.0, Some(SYMBOL_FILL), 1.7);
    let mut pts = Vec::new();
    for i in 0..=28 {
        let t = i as f32 / 28.0;
        let x = -11.0 + 22.0 * t;
        let y = -(t * std::f32::consts::TAU).sin() * 6.5;
        pts.push((x, y));
    }
    sym.poly(&pts, 1.8);
}

fn draw_vcc(sym: &Sym) {
    sym.line((0.0, 24.0), (0.0, 4.0), 1.7);
    sym.line_c((-13.0, 3.0), (13.0, 3.0), 2.2, SYMBOL);
    sym.line((-6.0, -4.0), (6.0, -4.0), 1.4);
    sym.circle_c(0.0, -12.0, 2.2, BG, 1.2, ACCENT);
}

fn draw_ground(sym: &Sym) {
    sym.line((0.0, -26.0), (0.0, -7.0), 1.7);
    sym.line_c((-14.0, -7.0), (14.0, -7.0), 2.2, SYMBOL);
    sym.line((-9.0, -1.0), (9.0, -1.0), 1.8);
    sym.line((-4.5, 5.0), (4.5, 5.0), 1.6);
}

fn draw_led(sym: &Sym, brightness: f32, ghost: bool) {
    sym.line((-40.0, 0.0), (-15.0, 0.0), 1.7);
    sym.line((40.0, 0.0), (9.0, 0.0), 1.7);
    let b = brightness.clamp(0.0, 1.0);
    if b > 0.02 && !ghost {
        let c = sym.lp(0.0, 0.0);
        sym.p.circle_filled(
            c,
            (26.0 + 8.0 * b) * sym.z,
            theme::with_alpha(ACCENT, 0.10 + 0.22 * b),
        );
    }
    let fill = if b > 0.02 && !ghost {
        theme::mix(SYMBOL_FILL, ACCENT, 0.35 + 0.55 * b)
    } else {
        SYMBOL_FILL
    };
    let edge = if b > 0.02 && !ghost { ACCENT_HI } else { SYMBOL };
    let tri = [
        sym.lp(-15.0, -14.0),
        sym.lp(-15.0, 14.0),
        sym.lp(9.0, 0.0),
    ];
    sym.p.add(Shape::convex_polygon(
        tri.to_vec(),
        sym.col(fill),
        sym.swc(1.8, edge),
    ));
    sym.line_c((9.0, -14.0), (9.0, 14.0), 2.4, edge);
    // Emitted light arrows.
    sym.line_c((14.0, -12.0), (24.0, -22.0), 1.4, if b > 0.02 { ACCENT } else { SYMBOL });
    sym.arrow_head((25.0, -23.0), (18.0, -16.0), 5.0);
    sym.line_c((18.0, -6.0), (28.0, -16.0), 1.4, if b > 0.02 { ACCENT } else { SYMBOL });
    sym.arrow_head((29.0, -17.0), (22.0, -10.0), 5.0);
}

fn draw_switch(sym: &Sym, closed: bool) {
    sym.line((-40.0, 0.0), (-21.0, 0.0), 1.7);
    sym.line((40.0, 0.0), (21.0, 0.0), 1.7);
    sym.circle_c(-21.0, 0.0, 3.0, BG, 1.6, SYMBOL);
    sym.circle_c(21.0, 0.0, 3.0, BG, 1.6, SYMBOL);
    if closed {
        sym.line_c((-21.0, 0.0), (21.0, 0.0), 2.2, ACCENT);
    } else {
        sym.line_c((-21.0, 0.0), (16.0, -16.0), 2.2, SYMBOL);
        sym.circle_c(16.0, -16.0, 2.2, ACCENT, 1.0, ACCENT);
    }
}

fn gate_body(sym: &Sym, paths: Vec<Vec<(f32, f32)>>) {
    // Draw the outline as a filled shape then re-stroke it.
    for path in paths {
        let v: Vec<Pos2> = path.iter().map(|(x, y)| sym.lp(*x, *y)).collect();
        sym.p.add(Shape::convex_polygon(
            v.clone(),
            sym.col(SYMBOL_FILL),
            Stroke::NONE,
        ));
        sym.p.add(Shape::closed_line(v, sym.sw(1.8)));
    }
}

fn and_outline(sym: &Sym) -> Vec<(f32, f32)> {
    let mut pts = vec![(-26.0, -26.0)];
    pts.push((-2.0, -26.0));
    let arc = sym.bezier((-2.0, -26.0), (16.0, -26.0), (28.0, -14.0), (28.0, 0.0), 16);
    pts.extend(arc);
    let arc2 = sym.bezier((28.0, 0.0), (28.0, 14.0), (16.0, 26.0), (-2.0, 26.0), 16);
    pts.extend(arc2);
    pts.push((-26.0, 26.0));
    pts
}

fn draw_and(sym: &Sym, bubble: bool) {
    sym.line((-40.0, -14.0), (-26.0, -14.0), 1.7);
    sym.line((-40.0, 14.0), (-26.0, 14.0), 1.7);
    let outline = and_outline(sym);
    let v: Vec<Pos2> = outline.iter().map(|(x, y)| sym.lp(*x, *y)).collect();
    sym.p
        .add(Shape::convex_polygon(v.clone(), sym.col(SYMBOL_FILL), Stroke::NONE));
    sym.p.add(Shape::closed_line(v, sym.sw(1.8)));
    if bubble {
        sym.circle_c(32.0, 0.0, 4.0, SYMBOL_FILL, 1.8, sym.tint);
        sym.line((36.0, 0.0), (40.0, 0.0), 1.7);
    } else {
        sym.line((28.0, 0.0), (40.0, 0.0), 1.7);
    }
}

fn draw_or(sym: &Sym, xor: bool) {
    sym.line((-40.0, -14.0), (-18.0, -14.0), 1.7);
    sym.line((-40.0, 14.0), (-18.0, 14.0), 1.7);
    let mut path = Vec::new();
    path.push((-26.0, -26.0));
    let a = sym.bezier((-26.0, -26.0), (2.0, -24.0), (16.0, -14.0), (28.0, 0.0), 18);
    path.extend(a);
    let b = sym.bezier((28.0, 0.0), (16.0, 14.0), (2.0, 24.0), (-26.0, 26.0), 18);
    path.extend(b);
    let c = sym.bezier((-26.0, 26.0), (-10.0, 12.0), (-10.0, -12.0), (-26.0, -26.0), 18);
    path.extend(c);
    let v: Vec<Pos2> = path.iter().map(|(x, y)| sym.lp(*x, *y)).collect();
    sym.p
        .add(Shape::convex_polygon(v.clone(), sym.col(SYMBOL_FILL), Stroke::NONE));
    sym.p.add(Shape::closed_line(v, sym.sw(1.8)));
    if xor {
        let arc = sym.bezier((-33.0, -26.0), (-17.0, -12.0), (-17.0, 12.0), (-33.0, 26.0), 18);
        let p: Vec<Pos2> = arc.iter().map(|(x, y)| sym.lp(*x, *y)).collect();
        sym.p.add(Shape::line(p, sym.sw(1.6)));
    }
    sym.line((28.0, 0.0), (40.0, 0.0), 1.7);
}

fn draw_not(sym: &Sym) {
    sym.line((-40.0, 0.0), (-22.0, 0.0), 1.7);
    let tri = [
        sym.lp(-22.0, -20.0),
        sym.lp(-22.0, 20.0),
        sym.lp(18.0, 0.0),
    ];
    sym.p.add(Shape::convex_polygon(
        tri.to_vec(),
        sym.col(SYMBOL_FILL),
        sym.sw(1.8),
    ));
    sym.circle_c(22.0, 0.0, 4.0, SYMBOL_FILL, 1.8, sym.tint);
    sym.line((26.0, 0.0), (40.0, 0.0), 1.7);
}

fn draw_uno(sym: &Sym) {
    sym.rect(-110.0, -70.0, 110.0, 70.0, Some(Color32::from_rgb(0x17, 0x1b, 0x19)), 1.8);
    // Pin headers.
    sym.rect(-92.0, -64.0, 92.0, -50.0, Some(Color32::from_rgb(0x22, 0x27, 0x23)), 1.0);
    sym.rect(-92.0, 50.0, 92.0, 64.0, Some(Color32::from_rgb(0x22, 0x27, 0x23)), 1.0);
    for i in 0..14 {
        let x = -84.0 + i as f32 * 12.8;
        sym.rect(x, -61.0, x + 5.0, -55.0, Some(Color32::from_rgb(0x3a, 0x3f, 0x39)), 0.0);
        sym.rect(x, 55.0, x + 5.0, 61.0, Some(Color32::from_rgb(0x3a, 0x3f, 0x39)), 0.0);
    }
    // USB socket and barrel jack.
    sym.rect(-108.0, -22.0, -80.0, 16.0, Some(Color32::from_rgb(0x2b, 0x2f, 0x2c)), 1.2);
    sym.rect(-70.0, 22.0, -46.0, 46.0, Some(Color32::from_rgb(0x2b, 0x2f, 0x2c)), 1.2);
    sym.circle_c(-58.0, 34.0, 7.0, Color32::from_rgb(0x17, 0x1b, 0x19), 1.4, BORDER_HI);
    // MCU and support parts.
    sym.rect(-14.0, 4.0, 38.0, 46.0, Some(Color32::from_rgb(0x22, 0x22, 0x22)), 1.4);
    sym.circle_c(12.0, 34.0, 2.0, BORDER_HI, 0.8, BORDER_HI);
    sym.rect(52.0, 6.0, 80.0, 40.0, Some(Color32::from_rgb(0x20, 0x20, 0x20)), 1.2);
    // Silkscreen.
    sym.text(
        -100.0,
        -34.0,
        "UNO",
        Align2::LEFT_CENTER,
        FontId::new(16.0, egui::FontFamily::Name("bold".into())),
        ACCENT,
    );
    sym.text(
        -100.0,
        -20.0,
        "ATmega328P",
        Align2::LEFT_CENTER,
        FontId::new(8.0, egui::FontFamily::Monospace),
        TEXT_FAINT,
    );
    sym.text(
        -33.0,
        -8.0,
        "DIGITAL",
        Align2::LEFT_CENTER,
        FontId::new(7.0, egui::FontFamily::Monospace),
        Color32::from_rgb(0x59, 0x60, 0x5a),
    );
    // Pins stubs out to the header edges.
    sym.line((-130.0, -38.0), (-110.0, -38.0), 1.7);
    sym.line((-130.0, 38.0), (-110.0, 38.0), 1.7);
    sym.line((130.0, -38.0), (110.0, -38.0), 1.7);
    sym.line((130.0, 38.0), (110.0, 38.0), 1.7);
}

fn draw_atmega(sym: &Sym) {
    sym.rect(-38.0, -60.0, 38.0, 60.0, Some(SYMBOL_FILL), 1.8);
    // DIP notch.
    sym.arc(0.0, -60.0, 9.0, 0.05, std::f32::consts::PI - 0.05, 1.6, 14);
    sym.circle_c(-26.0, -60.0, 2.2, BG, 1.2, BORDER_HI);
    for i in 0..6 {
        let y = -50.0 + i as f32 * 19.0;
        sym.line((-52.0, y), (-38.0, y), 1.4);
        sym.line((52.0, y), (38.0, y), 1.4);
    }
    sym.text(
        0.0,
        0.0,
        "ATmega",
        Align2::CENTER_CENTER,
        FontId::new(9.0, egui::FontFamily::Monospace),
        TEXT_DIM,
    );
    sym.text(
        0.0,
        11.0,
        "328P",
        Align2::CENTER_CENTER,
        FontId::new(9.0, egui::FontFamily::Monospace),
        TEXT_DIM,
    );
    sym.line((-66.0, -38.0), (-52.0, -38.0), 1.7);
    sym.line((-66.0, 38.0), (-52.0, 38.0), 1.7);
    sym.line((66.0, -38.0), (52.0, -38.0), 1.7);
    sym.line((66.0, 38.0), (52.0, 38.0), 1.7);
}

fn draw_esp32(sym: &Sym) {
    sym.rect(-115.0, -130.0, 115.0, 130.0, Some(SYMBOL_FILL), 1.8);
    sym.rect(-104.0, -118.0, 104.0, 118.0, Some(Color32::from_rgb(0x1d, 0x22, 0x20)), 1.2);
    // Antenna trace.
    sym.poly(
        &[
            (-58.0, -118.0),
            (-58.0, -134.0),
            (58.0, -134.0),
            (58.0, -118.0),
        ],
        1.6,
    );
    sym.text(
        0.0,
        -80.0,
        "ESP32",
        Align2::CENTER_CENTER,
        FontId::new(16.0, egui::FontFamily::Name("bold".into())),
        ACCENT,
    );
    sym.text(
        0.0,
        -60.0,
        "DEVKIT v1",
        Align2::CENTER_CENTER,
        FontId::new(9.0, egui::FontFamily::Monospace),
        TEXT_DIM,
    );
    sym.rect(-60.0, -30.0, 60.0, 70.0, Some(Color32::from_rgb(0x16, 0x1a, 0x18)), 1.2);
    sym.text(
        0.0,
        18.0,
        "Wi-Fi + BLE",
        Align2::CENTER_CENTER,
        FontId::new(8.0, egui::FontFamily::Monospace),
        TEXT_FAINT,
    );
    // Header rails matching the Wokwi pin rows.
    sym.rect(-115.0, -76.0, -101.0, 190.0, Some(Color32::from_rgb(0x22, 0x27, 0x23)), 1.0);
    sym.rect(101.0, -76.0, 115.0, 190.0, Some(Color32::from_rgb(0x22, 0x27, 0x23)), 1.0);
}

fn draw_nano(sym: &Sym) {
    sym.rect(-150.0, -55.0, 150.0, 55.0, Some(Color32::from_rgb(0x16, 0x1b, 0x18)), 1.8);
    sym.rect(-148.0, -48.0, 148.0, -34.0, Some(Color32::from_rgb(0x22, 0x27, 0x23)), 1.0);
    sym.rect(-148.0, 34.0, 148.0, 48.0, Some(Color32::from_rgb(0x22, 0x27, 0x23)), 1.0);
    for i in 0..15 {
        let x = -132.0 + i as f32 * 18.0;
        sym.rect(x, -46.0, x + 8.0, -36.0, Some(Color32::from_rgb(0x3a, 0x3f, 0x39)), 0.0);
        sym.rect(x, 36.0, x + 8.0, 46.0, Some(Color32::from_rgb(0x3a, 0x3f, 0x39)), 0.0);
    }
    sym.text(
        -140.0,
        0.0,
        "NANO",
        Align2::LEFT_CENTER,
        FontId::new(18.0, egui::FontFamily::Name("bold".into())),
        ACCENT,
    );
    sym.text(
        -140.0,
        18.0,
        "ATmega328P",
        Align2::LEFT_CENTER,
        FontId::new(8.0, egui::FontFamily::Monospace),
        TEXT_FAINT,
    );
}

fn draw_mpu(sym: &Sym) {
    sym.rect(-80.0, -46.0, 80.0, 46.0, Some(SYMBOL_FILL), 1.8);
    sym.rect(-60.0, -34.0, 60.0, 34.0, Some(Color32::from_rgb(0x18, 0x1d, 0x1a)), 1.2);
    sym.circle_c(0.0, 0.0, 14.0, Color32::from_rgb(0x20, 0x26, 0x22), 1.2, BORDER_HI);
    sym.line((-8.0, 0.0), (8.0, 0.0), 1.2);
    sym.line((0.0, -8.0), (0.0, 8.0), 1.2);
    sym.text(
        0.0,
        -22.0,
        "MPU6050",
        Align2::CENTER_CENTER,
        FontId::new(11.0, egui::FontFamily::Name("bold".into())),
        TEXT_DIM,
    );
    sym.text(
        0.0,
        30.0,
        "I2C 0x68",
        Align2::CENTER_CENTER,
        FontId::new(8.0, egui::FontFamily::Monospace),
        TEXT_FAINT,
    );
}

fn draw_ina(sym: &Sym) {
    sym.rect(-90.0, -44.0, 90.0, 44.0, Some(SYMBOL_FILL), 1.8);
    sym.rect(-30.0, -30.0, 30.0, 30.0, Some(Color32::from_rgb(0x1c, 0x21, 0x1e)), 1.2);
    sym.text(
        0.0,
        -6.0,
        "INA219",
        Align2::CENTER_CENTER,
        FontId::new(12.0, egui::FontFamily::Name("bold".into())),
        ACCENT,
    );
    sym.text(
        0.0,
        12.0,
        "V/I MONITOR",
        Align2::CENTER_CENTER,
        FontId::new(7.0, egui::FontFamily::Monospace),
        TEXT_FAINT,
    );
}

fn draw_dht(sym: &Sym) {
    sym.rect(-60.0, -70.0, 60.0, 70.0, Some(Color32::from_rgb(0x1a, 0x1e, 0x1c)), 1.8);
    sym.rect(-44.0, -44.0, 44.0, 26.0, Some(Color32::from_rgb(0x24, 0x28, 0x25)), 1.2);
    for i in 0..4 {
        let y = -34.0 + i as f32 * 14.0;
        sym.line((-30.0, y), (30.0, y), 0.8);
    }
    sym.text(
        0.0,
        46.0,
        "DHT22",
        Align2::CENTER_CENTER,
        FontId::new(12.0, egui::FontFamily::Name("bold".into())),
        ACCENT,
    );
}

fn draw_lcd(sym: &Sym) {
    sym.rect(-140.0, -50.0, 140.0, 50.0, Some(Color32::from_rgb(0x12, 0x17, 0x15)), 1.8);
    sym.rect(-124.0, -36.0, 124.0, 32.0, Some(Color32::from_rgb(0x1d, 0x26, 0x21)), 1.2);
    for i in 0..2 {
        let y = -18.0 + i as f32 * 26.0;
        sym.text(
            -112.0,
            y,
            "8888888888888888",
            Align2::LEFT_CENTER,
            FontId::new(11.0, egui::FontFamily::Monospace),
            Color32::from_rgb(0x39, 0x66, 0x4b),
        );
    }
    sym.text(
        130.0,
        40.0,
        "LCD 1602",
        Align2::RIGHT_CENTER,
        FontId::new(9.0, egui::FontFamily::Monospace),
        TEXT_FAINT,
    );
}

fn draw_button(sym: &Sym, pressed: bool) {
    sym.rect(-50.0, -40.0, 50.0, 40.0, Some(SYMBOL_FILL), 1.8);
    sym.circle_c(0.0, 0.0, 18.0, Color32::from_rgb(0x25, 0x2a, 0x26), 1.4, BORDER_HI);
    if !pressed {
        sym.circle_c(0.0, -3.0, 13.0, Color32::from_rgb(0x33, 0x39, 0x34), 1.0, BORDER_HI);
    }
    sym.line((-50.0, -40.0), (-50.0, -56.0), 1.5);
    sym.line((50.0, -40.0), (50.0, -56.0), 1.5);
    sym.line((-50.0, 40.0), (-50.0, 56.0), 1.5);
    sym.line((50.0, 40.0), (50.0, 56.0), 1.5);
}

fn draw_custom(sym: &Sym, comp: &Component) {
    let size = comp.size();
    let hx = size.x / 2.0;
    let hy = size.y / 2.0;
    sym.rect(-hx, -hy, hx, hy, Some(SYMBOL_FILL), 1.8);
    let title = comp.label.chars().take(14).collect::<String>();
    sym.text(
        0.0,
        -6.0,
        &title,
        Align2::CENTER_CENTER,
        FontId::new(10.0, egui::FontFamily::Name("bold".into())),
        ACCENT,
    );
    let value = comp.value.chars().take(16).collect::<String>();
    sym.text(
        0.0,
        10.0,
        &value,
        Align2::CENTER_CENTER,
        FontId::new(8.0, egui::FontFamily::Monospace),
        TEXT_FAINT,
    );
}

/// Level colour used by the canvas, instruments and the legend.
pub fn level_color(level: Level) -> Color32 {
    match level {
        Level::High => HIGH,
        Level::Low => LOW,
        Level::Mid => MID,
    }
}
