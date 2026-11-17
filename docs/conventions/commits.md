# Commits et PR

Baskul suit [Conventional Commits](https://www.conventionalcommits.org/fr/v1.0.0/) : chaque message de commit, et chaque titre de PR, commence par un type qui dit ce que fait le changement. La CI le vérifie (workflow `checks`, script `scripts/check-commits.mjs`).

## Format

```
type(portée): description

Corps, facultatif : pourquoi ce changement, ce qu'il faut savoir pour le relire.

BREAKING CHANGE: ce qui casse, et comment s'adapter.
```

- **type** : en anglais, en minuscules, dans la liste ci-dessous.
- **portée** : facultative, en minuscules, dans la liste ci-dessous.
- **description** : en français, à l'impératif ou au présent, sans majuscule au début (sauf un nom propre) ni point à la fin, 72 caractères au plus pour toute la ligne. Elle dit ce que fait le commit, pas comment.
- `!` après le type ou la portée (`feat(core)!: …`) signale un changement incompatible : configuration à réécrire, protocole réseau changé (les deux machines doivent être à jour), commande retirée. Le corps l'explique avec `BREAKING CHANGE:`.

## Types

| Type | Pour | Version |
|---|---|---|
| `feat` | Une nouvelle fonction visible par l'utilisateur | mineure (0.**8**.0) |
| `fix` | Un bug corrigé | correctif (0.8.**1**) |
| `perf` | Plus rapide ou plus léger, sans changement visible | correctif |
| `refactor` | Code réorganisé, sans changement de comportement | — |
| `style` | Mise en forme seule (Prettier, rustfmt) | — |
| `test` | Tests ajoutés ou corrigés | — |
| `docs` | README, `docs/`, `AGENTS.md`, commentaires | — |
| `build` | Dépendances, scripts de build, `Cargo.toml`, `package.json`, Tauri | — |
| `ci` | Workflows GitHub Actions | — |
| `chore` | Le reste : version publiée, fichiers ignorés… | — |
| `revert` | Annule un commit précédent : `revert: feat(front): …` | — |

## Portées

| Portée | Dossier |
|---|---|
| `core` | `core/` : réseau, protocole, logique de la principale et des pilotées, configuration |
| `desktop` | `src/` : couche système, API locale, ligne de commande, mises à jour |
| `windows`, `macos`, `linux` | Code propre à un système (`src/platform/`) |
| `app` | `app/src-tauri/` : exécutable Tauri, icône |
| `front` | `app/src/` : fenêtre de réglages |
| `i18n` | `locales/` |
| `deps` | Mise à jour de dépendances |
| `release` | Publication d'une version |

Un changement qui touche plusieurs dossiers prend la portée de l'essentiel du travail, ou aucune.

## Exemples

```
feat(front): activité récente sur l'accueil
fix(windows): la souris ne remonte plus après un jeu qui la capture
perf(core): un seul paquet par mouvement de souris
build(deps): Tauri 2.12
ci: builds de test sans LTO
feat(core)!: secret partagé sur 24 caractères
chore(release): 0.8.0
```

## PR

- Le titre de la PR suit le même format : c'est le message du commit quand la PR est fusionnée en un seul commit (*squash*).
- Un commit, un changement : un correctif glissé dans une nouvelle fonction a son propre commit.
- Pas de commit "wip" ni "fix lint" dans une PR prête à relire : on les fusionne avant (`git commit --fixup`, puis `git rebase --autosquash`).
