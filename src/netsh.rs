use std::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// Prevents a console window from flashing up every time netsh is run from the GUI.
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

fn netsh(args: &[&str], raw_arg: Option<&str>) -> Option<String> {
    let mut cmd = Command::new("netsh");
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

pub struct WifiState {
    /// False when wifi is switched off in Windows, or there's no wifi adapter.
    pub radio_on: bool,
    /// SSIDs the wifi adapter(s) are connected to.
    pub connected: Vec<String>,
}

pub fn wifi_state() -> WifiState {
    let out = netsh(&["wlan", "show", "interfaces"], None).unwrap_or_default();
    let has_interface = !values_of(&out, |key| key == "State").is_empty();
    // "Radio status : Hardware On" is followed by a line with just "Software Off" when
    // wifi is turned off in Windows.
    let radio_off = out.lines().any(|line| {
        let line = line.trim().to_lowercase();
        line.ends_with("software off") || line.ends_with("hardware off")
    });
    WifiState {
        radio_on: has_interface && !radio_off,
        connected: values_of(&out, |key| key == "SSID"),
    }
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
