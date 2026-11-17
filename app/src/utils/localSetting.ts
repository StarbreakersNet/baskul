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
