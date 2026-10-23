<script setup lang="ts">
// Réglages de Bascule : rôle de la machine, puis onglets par thème. Les
// onglets qui ne concernent pas le rôle choisi sont masqués.
import { computed, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { NAlert, NButton, NCard, NFlex, NSpin, NTabPane, NTabs } from "naive-ui";
import { useActions } from "../composables/useActions";
import { useSettings } from "../composables/useSettings";
import AdvancedSection from "../sections/AdvancedSection.vue";
import ClipboardSection from "../sections/ClipboardSection.vue";
import ConnectionSection from "../sections/ConnectionSection.vue";
import InterfaceSection from "../sections/InterfaceSection.vue";
import KeyboardSection from "../sections/KeyboardSection.vue";
import MachineSection from "../sections/MachineSection.vue";
import MouseSection from "../sections/MouseSection.vue";
import SwitchingSection from "../sections/SwitchingSection.vue";

const { phase, info, config, status, welcome } = useSettings();
const actions = useActions();
const route = useRoute();
const router = useRouter();

const role = computed(() => config.value?.role ?? "primary");
const restarting = computed(() => phase.value === "restarting");

// Onglet affiché, gardé dans l'adresse (`/reglages?onglet=clavier`).
const tab = computed({
  get: () => (typeof route.query.onglet === "string" ? route.query.onglet : "connexion"),
  set: (onglet: string) => void router.replace({ query: { onglet } }),
});
watch(role, (r) => {
  if ((r === "primary" && tab.value === "souris") || (r === "secondary" && tab.value === "bascule")) {
    tab.value = "connexion";
  }
});
</script>

<template>
  <n-spin v-if="info" :show="restarting" :description="$t('settings.restarting')">
    <n-flex vertical :size="16">
      <n-alert v-if="welcome" type="info" :title="$t('settings.welcomeTitle')" closable>
        {{ $t("settings.welcomeIntro") }}
        <ol style="margin: 6px 0; padding-left: 20px">
          <li>{{ $t("settings.welcomeStep1") }}</li>
          <li>{{ $t("settings.welcomeStep2") }}</li>
          <li>{{ $t("settings.welcomeStep3") }}</li>
        </ol>
        {{ $t("settings.welcomeEnd") }}
      </n-alert>

      <n-alert v-if="!config" type="error" :title="$t('settings.configUnreadable')">
        <p class="mono">{{ info.error }}</p>
        <n-flex :size="8">
          <n-button size="small" @click="actions.openConfigFile">{{ $t("common.openFile") }}</n-button>
          <n-button size="small" @click="actions.restart">{{ $t("common.restart") }}</n-button>
        </n-flex>
      </n-alert>
      <n-alert v-else-if="status?.state === 'error'" type="error">{{ status.detail }}</n-alert>

      <template v-if="config">
        <machine-section :config="config" :info="info" />
        <n-card size="small">
          <n-tabs v-model:value="tab" type="segment" animated>
            <n-tab-pane name="connexion" :tab="$t('settings.tabs.connection')">
              <connection-section :config="config" :info="info" />
            </n-tab-pane>
            <n-tab-pane v-if="role === 'primary'" name="bascule" :tab="$t('settings.tabs.switching')">
              <switching-section :config="config" :info="info" />
            </n-tab-pane>
            <n-tab-pane v-if="role === 'secondary'" name="souris" :tab="$t('settings.tabs.mouse')">
              <mouse-section :config="config" />
            </n-tab-pane>
            <n-tab-pane name="clavier" :tab="$t('settings.tabs.keyboard')">
              <keyboard-section :config="config" @open-file="actions.openConfigFile" />
            </n-tab-pane>
            <n-tab-pane name="presse-papiers" :tab="$t('settings.tabs.clipboard')">
              <clipboard-section :config="config" />
            </n-tab-pane>
            <n-tab-pane name="interface" :tab="$t('settings.tabs.interface')">
              <interface-section :config="config" />
            </n-tab-pane>
            <n-tab-pane name="avance" :tab="$t('settings.tabs.advanced')">
              <advanced-section
                :config="config"
                :info="info"
                :restarting="restarting"
                @open-file="actions.openConfigFile"
                @restart="actions.restart"
              />
            </n-tab-pane>
          </n-tabs>
        </n-card>
      </template>
    </n-flex>
  </n-spin>
</template>
