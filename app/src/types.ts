// Types partagés avec le daemon (voir `src/config.rs` et `src/api.rs`).

export type Role = "primary" | "secondary";
export type Edge = "left" | "right" | "top" | "bottom";

/** Contenu du fichier `config.toml`, tel que le daemon le renvoie. */
export interface BasculeConfig {
  role: Role;
  name: string;
  network: { listen: string; connect: string; secret: string };
  screen: { edge_switching: boolean; secondary_position: Edge; corner_size: number };
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
  clipboard: { enabled: boolean; images: boolean; html: boolean; max_size_mb: number; skip_sensitive: boolean };
  hotkeys: { switch_to_secondary: string; return_to_primary: string; toggle_lock: string };
  ui: { theme: "system" | "light" | "dark"; language: "system" | "fr" | "en"; tray_icon: "color" | "mono" };
  log: { level: string };
}

/** Réponse de `load_settings`. */
export interface DaemonInfo {
  /** `null` si le fichier ne peut pas être lu (voir `error`). */
  config: BasculeConfig | null;
  error: string | null;
  path: string;
  os: string;
  hostname: string;
  /** Adresse IP de cette machine sur le réseau local. */
  address: string | null;
  version: string;
}

/** Réponse de `daemon_status`. */
export interface DaemonStatus {
  state: "waiting" | "connected" | "error";
  detail: string;
  summary: string;
  remote_active: boolean;
  locked: boolean;
  warning: string | null;
  /** Autre machine, quand la connexion est établie. */
  peer: { os: string; since: number } | null;
  version: string;
  /** Démarrage du daemon (ms) : change à chaque redémarrage. */
  started: number;
}

/** Saisie d'un raccourci par le daemon. */
export type CaptureState = { state: "idle" | "waiting" } | { state: "done"; hotkey: string };
