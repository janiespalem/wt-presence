# Discord Identity Implementation Plan

> Execute task-by-task. Every behavior change starts with a failing test and each task ends in an atomic commit.

**Goal:** Ship a zero-configuration, project-owned Discord identity with original artwork, automatic activity assets, stable updates, and no user-supplied Application ID.

**Architecture:** A small `DiscordIdentity` module owns the public application ID and maps semantic game states to portal asset keys. Settings migrate from schema 1 to schema 2 without losing presets, while a publisher sends changes immediately and uses a 15-second heartbeat for efficient reconnect detection. The dashboard exposes only meaningful presence controls and explains the one-time vanilla activity toggle.

**Tech Stack:** Rust 2024, Serde, Tokio, `discord-rich-presence`, Vue 3, TypeScript, Vite, Discord Developer Portal assets

**Spec:** `docs/design/discord-identity.md`

## Global Constraints

- Canonical Discord Application ID: `1555607328965926974`.
- Product name: `WT Presence`.
- No Discord client secret, bot token, OAuth flow, or user-provided Application ID.
- No official War Thunder logo, copied Gaijin artwork, flags, screenshots, or copied vehicle silhouettes.
- Existing templates, telemetry URL, dashboard port, and startup preferences survive schema migration.
- Discord connection failures never stop telemetry collection or the local dashboard.
- Keep unrelated existing README and screenshot work intact.

## Review Focus

- A schema-1 settings file containing `discord_application_id` must migrate in place without being renamed as corrupt or losing custom presets.
- An unknown game state or legacy `war_thunder` artwork key must resolve to `presence-default`, not a blank asset.
- Two identical rendered activities inside 15 seconds must produce one IPC publish, while any visible field change must publish immediately.
- A 15-second heartbeat, a failed publish, and clearing presence must each allow the unchanged activity to be retried after Discord or War Thunder reconnects.
- Discord being closed or reconnecting must remain a recoverable runtime diagnostic, not terminate the process.

---

### Task 1: Canonical Identity and Automatic Artwork

**Files:**
- Create: `src/discord_identity.rs`
- Modify: `src/lib.rs`
- Modify: `src/presence.rs`
- Test: `tests/presence_renderer.rs`

**Interfaces:**
- Consumes: `GameSnapshot`, `GamePhase`, and `VehicleKind` from the existing domain module.
- Produces: `DiscordIdentity::application_id() -> &'static str`, `DiscordIdentity::asset_key(PresenceArtwork) -> &'static str`, and `DiscordIdentity::artwork_for(&GameSnapshot) -> PresenceArtwork`.
- Produces: automatic artwork when `PresencePreset.large_image` is `None`; explicit canonical keys remain valid overrides.

- [ ] **Step 1: Write failing renderer tests for semantic artwork**

Add `automatic_artwork_tracks_phase_and_vehicle_domain` with literal expectations:

```rust
assert_eq!(render(&hangar).large_image.as_deref(), Some("presence-hangar"));
assert_eq!(render(&air_battle).large_image.as_deref(), Some("presence-air"));
assert_eq!(render(&ground_battle).large_image.as_deref(), Some("presence-ground"));
assert_eq!(render(&naval_battle).large_image.as_deref(), Some("presence-naval"));
assert_eq!(render(&unknown).large_image.as_deref(), Some("presence-default"));
```

Add `legacy_default_artwork_uses_canonical_asset` and assert a preset containing `large_image: Some("war_thunder")` renders `presence-default`.

- [ ] **Step 2: Run the focused tests and verify RED**

Run: `cargo test --test presence_renderer artwork`

Expected: FAIL because automatic semantic artwork and the legacy alias do not exist.

- [ ] **Step 3: Implement the identity module and renderer mapping**

Create:

```rust
pub struct DiscordIdentity;

pub enum PresenceArtwork {
    Default,
    Air,
    Ground,
    Naval,
    Hangar,
}
```

