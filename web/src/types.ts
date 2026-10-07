export type GamePhase = "offline" | "hangar" | "loading" | "battle";
export type PreviewScenario = "live" | "hangar" | "air" | "ground";

export interface Telemetry {
  speed_ias_kph: number | null;
  speed_tas_kph: number | null;
  altitude_agl_m: number | null;
  altitude_msl_m: number | null;
  speed_ground_kph: number | null;
  crew_current: number | null;
  crew_total: number | null;
}

export interface Vehicle {
  technical_name: string;
  display_name: string;
  kind: "aircraft" | "ground" | "naval" | "unknown";
}

export interface GameSnapshot {
  phase: GamePhase;
  battle_id: string | null;
  vehicle: Vehicle | null;
  telemetry: Telemetry;
  map: string | null;
  mode: string | null;
  captured_at: string;
}

export interface RuntimeStatus {
  version: string;
  telemetry_connected: boolean;
  discord_connected: boolean;
  started_at: string;
  last_telemetry_at: string | null;
  last_discord_at: string | null;
  last_error: string | null;
  updated_at: string;
}

export interface DiagnosticReport extends Omit<RuntimeStatus, "last_error"> {
  has_error: boolean;
  platform: string;
  architecture: string;
  phase: GamePhase;
}

export interface PresencePreset {
  id: string;
  name: string;
  details_template: string;
  state_template: string;
  large_image: string | null;
  small_image: string | null;
  show_elapsed: boolean;
}

export interface AppSettings {
  schema_version: number;
  telemetry_url: string;
  dashboard_port: number;
  active_preset_id: string;
  presets: PresencePreset[];
  open_dashboard_on_start: boolean;
  start_minimized: boolean;
}

export interface DiscordActivity {
  details: string | null;
  state: string | null;
  large_image: string | null;
  small_image: string | null;
  started_at: number | null;
}

export interface StoredSession {
  id: string;
  summary: {
    started_at: string;
    updated_at: string;
    kills: number;
    deaths: number;
    completed_battles: unknown[];
  };
}
