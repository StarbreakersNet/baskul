<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import HButton from "@/components/c/hButton.vue";
import HFlex from "@/components/c/hFlex.vue";
import HSelect from "@/components/c/hSelect.vue";
import HSwitch from "@/components/c/hSwitch.vue";
import HText from "@/components/c/hText.vue";
import ChoiceGroup from "@/components/ChoiceGroup.vue";
import SettingRow from "@/components/SettingRow.vue";
import { useCopy } from "@/composables/useCopy";
import { useSettingsStore } from "@/stores/settings";
import { useStatusStore } from "@/stores/status";
import { shellCommand } from "@/utils/address";
import type { BaskulConfig } from "@/types/Config";
import type { ChoiceOption } from "@/types/Choice";
import type { DaemonInfo } from "@/types/Daemon";

const config = defineModel<BaskulConfig>("config", { required: true });

const { info } = defineProps<{
  info: DaemonInfo;
}>();

const LOG_LEVELS = ["error", "warn", "info", "debug", "trace"];
const COMMANDS = ["switch here", "switch primary", "switch next", "switch toggle", "status"];

const settings = useSettingsStore();
const statusStore = useStatusStore();
const { copy } = useCopy();
const { t } = useI18n();

const isRestarting = computed(() => statusStore.phase === "restarting");
const isoSwapOptions = computed<ChoiceOption<BaskulConfig["keyboard"]["iso_swap"]>[]>(() => [
  { value: "auto", label: t("settings.advanced.auto") },
  { value: "on", label: t("settings.advanced.swap") },
  { value: "off", label: t("settings.advanced.noSwap") },
]);
const levelOptions = computed(() =>
  LOG_LEVELS.map(value => ({ value, label: t(`settings.advanced.levels.${value}`) }))
);
// "Copier" donne la commande complète, avec le chemin de Baskul sur cette machine
const commands = computed(() =>
  COMMANDS.map(args => ({
    shown: `baskul ${args}`,
    full: shellCommand(info.exe ?? "baskul", args, info.os),
  }))
);
</script>

<template>
  <div>
    <setting-row
      :help="$t('settings.advanced.isoSwapHelp')"
      :label="$t('settings.advanced.isoSwap')"
      machine="secondary"
      os="macos">
      <choice-group v-model="config.keyboard.iso_swap" :options="isoSwapOptions" />
    </setting-row>
    <setting-row :label="$t('settings.advanced.commands')">
      <template #help>
        {{ $t("settings.advanced.commandsHelp") }}
        <span v-if="info.exe" class="mono break-all advanced-path">{{ info.exe }}</span>
      </template>
      <h-flex :size="6" vertical>
        <h-flex
          v-for="command in commands"
          :key="command.full"
          align="center"
          :size="8"
          :wrap="false">
          <h-text class="mono advanced-command" code>{{ command.shown }}</h-text>
          <h-button quaternary size="tiny" :title="command.full" @click="copy(command.full)">
            {{ $t("common.copy") }}
          </h-button>
        </h-flex>
      </h-flex>
    </setting-row>
    <setting-row
      :help="$t('settings.advanced.updatesHelp')"
      :label="$t('settings.advanced.updates')">
      <h-switch v-model:value="config.updates.check" />
    </setting-row>
    <setting-row :label="$t('settings.advanced.logLevel')">
      <h-select v-model:value="config.log.level" class="advanced-select" :options="levelOptions" />
    </setting-row>
    <setting-row :label="$t('settings.advanced.configFile')">
      <template #help>
        <span class="mono break-all">{{ info.path }}</span>
      </template>
      <h-button size="small" @click="settings.openConfigFile()">
        {{ $t("common.openFile") }}
      </h-button>
    </setting-row>
    <setting-row :help="$t('common.version', { version: info.version })" label="Baskul">
      <h-button :loading="isRestarting" size="small" @click="statusStore.restart()">
        {{ $t("common.restart") }}
      </h-button>
    </setting-row>
  </div>
</template>

<style lang="scss" scoped>
.advanced-command {
  min-width: 12em;
}

.advanced-path {
  display: block;
  margin-top: 4px;
}

.advanced-select {
  width: 220px;
}
</style>
