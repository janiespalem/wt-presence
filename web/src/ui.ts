import type { DiagnosticReport } from "./types";

export type Locale = "ru" | "en";

export function preferredLocale(languages: readonly string[]): Locale {
  return languages[0]?.toLowerCase().startsWith("ru") ? "ru" : "en";
}

export function insertTemplateValue(
  source: string,
  selectionStart: number,
  selectionEnd: number,
  expression: string,
): { value: string; cursor: number } {
  return {
    value: `${source.slice(0, selectionStart)}${expression}${source.slice(selectionEnd)}`,
    cursor: selectionStart + expression.length,
  };
}

export function diagnosticReportText(report: DiagnosticReport): string {
  return [
    "WT Presence diagnostics",
    `Version: ${report.version}`,
    `Platform: ${report.platform} / ${report.architecture}`,
    `Telemetry: ${report.telemetry_connected ? "connected" : "unavailable"}`,
    `Discord: ${report.discord_connected ? "connected" : "unavailable"}`,
    `Phase: ${report.phase}`,
    `Started: ${report.started_at}`,
    `Last telemetry: ${report.last_telemetry_at ?? "never"}`,
    `Last Discord publish: ${report.last_discord_at ?? "never"}`,
    `Error present: ${report.has_error ? "yes" : "no"}`,
    `Updated: ${report.updated_at}`,
  ].join("\n");
}
