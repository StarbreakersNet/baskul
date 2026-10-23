<script setup lang="ts">
// Grille des machines : la principale au centre (case 0, 0), les pilotées
// autour. On passe d'un écran à l'autre par leur bord commun. Une pilotée se
// déplace en la faisant glisser sur une case, ou en la sélectionnant puis en
// cliquant sur la case voulue ; deux pilotées échangent leurs places.
import { computed, ref } from "vue";
import { NAlert, NButton, NFlex, NIcon, NText, useThemeVars } from "naive-ui";
import { IconDeviceDesktop, IconDeviceLaptop, IconHome } from "@tabler/icons-vue";
import HotkeyInput from "./HotkeyInput.vue";
import SettingRow from "./SettingRow.vue";
import type { LayoutMachine, Peer } from "../types";
import { osLabel } from "../utils";

const machines = defineModel<LayoutMachine[]>({ required: true });
const props = defineProps<{ primaryName: string; os: string; peers: Peer[] }>();

const vars = useThemeVars();
const colors = computed(() => ({
  border: vars.value.borderColor,
  muted: vars.value.textColor3,
  accent: vars.value.primaryColor,
  soft: `color-mix(in srgb, ${vars.value.primaryColor} 14%, transparent)`,
  tile: vars.value.cardColor,
  success: vars.value.successColor,
  screen: vars.value.dividerColor,
}));

const selected = ref<string | null>(null);
/** Glisser en cours : pilotée, position du pointeur, case survolée. */
const drag = ref<{ name: string; x: number; y: number; sx: number; sy: number; moved: boolean } | null>(null);
const hover = ref<string | null>(null);

const peerOf = (name: string) => props.peers.find((p) => p.name === name);
const osIcon = (os?: string) => (os === "macos" ? IconDeviceLaptop : IconDeviceDesktop);

/** Pilotées connectées mais pas encore sur la grille. */
const unplaced = computed(() => props.peers.filter((p) => !machines.value.some((m) => m.name === p.name)));

/** Cases affichées : les machines placées et une rangée libre tout autour. */
const bounds = computed(() => {
  const xs = [0, ...machines.value.map((m) => m.x)];
  const ys = [0, ...machines.value.map((m) => m.y)];
  return { x0: Math.min(...xs) - 1, x1: Math.max(...xs) + 1, y0: Math.min(...ys) - 1, y1: Math.max(...ys) + 1 };
});
const cells = computed(() => {
  const b = bounds.value;
  const out = [];
  for (let y = b.y0; y <= b.y1; y++) {
    for (let x = b.x0; x <= b.x1; x++) {
      out.push({ x, y, key: `${x},${y}`, primary: x === 0 && y === 0, machine: machines.value.find((m) => m.x === x && m.y === y) });
    }
  }
  return out;
});
const columns = computed(() => bounds.value.x1 - bounds.value.x0 + 1);

/** Pilotées qu'aucun bord ne relie à la principale : seul un raccourci y mène. */
const isolated = computed(() => {
  const at = new Map(machines.value.map((m) => [`${m.x},${m.y}`, m.name]));
  const seen = new Set(["0,0"]);
  const queue = [[0, 0]];
  while (queue.length) {
    const [x, y] = queue.shift()!;
    for (const [dx, dy] of [[1, 0], [-1, 0], [0, 1], [0, -1]]) {
      const key = `${x + dx},${y + dy}`;
      if (!seen.has(key) && at.has(key)) {
        seen.add(key);
        queue.push([x + dx, y + dy]);
      }
    }
  }
  return machines.value.filter((m) => !seen.has(`${m.x},${m.y}`)).map((m) => m.name);
});

