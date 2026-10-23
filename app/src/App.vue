<script setup lang="ts">
import { computed } from "vue";
import {
  NConfigProvider,
  NDialogProvider,
  NGlobalStyle,
  NMessageProvider,
  darkTheme,
  dateEnUS,
  dateFrFR,
  enUS,
  frFR,
  type GlobalThemeOverrides,
} from "naive-ui";
import { usePreferences } from "./composables/usePreferences";
import AppLayout from "./layouts/AppLayout.vue";

// Thème (système, clair ou sombre) et langue choisis dans les réglages.
const { dark, locale } = usePreferences();
const naiveLocale = computed(() => (locale.value === "fr" ? frFR : enUS));
const naiveDateLocale = computed(() => (locale.value === "fr" ? dateFrFR : dateEnUS));

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
  <n-config-provider
    :theme="dark ? darkTheme : null"
    :theme-overrides="overrides"
    :locale="naiveLocale"
    :date-locale="naiveDateLocale"
  >
    <n-global-style />
    <n-message-provider placement="top">
      <n-dialog-provider>
        <app-layout />
      </n-dialog-provider>
    </n-message-provider>
  </n-config-provider>
</template>
