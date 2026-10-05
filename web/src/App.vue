<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import {
  Activity,
  AlertTriangle,
  Check,
  Crosshair,
  Database,
  Gauge,
  History,
  Plane,
  RadioTower,
  Save,
  Settings2,
  ShieldCheck,
  Terminal,
} from "@lucide/vue";
import { api } from "./api";
import type {
  AppSettings,
  DiscordActivity,
  GameSnapshot,
  PreviewScenario,
  RuntimeStatus,
  StoredSession,
} from "./types";

type Tab = "overview" | "presence" | "sessions" | "diagnostics";

const tab = ref<Tab>("overview");
const status = ref<RuntimeStatus | null>(null);
const snapshot = ref<GameSnapshot | null>(null);
const settings = ref<AppSettings | null>(null);
const livePresence = ref<DiscordActivity | null>(null);
const preview = ref<DiscordActivity | null>(null);
const previewScenario = ref<PreviewScenario>("air");
const sessions = ref<StoredSession[]>([]);
const fatalError = ref("");
const previewError = ref("");
const saveState = ref<"idle" | "saving" | "saved">("idle");
let pollingTimer: number | undefined;
let previewTimer: number | undefined;

const previewScenarios: { value: PreviewScenario; label: string }[] = [
  { value: "live", label: "Live" },
  { value: "air", label: "Air" },
  { value: "ground", label: "Ground" },
  { value: "hangar", label: "Hangar" },
];

const artworkChoices: { value: string | null; label: string }[] = [
  { value: null, label: "Automatic" },
  { value: "presence-default", label: "Default" },
  { value: "presence-air", label: "Air" },
  { value: "presence-ground", label: "Ground" },
  { value: "presence-naval", label: "Naval" },
  { value: "presence-hangar", label: "Hangar" },
];

const activePreset = computed(() =>
  settings.value?.presets.find(
    (preset) => preset.id === settings.value?.active_preset_id,
  ),
);

const gameLabel = computed(() => {
  const phase = snapshot.value?.phase;
  return {
    offline: "GAME OFFLINE",
    hangar: "IN HANGAR",
    loading: "LOADING SORTIE",
    battle: "IN BATTLE",
  }[phase ?? "offline"];
});

const telemetryItems = computed(() => [
  ["IAS", number(snapshot.value?.telemetry.speed_ias_kph), "KM/H"],
  ["ALT AGL", number(snapshot.value?.telemetry.altitude_agl_m), "M"],
  ["ALT MSL", number(snapshot.value?.telemetry.altitude_msl_m), "M"],
  ["CREW", crew(), ""],
]);

function number(value: number | null | undefined): string {
  return value == null ? "—" : Math.round(value).toLocaleString();
}

function crew(): string {
  const telemetry = snapshot.value?.telemetry;
  if (telemetry?.crew_current == null) return "—";
  return `${telemetry.crew_current}/${telemetry.crew_total ?? "?"}`;
}

function date(value: string): string {
  return new Intl.DateTimeFormat(undefined, {
    month: "short",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  }).format(new Date(value));
}

async function refreshRuntime(): Promise<void> {
  try {
    const [nextStatus, nextSnapshot, nextPresence] = await Promise.all([
      api.status(),
      api.snapshot(),
      api.presence(),
    ]);
    status.value = nextStatus;
    snapshot.value = nextSnapshot;
    livePresence.value = nextPresence;
    fatalError.value = "";
  } catch (error) {
    fatalError.value = error instanceof Error ? error.message : String(error);
  }
}

async function load(): Promise<void> {
  try {
    const [nextSettings, nextSessions] = await Promise.all([
      api.settings(),
      api.sessions(),
    ]);
    settings.value = nextSettings;
    sessions.value = nextSessions;
    await refreshRuntime();
    await refreshPreview();
  } catch (error) {
    fatalError.value = error instanceof Error ? error.message : String(error);
  }
}

async function refreshPreview(): Promise<void> {
  if (!activePreset.value) return;
  try {
    preview.value = await api.preview(activePreset.value, previewScenario.value);
    previewError.value = "";
  } catch (error) {
    previewError.value = error instanceof Error ? error.message : String(error);
  }
}

async function save(): Promise<void> {
  if (!settings.value) return;
  saveState.value = "saving";
  try {
    settings.value = await api.saveSettings(settings.value);
    saveState.value = "saved";
    window.setTimeout(() => (saveState.value = "idle"), 1600);
  } catch (error) {
    saveState.value = "idle";
    fatalError.value = error instanceof Error ? error.message : String(error);
  }
}

watch(
  () => [
    activePreset.value ? JSON.stringify(activePreset.value) : "",
    previewScenario.value,
  ],
  () => {
    window.clearTimeout(previewTimer);
    previewTimer = window.setTimeout(refreshPreview, 220);
  },
);

onMounted(() => {
  void load();
  pollingTimer = window.setInterval(refreshRuntime, 1500);
});

