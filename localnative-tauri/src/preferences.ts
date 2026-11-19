export type ThemePreference = "system" | "light" | "dark";

const PREFIX = "localnative.";

function read(key: string) {
  try {
    return localStorage.getItem(PREFIX + key);
  } catch {
    return null;
  }
}

function write(key: string, value: string) {
  try {
    localStorage.setItem(PREFIX + key, value);
  } catch {
    // The preference still applies for this session when storage is unavailable.
  }
}

export function loadPageSize() {
  const value = Number(read("pageSize"));
  return Number.isInteger(value) && value >= 5 && value <= 1000 ? value : 10;
}

export function savePageSize(value: number) {
  const normalized = Number.isFinite(value)
    ? Math.min(1000, Math.max(5, Math.round(value)))
    : 10;
  write("pageSize", String(normalized));
  return normalized;
}

export function loadDeleteConfirmation() {
  return read("confirmDelete") !== "false";
}

export function saveDeleteConfirmation(value: boolean) {
  write("confirmDelete", String(value));
}

export function loadThemePreference(): ThemePreference {
  const value = read("theme");
  return value === "light" || value === "dark" ? value : "system";
}

export function applyThemePreference(preference: ThemePreference) {
  const resolved =
    preference === "system"
      ? window.matchMedia("(prefers-color-scheme: dark)").matches
        ? "dark"
        : "light"
      : preference;
  document.documentElement.dataset.theme = resolved;
  document.documentElement.style.colorScheme = resolved;
}

export function saveThemePreference(preference: ThemePreference) {
  write("theme", preference);
  applyThemePreference(preference);
}

export function loadLocalePreference() {
  return read("locale");
}

export function saveLocalePreference(locale: string) {
  write("locale", locale);
}
