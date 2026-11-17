//! Floating virtual instruments: voltmeter, ammeter, logic probe and a compact
//! two channel oscilloscope. They are draggable panels that always sit above the
//! schematic canvas.

use crate::app::App;
use crate::model::{Kind, PinRef};
use crate::sim::{Netlist, SimResult};
use crate::theme;
use crate::theme::*;
use crate::ui_kit::{self, Icon};
use eframe::egui::{
    self, pos2, vec2, Align2, Color32, Context, FontId, Pos2, Rect, Sense, Stroke, Ui, Vec2,
};

#[derive(Clone, Debug)]
pub struct Instruments {
    pub bound: bool,
    pub volt_open: bool,
    pub volt_pos: Option<Pos2>,
    pub volt_plus: usize,
    pub volt_minus: usize,
    pub amp_open: bool,
    pub amp_pos: Option<Pos2>,
    pub amp_comp: Option<usize>,
    pub probe_open: bool,
    pub probe_pos: Option<Pos2>,
    pub probe_net: usize,
    pub scope_open: bool,
    pub scope_pos: Option<Pos2>,
    pub scope_ch: [usize; 2],
    pub traces: [Vec<f32>; 2],
    pub sample_acc: f32,
}

impl Default for Instruments {
    fn default() -> Self {
        Instruments {
            bound: false,
            volt_open: true,
            volt_pos: None,
            volt_plus: 1,
            volt_minus: 0,
            amp_open: false,
            amp_pos: None,
            amp_comp: None,
            probe_open: false,
            probe_pos: None,
            probe_net: 1,
            scope_open: true,
            scope_pos: None,
            scope_ch: [1, 2],
            traces: [Vec::new(), Vec::new()],
            sample_acc: 0.0,
        }
    }
}

impl Instruments {
    pub fn bind_defaults(&mut self, project: &crate::model::Project, netlist: &Netlist) {
        let net_of = |c: &crate::model::Component, pin: usize| {
            netlist
                .pin_net
                .get(&PinRef {
                    comp: c.id,
                    pin,
                })
                .copied()
                .unwrap_or(0)
        };
        if let Some(ac) = project.comps.iter().find(|c| c.kind == Kind::AcSource) {
            self.scope_ch[0] = net_of(ac, 0);
            self.volt_plus = net_of(ac, 0);
        }
        if let Some(mcu) = project.comps.iter().find(|c| c.kind.is_mcu()) {
            self.scope_ch[1] = net_of(mcu, 2);
            self.probe_net = net_of(mcu, 2);
        }
        self.volt_minus = 0;
        self.amp_comp = project
            .comps
            .iter()
            .find(|c| c.kind == Kind::Resistor)
            .map(|c| c.id);
        self.bound = true;
    }

    pub fn clamp_nets(&mut self, netlist: &Netlist) {
        let n = netlist.nets.len();
        if n == 0 {
            return;
        }
        self.volt_plus = self.volt_plus.min(n - 1);
        self.volt_minus = self.volt_minus.min(n - 1);
        self.probe_net = self.probe_net.min(n - 1);
        self.scope_ch[0] = self.scope_ch[0].min(n - 1);
        self.scope_ch[1] = self.scope_ch[1].min(n - 1);
    }

    pub fn sample(&mut self, netlist: &Netlist, result: &SimResult) {
        let _ = netlist;
        for ch in 0..2 {
            let v = result.volt(self.scope_ch[ch]) as f32;
            self.traces[ch].push(v);
            if self.traces[ch].len() > 420 {
                self.traces[ch].remove(0);
            }
        }
    }

    pub fn clear_traces(&mut self) {
        self.traces = [Vec::new(), Vec::new()];
    }

    pub fn toggle(&mut self, which: usize) {
        match which {
            0 => self.volt_open = !self.volt_open,
            1 => self.amp_open = !self.amp_open,
            2 => self.probe_open = !self.probe_open,
            3 => self.scope_open = !self.scope_open,
            _ => {}
        }
    }
}

