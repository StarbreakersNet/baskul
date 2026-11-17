import { defineStore } from "pinia";
import { ref, watch } from "vue";
import { readLocalJson, writeLocalJson } from "@/utils/localSetting";

export interface BackgroundTuning {
  speed: number;
  /** Opacité du cadre (barre latérale, en-tête, pied de page), de 0 à 1 */
  frameOpacity: number;
  /** Flou sous le cadre (px) */
  frameBlur: number;
  /** Pas de la grille de points (px) */
  spacing: number;
}

const STORAGE_KEY = "baskul.backgroundTrial";

export const DEFAULT_TUNING: BackgroundTuning = {
  speed: 0.7,
  frameOpacity: 0.8,
  frameBlur: 10,
  spacing: 6,
};

// Réglages d'essai du fond animé, réglables seulement en développement (yarn tauri dev, yarn dev:mock).
// Ailleurs, toujours les valeurs par défaut : celles qu'un build d'essai aurait gardées sont ignorées
export const useBackgroundTuningStore = defineStore("backgroundTuning", () => {
  const canTune = import.meta.env.DEV;
  const tuning = ref<BackgroundTuning>(
    canTune ? readLocalJson(STORAGE_KEY, DEFAULT_TUNING) : { ...DEFAULT_TUNING }
  );

  function reset() {
    tuning.value = { ...DEFAULT_TUNING };
  }

  if (canTune) {
    watch(tuning, value => writeLocalJson(STORAGE_KEY, value), { deep: true });
  }

  return { canTune, tuning, reset };
});
