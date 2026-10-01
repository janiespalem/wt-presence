# WT Presence

Privacy-first Discord Rich Presence for War Thunder.

WT Presence reads the game's local telemetry interface, turns live game state into a configurable Discord activity, and stores optional session summaries locally. It does not require a Gaijin login or a cloud account.

The project is under active development. Windows 10/11 x64 is the first supported target; packaged releases are not available yet.

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
