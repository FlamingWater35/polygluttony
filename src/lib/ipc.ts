import { invoke } from "@tauri-apps/api/core";
import { listen, type EventCallback, type UnlistenFn } from "@tauri-apps/api/event";
import type { AppInfo } from "@/types/generated/AppInfo";
import type { Connection } from "@/types/generated/Connection";
import type { ConnectionsView } from "@/types/generated/ConnectionsView";
import type { Preset } from "@/types/generated/Preset";
import type { TestResult } from "@/types/generated/TestResult";
import type { FirstRunStatus } from "@/types/generated/FirstRunStatus";
import type { Language } from "@/types/generated/Language";
import type { ProjectView } from "@/types/generated/ProjectView";
import type { FolderPrefs } from "@/types/generated/FolderPrefs";
import type { RecentFolder } from "@/types/generated/RecentFolder";

/**
 * Typed wrappers around the Rust core's Tauri commands. The webview never talks
 * to LLMs or the filesystem directly — every backend capability is exposed here.
 */
export const ipc = {
  /** App/core metadata. Doubles as a startup health check for the IPC bridge. */
  appInfo: () => invoke<AppInfo>("app_info"),
  /** O21 — check whether any usable connection exists (first-run gate). */
  firstRunStatus: () => invoke<FirstRunStatus>("first_run_status"),
  /** Return the provider preset table for the Connections UI. */
  listPresets: () => invoke<Preset[]>("list_presets"),
  /** O1 — list all connections with active/personalization state (no keys). */
  listConnections: () => invoke<ConnectionsView>("list_connections"),
  /** Read a single connection by name (includes api_key). */
  readConnection: (name: string) => invoke<Connection>("read_connection", { name }),
  /** O3 — upsert a connection. */
  saveConnection: (name: string, connection: Connection) =>
    invoke<void>("save_connection", { name, connection }),
  /** O4 — remove a connection (fails if it is the active one). */
  deleteConnection: (name: string) => invoke<void>("delete_connection", { name }),
  /** Rename a connection, preserving active/personalization references. */
  renameConnection: (oldName: string, newName: string) =>
    invoke<void>("rename_connection", { old: oldName, new: newName }),
  /** O2 — set the active connection. */
  setActiveConnection: (name: string) => invoke<void>("set_active_connection", { name }),
  /** Set the personalization (web-lookup) connection. */
  setPersonalizationConnection: (name: string) =>
    invoke<void>("set_personalization_connection", { name }),
  /**
   * O5 — test a connection. For Custom connections the caller should set
   * `connection.prompt_template = "__detect__"` to trigger auto-detection.
   */
  testConnection: (connection: Connection) =>
    invoke<TestResult>("test_connection", { connection }),
  /**
   * Fetch live model list for a connection. Same detection sentinel applies
   * for Custom: set `connection.prompt_template = "__detect__"`.
   */
  listModels: (connection: Connection) => invoke<string[]>("list_models", { connection }),
  /** O6/O7/O8 — open a folder: discover files, counts, detect world/language, load prefs. */
  openFolder: (path: string) =>
    invoke<ProjectView>("open_folder", { path, now: Math.floor(Date.now() / 1000) }),
  /** O8 — the supported-language table for the selects. */
  listLanguages: () => invoke<Language[]>("list_languages"),
  /** Persist per-folder preferences (languages, world override, tone, selection). */
  saveFolderPrefs: (path: string, prefs: FolderPrefs) =>
    invoke<void>("save_folder_prefs", { path, prefs }),
  /** Persist the source/target pair as the global default (new folders + future sessions). */
  setDefaultLanguages: (source: string, target: string) =>
    invoke<void>("set_default_languages", { source, target }),
  /** Recent folders (MRU; missing folders pruned server-side). */
  listRecents: () => invoke<RecentFolder[]>("list_recents"),
  removeRecent: (path: string) => invoke<void>("remove_recent", { path }),
  clearRecents: () => invoke<void>("clear_recents"),
};

/** Subscribe to a backend-emitted event (progress, logs, …). Returns an unlisten fn. */
export function onBackendEvent<T>(
  name: string,
  handler: EventCallback<T>,
): Promise<UnlistenFn> {
  return listen<T>(name, handler);
}
