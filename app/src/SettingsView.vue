<script setup lang="ts">
import { computed, onMounted, provide, ref, watch } from "vue";
import {
  NAlert,
  NButton,
  NCard,
  NResult,
  NSpin,
  NTabPane,
  NTabs,
  useDialog,
  useMessage,
  useThemeVars,
} from "naive-ui";
import { getCurrentWindow } from "@tauri-apps/api/window";
import StatusBadge from "./components/StatusBadge.vue";
import { useSettings } from "./composables/useSettings";
import { osKey, roleKey } from "./keys";
import AdvancedSection from "./sections/AdvancedSection.vue";
import ClipboardSection from "./sections/ClipboardSection.vue";
import ConnectionSection from "./sections/ConnectionSection.vue";
import KeyboardSection from "./sections/KeyboardSection.vue";
import MachineSection from "./sections/MachineSection.vue";
import MouseSection from "./sections/MouseSection.vue";
import SwitchingSection from "./sections/SwitchingSection.vue";
import { osLabel } from "./utils";

const settings = useSettings();
const { phase, info, config, status, dirty, saving, welcome } = settings;
const message = useMessage();
const dialog = useDialog();
const vars = useThemeVars();

const role = computed(() => config.value?.role ?? "primary");
provide(roleKey, role);
provide(
  osKey,
  computed(() => info.value?.os ?? ""),
);

// Onglet affiché ; ceux qui ne concernent pas le rôle choisi sont masqués.
const tab = ref("connexion");
watch(role, (r) => {
  if ((r === "primary" && tab.value === "souris") || (r === "secondary" && tab.value === "bascule")) {
    tab.value = "connexion";
  }
});

// Couleurs du thème Naive UI, pour les styles maison (styles.css).
const cssVars = computed(() => ({
  "--muted": vars.value.textColor3,
  "--border": vars.value.dividerColor,
  "--accent": vars.value.primaryColor,
  "--card": vars.value.cardColor,
}));

const restarting = computed(() => phase.value === "restarting");
const busy = computed(() => saving.value || restarting.value);
const subtitle = computed(() => (info.value ? `${info.value.hostname} · ${osLabel(info.value.os)}` : ""));
const footerText = computed(() => {
  if (restarting.value) return "Bascule redémarre avec les nouveaux réglages…";
  return dirty.value ? "Modifications non enregistrées." : "";
});

/** Affiche l'erreur d'une action. */
async function attempt(action: () => Promise<unknown>) {
  try {
    await action();
  } catch (e) {
    message.error(settings.errorMessage(e), { duration: 8000, closable: true });
  }
}

const save = () =>
  attempt(async () => {
    if (await settings.save()) message.success("Réglages enregistrés et appliqués.");
    else message.warning("Réglages enregistrés, mais Bascule n'a pas redémarré.");
  });

const restart = () =>
  attempt(async () => {
    if (await settings.restart()) message.success("Bascule a redémarré.");
  });

const openFile = () => attempt(settings.openConfigFile);
const start = () => attempt(settings.start);

onMounted(async () => {
  await settings.init();
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
  <div class="page" :style="cssVars">
    <header class="top">
      <div class="logo" aria-hidden="true">⇄</div>
      <div class="brand">
        <h1>Réglages de Bascule</h1>
        <p class="sub">{{ subtitle }}</p>
      </div>
      <status-badge v-if="phase !== 'loading'" :status="status" :restarting="restarting" />
    </header>

    <main>
      <div v-if="phase === 'loading'" class="center"><n-spin /></div>

      <n-result
        v-else-if="phase === 'offline' || phase === 'starting'"
        status="info"
        title="Bascule n'est pas lancé"
        description="Les réglages s'affichent dès que Bascule fonctionne."
        class="offline"
      >
        <template #footer>
          <n-button type="primary" :loading="phase === 'starting'" @click="start">Démarrer Bascule</n-button>
        </template>
      </n-result>

      <n-spin v-else-if="info" :show="restarting" description="Redémarrage de Bascule…">
        <n-alert v-if="welcome" type="info" title="Bienvenue dans Bascule !" closable class="block">
          Pour relier vos deux machines :
          <ol>
            <li>Choisissez le rôle de cette machine. La <em>principale</em> est celle qui a le clavier et la souris.</li>
            <li>
              Le secret doit être identique sur les deux machines : copiez celui-ci dans les réglages de l'autre
              machine, ou collez ici le sien.
            </li>
            <li>
              Sur la machine principale, indiquez l'adresse de l'autre dans « Se connecter à ». L'autre machine laisse ce
              champ vide.
            </li>
          </ol>
          Enregistrez : Bascule redémarre avec ces réglages.
        </n-alert>

        <n-alert v-if="!config" type="error" title="Le fichier de configuration ne peut pas être lu" class="block">
          <p class="mono">{{ info.error }}</p>
          <div class="actions">
            <n-button size="small" @click="openFile">Ouvrir le fichier</n-button>
            <n-button size="small" @click="restart">Redémarrer Bascule</n-button>
          </div>
        </n-alert>
        <n-alert v-else-if="status?.state === 'error'" type="error" class="block">{{ status.detail }}</n-alert>

        <template v-if="config">
          <machine-section :config="config" :info="info" />
          <n-card class="section tabs-card">
            <n-tabs v-model:value="tab" type="line" animated>
              <n-tab-pane name="connexion" tab="Connexion">
                <connection-section :config="config" :info="info" />
              </n-tab-pane>
              <n-tab-pane v-if="role === 'primary'" name="bascule" tab="Changer de machine">
                <switching-section :config="config" :info="info" />
              </n-tab-pane>
              <n-tab-pane v-if="role === 'secondary'" name="souris" tab="Souris">
                <mouse-section :config="config" />
              </n-tab-pane>
              <n-tab-pane name="clavier" tab="Clavier">
                <keyboard-section :config="config" @open-file="openFile" />
              </n-tab-pane>
              <n-tab-pane name="presse-papiers" tab="Presse-papiers">
                <clipboard-section :config="config" />
              </n-tab-pane>
              <n-tab-pane name="avance" tab="Avancé">
                <advanced-section
                  :config="config"
                  :info="info"
                  :restarting="restarting"
                  @open-file="openFile"
                  @restart="restart"
                />
              </n-tab-pane>
            </n-tabs>
          </n-card>
        </template>
      </n-spin>
    </main>

    <footer v-if="config && (phase === 'ready' || restarting)" class="actionbar">
      <div class="actionbar-inner">
        <p class="msg">{{ footerText }}</p>
        <n-button :disabled="!dirty || busy" @click="settings.reset()">Annuler</n-button>
        <n-button type="primary" :disabled="!dirty || busy" :loading="busy" @click="save">
          Enregistrer et redémarrer
        </n-button>
      </div>
    </footer>
  </div>
</template>
