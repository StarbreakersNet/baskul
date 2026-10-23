<script setup lang="ts">
import { computed } from "vue";
import { NAlert, NSwitch, NText } from "naive-ui";
import HotkeyInput from "../components/HotkeyInput.vue";
import LayoutEditor from "../components/LayoutEditor.vue";
import NumberInput from "../components/NumberInput.vue";
import PositionPicker from "../components/PositionPicker.vue";
import SettingRow from "../components/SettingRow.vue";
import LinuxNotice from "../components/LinuxNotice.vue";
import type { BaskulConfig, DaemonInfo, Peer } from "../types";

const props = defineProps<{ config: BaskulConfig; info: DaemonInfo; peers: Peer[] }>();
const edges = computed(() => props.config.screen.edge_switching);
const missingHotkey = computed(
  () => !props.config.hotkeys.switch_to_secondary || !props.config.hotkeys.return_to_primary,
);
</script>

<template>
  <div>
    <linux-notice />
    <div class="block">
      <n-text strong>{{ $t("settings.layout.title") }}</n-text>
      <n-text depth="3" tag="div" class="help">{{ $t("settings.layout.help") }}</n-text>
      <layout-editor
        v-model="config.layout.machines"
        :primary-name="config.name || info.hostname"
        :os="info.os"
        :peers="peers"
        style="margin-top: 12px"
      />
    </div>
    <setting-row :label="$t('settings.switching.goToSecondary')" :help="$t('settings.switching.goToSecondaryHelp')">
      <hotkey-input v-model="config.hotkeys.switch_to_secondary" :os="info.os" />
    </setting-row>
    <setting-row :label="$t('settings.switching.returnHere')" :help="$t('settings.switching.returnHereHelp')">
      <hotkey-input v-model="config.hotkeys.return_to_primary" :os="info.os" />
    </setting-row>
    <setting-row :label="$t('settings.switching.next')" :help="$t('settings.switching.nextHelp')">
      <hotkey-input v-model="config.hotkeys.next_machine" :os="info.os" />
    </setting-row>
    <n-alert v-if="!edges && missingHotkey" type="warning" style="margin: 4px 0 8px">
      {{ $t("settings.switching.hotkeysNeeded") }}
    </n-alert>
    <setting-row :label="$t('settings.switching.edges')" :help="$t('settings.switching.edgesHelp')">
      <n-switch v-model:value="config.screen.edge_switching" />
    </setting-row>
    <setting-row :label="$t('settings.switching.position')" :help="$t('settings.switching.positionHelp')">
      <position-picker v-model="config.screen.secondary_position" />
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

<style scoped>
.block {
  padding: 12px 0 16px;
}
.help {
  margin-top: 2px;
  font-size: 13px;
}
</style>
