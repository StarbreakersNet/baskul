<script setup lang="ts">
import { NSwitch } from "naive-ui";
import ChoiceGroup from "../components/ChoiceGroup.vue";
import NumberInput from "../components/NumberInput.vue";
import SettingRow from "../components/SettingRow.vue";
import SettingsSection from "../components/SettingsSection.vue";
import type { BasculeConfig } from "../types";

defineProps<{ config: BasculeConfig }>();
</script>

<template>
  <settings-section title="Souris et molette" machine="secondary">
    <setting-row label="Vitesse de la souris" help="Multiplie les déplacements reçus. 1 = identique.">
      <number-input v-model="config.mouse.speed" :min="0.05" :max="20" :step="0.05" suffix="×" />
    </setting-row>
    <setting-row label="Défilement" help="Par lignes, comme une molette Windows, ou au pixel, plus fluide.">
      <choice-group
        v-model="config.scroll.mode"
        :options="[
          { value: 'lines', label: 'Par lignes' },
          { value: 'pixels', label: 'Au pixel' },
        ]"
      />
    </setting-row>
    <setting-row label="Lignes par cran" :disabled="config.scroll.mode !== 'lines'">
      <number-input
        v-model="config.scroll.lines_per_notch"
        :min="0.1"
        :max="50"
        :step="0.5"
        :disabled="config.scroll.mode !== 'lines'"
      />
    </setting-row>
    <setting-row label="Pixels par cran" :disabled="config.scroll.mode !== 'pixels'">
      <number-input
        v-model="config.scroll.pixels_per_notch"
        :min="1"
        :max="2000"
        :step="10"
        :disabled="config.scroll.mode !== 'pixels'"
      />
    </setting-row>
    <setting-row label="Inverser le défilement vertical">
      <n-switch v-model:value="config.scroll.invert_vertical" />
    </setting-row>
    <setting-row label="Inverser le défilement horizontal">
      <n-switch v-model:value="config.scroll.invert_horizontal" />
    </setting-row>
    <setting-row
      label="Boutons latéraux (4 et 5)"
      help="Boutons pour Chrome, Firefox, VS Code… Geste « balayage » pour Safari, le Finder, Xcode…"
    >
      <choice-group
        v-model="config.buttons.side_buttons"
        :options="[
          { value: 'buttons', label: 'Boutons' },
          { value: 'swipe', label: 'Geste balayage' },
        ]"
      />
    </setting-row>
    <setting-row label="Inverser les boutons 4 et 5">
      <n-switch v-model:value="config.buttons.swap_side_buttons" />
    </setting-row>
  </settings-section>
</template>
