import { defineStore } from "pinia";
import { daemonClient } from "@/utils/daemonClient";
import type { CaptureState } from "@/types/Hotkey";

// Machine principale : le crochet clavier de Baskul relève la combinaison, qui ne déclenche alors
// aucun raccourci, et les touches que la fenêtre ne reçoit pas (F11, F12…) fonctionnent aussi
export const useHotkeyCaptureStore = defineStore("hotkeyCapture", () => {
  /** Renvoie `false` sans crochet clavier (machine pilotée, Linux) : la fenêtre relève seule */
  async function start() {
    let started = false;
    try {
      await daemonClient.startCapture();
      started = true;
    } catch {
      started = false;
    }
    return started;
  }

  /** Renvoie `null` si Baskul ne répond pas */
  async function poll(): Promise<CaptureState | null> {
    try {
      return await daemonClient.pollCapture();
    } catch {
      return null;
    }
  }

  async function cancel() {
    try {
      await daemonClient.cancelCapture();
    } catch {
      // Saisie déjà terminée côté Baskul
    }
  }

  return { start, poll, cancel };
});
