/** Saisie d'un raccourci par le crochet clavier de Baskul */
export type CaptureState = { state: "idle" | "waiting" } | { state: "done"; hotkey: string };
