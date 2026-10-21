// Commandes Tauri (voir `src-tauri/src/main.rs`), relayées à l'API du daemon.
import { invoke } from "@tauri-apps/api/core";
import type { BasculeConfig, DaemonInfo, DaemonStatus } from "./types";

/** Erreur renvoyée par les commandes. */
export interface CommandError {
  kind: "offline" | "refused";
  message: string;
}

export const daemon = {
  status: () => invoke<DaemonStatus>("daemon_status"),
  load: () => invoke<DaemonInfo>("load_settings"),
  save: (config: BasculeConfig) => invoke<{ restart: boolean }>("save_settings", { config }),
  restart: () => invoke<{ restart: boolean }>("restart_daemon"),
  openConfigFile: () => invoke("open_config_file"),
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
