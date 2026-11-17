<script setup lang="ts">
import { computed } from "vue";
import { useRouter } from "vue-router";
import { IconBug, IconCircleCheck, IconKeyboard } from "@tabler/icons-vue";
import HButton from "@/components/c/hButton.vue";
import HFlex from "@/components/c/hFlex.vue";
import HIcon from "@/components/c/hIcon.vue";
import HPopover from "@/components/c/hPopover.vue";
import HTag from "@/components/c/hTag.vue";
import HText from "@/components/c/hText.vue";
import HTooltip from "@/components/c/hTooltip.vue";
import { useExternalLink } from "@/composables/useExternalLink";
import { useSettingsStore } from "@/stores/settings";
import { useStatusStore } from "@/stores/status";

const MODIFIER = navigator.platform.toLowerCase().startsWith("mac") ? "⌘" : "Ctrl";

const settings = useSettingsStore();
const statusStore = useStatusStore();
const router = useRouter();
const { reportIssue } = useExternalLink();

const isRestarting = computed(() => statusStore.phase === "restarting");
const isBusy = computed(() => settings.saving.loading || isRestarting.value);
const version = computed(() => settings.info?.version ?? statusStore.status?.version ?? "");
const showSaveBar = computed(
  () => settings.config !== null && (settings.isDirty || isRestarting.value)
);

function openAbout() {
  router.push({ name: "about" });
}
</script>

<template>
  <h-flex align="center" class="app-footer" :size="10" :wrap="false">
    <template v-if="showSaveBar">
      <h-text class="app-footer-state" depth="2">
        {{ isRestarting ? $t("layout.restartingWith") : $t("layout.unsaved") }}
      </h-text>
      <h-button :disabled="!settings.isDirty || isBusy" size="small" @click="settings.reset()">
        {{ $t("common.cancel") }}
      </h-button>
      <h-button
        :disabled="!settings.isDirty || isBusy"
        :loading="isBusy"
        size="small"
        type="primary"
        @click="settings.save()">
        {{ $t("layout.saveRestart") }}
      </h-button>
    </template>
    <h-flex v-else-if="settings.config" align="center" class="app-footer-state" :size="6">
      <h-icon depth="3" :size="16"><icon-circle-check /></h-icon>
      <h-text depth="3">{{ $t("layout.saved") }}</h-text>
    </h-flex>
    <div v-else class="app-footer-state" />

    <h-popover placement="top-end" trigger="click">
      <template #trigger>
        <h-button :aria-label="$t('layout.shortcuts')" quaternary size="small">
          <template #icon>
            <h-icon><icon-keyboard /></h-icon>
          </template>
        </h-button>
      </template>
      <h-flex class="app-footer-shortcuts" :size="6" vertical>
        <h-text strong>{{ $t("layout.shortcuts") }}</h-text>
        <h-flex align="center" justify="space-between" :size="16" :wrap="false">
          <h-text depth="2">{{ $t("layout.shortcutPages") }}</h-text>
          <h-tag class="mono" size="small">{{ MODIFIER }} + 1…4</h-tag>
        </h-flex>
        <h-flex align="center" justify="space-between" :size="16" :wrap="false">
          <h-text depth="2">{{ $t("layout.shortcutSave") }}</h-text>
          <h-tag class="mono" size="small">{{ MODIFIER }} + S</h-tag>
        </h-flex>
      </h-flex>
    </h-popover>
    <h-tooltip placement="top">
      <template #trigger>
        <h-button
          :aria-label="$t('layout.reportIssue')"
          quaternary
          size="small"
          @click="reportIssue()">
          <template #icon>
            <h-icon><icon-bug /></h-icon>
          </template>
        </h-button>
      </template>
      {{ $t("layout.reportIssue") }}
    </h-tooltip>
    <h-tag
      v-if="version"
      :bordered="false"
      class="app-footer-version"
      round
      size="small"
      :title="$t('layout.aboutVersion', { version })"
      type="primary"
      @click="openAbout()">
      v{{ version }}
    </h-tag>
  </h-flex>
</template>

<style lang="scss" scoped>
@use "@/assets/variables";

.app-footer {
  height: 100%;
}

.app-footer-state {
  flex: 1;
  min-width: 0;
}

.app-footer-shortcuts {
  min-width: 260px;
}

.app-footer-version {
  cursor: pointer;
  font-variant-numeric: tabular-nums;
}
</style>
