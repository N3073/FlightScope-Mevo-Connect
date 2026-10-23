use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use std::{env, fs};

use semver::Version;
use serde::Deserialize;
use sha2::{Digest, Sha256};

pub const RELEASES_PAGE: &str = "https://github.com/N3073/FlightScope-Mevo-Connect/releases/latest";
const LATEST_RELEASE_API: &str = "https://api.github.com/repos/N3073/FlightScope-Mevo-Connect/releases/latest";
const USER_AGENT: &str = concat!("FlightScope-Mevo-Connect/", env!("CARGO_PKG_VERSION"));
/// Kept short so a PC without internet (e.g. already on the Mevo's wifi) doesn't wait long.
const CHECK_TIMEOUT: Duration = Duration::from_secs(8);
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(300);

#[derive(Deserialize)]
struct GithubRelease {
    tag_name: String,
    assets: Vec<GithubAsset>,
}

#[derive(Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
    /// "sha256:<hex>", provided by GitHub for release assets.
    digest: Option<String>,
}

#[derive(Clone)]
pub struct Release {
    pub version: Version,
    file_name: String,
    url: String,
    sha256: Option<String>,
}

pub fn current_version() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION")).expect("crate version is valid semver")
}

/// True when running from a folder made by the installer; only then can the app update itself.
pub fn is_installed() -> bool {
    env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join("unins000.exe").exists()))
        .unwrap_or(false)
}

fn agent(timeout: Duration) -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(timeout))
        .build()
        .into()
}

/// Returns the latest GitHub release if it is newer than this build.
pub fn check() -> Result<Option<Release>, String> {
    let response = agent(CHECK_TIMEOUT)
        .get(LATEST_RELEASE_API)
        .header("User-Agent", USER_AGENT)
        .header("Accept", "application/vnd.github+json")
        .call();
    let release: GithubRelease = match response {
        Ok(mut response) => response.body_mut().read_json().map_err(|e| e.to_string())?,
        // No releases published yet.
        Err(ureq::Error::StatusCode(404)) => return Ok(None),
        Err(e) => return Err(e.to_string()),
    };
    let version = Version::parse(release.tag_name.trim_start_matches('v'))
        .map_err(|e| format!("bad release tag {:?}: {e}", release.tag_name))?;
    if version <= current_version() {
        return Ok(None);
    }
    let asset = release
        .assets
        .into_iter()
        .find(|asset| asset.name.to_lowercase().ends_with("setup.exe"))
        .ok_or_else(|| format!("release {version} has no installer"))?;
    Ok(Some(Release {
        version,
        file_name: asset.name,
        url: asset.browser_download_url,
        sha256: asset
            .digest
            .and_then(|digest| digest.strip_prefix("sha256:").map(str::to_lowercase)),
    }))
}

/// Downloads the release's installer to the temp folder and verifies its checksum.
pub fn download(release: &Release) -> Result<PathBuf, String> {
    let mut response = agent(DOWNLOAD_TIMEOUT)
        .get(&release.url)
        .header("User-Agent", USER_AGENT)
        .call()
        .map_err(|e| e.to_string())?;
    let path = env::temp_dir().join(&release.file_name);
    let mut file = File::create(&path).map_err(|e| e.to_string())?;
    let mut reader = response.body_mut().as_reader();
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = reader.read(&mut buffer).map_err(|e| e.to_string())?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        file.write_all(&buffer[..read]).map_err(|e| e.to_string())?;
    }
    drop(file);
    if let Some(expected) = &release.sha256 {
        let actual: String = hasher.finalize().iter().map(|b| format!("{b:02x}")).collect();
        if &actual != expected {
            let _ = fs::remove_file(&path);
            return Err(String::from("downloaded installer failed checksum verification"));
        }
    }
    Ok(path)
}

/// Starts the installer silently. It replaces this program and restarts it, so the caller
/// should exit right after.
pub fn install(installer: &PathBuf) -> Result<(), String> {
    Command::new(installer)
        .args(["/VERYSILENT", "/SUPPRESSMSGBOXES", "/NORESTART", "/RELAUNCH"])
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}
