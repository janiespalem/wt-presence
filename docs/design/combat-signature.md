# Combat Signature

Status: approved product direction; implementation pending

## Product promise

WT Presence turns locally observed play into a compact, shareable identity instead of dumping raw counters into Discord.

Example activity:

```text
J-7D · Air Simulator
SIGNAL 96 // AIRSPACE ANOMALY
```

The feature remains privacy-first:

- no Gaijin login;
- no cloud account;
- no profile scraping;
- no reading process memory, packets, or hidden map information;
- all observations and classification stay on the player's machine.

The product slogan becomes:

> Fight the battle. Broadcast your signature.

## What the signature contains

```text
CombatSignature
├── signal: 0..99
├── archetype: stable behavioral label
├── transmission: contextual short line
├── confidence: calibrating | provisional | established
├── battles_observed: integer
└── capabilities: which inputs were actually available
```

The signature is an observed-device estimate, not a claim about the whole account. The UI always exposes its sample size. Before three attributable completed battles, the public output is:

```text
SIGNAL CALIBRATING // 2 BATTLES OBSERVED
```

## Data boundary

The local War Thunder web interface exposes `/hudmsg?lastEvt=…&lastDmg=…` while a battle is active. The game's own web UI polls that endpoint, but its payload is not a stable documented API. Community reports also show fields such as `enemy` behaving inconsistently. WT Presence therefore treats the feed as an untrusted adapter, not as domain truth.

Attribution requires a player callsign in local settings. This is plain text, not Gaijin authentication. A HUD event counts only when the callsign occupies an unambiguous killer or victim position. Unknown formats and unsupported event types are ignored and surfaced in diagnostics. If an event contains the callsign but cannot be attributed unambiguously, the battle's event coverage becomes `partial` rather than silently accepting a possibly wrong zero.

On the first HUD poll, the adapter advances its cursors without emitting old events. Later polls use separate event and damage cursors, deduplicate by battle and event ID, and never infer information from enemy map markers.

The combat feed tracks whether it covered the complete battle. A zero is valid only after the adapter was primed at battle start and remained recoverable through battle completion. If coverage begins late or an outage cannot be replayed from the unchanged cursor, kill and death values remain unknown and the battle does not enter the scoring sample.

The first release supports only facts proven by live fixtures:

- completed battle count;
- player-attributed kills;
- player-attributed deaths;
- vehicle domain used: air, ground, naval, or mixed.

Win rate, captures, assists, target type, nuclear strikes, flanking, and ground-versus-air kills stay out until a reliable local source is demonstrated. Missing data is `unknown`, never `0`.

## Scoring model

The score uses the latest 20 completed battles with complete attributed combat-event coverage. Full observations remain stored locally; only the evaluation window is bounded.

For `b` battles, `k` kills, and `d` deaths:

```text
smoothed_kd = (k + 2) / (d + 2)
kills_per_battle = k / b
raw = 50
    + 18 * log2(smoothed_kd)
    + 10 * (kills_per_battle - 1)
raw = clamp(raw, 0, 99)

sample_weight = min(b / 20, 1)
signal = round(50 + sample_weight * (raw - 50))
```

The prior prevents one absurd match from immediately producing `SIGNAL 99`; sample weighting pulls small samples toward neutral.

Confidence is deterministic:

- `calibrating`: 0–2 attributable battles;
- `provisional`: 3–9 attributable battles;
- `established`: 10 or more attributable battles.

The dashboard shows the inputs and formula result so the score is inspectable rather than mystical.

## Archetypes

Archetype selection is a priority-ordered rule set over the same evaluation window. Every rule states the data capability it requires.

