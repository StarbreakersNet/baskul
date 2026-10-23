<script setup lang="ts">
import { computed } from "vue";
import { NButton, NText } from "naive-ui";
import ChoiceGroup from "../components/ChoiceGroup.vue";
import LinuxNotice from "../components/LinuxNotice.vue";
import SettingRow from "../components/SettingRow.vue";
import type { BaskulConfig } from "../types";

const props = defineProps<{ config: BaskulConfig }>();
const emit = defineEmits<{ openFile: [] }>();
const rules = computed(() => Object.keys(props.config.keyboard.remap).length);
</script>

<template>
  <div>
    <linux-notice />
    <setting-row
      :label="$t('settings.keyboard.mediaKeys')"
      :help="$t('settings.keyboard.mediaKeysHelp')"
      machine="primary"
    >
      <choice-group
        v-model="config.keyboard.media_keys"
        :options="[
          { value: 'cursor', label: $t('settings.keyboard.cursorMachine') },
          { value: 'primary', label: $t('settings.keyboard.primaryMachine') },
          { value: 'secondary', label: $t('settings.keyboard.secondaryMachine') },
        ]"
      />
    </setting-row>
    <setting-row :label="$t('settings.keyboard.remap')" :help="$t('settings.keyboard.remapHelp')" machine="secondary">
      <n-text depth="3">{{ $t("settings.keyboard.rules", rules) }}</n-text>
      <n-button size="small" @click="emit('openFile')">{{ $t("common.openFile") }}</n-button>
    </setting-row>
  </div>
</template>
