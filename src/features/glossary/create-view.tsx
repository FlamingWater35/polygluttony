import { useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { Sparkle, Books, Globe, X } from "@phosphor-icons/react";
import { toast } from "sonner";
import type { ProjectView } from "@/types/generated/ProjectView";
import type { WorldType } from "@/types/generated/WorldType";
import { ipc } from "@/lib/ipc";
import { useGlossaryRun } from "@/stores/glossary-store";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Textarea } from "@/components/ui/textarea";
import { HelpText } from "@/components/help-text";

// ── reference status query key (shared with future editor) ────────────────────

export function referenceStatusKey(folder: string) {
  return ["reference-status", folder] as const;
}

// ── CreateView ────────────────────────────────────────────────────────────────

export function CreateView({ view }: { view: ProjectView }) {
  const qc = useQueryClient();
  const { startOp, endOp } = useGlossaryRun.getState();
  const busy = useGlossaryRun((s) => s.busy);
  const summary = useGlossaryRun((s) => s.summary);
  const error = useGlossaryRun((s) => s.error);

  // Generate card state
  const [normalize, setNormalize] = useState(true);
  const [personalize, setPersonalize] = useState(false);
  const [context, setContext] = useState("");

  // Personalization connection availability
  const { data: personalizationStatus } = useQuery({
    queryKey: ["personalization-status"],
    queryFn: ipc.personalizationStatus,
  });

  // Reference import status chip
  const { data: refStatus } = useQuery({
    queryKey: referenceStatusKey(view.folder),
    queryFn: () => ipc.referenceStatus(view.folder),
  });

  const selected = view.prefs.selected_files;
  const effectiveWorld: WorldType = view.prefs.world_override ?? view.detected_world;
  const canGenerate = selected.length > 0 && busy === null;

  // ── generate action ──────────────────────────────────────────────────────────

  const generate = () => {
    startOp("build");
    // Rejected invoke = run never started; un-stick the page (step-3 lesson).
    ipc
      .startGlossaryBuild({
        folder: view.folder,
        files: selected,
        worldType: effectiveWorld,
        sourceLang: view.prefs.source_lang,
        targetLang: view.prefs.target_lang,
        normalize,
        personalize,
        personalizeContext: context,
      })
      .catch((e: unknown) => {
        endOp();
        toast.error(String(e));
      });
  };

  // ── import action ────────────────────────────────────────────────────────────

  const importFiles = async () => {
    const paths = await openDialog({
      multiple: true,
      filters: [{ name: "ASS subtitles", extensions: ["ass"] }],
    });
    if (!paths || (Array.isArray(paths) && paths.length === 0)) return;
    const fileList = Array.isArray(paths) ? paths : [paths];

    startOp("import");
    try {
      const result = await ipc.importReferenceFiles(view.folder, fileList);
      toast.success(
        `Imported ${result.count} reference terms from ${result.files_processed} files`,
      );
      for (const err of result.errors) {
        toast.warning(err);
      }
      await qc.invalidateQueries({ queryKey: referenceStatusKey(view.folder) });
    } catch (e: unknown) {
      toast.error(String(e));
    } finally {
      endOp();
    }
  };

  // ── clear reference action ────────────────────────────────────────────────────

  const clearRef = async () => {
    await ipc.clearReference(view.folder);
    await qc.invalidateQueries({ queryKey: referenceStatusKey(view.folder) });
  };

  // ── render ────────────────────────────────────────────────────────────────────

  return (
    <div className="flex h-full flex-col">
      <div className="flex-1 overflow-auto p-5 flex flex-col gap-4">

        {/* Error surfacing — hard requirement: failed/empty build returns here, user MUST see why */}
        {error ? (
          <p className="text-sm text-[color:var(--color-danger)]">{error}</p>
        ) : null}
        {summary && summary.errors.length > 0 ? (
          <div className="rounded-md border border-[color:var(--color-danger)]/30 bg-[color:var(--color-danger)]/5 px-4 py-3 text-sm">
            <p className="font-medium text-foreground mb-1.5">
              Last build finished with {summary.errors.length} issue
              {summary.errors.length !== 1 ? "s" : ""} — {summary.terms_final} terms were still
              saved:
            </p>
            <ul className="space-y-0.5">
              {summary.errors.map((msg, i) => (
                <li key={i} className="text-[12px] text-muted-foreground">
                  {msg}
                </li>
              ))}
            </ul>
          </div>
        ) : null}

        {/* Generate card */}
        <div className="rounded-lg border border-border bg-[color:var(--card)] p-5">
          <div className="mb-3 flex items-center gap-2">
            <Sparkle weight="duotone" className="size-4 text-primary" />
            <h2 className="text-sm font-semibold text-foreground">Generate glossary</h2>
          </div>
          <p className="mb-4 text-[12.5px] text-muted-foreground">
            Reads your selected subtitle files and extracts character names, place names,
            cultivation terms, and other recurring phrases into a structured glossary.
          </p>

          {/* Normalize checkbox */}
          <label className="flex items-start gap-2.5 cursor-pointer mb-3">
            <Checkbox
              checked={normalize}
              onCheckedChange={(v) => setNormalize(v === true)}
              className="mt-0.5"
            />
            <span className="text-sm text-foreground select-none">
              Clean up &amp; standardize
            </span>
          </label>
          <div className="ml-6 mb-4">
            <HelpText>Merges duplicate names and fixes inconsistent spellings.</HelpText>
          </div>

          {/* Personalize checkbox */}
          <label
            className={`flex items-start gap-2.5 mb-1 ${personalizationStatus === null ? "opacity-50 cursor-not-allowed" : "cursor-pointer"}`}
          >
            <Checkbox
              checked={personalize}
              onCheckedChange={(v) => setPersonalize(v === true)}
              disabled={personalizationStatus === null}
              className="mt-0.5"
            />
            <span className="text-sm text-foreground select-none">
              Look up established names online
            </span>
          </label>
          <div className="ml-6 mb-3">
            {personalizationStatus === null ? (
              <p className="mt-1 flex items-start gap-1 text-[11px] leading-snug text-muted-foreground">
                <Globe className="mt-px size-3 shrink-0 text-muted-foreground" />
                <span>
                  Needs a web-capable personalization connection — set one in Connections.
                </span>
              </p>
            ) : (
              <HelpText>
                Searches the web for this show&apos;s commonly-used names, so your glossary
                matches what fans expect.
              </HelpText>
            )}
          </div>

          {/* Context textarea — shown when personalize is checked and available */}
          {personalize && personalizationStatus !== null ? (
            <div className="ml-6 mb-2">
              <Textarea
                value={context}
                onChange={(e) => setContext(e.target.value)}
                placeholder="Show name (first line), wiki links or notes…"
                className="text-sm"
              />
            </div>
          ) : null}
        </div>

        {/* Import card */}
        <div className="rounded-lg border border-border bg-[color:var(--card)] p-5">
          <div className="mb-3 flex items-center gap-2">
            <Books weight="duotone" className="size-4 text-primary" />
            <h2 className="text-sm font-semibold text-foreground">Import reference files</h2>
          </div>
          <p className="mb-4 text-[12.5px] text-muted-foreground">
            Point me to .ass files you&apos;ve already translated well — their wording guides the
            new glossary.
          </p>

          <div className="flex items-center gap-3">
            <Button
              variant="secondary"
              onClick={() => void importFiles()}
              disabled={busy !== null}
            >
              Choose files…
            </Button>

            {/* Reference status chip */}
            {refStatus && refStatus.source !== "none" ? (
              <span className="inline-flex items-center gap-1.5 rounded-full border border-border bg-[color:var(--color-bg-raised)] px-2.5 py-1 text-[11px] text-muted-foreground">
                {refStatus.source === "cached" ? (
                  <>
                    {refStatus.count} reference terms · imported
                    <button
                      type="button"
                      aria-label="Clear reference terms"
                      className="ml-0.5 rounded-sm hover:text-foreground transition-colors"
                      onClick={() => void clearRef()}
                    >
                      <X className="size-3" />
                    </button>
                  </>
                ) : (
                  <>ref/ folder detected · {refStatus.count} files</>
                )}
              </span>
            ) : null}
          </div>
        </div>
      </div>

      {/* Footer bar */}
      <div className="flex items-center gap-3 border-t border-border bg-[color:var(--popover)] px-5 py-3">
        <Button onClick={generate} disabled={!canGenerate}>
          Generate glossary →
        </Button>
        <span className="text-[11px] text-muted-foreground">
          {selected.length === 0
            ? "Select files in Project first."
            : `${selected.length} file${selected.length !== 1 ? "s" : ""} · world: ${effectiveWorld}`}
        </span>
      </div>
    </div>
  );
}
