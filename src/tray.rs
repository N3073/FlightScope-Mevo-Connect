//! System tray icon, and hiding/showing the window from any thread.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};

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

    /// The window of an already running copy of the app, found by its title even while it's
    /// hidden in the tray. winit and the tray icon also make small hidden windows, some with
    /// the same title, so the largest is taken.
    pub fn find_existing() -> Option<Self> {
        #[cfg(windows)]
        {
            use windows_sys::Win32::Foundation::RECT;
            use windows_sys::Win32::UI::WindowsAndMessaging::{FindWindowExW, GetWindowRect};
            let title: Vec<u16> = APP_NAME.encode_utf16().chain(std::iter::once(0)).collect();
            let (mut best, mut best_area) = (0isize, 0i64);
            let mut hwnd = std::ptr::null_mut();
            loop {
                hwnd = unsafe { FindWindowExW(std::ptr::null_mut(), hwnd, std::ptr::null(), title.as_ptr()) };
                if hwnd.is_null() {
                    break;
                }
                let mut rect = RECT { left: 0, top: 0, right: 0, bottom: 0 };
                if unsafe { GetWindowRect(hwnd, &mut rect) } != 0 {
                    let area = (rect.right - rect.left) as i64 * (rect.bottom - rect.top) as i64;
                    if area > best_area {
                        (best, best_area) = (hwnd as isize, area);
                    }
                }
            }
            (best != 0).then(|| Self(Arc::new(AtomicIsize::new(best))))
        }
        #[cfg(not(windows))]
        None
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

    /// The cursor position and the window's top-left corner, in screen pixels.
    pub fn cursor_and_position(&self) -> Option<((i32, i32), (i32, i32))> {
        #[cfg(windows)]
        if let Some(hwnd) = self.hwnd() {
            use windows_sys::Win32::Foundation::{POINT, RECT};
            use windows_sys::Win32::UI::WindowsAndMessaging::{GetCursorPos, GetWindowRect};
            let mut cursor = POINT { x: 0, y: 0 };
            let mut rect = RECT { left: 0, top: 0, right: 0, bottom: 0 };
            if unsafe { GetCursorPos(&mut cursor) != 0 && GetWindowRect(hwnd, &mut rect) != 0 } {
                return Some(((cursor.x, cursor.y), (rect.left, rect.top)));
            }
        }
        None
    }

    /// Moves the window's top-left corner to the given screen position.
    pub fn move_to(&self, x: i32, y: i32) {
        #[cfg(windows)]
        if let Some(hwnd) = self.hwnd() {
            use windows_sys::Win32::UI::WindowsAndMessaging::{
                SWP_NOACTIVATE, SWP_NOSIZE, SWP_NOZORDER, SetWindowPos,
            };
            unsafe { SetWindowPos(hwnd, std::ptr::null_mut(), x, y, 0, 0, SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE) };
        }
        #[cfg(not(windows))]
        let _ = (x, y);
    }

    #[cfg(windows)]
    fn hwnd(&self) -> Option<windows_sys::Win32::Foundation::HWND> {
        match self.0.load(Ordering::Relaxed) {
            0 => None,
            hwnd => Some(hwnd as windows_sys::Win32::Foundation::HWND),
        }
    }
}

/// Makes sure only one copy of the app runs per signed-in user. Held for the app's lifetime;
/// Windows releases it when the app exits.
pub struct SingleInstance {
    #[cfg(windows)]
    _mutex: windows_sys::Win32::Foundation::HANDLE,
}

impl SingleInstance {
    /// None if another copy of the app is already running in this Windows session.
    pub fn acquire() -> Option<Self> {
        #[cfg(windows)]
        {
            use windows_sys::Win32::Foundation::{CloseHandle, ERROR_ALREADY_EXISTS, GetLastError};
            use windows_sys::Win32::System::Threading::CreateMutexW;
            // "Local\" scopes it to the current sign-in session. The GUID is the installer's AppId.
            let name: Vec<u16> = r"Local\FlightScope-Mevo-Connect-8F3C2A51-6B7E-4D2F-9A41-3E5C7B9D0F12"
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();
            let mutex = unsafe { CreateMutexW(std::ptr::null(), 0, name.as_ptr()) };
            if !mutex.is_null() && unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
                unsafe { CloseHandle(mutex) };
                return None;
            }
            // If the mutex couldn't be created at all, run anyway rather than not at all.
            Some(Self { _mutex: mutex })
        }
        #[cfg(not(windows))]
        Some(Self {})
    }
}

pub struct Tray {
    _icon: TrayIcon,
    /// Set when Exit is chosen, so the close request isn't turned into hiding.
    exit_requested: Arc<AtomicBool>,
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
        let exit_requested = Arc::new(AtomicBool::new(false));
        let exit_flag = exit_requested.clone();
        MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
            window.show();
            if event.id == exit_id {
                exit_flag.store(true, Ordering::Relaxed);
                // The window has to be visible for egui to process the close request.
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            } else if event.id != show_id {
                return;
            }
            ctx.request_repaint();
        }));

        Ok(Self { _icon: tray, exit_requested, show, exit })
    }

    pub fn exit_requested(&self) -> bool {
        self.exit_requested.load(Ordering::Relaxed)
    }

    /// Updates the menu after the language is changed.
    pub fn set_language(&self, t: &Strings) {
        self.show.set_text(t.tray_show);
        self.exit.set_text(t.tray_exit);
    }
}
