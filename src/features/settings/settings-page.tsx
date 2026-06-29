import { PageHeader } from "@/components/page-header";
import { useSettingsStore, UI_SCALE_LABELS, type UIScale } from "@/stores/settings-store";
import { HelpText } from "@/components/help-text";
import { TextAa } from "@phosphor-icons/react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

/**
 * Settings page: central hub for user preferences.
 * Currently houses UI scale/font-size options which apply instantly via root CSS.
 */
export function SettingsPage() {
  const uiScale = useSettingsStore((s) => s.uiScale);
  const setUIScale = useSettingsStore((s) => s.setUIScale);

  return (
    <div className="flex h-full flex-col">
      <PageHeader
        title="Settings"
        description="Customize your workspace and application appearance."
      />
      <div className="flex-1 overflow-auto p-5">
        <div className="mx-auto max-w-2xl space-y-8">
          <section className="space-y-4">
            <div className="flex items-center gap-2">
              <TextAa weight="duotone" className="size-5 text-primary" />
              <h2 className="text-[15px] font-semibold text-foreground">Appearance</h2>
            </div>

            <div className="rounded-lg border border-border bg-card p-5 space-y-4">
              <div>
                <h3 className="text-[13px] font-medium text-foreground">UI Scale & Font Size</h3>
                <HelpText>
                  Adjust the overall size of the interface. Changes apply instantly and are saved for next time.
                </HelpText>
              </div>

              <div className="grid grid-cols-2 gap-3 sm:grid-cols-4">
                {(Object.keys(UI_SCALE_LABELS) as UIScale[]).map((key) => {
                  const isSelected = uiScale === key;
                  return (
                    <Button
                      key={key}
                      variant={isSelected ? "default" : "outline"}
                      onClick={() => setUIScale(key)}
                      className={cn(
                        "h-10 justify-center text-[13px] font-medium transition-all",
                        isSelected && "bg-primary text-primary-foreground hover:bg-primary/90"
                      )}
                      aria-pressed={isSelected}
                    >
                      {UI_SCALE_LABELS[key]}
                    </Button>
                  );
                })}
              </div>
            </div>
          </section>
        </div>
      </div>
    </div>
  );
}
