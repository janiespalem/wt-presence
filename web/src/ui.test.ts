import { describe, expect, it } from "vitest";
import type { DiagnosticReport } from "./types";
import {
  diagnosticReportText,
  insertTemplateValue,
  preferredLocale,
} from "./ui";

describe("preferredLocale", () => {
  it("uses Russian only when the browser advertises a Russian locale", () => {
    expect(preferredLocale(["ru-RU", "en-US"])).toBe("ru");
    expect(preferredLocale(["pl-PL", "en-US"])).toBe("en");
  });
});

describe("insertTemplateValue", () => {
  it("replaces the selected text and returns the next cursor position", () => {
    expect(
      insertTemplateValue("Speed: old km/h", 7, 10, "{{ telemetry.ias }}"),
    ).toEqual({
      value: "Speed: {{ telemetry.ias }} km/h",
      cursor: 26,
    });
  });
});

describe("diagnosticReportText", () => {
  it("creates a stable support report from the public diagnostic fields", () => {
    const report: DiagnosticReport = {
      version: "0.1.0",
      platform: "windows",
      architecture: "x86_64",
      telemetry_connected: true,
      discord_connected: false,
      phase: "hangar",
      started_at: "2026-10-06T10:00:00Z",
      last_telemetry_at: "2026-10-06T10:05:00Z",
      last_discord_at: null,
      has_error: true,
      updated_at: "2026-10-06T10:05:01Z",
    };

    expect(diagnosticReportText(report)).toBe(
      [
        "WT Presence diagnostics",
        "Version: 0.1.0",
        "Platform: windows / x86_64",
        "Telemetry: connected",
        "Discord: unavailable",
        "Phase: hangar",
        "Started: 2026-10-06T10:00:00Z",
        "Last telemetry: 2026-10-06T10:05:00Z",
        "Last Discord publish: never",
        "Error present: yes",
        "Updated: 2026-10-06T10:05:01Z",
      ].join("\n"),
    );
  });
});
