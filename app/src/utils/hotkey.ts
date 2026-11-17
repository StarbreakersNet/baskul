// Format de la configuration : "Shift+F12", "Ctrl+Alt+B" (voir core/src/hotkey.rs)

const MODIFIER_CODES = new Set([
  "ShiftLeft",
  "ShiftRight",
  "ControlLeft",
  "ControlRight",
  "AltLeft",
  "AltRight",
  "MetaLeft",
  "MetaRight",
  "OSLeft",
  "OSRight",
]);

export function isModifierKey(event: KeyboardEvent) {
  return MODIFIER_CODES.has(event.code);
}

export function pressedModifiers(event: KeyboardEvent, os?: string) {
  const modifiers: string[] = [];
  if (event.ctrlKey) {
    modifiers.push("Ctrl");
  }
  if (event.altKey) {
    modifiers.push("Alt");
  }
  if (event.shiftKey) {
    modifiers.push("Shift");
  }
  if (event.metaKey) {
    modifiers.push(os === "macos" ? "Cmd" : "Win");
  }
  return modifiers;
}

/** Nom d'une touche physique (`KeyboardEvent.code`) dans la configuration */
export function keyName(code: string) {
  return /^Key([A-Z])$/.exec(code)?.[1] ?? /^Digit(\d)$/.exec(code)?.[1] ?? code;
}

export function hotkeyParts(hotkey: string) {
  return hotkey ? hotkey.split("+") : [];
}

export function formatHotkey(parts: string[]) {
  return parts.join(" + ");
}
