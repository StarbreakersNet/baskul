import { defineStore } from "pinia";
import { ref } from "vue";
import { useAppMessage } from "@/composables/useAppMessage";
import { useActivityStore } from "@/stores/activity";
import { useSettingsStore } from "@/stores/settings";
import { daemonClient, errorMessage, isOffline } from "@/utils/daemonClient";
import { i18n } from "@/utils/locale";
import type { DaemonStatus } from "@/types/Daemon";

/** loading → ready ; offline tant que Baskul ne répond pas ; restarting après un enregistrement */
export type Phase = "loading" | "offline" | "starting" | "ready" | "restarting";

const POLL_MS = 1500;
const RESTART_POLL_MS = 400;
const RESTART_TIMEOUT_MS = 20_000;
// Baskul place une nouvelle pilotée sur la grille un peu après sa connexion
const LAYOUT_RELOADS = 3;

export const useStatusStore = defineStore("status", () => {
  const message = useAppMessage();

  const phase = ref<Phase>("loading");
  const status = ref<DaemonStatus | null>(null);

  let timer: ReturnType<typeof setTimeout> | undefined;
  let restartFrom: number | undefined;
  let restartDeadline = 0;
  let resolveRestart: ((restarted: boolean) => void) | undefined;
  let peersSeen = "";
  let layoutReloads = 0;

  async function init() {
    await useSettingsStore().readLaunchOptions();
    await poll();
  }

  async function poll() {
    try {
      const current = await daemonClient.status();
      status.value = current;
      useActivityStore().observe(current);
      await reloadSettingsIfNeeded(current);
    } catch (error) {
      handleOffline(error);
    }
    if (phase.value === "restarting" && Date.now() > restartDeadline) {
      phase.value = "offline";
      finishRestart(false);
    }
    schedule();
  }

  async function reloadSettingsIfNeeded(current: DaemonStatus) {
    if (phase.value === "restarting") {
      if (current.started !== restartFrom) {
        await loadSettings();
        finishRestart(true);
      }
    } else if (phase.value !== "ready" || hasUnplacedPeer(current)) {
      await loadSettings();
    }
  }

  async function loadSettings() {
    await useSettingsStore().load();
    phase.value = "ready";
  }

  function handleOffline(error: unknown) {
    if (!isOffline(error)) {
      console.error(error);
    }
    status.value = null;
    if (phase.value === "ready" || phase.value === "loading") {
      phase.value = "offline";
    }
  }

  /** Machine principale : une pilotée connectée manque à la grille chargée */
  function hasUnplacedPeer(current: DaemonStatus) {
    const names = current.peers.map(peer => peer.name).join("\n");
    if (names !== peersSeen) {
      peersSeen = names;
      layoutReloads = LAYOUT_RELOADS;
    }
    const settings = useSettingsStore();
    const config = settings.config;
    let missing = false;
    if (layoutReloads > 0 && config?.role === "primary" && !settings.isDirty) {
      missing = current.peers.some(
        peer => !config.layout.machines.some(machine => machine.name === peer.name)
      );
      layoutReloads = missing ? layoutReloads - 1 : 0;
    }
    return missing;
  }

  function schedule() {
    clearTimeout(timer);
    const isWaiting = phase.value === "restarting" || phase.value === "starting";
    timer = setTimeout(poll, isWaiting ? RESTART_POLL_MS : POLL_MS);
  }

  /** Résout `true` quand Baskul répond avec une nouvelle date de démarrage */
  function waitForRestart() {
    restartFrom = status.value?.started;
    restartDeadline = Date.now() + RESTART_TIMEOUT_MS;
    phase.value = "restarting";
    schedule();
    return new Promise<boolean>(resolve => {
      resolveRestart = resolve;
    });
  }

  function finishRestart(restarted: boolean) {
    resolveRestart?.(restarted);
    resolveRestart = undefined;
  }

  async function restart() {
    try {
      const result = await daemonClient.restart();
      if (result.restart && (await waitForRestart())) {
        message.success(i18n.global.t("actions.restarted"));
      }
    } catch (error) {
      message.error(errorMessage(error));
    }
  }

  async function start() {
    try {
      await daemonClient.start();
      phase.value = "starting";
      schedule();
    } catch (error) {
      message.error(errorMessage(error));
    }
  }

  /** `to` : "primary", "here" ou le nom d'une pilotée. Renvoie `true` si Baskul a basculé */
  async function switchTo(to: string) {
    let hasSwitched = false;
    try {
      await daemonClient.switchTo(to);
      hasSwitched = true;
    } catch (error) {
      message.error(errorMessage(error));
    }
    return hasSwitched;
  }

  return { phase, status, init, waitForRestart, restart, start, switchTo };
});
