<script setup lang="ts">
import { computed } from "vue";
import { NButton, NInput, NPopconfirm, useMessage } from "naive-ui";
import SettingRow from "../components/SettingRow.vue";
import SettingsSection from "../components/SettingsSection.vue";
import type { BasculeConfig, DaemonInfo } from "../types";
import { copyText, newSecret } from "../utils";

const props = defineProps<{ config: BasculeConfig; info: DaemonInfo }>();
const message = useMessage();
const secretTooShort = computed(() => props.config.network.secret.trim().length < 12);

async function copy(text: string) {
  try {
    await copyText(text);
    message.success("Copié");
  } catch {
    message.error("Copie impossible : sélectionnez le texte et copiez-le à la main.");
  }
}
</script>

<template>
  <settings-section title="Connexion">
    <template #intro>
      Une machine attend la connexion, l'autre s'y connecte. Conseillé&nbsp;: le Mac attend, le PC se connecte.
    </template>
    <setting-row v-if="info.address" label="Adresse de cette machine" help="À indiquer dans « Se connecter à » sur l'autre machine.">
      <code class="address">{{ info.address }}</code>
      <n-button size="small" @click="copy(info.address!)">Copier</n-button>
    </setting-row>
    <setting-row label="Se connecter à">
      <template #help>
        Adresse IP (ou nom <code>.local</code>) de l'autre machine. Vide&nbsp;: cette machine attend la connexion.
      </template>
      <n-input
        v-model:value="config.network.connect"
        class="mono"
        placeholder="Vide : attendre la connexion"
        :input-props="{ spellcheck: false, autocomplete: 'off' }"
      />
    </setting-row>
    <setting-row label="Secret partagé" help="Identique sur les deux machines (12 caractères minimum).">
      <n-input
        v-model:value="config.network.secret"
        class="mono full"
        :status="secretTooShort ? 'error' : undefined"
        :input-props="{ spellcheck: false, autocomplete: 'off' }"
      />
      <n-button size="small" @click="copy(config.network.secret)">Copier</n-button>
      <n-popconfirm positive-text="Générer" negative-text="Annuler" @positive-click="config.network.secret = newSecret()">
        <template #trigger>
          <n-button size="small" quaternary>Nouveau</n-button>
        </template>
        Le nouveau secret devra aussi être recopié sur l'autre machine.
      </n-popconfirm>
    </setting-row>
    <setting-row label="Adresse d'écoute" help="Utilisée quand « Se connecter à » est vide. Rarement à changer.">
      <n-input v-model:value="config.network.listen" class="mono" :input-props="{ spellcheck: false }" />
    </setting-row>
  </settings-section>
</template>
