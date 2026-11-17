<script setup lang="ts">
import {
  IconBell,
  IconBlur,
  IconGauge,
  IconGridDots,
  IconLanguage,
  IconLayoutGrid,
  IconLayoutNavbar,
  IconMessageCircle,
  IconPalette,
  IconSquareHalf,
} from "@tabler/icons-vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import HButton from "@/components/c/hButton.vue";
import HSlider from "@/components/c/hSlider.vue";
import HSwitch from "@/components/c/hSwitch.vue";
import HText from "@/components/c/hText.vue";
import ChoiceGroup from "@/components/ChoiceGroup.vue";
import SettingRow from "@/components/SettingRow.vue";
import { useBackgroundTuningStore } from "@/stores/backgroundTuning";
import type { BaskulConfig } from "@/types/Config";
import type { ChoiceOption } from "@/types/Choice";

type UiConfig = BaskulConfig["ui"];

const config = defineModel<BaskulConfig>("config", { required: true });

const { t } = useI18n();
const backgroundTuning = useBackgroundTuningStore();

const themeOptions = computed<ChoiceOption<UiConfig["theme"]>[]>(() => [
  { value: "system", label: t("settings.interface.system") },
  { value: "light", label: t("settings.interface.light") },
  { value: "dark", label: t("settings.interface.dark") },
]);
const languageOptions = computed<ChoiceOption<UiConfig["language"]>[]>(() => [
  { value: "system", label: t("settings.interface.systemLanguage") },
  { value: "fr", label: "Français" },
  { value: "en", label: "English" },
]);
const backgroundOptions = computed<ChoiceOption<UiConfig["background"]>[]>(() => [
  { value: "none", label: t("settings.interface.backgroundNone") },
  { value: "current", label: t("settings.interface.backgroundCurrent") },
  { value: "wave", label: t("settings.interface.backgroundWave") },
  { value: "breathing", label: t("settings.interface.backgroundBreathing") },
  { value: "ripple", label: t("settings.interface.backgroundRipple") },
]);
const trayIconOptions = computed<ChoiceOption<UiConfig["tray_icon"]>[]>(() => [
  { value: "color", label: t("settings.interface.color") },
  { value: "mono", label: t("settings.interface.mono") },
]);
</script>

<template>
  <div>
    <setting-row
      :icon="IconPalette"
      :help="$t('settings.interface.themeHelp')"
      :label="$t('settings.interface.theme')">
      <choice-group v-model="config.ui.theme" :options="themeOptions" />
    </setting-row>
    <setting-row
      :icon="IconGridDots"
      :help="$t('settings.interface.backgroundHelp')"
      :label="$t('settings.interface.background')">
      <choice-group v-model="config.ui.background" :options="backgroundOptions" />
    </setting-row>
    <template v-if="backgroundTuning.canTune && config.ui.background !== 'none'">
      <setting-row
        :icon="IconGauge"
        :help="$t('settings.interface.tuningHelp')"
        :label="$t('settings.interface.tuningSpeed')">
        <h-slider
          v-model:value="backgroundTuning.tuning.speed"
          class="interface-slider"
          :max="2"
          :min="0"
          :step="0.05"
          :tooltip="false" />
        <h-text class="interface-value mono">{{ backgroundTuning.tuning.speed.toFixed(2) }}</h-text>
      </setting-row>
      <setting-row
        :icon="IconSquareHalf"
        :help="$t('settings.interface.tuningHelp')"
        :label="$t('settings.interface.tuningOpacity')">
        <h-slider
          v-model:value="backgroundTuning.tuning.frameOpacity"
          class="interface-slider"
          :max="1"
          :min="0.5"
          :step="0.01"
          :tooltip="false" />
        <h-text class="interface-value mono">
          {{
            $t("settings.interface.valuePercent", {
              value: Math.round(backgroundTuning.tuning.frameOpacity * 100),
            })
          }}
        </h-text>
      </setting-row>
      <setting-row
        :icon="IconBlur"
        :help="$t('settings.interface.tuningHelp')"
        :label="$t('settings.interface.tuningBlur')">
        <h-slider
          v-model:value="backgroundTuning.tuning.frameBlur"
          class="interface-slider"
          :max="24"
          :min="0"
          :step="1"
          :tooltip="false" />
        <h-text class="interface-value mono">
          {{ $t("settings.interface.valuePx", { value: backgroundTuning.tuning.frameBlur }) }}
        </h-text>
      </setting-row>
      <setting-row
        :icon="IconLayoutGrid"
        :help="$t('settings.interface.tuningHelp')"
        :label="$t('settings.interface.tuningSpacing')">
        <h-slider
          v-model:value="backgroundTuning.tuning.spacing"
          class="interface-slider"
          :max="20"
          :min="3"
          :step="1"
          :tooltip="false" />
        <h-text class="interface-value mono">
          {{ $t("settings.interface.valuePx", { value: backgroundTuning.tuning.spacing }) }}
        </h-text>
        <h-button size="small" @click="backgroundTuning.reset()">
          {{ $t("settings.interface.tuningReset") }}
        </h-button>
      </setting-row>
    </template>
    <setting-row
      :icon="IconLanguage"
      :help="$t('settings.interface.languageHelp')"
      :label="$t('settings.interface.language')">
      <choice-group v-model="config.ui.language" :options="languageOptions" />
    </setting-row>
    <setting-row
      :icon="IconLayoutNavbar"
      :help="$t('settings.interface.trayIconHelp')"
      :label="$t('settings.interface.trayIcon')">
      <choice-group v-model="config.ui.tray_icon" :options="trayIconOptions" />
    </setting-row>
    <setting-row
      :icon="IconMessageCircle"
      :help="$t('settings.interface.bubbleHelp')"
      :label="$t('settings.interface.bubble')">
      <h-switch v-model:value="config.ui.arrival_bubble" />
    </setting-row>
    <setting-row
      :icon="IconBell"
      :help="$t('settings.interface.notificationsHelp')"
      :label="$t('settings.interface.notifications')">
      <h-switch v-model:value="config.ui.notifications" />
    </setting-row>
  </div>
</template>

<style lang="scss" scoped>
.interface-slider {
  width: 180px;
}

.interface-value {
  min-width: 5ch;
  font-variant-numeric: tabular-nums;
}
</style>