onBeforeUnmount(() => {
  window.clearInterval(pollingTimer);
  window.clearTimeout(previewTimer);
});
</script>

<template>
  <div class="app-shell">
    <aside class="sidebar">
      <div class="wordmark">
        <div class="mark"><Plane :size="23" /></div>
        <div>
          <strong>WT//PRESENCE</strong>
          <span>LOCAL CONTROL</span>
        </div>
      </div>

      <nav aria-label="Dashboard sections">
        <button :class="{ active: tab === 'overview' }" @click="tab = 'overview'">
          <Gauge :size="18" /> Overview
        </button>
        <button :class="{ active: tab === 'presence' }" @click="tab = 'presence'">
          <RadioTower :size="18" /> Presence
        </button>
        <button :class="{ active: tab === 'sessions' }" @click="tab = 'sessions'">
          <History :size="18" /> Sessions
        </button>
        <button :class="{ active: tab === 'diagnostics' }" @click="tab = 'diagnostics'">
          <Terminal :size="18" /> Diagnostics
        </button>
      </nav>

      <div class="privacy-stamp">
        <ShieldCheck :size="18" />
        <div><strong>LOCAL ONLY</strong><span>No Gaijin login. No cloud.</span></div>
      </div>
    </aside>

    <main>
      <header class="topbar">
        <div>
          <span class="eyebrow">MISSION CONTROL / {{ tab }}</span>
          <h1>{{ tab }}</h1>
        </div>
        <div class="connections">
          <span :class="['connection', { online: status?.telemetry_connected }]">
            <i></i> War Thunder
          </span>
          <span :class="['connection', { online: status?.discord_connected }]">
            <i></i> Discord
          </span>
        </div>
      </header>

      <div v-if="fatalError" class="alert">
        <AlertTriangle :size="19" />
        <div><strong>LINK FAILURE</strong><span>{{ fatalError }}</span></div>
      </div>

      <section v-if="tab === 'overview'" class="view overview-view">
        <div class="hero-panel panel">
          <div class="hero-grid">
            <div>
              <span class="panel-code">LIVE / 8111</span>
              <div class="phase" :class="snapshot?.phase">{{ gameLabel }}</div>
              <h2>{{ snapshot?.vehicle?.display_name ?? "Awaiting telemetry" }}</h2>
              <p>{{ snapshot?.mode ?? "Launch War Thunder to establish a local link." }}</p>
            </div>
            <Crosshair class="crosshair" :size="114" :stroke-width="0.75" />
          </div>
          <div class="telemetry-strip">
            <div v-for="item in telemetryItems" :key="item[0]">
              <span>{{ item[0] }}</span><strong>{{ item[1] }}</strong><small>{{ item[2] }}</small>
            </div>
          </div>
        </div>

        <div class="overview-columns">
          <article class="panel discord-card">
            <div class="panel-heading">
              <span><Activity :size="18" /> Discord output</span>
              <b>{{ status?.discord_connected ? "TRANSMITTING" : "STANDBY" }}</b>
            </div>
            <div class="discord-preview compact">
              <div class="presence-art"><Plane :size="40" /></div>
              <div>
                <span>PLAYING WAR THUNDER</span>
                <strong>{{ livePresence?.details ?? "No activity published" }}</strong>
                <p>{{ livePresence?.state ?? "Discord is waiting for a link." }}</p>
              </div>
            </div>
          </article>

          <article class="panel route-card">
            <div class="panel-heading"><span><Database :size="18" /> Data route</span></div>
            <div class="data-route">
              <div><b>01</b><span>War Thunder</span><small>127.0.0.1:8111</small></div>
              <i>→</i>
              <div><b>02</b><span>WT Presence</span><small>in memory</small></div>
              <i>→</i>
              <div><b>03</b><span>Discord IPC</span><small>local socket</small></div>
            </div>
          </article>
        </div>
      </section>

      <section v-else-if="tab === 'presence' && settings && activePreset" class="view presence-view">
        <div class="editor panel">
          <div class="panel-heading">
            <span><Settings2 :size="18" /> Activity template</span>
            <select v-model="settings.active_preset_id">
              <option v-for="preset in settings.presets" :key="preset.id" :value="preset.id">
                {{ preset.name }}
              </option>
            </select>
          </div>

          <label>
            <span>DETAILS LINE</span>
            <input v-model="activePreset.details_template" spellcheck="false" />
            <small>Example: <code v-pre>{{ vehicle.name }} · {{ game.mode }}</code></small>
          </label>
          <label>
            <span>STATE LINE</span>
            <input v-model="activePreset.state_template" spellcheck="false" />
            <small>Example: <code v-pre>{{ telemetry.ias | round }} km/h</code></small>
          </label>
          <label>
            <span>ARTWORK</span>
            <select v-model="activePreset.large_image">
              <option v-for="artwork in artworkChoices" :key="artwork.label" :value="artwork.value">
                {{ artwork.label }}
              </option>
            </select>
            <small>Automatic follows the game phase and vehicle type.</small>
          </label>
          <label class="switch-line">
            <input v-model="activePreset.show_elapsed" type="checkbox" />
            <span>Show elapsed sortie time</span>
          </label>
          <button class="save-button" :disabled="saveState === 'saving'" @click="save">
            <Check v-if="saveState === 'saved'" :size="18" />
            <Save v-else :size="18" />
            {{ saveState === "saved" ? "SAVED" : saveState === "saving" ? "SAVING" : "SAVE CONFIGURATION" }}
          </button>
        </div>

        <aside class="preview-column">
          <div class="preview-heading">
            <div>
              <span class="eyebrow">PREVIEW LAB</span>
              <small>TEST SIGNAL</small>
            </div>
            <div class="scenario-rail" role="group" aria-label="Preview scenario">
              <button
                v-for="scenario in previewScenarios"
                :key="scenario.value"
                type="button"
                :class="{ active: previewScenario === scenario.value }"
                :aria-pressed="previewScenario === scenario.value"
                @click="previewScenario = scenario.value"
              >
                {{ scenario.label }}
              </button>
            </div>
          </div>
          <div class="discord-preview large">
            <div class="presence-art"><Plane :size="58" /></div>
            <div>
              <span>PLAYING WAR THUNDER</span>
              <strong>{{ preview?.details ?? "—" }}</strong>
              <p>{{ preview?.state ?? "—" }}</p>
              <small v-if="preview?.started_at">elapsed time enabled</small>
            </div>
          </div>
          <div v-if="previewError" class="inline-error">{{ previewError }}</div>
          <div class="variable-list">
            <strong>AVAILABLE SIGNALS</strong>
            <code>vehicle.name</code><code>game.mode</code><code>game.map</code>
            <code>telemetry.ias</code><code>telemetry.agl</code><code>session.kills</code>
          </div>
          <div class="discord-guidance">
            <strong>DISCORD SETUP</strong>
            <p>Keep Discord Desktop running. WT Presence connects automatically.</p>
            <p>To show only the WT Presence card, disable War Thunder in Discord under <b>User Settings → Registered Games</b>. Change this setting once in Discord.</p>
          </div>
        </aside>
      </section>

      <section v-else-if="tab === 'sessions'" class="view sessions-view">
        <div class="section-title">
          <div><span class="eyebrow">LOCAL FLIGHT LOG</span><h2>Recent sessions</h2></div>
          <span>{{ sessions.length }} STORED</span>
        </div>
        <div v-if="sessions.length" class="session-list">
          <article v-for="entry in sessions" :key="entry.id" class="session-row">
            <div><span>STARTED</span><strong>{{ date(entry.summary.started_at) }}</strong></div>
            <div><span>BATTLES</span><strong>{{ entry.summary.completed_battles.length }}</strong></div>
            <div><span>KILLS</span><strong>{{ entry.summary.kills }}</strong></div>
            <div><span>DEATHS</span><strong>{{ entry.summary.deaths }}</strong></div>
            <code>{{ entry.id.slice(0, 8) }}</code>
          </article>
        </div>
        <div v-else class="empty-state"><History :size="42" /><strong>No sorties logged yet</strong><span>Session history stays on this machine.</span></div>
      </section>

      <section v-else class="view diagnostics-view">
        <div class="diagnostic-grid">
          <article class="panel">
            <div class="panel-heading"><span><RadioTower :size="18" /> Connections</span></div>
            <dl>
              <div><dt>Telemetry</dt><dd>{{ status?.telemetry_connected ? "Connected" : "Unavailable" }}</dd></div>
              <div><dt>Discord IPC</dt><dd>{{ status?.discord_connected ? "Connected" : "Unavailable" }}</dd></div>
              <div><dt>Agent version</dt><dd>v{{ status?.version ?? "—" }}</dd></div>
              <div><dt>Last update</dt><dd>{{ status ? date(status.updated_at) : "—" }}</dd></div>
            </dl>
            <div class="discord-guidance">
              <strong>ONE VISIBLE ACTIVITY</strong>
              <p>Disable War Thunder under Discord's <b>User Settings → Registered Games</b> if you want WT Presence to be the only visible card. Change this setting once in Discord.</p>
            </div>
          </article>
          <article class="panel">
            <div class="panel-heading"><span><ShieldCheck :size="18" /> Privacy boundary</span></div>
            <ul class="check-list">
              <li><Check :size="16" /> Reads only War Thunder's localhost telemetry</li>
              <li><Check :size="16" /> Talks to Discord through its local IPC socket</li>
              <li><Check :size="16" /> Stores settings and sessions on this device</li>
              <li><Check :size="16" /> Requires no Gaijin credentials</li>
            </ul>
          </article>
        </div>
        <div v-if="status?.last_error" class="log-line">
          <AlertTriangle :size="18" /><span>LAST ERROR</span><code>{{ status.last_error }}</code>
        </div>
      </section>
    </main>
  </div>
</template>
