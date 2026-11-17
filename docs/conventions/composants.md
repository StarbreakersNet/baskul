# Écrire un composant

Les règles marquées **`lint`** sont vérifiées par `yarn lint`.

## Anatomie d'un composant

Un composant complet, conforme à toutes les règles qui suivent :

```vue
<script setup lang="ts">
import { computed } from "vue";
import HButton from "@/components/c/hButton.vue";
import HCard from "@/components/c/hCard.vue";
import HFlex from "@/components/c/hFlex.vue";
import HText from "@/components/c/hText.vue";
import { useLoader } from "@/composables/useLoader";
import { useStatusStore } from "@/stores/status";
import { osLabel } from "@/utils/os";
import type { Peer } from "@/types/Daemon";

const isExpanded = defineModel<boolean>("expanded", { default: false });

const { peer, canSwitch = false } = defineProps<{
  peer: Peer;
  /** Affiche le bouton "Aller sur" */
  canSwitch?: boolean;
}>();

const emit = defineEmits<{
  switched: [peer: Peer];
}>();

const statusStore = useStatusStore();

const switching = useLoader();

const subtitle = computed(() => `${osLabel(peer.os)} · ${peer.name}`);

async function handleSwitch() {
  if (await switching.run(() => statusStore.switchTo(peer.name))) {
    emit("switched", peer);
  }
}
</script>

<template>
  <h-card class="peer-card" size="small">
    <h-flex justify="space-between">
      <h-text strong>{{ peer.name }}</h-text>
      <h-flex>
        <h-button quaternary size="small" @click="isExpanded = !isExpanded">{{ $t("common.details") }}</h-button>
        <h-button v-if="canSwitch" :loading="switching.loading" size="small" @click="handleSwitch()">
          {{ $t("home.goTo") }}
        </h-button>
      </h-flex>
    </h-flex>
    <p v-if="isExpanded" class="peer-card-subtitle">{{ subtitle }}</p>
  </h-card>
</template>

<style lang="scss" scoped>
@use "@/assets/variables";

.peer-card {
  border-radius: variables.$radius;
}

.peer-card-subtitle {
  margin-top: 8px;
}
</style>
```

- **Ordre des blocs** : `<script setup lang="ts">`, puis `<template>`, puis `<style lang="scss" scoped>`. **`lint`**
- **Pas de bloc vide** : un composant sans style n'a pas de `<style>`. **`lint`**
- **Ordre dans le script** :
  1. imports **`lint`** ;
  2. types locaux (`interface Props`…) ;
  3. macros, dans l'ordre `defineOptions`, `defineModel`, `defineProps`, `defineEmits`, `defineSlots` **`lint`** ;
  4. constantes du module (`REFRESH_MS`) ;
  5. stores, router et composables (`const statusStore = useStatusStore()`) ;
  6. état (`ref`, `reactive`) et valeurs dérivées (`computed`) ;
  7. fonctions ;
  8. `watch` ;
  9. hooks de cycle de vie (`onMounted`…) ;
  10. `defineExpose` **`lint`**.

  Dans un composant long, on regroupe plutôt par fonctionnalité (état, `computed` et fonctions du glisser-déposer ensemble), avec un commentaire en tête de groupe.

## Script

### TypeScript

- Toujours `<script setup lang="ts">` : ni Options API, ni `<script>` classique, ni JavaScript. **`lint`**
- Pas de `any` **`lint`** : on prend le type exact (Naive UI exporte les siens : `ButtonProps`, `FormRules`…) ou `unknown` qu'on affine ensuite.
- `import type` pour ce qui n'est qu'un type (imposé par `verbatimModuleSyntax`, `yarn type-check` le signale).
- Imports internes par l'alias `@/`, sans extension `.ts` : `@/stores/status`. Les traductions, hors de `app/`, passent par l'alias `@locales/`. **`lint`**
- Ordre des imports conseillé : `vue` et bibliothèques, puis composants `@/components/…`, puis le reste de `@/…`, puis les `import type`.

### Props

Props **typées** et **déstructurées** (syntaxe Vue 3.5). **`lint`**

```ts
const { size = "medium", disabled = false, tooltip } = defineProps<{
  size?: ButtonProps["size"];
  disabled?: boolean;
  /** Texte de l'infobulle, sans infobulle si vide */
  tooltip?: string;
}>();
```

