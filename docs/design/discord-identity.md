# Discord Identity

Status: implemented; portal verification pending

## Product promise

WT Presence replaces Discord's basic detected-game card with a richer, configurable War Thunder activity. It does not modify or impersonate Gaijin's Discord application.

Users disable War Thunder once under Discord's Registered Games settings when they want WT Presence to be the only visible activity. WT Presence cannot change that Discord setting on the user's behalf.

## Canonical application

Every release uses one project-owned Discord application:

```text
name: WT Presence
application_id: 1555607328965926974
```

The application ID is public configuration and is compiled into the client. Users do not create Discord applications or paste IDs into settings. No client secret, bot token, OAuth token, or Discord account login is required.

The application profile must state that WT Presence is an unofficial companion for War Thunder and is not affiliated with Gaijin Entertainment.

## Visual identity

WT Presence uses original artwork rather than the War Thunder logo or other Gaijin branding.

The application icon is a square, high-contrast mark inspired by a radar or multifunction display:

- dark graphite background;
- cyan telemetry sweep;
- warm threat/contact accent;
- simple geometry that remains legible at Discord's smallest icon size;
- no letters, game logos, flags, screenshots, or copied vehicle silhouettes.

The first asset set is intentionally small:

```text
presence-default
presence-air
presence-ground
presence-naval
presence-hangar
```

The application icon is the fallback. A known game phase or vehicle domain selects a corresponding large asset. Unknown semantic states and the legacy `war_thunder` key resolve to `presence-default`. The release gate verifies that every referenced portal asset exists; local IPC cannot inspect the portal catalog at runtime.

## Client architecture

A `DiscordIdentity` module owns the canonical application ID and bundled asset keys. Runtime callers select a semantic asset such as `Air` or `Hangar`; they do not know Discord portal keys.

```rust
pub enum PresenceArtwork {
    Default,
    Air,
    Ground,
    Naval,
    Hangar,
}

impl DiscordIdentity {
    pub fn application_id(&self) -> &'static str;
    pub fn asset_key(&self, artwork: PresenceArtwork) -> &'static str;
}
```

`DiscordPresenceSink` always receives the canonical application ID from this module. Failure to connect to a local Discord client remains non-fatal and is exposed through runtime diagnostics.

## Settings migration

The settings schema removes `discord_application_id` from the user-facing interface. Loading an older settings file ignores the obsolete value while preserving presets, ports, telemetry configuration, and dashboard preferences.

The dashboard removes the Discord Application ID control. It instead reports one of three states:

- connected to Discord;
- Discord unavailable;
- connected, with a reminder that vanilla War Thunder may also appear until disabled in Registered Games.

## Presence behavior

Changing phase, vehicle, map, details, or state updates the existing rich presence while retaining the session start timestamp. The activity is cleared only when War Thunder becomes unavailable or the application exits.

Repeated identical activities are suppressed between 15-second heartbeats. Any visible change publishes immediately. A heartbeat rechecks the IPC connection so Discord can recover after being closed and reopened without waiting for another game-state change.

## Distribution and portal setup

Repository assets include the source artwork and exported PNG files. Uploading the application icon and rich-presence asset keys to Discord's Developer Portal is a release task because Discord does not accept those uploads through the local IPC client.

The original SVG masters, 1024×1024 PNG exports, exact asset-key manifest, and maintainer checklist are in [`assets/brand/`](../../assets/brand/README.md). The portal application is currently named `12`; the maintainer must rename it to `WT Presence`, set the application icon, and upload all five keyed assets before release. Portal upload and Windows/Discord verification remain pending. Record successful manual verification in a separate follow-up commit before changing this status to `implemented and manually verified`.

A release is not considered ready until a clean Discord account verifies:

1. no user-provided Application ID is requested;
2. the application icon renders;
3. every bundled asset key renders;
4. status changes retain elapsed time;
5. disabling vanilla War Thunder leaves exactly one activity card;
6. Discord being closed does not stop telemetry or the dashboard.

## Out of scope

- modifying Gaijin's Discord application;
- automatically changing Discord Registered Games settings;
- using the official War Thunder logo without written permission;
- Discord OAuth, bots, servers, or account authorization;
- per-vehicle artwork in the first identity release;
- vehicle display-name resolution, which is a separate data-quality change.
