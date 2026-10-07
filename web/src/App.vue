<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { api } from "./api";
import HelpLabel from "./HelpLabel.vue";
import TemplateLine from "./TemplateLine.vue";
import { translate, type MessageKey } from "./i18n";
import { diagnosticReportText, preferredLocale, type Locale } from "./ui";
import type {
  AppSettings,
  DiscordActivity,
  GameSnapshot,
  PreviewScenario,
  RuntimeStatus,
  StoredSession,
} from "./types";

type Tab = "presence" | "overview" | "sessions" | "diagnostics";
const tabs: Tab[] = ["presence", "overview", "sessions", "diagnostics"];
const tab = ref<Tab>("presence");
const locale = ref<Locale>(preferredLocale(navigator.languages));
try {
  const saved = localStorage.getItem("wt-presence-language");
  if (saved === "ru" || saved === "en") locale.value = saved;
} catch {
  /* Browser storage can be disabled. */
}
const t = (key: MessageKey) => translate(locale.value, key);
watch(
  locale,
  (value) => {
    document.documentElement.lang = value;
    try {
      localStorage.setItem("wt-presence-language", value);
    } catch {
      /* Optional preference. */
    }
  },
  { immediate: true },
);

const status = ref<RuntimeStatus | null>(null);
const snapshot = ref<GameSnapshot | null>(null);
const settings = ref<AppSettings | null>(null);
const livePresence = ref<DiscordActivity | null>(null);
const preview = ref<DiscordActivity | null>(null);
const scenario = ref<PreviewScenario>("air");
const sessions = ref<StoredSession[]>([]);
const savedSettings = ref("");
const fatalError = ref(false);
const saveError = ref(false);
const previewError = ref(false);
const saving = ref(false);
const copied = ref(false);
const copyFallback = ref("");
const restartNeeded = ref(false);
const loading = ref(true);
let initialConnection = "";
let pollingTimer: number | undefined;
let previewTimer: number | undefined;
let previewRevision = 0;
let disposed = false;

const activePreset = computed(() =>
  settings.value?.presets.find(
    (p) => p.id === settings.value?.active_preset_id,
  ),
);
const dirty = computed(
  () =>
    settings.value !== null &&
    JSON.stringify(settings.value) !== savedSettings.value,
);
const startup = computed({
  get: () =>
    settings.value?.open_dashboard_on_start && !settings.value.start_minimized
      ? "open"
      : "tray",
  set: (value) => {
    if (!settings.value) return;
    settings.value.open_dashboard_on_start = value === "open";
    settings.value.start_minimized = value === "tray";
  },
});
const artwork = [
  "auto",
  "default",
  "air",
  "ground",
  "naval",
  "hangar",
] as const;
const scenarios: PreviewScenario[] = ["live", "air", "ground", "hangar"];
const customArtwork = computed(
  () =>
    activePreset.value?.large_image &&
    !artwork.some((a) => "presence-" + a === activePreset.value?.large_image),
);
const phaseLabel = computed(() =>
  t(
    ("overview." +
      (snapshot.value?.phase === "offline" || !snapshot.value
        ? "waiting"
        : snapshot.value.phase)) as MessageKey,
  ),
);
const diagnosticHint = computed(() =>
  !status.value?.telemetry_connected
    ? "diagnostics.launchWt"
    : !status.value.discord_connected
      ? "diagnostics.launchDiscord"
      : "diagnostics.ready",
);
const connectionKey = (value: AppSettings) =>
  JSON.stringify([value.telemetry_url, value.dashboard_port]);
function date(value?: string | null): string {
  return value
    ? new Intl.DateTimeFormat(locale.value, {
        dateStyle: "medium",
        timeStyle: "short",
      }).format(new Date(value))
    : t("common.never");
}
function number(value?: number | null): string {
  return value == null ? "—" : Math.round(value).toLocaleString(locale.value);
}

