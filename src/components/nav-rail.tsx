import {
  BookOpen,
  Folder,
  Gear,
  Lightning,
  Play,
  Question,
  type Icon,
} from "@phosphor-icons/react";
import { Link, useRouterState } from "@tanstack/react-router";
import { useAppStore } from "@/stores/app-store";
import { cn } from "@/lib/utils";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";

interface RailItem {
  to: string;
  label: string;
  icon: Icon;
  group: "workflow" | "setup";
  needsFolder?: boolean;
}

const ITEMS: RailItem[] = [
  { to: "/project", label: "Project", icon: Folder, group: "workflow", needsFolder: true },
  { to: "/glossary", label: "Glossary", icon: BookOpen, group: "workflow", needsFolder: true },
  { to: "/translate", label: "Translate", icon: Play, group: "workflow", needsFolder: true },
  { to: "/connections", label: "Connections", icon: Lightning, group: "setup" },
  { to: "/settings", label: "Settings", icon: Gear, group: "setup" },
  { to: "/help", label: "Help", icon: Question, group: "setup" },
];

export function NavRail() {
  const workdir = useAppStore((s) => s.workdir);
  const hasUsableConnection = useAppStore((s) => s.hasUsableConnection);
  const hasUntranslated = useAppStore((s) => s.hasUntranslated);
  const glossaryTerms = useAppStore((s) => s.glossaryTerms);
  const pathname = useRouterState({ select: (s) => s.location.pathname });

  // Returns a gating hint when the destination is disabled, else null.
  const gateHint = (item: RailItem): string | null => {
    if (!item.needsFolder) return null;
    if (!workdir) return "Open a folder first";
    if (item.to === "/translate") {
      if (!hasUsableConnection) return "Connect an AI provider";
      if (!hasUntranslated) return "No untranslated files in this folder";
    }
    return null;
  };

  const workflow = ITEMS.filter((i) => i.group === "workflow");
  const setup = ITEMS.filter((i) => i.group === "setup");

  const render = (item: RailItem) => {
    const hint = gateHint(item);
    const disabled = hint !== null;
    const active = pathname.startsWith(item.to);
    const Icon = item.icon;
    const body = (
      <div
        className={cn(
          "flex w-16 flex-col items-center gap-1 rounded-md py-2 text-[10px]",
          active && "bg-[color:var(--popover)] text-primary",
          disabled
            ? "cursor-not-allowed text-muted-foreground/50"
            : "hover:bg-[color:var(--color-bg-hover)]",
        )}
      >
        <Icon weight={active ? "fill" : "regular"} className="size-5" />
        {item.label}
        {item.to === "/connections" ? (
          <span
            className={
              hasUsableConnection
                ? "text-[color:var(--color-success)]"
                : "text-[color:var(--color-alert)]"
            }
          >
            {hasUsableConnection ? "✓" : "⚠"}
          </span>
        ) : null}
        {item.to === "/glossary" && glossaryTerms ? (
          <span className="text-[9px] tabular-nums text-muted-foreground">{glossaryTerms}</span>
        ) : null}
      </div>
    );
    if (disabled) {
      return (
        <Tooltip key={item.to}>
          <TooltipTrigger asChild>
            <div>{body}</div>
          </TooltipTrigger>
          <TooltipContent side="right">{hint}</TooltipContent>
        </Tooltip>
      );
    }
    return (
      <Link key={item.to} to={item.to as never}>
        {body}
      </Link>
    );
  };

  return (
    <nav className="flex w-20 flex-col items-center gap-1 border-r border-border bg-[color:var(--sidebar)] py-3">
      {workflow.map(render)}
      <div className="my-1 h-px w-10 bg-border" />
      {render(setup[0])}
      <div className="flex-1" />
      {setup.slice(1).map(render)}
    </nav>
  );
}
