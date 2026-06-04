import { createFileRoute } from "@tanstack/react-router";
import { EmptyState } from "@/components/empty-state";

export const Route = createFileRoute("/help")({
  component: () => <EmptyState title="Help" description="Coming soon." />,
});
