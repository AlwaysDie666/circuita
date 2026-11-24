//! Application state, the demo circuit and the simulation clock.

use crate::canvas;
use crate::instruments::{self, Instruments};
use crate::model::*;
use crate::panels;
use crate::sim::{self, Netlist, SimResult, Solver};
use crate::theme;
use eframe::egui::{self, pos2, Context, Pos2, Rect, Vec2};
use std::path::PathBuf;

pub const TOP_H: f32 = 58.0;
pub const STATUS_H: f32 = 26.0;
pub const LEFT_W: f32 = 302.0;
pub const RIGHT_W: f32 = 294.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tool {
    Select,
    Wire,
    Pan,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Selection {
    Comp(usize),
    Wire(usize),
}

#[derive(Clone)]
pub struct Snapshot {
    pub comps: Vec<Component>,
    pub wires: Vec<Wire>,
    pub next_id: usize,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum DragKind {
    Comp,
    Segment { wire: usize, seg: usize },
    Pan,
    Marquee,
}

#[derive(Clone, Debug)]
pub struct DragState {
    pub kind: DragKind,
    pub start_screen: Pos2,
    pub start_world: Pos2,
    pub orig_pts: Vec<Pos2>,
    pub orig_pos: Pos2,
    pub moved: bool,
}

pub struct App {
    pub project: Project,
    pub netlist: Netlist,
    pub result: SimResult,
    pub solver: Solver,

    pub running: bool,
    pub time: f64,
    pub clock: f64,
    pub dt: f32,
    pub speed: f32,
    pub step_count: u64,

    pub tool: Tool,
    pub armed: Option<Kind>,
    pub selection: Option<Selection>,
    pub hover_comp: Option<usize>,
    pub hover_pin: Option<PinRef>,
    pub hover_seg: Option<(usize, usize)>,
    pub wire_start: Option<PinRef>,
    pub wire_pressed_at: Option<PinRef>,
    pub wire_cursor: Option<Pos2>,
    pub drag: Option<DragState>,

    pub zoom: f32,
    pub view_center: Pos2,
    pub show_grid: bool,
    pub snap: bool,
    pub show_hints: bool,
    pub canvas_rect: Rect,
    pub last_pointer: Option<Pos2>,

    pub search: String,
    pub category: Option<Category>,
    pub lib_hover: Option<Kind>,
    pub lib_preview: Option<Kind>,
    pub inspector_tab: usize,

    pub instruments: Instruments,
    pub undo: Vec<Snapshot>,
    pub redo: Vec<Snapshot>,
    pub toast: Option<(String, f64)>,

    pub projects_open: bool,
    pub projects_pos: Option<Pos2>,
    pub project_name_edit: String,
    pub saved_list: Vec<crate::project_io::SavedProject>,
    pub delete_confirm: Option<String>,
    pub wants_fit: bool,

    pub frames: u64,
    pub screenshot: Option<PathBuf>,
    pub shot_requested: bool,
    pub shot_saved: bool,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, screenshot: Option<PathBuf>) -> Self {
        theme::install_fonts(&cc.egui_ctx);
        theme::install_style(&cc.egui_ctx);
        let project = demo_project();
        let netlist = sim::build_netlist(&project);
        // Start the simulation part-way into the blink so the first frames are lit.
        let start_time = if screenshot.is_some() { 0.30 } else { 0.0 };
        let project_name = project.name.clone();
        let mut app = App {
            project,
            netlist,
            result: SimResult::default(),
            solver: Solver::default(),
            running: true,
            time: 0.0,
            clock: 0.0,
            dt: 0.016,
            speed: 1.0,
            step_count: 0,
            tool: Tool::Select,
            armed: None,
            selection: Some(Selection::Comp(4)),
            hover_comp: None,
            hover_pin: None,
            hover_seg: None,
            wire_start: None,
            wire_pressed_at: None,
            wire_cursor: None,
            drag: None,
            zoom: 0.8,
            view_center: pos2(110.0, 10.0),
            show_grid: true,
            snap: true,
            show_hints: true,
            canvas_rect: Rect::from_min_size(Pos2::ZERO, Vec2::splat(100.0)),
            last_pointer: None,
            search: String::new(),
            category: None,
            lib_hover: None,
            lib_preview: Some(Kind::Resistor),
            inspector_tab: 0,
            instruments: Instruments::default(),
            undo: Vec::new(),
            redo: Vec::new(),
            toast: None,
            projects_open: false,
            projects_pos: None,
            project_name_edit: project_name,
            saved_list: Vec::new(),
            delete_confirm: None,
            wants_fit: false,
            frames: 0,
            screenshot,
            shot_requested: false,
            shot_saved: false,
        };
        app.netlist = sim::build_netlist(&app.project);
        app.result = app
            .solver
            .solve(&app.project, &app.netlist, app.time);
        app.instruments.bind_defaults(&app.project, &app.netlist);
        app
    }

    pub fn snapshot(&mut self) {
        self.undo.push(Snapshot {
            comps: self.project.comps.clone(),
            wires: self.project.wires.clone(),
            next_id: self.project.next_id,
        });
        if self.undo.len() > 80 {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    pub fn undo(&mut self) {
        if let Some(s) = self.undo.pop() {
            self.redo.push(Snapshot {
                comps: self.project.comps.clone(),
                wires: self.project.wires.clone(),
                next_id: self.project.next_id,
            });
            self.project.comps = s.comps;
            self.project.wires = s.wires;
            self.project.next_id = s.next_id;
            self.selection = None;
            self.toast("Undo", self.clock);
        }
    }

    pub fn redo(&mut self) {
        if let Some(s) = self.redo.pop() {
            self.undo.push(Snapshot {
                comps: self.project.comps.clone(),
                wires: self.project.wires.clone(),
                next_id: self.project.next_id,
            });
            self.project.comps = s.comps;
            self.project.wires = s.wires;
            self.project.next_id = s.next_id;
            self.selection = None;
            self.toast("Redo", self.clock);
        }
    }

    pub fn toast(&mut self, text: impl Into<String>, clock: f64) {
        self.toast = Some((text.into(), clock + 2.6));
    }

    pub fn refresh_saved(&mut self) {
        self.saved_list = crate::project_io::list(&crate::project_io::projects_dir());
    }

    pub fn open_projects(&mut self) {
        if !self.projects_open {
            self.projects_open = true;
            self.project_name_edit = self.project.name.clone();
            self.delete_confirm = None;
            self.refresh_saved();
        } else {
            self.projects_open = false;
        }
    }

    pub fn save_project(&mut self) {
        self.project.name = self.project_name_edit.trim().to_string();
        let stem = crate::project_io::sanitize_name(&self.project.name);
        if stem.is_empty() || stem == "-" {
            self.toast("Type a project name first", self.clock);
            return;
        }
        let path = crate::project_io::projects_dir().join(format!("{stem}.circuita"));
        match crate::project_io::save(&self.project, &path) {
            Ok(()) => {
                self.refresh_saved();
                self.toast(format!("Saved \u{201c}{}\u{201d}", self.project.name), self.clock);
            }
            Err(e) => self.toast(format!("Save failed: {e}"), self.clock),
        }
    }

    pub fn load_project(&mut self, file: &str) {
        let path = crate::project_io::projects_dir().join(file);
        match crate::project_io::load(&path) {
            Ok(p) => {
                self.project = p;
                self.project.reroute_all();
                self.netlist = sim::build_netlist(&self.project);
                self.solver.reset();
                self.result = self.solver.solve(&self.project, &self.netlist, self.time);
                self.instruments.bind_defaults(&self.project, &self.netlist);
                self.instruments.clear_traces();
                self.selection = None;
                self.wire_start = None;
                self.drag = None;
                self.undo.clear();
                self.redo.clear();
                self.project_name_edit = self.project.name.clone();
                self.delete_confirm = None;
                self.wants_fit = true;
                self.toast(format!("Opened \u{201c}{}\u{201d}", self.project.name), self.clock);
                self.projects_open = false;
            }
            Err(e) => self.toast(format!("Open failed: {e}"), self.clock),
        }
    }

    pub fn delete_saved(&mut self, file: &str) {
        let path = crate::project_io::projects_dir().join(file);
        match std::fs::remove_file(&path) {
            Ok(()) => {
                self.delete_confirm = None;
                self.refresh_saved();
                self.toast("Project deleted", self.clock);
            }
            Err(e) => self.toast(format!("Delete failed: {e}"), self.clock),
        }
    }

    pub fn netlet_name(&self, net: usize) -> String {
        self.netlist
            .nets
            .get(net)
            .map(|n| n.name.clone())
            .unwrap_or_default()
    }

    pub fn netlet_wire_net(&self, wire_id: usize) -> usize {
        self.netlist.wire_net.get(&wire_id).copied().unwrap_or(0)
    }

    pub fn place(&mut self, kind: Kind, pos: Pos2) {
        self.snapshot();
        let label = self.project.unique_label(kind);
        let comp = Component::new(0, kind, pos, label);
        let id = self.project.add_component(comp);
        self.selection = Some(Selection::Comp(id));
        if kind == Kind::Ground {
            self.project.reroute_all();
        }
        let name = kind.spec().name;
        self.toast(format!("Placed {name}"), self.clock);
    }

    pub fn add_wire(&mut self, a: PinRef, b: PinRef) {
        if a == b || a.comp == b.comp {
            self.toast("Pick two different pins", self.clock);
            return;
        }
        if self.project.has_wire_between(a, b) {
            self.toast("Those pins are already wired", self.clock);
            return;
        }
        let (Some(pa), Some(pb), Some(da), Some(db)) = (
            self.project.pin_pos(a),
            self.project.pin_pos(b),
            self.project.pin_dir(a),
            self.project.pin_dir(b),
        ) else {
            return;
        };
        self.snapshot();
        let obstacles = self.project.obstacles(a.comp, b.comp);
        let wires = self.project.wire_obstacles(0);
        let pts = crate::routing::route(pa, da, pb, db, &obstacles, &wires);
        let id = self.project.next_id;
        self.project.next_id += 1;
        self.project.wires.push(Wire {
            id,
            a,
            b,
            pts,
            manual: false,
        });
        self.selection = Some(Selection::Wire(id));
        self.toast("Wire connected", self.clock);
    }

    pub fn delete_selection(&mut self) {
        match self.selection {
            Some(Selection::Comp(id)) => {
                self.snapshot();
                self.project.delete_component(id);
                self.selection = None;
                self.toast("Component deleted", self.clock);
            }
            Some(Selection::Wire(id)) => {
                self.snapshot();
                self.project.delete_wire(id);
                self.selection = None;
                self.toast("Wire deleted", self.clock);
            }
            None => {}
        }
    }

    pub fn rotate(&mut self, dir: i32) {
        if let Some(Selection::Comp(id)) = self.selection {
            self.snapshot();
            if let Some(c) = self.project.comp_mut(id) {
                c.rot = (c.rot + dir).rem_euclid(4);
            }
            self.project.reroute_all();
            self.toast("Rotated 90°", self.clock);
        }
    }

    pub fn mirror(&mut self) {
        if let Some(Selection::Comp(id)) = self.selection {
            self.snapshot();
            if let Some(c) = self.project.comp_mut(id) {
                c.mirror = !c.mirror;
            }
            self.project.reroute_all();
            self.toast("Mirrored", self.clock);
        }
    }

    pub fn toggle_selected_switch(&mut self) {
        if let Some(Selection::Comp(id)) = self.selection {
            self.snapshot();
            let mut closed = None;
            if let Some(c) = self.project.comp_mut(id) {
                if c.kind == Kind::SwitchSpst {
                    c.state = !c.state;
                    closed = Some(c.state);
                }
            }
            if let Some(closed) = closed {
                let clock = self.clock;
                self.toast(
                    if closed { "Switch closed" } else { "Switch opened" },
                    clock,
                );
            }
        }
    }

    pub fn play_pause(&mut self) {
        self.running = !self.running;
        if self.running {
            self.toast("Simulation running", self.clock);
        } else {
            self.toast("Simulation paused", self.clock);
        }
    }

    pub fn step(&mut self) {
        self.running = false;
        self.time += 1.0 / 30.0;
        self.step_count += 1;
    }

    pub fn stop(&mut self) {
        self.running = false;
        self.time = 0.0;
        self.solver.reset();
        self.instruments.clear_traces();
        self.toast("Simulation reset", self.clock);
    }

    pub fn fit_view(&mut self, rect: Rect) {
        let mut min = pos2(f32::INFINITY, f32::INFINITY);
        let mut max = pos2(f32::NEG_INFINITY, f32::NEG_INFINITY);
        for c in &self.project.comps {
            let b = c.bounds().expand(24.0);
            min.x = min.x.min(b.left());
            min.y = min.y.min(b.top());
            max.x = max.x.max(b.right());
            max.y = max.y.max(b.bottom());
        }
        if !min.x.is_finite() || !max.x.is_finite() {
            return;
        }
        let size = max - min;
        let usable = Vec2::new(rect.width() - 120.0, rect.height() - 100.0);
        if size.x <= 0.0 || size.y <= 0.0 || usable.x <= 40.0 || usable.y <= 40.0 {
            return;
        }
        self.zoom = (usable.x / size.x)
            .min(usable.y / size.y)
            .clamp(0.25, 2.2);
        self.view_center = min + size * 0.5;
    }

    pub fn tick(&mut self, ctx: &Context) {
        // Screenshot runs use a fixed time step so the captured frame is the same
        // on every machine (the blink and the sine land in a known phase).
        self.dt = if self.screenshot.is_some() {
            1.0 / 60.0
        } else {
            ctx.input(|i| i.stable_dt).clamp(0.0, 0.05)
        };
        self.clock += self.dt as f64;
        if self.running {
            self.time += self.dt as f64 * self.speed as f64;
        }
        if let Some((_, expiry)) = &self.toast {
            if self.clock > *expiry {
                self.toast = None;
            }
        }
        self.netlist = sim::build_netlist(&self.project);
        self.result = self.solver.solve(&self.project, &self.netlist, self.time);
        self.instruments.clamp_nets(&self.netlist);
        if self.running {
            self.instruments.sample(&self.netlist, &self.result);
        }
    }

    pub fn handle_keys(&mut self, ctx: &Context) {
        // Save and open work even while a text field has focus.
        let (mods, s_pressed, o_pressed) = ctx.input(|i| {
            (
                i.modifiers,
                i.key_pressed(egui::Key::S),
                i.key_pressed(egui::Key::O),
            )
        });
        if mods.ctrl && s_pressed {
            self.save_project();
            return;
        }
        if mods.ctrl && o_pressed {
            self.open_projects();
            return;
        }
        if ctx.wants_keyboard_input() {
            if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                ctx.memory_mut(|m| m.surrender_focus(egui::Id::NULL));
            }
            return;
        }
        let (pressed, mods) = ctx.input(|i| (i.keys_down.clone(), i.modifiers));
        let _ = pressed;
        let keys: Vec<egui::Key> = ctx.input(|i| {
            i.events
                .iter()
                .filter_map(|e| match e {
                    egui::Event::Key {
                        key,
                        pressed: true,
                        repeat: false,
                        ..
                    } => Some(*key),
                    _ => None,
                })
                .collect()
        });
        for k in keys {
            match k {
                egui::Key::Space => {
                    if !mods.ctrl {
                        self.play_pause();
                    }
                }
                egui::Key::R => {
                    if mods.shift {
                        self.rotate(-1);
                    } else {
                        self.rotate(1);
                    }
                }
                egui::Key::M => self.mirror(),
                egui::Key::V => {
                    self.tool = Tool::Select;
                    self.armed = None;
                }
                egui::Key::W => {
                    self.tool = Tool::Wire;
                    self.armed = None;
                }
                egui::Key::H => {
                    self.tool = Tool::Pan;
                    self.armed = None;
                }
                egui::Key::Delete | egui::Key::Backspace => self.delete_selection(),
                egui::Key::G => self.snap = !self.snap,
                egui::Key::Escape => {
                    self.armed = None;
                    self.wire_start = None;
                    self.drag = None;
                    self.selection = None;
                }
                egui::Key::F => self.fit_view(self.canvas_rect),
                egui::Key::Plus | egui::Key::Equals => self.zoom = (self.zoom * 1.15).min(3.0),
                egui::Key::Minus => self.zoom = (self.zoom / 1.15).max(0.25),
                egui::Key::Z if mods.ctrl => {
                    if mods.shift {
                        self.redo();
                    } else {
                        self.undo();
                    }
                }
                egui::Key::Y if mods.ctrl => self.redo(),
                egui::Key::ArrowLeft => self.nudge(-GRID, 0.0),
                egui::Key::ArrowRight => self.nudge(GRID, 0.0),
                egui::Key::ArrowUp => self.nudge(0.0, -GRID),
                egui::Key::ArrowDown => self.nudge(0.0, GRID),
                _ => {}
            }
        }
    }

    fn nudge(&mut self, dx: f32, dy: f32) {
        if let Some(Selection::Comp(id)) = self.selection {
            if let Some(c) = self.project.comp_mut(id) {
                c.pos += Vec2::new(dx, dy);
            }
            self.project.reroute_all();
        }
    }

    pub fn handle_screenshot(&mut self, ctx: &Context) {
        if self.screenshot.is_none() {
            return;
        }
        // Warm-up: let layout, fonts and the simulation settle before grabbing.
        if self.frames < 120 {
            return;
        }
        if !self.shot_requested {
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
            self.shot_requested = true;
        }
        let shot = ctx.input(|i| {
            i.events.iter().find_map(|e| match e {
                egui::Event::Screenshot { image, .. } => Some(image.clone()),
                _ => None,
            })
        });
        if let Some(image) = shot {
            if let Some(path) = &self.screenshot {
                if let Some(dir) = path.parent() {
                    let _ = std::fs::create_dir_all(dir);
                }
                let [w, h] = image.size;
                let bytes: Vec<u8> = image
                    .pixels
                    .iter()
                    .flat_map(|p| p.to_array())
                    .collect();
                match image::RgbaImage::from_raw(w as u32, h as u32, bytes) {
                    Some(buf) => match buf.save(path) {
                        Ok(_) => {
                            println!("screenshot saved: {}", path.display());
                            self.shot_saved = true;
                        }
                        Err(e) => eprintln!("screenshot save failed: {e}"),
                    },
                    None => eprintln!("screenshot buffer size mismatch"),
                }
            }
            self.screenshot = None;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        let _ = frame;
        self.frames += 1;
        ctx.request_repaint();
        self.handle_screenshot(ctx);
        self.handle_keys(ctx);
        self.tick(ctx);

        panels::top_bar(self, ctx);
        panels::status_bar(self, ctx);
        panels::library(self, ctx);
        panels::inspector(self, ctx);
        canvas::show(self, ctx);
        instruments::show(self, ctx);
        panels::projects_window(self, ctx);
        if self.wants_fit {
            self.wants_fit = false;
            self.fit_view(self.canvas_rect);
        }
    }
}

fn comp(kind: Kind, x: f32, y: f32) -> Component {
    Component::new(0, kind, pos2(x, y), String::new())
}

fn with(kind: Kind, x: f32, y: f32, value: &str, rot: i32) -> Component {
    let mut c = comp(kind, x, y);
    c.value = value.to_string();
    c.rot = rot;
    c
}

pub fn demo_project() -> Project {
    let mut p = Project {
        name: "Blink and analog demo".into(),
        ..Default::default()
    };
    let uno = {
        let c = comp(Kind::UnoBoard, -250.0, -110.0);
        p.add_component(c)
    };
    let vcc1 = {
        let c = with(Kind::Vcc, -394.0, -300.0, "5V", 0);
        p.add_component(c)
    };
    let gnd1 = {
        let c = comp(Kind::Ground, -394.0, 40.0);
        p.add_component(c)
    };
    let r1 = {
        let c = with(Kind::Resistor, 10.0, -148.0, "330", 0);
        p.add_component(c)
    };
    let led1 = {
        let c = with(Kind::Led, 170.0, -148.0, "RED", 0);
        p.add_component(c)
    };
    let gnd2 = {
        let c = comp(Kind::Ground, 224.0, -122.0);
        p.add_component(c)
    };
    let rv1 = {
        let c = with(Kind::Potentiometer, 34.0, 120.0, "10k", 0);
        p.add_component(c)
    };
    let vcc2 = {
        let c = with(Kind::Vcc, -24.0, 20.0, "5V", 0);
        p.add_component(c)
    };
    let gnd3 = {
        let c = comp(Kind::Ground, 92.0, 240.0);
        p.add_component(c)
    };
    let ac1 = {
        let mut c = with(Kind::AcSource, -100.0, 280.0, "5V", 0);
        c.freq = 1.2;
        p.add_component(c)
    };
    let r2 = {
        let c = with(Kind::Resistor, -240.0, 280.0, "1k", 0);
        p.add_component(c)
    };
    let gnd4 = {
        let c = comp(Kind::Ground, -294.0, 380.0);
        p.add_component(c)
    };
    let gnd5 = {
        let c = comp(Kind::Ground, -46.0, 380.0);
        p.add_component(c)
    };
    let vcc3 = {
        let c = with(Kind::Vcc, -394.0, 401.0, "5V", 0);
        p.add_component(c)
    };
    let sw2 = {
        let c = with(Kind::SwitchSpst, -250.0, 425.0, "SPST", 0);
        p.add_component(c)
    };
    let r3 = {
        let c = with(Kind::Resistor, -100.0, 425.0, "220", 0);
        p.add_component(c)
    };
    let led2 = {
        let c = with(Kind::Led, 60.0, 425.0, "AMBER", 0);
        p.add_component(c)
    };
    let gnd6 = {
        let c = comp(Kind::Ground, 200.0, 451.0);
        p.add_component(c)
    };

    let mut labels = [
        (uno, "MCU1"),
        (vcc1, "PWR1"),
        (gnd1, "GND1"),
        (r1, "R1"),
        (led1, "D1"),
        (gnd2, "GND2"),
        (rv1, "RV1"),
        (vcc2, "PWR2"),
        (gnd3, "GND3"),
        (ac1, "V1"),
        (r2, "R2"),
        (gnd4, "GND4"),
        (gnd5, "GND5"),
        (vcc3, "PWR3"),
        (sw2, "SW1"),
        (r3, "R3"),
        (led2, "D2"),
        (gnd6, "GND6"),
    ];
    for (id, name) in labels.iter_mut() {
        if let Some(c) = p.comp_mut(*id) {
            c.label = name.to_string();
        }
    }

    let wires: [(usize, usize, usize, usize); 15] = [
        (vcc1, 0, uno, 0),
        (uno, 1, gnd1, 0),
        (uno, 2, r1, 0),
        (r1, 1, led1, 0),
        (led1, 1, gnd2, 0),
        (vcc2, 0, rv1, 0),
        (rv1, 1, gnd3, 0),
        (rv1, 2, uno, 3),
        (r2, 1, ac1, 0),
        (r2, 0, gnd4, 0),
        (ac1, 1, gnd5, 0),
        (vcc3, 0, sw2, 0),
        (sw2, 1, r3, 0),
        (r3, 1, led2, 0),
        (led2, 1, gnd6, 0),
    ];
    for (ac, ap, bc, bp) in wires {
        p.wires.push(Wire {
            id: p.next_id,
            a: PinRef {
                comp: ac,
                pin: ap,
            },
            b: PinRef {
                comp: bc,
                pin: bp,
            },
            pts: Vec::new(),
            manual: false,
        });
        p.next_id += 1;
    }
    p.reroute_all();
    p
}
