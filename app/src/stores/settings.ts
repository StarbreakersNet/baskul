import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { useAppMessage } from "@/composables/useAppMessage";
import { useLoader } from "@/composables/useLoader";
import { useStatusStore } from "@/stores/status";
import { daemonClient, errorMessage } from "@/utils/daemonClient";
import { i18n } from "@/utils/locale";
import type { BaskulConfig } from "@/types/Config";
import type { DaemonInfo } from "@/types/Daemon";

export const useSettingsStore = defineStore("settings", () => {
  const message = useAppMessage();

  const info = ref<DaemonInfo | null>(null);
  const config = ref<BaskulConfig | null>(null);
  const savedJson = ref("");
  const welcome = ref(false);
  const saving = useLoader();

  const isDirty = computed(
    () => config.value !== null && JSON.stringify(config.value) !== savedJson.value
  );

  async function load() {
    const loaded = await daemonClient.load();
    info.value = loaded;
    config.value = loaded.config ? structuredClone(loaded.config) : null;
    savedJson.value = JSON.stringify(loaded.config);
  }

  async function readLaunchOptions() {
    try {
      welcome.value = (await daemonClient.launchOptions()).welcome;
    } catch {
      welcome.value = false;
    }
  }

  async function sendConfig(edited: BaskulConfig) {
    const result = await daemonClient.save(edited);
    savedJson.value = JSON.stringify(edited);
    welcome.value = false;
    return !result.restart || (await useStatusStore().waitForRestart());
  }

  /** Renvoie `true` si Baskul a redémarré avec les nouveaux réglages */
  async function save() {
    let applied = false;
    const edited = config.value;
    if (edited) {
      try {
        applied = await saving.run(() => sendConfig(edited));
        showSaveResult(applied);
      } catch (error) {
        message.error(errorMessage(error));
      }
    }
    return applied;
  }

  function showSaveResult(applied: boolean) {
    if (applied) {
      message.success(i18n.global.t("actions.saved"));
    } else {
      message.warning(i18n.global.t("actions.savedNoRestart"));
    }
  }

  function reset() {
    config.value = JSON.parse(savedJson.value);
  }

  async function openConfigFile() {
    try {
      await daemonClient.openConfigFile();
    } catch (error) {
      message.error(errorMessage(error));
    }
  }

  return {
    info,
    config,
    welcome,
    saving,
    isDirty,
    load,
    readLaunchOptions,
    save,
    reset,
    openConfigFile,
  };
});
