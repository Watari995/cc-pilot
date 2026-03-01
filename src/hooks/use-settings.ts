import { invoke } from "@tauri-apps/api/core";
import { create } from "zustand";
import type { Settings } from "../lib/types";

const DEFAULT_SETTINGS: Settings = {
  accentColor: "#E8734A",
  terminalApp: "ghostty",
  defaultIde: "cursor",
  launchAtLogin: true,
  notificationsEnabled: true,
};

interface SettingsStore {
  settings: Settings;
  loaded: boolean;
  load: () => Promise<void>;
  update: (patch: Partial<Settings>) => Promise<void>;
}

export const useSettingsStore = create<SettingsStore>((set, get) => ({
  settings: DEFAULT_SETTINGS,
  loaded: false,

  load: async () => {
    try {
      const settings = await invoke<Settings>("get_settings");
      set({ settings, loaded: true });
      applyAccentColor(settings.accentColor);
    } catch (err) {
      console.error("Failed to load settings:", err);
      set({ loaded: true });
    }
  },

  update: async (patch) => {
    const merged = { ...get().settings, ...patch };
    set({ settings: merged });
    try {
      await invoke("save_settings", { newSettings: merged });
    } catch (err) {
      console.error("Failed to save settings:", err);
    }
    if (patch.accentColor) {
      applyAccentColor(patch.accentColor);
    }
  },
}));

function applyAccentColor(color: string) {
  const root = document.documentElement;
  root.style.setProperty("--accent", color);
  // accent-dim は 15% 不透明度
  root.style.setProperty("--accent-dim", `${color}26`);
}
