<script setup lang="ts">
import { computed, ref, type Component } from "vue";
import { useRouter } from "vue-router";
import { useI18n } from "vue-i18n";
import { useThemeVars } from "naive-ui";
import { IconCircleCheckFilled, IconCircleXFilled, IconHourglassHigh } from "@tabler/icons-vue";
import HAlert from "@/components/c/hAlert.vue";
import HButton from "@/components/c/hButton.vue";
import HCard from "@/components/c/hCard.vue";
import HDescriptions from "@/components/c/hDescriptions.vue";
import HDescriptionsItem from "@/components/c/hDescriptionsItem.vue";
import HEmpty from "@/components/c/hEmpty.vue";
import HFlex from "@/components/c/hFlex.vue";
import HGrid from "@/components/c/hGrid.vue";
import HGridItem from "@/components/c/hGridItem.vue";
import HIcon from "@/components/c/hIcon.vue";
import HTag from "@/components/c/hTag.vue";
import HSkeleton from "@/components/c/hSkeleton.vue";
import HText from "@/components/c/hText.vue";
import HTime from "@/components/c/hTime.vue";
import HTimeline from "@/components/c/hTimeline.vue";
import HTimelineItem from "@/components/c/hTimelineItem.vue";
import ExpandTransition from "@/components/ExpandTransition.vue";
import NetworkCard from "@/components/NetworkCard.vue";
import PairingCard from "@/components/PairingCard.vue";
import { useCurrentTime } from "@/composables/useCurrentTime";
import { useLoader } from "@/composables/useLoader";
import { useActivityStore } from "@/stores/activity";
import { useSettingsStore } from "@/stores/settings";
import { useStatusStore } from "@/stores/status";
import { describeActivity } from "@/utils/activity";
import { formatHotkey, hotkeyParts } from "@/utils/hotkey";
import { osIcon, osLabel } from "@/utils/os";
import { capitalize } from "@/utils/text";

interface ConnectionSummary {
  icon: Component;
  color: string;
  title: string;
  help?: string;
}

interface MachineEntry {
  key: string;
  name: string;
  subtitle: string;
  os?: string;
  since?: number;
  isSelf: boolean;
  isActive: boolean;
  /** Cible de "Aller sur" ou "Revenir ici", quand le clavier et la souris n'y sont pas */
  target: string | null;
  targetLabel: string;
}

const CLOCK_MS = 15_000;

const settings = useSettingsStore();
const statusStore = useStatusStore();
const activity = useActivityStore();
const router = useRouter();
const themeVars = useThemeVars();
const { t } = useI18n();
const now = useCurrentTime(CLOCK_MS);

const switching = useLoader();
const switchingTarget = ref("");

const activityItems = computed(() =>
  activity.events.map(event => ({ ...event, label: describeActivity(event) }))
);

const status = computed(() => statusStore.status);
const config = computed(() => settings.config);
const isPrimary = computed(() => (config.value?.role ?? status.value?.role) !== "secondary");
const peers = computed(() => status.value?.peers ?? []);
const isConnected = computed(() => peers.value.length > 0);

const connection = computed<ConnectionSummary>(() => {
  const current = status.value;
  if (!current) {
    return {
      icon: IconHourglassHigh,
      color: themeVars.value.textColor3,
      title: t("home.starting"),
    };
  }
  if (isConnected.value) {
    return {
      icon: IconCircleCheckFilled,
      color: themeVars.value.successColor,
      title: connectedTitle(),
    };
  }
  if (current.state === "error") {
    return {
      icon: IconCircleXFilled,
      color: themeVars.value.errorColor,
      title: t("home.noConnection"),
      help: capitalize(current.detail),
    };
  }
  return {
    icon: IconHourglassHigh,
    color: themeVars.value.warningColor,
    title: current.detail ? capitalize(current.detail) : t("home.waiting"),
    help: isPrimary.value ? t("home.waitingClientsHelp") : t("home.waitingPrimaryHelp"),
  };
});

const machines = computed<MachineEntry[]>(() => [selfEntry(), ...peers.value.map(peerEntry)]);

const activeTileStyle = computed(() => ({
  boxShadow: `inset 0 0 0 1.5px ${themeVars.value.primaryColor}`,
  background: `color-mix(in srgb, ${themeVars.value.primaryColor} 14%, transparent)`,
}));

