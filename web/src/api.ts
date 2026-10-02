import type {
  AppSettings,
  DiscordActivity,
  GameSnapshot,
  PresencePreset,
  PreviewScenario,
  RuntimeStatus,
  StoredSession,
} from "./types";

const tokenKey = "wt-presence-token";

function processToken(): string {
  const fragment = new URLSearchParams(window.location.hash.slice(1));
  const incoming = fragment.get("token");
  if (incoming) {
    sessionStorage.setItem(tokenKey, incoming);
    history.replaceState(null, "", `${location.pathname}${location.search}`);
    return incoming;
  }
  return sessionStorage.getItem(tokenKey) ?? "";
}

const token = processToken();

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(path, {
    ...init,
    headers: {
      "x-wt-presence-token": token,
      ...(init?.body ? { "content-type": "application/json" } : {}),
      ...init?.headers,
    },
  });
  const body = await response.json().catch(() => null);
  if (!response.ok) {
    throw new Error(body?.error?.message ?? `Request failed (${response.status})`);
  }
  return body as T;
}

export const api = {
  status: () => request<RuntimeStatus>("/api/v1/status"),
  snapshot: () => request<GameSnapshot>("/api/v1/snapshot"),
  settings: () => request<AppSettings>("/api/v1/settings"),
  presence: () => request<DiscordActivity | null>("/api/v1/presence/state"),
  sessions: () => request<StoredSession[]>("/api/v1/sessions?limit=20"),
  preview: (preset: PresencePreset, scenario: PreviewScenario) =>
    request<DiscordActivity>("/api/v1/preview", {
      method: "POST",
      body: JSON.stringify({ preset, scenario }),
    }),
  saveSettings: (settings: AppSettings) =>
    request<AppSettings>("/api/v1/settings", {
      method: "PUT",
      body: JSON.stringify(settings),
    }),
};
