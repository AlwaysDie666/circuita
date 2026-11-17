//! Circuita — a schematic capture and circuit simulation studio.
//!
//! Run normally for the interactive editor, or with `--screenshot <file.png>`
//! to boot headless-ish (unfocused, at the bottom of the window stack), draw a
//! few frames, save the window to PNG and quit. The screenshot path is how the
//! build is checked without anyone touching the mouse.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod canvas;
mod instruments;
mod model;
mod panels;
mod routing;
mod sim;
mod symbols;
mod theme;
mod ui_kit;

use std::path::PathBuf;

fn main() -> eframe::Result<()> {
    let screenshot = parse_screenshot_arg();

    let mut viewport = eframe::egui::ViewportBuilder::default()
        .with_title("Circuita — Schematic Studio")
        .with_inner_size([1600.0, 1000.0])
        .with_min_inner_size([1120.0, 720.0])
        .with_app_id("circuita")
        .with_icon(app_icon());

    if screenshot.is_some() {
        // Never steal focus while capturing: the window only has to paint.
        viewport = viewport.with_active(false);
        #[cfg(target_os = "windows")]
        {
            viewport = viewport.with_window_level(eframe::egui::WindowLevel::AlwaysOnBottom);
        }
        #[cfg(target_os = "macos")]
        {
            viewport = viewport.with_activate_ignoring_other_apps(false);
        }
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    let shot = screenshot.clone();
    eframe::run_native(
        "Circuita",
        options,
        Box::new(move |cc| Ok(Box::new(app::App::new(cc, shot.clone())))),
    )
}

/// Accepts `--screenshot out.png` and `--screenshot=out.png`.
fn parse_screenshot_arg() -> Option<PathBuf> {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--screenshot" {
            return args.next().map(PathBuf::from);
        }
        if let Some(rest) = arg.strip_prefix("--screenshot=") {
            if !rest.is_empty() {
                return Some(PathBuf::from(rest));
            }
        }
    }
    None
}

/// A 64×64 window icon: an orange junction node on a matte black tile.
fn app_icon() -> eframe::egui::IconData {
    const S: usize = 64;
    let mut rgba = vec![0u8; S * S * 4];
    let radius = 15.0f32;
    for y in 0..S {
        for x in 0..S {
            let i = (y * S + x) * 4;
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;
            // Rounded square coverage.
            let dx = (px - S as f32 * 0.5).abs() - (S as f32 * 0.5 - radius);
            let dy = (py - S as f32 * 0.5).abs() - (S as f32 * 0.5 - radius);
            let outside = (dx.max(0.0).powi(2) + dy.max(0.0).powi(2)).sqrt()
                + dx.max(dy).min(0.0)
                - radius;
            let inside = outside < 0.0;
            if !inside {
                continue;
            }
            // Orange tile with a black circuit glyph: a wire with a junction.
            let on_h = (py - 40.0).abs() < 3.0 && (14.0..50.0).contains(&px);
            let on_v = (px - 40.0).abs() < 3.0 && (16.0..40.0).contains(&py);
            let node = ((px - 40.0).powi(2) + (py - 40.0).powi(2)).sqrt() < 7.0;
            let glyph = on_h || on_v || node;
            let (r, g, b) = if glyph {
                (0x12u8, 0x12u8, 0x12u8)
            } else {
                (0xffu8, 0x6bu8, 0x00u8)
            };
            rgba[i] = r;
            rgba[i + 1] = g;
            rgba[i + 2] = b;
            rgba[i + 3] = 255;
        }
    }
    eframe::egui::IconData {
        rgba,
        width: S as u32,
        height: S as u32,
    }
}
