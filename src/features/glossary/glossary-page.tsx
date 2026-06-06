import { useEffect } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { ipc } from "@/lib/ipc";
import { useAppStore } from "@/stores/app-store";
import { useGlossaryRun } from "@/stores/glossary-store";
import { useProject } from "@/features/project/use-project";
import { EmptyState } from "@/components/empty-state";
import { CreateView } from "./create-view";
import { BuildProgress } from "./build-progress";
import { EditorView } from "./editor-view";

export function glossaryKey(folder: string) {
  return ["glossary", folder] as const;
}

// I3 — self-save suppression: every persist() renames the file which fires
// FileChanged → fileTick → invalidate. That invalidation can resolve after a
// newer optimistic setQueryData and silently revert the edit. We suppress the
// invalidation for 1500 ms after an in-app save. External edits (OS editor)
// cannot coincide with active in-app editing, so this is an accepted blind spot.
let lastLocalSaveAt = 0;
export function markLocalSave() {
  lastLocalSaveAt = Date.now();
}

export function GlossaryPage() {
  const workdir = useAppStore((s) => s.workdir);
  const qc = useQueryClient();
  const busy = useGlossaryRun((s) => s.busy);
  const fileTick = useGlossaryRun((s) => s.fileTick);
  const { data: view } = useProject(workdir ?? "");
  const { data: doc, isPending } = useQuery({
    queryKey: glossaryKey(workdir ?? ""),
    queryFn: () => ipc.loadGlossary(workdir ?? ""),
    enabled: !!workdir,
  });

  // Run state is global; a run (or its results) belongs to the folder it was
  // started in. Reset only when the state belongs to a DIFFERENT folder —
  // never on plain remounts (results of a run that finished while the user
  // was on another view must survive), and never mid-run for this folder.
  useEffect(() => {
    const s = useGlossaryRun.getState();
    if (s.folder !== null && s.folder !== workdir && !s.busy) s.reset();
  }, [workdir]);

  // O15 — watch glossary.json for external edits while this view is mounted.
  useEffect(() => {
    if (!workdir) return;
    void ipc.watchGlossary(workdir);
    return () => {
      void ipc.unwatchGlossary();
    };
  }, [workdir]);

  // Build completion / external edits → refetch the glossary.
  // Self-save suppression (I3): skip if the tick was caused by our own persist()
  // rename — the optimistic setQueryData is already up to date.
  useEffect(() => {
    if (!workdir || fileTick === 0) return;
    if (Date.now() - lastLocalSaveAt < 1500) return;
    void qc.invalidateQueries({ queryKey: glossaryKey(workdir) });
  }, [fileTick, workdir, qc]);

  if (!workdir) return <EmptyState title="Glossary" description="Open a folder first." />;
  if (view && !view.supports_glossary) {
    return (
      <EmptyState
        title="Glossary"
        description="Glossary extraction isn't available for this source language — it currently supports Chinese sources."
      />
    );
  }
  if (busy === "build") return <BuildProgress />;
  if (!view || isPending) return null;
  if (!doc || doc.count === 0) return <CreateView view={view} />;
  return <EditorView view={view} doc={doc} />;
}
