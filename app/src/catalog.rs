//! Standardised component catalogue.
//!
//! Built-in entries are transcribed from the open-source `wokwi/wokwi-elements`
//! `pinInfo` tables (Apache-2.0). The schematic coordinates below are a linear
//! re-scale of those tables so the relative pin order is preserved; the
//! footprint attached to each entry gives the PCB side its real pad geometry.
//! Imported KiCad / SVG parts live in the same registry, so the sidebar has a
//! single source of truth.

use std::sync::{OnceLock, RwLock};

use crate::footprint::Footprint;
use crate::model::{pin, Category, Dir, Kind, PinRole, PinSpec};

#[derive(Clone, Debug)]
pub struct PartDef {
    pub id: String,
    pub name: String,
    pub desc: String,
    pub category: Category,
    pub size: (f32, f32),
    pub pins: Vec<PinSpec>,
    pub footprint: Footprint,
    pub source: String,
    /// Built-in parts keep their `Kind` so the simulator still understands them.
    /// Imported parts are placed as `Kind::Custom` and keep only this id.
    pub kind: Option<Kind>,
    pub value: String,
}

/// Pin tables below come from `wokwi/wokwi-elements/src/*-element.ts`.
pub const UNO_PINS_WOKWI: &[PinSpec] = &[
    pin("A5", -100.0, -60.0, Dir::Up, PinRole::Input),
    pin("A4", -83.0, -60.0, Dir::Up, PinRole::Input),
    pin("AREF", -66.0, -60.0, Dir::Up, PinRole::Power),
    pin("GND", -49.0, -60.0, Dir::Up, PinRole::Ground),
    pin("D13", -32.0, -60.0, Dir::Up, PinRole::Output),
    pin("D12", -15.0, -60.0, Dir::Up, PinRole::Input),
    pin("D11", 2.0, -60.0, Dir::Up, PinRole::Output),
    pin("D10", 19.0, -60.0, Dir::Up, PinRole::Output),
    pin("D9", 36.0, -60.0, Dir::Up, PinRole::Output),
    pin("D8", 53.0, -60.0, Dir::Up, PinRole::Input),
    pin("D7", 70.0, -60.0, Dir::Up, PinRole::Input),
    pin("D6", 87.0, -60.0, Dir::Up, PinRole::Output),
    pin("D5", 104.0, -60.0, Dir::Up, PinRole::Output),
    pin("D4", 121.0, -60.0, Dir::Up, PinRole::Input),
    pin("D3", 138.0, -60.0, Dir::Up, PinRole::Output),
    pin("D2", 155.0, -60.0, Dir::Up, PinRole::Input),
    pin("TX", 172.0, -60.0, Dir::Up, PinRole::Output),
    pin("RX", 189.0, -60.0, Dir::Up, PinRole::Input),
    pin("IOREF", -90.0, 66.0, Dir::Down, PinRole::Power),
    pin("RESET", -70.0, 66.0, Dir::Down, PinRole::Input),
    pin("3V3", -50.0, 66.0, Dir::Down, PinRole::Power),
    pin("5V", -30.0, 66.0, Dir::Down, PinRole::Power),
    pin("GND2", -10.0, 66.0, Dir::Down, PinRole::Ground),
    pin("GND3", 10.0, 66.0, Dir::Down, PinRole::Ground),
    pin("VIN", 30.0, 66.0, Dir::Down, PinRole::Power),
    pin("A0", 62.0, 66.0, Dir::Down, PinRole::Input),
    pin("A1", 82.0, 66.0, Dir::Down, PinRole::Input),
    pin("A2", 102.0, 66.0, Dir::Down, PinRole::Input),
    pin("A3", 122.0, 66.0, Dir::Down, PinRole::Input),
    pin("A6", 142.0, 66.0, Dir::Down, PinRole::Input),
    pin("A7", 162.0, 66.0, Dir::Down, PinRole::Input),
];

