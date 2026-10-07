import type { Locale } from "./ui";

const en = {
  "common.choose": "Choose…",
  "common.retry": "Try again",
  "common.loading": "Loading…",
  "common.custom": "Custom",
  "common.refresh": "Refresh",
  "common.savedProfile": "Default",
  "variable.phase": "Game status",
  "language.help":
    "Changes this page's language. Your Discord text stays as you wrote it.",
  "save.restart":
    "Restart WT Presence to apply connection changes. Reopen settings from the tray afterwards.",
  "save.error": "Could not save. Your changes are still here; try again.",
  "error.connection":
    "Cannot reach WT Presence. Open settings from its tray icon to reconnect.",
  "error.preview":
    "This text could not be rendered. Check your values and keep each Discord line under 128 characters.",
  "error.copy": "Clipboard unavailable. Select and copy the report below.",
  "presence.sample": "Sample data · nothing is sent to Discord",
  "presence.live": "Current game data",
  "presence.elapsedPreview": "Elapsed time enabled",
  "presence.textPreview": "Text preview. The image is set in Discord.",
  "presence.valuesNote":
    "Values in brackets update from the game. Unavailable data appears as —. Existing custom template code is preserved.",
  "sessions.limits":
    "Detected sessions and battles only. Kills and deaths are not tracked yet.",
  "diagnostics.idle": "Waiting for game data before publishing to Discord.",
  "diagnostics.errorPresent":
    "An error is present. Check the local details below.",
  "diagnostics.details":
    "Local error details (may contain private information)",
  "settings.startup": "When WT Presence starts",
  "settings.startupHelp":
    "Choose whether to open settings or stay in the tray. This does not enable automatic startup with Windows.",
  "settings.startupOpen": "Open settings",
  "settings.startupTray": "Stay in the tray",
  "app.name": "WT Presence",
  "app.local": "Runs locally on this computer",
  "nav.overview": "Overview",
  "nav.presence": "Discord status",
  "nav.sessions": "Sessions",
  "nav.diagnostics": "Diagnostics",
  "language.label": "Language",
  "connection.wt": "War Thunder",
  "connection.discord": "Discord",
  "connection.connected": "Connected",
  "connection.unavailable": "Unavailable",
  "overview.title": "Current activity",
  "overview.waiting": "Waiting for War Thunder",
  "overview.waitingHelp": "Start War Thunder and enter the hangar or a battle.",
  "overview.hangar": "In hangar",
  "overview.loading": "Loading",
  "overview.battle": "In battle",
  "overview.speed": "Speed",
  "overview.altitude": "Altitude",
  "overview.crew": "Crew",
  "overview.discordPreview": "Discord preview",
  "overview.noPresence": "Nothing is being sent yet",
  "presence.title": "Discord status",
  "presence.description": "Choose what your friends see while you play.",
  "presence.preset": "Profile",
  "presence.presetHelp": "Select the status profile you want to edit and use.",
  "presence.firstLine": "First line in Discord",
  "presence.firstLineHelp":
    "The main text shown under WT Presence. Add live game values with the button below.",
  "presence.secondLine": "Second line in Discord",
  "presence.secondLineHelp": "Extra context such as speed, map, or game mode.",
  "presence.addValue": "Add live value",
  "presence.addValueHelp":
    "Choose a value and WT Presence will insert it at the cursor. You do not need to type template code.",
  "presence.artwork": "Image",
  "presence.artworkHelp":
    "Automatic follows the current vehicle and game state.",
  "presence.elapsed": "Show elapsed time",
  "presence.elapsedHelp":
    "Shows how long the current WT Presence session has been running.",
  "presence.preview": "Preview",
  "presence.previewHelp":
    "Preview uses sample data and works without War Thunder or Discord.",
  "presence.scenario.live": "Live",
  "presence.scenario.air": "Aircraft",
  "presence.scenario.ground": "Ground",
  "presence.scenario.hangar": "Hangar",
  "presence.artwork.auto": "Automatic",
  "presence.artwork.default": "WT Presence",
  "presence.artwork.air": "Aircraft",
  "presence.artwork.ground": "Ground vehicle",
  "presence.artwork.naval": "Naval",
  "presence.artwork.hangar": "Hangar",
  "presence.discordTip":
    "Keep Discord Desktop open. To hide the original War Thunder card, disable War Thunder in Discord → User Settings → Registered Games.",
  "variable.vehicle": "Vehicle name",
  "variable.mode": "Game mode",
  "variable.map": "Map",
  "variable.speed": "Speed",
  "variable.altitude": "Altitude",
  "variable.kills": "Kills",
  "variable.crew": "Crew",
  "settings.application": "Application",
  "settings.applicationDescription": "How WT Presence behaves when it starts.",
  "settings.openDashboard": "Open this page on startup",
  "settings.openDashboardHelp":
    "Automatically opens the settings page when WT Presence starts.",
  "settings.startMinimized": "Start minimized",
  "settings.startMinimizedHelp":
    "Keeps WT Presence in the Windows tray instead of opening the page immediately.",
  "settings.advanced": "Advanced settings",
  "settings.advancedHelp":
    "Connection details for troubleshooting. Defaults work for normal installations.",
  "settings.telemetryUrl": "War Thunder telemetry address",
  "settings.telemetryUrlHelp":
    "Local address used to read game data. Leave it unchanged for a normal installation. Requires restarting WT Presence.",
  "settings.dashboardPort": "Settings page port",
  "settings.dashboardPortHelp":
    "Local port used by this page. Change only to resolve a port conflict. Requires restarting WT Presence and reopening settings from the tray.",
  "save.changes": "Save changes",
  "save.saving": "Saving…",
  "save.saved": "Saved",
  "save.unsaved": "Unsaved changes",
  "sessions.title": "Recent sessions",
  "sessions.empty": "No sessions recorded yet",
  "sessions.local": "Session history stays on this computer.",
  "sessions.started": "Started",
  "sessions.battles": "Battles",
  "sessions.kills": "Kills",
  "sessions.deaths": "Deaths",
  "diagnostics.title": "Diagnostics",
  "diagnostics.description":
    "Check the two local connections WT Presence needs.",
  "diagnostics.telemetry": "War Thunder data",
  "diagnostics.telemetryHelp":
    "WT Presence reads the game's local telemetry at 127.0.0.1. No Gaijin login is used.",
  "diagnostics.discord": "Discord connection",
  "diagnostics.discordHelp":
    "WT Presence sends activity through Discord Desktop on this computer.",
  "diagnostics.lastTelemetry": "Last War Thunder response",
  "diagnostics.lastTelemetryHelp":
    "The last time valid local game data was received.",
  "diagnostics.lastDiscord": "Last Discord update",
  "diagnostics.lastDiscordHelp":
    "Last successful write to Discord. This does not confirm how the card looks in your profile.",
  "diagnostics.started": "Started",
  "diagnostics.version": "Version",
  "diagnostics.lastError": "Last error",
  "diagnostics.noError": "No current error",
  "diagnostics.launchWt": "Start War Thunder and enter the hangar or a battle.",
  "diagnostics.launchDiscord": "Open Discord Desktop and keep it running.",
  "diagnostics.ready": "Everything needed for Rich Presence is connected.",
  "diagnostics.copy": "Copy diagnostic report",
  "diagnostics.copied": "Copied",
  "diagnostics.copyHelp":
    "Copies a safe support report without tokens, user paths, templates, or account data.",
  "diagnostics.logs": "Local logs",
  "diagnostics.logsHelp":
    "WT Presence keeps daily logs for seven days in its local application data folder.",
  "common.never": "Never",
  "common.none": "None",
  "common.help": "Help",
  "error.title": "Something went wrong",
} as const;

