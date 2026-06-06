import { create } from "zustand"
import type { GlossaryBuildSummary } from "@/types/generated/GlossaryBuildSummary"
import type { GlossaryDiff } from "@/types/generated/GlossaryDiff"
import type { GlossaryEvent } from "@/types/generated/GlossaryEvent"
import type { GlossaryPhase } from "@/types/generated/GlossaryPhase"
import type { LogLevel } from "@/types/generated/LogLevel"
import { useAppStore } from "@/stores/app-store"

const MAX_LOG_LINES = 500

export type GlossaryOp = "build" | "normalize" | "import"

export interface GlossaryLogLine {
  level: LogLevel
  message: string
}

interface GlossaryRunStore {
  busy: GlossaryOp | null
  /** The folder this run (or its results) belongs to; null = never ran. */
  folder: string | null
  phase: GlossaryPhase | null
  phaseDetail: string | null
  done: number
  total: number
  logs: GlossaryLogLine[]
  summary: GlossaryBuildSummary | null
  lastDiff: GlossaryDiff | null
  error: string | null
  /** Bumped on Done / FileChanged — the page refetches the glossary query. */
  fileTick: number
  startOp: (op: GlossaryOp, folder: string) => void
  endOp: () => void
  setLastDiff: (d: GlossaryDiff) => void
  applyEvent: (e: GlossaryEvent) => void
  reset: () => void
}

export const useGlossaryRun = create<GlossaryRunStore>((set) => ({
  busy: null,
  folder: null,
  phase: null,
  phaseDetail: null,
  done: 0,
  total: 0,
  logs: [],
  summary: null,
  lastDiff: null,
  error: null,
  fileTick: 0,

  // busy is set optimistically before the invoke; a rejected invoke must call
  // endOp() or the page soft-locks (step-3 lesson).
  startOp: (op, folder) =>
    set((s) => ({
      busy: op,
      folder,
      phase: null,
      phaseDetail: null,
      done: 0,
      total: 0,
      logs: [],
      error: null,
      summary: op === "build" ? null : s.summary,
    })),
  endOp: () => set({ busy: null }),
  setLastDiff: (lastDiff) => set({ lastDiff }),

  applyEvent: (e) =>
    set((s) => {
      switch (e.kind) {
        case "phase":
          return { phase: e.phase, phaseDetail: e.detail }
        case "progress":
          return { done: e.done, total: e.total }
        case "log":
          return {
            logs: [
              ...s.logs.slice(-(MAX_LOG_LINES - 1)),
              { level: e.level, message: e.message },
            ],
          }
        case "done":
          // Keep the rail badge live (cross-store side effect, deliberate).
          useAppStore.getState().setGlossaryTerms(e.summary.terms_final)
          return {
            busy: null,
            summary: e.summary,
            lastDiff: e.summary.diff.has_changes ? e.summary.diff : s.lastDiff,
            fileTick: s.fileTick + 1,
          }
        case "error":
          return {
            busy: null,
            error: e.message,
            logs: [
              ...s.logs.slice(-(MAX_LOG_LINES - 1)),
              { level: "error" as LogLevel, message: e.message },
            ],
          }
        case "file_changed":
          return { fileTick: s.fileTick + 1 }
        default:
          return {}
      }
    }),

  reset: () =>
    set({
      busy: null, folder: null, phase: null, phaseDetail: null, done: 0, total: 0,
      logs: [], summary: null, lastDiff: null, error: null,
    }),
}))
