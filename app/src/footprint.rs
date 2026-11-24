//! PCB footprint data.
//!
//! Every component in the catalog carries one of these so that anything placed
//! on the schematic can be sent to the PCB without asking the person for a
//! second file. Units are millimetres, matching KiCad's footprint convention:
//! `at` is the pad centre relative to the footprint origin, `size` is the copper
//! pad size and `drill` is 0 for an SMD pad or the hole diameter for a
//! through-hole pad.

use crate::model::Kind;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PadShape {
    Rect,
    Circle,
    Oval,
}

#[derive(Clone, Debug)]
pub struct Pad {
    pub number: String,
    pub at: (f32, f32),
    pub size: (f32, f32),
    pub drill: f32,
    pub shape: PadShape,
}

impl Pad {
    pub fn through_hole(&self) -> bool {
        self.drill > 0.0
    }
}

fn smd(number: &str, x: f32, y: f32, w: f32, h: f32) -> Pad {
    Pad {
        number: number.to_string(),
        at: (x, y),
        size: (w, h),
        drill: 0.0,
        shape: PadShape::Rect,
    }
}

fn th(number: &str, x: f32, y: f32, d: f32, drill: f32) -> Pad {
    Pad {
        number: number.to_string(),
        at: (x, y),
        size: (d, d),
        drill,
        shape: PadShape::Circle,
    }
}

#[derive(Clone, Debug)]
pub struct Footprint {
    pub name: String,
    pub body: (f32, f32),
    pub pads: Vec<Pad>,
}

impl Footprint {
    pub fn new(name: &str, body: (f32, f32), pads: Vec<Pad>) -> Self {
        Self {
            name: name.to_string(),
            body,
            pads,
        }
    }

    /// Two-pad chip resistor/capacitor/LED style.
    fn chip(name: &str, w: f32, h: f32, gap: f32) -> Self {
        Self::new(
            name,
            (w, h),
            vec![
                smd("1", -gap, 0.0, w * 0.5, h + 0.3),
                smd("2", gap, 0.0, w * 0.5, h + 0.3),
            ],
        )
    }

    fn dip(name: &str, pins: usize, pitch: f32, span: f32) -> Self {
        let rows = pins / 2;
        let mut pads = Vec::with_capacity(pins);
        for i in 0..rows {
            let y = (i as f32 - (rows as f32 - 1.0) / 2.0) * pitch;
            pads.push(th(&(i + 1).to_string(), -span / 2.0, y, 1.6, 0.8));
            pads.push(th(
                &(pins - i).to_string(),
                span / 2.0,
                y,
                1.6,
                0.8,
            ));
        }
        Self::new(name, (span + 2.4, rows as f32 * pitch + 2.4), pads)
    }

    /// Double header, like the dev boards: pads down each side.
    fn devboard(name: &str, body: (f32, f32), left: &[&str], right: &[&str]) -> Self {
        let mut pads = Vec::new();
        let pitch = 2.54;
        for (i, n) in left.iter().enumerate() {
            let y = (i as f32 - (left.len() as f32 - 1.0) / 2.0) * pitch;
            pads.push(th(n, -body.0 / 2.0 + 1.27, y, 1.7, 1.0));
        }
        for (i, n) in right.iter().enumerate() {
            let y = (i as f32 - (right.len() as f32 - 1.0) / 2.0) * pitch;
            pads.push(th(n, body.0 / 2.0 - 1.27, y, 1.7, 1.0));
        }
        Self::new(name, body, pads)
    }

    fn generic(name: &str, count: usize) -> Self {
        let pins: Vec<String> = (1..=count).map(|i| i.to_string()).collect();
        let refs: Vec<&str> = pins.iter().map(String::as_str).collect();
        let half = (refs.len() as f32 - 1.0) / 2.0;
        let body = (refs.len() as f32 * 1.27 + 2.0, 3.4);
        let pads = refs
            .iter()
            .enumerate()
            .map(|(i, n)| smd(n, (i as f32 - half) * 1.27, 0.0, 0.7, 1.6))
            .collect();
        Self::new(name, body, pads)
    }
}

