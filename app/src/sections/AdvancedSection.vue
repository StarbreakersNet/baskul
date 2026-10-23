<script setup lang="ts">
import { computed } from "vue";
import { NButton, NSelect } from "naive-ui";
import { useI18n } from "vue-i18n";
import ChoiceGroup from "../components/ChoiceGroup.vue";
import SettingRow from "../components/SettingRow.vue";
import type { BasculeConfig, DaemonInfo } from "../types";

defineProps<{ config: BasculeConfig; info: DaemonInfo; restarting?: boolean }>();
const emit = defineEmits<{ openFile: []; restart: [] }>();

const { t } = useI18n();
const levels = computed(() =>
  ["error", "warn", "info", "debug", "trace"].map((value) => ({ value, label: t(`settings.advanced.levels.${value}`) })),
);
</script>

<template>
  <div>
    <setting-row
      :label="$t('settings.advanced.isoSwap')"
      :help="$t('settings.advanced.isoSwapHelp')"
      machine="secondary"
      os="macos"
    >
      <choice-group
        v-model="config.keyboard.iso_swap"
        :options="[
          { value: 'auto', label: $t('settings.advanced.auto') },
          { value: 'on', label: $t('settings.advanced.swap') },
          { value: 'off', label: $t('settings.advanced.noSwap') },
        ]"
      />
    </setting-row>
    <setting-row :label="$t('settings.advanced.logLevel')">
      <n-select v-model:value="config.log.level" :options="levels" style="width: 220px" />
    </setting-row>
    <setting-row :label="$t('settings.advanced.configFile')">
      <template #help><span class="mono" style="word-break: break-all">{{ info.path }}</span></template>
      <n-button size="small" @click="emit('openFile')">{{ $t("common.openFile") }}</n-button>
    </setting-row>
    <setting-row label="Bascule" :help="$t('common.version', { version: info.version })">
      <n-button size="small" :loading="restarting" @click="emit('restart')">{{ $t("common.restart") }}</n-button>
    </setting-row>
  </div>
</template>
