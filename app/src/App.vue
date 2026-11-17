<script setup lang="ts">
import { computed } from "vue";
import {
  darkTheme,
  dateEnUS,
  dateFrFR,
  enUS,
  frFR,
  NConfigProvider,
  NDialogProvider,
  NGlobalStyle,
  NMessageProvider,
  type GlobalThemeOverrides,
} from "naive-ui";
import AppLayout from "@/AppLayout.vue";
import { usePreferences } from "@/composables/usePreferences";

const { isDark, locale } = usePreferences();

const naiveLocale = computed(() => (locale.value === "fr" ? frFR : enUS));
const naiveDateLocale = computed(() => (locale.value === "fr" ? dateFrFR : dateEnUS));

const themeOverrides = computed<GlobalThemeOverrides>(() => ({
  common: {
    primaryColor: isDark.value ? "#6d9eff" : "#2563eb",
    primaryColorHover: isDark.value ? "#8db2ff" : "#3b74f0",
    primaryColorPressed: isDark.value ? "#5585ee" : "#1d4ed8",
    primaryColorSuppl: isDark.value ? "#8db2ff" : "#3b74f0",
    borderRadius: "8px",
    fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", system-ui, sans-serif',
    ...(isDark.value ? {} : { bodyColor: "#f4f5f7" }),
  },
}));
</script>

<template>
  <n-config-provider
    :date-locale="naiveDateLocale"
    :locale="naiveLocale"
    :theme="isDark ? darkTheme : null"
    :theme-overrides="themeOverrides">
    <n-global-style />
    <n-message-provider placement="top">
      <n-dialog-provider>
        <app-layout />
      </n-dialog-provider>
    </n-message-provider>
  </n-config-provider>
</template>
