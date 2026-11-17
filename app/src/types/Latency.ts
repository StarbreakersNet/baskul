export interface LatencySample {
  /** Date (ms) */
  at: number;
  /** Aller-retour (ms) */
  ms: number;
}

/** Réponse de `read_latency` */
export interface LatencyInfo {
  peers: { name: string; samples: LatencySample[] }[];
}
