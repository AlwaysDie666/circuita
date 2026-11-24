//! Schematic document model: components, pins, wires and the small amount of
//! bookkeeping the editor needs to keep a circuit tidy.

use eframe::egui::{pos2, vec2, Pos2, Rect, Vec2};

pub const GRID: f32 = 10.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

impl Dir {
    pub fn vec(self) -> Vec2 {
        match self {
            Dir::Up => vec2(0.0, -1.0),
            Dir::Down => vec2(0.0, 1.0),
            Dir::Left => vec2(-1.0, 0.0),
            Dir::Right => vec2(1.0, 0.0),
        }
    }

    pub fn rotate_cw(self) -> Dir {
        match self {
            Dir::Up => Dir::Right,
            Dir::Right => Dir::Down,
            Dir::Down => Dir::Left,
            Dir::Left => Dir::Up,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PinRole {
    Passive,
    Power,
    Ground,
    Input,
    Output,
}

#[derive(Clone, Copy, Debug)]
pub struct PinSpec {
    pub name: &'static str,
    pub at: (f32, f32),
    pub dir: Dir,
    pub role: PinRole,
}

pub const fn pin(name: &'static str, x: f32, y: f32, dir: Dir, role: PinRole) -> PinSpec {
    PinSpec {
        name,
        at: (x, y),
        dir,
        role,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Category {
    Passive,
    Source,
    Output,
    Switch,
    Logic,
    Mcu,
}

impl Category {
    pub fn label(self) -> &'static str {
        match self {
            Category::Passive => "Passive",
            Category::Source => "Sources",
            Category::Output => "Output",
            Category::Switch => "Switching",
            Category::Logic => "Logic gates",
            Category::Mcu => "Microcontrollers",
        }
    }

    pub const ALL: [Category; 6] = [
        Category::Passive,
        Category::Source,
        Category::Output,
        Category::Switch,
        Category::Logic,
        Category::Mcu,
    ];
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Kind {
    Resistor,
    Capacitor,
    Inductor,
    Potentiometer,
    DcSource,
    AcSource,
    Vcc,
    Ground,
    Led,
    SwitchSpst,
    GateAnd,
    GateOr,
    GateNot,
    GateNand,
    GateXor,
    UnoBoard,
    Atmega,
    Esp32,
    NanoBoard,
    Mpu6050,
    Ina219,
    Dht22,
    Lcd1602,
    PushButton,
    /// A part imported from KiCad or an SVG library. Its pins and footprint
    /// come from the catalog registry via `Component::part`.
    Custom,
}

pub struct Spec {
    pub name: &'static str,
    pub short: &'static str,
    pub desc: &'static str,
    pub category: Category,
    pub pins: &'static [PinSpec],
    pub size: (f32, f32),
    pub prefix: &'static str,
    pub value: &'static str,
    pub value_label: &'static str,
}

const RES_PINS: [PinSpec; 2] = [
    pin("1", -40.0, 0.0, Dir::Left, PinRole::Passive),
    pin("2", 40.0, 0.0, Dir::Right, PinRole::Passive),
];
const CAP_PINS: [PinSpec; 2] = [
    pin("1", -40.0, 0.0, Dir::Left, PinRole::Passive),
    pin("2", 40.0, 0.0, Dir::Right, PinRole::Passive),
];
const POT_PINS: [PinSpec; 3] = [
    pin("A", -44.0, 0.0, Dir::Left, PinRole::Passive),
    pin("B", 44.0, 0.0, Dir::Right, PinRole::Passive),
    pin("W", 0.0, -34.0, Dir::Up, PinRole::Passive),
];
const SRC_PINS: [PinSpec; 2] = [
    pin("+", -40.0, 0.0, Dir::Left, PinRole::Power),
    pin("-", 40.0, 0.0, Dir::Right, PinRole::Power),
];
const VCC_PINS: [PinSpec; 1] = [pin("+", 0.0, 24.0, Dir::Down, PinRole::Power)];
const GND_PINS: [PinSpec; 1] = [pin("GND", 0.0, -26.0, Dir::Up, PinRole::Ground)];
const LED_PINS: [PinSpec; 2] = [
    pin("A", -40.0, 0.0, Dir::Left, PinRole::Passive),
    pin("K", 40.0, 0.0, Dir::Right, PinRole::Passive),
];
const SW_PINS: [PinSpec; 2] = [
    pin("1", -40.0, 0.0, Dir::Left, PinRole::Passive),
    pin("2", 40.0, 0.0, Dir::Right, PinRole::Passive),
];
const GATE_PINS: [PinSpec; 3] = [
    pin("A", -40.0, -14.0, Dir::Left, PinRole::Input),
    pin("B", -40.0, 14.0, Dir::Left, PinRole::Input),
    pin("Y", 40.0, 0.0, Dir::Right, PinRole::Output),
];
const NOT_PINS: [PinSpec; 2] = [
    pin("A", -40.0, 0.0, Dir::Left, PinRole::Input),
    pin("Y", 40.0, 0.0, Dir::Right, PinRole::Output),
];
const UNO_PINS: [PinSpec; 4] = [
    pin("5V", -130.0, -38.0, Dir::Left, PinRole::Power),
    pin("GND", -130.0, 38.0, Dir::Left, PinRole::Ground),
    pin("D13", 130.0, -38.0, Dir::Right, PinRole::Output),
    pin("A0", 130.0, 38.0, Dir::Right, PinRole::Input),
];
const ATMEGA_PINS: [PinSpec; 4] = [
    pin("VCC", -66.0, -38.0, Dir::Left, PinRole::Power),
    pin("GND", -66.0, 38.0, Dir::Left, PinRole::Ground),
    pin("PB5", 66.0, -38.0, Dir::Right, PinRole::Output),
    pin("PC0", 66.0, 38.0, Dir::Right, PinRole::Input),
];
const ESP_PINS: [PinSpec; 4] = [
    pin("3V3", -76.0, -34.0, Dir::Left, PinRole::Power),
    pin("GND", -76.0, 34.0, Dir::Left, PinRole::Ground),
    pin("IO2", 76.0, -34.0, Dir::Right, PinRole::Output),
    pin("IO4", 76.0, 34.0, Dir::Right, PinRole::Input),
];

impl Kind {
    pub fn spec(self) -> Spec {
        match self {
            Kind::Resistor => Spec {
                name: "Resistor",
                short: "R",
                desc: "Limits current. Value in ohms, k for kilo.",
                category: Category::Passive,
                pins: &RES_PINS,
                size: (46.0, 20.0),
                prefix: "R",
                value: "10k",
                value_label: "Resistance",
            },
            Kind::Capacitor => Spec {
                name: "Capacitor",
                short: "C",
                desc: "Stores charge. Open circuit in DC.",
                category: Category::Passive,
                pins: &CAP_PINS,
                size: (24.0, 30.0),
                prefix: "C",
                value: "100nF",
                value_label: "Capacitance",
            },
            Kind::Inductor => Spec {
                name: "Inductor",
                short: "L",
                desc: "Coil that resists change in current.",
                category: Category::Passive,
                pins: &RES_PINS,
                size: (46.0, 20.0),
                prefix: "L",
                value: "10mH",
                value_label: "Inductance",
            },
            Kind::Potentiometer => Spec {
                name: "Potentiometer",
                short: "RV",
                desc: "Variable divider. Drag the wiper in the panel.",
                category: Category::Passive,
                pins: &POT_PINS,
                size: (52.0, 24.0),
                prefix: "RV",
                value: "10k",
                value_label: "Total resistance",
            },
            Kind::DcSource => Spec {
                name: "DC source",
                short: "V",
                desc: "Battery style supply with a steady voltage.",
                category: Category::Source,
                pins: &SRC_PINS,
                size: (38.0, 38.0),
                prefix: "V",
                value: "9V",
                value_label: "Voltage",
            },
            Kind::AcSource => Spec {
                name: "AC source",
                short: "~",
                desc: "Sine wave supply, ideal for the oscilloscope.",
                category: Category::Source,
                pins: &SRC_PINS,
                size: (38.0, 38.0),
                prefix: "V",
                value: "5V",
                value_label: "Peak voltage",
            },
            Kind::Vcc => Spec {
                name: "+5V rail",
                short: "5V",
                desc: "Logic supply rail tied to ground.",
                category: Category::Source,
                pins: &VCC_PINS,
                size: (22.0, 30.0),
                prefix: "PWR",
                value: "5V",
                value_label: "Rail voltage",
            },
            Kind::Ground => Spec {
                name: "Ground",
                short: "GND",
                desc: "Zero volt reference for the whole circuit.",
                category: Category::Source,
                pins: &GND_PINS,
                size: (34.0, 32.0),
                prefix: "GND",
                value: "0V",
                value_label: "Reference",
            },
            Kind::Led => Spec {
                name: "LED",
                short: "LED",
                desc: "Light emitting diode, glows with current.",
                category: Category::Output,
                pins: &LED_PINS,
                size: (30.0, 30.0),
                prefix: "D",
                value: "RED",
                value_label: "Colour",
            },
            Kind::SwitchSpst => Spec {
                name: "Switch",
                short: "SW",
                desc: "Single pole switch. Click it to toggle.",
                category: Category::Switch,
                pins: &SW_PINS,
                size: (44.0, 22.0),
                prefix: "SW",
                value: "SPST",
                value_label: "Type",
            },
            Kind::GateAnd => Spec {
                name: "AND gate",
                short: "AND",
                desc: "Output high only when A and B are high.",
                category: Category::Logic,
                pins: &GATE_PINS,
                size: (58.0, 52.0),
                prefix: "U",
                value: "74HC08",
                value_label: "Part",
            },
            Kind::GateOr => Spec {
                name: "OR gate",
                short: "OR",
                desc: "Output high when A or B is high.",
                category: Category::Logic,
                pins: &GATE_PINS,
                size: (58.0, 52.0),
                prefix: "U",
                value: "74HC32",
                value_label: "Part",
            },
            Kind::GateNot => Spec {
                name: "NOT gate",
                short: "NOT",
                desc: "Inverter: high in, low out.",
                category: Category::Logic,
                pins: &NOT_PINS,
                size: (46.0, 40.0),
                prefix: "U",
                value: "74HC04",
                value_label: "Part",
            },
            Kind::GateNand => Spec {
                name: "NAND gate",
                short: "NAND",
                desc: "Inverted AND, the classic logic workhorse.",
                category: Category::Logic,
                pins: &GATE_PINS,
                size: (64.0, 52.0),
                prefix: "U",
                value: "74HC00",
                value_label: "Part",
            },
            Kind::GateXor => Spec {
                name: "XOR gate",
                short: "XOR",
                desc: "Output high when inputs differ.",
                category: Category::Logic,
                pins: &GATE_PINS,
                size: (58.0, 52.0),
                prefix: "U",
                value: "74HC86",
                value_label: "Part",
            },
            Kind::UnoBoard => Spec {
                name: "Arduino Uno",
                short: "UNO",
                desc: "ATmega328 board, blinks D13 every second. Wokwi pin layout.",
                category: Category::Mcu,
                pins: &UNO_PINS,
                size: (220.0, 150.0),
                prefix: "MCU",
                value: "Blink sketch",
                value_label: "Firmware",
            },
            Kind::Atmega => Spec {
                name: "ATmega328P",
                short: "328P",
                desc: "Bare 28 pin AVR in a DIP package.",
                category: Category::Mcu,
                pins: &ATMEGA_PINS,
                size: (78.0, 120.0),
                prefix: "U",
                value: "Blink",
                value_label: "Firmware",
            },
            Kind::Esp32 => Spec {
                name: "ESP32 DevKit",
                short: "ESP",
                desc: "3.3V wireless module with a toggling IO pin. Wokwi pin layout.",
                category: Category::Mcu,
                pins: &ESP_PINS,
                size: (230.0, 260.0),
                prefix: "U",
                value: "Blink",
                value_label: "Firmware",
            },
            Kind::NanoBoard => Spec {
                name: "Arduino Nano",
                short: "NANO",
                desc: "Compact Arduino board with the Wokwi 30-pin layout.",
                category: Category::Mcu,
                pins: crate::catalog::NANO_PINS_WOKWI,
                size: (300.0, 110.0),
                prefix: "MCU",
                value: "5V",
                value_label: "Supply",
            },
            Kind::Mpu6050 => Spec {
                name: "MPU6050",
                short: "MPU",
                desc: "3-axis motion sensor, 8 pins from the Wokwi element.",
                category: Category::Mcu,
                pins: crate::catalog::MPU_PINS_WOKWI,
                size: (160.0, 90.0),
                prefix: "U",
                value: "I2C",
                value_label: "Bus",
            },
            Kind::Ina219 => Spec {
                name: "INA219",
                short: "INA",
                desc: "Current monitor with VCC, GND, SCL, SDA, IN+ and IN-.",
                category: Category::Mcu,
                pins: crate::catalog::INA_PINS,
                size: (180.0, 90.0),
                prefix: "U",
                value: "I2C",
                value_label: "Bus",
            },
            Kind::Dht22 => Spec {
                name: "DHT22",
                short: "DHT",
                desc: "Temperature and humidity sensor, Wokwi pin order.",
                category: Category::Mcu,
                pins: crate::catalog::DHT_PINS_WOKWI,
                size: (130.0, 140.0),
                prefix: "U",
                value: "DHT",
                value_label: "Protocol",
            },
            Kind::Lcd1602 => Spec {
                name: "LCD 1602 (I2C)",
                short: "LCD",
                desc: "16x2 character display with four I2C backpack pins.",
                category: Category::Output,
                pins: crate::catalog::LCD_PINS_WOKWI,
                size: (280.0, 100.0),
                prefix: "LCD",
                value: "0x27",
                value_label: "Address",
            },
            Kind::PushButton => Spec {
                name: "Push button",
                short: "SW",
                desc: "Momentary tactile button. Click it to toggle.",
                category: Category::Switch,
                pins: crate::catalog::BUTTON_PINS_WOKWI,
                size: (110.0, 90.0),
                prefix: "SW",
                value: "TACT",
                value_label: "Type",
            },
            Kind::Custom => Spec {
                name: "Imported part",
                short: "IC",
                desc: "Component imported from a KiCad or SVG library.",
                category: Category::Passive,
                pins: crate::catalog::INA_PINS,
                size: (120.0, 90.0),
                prefix: "U",
                value: "",
                value_label: "Value",
            },
        }
    }

    pub const ALL: [Kind; 18] = [
        Kind::Resistor,
        Kind::Capacitor,
        Kind::Inductor,
        Kind::Potentiometer,
        Kind::DcSource,
        Kind::AcSource,
        Kind::Vcc,
        Kind::Ground,
        Kind::Led,
        Kind::SwitchSpst,
        Kind::GateAnd,
        Kind::GateOr,
        Kind::GateNot,
        Kind::GateNand,
        Kind::GateXor,
        Kind::UnoBoard,
        Kind::Atmega,
        Kind::Esp32,
    ];

    /// Extra Wokwi-backed parts shown in the catalogue panel. They are kept
    /// separate from ALL so the original built-in library order is untouched.
    pub const EXTRA: [Kind; 6] = [
        Kind::NanoBoard,
        Kind::Mpu6050,
        Kind::Ina219,
        Kind::Dht22,
        Kind::Lcd1602,
        Kind::PushButton,
    ];

    pub fn is_mcu(self) -> bool {
        matches!(
            self,
            Kind::UnoBoard
                | Kind::Atmega
                | Kind::Esp32
                | Kind::NanoBoard
                | Kind::Mpu6050
                | Kind::Ina219
                | Kind::Dht22
        )
    }
}

#[derive(Clone, Debug)]
pub struct Component {
    pub id: usize,
    pub kind: Kind,
    pub pos: Pos2,
    pub rot: i32,
    pub mirror: bool,
    pub label: String,
    pub value: String,
    pub state: bool,
    pub pot: f32,
    pub freq: f32,
    /// Catalog id for imported parts; built-in parts leave this empty.
    pub part: Option<String>,
}

impl Component {
    pub fn new(id: usize, kind: Kind, pos: Pos2, label: String) -> Self {
        let spec = kind.spec();
        Component {
            id,
            kind,
            pos,
            rot: 0,
            mirror: false,
            label,
            value: spec.value.to_string(),
            state: kind == Kind::SwitchSpst || kind == Kind::PushButton,
            pot: 0.5,
            freq: 500.0,
            part: None,
        }
    }

    pub fn pins(&self) -> &'static [PinSpec] {
        if let Some(id) = &self.part {
            if let Some(pins) = crate::catalog::leaked_pins(id) {
                return pins;
            }
        }
        self.kind.spec().pins
    }

    /// PCB footprint attached to this component, with real pad sizes and holes.
    pub fn footprint(&self) -> crate::footprint::Footprint {
        if let Some(id) = &self.part {
            if let Some(part) = crate::catalog::get(id) {
                return part.footprint;
            }
        }
        crate::footprint::for_kind(self.kind)
    }

    /// Local offset after mirror and rotation, in world units.
    pub fn pin_offset(&self, pin: usize) -> Vec2 {
        let spec = self.pins()[pin];
        let mut v = vec2(spec.at.0, spec.at.1);
        if self.mirror {
            v.x = -v.x;
        }
        for _ in 0..(self.rot.rem_euclid(4)) {
            v = vec2(-v.y, v.x);
        }
        v
    }

    pub fn pin_pos(&self, pin: usize) -> Pos2 {
        self.pos + self.pin_offset(pin)
    }

    pub fn pin_dir(&self, pin: usize) -> Dir {
        let mut d = self.pins()[pin].dir;
        if self.mirror {
            d = match d {
                Dir::Left => Dir::Right,
                Dir::Right => Dir::Left,
                other => other,
            };
        }
        for _ in 0..(self.rot.rem_euclid(4)) {
            d = d.rotate_cw();
        }
        d
    }

    pub fn size(&self) -> Vec2 {
        let s = if let Some(id) = &self.part {
            crate::catalog::get(id)
                .map(|p| p.size)
                .unwrap_or_else(|| self.kind.spec().size)
        } else {
            self.kind.spec().size
        };
        if self.rot.rem_euclid(2) == 1 {
            vec2(s.1, s.0)
        } else {
            vec2(s.0, s.1)
        }
    }

    pub fn bounds(&self) -> Rect {
        Rect::from_center_size(self.pos, self.size())
    }

    pub fn term_count(&self) -> usize {
        self.pins().len()
    }

    pub fn value_numeric(&self) -> f64 {
        parse_engineering(&self.value).unwrap_or(0.0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PinRef {
    pub comp: usize,
    pub pin: usize,
}

#[derive(Clone, Debug)]
pub struct Wire {
    pub id: usize,
    pub a: PinRef,
    pub b: PinRef,
    pub pts: Vec<Pos2>,
    pub manual: bool,
}

impl Wire {
    pub fn length(&self) -> f32 {
        self.pts.windows(2).map(|w| w[0].distance(w[1])).sum()
    }

    pub fn bounds(&self) -> Rect {
        Rect::from_points(&self.pts).expand(4.0)
    }
}

/// Where one component sits on the PCB. Millimetres, board origin at the
/// top-left of the board outline.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placement {
    pub comp: usize,
    pub x: f32,
    pub y: f32,
    pub rot: i32,
}

#[derive(Clone, Default)]
pub struct Project {
    pub comps: Vec<Component>,
    pub wires: Vec<Wire>,
    pub next_id: usize,
    pub name: String,
    /// PCB placements, kept on the document so a saved project reopens with
    /// the same board layout.
    pub placements: Vec<Placement>,
}

impl Project {
    pub fn comp(&self, id: usize) -> Option<&Component> {
        self.comps.iter().find(|c| c.id == id)
    }

    pub fn comp_mut(&mut self, id: usize) -> Option<&mut Component> {
        self.comps.iter_mut().find(|c| c.id == id)
    }

    pub fn wire(&self, id: usize) -> Option<&Wire> {
        self.wires.iter().find(|w| w.id == id)
    }

    pub fn pin_pos(&self, p: PinRef) -> Option<Pos2> {
        self.comp(p.comp).map(|c| c.pin_pos(p.pin))
    }

    pub fn pin_dir(&self, p: PinRef) -> Option<Dir> {
        self.comp(p.comp).map(|c| c.pin_dir(p.pin))
    }

    pub fn add_component(&mut self, mut comp: Component) -> usize {
        comp.id = self.next_id;
        self.next_id += 1;
        let id = comp.id;
        self.comps.push(comp);
        id
    }

    pub fn unique_label(&self, kind: Kind) -> String {
        let prefix = kind.spec().prefix;
        let mut n = 1;
        loop {
            let candidate = format!("{prefix}{n}");
            if !self.comps.iter().any(|c| c.label == candidate) {
                return candidate;
            }
            n += 1;
        }
    }

    pub fn delete_component(&mut self, id: usize) {
        self.comps.retain(|c| c.id != id);
        self.wires.retain(|w| w.a.comp != id && w.b.comp != id);
    }

    pub fn delete_wire(&mut self, id: usize) {
        self.wires.retain(|w| w.id != id);
    }

    pub fn has_wire_between(&self, a: PinRef, b: PinRef) -> bool {
        self.wires
            .iter()
            .any(|w| (w.a == a && w.b == b) || (w.a == b && w.b == a))
    }

    pub fn pin_wires(&self, p: PinRef) -> usize {
        self.wires.iter().filter(|w| w.a == p || w.b == p).count()
    }

    /// Recompute every route that has not been reshaped by hand.
    pub fn reroute_all(&mut self) {
        let routes: Vec<(usize, Vec<Pos2>)> = self
            .wires
            .iter()
            .filter(|w| !w.manual)
            .filter_map(|w| {
                let a = self.pin_pos(w.a)?;
                let b = self.pin_pos(w.b)?;
                let da = self.pin_dir(w.a)?;
                let db = self.pin_dir(w.b)?;
                let obstacles = self.obstacles(w.a.comp, w.b.comp);
                let others = self.wire_obstacles(w.id);
                Some((w.id, crate::routing::route(a, da, b, db, &obstacles, &others)))
            })
            .collect();
        for (id, pts) in routes {
            if let Some(w) = self.wires.iter_mut().find(|w| w.id == id) {
                w.pts = pts;
            }
        }
    }

    /// Component bodies that a wire should try to avoid.
    pub fn obstacles(&self, skip_a: usize, skip_b: usize) -> Vec<Rect> {
        self.comps
            .iter()
            .filter(|c| c.id != skip_a && c.id != skip_b)
            .map(|c| c.bounds().expand(5.0))
            .collect()
    }

    pub fn wire_obstacles(&self, skip: usize) -> Vec<(Pos2, Pos2)> {
        let mut out = Vec::new();
        for w in self.wires.iter().filter(|w| w.id != skip) {
            for pair in w.pts.windows(2) {
                out.push((pair[0], pair[1]));
            }
        }
        out
    }

    pub fn stats(&self) -> (usize, usize) {
        (self.comps.len(), self.wires.len())
    }
}

/// Parse "10k", "4.7k", "2M2", "100nF", "9V", "5" into a number.
pub fn parse_engineering(s: &str) -> Option<f64> {
    let cleaned: String = s
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '.' || *c == '-')
        .collect();
    let cleaned = cleaned.trim();
    if cleaned.is_empty() {
        return None;
    }
    let mut num = String::new();
    let mut rest = String::new();
    let mut seen_sep = false;
    for ch in cleaned.chars() {
        if ch.is_ascii_digit() || (ch == '.' && !seen_sep) || ch == '-' {
            if !rest.is_empty() {
                break;
            }
            num.push(ch);
            if ch == '.' {
                seen_sep = true;
            }
        } else {
            rest.push(ch);
        }
    }
    if num.is_empty() || num == "-" {
        return None;
    }
    let base: f64 = num.parse().ok()?;
    let mult = match rest.chars().next() {
        Some('k') | Some('K') => 1e3,
        Some('M') | Some('m') if rest.len() > 1 && !rest.starts_with("mH") => 1e6,
        Some('m') => 1e-3,
        Some('u') | Some('U') => 1e-6,
        Some('n') | Some('N') => 1e-9,
        Some('p') | Some('P') => 1e-12,
        _ => 1.0,
    };
    Some(base * mult)
}

pub fn snap(p: Pos2, enabled: bool) -> Pos2 {
    if enabled {
        pos2(
            (p.x / GRID).round() * GRID,
            (p.y / GRID).round() * GRID,
        )
    } else {
        p
    }
}