async function refreshRuntime() {
  try {
    const [nextStatus, nextSnapshot, presence] = await Promise.all([
      api.status(),
      api.snapshot(),
      api.presence(),
    ]);
    status.value = nextStatus;
    snapshot.value = nextSnapshot;
    livePresence.value = presence;
    if (tab.value === "sessions") sessions.value = await api.sessions();
    fatalError.value = false;
  } catch {
    fatalError.value = true;
  }
}
async function poll() {
  if (!document.hidden) await refreshRuntime();
  if (!disposed) pollingTimer = window.setTimeout(poll, 2500);
}
async function load() {
  loading.value = true;
  try {
    const loaded = await api.settings();
    settings.value = loaded;
    savedSettings.value = JSON.stringify(loaded);
    initialConnection = connectionKey(loaded);
    await refreshRuntime();
  } catch {
    fatalError.value = true;
  } finally {
    loading.value = false;
  }
}
async function refreshPreview(revision: number) {
  if (!activePreset.value) return;
  try {
    const activity = await api.preview(activePreset.value, scenario.value);
    if (revision !== previewRevision || disposed) return;
    preview.value = activity;
    previewError.value = false;
  } catch {
    if (revision === previewRevision) {
      preview.value = null;
      previewError.value = true;
    }
  }
}
watch(
  () => [
    JSON.stringify(activePreset.value),
    scenario.value,
    scenario.value === "live" ? snapshot.value?.captured_at : null,
  ],
  () => {
    window.clearTimeout(previewTimer);
    const revision = ++previewRevision;
    previewTimer = window.setTimeout(() => refreshPreview(revision), 220);
  },
);
watch(tab, () => {
  void refreshRuntime();
});
async function save() {
  if (!settings.value || saving.value) return;
  saving.value = true;
  saveError.value = false;
  try {
    const saved = await api.saveSettings(settings.value);
    settings.value = saved;
    savedSettings.value = JSON.stringify(saved);
    restartNeeded.value = connectionKey(saved) !== initialConnection;
  } catch {
    saveError.value = true;
  } finally {
    saving.value = false;
  }
}
async function copyReport() {
  try {
    const current = await api.diagnostics();
    const text = diagnosticReportText(current);
    try {
      await navigator.clipboard.writeText(text);
      copied.value = true;
      copyFallback.value = "";
    } catch {
      copyFallback.value = text;
    }
  } catch {
    fatalError.value = true;
  }
}
function beforeLeave(event: BeforeUnloadEvent) {
  if (dirty.value) {
    event.preventDefault();
    event.returnValue = "";
  }
}
onMounted(async () => {
  window.addEventListener("beforeunload", beforeLeave);
  await load();
  if (!disposed) void poll();
});
onBeforeUnmount(() => {
  disposed = true;
  window.clearTimeout(pollingTimer);
  window.clearTimeout(previewTimer);
  window.removeEventListener("beforeunload", beforeLeave);
});
</script>

