<script setup lang="ts">
import { computed } from "vue";
import { NAlert, NSwitch } from "naive-ui";
import HotkeyInput from "../components/HotkeyInput.vue";
import NumberInput from "../components/NumberInput.vue";
import PositionPicker from "../components/PositionPicker.vue";
import SettingRow from "../components/SettingRow.vue";
import SettingsSection from "../components/SettingsSection.vue";
import type { BasculeConfig, DaemonInfo } from "../types";

const props = defineProps<{ config: BasculeConfig; info: DaemonInfo }>();
const edges = computed(() => props.config.screen.edge_switching);
const missingHotkey = computed(
  () => !props.config.hotkeys.switch_to_secondary || !props.config.hotkeys.return_to_primary,
);
</script>

<template>
  <settings-section title="Passer d'une machine à l'autre" machine="primary">
    <setting-row label="Aller sur la machine pilotée" help="Le curseur y reprend là où il était.">
      <hotkey-input v-model="config.hotkeys.switch_to_secondary" :os="info.os" />
    </setting-row>
    <setting-row label="Revenir sur cette machine" help="Le même raccourci que ci-dessus permet d'aller et venir.">
      <hotkey-input v-model="config.hotkeys.return_to_primary" :os="info.os" />
    </setting-row>
    <n-alert v-if="!edges && missingHotkey && config.role === 'primary'" type="warning" class="inline-alert">
      Sans bascule par les bords, ces deux raccourcis sont indispensables.
    </n-alert>
    <setting-row label="Bascule par les bords de l'écran" help="Pousser la souris contre le bord fait passer sur l'autre machine.">
      <n-switch v-model:value="config.screen.edge_switching" />
    </setting-row>
    <setting-row
      label="Position de l'autre écran"
      help="Où se trouve l'écran de la machine pilotée par rapport à celui-ci."
      :disabled="!edges"
    >
      <position-picker v-model="config.screen.secondary_position" :disabled="!edges" />
    </setting-row>
    <setting-row
      label="Coins neutres"
      help="Dans les coins, le bord ne fait pas changer de machine (bouton ✕ d'une fenêtre…). 0 = aucun."
      :disabled="!edges"
    >
      <number-input v-model="config.screen.corner_size" :min="0" :max="500" :precision="0" :disabled="!edges" suffix="px" />
    </setting-row>
    <setting-row label="Verrouiller le curseur" help="Bloque ou débloque le curseur sur l'écran où il se trouve.">
      <hotkey-input v-model="config.hotkeys.toggle_lock" :os="info.os" />
    </setting-row>
  </settings-section>
</template>
