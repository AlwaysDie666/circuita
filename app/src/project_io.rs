//! Saving and opening schematic projects as small plain-text files.
//!
//! The format is a tab-separated document so it can be written and read back
//! with the standard library alone, and still be inspected in any text editor:
//!
//! ```text
//! circuita-v1
//! name\t<document name>
//! comp\t<kind>\t<x>\t<y>\t<rot>\t<mirror>\t<label>\t<value>\t<pot>\t<freq>\t<state>
//! wire\t<comp index A>\t<pin A>\t<comp index B>\t<pin B>\t<manual>\t<n points>\t<x1>\t<y1>\t...
//! ```

use crate::model::{Component, Kind, Project, Wire};
use std::path::{Path, PathBuf};

const MAGIC: &str = "circuita-v1";
pub const EXTENSION: &str = "circuita";

/// One entry in the saved-projects list.
#[derive(Clone, Debug)]
pub struct SavedProject {
    /// File name including the extension.
    pub file: String,
    /// Document name stored inside the file.
    pub name: String,
    pub comps: usize,
    pub wires: usize,
    /// Last modified time as unix seconds.
    pub modified: u64,
}

/// Where saved projects live. Prefers a friendly "Circuita Projects" folder in
/// the user's Documents directory and falls back to a folder next to the app.
pub fn projects_dir() -> PathBuf {
    if let Some(docs) = documents_dir() {
        return docs.join("Circuita Projects");
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            return dir.join("projects");
        }
    }
    PathBuf::from("projects")
}

fn documents_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("USERPROFILE").map(|home| PathBuf::from(home).join("Documents"))
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::env::var_os("HOME").map(|home| PathBuf::from(home).join("Documents"))
    }
}

/// Turn a document name into a safe file stem (no path separators or reserved
/// characters), keeping it short enough to read in a file listing.
pub fn sanitize_name(s: &str) -> String {
    let mut out: String = s
        .chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | ' ' | '-' | '_' | '.' | '(' | ')' => c,
            _ => '-',
        })
        .collect();
    out = out.trim().trim_matches('.').to_string();
    if out.chars().count() > 48 {
        out = out.chars().take(48).collect();
    }
    out.trim().to_string()
}

pub fn save(project: &Project, path: &Path) -> Result<(), String> {
    let mut out = String::new();
    out.push_str(MAGIC);
    out.push('\n');
    out.push_str(&format!("name\t{}\n", esc(&project.name)));
    for c in &project.comps {
        out.push_str(&format!(
            "comp\t{}\t{:.1}\t{:.1}\t{}\t{}\t{}\t{}\t{:.4}\t{:.2}\t{}\t{}\n",
            kind_str(c.kind),
            c.pos.x,
            c.pos.y,
            c.rot,
            c.mirror as i32,
            esc(&c.label),
            esc(&c.value),
            c.pot,
            c.freq,
            c.state as i32,
            c.part.as_deref().unwrap_or(""),
        ));
    }
    for p in &project.placements {
        out.push_str(&format!(
            "place\t{}\t{:.2}\t{:.2}\t{}\n",
            p.comp, p.x, p.y, p.rot
        ));
    }
    for w in &project.wires {
        let ai = project
            .comps
            .iter()
            .position(|c| c.id == w.a.comp)
            .ok_or_else(|| "wire points at a missing component".to_string())?;
        let bi = project
            .comps
            .iter()
            .position(|c| c.id == w.b.comp)
            .ok_or_else(|| "wire points at a missing component".to_string())?;
        out.push_str(&format!(
            "wire\t{}\t{}\t{}\t{}\t{}\t{}",
            ai, w.a.pin, bi, w.b.pin, w.manual as i32, w.pts.len()
        ));
        for pt in &w.pts {
            out.push_str(&format!("\t{:.1}\t{:.1}", pt.x, pt.y));
        }
        out.push('\n');
    }
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    std::fs::write(path, out).map_err(|e| e.to_string())
}

