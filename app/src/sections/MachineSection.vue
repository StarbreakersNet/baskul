<script setup lang="ts">
import { NCard, NInput } from "naive-ui";
import SettingRow from "../components/SettingRow.vue";
import type { BasculeConfig, DaemonInfo, Role } from "../types";

// Les sections modifient directement l'objet `config` partagé (formulaire).
defineProps<{ config: BasculeConfig; info: DaemonInfo }>();

const roles: { value: Role; title: string; desc: string }[] = [
  { value: "primary", title: "Principale", desc: "Le clavier et la souris sont branchés sur cette machine (en général le PC)." },
  { value: "secondary", title: "Pilotée", desc: "Cette machine est contrôlée depuis l'autre (en général le Mac)." },
];
</script>

<template>
  <n-card title="Cette machine" class="section">
    <div class="roles" role="radiogroup" aria-label="Rôle de cette machine">
      <button
        v-for="r in roles"
        :key="r.value"
        type="button"
        role="radio"
        :aria-checked="config.role === r.value"
        :class="['role-card', { on: config.role === r.value }]"
        @click="config.role = r.value"
      >
        <span class="role-title">{{ r.title }}</span>
        <span class="role-desc">{{ r.desc }}</span>
      </button>
    </div>
    <setting-row label="Nom affiché" help="Facultatif. Par défaut, le nom de la machine.">
      <n-input v-model:value="config.name" :placeholder="info.hostname" />
    </setting-row>
  </n-card>
</template>