const ru = {
  "common.choose": "Выбрать…",
  "common.retry": "Повторить",
  "common.loading": "Загрузка…",
  "common.custom": "Своё изображение",
  "common.refresh": "Обновить",
  "common.savedProfile": "Стандартный",
  "variable.phase": "Состояние игры",
  "language.help":
    "Меняет язык этой страницы. Написанный вами текст в Discord останется прежним.",
  "save.restart":
    "Перезапустите WT Presence для применения настроек подключения. Затем откройте настройки через значок в трее.",
  "save.error":
    "Не удалось сохранить. Изменения остались на странице — попробуйте ещё раз.",
  "error.connection":
    "Нет связи с WT Presence. Откройте настройки через значок приложения в трее.",
  "error.preview":
    "Не удалось отобразить текст. Проверьте значения: в каждой строке Discord должно быть не больше 128 символов.",
  "error.copy": "Буфер обмена недоступен. Выделите и скопируйте отчёт ниже.",
  "presence.sample": "Пример · в Discord ничего не отправляется",
  "presence.live": "Текущие данные игры",
  "presence.elapsedPreview": "Счётчик времени включён",
  "presence.textPreview":
    "Предпросмотр текста. Изображение задаётся в Discord.",
  "presence.valuesNote":
    "Значения в скобках обновляются из игры. Недоступные данные отображаются как —. Старые сложные шаблоны сохраняются.",
  "sessions.limits":
    "Только обнаруженные сессии и бои. Убийства и смерти пока не отслеживаются.",
  "diagnostics.idle":
    "Ожидание игровых данных перед отправкой статуса в Discord.",
  "diagnostics.errorPresent":
    "Есть ошибка. Подробности доступны ниже на этом компьютере.",
  "diagnostics.details": "Подробности ошибки (могут содержать личные данные)",
  "settings.startup": "При запуске WT Presence",
  "settings.startupHelp":
    "Открывать настройки или оставаться в трее. Это не включает автозапуск вместе с Windows.",
  "settings.startupOpen": "Открывать настройки",
  "settings.startupTray": "Оставаться в трее",
  "app.name": "WT Presence",
  "app.local": "Работает локально на этом компьютере",
  "nav.overview": "Обзор",
  "nav.presence": "Статус Discord",
  "nav.sessions": "Сессии",
  "nav.diagnostics": "Диагностика",
  "language.label": "Язык",
  "connection.wt": "War Thunder",
  "connection.discord": "Discord",
  "connection.connected": "Подключено",
  "connection.unavailable": "Недоступно",
  "overview.title": "Текущая активность",
  "overview.waiting": "Ожидание War Thunder",
  "overview.waitingHelp": "Запустите War Thunder и войдите в ангар или бой.",
  "overview.hangar": "В ангаре",
  "overview.loading": "Загрузка",
  "overview.battle": "В бою",
  "overview.speed": "Скорость",
  "overview.altitude": "Высота",
  "overview.crew": "Экипаж",
  "overview.discordPreview": "Предпросмотр Discord",
  "overview.noPresence": "Статус пока не отправляется",
  "presence.title": "Статус Discord",
  "presence.description": "Выберите, что друзья увидят во время вашей игры.",
  "presence.preset": "Профиль",
  "presence.presetHelp":
    "Выберите профиль статуса, который хотите изменить и использовать.",
  "presence.firstLine": "Первая строка в Discord",
  "presence.firstLineHelp":
    "Главный текст под названием WT Presence. Игровые данные добавляются кнопкой ниже.",
  "presence.secondLine": "Вторая строка в Discord",
  "presence.secondLineHelp":
    "Дополнительные данные: скорость, карта или режим игры.",
  "presence.addValue": "Добавить игровое значение",
  "presence.addValueHelp":
    "Выберите значение, и WT Presence вставит его в позицию курсора. Код шаблонов печатать не нужно.",
  "presence.artwork": "Изображение",
  "presence.artworkHelp":
    "Автоматический режим выбирает изображение по текущей технике и состоянию игры.",
  "presence.elapsed": "Показывать прошедшее время",
  "presence.elapsedHelp":
    "Показывает, сколько длится текущая сессия WT Presence.",
  "presence.preview": "Предпросмотр",
  "presence.previewHelp":
    "Использует тестовые данные и работает без War Thunder и Discord.",
  "presence.scenario.live": "Сейчас",
  "presence.scenario.air": "Самолёт",
  "presence.scenario.ground": "Наземная техника",
  "presence.scenario.hangar": "Ангар",
  "presence.artwork.auto": "Автоматически",
  "presence.artwork.default": "WT Presence",
  "presence.artwork.air": "Самолёт",
  "presence.artwork.ground": "Наземная техника",
  "presence.artwork.naval": "Флот",
  "presence.artwork.hangar": "Ангар",
  "presence.discordTip":
    "Оставьте Discord Desktop открытым. Чтобы скрыть стандартную карточку War Thunder, отключите War Thunder в Discord → Настройки пользователя → Зарегистрированные игры.",
  "variable.vehicle": "Название техники",
  "variable.mode": "Режим игры",
  "variable.map": "Карта",
  "variable.speed": "Скорость",
  "variable.altitude": "Высота",
  "variable.kills": "Убийства",
  "variable.crew": "Экипаж",
  "settings.application": "Приложение",
  "settings.applicationDescription": "Как WT Presence ведёт себя при запуске.",
  "settings.openDashboard": "Открывать эту страницу при запуске",
  "settings.openDashboardHelp":
    "Автоматически открывает страницу настроек после запуска WT Presence.",
  "settings.startMinimized": "Запускать свёрнутым",
  "settings.startMinimizedHelp":
    "Оставляет WT Presence в трее Windows и не открывает страницу сразу.",
  "settings.advanced": "Дополнительные настройки",
  "settings.advancedHelp":
    "Параметры подключения для устранения проблем. Обычной установке подходят значения по умолчанию.",
  "settings.telemetryUrl": "Адрес телеметрии War Thunder",
  "settings.telemetryUrlHelp":
    "Локальный адрес для чтения данных игры. Обычно менять не нужно. После изменения перезапустите WT Presence.",
  "settings.dashboardPort": "Порт страницы настроек",
  "settings.dashboardPortHelp":
    "Меняйте, только если порт занят другой программой. После изменения перезапустите WT Presence и откройте настройки через значок в трее.",
  "save.changes": "Сохранить изменения",
  "save.saving": "Сохранение…",
  "save.saved": "Сохранено",
  "save.unsaved": "Есть несохранённые изменения",
  "sessions.title": "Последние сессии",
  "sessions.empty": "Сессий пока нет",
  "sessions.local": "История сессий хранится только на этом компьютере.",
  "sessions.started": "Начало",
  "sessions.battles": "Бои",
  "sessions.kills": "Убийства",
  "sessions.deaths": "Смерти",
  "diagnostics.title": "Диагностика",
  "diagnostics.description":
    "Проверка двух локальных подключений, необходимых WT Presence.",
  "diagnostics.telemetry": "Данные War Thunder",
  "diagnostics.telemetryHelp":
    "WT Presence читает локальную телеметрию игры по адресу 127.0.0.1. Вход в Gaijin не используется.",
  "diagnostics.discord": "Подключение к Discord",
  "diagnostics.discordHelp":
    "WT Presence отправляет активность через Discord Desktop на этом компьютере.",
  "diagnostics.lastTelemetry": "Последний ответ War Thunder",
  "diagnostics.lastTelemetryHelp":
    "Последнее время получения корректных локальных данных игры.",
  "diagnostics.lastDiscord": "Последнее обновление Discord",
  "diagnostics.lastDiscordHelp":
    "Последняя успешная отправка данных в Discord. Это не подтверждает внешний вид карточки в профиле.",
  "diagnostics.started": "Запущено",
  "diagnostics.version": "Версия",
  "diagnostics.lastError": "Последняя ошибка",
  "diagnostics.noError": "Текущих ошибок нет",
  "diagnostics.launchWt": "Запустите War Thunder и войдите в ангар или бой.",
  "diagnostics.launchDiscord":
    "Откройте Discord Desktop и оставьте его запущенным.",
  "diagnostics.ready": "Все подключения для Rich Presence работают.",
  "diagnostics.copy": "Скопировать отчёт",
  "diagnostics.copied": "Скопировано",
  "diagnostics.copyHelp":
    "Копирует безопасный отчёт без токенов, путей пользователя, шаблонов и данных аккаунта.",
  "diagnostics.logs": "Локальные логи",
  "diagnostics.logsHelp":
    "WT Presence хранит ежедневные логи семь дней в локальной папке данных приложения.",
  "common.never": "Никогда",
  "common.none": "Нет",
  "common.help": "Справка",
  "error.title": "Произошла ошибка",
} satisfies Record<keyof typeof en, string>;

export const messages = { en, ru };
export type MessageKey = keyof typeof en;

export function translate(locale: Locale, key: MessageKey): string {
  return messages[locale][key];
}
