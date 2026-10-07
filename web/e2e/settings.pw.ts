import { expect, test } from "@playwright/test";
import type { AppSettings, PresencePreset } from "../src/types";
import { templateValues } from "../src/template-values";

test("settings survive language switches, failed saves and reloads", async ({
  page,
}) => {
  let settings: AppSettings = {
    schema_version: 2,
    telemetry_url: "http://127.0.0.1:8111",
    dashboard_port: 32147,
    active_preset_id: "minimal",
    presets: [
      {
        id: "minimal",
        name: "Minimal",
        details_template: templateValues[0].expression,
        state_template: "{{ game.phase_label }}",
        large_image: null,
        small_image: null,
        show_elapsed: false,
      },
    ],
    open_dashboard_on_start: true,
    start_minimized: false,
  };
  let rejectSave = false;
  let delayAir = false;
  let writes = 0;
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  const runtime = {
    version: "0.1.0-beta.5",
    telemetry_connected: false,
    discord_connected: false,
    started_at: "2026-10-07T10:00:00Z",
    last_telemetry_at: null,
    last_discord_at: null,
    last_error: null,
    updated_at: "2026-10-07T10:00:00Z",
  };
  await page.route("**/api/v1/**", async (route) => {
    const path = new URL(route.request().url()).pathname;
    let body: unknown = null;
    if (path.endsWith("/settings")) {
      if (route.request().method() === "PUT") {
        writes++;
        if (rejectSave) {
          await route.fulfill({
            status: 500,
            json: { error: { message: "disk unavailable" } },
          });
          return;
        }
        settings = route.request().postDataJSON();
      }
      body = settings;
    } else if (path.endsWith("/status")) body = runtime;
    else if (path.endsWith("/snapshot"))
      body = {
        phase: "offline",
        vehicle: null,
        telemetry: {},
        map: null,
        mode: null,
        captured_at: runtime.updated_at,
      };
    else if (path.endsWith("/sessions")) body = [];
    else if (path.endsWith("/diagnostics"))
      body = {
        ...runtime,
        last_error: undefined,
        has_error: false,
        phase: "offline",
        platform: "windows",
        architecture: "x86_64",
      };
    else if (path.endsWith("/preview")) {
      const data = route.request().postDataJSON() as {
        scenario: string;
        preset: PresencePreset;
      };
      if (data.scenario === "air" && delayAir)
        await new Promise((resolve) => setTimeout(resolve, 700));
      body = {
        details:
          data.scenario === "ground"
            ? "T-80UD"
            : data.scenario === "hangar"
              ? "War Thunder"
              : "J-7D",
        state: "In battle",
        started_at: data.preset.show_elapsed ? 1791367200 : null,
      };
    }
    await route.fulfill({ json: body });
  });
  await page.goto("/#token=browser-test-token");
  await expect(page).toHaveURL("http://127.0.0.1:4173/");
  await page.locator("#language").selectOption("ru");
  const first = page.getByRole("textbox", {
    name: "Первая строка в Discord",
    exact: true,
  });
  await expect(first).toHaveValue("[Название техники]");
  await page
    .getByRole("button", { name: "Первая строка в Discord", exact: true })
    .focus();
  await expect(
    page.getByRole("tooltip").filter({ hasText: "Главный текст" }),
  ).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(
    page.getByRole("tooltip").filter({ hasText: "Главный текст" }),
  ).toBeHidden();
  await first.fill("Лечу на [Название техники] · ");
  await page
    .locator(".insert-row select")
    .first()
    .selectOption("Состояние игры");
  await expect(first).toHaveValue(
    "Лечу на [Название техники] · [Состояние игры]",
  );
  await page.locator("#language").selectOption("en");
  await expect(
    page.getByRole("textbox", { name: "First line in Discord", exact: true }),
  ).toHaveValue("Лечу на [Vehicle name] · [Game status]");
  await page.locator("#startup").selectOption("tray");
  await page.locator("#elapsed").check();
  await page.getByText("Advanced settings", { exact: true }).click();
  await page.locator("#port").fill("32148");
  rejectSave = true;
  await page.getByRole("button", { name: "Save changes", exact: true }).click();
  await expect(page.getByRole("alert")).toContainText("Could not save");
  await expect(
    page.getByRole("textbox", { name: "First line in Discord", exact: true }),
  ).toHaveValue("Лечу на [Vehicle name] · [Game status]");
  rejectSave = false;
  await page.getByRole("button", { name: "Save changes", exact: true }).click();
  await expect(
    page.getByRole("button", { name: "Save changes", exact: true }),
  ).toBeDisabled();
  expect(settings.presets[0].details_template).toContain(
    "{{ game.phase_label }}",
  );
  expect(settings.presets[0].details_template).not.toContain("[Vehicle name]");
  expect(settings.start_minimized).toBe(true);
  expect(settings.open_dashboard_on_start).toBe(false);
  await expect(
    page.getByText("Restart WT Presence to apply connection changes.", {
      exact: false,
    }),
  ).toBeVisible();
  expect(writes).toBe(2);
  await page.reload();
  await expect(page.locator("#language")).toHaveValue("en");
  await expect(page.locator("#elapsed")).toBeChecked();
  await page.getByRole("button", { name: "Ground", exact: true }).click();
  await expect(page.locator(".discord-preview strong")).toHaveText("T-80UD");
  delayAir = true;
  const airRequest = page.waitForRequest(
    (req) =>
      req.url().endsWith("/preview") && req.postDataJSON().scenario === "air",
  );
  await page.getByRole("button", { name: "Aircraft", exact: true }).click();
  await airRequest;
  await page.getByRole("button", { name: "Ground", exact: true }).click();
  await expect(page.locator(".discord-preview strong")).toHaveText("T-80UD");
  await page.waitForTimeout(800);
  await expect(page.locator(".discord-preview strong")).toHaveText("T-80UD");
  await page.locator("#language").selectOption("ru");
  if (process.env.WT_SCREENSHOT_DIR)
    await page.screenshot({
      path: process.env.WT_SCREENSHOT_DIR + "/settings-ru.png",
      fullPage: true,
    });
  await page.setViewportSize({ width: 390, height: 844 });
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= window.innerWidth,
    ),
  ).toBe(true);
  await page.getByRole("button", { name: "Диагностика", exact: true }).click();
  await page.evaluate(() =>
    Object.defineProperty(navigator, "clipboard", {
      value: undefined,
      configurable: true,
    }),
  );
  await page
    .getByRole("button", { name: "Скопировать отчёт", exact: true })
    .click();
  await expect(page.locator("textarea")).toHaveValue(/WT Presence diagnostics/);
  expect(await page.locator("textarea").inputValue()).not.toContain(
    "browser-test-token",
  );
  expect(errors).toEqual([]);
});
