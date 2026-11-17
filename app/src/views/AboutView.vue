<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { getVersion } from "@tauri-apps/api/app";
import { IconBrandGithub, IconBug, IconExternalLink, IconHistory } from "@tabler/icons-vue";
import HAvatar from "@/components/c/hAvatar.vue";
import HButton from "@/components/c/hButton.vue";
import HCard from "@/components/c/hCard.vue";
import HCollapse from "@/components/c/hCollapse.vue";
import HCollapseItem from "@/components/c/hCollapseItem.vue";
import HDescriptions from "@/components/c/hDescriptions.vue";
import HDescriptionsItem from "@/components/c/hDescriptionsItem.vue";
import HFlex from "@/components/c/hFlex.vue";
import HH2 from "@/components/c/hH2.vue";
import HIcon from "@/components/c/hIcon.vue";
import HProgress from "@/components/c/hProgress.vue";
import HSkeleton from "@/components/c/hSkeleton.vue";
import HText from "@/components/c/hText.vue";
import HTime from "@/components/c/hTime.vue";
import ExpandTransition from "@/components/ExpandTransition.vue";
import logo from "@/assets/logo.svg";
import { useCopy } from "@/composables/useCopy";
import { useExternalLink } from "@/composables/useExternalLink";
import { useUpdateInstall } from "@/composables/useUpdateInstall";
import { useSettingsStore } from "@/stores/settings";
import { useUpdateStore } from "@/stores/update";
import { RELEASES_URL, REPOSITORY_URL } from "@/models/Repository";
import { osLabel } from "@/utils/os";
import type { UpdateInfo } from "@/types/Update";

interface UpdateView {
  text: string;
  notes?: string;
  url?: string;
  /** Téléchargement en cours, en pourcentage */
  progress?: number;
}

const settings = useSettingsStore();
const update = useUpdateStore();
const { install } = useUpdateInstall();
const { copy } = useCopy();
const { openLink, reportIssue } = useExternalLink();
const { t } = useI18n();

const appVersion = ref("");

const updateView = computed(() => describeUpdate(update.info));
const isUpdateReady = computed(() => ["ready", "installing"].includes(update.info?.state ?? ""));
const isInstalling = computed(() => update.info?.state === "installing");
const isChecking = computed(
  () => update.checking.loading || ["checking", "downloading"].includes(update.info?.state ?? "")
);

function describeUpdate(info: UpdateInfo | null): UpdateView {
  switch (info?.state) {
    case "checking":
      return { text: t("update.checking") };
    case "upToDate":
      return { text: t("update.upToDate", { version: info.current }) };
    case "available":
      return {
        text: t(info.canInstall ? "update.available" : "update.availableManual", {
          version: info.version,
        }),
        notes: info.notes,
        url: info.url,
      };
    case "downloading":
      return {
        text: t("update.downloading", { version: info.version }),
        progress: info.total ? Math.round((info.done / info.total) * 100) : 0,
      };
    case "ready":
      return { text: t("update.ready", { version: info.version }), notes: info.notes };
    case "installing":
      return { text: t("update.installing", { version: info.version }) };
    case "error":
      return { text: t("update.error", { message: info.message }), url: info.releases };
    default:
      return { text: t("update.idle") };
  }
}

onMounted(async () => {
  try {
    appVersion.value = await getVersion();
  } catch {
    appVersion.value = "";
  }
});
</script>