pub fn show(app: &mut App, ctx: &Context) {
    let rect = app.canvas_rect;
    // Instruments live in the free band on the right of the sheet, clear of the
    // demo circuit, and can be dragged anywhere from there.
    let default_scope = pos2(rect.right() - 14.0 - 390.0, rect.bottom() - 14.0 - 396.0);
    let default_volt = pos2(rect.right() - 14.0 - 250.0, rect.top() + 16.0);
    let default_amp = pos2(rect.right() - 14.0 - 250.0, rect.top() + 246.0);
    let default_probe = pos2(rect.right() - 14.0 - 222.0, rect.top() + 452.0);

    if !app.instruments.bound {
        app.instruments.bind_defaults(&app.project, &app.netlist);
    }

    if app.instruments.volt_open {
        voltmeter(app, ctx, default_volt);
    }
    if app.instruments.amp_open {
        ammeter(app, ctx, default_amp);
    }
    if app.instruments.probe_open {
        logic_probe(app, ctx, default_probe);
    }
    if app.instruments.scope_open {
        oscilloscope(app, ctx, default_scope);
    }
}

fn floating(
    ctx: &Context,
    id: &str,
    title: &str,
    icon: Icon,
    accent: Color32,
    pos: &mut Option<Pos2>,
    default: Pos2,
    open: &mut bool,
    width: f32,
    add: impl FnOnce(&mut Ui),
) {
    let mut p = pos.unwrap_or(default);
    let id = egui::Id::new(("instrument", id));
    let area = egui::Area::new(id)
        .order(egui::Order::Foreground)
        .fixed_pos(p)
        .constrain(true)
        .show(ctx, |ui| {
            ui.set_width(width);
            egui::Frame::new()
                .fill(theme::PANEL_2)
                .stroke(Stroke::new(1.0, BORDER))
                .corner_radius(egui::CornerRadius::same(12))
                .inner_margin(egui::Margin::same(12))
                .shadow(egui::epaint::Shadow {
                    offset: [0, 10],
                    blur: 26,
                    spread: 0,
                    color: Color32::from_black_alpha(140),
                })
                .show(ui, |ui| {
                    let header = ui.horizontal(|ui| {
                        let (ir, _) = ui.allocate_exact_size(vec2(18.0, 18.0), Sense::hover());
                        ui_kit::paint_icon(ui.painter(), ir, icon, accent, 1.0);
                        ui.add_space(2.0);
                        ui.label(
                            egui::RichText::new(title)
                                .font(display(11.0))
                                .color(TEXT),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let (r, resp) =
                                ui.allocate_exact_size(vec2(20.0, 20.0), Sense::click());
                            let col = if resp.hovered() { ACCENT } else { TEXT_FAINT };
                            ui_kit::paint_icon(ui.painter(), r, Icon::Close, col, 0.8);
                            if resp.clicked() {
                                *open = false;
                            }
                        });
                    });
                    let hr = header.response.rect;
                    let drag = ui.interact(
                        Rect::from_min_max(pos2(hr.left(), hr.top() - 6.0), hr.right_bottom()),
                        id.with("drag"),
                        Sense::click_and_drag(),
                    );
                    if drag.dragged() {
                        p += drag.drag_delta();
                        *pos = Some(p);
                    }
                    ui.add_space(6.0);
                    add(ui);
                });
        });
    let _ = area;
    if pos.is_none() {
        *pos = Some(default);
    }
}

fn net_name(netlist: &Netlist, net: usize) -> String {
    netlist
        .nets
        .get(net)
        .map(|n| n.name.clone())
        .unwrap_or_else(|| "—".into())
}

fn net_combo(ui: &mut Ui, id: &str, netlist: &Netlist, value: &mut usize, width: f32) {
    egui::ComboBox::from_id_salt(id)
        .width(width)
        .selected_text(
            egui::RichText::new(net_name(netlist, *value))
                .font(mono(11.0))
                .color(TEXT),
        )
        .show_ui(ui, |ui| {
            for net in &netlist.nets {
                ui.selectable_value(
                    value,
                    net.id,
                    egui::RichText::new(format!("{}  ·  {:.2} V", net.name, 0.0)).font(mono(11.0)),
                );
            }
        });
}

fn reading(ui: &mut Ui, value: &str, unit: &str, color: Color32, hint: &str) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(value)
                .font(display(30.0))
                .color(color),
        );
        ui.vertical(|ui| {
            ui.add_space(7.0);
            ui.label(egui::RichText::new(unit).font(medium(12.0)).color(TEXT_DIM));
        });
    });
    ui.label(egui::RichText::new(hint).font(body(10.5)).color(TEXT_FAINT));
}

