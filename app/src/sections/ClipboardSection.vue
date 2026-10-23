<script setup lang="ts">
import { NSwitch } from "naive-ui";
import NumberInput from "../components/NumberInput.vue";
import SettingRow from "../components/SettingRow.vue";
import type { BasculeConfig } from "../types";

defineProps<{ config: BasculeConfig }>();
</script>

<template>
  <div>
    <setting-row :label="$t('settings.clipboard.share')">
      <n-switch v-model:value="config.clipboard.enabled" />
    </setting-row>
    <setting-row :label="$t('settings.clipboard.images')" :disabled="!config.clipboard.enabled">
      <n-switch v-model:value="config.clipboard.images" :disabled="!config.clipboard.enabled" />
    </setting-row>
    <setting-row
      :label="$t('settings.clipboard.html')"
      :help="$t('settings.clipboard.htmlHelp')"
      :disabled="!config.clipboard.enabled"
    >
      <n-switch v-model:value="config.clipboard.html" :disabled="!config.clipboard.enabled" />
    </setting-row>
    <setting-row
      :label="$t('settings.clipboard.sensitive')"
      :help="$t('settings.clipboard.sensitiveHelp')"
      :disabled="!config.clipboard.enabled"
    >
      <n-switch v-model:value="config.clipboard.skip_sensitive" :disabled="!config.clipboard.enabled" />
    </setting-row>
    <setting-row :label="$t('settings.clipboard.maxSize')" :disabled="!config.clipboard.enabled">
      <number-input
        v-model="config.clipboard.max_size_mb"
        :min="1"
        :max="1024"
        :precision="0"
        :disabled="!config.clipboard.enabled"
        :suffix="$t('settings.clipboard.mb')"
      />
    </setting-row>
  </div>
</template>
