<script setup lang="ts">
import { NButton, NSelect } from "naive-ui";
import SettingRow from "../components/SettingRow.vue";
import SettingsSection from "../components/SettingsSection.vue";
import type { BasculeConfig, DaemonInfo } from "../types";

defineProps<{ config: BasculeConfig; info: DaemonInfo; restarting?: boolean }>();
const emit = defineEmits<{ openFile: []; restart: [] }>();

const levels = [
  { label: "Erreurs seulement", value: "error" },
  { label: "Avertissements", value: "warn" },
  { label: "Normal", value: "info" },
  { label: "Détaillé", value: "debug" },
  { label: "Très détaillé", value: "trace" },
];
</script>

<template>
  <settings-section title="Avancé">
    <setting-row label="Niveau du journal">
      <n-select v-model:value="config.log.level" :options="levels" class="select" />
    </setting-row>
    <setting-row label="Fichier de configuration">
      <template #help><span class="mono path">{{ info.path }}</span></template>
      <n-button size="small" @click="emit('openFile')">Ouvrir le fichier</n-button>
    </setting-row>
    <setting-row label="Bascule" :help="`Version ${info.version}`">
      <n-button size="small" :loading="restarting" @click="emit('restart')">Redémarrer Bascule</n-button>
    </setting-row>
  </settings-section>
</template>
