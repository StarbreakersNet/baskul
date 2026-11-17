import { describe, expect, it } from "vitest";
import fr from "@locales/fr.json";
import { describeActivity, groupByDay } from "@/utils/activity";
import type { ActivityEvent, ActivityKind } from "@/types/Activity";

const KINDS: ActivityKind[] = [
  "started",
  "connected",
  "disconnected",
  "reconnected",
  "switchedTo",
  "switchedHere",
  "locked",
  "unlocked",
  "warning",
  "edgesPaused",
  "edgesResumed",
  "paired",
];

function event(kind: ActivityKind, extra: Partial<ActivityEvent> = {}): ActivityEvent {
  return { id: 1, at: 0, until: 0, count: 1, kind, ...extra };
}

describe("activité récente", () => {
  it("a un texte pour chaque sorte d'événement, seul ou en suite", () => {
    const home: Record<string, string> = fr.app.home;
    for (const kind of KINDS) {
      expect(home[describeActivity(event(kind)).key]).toBeTypeOf("string");
      expect(home[describeActivity(event(kind, { count: 3 })).key]).toBeTypeOf("string");
    }
  });

  it("résume une suite de bascules", () => {
    const label = describeActivity(event("switchedTo", { count: 14, names: ["Mac", "Salon"] }));
    expect(label.key).toBe("eventSwitches");
    expect(label.params).toMatchObject({ count: 14, names: '"Mac" et "Salon"' });
    expect(
      describeActivity(event("switchedTo", { count: 2, names: ["A", "B"] }), "en").params.names
    ).toBe('"A" and "B"');
  });

  it("distingue un redémarrage et donne la durée d'une coupure", () => {
    expect(describeActivity(event("started", { restarted: true })).key).toBe("eventRestarted");
    const reconnected = describeActivity(event("reconnected", { peer: "Mac", gap: 3_600 }));
    expect(reconnected.params).toMatchObject({ name: "Mac", seconds: 4 });
  });

  it("regroupe par jour de la dernière occurrence", () => {
    const day = new Date(2026, 10, 17, 12).getTime();
    const hour = 60 * 60 * 1000;
    const days = groupByDay([
      event("locked", { id: 3, at: day, until: day + hour }),
      event("connected", { id: 2, at: day - 2 * hour, until: day - 2 * hour }),
      event("started", { id: 1, at: day - 24 * hour, until: day - 24 * hour }),
    ]);
    expect(days.map(group => group.events.map(e => e.id))).toEqual([[3, 2], [1]]);
  });
});
