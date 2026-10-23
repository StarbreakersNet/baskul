// État de l'app de réglages, partagé par tous les composants.
//
// Cycle : chargement → prêt ; si Baskul ne répond pas → hors ligne (l'état
// est réinterrogé régulièrement) ; après un enregistrement → redémarrage,
// jusqu'à ce que le daemon réponde avec une nouvelle date de démarrage.
import { computed, ref } from "vue";
import { daemon, errorMessage, isOffline } from "../daemon";
import type { BaskulConfig, DaemonInfo, DaemonStatus } from "../types";

type Phase = "loading" | "offline" | "starting" | "ready" | "restarting";

const POLL_MS = 1500;
const RESTART_POLL_MS = 400;
const RESTART_TIMEOUT_MS = 20_000;

const phase = ref<Phase>("loading");
const info = ref<DaemonInfo | null>(null);
const config = ref<BaskulConfig | null>(null);
const status = ref<DaemonStatus | null>(null);
const savedJson = ref("");
const saving = ref(false);
const welcome = ref(false);

let timer: ReturnType<typeof setTimeout> | undefined;
let restartFrom: number | undefined;
let restartDeadline = 0;
let onRestarted: ((ok: boolean) => void) | undefined;
/** Pilotées connectées lors de la dernière vérification de la grille, et
 * relectures restantes (l'enregistrement par Baskul peut avoir un temps de
 * retard). */
let peersSeen = "";
let layoutRetries = 0;

const clone = <T>(v: T): T => JSON.parse(JSON.stringify(v));

const dirty = computed(() => config.value !== null && JSON.stringify(config.value) !== savedJson.value);

async function load() {
  const i = await daemon.load();
  info.value = i;
  config.value = i.config ? clone(i.config) : null;
  savedJson.value = JSON.stringify(i.config);
  phase.value = "ready";
}

/** Interroge l'état du daemon, et recharge les réglages quand il revient. */
async function poll() {
  try {
    const s = await daemon.status();
    status.value = s;
    if (phase.value === "restarting") {
      if (s.started !== restartFrom) {
        await load();
        onRestarted?.(true);
      }
    } else if (phase.value !== "ready") {
      await load();
    } else if (newlyPlaced(s)) {
      // Une pilotée vient d'être placée sur la grille par Baskul (première
      // connexion) : on relit les réglages, sauf modifications en cours.
      await load();
    }
  } catch (e) {
    if (!isOffline(e)) console.error(e);
    status.value = null;
    if (phase.value === "ready" || phase.value === "loading") phase.value = "offline";
  }
  if (phase.value === "restarting" && Date.now() > restartDeadline) {
    phase.value = "offline";
    onRestarted?.(false);
  }
  schedule();
}

/** Une pilotée connectée manque à la grille chargée (machine principale). */
function newlyPlaced(s: DaemonStatus): boolean {
  const names = s.peers.map((p) => p.name).join("\n");
  if (names !== peersSeen) {
    peersSeen = names;
    layoutRetries = 3;
  }
  const c = config.value;
  if (layoutRetries === 0 || !c || c.role !== "primary" || dirty.value) return false;
  const missing = s.peers.some((p) => !c.layout.machines.some((m) => m.name === p.name));
  layoutRetries = missing ? layoutRetries - 1 : 0;
  return missing;
}

function schedule() {
  clearTimeout(timer);
  const fast = phase.value === "restarting" || phase.value === "starting";
  timer = setTimeout(poll, fast ? RESTART_POLL_MS : POLL_MS);
}

/** Attend la fin du redémarrage ; résout `true` s'il a abouti. */
function waitForRestart(): Promise<boolean> {
  restartFrom = status.value?.started;
  restartDeadline = Date.now() + RESTART_TIMEOUT_MS;
  phase.value = "restarting";
  schedule();
  return new Promise((resolve) => {
    onRestarted = (ok) => {
      onRestarted = undefined;
      resolve(ok);
    };
  });
}

export function useSettings() {
  return {
    phase,
    info,
    config,
    status,
    dirty,
    saving,
    welcome,

    async init() {
      welcome.value = (await daemon.launchOptions().catch(() => ({ welcome: false }))).welcome;
      await poll();
    },

    /** Enregistre. Résout `true` si les réglages sont appliqués. */
    async save(): Promise<boolean> {
      if (!config.value) return false;
      saving.value = true;
      try {
        const r = await daemon.save(config.value);
        savedJson.value = JSON.stringify(config.value);
        welcome.value = false;
        return r.restart ? await waitForRestart() : true;
      } finally {
        saving.value = false;
      }
    },

    reset() {
      config.value = JSON.parse(savedJson.value);
    },

    async restart(): Promise<boolean> {
      const r = await daemon.restart();
      return r.restart ? await waitForRestart() : false;
    },

    async start() {
      await daemon.start();
      phase.value = "starting";
      schedule();
    },

    openConfigFile: () => daemon.openConfigFile(),
    errorMessage,
  };
}
