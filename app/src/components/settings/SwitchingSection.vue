<script setup lang="ts">
import { computed } from "vue";
import HAlert from "@/components/c/hAlert.vue";
import HSwitch from "@/components/c/hSwitch.vue";
import HText from "@/components/c/hText.vue";
import HotkeyInput from "@/components/HotkeyInput.vue";
import LayoutEditor from "@/components/LayoutEditor.vue";
import LinuxNotice from "@/components/LinuxNotice.vue";
import NumberInput from "@/components/NumberInput.vue";
import PositionPicker from "@/components/PositionPicker.vue";
import SettingRow from "@/components/SettingRow.vue";
import type { BaskulConfig } from "@/types/Config";
import type { DaemonInfo, Peer } from "@/types/Daemon";

const config = defineModel<BaskulConfig>("config", { required: true });

const { info, peers } = defineProps<{
  info: DaemonInfo;
  peers: Peer[];
}>();

const hasEdgeSwitching = computed(() => config.value.screen.edge_switching);
// Sans bascule par les bords, seuls les raccourcis mènent d'une machine à l'autre
const isMissingHotkey = computed(
  () => !config.value.hotkeys.switch_to_secondary || !config.value.hotkeys.return_to_primary
);
</script>

<template>
  <div>
    <linux-notice />
    <div class="switching-layout">
      <h-text strong>{{ $t("settings.layout.title") }}</h-text>
      <h-text class="help-text" depth="3" tag="div">{{ $t("settings.layout.help") }}</h-text>
      <layout-editor
        v-model="config.layout.machines"
        class="switching-editor"
        :os="info.os"
        :peers="peers"
        :primary-name="config.name || info.hostname" />
    </div>
    <setting-row
      :help="$t('settings.switching.goToSecondaryHelp')"
      :label="$t('settings.switching.goToSecondary')">
      <hotkey-input v-model="config.hotkeys.switch_to_secondary" :os="info.os" />
    </setting-row>
    <setting-row
      :help="$t('settings.switching.returnHereHelp')"
      :label="$t('settings.switching.returnHere')">
      <hotkey-input v-model="config.hotkeys.return_to_primary" :os="info.os" />
    </setting-row>
    <setting-row :help="$t('settings.switching.nextHelp')" :label="$t('settings.switching.next')">
      <hotkey-input v-model="config.hotkeys.next_machine" :os="info.os" />
    </setting-row>
    <h-alert v-if="!hasEdgeSwitching && isMissingHotkey" class="switching-alert" type="warning">
      {{ $t("settings.switching.hotkeysNeeded") }}
    </h-alert>
    <setting-row :help="$t('settings.switching.edgesHelp')" :label="$t('settings.switching.edges')">
      <h-switch v-model:value="config.screen.edge_switching" />
    </setting-row>
    <setting-row
      :help="$t('settings.switching.positionHelp')"
      :label="$t('settings.switching.position')">
      <position-picker v-model="config.screen.secondary_position" />
    </setting-row>
    <setting-row
      :disabled="!hasEdgeSwitching"
      :help="$t('settings.switching.cornersHelp')"
      :label="$t('settings.switching.corners')">
      <number-input
        v-model="config.screen.corner_size"
        :disabled="!hasEdgeSwitching"
        :max="500"
        :min="0"
        :precision="0"
        suffix="px" />
    </setting-row>
    <setting-row :help="$t('settings.switching.lockHelp')" :label="$t('settings.switching.lock')">
      <hotkey-input v-model="config.hotkeys.toggle_lock" :os="info.os" />
    </setting-row>
  </div>
</template>

<style lang="scss" scoped>
.switching-layout {
  padding: 12px 0 16px;
}

.switching-editor {
  margin-top: 12px;
}

.switching-alert {
  margin: 4px 0 8px;
}
</style>
