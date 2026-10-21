<script setup lang="ts">
// Raccourci clavier : « Modifier » puis taper la combinaison. Le format est
// celui de la configuration, par exemple « Shift+F12 » ou « Ctrl+Alt+B ».
import { computed, onBeforeUnmount, ref } from "vue";
import { NButton } from "naive-ui";

const model = defineModel<string>({ required: true });
const props = defineProps<{ os?: string }>();

const MODIFIERS = new Set([
  "ShiftLeft", "ShiftRight", "ControlLeft", "ControlRight", "AltLeft", "AltRight",
  "MetaLeft", "MetaRight", "OSLeft", "OSRight",
]);

const recording = ref(false);
const preview = ref<string[]>([]);
const root = ref<HTMLElement>();

const keys = computed(() => (recording.value ? preview.value : model.value ? model.value.split("+") : []));

function modifiers(e: KeyboardEvent): string[] {
  const m: string[] = [];
  if (e.ctrlKey) m.push("Ctrl");
  if (e.altKey) m.push("Alt");
  if (e.shiftKey) m.push("Shift");
  if (e.metaKey) m.push(props.os === "macos" ? "Cmd" : "Win");
  return m;
}

/** Nom Bascule d'une touche physique (codes `KeyboardEvent.code`). */
function keyName(code: string): string {
  return /^Key([A-Z])$/.exec(code)?.[1] ?? /^Digit(\d)$/.exec(code)?.[1] ?? code;
}

function onKeyDown(e: KeyboardEvent) {
  e.preventDefault();
  e.stopPropagation();
  const mods = modifiers(e);
  if (e.code === "Escape" && mods.length === 0) return stop();
  if (MODIFIERS.has(e.code)) {
    preview.value = mods;
    return;
  }
  if (!e.code) return;
  model.value = [...mods, keyName(e.code)].join("+");
  stop();
}

function onKeyUp(e: KeyboardEvent) {
  if (MODIFIERS.has(e.code)) preview.value = modifiers(e);
}

function onPointerDown(e: PointerEvent) {
  if (!root.value?.contains(e.target as Node)) stop();
}

function start() {
  recording.value = true;
  preview.value = [];
  window.addEventListener("keydown", onKeyDown, true);
  window.addEventListener("keyup", onKeyUp, true);
  document.addEventListener("pointerdown", onPointerDown);
}

function stop() {
  recording.value = false;
  window.removeEventListener("keydown", onKeyDown, true);
  window.removeEventListener("keyup", onKeyUp, true);
  document.removeEventListener("pointerdown", onPointerDown);
}

onBeforeUnmount(stop);
</script>

<template>
  <div ref="root" class="hotkey">
    <span class="keys" :class="{ recording }" aria-live="polite">
      <template v-if="keys.length">
        <kbd v-for="(k, i) in keys" :key="i">{{ k }}</kbd>
      </template>
      <span v-else class="none">{{ recording ? "Appuyez sur la combinaison…" : "Aucun" }}</span>
    </span>
    <n-button size="small" @click="recording ? stop() : start()">{{ recording ? "Annuler" : "Modifier" }}</n-button>
    <n-button
      size="small"
      quaternary
      title="Désactiver ce raccourci"
      aria-label="Désactiver ce raccourci"
      :disabled="!model"
      @click="
        stop();
        model = '';
      "
    >
      ✕
    </n-button>
  </div>
</template>

<style scoped>
.hotkey {
  display: flex;
  gap: 6px;
  align-items: center;
  flex-wrap: wrap;
}
.keys {
  display: inline-flex;
  gap: 4px;
  align-items: center;
  min-height: 34px;
  min-width: 120px;
  padding: 3px 8px;
  border: 1px solid var(--border);
  border-radius: 8px;
}
.keys.recording {
  border-color: var(--accent);
  box-shadow: 0 0 0 3px var(--accent-soft);
}
.none {
  color: var(--muted);
  font-size: 13.5px;
}
kbd {
  font: 600 12.5px/1 ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  padding: 5px 7px;
  border-radius: 6px;
  border: 1px solid var(--border);
  border-bottom-width: 2px;
}
</style>
