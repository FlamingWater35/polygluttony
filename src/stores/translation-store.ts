import { create } from "zustand"
import type { FileResult } from "@/types/generated/FileResult"
import type { FileStateKind } from "@/types/generated/FileStateKind"
import type { LogLevel } from "@/types/generated/LogLevel"
import type { LogPhase } from "@/types/generated/LogPhase"
import type { RunEvent } from "@/types/generated/RunEvent"

const MAX_LOG_LINES = 500

export interface FileRow {
  state: FileStateKind
  detail: string | null
  translated: number
  total: number
  batch: number
  totalBatches: number
  retries: number
  hasWarnings: boolean | null
  error: string | null
}

export interface LogLine {
  file: string | null
  level: LogLevel
  phase: LogPhase
  message: string
}

interface TranslationRunState {
  running: boolean
  files: Record<string, FileRow>
  logs: LogLine[]
  results: FileResult[] | null
  start: (files: string[]) => void
  applyEvent: (e: RunEvent) => void
  reset: () => void
}

const emptyRow = (): FileRow => ({
  state: "pending",
  detail: null,
  translated: 0,
  total: 0,
  batch: 0,
  totalBatches: 0,
  retries: 0,
  hasWarnings: null,
  error: null,
})

export const useTranslationRun = create<TranslationRunState>((set) => ({
  running: false,
  files: {},
  logs: [],
  results: null,

  start: (files) =>
    set({
      running: true,
      results: null,
      logs: [],
      files: Object.fromEntries(files.map((f) => [f, emptyRow()])),
    }),

  applyEvent: (e) =>
    set((s) => {
      const files = { ...s.files }
      const touch = (name: string, patch: Partial<FileRow>) => {
        files[name] = { ...(files[name] ?? emptyRow()), ...patch }
      }
      switch (e.kind) {
        case "state":
          touch(e.file, { state: e.state, detail: e.detail })
          return { files }
        case "progress":
          touch(e.file, {
            translated: e.translated,
            total: e.total,
            batch: e.batch,
            totalBatches: e.total_batches,
            retries: e.retries,
          })
          return { files }
        case "log":
          return {
            logs: [
              ...s.logs.slice(-(MAX_LOG_LINES - 1)),
              { file: e.file, level: e.level, phase: e.phase, message: e.message },
            ],
          }
        case "file_done":
          touch(e.file, { hasWarnings: e.has_warnings })
          return { files }
        case "error":
          touch(e.file, { error: e.message, state: "failed" })
          return { files }
        case "run_finished":
          return { files, results: e.results, running: false }
        default:
          return {}
      }
    }),

  reset: () => set({ running: false, files: {}, logs: [], results: null }),
}))
