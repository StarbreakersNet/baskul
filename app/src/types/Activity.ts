export type ActivityKind =
  | "started"
  | "connected"
  | "disconnected"
  | "reconnected"
  | "switchedTo"
  | "switchedHere"
  | "locked"
  | "unlocked"
  | "warning";

/** Événement de `read_activity`, ou suite d'événements semblables regroupés (core/src/activity.rs) */
export interface ActivityEvent {
  id: number;
  /** Première occurrence (ms) */
  at: number;
  /** Dernière occurrence (ms) */
  until: number;
  count: number;
  /** Machines concernées par une suite */
  names?: string[];
  /** Le plus récent de la suite */
  kind: ActivityKind;
  /** connected, disconnected, reconnected, switchedTo */
  peer?: string;
  /** warning */
  text?: string;
  /** started : redémarrage plutôt que lancement */
  restarted?: boolean;
  /** reconnected : durée de la coupure (ms) */
  gap?: number;
}
