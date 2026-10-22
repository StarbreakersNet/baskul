<script setup lang="ts">
// Cadre de l'application : menu latéral, en-tête, page courante (avec barre de
// défilement Naive UI) et pied de page d'enregistrement des réglages.
import { computed, h, onMounted, provide, ref, watch, type Component } from "vue";
import { RouterView, useRoute, useRouter } from "vue-router";
import {
  NAvatar,
  NBadge,
  NButton,
  NFlex,
  NIcon,
  NLayout,
  NLayoutContent,
  NLayoutFooter,
  NLayoutHeader,
  NLayoutSider,
  NMenu,
  NResult,
  NSpin,
  NText,
  useDialog,
  type MenuOption,
} from "naive-ui";
import { IconFileText, IconHome, IconInfoCircle, IconSettings } from "@tabler/icons-vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import logo from "../assets/logo.svg";
import StatusBadge from "../components/StatusBadge.vue";
import { useActions } from "../composables/useActions";
import { useSettings } from "../composables/useSettings";
import { osKey, roleKey } from "../keys";
import { osLabel } from "../utils";

const HEADER = 64;
const FOOTER = 64;

const settings = useSettings();
const { phase, info, config, status, dirty, saving, welcome } = settings;
const actions = useActions();
const route = useRoute();
const router = useRouter();
const dialog = useDialog();

provide(
  roleKey,
  computed(() => config.value?.role ?? "primary"),
);
provide(
  osKey,
  computed(() => info.value?.os ?? ""),
);

const collapsed = ref(false);
const content = ref<InstanceType<typeof NLayoutContent>>();

// Nouvelle page : on repart du haut.
watch(
  () => route.name,
  () => content.value?.scrollTo({ top: 0 }),
);

const icon = (c: Component) => () => h(NIcon, null, { default: () => h(c, { stroke: 1.75 }) });
const menu = computed<MenuOption[]>(() => [
  { key: "home", label: "Accueil", icon: icon(IconHome) },
  {
    key: "settings",
    // Pastille : réglages modifiés, pas encore enregistrés.
    label: () => h(NBadge, { dot: true, show: dirty.value, offset: [8, 2] }, { default: () => "Réglages" }),
    icon: icon(IconSettings),
  },
  { key: "log", label: "Journal", icon: icon(IconFileText) },
  { key: "about", label: "À propos", icon: icon(IconInfoCircle) },
]);

const restarting = computed(() => phase.value === "restarting");
const busy = computed(() => saving.value || restarting.value);
const subtitle = computed(() => (info.value ? `${info.value.hostname} · ${osLabel(info.value.os)}` : ""));
/** Bascule doit tourner pour afficher la page. */
const gated = computed(() => route.meta.daemon && phase.value !== "ready" && !restarting.value);

const footer = computed(
  () => !!config.value && (phase.value === "ready" || restarting.value) && (route.name === "settings" || dirty.value),
);
const footerText = computed(() => {
  if (restarting.value) return "Bascule redémarre avec les nouveaux réglages…";
  return dirty.value ? "Modifications non enregistrées." : "";
});

onMounted(async () => {
  await settings.init();
  if (welcome.value) await router.replace({ name: "settings" });
  // Fermeture avec des modifications non enregistrées : confirmation.
  const win = getCurrentWindow();
  await win.onCloseRequested((event) => {
    if (!dirty.value) return;
    event.preventDefault();
    dialog.warning({
      title: "Modifications non enregistrées",
      content: "Fermer sans enregistrer ?",
      positiveText: "Fermer",
      negativeText: "Annuler",
      onPositiveClick: () => void win.destroy(),
    });
  });
});
</script>

<template>
  <n-layout has-sider position="absolute">
    <n-layout-sider
      bordered
      show-trigger
      collapse-mode="width"
      :collapsed="collapsed"
      :collapsed-width="64"
      :width="220"
      :native-scrollbar="false"
      @collapse="collapsed = true"
      @expand="collapsed = false"
    >
      <n-flex align="center" :size="12" :wrap="false" :style="{ height: `${HEADER}px`, padding: '0 16px' }">
        <n-avatar :src="logo" :size="32" color="transparent" />
        <n-text v-if="!collapsed" strong style="font-size: 17px">Bascule</n-text>
      </n-flex>
      <n-menu
        :value="String(route.name)"
        :options="menu"
        :collapsed="collapsed"
        :collapsed-width="64"
        :collapsed-icon-size="22"
        @update:value="(name: string) => router.push({ name })"
      />
    </n-layout-sider>

    <n-layout>
      <n-layout-header bordered position="absolute" :style="{ height: `${HEADER}px`, padding: '0 24px' }">
        <n-flex align="center" justify="space-between" :wrap="false" style="height: 100%">
          <n-flex vertical :size="0">
            <n-text strong style="font-size: 18px">{{ route.meta.title }}</n-text>
            <n-text depth="3" style="font-size: 13px">{{ subtitle }}</n-text>
          </n-flex>
          <status-badge v-if="phase !== 'loading'" :status="status" :restarting="restarting" />
        </n-flex>
      </n-layout-header>

      <n-layout-content
        ref="content"
        position="absolute"
        :native-scrollbar="false"
        :style="{ top: `${HEADER}px`, bottom: footer ? `${FOOTER}px` : '0' }"
        content-style="padding: 24px"
      >
        <div class="page-body">
          <n-flex v-if="gated && phase === 'loading'" justify="center" style="padding: 80px 0">
            <n-spin />
          </n-flex>
          <n-result
            v-else-if="gated"
            status="info"
            title="Bascule n'est pas lancé"
            description="Cette page s'affiche dès que Bascule fonctionne."
            style="padding: 60px 0"
          >
            <template #footer>
              <n-button type="primary" :loading="phase === 'starting'" @click="actions.start">
                Démarrer Bascule
              </n-button>
            </template>
          </n-result>
          <!-- Transition à la manière d'iOS : la nouvelle page apparaît en
               fondu avec un léger zoom, l'ancienne disparaît aussitôt. -->
          <router-view v-else v-slot="{ Component }">
            <transition name="page" mode="out-in">
              <component :is="Component" :key="route.name" />
            </transition>
          </router-view>
        </div>
      </n-layout-content>

      <n-layout-footer v-if="footer" bordered position="absolute" :style="{ height: `${FOOTER}px`, padding: '0 24px' }">
        <n-flex align="center" :size="10" :wrap="false" style="height: 100%">
          <n-text depth="3" style="flex: 1">{{ footerText }}</n-text>
          <n-button :disabled="!dirty || busy" @click="settings.reset()">Annuler</n-button>
          <n-button type="primary" :disabled="!dirty || busy" :loading="busy" @click="actions.save">
            Enregistrer et redémarrer
          </n-button>
        </n-flex>
      </n-layout-footer>
    </n-layout>
  </n-layout>
</template>

<style scoped>
/* Entrée : fondu et léger zoom ; sortie immédiate (aucune transition). */
.page-enter-active {
  transition:
    opacity 0.3s cubic-bezier(0.2, 0.8, 0.2, 1),
    transform 0.3s cubic-bezier(0.2, 0.8, 0.2, 1);
}
.page-enter-from {
  opacity: 0;
  transform: scale(0.98);
}
@media (prefers-reduced-motion: reduce) {
  .page-enter-from {
    transform: none;
  }
}
</style>
