import { defineStore } from "pinia";
import { open, save } from "@tauri-apps/plugin-dialog";
import { useAppMessage } from "@/composables/useAppMessage";
import { useLoader } from "@/composables/useLoader";
import { useStatusStore } from "@/stores/status";
import { daemonClient, errorMessage } from "@/utils/daemonClient";
import { i18n } from "@/utils/locale";

const FILTERS = [{ name: "Baskul", extensions: ["json"] }];

/** Export et import des réglages (configuration et profils) dans un fichier JSON */
export const useTransferStore = defineStore("transfer", () => {
  const message = useAppMessage();
  const exporting = useLoader();
  const importing = useLoader();

  async function exportSettings(hostname: string) {
    try {
      const path = await save({ defaultPath: `baskul-${hostname}.json`, filters: FILTERS });
      if (path) {
        await exporting.run(() => daemonClient.exportSettings(path));
        message.success(i18n.global.t("transfer.exported"));
      }
    } catch (error) {
      message.error(errorMessage(error));
    }
  }

  /** Demande le fichier ; renvoie son chemin, à confirmer avant `importSettings` */
  async function pickImport() {
    let path: string | null = null;
    try {
      const picked = await open({ multiple: false, directory: false, filters: FILTERS });
      path = typeof picked === "string" ? picked : null;
    } catch (error) {
      message.error(errorMessage(error));
    }
    return path;
  }

  async function importSettings(path: string) {
    try {
      const result = await importing.run(() => daemonClient.importSettings(path));
      if (!result.restart || (await useStatusStore().waitForRestart())) {
        message.success(i18n.global.t("transfer.imported"));
      }
    } catch (error) {
      message.error(errorMessage(error));
    }
  }

  return { exporting, importing, exportSettings, pickImport, importSettings };
});