const connectionText = computed(() => {
  const network = config.value?.network;
  if (isPrimary.value) {
    return t("home.listensOn", { address: network?.listen });
  }
  return network?.connect
    ? t("home.connectsTo", { address: network.connect })
    : t("home.findsPrimary");
});

const hotkeys = computed(() => {
  const configured = config.value?.hotkeys;
  return configured
    ? [
        { label: t("home.goToSecondary"), keys: configured.switch_to_secondary },
        { label: t("home.returnHere"), keys: configured.return_to_primary },
        { label: t("home.nextMachine"), keys: configured.next_machine },
        { label: t("home.lockCursor"), keys: configured.toggle_lock },
      ]
    : [];
});

const edgesText = computed(() => {
  const machineCount = config.value?.layout.machines.length ?? 0;
  if (!config.value?.screen.edge_switching) {
    return t("home.edgesOff");
  }
  return machineCount ? t("home.edgesGrid", { n: machineCount }, machineCount) : t("home.edgesOn");
});

const scrollText = computed(() => {
  const scroll = config.value?.scroll;
  if (scroll?.mode === "smooth") {
    return t("home.scrollSmooth");
  }
  if (scroll?.mode === "pixels") {
    return t("home.scrollPixels", { n: scroll.pixels_per_notch });
  }
  return scroll ? t("home.scrollLines", { n: scroll.lines_per_notch }) : "";
});

const clipboardText = computed(() => {
  const clipboard = config.value?.clipboard;
  if (!clipboard?.enabled) {
    return t("home.clipboardOff");
  }
  if (clipboard.images && clipboard.files) {
    return t("home.clipboardAll");
  }
  return clipboard.images ? t("home.clipboardImages") : t("home.clipboardText");
});

function connectedTitle() {
  const [first] = peers.value;
  if (!isPrimary.value) {
    return t("home.connectedToPrimary", { peer: first.name });
  }
  return peers.value.length === 1
    ? t("home.connectedTo", { peer: first.name })
    : t("home.connectedMany", { n: peers.value.length });
}

function machineSubtitle(label: string, os?: string) {
  return os ? `${label} · ${osLabel(os)}` : label;
}

function selfEntry(): MachineEntry {
  const isRemote = !!status.value?.remote_active;
  const isHere = isPrimary.value ? !isRemote : isConnected.value && isRemote;
  const os = settings.info?.os;
  let target: string | null = null;
  if (isConnected.value && !isHere) {
    target = isPrimary.value ? "primary" : "here";
  }
  return {
    key: "self",
    name: config.value?.name || settings.info?.hostname || t("home.thisMachine"),
    subtitle: machineSubtitle(t("home.thisMachine"), os),
    os,
    isSelf: true,
    isActive: isHere,
    target,
    targetLabel: isPrimary.value ? t("home.returnHere") : t("home.comeHere"),
  };
}

function peerEntry(peer: { name: string; os: string; since: number }): MachineEntry {
  const isRemote = !!status.value?.remote_active;
  const isActive = isPrimary.value
    ? status.value?.active_peer === peer.name
    : isConnected.value && !isRemote;
  const label = isPrimary.value ? t("home.client") : t("home.primaryMachine");
  return {
    key: `peer:${peer.name}`,
    name: peer.name,
    subtitle: machineSubtitle(label, peer.os),
    os: peer.os,
    since: peer.since,
    isSelf: false,
    isActive,
    target: isPrimary.value && !isActive ? peer.name : null,
    targetLabel: t("home.goTo"),
  };
}

async function goTo(target: string) {
  switchingTarget.value = target;
  await switching.run(() => statusStore.switchTo(target));
}

function editSettings(onglet?: string) {
  router.push({ name: "settings", query: onglet ? { onglet } : {} });
}

function openLog() {
  router.push({ name: "log" });
}
</script>

