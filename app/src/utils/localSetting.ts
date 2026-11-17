// Préférences propres à la fenêtre (pas à Baskul) : le stockage peut être indisponible

export function readLocalFlag(key: string, fallback: boolean) {
  try {
    const stored = localStorage.getItem(key);
    return stored === null ? fallback : stored === "true";
  } catch {
    return fallback;
  }
}

export function writeLocalFlag(key: string, value: boolean) {
  try {
    localStorage.setItem(key, String(value));
  } catch {
    // Stockage indisponible : la préférence ne survivra pas à la fermeture de la fenêtre
  }
}

export function readLocalJson<T extends object>(key: string, fallback: T): T {
  try {
    return { ...fallback, ...(JSON.parse(localStorage.getItem(key) ?? "{}") as Partial<T>) };
  } catch {
    return fallback;
  }
}

export function writeLocalJson(key: string, value: object) {
  try {
    localStorage.setItem(key, JSON.stringify(value));
  } catch {
    // Stockage indisponible : la préférence ne survivra pas à la fermeture de la fenêtre
  }
}
