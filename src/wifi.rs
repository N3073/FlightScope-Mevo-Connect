//! Wifi state and control through the Windows WLAN API (wlanapi.dll).
//!
//! This used to run `netsh`, but starting a program every couple of seconds made Windows show
//! "netsh.exe - Application Error (0xc0000142)" popups when one was started while Windows
//! was shutting down or restarting. Calling the API directly starts nothing, and it doesn't
//! depend on the language Windows writes its messages in.

use std::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// Prevents a console window from flashing up when a command-line program is started.
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Debug, PartialEq)]
pub enum Wifi {
    /// The PC has no wifi adapter.
    NoAdapter,
    /// The wifi adapter is disabled in Windows network connections.
    AdapterDisabled,
    /// Wifi is switched off in Windows.
    RadioOff,
    /// Wifi is on; `connected` holds the SSIDs the adapter(s) are connected to.
    On { connected: Vec<String> },
}

/// Stops Windows from showing error popups for programs this app starts (they inherit the
/// setting), e.g. one that fails to start because Windows is shutting down. Call at startup.
pub fn suppress_error_popups() {
    #[cfg(windows)]
    {
        use windows_sys::Win32::System::Diagnostics::Debug::{
            SEM_FAILCRITICALERRORS, SEM_NOGPFAULTERRORBOX, SetErrorMode,
        };
        unsafe { SetErrorMode(SEM_FAILCRITICALERRORS | SEM_NOGPFAULTERRORBOX) };
    }
}

/// True while Windows is shutting down, restarting or signing the user out.
pub fn shutting_down() -> bool {
    #[cfg(windows)]
    {
        use windows_sys::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_SHUTTINGDOWN};
        unsafe { GetSystemMetrics(SM_SHUTTINGDOWN) != 0 }
    }
    #[cfg(not(windows))]
    false
}

/// Runs a program without a console window and returns what it printed. Skipped while
/// Windows is shutting down, when new programs can fail to start.
fn run(program: &str, args: &[&str]) -> Option<String> {
    if shutting_down() {
        return None;
    }
    let mut cmd = Command::new(program);
    cmd.args(args);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    let output = cmd.output().ok()?;
    Some(String::from_utf8_lossy(&output.stdout).into_owned())
}

pub fn wifi_state() -> Wifi {
    #[cfg(windows)]
    if let Some(client) = wlan::Client::open() {
        let interfaces = client.interfaces();
        if !interfaces.is_empty() {
            if !interfaces.iter().any(|interface| client.radio_on(interface)) {
                return Wifi::RadioOff;
            }
            let connected = interfaces.iter().filter_map(|interface| client.connected_ssid(interface)).collect();
            return Wifi::On { connected };
        }
    }
    adapter_state()
}

/// Tells a missing wifi adapter from a disabled one by asking Windows for its physical wifi
/// adapters (NdisPhysicalMedium 9 is native 802.11). The WLAN API can't, as it only sees
/// enabled adapters. This starts PowerShell and takes about a second, so it only runs when
/// the WLAN API finds no wifi adapter.
fn adapter_state() -> Wifi {
    let query = "Get-NetAdapter -Physical | Where-Object NdisPhysicalMedium -eq 9 | ForEach-Object { [string]$_.AdminStatus }";
    let out = run("powershell", &["-NoProfile", "-NonInteractive", "-Command", query]).unwrap_or_default();
    parse_admin_status(&out)
}

fn parse_admin_status(out: &str) -> Wifi {
    let statuses: Vec<&str> = out.lines().map(str::trim).filter(|line| !line.is_empty()).collect();
    if statuses.is_empty() {
        Wifi::NoAdapter
    } else if statuses.iter().all(|status| status.eq_ignore_ascii_case("down")) {
        Wifi::AdapterDisabled
    } else {
        // Enabled, but the WLAN API doesn't see it (e.g. the WLAN service is stopped).
        Wifi::RadioOff
    }
}

/// Names of the wifi profiles saved in Windows. Connecting only works for these.
pub fn get_saved_profiles() -> Vec<String> {
    #[cfg(windows)]
    if let Some(client) = wlan::Client::open() {
        return client.interfaces().iter().flat_map(|interface| client.profiles(interface)).collect();
    }
    Vec::new()
}

/// SSIDs of the networks in range, from Windows' latest wifi scan.
pub fn get_visible_networks() -> Vec<String> {
    #[cfg(windows)]
    if let Some(client) = wlan::Client::open() {
        return client.interfaces().iter().flat_map(|interface| client.visible(interface)).collect();
    }
    Vec::new()
}

/// Asks Windows to connect to the saved profile `name`. Success shows up in `wifi_state`.
pub fn connect(name: &str) {
    #[cfg(windows)]
    if let Some(client) = wlan::Client::open() {
        for interface in client.interfaces() {
            if client.profiles(&interface).iter().any(|profile| profile == name) {
                client.connect(&interface, name);
            }
        }
    }
    #[cfg(not(windows))]
    let _ = name;
}

