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
  | "edgesResumed";

/** Événement de `read_activity` (core/src/activity.rs) */
export interface ActivityEvent {
  id: number;
  /** Date (ms) */
  at: number;
  kind: ActivityKind;
  /** connected, disconnected, switchedTo */
  peer?: string;
  /** warning */
  text?: string;
  /** edgesPaused */
  program?: string;
}
