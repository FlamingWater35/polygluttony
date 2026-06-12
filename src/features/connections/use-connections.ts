import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import type { Connection } from "@/types/generated/Connection";
import { ipc } from "@/lib/ipc";
import { useAppStore } from "@/stores/app-store";

const KEY = ["connections"] as const;

export function useConnections() {
  const setActive = useAppStore((s) => s.setActiveConnection);
  const setHasUsable = useAppStore((s) => s.setHasUsableConnection);
  return useQuery({
    queryKey: KEY,
    queryFn: async () => {
      const view = await ipc.listConnections();
      setActive(view.active);
      setHasUsable(view.connections.some((c) => c.has_key));
      return view;
    },
  });
}

export function usePresets() {
  return useQuery({ queryKey: ["presets"], queryFn: ipc.listPresets, staleTime: Infinity });
}

export function useConnection(name: string | null) {
  return useQuery({
    queryKey: ["connection", name],
    queryFn: () => ipc.readConnection(name as string),
    // Synthetic "new-*" names aren't persisted yet — don't fetch (or retry) them.
    enabled: !!name && !name.startsWith("new-"),
    retry: false,
  });
}

export function useConnectionMutations() {
  const qc = useQueryClient();
  const invalidate = () => {
    qc.invalidateQueries({ queryKey: KEY });
    // Glossary view gates "look up names online" on this; keep it fresh when connections change.
    qc.invalidateQueries({ queryKey: ["personalization-status"] });
  };
  return {
    save: useMutation({
      mutationFn: ({ name, connection }: { name: string; connection: Connection }) =>
        ipc.saveConnection(name, connection),
      onSuccess: invalidate,
    }),
    remove: useMutation({ mutationFn: ipc.deleteConnection, onSuccess: invalidate }),
    rename: useMutation({
      mutationFn: ({ oldName, newName }: { oldName: string; newName: string }) =>
        ipc.renameConnection(oldName, newName),
      onSuccess: invalidate,
    }),
    setActive: useMutation({ mutationFn: ipc.setActiveConnection, onSuccess: invalidate }),
    setPersonalization: useMutation({
      mutationFn: ipc.setPersonalizationConnection,
      onSuccess: invalidate,
    }),
    clearPersonalization: useMutation({
      mutationFn: ipc.clearPersonalizationConnection,
      onSuccess: invalidate,
    }),
    test: useMutation({
      mutationFn: ({ connection, detect }: { connection: Connection; detect: boolean }) =>
        ipc.testConnection(connection, detect),
    }),
    listModels: useMutation({
      mutationFn: ({ connection, detect }: { connection: Connection; detect: boolean }) =>
        ipc.listModels(connection, detect),
    }),
  };
}