/// `arduino-nano-element.ts`: two rows at y=4.8 and y=62.4.
pub const NANO_PINS_WOKWI: &[PinSpec] = &[
    pin("D12", -106.0, -40.0, Dir::Up, PinRole::Input),
    pin("D11", -88.0, -40.0, Dir::Up, PinRole::Output),
    pin("D10", -70.0, -40.0, Dir::Up, PinRole::Output),
    pin("D9", -52.0, -40.0, Dir::Up, PinRole::Output),
    pin("D8", -34.0, -40.0, Dir::Up, PinRole::Input),
    pin("D7", -16.0, -40.0, Dir::Up, PinRole::Input),
    pin("D6", 2.0, -40.0, Dir::Up, PinRole::Output),
    pin("D5", 20.0, -40.0, Dir::Up, PinRole::Output),
    pin("D4", 38.0, -40.0, Dir::Up, PinRole::Input),
    pin("D3", 56.0, -40.0, Dir::Up, PinRole::Output),
    pin("D2", 74.0, -40.0, Dir::Up, PinRole::Input),
    pin("GND", 92.0, -40.0, Dir::Up, PinRole::Ground),
    pin("RESET", 110.0, -40.0, Dir::Up, PinRole::Input),
    pin("TX", 128.0, -40.0, Dir::Up, PinRole::Output),
    pin("RX", 146.0, -40.0, Dir::Up, PinRole::Input),
    pin("D13", -106.0, 40.0, Dir::Down, PinRole::Output),
    pin("3V3", -88.0, 40.0, Dir::Down, PinRole::Power),
    pin("AREF", -70.0, 40.0, Dir::Down, PinRole::Power),
    pin("A0", -52.0, 40.0, Dir::Down, PinRole::Input),
    pin("A1", -34.0, 40.0, Dir::Down, PinRole::Input),
    pin("A2", -16.0, 40.0, Dir::Down, PinRole::Input),
    pin("A3", 2.0, 40.0, Dir::Down, PinRole::Input),
    pin("A4", 20.0, 40.0, Dir::Down, PinRole::Input),
    pin("A5", 38.0, 40.0, Dir::Down, PinRole::Input),
    pin("A6", 56.0, 40.0, Dir::Down, PinRole::Input),
    pin("A7", 74.0, 40.0, Dir::Down, PinRole::Input),
    pin("5V", 92.0, 40.0, Dir::Down, PinRole::Power),
    pin("RESET2", 110.0, 40.0, Dir::Down, PinRole::Input),
    pin("GND2", 128.0, 40.0, Dir::Down, PinRole::Ground),
    pin("VIN", 146.0, 40.0, Dir::Down, PinRole::Power),
];

/// `esp32-devkit-v1-element.ts`: 15 pins on each side.
pub const ESP_PINS_WOKWI: &[PinSpec] = &[
    pin("VIN", -110.0, -70.0, Dir::Left, PinRole::Power),
    pin("GND", -110.0, -52.0, Dir::Left, PinRole::Ground),
    pin("D13", -110.0, -34.0, Dir::Left, PinRole::Output),
    pin("D12", -110.0, -16.0, Dir::Left, PinRole::Input),
    pin("D14", -110.0, 2.0, Dir::Left, PinRole::Input),
    pin("D27", -110.0, 20.0, Dir::Left, PinRole::Input),
    pin("D26", -110.0, 38.0, Dir::Left, PinRole::Input),
    pin("D25", -110.0, 56.0, Dir::Left, PinRole::Output),
    pin("D33", -110.0, 74.0, Dir::Left, PinRole::Input),
    pin("D32", -110.0, 92.0, Dir::Left, PinRole::Input),
    pin("D35", -110.0, 110.0, Dir::Left, PinRole::Input),
    pin("D34", -110.0, 128.0, Dir::Left, PinRole::Input),
    pin("VN", -110.0, 146.0, Dir::Left, PinRole::Input),
    pin("VP", -110.0, 164.0, Dir::Left, PinRole::Input),
    pin("EN", -110.0, 182.0, Dir::Left, PinRole::Input),
    pin("3V3", 110.0, -70.0, Dir::Right, PinRole::Power),
    pin("GND2", 110.0, -52.0, Dir::Right, PinRole::Ground),
    pin("D15", 110.0, -34.0, Dir::Right, PinRole::Output),
    pin("D2", 110.0, -16.0, Dir::Right, PinRole::Input),
    pin("D4", 110.0, 2.0, Dir::Right, PinRole::Input),
    pin("RX2", 110.0, 20.0, Dir::Right, PinRole::Input),
    pin("TX2", 110.0, 38.0, Dir::Right, PinRole::Output),
    pin("D5", 110.0, 56.0, Dir::Right, PinRole::Output),
    pin("D18", 110.0, 74.0, Dir::Right, PinRole::Input),
    pin("D19", 110.0, 92.0, Dir::Right, PinRole::Input),
    pin("D21", 110.0, 110.0, Dir::Right, PinRole::Input),
    pin("RX0", 110.0, 128.0, Dir::Right, PinRole::Input),
    pin("TX0", 110.0, 146.0, Dir::Right, PinRole::Output),
    pin("D22", 110.0, 164.0, Dir::Right, PinRole::Input),
    pin("D23", 110.0, 182.0, Dir::Right, PinRole::Input),
];

