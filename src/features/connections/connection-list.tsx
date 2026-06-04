import { Plus } from "@phosphor-icons/react";
import type { ConnectionsView } from "@/types/generated/ConnectionsView";
import { cn } from "@/lib/utils";

export function ConnectionList({
  view,
  selected,
  onSelect,
  onAdd,
}: {
  view: ConnectionsView | undefined;
  selected: string | null;
  onSelect: (name: string) => void;
  onAdd: () => void;
}) {
  return (
    <div className="w-52 shrink-0 border-r border-border bg-[color:var(--color-bg-deepest)] p-2.5">
      {view?.connections.map((c) => (
        <button
          key={c.name}
          type="button"
          onClick={() => onSelect(c.name)}
          className={cn(
            "mb-0.5 flex w-full items-center gap-2 rounded-md px-2.5 py-1.5 text-left text-xs",
            selected === c.name
              ? "bg-[color:var(--popover)] text-foreground"
              : "text-muted-foreground hover:bg-[color:var(--color-bg-hover)]",
          )}
        >
          <span className="truncate">{c.name}</span>
          {view.active === c.name ? (
            <span className="ml-auto text-[9px] text-[color:var(--color-success)]">
              ● active
            </span>
          ) : null}
        </button>
      ))}
      <button
        type="button"
        onClick={onAdd}
        className="mt-1 flex w-full items-center gap-1.5 border-t border-border px-2.5 py-2 text-[11px] text-muted-foreground hover:text-foreground"
      >
        <Plus className="size-3.5" /> Add connection
      </button>
    </div>
  );
}
