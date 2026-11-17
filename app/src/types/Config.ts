export type Role = "primary" | "secondary";
export type Edge = "left" | "right" | "top" | "bottom";

/** Contenu de `config.toml` (voir `core/src/config.rs`) */
export interface BaskulConfig {
  role: Role;
  name: string;
  network: {
    connect: string;
    listen: string;
    interface: string;
    discovery: boolean;
    secret: string;
  };
  screen: { edge_switching: boolean; secondary_position: Edge; corner_size: number };
  /** Principale en 0, 0 ; x vers la droite, y vers le bas */
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
  keyboard: {
    iso_swap: "auto" | "on" | "off";
    media_keys: "cursor" | "primary" | "secondary";
    remap: Record<string, string>;
  };
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
  hotkeys: {
    switch_to_secondary: string;
    return_to_primary: string;
    next_machine: string;
    toggle_lock: string;
  };
  ui: {
    theme: "system" | "light" | "dark";
    language: "system" | "fr" | "en";
    tray_icon: "color" | "mono";
    arrival_bubble: boolean;
    notifications: boolean;
  };
  updates: { check: boolean };
  log: { level: string };
}

export interface LayoutMachine {
  name: string;
  x: number;
  y: number;
  /** Raccourci qui mène directement sur cette pilotée */
  hotkey?: string;
}