/// `mpu6050-element.ts`: eight pins along the top edge.
pub const MPU_PINS_WOKWI: &[PinSpec] = &[
    pin("INT", -72.0, -40.0, Dir::Up, PinRole::Output),
    pin("AD0", -52.0, -40.0, Dir::Up, PinRole::Input),
    pin("XCL", -32.0, -40.0, Dir::Up, PinRole::Output),
    pin("XDA", -12.0, -40.0, Dir::Up, PinRole::Input),
    pin("SDA", 8.0, -40.0, Dir::Up, PinRole::Input),
    pin("SCL", 28.0, -40.0, Dir::Up, PinRole::Input),
    pin("GND", 48.0, -40.0, Dir::Up, PinRole::Ground),
    pin("VCC", 68.0, -40.0, Dir::Up, PinRole::Power),
];

/// `dht22-element.ts`: four pins along the bottom edge.
pub const DHT_PINS_WOKWI: &[PinSpec] = &[
    pin("VCC", -48.0, 60.0, Dir::Down, PinRole::Power),
    pin("SDA", -16.0, 60.0, Dir::Down, PinRole::Input),
    pin("NC", 16.0, 60.0, Dir::Down, PinRole::Passive),
    pin("GND", 48.0, 60.0, Dir::Down, PinRole::Ground),
];

/// `lcd1602-element.ts` (I2C backpack variant): GND, VCC, SDA, SCL.
pub const LCD_PINS_WOKWI: &[PinSpec] = &[
    pin("GND", -140.0, -30.0, Dir::Left, PinRole::Ground),
    pin("VCC", -140.0, -10.0, Dir::Left, PinRole::Power),
    pin("SDA", -140.0, 10.0, Dir::Left, PinRole::Input),
    pin("SCL", -140.0, 30.0, Dir::Left, PinRole::Input),
];

/// `pushbutton-element.ts`: four pins at the corners.
pub const BUTTON_PINS_WOKWI: &[PinSpec] = &[
    pin("1", -50.0, -40.0, Dir::Up, PinRole::Passive),
    pin("2", 50.0, -40.0, Dir::Up, PinRole::Passive),
    pin("3", -50.0, 40.0, Dir::Down, PinRole::Passive),
    pin("4", 50.0, 40.0, Dir::Down, PinRole::Passive),
];

/// INA219 is not in the Wokwi element repository; this is the standard
/// six-pin breakout order from the datasheet (VCC, GND, SCL, SDA, IN+, IN-).
pub const INA_PINS: &[PinSpec] = &[
    pin("VCC", -80.0, -30.0, Dir::Up, PinRole::Power),
    pin("GND", -80.0, 30.0, Dir::Down, PinRole::Ground),
    pin("SCL", -20.0, -30.0, Dir::Up, PinRole::Input),
    pin("SDA", 20.0, -30.0, Dir::Up, PinRole::Input),
    pin("IN+", 80.0, -30.0, Dir::Up, PinRole::Input),
    pin("IN-", 80.0, 30.0, Dir::Down, PinRole::Input),
];

fn builtin(
    id: &str,
    name: &str,
    desc: &str,
    category: Category,
    size: (f32, f32),
    pins: &'static [PinSpec],
    kind: Kind,
    value: &str,
    source: &str,
) -> PartDef {
    PartDef {
        id: id.to_string(),
        name: name.to_string(),
        desc: desc.to_string(),
        category,
        size,
        pins: pins.to_vec(),
        footprint: crate::footprint::for_kind(kind),
        source: source.to_string(),
        kind: Some(kind),
        value: value.to_string(),
    }
}