pub fn load(path: &Path) -> Result<Project, String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut project = Project {
        name: "Untitled".into(),
        ..Default::default()
    };
    let mut wires_raw: Vec<(usize, usize, usize, usize, bool, Vec<f32>)> = Vec::new();
    for (lineno, line) in text.lines().enumerate() {
        let lineno = lineno + 1;
        let mut fields = line.split('\t');
        let tag = fields.next().unwrap_or("");
        match tag {
            MAGIC => continue,
            "name" => {
                let v = fields.next().unwrap_or("");
                project.name = unesc(v);
            }
            "comp" => {
                let kind = fields
                    .next()
                    .and_then(kind_from)
                    .ok_or_else(|| format!("line {lineno}: unknown component type"))?;
                let x: f32 = parse(fields.next(), lineno, "x")?;
                let y: f32 = parse(fields.next(), lineno, "y")?;
                let rot: i32 = parse(fields.next(), lineno, "rotation")?;
                let mirror: u8 = parse(fields.next(), lineno, "mirror")?;
                let label = unesc(fields.next().unwrap_or(""));
                let value = unesc(fields.next().unwrap_or(""));
                let pot: f32 = parse(fields.next(), lineno, "wiper")?;
                let freq: f32 = parse(fields.next(), lineno, "frequency")?;
                let state: u8 = parse(fields.next(), lineno, "switch state")?;
                let mut comp =
                    Component::new(0, kind, eframe_pos(x, y), label);
                comp.value = value;
                comp.rot = rot.rem_euclid(4);
                comp.mirror = mirror != 0;
                comp.pot = pot.clamp(0.0, 1.0);
                comp.freq = freq.max(0.01);
                comp.state = state != 0;
                let part = unesc(fields.next().unwrap_or(""));
                comp.part = if part.is_empty() {
                    None
                } else {
                    Some(part)
                };
                comp.id = project.comps.len();
                project.comps.push(comp);
            }
            "place" => {
                let comp: usize = parse(fields.next(), lineno, "component")?;
                let x: f32 = parse(fields.next(), lineno, "board x")?;
                let y: f32 = parse(fields.next(), lineno, "board y")?;
                let rot: i32 = parse(fields.next(), lineno, "board rotation")?;
                project.placements.push(crate::model::Placement {
                    comp,
                    x,
                    y,
                    rot,
                });
            }
            "wire" => {
                let ai: usize = parse(fields.next(), lineno, "component")?;
                let ap: usize = parse(fields.next(), lineno, "pin")?;
                let bi: usize = parse(fields.next(), lineno, "component")?;
                let bp: usize = parse(fields.next(), lineno, "pin")?;
                let manual: u8 = parse(fields.next(), lineno, "manual")?;
                let npts: usize = parse(fields.next(), lineno, "point count")?;
                let mut pts: Vec<f32> = Vec::new();
                for _ in 0..(npts * 2) {
                    pts.push(parse(fields.next(), lineno, "route point")?);
                }
                wires_raw.push((ai, ap, bi, bp, manual != 0, pts));
            }
            "" => continue,
            other => {
                return Err(format!("line {lineno}: unexpected section '{other}'"));
            }
        }
    }
    project.next_id = project.comps.len();
    for (ai, ap, bi, bp, manual, coords) in wires_raw {
        if ai >= project.comps.len() || bi >= project.comps.len() {
            return Err("wire points past the end of the component list".into());
        }
        let mut pts = Vec::new();
        for chunk in coords.chunks(2) {
            pts.push(eframe_pos(chunk[0], chunk[1]));
        }
        project.wires.push(Wire {
            id: project.next_id,
            a: crate::model::PinRef {
                comp: ai,
                pin: ap,
            },
            b: crate::model::PinRef {
                comp: bi,
                pin: bp,
            },
            pts,
            manual,
        });
        project.next_id += 1;
    }
    Ok(project)
}

/// List the saved projects in a folder, newest first.
pub fn list(dir: &Path) -> Vec<SavedProject> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some(EXTENSION) {
            continue;
        }
        let Some(file) = path.file_name().and_then(|f| f.to_str()) else {
            continue;
        };
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let mut name = String::new();
        let mut comps = 0usize;
        let mut wires = 0usize;
        for line in text.lines() {
            if let Some(rest) = line.strip_prefix("name\t") {
                name = unesc(rest);
            } else if line.starts_with("comp\t") {
                comps += 1;
            } else if line.starts_with("wire\t") {
                wires += 1;
            }
        }
        let modified = entry
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        out.push(SavedProject {
            file: file.to_string(),
            name: if name.is_empty() {
                file.strip_suffix(EXTENSION)
                    .unwrap_or(file)
                    .trim_end_matches('.')
                    .to_string()
            } else {
                name
            },
            comps,
            wires,
            modified,
        });
    }
    out.sort_by(|a, b| b.modified.cmp(&a.modified));
    out
}

/// Format unix seconds as "YYYY-MM-DD HH:MM" in UTC, without pulling in a
/// date library.
pub fn fmt_time(secs: u64) -> String {
    let days = (secs / 86400) as i64;
    let rem = secs % 86400;
    let (h, m) = (rem / 3600, (rem % 3600) / 60);
    let z = days + 719_468;
    let era = z / 146_097;
    let doe = (z - era * 146_097) as i64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mth = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if mth <= 2 { y + 1 } else { y };
    format!("{y:04}-{mth:02}-{d:02} {h:02}:{m:02}")
}

