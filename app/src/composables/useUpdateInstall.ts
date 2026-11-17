import { useDialog } from "naive-ui";
import { useI18n } from "vue-i18n";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useSettingsStore } from "@/stores/settings";
import { useUpdateStore } from "@/stores/update";

/** Installe la mise à jour prête ; Baskul rouvre la fenêtre une fois redémarré */
export function useUpdateInstall() {
  const settings = useSettingsStore();
  const update = useUpdateStore();
  const dialog = useDialog();
  const { t } = useI18n();

  async function installAndClose() {
    if (await update.install()) {
      await getCurrentWindow().destroy();
    }
  }

  function install() {
    if (settings.isDirty) {
      dialog.warning({
        title: t("update.unsavedTitle"),
        content: t("update.unsavedText"),
        positiveText: t("update.restart"),
        negativeText: t("common.cancel"),
        onPositiveClick: () => void installAndClose(),
      });
    } else {
      void installAndClose();
    }
  }

  return { install };
}
