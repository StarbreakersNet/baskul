<script setup lang="ts">
import { NButton, NSelect } from "naive-ui";
import ChoiceGroup from "../components/ChoiceGroup.vue";
import SettingRow from "../components/SettingRow.vue";
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
  <div class="pane">
    <setting-row
      label="Touches « ² » et « < > »"
      help="Les claviers européens (ISO) du Mac inversent ces deux touches par rapport à un clavier de PC. « Auto » corrige selon le clavier du Mac : à changer seulement si elles restent inversées."
      machine="secondary"
      os="macos"
    >
      <choice-group
        v-model="config.keyboard.iso_swap"
        :options="[
          { value: 'auto', label: 'Auto' },
          { value: 'on', label: 'Échanger' },
          { value: 'off', label: 'Ne pas échanger' },
        ]"
      />
    </setting-row>
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
  </div>
</template>