- Le type est la seule source de vérité : pas de `PropType` **`lint`**. Le compilateur génère la déclaration runtime.
- Les **valeurs par défaut** s'écrivent dans la déstructuration. Une prop obligatoire n'a pas de `?`.
- Au-delà de quelques props, ou pour étendre un type, on déclare une `interface Props` au-dessus des macros.
- Un commentaire `/** … */` sur chaque prop dont le nom ne suffit pas : l'IDE l'affiche au survol, côté parent.
- Les props s'utilisent **directement**, dans le script comme dans le template : `size`, jamais `props.size`. Les variables déstructurées restent réactives.
- Pour un `watch` ou un composable, on passe un getter : `watch(() => size, …)`. Écrire `watch(size, …)` provoque une erreur de compilation.
- **Booléens** : une prop booléenne absente vaut `false`. Pour qu'elle reste `undefined` (wrapper qui laisse Naive UI décider), on écrit `disabled = undefined` dans la déstructuration. Écrire `boolean | undefined` dans le type ne suffit pas.
- On ne modifie jamais une prop. **`lint`** On émet un événement ou on utilise `defineModel`.
- **Dans un wrapper**, on ne redéclare pas les props Naive : on ne déclare que ce que le wrapper ajoute ou modifie, et le reste passe par `$attrs`, typé. Voir [wrappers.md](wrappers.md).

### Événements

```ts
const emit = defineEmits<{
  connected: [];
  select: [key: string];
}>();
```

- Déclaration typée, avec la syntaxe en tuples nommés ci-dessus. **`lint`**
- Le nom dit ce qui s'est passé, pas ce que le parent doit faire : `saved`, `switched`, `removed`.
- Noms en camelCase dans le script (`emit("itemClick")`), écoutés en kebab-case dans le template (`@item-click`). **`lint`**
- On émet avec la constante `emit`, y compris dans le template, plutôt qu'avec `$emit`. **`lint`**
- **Dans un wrapper**, on ne redéclare pas les événements Naive : ils passent par `$attrs`, typés. `defineEmits` ne sert qu'aux événements que le wrapper ajoute ou intercepte. Voir [wrappers.md](wrappers.md).

### v-model

- `defineModel` plutôt qu'une prop et un événement `update:xxx` écrits à la main. **`lint`**
- Composant de l'application : `defineModel()` donne `v-model`, `defineModel("expanded")` donne `v-model:expanded`. Le type se passe en paramètre, `defineModel<boolean>("expanded")`, jamais par l'option `type`. **`lint`**
- Dans un wrapper, v-model passe tout seul par `$attrs` ([voir wrappers.md](wrappers.md)). On ne déclare `defineModel("value")` que si le wrapper doit lire ou modifier la valeur, et on garde alors le nom de Naive UI (`value`, `show`, `checked`).

### État et réactivité

- `reactive` pour une **collection** de valeurs liées :
  - un objet qui les regroupe : modèle de formulaire passé à `<h-form :model>`, filtres, options d'affichage ;
  - une liste qu'on modifie sur place, avec `push` ou `splice`.
- `ref` pour une valeur seule (booléen, texte, nombre, élément sélectionné), et pour une liste qu'on remplace d'un bloc (`items.value = await …`).
- Un `reactive` se modifie **champ par champ, et seulement comme ça** : ni `Object.assign` **`lint`**, ni déstructuration, ni `toRefs` **`lint`**. Si on a besoin de le remplacer d'un bloc (valeurs rechargées depuis le serveur, remise à zéro) ou d'en extraire des morceaux, c'est qu'il fallait un `ref` d'objet : `const filters = ref({ … })`, puis `filters.value = { … }`.
- Une valeur qui se déduit d'une autre est un `computed`, jamais un `watch` qui recopie.
- **Les données d'un parent se modifient par `defineModel`**, jamais en modifiant une prop, même en profondeur : les sections des réglages reçoivent `v-model:config` et écrivent dans `config.value`.
- Un `watch` qui doit aussi s'exécuter au montage prend `{ immediate: true }` : on ne recopie pas son code dans `onMounted`.
- Refs de template : `useTemplateRef<Type>("formRef")` **`lint`**, avec un nom qui finit par `Ref` **`lint`**. Un `ref="…"` inutilisé est retiré. **`lint`**
- Tout ce qui est démarré au montage (`setInterval`, écouteur sur `window`) est arrêté dans `onBeforeUnmount`. Les vues restent en mémoire (`keep-alive`) : on démarre aussi dans `onActivated` et on arrête dans `onDeactivated` (voir `composables/useCurrentTime.ts`).
- Pas d'état partagé dans une variable de module (`let x` hors du composant) : c'est le rôle d'un store.

