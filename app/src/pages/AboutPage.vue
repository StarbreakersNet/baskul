<script setup lang="ts">
// À propos de Bascule.
import { onMounted, ref } from "vue";
import { NAvatar, NButton, NCard, NDescriptions, NDescriptionsItem, NFlex, NH2, NText, useMessage } from "naive-ui";
import { getVersion } from "@tauri-apps/api/app";
import { useI18n } from "vue-i18n";
import logo from "../assets/logo.svg";
import { useActions } from "../composables/useActions";
import { useSettings } from "../composables/useSettings";
import { copyText, osLabel } from "../utils";

const REPO = "https://github.com/StarbreakersNet/bascule";

const { info } = useSettings();
const actions = useActions();
const message = useMessage();
const { t } = useI18n();
const appVersion = ref("");

onMounted(async () => {
  appVersion.value = await getVersion().catch(() => "");
});

async function copyRepo() {
  try {
    await copyText(REPO);
    message.success(t("about.addressCopied"));
  } catch {
    message.error(t("common.copyFailed"));
  }
}
</script>

<template>
  <n-flex vertical :size="16">
    <n-card>
      <n-flex vertical align="center" :size="6" style="padding: 8px 0">
        <n-avatar :src="logo" :size="80" color="transparent" />
        <n-h2 style="margin: 8px 0 0">Bascule</n-h2>
        <n-text depth="3">{{ $t("common.version", { version: info?.version ?? appVersion }) }}</n-text>
        <n-text style="text-align: center">{{ $t("about.tagline") }}</n-text>
      </n-flex>
    </n-card>

    <n-card :title="$t('about.info')" size="small">
      <n-descriptions label-placement="left" :column="1" size="small" bordered label-style="white-space: nowrap">
        <n-descriptions-item :label="$t('about.bascule')">{{ info?.version ?? $t("about.notRunning") }}</n-descriptions-item>
        <n-descriptions-item :label="$t('about.app')">{{ appVersion }}</n-descriptions-item>
        <n-descriptions-item v-if="info" :label="$t('about.machine')">
          {{ info.hostname }} · {{ osLabel(info.os) }}
        </n-descriptions-item>
        <n-descriptions-item v-if="info" :label="$t('about.config')">
          <n-flex align="center" :size="8">
            <n-text class="mono" style="word-break: break-all">{{ info.path }}</n-text>
            <n-button size="tiny" @click="actions.openConfigFile">{{ $t("common.open") }}</n-button>
          </n-flex>
        </n-descriptions-item>
        <n-descriptions-item :label="$t('about.source')">
          <n-flex align="center" :size="8">
            <n-text class="mono">{{ REPO }}</n-text>
            <n-button size="tiny" @click="copyRepo">{{ $t("common.copy") }}</n-button>
          </n-flex>
        </n-descriptions-item>
        <n-descriptions-item :label="$t('about.license')">MIT</n-descriptions-item>
      </n-descriptions>
    </n-card>

    <n-card :title="$t('about.thanks')" size="small">
      <n-text>{{ $t("about.thanksText") }}</n-text>
    </n-card>
  </n-flex>
</template>
