import { create } from "zustand"
import { toast } from "sonner"
import type { GlossaryBuildSummary } from "@/types/generated/GlossaryBuildSummary"
import type { GlossaryDiff } from "@/types/generated/GlossaryDiff"
import type { GlossaryEvent } from "@/types/generated/GlossaryEvent"
import type { GlossaryPhase } from "@/types/generated/GlossaryPhase"
import type { LogLevel } from "@/types/generated/LogLevel"
import type { ReferenceSummary } from "@/types/generated/ReferenceSummary"
import { useAppStore } from "@/stores/app-store"

/** HH:MM:SS receive-time stamp for log lines (close enough to emit time). */
const now = () => new Date().toLocaleTimeString("en-GB", { hour12: false })

const MAX_LOG_LINES = 500

export type GlossaryOp = "build" | "normalize" | "import"

export interface GlossaryLogLine {
  at: string
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
  /** Free-text shown in the import run description ("40 translated files"). */
  opDetail: string | null
  /** ③ Reference review screen visibility + the import that opened it. */
  reviewOpen: boolean
  lastImport: ReferenceSummary | null
  startOp: (op: GlossaryOp, folder: string, detail?: string) => void
  endOp: () => void
  setLastDiff: (d: GlossaryDiff) => void
  applyEvent: (e: GlossaryEvent) => void
  openReview: (folder: string, lastImport?: ReferenceSummary) => void
  closeReview: () => void
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
  opDetail: null,
  reviewOpen: false,
  lastImport: null,

  // busy is set optimistically before the invoke; a rejected invoke must call
  // endOp() or the page soft-locks (step-3 lesson).
  startOp: (op, folder, detail) =>
    set((s) => ({
      busy: op,
      folder,
      opDetail: detail ?? null,
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
  openReview: (folder, lastImport) =>
    set((s) => ({
      reviewOpen: true,
      // The review belongs to a folder; tagging it lets the folder-change
      // reset close a review opened before any run this session.
      folder: s.folder ?? folder,
      lastImport: lastImport ?? s.lastImport,
    })),
  closeReview: () => set({ reviewOpen: false, lastImport: null }),

  applyEvent: (e) =>
    set((s) => {
      switch (e.kind) {
        case "phase":
          return { phase: e.phase, phaseDetail: e.detail }
        case "progress":
          // Completion-order emission can deliver counts out of order → clamp
          // with max(). BUT a `done: 0` (or a total change) starts a NEW
          // sequence (reference phase → extraction phase) — accept it as a
          // reset instead of clamping it away.
          return {
            done:
              e.done === 0 || e.total !== s.total ? e.done : Math.max(s.done, e.done),
            total: e.total,
          }
        case "log":
          return {
            logs: [
              ...s.logs.slice(-(MAX_LOG_LINES - 1)),
              { at: now(), level: e.level, message: e.message },
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
          toast.error(e.message)
          return {
            busy: null,
            error: e.message,
            logs: [
              ...s.logs.slice(-(MAX_LOG_LINES - 1)),
              { at: now(), level: "error" as LogLevel, message: e.message },
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
      opDetail: null, reviewOpen: false, lastImport: null,
    }),
}))
