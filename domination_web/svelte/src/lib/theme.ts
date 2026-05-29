import { writable } from "svelte/store";

export type ThemeId = "light" | "dark";

const STORAGE_KEY = "domination-theme";

export const theme = writable<ThemeId>("dark");

function applyTheme(id: ThemeId) {
  const root = document.documentElement;
  root.classList.remove("theme-light", "theme-dark");
  root.classList.add(id === "light" ? "theme-light" : "theme-dark");
}

export function initTheme(): void {
  let id: ThemeId = "dark";
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored === "light" || stored === "dark") {
      id = stored;
    }
  } catch {
    /* private browsing */
  }
  applyTheme(id);
  theme.set(id);
}

export function setTheme(id: ThemeId): void {
  applyTheme(id);
  theme.set(id);
  try {
    localStorage.setItem(STORAGE_KEY, id);
  } catch {
    /* ignore */
  }
}

export function toggleTheme(): void {
  let next: ThemeId = "dark";
  theme.update((current) => {
    next = current === "dark" ? "light" : "dark";
    return next;
  });
  setTheme(next);
}
