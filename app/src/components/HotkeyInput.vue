<script setup lang="ts">
import { computed, onBeforeUnmount, onDeactivated, ref, useTemplateRef } from "vue";
import HButton from "@/components/c/hButton.vue";
import HInput from "@/components/c/hInput.vue";
import HInputGroup from "@/components/c/hInputGroup.vue";
import { useHotkeyCaptureStore } from "@/stores/hotkeyCapture";
import {
  formatHotkey,
  hotkeyParts,
  isModifierKey,
  keyName,
  pressedModifiers,
} from "@/utils/hotkey";
import type { HInputInst } from "@/types/components/c/hInput";
import type { CaptureState } from "@/types/Hotkey";

const model = defineModel<string>({ required: true });

const { os } = defineProps<{
  os?: string;
}>();

const POLL_MS = 120;

const hotkeyCapture = useHotkeyCaptureStore();

const isRecording = ref(false);
const isDaemonCapturing = ref(false);
const preview = ref<string[]>([]);
const rootRef = useTemplateRef<HTMLElement>("rootRef");
const inputRef = useTemplateRef<HInputInst>("inputRef");

const displayText = computed(() =>
  formatHotkey(isRecording.value ? preview.value : hotkeyParts(model.value))
);

function handleKeyDown(event: KeyboardEvent) {
  event.preventDefault();
  event.stopPropagation();
  const modifiers = pressedModifiers(event, os);
  if (isModifierKey(event)) {
    preview.value = modifiers;
  } else if (event.code) {
    // Touche arrivée jusqu'à la fenêtre : le crochet de Baskul ne l'a pas relevée
    finish([...modifiers, keyName(event.code)].join("+"));
  }
}

function handleKeyUp(event: KeyboardEvent) {
  if (isModifierKey(event)) {
    preview.value = pressedModifiers(event, os);
  }
}

function handlePointerDown(event: PointerEvent) {
  if (!rootRef.value?.contains(event.target as Node)) {
    stop();
  }
}

/** Échap seule annule la saisie */
function finish(hotkey: string) {
  if (hotkey !== "Escape") {
    model.value = hotkey;
  }
  stop();
}

function isListeningToDaemon() {
  return isRecording.value && isDaemonCapturing.value;
}

/** Renvoie `true` tant que Baskul attend la combinaison */
function applyCaptureState(state: CaptureState | null) {
  let isWaiting = false;
  if (state?.state === "done") {
    finish(state.hotkey);
  } else if (state?.state === "waiting") {
    isWaiting = true;
  } else {
    stop();
  }
  return isWaiting;
}

async function waitForDaemonHotkey() {
  let isWaiting = true;
  while (isWaiting && isListeningToDaemon()) {
    await new Promise(resolve => setTimeout(resolve, POLL_MS));
    if (isListeningToDaemon()) {
      const state = await hotkeyCapture.poll();
      isWaiting = isListeningToDaemon() && applyCaptureState(state);
    }
  }
}

function listenToKeyboard() {
  window.addEventListener("keydown", handleKeyDown, true);
  window.addEventListener("keyup", handleKeyUp, true);
  document.addEventListener("pointerdown", handlePointerDown);
}

function stopListening() {
  window.removeEventListener("keydown", handleKeyDown, true);
  window.removeEventListener("keyup", handleKeyUp, true);
  document.removeEventListener("pointerdown", handlePointerDown);
}

async function start() {
  isRecording.value = true;
  preview.value = [];
  listenToKeyboard();
  inputRef.value?.focus();
  const hasDaemonCapture = await hotkeyCapture.start();
  if (hasDaemonCapture && isRecording.value) {
    isDaemonCapturing.value = true;
    void waitForDaemonHotkey();
  } else if (hasDaemonCapture) {
    void hotkeyCapture.cancel();
  }
}

function stop() {
  isRecording.value = false;
  preview.value = [];
  stopListening();
  inputRef.value?.blur();
  if (isDaemonCapturing.value) {
    isDaemonCapturing.value = false;
    void hotkeyCapture.cancel();
  }
}

function toggleRecording() {
  if (isRecording.value) {
    stop();
  } else {
    void start();
  }
}

onBeforeUnmount(stop);
onDeactivated(stop);
</script>

<template>
  <div ref="rootRef">
    <h-input-group>
      <h-input
        ref="inputRef"
        class="mono"
        :input-props="{ 'aria-live': 'polite' }"
        :placeholder="isRecording ? $t('settings.hotkey.press') : $t('common.none')"
        readonly
        :value="displayText" />
      <h-button :type="isRecording ? 'primary' : 'default'" @click="toggleRecording()">
        {{ isRecording ? $t("common.cancel") : $t("common.edit") }}
      </h-button>
      <h-button
        :aria-label="$t('settings.hotkey.disable')"
        :disabled="!model || isRecording"
        :title="$t('settings.hotkey.disable')"
        @click="model = ''">
        ✕
      </h-button>
    </h-input-group>
  </div>
</template>