Map the exact application ID and asset keys from the spec. Loading and offline resolve to `Default`; hangar resolves to `Hangar`; battle uses the active vehicle domain. Change `PresencePreset::minimal()` to use `large_image: None` so automatic selection is the default. Treat `war_thunder` as a legacy alias for `presence-default`.

- [ ] **Step 4: Run focused and complete Rust tests**

Run: `cargo test --test presence_renderer`

Expected: all presence renderer tests pass.

Run: `cargo test`

Expected: all Rust tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/discord_identity.rs src/lib.rs src/presence.rs tests/presence_renderer.rs
git commit -m "feat: add canonical Discord identity"
```

### Task 2: Lossless Settings Migration

**Files:**
- Modify: `src/config.rs`
- Modify: `tests/settings_store.rs`
- Modify: `tests/local_api.rs`

**Interfaces:**
- Consumes: existing `SettingsStore::load_or_create()` and `AppSettings` serialization.
- Produces: settings schema version `2` without `discord_application_id`.
- Produces: an internal schema-1-to-schema-2 migration performed before validation and persisted atomically.

- [ ] **Step 1: Write a failing schema migration test**

Add `migrates_schema_one_without_losing_user_configuration`. Write a literal schema-1 JSON fixture containing:

- `discord_application_id: "999"`;
- a custom telemetry URL on loopback;
- a custom active preset and templates;
- non-default startup preferences.

Assert that `load_or_create()` returns schema 2, preserves every supported value, does not create a corrupt backup, rewrites the file without `discord_application_id`, and remains stable on a second load.

- [ ] **Step 2: Write a failing local API serialization test**

Add `settings_response_does_not_expose_obsolete_discord_id` and assert the JSON returned by `GET /api/v1/settings` has `schema_version == 2` and no `discord_application_id` key.

- [ ] **Step 3: Run the focused tests and verify RED**

Run: `cargo test --test settings_store migrates_schema_one_without_losing_user_configuration`

Expected: FAIL because schema 1 is rejected and the obsolete field still exists.

Run: `cargo test --test local_api settings_response_does_not_expose_obsolete_discord_id`

Expected: FAIL because the current settings response includes the field.

- [ ] **Step 4: Implement schema version 2 and migration**

Remove `discord_application_id` from `AppSettings`, set the default schema to 2, and make `SettingsStore::load_or_create()` upgrade version 1 before validation. Unknown legacy JSON fields are ignored by Serde; after migration, save the supported schema-2 representation atomically.

Unsupported future schema versions must still return `SettingsError::Invalid` and must not be treated as corrupt JSON.

- [ ] **Step 5: Run focused and complete Rust tests**

Run: `cargo test --test settings_store --test local_api`

Expected: all settings and local API tests pass.

Run: `cargo test`

Expected: all Rust tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/config.rs tests/settings_store.rs tests/local_api.rs
git commit -m "feat: migrate to built-in Discord identity"
```

### Task 3: Canonical Runtime and Idempotent Publishing

**Files:**
- Modify: `src/runtime.rs`
- Modify: `src/main.rs`
- Test: `tests/runtime_engine.rs`

**Interfaces:**
- Consumes: `DiscordIdentity::application_id()` from Task 1 and the existing `PresenceSink` interface.
- Produces: `DiscordPresenceSink::canonical() -> DiscordPresenceSink`.
- Produces: `PresencePublisher<S>::new(inner: S, refresh_interval: Duration) -> PresencePublisher<S>` with `publish(&mut self, activity: &DiscordActivity, now: DateTime<Utc>) -> Result<PublishOutcome, String>` and `clear(&mut self) -> Result<(), String>`.

- [ ] **Step 1: Write failing deduplication tests**

Add `suppresses_identical_activities_until_heartbeat_but_publishes_changes` using the real publisher around `FakeSink`. Publish the same literal `DiscordActivity` at `t=0` and `t=1`, change `state` at `t=2`, then publish that unchanged activity again at `t=17`. Assert the inner event sequence contains the initial publish, the immediate changed publish, and the 15-second heartbeat publish.