fn meter_bar(ui: &mut Ui, fraction: f32, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 6.0), Sense::hover());
    let p = ui.painter();
    p.rect_filled(rect, 3, theme::ELEV);
    let f = fraction.clamp(0.0, 1.0);
    if f > 0.01 {
        let filled = Rect::from_min_size(rect.min, vec2(rect.width() * f, rect.height()));
        p.rect_filled(filled, 3, theme::with_alpha(color, 0.85));
    }
    for i in 1..10 {
        let x = rect.left() + rect.width() * i as f32 / 10.0;
        p.line_segment(
            [pos2(x, rect.top()), pos2(x, rect.bottom())],
            Stroke::new(1.0, theme::PANEL_2),
        );
    }
}

fn voltmeter(app: &mut App, ctx: &Context, default: Pos2) {
    let plus = app.instruments.volt_plus;
    let minus = app.instruments.volt_minus;
    let v = app.result.volt(plus) - app.result.volt(minus);
    let color = if v.abs() > 0.5 { ACCENT_HI } else { TEXT };
    let mut open = app.instruments.volt_open;
    let mut pos = app.instruments.volt_pos;
    let level = app
        .netlist
        .nets
        .get(plus)
        .map(|_| app.result.level(plus))
        .unwrap_or(crate::sim::Level::Low);
    floating(
        ctx,
        "voltmeter",
        "VOLTMETER",
        Icon::Voltmeter,
        ACCENT,
        &mut pos,
        default,
        &mut open,
        250.0,
        |ui| {
            reading(ui, &format!("{:.3}", v), "V", color, "Differential probe");
            ui.add_space(4.0);
            meter_bar(ui, (v.abs() / 10.0) as f32, crate::symbols::level_color(level));
            ui.add_space(8.0);
            egui::Grid::new("vm_grid")
                .num_columns(2)
                .spacing(vec2(8.0, 6.0))
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("V+").font(mono(10.5)).color(HIGH));
                    net_combo(ui, "vm_plus", &app.netlist, &mut app.instruments.volt_plus, 150.0);
                    ui.end_row();
                    ui.label(egui::RichText::new("V−").font(mono(10.5)).color(LOW));
                    net_combo(ui, "vm_minus", &app.netlist, &mut app.instruments.volt_minus, 150.0);
                    ui.end_row();
                });
        },
    );
    app.instruments.volt_pos = pos;
    app.instruments.volt_open = open;
}

fn ammeter(app: &mut App, ctx: &Context, default: Pos2) {
    let mut open = app.instruments.amp_open;
    let mut pos = app.instruments.amp_pos;
    let comp_id = app.instruments.amp_comp;
    let current = comp_id.map(|id| app.result.comp_amp(id)).unwrap_or(0.0);
    let label = comp_id
        .and_then(|id| app.project.comp(id))
        .map(|c| format!("{} · {}", c.label, c.kind.spec().name))
        .unwrap_or_else(|| "No branch".into());
    let color = if current.abs() > 0.02 {
        Color32::from_rgb(0xff, 0xb2, 0x5c)
    } else {
        TEXT
    };
    floating(
        ctx,
        "ammeter",
        "AMMETER",
        Icon::Ammeter,
        Color32::from_rgb(0xff, 0xb2, 0x5c),
        &mut pos,
        default,
        &mut open,
        250.0,
        |ui| {
            let text = if current.abs() < 1.0 {
                format!("{:.2}", current.abs() * 1000.0)
            } else {
                format!("{:.2}", current.abs())
            };
            let unit = if current.abs() < 1.0 { "mA" } else { "A" };
            reading(ui, &text, unit, color, &label);
            ui.add_space(4.0);
            meter_bar(ui, (current.abs() / 0.05) as f32, ACCENT);
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("BRANCH").font(mono(9.5)).color(TEXT_FAINT));
                let current_name = comp_id
                    .and_then(|id| app.project.comp(id))
                    .map(|c| c.label.clone())
                    .unwrap_or_default();
                egui::ComboBox::from_id_salt("am_pick")
                    .width(140.0)
                    .selected_text(
                        egui::RichText::new(current_name)
                            .font(mono(11.0))
                            .color(TEXT),
                    )
                    .show_ui(ui, |ui| {
                        let options: Vec<(usize, String)> = app
                            .project
                            .comps
                            .iter()
                            .filter(|c| {
                                matches!(
                                    c.kind,
                                    Kind::Resistor
                                        | Kind::Led
                                        | Kind::SwitchSpst
                                        | Kind::Inductor
                                        | Kind::DcSource
                                        | Kind::AcSource
                                        | Kind::Potentiometer
                                )
                            })
                            .map(|c| (c.id, c.label.clone()))
                            .collect();
                        for (id, name) in options {
                            ui.selectable_value(&mut app.instruments.amp_comp, Some(id), name);
                        }
                    });
            });
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(if current >= 0.0 {
                    "Current flow →"
                } else {
                    "← Current flow"
                })
                .font(body(10.0))
                .color(TEXT_FAINT),
            );
        },
    );
    app.instruments.amp_pos = pos;
    app.instruments.amp_open = open;
}