1. `STATISTICAL ANOMALY` — signal at least 90 with established confidence.
2. `AIRSPACE DENIAL` — at least 70% of observed battles are air and signal is at least 65.
3. `COMBINED ARMS MENACE` — at least three air and three ground battles, neither domain below 25% of the sample, and signal is at least 60.
4. `CAREER SURVIVOR` — at least ten battles, smoothed K/D at least 2.0, and deaths per battle below 0.75.
5. `ROCK INSPECTOR` — at least five battles, smoothed K/D below 0.65.
6. `COMBAT EFFECTIVE` — fallback once calibration is complete.
7. `SIGNAL CALIBRATING` — fewer than three attributable battles.

Labels that require unavailable evidence are intentionally deferred:

- `CAS OFFENDER` needs target-domain classification;
- `OBJECTIVE GREMLIN` needs capture events;
- `APEX FLANKER` needs movement and engagement geometry;
- `LAST LINE` needs trustworthy team and objective state.

## Transmissions and voice packs

The score and archetype remain factual. Tone is a separate presentation choice.

`Tactical` is the default:

- `AIRSPACE CONTROL ESTABLISHED`
- `MULTI-DOMAIN CONTACT`
- `SURVIVAL PATTERN CONFIRMED`
- `COMBAT DATA INSUFFICIENT`

`Unhinged` is opt-in:

- `AIRSPACE ANOMALY`
- `MATCHMAKER INCIDENT`
- `TEAM NOT INCLUDED`
- `CREW REASSIGNMENT PENDING`
- `SUPERNATURAL CONTACT`
- `YOUR MISSILES ARE MY DANCING PARTNERS`

Contextual transmissions may react to the current battle without changing the observed score:

- ten kills and no death: `NUCLEAR INTENT DETECTED`;
- K/D within 0.95–1.05 after ten battles: `PERFECTLY BALANCED`;
- signal at least 85 during a mixed-domain session: `MULTI-DOMAIN INCIDENT`;
- three deaths and no kill in the current battle: `CREW REASSIGNMENT PENDING`;
- a rare deterministic selection for signal 95+: `SUPERNATURAL CONTACT` or `YOUR MISSILES ARE MY DANCING PARTNERS`.

Rare selection is seeded from the battle ID. It must be stable during a battle and must not flicker between polls.

The bundled signature preset renders `SIGNAL {{ signature.signal }} // {{ signature.transmission }}`. Archetype remains independently available to custom templates and appears beside the transmission in the dashboard.

## Module design

The feature uses two deep interfaces.

### Local combat feed

```rust
trait CombatEventSource {
    async fn poll(&mut self, battle: Option<BattleId>) -> Result<CombatFeedBatch, CombatFeedError>;
}
```

The production adapter owns HTTP cursors, cold-start priming, payload parsing, callsign attribution, and deduplication. Runtime code receives only normalized `CombatEvent` values plus diagnostics. Tests use an in-memory source through the same interface.

### Signature engine

```rust
impl CombatSignatureEngine {
    fn evaluate(
        &self,
        record: &ObservedRecord,
        current: Option<&BattleSummary>,
        voice: VoicePack,
    ) -> CombatSignature;
}
```

This is a pure module. Callers do not reproduce thresholds, confidence rules, priority, or phrase selection.

Runtime flow:

```text
War Thunder localhost
  ├── telemetry adapter ──> GameSnapshot
  └── HUD event adapter ──> CombatFeedBatch
                              │
                              v
                         SessionEngine
                              │ battle completed
                              v
                    BattleObservationRepository
                              │
                              v
                    CombatSignatureEngine
                              │
                  ┌───────────┴───────────┐
                  v                       v
           PresenceRenderer          Local API/UI
```

`RuntimeEngine` orchestrates these modules but owns none of their parsing or scoring rules.

## Persistence

SQLite schema version 2 adds immutable completed observations:

```text
battle_observations
  battle_id        TEXT PRIMARY KEY
  started_at       INTEGER NOT NULL
  ended_at         INTEGER NOT NULL
  vehicle_kinds    TEXT NOT NULL
  event_coverage   TEXT NOT NULL
  kills            INTEGER NULL
  deaths           INTEGER NULL
  payload_json     TEXT NOT NULL
```

