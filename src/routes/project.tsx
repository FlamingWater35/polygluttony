import { createFileRoute } from "@tanstack/react-router";
import { EmptyState } from "@/components/empty-state";

export const Route = createFileRoute("/project")({
  component: () => (
    <EmptyState title="Project" description="Open a folder first." />
  ),
});
