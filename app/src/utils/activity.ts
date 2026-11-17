import type { ActivityEvent, ActivityKind } from "@/types/Activity";

export type ActivityColor = "success" | "warning" | "info" | "default" | "error";

interface ActivityLabel {
  /** Clé de traduction, sous `home` */
  key: string;
  params: Record<string, string | number>;
  color: ActivityColor;
}

export interface ActivityDay {
  /** Début du jour (ms, heure locale) */
  day: number;
  events: ActivityEvent[];
}

const LABELS: Record<ActivityKind, { key: string; runKey: string; color: ActivityColor }> = {
  started: { key: "eventStarted", runKey: "eventRestarts", color: "info" },
  connected: { key: "eventConnected", runKey: "eventConnected", color: "success" },
  disconnected: { key: "eventDisconnected", runKey: "eventDisconnected", color: "warning" },
  reconnected: { key: "eventReconnected", runKey: "eventReconnects", color: "warning" },
  switchedTo: { key: "eventSwitchedTo", runKey: "eventSwitches", color: "info" },
  switchedHere: { key: "eventSwitchedHere", runKey: "eventSwitches", color: "info" },
  locked: { key: "eventLocked", runKey: "eventLocks", color: "default" },
  unlocked: { key: "eventUnlocked", runKey: "eventLocks", color: "default" },
  warning: { key: "eventWarning", runKey: "eventWarnings", color: "error" },
  edgesPaused: { key: "eventEdgesPaused", runKey: "eventEdgesRuns", color: "warning" },
  edgesResumed: { key: "eventEdgesResumed", runKey: "eventEdgesRuns", color: "default" },
  paired: { key: "eventPaired", runKey: "eventPaired", color: "success" },
};

function singleKey(event: ActivityEvent) {
  return event.kind === "started" && event.restarted ? "eventRestarted" : LABELS[event.kind].key;
}

/** `locale` : pour relier les noms ("A, B et C") */
export function describeActivity(event: ActivityEvent, locale = "fr"): ActivityLabel {
  const isRun = event.count > 1;
  return {
    key: isRun ? LABELS[event.kind].runKey : singleKey(event),
    color: LABELS[event.kind].color,
    params: {
      name: event.peer ?? event.program ?? "",
      names: new Intl.ListFormat(locale, { type: "conjunction" }).format(
        (event.names ?? []).map(name => `"${name}"`)
      ),
      text: event.text ?? "",
      count: event.count,
      seconds: Math.max(1, Math.round((event.gap ?? 0) / 1000)),
    },
  };
}

function startOfDay(time: number) {
  const date = new Date(time);
  date.setHours(0, 0, 0, 0);
  return date.getTime();
}

/** Par jour de la dernière occurrence, du plus récent au plus ancien ; `events` déjà dans cet ordre */
export function groupByDay(events: ActivityEvent[]): ActivityDay[] {
  const days: ActivityDay[] = [];
  for (const event of events) {
    const day = startOfDay(Math.max(event.until, event.at));
    const last = days[days.length - 1];
    if (last?.day === day) {
      last.events.push(event);
    } else {
      days.push({ day, events: [event] });
    }
  }
  return days;
}
