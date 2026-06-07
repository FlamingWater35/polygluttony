import type { InputHTMLAttributes, ReactNode } from "react";
import type { UseFormReturn } from "react-hook-form";
import type { Connection } from "@/types/generated/Connection";
import { Input } from "@/components/ui/input";
import { HelpText } from "@/components/help-text";
import { SectionHelp } from "@/components/section-help";

/** Labelled input with help text and optional error/warning lines. */
export function AdvField({
  label,
  help,
  error,
  warn,
  ...rest
}: {
  label: string;
  help: string;
  error?: string;
  warn?: string;
} & InputHTMLAttributes<HTMLInputElement>) {
  return (
    <label className="flex flex-col gap-1">
      <span className="text-muted-foreground">{label}</span>
      <Input className="h-8" {...rest} />
      {error ? (
        <span className="text-[10.5px] text-[color:var(--color-danger)]">{error}</span>
      ) : null}
      <HelpText>{help}</HelpText>
      {warn ? <span className="text-[10.5px] text-amber-600">⚠ {warn}</span> : null}
    </label>
  );
}

/** Endpoint + throughput fields. Flat grids with a hairline divider. */
export function AdvancedSettingsSection({
  form,
  footer,
}: {
  form: UseFormReturn<Connection>;
  footer?: ReactNode;
}) {
  const { register } = form;
  return (
    <SectionHelp
      title="Advanced settings"
      hint="(address, tokens, parallelism, timeouts)"
    >
      <div className="grid grid-cols-2 gap-2 text-[11px]">
        <div className="col-span-2">
          <AdvField
            label="Base URL"
            help="Where requests are sent. Only change for proxies or alternative providers."
            {...register("base_url")}
          />
        </div>
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
          {...register("max_tokens", { valueAsNumber: true })}
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
      {footer}
    </SectionHelp>
  );
}
