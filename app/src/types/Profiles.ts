/** Réponse de `list_profiles` */
export interface ProfilesInfo {
  profiles: string[];
  /** Profil identique aux réglages actuels */
  active: string | null;
}
