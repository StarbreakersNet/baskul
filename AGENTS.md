# AGENTS.md

Consignes communes à tous les agents de code (Claude, Codex, Copilot, Gemini…), et règles valables partout dans Baskul. `CLAUDE.md` importe ce fichier.

Baskul partage le clavier, la souris et le presse-papiers entre plusieurs ordinateurs. Trois crates Rust (le cœur `core/`, la couche système `src/`, l'app Tauri `app/src-tauri/`) et un front Vue 3 (`app/src/` : TypeScript, Pinia, Naive UI via les wrappers `h-*`). Installation, compilation et architecture : [`README.md`](README.md).

## Avant de modifier

Lire d'abord le fichier de [`docs/conventions/`](docs/conventions/README.md) qui correspond à ce qui est modifié. Les règles marquées **`lint`** y sont vérifiées par `yarn lint` (front) ou par `cargo clippy` (Rust).

| Ce qui est modifié | À lire |
|---|---|
| Du Rust (`core/`, `src/`, `app/src-tauri/`) | [`rust.md`](docs/conventions/rust.md) |
| Un composant ou une vue (`.vue`) | [`composants.md`](docs/conventions/composants.md) |
| Un wrapper de `app/src/components/c/`, ou un composant Naive UI sans wrapper | [`wrappers.md`](docs/conventions/wrappers.md) |
| Des styles : bloc `<style>`, `app/src/assets/` | [`styles.md`](docs/conventions/styles.md) |
| Un store, un composable, `app/src/utils/`, ou un nouveau fichier du front | [`organisation.md`](docs/conventions/organisation.md) |

## Avant de rendre la main

- **Front**, dans `app/` : `yarn format`, `yarn lint` et `yarn type-check`, sans nouvelle erreur ni nouvel avertissement dans les fichiers touchés.
- **Rust** : `cargo fmt --all`, `cargo clippy -p baskul-core -p baskul-desktop --all-targets -- -D warnings`, `cargo test -p baskul-core -p baskul-desktop` et `./scripts/xcheck.sh` (code Windows et macOS vérifié depuis Linux).
- Si une règle de la charte semble inadaptée, le signaler plutôt que la contourner.

## Nommage

| Élément | Convention | Exemples |
|---|---|---|
| Composant | PascalCase, au moins deux mots | `SettingRow.vue`, `HotkeyInput.vue` |
| Vue | suffixe `View` | `HomeView.vue` |
| Wrapper | `hXxx.vue`, importé `HXxx`, balise `<h-xxx>` | `hButton.vue` |
| Variable, fonction (TypeScript) | camelCase, en anglais | `selectedName`, `startRefreshing` |
| Constante de module (TypeScript) | UPPER_SNAKE_CASE | `REFRESH_MS` |
| Booléen | `isXxx`, `hasXxx`, `canXxx`, `showXxx` (Rust : `is_xxx`, `has_xxx`…) | `isRestarting`, `showFooter` |
| Loader (`useLoader`) | L'action en cours, en *-ing* | `saving`, `switching` |
| Gestionnaire d'événement | `handleXxx`, ou le verbe de son action ; jamais `onXxx` | `handleKeyDown`, `generateSecret` |
| Ref de template | suffixe `Ref` | `inputRef` |
| Type, interface | PascalCase ; `interface` pour un objet, `type` pour une union | `DaemonStatus`, `Phase` |
| Store | `useXxxStore`, fichier et id identiques | `stores/status.ts`, `useStatusStore`, `"status"` |
| Événement émis | camelCase, au participe passé | `saved`, `itemClick` |
| Classe CSS | kebab-case, préfixée par le nom du composant | `.layout-tile`, `.home-title` |
| Rust | conventions de Rust : `snake_case`, `CamelCase` pour les types, `SCREAMING_SNAKE_CASE` pour les constantes | `remote_abs_motion`, `HookWatch`, `ACTIVITY_INTERVAL` |

`yarn lint` vérifie ces noms dans le front, sauf ceux des booléens et des classes CSS, et le participe passé des événements. Pour les gestionnaires d'événement, il refuse seulement `onXxx`.

## Langue, textes et commentaires

- Identifiants en anglais. Commentaires et journaux en français, avec des guillemets droits : `"Retirer"`, pas `« Retirer »`. **`lint`** (front)
- **Textes affichés** : jamais en dur dans le code. Ils sont dans `locales/fr.json` et `locales/en.json` (section `app` pour la fenêtre, `daemon` pour les menus et l'état), avec des guillemets droits eux aussi.
- **Par défaut, pas de commentaire.** Un commentaire n'existe que si, sans lui, quelqu'un qui lit ou modifie le code mergé se tromperait : une contrainte qui ne se voit pas dans le code, un comportement surprenant de Windows, de macOS, de Naive UI ou de Tauri, un choix de sécurité. Par exemple, dans `hRadioButton.vue` : `NRadioGroup` reconnaît ses boutons à leur nom de composant.
- **Le test : servira-t-il dans six mois à quelqu'un qui découvre le fichier ?** Un commentaire qui n'a servi que pendant le développement ou le débogage est retiré avant la PR.
- **Jamais de commentaire pour** :
  - dire ce que fait le code, ou ce qu'est un champ ou une fonction dont le nom le dit déjà ;
  - rappeler le fonctionnement de Vue, de Naive UI, de Rust ou du navigateur ;
  - raconter l'historique du développement : ce qui existait avant, ce qui a été essayé, pourquoi on n'a pas fait autrement ;
  - répéter une règle de la charte ou le message d'une règle de lint.
- Exception en Rust : chaque bloc `unsafe` garde son commentaire `// SAFETY:`, qui dit pourquoi l'appel est sûr.
- Un commentaire tient en une ligne si possible.
- Pas de code commenté : l'historique git le garde.
- Pas de `console.log` oublié. **`lint`** `console.warn` et `console.error` restent pour un vrai diagnostic. En Rust, pas de `println!` ni de `dbg!` : les journaux passent par `log::…`.

## À éviter (front)

| À éviter | À la place |
|---|---|
| `defineProps({ size: { type: String as PropType<…> } })` | `defineProps<{ size?: … }>()` |
| `props.size` | `size`, déstructuré |
| `defineEmits(["saved"])` | `defineEmits<{ saved: [] }>()` |
| Prop `value` + émission `update:value` à la main | `defineModel` |
| Modifier une prop, même en profondeur (`config.role = …`) | `defineModel("config")` et `v-model:config` côté parent |
| `<n-button>` dans une vue ou un composant | `<h-button>`, ou un nouveau wrapper |
| Wrapper qui recopie chaque prop Naive | `extends /* @vue-ignore */ XxxProps` |
| Booléen Naive redéclaré sans défaut (`const { disabled } = …`) | `disabled = undefined` |
| `useAttrs()` lu dans un `computed` | `v-bind="$attrs"` dans le template |
| `style="margin-top: 8px"` | Une classe nommée en SCSS scoped |
| `const inputRef = ref()` pour une ref de template | `useTemplateRef("inputRef")` |
| `const handleClick = () => {}` | `function handleClick() {}` |
| `if (!isReady) { return; }`, puis la suite | `if (isReady) { … }`, la suite dans le `if` |
| Quatre niveaux de blocs imbriqués | Une fonction nommée pour le bloc intérieur |
| `watch` qui recopie une valeur dérivée | `computed` |
| `Object.assign`, déstructuration ou `toRefs` sur un `reactive` | Un `ref` d'objet : `form.value = { … }` |
| `any` | Le type exact, ou `unknown` |
| `import … from "@/stores/status.ts"`, ou `../stores/status` | `import … from "@/stores/status"` |
| `@click="$router.push(…)"` | Une fonction qui appelle `router.push` |
| `daemonClient.…` ou `invoke(…)` dans un composant | Une action de store |
| `useMessage()` ou `useNotification()` de `naive-ui` | `useAppMessage()` |
| `const saving = ref(false)`, passé à `true` puis à `false` | `const saving = useLoader()` et `await saving.run(…)` |
| `<style scoped></style>` vide | Pas de bloc `<style>` |
