<script setup lang="ts">
import { computed } from "vue";
import {
  NConfigProvider,
  NDialogProvider,
  NGlobalStyle,
  NMessageProvider,
  darkTheme,
  dateFrFR,
  frFR,
  useOsTheme,
  type GlobalThemeOverrides,
} from "naive-ui";
import SettingsView from "./SettingsView.vue";

// Thème clair ou sombre selon le système.
const osTheme = useOsTheme();
const dark = computed(() => osTheme.value === "dark");

const overrides = computed<GlobalThemeOverrides>(() => ({
  common: {
    primaryColor: dark.value ? "#6d9eff" : "#2563eb",
    primaryColorHover: dark.value ? "#8db2ff" : "#3b74f0",
    primaryColorPressed: dark.value ? "#5585ee" : "#1d4ed8",
    primaryColorSuppl: dark.value ? "#8db2ff" : "#3b74f0",
    borderRadius: "8px",
    fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", system-ui, sans-serif',
    ...(dark.value ? {} : { bodyColor: "#f4f5f7" }),
  },
}));
</script>

<template>
  <n-config-provider :theme="dark ? darkTheme : null" :theme-overrides="overrides" :locale="frFR" :date-locale="dateFrFR">
    <n-global-style />
    <n-message-provider placement="top">
      <n-dialog-provider>
        <settings-view />
      </n-dialog-provider>
    </n-message-provider>
  </n-config-provider>
</template>
