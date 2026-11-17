<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import { useThemeVars } from "naive-ui";
import { IconHome } from "@tabler/icons-vue";
import HAlert from "@/components/c/hAlert.vue";
import HButton from "@/components/c/hButton.vue";
import HFlex from "@/components/c/hFlex.vue";
import HIcon from "@/components/c/hIcon.vue";
import HText from "@/components/c/hText.vue";
import HotkeyInput from "@/components/HotkeyInput.vue";
import SettingRow from "@/components/SettingRow.vue";
import { gridCells, gridColumns, isolatedMachines, type GridCell } from "@/utils/layoutGrid";
import { osIcon, osLabel } from "@/utils/os";
import type { LayoutMachine } from "@/types/Config";
import type { Peer } from "@/types/Daemon";

interface Drag {
  name: string;
  x: number;
  y: number;
  startX: number;
  startY: number;
  hasMoved: boolean;
}

const machines = defineModel<LayoutMachine[]>({ required: true });

const { primaryName, os, peers } = defineProps<{
  primaryName: string;
  os: string;
  peers: Peer[];
}>();

// Au-delà, le geste est un glisser et non un clic
const DRAG_THRESHOLD_PX = 4;

const themeVars = useThemeVars();
const { t } = useI18n();

const selectedName = ref<string | null>(null);
const drag = ref<Drag | null>(null);
const hoveredCell = ref<string | null>(null);

const colors = computed(() => ({
  border: themeVars.value.borderColor,
  muted: themeVars.value.textColor3,
  accent: themeVars.value.primaryColor,
  tile: themeVars.value.cardColor,
  success: themeVars.value.successColor,
  screen: themeVars.value.dividerColor,
}));

const unplacedPeers = computed(() =>
  peers.filter(peer => !machines.value.some(machine => machine.name === peer.name))
);
const cells = computed(() => gridCells(machines.value));
const columnsStyle = computed(() => ({
  gridTemplateColumns: `repeat(${gridColumns(machines.value)}, minmax(84px, 128px))`,
}));
const isolated = computed(() => isolatedMachines(machines.value));
const selectedMachine = computed(() =>
  machines.value.find(machine => machine.name === selectedName.value)
);
const selectedHelp = computed(() => {
  const peer = selectedMachine.value && peerOf(selectedMachine.value.name);
  return peer ? osLabel(peer.os) : t("settings.layout.offlineHelp");
});
const hotkey = computed({
  get: () => selectedMachine.value?.hotkey ?? "",
  set: (value: string) => {
    machines.value = machines.value.map(machine =>
      machine.name === selectedName.value ? withHotkey(machine, value) : machine
    );
  },
});

function peerOf(name: string) {
  return peers.find(peer => peer.name === name);
}

function withHotkey(machine: LayoutMachine, value: string): LayoutMachine {
  const { name, x, y } = machine;
  return value ? { name, x, y, hotkey: value } : { name, x, y };
}

/** Une pilotée déjà en (x, y) prend la place de celle qu'on déplace */
function swapInto(moving: LayoutMachine, x: number, y: number) {
  return machines.value.map(machine => {
    if (machine.name === moving.name) {
      return { ...machine, x, y };
    }
    return machine.x === x && machine.y === y ? { ...machine, x: moving.x, y: moving.y } : machine;
  });
}

function place(name: string, x: number, y: number) {
  const moving = machines.value.find(machine => machine.name === name);
  const isOccupied = machines.value.some(machine => machine.x === x && machine.y === y);
  const isPrimaryCell = x === 0 && y === 0;
  if (!isPrimaryCell && (moving || !isOccupied)) {
    machines.value = moving ? swapInto(moving, x, y) : [...machines.value, { name, x, y }];
    selectedName.value = name;
  }
}

function remove(name: string) {
  machines.value = machines.value.filter(machine => machine.name !== name);
  selectedName.value = null;
}

