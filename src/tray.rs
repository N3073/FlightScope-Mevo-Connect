//! System tray icon, and hiding/showing the window from any thread.

use std::sync::Arc;
use std::sync::atomic::{AtomicIsize, Ordering};

use eframe::egui;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

use crate::APP_NAME;
use crate::i18n::Strings;

/// The native window, shareable with background threads. egui stops running while the
/// window is hidden, so the window is hidden and shown directly through Windows instead.
#[derive(Clone)]
pub struct Window(Arc<AtomicIsize>);

impl Window {
    pub fn from_handle(handle: &impl HasWindowHandle) -> Self {
        let hwnd = match handle.window_handle().map(|h| h.as_raw()) {
            Ok(RawWindowHandle::Win32(h)) => h.hwnd.get(),
            _ => 0,
        };
        Self(Arc::new(AtomicIsize::new(hwnd)))
    }

    pub fn is_known(&self) -> bool {
        self.0.load(Ordering::Relaxed) != 0
    }

    pub fn hide(&self) {
        #[cfg(windows)]
        if let Some(hwnd) = self.hwnd() {
            use windows_sys::Win32::UI::WindowsAndMessaging::{SW_HIDE, ShowWindow};
            unsafe { ShowWindow(hwnd, SW_HIDE) };
        }
    }

    /// Shows the window, restores it if minimised, and brings it to the front.
    pub fn show(&self) {
        #[cfg(windows)]
        if let Some(hwnd) = self.hwnd() {
            use windows_sys::Win32::UI::WindowsAndMessaging::{
                IsIconic, SW_RESTORE, SW_SHOW, SetForegroundWindow, ShowWindow,
            };
            unsafe {
                ShowWindow(hwnd, if IsIconic(hwnd) != 0 { SW_RESTORE } else { SW_SHOW });
                SetForegroundWindow(hwnd);
            }
        }
    }

    #[cfg(windows)]
    fn hwnd(&self) -> Option<windows_sys::Win32::Foundation::HWND> {
        match self.0.load(Ordering::Relaxed) {
            0 => None,
            hwnd => Some(hwnd as windows_sys::Win32::Foundation::HWND),
        }
    }
}

pub struct Tray {
    _icon: TrayIcon,
    show: MenuItem,
    exit: MenuItem,
}

impl Tray {
    /// Adds the tray icon. Left-clicking it shows the window; its menu has Show and Exit.
    pub fn new(window: Window, ctx: egui::Context, t: &Strings) -> Result<Self, String> {
        let show = MenuItem::new(t.tray_show, true, None);
        let exit = MenuItem::new(t.tray_exit, true, None);
        let menu = Menu::new();
        menu.append(&show).map_err(|e| e.to_string())?;
        menu.append(&exit).map_err(|e| e.to_string())?;

        let png = eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon.png"))
            .map_err(|e| e.to_string())?;
        let icon = tray_icon::Icon::from_rgba(png.rgba, png.width, png.height).map_err(|e| e.to_string())?;
        let tray = TrayIconBuilder::new()
            .with_icon(icon)
            .with_tooltip(APP_NAME)
            .with_menu(Box::new(menu))
            .with_menu_on_left_click(false)
            .build()
            .map_err(|e| e.to_string())?;

        let (click_window, click_ctx) = (window.clone(), ctx.clone());
        TrayIconEvent::set_event_handler(Some(move |event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                click_window.show();
                click_ctx.request_repaint();
            }
        }));
        let (show_id, exit_id) = (show.id().clone(), exit.id().clone());
        MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
            window.show();
            if event.id == exit_id {
                // The window has to be visible for egui to process the close request.
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            } else if event.id != show_id {
                return;
            }
            ctx.request_repaint();
        }));

        Ok(Self { _icon: tray, show, exit })
    }

    /// Updates the menu after the language is changed.
    pub fn set_language(&self, t: &Strings) {
        self.show.set_text(t.tray_show);
        self.exit.set_text(t.tray_exit);
    }
}
