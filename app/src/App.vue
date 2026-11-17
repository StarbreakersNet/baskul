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
} from "naive-ui";
import AppLayout from "@/AppLayout.vue";
import { usePreferences } from "@/composables/usePreferences";
import { themeOverrides } from "@/utils/naiveTheme";

const { isDark, locale } = usePreferences();

const naiveLocale = computed(() => (locale.value === "fr" ? frFR : enUS));
const naiveDateLocale = computed(() => (locale.value === "fr" ? dateFrFR : dateEnUS));

const overrides = computed(() => themeOverrides(isDark.value));
</script>

<template>
  <n-config-provider
    :date-locale="naiveDateLocale"
    :locale="naiveLocale"
    :theme="isDark ? darkTheme : null"
    :theme-overrides="overrides">
    <n-global-style />
    <n-message-provider placement="top">
      <n-dialog-provider>
        <app-layout />
      </n-dialog-provider>
    </n-message-provider>
  </n-config-provider>
</template>
