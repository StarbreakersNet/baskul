<script setup lang="ts">
import { computed } from "vue";
import { NAlert, NSwitch } from "naive-ui";
import HotkeyInput from "../components/HotkeyInput.vue";
import NumberInput from "../components/NumberInput.vue";
import PositionPicker from "../components/PositionPicker.vue";
import SettingRow from "../components/SettingRow.vue";
import LinuxNotice from "../components/LinuxNotice.vue";
import type { BaskulConfig, DaemonInfo } from "../types";

const props = defineProps<{ config: BaskulConfig; info: DaemonInfo }>();
const edges = computed(() => props.config.screen.edge_switching);
const missingHotkey = computed(
  () => !props.config.hotkeys.switch_to_secondary || !props.config.hotkeys.return_to_primary,
);
</script>

<template>
  <div>
    <linux-notice />
    <setting-row :label="$t('settings.switching.goToSecondary')" :help="$t('settings.switching.goToSecondaryHelp')">
      <hotkey-input v-model="config.hotkeys.switch_to_secondary" :os="info.os" />
    </setting-row>
    <setting-row :label="$t('settings.switching.returnHere')" :help="$t('settings.switching.returnHereHelp')">
      <hotkey-input v-model="config.hotkeys.return_to_primary" :os="info.os" />
    </setting-row>
    <n-alert v-if="!edges && missingHotkey" type="warning" style="margin: 4px 0 8px">
      {{ $t("settings.switching.hotkeysNeeded") }}
    </n-alert>
    <setting-row :label="$t('settings.switching.edges')" :help="$t('settings.switching.edgesHelp')">
      <n-switch v-model:value="config.screen.edge_switching" />
    </setting-row>
    <setting-row
      :label="$t('settings.switching.position')"
      :help="$t('settings.switching.positionHelp')"
      :disabled="!edges"
    >
      <position-picker v-model="config.screen.secondary_position" :disabled="!edges" />
    </setting-row>
    <setting-row
      :label="$t('settings.switching.corners')"
      :help="$t('settings.switching.cornersHelp')"
      :disabled="!edges"
    >
      <number-input v-model="config.screen.corner_size" :min="0" :max="500" :precision="0" :disabled="!edges" suffix="px" />
    </setting-row>
    <setting-row :label="$t('settings.switching.lock')" :help="$t('settings.switching.lockHelp')">
      <hotkey-input v-model="config.hotkeys.toggle_lock" :os="info.os" />
    </setting-row>
  </div>
</template>
