# FlightScope Mevo Connect

A small Windows app that connects the PC's wifi adapter to a FlightScope Mevo ball flight monitor. It is meant to run when the machine starts, so the PC joins the Mevo's wifi network without anyone having to do it by hand.

## How it works

1. When the app starts, it reads the name of the Mevo's wifi network from the `FS_name` environment variable. This is the only network the app will ever connect to, and it can't be changed from the app.
2. The window shows that network name with a badge:
   - **✔ Mevo network** (green): the name matches the Mevo naming pattern. Names count as a match if they start with `FS` or contain `Mevo`; case doesn't matter.
   - **⚠ Not a Mevo name** (amber): the name doesn't match. The app still connects to it, but check that `FS_name` is set correctly.
3. The app starts connecting straight away. Every 2 seconds it checks whether the PC is already on the network. If it isn't, it runs `netsh wlan connect` again. The status panel shows the attempt number and Windows' reply. It keeps retrying until it connects, so you can start it before the Mevo is switched on.
4. Once connected, the app shows **Connected** for 3 seconds and then closes.

**Cancel** stops the retries, and **Connect** starts them again.

If `FS_name` isn't set, the app shows an error and doesn't try to connect.

## Settings

Click the **⚙** button in the top-right corner to open the settings view.

| Setting | Description |
| --- | --- |
| Language | The language of the app: English (UK), Suomi, Svenska, Deutsch, Italiano, Français, Español or Português. On first launch the app uses the Windows display language if it's one of these, otherwise English. |

Settings are saved when the app closes, to `%APPDATA%\FlightScope Mevo Connect\data\app.ron`. Messages that come from Windows itself (`netsh` output) stay in the Windows display language.

## Requirements

- Windows 10 or 11 with a wifi adapter.
- A **saved Windows wifi profile** for the Mevo's network. `netsh` can only connect to networks Windows has joined before. Before using the app, join the Mevo's network once from the Windows wifi menu and enter its password. If no profile is found, the app shows a warning.

## Setup

1. Set the environment variable `FS_name` to the exact name of the Mevo's wifi network. In PowerShell:

   ```powershell
   setx FS_name "FS M2-123456"
   ```

   This sets it permanently for the current user. To set it for all users, use *System Properties → Environment Variables*.

2. Build the app (see below), or use a pre-built `FlightScope-connect.exe`.

3. To run it at boot, either:
   - press <kbd>Win</kbd>+<kbd>R</kbd>, type `shell:startup`, and put a shortcut to `FlightScope-connect.exe` in the folder that opens; or
   - create a Task Scheduler task with the trigger **At log on** that runs `FlightScope-connect.exe`.

## Building

Install [Rust](https://rustup.rs), then run:

```powershell
cargo build --release
```

The exe is created at `target\release\FlightScope-connect.exe`. Release builds open no console window. Debug builds (`cargo run`) keep the console so you can see log output.

## Project layout

| File | Contents |
| --- | --- |
| `src/main.rs` | The window, connection loop and settings view (built with [egui/eframe](https://github.com/emilk/egui)) |
| `src/netsh.rs` | Wrappers around the Windows `netsh wlan` commands |
| `src/i18n.rs` | Languages and the translated text |

## Troubleshooting

| Problem | What to check |
| --- | --- |
| "FS_name is not set" | Set the variable and **restart** the app. Programs only see environment variables that existed when they started, so you may need to sign out and back in. |
| "No saved profile" warning | Join the Mevo's network once from the Windows wifi menu. |
| Stays on "Connecting…" | Check that the Mevo is switched on and in range, and that `FS_name` matches the network name exactly, including spaces and case. |
| Badge shows "Not a Mevo name" | `FS_name` probably contains the wrong network name. |
