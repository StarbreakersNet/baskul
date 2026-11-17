<script setup lang="ts">
import { computed, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import {
  IconAdjustments,
  IconArrowsRightLeft,
  IconClipboard,
  IconKeyboard,
  IconMouse,
  IconPalette,
  IconPlugConnected,
} from "@tabler/icons-vue";
import HAlert from "@/components/c/hAlert.vue";
import HButton from "@/components/c/hButton.vue";
import HCard from "@/components/c/hCard.vue";
import HFlex from "@/components/c/hFlex.vue";
import HIcon from "@/components/c/hIcon.vue";
import HSpin from "@/components/c/hSpin.vue";
import HTabPane from "@/components/c/hTabPane.vue";
import HTabs from "@/components/c/hTabs.vue";
import AdvancedSection from "@/components/settings/AdvancedSection.vue";
import ClipboardSection from "@/components/settings/ClipboardSection.vue";
import ConnectionSection from "@/components/settings/ConnectionSection.vue";
import InterfaceSection from "@/components/settings/InterfaceSection.vue";
import KeyboardSection from "@/components/settings/KeyboardSection.vue";
import MachineSection from "@/components/settings/MachineSection.vue";
import MouseSection from "@/components/settings/MouseSection.vue";
import SwitchingSection from "@/components/settings/SwitchingSection.vue";
import { useSettingsStore } from "@/stores/settings";
import { useStatusStore } from "@/stores/status";

const DEFAULT_TAB = "connexion";
// Onglets propres à un rôle : quitter le rôle ramène à l'onglet par défaut
const ROLE_TABS = { primary: "baskul", secondary: "souris" };

const settings = useSettingsStore();
const statusStore = useStatusStore();
const route = useRoute();
const router = useRouter();

const role = computed(() => settings.config?.role ?? "primary");
const isRestarting = computed(() => statusStore.phase === "restarting");
const peers = computed(() => statusStore.status?.peers ?? []);
const hasConnectionError = computed(() => statusStore.status?.state === "error");

// Onglet gardé dans l'adresse : /reglages?onglet=clavier
const tab = computed({
  get: () => (typeof route.query.onglet === "string" ? route.query.onglet : DEFAULT_TAB),
  set: (onglet: string) => void router.replace({ query: { onglet } }),
});

watch(role, value => {
  const otherRoleTab = value === "primary" ? ROLE_TABS.secondary : ROLE_TABS.primary;
  if (tab.value === otherRoleTab) {
    tab.value = DEFAULT_TAB;
  }
});
</script>

<template>
  <h-spin v-if="settings.info" :description="$t('settings.restarting')" :show="isRestarting">
    <h-flex :size="16" vertical>
      <h-alert v-if="settings.welcome" closable :title="$t('settings.welcomeTitle')" type="info">
        {{ $t("settings.welcomeIntro") }}
        <ol class="settings-steps">
          <li>{{ $t("settings.welcomeStep1") }}</li>
          <li>{{ $t("settings.welcomeStep2") }}</li>
          <li>{{ $t("settings.welcomeStep3") }}</li>
        </ol>
        {{ $t("settings.welcomeEnd") }}
      </h-alert>

      <h-alert v-if="!settings.config" :title="$t('settings.configUnreadable')" type="error">
        <p class="mono">{{ settings.info.error }}</p>
        <h-flex :size="8">
          <h-button size="small" @click="settings.openConfigFile()">
            {{ $t("common.openFile") }}
          </h-button>
          <h-button size="small" @click="statusStore.restart()">
            {{ $t("common.restart") }}
          </h-button>
        </h-flex>
      </h-alert>
      <h-alert v-else-if="hasConnectionError" type="error">
        {{ statusStore.status?.detail }}
      </h-alert>

      <template v-if="settings.config">
        <machine-section v-model:config="settings.config" :info="settings.info" />
        <h-card size="small">
          <h-tabs v-model:value="tab" animated type="segment">
            <h-tab-pane name="connexion">
              <template #tab>
                <h-flex align="center" :size="6" :wrap="false">
                  <h-icon :component="IconPlugConnected" :size="16" />
                  {{ $t("settings.tabs.connection") }}
                </h-flex>
              </template>
              <connection-section v-model:config="settings.config" :info="settings.info" />
            </h-tab-pane>
            <h-tab-pane v-if="role === 'primary'" name="baskul">
              <template #tab>
                <h-flex align="center" :size="6" :wrap="false">
                  <h-icon :component="IconArrowsRightLeft" :size="16" />
                  {{ $t("settings.tabs.switching") }}
                </h-flex>
              </template>
              <switching-section
                v-model:config="settings.config"
                :info="settings.info"
                :peers="peers" />
            </h-tab-pane>
            <h-tab-pane v-if="role === 'secondary'" name="souris">
              <template #tab>
                <h-flex align="center" :size="6" :wrap="false">
                  <h-icon :component="IconMouse" :size="16" />
                  {{ $t("settings.tabs.mouse") }}
                </h-flex>
              </template>
              <mouse-section v-model:config="settings.config" />
            </h-tab-pane>
            <h-tab-pane name="clavier">
              <template #tab>
                <h-flex align="center" :size="6" :wrap="false">
                  <h-icon :component="IconKeyboard" :size="16" />
                  {{ $t("settings.tabs.keyboard") }}
                </h-flex>
              </template>
              <keyboard-section v-model:config="settings.config" />
            </h-tab-pane>
            <h-tab-pane name="presse-papiers">
              <template #tab>
                <h-flex align="center" :size="6" :wrap="false">
                  <h-icon :component="IconClipboard" :size="16" />
                  {{ $t("settings.tabs.clipboard") }}
                </h-flex>
              </template>
              <clipboard-section v-model:config="settings.config" />
            </h-tab-pane>
            <h-tab-pane name="interface">
              <template #tab>
                <h-flex align="center" :size="6" :wrap="false">
                  <h-icon :component="IconPalette" :size="16" />
                  {{ $t("settings.tabs.interface") }}
                </h-flex>
              </template>
              <interface-section v-model:config="settings.config" />
            </h-tab-pane>
            <h-tab-pane name="avance">
              <template #tab>
                <h-flex align="center" :size="6" :wrap="false">
                  <h-icon :component="IconAdjustments" :size="16" />
                  {{ $t("settings.tabs.advanced") }}
                </h-flex>
              </template>
              <advanced-section v-model:config="settings.config" :info="settings.info" />
            </h-tab-pane>
          </h-tabs>
        </h-card>
      </template>
    </h-flex>
  </h-spin>
</template>

<style lang="scss" scoped>
.settings-steps {
  margin: 6px 0;
  padding-left: 20px;
}
</style>
