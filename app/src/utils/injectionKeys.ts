import type { InjectionKey, Ref } from "vue";
import type { Role } from "@/types/Config";

/** Rôle choisi pour cette machine : les réglages de l'autre rôle sont masqués */
export const ROLE_KEY: InjectionKey<Readonly<Ref<Role>>> = Symbol("role");
/** Système de cette machine : "macos", "windows" ou "linux" */
export const OS_KEY: InjectionKey<Readonly<Ref<string>>> = Symbol("os");
