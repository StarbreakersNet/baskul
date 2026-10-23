<script setup lang="ts">
// Connexion : la machine principale attend les pilotées ; chaque pilotée la
// trouve sur le réseau local (découverte automatique) ou se connecte à son
// adresse.
import { computed, onActivated, onDeactivated, onMounted, onUnmounted, ref } from "vue";
import { NButton, NFlex, NInput, NPopconfirm, NSelect, NSwitch, NTag, NText, useMessage } from "naive-ui";
import { useI18n } from "vue-i18n";
import SettingRow from "../components/SettingRow.vue";
import { daemon } from "../daemon";
import type { BaskulConfig, DaemonInfo, FoundPrimary, NetworkInfo } from "../types";
import { copyText, newSecret, osLabel } from "../utils";

const props = defineProps<{ config: BaskulConfig; info: DaemonInfo }>();
const message = useMessage();
const { t } = useI18n();
const secretTooShort = computed(() => props.config.network.secret.trim().length < 12);
const primary = computed(() => props.config.role === "primary");

// Cartes réseau et machines principales trouvées, actualisées tant que la
// page est affichée.
const network = ref<NetworkInfo | null>(null);
let timer: ReturnType<typeof setInterval> | undefined;
async function refresh() {
  network.value = await daemon.network().catch(() => network.value);
}
function watchNetwork() {
  clearInterval(timer);
  void refresh();
  timer = setInterval(refresh, 3000);
}
onMounted(watchNetwork);
onActivated(watchNetwork);
onDeactivated(() => clearInterval(timer));
onUnmounted(() => clearInterval(timer));

const interfaces = computed(() => {
  const list = network.value?.interfaces ?? [];
  const options = [
    { value: "", label: t("settings.connection.interfaceAuto") },
    ...list.map((i) => ({ value: i.name, label: `${i.label} — ${i.addresses.join(", ")}` })),
  ];
  const chosen = props.config.network.interface;
  if (chosen && !list.some((i) => i.name === chosen)) {
    options.push({ value: chosen, label: t("settings.connection.interfaceMissing", { name: chosen }) });
  }
  return options;
});

/** Adresse de cette machine : celle de la carte choisie, sinon l'adresse
 * principale. */
const address = computed(() => {
  const chosen = network.value?.interfaces.find((i) => i.name === props.config.network.interface);
  return chosen?.addresses[0] ?? props.info.address;
});

const primaries = computed(() => network.value?.primaries ?? []);
/** Adresse à saisir : sans le port quand c'est celui par défaut. */
const hostOf = (p: FoundPrimary) => {
  const a = p.addresses[0] ?? "";
  const port = network.value?.defaultPort;
  return port && a.endsWith(`:${port}`) ? a.slice(0, -`:${port}`.length).replace(/^\[(.*)\]$/, "$1") : a;
};
const usable = (p: FoundPrimary) => p.protocol === network.value?.protocol;

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

    <template v-if="primary">
      <setting-row v-if="address" :label="$t('settings.connection.address')" :help="$t('settings.connection.addressHelp')">
        <n-text code strong>{{ address }}</n-text>
        <n-button size="small" @click="copy(address!)">{{ $t("common.copy") }}</n-button>
      </setting-row>
      <setting-row :label="$t('settings.connection.announce')" :help="$t('settings.connection.announceHelp')">
        <n-switch v-model:value="config.network.discovery" />
      </setting-row>
    </template>

    <template v-else>
      <setting-row :label="$t('settings.connection.discovery')" :help="$t('settings.connection.discoveryHelp')">
        <n-switch v-model:value="config.network.discovery" />
      </setting-row>
      <setting-row
        v-if="config.network.discovery"
        :label="$t('settings.connection.found')"
        :help="$t('settings.connection.foundHelp')"
      >
        <n-flex vertical :size="6" style="width: 100%">
          <n-text v-if="!primaries.length" depth="3">{{ $t("settings.connection.searching") }}</n-text>
          <n-flex v-for="p in primaries" :key="p.name + p.addresses.join()" align="center" :size="8">
            <n-text strong>{{ p.name }}</n-text>
            <n-text depth="3">{{ osLabel(p.os) }} · {{ hostOf(p) }}</n-text>
            <n-tag v-if="!usable(p)" size="small" type="warning" :bordered="false">
              {{ $t("settings.connection.otherVersion") }}
            </n-tag>
            <n-button
              v-else-if="config.network.connect !== hostOf(p)"
              size="tiny"
              quaternary
              @click="config.network.connect = hostOf(p)"
            >
              {{ $t("settings.connection.pin") }}
            </n-button>
          </n-flex>
        </n-flex>
      </setting-row>
      <setting-row :label="$t('settings.connection.connect')" :help="$t('settings.connection.connectHelp')">
        <n-input
          v-model:value="config.network.connect"
          class="mono"
          :placeholder="
            config.network.discovery
              ? $t('settings.connection.connectAuto')
              : $t('settings.connection.connectPlaceholder')
          "
          :status="!config.network.discovery && !config.network.connect.trim() ? 'error' : undefined"
          :input-props="{ spellcheck: false, autocomplete: 'off' }"
        />
      </setting-row>
    </template>

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
    <setting-row :label="$t('settings.connection.interface')" :help="$t('settings.connection.interfaceHelp')">
      <n-select v-model:value="config.network.interface" :options="interfaces" style="min-width: 260px" />
    </setting-row>
    <setting-row
      v-if="primary"
      :label="$t('settings.connection.listen')"
      :help="$t('settings.connection.listenHelp')"
    >
      <n-input v-model:value="config.network.listen" class="mono" :input-props="{ spellcheck: false }" />
    </setting-row>
    <setting-row :label="$t('settings.connection.activity')" :help="$t('settings.connection.activityHelp')">
      <n-switch v-model:value="config.activity.mirror" />
    </setting-row>
  </div>
</template>