Add `clear_allows_the_same_activity_after_reconnect`. Publish, clear, publish the same activity, and assert `publish`, `clear`, `publish` reaches the inner sink.

Add `failed_publish_is_retried_without_waiting_for_heartbeat` with a sink that fails once. Assert the first error is returned and an identical activity at the next timestamp reaches the sink again.

- [ ] **Step 2: Run the focused tests and verify RED**

Run: `cargo test --test runtime_engine heartbeat`

Run: `cargo test --test runtime_engine reconnect`

Expected: FAIL because `PresencePublisher` does not exist.

- [ ] **Step 3: Implement the publisher with deduplication and heartbeat**

Store the last successfully published `DiscordActivity` and timestamp. Skip an identical publish before 15 seconds, publish changes immediately, and republish unchanged activity when the heartbeat is due. Cache only successful publishes. Reset the cache whenever clear is requested, even if the underlying clear reports a lost Discord connection. Keep every transport error visible to the caller.

- [ ] **Step 4: Make the production sink canonical**

Add `DiscordPresenceSink::canonical()` and make its ID-specific constructor private. Make `RuntimeEngine` own a `PresencePublisher<S>` configured with a 15-second refresh interval. In `spawn_runtime`, always construct the canonical sink:

```rust
DiscordPresenceSink::canonical()
```

Remove the settings-based branch and the production use of `DisabledPresenceSink`. Preserve reconnect behavior when Discord starts after WT Presence.

- [ ] **Step 5: Run runtime tests and the complete Rust suite**

Run: `cargo test --test runtime_engine`

Expected: all runtime tests pass.

Run: `cargo test`

