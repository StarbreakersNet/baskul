import { defineStore } from "pinia";
import { ref } from "vue";
import { useAppMessage } from "@/composables/useAppMessage";
import { useLoader } from "@/composables/useLoader";
import { useStatusStore } from "@/stores/status";
import { daemonClient, errorMessage } from "@/utils/daemonClient";
import { i18n } from "@/utils/locale";
import type { PairingState } from "@/types/Pairing";

const POLL_MS = 1000;

/** Ajout d'une machine par code : la principale l'affiche, la pilotée le saisit */
export const usePairingStore = defineStore("pairing", () => {
  const message = useAppMessage();

  const state = ref<PairingState>({ state: "idle" });
  const starting = useLoader();
  const joining = useLoader();

  let timer: ReturnType<typeof setTimeout> | undefined;

  async function poll() {
    try {
      state.value = await daemonClient.pairing("poll");
    } catch (error) {
      console.error(error);
    }
    schedule();
  }

  function schedule() {
    clearTimeout(timer);
    if (state.value.state === "waiting") {
      timer = setTimeout(poll, POLL_MS);
    }
  }

  async function start() {
    try {
      state.value = await starting.run(() => daemonClient.pairing("start"));
      schedule();
    } catch (error) {
      message.error(errorMessage(error));
    }
  }

  async function cancel() {
    clearTimeout(timer);
    try {
      state.value = await daemonClient.pairing("cancel");
    } catch (error) {
      message.error(errorMessage(error));
    }
  }

  /** Pilotée : reçoit le secret de la principale, puis Baskul redémarre */
  async function join(address: string, code: string) {
    try {
      const result = await joining.run(() => daemonClient.joinPrimary(address, code));
      if (!result.restart || (await useStatusStore().waitForRestart())) {
        message.success(i18n.global.t("pairing.joined", { name: result.primary }));
      }
    } catch (error) {
      message.error(errorMessage(error));
    }
  }

  return { state, starting, joining, start, cancel, join };
});
