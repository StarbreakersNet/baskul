import { invoke } from "@tauri-apps/api/core";
import type { ActivityEvent } from "@/types/Activity";
import type { BaskulConfig } from "@/types/Config";
import type { CommandError, DaemonInfo, DaemonStatus, LogTail } from "@/types/Daemon";
import type { CaptureState } from "@/types/Hotkey";
import type { ProfilesInfo } from "@/types/Profiles";
import type { NetworkInfo } from "@/types/Network";
import type { UpdateInfo } from "@/types/Update";

interface RestartResult {
  restart: boolean;
}

export const daemonClient = {
  status: () => invoke<DaemonStatus>("daemon_status"),
  load: () => invoke<DaemonInfo>("load_settings"),
  save: (config: BaskulConfig) => invoke<RestartResult>("save_settings", { config }),
  restart: () => invoke<RestartResult>("restart_daemon"),
  openConfigFile: () => invoke("open_config_file"),
  readLog: () => invoke<LogTail>("read_log"),
  openLogFile: () => invoke("open_log_file"),
  readActivity: () => invoke<{ events: ActivityEvent[] }>("read_activity"),
  clearActivity: () => invoke("clear_activity"),
  profiles: () => invoke<ProfilesInfo>("list_profiles"),
  saveProfile: (name: string) => invoke<ProfilesInfo>("save_profile", { name }),
  deleteProfile: (name: string) => invoke<ProfilesInfo>("delete_profile", { name }),
  applyProfile: (name: string) => invoke<RestartResult>("apply_profile", { name }),
  exportSettings: (path: string) => invoke("export_settings", { path }),
  importSettings: (path: string) => invoke<RestartResult>("import_settings", { path }),
  startCapture: () => invoke("capture_hotkey", { action: "start" }),
  pollCapture: () => invoke<CaptureState>("capture_hotkey", { action: "poll" }),
  cancelCapture: () => invoke("capture_hotkey", { action: "cancel" }),
  network: () => invoke<NetworkInfo>("network_info"),
  /** `to` : "primary", "here", "next" ou le nom d'une pilotée */
  switchTo: (to: string) => invoke("switch_to", { to }),
  checkUpdate: () => invoke<UpdateInfo>("update_action", { action: "check" }),
  installUpdate: () => invoke<UpdateInfo>("update_action", { action: "install" }),
  start: () => invoke("start_daemon"),
  launchOptions: () => invoke<{ welcome: boolean }>("launch_options"),
};

export function isOffline(error: unknown) {
  return (error as CommandError)?.kind === "offline";
}

export function errorMessage(error: unknown) {
  return typeof error === "string" ? error : ((error as CommandError)?.message ?? String(error));
}
