<script setup lang="ts">
import { computed } from "vue";
import { NButton } from "naive-ui";
import ChoiceGroup from "../components/ChoiceGroup.vue";
import SettingRow from "../components/SettingRow.vue";
import SettingsSection from "../components/SettingsSection.vue";
import type { BasculeConfig } from "../types";

const props = defineProps<{ config: BasculeConfig }>();
const emit = defineEmits<{ openFile: [] }>();
const rules = computed(() => Object.keys(props.config.keyboard.remap).length);
</script>

<template>
  <settings-section title="Clavier">
    <setting-row
      label="Touches multimédia"
      help="Volume, lecture… sur la machine du curseur, ou toujours sur la machine principale."
      machine="primary"
    >
      <choice-group
        v-model="config.keyboard.media_keys"
        :options="[
          { value: 'remote', label: 'Machine du curseur' },
          { value: 'local', label: 'Machine principale' },
        ]"
      />
    </setting-row>
    <setting-row
      label="Touches « ² » et « < > » (Mac)"
      help="Échange pour les claviers ISO (français). Si ces deux touches sont inversées, essayez « Oui » ou « Non »."
      machine="secondary"
    >
      <choice-group
        v-model="config.keyboard.iso_swap"
        :options="[
          { value: 'auto', label: 'Auto' },
          { value: 'on', label: 'Oui' },
          { value: 'off', label: 'Non' },
        ]"
      />
    </setting-row>
    <setting-row label="Remappage de touches" machine="secondary">
      <template #help>Section <code>[keyboard.remap]</code> du fichier de configuration.</template>
      <span class="muted">{{ rules ? `${rules} règle${rules > 1 ? "s" : ""}` : "Aucune règle" }}</span>
      <n-button size="small" @click="emit('openFile')">Ouvrir le fichier</n-button>
    </setting-row>
  </settings-section>
</template>
