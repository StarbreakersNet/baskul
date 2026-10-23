<script setup lang="ts">
// Accueil : l'état de Bascule en un coup d'œil (connexion, machine pilotée en
// ce moment, raccourcis), mis à jour en direct.
import { computed, onActivated, onDeactivated, ref, type Component } from "vue";
import { useRouter } from "vue-router";
import { useI18n } from "vue-i18n";
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
  IconArrowsLeftRight,
  IconCircleCheckFilled,
  IconCircleXFilled,
  IconDeviceDesktop,
  IconDeviceLaptop,
  IconHourglassHigh,
} from "@tabler/icons-vue";
import { useActions } from "../composables/useActions";
import { useSettings } from "../composables/useSettings";
import { osLabel } from "../utils";

const { info, config, status } = useSettings();
const actions = useActions();
const router = useRouter();
const vars = useThemeVars();
const { t } = useI18n();

// Heure actuelle, pour la durée de la connexion (page affichée seulement).
const now = ref(Date.now());
let clock: ReturnType<typeof setInterval> | undefined;
onActivated(() => {
  now.value = Date.now();
  clock = setInterval(() => (now.value = Date.now()), 15_000);
});
onDeactivated(() => clearInterval(clock));

const primary = computed(() => config.value?.role !== "secondary");
const connected = computed(() => status.value?.state === "connected");
const peerName = computed(() => (connected.value ? status.value!.detail : null));
const capitalize = (s: string) => s.charAt(0).toUpperCase() + s.slice(1);

/** État de la connexion. */
const conn = computed((): { icon: Component; color: string; title: string; help?: string } => {
  const s = status.value;
  if (!s) return { icon: IconHourglassHigh, color: vars.value.textColor3, title: t("home.starting") };
  if (s.state === "connected") {
    return { icon: IconCircleCheckFilled, color: vars.value.successColor, title: t("home.connectedTo", { peer: s.detail }) };
  }
  if (s.state === "error") {
    return { icon: IconCircleXFilled, color: vars.value.errorColor, title: t("home.noConnection"), help: capitalize(s.detail) };
  }
  // L'état détaillé vient de Bascule, dans la langue choisie.
  return {
    icon: IconHourglassHigh,
    color: vars.value.warningColor,
    title: s.detail ? capitalize(s.detail) : t("home.waiting"),
    help: t("home.waitingHelp"),
  };
});

/** Le clavier et la souris agissent en ce moment sur cette machine. */
const here = computed(() => {
  const remote = !!status.value?.remote_active;
  return primary.value ? !remote : connected.value && remote;
});

const osIcon = (os?: string) => (os === "macos" ? IconDeviceLaptop : IconDeviceDesktop);

const machines = computed(() => [
  {
    key: "here",
    name: config.value?.name || info.value?.hostname || t("home.thisMachine"),
    label: t("home.thisMachine"),
    os: info.value?.os,
    active: here.value,
  },
  {
    key: "peer",
    name: peerName.value ?? t("home.notConnected"),
    label: t("home.otherMachine"),
    os: status.value?.peer?.os,
    active: connected.value && !here.value,
  },
]);
const activeStyle = computed(() => ({ boxShadow: `inset 0 0 0 1.5px ${vars.value.primaryColor}` }));

const hotkeys = computed(() => {
  const h = config.value?.hotkeys;
  return h
    ? [
        { label: t("home.goToSecondary"), keys: h.switch_to_secondary },
        { label: t("home.returnHere"), keys: h.return_to_primary },
        { label: t("home.lockCursor"), keys: h.toggle_lock },
      ]
    : [];
});
const edges = computed(() => {
  const s = config.value?.screen;
  if (!s?.edge_switching) return t("home.edgesOff");
  return t("home.edgesOn", { where: t(`home.where.${s.secondary_position}`) });
});
const scroll = computed(() => {
  const s = config.value?.scroll;
  if (!s) return "";
  if (s.mode === "smooth") return t("home.scrollSmooth");
  return s.mode === "pixels"
    ? t("home.scrollPixels", { n: s.pixels_per_notch })
    : t("home.scrollLines", { n: s.lines_per_notch });
});
const clipboard = computed(() => {
  const c = config.value?.clipboard;
  if (!c?.enabled) return t("home.clipboardOff");
  return c.images ? t("home.clipboardImages") : t("home.clipboardText");
});

const editSettings = (onglet?: string) => router.push({ name: "settings", query: onglet ? { onglet } : {} });
</script>