<template>
  <div class="app-shell">
    <header class="app-header">
      <a class="wordmark" href="#" @click.prevent="tab = 'presence'"
        >WT Presence<span>Discord · War Thunder</span></a
      >
      <div class="language-control">
        <HelpLabel
          for-id="language"
          :label="t('language.label')"
          :help="t('language.help')"
        />
        <select id="language" v-model="locale">
          <option value="en">English</option>
          <option value="ru">Русский</option>
        </select>
      </div>
    </header>
    <nav :aria-label="t('app.name')">
      <button
        v-for="item in tabs"
        :key="item"
        :class="{ active: tab === item }"
        :aria-current="tab === item ? 'page' : undefined"
        @click="tab = item"
      >
        {{ t(`nav.${item}`) }}
      </button>
    </nav>
    <main>
      <div v-if="fatalError" class="notice error" role="alert">
        {{ t("error.connection") }}
        <button type="button" @click="settings ? refreshRuntime() : load()">
          {{ t("common.retry") }}
        </button>
      </div>
      <p v-if="loading" role="status">{{ t("common.loading") }}</p>
      <section
        v-if="tab === 'presence' && settings && activePreset"
        class="settings-layout"
      >
        <form @submit.prevent="save">
          <h1>{{ t("presence.title") }}</h1>
          <p class="intro">{{ t("presence.description") }}</p>
          <fieldset :disabled="saving">
            <div v-if="settings.presets.length > 1" class="field">
              <HelpLabel
                for-id="profile"
                :label="t('presence.preset')"
                :help="t('presence.presetHelp')"
              />
              <select id="profile" v-model="settings.active_preset_id">
                <option v-for="p in settings.presets" :key="p.id" :value="p.id">
                  {{
                    p.id === "minimal" && p.name === "Minimal"
                      ? t("common.savedProfile")
                      : p.name
                  }}
                </option>
              </select>
            </div>
            <TemplateLine
              v-model="activePreset.details_template"
              :locale="locale"
              :label="t('presence.firstLine')"
              :help="t('presence.firstLineHelp')"
              :disabled="saving"
            />
            <TemplateLine
              v-model="activePreset.state_template"
              :locale="locale"
              :label="t('presence.secondLine')"
              :help="t('presence.secondLineHelp')"
              :disabled="saving"
            />
            <p class="hint">{{ t("presence.valuesNote") }}</p>
            <div class="field">
              <HelpLabel
                for-id="artwork"
                :label="t('presence.artwork')"
                :help="t('presence.artworkHelp')"
              />
              <select id="artwork" v-model="activePreset.large_image">
                <option
                  v-for="a in artwork"
                  :key="a"
                  :value="a === 'auto' ? null : `presence-${a}`"
                >
                  {{ t(`presence.artwork.${a}`) }}
                </option>
                <option v-if="customArtwork" :value="activePreset.large_image">
                  {{ t("common.custom") }} · {{ activePreset.large_image }}
                </option>
              </select>
            </div>
            <div class="toggle-row">
              <HelpLabel
                for-id="elapsed"
                :label="t('presence.elapsed')"
                :help="t('presence.elapsedHelp')"
              /><input
                id="elapsed"
                v-model="activePreset.show_elapsed"
                type="checkbox"
              />
            </div>
            <h2>{{ t("settings.application") }}</h2>
            <div class="field">
              <HelpLabel
                for-id="startup"
                :label="t('settings.startup')"
                :help="t('settings.startupHelp')"
              /><select id="startup" v-model="startup">
                <option value="open">{{ t("settings.startupOpen") }}</option>
                <option value="tray">{{ t("settings.startupTray") }}</option>
              </select>
            </div>
            <details class="advanced">
              <summary>{{ t("settings.advanced") }}</summary>
              <p class="hint">{{ t("settings.advancedHelp") }}</p>
              <div class="field">
                <HelpLabel
                  for-id="telemetry"
                  :label="t('settings.telemetryUrl')"
                  :help="t('settings.telemetryUrlHelp')"
                /><input
                  id="telemetry"
                  v-model="settings.telemetry_url"
                  type="url"
                  required
                />
              </div>
              <div class="field">
                <HelpLabel
                  for-id="port"
                  :label="t('settings.dashboardPort')"
                  :help="t('settings.dashboardPortHelp')"
                /><input
                  id="port"
                  v-model.number="settings.dashboard_port"
                  type="number"
                  min="1"
                  max="65535"
                  required
                />
              </div>
            </details>
          </fieldset>
          <div class="save-bar">
            <button class="primary" :disabled="saving || !dirty" type="submit">
              {{ t(saving ? "save.saving" : "save.changes") }}</button
            ><span role="status">{{
              t(dirty ? "save.unsaved" : "save.saved")
            }}</span>
          </div>
          <p v-if="saveError" class="notice error" role="alert">
            {{ t("save.error") }}
          </p>
          <p v-if="restartNeeded" class="notice" role="status">
            {{ t("save.restart") }}
          </p>
        </form>
        <aside class="preview-column">
          <h2>
            <HelpLabel
              :label="t('presence.preview')"
              :help="t('presence.previewHelp')"
            />
          </h2>
          <div
            class="scenario-picker"
            role="group"
            :aria-label="t('presence.preview')"
          >
            <button
              v-for="s in scenarios"
              :key="s"
              :aria-pressed="scenario === s"
              :class="{ selected: scenario === s }"
              @click="scenario = s"
            >
              {{ t(`presence.scenario.${s}`) }}
            </button>
          </div>
          <div class="discord-preview">
            <span class="app-name">WT Presence</span
            ><strong>{{ preview?.details || "—" }}</strong
            ><span>{{ preview?.state || "—" }}</span
            ><small v-if="preview?.started_at">{{
              t("presence.elapsedPreview")
            }}</small>
          </div>
          <p class="hint">
            {{ t(scenario === "live" ? "presence.live" : "presence.sample")
            }}<br />{{ t("presence.textPreview") }}
          </p>
          <p v-if="previewError" class="notice error" role="alert">
            {{ t("error.preview") }}
          </p>
          <div class="setup-note">
            <h3>Discord</h3>
            <p>{{ t("presence.discordTip") }}</p>
          </div>
        </aside>
      </section>
      <section v-else-if="tab === 'overview'">
        <h1>{{ t("overview.title") }}</h1>
        <p class="intro">{{ phaseLabel }}</p>
        <div class="overview-grid">
          <article class="panel">
            <h2>
              {{ snapshot?.vehicle?.display_name || t("overview.waiting") }}
            </h2>
            <p>{{ snapshot?.mode || t("overview.waitingHelp") }}</p>
            <dl>
              <div>
                <dt>{{ t("overview.speed") }}</dt>
                <dd>
                  {{
                    number(
                      snapshot?.telemetry.speed_ground_kph ??
                        snapshot?.telemetry.speed_ias_kph,
                    )
                  }}
                  km/h
                </dd>
              </div>
              <div>
                <dt>{{ t("overview.altitude") }}</dt>
                <dd>{{ number(snapshot?.telemetry.altitude_agl_m) }} m</dd>
              </div>
              <div>
                <dt>{{ t("overview.crew") }}</dt>
                <dd>{{ number(snapshot?.telemetry.crew_current) }}</dd>
              </div>
            </dl>
          </article>
          <article class="panel">
            <h2>{{ t("overview.discordPreview") }}</h2>
            <div class="discord-preview">
              <span class="app-name">WT Presence</span
              ><strong>{{
                livePresence?.details || t("overview.noPresence")
              }}</strong
              ><span>{{ livePresence?.state || "—" }}</span>
            </div>
          </article>
        </div>
      </section>
      <section v-else-if="tab === 'sessions'">
        <h1>{{ t("sessions.title") }}</h1>
        <p class="intro">{{ t("sessions.local") }}</p>
        <p class="notice">{{ t("sessions.limits") }}</p>
        <table v-if="sessions.length">
          <thead>
            <tr>
              <th>{{ t("sessions.started") }}</th>
              <th>{{ t("sessions.battles") }}</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="entry in sessions" :key="entry.id">
              <td>{{ date(entry.summary.started_at) }}</td>
              <td>{{ entry.summary.completed_battles.length }}</td>
            </tr>
          </tbody>
        </table>
        <p v-else class="empty-state">{{ t("sessions.empty") }}</p>
      </section>
      <section v-else-if="tab === 'diagnostics'" class="diagnostics">
        <h1>{{ t("diagnostics.title") }}</h1>
        <p class="intro">{{ t("diagnostics.description") }}</p>
        <p class="notice">{{ t(diagnosticHint) }}</p>
        <dl>
          <div>
            <dt>
              <HelpLabel
                :label="t('diagnostics.telemetry')"
                :help="t('diagnostics.telemetryHelp')"
              />
            </dt>
            <dd>
              {{
                t(
                  status?.telemetry_connected
                    ? "connection.connected"
                    : "connection.unavailable",
                )
              }}
            </dd>
          </div>
          <div>
            <dt>
              <HelpLabel
                :label="t('diagnostics.discord')"
                :help="t('diagnostics.discordHelp')"
              />
            </dt>
            <dd>
              {{
                t(
                  status?.discord_connected
                    ? "connection.connected"
                    : !status?.telemetry_connected
                      ? "diagnostics.idle"
                      : "connection.unavailable",
                )
              }}
            </dd>
          </div>
          <div>
            <dt>
              <HelpLabel
                :label="t('diagnostics.lastTelemetry')"
                :help="t('diagnostics.lastTelemetryHelp')"
              />
            </dt>
            <dd>{{ date(status?.last_telemetry_at) }}</dd>
          </div>
          <div>
            <dt>
              <HelpLabel
                :label="t('diagnostics.lastDiscord')"
                :help="t('diagnostics.lastDiscordHelp')"
              />
            </dt>
            <dd>{{ date(status?.last_discord_at) }}</dd>
          </div>
          <div>
            <dt>{{ t("diagnostics.started") }}</dt>
            <dd>{{ date(status?.started_at) }}</dd>
          </div>
          <div>
            <dt>{{ t("diagnostics.version") }}</dt>
            <dd>{{ status?.version || "—" }}</dd>
          </div>
        </dl>
        <details v-if="status?.last_error" class="error-details">
          <summary>{{ t("diagnostics.details") }}</summary>
          <pre>{{ status.last_error }}</pre>
        </details>
        <button class="primary" type="button" @click="copyReport">
          {{ t(copied ? "diagnostics.copied" : "diagnostics.copy") }}
        </button>
        <p class="hint">{{ t("diagnostics.copyHelp") }}</p>
        <div v-if="copyFallback">
          <p role="status">{{ t("error.copy") }}</p>
          <textarea
            :value="copyFallback"
            readonly
            :aria-label="t('diagnostics.copy')"
            rows="12"
            @focus="($event.target as HTMLTextAreaElement).select()"
          />
        </div>
        <h2>{{ t("diagnostics.logs") }}</h2>
        <p class="hint">{{ t("diagnostics.logsHelp") }}</p>
      </section>
    </main>
    <footer>
      <span>{{ t("app.local") }}</span
      ><span :class="{ connected: !fatalError && status?.telemetry_connected }"
        >● War Thunder</span
      ><span :class="{ connected: !fatalError && status?.discord_connected }"
        >● Discord</span
      >
    </footer>
  </div>
</template>
