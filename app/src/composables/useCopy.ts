import { useI18n } from "vue-i18n";
import { useAppMessage } from "@/composables/useAppMessage";
import { copyText } from "@/utils/clipboard";

export function useCopy() {
  const message = useAppMessage();
  const { t } = useI18n();

  async function copy(text: string, successKey = "common.copied") {
    try {
      await copyText(text);
      message.success(t(successKey));
    } catch {
      message.error(t("common.copyFailed"));
    }
  }

  return { copy };
}