function toggleSelection(name: string) {
  selectedName.value = selectedName.value === name ? null : name;
}

function cellAt(x: number, y: number) {
  const element = document.elementFromPoint(x, y)?.closest<HTMLElement>("[data-cell]");
  return element?.dataset.cell ?? null;
}

function handlePointerDown(event: PointerEvent, name: string) {
  if (event.button === 0) {
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
    const { clientX: x, clientY: y } = event;
    drag.value = { name, x, y, startX: x, startY: y, hasMoved: false };
  }
}

function handlePointerMove(event: PointerEvent) {
  const current = drag.value;
  if (current) {
    current.x = event.clientX;
    current.y = event.clientY;
    current.hasMoved ||=
      Math.hypot(current.x - current.startX, current.y - current.startY) > DRAG_THRESHOLD_PX;
    hoveredCell.value = current.hasMoved ? cellAt(current.x, current.y) : null;
  }
}

function handlePointerUp() {
  const current = drag.value;
  const target = hoveredCell.value;
  drag.value = null;
  hoveredCell.value = null;
  if (current && !current.hasMoved) {
    toggleSelection(current.name);
  } else if (current && target) {
    const [x, y] = target.split(",").map(Number);
    place(current.name, x, y);
  }
}

function handleCellClick(cell: GridCell) {
  if (selectedName.value && !cell.isPrimary && !cell.machine) {
    place(selectedName.value, cell.x, cell.y);
  }
}

function cellClasses(cell: GridCell) {
  const isFree = !cell.machine && !cell.isPrimary;
  return {
    "layout-cell--empty": isFree,
    "layout-cell--target": hoveredCell.value === cell.key && !cell.isPrimary,
    "layout-cell--pick": !!selectedName.value && isFree,
  };
}

function tileClasses(name: string) {
  return {
    "layout-tile--on": selectedName.value === name,
    "layout-tile--dragging": drag.value?.hasMoved && drag.value.name === name,
  };
}
</script>

<template>
  <div>
    <div class="layout-grid" role="grid" :style="columnsStyle">
      <div
        v-for="cell in cells"
        :key="cell.key"
        :class="['layout-cell', cellClasses(cell)]"
        :data-cell="cell.key"
        @click="handleCellClick(cell)">
        <div v-if="cell.isPrimary" class="layout-tile layout-tile--primary">
          <h-icon :size="20"><icon-home /></h-icon>
          <span class="layout-name">{{ primaryName }}</span>
          <span class="layout-sub">{{ $t("settings.layout.primary") }}</span>
        </div>
        <div
          v-else-if="cell.machine"
          :aria-pressed="selectedName === cell.machine.name"
          :class="['layout-tile', tileClasses(cell.machine.name)]"
          role="button"
          tabindex="0"
          @keydown.enter.prevent="toggleSelection(cell.machine.name)"
          @pointercancel="drag = null"
          @pointerdown="handlePointerDown($event, cell.machine.name)"
          @pointermove="handlePointerMove($event)"
          @pointerup="handlePointerUp()">
          <h-icon :depth="peerOf(cell.machine.name) ? undefined : 3" :size="20">
            <component :is="osIcon(peerOf(cell.machine.name)?.os)" />
          </h-icon>
          <span class="layout-name">{{ cell.machine.name }}</span>
          <span class="layout-sub">
            <span :class="['layout-dot', { 'layout-dot--live': !!peerOf(cell.machine.name) }]" />
            {{
              peerOf(cell.machine.name)
                ? $t("settings.layout.connected")
                : $t("settings.layout.offline")
            }}
          </span>
        </div>
      </div>
    </div>

    <h-flex v-if="unplacedPeers.length" align="center" class="layout-block" :size="8">
      <h-text class="small-text" depth="3">{{ $t("settings.layout.unplaced") }}</h-text>
      <div
        v-for="peer in unplacedPeers"
        :key="peer.name"
        :class="['layout-tile', 'layout-tile--chip', tileClasses(peer.name)]"
        role="button"
        tabindex="0"
        @pointercancel="drag = null"
        @pointerdown="handlePointerDown($event, peer.name)"
        @pointermove="handlePointerMove($event)"
        @pointerup="handlePointerUp()">
        <h-icon :size="16"><component :is="osIcon(peer.os)" /></h-icon>
        <span class="layout-name">{{ peer.name }}</span>
      </div>
    </h-flex>

    <h-alert v-if="isolated.length" class="layout-block" type="warning">
      {{ $t("settings.layout.isolated", { names: isolated.join(", ") }, isolated.length) }}
    </h-alert>

    <h-text class="layout-hint" depth="3" tag="div">
      {{ selectedName ? $t("settings.layout.pickHint") : $t("settings.layout.dragHint") }}
    </h-text>

    <div v-if="selectedMachine" class="layout-details">
      <setting-row :help="selectedHelp" :label="selectedMachine.name">
        <h-button size="small" @click="remove(selectedMachine.name)">
          {{ $t("settings.layout.remove") }}
        </h-button>
      </setting-row>
      <setting-row :help="$t('settings.layout.hotkeyHelp')" :label="$t('settings.layout.hotkey')">
        <hotkey-input v-model="hotkey" :os="os" />
      </setting-row>
    </div>

    <div
      v-if="drag?.hasMoved"
      class="layout-tile layout-tile--ghost"
      :style="{ left: `${drag.x}px`, top: `${drag.y}px` }">
      <span class="layout-name">{{ drag.name }}</span>
    </div>
  </div>
