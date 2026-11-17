/** État de l'ajout d'une machine par code, côté principale (`pairing`) */
export type PairingState =
  | { state: "idle" }
  | { state: "waiting"; code: string; /** Fin de validité (ms) */ expires: number }
  | { state: "paired"; name: string }
  | { state: "failed"; reason: string };
