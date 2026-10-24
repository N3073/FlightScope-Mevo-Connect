use std::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// Prevents a console window from flashing up every time netsh is run from the GUI.
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

fn netsh(args: &[&str], raw_arg: Option<&str>) -> Option<String> {
    run("netsh", args, raw_arg)
}

fn run(program: &str, args: &[&str], raw_arg: Option<&str>) -> Option<String> {
    let mut cmd = Command::new(program);
    cmd.args(args);
    #[cfg(windows)]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
        if let Some(raw) = raw_arg {
            cmd.raw_arg(raw);
        }
    }
    #[cfg(not(windows))]
    let _ = raw_arg;
    let output = cmd.output().ok()?;
    Some(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Collects the values of all `key : value` lines whose key matches `key_matches`.
fn values_of(output: &str, key_matches: impl Fn(&str) -> bool) -> Vec<String> {
    output
        .lines()
        .filter_map(|line| line.split_once(':'))
        .filter(|(key, _)| key_matches(key.trim()))
        .map(|(_, value)| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect()
}

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

pub fn wifi_state() -> Wifi {
    let out = netsh(&["wlan", "show", "interfaces"], None).unwrap_or_default();
    parse_interfaces(&out).unwrap_or_else(adapter_state)
}

/// Reads `netsh wlan show interfaces`. None when it lists no interface, which happens both
/// when there's no wifi adapter and when the adapter is disabled.
fn parse_interfaces(out: &str) -> Option<Wifi> {
    if values_of(out, |key| key == "State").is_empty() {
        return None;
    }
    // "Radio status : Hardware On" is followed by a line with just "Software Off" when
    // wifi is turned off in Windows.
    let radio_off = out.lines().any(|line| {
        let line = line.trim().to_lowercase();
        line.ends_with("software off") || line.ends_with("hardware off")
    });
    Some(match radio_off {
        true => Wifi::RadioOff,
        false => Wifi::On { connected: values_of(out, |key| key == "SSID") },
    })
}

/// Tells a missing wifi adapter from a disabled one by asking Windows for its physical wifi
/// adapters (NdisPhysicalMedium 9 is native 802.11). Slower than netsh (about a second),
/// so it only runs when netsh shows no wifi interface.
fn adapter_state() -> Wifi {
    let query = "Get-NetAdapter -Physical | Where-Object NdisPhysicalMedium -eq 9 | ForEach-Object { [string]$_.AdminStatus }";
    let out = run("powershell", &["-NoProfile", "-NonInteractive", "-Command", query], None).unwrap_or_default();
    parse_admin_status(&out)
}

fn parse_admin_status(out: &str) -> Wifi {
    let statuses: Vec<&str> = out.lines().map(str::trim).filter(|line| !line.is_empty()).collect();
    if statuses.is_empty() {
        Wifi::NoAdapter
    } else if statuses.iter().all(|status| status.eq_ignore_ascii_case("down")) {
        Wifi::AdapterDisabled
    } else {
        // Enabled, but netsh doesn't see it (e.g. the WLAN service is stopped).
        Wifi::RadioOff
    }
}

/// SSIDs of the networks in range, from Windows' latest wifi scan.
pub fn get_visible_networks() -> Vec<String> {
    netsh(&["wlan", "show", "networks"], None)
        .map(|out| parse_visible_networks(&out))
        .unwrap_or_default()
}

fn parse_visible_networks(out: &str) -> Vec<String> {
    values_of(out, |key| key.starts_with("SSID "))
}

/// Names of the wifi profiles saved in Windows. `netsh wlan connect` only works for these.
pub fn get_saved_profiles() -> Vec<String> {
    netsh(&["wlan", "show", "profiles"], None)
        .map(|out| values_of(&out, |key| key.ends_with("User Profile")))
        .unwrap_or_default()
}

/// Asks Windows to connect to the saved profile `name`. Success shows up in `wifi_state`.
pub fn connect(name: &str) {
    let arg = format!("name=\"{name}\"");
    netsh(&["wlan", "connect"], Some(&arg));
}

/// Opens the Windows list of available wifi networks.
pub fn open_wifi_menu() {
    let _ = Command::new("explorer").arg("ms-availablenetworks:").spawn();
}

/// Opens Windows' Network Connections, where adapters are enabled and disabled.
pub fn open_network_connections() {
    let _ = Command::new("control").arg("ncpa.cpl").spawn();
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONNECTED: &str = "
There is 1 interface on the system:

    Name                   : WLAN
    Description            : Intel(R) Wi-Fi 6 AX200 160MHz
    State                  : connected
    SSID                   : FS M2-123456
    BSSID                  : 00:11:22:33:44:55
    Radio status           : Hardware On
                             Software On
";

    const RADIO_OFF: &str = "
There is 1 interface on the system:

    Name                   : WLAN
    State                  : disconnected
    Radio status           : Hardware On
                             Software Off
";

    #[test]
    fn connected_interface() {
        assert_eq!(parse_interfaces(CONNECTED), Some(Wifi::On { connected: vec!["FS M2-123456".into()] }));
    }

    #[test]
    fn radio_off() {
        assert_eq!(parse_interfaces(RADIO_OFF), Some(Wifi::RadioOff));
    }

    #[test]
    fn no_interface() {
        assert_eq!(parse_interfaces("There is no wireless interface on the system."), None);
    }

    /// Runs the real adapter query; needs a PC with an enabled wifi adapter.
    /// Run with `cargo test -- --ignored`.
    #[test]
    #[ignore]
    fn adapter_query_finds_enabled_adapter() {
        assert_eq!(adapter_state(), Wifi::RadioOff);
    }

    #[test]
    fn visible_networks() {
        let out = "
Interface name : WLAN
There are 3 networks currently visible.

SSID 1 :
    Network type            : Infrastructure

SSID 2 : MORDOR
    Network type            : Infrastructure

SSID 3 : FS M2-123456
    Network type            : Infrastructure
";
        // Hidden networks have an empty name and are left out.
        assert_eq!(parse_visible_networks(out), vec!["MORDOR", "FS M2-123456"]);
    }

    #[test]
    fn adapter_states() {
        assert_eq!(parse_admin_status(""), Wifi::NoAdapter);
        assert_eq!(parse_admin_status("Down\r\n"), Wifi::AdapterDisabled);
        assert_eq!(parse_admin_status("Up\r\n"), Wifi::RadioOff);
    }
}
