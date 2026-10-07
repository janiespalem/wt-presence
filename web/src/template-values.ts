import type { MessageKey } from "./i18n";

export const templateValues: { key: MessageKey; expression: string }[] = [
  {
    key: "variable.vehicle",
    expression:
      "{% if vehicle %}{{ vehicle.name }}{% else %}War Thunder{% endif %}",
  },
  { key: "variable.phase", expression: "{{ game.phase_label }}" },
  { key: "variable.mode", expression: "{{ game.mode or '—' }}" },
  { key: "variable.map", expression: "{{ game.map or '—' }}" },
  {
    key: "variable.speed",
    expression:
      "{% if telemetry.ground_speed is not none %}{{ telemetry.ground_speed | round }} km/h{% elif telemetry.ias is not none %}{{ telemetry.ias | round }} km/h{% else %}—{% endif %}",
  },
  {
    key: "variable.altitude",
    expression:
      "{% if telemetry.agl is not none %}{{ telemetry.agl | round }} m{% else %}—{% endif %}",
  },
  {
    key: "variable.crew",
    expression:
      "{{ telemetry.crew_current if telemetry.crew_current is not none else '—' }}",
  },
];

export function friendlyTemplate(
  source: string,
  label: (key: MessageKey) => string,
): string {
  for (const value of templateValues)
    source = source.split(value.expression).join(`[${label(value.key)}]`);
  return source;
}

export function storedTemplate(
  source: string,
  label: (key: MessageKey) => string,
): string {
  for (const value of templateValues)
    source = source.split(`[${label(value.key)}]`).join(value.expression);
  return source;
}
