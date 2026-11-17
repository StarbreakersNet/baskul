import { describe, expect, it } from "vitest";
import fr from "@locales/fr.json";
import { describeActivity } from "@/utils/activity";
import type { ActivityKind } from "@/types/Activity";

const KINDS: ActivityKind[] = [
  "started",
  "connected",
  "disconnected",
  "switchedTo",
  "switchedHere",
  "locked",
  "unlocked",
  "warning",
];

describe("activité récente", () => {
  it("a un texte pour chaque sorte d'événement", () => {
    const home: Record<string, string> = fr.app.home;
    for (const kind of KINDS) {
      expect(home[describeActivity({ id: 1, at: 0, kind }).key]).toBeTypeOf("string");
    }
  });

  it("passe la machine et le texte de l'avertissement", () => {
    const connected = describeActivity({ id: 1, at: 0, kind: "connected", peer: "Mac" });
    expect(connected.params.name).toBe("Mac");
    expect(connected.color).toBe("success");
    const warning = describeActivity({ id: 2, at: 0, kind: "warning", text: "Accessibilité" });
    expect(warning.params.text).toBe("Accessibilité");
  });
});
