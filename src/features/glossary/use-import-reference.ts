import { useQueryClient } from "@tanstack/react-query";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { toast } from "sonner";
import { ipc } from "@/lib/ipc";
import { useAppStore } from "@/stores/app-store";
import { useGlossaryRun } from "@/stores/glossary-store";

// ── reference query keys (shared by create card, review screen, editor) ───────

export function referenceStatusKey(folder: string) {
  return ["reference-status", folder] as const;
}

export function referenceKey(folder: string) {
  return ["reference", folder] as const;
}

/** Pick translated .ass files → full-screen import run → reference review.
 *  Shared by CreateView's import card and the review screen's empty state.
 *  NO success/warning toasts — the review screen IS the result. */
export function useImportReference(folder: string) {
  const qc = useQueryClient();
  return async () => {
    const paths = await openDialog({
      multiple: true,
      filters: [{ name: "ASS subtitles", extensions: ["ass"] }],
    });
    if (!paths || (Array.isArray(paths) && paths.length === 0)) return;
    const fileList = Array.isArray(paths) ? paths : [paths];

    // A run may have started while the dialog was open — don't clobber it.
    const store = useGlossaryRun.getState();
    if (store.busy !== null) return;
    store.startOp("import", folder, `${fileList.length} translated files`);
    try {
      const summary = await ipc.importReferenceFiles(folder, fileList);
      await qc.invalidateQueries({ queryKey: referenceStatusKey(folder) });
      await qc.invalidateQueries({ queryKey: referenceKey(folder) });
      // User switched folders mid-import — don't open another folder's review over this one.
      if (useAppStore.getState().workdir !== folder) return;
      // Land on the review — it shows counts, errors, and cancellation.
      useGlossaryRun.getState().openReview(folder, summary);
    } catch (e: unknown) {
      toast.error(String(e));
    } finally {
      useGlossaryRun.getState().endOp();
    }
  };
}
