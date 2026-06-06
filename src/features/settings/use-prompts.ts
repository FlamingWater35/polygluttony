import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import type { PromptId } from "@/types/generated/PromptId";
import { ipc } from "@/lib/ipc";

const KEY = ["prompts"] as const;

export function usePrompts() {
  return useQuery({ queryKey: KEY, queryFn: ipc.listPrompts });
}

export function usePromptText(id: PromptId | null) {
  return useQuery({
    queryKey: ["prompt", id],
    queryFn: () => ipc.getPrompt(id as PromptId),
    enabled: !!id,
  });
}

export function usePromptMutations() {
  const qc = useQueryClient();
  const invalidate = (id: PromptId) => {
    qc.invalidateQueries({ queryKey: KEY });
    qc.invalidateQueries({ queryKey: ["prompt", id] });
  };
  return {
    save: useMutation({
      mutationFn: ({ id, text }: { id: PromptId; text: string }) => ipc.savePrompt(id, text),
      onSuccess: (_void, v) => invalidate(v.id),
    }),
    reset: useMutation({
      mutationFn: (id: PromptId) => ipc.resetPrompt(id),
      onSuccess: (_text, id) => invalidate(id),
    }),
  };
}
