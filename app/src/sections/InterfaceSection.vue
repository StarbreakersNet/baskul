<script setup lang="ts">
// Apparence et langue. Le thème et la langue s'appliquent aussitôt à cette
// fenêtre ; les menus de Baskul suivent après l'enregistrement.
import { NSwitch } from "naive-ui";
import ChoiceGroup from "../components/ChoiceGroup.vue";
import SettingRow from "../components/SettingRow.vue";
import type { BaskulConfig } from "../types";

defineProps<{ config: BaskulConfig }>();
</script>

<template>
  <div>
    <setting-row :label="$t('settings.interface.theme')" :help="$t('settings.interface.themeHelp')">
      <choice-group
        v-model="config.ui.theme"
        :options="[
          { value: 'system', label: $t('settings.interface.system') },
          { value: 'light', label: $t('settings.interface.light') },
          { value: 'dark', label: $t('settings.interface.dark') },
        ]"
      />
    </setting-row>
    <setting-row :label="$t('settings.interface.language')" :help="$t('settings.interface.languageHelp')">
      <choice-group
        v-model="config.ui.language"
        :options="[
          { value: 'system', label: $t('settings.interface.systemLanguage') },
          { value: 'fr', label: 'Français' },
          { value: 'en', label: 'English' },
        ]"
      />
    </setting-row>
    <setting-row :label="$t('settings.interface.trayIcon')" :help="$t('settings.interface.trayIconHelp')">
      <choice-group
        v-model="config.ui.tray_icon"
        :options="[
          { value: 'color', label: $t('settings.interface.color') },
          { value: 'mono', label: $t('settings.interface.mono') },
        ]"
      />
    </setting-row>
    <setting-row :label="$t('settings.interface.bubble')" :help="$t('settings.interface.bubbleHelp')">
      <n-switch v-model:value="config.ui.arrival_bubble" />
    </setting-row>
  </div>
</template>
