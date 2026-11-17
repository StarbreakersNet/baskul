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

## Branches

Une branche se nomme `type/sujet`, avec les mêmes types que les commits et un sujet court en kebab-case : `feat/profils`, `fix/souris-qui-remonte`, `ci/builds-rapides`, `docs/identite`. La portée n'y figure pas.

## PR

- **Une PR part de `main`**, pour être relue et fusionnée seule, dans n'importe quel ordre.
- Elle n'est **empilée** sur une autre PR ouverte que si elle en dépend vraiment (elle utilise son code, ou modifie les mêmes lignes). Sa base est alors la branche dont elle dépend, et sa description commence par le dire, avec le lien de cette PR : voir [PR empilées](#pr-empilées).
- Le titre de la PR suit le même format : la PR est fusionnée par un commit de fusion (*merge commit*), dont le message est le titre de la PR suivi de sa description (pour une pile, voir plus bas).
- **En retard sur `main` ou en conflit avec lui**, une PR qui n'est pas dans une pile reçoit `main` par un merge : bouton "Update branch", éditeur de conflits de GitHub, ou en local.
- **Pas de mention de l'outil** qui a écrit le code : ni "Generated with…" ni lien vers une session, dans la description d'une PR comme dans un commentaire. Les commits parlent d'eux-mêmes ; seule la ligne `Co-Authored-By:` d'un agent de code est admise, à la fin du message de commit.
- Un commit, un changement : un correctif glissé dans une nouvelle fonction a son propre commit.
- Les commits de la PR restent tels quels dans l'historique de `main` : pas de commit "wip" ni "fix lint" dans une PR prête à relire, on les fusionne avant (`git commit --fixup`, puis `git rebase --autosquash`).

## PR empilées

Une pile est une suite de PR dont chacune part de la branche de la précédente : la première part de `main`, la dernière est celle sur laquelle rien n'est construit. Elle est déclarée comme pile sur GitHub (dans l'interface, ou avec `gh stack`).

- **Trois PR au plus.** Au-delà, on regroupe des PR.
- **Jamais de merge dans une branche de pile**, ni d'une autre branche de la pile ni de `main` : elle se met à jour par rebase, avec "Update with rebase" sur la PR. Un conflit résolu dans un commit de merge est perdu au rebase suivant, et revient.
- **Un conflit de rebase se résout en local** (`gh stack rebase`, puis `gh stack rebase --continue`) : l'éditeur de conflits de GitHub ferait un merge. Les branches sont ensuite poussées avec `git push --force-with-lease`, permis sur les branches d'une pile, jamais sur `main`.
- **Une branche où `main` a été fusionné ne sert pas de base à une pile** : elle est d'abord fusionnée dans `main`, et la suite part de `main`.
- **Une pile se fusionne d'un coup**, avec "Merge stack" depuis sa dernière PR. Dans la fenêtre, le titre et la description pré-remplis, ceux de la dernière PR, sont remplacés par le message de la pile : un titre au même format qui la résume, avec les numéros des PR, puis une ligne par PR. Ce message est préparé à la fin de la description de la dernière PR, prêt à copier :

  ```text
  chore: ignorer les fichiers des systèmes et des éditeurs (#37, #38, #39)

  - #37 ignorer les fichiers que Windows dépose dans les dossiers
  - #38 séparer les fichiers du système et ceux des éditeurs
  - #39 ignorer les fichiers temporaires de Vim et d'Emacs
  ```

- Une pile peut aussi se fusionner PR par PR, du bas vers le haut : GitHub rebase les suivantes, et chacune a son propre commit de fusion. On attend que la CI repasse après chaque rebase.
