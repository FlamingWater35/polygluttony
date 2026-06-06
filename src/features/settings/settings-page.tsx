import { useEffect, useState } from "react";
import { toast } from "sonner";
import type { PromptId } from "@/types/generated/PromptId";
import { PageHeader } from "@/components/page-header";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { usePrompts, usePromptText, usePromptMutations } from "./use-prompts";
import { PromptList } from "./prompt-list";
import { PromptEditor } from "./prompt-editor";

export function SettingsPage() {
  const { data: prompts } = usePrompts();
  const m = usePromptMutations();
  const [selected, setSelected] = useState<PromptId | null>(null);
  const [draft, setDraft] = useState<string | null>(null);
  const [pendingSelect, setPendingSelect] = useState<PromptId | null>(null);
  const { data: loaded } = usePromptText(selected);

  useEffect(() => {
    if (!selected && prompts?.length) setSelected(prompts[0].id);
  }, [prompts, selected]);

  const meta = prompts?.find((p) => p.id === selected);
  const dirty = draft !== null && loaded !== undefined && draft !== loaded;

  const select = (id: PromptId) => {
    if (id === selected) return;
    if (dirty) {
      setPendingSelect(id);
      return;
    }
    setSelected(id);
    setDraft(null);
  };

  return (
    <div className="flex h-full flex-col">
      <PageHeader
        title="Settings"
        description="Customize the prompts sent to the AI. Restore any prompt to its default at any time."
      />
      <div className="flex min-h-0 flex-1">
        <PromptList prompts={prompts} selected={selected} onSelect={select} />
        {meta && loaded !== undefined ? (
          <PromptEditor
            key={meta.id}
            meta={meta}
            loaded={loaded}
            draft={draft}
            onDraftChange={setDraft}
            onSave={async (text) => {
              try {
                await m.save.mutateAsync({ id: meta.id, text });
                setDraft(null);
                toast.success(`Saved "${meta.name}"`);
              } catch (e) {
                toast.error(String(e));
              }
            }}
            onReset={async () => {
              try {
                await m.reset.mutateAsync(meta.id);
                setDraft(null);
                toast.success(`"${meta.name}" restored to default`);
              } catch (e) {
                toast.error(String(e));
              }
            }}
          />
        ) : null}
      </div>

      <AlertDialog
        open={pendingSelect !== null}
        onOpenChange={(open) => {
          if (!open) setPendingSelect(null);
        }}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Discard unsaved changes?</AlertDialogTitle>
            <AlertDialogDescription>
              &ldquo;{meta?.name}&rdquo; has edits you haven&apos;t saved. Switching prompts will
              discard them.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Keep editing</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => {
                if (pendingSelect) {
                  setSelected(pendingSelect);
                  setDraft(null);
                  setPendingSelect(null);
                }
              }}
            >
              Discard changes
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}
