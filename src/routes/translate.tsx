import { createFileRoute } from "@tanstack/react-router";
import { EmptyState } from "@/components/empty-state";

export const Route = createFileRoute("/translate")({
  component: () => (
    <EmptyState title="Translate" description="Open a folder first." />
  ),
});
