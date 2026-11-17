import { openUrl } from "@tauri-apps/plugin-opener";
import { useI18n } from "vue-i18n";
import { useAppMessage } from "@/composables/useAppMessage";
import { useSettingsStore } from "@/stores/settings";
import { useStatusStore } from "@/stores/status";
import { bugReportUrl } from "@/utils/issues";
import { osLabel } from "@/utils/os";

/** Ouvre un lien dans le navigateur du système */
export function useExternalLink() {
  const message = useAppMessage();
  const settings = useSettingsStore();
  const statusStore = useStatusStore();
  const { t } = useI18n();

  async function openLink(url: string) {
    try {
      await openUrl(url);
    } catch {
      message.error(t("common.linkFailed", { url }));
    }
  }

  /** Nouvelle issue GitHub, avec la version et le système déjà remplis */
  function reportIssue() {
    const body = t("layout.issueBody", {
      version: settings.info?.version ?? statusStore.status?.version ?? "?",
      os: osLabel(settings.info?.os ?? navigator.platform),
    });
    return openLink(bugReportUrl(body));
  }

  return { openLink, reportIssue };
}
