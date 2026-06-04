import { createFileRoute, redirect } from "@tanstack/react-router";
import { ipc } from "@/lib/ipc";
import { EmptyState } from "@/components/empty-state";

export const Route = createFileRoute("/")({
  beforeLoad: async () => {
    const status = await ipc.firstRunStatus();
    if (!status.has_usable_connection) {
      throw redirect({ to: "/connections" });
    }
  },
  component: () => (
    <EmptyState
      title="Open a folder of subtitles to begin"
      description="Folder pickup arrives in the next step. For now, manage your AI providers in Connections."
    />
  ),
});
