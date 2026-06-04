import { invoke } from "@tauri-apps/api/core";
import { listen, type EventCallback, type UnlistenFn } from "@tauri-apps/api/event";
import type { AppInfo } from "@/types/generated/AppInfo";

/**
 * Typed wrappers around the Rust core's Tauri commands. The webview never talks
 * to LLMs or the filesystem directly — every backend capability is exposed here.
 */
export const ipc = {
  /** App/core metadata. Doubles as a startup health check for the IPC bridge. */
  appInfo: () => invoke<AppInfo>("app_info"),
  /** O21 — check whether any usable connection exists (first-run gate). */
  firstRunStatus: () =>
    invoke<import("@/types/generated/FirstRunStatus").FirstRunStatus>("first_run_status"),
};

/** Subscribe to a backend-emitted event (progress, logs, …). Returns an unlisten fn. */
export function onBackendEvent<T>(
  name: string,
  handler: EventCallback<T>,
): Promise<UnlistenFn> {
  return listen<T>(name, handler);
}
