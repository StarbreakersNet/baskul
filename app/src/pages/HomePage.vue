<script setup lang="ts">
// Accueil : l'état de Bascule en un coup d'œil (connexion, machine pilotée en
// ce moment, raccourcis), mis à jour en direct.
import { computed, onBeforeUnmount, ref, type Component } from "vue";
import { useRouter } from "vue-router";
import {
  NAlert,
  NButton,
  NCard,
  NDescriptions,
  NDescriptionsItem,
  NFlex,
  NGi,
  NGrid,
  NIcon,
  NTag,
  NText,
  NTime,
  useThemeVars,
} from "naive-ui";
import {
  CheckmarkCircle,
  CloseCircle,
  DesktopOutline,
  HourglassOutline,
  LaptopOutline,
  SwapHorizontal,
} from "@vicons/ionicons5";
import { useActions } from "../composables/useActions";
import { useSettings } from "../composables/useSettings";
import { osLabel } from "../utils";

const { info, config, status } = useSettings();
const actions = useActions();
const router = useRouter();
const vars = useThemeVars();

// Heure actuelle, pour la durée de la connexion.
const now = ref(Date.now());
const clock = setInterval(() => (now.value = Date.now()), 15_000);
onBeforeUnmount(() => clearInterval(clock));

const primary = computed(() => config.value?.role !== "secondary");
const connected = computed(() => status.value?.state === "connected");
const peerName = computed(() => (connected.value ? status.value!.detail : null));
const capitalize = (s: string) => s.charAt(0).toUpperCase() + s.slice(1);

/** État de la connexion. */
const conn = computed((): { icon: Component; color: string; title: string; help?: string } => {
  const s = status.value;
  if (!s) return { icon: HourglassOutline, color: vars.value.textColor3, title: "Démarrage…" };
  if (s.state === "connected") {
    return { icon: CheckmarkCircle, color: vars.value.successColor, title: `Connecté à « ${s.detail} »` };
  }
  if (s.state === "error") {
    return { icon: CloseCircle, color: vars.value.errorColor, title: "Pas de connexion", help: capitalize(s.detail) };
  }
  return {
    icon: HourglassOutline,
    color: vars.value.warningColor,
    title: capitalize(s.detail || "en attente de l'autre machine"),
    help: "Vérifiez que Bascule est lancé sur l'autre machine, avec le même secret.",
  };
});

/** Le clavier et la souris agissent en ce moment sur cette machine. */
const here = computed(() => {
  const remote = !!status.value?.remote_active;
  return primary.value ? !remote : connected.value && remote;
});

const osIcon = (os?: string) => (os === "macos" ? LaptopOutline : DesktopOutline);

const machines = computed(() => [
  {
    key: "here",
    name: config.value?.name || info.value?.hostname || "Cette machine",
    label: "Cette machine",
    os: info.value?.os,
    active: here.value,
  },
  {
    key: "peer",
    name: peerName.value ?? "Non connectée",
    label: "Autre machine",
    os: status.value?.peer?.os,
    active: connected.value && !here.value,
  },
]);
const activeStyle = computed(() => ({ boxShadow: `inset 0 0 0 1.5px ${vars.value.primaryColor}` }));

const hotkeys = computed(() => {
  const h = config.value?.hotkeys;
  return h
    ? [
        { label: "Aller sur la machine pilotée", keys: h.switch_to_secondary },
        { label: "Revenir sur cette machine", keys: h.return_to_primary },
        { label: "Verrouiller le curseur", keys: h.toggle_lock },
      ]
    : [];
});
const edges = computed(() => {
  const s = config.value?.screen;
  if (!s?.edge_switching) return "Désactivée";
  const where = { left: "à gauche", right: "à droite", top: "en haut", bottom: "en bas" }[s.secondary_position];
  return `Activée (autre écran ${where})`;
});
const scroll = computed(() => {
  const s = config.value?.scroll;
  if (!s) return "";
  if (s.mode === "smooth") return "Fluide, comme Mac Mouse Fix";
  return s.mode === "pixels" ? `Au pixel (${s.pixels_per_notch} px par cran)` : `Par lignes (${s.lines_per_notch} par cran)`;
});
const clipboard = computed(() => {
  const c = config.value?.clipboard;
  if (!c?.enabled) return "Non partagé";
  return c.images ? "Partagé, images comprises" : "Partagé (texte seulement)";
});

const editSettings = (onglet?: string) => router.push({ name: "settings", query: onglet ? { onglet } : {} });
</script>

