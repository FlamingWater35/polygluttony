import { createFileRoute } from "@tanstack/react-router";
import { EmptyState } from "@/components/empty-state";

export const Route = createFileRoute("/connections")({
  component: () => (
    <EmptyState title="Connections" description="Coming next." />
  ),
});
