<script setup lang="ts">
import { computed, h, onMounted, provide, ref, useTemplateRef, watch, type Component } from "vue";
import { RouterView, useRoute, useRouter } from "vue-router";
import { useDialog, useMessage, type MenuOption } from "naive-ui";
import { IconFileText, IconHome, IconInfoCircle, IconSettings } from "@tabler/icons-vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useI18n } from "vue-i18n";
import HAvatar from "@/components/c/hAvatar.vue";
import HBadge from "@/components/c/hBadge.vue";
import HButton from "@/components/c/hButton.vue";
import HFlex from "@/components/c/hFlex.vue";
import HIcon from "@/components/c/hIcon.vue";
import HLayout from "@/components/c/hLayout.vue";
import HLayoutContent from "@/components/c/hLayoutContent.vue";
import HLayoutFooter from "@/components/c/hLayoutFooter.vue";
import HLayoutHeader from "@/components/c/hLayoutHeader.vue";
import HLayoutSider from "@/components/c/hLayoutSider.vue";
import HMenu from "@/components/c/hMenu.vue";
import HResult from "@/components/c/hResult.vue";
import HText from "@/components/c/hText.vue";
import AppFooter from "@/components/AppFooter.vue";
import AppSplash from "@/components/AppSplash.vue";
import ProfileMenu from "@/components/ProfileMenu.vue";
import StatusBadge from "@/components/StatusBadge.vue";
import UpdateBanner from "@/components/UpdateBanner.vue";
import logo from "@/assets/logo.svg";
import { useAppShortcuts } from "@/composables/useAppShortcuts";
import { registerMessageApi } from "@/composables/useAppMessage";
import { useSettingsStore } from "@/stores/settings";
import { useStatusStore } from "@/stores/status";
import { OS_KEY, ROLE_KEY } from "@/utils/injectionKeys";
import { readLocalFlag, writeLocalFlag } from "@/utils/localSetting";
import { osLabel } from "@/utils/os";
import type { HLayoutContentInst } from "@/types/components/c/hLayoutContent";

const SIDER_COLLAPSED_KEY = "baskul.siderCollapsed";

registerMessageApi(useMessage());

const settings = useSettingsStore();
const statusStore = useStatusStore();
const route = useRoute();
const router = useRouter();
const dialog = useDialog();
const { t } = useI18n();
useAppShortcuts();

provide(
  ROLE_KEY,
  computed(() => settings.config?.role ?? "primary")
);
provide(
  OS_KEY,
  computed(() => settings.info?.os ?? "")
);

const isCollapsed = ref(readLocalFlag(SIDER_COLLAPSED_KEY, false));
const contentRef = useTemplateRef<HLayoutContentInst>("contentRef");

const menuOptions = computed<MenuOption[]>(() => [
  { key: "home", label: t("nav.home"), icon: renderIcon(IconHome) },
  { key: "settings", label: renderSettingsLabel, icon: renderIcon(IconSettings) },
  { key: "log", label: t("nav.log"), icon: renderIcon(IconFileText) },
  { key: "about", label: t("nav.about"), icon: renderIcon(IconInfoCircle) },
]);

const isLoading = computed(() => statusStore.phase === "loading");
const isRestarting = computed(() => statusStore.phase === "restarting");
const subtitle = computed(() =>
  settings.info ? `${settings.info.hostname} · ${osLabel(settings.info.os)}` : ""
);
const isWaitingForDaemon = computed(
  () => route.meta.needsDaemon && statusStore.phase !== "ready" && !isRestarting.value
);

function renderIcon(icon: Component) {
  return () => h(HIcon, null, { default: () => h(icon, { stroke: 1.75 }) });
}

// Pastille : réglages modifiés, pas encore enregistrés
function renderSettingsLabel() {
  return h(
    HBadge,
    { dot: true, show: settings.isDirty, offset: [8, 2] },
    { default: () => t("nav.settings") }
  );
}

function navigate(name: string) {
  router.push({ name });
}

function setCollapsed(value: boolean) {
  isCollapsed.value = value;
  writeLocalFlag(SIDER_COLLAPSED_KEY, value);
}

function confirmClose(close: () => void) {
  dialog.warning({
    title: t("layout.closeTitle"),
    content: t("layout.closeText"),
    positiveText: t("common.close"),
    negativeText: t("common.cancel"),
    onPositiveClick: close,
  });
}

async function guardWindowClose() {
  const appWindow = getCurrentWindow();
  await appWindow.onCloseRequested(event => {
    if (settings.isDirty) {
      event.preventDefault();
      confirmClose(() => void appWindow.destroy());
    }
  });
}

