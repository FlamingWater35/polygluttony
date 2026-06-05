import { useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { Play, Stop, CaretDown, CaretRight } from "@phosphor-icons/react";
import type { Tone } from "@/types/generated/Tone";
import { ipc } from "@/lib/ipc";
import { useAppStore } from "@/stores/app-store";
import { useProject, syncProjectStore, projectKey } from "@/features/project/use-project";
import { useTranslationRun } from "@/stores/translation-store";
import type { FileStateKind } from "@/types/generated/FileStateKind";
import { PageHeader } from "@/components/page-header";
import { Button } from "@/components/ui/button";

// ── constants ────────────────────────────────────────────────────────────────

const TONES: Tone[] = ["standard", "xianxia", "wuxia", "comedic", "funny"];

const SELECT_CLS =
  "h-9 rounded-md border border-input bg-[color:var(--card)] px-2 text-sm";

/** Lines-per-minute heuristic for the estimate banner. */
const LINES_PER_MINUTE = 600;

// ── state chip helpers ────────────────────────────────────────────────────────

const STATE_LABEL: Record<FileStateKind, string> = {
  pending: "Waiting",
  translating: "Translating…",
  retranslating: "Retranslating…",
  cleanup: "Cleaning up…",
  verifying: "Verifying…",
  done: "Done",
  warning: "Needs a look",
  failed: "Failed",
};

const STATE_COLOR: Record<FileStateKind, string> = {
  pending: "var(--muted-foreground)",
  translating: "var(--primary)",
  retranslating: "var(--color-alert)",
  cleanup: "var(--color-state-cleanup)",
  verifying: "var(--color-state-verify)",
  done: "var(--color-success)",
  warning: "var(--color-alert)",
  failed: "var(--color-danger)",
};

/** Whether the state should pulse to indicate live activity. */
const STATE_ANIMATE = new Set<FileStateKind>([
  "translating",
  "retranslating",
  "cleanup",
  "verifying",
]);

function FileStateChip({ state }: { state: FileStateKind }) {
  const color = STATE_COLOR[state] ?? STATE_COLOR.pending;
  const animate = STATE_ANIMATE.has(state);
  return (
    <span className="inline-flex items-center gap-1.5 rounded-[13px] border border-border px-2.5 py-0.5 text-[11px] shrink-0">
      <span
        className={`size-2 rounded-full${animate ? " animate-pulse" : ""}`}
        style={{ background: color }}
      />
      {STATE_LABEL[state]}
    </span>
  );
}

// ── log level colors ──────────────────────────────────────────────────────────

const LOG_COLOR: Record<string, string> = {
  debug: "text-muted-foreground",
  info: "text-foreground",
  warning: "text-[color:var(--color-alert)]",
  error: "text-[color:var(--color-danger)]",
};

// ── main component ────────────────────────────────────────────────────────────

export function TranslatePage() {
  const workdir = useAppStore((s) => s.workdir);
  const activeConnection = useAppStore((s) => s.activeConnection);
  const hasUsableConnection = useAppStore((s) => s.hasUsableConnection);
  const qc = useQueryClient();

  const { data: view } = useProject(workdir ?? "");

  const running = useTranslationRun((s) => s.running);
  const storeFiles = useTranslationRun((s) => s.files);
  const logs = useTranslationRun((s) => s.logs);
  const results = useTranslationRun((s) => s.results);

  const [logsOpen, setLogsOpen] = useState(false);

  if (!workdir || !view) return null;

  const prefs = view.prefs;
  const selected = prefs.selected_files;
  const sourceLang = prefs.source_lang;
  const targetLang = prefs.target_lang;
  const tone = prefs.tone;

  // ── persist helpers ─────────────────────────────────────────────────────────

  const persistTone = (next: Tone) => {
    const nextPrefs = { ...prefs, tone: next };
    void ipc.saveFolderPrefs(view.folder, nextPrefs);
    qc.setQueryData(projectKey(view.folder), { ...view, prefs: nextPrefs });
    syncProjectStore({ ...view, prefs: nextPrefs });
  };

  // ── estimate banner ──────────────────────────────────────────────────────────

  const selectedFiles = view.files.filter((f) => selected.includes(f.name));
  const totalLines = selectedFiles.reduce((s, f) => s + f.dialogue_count, 0);
  const estimateMinutes = Math.ceil(totalLines / LINES_PER_MINUTE);

  const glossaryClause =
    view.glossary_terms != null
      ? ` · using glossary (${view.glossary_terms} terms)`
      : "";

  const connLabel = activeConnection ?? "no connection";

  // ── actions ──────────────────────────────────────────────────────────────────

  const canStart = !running && selected.length > 0 && hasUsableConnection;

  const handleStart = () => {
    useTranslationRun.getState().start(selected);
    ipc.startTranslation({
      folder: workdir,
      files: selected,
      tone,
      sourceLang,
      targetLang,
    }).catch((err: unknown) => {
      const store = useTranslationRun.getState();
      store.reset();
      store.applyEvent({
        kind: "log",
        file: null,
        level: "error",
        phase: "error",
        message: err instanceof Error ? err.message : String(err),
      });
    });
  };

  const handleCancel = () => {
    void ipc.cancelTranslation();
  };

  // ── completion summary ────────────────────────────────────────────────────────

  let summaryEl: React.ReactNode = null;
  if (results !== null) {
    const clean = results.filter((r) => !r.has_warnings).length;
    const warn = results.filter((r) => r.has_warnings).length;
    summaryEl = (
      <div className="rounded-md border border-[color:var(--color-success)]/30 bg-[color:var(--color-success)]/5 px-4 py-2.5 text-sm">
        <span className="font-medium text-foreground">
          Translated {results.length} files —{" "}
        </span>
        <span className="text-[color:var(--color-success)]">{clean} clean</span>
        {warn > 0 ? (
          <>
            <span className="text-muted-foreground">, </span>
            <span className="text-[color:var(--color-alert)]">{warn} need a look</span>
          </>
        ) : null}
        <span className="text-muted-foreground">.</span>
        {warn > 0 ? (
          <span
            className="ml-3 cursor-not-allowed text-xs text-muted-foreground line-through"
            title="Verify view arrives in a later step"
          >
            Review issues →
          </span>
        ) : null}
      </div>
    );
  }

  // ── file table rows ───────────────────────────────────────────────────────────

  // Merge store state with static file list.
  const tableRows = selectedFiles.map((f) => {
    const row = storeFiles[f.name];
    const result = results?.find((r) => r.file === f.name);

    let state: FileStateKind = "pending";
    if (result) {
      state = !result.success ? "failed" : result.has_warnings ? "warning" : "done";
    } else if (row) {
      state = row.state;
    }

    return { file: f, row, state };
  });

  const inProgressCount = tableRows.filter(
    ({ state }) => state !== "pending" && state !== "done" && state !== "warning" && state !== "failed",
  ).length;

  // Progress bar calculations (while running)
  const doneMeterCount = tableRows.filter(({ state }) => state === "done" || state === "warning" || state === "failed").length;
  const progressPct = Math.round((doneMeterCount / tableRows.length) * 100);

  // ── render ────────────────────────────────────────────────────────────────────

  return (
    <div className="flex h-full flex-col">
      {/* Header */}
      <PageHeader
        title="Translate"
        description="Pick files, see an estimate, then run the pipeline."
        actions={
          <div className="flex items-center gap-2 shrink-0">
            <label className="text-[11.5px] font-semibold text-muted-foreground" htmlFor="tone-select-translate">
              Tone
            </label>
            <select
              id="tone-select-translate"
              className={SELECT_CLS}
              value={tone}
              onChange={(e) => persistTone(e.target.value as Tone)}
            >
              {TONES.map((t) => (
                <option key={t} value={t}>
                  {t}
                </option>
              ))}
            </select>
          </div>
        }
      />

      {/* Scrollable body */}
      <div className="flex-1 overflow-auto p-5 flex flex-col gap-4">

        {/* Estimate banner */}
        {selected.length > 0 ? (
          <div className="rounded-md border border-primary/30 bg-primary/5 px-4 py-2.5 text-[12.5px] text-foreground">
            <span className="tabular-nums">
              {selectedFiles.length} file{selectedFiles.length !== 1 ? "s" : ""} · {totalLines.toLocaleString()} lines
              {glossaryClause} · est.{" "}
              <span className="font-medium">~{estimateMinutes}m</span> on{" "}
              <span className="font-medium text-primary">{connLabel}</span>.
            </span>
          </div>
        ) : (
          <div className="rounded-md border border-border bg-[color:var(--card)] px-4 py-2.5 text-[12.5px] text-muted-foreground">
            Select files below to see an estimate.
          </div>
        )}

        {/* Completion summary */}
        {summaryEl}

        {/* File table */}
        <div className="rounded-md border border-border overflow-hidden">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-border bg-[color:var(--color-bg-raised)] text-[11px] text-muted-foreground uppercase tracking-wide">
                <th className="px-4 py-2 text-left font-medium w-[40%]">File</th>
                <th className="px-4 py-2 text-right font-medium w-[10%] tabular-nums">Lines</th>
                <th className="px-4 py-2 text-left font-medium w-[16%]">State</th>
                <th className="px-4 py-2 text-left font-medium">Progress</th>
              </tr>
            </thead>
            <tbody>
              {tableRows.length === 0 ? (
                <tr>
                  <td colSpan={4} className="px-4 py-6 text-center text-sm text-muted-foreground">
                    No files selected. Go to the Project view to select files.
                  </td>
                </tr>
              ) : (
                tableRows.map(({ file, row, state }) => {
                  const showProgress =
                    row &&
                    row.total > 0 &&
                    state !== "pending" &&
                    state !== "done" &&
                    state !== "warning";

                  return (
                    <tr
                      key={file.name}
                      className="border-b border-border last:border-0 hover:bg-[color:var(--bg-hover)]"
                    >
                      <td className="px-4 py-2.5 text-[12.5px] font-mono truncate max-w-0 w-[40%]">
                        <span className="truncate block">{file.name}</span>
                      </td>
                      <td className="px-4 py-2.5 text-right tabular-nums text-[12px] text-muted-foreground w-[10%]">
                        {file.dialogue_count.toLocaleString()}
                      </td>
                      <td className="px-4 py-2.5 w-[16%]">
                        <FileStateChip state={state} />
                      </td>
                      <td className="px-4 py-2.5 text-[12px] text-muted-foreground tabular-nums">
                        {showProgress && row ? (
                          <span>
                            {row.translated}/{row.total} · batch {row.batch}/{row.totalBatches}
                            {row.retries > 0 ? (
                              <span className="ml-2 text-[color:var(--color-alert)]">
                                ↺ {row.retries}
                              </span>
                            ) : null}
                          </span>
                        ) : row?.error ? (
                          <span className="text-[color:var(--color-danger)] truncate block max-w-xs">
                            {row.error}
                          </span>
                        ) : (
                          <span>—</span>
                        )}
                      </td>
                    </tr>
                  );
                })
              )}
            </tbody>
          </table>
        </div>

        {/* Overall progress bar (while running) */}
        {running && tableRows.length > 0 ? (
          <div className="space-y-1">
            <div className="flex justify-between text-[11px] text-muted-foreground tabular-nums">
              <span>{inProgressCount > 0 ? `Translating file ${doneMeterCount + 1} of ${tableRows.length}…` : "Wrapping up…"}</span>
              <span>{progressPct}%</span>
            </div>
            <div className="relative h-2 w-full overflow-hidden rounded-full bg-muted">
              <div
                className="h-full bg-primary transition-all duration-500"
                style={{ width: `${progressPct}%` }}
              />
            </div>
          </div>
        ) : null}

      </div>

      {/* Controls row + log toggle */}
      <div className="border-t border-border bg-[color:var(--popover)] px-5 py-3 flex items-center gap-3">
        {running ? (
          <Button variant="destructive" onClick={handleCancel}>
            <Stop className="size-4" /> Cancel
          </Button>
        ) : (
          <Button disabled={!canStart} onClick={handleStart}>
            <Play className="size-4" /> Start translation
          </Button>
        )}

        {selected.length === 0 && !running ? (
          <span className="text-[11.5px] text-muted-foreground">
            Select files in the Project view first.
          </span>
        ) : !hasUsableConnection && !running ? (
          <span className="text-[11.5px] text-[color:var(--color-alert)]">
            No usable connection — add one in Connections.
          </span>
        ) : null}

        <span className="flex-1" />

        {/* Log toggle */}
        <button
          type="button"
          className="flex items-center gap-1.5 text-[12px] text-muted-foreground hover:text-foreground transition-colors"
          onClick={() => setLogsOpen((o) => !o)}
        >
          {logsOpen ? <CaretDown className="size-3.5" /> : <CaretRight className="size-3.5" />}
          Logs
          {logs.length > 0 ? (
            <span className="ml-1 tabular-nums text-[11px]">({logs.length})</span>
          ) : null}
        </button>
      </div>

      {/* Log panel */}
      {logsOpen ? (
        <div className="max-h-48 overflow-auto border-t border-border bg-[color:var(--color-bg-deepest)] px-4 py-3 font-mono text-[11px]">
          {logs.length === 0 ? (
            <span className="text-muted-foreground">No logs yet.</span>
          ) : (
            logs.map((entry, i) => (
              <div key={i} className="flex gap-2 leading-5">
                {entry.file ? (
                  <span className="shrink-0 text-muted-foreground truncate max-w-[120px]">
                    {entry.file}
                  </span>
                ) : (
                  <span className="shrink-0 text-muted-foreground">run</span>
                )}
                <span className="shrink-0 text-muted-foreground/60">[{entry.phase}]</span>
                <span className={LOG_COLOR[entry.level] ?? "text-foreground"}>
                  {entry.message}
                </span>
              </div>
            ))
          )}
        </div>
      ) : null}
    </div>
  );
}
