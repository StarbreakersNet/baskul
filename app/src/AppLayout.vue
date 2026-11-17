<script setup lang="ts">
import {
  computed,
  h,
  onMounted,
  provide,
  ref,
  useTemplateRef,
  watch,
  type Component,
  type ComponentPublicInstance,
} from "vue";
import { RouterView, useRoute, useRouter } from "vue-router";
import { useDialog, useMessage, useThemeVars, type MenuOption } from "naive-ui";
import { IconFileText, IconHome, IconInfoCircle, IconSettings } from "@tabler/icons-vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useI18n } from "vue-i18n";
import HAvatar from "@/components/c/hAvatar.vue";
import HBadge from "@/components/c/hBadge.vue";
import HButton from "@/components/c/hButton.vue";
import HCard from "@/components/c/hCard.vue";
import HFlex from "@/components/c/hFlex.vue";
import HIcon from "@/components/c/hIcon.vue";
import HLayout from "@/components/c/hLayout.vue";
import HLayoutContent from "@/components/c/hLayoutContent.vue";
import HLayoutSider from "@/components/c/hLayoutSider.vue";
import HMenu from "@/components/c/hMenu.vue";
import HResult from "@/components/c/hResult.vue";
import HText from "@/components/c/hText.vue";
import AppBackground from "@/components/AppBackground.vue";
import BrandName from "@/components/BrandName.vue";
import AppFooter from "@/components/AppFooter.vue";
import AppSplash from "@/components/AppSplash.vue";
import ProfileMenu from "@/components/ProfileMenu.vue";
import StatusBadge from "@/components/StatusBadge.vue";
import UpdateBanner from "@/components/UpdateBanner.vue";
import logo from "@/assets/logo.svg";
import { useAppShortcuts } from "@/composables/useAppShortcuts";
import { registerMessageApi } from "@/composables/useAppMessage";
import { useSettingsStore } from "@/stores/settings";
import { useBackgroundTuningStore } from "@/stores/backgroundTuning";
import { useStatusStore } from "@/stores/status";
import { OS_KEY, ROLE_KEY } from "@/utils/injectionKeys";
import { readLocalFlag, writeLocalFlag } from "@/utils/localSetting";
import { withAlpha } from "@/utils/color";
import { PANEL_RADIUS } from "@/utils/naiveTheme";
import { osLabel } from "@/utils/os";
import type { BaskulConfig } from "@/types/Config";
import type { HLayoutContentInst } from "@/types/components/c/hLayoutContent";

const { background } = defineProps<{
  background: BaskulConfig["ui"]["background"];
}>();

const SIDER_COLLAPSED_KEY = "baskul.siderCollapsed";

registerMessageApi(useMessage());

const settings = useSettingsStore();
const statusStore = useStatusStore();
const route = useRoute();
const router = useRouter();
const dialog = useDialog();
const themeVars = useThemeVars();
const backgroundTuning = useBackgroundTuningStore();
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
const statusBadgeRef = useTemplateRef<ComponentPublicInstance>("statusBadgeRef");

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
// Cadre (barre latérale, en-tête, pied de page) : translucide et flou quand le fond animé est actif,
// les cartes de la page restent opaques
const frameColor = computed(() =>
  withAlpha(
    themeVars.value.cardColor,
    background === "none" ? 1 : backgroundTuning.tuning.frameOpacity
  )
);
const frameBackdrop = computed(() =>
  background !== "none" && backgroundTuning.tuning.frameBlur > 0
    ? `blur(${backgroundTuning.tuning.frameBlur}px)`
    : "none"
);
const pulseOrigin = computed(() => statusBadgeRef.value?.$el as HTMLElement | undefined);
const pulseColor = computed(() => {
  const state = isRestarting.value ? null : statusStore.status?.state;
  if (state === "connected") {
    return themeVars.value.successColor;
  }
  if (state === "waiting") {
    return themeVars.value.warningColor;
  }
  return state === "error" ? themeVars.value.errorColor : themeVars.value.infoColor;
});
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
  <app-background
    v-if="background !== 'none'"
    :origin="pulseOrigin"
    :pattern="background"
    :pulse-color="pulseColor" />
  <transition name="app-fade">
    <app-splash v-if="isLoading" />
  </transition>
  <h-layout has-sider position="absolute">
    <h-layout-sider
      class="app-frame app-sider"
      :collapsed="isCollapsed"
      :collapsed-width="64"
      collapse-mode="width"
      :native-scrollbar="false"
      show-trigger="arrow-circle"
      :theme-overrides="{ siderColor: frameColor }"
      :width="220"
      @collapse="setCollapsed(true)"
      @expand="setCollapsed(false)">
      <h-flex align="center" class="app-brand" :size="10" :wrap="false">
        <h-avatar color="transparent" :size="34" :src="logo" />
        <transition name="fade">
          <brand-name v-if="!isCollapsed" class="app-brand-name" />
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

    <div class="app-main">
      <h-card
        class="app-frame app-header"
        :theme-overrides="{ color: frameColor, paddingMedium: '0 24px' }">
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
              <status-badge
                ref="statusBadgeRef"
                :is-restarting="isRestarting"
                :status="statusStore.status" />
            </h-flex>
          </transition>
        </h-flex>
      </h-card>

      <h-layout-content ref="contentRef" class="app-content" :native-scrollbar="false">
        <div class="app-page">
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

      <h-card
        class="app-frame app-footer"
        :theme-overrides="{ color: frameColor, paddingMedium: '0 16px 0 24px' }">
        <app-footer />
      </h-card>
    </div>
  </h-layout>
</template>

<style lang="scss" scoped>
@use "@/assets/variables";

$page-easing: cubic-bezier(0.2, 0.8, 0.2, 1);
$footer-height: 52px;
$gap: variables.$panel-gap;

.app-frame {
  -webkit-backdrop-filter: v-bind(frameBackdrop);
  backdrop-filter: v-bind(frameBackdrop);
}

// Naive ne trace qu'un côté du bord de la barre latérale ; sa couleur vient du thème (Layout)
.app-sider {
  height: calc(100% - #{2 * $gap});
  margin: $gap 0 $gap $gap;
  border: 1px solid var(--n-border-color);
  border-radius: v-bind(PANEL_RADIUS);
}

.app-brand {
  height: variables.$bar-height;
  padding: 0 14px;
}

.app-brand-name {
  font-size: 19px;
}

// En-tête et pied de page fixes ; entre les deux, la seule zone qui défile, au ras de l'un et de l'autre.
// Le contenu commence à un écart d'eux, et s'efface dans cet écart en défilant. La zone déborde dans la
// marge de droite : la barre de défilement y passe, à côté des cartes
.app-main {
  display: flex;
  flex: 1;
  flex-direction: column;
  box-sizing: border-box;
  min-width: 0;
  padding: $gap;
}

.app-header {
  flex: none;
  height: variables.$bar-height;
}

.app-footer {
  flex: none;
  height: $footer-height;
}

.app-content {
  flex: 1;
  min-height: 0;
  margin-right: -$gap;
  -webkit-mask-image: linear-gradient(
    transparent,
    #000 $gap,
    #000 calc(100% - #{$gap}),
    transparent
  );
  mask-image: linear-gradient(transparent, #000 $gap, #000 calc(100% - #{$gap}), transparent);
}

.app-page {
  padding: $gap $gap $gap 0;
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
