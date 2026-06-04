import { create } from "zustand";

interface AppState {
  workdir: string | null;
  sourceLang: string;
  targetLang: string;
  activeConnection: string | null;
  hasUsableConnection: boolean;
  setWorkdir: (dir: string | null) => void;
  setLanguages: (source: string, target: string) => void;
  setActiveConnection: (name: string | null) => void;
  setHasUsableConnection: (v: boolean) => void;
}

export const useAppStore = create<AppState>((set) => ({
  workdir: null,
  sourceLang: "zh",
  targetLang: "en",
  activeConnection: null,
  hasUsableConnection: false,
  setWorkdir: (workdir) => set({ workdir }),
  setLanguages: (sourceLang, targetLang) => set({ sourceLang, targetLang }),
  setActiveConnection: (activeConnection) => set({ activeConnection }),
  setHasUsableConnection: (hasUsableConnection) => set({ hasUsableConnection }),
}));
