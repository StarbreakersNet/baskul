// Faux Baskul pour `yarn dev:mock` : le front tourne seul dans un navigateur, sans Tauri ni Rust.
// Chargé par main.ts dans ce mode seulement, jamais dans le build. État choisi dans l'adresse :
// ?state=connected|waiting|error|offline&role=primary|secondary&welcome
import { mockIPC, mockWindows } from "@tauri-apps/api/mocks";
import type { InvokeArgs } from "@tauri-apps/api/core";
import en from "@locales/en.json";
import fr from "@locales/fr.json";
import type { ActivityEvent } from "@/types/Activity";
import type { BaskulConfig, Role } from "@/types/Config";
import type { CommandError, DaemonInfo, DaemonStatus, LogTail, Peer } from "@/types/Daemon";
import type { CaptureState } from "@/types/Hotkey";
import type { LatencyInfo } from "@/types/Latency";
import type { NetworkInfo } from "@/types/Network";
import type { PairingState } from "@/types/Pairing";
import type { ProfilesInfo } from "@/types/Profiles";
import type { UpdateInfo } from "@/types/Update";
import { i18n } from "@/utils/locale";

type Scenario = "connected" | "waiting" | "error" | "offline";

interface MockState {
  scenario: Scenario;
  config: BaskulConfig;
  started: number;
  activePeer: string | null;
  captureStartedAt: number | null;
  pairingStartedAt: number | null;
  profiles: Record<string, BaskulConfig>;
  activity: ActivityEvent[];
  activityVersion: number;
}

const VERSION = "0.8.0";
const HOSTNAME = "atelier";
const ADDRESS = "192.168.1.20";
const PORT = 24800;
const CAPTURE_MS = 1500;
const START_MS = 1200;
const PAIRING_MS = 4000;
const LOG_LINES = 40;

// Comme Tauri, qui sérialise les arguments en JSON : la config arrive en objet réactif de Vue
function copyAsJson<T>(value: T): T {
  return JSON.parse(JSON.stringify(value)) as T;
}

function param(name: string) {
  return new URLSearchParams(window.location.search).get(name);
}

function scenarioFromUrl(): Scenario {
  const value = param("state");
  return value === "waiting" || value === "error" || value === "offline" ? value : "connected";
}

function defaultConfig(role: Role): BaskulConfig {
  return {
    role,
    name: "",
    network: {
      connect: "",
      listen: `0.0.0.0:${PORT}`,
      interface: "",
      discovery: true,
      secret: "abcd-efgh-jkmn-pqrs",
    },
    screen: {
      edge_switching: true,
      secondary_position: "right",
      corner_size: 10,
      pause_edges_for: [],
    },
    layout: {
      machines: [
        { name: "macbook", x: 1, y: 0 },
        { name: "pc-salon", x: 2, y: 0 },
      ],
    },
    mouse: { speed: 1 },
    scroll: {
      mode: "lines",
      lines_per_notch: 3,
      pixels_per_notch: 40,
      smooth_speed: "medium",
      smoothness: "regular",
      smooth_precision: false,
      trackpad_simulation: false,
      invert_vertical: false,
      invert_horizontal: false,
    },
    buttons: { side_buttons: "buttons" },
    keyboard: { iso_swap: "auto", media_keys: "cursor", remap: {} },
    clipboard: {
      enabled: true,
      images: true,
      html: true,
      files: false,
      max_size_mb: 20,
      max_files_mb: 100,
      skip_sensitive: true,
    },
    activity: { mirror: true },
    hotkeys: {
      switch_to_secondary: "Shift+F12",
      return_to_primary: "Shift+F11",
      next_machine: "",
      toggle_lock: "",
    },
    ui: {
      theme: "system",
      language: "system",
      tray_icon: "color",
      arrival_bubble: true,
      notifications: false,
      background: "current",
    },
    updates: { check: true },
    log: { level: "info" },
  };
}

// Textes que Baskul envoie lui-même, dans la langue de la fenêtre (section "daemon" des traductions)
function daemonText(key: keyof typeof fr.daemon.status, params: Record<string, string> = {}) {
  const messages = i18n.global.locale.value === "fr" ? fr.daemon.status : en.daemon.status;
  return Object.entries(params).reduce(
    (text, [name, value]) => text.replace(`{${name}}`, value),
    messages[key]
  );
}

function peers(state: MockState): Peer[] {
  const now = Date.now();
  return state.config.role === "primary"
    ? [
        { name: "macbook", os: "macos", since: now - 42 * 60_000 },
        { name: "pc-salon", os: "windows", since: now - 5 * 60_000 },
      ]
    : [{ name: "bureau", os: "windows", since: now - 18 * 60_000 }];
}

