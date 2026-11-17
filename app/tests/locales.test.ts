import { describe, expect, it } from "vitest";
import en from "@locales/en.json";
import fr from "@locales/fr.json";

function keysOf(value: unknown, prefix = ""): string[] {
  if (value && typeof value === "object") {
    return Object.entries(value).flatMap(([key, child]) =>
      keysOf(child, prefix ? `${prefix}.${key}` : key)
    );
  }
  return [prefix];
}

function placeholdersOf(text: string) {
  return [...text.matchAll(/\{(\w+)\}/g)].map(match => match[1]).sort();
}

function valueAt(messages: unknown, key: string) {
  return key
    .split(".")
    .reduce<unknown>((value, part) => (value as Record<string, unknown>)[part], messages);
}

describe("traductions", () => {
  const frKeys = keysOf(fr);
  const enKeys = keysOf(en);

  it("ont les mêmes clés en français et en anglais", () => {
    expect(frKeys.filter(key => !enKeys.includes(key))).toEqual([]);
    expect(enKeys.filter(key => !frKeys.includes(key))).toEqual([]);
  });

  it("ont les mêmes valeurs à remplacer dans les deux langues", () => {
    const mismatched = frKeys.filter(
      key =>
        placeholdersOf(String(valueAt(fr, key))).join() !==
        placeholdersOf(String(valueAt(en, key))).join()
    );
    expect(mismatched).toEqual([]);
  });

  it("n'utilisent pas de guillemets typographiques", () => {
    expect(frKeys.filter(key => /[«»]/.test(String(valueAt(fr, key))))).toEqual([]);
  });
});
