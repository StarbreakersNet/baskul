import type { BaskulConfig, Role } from "@/types/Config";
import type { UpdateInfo } from "@/types/Update";

export interface Peer {
  name: string;
  os: string;
  /** Début de la connexion (ms) */
  since: number;
}

/** Réponse de `load_settings` */
export interface DaemonInfo {
  /** `null` si le fichier est illisible : voir `error` */
  config: BaskulConfig | null;
  error: string | null;
  path: string;
  os: string;
  hostname: string;
  /** Adresse IP de cette machine sur le réseau local */
  address: string | null;
  version: string;
  /** Exécutable de Baskul, pour les commandes `baskul switch …` */
  exe: string | null;
}

export type ConnectionState = "waiting" | "connected" | "error";

/** Réponse de `daemon_status` */
export interface DaemonStatus {
  state: ConnectionState;
  detail: string;
  summary: string;
  remote_active: boolean;
  locked: boolean;
  warning: string | null;
  /** Dans l'ordre de connexion */
  peers: Peer[];
  /** Machine principale : pilotée où agissent le clavier et la souris */
  active_peer: string | null;
  role: Role | null;
  version: string;
  /** Démarrage de Baskul (ms) : change à chaque redémarrage */
  started: number;
  /** Change à chaque modification de l'activité récente */
  activity: number;
  update: UpdateInfo;
}

export interface CommandError {
  kind: "offline" | "refused";
  message: string;
}

export interface LogTail {
  path: string | null;
  text: string;
}
