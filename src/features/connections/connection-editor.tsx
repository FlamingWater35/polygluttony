import { useEffect, useMemo, useState } from "react";
import { useForm } from "react-hook-form";
import { Eye, EyeSlash } from "@phosphor-icons/react";
import type { Connection } from "@/types/generated/Connection";
import type { Preset } from "@/types/generated/Preset";
import type { TestResult } from "@/types/generated/TestResult";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Checkbox } from "@/components/ui/checkbox";
import { SetupField } from "@/components/setup-field";
import { HelpText } from "@/components/help-text";
import { SectionHelp } from "@/components/section-help";
import { ModelCombobox } from "./model-combobox";

const EMPTY: Connection = {
  driver: "openai",
  base_url: "",
  api_key: "",
  model: "",
  max_tokens: 16000,
  batch_dialogue_limit: 100,
  timeout: 120,
  connect_timeout: 10,
  concurrency: 5,
  thinking_enabled: null,
  thinking_budget: null,
  web_search: null,
  prompt_template: null,
  thinking_glossary_norm_budget: null,
} as unknown as Connection;

/**
 * Infer which provider preset a saved connection came from, by matching its
 * base URL. Anything that doesn't match a known preset is treated as Custom.
 */
function matchPresetKey(conn: Connection, presets: Preset[]): string {
  const hit = presets.find(
    (p) => p.key !== "custom" && p.base_url !== "" && p.base_url === conn.base_url,
  );
  return hit ? hit.key : "custom";
}

