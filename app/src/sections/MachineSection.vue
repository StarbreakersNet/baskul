<script setup lang="ts">
import { computed } from "vue";
import { NCard, NGi, NGrid, NInput, NText, useThemeVars } from "naive-ui";
import SettingRow from "../components/SettingRow.vue";
import type { BasculeConfig, DaemonInfo, Role } from "../types";

// Les sections modifient directement l'objet `config` partagé (formulaire).
defineProps<{ config: BasculeConfig; info: DaemonInfo }>();

const roles: { value: Role; title: string; desc: string }[] = [
  { value: "primary", title: "Principale", desc: "Le clavier et la souris sont branchés sur cette machine (en général le PC)." },
  { value: "secondary", title: "Pilotée", desc: "Cette machine est contrôlée depuis l'autre (en général le Mac)." },
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
  <n-card title="Cette machine">
    <n-grid cols="1 480:2" responsive="self" :x-gap="10" :y-gap="10" role="radiogroup" aria-label="Rôle de cette machine">
      <n-gi v-for="r in roles" :key="r.value">
        <button
          type="button"
          role="radio"
          :aria-checked="config.role === r.value"
          :class="['role', { on: config.role === r.value }]"
          @click="config.role = r.value"
        >
          <n-text strong tag="div">{{ r.title }}</n-text>
          <n-text depth="3" tag="div" class="desc">{{ r.desc }}</n-text>
        </button>
      </n-gi>
    </n-grid>
    <setting-row label="Nom affiché" help="Facultatif. Par défaut, le nom de la machine.">
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
