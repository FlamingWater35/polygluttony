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

  // O15 — watch glossary.json for external edits while this view is mounted.
  useEffect(() => {
    if (!workdir) return;
    void ipc.watchGlossary(workdir);
    return () => {
      void ipc.unwatchGlossary();
    };
  }, [workdir]);

  // Build completion / external edits → refetch the glossary.
  useEffect(() => {
    if (workdir && fileTick > 0) void qc.invalidateQueries({ queryKey: glossaryKey(workdir) });
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
