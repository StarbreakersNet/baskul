<script setup lang="ts">
import {
  IconClipboard,
  IconCode,
  IconFileDescription,
  IconFiles,
  IconFolders,
  IconPhoto,
  IconShieldLock,
} from "@tabler/icons-vue";
import { computed } from "vue";
import HSwitch from "@/components/c/hSwitch.vue";
import NumberInput from "@/components/NumberInput.vue";
import SettingRow from "@/components/SettingRow.vue";
import type { BaskulConfig } from "@/types/Config";

const config = defineModel<BaskulConfig>("config", { required: true });

const isOff = computed(() => !config.value.clipboard.enabled);
const areFilesOff = computed(() => isOff.value || !config.value.clipboard.files);
</script>

<template>
  <div>
    <setting-row :icon="IconClipboard" :label="$t('settings.clipboard.share')">
      <h-switch v-model:value="config.clipboard.enabled" />
    </setting-row>
    <setting-row :icon="IconPhoto" :disabled="isOff" :label="$t('settings.clipboard.images')">
      <h-switch v-model:value="config.clipboard.images" :disabled="isOff" />
    </setting-row>
    <setting-row
      :icon="IconCode"
      :disabled="isOff"
      :help="$t('settings.clipboard.htmlHelp')"
      :label="$t('settings.clipboard.html')">
      <h-switch v-model:value="config.clipboard.html" :disabled="isOff" />
    </setting-row>
    <setting-row
      :icon="IconFiles"
      :disabled="isOff"
      :help="$t('settings.clipboard.filesHelp')"
      :label="$t('settings.clipboard.files')">
      <h-switch v-model:value="config.clipboard.files" :disabled="isOff" />
    </setting-row>
    <setting-row
      :icon="IconShieldLock"
      :disabled="isOff"
      :help="$t('settings.clipboard.sensitiveHelp')"
      :label="$t('settings.clipboard.sensitive')">
      <h-switch v-model:value="config.clipboard.skip_sensitive" :disabled="isOff" />
    </setting-row>
    <setting-row
      :icon="IconFileDescription"
      :disabled="isOff"
      :help="$t('settings.clipboard.maxSizeHelp')"
      :label="$t('settings.clipboard.maxSize')">
      <number-input
        v-model="config.clipboard.max_size_mb"
        :disabled="isOff"
        :max="1024"
        :min="1"
        :precision="0"
        :suffix="$t('settings.clipboard.mb')" />
    </setting-row>
    <setting-row
      :icon="IconFolders"
      :disabled="areFilesOff"
      :label="$t('settings.clipboard.maxFiles')">
      <number-input
        v-model="config.clipboard.max_files_mb"
        :disabled="areFilesOff"
        :max="65536"
        :min="1"
        :precision="0"
        :suffix="$t('settings.clipboard.mb')" />
    </setting-row>
  </div>
</template>
