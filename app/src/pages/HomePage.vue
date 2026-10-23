<script setup lang="ts">
// Accueil : l'état de Baskul en un coup d'œil (connexion, machine pilotée en
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
  useMessage,
  useThemeVars,
} from "naive-ui";
import {
  IconCircleCheckFilled,
  IconCircleXFilled,
  IconDeviceDesktop,
  IconDeviceLaptop,
  IconHourglassHigh,
} from "@tabler/icons-vue";
import { daemon, errorMessage } from "../daemon";
import { useActions } from "../composables/useActions";
import { useSettings } from "../composables/useSettings";
import { osLabel } from "../utils";

const { info, config, status } = useSettings();
const actions = useActions();
const router = useRouter();
const vars = useThemeVars();
const message = useMessage();
const { t } = useI18n();

// Heure actuelle, pour la durée de la connexion (page affichée seulement).
const now = ref(Date.now());
let clock: ReturnType<typeof setInterval> | undefined;
onActivated(() => {
  now.value = Date.now();
  clock = setInterval(() => (now.value = Date.now()), 15_000);
});
onDeactivated(() => clearInterval(clock));

const primary = computed(() => (config.value?.role ?? status.value?.role) !== "secondary");
const peers = computed(() => status.value?.peers ?? []);
const connected = computed(() => peers.value.length > 0);
const capitalize = (s: string) => s.charAt(0).toUpperCase() + s.slice(1);

/** État de la connexion. */
const conn = computed((): { icon: Component; color: string; title: string; help?: string } => {
  const s = status.value;
  if (!s) return { icon: IconHourglassHigh, color: vars.value.textColor3, title: t("home.starting") };
  if (connected.value) {
    const p = peers.value;
    const title = !primary.value
      ? t("home.connectedToPrimary", { peer: p[0].name })
      : p.length === 1
        ? t("home.connectedTo", { peer: p[0].name })
        : t("home.connectedMany", { n: p.length });
    return { icon: IconCircleCheckFilled, color: vars.value.successColor, title };
  }
  if (s.state === "error") {
    return { icon: IconCircleXFilled, color: vars.value.errorColor, title: t("home.noConnection"), help: capitalize(s.detail) };
  }
  // L'état détaillé vient de Baskul, dans la langue choisie.
  return {
    icon: IconHourglassHigh,
    color: vars.value.warningColor,
    title: s.detail ? capitalize(s.detail) : t("home.waiting"),
    help: primary.value ? t("home.waitingClientsHelp") : t("home.waitingPrimaryHelp"),
  };
});

const osIcon = (os?: string) => (os === "macos" ? IconDeviceLaptop : IconDeviceDesktop);

/** Cette machine puis les autres machines connectées. `go` : cible de
 * « Aller sur » (ou « Revenir ici »), quand le clavier et la souris n'y
 * sont pas. */
const machines = computed(() => {
  const s = status.value;
  const remote = !!s?.remote_active;
  const here = primary.value ? !remote : connected.value && remote;
  return [
    {
      key: "self",
      self: true,
      name: config.value?.name || info.value?.hostname || t("home.thisMachine"),
      label: t("home.thisMachine"),
      os: info.value?.os,
      since: undefined as number | undefined,
      active: here,
      go: !connected.value || here ? null : primary.value ? "primary" : "here",
      goLabel: primary.value ? t("home.returnHere") : t("home.comeHere"),
    },
    ...peers.value.map((p) => {
      const active = primary.value ? s?.active_peer === p.name : connected.value && !remote;
      return {
        key: `peer:${p.name}`,
        self: false,
        name: p.name,
        label: primary.value ? t("home.client") : t("home.primaryMachine"),
        os: p.os,
        since: p.since,
        active,
        go: primary.value && !active ? p.name : null,
        goLabel: t("home.goTo"),
      };
    }),
  ];
});
const activeStyle = computed(() => ({ boxShadow: `inset 0 0 0 1.5px ${vars.value.primaryColor}` }));
const switching = ref("");

