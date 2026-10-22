<script setup lang="ts">
// À propos de Bascule.
import { onMounted, ref } from "vue";
import { NAvatar, NButton, NCard, NDescriptions, NDescriptionsItem, NFlex, NH2, NText, useMessage } from "naive-ui";
import { getVersion } from "@tauri-apps/api/app";
import logo from "../assets/logo.svg";
import { useActions } from "../composables/useActions";
import { useSettings } from "../composables/useSettings";
import { copyText, osLabel } from "../utils";

const REPO = "https://github.com/StarbreakersNet/bascule";

const { info } = useSettings();
const actions = useActions();
const message = useMessage();
const appVersion = ref("");

onMounted(async () => {
  appVersion.value = await getVersion().catch(() => "");
});

async function copyRepo() {
  try {
    await copyText(REPO);
    message.success("Adresse copiée");
  } catch {
    message.error("Copie impossible");
  }
}
</script>

<template>
  <n-flex vertical :size="16">
    <n-card>
      <n-flex vertical align="center" :size="6" style="padding: 8px 0">
        <n-avatar :src="logo" :size="80" color="transparent" />
        <n-h2 style="margin: 8px 0 0">Bascule</n-h2>
        <n-text depth="3">Version {{ info?.version ?? appVersion }}</n-text>
        <n-text style="text-align: center">
          Un seul clavier, une seule souris et un presse-papiers partagé entre un PC et un Mac.
        </n-text>
      </n-flex>
    </n-card>

    <n-card title="Informations" size="small">
      <n-descriptions label-placement="left" :column="1" size="small" bordered label-style="white-space: nowrap">
        <n-descriptions-item label="Bascule">{{ info?.version ?? "non lancé" }}</n-descriptions-item>
        <n-descriptions-item label="Application">{{ appVersion }}</n-descriptions-item>
        <n-descriptions-item v-if="info" label="Machine">{{ info.hostname }} · {{ osLabel(info.os) }}</n-descriptions-item>
        <n-descriptions-item v-if="info" label="Configuration">
          <n-flex align="center" :size="8">
            <n-text class="mono" style="word-break: break-all">{{ info.path }}</n-text>
            <n-button size="tiny" @click="actions.openConfigFile">Ouvrir</n-button>
          </n-flex>
        </n-descriptions-item>
        <n-descriptions-item label="Code source">
          <n-flex align="center" :size="8">
            <n-text class="mono">{{ REPO }}</n-text>
            <n-button size="tiny" @click="copyRepo">Copier</n-button>
          </n-flex>
        </n-descriptions-item>
        <n-descriptions-item label="Licence">MIT</n-descriptions-item>
      </n-descriptions>
    </n-card>

    <n-card title="Remerciements" size="small">
      <n-text>
        Construit avec Rust, Tauri, Vue et Naive UI. Le défilement fluide reprend l'algorithme de Mac Mouse Fix, de Noah
        Nuebling.
      </n-text>
    </n-card>
  </n-flex>
</template>