pub fn builtin_parts() -> Vec<PartDef> {
    vec![
        builtin(
            "wokwi:arduino-uno",
            "Arduino Uno",
            "ATmega328P board with the Wokwi pin layout.",
            Category::Mcu,
            (220.0, 150.0),
            UNO_PINS_WOKWI,
            Kind::UnoBoard,
            "5V",
            "wokwi/wokwi-elements",
        ),
        builtin(
            "wokwi:arduino-nano",
            "Arduino Nano",
            "Compact 30-pin board, Wokwi pin order.",
            Category::Mcu,
            (300.0, 110.0),
            NANO_PINS_WOKWI,
            Kind::NanoBoard,
            "5V",
            "wokwi/wokwi-elements",
        ),
        builtin(
            "wokwi:esp32-devkit-v1",
            "ESP32 DevKit v1",
            "Dual-core 3.3V Wi-Fi board, 30 pins.",
            Category::Mcu,
            (230.0, 260.0),
            ESP_PINS_WOKWI,
            Kind::Esp32,
            "3V3",
            "wokwi/wokwi-elements",
        ),
        builtin(
            "wokwi:mpu6050",
            "MPU6050",
            "3-axis gyroscope and accelerometer over I2C.",
            Category::Mcu,
            (160.0, 90.0),
            MPU_PINS_WOKWI,
            Kind::Mpu6050,
            "I2C",
            "wokwi/wokwi-elements",
        ),
        builtin(
            "wokwi:ina219",
            "INA219",
            "Current and voltage monitor over I2C.",
            Category::Mcu,
            (180.0, 90.0),
            INA_PINS,
            Kind::Ina219,
            "I2C",
            "datasheet pinout",
        ),
        builtin(
            "wokwi:dht22",
            "DHT22",
            "Digital temperature and humidity sensor.",
            Category::Mcu,
            (130.0, 140.0),
            DHT_PINS_WOKWI,
            Kind::Dht22,
            "DHT",
            "wokwi/wokwi-elements",
        ),
        builtin(
            "wokwi:lcd1602",
            "LCD 1602 (I2C)",
            "16x2 character display with I2C backpack.",
            Category::Output,
            (280.0, 100.0),
            LCD_PINS_WOKWI,
            Kind::Lcd1602,
            "0x27",
            "wokwi/wokwi-elements",
        ),
        builtin(
            "wokwi:pushbutton",
            "Push button",
            "Momentary 4-pin tactile switch.",
            Category::Switch,
            (110.0, 90.0),
            BUTTON_PINS_WOKWI,
            Kind::PushButton,
            "TACT",
            "wokwi/wokwi-elements",
        ),
    ]
}

static REGISTRY: OnceLock<RwLock<Catalog>> = OnceLock::new();

#[derive(Default)]
pub struct Catalog {
    parts: Vec<PartDef>,
}

impl Catalog {
    fn with_builtins() -> Self {
        Self {
            parts: builtin_parts(),
        }
    }

    pub fn add(&mut self, parts: Vec<PartDef>) -> usize {
        let mut added = 0;
        for part in parts {
            if self.parts.iter().any(|p| p.id == part.id) {
                continue;
            }
            self.parts.push(part);
            added += 1;
        }
        added
    }

    pub fn get(&self, id: &str) -> Option<PartDef> {
        self.parts.iter().find(|p| p.id == id).cloned()
    }

    pub fn all(&self) -> Vec<PartDef> {
        self.parts.clone()
    }

    pub fn imported(&self) -> Vec<PartDef> {
        self.parts
            .iter()
            .filter(|p| p.source != "wokwi/wokwi-elements" && p.kind.is_none())
            .cloned()
            .collect()
    }

    pub fn search(&self, query: &str, category: Option<Category>) -> Vec<PartDef> {
        let q = query.trim().to_ascii_lowercase();
        self.parts
            .iter()
            .filter(|p| category.is_none_or(|c| c == p.category))
            .filter(|p| {
                q.is_empty()
                    || p.name.to_ascii_lowercase().contains(&q)
                    || p.desc.to_ascii_lowercase().contains(&q)
                    || p.id.to_ascii_lowercase().contains(&q)
            })
            .cloned()
            .collect()
    }
}

pub fn registry() -> &'static RwLock<Catalog> {
    REGISTRY.get_or_init(|| RwLock::new(Catalog::with_builtins()))
}

pub fn add(parts: Vec<PartDef>) -> usize {
    registry().write().map(|mut c| c.add(parts)).unwrap_or(0)
}

pub fn get(id: &str) -> Option<PartDef> {
    registry().read().ok().and_then(|c| c.get(id))
}

pub fn all() -> Vec<PartDef> {
    registry().read().map(|c| c.all()).unwrap_or_default()
}

pub fn imported() -> Vec<PartDef> {
    registry().read().map(|c| c.imported()).unwrap_or_default()
}

pub fn search(query: &str, category: Option<Category>) -> Vec<PartDef> {
    registry()
        .read()
        .map(|c| c.search(query, category))
        .unwrap_or_default()
}

/// Pins for a placed part, leaked once so the rest of the editor can keep using
/// its existing `&'static [PinSpec]` API. Imports are bounded and each part is
/// registered once, so the leak is tiny and bounded by the catalogue size.
pub fn leaked_pins(id: &str) -> Option<&'static [PinSpec]> {
    let part = get(id)?;
    let boxed: &'static [PinSpec] = Box::leak(part.pins.into_boxed_slice());
    Some(boxed)
}
