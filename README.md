# WT Presence

> Fight the battle. Broadcast your signature.

Configurable Discord Rich Presence for War Thunder. A small Windows tray app reads local game data and updates Discord. Settings open in your browser; closing that page does not stop the app. No Gaijin login or cloud account.

WT Presence is an unofficial project and is not affiliated with Gaijin Entertainment.

![WT Presence settings in Russian, with an offline aircraft preview](docs/assets/dashboard-preview-lab.png)

Windows 10/11 x64 is the first supported target. The project is in beta.

## What works today

- Automatic offline, hangar, loading, and battle detection.
- Live vehicle, mode, map, speed, altitude, and crew data when War Thunder exposes it.
- English and Russian settings, with help beside every field.
- Two editable status lines with a menu for vehicle, speed, map, crew, and game status.
- Aircraft, ground, and hangar previews that work without the game or Discord.
- Optional elapsed application-session time and automatic artwork. Changing vehicle or game phase keeps the timer running.
- Local session summaries stored in SQLite.
- A loopback-only dashboard protected by a per-process token.
- A user-local Windows installer and tray integration.
- Connection diagnostics, a redacted support report, and up to seven daily local logs.

Combat-event attribution and [Combat Signature](docs/design/combat-signature.md) are designed but not implemented yet. StatShark is not a runtime dependency; possible career-stat integration remains a future optional adapter.

## Install on Windows

Download `WT-Presence-Setup.exe` from the [latest beta release](https://github.com/janiespalem/wt-presence/releases). The installer is per-user and does not require administrator access.

Early builds are unsigned, so Windows SmartScreen may show an unknown-publisher warning.

## Connect Discord

WT Presence includes its Discord identity and artwork. No Application ID, Developer Portal setup, or asset uploads are required.

1. Keep Discord Desktop running.
2. Start WT Presence and launch War Thunder. The Discord connection is automatic.
3. If you want WT Presence to be the only visible activity card, disable War Thunder in Discord under **User Settings → Registered Games**. This is a one-time setting you change in Discord.

Open **Discord status** to edit the two lines, choose an image, and enable the timer. **Add live value** inserts a readable placeholder such as `[Vehicle name]`. The preview uses sample data unless **Live** is selected. Click **Save changes** to apply your edits.

The language selector changes the settings page, not your custom Discord text. Connection settings are under **Advanced settings** and require restarting WT Presence. Open settings from the tray after changing the port.

The page previews text, not Discord's exact card layout. Image assets are resolved by Discord.

## Troubleshooting

Open **Diagnostics** to check game data, the last successful Discord write, and the application version. **Copy diagnostic report** omits tokens, account details, templates, paths, and raw errors. Detailed errors stay on the local page.

On Windows, logs are in `%LOCALAPPDATA%\WT Presence\WT Presence\data\logs`. Old logs are pruned at startup and daily rotation retains at most seven files. Inspect log contents before sharing them.

Kills and deaths are not collected yet. Session history shows detected sessions and completed battles; it is not a substitute for career statistics.

## Templates

Advanced users can keep existing MiniJinja templates. The friendly editor converts its named values to templates internally. Available values include:

```text
vehicle.name          game.phase_label
vehicle.kind          game.mode
telemetry.ias         game.map
telemetry.agl         telemetry.ground_speed
telemetry.crew_current
```

Example:

```jinja
{{ vehicle.name }} · {{ game.mode }}
{{ telemetry.ias | round }} km/h · {{ telemetry.agl | round }} m
```

Invalid variables and Discord's 128-character field limit produce visible diagnostics instead of silently publishing a broken activity.

## Privacy boundary

```text
War Thunder :8111  ──>  WT Presence  ──>  Discord local IPC
                              │
                              └──> local settings and SQLite sessions
```

WT Presence binds its dashboard to `127.0.0.1`, accepts telemetry URLs only on loopback hosts, and never asks for Gaijin credentials. Session data stays on the machine unless the user exports or shares it.

## Run from source

Requirements: Rust 1.95, Node.js 22+, npm, War Thunder, and Discord Desktop.

```sh
cd web
npm ci
npm run build
cd ..
cargo run --release
```

The dashboard opens automatically. War Thunder must be running for live telemetry to appear.

## Verify a checkout

```sh
cd web
npm ci
npm test
npm run build
npx playwright install chromium
npm run test:e2e
cd ..
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

CI runs the frontend build, formatting, lint, and unit tests on Ubuntu and Windows, plus browser checks on Ubuntu.

## License

[Mozilla Public License 2.0](LICENSE)
