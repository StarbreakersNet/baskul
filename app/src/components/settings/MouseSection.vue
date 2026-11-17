<script setup lang="ts">
import {
  IconArrowBackUp,
  IconArrowsHorizontal,
  IconArrowsVertical,
  IconBolt,
  IconGauge,
  IconHandFinger,
  IconListNumbers,
  IconMouse,
  IconRuler,
  IconTarget,
  IconWaveSine,
} from "@tabler/icons-vue";
import { computed, inject } from "vue";
import { useI18n } from "vue-i18n";
import HSwitch from "@/components/c/hSwitch.vue";
import ChoiceGroup from "@/components/ChoiceGroup.vue";
import LinuxNotice from "@/components/LinuxNotice.vue";
import NumberInput from "@/components/NumberInput.vue";
import SettingRow from "@/components/SettingRow.vue";
import { OS_KEY } from "@/utils/injectionKeys";
import type { BaskulConfig } from "@/types/Config";
import type { ChoiceOption } from "@/types/Choice";

type ScrollConfig = BaskulConfig["scroll"];

const config = defineModel<BaskulConfig>("config", { required: true });

const os = inject(OS_KEY);
const { t } = useI18n();

const scrollMode = computed(() => config.value.scroll.mode);
const modeOptions = computed<ChoiceOption<ScrollConfig["mode"]>[]>(() => [
  { value: "lines", label: t("settings.mouse.lines") },
  { value: "pixels", label: t("settings.mouse.pixels") },
  { value: "smooth", label: t("settings.mouse.smooth"), disabled: os?.value !== "macos" },
]);
const speedOptions = computed<ChoiceOption<ScrollConfig["smooth_speed"]>[]>(() => [
  { value: "low", label: t("settings.mouse.slow") },
  { value: "medium", label: t("settings.mouse.medium") },
  { value: "high", label: t("settings.mouse.fast") },
]);
const smoothnessOptions = computed<ChoiceOption<ScrollConfig["smoothness"]>[]>(() => [
  { value: "regular", label: t("settings.mouse.regular") },
  { value: "high", label: t("settings.mouse.high") },
]);
const sideButtonOptions = computed<ChoiceOption<BaskulConfig["buttons"]["side_buttons"]>[]>(() => [
  { value: "buttons", label: t("settings.mouse.buttons") },
  { value: "swipe", label: t("settings.mouse.swipe") },
]);
</script>

<template>
  <div>
    <linux-notice />
    <setting-row
      :icon="IconGauge"
      :help="$t('settings.mouse.speedHelp')"
      :label="$t('settings.mouse.speed')">
      <number-input v-model="config.mouse.speed" :max="20" :min="0.05" :step="0.05" suffix="×" />
    </setting-row>
    <setting-row
      :icon="IconMouse"
      :help="$t('settings.mouse.scrollingHelp')"
      :label="$t('settings.mouse.scrolling')">
      <choice-group v-model="config.scroll.mode" :options="modeOptions" />
    </setting-row>
    <setting-row
      :icon="IconBolt"
      :help="$t('settings.mouse.smoothSpeedHelp')"
      :label="$t('settings.mouse.smoothSpeed')"
      :show="scrollMode === 'smooth'">
      <choice-group v-model="config.scroll.smooth_speed" :options="speedOptions" />
    </setting-row>
    <setting-row
      :icon="IconWaveSine"
      :help="$t('settings.mouse.smoothnessHelp')"
      :label="$t('settings.mouse.smoothness')"
      :show="scrollMode === 'smooth'">
      <choice-group v-model="config.scroll.smoothness" :options="smoothnessOptions" />
    </setting-row>
    <setting-row
      :icon="IconTarget"
      :help="$t('settings.mouse.precisionHelp')"
      :label="$t('settings.mouse.precision')"
      :show="scrollMode === 'smooth'">
      <h-switch v-model:value="config.scroll.smooth_precision" />
    </setting-row>
    <setting-row
      :icon="IconHandFinger"
      :help="$t('settings.mouse.trackpadHelp')"
      :label="$t('settings.mouse.trackpad')"
      :show="scrollMode === 'smooth' && config.scroll.smoothness === 'high'">
      <h-switch v-model:value="config.scroll.trackpad_simulation" />
    </setting-row>
    <setting-row
      :icon="IconListNumbers"
      :label="$t('settings.mouse.linesPerNotch')"
      :show="scrollMode === 'lines'">
      <number-input v-model="config.scroll.lines_per_notch" :max="50" :min="0.1" :step="0.5" />
    </setting-row>
    <setting-row
      :icon="IconRuler"
      :label="$t('settings.mouse.pixelsPerNotch')"
      :show="scrollMode === 'pixels'">
      <number-input v-model="config.scroll.pixels_per_notch" :max="2000" :min="1" :step="10" />
    </setting-row>
    <setting-row :icon="IconArrowsVertical" :label="$t('settings.mouse.invertVertical')">
      <h-switch v-model:value="config.scroll.invert_vertical" />
    </setting-row>
    <setting-row :icon="IconArrowsHorizontal" :label="$t('settings.mouse.invertHorizontal')">
      <h-switch v-model:value="config.scroll.invert_horizontal" />
    </setting-row>
    <setting-row
      :icon="IconArrowBackUp"
      :help="$t('settings.mouse.sideButtonsHelp')"
      :label="$t('settings.mouse.sideButtons')"
      os="macos">
      <choice-group v-model="config.buttons.side_buttons" :options="sideButtonOptions" />
    </setting-row>
  </div>
</template>
