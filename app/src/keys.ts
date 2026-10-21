import type { InjectionKey, Ref } from "vue";
import type { Role } from "./types";

/** Rôle choisi pour cette machine (pour griser les réglages de l'autre). */
export const roleKey: InjectionKey<Readonly<Ref<Role>>> = Symbol("role");
/** Machine concernée par la section englobante. */
export const sectionMachineKey: InjectionKey<Role | undefined> = Symbol("sectionMachine");

export const machineLabel = (r: Role) => (r === "primary" ? "Machine principale" : "Machine pilotée");
