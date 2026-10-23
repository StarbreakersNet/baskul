<script setup lang="ts">
// À propos de Baskul.
import { computed, onMounted, ref } from "vue";
import {
  NAvatar,
  NButton,
  NCard,
  NCollapse,
  NCollapseItem,
  NDescriptions,
  NDescriptionsItem,
  NFlex,
  NH2,
  NProgress,
  NText,
  NTime,
  useMessage,
} from "naive-ui";
import { getVersion } from "@tauri-apps/api/app";
import { useI18n } from "vue-i18n";
import logo from "../assets/logo.svg";
import { useActions } from "../composables/useActions";
import { useSettings } from "../composables/useSettings";
import { useUpdate } from "../composables/useUpdate";
import { copyText, osLabel } from "../utils";

const REPO = "https://github.com/StarbreakersNet/baskul";

const { info } = useSettings();
const actions = useActions();
const message = useMessage();
const { t } = useI18n();
const appVersion = ref("");
const update = useUpdate();

/** Mises à jour : texte de l'état et progression du téléchargement. */
const updateView = computed(() => {
  const u = update.info.value;
  if (!u) return { text: t("update.idle") };
  switch (u.state) {
    case "idle":
      return { text: t("update.idle") };
    case "checking":
      return { text: t("update.checking") };
    case "upToDate":
      return { text: t("update.upToDate", { version: u.current }) };
    case "available":
      return { text: t(u.canInstall ? "update.available" : "update.availableManual", { version: u.version }), notes: u.notes, url: u.url };
    case "downloading":
      return { text: t("update.downloading", { version: u.version }), progress: u.total ? Math.round((u.done / u.total) * 100) : 0 };
    case "ready":
      return { text: t("update.ready", { version: u.version }), notes: u.notes };
    case "installing":
      return { text: t("update.installing", { version: u.version }) };
    case "error":
      return { text: t("update.error", { message: u.message }), url: u.releases };
  }
});
const busy = computed(() => {
  const s = update.info.value?.state;
  return update.checking.value || s === "checking" || s === "downloading";
});

async function copy(text: string) {
  try {
    await copyText(text);
    message.success(t("common.copied"));
  } catch {
    message.error(t("common.copyFailed"));
  }
}

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
        <n-h2 style="margin: 8px 0 0">Baskul</n-h2>
        <n-text depth="3">{{ $t("common.version", { version: info?.version ?? appVersion }) }}</n-text>
        <n-text style="text-align: center">{{ $t("about.tagline") }}</n-text>
      </n-flex>
    </n-card>

    <n-card :title="$t('about.info')" size="small">
      <n-descriptions label-placement="left" :column="1" size="small" bordered label-style="white-space: nowrap">
        <n-descriptions-item :label="$t('about.baskul')">{{ info?.version ?? $t("about.notRunning") }}</n-descriptions-item>
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

    <n-card :title="$t('update.title')" size="small">
      <n-flex align="center" justify="space-between" :size="12">
        <n-flex vertical :size="2" style="flex: 1; min-width: 220px">
          <n-text>{{ updateView.text }}</n-text>
          <n-text v-if="update.info.value?.checkedAt" depth="3" style="font-size: 13px">
            <i18n-t keypath="update.checkedAt" tag="span">
              <template #ago><n-time :time="update.info.value.checkedAt" type="relative" /></template>
            </i18n-t>
          </n-text>
        </n-flex>
        <n-button
          v-if="update.info.value?.state === 'ready' || update.info.value?.state === 'installing'"
          type="primary"
          :loading="update.info.value?.state === 'installing'"
          @click="update.install"
        >
          {{ $t("update.restart") }}
        </n-button>
        <n-button v-else :loading="busy" @click="update.check">{{ $t("update.check") }}</n-button>
      </n-flex>
      <n-progress
        v-if="updateView.progress !== undefined"
        type="line"
        :percentage="updateView.progress"
        :show-indicator="false"
        style="margin-top: 10px"
      />
      <n-flex v-if="updateView.url" align="center" :size="8" style="margin-top: 8px">
        <n-text class="mono" depth="3" style="word-break: break-all">{{ updateView.url }}</n-text>
        <n-button size="tiny" @click="copy(updateView.url!)">{{ $t("common.copy") }}</n-button>
      </n-flex>
      <n-collapse v-if="updateView.notes" style="margin-top: 8px">
        <n-collapse-item :title="$t('update.notes')" name="notes">
          <n-text style="white-space: pre-wrap">{{ updateView.notes }}</n-text>
        </n-collapse-item>
      </n-collapse>
    </n-card>

    <n-card :title="$t('about.thanks')" size="small">
      <n-text>{{ $t("about.thanksText") }}</n-text>
    </n-card>
  </n-flex>
</template>