<template>
  <h-flex :size="16" vertical>
    <h-card>
      <h-flex align="center" class="about-header" :size="6" vertical>
        <h-avatar color="transparent" :size="80" :src="logo" />
        <h-h2 class="about-name brand-wordmark">Baskul</h-h2>
        <h-text depth="3">
          {{ $t("common.version", { version: settings.info?.version ?? appVersion }) }}
        </h-text>
        <h-text class="about-tagline">{{ $t("about.tagline") }}</h-text>
        <h-flex class="about-links" justify="center" :size="8">
          <h-button secondary size="small" @click="openLink(REPOSITORY_URL)">
            <template #icon>
              <h-icon><icon-brand-github /></h-icon>
            </template>
            GitHub
          </h-button>
          <h-button secondary size="small" @click="openLink(RELEASES_URL)">
            <template #icon>
              <h-icon><icon-history /></h-icon>
            </template>
            {{ $t("about.releases") }}
          </h-button>
          <h-button secondary size="small" @click="reportIssue()">
            <template #icon>
              <h-icon><icon-bug /></h-icon>
            </template>
            {{ $t("layout.reportIssue") }}
          </h-button>
        </h-flex>
      </h-flex>
    </h-card>

    <h-card size="small" :title="$t('about.info')">
      <h-descriptions
        bordered
        :column="1"
        label-placement="left"
        label-style="white-space: nowrap"
        size="small">
        <h-descriptions-item :label="$t('about.baskul')">
          {{ settings.info?.version ?? $t("about.notRunning") }}
        </h-descriptions-item>
        <h-descriptions-item :label="$t('about.app')">{{ appVersion }}</h-descriptions-item>
        <h-descriptions-item :label="$t('about.machine')">
          <transition mode="out-in" name="fade">
            <span v-if="settings.info">
              {{ settings.info.hostname }} · {{ osLabel(settings.info.os) }}
            </span>
            <h-skeleton v-else text :width="180" />
          </transition>
        </h-descriptions-item>
        <h-descriptions-item :label="$t('about.config')">
          <transition mode="out-in" name="fade">
            <h-flex v-if="settings.info" align="center" :size="8">
              <h-text class="mono break-all">{{ settings.info.path }}</h-text>
              <h-button size="tiny" @click="settings.openConfigFile()">
                {{ $t("common.open") }}
              </h-button>
            </h-flex>
            <h-skeleton v-else text :width="320" />
          </transition>
        </h-descriptions-item>
        <h-descriptions-item :label="$t('about.source')">
          <h-flex align="center" :size="8">
            <h-text class="mono">{{ REPOSITORY_URL }}</h-text>
            <h-button size="tiny" @click="copy(REPOSITORY_URL, 'about.addressCopied')">
              {{ $t("common.copy") }}
            </h-button>
          </h-flex>
        </h-descriptions-item>
        <h-descriptions-item :label="$t('about.license')">MIT</h-descriptions-item>
      </h-descriptions>
    </h-card>

    <h-card size="small" :title="$t('update.title')">
      <h-flex align="center" justify="space-between" :size="12">
        <h-flex class="about-update-text" :size="2" vertical>
          <h-text>{{ updateView.text }}</h-text>
          <expand-transition>
            <h-text v-if="update.info?.checkedAt" class="small-text" depth="3" tag="div">
              <i18n-t keypath="update.checkedAt" tag="span">
                <template #ago><h-time :time="update.info.checkedAt" type="relative" /></template>
              </i18n-t>
            </h-text>
          </expand-transition>
        </h-flex>
        <transition mode="out-in" name="fade">
          <h-button v-if="isUpdateReady" :loading="isInstalling" type="primary" @click="install()">
            {{ $t("update.restart") }}
          </h-button>
          <h-button v-else :loading="isChecking" @click="update.check()">
            {{ $t("update.check") }}
          </h-button>
        </transition>
      </h-flex>
      <expand-transition>
        <div v-if="updateView.progress !== undefined" class="about-block">
          <h-progress :percentage="updateView.progress" :show-indicator="false" type="line" />
        </div>
      </expand-transition>
      <expand-transition>
        <div v-if="updateView.url" class="about-block">
          <h-flex align="center" :size="8">
            <h-text class="mono break-all" depth="3">{{ updateView.url }}</h-text>
            <h-button size="tiny" @click="openLink(updateView.url)">
              <template #icon>
                <h-icon><icon-external-link /></h-icon>
              </template>
              {{ $t("common.open") }}
            </h-button>
            <h-button size="tiny" @click="copy(updateView.url)">{{ $t("common.copy") }}</h-button>
          </h-flex>
        </div>
      </expand-transition>
      <expand-transition>
        <div v-if="updateView.notes" class="about-block">
          <h-collapse>
            <h-collapse-item name="notes" :title="$t('update.notes')">
              <h-text class="about-notes">{{ updateView.notes }}</h-text>
            </h-collapse-item>
          </h-collapse>
        </div>
      </expand-transition>
    </h-card>

    <h-card size="small" :title="$t('about.thanks')">
      <h-text>{{ $t("about.thanksText") }}</h-text>
    </h-card>
  </h-flex>
</template>

<style lang="scss" scoped>
.about-header {
  padding: 8px 0;
}

.about-name {
  margin: 8px 0 0;
}

.about-tagline {
  text-align: center;
}

.about-links {
  margin-top: 10px;
}

.about-update-text {
  flex: 1;
  min-width: 220px;
}

.about-block {
  padding-top: 10px;
}

.about-notes {
  white-space: pre-wrap;
}
</style>
