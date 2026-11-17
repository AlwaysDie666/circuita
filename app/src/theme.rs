//! Visual language: matte black surfaces, restrained greys and one hot orange accent.

use eframe::egui::{
    self, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Stroke, TextStyle,
    Visuals,
};
use std::sync::Arc;

pub const BG: Color32 = Color32::from_rgb(0x12, 0x12, 0x12);
pub const CANVAS: Color32 = Color32::from_rgb(0x10, 0x10, 0x10);
pub const PANEL: Color32 = Color32::from_rgb(0x15, 0x15, 0x15);
pub const PANEL_2: Color32 = Color32::from_rgb(0x1a, 0x1a, 0x1a);
pub const ELEV: Color32 = Color32::from_rgb(0x1f, 0x1f, 0x1f);
pub const ELEV_2: Color32 = Color32::from_rgb(0x26, 0x26, 0x26);
pub const BORDER: Color32 = Color32::from_rgb(0x2a, 0x2a, 0x2a);
pub const BORDER_SOFT: Color32 = Color32::from_rgb(0x22, 0x22, 0x22);
pub const BORDER_HI: Color32 = Color32::from_rgb(0x3a, 0x3a, 0x3a);
pub const TEXT: Color32 = Color32::from_rgb(0xf2, 0xf2, 0xf2);
pub const TEXT_DIM: Color32 = Color32::from_rgb(0x9a, 0x9a, 0x9a);
pub const TEXT_FAINT: Color32 = Color32::from_rgb(0x64, 0x64, 0x64);
pub const ACCENT: Color32 = Color32::from_rgb(0xff, 0x6b, 0x00);
pub const ACCENT_HI: Color32 = Color32::from_rgb(0xff, 0x8c, 0x33);
pub const ACCENT_DARK: Color32 = Color32::from_rgb(0x4a, 0x21, 0x00);
pub const HIGH: Color32 = Color32::from_rgb(0xff, 0x4d, 0x4d);
pub const LOW: Color32 = Color32::from_rgb(0x3b, 0x8c, 0xff);
pub const MID: Color32 = Color32::from_rgb(0xd8, 0xa8, 0x3c);
pub const OK: Color32 = Color32::from_rgb(0x3d, 0xd5, 0x9c);
pub const GRID_DOT: Color32 = Color32::from_rgb(0x27, 0x27, 0x27);
pub const GRID_DOT_HI: Color32 = Color32::from_rgb(0x33, 0x33, 0x33);
pub const WIRE: Color32 = Color32::from_rgb(0x7c, 0x7c, 0x7c);
pub const SYMBOL: Color32 = Color32::from_rgb(0xd8, 0xd8, 0xd8);
pub const SYMBOL_FILL: Color32 = Color32::from_rgb(0x1a, 0x1a, 0x1a);

pub fn display(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("semibold".into()))
}

pub fn medium(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("medium".into()))
}

pub fn body(size: f32) -> FontId {
    FontId::new(size, FontFamily::Proportional)
}

pub fn mono(size: f32) -> FontId {
    FontId::new(size, FontFamily::Monospace)
}

pub fn accent_glow(alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(0xff, 0x6b, 0x00, alpha)
}

pub fn with_alpha(c: Color32, a: f32) -> Color32 {
    let a = (a.clamp(0.0, 1.0) * c.a() as f32) as u8;
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), a)
}

pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    Color32::from_rgb(
        (a.r() as f32 + (b.r() as f32 - a.r() as f32) * t) as u8,
        (a.g() as f32 + (b.g() as f32 - a.g() as f32) * t) as u8,
        (a.b() as f32 + (b.b() as f32 - a.b() as f32) * t) as u8,
    )
}

