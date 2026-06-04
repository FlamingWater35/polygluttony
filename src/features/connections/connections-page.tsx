import { useEffect, useState } from "react";
import { toast } from "sonner";
import { PageHeader } from "@/components/page-header";
import {
  useConnections,
  useConnection,
  usePresets,
  useConnectionMutations,
} from "./use-connections";
import { ConnectionList } from "./connection-list";
import { ConnectionEditor } from "./connection-editor";

export function ConnectionsPage() {
  const { data: view } = useConnections();
  const { data: presets } = usePresets();
  const m = useConnectionMutations();
  const [selected, setSelected] = useState<string | null>(null);
  const { data: initial } = useConnection(selected);

  useEffect(() => {
    if (!selected && view?.connections.length) {
      setSelected(view.active || view.connections[0].name);
    }
  }, [view, selected]);

  const firstRun = view && !view.connections.some((c) => c.has_key);

  return (
    <div className="flex h-full flex-col">
      <PageHeader
        title="LLM Connections"
        description="An AI provider does the translating. Pick one, paste a key, test."
      />
      {firstRun ? (
        <div className="border-b border-border bg-[color:var(--popover)] px-5 py-2 text-[12.5px] text-primary">
          Welcome — let&apos;s connect an AI provider so you can start translating.
        </div>
      ) : null}
      <div className="flex min-h-0 flex-1">
        <ConnectionList
          view={view}
          selected={selected}
          onSelect={setSelected}
          onAdd={() => setSelected(`new-${Date.now()}`)}
        />
        {selected && presets ? (
          <ConnectionEditor
            name={selected}
            initial={initial}
            presets={presets}
            isActive={view?.active === selected}
            isPersonalization={view?.personalization === selected}
            onSave={async (name, c) => {
              await m.save.mutateAsync({ name, connection: c });
              setSelected(name);
              toast.success(`Saved “${name}”`);
            }}
            onSetActive={(name) =>
              m.setActive.mutate(name, {
                onSuccess: () => toast.success(`“${name}” is now active`),
              })
            }
            onSetPersonalization={(name) => m.setPersonalization.mutate(name)}
            onRemove={(name) => {
              m.remove.mutate(name, {
                onSuccess: () => toast.success(`Removed “${name}”`),
              });
              setSelected(null);
            }}
            onRename={async (oldName, newName) => {
              await m.rename.mutateAsync({ oldName, newName });
              setSelected(newName);
            }}
            onTest={(c) => m.test.mutateAsync(c)}
            onListModels={(c) => m.listModels.mutateAsync(c)}
          />
        ) : null}
      </div>
    </div>
  );
}
