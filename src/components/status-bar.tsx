import { useQuery } from "@tanstack/react-query";
import { Folder } from "@phosphor-icons/react";
import { StateChip } from "@/components/state-chip";
import { StatusChip } from "@/components/status-chip";
import { Separator } from "@/components/ui/separator";
import { ipc } from "@/lib/ipc";
import { useAppStore } from "@/stores/app-store";

export function StatusBar() {
  const workdir = useAppStore((s) => s.workdir);
  const sourceLang = useAppStore((s) => s.sourceLang);
  const targetLang = useAppStore((s) => s.targetLang);
  const connection = useAppStore((s) => s.activeConnection);
  const { data: appInfo } = useQuery({ queryKey: ["app-info"], queryFn: ipc.appInfo });

  return (
    <footer className="col-start-2 flex h-8 items-center gap-3 border-t border-border bg-[color:var(--color-bg-deepest)] px-3 text-[11px] text-muted-foreground">
      <span className="flex min-w-0 items-center gap-1.5">
        <Folder className="size-3.5 shrink-0" />
        <span className="truncate">{workdir ?? "No folder selected"}</span>
      </span>
      <Separator orientation="vertical" className="h-4" />
      <span className="shrink-0 tabular-nums">— files · — lines</span>
      <span className="flex-1" />
      <span className="shrink-0">
        {sourceLang}→{targetLang}
      </span>
      <StatusChip variant="accent">{connection ?? "No connection"}</StatusChip>
      <StateChip state="idle" />
      <span className="shrink-0 opacity-60">core {appInfo?.version ?? "…"}</span>
    </footer>
  );
}
