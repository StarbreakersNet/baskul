// Mises à jour de Baskul : état (avec celui de la connexion), recherche et
// installation. Pendant l'installation, cette fenêtre se ferme ; Baskul la
// rouvre une fois redémarré dans sa nouvelle version.
import { computed, ref } from "vue";
import { useDialog, useMessage } from "naive-ui";
import { useI18n } from "vue-i18n";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { daemon, errorMessage } from "../daemon";
import type { UpdateInfo } from "../types";
import { useSettings } from "./useSettings";

const checking = ref(false);
const lastCheck = ref<UpdateInfo | null>(null);

export function useUpdate() {
  const { status, dirty } = useSettings();
  const message = useMessage();
  const dialog = useDialog();
  const { t } = useI18n();

  /** État le plus récent : celui de l'état général, ou de la dernière recherche. */
  const info = computed<UpdateInfo | null>(() => status.value?.update ?? lastCheck.value);

  async function check() {
    checking.value = true;
    try {
      lastCheck.value = await daemon.update.check();
    } catch (e) {
      message.error(errorMessage(e));
    } finally {
      // La recherche continue dans Baskul : l'état suit avec la connexion.
      setTimeout(() => (checking.value = false), 1500);
    }
  }

  async function run() {
    try {
      await daemon.update.install();
      await getCurrentWindow().destroy();
    } catch (e) {
      message.error(errorMessage(e), { duration: 8000, closable: true });
    }
  }

  function install() {
    if (!dirty.value) return run();
    dialog.warning({
      title: t("update.unsavedTitle"),
      content: t("update.unsavedText"),
      positiveText: t("update.restart"),
      negativeText: t("common.cancel"),
      onPositiveClick: () => void run(),
    });
  }

  return { info, checking, check, install };
}