/** Place `name` en (x, y) ; une pilotée déjà là prend son ancienne place. */
function place(name: string, x: number, y: number) {
  if (x === 0 && y === 0) return;
  const list = machines.value.map((m) => ({ ...m }));
  const moving = list.find((m) => m.name === name);
  const other = list.find((m) => m.x === x && m.y === y);
  if (moving) {
    if (other && other !== moving) Object.assign(other, { x: moving.x, y: moving.y });
    Object.assign(moving, { x, y });
  } else if (!other) {
    list.push({ name, x, y });
  } else {
    return;
  }
  machines.value = list;
  selected.value = name;
}

function remove(name: string) {
  machines.value = machines.value.filter((m) => m.name !== name);
  selected.value = null;
}

const current = computed(() => machines.value.find((m) => m.name === selected.value));
const hotkey = computed({
  get: () => current.value?.hotkey ?? "",
  set: (v: string) => {
    machines.value = machines.value.map((m) => {
      if (m.name !== selected.value) return m;
      const { hotkey: _, ...rest } = m;
      return v ? { ...rest, hotkey: v } : rest;
    });
  },
});

function cellAt(x: number, y: number): string | null {
  const el = document.elementFromPoint(x, y)?.closest<HTMLElement>("[data-cell]");
  return el?.dataset.cell ?? null;
}

