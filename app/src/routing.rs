//! Orthogonal auto-routing with a small cost model: shortest path wins unless it
//! cuts through a symbol, crosses existing wiring or starts by going backwards
//! out of a pin.

use crate::model::Dir;
use eframe::egui::{pos2, Pos2, Rect};

const STUB: f32 = 14.0;

pub fn route(
    a: Pos2,
    da: Dir,
    b: Pos2,
    db: Dir,
    obstacles: &[Rect],
    wires: &[(Pos2, Pos2)],
) -> Vec<Pos2> {
    let s = a + da.vec() * STUB;
    let e = b + db.vec() * STUB;

    let mut candidates: Vec<Vec<Pos2>> = Vec::new();

    // Straight run when the stubs already line up.
    if (s.x - e.x).abs() < 0.01 || (s.y - e.y).abs() < 0.01 {
        candidates.push(vec![s, e]);
    }

    // Plain L shapes.
    candidates.push(vec![s, pos2(e.x, s.y), e]);
    candidates.push(vec![s, pos2(s.x, e.y), e]);

    // Z shapes through a corridor.
    let mut xs: Vec<f32> = vec![(s.x + e.x) * 0.5, s.x, e.x];
    let mut ys: Vec<f32> = vec![(s.y + e.y) * 0.5, s.y, e.y];
    for r in obstacles {
        xs.push(r.left() - 10.0);
        xs.push(r.right() + 10.0);
        ys.push(r.top() - 10.0);
        ys.push(r.bottom() + 10.0);
    }
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    xs.dedup_by(|a, b| (*a - *b).abs() < 0.5);
    ys.sort_by(|a, b| a.partial_cmp(b).unwrap());
    ys.dedup_by(|a, b| (*a - *b).abs() < 0.5);

    for x in &xs {
        candidates.push(vec![s, pos2(*x, s.y), pos2(*x, e.y), e]);
    }
    for y in &ys {
        candidates.push(vec![s, pos2(s.x, *y), pos2(e.x, *y), e]);
    }

    let mut best: Option<(f32, Vec<Pos2>)> = None;
    for mid in candidates {
        let mut path = Vec::with_capacity(mid.len() + 2);
        path.push(a);
        path.extend(mid.iter().copied());
        path.push(b);
        let path = simplify(path);
        if path.len() < 2 {
            continue;
        }
        let cost = path_cost(&path, da, db, obstacles, wires);
        if best.as_ref().map(|(c, _)| cost < *c).unwrap_or(true) {
            best = Some((cost, path));
        }
    }

    best.map(|(_, p)| p).unwrap_or_else(|| vec![a, b])
}

pub fn simplify(mut pts: Vec<Pos2>) -> Vec<Pos2> {
    pts.dedup_by(|a, b| a.distance(*b) < 0.05);
    let mut out: Vec<Pos2> = Vec::with_capacity(pts.len());
    for p in pts {
        if let Some(&last) = out.last() {
            if last.distance(p) < 0.05 {
                continue;
            }
        }
        if out.len() >= 2 {
            let a = out[out.len() - 2];
            let b = out[out.len() - 1];
            let cross = (b.x - a.x) * (p.y - b.y) - (b.y - a.y) * (p.x - b.x);
            let same_dir = ((b.x - a.x) * (p.x - b.x) >= 0.0) && ((b.y - a.y) * (p.y - b.y) >= 0.0);
            if cross.abs() < 0.01 && same_dir {
                out.pop();
            }
        }
        out.push(p);
    }
    out
}

fn path_cost(
    path: &[Pos2],
    da: Dir,
    db: Dir,
    obstacles: &[Rect],
    wires: &[(Pos2, Pos2)],
) -> f32 {
    let mut cost = 0.0f32;
    let mut bends = 0usize;
    for (i, seg) in path.windows(2).enumerate() {
        let len = seg[0].distance(seg[1]);
        cost += len;
        // Crossing a wire is worse than being long.
        for (w1, w2) in wires {
            if seg_cross(seg[0], seg[1], *w1, *w2) {
                cost += 90.0;
            }
        }
        // Cutting through a symbol body is almost never right.
        for r in obstacles {
            if seg_hits_rect(seg[0], seg[1], *r) {
                cost += 2400.0;
            }
        }
        if i > 0 {
            let p = path[i - 1];
            let mid = path[i];
            let cross = (mid.x - p.x) * (seg[1].y - mid.y) - (mid.y - p.y) * (seg[1].x - mid.x);
            if cross.abs() > 0.01 {
                bends += 1;
            }
        }
    }
    cost += bends as f32 * 9.0;

    // Leaving a pin means travelling along its outward direction first, and
    // arriving means coming in from the outside.
    if path.len() > 1 {
        let first = path[1] - path[0];
        if first.dot(da.vec()) < 0.0 {
            cost += 400.0;
        }
    }
    if path.len() > 2 {
        let last = path[path.len() - 1] - path[path.len() - 2];
        if last.dot(db.vec()) > 0.0 {
            cost += 400.0;
        }
    }
    cost
}

fn seg_hits_rect(a: Pos2, b: Pos2, r: Rect) -> bool {
    // Liang-Barsky clipping against an axis aligned box.
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let mut t0 = 0.0f32;
    let mut t1 = 1.0f32;
    let checks = [
        (-dx, a.x - r.left()),
        (dx, r.right() - a.x),
        (-dy, a.y - r.top()),
        (dy, r.bottom() - a.y),
    ];
    for (p, q) in checks {
        if p.abs() < 1e-6 {
            if q < 0.0 {
                return false;
            }
        } else {
            let t = q / p;
            if p < 0.0 {
                t0 = t0.max(t);
            } else {
                t1 = t1.min(t);
            }
            if t0 > t1 {
                return false;
            }
        }
    }
    true
}

fn orient(a: Pos2, b: Pos2, c: Pos2) -> f32 {
    (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
}

fn seg_cross(a1: Pos2, a2: Pos2, b1: Pos2, b2: Pos2) -> bool {
    let d1 = orient(b1, b2, a1);
    let d2 = orient(b1, b2, a2);
    let d3 = orient(a1, a2, b1);
    let d4 = orient(a1, a2, b2);
    ((d1 > 0.01 && d2 < -0.01) || (d1 < -0.01 && d2 > 0.01))
        && ((d3 > 0.01 && d4 < -0.01) || (d3 < -0.01 && d4 > 0.01))
}

/// Total polyline length helper shared with the canvas.
pub fn polyline_length(pts: &[Pos2]) -> f32 {
    pts.windows(2).map(|w| w[0].distance(w[1])).sum()
}

/// Point at a given distance along a polyline, used for the flowing current dots.
pub fn point_along(pts: &[Pos2], dist: f32) -> Option<Pos2> {
    let mut d = dist.max(0.0);
    for seg in pts.windows(2) {
        let len = seg[0].distance(seg[1]);
        if d <= len && len > 0.001 {
            let t = d / len;
            return Some(seg[0] + (seg[1] - seg[0]) * t);
        }
        d -= len;
    }
    None
}
