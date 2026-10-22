<script setup lang="ts">
import { computed, inject } from "vue";
import { NSwitch } from "naive-ui";
import ChoiceGroup from "../components/ChoiceGroup.vue";
import LinuxNotice from "../components/LinuxNotice.vue";
import NumberInput from "../components/NumberInput.vue";
import SettingRow from "../components/SettingRow.vue";
import { osKey } from "../keys";
import type { BasculeConfig } from "../types";

const props = defineProps<{ config: BasculeConfig }>();
const os = inject(osKey);
const mode = computed(() => props.config.scroll.mode);
</script>

<template>
  <div class="pane">
    <linux-notice />
    <setting-row label="Vitesse de la souris" help="Multiplie les déplacements reçus. 1 = identique.">
      <number-input v-model="config.mouse.speed" :min="0.05" :max="20" :step="0.05" suffix="×" />
    </setting-row>
    <setting-row label="Défilement">
      <template #help>
        Par lignes, comme une molette Windows ; au pixel, plus précis ; fluide, chaque cran est animé comme avec Mac
        Mouse Fix (macOS).
      </template>
      <choice-group
        v-model="config.scroll.mode"
        :options="[
          { value: 'lines', label: 'Par lignes' },
          { value: 'pixels', label: 'Au pixel' },
          { value: 'smooth', label: 'Fluide', disabled: os !== 'macos' },
        ]"
      />
    </setting-row>
    <setting-row label="Lignes par cran" :disabled="mode !== 'lines'">
      <number-input
        v-model="config.scroll.lines_per_notch"
        :min="0.1"
        :max="50"
        :step="0.5"
        :disabled="mode !== 'lines'"
      />
    </setting-row>
    <setting-row label="Pixels par cran" :disabled="mode === 'lines'">
      <number-input
        v-model="config.scroll.pixels_per_notch"
        :min="1"
        :max="2000"
        :step="10"
        :disabled="mode === 'lines'"
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
      os="macos"
    >
      <choice-group
        v-model="config.buttons.side_buttons"
        :options="[
          { value: 'buttons', label: 'Boutons' },
          { value: 'swipe', label: 'Geste balayage' },
        ]"
      />
    </setting-row>
  </div>
</template>