fn logic_probe(app: &mut App, ctx: &Context, default: Pos2) {
    let mut open = app.instruments.probe_open;
    let mut pos = app.instruments.probe_pos;
    let net = app.instruments.probe_net;
    let level = app.result.level(net);
    let color = crate::symbols::level_color(level);
    let label = match level {
        crate::sim::Level::High => "HIGH",
        crate::sim::Level::Low => "LOW",
        crate::sim::Level::Mid => "FLOAT",
    };
    let v = app.result.volt(net);
    let pulse = (app.clock * 2.4).sin() as f32 * 0.5 + 0.5;
    floating(
        ctx,
        "logicprobe",
        "LOGIC PROBE",
        Icon::Probe,
        color,
        &mut pos,
        default,
        &mut open,
        222.0,
        |ui| {
            ui.horizontal(|ui| {
                let (r, _) = ui.allocate_exact_size(vec2(46.0, 46.0), Sense::hover());
                let p = ui.painter();
                let center = r.center();
                p.circle_filled(center, 17.0, theme::with_alpha(color, 0.16));
                p.circle_filled(center, 11.0 + pulse * 2.0, theme::with_alpha(color, 0.3));
                p.circle_filled(center, 6.5, color);
                ui.vertical(|ui| {
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new(label).font(display(22.0)).color(color));
                    ui.label(
                        egui::RichText::new(format!("{:.2} V · {}", v, net_name(&app.netlist, net)))
                            .font(mono(10.5))
                            .color(TEXT_DIM),
                    );
                });
            });
            ui.add_space(8.0);
            net_combo(ui, "probe_net", &app.netlist, &mut app.instruments.probe_net, 188.0);
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("Red is high, blue is low")
                    .font(body(9.5))
                    .color(TEXT_FAINT),
            );
        },
    );
    app.instruments.probe_pos = pos;
    app.instruments.probe_open = open;
}