<template>
  <n-flex vertical :size="16">
    <n-alert v-if="status?.warning" type="warning" title="Attention">{{ status.warning }}</n-alert>

    <n-card>
      <n-flex align="center" :size="16" :wrap="false">
        <n-icon :size="44" :color="conn.color"><component :is="conn.icon" /></n-icon>
        <n-flex vertical :size="2">
          <n-text strong style="font-size: 18px">{{ conn.title }}</n-text>
          <n-text v-if="connected && status?.peer" depth="3">
            {{ osLabel(status.peer.os) }} · depuis <n-time :time="status.peer.since" format="HH:mm" /> (<n-time
              :time="status.peer.since"
              :to="now"
              type="relative"
            />)
          </n-text>
          <n-text v-else-if="conn.help" depth="3">{{ conn.help }}</n-text>
        </n-flex>
      </n-flex>

      <n-flex align="center" justify="center" :size="16" :wrap="false" style="margin-top: 20px">
        <template v-for="(m, i) in machines" :key="m.key">
          <n-icon v-if="i > 0" :size="26" :depth="3"><swap-horizontal /></n-icon>
          <n-card size="small" embedded :bordered="false" style="flex: 1; max-width: 300px" :style="m.active ? activeStyle : {}">
            <n-flex vertical align="center" :size="4">
              <n-icon :size="36" :color="m.active ? vars.primaryColor : undefined" :depth="m.active ? undefined : 3">
                <component :is="osIcon(m.os)" />
              </n-icon>
              <n-text strong>{{ m.name }}</n-text>
              <n-text depth="3" style="font-size: 13px">{{ m.label }}{{ m.os ? ` · ${osLabel(m.os)}` : "" }}</n-text>
              <n-tag v-if="m.active" type="primary" size="small" round :bordered="false">Clavier et souris</n-tag>
              <n-tag v-else-if="m.key === 'here' && status?.locked" size="small" round :bordered="false">
                Curseur verrouillé
              </n-tag>
            </n-flex>
          </n-card>
        </template>
      </n-flex>
    </n-card>

    <n-grid v-if="info && config" cols="1 640:2" responsive="self" :x-gap="16" :y-gap="16">
      <n-gi>
        <n-card title="Cette machine" size="small" style="height: 100%">
          <n-descriptions label-placement="left" :column="1" size="small">
            <n-descriptions-item label="Rôle">
              {{ primary ? "Principale (clavier et souris)" : "Pilotée" }}
            </n-descriptions-item>
            <n-descriptions-item label="Système">{{ osLabel(info.os) }}</n-descriptions-item>
            <n-descriptions-item v-if="info.address" label="Adresse IP">
              <n-text code>{{ info.address }}</n-text>
            </n-descriptions-item>
            <n-descriptions-item label="Connexion">
              <template v-if="config.network.connect">Se connecte à <n-text code>{{ config.network.connect }}</n-text></template>
              <template v-else>Attend sur <n-text code>{{ config.network.listen }}</n-text></template>
            </n-descriptions-item>
            <n-descriptions-item label="Version">{{ info.version }}</n-descriptions-item>
          </n-descriptions>
        </n-card>
      </n-gi>
      <n-gi>
        <n-card v-if="primary" title="Changer de machine" size="small" style="height: 100%">
          <template #header-extra>
            <n-button text type="primary" @click="editSettings('bascule')">Modifier</n-button>
          </template>
          <n-descriptions label-placement="left" :column="1" size="small">
            <n-descriptions-item v-for="hk in hotkeys" :key="hk.label" :label="hk.label">
              <n-tag v-if="hk.keys" size="small" class="mono">{{ hk.keys.split("+").join(" + ") }}</n-tag>
              <n-text v-else depth="3">Aucun</n-text>
            </n-descriptions-item>
            <n-descriptions-item label="Bords de l'écran">{{ edges }}</n-descriptions-item>
          </n-descriptions>
        </n-card>
        <n-card v-else title="Pilotage" size="small" style="height: 100%">
          <template #header-extra>
            <n-button text type="primary" @click="editSettings('souris')">Modifier</n-button>
          </template>
          <n-descriptions label-placement="left" :column="1" size="small">
            <n-descriptions-item label="Vitesse de la souris">× {{ config.mouse.speed }}</n-descriptions-item>
            <n-descriptions-item label="Défilement">{{ scroll }}</n-descriptions-item>
            <n-descriptions-item label="Presse-papiers">{{ clipboard }}</n-descriptions-item>
          </n-descriptions>
        </n-card>
      </n-gi>
    </n-grid>

    <n-flex :size="8">
      <n-button @click="editSettings()">Réglages</n-button>
      <n-button @click="router.push({ name: 'log' })">Journal</n-button>
      <n-button @click="actions.restart">Redémarrer Bascule</n-button>
    </n-flex>
  </n-flex>
</template>
