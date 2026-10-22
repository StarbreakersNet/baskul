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
  <n-spin v-if="info" :show="restarting" description="Redémarrage de Bascule…">
    <n-flex vertical :size="16">
      <n-alert v-if="welcome" type="info" title="Bienvenue dans Bascule !" closable>
        Pour relier vos deux machines :
        <ol style="margin: 6px 0; padding-left: 20px">
          <li>Choisissez le rôle de cette machine. La <em>principale</em> est celle qui a le clavier et la souris.</li>
          <li>
            Le secret doit être identique sur les deux machines : copiez celui-ci dans les réglages de l'autre machine,
            ou collez ici le sien.
          </li>
          <li>
            Sur la machine principale, indiquez l'adresse de l'autre dans « Se connecter à ». L'autre machine laisse ce
            champ vide.
          </li>
        </ol>
        Enregistrez : Bascule redémarre avec ces réglages.
      </n-alert>

      <n-alert v-if="!config" type="error" title="Le fichier de configuration ne peut pas être lu">
        <p class="mono">{{ info.error }}</p>
        <n-flex :size="8">
          <n-button size="small" @click="actions.openConfigFile">Ouvrir le fichier</n-button>
          <n-button size="small" @click="actions.restart">Redémarrer Bascule</n-button>
        </n-flex>
      </n-alert>
      <n-alert v-else-if="status?.state === 'error'" type="error">{{ status.detail }}</n-alert>

      <template v-if="config">
        <machine-section :config="config" :info="info" />
        <n-card size="small">
          <n-tabs v-model:value="tab" type="segment" animated>
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
              <keyboard-section :config="config" @open-file="actions.openConfigFile" />
            </n-tab-pane>
            <n-tab-pane name="presse-papiers" tab="Presse-papiers">
              <clipboard-section :config="config" />
            </n-tab-pane>
            <n-tab-pane name="avance" tab="Avancé">
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
