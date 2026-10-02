# Domain glossary

- **Telemetry snapshot** — one poll of War Thunder's loopback telemetry interface.
- **Combat event** — a kill or death attributed to the configured player from the local HUD event feed.
- **Battle observation** — the immutable, locally recorded result of one completed battle. It records which inputs covered the full battle; unsupported, incomplete, or ambiguous facts remain unknown rather than becoming zero.
- **Observed record** — an aggregate over complete battle observations stored on this device.
- **Combat Signature** — a presentation derived from the observed record: signal score, archetype, transmission, confidence, and sample size.
- **Signal score** — a 0–99 estimate based only on supported local observations. It is not a global rank or a lifetime-account rating.
- **Archetype** — a deterministic label describing observed play, such as `AIRSPACE DENIAL` or `CAREER SURVIVOR`.
- **Transmission** — the short tactical or unhinged line selected for Discord Rich Presence.
- **Career Signature** — reserved for a future external statistics provider. Career data must never be silently mixed with the local Combat Signature.
