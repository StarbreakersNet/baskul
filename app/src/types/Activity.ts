export type ActivityKind =
  | "started"
  | "connected"
  | "disconnected"
  | "switchedTo"
  | "switchedHere"
  | "locked"
  | "unlocked"
  | "warning"
  | "edgesPaused"
  | "edgesResumed"
  | "paired";

/** Événement de `read_activity` (core/src/activity.rs) */
export interface ActivityEvent {
  id: number;
  /** Date (ms) */
  at: number;
  kind: ActivityKind;
  /** connected, disconnected, switchedTo, paired */
  peer?: string;
  /** warning */
  text?: string;
  /** edgesPaused */
  program?: string;
}
