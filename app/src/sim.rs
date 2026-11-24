//! Netlist extraction and a small modified-nodal-analysis solver.
//!
//! Analog parts (resistors, sources, LEDs, switches) are solved every frame with
//! conductance stamping. Digital parts (gates and the boards) drive their output
//! pin through a Thevenin resistance, and are evaluated a few times per frame so
//! cascades settle. The solver also derives per-pin currents, which the canvas
//! turns into flowing dots along each wire.

use crate::model::{Component, Kind, PinRef, PinRole, Project};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Clone, Debug, Default)]
pub struct Net {
    pub id: usize,
    pub name: String,
    pub pins: Vec<PinRef>,
    pub is_ground: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Netlist {
    pub nets: Vec<Net>,
    pub pin_net: HashMap<PinRef, usize>,
    pub wire_net: HashMap<usize, usize>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Level {
    High,
    Low,
    Mid,
}

#[derive(Clone, Debug, Default)]
pub struct SimResult {
    pub volts: Vec<f64>,
    pub levels: Vec<Level>,
    pub pin_inj: HashMap<PinRef, f64>,
    pub comp_current: HashMap<usize, f64>,
    pub wire_flow: HashMap<usize, f64>,
    pub led: HashMap<usize, f32>,
    pub gate_out: HashMap<usize, bool>,
    pub short: bool,
    pub active_nodes: usize,
}

impl SimResult {
    pub fn volt(&self, net: usize) -> f64 {
        self.volts.get(net).copied().unwrap_or(0.0)
    }

    pub fn level(&self, net: usize) -> Level {
        self.levels.get(net).copied().unwrap_or(Level::Low)
    }

    pub fn comp_amp(&self, comp: usize) -> f64 {
        self.comp_current.get(&comp).copied().unwrap_or(0.0)
    }
}

fn find(parent: &mut Vec<usize>, mut x: usize) -> usize {
    while parent[x] != x {
        parent[x] = parent[parent[x]];
        x = parent[x];
    }
    x
}

fn union(parent: &mut Vec<usize>, a: usize, b: usize) {
    let ra = find(parent, a);
    let rb = find(parent, b);
    if ra != rb {
        parent[rb] = ra;
    }
}

pub fn build_netlist(project: &Project) -> Netlist {
    let mut pins: Vec<PinRef> = Vec::new();
    let mut index_of: HashMap<PinRef, usize> = HashMap::new();
    for c in &project.comps {
        for p in 0..c.pins().len() {
            let r = PinRef {
                comp: c.id,
                pin: p,
            };
            index_of.insert(r, pins.len());
            pins.push(r);
        }
    }
    if pins.is_empty() {
        return Netlist {
            nets: vec![Net {
                id: 0,
                name: "GND".into(),
                pins: vec![],
                is_ground: true,
            }],
            pin_net: HashMap::new(),
            wire_net: HashMap::new(),
        };
    }

    let mut parent: Vec<usize> = (0..pins.len()).collect();
    for w in &project.wires {
        if let (Some(&ia), Some(&ib)) = (index_of.get(&w.a), index_of.get(&w.b)) {
            union(&mut parent, ia, ib);
        }
    }

    // Every ground pin joins a single reference net.
    let ground_pins: Vec<usize> = pins
        .iter()
        .enumerate()
        .filter(|(_, p)| {
            project
                .comp(p.comp)
                .map(|c| c.pins()[p.pin].role == PinRole::Ground)
                .unwrap_or(false)
        })
        .map(|(i, _)| i)
        .collect();
    let has_ground = !ground_pins.is_empty();
    if let Some(&first) = ground_pins.first() {
        for &other in ground_pins.iter().skip(1) {
            union(&mut parent, first, other);
        }
    }

    // Root order: the reference first, then by first appearance.
    let mut roots: Vec<usize> = Vec::new();
    for i in 0..pins.len() {
        let r = find(&mut parent, i);
        if !roots.contains(&r) {
            roots.push(r);
        }
    }
    let ref_root = ground_pins
        .first()
        .map(|i| find(&mut parent, *i))
        .or_else(|| roots.first().copied());

    let mut ordered: Vec<usize> = Vec::new();
    if let Some(r) = ref_root {
        ordered.push(r);
    }
    for r in roots {
        if Some(r) != ref_root {
            ordered.push(r);
        }
    }

    let mut root_to_net: HashMap<usize, usize> = HashMap::new();
    let mut nets: Vec<Net> = Vec::new();
    for (net_id, root) in ordered.iter().enumerate() {
        root_to_net.insert(*root, net_id);
        nets.push(Net {
            id: net_id,
            name: String::new(),
            pins: Vec::new(),
            is_ground: Some(*root) == ref_root && has_ground,
        });
    }

    for (i, p) in pins.iter().enumerate() {
        let root = find(&mut parent, i);
        let net_id = *root_to_net.get(&root).unwrap_or(&0);
        nets[net_id].pins.push(*p);
    }

    let pin_net: HashMap<PinRef, usize> = nets
        .iter()
        .flat_map(|n| n.pins.iter().map(move |p| (*p, n.id)))
        .collect();
    let mut wire_net = HashMap::new();
    for w in &project.wires {
        if let Some(&net) = pin_net.get(&w.a) {
            wire_net.insert(w.id, net);
        }
    }

    // Readable names from the pins that landed on each net.
    let mut used: HashSet<String> = HashSet::new();
    used.insert("GND".into());
    for net in nets.iter_mut() {
        if net.id == 0 {
            net.name = "GND".into();
            continue;
        }
        let mut best: Option<String> = None;
        for p in &net.pins {
            if let Some(c) = project.comp(p.comp) {
                let pin = c.pins()[p.pin];
                if pin.role == PinRole::Ground {
                    best = Some("GND".into());
                    break;
                }
                if best.is_none()
                    && matches!(pin.role, PinRole::Power | PinRole::Output | PinRole::Input)
                {
                    best = Some(pin.name.to_string());
                }
            }
        }
        let mut name = best.unwrap_or_else(|| format!("N{}", net.id));
        if used.contains(&name) {
            name = format!("{name}.{}", net.id);
        }
        used.insert(name.clone());
        net.name = name;
    }
    if nets.is_empty() {
        nets.push(Net {
            id: 0,
            name: "GND".into(),
            pins: vec![],
            is_ground: true,
        });
    }

    Netlist {
        nets,
        pin_net,
        wire_net,
    }
}

#[derive(Clone, Debug)]
enum Element {
    Res {
        comp: usize,
        a: PinRef,
        b: PinRef,
        na: usize,
        nb: usize,
        r: f64,
    },
    Source {
        comp: usize,
        a: Option<PinRef>,
        b: Option<PinRef>,
        na: usize,
        nb: usize,
        v: f64,
    },
    Drive {
        comp: usize,
        pin: PinRef,
        node: usize,
        v: f64,
        r: f64,
    },
}

pub struct Solver {
    prev_gate: HashMap<usize, f64>,
}

impl Default for Solver {
    fn default() -> Self {
        Solver {
            prev_gate: HashMap::new(),
        }
    }
}

impl Solver {
    pub fn reset(&mut self) {
        self.prev_gate.clear();
    }

    pub fn solve(&mut self, project: &Project, netlist: &Netlist, time: f64) -> SimResult {
        let mut gate: HashMap<usize, f64> = HashMap::new();
        for c in &project.comps {
            if is_gate(c.kind) {
                gate.insert(c.id, self.prev_gate.get(&c.id).copied().unwrap_or(0.0));
            }
        }

        let mut result = SimResult::default();
        for _pass in 0..8 {
            let elements = build_elements(project, netlist, time, &gate);
            match solve_once(project, netlist, &elements) {
                Some(res) => {
                    result = res;
                    let mut changed = false;
                    for c in &project.comps {
                        if is_gate(c.kind) {
                            let out = eval_gate(c, netlist, &result) * 5.0;
                            let old = gate.get(&c.id).copied().unwrap_or(0.0);
                            if (out - old).abs() > 0.01 {
                                changed = true;
                            }
                            gate.insert(c.id, out);
                        }
                    }
                    if !changed {
                        break;
                    }
                }
                None => {
                    result.short = true;
                    result.volts = vec![0.0; netlist.nets.len()];
                    result.levels = vec![Level::Low; netlist.nets.len()];
                    break;
                }
            }
        }
        self.prev_gate = gate.clone();

        for v in result.volts.iter_mut() {
            if v.abs() < 1e-6 {
                *v = 0.0;
            }
        }
        result.levels = result
            .volts
            .iter()
            .enumerate()
            .map(|(i, v)| {
                if i == 0 || netlist.nets.get(i).map(|n| n.is_ground).unwrap_or(false) {
                    Level::Low
                } else if *v > 2.0 {
                    Level::High
                } else if *v < 0.8 {
                    Level::Low
                } else {
                    Level::Mid
                }
            })
            .collect();
        result.gate_out = gate
            .iter()
            .map(|(id, v)| (*id, *v > 2.5))
            .collect();
        compute_wire_flow(project, netlist, &mut result);

        if result.comp_current.values().any(|i| i.abs() > 30.0) {
            result.short = true;
        }
        result
    }
}

fn build_elements(
    project: &Project,
    netlist: &Netlist,
    time: f64,
    gate: &HashMap<usize, f64>,
) -> Vec<Element> {
    let net_of = |p: PinRef| netlist.pin_net.get(&p).copied().unwrap_or(0);
    let mut out = Vec::new();
    for c in &project.comps {
        let p = |n: usize| PinRef {
            comp: c.id,
            pin: n,
        };
        let res = |n0: usize, n1: usize, r: f64| Element::Res {
            comp: c.id,
            a: p(n0),
            b: p(n1),
            na: net_of(p(n0)),
            nb: net_of(p(n1)),
            r,
        };
        match c.kind {
            Kind::Resistor => out.push(res(0, 1, c.value_numeric().max(0.01))),
            Kind::Capacitor => out.push(res(0, 1, 1.0e9)),
            Kind::Inductor => out.push(res(0, 1, 0.08)),
            Kind::Led => out.push(res(0, 1, 220.0)),
            Kind::SwitchSpst => out.push(res(0, 1, if c.state { 0.02 } else { 1.0e9 })),
            Kind::Potentiometer => {
                let total = c.value_numeric().max(1.0);
                let w = c.pot.clamp(0.002, 0.998) as f64;
                out.push(res(0, 2, total * w));
                out.push(res(2, 1, total * (1.0 - w)));
            }
            Kind::DcSource => out.push(Element::Source {
                comp: c.id,
                a: Some(p(0)),
                b: Some(p(1)),
                na: net_of(p(0)),
                nb: net_of(p(1)),
                v: c.value_numeric(),
            }),
            Kind::AcSource => {
                let amp = c.value_numeric();
                let f = c.freq.max(0.01) as f64;
                out.push(Element::Source {
                    comp: c.id,
                    a: Some(p(0)),
                    b: Some(p(1)),
                    na: net_of(p(0)),
                    nb: net_of(p(1)),
                    v: amp * (std::f64::consts::TAU * f * time).sin(),
                });
            }
            Kind::Vcc => out.push(Element::Source {
                comp: c.id,
                a: Some(p(0)),
                b: None,
                na: net_of(p(0)),
                nb: 0,
                v: c.value_numeric(),
            }),
            Kind::GateAnd | Kind::GateOr | Kind::GateNot | Kind::GateNand | Kind::GateXor => {
                out.push(Element::Drive {
                    comp: c.id,
                    pin: p(2),
                    node: net_of(p(2)),
                    v: gate.get(&c.id).copied().unwrap_or(0.0),
                    r: 470.0,
                });
            }
            Kind::UnoBoard | Kind::Atmega | Kind::Esp32 | Kind::NanoBoard => {
                let (supply, out_pin) = match c.kind {
                    Kind::Esp32 => (3.3, 2),
                    _ => (5.0, 2),
                };
                // On-board regulator: modelled with a little series resistance so
                // an external rail feeding the same net never makes the solve singular.
                out.push(Element::Drive {
                    comp: c.id,
                    pin: p(0),
                    node: net_of(p(0)),
                    v: supply,
                    r: 0.4,
                });
                let blink = if (time * 2.0).rem_euclid(2.0) < 1.0 {
                    supply
                } else {
                    0.0
                };
                out.push(Element::Drive {
                    comp: c.id,
                    pin: p(out_pin),
                    node: net_of(p(out_pin)),
                    v: blink,
                    r: 330.0,
                });
            }
            // Sensor and imported parts are passive from the solver's point of
            // view: their pins join nets and carry measured levels, but they do
            // not inject a source into the DC/AC matrix.
            Kind::Ground
            | Kind::Mpu6050
            | Kind::Ina219
            | Kind::Dht22
            | Kind::Lcd1602
            | Kind::PushButton
            | Kind::Custom => {}
        }
    }
    out
}

fn is_gate(k: Kind) -> bool {
    matches!(
        k,
        Kind::GateAnd | Kind::GateOr | Kind::GateNot | Kind::GateNand | Kind::GateXor
    )
}

fn eval_gate(c: &Component, netlist: &Netlist, res: &SimResult) -> f64 {
    let level = |pin: usize| -> bool {
        let net = netlist
            .pin_net
            .get(&PinRef {
                comp: c.id,
                pin,
            })
            .copied()
            .unwrap_or(0);
        res.volt(net) > 2.4
    };
    match c.kind {
        Kind::GateAnd => (level(0) && level(1)) as u8 as f64,
        Kind::GateOr => (level(0) || level(1)) as u8 as f64,
        Kind::GateNot => (!level(0)) as u8 as f64,
        Kind::GateNand => (!(level(0) && level(1))) as u8 as f64,
        Kind::GateXor => (level(0) != level(1)) as u8 as f64,
        _ => 0.0,
    }
}

fn solve_once(project: &Project, netlist: &Netlist, elements: &[Element]) -> Option<SimResult> {
    let net_count = netlist.nets.len();
    let nodes = net_count.saturating_sub(1);
    let sources = elements
        .iter()
        .filter(|e| matches!(e, Element::Source { .. }))
        .count();
    let size = nodes + sources;
    let mut a = vec![0.0f64; size * size];
    let mut b = vec![0.0f64; size];
    let idx = |net: usize| -> Option<usize> {
        if net == 0 {
            None
        } else {
            Some(net - 1)
        }
    };

    // A whisper of leakage keeps floating sub-circuits defined.
    for net in 1..net_count {
        a[(net - 1) * size + (net - 1)] += 1e-9;
    }

    let mut src_slot: Vec<usize> = vec![usize::MAX; elements.len()];
    let mut source_row = 0usize;
    for (ei, e) in elements.iter().enumerate() {
        match *e {
            Element::Res { na, nb, r, .. } => {
                if na == nb {
                    continue;
                }
                let cond = 1.0 / r.max(1e-6);
                if let Some(i) = idx(na) {
                    a[i * size + i] += cond;
                }
                if let Some(j) = idx(nb) {
                    a[j * size + j] += cond;
                }
                if let (Some(i), Some(j)) = (idx(na), idx(nb)) {
                    a[i * size + j] -= cond;
                    a[j * size + i] -= cond;
                }
            }
            Element::Source { na, nb, v, .. } => {
                src_slot[ei] = source_row;
                if na == nb {
                    source_row += 1;
                    continue;
                }
                let k = nodes + source_row;
                if let Some(i) = idx(na) {
                    a[i * size + k] += 1.0;
                    a[k * size + i] += 1.0;
                }
                if let Some(j) = idx(nb) {
                    a[j * size + k] -= 1.0;
                    a[k * size + j] -= 1.0;
                }
                b[k] += v;
                source_row += 1;
            }
            Element::Drive { node, v, r, .. } => {
                let Some(i) = idx(node) else { continue };
                let gg = 1.0 / r.max(0.01);
                a[i * size + i] += gg;
                b[i] += v * gg;
            }
        }
    }

    if size == 0 {
        return Some(SimResult::default());
    }
    let x = gauss(a, b, size)?;

    let mut res = SimResult {
        volts: vec![0.0; net_count],
        levels: vec![Level::Low; net_count],
        active_nodes: nodes,
        ..Default::default()
    };
    for net in 1..net_count {
        res.volts[net] = x[net - 1];
    }

    let mut inj: HashMap<PinRef, f64> = HashMap::new();
    for (ei, e) in elements.iter().enumerate() {
        match *e {
            Element::Res {
                comp,
                a: pa,
                b: pb,
                na,
                nb,
                r,
            } => {
                let i = (res.volts[na] - res.volts[nb]) / r.max(1e-6);
                *inj.entry(pa).or_insert(0.0) -= i;
                *inj.entry(pb).or_insert(0.0) += i;
                res.comp_current.insert(comp, i);
                if let Some(c) = project.comp(comp) {
                    if c.kind == Kind::Led {
                        let bright = (i.abs() / 0.012).clamp(0.0, 1.0) as f32;
                        res.led.insert(comp, bright);
                    }
                }
            }
            Element::Source {
                comp, a: pa, b: pb, ..
            } => {
                let k = nodes + src_slot[ei];
                let i = if src_slot[ei] != usize::MAX && k < x.len() {
                    x[k]
                } else {
                    0.0
                };
                if let Some(pa) = pa {
                    *inj.entry(pa).or_insert(0.0) -= i;
                }
                if let Some(pb) = pb {
                    *inj.entry(pb).or_insert(0.0) += i;
                }
                res.comp_current.insert(comp, i);
            }
            Element::Drive {
                comp,
                pin,
                node,
                v,
                r,
            } => {
                let i = (v - res.volts[node]) / r.max(0.01);
                *inj.entry(pin).or_insert(0.0) += i;
                res.comp_current.insert(comp, i);
            }
        }
    }
    res.pin_inj = inj;
    Some(res)
}

fn gauss(mut a: Vec<f64>, mut b: Vec<f64>, n: usize) -> Option<Vec<f64>> {
    for col in 0..n {
        let mut pivot = col;
        let mut best = a[col * n + col].abs();
        for row in (col + 1)..n {
            let v = a[row * n + col].abs();
            if v > best {
                best = v;
                pivot = row;
            }
        }
        if best < 1e-14 {
            return None;
        }
        if pivot != col {
            for k in 0..n {
                a.swap(col * n + k, pivot * n + k);
            }
            b.swap(col, pivot);
        }
        let d = a[col * n + col];
        for row in (col + 1)..n {
            let f = a[row * n + col] / d;
            if f == 0.0 {
                continue;
            }
            for k in col..n {
                a[row * n + k] -= f * a[col * n + k];
            }
            b[row] -= f * b[col];
        }
    }
    let mut x = vec![0.0; n];
    for row in (0..n).rev() {
        let mut sum = b[row];
        for k in (row + 1)..n {
            sum -= a[row * n + k] * x[k];
        }
        x[row] = sum / a[row * n + row];
    }
    if x.iter().any(|v| !v.is_finite()) {
        return None;
    }
    Some(x)
}

/// Spread pin currents over the wire graph so each wire gets a direction and a
/// magnitude. The net is traversed as a tree, and every subtree's summed
/// injection is the current that must flow through the edge above it.
fn compute_wire_flow(project: &Project, netlist: &Netlist, res: &mut SimResult) {
    let mut adj: HashMap<PinRef, Vec<(usize, PinRef)>> = HashMap::new();
    for w in &project.wires {
        adj.entry(w.a).or_default().push((w.id, w.b));
        adj.entry(w.b).or_default().push((w.id, w.a));
    }
    let mut flows: HashMap<usize, f64> = HashMap::new();
    let mut seen_pins: HashSet<PinRef> = HashSet::new();

    for net in &netlist.nets {
        let pins: HashSet<PinRef> = net.pins.iter().copied().collect();
        let net_pins: Vec<PinRef> = net
            .pins
            .iter()
            .copied()
            .filter(|p| adj.contains_key(p))
            .collect();
        if net_pins.is_empty() {
            continue;
        }
        let root = *net_pins
            .iter()
            .max_by(|a, b| {
                inj_of(res, **a)
                    .abs()
                    .partial_cmp(&inj_of(res, **b).abs())
                    .unwrap()
            })
            .unwrap();

        let mut parent: HashMap<PinRef, (PinRef, usize)> = HashMap::new();
        let mut order: Vec<PinRef> = Vec::new();
        let mut queue: VecDeque<PinRef> = VecDeque::new();
        queue.push_back(root);
        seen_pins.insert(root);
        while let Some(p) = queue.pop_front() {
            order.push(p);
            if let Some(list) = adj.get(&p) {
                for (wid, other) in list {
                    if !pins.contains(other) || seen_pins.contains(other) {
                        continue;
                    }
                    seen_pins.insert(*other);
                    parent.insert(*other, (p, *wid));
                    queue.push_back(*other);
                }
            }
        }

        let mut subtotal: HashMap<PinRef, f64> = HashMap::new();
        for p in order.iter().rev() {
            let mut sum = inj_of(res, *p);
            if let Some(list) = adj.get(p) {
                for (_, other) in list {
                    if parent.get(other).map(|(pp, _)| pp == p).unwrap_or(false) {
                        sum += subtotal.get(other).copied().unwrap_or(0.0);
                    }
                }
            }
            subtotal.insert(*p, sum);
        }

        for child in order.iter() {
            if let Some((_par, wid)) = parent.get(child) {
                let current = subtotal.get(child).copied().unwrap_or(0.0);
                if let Some(w) = project.wire(*wid) {
                    let signed = if w.a == *child { current } else { -current };
                    flows.insert(*wid, signed);
                }
            }
        }
    }
    res.wire_flow = flows;
}

fn inj_of(res: &SimResult, p: PinRef) -> f64 {
    res.pin_inj.get(&p).copied().unwrap_or(0.0)
}
