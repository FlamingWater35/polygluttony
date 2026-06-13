const STATE_COLOR: Record<string, string> = {
  idle: "var(--muted-foreground)",
  pending: "var(--muted-foreground)",
  translating: "var(--color-gold)",
  retranslating: "var(--color-amber)",
  cleanup: "var(--color-state-cleanup)",
  verifying: "var(--color-state-verify)",
  done: "var(--color-success)",
  warning: "var(--color-alert)",
  failed: "var(--color-danger)",
};
const LIVE = new Set(["translating", "retranslating", "cleanup", "verifying"]);

export function StateChip({ state = "idle", label }: { state?: string; label?: string }) {
  const color = STATE_COLOR[state] ?? STATE_COLOR.idle;
  return (
    <span className="inline-flex items-center gap-1.5 rounded-[13px] border border-border px-2.5 py-0.5 text-[11px]">
      <span className={`size-2 rounded-full${LIVE.has(state) ? " [animation:sig-ping_1.5s_ease-out_infinite]" : ""}`} style={{ background: color }} />
      {label ?? state[0].toUpperCase() + state.slice(1)}
    </span>
  );
}