watch(
  () => route.name,
  () => contentRef.value?.scrollToTop()
);

onMounted(async () => {
  await statusStore.init();
  if (settings.welcome) {
    await router.replace({ name: "settings" });
  }
  await guardWindowClose();
});
</script>

<template>
  <transition name="app-fade">
    <app-splash v-if="isLoading" />
  </transition>
  <h-layout has-sider position="absolute">
    <h-layout-sider
      bordered
      class="glass"
      :collapsed="isCollapsed"
      :collapsed-width="64"
      collapse-mode="width"
      :native-scrollbar="false"
      show-trigger="bar"
      :width="220"
      @collapse="setCollapsed(true)"
      @expand="setCollapsed(false)">
      <h-flex align="center" class="app-brand" :size="10" :wrap="false">
        <h-avatar color="transparent" :size="34" :src="logo" />
        <transition name="fade">
          <h-text v-if="!isCollapsed" class="brand-wordmark app-brand-name">Baskul</h-text>
        </transition>
      </h-flex>
      <h-menu
        :collapsed="isCollapsed"
        :collapsed-icon-size="22"
        :collapsed-width="64"
        :icon-size="20"
        :options="menuOptions"
        :value="String(route.name)"
        @update:value="navigate($event)" />
    </h-layout-sider>

    <h-layout>
      <h-layout-header bordered class="app-bar glass" position="absolute">
        <h-flex align="center" class="app-bar-content" justify="space-between" :wrap="false">
          <h-flex :size="0" vertical>
            <h-text class="app-title" strong>
              {{ route.meta.title ? $t(route.meta.title) : "" }}
            </h-text>
            <h-text class="app-subtitle" depth="3">{{ subtitle }}</h-text>
          </h-flex>
          <transition name="fade">
            <h-flex v-if="!isLoading" align="center" :size="10" :wrap="false">
              <profile-menu v-if="settings.config" />
              <status-badge :is-restarting="isRestarting" :status="statusStore.status" />
            </h-flex>
          </transition>
        </h-flex>
      </h-layout-header>

      <h-layout-content
        ref="contentRef"
        class="app-content"
        content-style="padding: 24px"
        :native-scrollbar="false"
        position="absolute">
        <div class="page-body">
          <update-banner v-if="!isWaitingForDaemon" />
          <transition mode="out-in" name="fade">
            <h-result
              v-if="isWaitingForDaemon && !isLoading"
              class="app-offline"
              :description="$t('layout.offlineText')"
              status="info"
              :title="$t('layout.offlineTitle')">
              <template #footer>
                <h-button
                  :loading="statusStore.phase === 'starting'"
                  type="primary"
                  @click="statusStore.start()">
                  {{ $t("layout.start") }}
                </h-button>
              </template>
            </h-result>
            <div v-else-if="!isWaitingForDaemon">
              <!-- Sortie de durée nulle : sinon Vue attendrait la fin des transitions propres aux composants -->
              <router-view v-slot="{ Component }">
                <transition :duration="{ enter: 250, leave: 0 }" mode="out-in" name="page">
                  <keep-alive>
                    <component :is="Component" :key="route.name" />
                  </keep-alive>
                </transition>
              </router-view>
            </div>
          </transition>
        </div>
      </h-layout-content>

      <h-layout-footer bordered class="app-bar app-footer-bar glass" position="absolute">
        <app-footer />
      </h-layout-footer>
    </h-layout>
  </h-layout>
</template>

<style lang="scss" scoped>
@use "@/assets/variables";

$page-easing: cubic-bezier(0.2, 0.8, 0.2, 1);
$footer-height: 52px;

.app-brand {
  height: variables.$bar-height;
  padding: 0 15px;
}

.app-brand-name {
  font-size: 19px;
}

.app-bar {
  height: variables.$bar-height;
  padding: 0 24px;
}

.app-footer-bar {
  height: $footer-height;
  padding: 0 16px 0 24px;
}

.app-bar-content {
  height: 100%;
}

.app-title {
  font-size: 18px;
}

.app-subtitle {
  font-size: variables.$font-size-help;
}

.app-content {
  top: variables.$bar-height;
  bottom: $footer-height;
}

.app-offline {
  padding: 60px 0;
}

.page-enter-active {
  transition:
    opacity 0.25s $page-easing,
    transform 0.25s $page-easing;
}

.page-enter-from {
  opacity: 0;
  transform: scale(0.98);
}

.app-fade-leave-active {
  transition: opacity 0.3s ease;
}

.app-fade-leave-to {
  opacity: 0;
}

@media (prefers-reduced-motion: reduce) {
  .page-enter-from {
    transform: none;
  }
}
</style>
