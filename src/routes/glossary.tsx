import { createFileRoute } from "@tanstack/react-router";
import { EmptyState } from "@/components/empty-state";

export const Route = createFileRoute("/glossary")({
  component: () => (
    <EmptyState title="Glossary" description="Open a folder first." />
  ),
});
