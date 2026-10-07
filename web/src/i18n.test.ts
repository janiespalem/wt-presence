import { describe, expect, it } from "vitest";
import { messages } from "./i18n";

describe("translations", () => {
  it("keeps Russian and English dictionaries structurally complete", () => {
    expect(Object.keys(messages.ru).sort()).toEqual(
      Object.keys(messages.en).sort(),
    );
  });
});