/// Asks Windows to scan for wifi networks now, probing for `ssid` by name. The result shows
/// up in `get_visible_networks` once the scan finishes (within a few seconds). Returns how
/// many wifi adapters were asked, or a Windows error code.
pub fn request_scan(ssid: &str) -> Result<u32, u32> {
    #[cfg(windows)]
    {
        let probe = wlan::dot11_ssid(ssid).ok_or(windows_sys::Win32::Foundation::ERROR_INVALID_PARAMETER)?;
        let client = wlan::Client::open().ok_or(windows_sys::Win32::Foundation::ERROR_SERVICE_NOT_ACTIVE)?;
        let interfaces = client.interfaces();
        Ok(interfaces.iter().filter(|interface| client.scan(interface, &probe)).count() as u32)
    }
    #[cfg(not(windows))]
    {
        let _ = ssid;
        Ok(0)
    }
}

/// Opens the Windows list of available wifi networks.
pub fn open_wifi_menu() {
    let _ = Command::new("explorer").arg("ms-availablenetworks:").spawn();
}

/// Opens Windows' Network Connections, where adapters are enabled and disabled.
pub fn open_network_connections() {
    let _ = Command::new("control").arg("ncpa.cpl").spawn();
}

#[cfg(windows)]
mod wlan {
    use std::ffi::c_void;
    use std::ptr::{null, null_mut};

    use windows_sys::Win32::Foundation::HANDLE;
    use windows_sys::Win32::NetworkManagement::WiFi::*;
    use windows_sys::core::GUID;

    /// An open WLAN API session, closed when dropped.
    pub struct Client(HANDLE);

    pub struct Interface {
        guid: GUID,
        state: WLAN_INTERFACE_STATE,
    }

    impl Client {
        pub fn open() -> Option<Self> {
            let mut version = 0;
            let mut handle: HANDLE = null_mut();
            // Client version 2 is the API of Windows Vista and later.
            match unsafe { WlanOpenHandle(2, null(), &mut version, &mut handle) } {
                0 => Some(Self(handle)),
                _ => None,
            }
        }

        /// The wifi adapters Windows can use (enabled ones only).
        pub fn interfaces(&self) -> Vec<Interface> {
            let mut list: *mut WLAN_INTERFACE_INFO_LIST = null_mut();
            if unsafe { WlanEnumInterfaces(self.0, null(), &mut list) } != 0 {
                return Vec::new();
            }
            unsafe {
                let items = std::slice::from_raw_parts((*list).InterfaceInfo.as_ptr(), (*list).dwNumberOfItems as usize);
                let interfaces = items.iter().map(|info| Interface { guid: info.InterfaceGuid, state: info.isState }).collect();
                WlanFreeMemory(list as *const c_void);
                interfaces
            }
        }

        /// False when wifi is switched off in Windows (or by a hardware switch).
        pub fn radio_on(&self, interface: &Interface) -> bool {
            self.query::<WLAN_RADIO_STATE, _>(interface, wlan_intf_opcode_radio_state, |state| {
                let phys = &state.PhyRadioState[..(state.dwNumberOfPhys as usize).min(state.PhyRadioState.len())];
                phys.iter().any(|phy| {
                    phy.dot11SoftwareRadioState == dot11_radio_state_on && phy.dot11HardwareRadioState == dot11_radio_state_on
                })
            })
            // If Windows can't say, don't claim wifi is off.
            .unwrap_or(true)
        }

        /// The SSID the interface is connected to, if any.
        pub fn connected_ssid(&self, interface: &Interface) -> Option<String> {
            if interface.state != wlan_interface_state_connected {
                return None;
            }
            self.query::<WLAN_CONNECTION_ATTRIBUTES, _>(interface, wlan_intf_opcode_current_connection, |connection| {
                ssid_string(&connection.wlanAssociationAttributes.dot11Ssid)
            })
        }

        pub fn profiles(&self, interface: &Interface) -> Vec<String> {
            let mut list: *mut WLAN_PROFILE_INFO_LIST = null_mut();
            if unsafe { WlanGetProfileList(self.0, &interface.guid, null(), &mut list) } != 0 {
                return Vec::new();
            }
            unsafe {
                let items = std::slice::from_raw_parts((*list).ProfileInfo.as_ptr(), (*list).dwNumberOfItems as usize);
                let names = items.iter().map(|profile| wide_string(&profile.strProfileName)).collect();
                WlanFreeMemory(list as *const c_void);
                names
            }
        }

