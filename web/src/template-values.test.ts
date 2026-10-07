import { describe, expect, it } from "vitest";
import { translate } from "./i18n";
import {
  friendlyTemplate,
  storedTemplate,
  templateValues,
} from "./template-values";

describe("friendly status values", () => {
  for (const locale of ["en", "ru"] as const) {
    it(`round trips values and preserves custom syntax in ${locale}`, () => {
      const label = (key: Parameters<typeof translate>[1]) =>
        translate(locale, key);
      const source =
        templateValues.map((value) => value.expression).join(" · ") +
        " {{ custom | default('test') }}";
      const friendly = friendlyTemplate(source, label);
      expect(friendly).toContain(`[${label("variable.vehicle")}]`);
      expect(storedTemplate(friendly, label)).toBe(source);
    });
  }
});
