import { defineStore } from "pinia";
import { ref } from "vue";
import type { DaemonStatus } from "@/types/Daemon";

export type ActivityType =
  "connection" | "disconnection" | "switch" | "lock" | "restart" | "warning";

export interface ActivityEvent {
  id: number;
  at: number;
  type: ActivityType;
  /** Clé de traduction, sous `home` */
  key: string;
  params?: Record<string, string>;
}

const MAX_EVENTS = 30;

// Journal propre à la fenêtre : il se déduit des changements d'état de Baskul et repart à zéro avec elle
export const useActivityStore = defineStore("activity", () => {
  const events = ref<ActivityEvent[]>([]);

  let lastStatus: DaemonStatus | null = null;
  let nextId = 0;

  function add(type: ActivityType, key: string, params?: Record<string, string>) {
    nextId += 1;
    events.value = [{ id: nextId, at: Date.now(), type, key, params }, ...events.value].slice(
      0,
      MAX_EVENTS
    );
  }

  function observe(current: DaemonStatus) {
    if (lastStatus) {
      compare(lastStatus, current);
    }
    lastStatus = current;
  }

  function compare(previous: DaemonStatus, current: DaemonStatus) {
    if (previous.started !== current.started) {
      add("restart", "eventRestarted");
    } else {
      comparePeers(previous, current);
      compareActive(previous, current);
    }
    if (previous.locked !== current.locked) {
      add("lock", current.locked ? "eventLocked" : "eventUnlocked");
    }
    if (current.warning && current.warning !== previous.warning) {
      add("warning", "eventWarning", { text: current.warning });
    }
  }

  function comparePeers(previous: DaemonStatus, current: DaemonStatus) {
    const before = new Set(previous.peers.map(peer => peer.name));
    const after = new Set(current.peers.map(peer => peer.name));
    for (const name of after) {
      if (!before.has(name)) {
        add("connection", "eventConnected", { name });
      }
    }
    for (const name of before) {
      if (!after.has(name)) {
        add("disconnection", "eventDisconnected", { name });
      }
    }
  }

  // Principale : `active_peer` dit où sont le clavier et la souris. Pilotée : `remote_active` dit qu'ils sont ici
  function compareActive(previous: DaemonStatus, current: DaemonStatus) {
    if (current.role === "secondary") {
      const [primary] = current.peers;
      if (previous.remote_active !== current.remote_active && primary) {
        if (current.remote_active) {
          add("switch", "eventSwitchedHere");
        } else {
          add("switch", "eventSwitchedTo", { name: primary.name });
        }
      }
    } else if (previous.active_peer !== current.active_peer) {
      if (current.active_peer) {
        add("switch", "eventSwitchedTo", { name: current.active_peer });
      } else {
        add("switch", "eventSwitchedHere");
      }
    }
  }

  function clear() {
    events.value = [];
  }

  return { events, observe, clear };
});
