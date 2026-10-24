// Hide the console window in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Instant;
use std::{env, thread, time::Duration};

use eframe::egui::{self, Color32, RichText};

use i18n::{Language, Strings};
use theme::{AMBER, GREEN, RED};
use tray::{Tray, Window};
use update::Release;

mod i18n;
mod netsh;
mod theme;
mod tray;
mod update;

const APP_NAME: &str = "FlightScope Mevo Connect";
const LANGUAGE_KEY: &str = "language";
const AUTO_UPDATE_KEY: &str = "auto_update";
const RETRY_DELAY: Duration = Duration::from_millis(2_000);
/// How long "Connected" is shown before the window hides to the system tray.
const HIDE_DELAY: Duration = Duration::from_secs(2);

/// FlightScope Mevo networks are named like "FS M2-XXXXXX".
fn looks_like_mevo(ssid: &str) -> bool {
    let lower = ssid.to_lowercase();
    lower.starts_with("fs") || lower.contains("mevo")
}

/// The Mevo network name. The installer stores it as the user's FS_name environment variable.
/// The user and system variables are read from the registry first because a program started by
/// the installer (or from an already-open terminal) still has the environment from before the
/// variable was set.
fn configured_ssid() -> Option<String> {
    #[cfg(windows)]
    let from_registry = {
        use winreg::RegKey;
        use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
        let read = |hive, path| {
            RegKey::predef(hive)
                .open_subkey(path)
                .and_then(|key| key.get_value::<String, _>("FS_name"))
                .ok()
        };
        read(HKEY_CURRENT_USER, "Environment").or_else(|| {
            read(HKEY_LOCAL_MACHINE, r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment")
        })
    };
    #[cfg(not(windows))]
    let from_registry = None;
    from_registry
        .or_else(|| env::var("FS_name").ok())
        .map(|ssid| ssid.trim().to_string())
        .filter(|ssid| !ssid.is_empty())
}

/// A small grey uppercase heading above a section.
fn section_label(ui: &mut egui::Ui, text: &str) {
    ui.label(RichText::new(text.to_uppercase()).family(theme::bold()).size(12.5).color(theme::GREY));
}

/// A rounded, filled label such as the "Mevo network" confirmation.
fn badge(ui: &mut egui::Ui, text: &str, color: Color32) -> egui::Response {
    egui::Frame::new()
        .fill(color)
        .corner_radius(egui::CornerRadius::same(12))
        .inner_margin(egui::Margin::symmetric(10, 3))
        .show(ui, |ui| {
            ui.label(RichText::new(text).family(theme::bold()).size(13.5).color(Color32::WHITE))
        })
        .response
}

fn load_icon(ctx: &egui::Context) -> egui::TextureHandle {
    let png = eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon.png"))
        .expect("bundled icon is a valid PNG");
    let image = egui::ColorImage::from_rgba_unmultiplied([png.width as usize, png.height as usize], &png.rgba);
    let options = egui::TextureOptions {
        mipmap_mode: Some(egui::TextureFilter::Linear),
        ..egui::TextureOptions::LINEAR
    };
    ctx.load_texture("app-icon", image, options)
}

enum Msg {
    Attempt { attempt: u32, output: String },
    Connected,
    Cancelled,
    Update(UpdateState),
}

enum Status {
    Idle,
    Connecting { attempt: u32, output: String },
    Connected,
}

enum UpdateState {
    Idle,
    Checking,
    UpToDate,
    Available(Release),
    Downloading(Release),
    /// The installer is running; the program closes so it can be replaced.
    Installing,
    Failed(String),
}

struct App {
    /// The network from the FS_name environment variable, if set.
    ssid: Option<String>,
    has_profile: bool,
    status: Status,
    language: Language,
    auto_update: bool,
    update: UpdateState,
    /// Connecting waits for the startup update check when updates are automatic.
    connect_after_update_check: bool,
    show_settings: bool,
    window: Window,
    /// None if the tray icon couldn't be created; the window then never hides.
    tray: Option<Tray>,
    tray_language: Language,
    /// The app icon, shown in the title bar.
    icon: egui::TextureHandle,
    /// Cursor and window positions (screen pixels) when a title bar drag began.
    drag_origin: Option<((i32, i32), (i32, i32))>,
    /// When to hide the window to the tray after connecting.
    hide_at: Option<Instant>,
    cancel: Arc<AtomicBool>,
    tx: Sender<Msg>,
    rx: Receiver<Msg>,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::apply(&cc.egui_ctx);
        let ssid = configured_ssid();
        let has_profile = match &ssid {
            Some(ssid) => netsh::get_saved_profiles().contains(ssid),
            None => false,
        };
        let language = cc
            .storage
            .and_then(|storage| eframe::get_value::<String>(storage, LANGUAGE_KEY))
            .and_then(|code| Language::from_code(&code))
            .unwrap_or_else(Language::from_system);
        let auto_update = cc
            .storage
            .and_then(|storage| eframe::get_value::<bool>(storage, AUTO_UPDATE_KEY))
            .unwrap_or(false);
        let window = Window::from_handle(cc);
        let tray = match window.is_known() {
            true => Tray::new(window.clone(), cc.egui_ctx.clone(), language.strings())
                .map_err(|e| eprintln!("No tray icon: {e}"))
                .ok(),
            false => None,
        };
        let (tx, rx) = channel();
        let mut app = Self {
            ssid,
            has_profile,
            status: Status::Idle,
            language,
            auto_update,
            update: UpdateState::Idle,
            connect_after_update_check: false,
            show_settings: false,
            window,
            tray,
            tray_language: language,
            icon: load_icon(&cc.egui_ctx),
            drag_origin: None,
            hide_at: None,
            cancel: Arc::new(AtomicBool::new(false)),
            tx,
            rx,
        };
        app.check_for_updates(&cc.egui_ctx);
        // Start connecting right away; the program is meant to run on boot. With automatic
        // updates, install any update first so the download isn't cut off by joining the
        // Mevo's wifi, which has no internet.
        if app.auto_update && update::is_installed() {
            app.connect_after_update_check = true;
        } else {
            app.connect(&cc.egui_ctx);
        }
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
        let window = self.window.clone();
        self.status = Status::Connecting { attempt: 0, output: String::new() };
        // Keeps watching after connecting, and reconnects if the connection drops
        // (e.g. the Mevo is switched off and on again). The window comes back from the
        // tray when that happens; it's shown from here because egui is paused while hidden.
        thread::spawn(move || {
            let mut attempt = 0;
            let mut connected = false;
            loop {
                if cancel.load(Ordering::Relaxed) {
                    let _ = tx.send(Msg::Cancelled);
                    break;
                }
                if netsh::get_connected_networks().contains(&ssid) {
                    if !connected {
                        connected = true;
                        attempt = 0;
                        let _ = tx.send(Msg::Connected);
                        ctx.request_repaint();
                    }
                } else {
                    if connected {
                        connected = false;
                        window.show();
                    }
                    attempt += 1;
                    let output = netsh::connect(&ssid);
                    let _ = tx.send(Msg::Attempt { attempt, output });
                    ctx.request_repaint();
                }
                thread::sleep(RETRY_DELAY);
            }
            ctx.request_repaint();
        });
    }

    fn check_for_updates(&mut self, ctx: &egui::Context) {
        self.update = UpdateState::Checking;
        let tx = self.tx.clone();
        let ctx = ctx.clone();
        thread::spawn(move || {
            let state = match update::check() {
                Ok(Some(release)) => UpdateState::Available(release),
                Ok(None) => UpdateState::UpToDate,
                Err(e) => UpdateState::Failed(e),
            };
            let _ = tx.send(Msg::Update(state));
            ctx.request_repaint();
        });
    }

    fn install_update(&mut self, release: Release, ctx: &egui::Context) {
        self.update = UpdateState::Downloading(release.clone());
        let tx = self.tx.clone();
        let ctx = ctx.clone();
        thread::spawn(move || {
            let state = match update::download(&release).and_then(|path| update::install(&path)) {
                Ok(()) => UpdateState::Installing,
                Err(e) => UpdateState::Failed(e),
            };
            let _ = tx.send(Msg::Update(state));
            ctx.request_repaint();
        });
    }

    fn handle_messages(&mut self, ctx: &egui::Context) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                Msg::Attempt { attempt, output } => {
                    // Ignore a late attempt after Cancel was pressed.
                    if !matches!(self.status, Status::Idle) {
                        self.status = Status::Connecting { attempt, output };
                        self.hide_at = None;
                    }
                }
                Msg::Connected => {
                    self.status = Status::Connected;
                    if self.tray.is_some() {
                        self.hide_at = Some(Instant::now() + HIDE_DELAY);
                    }
                }
                Msg::Cancelled => self.status = Status::Idle,
                Msg::Update(state) => {
                    self.update = state;
                    if let UpdateState::Available(release) = &self.update {
                        if self.connect_after_update_check {
                            self.install_update(release.clone(), ctx);
                        }
                    }
                }
            }
        }
        let update_done = matches!(self.update, UpdateState::UpToDate | UpdateState::Failed(_));
        if self.connect_after_update_check && update_done {
            self.connect_after_update_check = false;
            self.connect(ctx);
        }
    }

    fn update_busy(&self) -> bool {
        matches!(
            self.update,
            UpdateState::Checking | UpdateState::Downloading(_) | UpdateState::Installing
        )
    }

    /// The custom title bar: icon, title, settings, minimise and close. Dragging anywhere on it
    /// that isn't a button moves the window.
    fn title_bar(&mut self, ui: &mut egui::Ui) {
        let t = self.language.strings();
        let bar = egui::Rect::from_min_size(ui.max_rect().min, egui::vec2(ui.max_rect().width(), theme::TITLE_BAR_HEIGHT));
        let drag = ui.interact(bar, ui.scope_id().with("title_bar"), egui::Sense::click_and_drag());
        // Move the window with the cursor while the button is held on the bar. This tracks the
        // cursor in screen coordinates itself rather than using Windows' move mode
        // (ViewportCommand::StartDrag), which didn't respond reliably.
        if drag.is_pointer_button_down_on() {
            if self.drag_origin.is_none() {
                self.drag_origin = self.window.cursor_and_position();
            }
            if let (Some((cursor0, window0)), Some((cursor, _))) = (self.drag_origin, self.window.cursor_and_position()) {
                self.window.move_to(window0.0 + cursor.0 - cursor0.0, window0.1 + cursor.1 - cursor0.1);
            }
        } else {
            self.drag_origin = None;
        }
        ui.horizontal(|ui| {
            ui.set_height(theme::TITLE_BAR_HEIGHT);
            ui.add_space(12.0);
            ui.add(egui::Image::new(&self.icon).fit_to_exact_size(egui::vec2(22.0, 22.0)));
            ui.label(RichText::new(APP_NAME).family(theme::bold()).size(18.0).color(Color32::WHITE));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                // Goes through the same close handling as Alt+F4: hides to the tray.
                if theme::title_button(ui, theme::TitleButton::Close).clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
                if theme::title_button(ui, theme::TitleButton::Minimize).clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                }
                ui.add_space(10.0);
                if theme::gear_button(ui, self.show_settings).on_hover_text(t.settings).clicked() {
                    self.show_settings = !self.show_settings;
                }
            });
        });
    }

    /// Shown above the connection view while an update is available or being installed.
    fn update_banner(&mut self, ui: &mut egui::Ui, t: &Strings) {
        match &self.update {
            UpdateState::Available(release) => {
                let release = release.clone();
                theme::card().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        let version = release.version.to_string();
                        ui.label(RichText::new(t.update_available.replace("{v}", &version)).family(theme::bold()));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            self.update_button(ui, t, release);
                        });
                    });
                });
                ui.add_space(8.0);
            }
            UpdateState::Downloading(_) | UpdateState::Installing => {
                self.update_progress(ui, t);
                ui.add_space(8.0);
            }
            UpdateState::Checking if self.connect_after_update_check => {
                self.update_progress(ui, t);
                ui.add_space(8.0);
            }
            _ => {}
        }
    }

    fn update_progress(&self, ui: &mut egui::Ui, t: &Strings) {
        let text = match &self.update {
            UpdateState::Checking => t.checking.to_string(),
            UpdateState::Downloading(release) => t.downloading.replace("{v}", &release.version.to_string()),
            UpdateState::Installing => t.installing.to_string(),
            _ => return,
        };
        ui.horizontal(|ui| {
            ui.add(egui::Spinner::new());
            ui.label(text);
        });
    }

    /// Installs the update, or opens the download page when not running from an installation.
    fn update_button(&mut self, ui: &mut egui::Ui, t: &Strings, release: Release) {
        if update::is_installed() {
            if ui.add(theme::primary_button(t.install_update).small()).clicked() {
                self.install_update(release, ui.ctx());
            }
        } else if ui.button(t.download).clicked() {
            ui.ctx().open_url(egui::OpenUrl::new_tab(update::RELEASES_PAGE));
        }
    }

    fn settings_view(&mut self, ui: &mut egui::Ui) {
        let t = self.language.strings();
        ui.label(RichText::new(t.settings).family(theme::bold()).size(18.0));
        ui.add_space(6.0);
        section_label(ui, t.language);
        egui::Grid::new("settings").num_columns(2).spacing([16.0, 8.0]).show(ui, |ui| {
            egui::ComboBox::from_id_salt("language")
                .width(200.0)
                .selected_text(self.language.native_name())
                .show_ui(ui, |ui| {
                    for language in Language::ALL {
                        ui.selectable_value(&mut self.language, language, language.native_name());
                    }
                });
            ui.end_row();
        });
        let t = self.language.strings();

        ui.add_space(12.0);
        section_label(ui, t.updates);
        ui.checkbox(&mut self.auto_update, t.auto_update);
        if ui.add_enabled(!self.update_busy(), egui::Button::new(t.check_now)).clicked() {
            self.check_for_updates(ui.ctx());
        }
        match &self.update {
            UpdateState::UpToDate => {
                ui.colored_label(GREEN, format!("✓ {}", t.up_to_date));
            }
            UpdateState::Available(release) => {
                let release = release.clone();
                ui.horizontal(|ui| {
                    ui.label(t.update_available.replace("{v}", &release.version.to_string()));
                    self.update_button(ui, t, release);
                });
            }
            UpdateState::Failed(e) => {
                ui.colored_label(RED, t.update_failed.replace("{e}", e));
            }
            _ => self.update_progress(ui, t),
        }

        ui.add_space(12.0);
        if ui.button(format!("← {}", t.back)).clicked() {
            self.show_settings = false;
        }
    }

    fn connection_view(&mut self, ui: &mut egui::Ui) {
        let t = self.language.strings();
        match self.ssid.clone() {
            Some(ssid) => {
                self.update_banner(ui, t);
                self.monitor_panel(ui, t, &ssid);
                ui.add_space(10.0);
                self.status_panel(ui, t);
                ui.add_space(10.0);
                self.connect_button(ui, t);
            }
            None => {
                ui.colored_label(RED, RichText::new(t.fs_name_missing).family(theme::bold()).size(17.0));
                ui.label(t.fs_name_missing_hint);
            }
        }
    }

    fn monitor_panel(&self, ui: &mut egui::Ui, t: &Strings, ssid: &str) {
        section_label(ui, t.flight_monitor);
        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(RichText::new(ssid).family(theme::bold()).size(18.0));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if looks_like_mevo(ssid) {
                        badge(ui, t.mevo_network, GREEN).on_hover_text(t.mevo_network_hover);
                    } else {
                        badge(ui, t.not_mevo, AMBER).on_hover_text(t.not_mevo_hover);
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
            Status::Idle => (theme::MUTED, t.not_connected.to_string()),
            Status::Connecting { attempt, .. } => (AMBER, t.connecting.replace("{n}", &attempt.to_string())),
            Status::Connected => (GREEN, t.connected.to_string()),
        };
        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                if let Status::Connecting { .. } = self.status {
                    ui.add(egui::Spinner::new().color(color));
                } else {
                    let (rect, _) = ui.allocate_exact_size(egui::vec2(14.0, 14.0), egui::Sense::hover());
                    ui.painter().circle_filled(rect.center(), 6.0, color);
                }
                ui.label(RichText::new(text).family(theme::bold()).size(16.0).color(color));
            });
            match &self.status {
                Status::Connecting { output, .. } => {
                    if !output.is_empty() {
                        ui.weak(output);
                    }
                    ui.weak(t.retry_hint);
                }
                Status::Connected => {
                    ui.weak(t.watching);
                }
                Status::Idle => {}
            }
        });
    }

    fn connect_button(&mut self, ui: &mut egui::Ui, t: &Strings) {
        let size = egui::vec2(ui.available_width(), 40.0);
        match self.status {
            Status::Connecting { .. } => {
                if ui.add_sized(size, theme::secondary_button(t.cancel)).clicked() {
                    self.cancel.store(true, Ordering::Relaxed);
                }
            }
            Status::Connected => {
                let button = theme::primary_button(&format!("✓ {}", t.connected)).fill(GREEN).stroke(egui::Stroke::NONE);
                ui.add_sized(size, button.sense(egui::Sense::hover()));
            }
            Status::Idle => {
                let button = theme::primary_button(t.connect);
                let enabled = !self.connect_after_update_check;
                if ui.add_enabled(enabled, button.min_size(size)).clicked() {
                    self.connect(ui.ctx());
                }
            }
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.handle_messages(&ctx);
        if let UpdateState::Installing = self.update {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        // The close button hides to the tray; Exit in the tray menu (or installing an update)
        // really closes. Minimising is left to Windows, so it goes to the taskbar as usual.
        if ctx.input(|i| i.viewport().close_requested()) {
            let installing = matches!(self.update, UpdateState::Installing);
            if let Some(tray) = &self.tray {
                if !tray.exit_requested() && !installing {
                    ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                    self.hide_at = None;
                    self.window.hide();
                }
            }
        }
        if let Some(at) = self.hide_at {
            let now = Instant::now();
            if now >= at {
                self.hide_at = None;
                self.window.hide();
            } else {
                ctx.request_repaint_after(at - now);
            }
        }
        if let Some(tray) = &self.tray {
            if self.tray_language != self.language {
                tray.set_language(self.language.strings());
                self.tray_language = self.language;
            }
        }
        egui::Panel::top("title_bar").frame(theme::title_bar()).show(ui, |ui| self.title_bar(ui));
        egui::Panel::bottom("footer").frame(theme::bar()).show(ui, |ui| {
            let version = self.language.strings().version.replace("{v}", env!("CARGO_PKG_VERSION"));
            ui.label(RichText::new(version).small().color(Color32::from_gray(0xcc)));
        });
        let body = egui::Frame::new().fill(Color32::WHITE).inner_margin(egui::Margin::same(16));
        egui::CentralPanel::default().frame(body).show(ui, |ui| {
            egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                if self.show_settings {
                    self.settings_view(ui);
                } else {
                    self.connection_view(ui);
                }
            });
        });
        // Without the Windows frame, a thin border keeps the window distinct from what's behind it.
        let border = egui::LayerId::new(egui::Order::Foreground, egui::Id::unique("window_border"));
        ctx.layer_painter(border).rect_stroke(
            ctx.content_rect(),
            0.0,
            egui::Stroke::new(1.0, theme::GREY),
            egui::StrokeKind::Inside,
        );
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, LANGUAGE_KEY, &self.language.code().to_string());
        eframe::set_value(storage, AUTO_UPDATE_KEY, &self.auto_update);
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(APP_NAME)
            // The app draws its own title bar (see App::title_bar).
            .with_decorations(false)
            .with_resizable(false)
            .with_icon(
                eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon.png"))
                    .expect("bundled icon is a valid PNG"),
            )
            .with_inner_size([460.0, 470.0]),
        // Always open at the default size rather than wherever it was last left.
        persist_window: false,
        ..Default::default()
    };
    eframe::run_native(APP_NAME, options, Box::new(|cc| Ok(Box::new(App::new(cc)))))
}
