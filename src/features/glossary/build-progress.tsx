import { toast } from "sonner";
import type { GlossaryPhase } from "@/types/generated/GlossaryPhase";
import { ipc } from "@/lib/ipc";
import { useGlossaryRun } from "@/stores/glossary-store";
import { PageHeader } from "@/components/page-header";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";

const PHASE_LABELS: Record<GlossaryPhase, string> = {
  loading: "Reading subtitle files…",
  reference: "Gathering reference terminology…",
  extracting: "Extracting terms…",
  normalizing: "Cleaning up & standardizing…",
  personalizing: "Looking up established names…",
  saving: "Saving glossary…",
};

// Log level → className, matching translate-page.tsx conventions.
const LOG_COLOR: Record<string, string> = {
  debug: "text-muted-foreground",
  info: "text-foreground",
  warning: "text-[color:var(--color-alert)]",
  error: "text-[color:var(--color-danger)]",
};

export function BuildProgress() {
  const phase = useGlossaryRun((s) => s.phase);
  const phaseDetail = useGlossaryRun((s) => s.phaseDetail);
  const done = useGlossaryRun((s) => s.done);
  const total = useGlossaryRun((s) => s.total);
  const logs = useGlossaryRun((s) => s.logs);

  return (
    <div className="flex h-full flex-col">
      <PageHeader
        title="Building glossary"
        description="Extracting names, terms & places from your subtitles."
      />
      <div className="flex-1 overflow-auto p-5">
        <p className="mb-2 text-sm">
          {phase ? PHASE_LABELS[phase] : "Starting…"}
          {phaseDetail ? (
            <span className="text-muted-foreground"> — {phaseDetail}</span>
          ) : null}
        </p>
        <div className="mb-1 flex items-center gap-3">
          <Progress value={total > 0 ? (done / total) * 100 : 0} className="flex-1" />
          <span className="text-[11px] text-muted-foreground tabular-nums">
            {done}/{total} batches
          </span>
        </div>
        <div className="mt-4 max-h-[50vh] overflow-auto rounded-md border border-border bg-[color:var(--card)] p-2 font-mono text-[11px]">
          {logs.map((l, i) => (
            <div key={i} className={LOG_COLOR[l.level] ?? "text-foreground"}>
              {l.message}
            </div>
          ))}
        </div>
      </div>
      <div className="flex items-center gap-3 border-t border-border bg-[color:var(--popover)] px-5 py-3">
        <Button
          variant="secondary"
          onClick={() => ipc.cancelGlossaryBuild().catch((e: unknown) => toast.error(String(e)))}
        >
          Cancel
        </Button>
        <span className="text-[11px] text-muted-foreground">
          Partial results are kept — cancelling never throws away extracted terms.
        </span>
      </div>
    </div>
  );
}
