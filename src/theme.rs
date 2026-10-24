//! Look and feel based on FlightScope's website: white pages, charcoal header and footer,
//! blue buttons, red accents and the Nunito font.

use std::sync::Arc;

use eframe::egui::{
    self, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Stroke, TextStyle,
};

pub const BLUE: Color32 = Color32::from_rgb(0x07, 0x7c, 0xbf);
pub const BLUE_LIGHT: Color32 = Color32::from_rgb(0x09, 0x9c, 0xf0);
pub const BLUE_DIM: Color32 = Color32::from_rgb(0x06, 0x6c, 0xa6);
pub const RED: Color32 = Color32::from_rgb(0xd6, 0x00, 0x1c);
pub const GREEN: Color32 = Color32::from_rgb(0x2e, 0x9e, 0x5b);
pub const AMBER: Color32 = Color32::from_rgb(0xd9, 0x8a, 0x00);
pub const CHARCOAL: Color32 = Color32::from_rgb(0x4b, 0x4b, 0x4b);
pub const GREY: Color32 = Color32::from_rgb(0x58, 0x58, 0x58);
pub const MUTED: Color32 = Color32::from_rgb(0x8a, 0x8a, 0x8a);
pub const CARD: Color32 = Color32::from_rgb(0xf2, 0xf2, 0xf2);
pub const BORDER: Color32 = Color32::from_rgb(0xe8, 0xe8, 0xe1);
pub const TEXT: Color32 = Color32::from_rgb(0x11, 0x11, 0x11);

const RADIUS: u8 = 6;

/// Font family for headings and button labels.
pub fn bold() -> FontFamily {
    FontFamily::Name("bold".into())
}

pub fn apply(ctx: &egui::Context) {
    install_fonts(ctx);
    ctx.set_theme(egui::Theme::Light);
    ctx.style_mut_of(egui::Theme::Light, |style| {
        style.text_styles = [
            (TextStyle::Heading, FontId::new(20.0, bold())),
            (TextStyle::Body, FontId::new(15.0, FontFamily::Proportional)),
            (TextStyle::Button, FontId::new(15.0, FontFamily::Proportional)),
            (TextStyle::Small, FontId::new(12.5, FontFamily::Proportional)),
            (TextStyle::Monospace, FontId::new(14.0, FontFamily::Monospace)),
        ]
        .into();
        style.spacing.button_padding = egui::vec2(12.0, 6.0);
        // Text isn't selectable; otherwise the title text would catch drags meant for the title bar.
        style.interaction.selectable_labels = false;
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);

        let visuals = &mut style.visuals;
        visuals.override_text_color = Some(TEXT);
        visuals.panel_fill = Color32::WHITE;
        visuals.window_fill = Color32::WHITE;
        visuals.extreme_bg_color = Color32::WHITE;
        visuals.faint_bg_color = CARD;
        visuals.hyperlink_color = BLUE;
        visuals.selection.bg_fill = BLUE;
        visuals.selection.stroke = Stroke::new(1.0, Color32::WHITE);
        visuals.window_stroke = Stroke::new(1.0, BORDER);
        // A hand cursor over anything clickable.
        visuals.interact_cursor = Some(egui::CursorIcon::PointingHand);

        let radius = CornerRadius::same(RADIUS);
        let widgets = &mut visuals.widgets;
        widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER);
        widgets.noninteractive.corner_radius = radius;
        for (state, fill, stroke) in [
            (&mut widgets.inactive, Color32::WHITE, BORDER),
            (&mut widgets.hovered, CARD, BLUE_LIGHT),
            (&mut widgets.active, BORDER, BLUE),
            (&mut widgets.open, CARD, BLUE),
        ] {
            state.bg_fill = fill;
            state.weak_bg_fill = fill;
            state.bg_stroke = Stroke::new(1.0, stroke);
            state.fg_stroke = Stroke::new(1.5, TEXT);
            state.corner_radius = radius;
            state.expansion = 0.0;
        }
    });
}

fn install_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "Nunito".into(),
        Arc::new(FontData::from_static(include_bytes!("../assets/fonts/Nunito-400.ttf"))),
    );
    fonts.font_data.insert(
        "Nunito-Bold".into(),
        Arc::new(FontData::from_static(include_bytes!("../assets/fonts/Nunito-700.ttf"))),
    );
    // Keep egui's default fonts as fallbacks for symbols such as ✔ and ⚙.
    let fallbacks = fonts.families[&FontFamily::Proportional].clone();
    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, "Nunito".into());
    fonts
        .families
        .insert(bold(), std::iter::once("Nunito-Bold".to_string()).chain(fallbacks).collect());
    ctx.set_fonts(fonts);
}