function waitingText(state: MockState) {
  return state.config.role === "primary"
    ? daemonText("waitingClients", { address: state.config.network.listen })
    : daemonText("searching");
}

function connectionOf(state: MockState) {
  const connected = state.scenario === "connected" ? peers(state) : [];
  if (state.scenario === "error") {
    const detail = daemonText("wrongSecret");
    return {
      state: "error" as const,
      detail,
      summary: daemonText("error", { error: detail }),
      connected,
    };
  }
  if (state.scenario === "waiting") {
    const detail = waitingText(state);
    return { state: "waiting" as const, detail, summary: detail, connected };
  }
  const names = connected.map(peer => peer.name).join(", ");
  return {
    state: "connected" as const,
    detail: "",
    summary: daemonText("connected", { peer: names }),
    connected,
  };
}

function updateInfo(): UpdateInfo {
  return {
    state: "upToDate",
    current: VERSION,
    checkedAt: Date.now() - 3_600_000,
    canInstall: true,
    releases: "https://github.com/StarbreakersNet/baskul/releases",
  };
}

function offlineError(): CommandError {
  return { kind: "offline", message: "Baskul ne répond pas (faux backend : ?state=offline)" };
}

function status(state: MockState): DaemonStatus {
  if (state.scenario === "offline") {
    throw offlineError();
  }
  const connection = connectionOf(state);
  return {
    state: connection.state,
    detail: connection.detail,
    summary: connection.summary,
    remote_active: state.activePeer !== null,
    locked: false,
    warning: null,
    peers: connection.connected,
    active_peer: connection.connected.length ? state.activePeer : null,
    role: state.config.role,
    version: VERSION,
    started: state.started,
    update: updateInfo(),
    activity: state.activityVersion,
    edges_paused_by: null,
    foreground: "firefox",
  };
}

function info(state: MockState): DaemonInfo {
  return {
    config: copyAsJson(state.config),
    error: null,
    path: "/home/moi/.config/baskul/config.toml",
    os: "linux",
    hostname: HOSTNAME,
    address: ADDRESS,
    version: VERSION,
    exe: "/usr/bin/baskul",
  };
}

function logTail(): LogTail {
  const levels = ["INFO ", "INFO ", "DEBUG", "INFO ", "WARN ", "INFO ", "ERROR", "INFO "];
  const text = Array.from({ length: LOG_LINES }, (_, index) => {
    const time = new Date(Date.now() - (LOG_LINES - index) * 7000).toISOString().replace("T", " ");
    return `${time.slice(0, 23)} ${levels[index % levels.length]} [baskul] ligne de démonstration ${index + 1}`;
  }).join("\n");
  return { path: "/home/moi/.local/state/baskul/baskul.log", text };
}

function network(): NetworkInfo {
  return {
    interfaces: [
      { name: "eth0", label: "Ethernet", addresses: [ADDRESS] },
      { name: "wlan0", label: "Wi-Fi", addresses: ["192.168.1.34"] },
    ],
    primaries: [{ name: "bureau", addresses: ["192.168.1.10"], os: "windows", protocol: 3 }],
    protocol: 3,
    defaultPort: PORT,
  };
}

function capture(state: MockState, action: unknown): CaptureState | null {
  if (action === "start") {
    state.captureStartedAt = Date.now();
  }
  if (action === "cancel") {
    state.captureStartedAt = null;
  }
  const isDone =
    state.captureStartedAt !== null && Date.now() - state.captureStartedAt > CAPTURE_MS;
  return action === "poll"
    ? isDone
      ? { state: "done", hotkey: "Ctrl+Alt+K" }
      : { state: "waiting" }
    : null;
}

function switchTo(state: MockState, to: unknown) {
  const names = peers(state).map(peer => peer.name);
  if (to === "next") {
    const index = state.activePeer ? names.indexOf(state.activePeer) + 1 : 0;
    state.activePeer = names[index] ?? null;
  } else {
    state.activePeer = typeof to === "string" && names.includes(to) ? to : null;
  }
  return null;
}

function sampleActivity(): ActivityEvent[] {
  const now = Date.now();
  return [
    {
      id: 3,
      at: now - 120_000,
      until: now - 120_000,
      count: 1,
      kind: "switchedTo",
      peer: "macbook",
    },
    {
      id: 2,
      at: now - 600_000,
      until: now - 600_000,
      count: 1,
      kind: "connected",
      peer: "pc-salon",
    },
    { id: 1, at: now - 900_000, until: now - 900_000, count: 1, kind: "started" },
  ];
}

