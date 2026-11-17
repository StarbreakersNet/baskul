<script setup lang="ts">
import { computed } from "vue";
import { useThemeVars } from "naive-ui";
import HCard from "@/components/c/hCard.vue";
import HGrid from "@/components/c/hGrid.vue";
import HGridItem from "@/components/c/hGridItem.vue";
import HInput from "@/components/c/hInput.vue";
import HText from "@/components/c/hText.vue";
import SettingRow from "@/components/SettingRow.vue";
import type { BaskulConfig, Role } from "@/types/Config";
import type { DaemonInfo } from "@/types/Daemon";

const config = defineModel<BaskulConfig>("config", { required: true });

const { info } = defineProps<{
  info: DaemonInfo;
}>();

const ROLES: Role[] = ["primary", "secondary"];

const themeVars = useThemeVars();

const colors = computed(() => ({
  border: themeVars.value.borderColor,
  accent: themeVars.value.primaryColor,
}));
</script>

<template>
  <h-card :title="$t('settings.machine.title')">
    <h-grid
      :aria-label="$t('settings.machine.aria')"
      cols="1 480:2"
      responsive="self"
      role="radiogroup"
      :x-gap="10"
      :y-gap="10">
      <h-grid-item v-for="role in ROLES" :key="role">
        <button
          :aria-checked="config.role === role"
          :class="['machine-role', { 'machine-role--on': config.role === role }]"
          role="radio"
          type="button"
          @click="config.role = role">
          <h-text strong tag="div">{{ $t(`settings.machine.${role}`) }}</h-text>
          <h-text class="help-text" depth="3" tag="div">
            {{ $t(`settings.machine.${role}Desc`) }}
          </h-text>
        </button>
      </h-grid-item>
    </h-grid>
    <setting-row :help="$t('settings.machine.nameHelp')" :label="$t('settings.machine.name')">
      <h-input v-model:value="config.name" :placeholder="info.hostname" />
    </setting-row>
  </h-card>
</template>

<style lang="scss" scoped>
@use "@/assets/variables";

.machine-role {
  width: 100%;
  height: 100%;
  font: inherit;
  text-align: left;
  cursor: pointer;
  padding: 12px 14px;
  border-radius: variables.$radius-large;
  border: 1.5px solid v-bind("colors.border");
  background: transparent;
}

.machine-role--on {
  border-color: v-bind("colors.accent");
  background: color-mix(in srgb, v-bind("colors.accent") variables.$accent-tint, transparent);
}
</style>