pub fn install_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    let load = |bytes: &'static [u8]| FontData::from_static(bytes).into();

    fonts.font_data.insert("inter".into(), load(include_bytes!("../assets/Inter-Regular.ttf")));
    fonts.font_data.insert("inter-medium".into(), load(include_bytes!("../assets/Inter-Medium.ttf")));
    fonts.font_data.insert("inter-semibold".into(), load(include_bytes!("../assets/Inter-SemiBold.ttf")));
    fonts.font_data.insert("inter-bold".into(), load(include_bytes!("../assets/Inter-Bold.ttf")));

    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, "inter".into());
    fonts.families.insert(
        FontFamily::Name("medium".into()),
        vec!["inter-medium".into(), "inter".into()],
    );
    fonts.families.insert(
        FontFamily::Name("semibold".into()),
        vec!["inter-semibold".into(), "inter".into()],
    );
    fonts.families.insert(
        FontFamily::Name("bold".into()),
        vec!["inter-bold".into(), "inter-semibold".into()],
    );
    fonts
        .families
        .get_mut(&FontFamily::Monospace)
        .unwrap()
        .insert(0, "inter".into());

    ctx.set_fonts(fonts);
    let _ = Arc::new(());
}

pub fn install_style(ctx: &egui::Context) {
    // The studio is always dark, whatever the OS says.
    ctx.set_theme(egui::ThemePreference::Dark);
    ctx.all_styles_mut(configure_style);
}

fn configure_style(style: &mut egui::Style) {
    let mut v = Visuals::dark();
    v.override_text_color = Some(TEXT);
    v.panel_fill = PANEL;
    v.window_fill = PANEL;
    v.extreme_bg_color = Color32::from_rgb(0x0d, 0x0d, 0x0d);
    v.faint_bg_color = PANEL_2;
    v.window_stroke = Stroke::new(1.0_f32, BORDER);
    v.window_corner_radius = CornerRadius::same(12);
    v.menu_corner_radius = CornerRadius::same(10);
    v.selection.bg_fill = ACCENT;
    v.selection.stroke = Stroke::new(1.0_f32, Color32::from_rgb(0x1a, 0x0d, 0x00));
    v.hyperlink_color = ACCENT_HI;
    v.slider_trailing_fill = true;

    let w = &mut v.widgets;
    w.noninteractive.bg_fill = PANEL;
    w.noninteractive.weak_bg_fill = PANEL;
    w.noninteractive.bg_stroke = Stroke::new(1.0_f32, BORDER);
    w.noninteractive.fg_stroke = Stroke::new(1.0_f32, TEXT_DIM);
    w.noninteractive.corner_radius = CornerRadius::same(8);

    w.inactive.bg_fill = ELEV;
    w.inactive.weak_bg_fill = ELEV;
    w.inactive.bg_stroke = Stroke::new(1.0_f32, BORDER);
    w.inactive.fg_stroke = Stroke::new(1.0_f32, TEXT);
    w.inactive.corner_radius = CornerRadius::same(8);

    w.hovered.bg_fill = ELEV_2;
    w.hovered.weak_bg_fill = ELEV_2;
    w.hovered.bg_stroke = Stroke::new(1.0_f32, BORDER_HI);
    w.hovered.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);
    w.hovered.corner_radius = CornerRadius::same(8);

    w.active.bg_fill = ACCENT;
    w.active.weak_bg_fill = ACCENT;
    w.active.bg_stroke = Stroke::new(1.0_f32, ACCENT);
    w.active.fg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(0x16, 0x0b, 0x00));
    w.active.corner_radius = CornerRadius::same(8);

    w.open.bg_fill = ELEV;
    w.open.weak_bg_fill = ELEV;
    w.open.bg_stroke = Stroke::new(1.0_f32, BORDER);
    w.open.fg_stroke = Stroke::new(1.0_f32, TEXT);
    w.open.corner_radius = CornerRadius::same(8);

    style.visuals = v;
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(10.0, 6.0);
    style.spacing.menu_margin = egui::Margin::same(8);
    style.spacing.indent = 16.0;
    style.text_styles = [
        (TextStyle::Heading, display(17.0)),
        (TextStyle::Body, body(13.5)),
        (TextStyle::Button, medium(13.0)),
        (TextStyle::Small, body(11.0)),
        (TextStyle::Monospace, mono(12.0)),
    ]
    .into();
}
