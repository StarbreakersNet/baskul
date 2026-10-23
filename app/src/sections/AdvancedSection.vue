<script setup lang="ts">
import { computed } from "vue";
import { NButton, NFlex, NSelect, NText, useMessage } from "naive-ui";
import { useI18n } from "vue-i18n";
import ChoiceGroup from "../components/ChoiceGroup.vue";
import SettingRow from "../components/SettingRow.vue";
import type { BaskulConfig, DaemonInfo } from "../types";
import { copyText } from "../utils";

const props = defineProps<{ config: BaskulConfig; info: DaemonInfo; restarting?: boolean }>();
const emit = defineEmits<{ openFile: []; restart: [] }>();

const { t } = useI18n();
const message = useMessage();
const levels = computed(() =>
  ["error", "warn", "info", "debug", "trace"].map((value) => ({ value, label: t(`settings.advanced.levels.${value}`) })),
);

/** Commandes pour d'autres applications ; « Copier » donne la commande
 * complète, avec le chemin de Baskul sur cette machine. */
const commands = computed(() => {
  const exe = props.info.exe ?? "baskul";
  const quoted = !/[\s'"()&;]/.test(exe)
    ? exe
    : props.info.os === "windows"
      ? `"${exe}"`
      : `'${exe.replace(/'/g, "'\\''")}'`;
  return ["switch here", "switch other", "switch toggle", "status"].map((c) => ({
    shown: `baskul ${c}`,
    full: `${quoted} ${c}`,
  }));
});

async function copy(text: string) {
  try {
    await copyText(text);
    message.success(t("common.copied"));
  } catch {
    message.error(t("common.copyFailed"));
  }
}
</script>

<template>
  <div>
    <setting-row
      :label="$t('settings.advanced.isoSwap')"
      :help="$t('settings.advanced.isoSwapHelp')"
      machine="secondary"
      os="macos"
    >
      <choice-group
        v-model="config.keyboard.iso_swap"
        :options="[
          { value: 'auto', label: $t('settings.advanced.auto') },
          { value: 'on', label: $t('settings.advanced.swap') },
          { value: 'off', label: $t('settings.advanced.noSwap') },
        ]"
      />
    </setting-row>
    <setting-row :label="$t('settings.advanced.commands')">
      <template #help>
        {{ $t("settings.advanced.commandsHelp") }}
        <span v-if="info.exe" class="mono path">{{ info.exe }}</span>
      </template>
      <n-flex vertical :size="6">
        <n-flex v-for="c in commands" :key="c.full" align="center" :size="8" :wrap="false">
          <n-text code class="mono command">{{ c.shown }}</n-text>
          <n-button size="tiny" quaternary :title="c.full" @click="copy(c.full)">{{ $t("common.copy") }}</n-button>
        </n-flex>
      </n-flex>
    </setting-row>
    <setting-row :label="$t('settings.advanced.logLevel')">
      <n-select v-model:value="config.log.level" :options="levels" style="width: 220px" />
    </setting-row>
    <setting-row :label="$t('settings.advanced.configFile')">
      <template #help><span class="mono" style="word-break: break-all">{{ info.path }}</span></template>
      <n-button size="small" @click="emit('openFile')">{{ $t("common.openFile") }}</n-button>
    </setting-row>
    <setting-row label="Baskul" :help="$t('common.version', { version: info.version })">
      <n-button size="small" :loading="restarting" @click="emit('restart')">{{ $t("common.restart") }}</n-button>
    </setting-row>
  </div>
</template>

<style scoped>
.command {
  min-width: 12em;
}
.path {
  display: block;
  margin-top: 4px;
  word-break: break-all;
}
</style>
