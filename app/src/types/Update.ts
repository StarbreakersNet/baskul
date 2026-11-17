/** Voir `src/update.rs` */
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
  /** Installation automatique possible sur ce système */
  canInstall: boolean;
  /** Page des versions publiées */
  releases: string;
};