<template>
  <n-flex vertical :size="16">
    <n-alert v-if="status?.warning" type="warning">{{ status.warning }}</n-alert>

    <n-card>
      <n-flex align="center" :size="16" :wrap="false">
        <n-icon :size="44" :color="conn.color"><component :is="conn.icon" /></n-icon>
        <n-flex vertical :size="2">
          <n-text strong style="font-size: 18px">{{ conn.title }}</n-text>
          <n-text v-if="connected && status?.peer" depth="3">
            <i18n-t keypath="home.since" tag="span">
              <template #os>{{ osLabel(status.peer.os) }}</template>
              <template #time><n-time :time="status.peer.since" format="HH:mm" /></template>
              <template #ago><n-time :time="status.peer.since" :to="now" type="relative" /></template>
            </i18n-t>
          </n-text>
          <n-text v-else-if="conn.help" depth="3">{{ conn.help }}</n-text>
        </n-flex>
      </n-flex>

      <n-flex align="center" justify="center" :size="16" :wrap="false" style="margin-top: 20px">
        <template v-for="(m, i) in machines" :key="m.key">
          <n-icon v-if="i > 0" :size="26" :depth="3"><icon-arrows-left-right /></n-icon>
          <n-card size="small" embedded :bordered="false" style="flex: 1; max-width: 300px" :style="m.active ? activeStyle : {}">
            <n-flex vertical align="center" :size="4">
              <n-icon :size="36" :color="m.active ? vars.primaryColor : undefined" :depth="m.active ? undefined : 3">
                <component :is="osIcon(m.os)" />
              </n-icon>
              <n-text strong>{{ m.name }}</n-text>
              <n-text depth="3" style="font-size: 13px">{{ m.label }}{{ m.os ? ` · ${osLabel(m.os)}` : "" }}</n-text>
              <n-tag v-if="m.active" type="primary" size="small" round :bordered="false">
                {{ $t("home.keyboardMouse") }}
              </n-tag>
              <n-tag v-else-if="m.key === 'here' && status?.locked" size="small" round :bordered="false">
                {{ $t("home.cursorLocked") }}
              </n-tag>
            </n-flex>
          </n-card>
        </template>
      </n-flex>
    </n-card>

    <n-grid v-if="info && config" cols="1 640:2" responsive="self" :x-gap="16" :y-gap="16">
      <n-gi>
        <n-card :title="$t('home.thisMachine')" size="small" style="height: 100%">
          <n-descriptions label-placement="left" :column="1" size="small">
            <n-descriptions-item :label="$t('home.role')">
              {{ primary ? $t("home.rolePrimary") : $t("home.roleSecondary") }}
            </n-descriptions-item>
            <n-descriptions-item :label="$t('home.system')">{{ osLabel(info.os) }}</n-descriptions-item>
            <n-descriptions-item v-if="info.address" :label="$t('home.ipAddress')">
              <n-text code>{{ info.address }}</n-text>
            </n-descriptions-item>
            <n-descriptions-item :label="$t('home.connection')">
              {{
                config.network.connect
                  ? $t("home.connectsTo", { address: config.network.connect })
                  : $t("home.listensOn", { address: config.network.listen })
              }}
            </n-descriptions-item>
            <n-descriptions-item :label="$t('home.version')">{{ info.version }}</n-descriptions-item>
          </n-descriptions>
        </n-card>
      </n-gi>
      <n-gi>
        <n-card v-if="primary" :title="$t('home.switching')" size="small" style="height: 100%">
          <template #header-extra>
            <n-button text type="primary" @click="editSettings('bascule')">{{ $t("common.edit") }}</n-button>
          </template>
          <n-descriptions label-placement="left" :column="1" size="small">
            <n-descriptions-item v-for="hk in hotkeys" :key="hk.label" :label="hk.label">
              <n-tag v-if="hk.keys" size="small" class="mono">{{ hk.keys.split("+").join(" + ") }}</n-tag>
              <n-text v-else depth="3">{{ $t("common.none") }}</n-text>
            </n-descriptions-item>
            <n-descriptions-item :label="$t('home.screenEdges')">{{ edges }}</n-descriptions-item>
          </n-descriptions>
        </n-card>
        <n-card v-else :title="$t('home.control')" size="small" style="height: 100%">
          <template #header-extra>
            <n-button text type="primary" @click="editSettings('souris')">{{ $t("common.edit") }}</n-button>
          </template>
          <n-descriptions label-placement="left" :column="1" size="small">
            <n-descriptions-item :label="$t('home.mouseSpeed')">× {{ config.mouse.speed }}</n-descriptions-item>
            <n-descriptions-item :label="$t('home.scrolling')">{{ scroll }}</n-descriptions-item>
            <n-descriptions-item :label="$t('home.clipboard')">{{ clipboard }}</n-descriptions-item>
          </n-descriptions>
        </n-card>
      </n-gi>
    </n-grid>

    <n-flex :size="8">
      <n-button @click="editSettings()">{{ $t("home.settings") }}</n-button>
      <n-button @click="router.push({ name: 'log' })">{{ $t("home.log") }}</n-button>
      <n-button @click="actions.restart">{{ $t("common.restart") }}</n-button>
    </n-flex>
  </n-flex>
</template>

