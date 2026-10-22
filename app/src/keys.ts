import type { InjectionKey, Ref } from "vue";
import type { Role } from "./types";

/** Rôle choisi pour cette machine : les réglages de l'autre rôle sont masqués. */
export const roleKey: InjectionKey<Readonly<Ref<Role>>> = Symbol("role");
/** Système de cette machine (`macos`, `windows`, `linux`). */
export const osKey: InjectionKey<Readonly<Ref<string>>> = Symbol("os");