function onPointerDown(e: PointerEvent, name: string) {
  if (e.button !== 0) return;
  (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  drag.value = { name, x: e.clientX, y: e.clientY, sx: e.clientX, sy: e.clientY, moved: false };
}

function onPointerMove(e: PointerEvent) {
  const d = drag.value;
  if (!d) return;
  d.x = e.clientX;
  d.y = e.clientY;
  if (!d.moved && Math.hypot(d.x - d.sx, d.y - d.sy) > 4) d.moved = true;
  if (d.moved) hover.value = cellAt(d.x, d.y);
}

function onPointerUp() {
  const d = drag.value;
  const target = hover.value;
  drag.value = null;
  hover.value = null;
  if (!d) return;
  if (!d.moved) {
    // Simple clic : sélection.
    selected.value = selected.value === d.name ? null : d.name;
    return;
  }
  if (target) {
    const [x, y] = target.split(",").map(Number);
    place(d.name, x, y);
  }
}

function onCellClick(cell: { x: number; y: number; primary: boolean; machine?: LayoutMachine }) {
  if (selected.value && !cell.primary && !cell.machine) place(selected.value, cell.x, cell.y);
}
</script>

<template>
  <div>
    <div class="grid" :style="{ gridTemplateColumns: `repeat(${columns}, minmax(84px, 128px))` }" role="grid">
      <div
        v-for="cell in cells"
        :key="cell.key"
        :data-cell="cell.key"
        :class="['cell', { empty: !cell.machine && !cell.primary, target: hover === cell.key && !cell.primary, pick: !!selected && !cell.machine && !cell.primary }]"
        @click="onCellClick(cell)"
      >
        <div v-if="cell.primary" class="tile primary">
          <n-icon :size="20"><icon-home /></n-icon>
          <span class="name">{{ primaryName }}</span>
          <span class="sub">{{ $t("settings.layout.primary") }}</span>
        </div>
        <div
          v-else-if="cell.machine"
          :class="['tile', { on: selected === cell.machine.name, dragging: drag?.moved && drag.name === cell.machine.name }]"
          role="button"
          tabindex="0"
          :aria-pressed="selected === cell.machine.name"
          @pointerdown="onPointerDown($event, cell.machine.name)"
          @pointermove="onPointerMove"
          @pointerup="onPointerUp"
          @pointercancel="drag = null"
          @keydown.enter.prevent="selected = selected === cell.machine.name ? null : cell.machine.name"
        >
          <n-icon :size="20" :depth="peerOf(cell.machine.name) ? undefined : 3">
            <component :is="osIcon(peerOf(cell.machine.name)?.os)" />
          </n-icon>
          <span class="name">{{ cell.machine.name }}</span>
          <span class="sub">
            <span :class="['dot', { live: !!peerOf(cell.machine.name) }]" />
            {{ peerOf(cell.machine.name) ? $t("settings.layout.connected") : $t("settings.layout.offline") }}
          </span>
        </div>
      </div>
    </div>

    <n-flex v-if="unplaced.length" align="center" :size="8" style="margin-top: 10px">
      <n-text depth="3" style="font-size: 13px">{{ $t("settings.layout.unplaced") }}</n-text>
      <div
        v-for="p in unplaced"
        :key="p.name"
        :class="['tile', 'chip', { on: selected === p.name }]"
        role="button"
        tabindex="0"
        @pointerdown="onPointerDown($event, p.name)"
        @pointermove="onPointerMove"
        @pointerup="onPointerUp"
        @pointercancel="drag = null"
      >
        <n-icon :size="16"><component :is="osIcon(p.os)" /></n-icon>
        <span class="name">{{ p.name }}</span>
      </div>
    </n-flex>

    <n-alert v-if="isolated.length" type="warning" style="margin-top: 10px">
      {{ $t("settings.layout.isolated", { names: isolated.join(", ") }, isolated.length) }}
    </n-alert>

    <n-text v-if="selected" depth="3" tag="div" class="hint">{{ $t("settings.layout.pickHint") }}</n-text>
    <n-text v-else depth="3" tag="div" class="hint">{{ $t("settings.layout.dragHint") }}</n-text>

    <div v-if="current" class="details">
      <setting-row
        :label="current.name"
        :help="peerOf(current.name) ? osLabel(peerOf(current.name)!.os) : $t('settings.layout.offlineHelp')"
      >
        <n-button size="small" @click="remove(current.name)">{{ $t("settings.layout.remove") }}</n-button>
      </setting-row>
      <setting-row :label="$t('settings.layout.hotkey')" :help="$t('settings.layout.hotkeyHelp')">
        <hotkey-input v-model="hotkey" :os="os" />
      </setting-row>
    </div>

    <!-- Pilotée qui suit le pointeur pendant le glisser. -->
    <div v-if="drag?.moved" class="tile ghost" :style="{ left: `${drag.x}px`, top: `${drag.y}px` }">
      <span class="name">{{ drag.name }}</span>
    </div>
  </div>
</template>

<style scoped>
.grid {
  display: grid;
  gap: 6px;
  justify-content: start;
  user-select: none;
}
.cell {
  height: 72px;
  border-radius: 9px;
}
.cell.empty {
  border: 1px dashed v-bind("colors.border");
}
.cell.pick {
  cursor: pointer;
}
.cell.target {
  border: 1.5px solid v-bind("colors.accent");
  background: v-bind("colors.soft");
}
.tile {
  height: 100%;
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 2px;
  padding: 6px;
  border-radius: 9px;
  border: 1.5px solid v-bind("colors.border");
  background: v-bind("colors.tile");
  cursor: grab;
  touch-action: none;
  overflow: hidden;
}
.tile.primary {
  cursor: default;
  background: v-bind("colors.screen");
  border-color: transparent;
}
.tile.on {
  border-color: v-bind("colors.accent");
  background: v-bind("colors.soft");
}
.tile.dragging {
  opacity: 0.4;
}
.tile.chip {
  height: auto;
  flex-direction: row;
  padding: 4px 10px;
  border-radius: 999px;
}
.tile.ghost {
  position: fixed;
  height: auto;
  padding: 8px 14px;
  transform: translate(-50%, -50%);
  pointer-events: none;
  border-color: v-bind("colors.accent");
  box-shadow: 0 6px 18px rgba(0, 0, 0, 0.18);
  z-index: 10;
}
.name {
  font-weight: 600;
  font-size: 13px;
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.sub {
  font-size: 11.5px;
  color: v-bind("colors.muted");
  display: flex;
  align-items: center;
  gap: 4px;
}
.dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: v-bind("colors.border");
}
.dot.live {
  background: v-bind("colors.success");
}
.hint {
  margin-top: 8px;
  font-size: 13px;
}
.details {
  margin-top: 6px;
}
</style>
