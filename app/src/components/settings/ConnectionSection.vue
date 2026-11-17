<script setup lang="ts">
import {
  IconActivityHeartbeat,
  IconBroadcast,
  IconDeviceLaptop,
  IconKey,
  IconNetwork,
  IconPlugConnected,
  IconRadar,
  IconRouter,
  IconServer,
} from "@tabler/icons-vue";
import { computed, onActivated, onBeforeUnmount, onDeactivated, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import HButton from "@/components/c/hButton.vue";
import HFlex from "@/components/c/hFlex.vue";
import HInput from "@/components/c/hInput.vue";
import HPopconfirm from "@/components/c/hPopconfirm.vue";
import HSelect from "@/components/c/hSelect.vue";
import HSwitch from "@/components/c/hSwitch.vue";
import HTag from "@/components/c/hTag.vue";
import HText from "@/components/c/hText.vue";
import SettingRow from "@/components/SettingRow.vue";
import ExpandTransition from "@/components/ExpandTransition.vue";
import { useCopy } from "@/composables/useCopy";
import { useNetworkStore } from "@/stores/network";
import { hostWithoutDefaultPort } from "@/utils/address";
import { osLabel } from "@/utils/os";
import { newSecret } from "@/utils/secret";
import type { BaskulConfig } from "@/types/Config";
import type { DaemonInfo } from "@/types/Daemon";
import type { FoundPrimary } from "@/types/Network";

const config = defineModel<BaskulConfig>("config", { required: true });

const { info } = defineProps<{
  info: DaemonInfo;
}>();

const REFRESH_MS = 3000;
const MIN_SECRET_LENGTH = 12;

const networkStore = useNetworkStore();
const { copy } = useCopy();
const { t } = useI18n();

let refreshTimer: ReturnType<typeof setInterval> | undefined;

const isPrimary = computed(() => config.value.role === "primary");
const isSecretTooShort = computed(
  () => config.value.network.secret.trim().length < MIN_SECRET_LENGTH
);
const isAddressMissing = computed(
  () => !config.value.network.discovery && !config.value.network.connect.trim()
);
const primaries = computed(() => networkStore.network?.primaries ?? []);

const interfaceOptions = computed(() => {
  const interfaces = networkStore.network?.interfaces ?? [];
  const chosen = config.value.network.interface;
  const options = [
    { value: "", label: t("settings.connection.interfaceAuto") },
    ...interfaces.map(i => ({ value: i.name, label: `${i.label} — ${i.addresses.join(", ")}` })),
  ];
  if (chosen && !interfaces.some(i => i.name === chosen)) {
    options.push({
      value: chosen,
      label: t("settings.connection.interfaceMissing", { name: chosen }),
    });
  }
  return options;
});

/** Celle de la carte choisie, sinon l'adresse principale */
const address = computed(() => {
  const chosen = networkStore.network?.interfaces.find(
    i => i.name === config.value.network.interface
  );
  return chosen?.addresses[0] ?? info.address ?? "";
});

function hostOf(primary: FoundPrimary) {
  return hostWithoutDefaultPort(primary.addresses[0] ?? "", networkStore.network?.defaultPort);
}

function isCompatible(primary: FoundPrimary) {
  return primary.protocol === networkStore.network?.protocol;
}

function pin(primary: FoundPrimary) {
  config.value.network.connect = hostOf(primary);
}

function generateSecret() {
  config.value.network.secret = newSecret();
}

function startRefreshing() {
  clearInterval(refreshTimer);
  void networkStore.refresh();
  refreshTimer = setInterval(networkStore.refresh, REFRESH_MS);
}

function stopRefreshing() {
  clearInterval(refreshTimer);
}

onMounted(startRefreshing);
onActivated(startRefreshing);
onDeactivated(stopRefreshing);
onBeforeUnmount(stopRefreshing);
</script>

<template>
  <div>
    <h-text depth="3" tag="p">{{ $t("settings.connection.intro") }}</h-text>

    <setting-row
      :icon="IconServer"
      :help="$t('settings.connection.addressHelp')"
      :label="$t('settings.connection.address')"
      :show="isPrimary && !!address">
      <h-text code strong>{{ address }}</h-text>
      <h-button size="small" @click="copy(address)">{{ $t("common.copy") }}</h-button>
    </setting-row>
    <setting-row
      :icon="IconBroadcast"
      :help="$t('settings.connection.announceHelp')"
      :label="$t('settings.connection.announce')"
      :show="isPrimary">
      <h-switch v-model:value="config.network.discovery" />
    </setting-row>

    <setting-row
      :icon="IconRadar"
      :help="$t('settings.connection.discoveryHelp')"
      :label="$t('settings.connection.discovery')"
      :show="!isPrimary">
      <h-switch v-model:value="config.network.discovery" />
    </setting-row>
    <setting-row
      :icon="IconDeviceLaptop"
      :help="$t('settings.connection.foundHelp')"
      :label="$t('settings.connection.found')"
      :show="!isPrimary && config.network.discovery">
      <div class="connection-found">
        <expand-transition>
          <h-text v-if="!primaries.length" depth="3" tag="div">
            {{ $t("settings.connection.searching") }}
          </h-text>
        </expand-transition>
        <expand-transition group>
          <div
            v-for="primary in primaries"
            :key="primary.name + primary.addresses.join()"
            class="connection-primary">
            <h-flex align="center" :size="8">
              <h-text strong>{{ primary.name }}</h-text>
              <h-text depth="3">{{ osLabel(primary.os) }} · {{ hostOf(primary) }}</h-text>
              <transition mode="out-in" name="fade">
                <h-tag v-if="!isCompatible(primary)" :bordered="false" size="small" type="warning">
                  {{ $t("settings.connection.otherVersion") }}
                </h-tag>
                <h-button
                  v-else-if="config.network.connect !== hostOf(primary)"
                  quaternary
                  size="tiny"
                  @click="pin(primary)">
                  {{ $t("settings.connection.pin") }}
                </h-button>
              </transition>
            </h-flex>
          </div>
        </expand-transition>
      </div>
    </setting-row>
    <setting-row
      :icon="IconPlugConnected"
      :help="$t('settings.connection.connectHelp')"
      :label="$t('settings.connection.connect')"
      :show="!isPrimary">
      <h-input
        v-model:value="config.network.connect"
        class="mono"
        :input-props="{ spellcheck: false, autocomplete: 'off' }"
        :placeholder="
          config.network.discovery
            ? $t('settings.connection.connectAuto')
            : $t('settings.connection.connectPlaceholder')
        "
        :status="isAddressMissing ? 'error' : undefined" />
    </setting-row>

    <setting-row
      :icon="IconKey"
      :help="$t('settings.connection.secretHelp')"
      :label="$t('settings.connection.secret')">
      <h-input
        v-model:value="config.network.secret"
        class="mono"
        :input-props="{ spellcheck: false, autocomplete: 'off' }"
        :status="isSecretTooShort ? 'error' : undefined" />
      <h-button size="small" @click="copy(config.network.secret)">{{ $t("common.copy") }}</h-button>
      <h-popconfirm
        :negative-text="$t('common.cancel')"
        :positive-text="$t('settings.connection.generate')"
        @positive-click="generateSecret()">
        <template #trigger>
          <h-button quaternary size="small">{{ $t("settings.connection.newSecret") }}</h-button>
        </template>
        {{ $t("settings.connection.newSecretWarning") }}
      </h-popconfirm>
    </setting-row>
    <setting-row
      :icon="IconNetwork"
      :help="$t('settings.connection.interfaceHelp')"
      :label="$t('settings.connection.interface')">
      <h-select
        v-model:value="config.network.interface"
        class="connection-select"
        :options="interfaceOptions" />
    </setting-row>
    <setting-row
      :icon="IconRouter"
      :help="$t('settings.connection.listenHelp')"
      :label="$t('settings.connection.listen')"
      :show="isPrimary">
      <h-input
        v-model:value="config.network.listen"
        class="mono"
        :input-props="{ spellcheck: false }" />
    </setting-row>
    <setting-row
      :icon="IconActivityHeartbeat"
      :help="$t('settings.connection.activityHelp')"
      :label="$t('settings.connection.activity')">
      <h-switch v-model:value="config.activity.mirror" />
    </setting-row>
  </div>
</template>

<style lang="scss" scoped>
.connection-found {
  width: 100%;
}

.connection-primary {
  padding: 3px 0;
}

.connection-select {
  min-width: 260px;
}
</style>