fn oscilloscope(app: &mut App, ctx: &Context, default: Pos2) {
    let mut open = app.instruments.scope_open;
    let mut pos = app.instruments.scope_pos;
    let ch1 = app.instruments.scope_ch[0];
    let ch2 = app.instruments.scope_ch[1];
    floating(
        ctx,
        "scope",
        "OSCILLOSCOPE",
        Icon::Scope,
        ACCENT,
        &mut pos,
        default,
        &mut open,
        366.0,
        |ui| {
            let (rect, _) = ui.allocate_exact_size(vec2(342.0, 168.0), Sense::hover());
            let p = ui.painter_at(rect);
            p.rect_filled(rect, 8, Color32::from_rgb(0x0c, 0x0c, 0x0c));
            p.rect_stroke(
                rect,
                8,
                Stroke::new(1.0, BORDER_SOFT),
                egui::StrokeKind::Inside,
            );
            let inner = rect.shrink(10.0);
            for i in 0..=8 {
                let x = inner.left() + inner.width() * i as f32 / 8.0;
                p.line_segment(
                    [pos2(x, inner.top()), pos2(x, inner.bottom())],
                    Stroke::new(1.0, Color32::from_rgb(0x1c, 0x1c, 0x1c)),
                );
            }
            for i in 0..=6 {
                let y = inner.top() + inner.height() * i as f32 / 6.0;
                p.line_segment(
                    [pos2(inner.left(), y), pos2(inner.right(), y)],
                    Stroke::new(
                        1.0,
                        if i == 3 {
                            Color32::from_rgb(0x2c, 0x2c, 0x2c)
                        } else {
                            Color32::from_rgb(0x1a, 0x1a, 0x1a)
                        },
                    ),
                );
            }
            // Volt scale labels.
            p.text(
                pos2(inner.left() + 4.0, inner.top() + 2.0),
                Align2::LEFT_TOP,
                "+6V",
                FontId::new(8.0, egui::FontFamily::Monospace),
                TEXT_FAINT,
            );
            p.text(
                pos2(inner.left() + 4.0, inner.bottom() - 2.0),
                Align2::LEFT_BOTTOM,
                "−6V",
                FontId::new(8.0, egui::FontFamily::Monospace),
                TEXT_FAINT,
            );

            let colors = [ACCENT_HI, Color32::from_rgb(0x39, 0xd0, 0xff)];
            for ch in 0..2 {
                let data = &app.instruments.traces[ch];
                if data.len() < 2 {
                    continue;
                }
                let n = data.len();
                let mut pts = Vec::with_capacity(n);
                for (i, v) in data.iter().enumerate() {
                    let t = i as f32 / (n.max(2) - 1) as f32;
                    let x = inner.left() + inner.width() * t;
                    let y = inner.center().y - (v / 6.0).clamp(-1.05, 1.05) * (inner.height() * 0.5);
                    pts.push(pos2(x, y));
                }
                p.add(egui::Shape::line(
                    pts.clone(),
                    Stroke::new(3.2, theme::with_alpha(colors[ch], 0.16)),
                ));
                p.add(egui::Shape::line(pts, Stroke::new(1.6, colors[ch])));
            }

            // Legend + live values.
            p.rect_filled(
                Rect::from_min_size(
                    pos2(inner.right() - 92.0, inner.top() + 6.0),
                    vec2(84.0, 34.0),
                ),
                5,
                theme::with_alpha(Color32::BLACK, 0.35),
            );
            p.circle_filled(pos2(inner.right() - 82.0, inner.top() + 16.0), 3.0, colors[0]);
            p.text(
                pos2(inner.right() - 74.0, inner.top() + 16.0),
                Align2::LEFT_CENTER,
                format!("{:.2}V", app.result.volt(ch1)),
                FontId::new(8.0, egui::FontFamily::Monospace),
                TEXT_DIM,
            );
            p.circle_filled(pos2(inner.right() - 82.0, inner.top() + 30.0), 3.0, colors[1]);
            p.text(
                pos2(inner.right() - 74.0, inner.top() + 30.0),
                Align2::LEFT_CENTER,
                format!("{:.2}V", app.result.volt(ch2)),
                FontId::new(8.0, egui::FontFamily::Monospace),
                TEXT_DIM,
            );

            if data_empty(&app.instruments.traces) {
                p.text(
                    inner.center(),
                    Align2::CENTER_CENTER,
                    "press play to capture",
                    FontId::new(10.0, egui::FontFamily::Proportional),
                    TEXT_FAINT,
                );
            }

            ui.add_space(6.0);
            ui.horizontal(|ui| {
                let (r, _) = ui.allocate_exact_size(vec2(8.0, 8.0), Sense::hover());
                ui.painter().circle_filled(r.center(), 3.0, colors[0]);
                net_combo(ui, "scope_ch1", &app.netlist, &mut app.instruments.scope_ch[0], 118.0);
                ui.add_space(4.0);
                let (r, _) = ui.allocate_exact_size(vec2(8.0, 8.0), Sense::hover());
                ui.painter().circle_filled(r.center(), 3.0, colors[1]);
                net_combo(ui, "scope_ch2", &app.netlist, &mut app.instruments.scope_ch[1], 118.0);
            });
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(if app.running { "RUN" } else { "HOLD" })
                        .font(mono(9.5))
                        .color(if app.running { OK } else { MID }),
                );
                ui.label(
                    egui::RichText::new(format!("{:.3} s/div", 1.0))
                        .font(mono(9.5))
                        .color(TEXT_FAINT),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(format!("t = {:.2}s", app.time))
                            .font(mono(9.5))
                            .color(TEXT_FAINT),
                    );
                });
            });
        },
    );
    app.instruments.scope_pos = pos;
    app.instruments.scope_open = open;
}

fn data_empty(traces: &[Vec<f32>; 2]) -> bool {
    traces.iter().all(|t| t.len() < 2)
}