`vehicle_kinds` represents every domain used in the battle, so a ground battle with an aircraft spawn is not flattened into one misleading value. `event_coverage` distinguishes `complete`, `partial`, and `unavailable`; only `complete` observations contribute kill/death metrics.

The repository writes a battle only after completion and uses the battle ID for idempotency. A crash may lose the unfinished battle but cannot duplicate a completed one on restart. Existing session history remains readable.

## Settings and public surface

Settings schema version 2 adds:

- `player_callsign: Option<String>`;
- `signature_enabled: bool`, default `true`;
- `signature_voice: tactical | unhinged`, default `tactical`.

Loading schema version 1 performs an explicit migration that preserves the Discord application ID, active preset, and every custom template. Invalid settings recovery remains a separate path; a valid older file is never renamed as corrupt merely because it needs migration.

Presence templates gain:

```text
signature.signal
signature.archetype
signature.transmission
signature.confidence
signature.battles_observed
signature.available
```

The renderer receives a `CombatSignature`; it does not query storage or calculate one. Existing presets continue to render unchanged. A new `Combat Signature` preset demonstrates the feature without silently replacing a user's active preset.

The local API adds `GET /api/v1/signature`. It returns the current computed value, input capability flags, and ignored-event diagnostics without returning unrelated raw HUD messages.

## Dashboard and preview

The dashboard adds one signature card:

- large signal value or `CALIBRATING`;
- archetype and active transmission;
- confidence and observed battle count;
- compact explanation of contributing metrics;
- callsign and voice-pack controls;
- a clear warning when kill/death attribution is unavailable.

Preview Lab receives deterministic `air`, `ground`, `mixed`, and `calibrating` signature fixtures. This makes the entire feature testable without launching War Thunder or Discord and provides honest README screenshots.

## Failure behavior

- Telemetry available, HUD feed unavailable: live vehicle presence continues; signature retains its last observed record and reports event capability unavailable.
- Malformed HUD payload: keep cursors unchanged, record a bounded diagnostic, retry later.
- Unknown localized message: ignore it; never guess.
- Callsign absent: show calibration/setup guidance and do not count kills or deaths.
- Storage failure: continue live presence, expose the error, and do not claim the observation was retained.
- Discord unavailable: signature and dashboard remain functional locally.

Diagnostics retain counts and reasons, not a permanent archive of chat or HUD text.

## Alternatives considered

### Session-only badges

Easy to implement but reset on every launch and reward one lucky match. Rejected as the primary identity; current-battle events remain useful only for transmissions.

### StatShark-first career score

Richer historical data, but API access is not public and would make the feature depend on a third party. Deferred behind a future `Career Signature` adapter.

### One giant telemetry client

Fewer types initially, but it would combine fast snapshots, cursor-based events, attribution, and scoring in one boundary. Rejected in favor of a small combat-feed adapter and a pure signature engine.

### Parsing every localized kill-feed sentence

Would appear comprehensive while silently corrupting stats. Rejected. The parser is fixture-driven and fail-closed.

## Verification contract

Implementation is complete only when:

1. captured fixtures prove cold-start cursoring, kill attribution, death attribution, duplicates, ambiguous callsigns, and at least two game languages;
2. scoring and archetype tables are covered by deterministic unit tests at every boundary;
3. database and settings migrations from schema 1 to 2 preserve existing sessions, Discord configuration, and custom presets;
4. runtime tests prove HUD failure cannot stop telemetry presence;
5. template tests prove old presets remain compatible and all signature variables render;
6. Preview Lab renders every confidence state and both voice packs offline;
7. the README includes a real dashboard capture and a real Discord activity capture, not a mock Discord interface.

## Explicit non-goals

- global rankings or comparisons with other players;
- Gaijin credentials or browser-session reuse;
- enemy tracking, aiming assistance, overlays, or extra in-battle awareness;
- promises about win rate or nuclear strikes without a trustworthy source;
- uploading observations by default;
- claiming local observations are lifetime account statistics.