### Fonctions

- Déclarées avec `function` **`lint`**. Les fonctions fléchées restent pour les callbacks en ligne (`items.filter(item => …)`).
- Gestionnaires d'événements : `handleXxx` (`handleKeyDown`, `handlePointerUp`), ou le verbe de l'action quand la fonction ne fait que ça (`generateSecret`, `goTo`). Jamais `onXxx` **`lint`**. Pour une modale : `openXxx` et `closeXxx`.
- `async` / `await`, pas `.then()` **`lint`**. Les indicateurs de chargement passent par `useLoader()` (voir [Chargements](#chargements)).
- **Courtes et unitaires** : une fonction fait une seule chose, et son nom dit laquelle (`isListeningToDaemon`, `applyCaptureState`). Un gestionnaire `handleXxx` se lit comme un résumé : il vérifie les conditions et confie le travail à des fonctions nommées.
- **Conditions en positif, et le code qui en dépend dans le `if`** : l'indentation montre ce qui ne s'exécute qu'à cette condition. Pas de sortie anticipée : ni `return;` vide **`lint`**, ni `continue` dans une boucle **`lint`**.
- **Au plus trois niveaux de blocs imbriqués** **`lint`**. Tous les blocs comptent (`if`, `for`, `switch`, `try`…), mais `else` et `else if` n'ajoutent pas de niveau, et un callback (`.some(item => …)`, `saving.run(async () => …)`) repart de zéro. Au-delà, on extrait une fonction nommée : l'imbrication qui grandit est le signal qu'une fonction fait trop de choses.
- **Deux autres signaux qu'il faut extraire** : un nom de fonction qui demanderait un « et », ou un bloc qu'on voudrait précéder d'un commentaire pour dire ce qu'il fait. Le bloc devient une fonction, et le commentaire son nom.
- **`return valeur`** dès que le résultat est connu, y compris dans chaque `case` d'un `switch` (`describeUpdate` dans `AboutView`). Dans une fonction qui ne renvoie rien, chaque `case` se termine par `break`. **`lint`**

La saisie d'un raccourci de `HotkeyInput`, une fois découpée :

```ts
function isListeningToDaemon() {
  return isRecording.value && isDaemonCapturing.value;
}

/** Renvoie `true` tant que Baskul attend la combinaison */
function applyCaptureState(state: CaptureState | null) {
  let isWaiting = false;
  if (state?.state === "done") {
    finish(state.hotkey);
  } else if (state?.state === "waiting") {
    isWaiting = true;
  } else {
    stop();
  }
  return isWaiting;
}

async function waitForDaemonHotkey() {
  let isWaiting = true;
  while (isWaiting && isListeningToDaemon()) {
    await new Promise(resolve => setTimeout(resolve, POLL_MS));
    if (isListeningToDaemon()) {
      const state = await hotkeyCapture.poll();
      isWaiting = isListeningToDaemon() && applyCaptureState(state);
    }
  }
}
```

### Chargements

Un indicateur de chargement est un `useLoader()` de `@/composables/useLoader`, jamais un `ref(false)` qu'on passe à `true` puis à `false`.

```ts
const switching = useLoader();

async function goTo(target: string) {
  await switching.run(() => statusStore.switchTo(target));
}
```

```vue
<h-button :loading="switching.loading" @click="goTo(machine.target)">{{ $t("home.goTo") }}</h-button>
```

- **`run()` par défaut** : le loader reste actif le temps de la tâche et s'arrête même si elle lève une exception. `run()` renvoie le résultat de la tâche, typé. On écrit toujours `await` devant. **`lint`**
- Pour plusieurs `await` à la suite : `await saving.run(async () => { … })`.
- **Appels qui se chevauchent** : le loader compte les appels en cours, et il reste actif jusqu'à la fin du dernier.
- `saving.loading` (booléen) et `saving.count` (nombre) se lisent **sans `.value`**, dans le script comme dans le template. Le loader est un objet `reactive` : on ne le déstructure pas. **`lint`**
- **`start()` / `stop()`** seulement pour un chargement réparti sur plusieurs fonctions ou événements (démarré au clic, arrêté dans un `@load`). `stop()` va alors dans un `finally` dès que c'est possible.
- Plusieurs loaders à surveiller ensemble : `const busy = computed(() => isAnyLoading(saving, deleting))`. Pour une collection de loaders (un par élément d'une liste…) : `isAnyLoading(...Object.values(loaders))`.
- Nom du loader : l'action en cours, en *-ing* : `saving`, `switching`, `checking`. **`lint`**

## Template

- Balises en kebab-case : `<h-button>`, `<setting-row>`, `<layout-editor>`. **`lint`**
- **Jamais de composant Naive UI (`n-*`) hors de `components/c/`** **`lint`**. On utilise le wrapper `h-*`. S'il n'existe pas, on le crée ([voir wrappers.md](wrappers.md)) : ça prépare le terrain pour un comportement maison. Seule exception : les providers de `App.vue`.
- Attributs en kebab-case (`:primary-name`) **`lint`**. Booléen vrai en raccourci (`strong`, pas `:strong="true"`) **`lint`**. Le raccourci de même nom est permis : `:peers` pour `:peers="peers"`.
- Expressions courtes : au-delà d'une comparaison ou d'un ternaire, on passe par un `computed` ou une fonction. Un ternaire imbriqué est refusé. **`lint`**
- Événements :
  - avec un gestionnaire nommé, appelé avec parenthèses : `@click="handleSubmit()"` **`lint`** ;
  - le code en ligne est réservé aux affectations simples : `@click="showModal = true"` ;
  - pas de `$router` dans le template : la navigation se fait dans une fonction du script. **`lint`**
- `v-for` toujours avec une `:key` stable, tirée de la donnée (id, nom), pas de l'index. **`lint`** Jamais de `v-if` sur le même élément qu'un `v-for`. **`lint`**
- Textes affichés : toujours `$t("…")`, jamais en dur (voir [`AGENTS.md`](../../AGENTS.md)).
- `v-show` pour ce qui s'affiche et se masque souvent, `v-if` pour le reste : dans les deux cas dans une transition (voir plus bas).
- Quand un composant reçoit plusieurs slots, chacun a son `<template #nom>`, y compris `#default`.

## Rien n'apparaît d'un coup

Tout ce qui apparaît, disparaît ou change de place sous les yeux de l'utilisateur le fait **avec une transition**, ou **à une place réservée par un squelette** (`h-skeleton`) le temps du chargement. Un `v-if`, `v-else` ou `v-show` qui change pendant que la page est affichée est toujours dans une transition.

| Cas | Transition |
|---|---|
| Un élément qui se superpose ou ne pousse rien (étiquette, bouton dans une ligne, fantôme du glisser-déposer) | `<transition name="fade">` |
| Deux contenus à la même place (`v-if` / `v-else`) | `<transition mode="out-in" name="fade">` |
| Un bloc qui pousse ce qui suit (alerte, ligne, détail) | `<expand-transition>` : hauteur animée |
| Une liste (`v-for`) | `<expand-transition group>`, ou un `<transition-group>` nommé dans le composant |
| Une ligne de réglage selon une autre valeur | `<setting-row :show="…">` plutôt qu'un `v-if` sur la ligne |
| Des données pas encore chargées | `h-skeleton` à la place du contenu, puis `fade` en `out-in` |

- Les transitions `fade` et `expand` sont dans `assets/main.scss`, avec leur version sans mouvement (`prefers-reduced-motion`). Une transition propre à un composant se nomme avec son préfixe (`home-tile`) et se déclare dans son `<style>`.
- Pas de marge sur ce qui s'anime en hauteur : l'écart est un `padding` d'un conteneur (`<div class="home-notice">` autour de l'alerte), sinon il apparaît d'un coup. Pour la même raison, un bloc qui apparaît dans un `h-flex` à espacement (`:size`) se place hors de ce `h-flex`.
- Un enfant reconnu par son parent Naive UI (`h-grid-item`, `h-tab-pane`, `h-descriptions-item`) ne peut pas être enveloppé dans une transition : on anime son contenu, on désactive (`disabled`) au lieu de retirer, ou on passe à une grille CSS avec `<transition-group>`.
- N'est pas concerné ce qui ne change pas pendant la vie du composant (une prop de structure comme `icon` sur `setting-row`).
