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

/// SSIDs the wifi adapter(s) are currently connected to.
pub fn get_connected_networks() -> Vec<String> {
    netsh(&["wlan", "show", "interfaces"], None)
        .map(|out| values_of(&out, |key| key == "SSID"))
        .unwrap_or_default()
}

/// Names of the wifi profiles saved in Windows. `netsh wlan connect` only works for these.
pub fn get_saved_profiles() -> Vec<String> {
    netsh(&["wlan", "show", "profiles"], None)
        .map(|out| values_of(&out, |key| key.ends_with("User Profile")))
        .unwrap_or_default()
}

/// Asks Windows to connect to the saved profile `name`. Returns netsh's message.
pub fn connect(name: &str) -> String {
    let arg = format!("name=\"{name}\"");
    netsh(&["wlan", "connect"], Some(&arg))
        .map(|out| out.trim().to_string())
        .unwrap_or_else(|| String::from("Failed to run netsh."))
}
