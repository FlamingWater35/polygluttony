import { create } from "zustand";
import { persist } from "zustand/middleware";

/**
 * Available UI scale presets.
 * Maps semantic names to CSS root font-size values.
 * Changing the root font-size scales all `rem`-based Tailwind utilities proportionally.
 */
export type UIScale = "sm" | "md" | "lg" | "xl";

export const UI_SCALE_MAP: Record<UIScale, string> = {
  sm: "14px",
  md: "16px",
  lg: "18px",
  xl: "20px",
};

export const UI_SCALE_LABELS: Record<UIScale, string> = {
  sm: "Compact",
  md: "Default",
  lg: "Large",
  xl: "Extra Large",
};

interface SettingsState {
  uiScale: UIScale;
  setUIScale: (scale: UIScale) => void;
}

/**
 * Global settings store.
 * Persisted to localStorage so user preferences survive reloads and app restarts.
 */
export const useSettingsStore = create<SettingsState>()(
  persist(
    (set) => ({
      uiScale: "md",
      setUIScale: (uiScale) => set({ uiScale }),
    }),
    {
      name: "polygluttony-settings",
    }
  )
);