function latency(state: MockState): LatencyInfo {
  const now = Date.now();
  return {
    peers: connectionOf(state).connected.map((peer, index) => ({
      name: peer.name,
      samples: Array.from({ length: 30 }, (_, i) => ({
        at: now - (30 - i) * 2000,
        ms: 2 + index * 3 + Math.abs(Math.sin(i * 0.7 + index)) * 4,
      })),
    })),
  };
}

function pairing(state: MockState, action: unknown): PairingState {
  if (action === "start") {
    state.pairingStartedAt = Date.now();
  }
  if (action === "cancel") {
    state.pairingStartedAt = null;
  }
  const startedAt = state.pairingStartedAt;
  return startedAt === null
    ? { state: "idle" }
    : Date.now() - startedAt > PAIRING_MS
      ? { state: "paired", name: "portable" }
      : { state: "waiting", code: "482913", expires: startedAt + 120_000 };
}

function profilesInfo(state: MockState): ProfilesInfo {
  const current = JSON.stringify(state.config);
  const names = Object.keys(state.profiles).sort();
  return {
    profiles: names,
    active: names.find(name => JSON.stringify(state.profiles[name]) === current) ?? null,
  };
}

function saveProfile(state: MockState, name: unknown) {
  state.profiles[String(name)] = copyAsJson(state.config);
  return profilesInfo(state);
}

function deleteProfile(state: MockState, name: unknown) {
  delete state.profiles[String(name)];
  return profilesInfo(state);
}

function applyProfile(state: MockState, name: unknown) {
  const profile = state.profiles[String(name)];
  if (profile) {
    state.config = copyAsJson(profile);
  }
  return restart(state);
}

function clearActivity(state: MockState) {
  state.activity = [];
  state.activityVersion++;
  return null;
}

function restart(state: MockState) {
  state.started = Date.now();
  return { restart: true };
}

function save(state: MockState, args: InvokeArgs | undefined) {
  state.config = copyAsJson((args as { config: BaskulConfig }).config);
  return restart(state);
}

function startDaemon(state: MockState) {
  setTimeout(() => {
    state.scenario = "connected";
    state.started = Date.now();
  }, START_MS);
  return null;
}

function handle(state: MockState, cmd: string, args: InvokeArgs | undefined): unknown {
  const payload = (args ?? {}) as Record<string, unknown>;
  const commands: Record<string, () => unknown> = {
    "daemon_status": () => status(state),
    "load_settings": () => info(state),
    "save_settings": () => save(state, args),
    "restart_daemon": () => restart(state),
    "read_log": () => logTail(),
    "network_info": () => network(),
    "capture_hotkey": () => capture(state, payload.action),
    "switch_to": () => switchTo(state, payload.to),
    "update_action": () => updateInfo(),
    "start_daemon": () => startDaemon(state),
    "launch_options": () => ({ welcome: param("welcome") !== null }),
    "read_activity": () => ({ events: state.activity }),
    "clear_activity": () => clearActivity(state),
    "read_latency": () => latency(state),
    "pairing": () => pairing(state, payload.action),
    "join_primary": () => ({ ...restart(state), primary: "bureau" }),
    "list_profiles": () => profilesInfo(state),
    "save_profile": () => saveProfile(state, payload.name),
    "delete_profile": () => deleteProfile(state, payload.name),
    "apply_profile": () => applyProfile(state, payload.name),
    "export_settings": () => null,
    "import_settings": () => restart(state),
    "plugin:dialog|save": () => null,
    "plugin:dialog|open": () => null,
    "open_config_file": () => null,
    "open_log_file": () => null,
    "plugin:app|version": () => VERSION,
    "plugin:opener|open_url": () => window.open(String(payload.url), "_blank") && null,
  };
  const command = commands[cmd];
  if (!command) {
    console.warn(`Faux backend : commande inconnue ${cmd}`, args);
  }
  return command ? command() : null;
}

export function installTauriMock() {
  const role: Role = param("role") === "secondary" ? "secondary" : "primary";
  const state: MockState = {
    scenario: scenarioFromUrl(),
    config: defaultConfig(role),
    started: Date.now(),
    activePeer: role === "primary" ? "macbook" : null,
    captureStartedAt: null,
    pairingStartedAt: null,
    profiles: {},
    activity: sampleActivity(),
    activityVersion: 1,
  };
  state.profiles = { Bureau: copyAsJson(state.config) };
  mockWindows("main");
  mockIPC((cmd, args) => handle(state, cmd, args), { shouldMockEvents: true });
}
