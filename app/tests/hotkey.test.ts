import { describe, expect, it } from "vitest";
import { formatHotkey, hotkeyParts, keyName, pressedModifiers } from "@/utils/hotkey";

function keyEvent(init: KeyboardEventInit) {
  return init as KeyboardEvent;
}

describe("raccourcis", () => {
  it("nomme les touches comme la configuration", () => {
    expect(keyName("KeyB")).toBe("B");
    expect(keyName("Digit1")).toBe("1");
    expect(keyName("F12")).toBe("F12");
  });

  it("donne les modificateurs dans l'ordre de la configuration", () => {
    const event = keyEvent({ ctrlKey: true, altKey: true, shiftKey: true, metaKey: true });
    expect(pressedModifiers(event, "macos")).toEqual(["Ctrl", "Alt", "Shift", "Cmd"]);
    expect(pressedModifiers(event, "windows")).toEqual(["Ctrl", "Alt", "Shift", "Win"]);
  });

  it("affiche une combinaison avec des espaces", () => {
    expect(formatHotkey(hotkeyParts("Ctrl+Alt+B"))).toBe("Ctrl + Alt + B");
    expect(hotkeyParts("")).toEqual([]);
  });
});
