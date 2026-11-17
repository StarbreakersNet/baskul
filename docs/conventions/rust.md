# Rust

Les règles marquées **`lint`** sont vérifiées par `cargo clippy` ou `cargo fmt`. Les règles communes (nommage, langue, commentaires) sont dans [`AGENTS.md`](../../AGENTS.md).

## Les trois crates

| Crate | Dossier | Contenu | Dépend de |
|---|---|---|---|
| `baskul-core` | `core/` | Réseau, protocole, chiffrement, logique de la principale et des pilotées, presse-papiers, configuration, état, traductions | Aucun système, aucune interface |
| `baskul-desktop` | `src/` | Couche système (`src/platform/`) : capture et injection du clavier et de la souris, bulle, actions de l'icône. API locale, ligne de commande, journaux, mises à jour | `baskul-core` |
| `baskul` | `app/src-tauri/` | L'exécutable Tauri : démarre Baskul, dessine l'icône, ouvre la fenêtre de réglages | `baskul-desktop`, Tauri |

- **Le cœur ne dépend de rien d'autre** : ni d'un système, ni de Tauri, ni de l'API. Ce dont il a besoin, la couche système le lui fournit par des traits (`primary::Host`, `secondary::Emulator`) et des fonctions (`services::ClipboardHooks`).
- **Le cœur et la couche système n'attendent jamais l'interface.** Sous Windows et Linux, la couche système tourne dans son propre thread, avec sa propre boucle de messages ; sur macOS, sur la boucle principale. Un crochet clavier ou souris qui tarde est retiré par le système.
- **L'app reste mince** : elle relaie, elle ne décide pas. Une règle qui vaut pour tous les systèmes va dans `baskul-desktop` (voir `src/tray.rs`) ou dans le cœur.
- **Le code propre à un système** est dans `src/platform/<système>.rs`, et nulle part ailleurs. `./scripts/xcheck.sh` le vérifie depuis Linux.

## Code

- `cargo fmt` décide de la mise en forme. **`lint`**
- `cargo clippy -- -D warnings` sans avertissement ; un `#[allow(…)]` dit pourquoi en une ligne. **`lint`**
- **Fonctions courtes et unitaires** : une fonction fait une seule chose, et son nom dit laquelle. Un bloc qu'on voudrait précéder d'un commentaire pour dire ce qu'il fait devient une fonction, et le commentaire son nom.
- **Imbrication** : au-delà de trois niveaux de blocs, on extrait une fonction. `let … else`, `?` et le retour anticipé restent idiomatiques en Rust, contrairement au front.
- **Erreurs** : `Result` et `?` ; `unwrap()` et `expect()` seulement quand l'échec est impossible, avec la raison dans `expect`. Un verrou empoisonné se récupère (`unwrap_or_else(|e| e.into_inner())`) : Baskul ne doit pas s'arrêter pour un thread qui a paniqué.
- **`unsafe`** : le bloc le plus petit possible, précédé de `// SAFETY:` qui dit pourquoi l'appel est sûr. **`lint`**
- **Tests** : la logique (cœur, modèle de l'icône, mises à jour) est testée sans système ni réseau réel quand c'est possible, avec un faux `Host` ou un faux `Emulator`.

## Textes et journaux

- **Textes affichés** (menus, état de la connexion, erreurs montrées à l'utilisateur) : par `i18n::t("…")` et `i18n::tf`, depuis la section `daemon` de `locales/*.json`. Jamais en dur.
- **Journaux** : `log::error!`, `warn!`, `info!`, `debug!`, en français. Une valeur citée l'est avec `{:?}`, qui l'écrit entre guillemets droits : `log::info!("passage sur {name:?}")`.

## Commentaires

Les règles d'[`AGENTS.md`](../../AGENTS.md) s'appliquent : par défaut, pas de commentaire. En Rust en particulier :

- `//!` en tête d'un module : son rôle et ce qui ne se voit pas dans le code (contrainte d'un système, choix d'architecture), en quelques lignes.
- `///` sur un élément public seulement quand son nom ne suffit pas : unité, cas limite, contrainte d'appel.
- `// SAFETY:` devant chaque `unsafe`, toujours.
- Pas de commentaire qui paraphrase le code, nomme un champ déjà clair ou raconte l'historique.
