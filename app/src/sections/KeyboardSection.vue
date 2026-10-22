<script setup lang="ts">
import { computed } from "vue";
import { NButton } from "naive-ui";
import ChoiceGroup from "../components/ChoiceGroup.vue";
import LinuxNotice from "../components/LinuxNotice.vue";
import SettingRow from "../components/SettingRow.vue";
import type { BasculeConfig } from "../types";

const props = defineProps<{ config: BasculeConfig }>();
const emit = defineEmits<{ openFile: [] }>();
const rules = computed(() => Object.keys(props.config.keyboard.remap).length);
</script>

<template>
  <div class="pane">
    <linux-notice />
    <setting-row
      label="Touches multimédia"
      help="Volume, lecture… : sur la machine où se trouve le curseur, ou toujours sur l'une des deux machines."
      machine="primary"
    >
      <choice-group
        v-model="config.keyboard.media_keys"
        :options="[
          { value: 'cursor', label: 'Machine du curseur' },
          { value: 'primary', label: 'Machine principale' },
          { value: 'secondary', label: 'Machine pilotée' },
        ]"
      />
    </setting-row>
    <setting-row label="Remappage de touches" machine="secondary">
      <template #help>Section <code>[keyboard.remap]</code> du fichier de configuration.</template>
      <span class="muted">{{ rules ? `${rules} règle${rules > 1 ? "s" : ""}` : "Aucune règle" }}</span>
      <n-button size="small" @click="emit('openFile')">Ouvrir le fichier</n-button>
    </setting-row>
  </div>
</template>