async function go(to: string) {
  switching.value = to;
  try {
    await daemon.switchTo(to);
  } catch (e) {
    message.error(errorMessage(e));
  } finally {
    switching.value = "";
  }
}

const hotkeys = computed(() => {
  const h = config.value?.hotkeys;
  return h
    ? [
        { label: t("home.goToSecondary"), keys: h.switch_to_secondary },
        { label: t("home.returnHere"), keys: h.return_to_primary },
        { label: t("home.nextMachine"), keys: h.next_machine },
        { label: t("home.lockCursor"), keys: h.toggle_lock },
      ]
    : [];
});
const edges = computed(() => {
  const s = config.value?.screen;
  if (!s?.edge_switching) return t("home.edgesOff");
  const n = config.value?.layout.machines.length ?? 0;
  return n ? t("home.edgesGrid", { n }, n) : t("home.edgesOn");
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
  if (c.images && c.files) return t("home.clipboardAll");
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
          <n-text v-if="conn.help && !connected" depth="3">{{ conn.help }}</n-text>
        </n-flex>
      </n-flex>

      <n-grid cols="1 480:2 800:3" responsive="self" :x-gap="12" :y-gap="12" style="margin-top: 20px">
        <n-gi v-for="m in machines" :key="m.key">
          <n-card size="small" embedded :bordered="false" style="height: 100%" :style="m.active ? activeStyle : {}">
            <n-flex vertical align="center" :size="4">
              <n-icon :size="36" :color="m.active ? vars.primaryColor : undefined" :depth="m.active ? undefined : 3">
                <component :is="osIcon(m.os)" />
              </n-icon>
              <n-text strong>{{ m.name }}</n-text>
              <n-text depth="3" style="font-size: 13px">{{ m.label }}{{ m.os ? ` · ${osLabel(m.os)}` : "" }}</n-text>
              <n-text v-if="m.since" depth="3" style="font-size: 12px">
                <i18n-t keypath="home.since" tag="span">
                  <template #time><n-time :time="m.since" format="HH:mm" /></template>
                  <template #ago><n-time :time="m.since" :to="now" type="relative" /></template>
                </i18n-t>
              </n-text>
              <n-tag v-if="m.active" type="primary" size="small" round :bordered="false">
                {{ $t("home.keyboardMouse") }}
              </n-tag>
              <n-tag v-else-if="m.self && status?.locked" size="small" round :bordered="false">
                {{ $t("home.cursorLocked") }}
              </n-tag>
              <n-button v-if="m.go" size="small" secondary :loading="switching === m.go" @click="go(m.go)">
                {{ m.goLabel }}
              </n-button>
            </n-flex>
          </n-card>
        </n-gi>
        <n-gi v-if="!connected">
          <n-card size="small" embedded :bordered="false" style="height: 100%">
            <n-flex vertical align="center" justify="center" :size="4" style="height: 100%; min-height: 96px">
              <n-text depth="3">{{ primary ? $t("home.noClients") : $t("home.noPrimary") }}</n-text>
            </n-flex>
          </n-card>
        </n-gi>
      </n-grid>
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
                primary
                  ? $t("home.listensOn", { address: config.network.listen })
                  : config.network.connect
                    ? $t("home.connectsTo", { address: config.network.connect })
                    : $t("home.findsPrimary")
              }}
            </n-descriptions-item>
            <n-descriptions-item :label="$t('home.version')">{{ info.version }}</n-descriptions-item>
          </n-descriptions>
        </n-card>
      </n-gi>
      <n-gi>
        <n-card v-if="primary" :title="$t('home.switching')" size="small" style="height: 100%">
          <template #header-extra>
            <n-button text type="primary" @click="editSettings('baskul')">{{ $t("common.edit") }}</n-button>
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

