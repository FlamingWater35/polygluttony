import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { toast } from "sonner";
import type { ProjectView } from "@/types/generated/ProjectView";
import { ipc } from "@/lib/ipc";
import { useAppStore } from "@/stores/app-store";

export function projectKey(path: string) {
  return ["project", path] as const;
}

/** Apply a ProjectView to global shell state (status bar + rail gating). */
export function syncProjectStore(view: ProjectView) {
  const untranslated = view.files.filter((f) => !f.has_translation).length;
  useAppStore.getState().setProject({
    workdir: view.folder,
    sourceLang: view.prefs.source_lang,
    targetLang: view.prefs.target_lang,
    worldType: view.prefs.world_override ?? view.detected_world,
    tone: view.prefs.tone,
    fileCount: view.files.length,
    dialogueLineCount: view.total_dialogue_lines,
    hasUntranslated: untranslated > 0,
    glossaryTerms: view.glossary_terms,
  });
}

/**
 * Open a folder: discover/analyze, seed the project cache, sync the shell, and
 * navigate to Project. A folder with zero `.ass` files stays on Welcome (the
 * caller renders an inline message from `mutation.data`).
 */
export function useOpenFolder() {
  const qc = useQueryClient();
  const navigate = useNavigate();
  return useMutation({
    mutationFn: ipc.openFolder,
    onSuccess: (view) => {
      qc.invalidateQueries({ queryKey: ["recents"] });
      if (view.files.length === 0) return; // empty: stay, show inline message
      qc.setQueryData(projectKey(view.folder), view);
      syncProjectStore(view);
      void navigate({ to: "/project" });
    },
    onError: (e) => toast.error(String(e)),
  });
}

/** The current folder's ProjectView (seeded by useOpenFolder; refetches on reload). */
export function useProject(path: string) {
  return useQuery({
    queryKey: projectKey(path),
    queryFn: () => ipc.openFolder(path),
    enabled: !!path,
    staleTime: Infinity,
  });
}
