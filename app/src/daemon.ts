// Commandes Tauri (voir `src-tauri/src/main.rs`), relayées à l'API du daemon.
import { invoke } from "@tauri-apps/api/core";
import type { BaskulConfig, CaptureState, DaemonInfo, DaemonStatus } from "./types";

/** Erreur renvoyée par les commandes. */
export interface CommandError {
  kind: "offline" | "refused";
  message: string;
}

export const daemon = {
  status: () => invoke<DaemonStatus>("daemon_status"),
  load: () => invoke<DaemonInfo>("load_settings"),
  save: (config: BaskulConfig) => invoke<{ restart: boolean }>("save_settings", { config }),
  restart: () => invoke<{ restart: boolean }>("restart_daemon"),
  openConfigFile: () => invoke("open_config_file"),
  readLog: () => invoke<{ path: string | null; text: string }>("read_log"),
  openLogFile: () => invoke("open_log_file"),
  /** Saisie d'un raccourci par le crochet clavier de Baskul (machine principale). */
  captureHotkey: {
    start: () => invoke<object>("capture_hotkey", { action: "start" }),
    poll: () => invoke<CaptureState>("capture_hotkey", { action: "poll" }),
    cancel: () => invoke<object>("capture_hotkey", { action: "cancel" }),
  },
  start: () => invoke<void>("start_daemon"),
  launchOptions: () => invoke<{ welcome: boolean }>("launch_options"),
};

export function isOffline(e: unknown): boolean {
  return (e as CommandError)?.kind === "offline";
}

export function errorMessage(e: unknown): string {
  if (typeof e === "string") return e;
  return (e as CommandError)?.message ?? String(e);
}
