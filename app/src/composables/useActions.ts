// Actions de l'app, avec leurs messages de réussite ou d'erreur. À appeler
// dans `setup` (les messages viennent de Naive UI).
import { useMessage } from "naive-ui";
import { useI18n } from "vue-i18n";
import { daemon } from "../daemon";
import { useSettings } from "./useSettings";

export function useActions() {
  const settings = useSettings();
  const message = useMessage();
  const { t } = useI18n();

  /** Affiche l'erreur d'une action. */
  async function attempt(action: () => Promise<unknown>) {
    try {
      await action();
    } catch (e) {
      message.error(settings.errorMessage(e), { duration: 8000, closable: true });
    }
  }

  return {
    save: () =>
      attempt(async () => {
        if (await settings.save()) message.success(t("actions.saved"));
        else message.warning(t("actions.savedNoRestart"));
      }),
    restart: () =>
      attempt(async () => {
        if (await settings.restart()) message.success(t("actions.restarted"));
      }),
    start: () => attempt(settings.start),
    openConfigFile: () => attempt(settings.openConfigFile),
    openLogFile: () => attempt(daemon.openLogFile),
  };
}
