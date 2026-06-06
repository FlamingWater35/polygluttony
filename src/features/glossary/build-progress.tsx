import type { GlossaryPhase } from "@/types/generated/GlossaryPhase";
import { useGlossaryRun } from "@/stores/glossary-store";
import { RunScreen } from "./run-screen";

const PHASE_LABELS: Record<GlossaryPhase, string> = {
  loading: "Reading subtitle files…",
  reference: "Gathering reference terminology…",
  extracting: "Extracting terms…",
  normalizing: "Cleaning up & standardizing…",
  personalizing: "Looking up established names…",
  saving: "Saving glossary…",
};

export function BuildProgress() {
  const phase = useGlossaryRun((s) => s.phase);
  const phaseDetail = useGlossaryRun((s) => s.phaseDetail);
  return (
    <RunScreen
      title="Building glossary"
      description="Extracting names, terms & places from your subtitles."
      cancelNote="Partial results are kept — cancelling never throws away extracted terms."
      phaseLine={
        <p className="mb-2 text-sm">
          {phase ? PHASE_LABELS[phase] : "Starting…"}
          {phaseDetail ? <span className="text-muted-foreground"> — {phaseDetail}</span> : null}
        </p>
      }
    />
  );
}
