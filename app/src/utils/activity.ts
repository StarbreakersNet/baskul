import type { ActivityEvent, ActivityKind } from "@/types/Activity";

export type ActivityColor = "success" | "warning" | "info" | "default" | "error";

interface ActivityLabel {
  /** Clé de traduction, sous `home` */
  key: string;
  params: Record<string, string>;
  color: ActivityColor;
}

const LABELS: Record<ActivityKind, { key: string; color: ActivityColor }> = {
  started: { key: "eventStarted", color: "info" },
  connected: { key: "eventConnected", color: "success" },
  disconnected: { key: "eventDisconnected", color: "warning" },
  switchedTo: { key: "eventSwitchedTo", color: "info" },
  switchedHere: { key: "eventSwitchedHere", color: "info" },
  locked: { key: "eventLocked", color: "default" },
  unlocked: { key: "eventUnlocked", color: "default" },
  warning: { key: "eventWarning", color: "error" },
  edgesPaused: { key: "eventEdgesPaused", color: "warning" },
  edgesResumed: { key: "eventEdgesResumed", color: "default" },
  paired: { key: "eventPaired", color: "success" },
};

export function describeActivity(event: ActivityEvent): ActivityLabel {
  const { key, color } = LABELS[event.kind];
  return {
    key,
    color,
    params: { name: event.peer ?? event.program ?? "", text: event.text ?? "" },
  };
}