</template>

<style lang="scss" scoped>
@use "@/assets/variables";

.layout-grid {
  display: grid;
  gap: 6px;
  justify-content: start;
  user-select: none;
}

.layout-cell {
  height: 72px;
  border-radius: variables.$radius;
}

.layout-cell--empty {
  border: 1px dashed v-bind("colors.border");
}

.layout-cell--pick {
  cursor: pointer;
}

.layout-cell--target {
  border: 1.5px solid v-bind("colors.accent");
  background: color-mix(in srgb, v-bind("colors.accent") variables.$accent-tint, transparent);
}

.layout-tile {
  height: 100%;
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 2px;
  padding: 6px;
  border-radius: variables.$radius;
  border: 1.5px solid v-bind("colors.border");
  background: v-bind("colors.tile");
  cursor: grab;
  touch-action: none;
  overflow: hidden;
}

.layout-tile--primary {
  cursor: default;
  background: v-bind("colors.screen");
  border-color: transparent;
}

.layout-tile--on {
  border-color: v-bind("colors.accent");
  background: color-mix(in srgb, v-bind("colors.accent") variables.$accent-tint, transparent);
}

.layout-tile--dragging {
  opacity: 0.4;
}

.layout-tile--chip {
  height: auto;
  flex-direction: row;
  padding: 4px 10px;
  border-radius: 999px;
}

.layout-tile--ghost {
  position: fixed;
  height: auto;
  padding: 8px 14px;
  transform: translate(-50%, -50%);
  pointer-events: none;
  border-color: v-bind("colors.accent");
  box-shadow: 0 6px 18px rgba(0, 0, 0, 0.18);
  z-index: 10;
}

.layout-name {
  font-weight: 600;
  font-size: variables.$font-size-help;
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.layout-sub {
  font-size: 11.5px;
  color: v-bind("colors.muted");
  display: flex;
  align-items: center;
  gap: 4px;
}

.layout-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: v-bind("colors.border");
}

.layout-dot--live {
  background: v-bind("colors.success");
}

.layout-block {
  margin-top: 10px;
}

.layout-hint {
  margin-top: 8px;
  font-size: variables.$font-size-help;
}

.layout-details {
  margin-top: 6px;
}
</style>
