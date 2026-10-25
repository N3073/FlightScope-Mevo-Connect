# FlightScope Mevo Connect

A small Windows app that connects the PC's wifi adapter to a FlightScope Mevo ball flight monitor. It is meant to run when you sign in to Windows, so the PC joins the Mevo's wifi network without anyone having to do it by hand.

## Installing

1. Download `FlightScope-Mevo-Connect-<version>-Setup.exe` from the [latest release](https://github.com/N3073/FlightScope-Mevo-Connect/releases/latest) and run it. Administrator rights aren't needed: the app is installed for the current user only, in `%LOCALAPPDATA%\Programs\FlightScope Mevo Connect`.
2. When asked, enter the name of the Mevo's wifi network, for example `FS M2-123456`. Networks already saved on the PC that look like a Mevo network are offered in the drop-down, and the first one is filled in for you. If the name doesn't look like a Mevo network, the installer asks you to confirm it.
3. Choose whether the app should **start automatically when you sign in to Windows**. You can also add a desktop shortcut.

The installer is available in the same languages as the app.

To change the network name or the startup option later, run the installer again; it remembers your previous answers. Uninstall the app from *Settings → Apps* in Windows. This also removes the startup shortcut and the saved network name.

> Windows SmartScreen may warn about the installer because it isn't code-signed. Choose *More info → Run anyway*.

## How it works

1. When the app starts, it reads the Mevo's network name from the `FS_name` environment variable, which the installer sets. This is the only network the app will ever connect to, and it can't be changed from within the app.
2. The window shows that network name with a badge:
   - **✓ Mevo network** (green): the name matches the Mevo naming pattern. Names count as a match if they start with `FS` or contain `Mevo`; case doesn't matter.
   - **⚠ Not a Mevo name** (amber): the name doesn't match. The app still connects to it, but check the name you entered.
3. The app starts connecting straight away. Every 2 seconds it checks whether the PC is already on the network. If it isn't, it asks Windows to connect again, and keeps retrying until it connects, so you can start it before the Mevo is switched on. It only retries when connecting can work. Otherwise it shows what to fix and carries on by itself once it's fixed:
   - **No wifi adapter**: the PC has no wifi adapter. Plug in a USB wifi adapter.
   - **Wifi adapter is disabled**: enable it in Windows network connections. There's an **Open network connections** button.
   - **Wifi is off**: turn wifi on in Windows.
   - **Mevo not found**: the Mevo's network isn't in range, usually because the Mevo is switched off. Switch it on and keep it near the PC. Meanwhile the app keeps trying, and asks Windows to scan for networks every 10 seconds so the Mevo shows up as soon as it's back.
   - **Network not saved in Windows**: join the network once from the Windows wifi menu, so that Windows saves its password.

   The last two have an **Open wifi menu** button.
4. Once connected, the app shows **Connected** for 2 seconds and then hides to the system tray (the notification area next to the clock). It keeps running and checks the connection every 2 seconds.
5. If the Mevo disconnects, for example because it's switched off, the window comes back to the front and the app starts reconnecting.

Click the tray icon to show the window at any time. Right-click it for **Show** and **Exit**. Only one copy of the app runs at a time: starting it again, for example from a shortcut, brings up the window of the copy that's already running.

The window's close button (✕) hides the app to the tray rather than quitting, so it keeps watching the connection. To quit, choose **Exit** from the tray menu. The minimise button works as usual and minimises the window to the taskbar.

**Cancel** stops the retries, and **Connect** starts them again.

## Settings

Click the **⚙** button in the top-right corner to open the settings view.

| Setting | Description |
| --- | --- |
| Language | The language of the app: English (UK), Suomi, Svenska, Deutsch, Italiano, Français, Español or Português. On first launch the app uses your Windows language settings if they match one of these, otherwise English. |
| Install updates automatically | Off by default. See [Updates](#updates). |
| Check for updates | Checks GitHub for a newer release now. |

Settings are saved when the app closes, to `%APPDATA%\FlightScope Mevo Connect\data\app.ron`.

## Updates

The app checks this repository's [GitHub releases](https://github.com/N3073/FlightScope-Mevo-Connect/releases) for a newer version.

- **Automatic updates off** (default): the app checks in the background when it starts. If a newer version exists, a banner offers to **Install** it. Nothing is installed unless you click Install.
- **Automatic updates on**: the app checks before connecting. If a newer version exists, it installs it straight away. The Mevo's wifi has no internet, so the app downloads updates before it joins that network.

When an update is installed, the app downloads the new installer and checks its SHA-256 hash against the one GitHub publishes for the file. It then runs the installer silently with your previous answers and closes. The installer restarts the app when it has finished.

Updates can only be installed into a copy that was installed with the installer. A copy you built yourself only offers a link to the download page.

If the PC has no internet connection, for example because it's already on the Mevo's network, the check fails quietly and the app carries on connecting.

## Requirements

- Windows 10 or 11 (64-bit) with a wifi adapter.
- A **saved Windows wifi profile** for the Mevo's network. Windows can only connect to networks it has joined before. Before using the app, join the Mevo's network once from the Windows wifi menu and enter its password. If no profile is found, the app says so and waits.

## Setting up without the installer

Set the `FS_name` environment variable yourself, for example `setx FS_name "FS M2-123456"`. The app looks for it in this order:

1. your user account's variables
2. the system-wide variables
3. the environment the app was started with

To run the app when you sign in, press <kbd>Win</kbd>+<kbd>R</kbd>, type `shell:startup`, and put a shortcut to `FlightScope-connect.exe` in the folder that opens.

## Development

### Building

Install [Rust](https://rustup.rs), then run:

```powershell
cargo build --release
```

The exe is created at `target\release\FlightScope-connect.exe`. Release builds open no console window. Debug builds (`cargo run`) keep the console so you can see log output.

To build the installer, install [Inno Setup](https://jrsoftware.org/isinfo.php) 6.3 or newer and run:

```powershell
& "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe" /DAppVersion=1.0.0 installer\setup.iss
```

The installer is written to `dist\`.

### Publishing a release

1. Bump `version` in `Cargo.toml` and commit.
2. Tag the commit with the same version and push the tag:

   ```powershell
   git tag v1.0.0
   git push origin v1.0.0
   ```

The [Release workflow](.github/workflows/release.yml) builds the app and the installer, then publishes a GitHub release with the installer attached. Installed copies of the app update from that release. The workflow stops with an error if the tag doesn't match the version in `Cargo.toml`. Without that check, installed copies would install the same release over and over.

### Project layout

| Path | Contents |
| --- | --- |
| `src/main.rs` | The window, connection loop, update flow and settings view (built with [egui/eframe](https://github.com/emilk/egui)) |
| `src/theme.rs` | Colours, fonts and widget styling, based on FlightScope's website |
| `src/tray.rs` | The system tray icon, and hiding/showing the window |
| `src/wifi.rs` | Wifi state and control through the Windows WLAN API (`wlanapi.dll`) |
| `src/update.rs` | Checks GitHub releases, downloads and verifies the installer, and runs it |
| `src/i18n.rs` | Languages and the translated text |
| `installer/setup.iss` | Inno Setup script for the installer |
| `assets/fonts/` | The Nunito font, under the SIL Open Font License (`OFL.txt`) |
| `assets/icon.ico`, `assets/icon.png` | The app icon. Regenerate both with `python assets/make_icon.py` (needs Pillow) |
| `build.rs` | Embeds the icon in the exe; needs the Windows SDK's resource compiler (`rc.exe`) |

## Troubleshooting

| Problem | What to check |
| --- | --- |
| "FS_name is not set" | Run the installer again, or set the variable yourself (see above). |
| "No wifi adapter" | The PC needs a wifi adapter to reach the Mevo; plug in a USB wifi adapter. |
| "Wifi adapter is disabled" | Click **Open network connections**, right-click the wifi adapter and choose **Enable**. This needs administrator rights. |
| "Wifi is off" | Turn wifi on in Windows, for example with **Open wifi menu**. |
| "Network not saved in Windows" | Join the Mevo's network once from the Windows wifi menu. |
| "Mevo not found" | Switch the Mevo on and move it closer to the PC. Check that the network name matches exactly, including spaces and case. |
| Stays on "Connecting…" | Check that the Mevo is switched on and in range, and that the network name matches exactly, including spaces and case. |
| Badge shows "Not a Mevo name" | The network name you entered is probably wrong. Run the installer again to change it. |
| "Update failed" | The PC probably has no internet connection. Try **Check for updates** again later. |

## Security

The app runs with the signed-in user's normal rights. It doesn't listen for network connections and stores no passwords or other secrets; its settings file holds only the language and the auto-update choice. It's installed per user without administrator rights, so it can't be used to gain extra privileges. Its dependencies are pinned in `Cargo.lock`, and release builds use `--locked`.

### Known risks

**Updates trust this GitHub repository.** This is the main risk. The updater downloads the installer from the latest GitHub release over HTTPS and checks its SHA-256 hash. The expected hash comes from the same GitHub response as the download, so the check catches corrupted downloads but not a malicious release. Anyone who can publish a release here can run code on every installed copy: right away if automatic updates are on, otherwise when the user clicks **Install**. That includes anyone who gains access to the repository owner's account or a collaborator's, or who can push a `v*` tag. It also includes whoever controls the third-party GitHub Actions the release workflow uses (`actions/checkout`, `dtolnay/rust-toolchain`), which are referenced by tag rather than by commit hash. The installer and the exe are also not code-signed.

Smaller issues, each of which needs code already running as the same user:

- **System tools found by name.** The app starts `powershell`, `explorer` and `control` by name rather than by full path. Windows' program search can include the app's own folder, which the user can write to.
- **Predictable download path.** The downloaded installer is saved in `%TEMP%` under a predictable name and run after the hash check. Another program could replace it in between.
- **Lookalike networks.** The app keeps reconnecting to any network with the configured name. If the Mevo's network has no password, a nearby device broadcasting the same name could be joined instead. A WPA2 password saved in the Windows profile prevents this.

### Planned improvements

1. Sign releases and have the updater refuse unsigned installers. Options are an Authenticode certificate, which also removes the SmartScreen warning, or an Ed25519/minisign key kept offline with its public half built into the app.
2. Pin the workflow's GitHub Actions to commit hashes, and protect `v*` tags so only the maintainer can create them.
3. Start system tools by their full `System32` paths, and save downloads under a random file name.

To report a security problem, use **Report a vulnerability** on the repository's **Security** tab, so it can be fixed before it's made public.

## License

Copyright (C) 2026 N3073

This program is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the [GNU Affero General Public License](LICENSE) for more details.

The bundled Nunito font (`assets/fonts/`) is licensed separately under the SIL Open Font License 1.1 (`assets/fonts/OFL.txt`).

FlightScope and Mevo are trademarks of FlightScope. This project is not affiliated with or endorsed by FlightScope.
