<script setup lang="ts">
import { computed } from "vue";
import { NButton, NInput, NPopconfirm, NSwitch, NText, useMessage } from "naive-ui";
import { useI18n } from "vue-i18n";
import SettingRow from "../components/SettingRow.vue";
import type { BasculeConfig, DaemonInfo } from "../types";
import { copyText, newSecret } from "../utils";

const props = defineProps<{ config: BasculeConfig; info: DaemonInfo }>();
const message = useMessage();
const { t } = useI18n();
const secretTooShort = computed(() => props.config.network.secret.trim().length < 12);

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
    <n-text depth="3" tag="p">{{ $t("settings.connection.intro") }}</n-text>
    <setting-row
      v-if="info.address"
      :label="$t('settings.connection.address')"
      :help="$t('settings.connection.addressHelp')"
    >
      <n-text code strong>{{ info.address }}</n-text>
      <n-button size="small" @click="copy(info.address!)">{{ $t("common.copy") }}</n-button>
    </setting-row>
    <setting-row :label="$t('settings.connection.connect')" :help="$t('settings.connection.connectHelp')">
      <n-input
        v-model:value="config.network.connect"
        class="mono"
        :placeholder="$t('settings.connection.connectPlaceholder')"
        :input-props="{ spellcheck: false, autocomplete: 'off' }"
      />
    </setting-row>
    <setting-row :label="$t('settings.connection.secret')" :help="$t('settings.connection.secretHelp')">
      <n-input
        v-model:value="config.network.secret"
        class="mono"
        :status="secretTooShort ? 'error' : undefined"
        :input-props="{ spellcheck: false, autocomplete: 'off' }"
      />
      <n-button size="small" @click="copy(config.network.secret)">{{ $t("common.copy") }}</n-button>
      <n-popconfirm
        :positive-text="$t('settings.connection.generate')"
        :negative-text="$t('common.cancel')"
        @positive-click="config.network.secret = newSecret()"
      >
        <template #trigger>
          <n-button size="small" quaternary>{{ $t("settings.connection.newSecret") }}</n-button>
        </template>
        {{ $t("settings.connection.newSecretWarning") }}
      </n-popconfirm>
    </setting-row>
    <setting-row :label="$t('settings.connection.listen')" :help="$t('settings.connection.listenHelp')">
      <n-input v-model:value="config.network.listen" class="mono" :input-props="{ spellcheck: false }" />
    </setting-row>
    <setting-row :label="$t('settings.connection.activity')" :help="$t('settings.connection.activityHelp')">
      <n-switch v-model:value="config.activity.mirror" />
    </setting-row>
  </div>
</template>
