# Conventions de code de Baskul

Comment on écrit Baskul : le Rust (cœur, couche système, app Tauri) et le front Vue de la fenêtre de réglages. Ces conventions s'appliquent à tout code écrit ou modifié, par un humain comme par un agent de code (Claude, Codex, Copilot, Gemini…). Les agents y arrivent par [`AGENTS.md`](../../AGENTS.md), à la racine du dépôt, qui contient les règles communes et dit quel fichier lire selon ce qu'on modifie.

Les conventions du front reprennent celles du site des Starbreakers (`starbreakersnet-website`), adaptées à Baskul : pas de Tailwind ni de Supabase, Baskul joue le rôle du backend.

Les règles marquées **`lint`** sont vérifiées automatiquement : par `yarn lint` pour le front (config dans `app/eslint.config.ts`, et `app/scripts/check-names.mjs` pour les noms de fichiers), par `cargo clippy` pour le Rust. Inutile de les surveiller en relecture. Les autres relèvent de la relecture. Pour faire évoluer une règle : une PR qui modifie ces fichiers (et le lint si besoin), comme pour le reste du code.

## Sommaire

- [Règles communes](../../AGENTS.md) : nommage, langue et commentaires, tableau "À éviter"
- [Rust](rust.md) : cœur, couche système, app Tauri
- [Écrire un composant](composants.md) : anatomie, script, chargements, template
- [Wrappers `components/c/`](wrappers.md)
- [Styles](styles.md)
- [Organisation du front](organisation.md) : dossiers de `app/src/`, stores, composables et `utils/`

## Outils

| Commande | Rôle |
|---|---|
| `yarn format` (dans `app/`) | Prettier sur `src/` : indentation, guillemets, retours à la ligne |
| `yarn lint` (dans `app/`) | oxlint et ESLint, avec correction automatique de ce qui peut l'être, puis les noms de fichiers |
| `yarn type-check` (dans `app/`) | `vue-tsc` sur tout le front |
| `yarn test` (dans `app/`) | Tests du front (`app/tests/`) : `utils/`, cohérence des traductions |
| `cargo fmt --all` | Mise en forme du Rust (`rustfmt.toml`) |
| `cargo clippy -p baskul-core -p baskul-desktop --all-targets -- -D warnings` | Lint du Rust |
| `cargo test -p baskul-core -p baskul-desktop` | Tests, dont des tests de bout en bout sur de vraies connexions |
| `./scripts/xcheck.sh` | Vérification du code Windows et macOS depuis Linux |

**Avant d'ouvrir une PR** : ces commandes, sans nouvelle erreur ni nouvel avertissement dans les fichiers touchés. La CI (workflow `checks`) refait ces vérifications à chaque PR, et toute erreur la fait échouer.

La mise en forme ne se discute pas en relecture : Prettier (`app/.prettierrc.yaml` : 2 espaces, guillemets doubles, points-virgules, 100 colonnes) et `rustfmt` décident. Dans l'IDE :

- **WebStorm / IntelliJ / RustRover** : *Settings › Languages & Frameworks › JavaScript › Prettier* : "Automatic configuration" et "Run on save" ; *› Code Quality Tools › ESLint* : "Automatic configuration" et "Run eslint --fix on save" ; *Rust › Rustfmt* : "Run rustfmt on save".
- **VS Code** : extensions *Vue - Official*, *ESLint*, *Prettier* et *rust-analyzer*, avec "Format on save".
