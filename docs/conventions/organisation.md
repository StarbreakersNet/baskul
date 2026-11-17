# Organisation du front

Les règles marquées **`lint`** sont vérifiées par `yarn lint`.

## Dossiers de `app/src/`

| Dossier | Contenu | Nommage |
|---|---|---|
| `App.vue`, `AppLayout.vue` | `App.vue` : providers Naive UI (thème, langue, messages, dialogues). `AppLayout.vue` : cadre de la fenêtre (menu latéral, en-tête, pied de page) | |
| `components/c/` | Wrappers des composants Naive UI, réduits au minimum : seulement ce qu'ils ajoutent ou modifient ([voir wrappers.md](wrappers.md)). Aucune logique métier | `hXxx.vue`, importé `HXxx`, utilisé `<h-xxx>` |
| `components/` | Composants de l'application ; `components/settings/` pour les sections des réglages | PascalCase, au moins deux mots : `SettingRow.vue`, `ConnectionSection.vue` |
| `views/` | Pages routées | `XxxView.vue` |
| `stores/` | Stores Pinia, seuls à appeler Baskul | `status.ts` → `useStatusStore()`, id `"status"` |
| `composables/` | Fonctions `useXxx()` liées à l'application en cours : état, service (messages), réactivité ou cycle de vie de Vue | `useXxx.ts` |
| `utils/` | Fonctions et constantes TypeScript indépendantes de l'application en cours (calculs, formats, constantes), clients (`daemonClient`, traductions). Un fichier par sujet | camelCase, au nom du sujet : `hotkey.ts`, jamais `utils.ts` ni `helpers.ts` |
| `types/` | Types partagés, un fichier par sujet ; `types/components/` pour les types publics des composants (`HLogInst`) | PascalCase : `Daemon.ts` |
| `models/` | Données statiques et leur type, s'il en faut. Pas de fonction : elles vont dans `utils/` **`lint`** | PascalCase, au nom du type. Constantes en UPPER_SNAKE_CASE **`lint`** |
| `assets/` | Styles globaux (`variables.scss`, `main.scss`), logo | |
| `router/` | Routes | |

Les traductions sont hors de `app/`, dans `locales/` à la racine du dépôt : la section `app` est celle de la fenêtre, la section `daemon` celle des menus de Baskul (Rust). On les importe par l'alias `@locales/`.

Les noms de fichiers de ces dossiers sont vérifiés par `yarn lint` (`scripts/check-names.mjs`). **`lint`**

## Stores, composables et `utils/`

- **Stores** : Pinia en mode setup (`defineStore("status", () => { … })`), avec un `return` qui liste explicitement ce qui est public. Dans un composant : `const statusStore = useStatusStore()`. On ne déstructure pas un store (on perdrait la réactivité), sauf avec `storeToRefs`.
- **Baskul ne s'appelle que depuis les stores**, par `utils/daemonClient` (les commandes Tauri de `app/src-tauri/src/main.rs`). **`lint`** Une action de store affiche elle-même son message d'erreur et renvoie un booléen. Le composant décide seulement de ce qu'il change à l'écran.
- **Messages à l'utilisateur** : `useAppMessage()` de `@/composables/`, qui applique les réglages du projet et marche aussi dans les stores. Jamais le `useMessage` de Naive UI : seul `AppLayout`, placé sous son provider, l'appelle pour transmettre son API au composable. **`lint`**
- **Composable** : une fonction `useXxx()` qui donne à un composant ou à un store ce dont il a besoin : un état (`useLoader`), un service de l'application (`useAppMessage`, `useCopy`) ou un comportement lié au cycle de vie de Vue (`useCurrentTime`). Une fonction qui calcule seulement un résultat à partir de ses paramètres va dans `utils/`.
- **Pas d'état partagé dans une variable de module** : c'est le rôle d'un store.
- **Natif d'abord** : `Array.prototype.find`, `some`, `structuredClone`… plutôt qu'une bibliothèque.