fn parse<T: std::str::FromStr>(
    field: Option<&str>,
    lineno: usize,
    what: &str,
) -> Result<T, String> {
    field
        .and_then(|v| v.trim().parse().ok())
        .ok_or_else(|| format!("line {lineno}: bad {what}"))
}

fn eframe_pos(x: f32, y: f32) -> eframe::egui::Pos2 {
    eframe::egui::pos2(x, y)
}

fn kind_str(k: Kind) -> &'static str {
    match k {
        Kind::Resistor => "Resistor",
        Kind::Capacitor => "Capacitor",
        Kind::Inductor => "Inductor",
        Kind::Potentiometer => "Potentiometer",
        Kind::DcSource => "DcSource",
        Kind::AcSource => "AcSource",
        Kind::Vcc => "Vcc",
        Kind::Ground => "Ground",
        Kind::Led => "Led",
        Kind::SwitchSpst => "Switch",
        Kind::GateAnd => "GateAnd",
        Kind::GateOr => "GateOr",
        Kind::GateNot => "GateNot",
        Kind::GateNand => "GateNand",
        Kind::GateXor => "GateXor",
        Kind::UnoBoard => "Uno",
        Kind::Atmega => "Atmega328P",
        Kind::Esp32 => "Esp32",
        Kind::NanoBoard => "Nano",
        Kind::Mpu6050 => "Mpu6050",
        Kind::Ina219 => "Ina219",
        Kind::Dht22 => "Dht22",
        Kind::Lcd1602 => "Lcd1602",
        Kind::PushButton => "PushButton",
        Kind::Custom => "Custom",
    }
}

fn kind_from(s: &str) -> Option<Kind> {
    Some(match s {
        "Resistor" => Kind::Resistor,
        "Capacitor" => Kind::Capacitor,
        "Inductor" => Kind::Inductor,
        "Potentiometer" => Kind::Potentiometer,
        "DcSource" => Kind::DcSource,
        "AcSource" => Kind::AcSource,
        "Vcc" => Kind::Vcc,
        "Ground" => Kind::Ground,
        "Led" => Kind::Led,
        "Switch" => Kind::SwitchSpst,
        "GateAnd" => Kind::GateAnd,
        "GateOr" => Kind::GateOr,
        "GateNot" => Kind::GateNot,
        "GateNand" => Kind::GateNand,
        "GateXor" => Kind::GateXor,
        "Uno" => Kind::UnoBoard,
        "Atmega328P" => Kind::Atmega,
        "Esp32" => Kind::Esp32,
        "Nano" => Kind::NanoBoard,
        "Mpu6050" => Kind::Mpu6050,
        "Ina219" => Kind::Ina219,
        "Dht22" => Kind::Dht22,
        "Lcd1602" => Kind::Lcd1602,
        "PushButton" => Kind::PushButton,
        "Custom" => Kind::Custom,
        _ => return None,
    })
}

fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\r' => {}
            other => out.push(other),
        }
    }
    out
}

fn unesc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        match chars.next() {
            Some('t') => out.push('\t'),
            Some('n') => out.push('\n'),
            Some('\\') => out.push('\\'),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Kind;

    #[test]
    fn round_trip_demo() {
        let dir = std::env::temp_dir();
        let path = dir.join("circuita-test.circuita");
        let p = crate::app::demo_project();
        save(&p, &path).expect("save");
        let q = load(&path).expect("load");
        assert_eq!(p.comps.len(), q.comps.len());
        assert_eq!(p.wires.len(), q.wires.len());
        assert_eq!(p.name, q.name);
        for (a, b) in p.comps.iter().zip(q.comps.iter()) {
            assert_eq!(a.kind, b.kind);
            assert_eq!(a.label, b.label);
            assert_eq!(a.value, b.value);
            assert_eq!(a.rot, b.rot);
            assert_eq!(a.mirror, b.mirror);
            assert!((a.pos.x - b.pos.x).abs() < 0.2);
            assert!((a.pos.y - b.pos.y).abs() < 0.2);
        }
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn weird_names_survive() {
        let mut p = Project::default();
        let mut c = Component::new(0, Kind::Resistor, eframe_pos(0.0, 0.0), String::new());
        c.label = "R\t1".into();
        c.value = "4\\7k".into();
        p.add_component(c);
        p.name = "tab\tname".into();
        let path = std::env::temp_dir().join("circuita-esc.circuita");
        save(&p, &path).unwrap();
        let q = load(&path).unwrap();
        assert_eq!(q.comps[0].label, "R\t1");
        assert_eq!(q.comps[0].value, "4\\7k");
        assert_eq!(q.name, "tab\tname");
        let _ = std::fs::remove_file(&path);
    }
}
