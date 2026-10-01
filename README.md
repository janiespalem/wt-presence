# WT Presence

Privacy-first Discord Rich Presence for War Thunder.

WT Presence reads the game's local telemetry interface, turns live game state into a configurable Discord activity, and stores optional session summaries locally. It does not require a Gaijin login or a cloud account.

The project is under active development. Windows 10/11 x64 is the first supported target.

## Install on Windows

Download `WT-Presence-Setup.exe` from the [latest beta release](https://github.com/janiespalem/wt-presence/releases). The installer is user-local and does not require administrator access. Early builds are unsigned, so Windows SmartScreen may show an unknown-publisher warning.

## Run from source

Requirements: Rust 1.95, Node.js 22+, npm, War Thunder, and Discord Desktop.

```sh
cd web
npm ci
npm run build
cd ..
cargo run --release
```

The dashboard is served only on `127.0.0.1`. War Thunder must be running for live telemetry to appear.
Set a Discord application ID in the dashboard and restart WT Presence to publish the activity.

## License

[Mozilla Public License 2.0](LICENSE)
