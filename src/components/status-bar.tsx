import { useQuery } from "@tanstack/react-query";
import { Folder } from "@phosphor-icons/react";
import { StateChip } from "@/components/state-chip";
import { StatusChip } from "@/components/status-chip";
import { Separator } from "@/components/ui/separator";
import { ipc } from "@/lib/ipc";
import { useAppStore } from "@/stores/app-store";
import { useTranslationRun } from "@/stores/translation-store";

export function StatusBar() {
  const workdir = useAppStore((s) => s.workdir);
  const sourceLang = useAppStore((s) => s.sourceLang);
  const targetLang = useAppStore((s) => s.targetLang);
  const worldType = useAppStore((s) => s.worldType);
  const fileCount = useAppStore((s) => s.fileCount);
  const lineCount = useAppStore((s) => s.dialogueLineCount);
  const connection = useAppStore((s) => s.activeConnection);
  const translating = useTranslationRun((s) => s.running);
  const { data: appInfo } = useQuery({ queryKey: ["app-info"], queryFn: ipc.appInfo });

  return (
    <footer className="col-start-2 flex h-8 items-center gap-3 border-t border-border bg-[color:var(--color-bg-deepest)] px-3 text-[11px] text-muted-foreground">
      <span className="flex min-w-0 items-center gap-1.5">
        <Folder className="size-3.5 shrink-0" />
        <span className="truncate">{workdir ?? "No folder selected"}</span>
      </span>
      <Separator orientation="vertical" className="h-4" />
      <span className="shrink-0 tabular-nums">
        {workdir ? `${fileCount} files · ${lineCount} lines` : "— files · — lines"}
      </span>
      <span className="flex-1" />
      {translating ? (
        <StatusChip variant="alert">Translating…</StatusChip>
      ) : null}
      {worldType ? <StatusChip variant="muted">{worldType}</StatusChip> : null}
      <span className="shrink-0">
        {sourceLang}→{targetLang}
      </span>
      <StatusChip variant="accent">{connection ?? "No connection"}</StatusChip>
      <StateChip state={translating ? "translating" : "idle"} />
      <span className="shrink-0 opacity-60">core {appInfo?.version ?? "…"}</span>
    </footer>
  );
}
