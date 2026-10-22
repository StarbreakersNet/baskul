<script setup lang="ts">
// Raccourci clavier : « Modifier » puis taper la combinaison. Le format est
// celui de la configuration, par exemple « Shift+F12 » ou « Ctrl+Alt+B ».
//
// Sur la machine principale, c'est le crochet clavier de Bascule qui relève
// la combinaison : elle ne déclenche alors aucun raccourci, et les touches que
// la fenêtre ne reçoit pas (F11, F12…) fonctionnent aussi. Sinon, la fenêtre
// la relève elle-même.
import { computed, onBeforeUnmount, ref } from "vue";
import { NButton, NInput, NInputGroup } from "naive-ui";
import { daemon } from "../daemon";

const model = defineModel<string>({ required: true });
const props = defineProps<{ os?: string }>();

const MODIFIERS = new Set([
  "ShiftLeft", "ShiftRight", "ControlLeft", "ControlRight", "AltLeft", "AltRight",
  "MetaLeft", "MetaRight", "OSLeft", "OSRight",
]);
const POLL_MS = 120;

const recording = ref(false);
const preview = ref<string[]>([]);
const root = ref<HTMLElement>();
const input = ref<InstanceType<typeof NInput>>();
/** Le daemon relève la combinaison. */
let daemonCapture = false;

const text = computed(() =>
  (recording.value ? preview.value : model.value ? model.value.split("+") : []).join(" + "),
);

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

/** Combinaison relevée ; Échap seule annule. */
function finish(hotkey: string) {
  if (hotkey !== "Escape") model.value = hotkey;
  stop();
}

function onKeyDown(e: KeyboardEvent) {
  e.preventDefault();
  e.stopPropagation();
  const mods = modifiers(e);
  if (MODIFIERS.has(e.code)) {
    preview.value = mods;
    return;
  }
  // La touche est arrivée jusqu'à la fenêtre : le daemon ne l'a pas relevée.
  if (e.code) finish([...mods, keyName(e.code)].join("+"));
}

function onKeyUp(e: KeyboardEvent) {
  if (MODIFIERS.has(e.code)) preview.value = modifiers(e);
}

/** Un clic ailleurs annule la saisie. */
function onPointerDown(e: PointerEvent) {
  if (!root.value?.contains(e.target as Node)) stop();
}

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

/** Attend la combinaison relevée par le daemon. */
async function pollDaemon() {
  while (recording.value && daemonCapture) {
    await sleep(POLL_MS);
    if (!recording.value || !daemonCapture) return;
    try {
      const s = await daemon.captureHotkey.poll();
      if (!recording.value || !daemonCapture) return;
      if (s.state === "done") return finish(s.hotkey);
      // Saisie abandonnée par le daemon (délai dépassé, redémarrage…).
      if (s.state === "idle") return stop();
    } catch {
      return stop();
    }
  }
}

async function start() {
  recording.value = true;
  preview.value = [];
  window.addEventListener("keydown", onKeyDown, true);
  window.addEventListener("keyup", onKeyUp, true);
  document.addEventListener("pointerdown", onPointerDown);
  input.value?.focus();
  try {
    await daemon.captureHotkey.start();
  } catch {
    // Pas de crochet clavier ici (machine pilotée, Linux…) : la fenêtre
    // relève seule la combinaison.
    return;
  }
  if (!recording.value) {
    void daemon.captureHotkey.cancel().catch(() => {});
    return;
  }
  daemonCapture = true;
  void pollDaemon();
}

function stop() {
  recording.value = false;
  preview.value = [];
  window.removeEventListener("keydown", onKeyDown, true);
  window.removeEventListener("keyup", onKeyUp, true);
  document.removeEventListener("pointerdown", onPointerDown);
  input.value?.blur();
  if (daemonCapture) {
    daemonCapture = false;
    void daemon.captureHotkey.cancel().catch(() => {});
  }
}

onBeforeUnmount(stop);
</script>

<template>
  <div ref="root">
    <n-input-group>
      <n-input
        ref="input"
        :value="text"
        readonly
        :placeholder="recording ? 'Appuyez sur la combinaison…' : 'Aucun'"
        :input-props="{ 'aria-live': 'polite' }"
        class="mono"
      />
      <n-button :type="recording ? 'primary' : 'default'" @click="recording ? stop() : start()">
        {{ recording ? "Annuler" : "Modifier" }}
      </n-button>
      <n-button
        title="Désactiver ce raccourci"
        aria-label="Désactiver ce raccourci"
        :disabled="!model || recording"
        @click="model = ''"
      >
        ✕
      </n-button>
    </n-input-group>
  </div>
</template>
