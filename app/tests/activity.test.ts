import { beforeEach, describe, expect, it } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { useActivityStore } from "@/stores/activity";
import type { DaemonStatus } from "@/types/Daemon";

function status(overrides: Partial<DaemonStatus> = {}): DaemonStatus {
  return {
    state: "connected",
    detail: "",
    summary: "",
    remote_active: false,
    locked: false,
    warning: null,
    peers: [{ name: "Mac", os: "macos", since: 0 }],
    active_peer: null,
    role: "primary",
    version: "0.8.0",
    started: 1,
    update: { state: "idle" } as DaemonStatus["update"],
    ...overrides,
  };
}

describe("activité récente", () => {
  beforeEach(() => setActivePinia(createPinia()));

  it("ne note rien au premier état", () => {
    const activity = useActivityStore();
    activity.observe(status());
    expect(activity.events).toHaveLength(0);
  });

  it("note les connexions, les bascules et le verrouillage, le plus récent d'abord", () => {
    const activity = useActivityStore();
    activity.observe(status());
    activity.observe(
      status({
        peers: [
          { name: "Mac", os: "macos", since: 0 },
          { name: "Salon", os: "windows", since: 1 },
        ],
      })
    );
    activity.observe(status({ active_peer: "Mac" }));
    activity.observe(status({ locked: true }));
    expect(activity.events.map(event => [event.key, event.params?.name])).toEqual([
      ["eventLocked", undefined],
      ["eventSwitchedHere", undefined],
      ["eventSwitchedTo", "Mac"],
      ["eventDisconnected", "Salon"],
      ["eventConnected", "Salon"],
    ]);
  });

  it("sur une pilotée, suit remote_active", () => {
    const activity = useActivityStore();
    activity.observe(status({ role: "secondary" }));
    activity.observe(status({ role: "secondary", remote_active: true }));
    expect(activity.events[0].key).toBe("eventSwitchedHere");
  });

  it("un redémarrage remplace les autres changements", () => {
    const activity = useActivityStore();
    activity.observe(status());
    activity.observe(status({ started: 2, peers: [] }));
    expect(activity.events.map(event => event.key)).toEqual(["eventRestarted"]);
  });
});
