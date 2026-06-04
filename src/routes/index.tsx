import { createFileRoute, redirect } from "@tanstack/react-router";
import { ipc } from "@/lib/ipc";
import { EmptyState } from "@/components/empty-state";
import { useAppStore } from "@/stores/app-store";

export const Route = createFileRoute("/")({
  beforeLoad: async () => {
    const status = await ipc.firstRunStatus();
    // Seed the rail badge before first render so it's correct without first
    // visiting Connections (useConnections also keeps it in sync afterwards).
    useAppStore.getState().setHasUsableConnection(status.has_usable_connection);
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
