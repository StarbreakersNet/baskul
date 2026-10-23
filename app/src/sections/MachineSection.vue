<script setup lang="ts">
import { computed } from "vue";
import { NCard, NGi, NGrid, NInput, NText, useThemeVars } from "naive-ui";
import SettingRow from "../components/SettingRow.vue";
import type { BasculeConfig, DaemonInfo, Role } from "../types";

// Les sections modifient directement l'objet `config` partagé (formulaire).
defineProps<{ config: BasculeConfig; info: DaemonInfo }>();

const roles: { value: Role; key: string }[] = [
  { value: "primary", key: "primary" },
  { value: "secondary", key: "secondary" },
];

const vars = useThemeVars();
const colors = computed(() => ({
  border: vars.value.borderColor,
  accent: vars.value.primaryColor,
  // Fond teinté de la couleur d'accent (14 %).
  soft: `color-mix(in srgb, ${vars.value.primaryColor} 14%, transparent)`,
}));
</script>

<template>
  <n-card :title="$t('settings.machine.title')">
    <n-grid
      cols="1 480:2"
      responsive="self"
      :x-gap="10"
      :y-gap="10"
      role="radiogroup"
      :aria-label="$t('settings.machine.aria')"
    >
      <n-gi v-for="r in roles" :key="r.value">
        <button
          type="button"
          role="radio"
          :aria-checked="config.role === r.value"
          :class="['role', { on: config.role === r.value }]"
          @click="config.role = r.value"
        >
          <n-text strong tag="div">{{ $t(`settings.machine.${r.key}`) }}</n-text>
          <n-text depth="3" tag="div" class="desc">{{ $t(`settings.machine.${r.key}Desc`) }}</n-text>
        </button>
      </n-gi>
    </n-grid>
    <setting-row :label="$t('settings.machine.name')" :help="$t('settings.machine.nameHelp')">
      <n-input v-model:value="config.name" :placeholder="info.hostname" />
    </setting-row>
  </n-card>
</template>

<style scoped>
.role {
  width: 100%;
  height: 100%;
  font: inherit;
  text-align: left;
  cursor: pointer;
  padding: 12px 14px;
  border-radius: 10px;
  border: 1.5px solid v-bind("colors.border");
  background: transparent;
}
.role.on {
  border-color: v-bind("colors.accent");
  background: v-bind("colors.soft");
}
.desc {
  margin-top: 2px;
  font-size: 13px;
}
</style>