Expected: all Rust tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/runtime.rs src/main.rs tests/runtime_engine.rs
git commit -m "feat: publish through canonical Discord app"
```

### Task 4: Zero-Configuration Dashboard and Documentation

**Files:**
- Modify: `web/src/types.ts`
- Modify: `web/src/App.vue`
- Modify: `web/src/style.css`
- Modify: `README.md`

**Interfaces:**
- Consumes: schema-2 `AppSettings` from Task 2 and canonical asset keys from Task 1.
- Produces: a settings editor with no Application ID field and an artwork selector whose `Automatic` value serializes as `null`.

- [ ] **Step 1: Remove obsolete client settings and expose known artwork choices**

Delete `discord_application_id` from the TypeScript `AppSettings`. Replace free-form large/small asset text inputs with one artwork selector:

```text
Automatic -> null
Default   -> presence-default
Air       -> presence-air
Ground    -> presence-ground
Naval     -> presence-naval
Hangar    -> presence-hangar
```

Keep `small_image` in serialized presets for backward compatibility but do not expose a free-form control in the default dashboard.

- [ ] **Step 2: Add the duplicate-activity guidance**

In the Discord diagnostics/presence view, explain that users can disable War Thunder under `User Settings → Registered Games` when they want WT Presence to be the only visible card. Do not claim that WT Presence can change Discord settings automatically.

- [ ] **Step 3: Update README without overwriting unrelated edits**

Replace the create-your-own-Discord-application instructions with zero-configuration startup and the one-time Registered Games step. State that the project is unofficial and unaffiliated with Gaijin Entertainment. Preserve the current screenshot and dashboard documentation edits already present in the worktree.

- [ ] **Step 4: Build the dashboard and run backend tests**

Run: `npm run build` from `web/`.

Expected: TypeScript and Vite build succeed.

Run: `cargo test`.

Expected: all Rust tests pass.

- [ ] **Step 5: Commit only the intended dashboard and README changes**

```bash
git add web/src/types.ts web/src/App.vue web/src/style.css README.md
git commit -m "feat: make Discord setup zero-configuration"
```

### Task 5: Original Discord Artwork and Release Gate

**Files:**
- Create: `assets/brand/source/wt-presence-icon.svg`
- Create: `assets/brand/source/presence-default.svg`
- Create: `assets/brand/source/presence-air.svg`
- Create: `assets/brand/source/presence-ground.svg`
- Create: `assets/brand/source/presence-naval.svg`
- Create: `assets/brand/source/presence-hangar.svg`
- Create: `assets/brand/wt-presence-icon-1024.png`
- Create: `assets/brand/presence-default.png`
- Create: `assets/brand/presence-air.png`
- Create: `assets/brand/presence-ground.png`
- Create: `assets/brand/presence-naval.png`
- Create: `assets/brand/presence-hangar.png`
- Create: `assets/brand/README.md`
- Modify: `docs/design/discord-identity.md`

**Interfaces:**
- Consumes: the exact portal keys selected by `DiscordIdentity` in Task 1.
- Produces: square 1024×1024 PNG exports and an upload manifest matching every runtime key.

- [ ] **Step 1: Produce the original master icon**

Create a high-contrast graphite/cyan/warm-accent radar mark following the spec, then reproduce the approved geometry as maintainable original SVG artwork. Inspect the export at full resolution and at 32×32. Reject any result containing letters, flags, screenshots, official game branding, copied vehicle silhouettes, gradients that collapse at small size, or fine detail that turns to noise.

- [ ] **Step 2: Produce the five related rich-presence assets**

Keep one visual system and vary only the semantic contact geometry for default, air, ground, naval, and hangar. Store each source as SVG and export a square PNG. Each file must remain distinguishable at Discord card size without relying on text.

- [ ] **Step 3: Validate image dimensions and manifest coverage**

Run: `identify assets/brand/*.png`

Expected: every file reports `1024x1024` and a valid PNG colorspace.

Document the application icon filename and exact upload key for each presence asset in `assets/brand/README.md`. Cross-check every documented key against `DiscordIdentity::asset_key`.

- [ ] **Step 4: Prepare portal upload and manual verification instructions**

Prepare the application icon and five Rich Presence assets for application `1555607328965926974`. The maintainer uploads them through Discord Developer Portal. Verify on Windows with Discord Desktop and War Thunder:

1. launch WT Presence without any local Application ID setting;
2. verify icon and automatic asset rendering;
3. move from hangar to air or ground battle and verify the asset changes;
4. verify the elapsed timer does not reset;
5. disable vanilla War Thunder in Registered Games and verify only WT Presence remains;
6. close Discord, confirm WT Presence stays running, reopen Discord, and confirm presence reconnects.

- [ ] **Step 5: Record implementation status and commit artwork**

Update the design status to `implemented; portal verification pending`. After the maintainer confirms the portal checklist, change it to `implemented and manually verified` in a separate follow-up commit.

```bash
git add assets/brand docs/design/discord-identity.md
git commit -m "assets: add WT Presence Discord identity"
```

### Task 6: Final Verification

**Files:**
- Verify only; modify files solely to fix failures discovered here.

**Interfaces:**
- Consumes: all previous tasks.
- Produces: a clean, reproducible release candidate with remaining manual limitations reported explicitly.

- [ ] **Step 1: Run repository checks**

Run: `cargo test`

Expected: all Rust tests pass.

Run: `npm run build` from `web/`.

Expected: the production dashboard build succeeds.

Run: `git diff --check`.

Expected: no whitespace errors.

- [ ] **Step 2: Inspect the complete branch diff**

Confirm there are no credentials, copied Gaijin branding, obsolete manual App ID instructions, tool artifacts, decorative source comments, or accidental changes outside the approved scope.

- [ ] **Step 3: Report environmental gaps honestly**

If Windows, Discord Desktop, War Thunder, Developer Portal upload access, `rustfmt`, `clippy`, or ImageMagick are unavailable, identify the exact unchecked item rather than treating it as passed.
