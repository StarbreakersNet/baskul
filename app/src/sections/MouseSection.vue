<script setup lang="ts">
import { computed, inject } from "vue";
import { NSwitch } from "naive-ui";
import ChoiceGroup from "../components/ChoiceGroup.vue";
import LinuxNotice from "../components/LinuxNotice.vue";
import NumberInput from "../components/NumberInput.vue";
import SettingRow from "../components/SettingRow.vue";
import { osKey } from "../keys";
import type { BaskulConfig } from "../types";

const props = defineProps<{ config: BaskulConfig }>();
const os = inject(osKey);
const mode = computed(() => props.config.scroll.mode);
</script>

<template>
  <div>
    <linux-notice />
    <setting-row :label="$t('settings.mouse.speed')" :help="$t('settings.mouse.speedHelp')">
      <number-input v-model="config.mouse.speed" :min="0.05" :max="20" :step="0.05" suffix="×" />
    </setting-row>
    <setting-row :label="$t('settings.mouse.scrolling')" :help="$t('settings.mouse.scrollingHelp')">
      <choice-group
        v-model="config.scroll.mode"
        :options="[
          { value: 'lines', label: $t('settings.mouse.lines') },
          { value: 'pixels', label: $t('settings.mouse.pixels') },
          { value: 'smooth', label: $t('settings.mouse.smooth'), disabled: os !== 'macos' },
        ]"
      />
    </setting-row>
    <template v-if="mode === 'smooth'">
      <setting-row :label="$t('settings.mouse.smoothSpeed')" :help="$t('settings.mouse.smoothSpeedHelp')">
        <choice-group
          v-model="config.scroll.smooth_speed"
          :options="[
            { value: 'low', label: $t('settings.mouse.slow') },
            { value: 'medium', label: $t('settings.mouse.medium') },
            { value: 'high', label: $t('settings.mouse.fast') },
          ]"
        />
      </setting-row>
      <setting-row :label="$t('settings.mouse.smoothness')" :help="$t('settings.mouse.smoothnessHelp')">
        <choice-group
          v-model="config.scroll.smoothness"
          :options="[
            { value: 'regular', label: $t('settings.mouse.regular') },
            { value: 'high', label: $t('settings.mouse.high') },
          ]"
        />
      </setting-row>
      <setting-row :label="$t('settings.mouse.precision')" :help="$t('settings.mouse.precisionHelp')">
        <n-switch v-model:value="config.scroll.smooth_precision" />
      </setting-row>
      <setting-row
        v-if="config.scroll.smoothness === 'high'"
        :label="$t('settings.mouse.trackpad')"
        :help="$t('settings.mouse.trackpadHelp')"
      >
        <n-switch v-model:value="config.scroll.trackpad_simulation" />
      </setting-row>
    </template>
    <setting-row v-else-if="mode === 'lines'" :label="$t('settings.mouse.linesPerNotch')">
      <number-input v-model="config.scroll.lines_per_notch" :min="0.1" :max="50" :step="0.5" />
    </setting-row>
    <setting-row v-else :label="$t('settings.mouse.pixelsPerNotch')">
      <number-input v-model="config.scroll.pixels_per_notch" :min="1" :max="2000" :step="10" />
    </setting-row>
    <setting-row :label="$t('settings.mouse.invertVertical')">
      <n-switch v-model:value="config.scroll.invert_vertical" />
    </setting-row>
    <setting-row :label="$t('settings.mouse.invertHorizontal')">
      <n-switch v-model:value="config.scroll.invert_horizontal" />
    </setting-row>
    <setting-row
      :label="$t('settings.mouse.sideButtons')"
      :help="$t('settings.mouse.sideButtonsHelp')"
      os="macos"
    >
      <choice-group
        v-model="config.buttons.side_buttons"
        :options="[
          { value: 'buttons', label: $t('settings.mouse.buttons') },
          { value: 'swipe', label: $t('settings.mouse.swipe') },
        ]"
      />
    </setting-row>
  </div>
</template>
