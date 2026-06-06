import { create } from "zustand";
import type { WorldType } from "@/types/generated/WorldType";
import type { Tone } from "@/types/generated/Tone";

interface ProjectState {
  workdir: string;
  sourceLang: string;
  targetLang: string;
  worldType: WorldType;
  tone: Tone;
  fileCount: number;
  dialogueLineCount: number;
  hasUntranslated: boolean;
  hasTranslated: boolean;
  glossaryTerms: number | null;
}

interface AppState {
  workdir: string | null;
  sourceLang: string;
  targetLang: string;
  worldType: WorldType | null;
  tone: Tone;
  fileCount: number;
  dialogueLineCount: number;
  hasUntranslated: boolean;
  hasTranslated: boolean;
  glossaryTerms: number | null;
  activeConnection: string | null;
  hasUsableConnection: boolean;
  setWorkdir: (dir: string | null) => void;
  setLanguages: (source: string, target: string) => void;
  setProject: (p: ProjectState) => void;
  clearProject: () => void;
  setActiveConnection: (name: string | null) => void;
  setHasUsableConnection: (v: boolean) => void;
  setGlossaryTerms: (n: number | null) => void;
}

export const useAppStore = create<AppState>((set) => ({
  workdir: null,
  sourceLang: "zh",
  targetLang: "en",
  worldType: null,
  tone: "standard",
  fileCount: 0,
  dialogueLineCount: 0,
  hasUntranslated: false,
  hasTranslated: false,
  glossaryTerms: null,
  activeConnection: null,
  hasUsableConnection: false,
  setWorkdir: (workdir) => set({ workdir }),
  setLanguages: (sourceLang, targetLang) => set({ sourceLang, targetLang }),
  setProject: (p) => set({ ...p }),
  clearProject: () =>
    set({
      workdir: null,
      worldType: null,
      fileCount: 0,
      dialogueLineCount: 0,
      hasUntranslated: false,
      hasTranslated: false,
      glossaryTerms: null,
    }),
  setActiveConnection: (activeConnection) => set({ activeConnection }),
  setHasUsableConnection: (hasUsableConnection) => set({ hasUsableConnection }),
  setGlossaryTerms: (glossaryTerms) => set({ glossaryTerms }),
}));
