// Types partagés avec le daemon (voir `src/config.rs` et `src/api.rs`).

export type Role = "primary" | "secondary";
export type Edge = "left" | "right" | "top" | "bottom";

/** Contenu du fichier `config.toml`, tel que le daemon le renvoie. */
export interface BaskulConfig {
  role: Role;
  name: string;
  network: { connect: string; listen: string; interface: string; discovery: boolean; secret: string };
  screen: { edge_switching: boolean; secondary_position: Edge; corner_size: number };
  /** Grille des machines (principale en 0, 0 ; x vers la droite, y vers le bas). */
  layout: { machines: LayoutMachine[] };
  mouse: { speed: number };
  scroll: {
    mode: "lines" | "pixels" | "smooth";
    lines_per_notch: number;
    pixels_per_notch: number;
    smooth_speed: "low" | "medium" | "high";
    smoothness: "regular" | "high";
    smooth_precision: boolean;
    trackpad_simulation: boolean;
    invert_vertical: boolean;
    invert_horizontal: boolean;
  };
  buttons: { side_buttons: "buttons" | "swipe" };
  keyboard: { iso_swap: "auto" | "on" | "off"; media_keys: "cursor" | "primary" | "secondary"; remap: Record<string, string> };
  clipboard: {
    enabled: boolean;
    images: boolean;
    html: boolean;
    files: boolean;
    max_size_mb: number;
    max_files_mb: number;
    skip_sensitive: boolean;
  };
  activity: { mirror: boolean };
  hotkeys: { switch_to_secondary: string; return_to_primary: string; next_machine: string; toggle_lock: string };
  ui: {
    theme: "system" | "light" | "dark";
    language: "system" | "fr" | "en";
    tray_icon: "color" | "mono";
    arrival_bubble: boolean;
  };
  updates: { check: boolean };
  log: { level: string };
}

/** Une pilotée placée sur la grille. */
export interface LayoutMachine {
  name: string;
  x: number;
  y: number;
  /** Raccourci pour aller directement sur elle (absent : aucun). */
  hotkey?: string;
}

/** Autre machine connectée. */
export interface Peer {
  name: string;
  os: string;
  /** Début de la connexion (ms). */
  since: number;
}

/** Réponse de `load_settings`. */
export interface DaemonInfo {
  /** `null` si le fichier ne peut pas être lu (voir `error`). */
  config: BaskulConfig | null;
  error: string | null;
  path: string;
  os: string;
  hostname: string;
  /** Adresse IP de cette machine sur le réseau local. */
  address: string | null;
  version: string;
  /** Exécutable de Baskul (commandes `baskul switch …`). */
  exe: string | null;
}

/** Réponse de `daemon_status`. */
export interface DaemonStatus {
  state: "waiting" | "connected" | "error";
  detail: string;
  summary: string;
  remote_active: boolean;
  locked: boolean;
  warning: string | null;
  /** Autres machines connectées, dans l'ordre de connexion. */
  peers: Peer[];
  /** Machine principale : pilotée où agissent le clavier et la souris. */
  active_peer: string | null;
  role: Role | null;
  version: string;
  /** Démarrage du daemon (ms) : change à chaque redémarrage. */
  started: number;
  update: UpdateInfo;
}

/** Carte réseau de cette machine. */
export interface NetInterface {
  /** Nom donné par le système, enregistré dans la configuration. */
  name: string;
  /** Nom lisible (« Wi-Fi », « Ethernet »…). */
  label: string;
  addresses: string[];
}

/** Machine principale trouvée sur le réseau local. */
export interface FoundPrimary {
  name: string;
  addresses: string[];
  os: string;
  protocol: number;
}

/** Réponse de `network_info`. */
export interface NetworkInfo {
  interfaces: NetInterface[];
  primaries: FoundPrimary[];
  /** Version du protocole de cette machine. */
  protocol: number;
  defaultPort: number;
}

/** État des mises à jour (voir `src/update.rs`). */
export type UpdateState =
  | { state: "idle" | "checking" | "upToDate" }
  | { state: "available"; version: string; notes: string; url: string }
  | { state: "downloading"; version: string; done: number; total: number }
  | { state: "ready"; version: string; notes: string; url: string }
  | { state: "installing"; version: string }
  | { state: "error"; message: string };

export type UpdateInfo = UpdateState & {
  current: string;
  checkedAt: number | null;
  /** Installation automatique possible sur ce système. */
  canInstall: boolean;
  /** Page des versions publiées. */
  releases: string;
};

/** Saisie d'un raccourci par le daemon. */
export type CaptureState = { state: "idle" | "waiting" } | { state: "done"; hotkey: string };
