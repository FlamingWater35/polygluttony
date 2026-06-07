import type { ComponentProps } from "react";
import { useId } from "react";
import type { UseFormReturn } from "react-hook-form";
import type { Connection } from "@/types/generated/Connection";
import { Input } from "@/components/ui/input";
import { Checkbox } from "@/components/ui/checkbox";
import { HelpText } from "@/components/help-text";
import { SectionHelp } from "@/components/section-help";
import { cn } from "@/lib/utils";

/** Labelled input with help text and optional error/warning lines. */
export function AdvField({
  label,
  help,
  error,
  warn,
  className,
  ...rest
}: {
  label: string;
  help: string;
  error?: string;
  warn?: string;
} & ComponentProps<"input">) {
  const id = useId();
  const descId = `${id}-desc`;
  return (
    <div className="flex flex-col gap-1">
      <label htmlFor={id} className="text-muted-foreground">{label}</label>
      <Input
        id={id}
        className={cn("h-8", className)}
        aria-invalid={error ? true : undefined}
        aria-describedby={descId}
        {...rest}
      />
      <div id={descId}>
        {error ? (
          <span className="text-[10.5px] text-[color:var(--color-danger)]">{error}</span>
        ) : null}
        <HelpText>{help}</HelpText>
        {warn ? (
          <span className="text-[10.5px] text-[color:var(--color-alert)]">⚠ {warn}</span>
        ) : null}
      </div>
    </div>
  );
}

/** Timeout + throughput fields. Flat grids with a hairline divider. */
export function AdvancedSettingsSection({ form }: { form: UseFormReturn<Connection> }) {
  const { register } = form;
  return (
    <SectionHelp
      title="Advanced settings"
      hint="(tokens, parallelism, timeouts)"
    >
      <div className="grid grid-cols-2 gap-2 text-[11px]">
        <AdvField
          label="Timeout (s)"
          type="number"
          help="How long to wait for each response."
          {...register("timeout", { valueAsNumber: true })}
        />
        <AdvField
          label="Connect timeout (s)"
          type="number"
          help="How long to wait to reach the server."
          {...register("connect_timeout", { valueAsNumber: true })}
        />
      </div>
      <div className="my-3 border-t border-border" />
      <div className="grid grid-cols-2 gap-2 text-[11px]">
        <AdvField
          label="Max tokens"
          type="number"
          help="Response size cap. Too low cuts off long batches."
          {...register("max_tokens", { valueAsNumber: true, deps: [...BUDGET_FIELDS] })}
        />
        <AdvField
          label="Batch dialogue limit"
          type="number"
          help="Subtitle lines per request. Lower = smaller, safer batches."
          {...register("batch_dialogue_limit", { valueAsNumber: true })}
        />
        <AdvField
          label="Concurrency"
          type="number"
          help="Parallel requests. Faster, but may hit rate limits."
          {...register("concurrency", { valueAsNumber: true })}
        />
      </div>
    </SectionHelp>
  );
}

const MIN_BUDGET = 1024; // Anthropic API floor for thinking.budget_tokens

export const BUDGET_FIELDS = [
  "thinking_budget",
  "thinking_glossary_budget",
  "thinking_glossary_norm_budget",
] as const;
type BudgetField = (typeof BUDGET_FIELDS)[number];

/** Numeric value or null — react-hook-form's valueAsNumber yields NaN for an empty input. */
function numOrNull(v: unknown): number | null {
  return typeof v === "number" && !Number.isNaN(v) ? v : null;
}

/** First-enable seed: a quarter of Max tokens, floored at the API minimum. */
function seedBudget(maxTokens: number | null): number {
  return Math.max(MIN_BUDGET, Math.floor((maxTokens ?? 16000) / 4));
}

/** Hard validation (blocks save): required · ≥1024 · integer · < Max tokens. */
export function budgetValidate(v: unknown, values: Connection): true | string {
  if (values.driver !== "anthropic" || !values.thinking_enabled) return true;
  const n = numOrNull(v);
  if (n == null) return "Required when thinking is enabled.";
  if (n < MIN_BUDGET) return `Minimum ${MIN_BUDGET} tokens.`;
  if (!Number.isInteger(n)) return "Must be a whole number.";
  const max = numOrNull(values.max_tokens);
  if (max != null && n >= max) return `Must be less than Max tokens (${max}).`;
  return true;
}

/** Soft warning (save allowed): more than half of Max tokens. */
function budgetWarning(v: unknown, maxTokens: unknown): string | undefined {
  const n = numOrNull(v);
  const max = numOrNull(maxTokens);
  return n != null && max != null && n > max / 2
    ? `More than half of Max tokens (${max}) — output may get cropped before it completes.`
    : undefined;
}

const BUDGETS: { field: BudgetField; label: string; help: string }[] = [
  {
    field: "thinking_budget",
    label: "Translate thinking *",
    help: "Reasoning tokens for translation, verification and cleanup — the more budget, the better the translation.",
  },
  {
    field: "thinking_glossary_budget",
    label: "Glossary thinking *",
    help: "Reasoning tokens for glossary extraction — more budget finds more consistent terms.",
  },
  {
    field: "thinking_glossary_norm_budget",
    label: "Normalization thinking *",
    help: "Reasoning tokens for glossary normalization — more budget merges terms more reliably.",
  },
];

/** Thinking checkbox + the three required, vertically stacked budget inputs. */
export function ExtendedThinkingSection({ form }: { form: UseFormReturn<Connection> }) {
  const { register, setValue, watch, formState } = form;
  const current = watch();
  const supported = current.driver === "anthropic";
  const enabled = supported && !!current.thinking_enabled;

  const onToggle = (checked: boolean) => {
    setValue("thinking_enabled", checked, { shouldDirty: true });
    if (!checked) return; // values are kept; re-checking restores them
    // Seed every empty budget with the same number: the translate budget if
    // one is stored, else a quarter of Max tokens.
    const base =
      numOrNull(current.thinking_budget) ?? seedBudget(numOrNull(current.max_tokens));
    for (const f of BUDGET_FIELDS) {
      if (numOrNull(current[f]) == null) setValue(f, base, { shouldDirty: true });
    }
  };

  return (
    <SectionHelp title="Extended thinking" hint="(reasoning budgets)">
      <div className={supported ? "" : "pointer-events-none opacity-50"}>
        <label className="flex items-center gap-2 text-[11.5px]">
          <Checkbox
            checked={enabled}
            disabled={!supported}
            onCheckedChange={(c) => onToggle(c === true)}
          />
          Thinking enabled
        </label>
        <HelpText>
          The model reasons before answering — better consistency on tricky
          dialogue, slower and costlier.
        </HelpText>
        {enabled ? (
          <div className="ml-5 mt-2 space-y-2 border-l-2 border-border pl-3 text-[11px]">
            <p className="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground">
              Thinking budgets
            </p>
            {BUDGETS.map(({ field, label, help }) => (
              <AdvField
                key={field}
                label={label}
                type="number"
                min={1024}
                step={1}
                help={help}
                error={formState.errors[field]?.message as string | undefined}
                warn={budgetWarning(current[field], current.max_tokens)}
                {...register(field, { valueAsNumber: true, validate: budgetValidate })}
              />
            ))}
          </div>
        ) : null}
      </div>
      {!supported ? (
        <p className="mt-1 text-[11px] text-[color:var(--color-alert)]">
          ⚠ Not supported by this provider — available on Anthropic-compatible
          connections.
        </p>
      ) : null}
    </SectionHelp>
  );
}
