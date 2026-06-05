import { createFileRoute } from "@tanstack/react-router";
import { useAppStore } from "@/stores/app-store";
import { TranslatePage } from "@/features/translate/translate-page";
import { EmptyState } from "@/components/empty-state";

function TranslateRoute() {
  const workdir = useAppStore((s) => s.workdir);
  if (!workdir) {
    return <EmptyState title="Translate" description="Open a folder first." />;
  }
  return <TranslatePage />;
}

export const Route = createFileRoute("/translate")({
  component: TranslateRoute,
});
