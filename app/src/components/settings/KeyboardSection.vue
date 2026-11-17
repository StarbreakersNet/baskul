<script setup lang="ts">
import { IconPlayerPlay, IconReplace } from "@tabler/icons-vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import HButton from "@/components/c/hButton.vue";
import HText from "@/components/c/hText.vue";
import ChoiceGroup from "@/components/ChoiceGroup.vue";
import LinuxNotice from "@/components/LinuxNotice.vue";
import SettingRow from "@/components/SettingRow.vue";
import { useSettingsStore } from "@/stores/settings";
import type { BaskulConfig } from "@/types/Config";
import type { ChoiceOption } from "@/types/Choice";

const config = defineModel<BaskulConfig>("config", { required: true });

const settings = useSettingsStore();
const { t } = useI18n();

const mediaKeyOptions = computed<ChoiceOption<BaskulConfig["keyboard"]["media_keys"]>[]>(() => [
  { value: "cursor", label: t("settings.keyboard.cursorMachine") },
  { value: "primary", label: t("settings.keyboard.primaryMachine") },
  { value: "secondary", label: t("settings.keyboard.secondaryMachine") },
]);
const remapCount = computed(() => Object.keys(config.value.keyboard.remap).length);
</script>

<template>
  <div>
    <linux-notice />
    <setting-row
      :icon="IconPlayerPlay"
      :help="$t('settings.keyboard.mediaKeysHelp')"
      :label="$t('settings.keyboard.mediaKeys')"
      machine="primary">
      <choice-group v-model="config.keyboard.media_keys" :options="mediaKeyOptions" />
    </setting-row>
    <setting-row
      :icon="IconReplace"
      :help="$t('settings.keyboard.remapHelp')"
      :label="$t('settings.keyboard.remap')"
      machine="secondary">
      <h-text depth="3">{{ $t("settings.keyboard.rules", remapCount) }}</h-text>
      <h-button size="small" @click="settings.openConfigFile()">
        {{ $t("common.openFile") }}
      </h-button>
    </setting-row>
  </div>
</template>
