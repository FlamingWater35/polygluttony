import type { GlossaryDiff } from "@/types/generated/GlossaryDiff";
import type { DiffStatus } from "@/types/generated/DiffStatus";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";

const STATUS_CLS: Record<DiffStatus, string> = {
  added: "text-[color:var(--color-success)]",
  removed: "text-[color:var(--color-danger)] line-through",
  modified: "text-[color:var(--color-alert)]",
  unchanged: "text-muted-foreground",
};

/**
 * Diff dialog, two flavors:
 * - `review` (O12 Normalize): nothing is saved yet — Accept saves, Discard drops.
 * - `info` (O13 View changes / post-build): read-only, the diff already happened.
 */
export function DiffReview({
  diff,
  mode,
  onAccept,
  onClose,
}: {
  diff: GlossaryDiff;
  mode: "review" | "info";
  onAccept?: () => void;
  onClose: () => void;
}) {
  const summary = [
    diff.total_added ? `${diff.total_added} added` : null,
    diff.total_removed ? `${diff.total_removed} removed` : null,
    diff.total_modified ? `${diff.total_modified} modified` : null,
  ]
    .filter(Boolean)
    .join(", ");

  return (
    <Dialog open onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="max-h-[80vh] overflow-hidden sm:max-w-2xl">
        <DialogHeader>
          <DialogTitle>
            {mode === "review" ? "Review normalization" : "Glossary changes"}
          </DialogTitle>
          <DialogDescription>{summary || "No changes."}</DialogDescription>
        </DialogHeader>
        <div className="max-h-[55vh] overflow-auto">
          {diff.categories
            .filter((c) => c.added + c.removed + c.modified > 0)
            .map((c) => (
              <div key={c.name} className="mb-3">
                <div className="mb-1 text-[11px] font-semibold uppercase tracking-wide text-muted-foreground">
                  {c.name} · {c.added} added, {c.removed} removed, {c.modified} modified
                </div>
                {c.terms
                  .filter((t) => t.status !== "unchanged")
                  .map((t) => (
                    <div
                      key={t.source}
                      className="grid grid-cols-[1fr_auto_1fr] gap-2 py-0.5 text-[12.5px]"
                    >
                      <span className="truncate">{t.source}</span>
                      <span className={`${STATUS_CLS[t.status]} min-w-0 truncate`}>
                        {t.status === "modified" ? `${t.old} →` : t.status}
                      </span>
                      <span className="truncate">{t.new ?? t.old}</span>
                    </div>
                  ))}
              </div>
            ))}
        </div>
        <DialogFooter>
          {mode === "review" ? (
            <>
              <Button variant="secondary" onClick={onClose}>
                Discard
              </Button>
              <Button onClick={onAccept}>Accept changes</Button>
            </>
          ) : (
            <Button onClick={onClose}>Close</Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