        pub fn visible(&self, interface: &Interface) -> Vec<String> {
            let mut list: *mut WLAN_AVAILABLE_NETWORK_LIST = null_mut();
            if unsafe { WlanGetAvailableNetworkList(self.0, &interface.guid, 0, null(), &mut list) } != 0 {
                return Vec::new();
            }
            unsafe {
                let items = std::slice::from_raw_parts((*list).Network.as_ptr(), (*list).dwNumberOfItems as usize);
                // Hidden networks have an empty name.
                let names = items
                    .iter()
                    .map(|network| ssid_string(&network.dot11Ssid))
                    .filter(|name| !name.is_empty())
                    .collect();
                WlanFreeMemory(list as *const c_void);
                names
            }
        }

        pub fn connect(&self, interface: &Interface, profile: &str) {
            let profile: Vec<u16> = profile.encode_utf16().chain(std::iter::once(0)).collect();
            let parameters = WLAN_CONNECTION_PARAMETERS {
                wlanConnectionMode: wlan_connection_mode_profile,
                strProfile: profile.as_ptr(),
                pDot11Ssid: null_mut(),
                pDesiredBssidList: null_mut(),
                dot11BssType: dot11_BSS_type_any,
                dwFlags: 0,
            };
            unsafe { WlanConnect(self.0, &interface.guid, &parameters, null()) };
        }

        /// Asks the interface to scan now, probing for `ssid`. True if Windows accepted.
        pub fn scan(&self, interface: &Interface, ssid: &DOT11_SSID) -> bool {
            unsafe { WlanScan(self.0, &interface.guid, ssid, null(), null()) == 0 }
        }

        /// Reads one WLAN_INTF_OPCODE value of type `T` and maps it with `read`.
        fn query<T, R>(&self, interface: &Interface, opcode: WLAN_INTF_OPCODE, read: impl FnOnce(&T) -> R) -> Option<R> {
            let mut size = 0;
            let mut data: *mut c_void = null_mut();
            let err = unsafe { WlanQueryInterface(self.0, &interface.guid, opcode, null(), &mut size, &mut data, null_mut()) };
            if err != 0 || data.is_null() {
                return None;
            }
            let value = (size as usize >= size_of::<T>()).then(|| read(unsafe { &*(data as *const T) }));
            unsafe { WlanFreeMemory(data) };
            value
        }
    }

    impl Drop for Client {
        fn drop(&mut self) {
            unsafe { WlanCloseHandle(self.0, null()) };
        }
    }

    /// The SSID in the API's form; None if it's empty or longer than 32 bytes.
    pub fn dot11_ssid(ssid: &str) -> Option<DOT11_SSID> {
        let bytes = ssid.as_bytes();
        if bytes.is_empty() || bytes.len() > 32 {
            return None;
        }
        let mut probe = DOT11_SSID { uSSIDLength: bytes.len() as u32, ucSSID: [0; 32] };
        probe.ucSSID[..bytes.len()].copy_from_slice(bytes);
        Some(probe)
    }

    fn ssid_string(ssid: &DOT11_SSID) -> String {
        let len = (ssid.uSSIDLength as usize).min(ssid.ucSSID.len());
        String::from_utf8_lossy(&ssid.ucSSID[..len]).into_owned()
    }

    fn wide_string(wide: &[u16]) -> String {
        let len = wide.iter().position(|&c| c == 0).unwrap_or(wide.len());
        String::from_utf16_lossy(&wide[..len])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adapter_states() {
        assert_eq!(parse_admin_status(""), Wifi::NoAdapter);
        assert_eq!(parse_admin_status("Down\r\n"), Wifi::AdapterDisabled);
        assert_eq!(parse_admin_status("Up\r\n"), Wifi::RadioOff);
    }

    #[test]
    fn scan_rejects_invalid_names() {
        assert!(request_scan("").is_err());
        assert!(request_scan(&"x".repeat(33)).is_err());
    }

    // The tests below use the real wifi adapter. Run with `cargo test -- --ignored --nocapture`
    // on a PC with wifi switched on; they print what they find for a manual check.

    #[test]
    #[ignore]
    fn reads_wifi_state() {
        let state = wifi_state();
        println!("wifi state: {state:?}");
        assert!(matches!(state, Wifi::On { .. } | Wifi::RadioOff));
    }

    #[test]
    #[ignore]
    fn reads_profiles_and_networks() {
        let profiles = get_saved_profiles();
        println!("saved profiles: {profiles:?}");
        println!("visible networks: {:?}", get_visible_networks());
        assert!(!profiles.is_empty());
    }

    #[test]
    #[ignore]
    fn scan_request_is_accepted() {
        assert!(matches!(request_scan("FS M2-123456"), Ok(n) if n >= 1));
    }

    #[test]
    #[ignore]
    fn adapter_query_finds_enabled_adapter() {
        assert_eq!(adapter_state(), Wifi::RadioOff);
    }
}