/// A light grey rounded panel, like the product cards on FlightScope's site.
pub fn card() -> egui::Frame {
    egui::Frame::new()
        .fill(CARD)
        .corner_radius(CornerRadius::same(RADIUS + 2))
        .inner_margin(egui::Margin::same(12))
}

/// The charcoal header and footer bars.
pub fn bar() -> egui::Frame {
    egui::Frame::new()
        .fill(CHARCOAL)
        .inner_margin(egui::Margin::symmetric(14, 10))
}

pub const TITLE_BAR_HEIGHT: f32 = 44.0;

/// The charcoal title bar that replaces the Windows one.
pub fn title_bar() -> egui::Frame {
    egui::Frame::new().fill(CHARCOAL)
}

pub enum TitleButton {
    Minimize,
    Close,
}

/// A Windows-style caption button for the title bar, drawn by hand. Close turns red on hover.
pub fn title_button(ui: &mut egui::Ui, kind: TitleButton) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(46.0, TITLE_BAR_HEIGHT), egui::Sense::click());
    let painter = ui.painter();
    if response.hovered() {
        let fill = match kind {
            TitleButton::Close => RED,
            TitleButton::Minimize => GREY,
        };
        painter.rect_filled(rect, CornerRadius::ZERO, fill);
    }
    let stroke = Stroke::new(1.2, Color32::WHITE);
    let c = rect.center();
    let r = 5.5;
    match kind {
        TitleButton::Minimize => {
            painter.line_segment([c + egui::vec2(-r, 0.0), c + egui::vec2(r, 0.0)], stroke);
        }
        TitleButton::Close => {
            painter.line_segment([c + egui::vec2(-r, -r), c + egui::vec2(r, r)], stroke);
            painter.line_segment([c + egui::vec2(-r, r), c + egui::vec2(r, -r)], stroke);
        }
    }
    response
}

/// A settings cog for the header bar, drawn by hand so it doesn't depend on font glyphs.
pub fn gear_button(ui: &mut egui::Ui, selected: bool) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(26.0, 26.0), egui::Sense::click());
    let color = if selected || response.hovered() { BLUE_LIGHT } else { Color32::WHITE };
    let center = rect.center();
    let painter = ui.painter();
    for i in 0..8 {
        let angle = i as f32 * std::f32::consts::TAU / 8.0;
        let dir = egui::vec2(angle.cos(), angle.sin());
        painter.line_segment([center + dir * 6.0, center + dir * 10.5], Stroke::new(3.5, color));
    }
    painter.circle_filled(center, 7.5, color);
    painter.circle_filled(center, 3.0, CHARCOAL);
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

#[derive(Clone, Copy)]
pub enum ButtonKind {
    /// Filled blue, for the main action (Connect, Install).
    Primary,
    /// Outlined grey, for secondary actions (Cancel).
    Secondary,
    /// Filled green and not clickable, for showing a done state (Connected).
    Success,
}

/// A bold call-to-action button whose colours change on hover and press.
pub fn action_button(ui: &mut egui::Ui, text: &str, kind: ButtonKind, size: egui::Vec2, enabled: bool) -> egui::Response {
    // (fill, border) for normal, hovered and pressed, plus the text colour.
    let (states, text_color) = match kind {
        ButtonKind::Primary => ([(BLUE, BLUE_DIM), (BLUE_LIGHT, BLUE), (BLUE_DIM, BLUE_DIM)], Color32::WHITE),
        ButtonKind::Secondary => ([(Color32::WHITE, GREY), (CARD, CHARCOAL), (BORDER, CHARCOAL)], GREY),
        ButtonKind::Success => ([(GREEN, GREEN); 3], Color32::WHITE),
    };
    let font_size = if size.y < 36.0 { 14.0 } else { 17.0 };
    ui.scope(|ui| {
        let visuals = ui.visuals_mut();
        // The app-wide text colour would otherwise win over the per-state colours.
        visuals.override_text_color = None;
        let widgets = &mut visuals.widgets;
        for (state, (fill, border)) in [&mut widgets.inactive, &mut widgets.hovered, &mut widgets.active]
            .into_iter()
            .zip(states)
        {
            state.weak_bg_fill = fill;
            state.bg_fill = fill;
            state.bg_stroke = Stroke::new(1.5, border);
            state.fg_stroke = Stroke::new(1.5, text_color);
            state.corner_radius = CornerRadius::same(RADIUS);
        }
        let button = egui::Button::new(egui::RichText::new(text).family(bold()).size(font_size));
        let button = match kind {
            ButtonKind::Success => button.sense(egui::Sense::hover()),
            _ => button,
        };
        // add_sized keeps the label centred in a wide button.
        ui.add_enabled_ui(enabled, |ui| ui.add_sized(size, button)).inner
    })
    .inner
}
