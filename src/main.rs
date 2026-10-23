// Hide the console window in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Instant;
use std::{env, thread, time::Duration};

use eframe::egui::{self, Color32, RichText};

use i18n::{Language, Strings};

mod i18n;
mod netsh;

const APP_NAME: &str = "FlightScope Mevo Connect";
const LANGUAGE_KEY: &str = "language";
const RETRY_DELAY: Duration = Duration::from_millis(2_000);
/// How long the "connected" confirmation stays on screen before the program closes.
const CLOSE_DELAY: Duration = Duration::from_secs(3);

const GREEN: Color32 = Color32::from_rgb(60, 190, 90);
const AMBER: Color32 = Color32::from_rgb(230, 170, 40);
const RED: Color32 = Color32::from_rgb(220, 70, 60);

/// FlightScope Mevo networks are named like "FS M2-XXXXXX".
fn looks_like_mevo(ssid: &str) -> bool {
    let lower = ssid.to_lowercase();
    lower.starts_with("fs") || lower.contains("mevo")
}

enum Msg {
    Attempt { attempt: u32, output: String },
    Connected,
    Cancelled,
}

enum Status {
    Idle,
    Connecting { attempt: u32, output: String },
    Connected { at: Instant },
}

struct App {
    /// The network from the FS_name environment variable, if set.
    ssid: Option<String>,
    has_profile: bool,
    status: Status,
    language: Language,
    show_settings: bool,
    cancel: Arc<AtomicBool>,
    tx: Sender<Msg>,
    rx: Receiver<Msg>,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let ssid = env::var("FS_name").ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
        let has_profile = match &ssid {
            Some(ssid) => netsh::get_saved_profiles().contains(ssid),
            None => false,
        };
        let language = cc
            .storage
            .and_then(|storage| eframe::get_value::<String>(storage, LANGUAGE_KEY))
            .and_then(|code| Language::from_code(&code))
            .unwrap_or_else(Language::from_system);
        let (tx, rx) = channel();
        let mut app = Self {
            ssid,
            has_profile,
            status: Status::Idle,
            language,
            show_settings: false,
            cancel: Arc::new(AtomicBool::new(false)),
            tx,
            rx,
        };
        // Start connecting right away; the program is meant to run on boot.
        app.connect(&cc.egui_ctx);
        app
    }

    fn connect(&mut self, ctx: &egui::Context) {
        let Some(ssid) = self.ssid.clone() else {
            return;
        };
        let tx = self.tx.clone();
        let ctx = ctx.clone();
        self.cancel = Arc::new(AtomicBool::new(false));
        let cancel = self.cancel.clone();
        self.status = Status::Connecting { attempt: 0, output: String::new() };
        thread::spawn(move || {
            let mut attempt = 0;
            loop {
                if netsh::get_connected_networks().contains(&ssid) {
                    let _ = tx.send(Msg::Connected);
                    break;
                }
                if cancel.load(Ordering::Relaxed) {
                    let _ = tx.send(Msg::Cancelled);
                    break;
                }
                attempt += 1;
                let output = netsh::connect(&ssid);
                let _ = tx.send(Msg::Attempt { attempt, output });
                ctx.request_repaint();
                thread::sleep(RETRY_DELAY);
            }
            ctx.request_repaint();
        });
    }

    fn handle_messages(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                Msg::Attempt { attempt, output } => {
                    if let Status::Connecting { .. } = self.status {
                        self.status = Status::Connecting { attempt, output };
                    }
                }
                Msg::Connected => self.status = Status::Connected { at: Instant::now() },
                Msg::Cancelled => self.status = Status::Idle,
            }
        }
    }

    fn header(&mut self, ui: &mut egui::Ui) {
        let t = self.language.strings();
        ui.horizontal(|ui| {
            ui.heading(APP_NAME);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let gear = ui.selectable_label(self.show_settings, RichText::new("⚙").size(18.0));
                if gear.on_hover_text(t.settings).clicked() {
                    self.show_settings = !self.show_settings;
                }
            });
        });
    }

    fn settings_view(&mut self, ui: &mut egui::Ui) {
        let t = self.language.strings();
        ui.label(RichText::new(t.settings).size(16.0).strong());
        ui.add_space(6.0);
        egui::Grid::new("settings").num_columns(2).spacing([16.0, 8.0]).show(ui, |ui| {
            ui.label(t.language);
            egui::ComboBox::from_id_salt("language")
                .selected_text(self.language.native_name())
                .show_ui(ui, |ui| {
                    for language in Language::ALL {
                        ui.selectable_value(&mut self.language, language, language.native_name());
                    }
                });
            ui.end_row();
        });
        ui.add_space(12.0);
        let t = self.language.strings();
        if ui.button(format!("⬅ {}", t.back)).clicked() {
            self.show_settings = false;
        }
    }

    fn connection_view(&mut self, ui: &mut egui::Ui) {
        let t = self.language.strings();
        match self.ssid.clone() {
            Some(ssid) => {
                self.monitor_panel(ui, t, &ssid);
                ui.add_space(10.0);
                self.status_panel(ui, t);
                ui.add_space(10.0);
                self.connect_button(ui, t);
            }
            None => {
                ui.colored_label(RED, RichText::new(t.fs_name_missing).size(16.0).strong());
                ui.label(t.fs_name_missing_hint);
            }
        }
    }

    fn monitor_panel(&self, ui: &mut egui::Ui, t: &Strings, ssid: &str) {
        ui.label(RichText::new(t.flight_monitor).strong());
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(RichText::new(ssid).size(17.0).monospace());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if looks_like_mevo(ssid) {
                        ui.label(RichText::new(t.mevo_network).color(GREEN).strong())
                            .on_hover_text(t.mevo_network_hover);
                    } else {
                        ui.label(RichText::new(t.not_mevo).color(AMBER).strong())
                            .on_hover_text(t.not_mevo_hover);
                    }
                });
            });
        });
        if !self.has_profile {
            ui.colored_label(AMBER, t.no_profile);
        }
    }

    fn status_panel(&self, ui: &mut egui::Ui, t: &Strings) {
        let (color, text) = match &self.status {
            Status::Idle => (Color32::GRAY, t.not_connected.to_string()),
            Status::Connecting { attempt, .. } => (AMBER, t.connecting.replace("{n}", &attempt.to_string())),
            Status::Connected { .. } => (GREEN, t.connected.to_string()),
        };
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                if let Status::Connecting { .. } = self.status {
                    ui.add(egui::Spinner::new().color(color));
                } else {
                    let (rect, _) = ui.allocate_exact_size(egui::vec2(14.0, 14.0), egui::Sense::hover());
                    ui.painter().circle_filled(rect.center(), 6.0, color);
                }
                ui.label(RichText::new(text).size(15.0));
            });
            match &self.status {
                Status::Connecting { output, .. } => {
                    if !output.is_empty() {
                        ui.weak(output);
                    }
                    ui.weak(t.retry_hint);
                }
                Status::Connected { .. } => {
                    ui.weak(t.closing);
                }
                Status::Idle => {}
            }
        });
    }

    fn connect_button(&mut self, ui: &mut egui::Ui, t: &Strings) {
        let size = egui::vec2(ui.available_width(), 40.0);
        match self.status {
            Status::Connecting { .. } => {
                let button = egui::Button::new(RichText::new(t.cancel).size(17.0));
                if ui.add_sized(size, button).clicked() {
                    self.cancel.store(true, Ordering::Relaxed);
                }
            }
            Status::Connected { .. } => {
                let button = egui::Button::new(RichText::new(format!("✔ {}", t.connected)).size(17.0));
                ui.add_enabled_ui(false, |ui| ui.add_sized(size, button));
            }
            Status::Idle => {
                let button = egui::Button::new(RichText::new(t.connect).size(17.0).strong())
                    .fill(Color32::from_rgb(30, 110, 200));
                if ui.add_sized(size, button).clicked() {
                    self.connect(ui.ctx());
                }
            }
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.handle_messages();
        if let Status::Connected { at } = self.status {
            let elapsed = at.elapsed();
            if elapsed >= CLOSE_DELAY {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            } else {
                ui.ctx().request_repaint_after(CLOSE_DELAY - elapsed);
            }
        }
        egui::CentralPanel::default().show(ui, |ui| {
            ui.add_space(4.0);
            self.header(ui);
            ui.add_space(10.0);
            if self.show_settings {
                self.settings_view(ui);
            } else {
                self.connection_view(ui);
            }
        });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, LANGUAGE_KEY, &self.language.code().to_string());
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(APP_NAME)
            .with_inner_size([420.0, 300.0])
            .with_min_inner_size([340.0, 250.0]),
        ..Default::default()
    };
    eframe::run_native(APP_NAME, options, Box::new(|cc| Ok(Box::new(App::new(cc)))))
}
