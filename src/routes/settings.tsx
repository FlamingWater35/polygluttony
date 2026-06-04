import { createFileRoute } from "@tanstack/react-router";
import { EmptyState } from "@/components/empty-state";

export const Route = createFileRoute("/settings")({
  component: () => (
    <EmptyState title="Settings" description="Coming soon." />
  ),
});