export function ConnectionEditor({
  name,
  initial,
  presets,
  isActive,
  isPersonalization,
  onSave,
  onSetActive,
  onSetPersonalization,
  onRemove,
  onRename,
  onTest,
  onListModels,
}: {
  name: string;
  initial: Connection | undefined;
  presets: Preset[];
  isActive: boolean;
  isPersonalization: boolean;
  onSave: (name: string, c: Connection) => Promise<void> | void;
  onSetActive: (name: string) => void;
  onSetPersonalization: (name: string) => void;
  onRemove: (name: string) => void;
  onRename: (oldName: string, newName: string) => Promise<void> | void;
  onTest: (c: Connection) => Promise<TestResult>;
  onListModels: (c: Connection) => Promise<string[]>;
}) {
  const { register, handleSubmit, watch, setValue, reset, formState } =
    useForm<Connection>({ defaultValues: initial ?? EMPTY });
  useEffect(() => {
    reset(initial ?? EMPTY);
    setPresetKey(initial ? matchPresetKey(initial, presets) : "");
    // Synthetic "new-*" selections start with an empty (user-supplied) name.
    setConnName(name.startsWith("new-") ? "" : name);
  }, [initial, name, reset, presets]);

  const [connName, setConnName] = useState<string>("");
  const [presetKey, setPresetKey] = useState<string>("");
  const [revealKey, setRevealKey] = useState(false);
  const [testState, setTestState] = useState<"idle" | "testing" | TestResult>("idle");
  const [liveModels, setLiveModels] = useState<string[]>([]);

  const current = watch();
  const isCustom = presetKey === "custom";
  const curated = useMemo(
    () => presets.find((p) => p.key === presetKey)?.models ?? [],
    [presets, presetKey],
  );

  const isNew = name.startsWith("new-");
  const nameChanged = connName.trim() !== (isNew ? "" : name);
  // Save is enabled only when there's a non-empty name AND something actually
  // changed (form fields dirty or the name edited), and not mid-submit.
  const canSave =
    connName.trim().length > 0 &&
    !formState.isSubmitting &&
    (isNew || formState.isDirty || nameChanged);

  const applyPreset = (key: string) => {
    setPresetKey(key);
    const p = presets.find((x) => x.key === key);
    if (!p) return;
    if (p.driver) setValue("driver", p.driver, { shouldDirty: true });
    setValue("base_url", p.base_url, { shouldDirty: true });
    if (p.model) setValue("model", p.model, { shouldDirty: true });
  };

  const withDetectSentinel = (c: Connection): Connection =>
    isCustom ? { ...c, prompt_template: "__detect__" } : c;

  const runTest = async () => {
    setTestState("testing");
    try {
      const res = await onTest(withDetectSentinel(current));
      if (res.detected_driver)
        setValue("driver", res.detected_driver, { shouldDirty: true });
      setTestState(res);
    } catch (e) {
      setTestState({
        ok: false,
        model: current.model ?? "",
        detected_driver: null,
        message: String(e),
      });
    }
  };

  const refreshModels = async () => {
    try {
      setLiveModels(await onListModels(withDetectSentinel(current)));
    } catch {
      // keep curated list on failure
    }
  };

  return (
    <form
      className="flex flex-1 flex-col"
      onSubmit={handleSubmit(async (c) => {
        const finalName = connName.trim();
        if (!finalName) return;
        const conn = { ...c, prompt_template: null } as Connection;
        // Renaming an existing connection moves the entry (and its active /
        // personalization references) before we persist the edited fields.
        if (!name.startsWith("new-") && finalName !== name) {
          await onRename(name, finalName);
        }
        await onSave(finalName, conn);
      })}
    >
      <div className="flex-1 space-y-1 overflow-auto p-4">
        <SetupField
          label="Name"
          help={<HelpText>A label for this connection (e.g. “work” or “z.ai”).</HelpText>}
        >
          <Input
            value={connName}
            onChange={(e) => setConnName(e.target.value)}
            placeholder="my-connection"
          />
        </SetupField>

        <SetupField
          label="Provider"
          help={
            <HelpText>
              Pick your provider — we fill in the technical bits for you.
            </HelpText>
          }
        >
          <select
            className="h-9 w-full rounded-md border border-input bg-[color:var(--card)] px-2 text-sm"
            value={presetKey}
            onChange={(e) => applyPreset(e.target.value)}
          >
            <option value="">— choose —</option>
            {presets.map((p) => (
              <option key={p.key} value={p.key}>
                {p.label}
              </option>
            ))}
          </select>
        </SetupField>

        <SetupField
          label="API key"
          help={
            <HelpText>
              Stored locally on your computer only — never uploaded.
            </HelpText>
          }
        >
          <div className="relative">
            <Input
              type={revealKey ? "text" : "password"}
              placeholder="••••••••"
              {...register("api_key")}
            />
            <button
              type="button"
              onClick={() => setRevealKey((v) => !v)}
              className="absolute right-2 top-1/2 -translate-y-1/2 text-muted-foreground"
            >
              {revealKey ? (
                <EyeSlash className="size-4" />
              ) : (
                <Eye className="size-4" />
              )}
            </button>
          </div>
        </SetupField>

        <SetupField label="Model">
          <ModelCombobox
            value={current.model ?? ""}
            onChange={(v) => setValue("model", v, { shouldDirty: true })}
            options={Array.from(new Set([...curated, ...liveModels]))}
          />
          <button
            type="button"
            onClick={refreshModels}
            className="mt-1 text-[11px] text-primary hover:underline"
          >
            Refresh model list
          </button>
        </SetupField>

        <SetupField
          label="Test"
          help={
            <HelpText>
              Verifies your API key and that the selected model responds.
            </HelpText>
          }
        >
          <Button type="button" variant="secondary" onClick={runTest}>
            Test connection
          </Button>
          {testState === "testing" ? (
            <p className="mt-1 text-[11px] text-muted-foreground">Testing…</p>
          ) : null}
          {typeof testState === "object" ? (
            <p
              className={
                "mt-1 text-[11px] " +
                (testState.ok
                  ? "text-[color:var(--color-success)]"
                  : "text-[color:var(--color-danger)]")
              }
            >
              {testState.message}
            </p>
          ) : null}
        </SetupField>

        <label className="mb-2 flex items-center gap-2 text-[11.5px]">
          <Checkbox
            checked={isPersonalization}
            onCheckedChange={() => onSetPersonalization(name)}
          />
          Use this connection for &quot;look up names online&quot;
        </label>
        <HelpText>
          The web-lookup step needs a model that can search the web (e.g. OpenAI/Gemini).
        </HelpText>

        <SectionHelp
          title="Advanced settings"
          hint="(address, tokens, parallelism, timeouts, thinking, web search)"
        >
          <div className="grid grid-cols-2 gap-2 text-[11px]">
            <Field label="Base URL" {...register("base_url")} />
            <Field
              label="Max tokens"
              type="number"
              {...register("max_tokens", { valueAsNumber: true })}
            />
            <Field
              label="Batch dialogue limit"
              type="number"
              {...register("batch_dialogue_limit", { valueAsNumber: true })}
            />
            <Field
              label="Concurrency"
              type="number"
              {...register("concurrency", { valueAsNumber: true })}
            />
            <Field
              label="Timeout (s)"
              type="number"
              {...register("timeout", { valueAsNumber: true })}
            />
            <Field
              label="Connect timeout (s)"
              type="number"
              {...register("connect_timeout", { valueAsNumber: true })}
            />
          </div>
          {isCustom ? (
            <HelpText>
              API format auto-detected on Test (currently: {current.driver}).
            </HelpText>
          ) : null}
        </SectionHelp>
      </div>

      <div className="flex items-center gap-2 border-t border-border bg-[color:var(--popover)] px-4 py-3">
        <Button
          type="button"
          variant="ghost"
          className="text-[color:var(--color-danger)]"
          onClick={() => onRemove(name)}
        >
          Remove
        </Button>
        <div className="flex-1" />
        {!isActive ? (
          <Button
            type="button"
            variant="secondary"
            onClick={() => onSetActive(name)}
          >
            Set as active
          </Button>
        ) : null}
        <Button type="submit" disabled={!canSave}>
          {formState.isSubmitting ? "Saving…" : "Save"}
        </Button>
      </div>
    </form>
  );
}

function Field({
  label,
  ...rest
}: { label: string } & React.InputHTMLAttributes<HTMLInputElement>) {
  return (
    <label className="flex flex-col gap-1">
      <span className="text-muted-foreground">{label}</span>
      <Input className="h-8" {...rest} />
    </label>
  );
}
