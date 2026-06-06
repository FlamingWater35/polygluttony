import type { ReactNode } from "react";
import { CaretDown, CaretRight } from "@phosphor-icons/react";

export const LOG_COLOR: Record<string, string> = {
  debug: "text-muted-foreground",
  info: "text-foreground",
  warning: "text-[color:var(--color-alert)]",
  error: "text-[color:var(--color-danger)]",
};

export interface LogDrawerLine {
  /** HH:MM:SS receive-time stamp. */
  at: string;
  level: string;
  message: string;
  /** Optional muted segments rendered between the stamp and the message
   *  (Translate uses [file, phase]). */
  meta?: ReactNode[];
}

/** The right-aligned "▸ Logs (N)" toggle for an action row. */
export function LogToggleButton({
  open,
  count,
  onToggle,
}: {
  open: boolean;
  count: number;
  onToggle: () => void;
}) {
  return (
    <button
      type="button"
      className="flex items-center gap-1.5 text-[12px] text-muted-foreground hover:text-foreground transition-colors"
      onClick={onToggle}
    >
      {open ? <CaretDown className="size-3.5" /> : <CaretRight className="size-3.5" />}
      Logs
      {count > 0 ? <span className="ml-1 tabular-nums text-[11px]">({count})</span> : null}
    </button>
  );
}

/** The drawer panel rendered below the action row while open. */
export function LogPanel({ lines }: { lines: LogDrawerLine[] }) {
  return (
    <div className="max-h-48 overflow-auto border-t border-border bg-[color:var(--color-bg-deepest)] px-4 py-3 font-mono text-[11px]">
      {lines.length === 0 ? (
        <span className="text-muted-foreground">No logs yet.</span>
      ) : (
        lines.map((l, i) => (
          <div key={i} className="flex gap-2 leading-5">
            <span className="shrink-0 text-muted-foreground/60 tabular-nums">[{l.at}]</span>
            {l.meta}
            <span className={LOG_COLOR[l.level] ?? "text-foreground"}>{l.message}</span>
          </div>
        ))
      )}
    </div>
  );
}
