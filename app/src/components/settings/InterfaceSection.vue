<script setup lang="ts">
import {
  IconBell,
  IconLanguage,
  IconLayoutNavbar,
  IconMessageCircle,
  IconPalette,
} from "@tabler/icons-vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import HSwitch from "@/components/c/hSwitch.vue";
import ChoiceGroup from "@/components/ChoiceGroup.vue";
import SettingRow from "@/components/SettingRow.vue";
import type { BaskulConfig } from "@/types/Config";
import type { ChoiceOption } from "@/types/Choice";

type UiConfig = BaskulConfig["ui"];

const config = defineModel<BaskulConfig>("config", { required: true });

const { t } = useI18n();

const themeOptions = computed<ChoiceOption<UiConfig["theme"]>[]>(() => [
  { value: "system", label: t("settings.interface.system") },
  { value: "light", label: t("settings.interface.light") },
  { value: "dark", label: t("settings.interface.dark") },
]);
const languageOptions = computed<ChoiceOption<UiConfig["language"]>[]>(() => [
  { value: "system", label: t("settings.interface.systemLanguage") },
  { value: "fr", label: "Français" },
  { value: "en", label: "English" },
]);
const trayIconOptions = computed<ChoiceOption<UiConfig["tray_icon"]>[]>(() => [
  { value: "color", label: t("settings.interface.color") },
  { value: "mono", label: t("settings.interface.mono") },
]);
</script>

<template>
  <div>
    <setting-row
      :icon="IconPalette"
      :help="$t('settings.interface.themeHelp')"
      :label="$t('settings.interface.theme')">
      <choice-group v-model="config.ui.theme" :options="themeOptions" />
    </setting-row>
    <setting-row
      :icon="IconLanguage"
      :help="$t('settings.interface.languageHelp')"
      :label="$t('settings.interface.language')">
      <choice-group v-model="config.ui.language" :options="languageOptions" />
    </setting-row>
    <setting-row
      :icon="IconLayoutNavbar"
      :help="$t('settings.interface.trayIconHelp')"
      :label="$t('settings.interface.trayIcon')">
      <choice-group v-model="config.ui.tray_icon" :options="trayIconOptions" />
    </setting-row>
    <setting-row
      :icon="IconMessageCircle"
      :help="$t('settings.interface.bubbleHelp')"
      :label="$t('settings.interface.bubble')">
      <h-switch v-model:value="config.ui.arrival_bubble" />
    </setting-row>
    <setting-row
      :icon="IconBell"
      :help="$t('settings.interface.notificationsHelp')"
      :label="$t('settings.interface.notifications')">
      <h-switch v-model:value="config.ui.notifications" />
    </setting-row>
  </div>
</template>