<template>
  <div>
    <expand-transition>
      <div v-if="status?.warning" class="home-notice">
        <h-alert type="warning">{{ status.warning }}</h-alert>
      </div>
    </expand-transition>
    <expand-transition>
      <div v-if="status?.edges_paused_by" class="home-notice">
        <h-alert :title="$t('home.edgesPausedTitle')" type="info">
          {{ $t("home.edgesPausedText", { program: status.edges_paused_by }) }}
        </h-alert>
      </div>
    </expand-transition>
    <h-flex :size="16" vertical>
      <h-card>
        <h-flex align="center" :size="16" :wrap="false">
          <h-icon :color="connection.color" :size="44"><component :is="connection.icon" /></h-icon>
          <h-flex :size="2" vertical>
            <h-text class="home-title" strong>{{ connection.title }}</h-text>
            <expand-transition>
              <h-text v-if="connection.help && !isConnected" depth="3" tag="div">
                {{ connection.help }}
              </h-text>
            </expand-transition>
          </h-flex>
        </h-flex>

        <transition-group class="home-machines" name="home-tile" tag="div">
          <h-card
            v-for="machine in machines"
            :key="machine.key"
            :bordered="false"
            class="home-tile"
            embedded
            size="small"
            :style="machine.isActive ? activeTileStyle : {}">
            <h-flex align="center" :size="4" vertical>
              <h-icon
                :color="machine.isActive ? themeVars.primaryColor : undefined"
                :depth="machine.isActive ? undefined : 3"
                :size="36">
                <component :is="osIcon(machine.os)" />
              </h-icon>
              <h-text strong>{{ machine.name }}</h-text>
              <h-text class="small-text" depth="3">{{ machine.subtitle }}</h-text>
              <h-text v-if="machine.since" class="home-since" depth="3">
                <i18n-t keypath="home.since" tag="span">
                  <template #time><h-time format="HH:mm" :time="machine.since" /></template>
                  <template #ago>
                    <h-time :time="machine.since" :to="now" type="relative" />
                  </template>
                </i18n-t>
              </h-text>
              <transition mode="out-in" name="fade">
                <h-tag v-if="machine.isActive" :bordered="false" round size="small" type="primary">
                  {{ $t("home.keyboardMouse") }}
                </h-tag>
                <h-tag
                  v-else-if="machine.isSelf && status?.locked"
                  :bordered="false"
                  round
                  size="small">
                  {{ $t("home.cursorLocked") }}
                </h-tag>
              </transition>
              <transition name="fade">
                <h-button
                  v-if="machine.target"
                  :loading="switching.loading && switchingTarget === machine.target"
                  secondary
                  size="small"
                  @click="goTo(machine.target)">
                  {{ machine.targetLabel }}
                </h-button>
              </transition>
            </h-flex>
          </h-card>
          <h-card
            v-if="!isConnected"
            key="empty"
            :bordered="false"
            class="home-tile"
            embedded
            size="small">
            <h-flex align="center" class="home-empty" justify="center" :size="4" vertical>
              <h-text depth="3">
                {{ isPrimary ? $t("home.noClients") : $t("home.noPrimary") }}
              </h-text>
            </h-flex>
          </h-card>
        </transition-group>
      </h-card>

      <transition mode="out-in" name="fade">
        <div v-if="settings.info && config">
          <h-flex :size="16" vertical>
            <h-grid cols="1 640:2" responsive="self" :x-gap="16" :y-gap="16">
              <h-grid-item>
                <h-card class="home-tile" size="small" :title="$t('home.thisMachine')">
                  <h-descriptions :column="1" label-placement="left" size="small">
                    <h-descriptions-item :label="$t('home.role')">
                      {{ isPrimary ? $t("home.rolePrimary") : $t("home.roleSecondary") }}
                    </h-descriptions-item>
                    <h-descriptions-item :label="$t('home.system')">
                      {{ osLabel(settings.info.os) }}
                    </h-descriptions-item>
                    <h-descriptions-item v-if="settings.info.address" :label="$t('home.ipAddress')">
                      <h-text code>{{ settings.info.address }}</h-text>
                    </h-descriptions-item>
                    <h-descriptions-item :label="$t('home.connection')">
                      {{ connectionText }}
                    </h-descriptions-item>
                    <h-descriptions-item :label="$t('home.version')">
                      {{ settings.info.version }}
                    </h-descriptions-item>
                  </h-descriptions>
                </h-card>
              </h-grid-item>
              <h-grid-item>
                <transition mode="out-in" name="fade">
                  <h-card
                    v-if="isPrimary"
                    class="home-tile"
                    size="small"
                    :title="$t('home.switching')">
                    <template #header-extra>
                      <h-button text type="primary" @click="editSettings('baskul')">
                        {{ $t("common.edit") }}
                      </h-button>
                    </template>
                    <h-descriptions :column="1" label-placement="left" size="small">
                      <h-descriptions-item
                        v-for="hotkey in hotkeys"
                        :key="hotkey.label"
                        :label="hotkey.label">
                        <h-tag v-if="hotkey.keys" class="mono" size="small">
                          {{ formatHotkey(hotkeyParts(hotkey.keys)) }}
                        </h-tag>
                        <h-text v-else depth="3">{{ $t("common.none") }}</h-text>
                      </h-descriptions-item>
                      <h-descriptions-item :label="$t('home.screenEdges')">
                        {{ edgesText }}
                      </h-descriptions-item>
                    </h-descriptions>
                  </h-card>
                  <h-card v-else class="home-tile" size="small" :title="$t('home.control')">
                    <template #header-extra>
                      <h-button text type="primary" @click="editSettings('souris')">
                        {{ $t("common.edit") }}
                      </h-button>
                    </template>
                    <h-descriptions :column="1" label-placement="left" size="small">
                      <h-descriptions-item :label="$t('home.mouseSpeed')">
                        × {{ config.mouse.speed }}
                      </h-descriptions-item>
                      <h-descriptions-item :label="$t('home.scrolling')">
                        {{ scrollText }}
                      </h-descriptions-item>
                      <h-descriptions-item :label="$t('home.clipboard')">
                        {{ clipboardText }}
                      </h-descriptions-item>
                    </h-descriptions>
                  </h-card>
                </transition>
              </h-grid-item>
            </h-grid>

            <h-grid cols="1 640:2" responsive="self" :x-gap="16" :y-gap="16">
              <h-grid-item>
                <h-card class="home-tile" size="small" :title="$t('home.activity')">
                  <template #header-extra>
                    <transition name="fade">
                      <h-button
                        v-if="activity.events.length"
                        text
                        type="primary"
                        @click="activity.clear()">
                        {{ $t("home.activityClear") }}
                      </h-button>
                    </transition>
                  </template>
                  <transition mode="out-in" name="fade">
                    <h-timeline v-if="activity.events.length" class="home-activity">
                      <transition-group name="home-event">
                        <h-timeline-item
                          v-for="item in activityItems"
                          :key="item.id"
                          :type="item.label.color">
                          <template #header>
                            {{ $t(`home.${item.label.key}`, item.label.params) }}
                          </template>
                          <template #footer>
                            <h-time :time="item.at" :to="Math.max(now, item.at)" type="relative" />
                          </template>
                        </h-timeline-item>
                      </transition-group>
                    </h-timeline>
                    <h-empty
                      v-else
                      class="home-activity-empty"
                      :description="$t('home.activityEmpty')" />
                  </transition>
                </h-card>
              </h-grid-item>
              <h-grid-item>
                <pairing-card class="home-tile" :is-primary="isPrimary" />
              </h-grid-item>
            </h-grid>
          </h-flex>
          <network-card />
        </div>
        <h-flex v-else :size="16" vertical>
          <h-grid cols="1 640:2" responsive="self" :x-gap="16" :y-gap="16">
            <h-grid-item v-for="index in 4" :key="index">
              <h-skeleton :height="200" :sharp="false" />
            </h-grid-item>
          </h-grid>
        </h-flex>
      </transition>

      <h-flex :size="8">
        <h-button @click="editSettings()">{{ $t("home.settings") }}</h-button>
        <h-button @click="openLog()">{{ $t("home.log") }}</h-button>
        <h-button @click="statusStore.restart()">{{ $t("common.restart") }}</h-button>
      </h-flex>
    </h-flex>
  </div>
</template>

<style lang="scss" scoped>
.home-title {
  font-size: 18px;
}

// Marge en padding : elle grandit avec l'alerte
.home-notice {
  padding-bottom: 16px;
}

.home-machines {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
  gap: 12px;
  margin-top: 20px;
}

.home-tile-enter-active,
.home-tile-leave-active {
  transition:
    opacity 0.25s ease,
    transform 0.25s ease;
}

.home-tile-enter-from,
.home-tile-leave-to {
  opacity: 0;
  transform: scale(0.96);
}

.home-tile-move {
  transition: transform 0.25s ease;
}

.home-tile {
  height: 100%;
}

.home-since {
  font-size: 12px;
}

.home-activity {
  max-height: 260px;
  overflow-y: auto;
  padding: 4px 4px 0 2px;
}

.home-event-enter-active {
  transition:
    opacity 0.3s ease,
    transform 0.3s ease;
}

.home-event-enter-from {
  opacity: 0;
  transform: translateY(-6px);
}

.home-activity-empty {
  padding: 16px 0;
}

.home-empty {
  height: 100%;
  min-height: 96px;
}
</style>