/// Footprint for a built-in component. Kept in one place so the schematic
/// library, the inspector and the PCB view cannot disagree.
pub fn for_kind(kind: Kind) -> Footprint {
    match kind {
        Kind::Resistor => Footprint::chip("R_0805_2012Metric", 2.0, 1.25, 0.95),
        Kind::Capacitor => Footprint::chip("C_0805_2012Metric", 2.0, 1.25, 0.95),
        Kind::Inductor => Footprint::chip("L_0805_2012Metric", 2.0, 1.25, 0.95),
        Kind::Led => Footprint::chip("LED_0805_2012Metric", 2.0, 1.4, 0.95),
        Kind::Potentiometer => Footprint::new(
            "Potentiometer_Bourns_3296W_Vertical",
            (9.5, 9.5),
            vec![
                th("A", -3.81, 2.54, 1.8, 1.0),
                th("W", 0.0, -3.81, 1.8, 1.0),
                th("B", 3.81, 2.54, 1.8, 1.0),
            ],
        ),
        Kind::DcSource | Kind::AcSource => Footprint::new(
            "PinHeader_1x02_P2.54mm_Vertical",
            (5.08, 5.08),
            vec![th("1", 0.0, -1.27, 1.7, 1.0), th("2", 0.0, 1.27, 1.7, 1.0)],
        ),
        Kind::Vcc => Footprint::new(
            "TestPoint_Pad_D1.5mm",
            (2.5, 2.5),
            vec![th("1", 0.0, 0.0, 1.5, 0.8)],
        ),
        Kind::Ground => Footprint::new(
            "TestPoint_Pad_D1.5mm",
            (2.5, 2.5),
            vec![th("1", 0.0, 0.0, 1.5, 0.8)],
        ),
        Kind::SwitchSpst => Footprint::new(
            "SW_SPST_SKQG_WithoutStem",
            (6.0, 6.0),
            vec![
                smd("1", -3.2, -2.0, 1.1, 1.4),
                smd("2", -3.2, 2.0, 1.1, 1.4),
                smd("3", 3.2, -2.0, 1.1, 1.4),
                smd("4", 3.2, 2.0, 1.1, 1.4),
            ],
        ),
        Kind::GateAnd | Kind::GateNand | Kind::GateOr | Kind::GateXor | Kind::GateNot => {
            Footprint::generic("Package_SO:SOIC-8_3.9x4.9mm_P1.27mm", 3)
        }
        Kind::UnoBoard => Footprint::devboard(
            "Module:Arduino_Uno_Rev3",
            (53.4, 68.6),
            &[
                "A5", "A4", "AREF", "GND", "D13", "D12", "D11", "D10", "D9", "D8", "D7", "D6",
                "D5", "D4", "D3", "D2", "D1", "D0",
            ],
            &[
                "IOREF", "RESET", "3V3", "5V", "GND2", "GND3", "VIN", "A0", "A1", "A2", "A3",
                "A4", "A5",
            ],
        ),
        Kind::Atmega => Footprint::dip("Package_DIP:DIP-28_W7.62mm", 28, 2.54, 7.62),
        Kind::Esp32 => Footprint::devboard(
            "Module:ESP32-DevKitC",
            (25.5, 51.0),
            &[
                "VIN", "GND", "D13", "D12", "D14", "D27", "D26", "D25", "D33", "D32", "D35",
                "D34", "VN", "VP", "EN",
            ],
            &[
                "3V3", "GND", "D15", "D2", "D4", "RX2", "TX2", "D5", "D18", "D19", "D21", "RX0",
                "TX0", "D22", "D23",
            ],
        ),
        Kind::NanoBoard => Footprint::devboard(
            "Module:Arduino_Nano",
            (43.2, 17.8),
            &[
                "D12", "D11", "D10", "D9", "D8", "D7", "D6", "D5", "D4", "D3", "D2", "GND",
                "RESET", "D0", "D1",
            ],
            &[
                "D13", "3V3", "AREF", "A0", "A1", "A2", "A3", "A4", "A5", "A6", "A7", "5V",
                "RESET", "GND", "VIN",
            ],
        ),
        Kind::Mpu6050 => Footprint::generic("Sensor:MPU-6050_QFN-24", 8),
        Kind::Ina219 => Footprint::generic("Package_TO_SOT_SMD:SOT-23-6", 6),
        Kind::Dht22 => Footprint::new(
            "Sensor:DHT22",
            (14.0, 25.0),
            vec![
                th("VCC", -4.0, -8.0, 1.8, 1.0),
                th("SDA", -4.0, -2.7, 1.8, 1.0),
                th("NC", -4.0, 2.7, 1.8, 1.0),
                th("GND", -4.0, 8.0, 1.8, 1.0),
            ],
        ),
        Kind::Lcd1602 => Footprint::new(
            "Display:LCD-1602_Module",
            (80.0, 36.0),
            (1..=16)
                .map(|i| th(&i.to_string(), -37.0 + (i as f32 - 1.0) * 4.9, 14.0, 1.8, 1.0))
                .collect(),
        ),
        Kind::PushButton => Footprint::new(
            "Button_Switch_THT:SW_PUSH_6mm",
            (6.0, 6.0),
            vec![
                th("1", -3.25, -2.75, 1.8, 0.9),
                th("2", 3.25, -2.75, 1.8, 0.9),
                th("3", -3.25, 2.75, 1.8, 0.9),
                th("4", 3.25, 2.75, 1.8, 0.9),
            ],
        ),
        Kind::Custom => Footprint::generic("Imported:generic", 2),
    }
}
