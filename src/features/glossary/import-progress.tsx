import { useGlossaryRun } from "@/stores/glossary-store";
import { RunScreen } from "./run-screen";

export function ImportProgress() {
  const opDetail = useGlossaryRun((s) => s.opDetail);
  return (
    <RunScreen
      title="Importing reference terms"
      description={`Lifting established terminology from ${opDetail ?? "your translated files"}.`}
      cancelNote="Partial results are kept — cancelling never throws away terms."
    />
  );
}
