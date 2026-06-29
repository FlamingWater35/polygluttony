import { useQuery } from "@tanstack/react-query";
import { openUrl } from "@tauri-apps/plugin-opener";
import { PageHeader } from "@/components/page-header";
import { useSettingsStore, UI_SCALE_LABELS, type UIScale } from "@/stores/settings-store";
import { HelpText } from "@/components/help-text";
import { TextAa, ArrowSquareOut, CheckCircle } from "@phosphor-icons/react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { ipc } from "@/lib/ipc";
import { Spinner } from "@/components/ui/spinner";

const GITHUB_REPO = "FlamingWater35/polygluttony-next";
const RELEASES_URL = `https://github.com/${GITHUB_REPO}/releases`;
const API_URL = `https://api.github.com/repos/${GITHUB_REPO}/releases/latest`;

/**
 * Fetches the latest release information from the GitHub API.
 * Returns the tag name and release URL, or null if the request fails.
 */
async function fetchLatestRelease() {
  try {
    const res = await fetch(API_URL);
    if (!res.ok) return null;
    const data = await res.json();
    return {
      tagName: data.tag_name as string,
      htmlUrl: data.html_url as string,
    };
  } catch {
    return null;
  }
}

/**
 * Strips leading 'v' and trims whitespace to normalize version strings for comparison.
 */
function normalizeVersion(v: string) {
  return v.replace(/^v/i, "").trim();
}

/**
 * Basic semver comparison to determine if the latest version is strictly newer than current.
 */
function isNewer(latest: string, current: string) {
  const l = latest.split(".").map(Number);
  const c = current.split(".").map(Number);
  for (let i = 0; i < Math.max(l.length, c.length); i++) {
    const lPart = l[i] || 0;
    const cPart = c[i] || 0;
    if (lPart > cPart) return true;
    if (lPart < cPart) return false;
  }
  return false;
}


export function SettingsPage() {
  const uiScale = useSettingsStore((s) => s.uiScale);
  const setUIScale = useSettingsStore((s) => s.setUIScale);

  // Fetch app metadata to display the current running version.
  const { data: appInfo } = useQuery({
    queryKey: ["app-info"],
    queryFn: ipc.appInfo,
    staleTime: Infinity,
  });

  // Check GitHub for the latest release tag.
  const { data: release, isPending: isChecking } = useQuery({
    queryKey: ["latest-release"],
    queryFn: fetchLatestRelease,
    staleTime: 1000 * 60 * 60, // Cache for 1 hour
    retry: 1,
  });

  const currentVersion = appInfo?.version ? normalizeVersion(appInfo.version) : null;
  const latestVersion = release?.tagName ? normalizeVersion(release.tagName) : null;

  const hasUpdate = !!(currentVersion && latestVersion && isNewer(latestVersion, currentVersion));

  return (
    <div className="flex h-full flex-col">
      <PageHeader
        title="Settings"
        description="Customize your workspace, check for updates, and view application info."
      />
      <div className="flex-1 overflow-auto p-5">
        <div className="mx-auto max-w-2xl space-y-8">

          {/* Appearance Section */}
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

          {/* Updates Section */}
          <section className="space-y-4">
            <div className="flex items-center gap-2">
              <CheckCircle weight="duotone" className="size-5 text-primary" />
              <h2 className="text-[15px] font-semibold text-foreground">Updates</h2>
            </div>
            <div className="rounded-lg border border-border bg-card p-5 space-y-4">
              <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
                <div>
                  <h3 className="text-[13px] font-medium text-foreground">
                    Polygluttony Next {currentVersion ? `v${currentVersion}` : "..."}
                  </h3>
                  <HelpText>
                    {isChecking
                      ? "Checking for updates..."
                      : hasUpdate
                      ? `A new version (v${latestVersion}) is available!`
                      : release
                      ? "You are running the latest version."
                      : "Could not check for updates. Check your internet connection."}
                  </HelpText>
                </div>
                <div className="flex items-center gap-2">
                  {isChecking && <Spinner className="size-4 text-muted-foreground" />}
                  {hasUpdate && release?.htmlUrl && (
                    <Button
                      size="sm"
                      onClick={() => openUrl(release.htmlUrl).catch(console.error)}
                      className="gap-1.5"
                    >
                      Download Update
                      <ArrowSquareOut className="size-4" />
                    </Button>
                  )}
                  {!hasUpdate && !isChecking && release && (
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => openUrl(RELEASES_URL).catch(console.error)}
                      className="gap-1.5"
                    >
                      View Releases
                      <ArrowSquareOut className="size-4" />
                    </Button>
                  )}
                  {!release && !isChecking && (
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => openUrl(RELEASES_URL).catch(console.error)}
                      className="gap-1.5"
                    >
                      View Releases
                      <ArrowSquareOut className="size-4" />
                    </Button>
                  )}
                </div>
              </div>
            </div>
          </section>

        </div>
      </div>
    </div>
  );
}
