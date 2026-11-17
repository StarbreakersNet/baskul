# Identité visuelle

Baskul fait passer le clavier et la souris d'un écran à l'autre : le logo montre deux écrans et le trajet du curseur de l'un à l'autre, et l'interface reprend son violet et des panneaux qui flottent au-dessus du fond.

## Logo

- Source : `app/app-icon.svg`, copiée dans `app/src/assets/logo.svg` pour la fenêtre.
- Carré arrondi de la couleur de l'accent en clair (`#745af4`), uni, sans dégradé ni reflet ; deux écrans blancs, celui de gauche en contour, celui de droite plein ; au-dessus, trois points de taille croissante en arc de l'un à l'autre : le trajet du curseur.
- Ce sont les seules couleurs écrites hors du thème, parce qu'une image ne peut pas le lire : si l'accent change, mettre à jour les deux SVG puis régénérer les icônes des systèmes (`app/src-tauri/icons/`) :

```bash
cd app && yarn tauri icon app-icon.svg
```

## Couleurs

Toutes les couleurs sont dans le thème Naive UI, `app/src/utils/naiveTheme.ts`. Chaque famille (`primary`, `info`, `success`, `warning`, `error`) y est déclarée par trois teintes : couleur, survol, appui ; l'accent, une seule fois par thème (`ACCENT_LIGHT`, `ACCENT_DARK`), sert à `primary` et à `info`.

| Rôle | Clair | Sombre |
|---|---|---|
| Accent (`primary`, `info`) : violet, teinte OKLCH 285° | `#745af4` | `#968cff` |
| Succès | `#16a34a` | `#4ade80` |
| Avertissement | `#d97706` | `#fbbf24` |
| Erreur | `#dc2626` | `#f87171` |
| Fond | `#f4f6fb` | `#0d1020` |
| Cartes, barre latérale, en-tête, pied de page | `#ffffff` | `#151a2e` |
| Texte | `#151a2d` | `rgba(236, 239, 255, 0.94)` |

L'accent est choisi hors des couleurs d'état, pour qu'un élément d'accent ne se lise jamais comme un état : sa teinte (OKLCH) est à plus de 30° de l'erreur (vers 25°), de l'avertissement (60 à 85°) et du succès (vers 150°, aussi le vert de Metrx). La zone libre va d'environ 185° à 355° ; le violet à 285° est à 97° de l'état le plus proche, l'erreur. Sa luminosité donne un contraste d'au moins 4,5 : 4,65 avec le texte blanc des boutons en clair, 6,15 sur les cartes en sombre. Survol et appui : même teinte, un peu plus claire ou plus foncée.

Ce que Naive UI ne dessine pas se calcule depuis le thème (`useThemeVars()`), sans variable à part :

| Élément | Couleurs | Où |
|---|---|---|
| Nom "Baskul" en dégradé | `primaryColorHover` → `primaryColorPressed` | `BrandName.vue` |
| Fond animé | `primaryColor`, et deux voisines : sa teinte tournée de +40° (magenta) et de -40° (bleu) | `AppBackground.vue`, `rotateHue` (`utils/color.ts`) |
| Teintes transparentes de l'accent (survol des menus, étiquettes en sombre) | `primaryColor` rendu translucide | `withAlpha` dans `naiveTheme.ts` |
| Ombre du logo sur l'écran de démarrage | `primaryColor` | `AppSplash.vue` |

## Panneaux flottants

Les cartes et le cadre de la fenêtre (barre latérale, en-tête, pied de page) sont des panneaux détachés du fond et des bords de la fenêtre. Naive UI garde son apparence : le style ne touche que le rayon, l'espacement et, avec le fond animé, l'opacité et le flou du cadre. Ni reflet ni ombre ajoutés.

Structure de `AppLayout.vue` : un `h-layout` avec la barre latérale et une colonne : l'en-tête (`h-card`), la zone qui défile (`h-layout-content`), le pied de page (`h-card`). L'en-tête et le pied de page ne bougent pas ; la zone qui défile va au ras de l'un et de l'autre. Le contenu y commence à un écart (12 px) d'eux, et s'efface dans cet écart en défilant (`mask-image` sur la zone de défilement), sans jamais déborder. La zone déborde dans la marge de droite, où passe la barre de défilement, à côté des cartes.

| | Valeur | Où |
|---|---|---|
| Rayon | 16 px | `PANEL_RADIUS` (`naiveTheme.ts`) : `Card.borderRadius` et la barre latérale |
| Couleur | Celle des cartes (`cardColor`). Le cadre seul devient translucide quand le fond animé est actif ; les cartes de la page restent opaques | `frameColor` (`AppLayout.vue`), par les `theme-overrides` de la barre latérale et des deux cartes |
| Flou | Le cadre seul floute ce qui passe dessous (`backdrop-filter`) | `.app-frame` (`AppLayout.vue`) |
| Bord | Celui de Naive UI (`dividerColor`) | Thème par défaut |
| Écart | 12 px entre les panneaux et avec les bords de la fenêtre | `$panel-gap` (`variables.scss`) |

- L'en-tête, les cartes et le pied de page prennent toute la largeur à droite de la barre latérale : ils ont les mêmes bords.
- Le contenu ne passe pas derrière l'en-tête ni le pied de page : derrière un panneau arrondi de même largeur, il se verrait dans ses coins.
- Naive UI ne trace le bord de la barre latérale que d'un côté (`bordered`) : `AppLayout.vue` le trace sur les quatre, avec la couleur du thème (`var(--n-border-color)`).

## Fond animé

Une grille de points derrière les panneaux, que du bruit simplex fait grossir et s'allumer par vagues. L'utilisateur choisit le motif dans Réglages → Interface (`ui.background` de `config.toml`) :

| Motif | Valeur | Rendu |
|---|---|---|
| Courant (par défaut) | `current` | Une vague, du violet de l'accent au magenta, avance en diagonale ; les points restent fixes et grossissent à son passage |
| Vague | `wave` | Un nuage dérive entre le violet, le bleu et le magenta |
| Respiration | `breathing` | Grille grise ; seules les crêtes passent à l'accent |
| Onde | `ripple` | Toutes les trois secondes, une onde part de la pastille d'état, dans la couleur de la connexion |
| Aucun | `none` | Pas de fond, panneaux opaques |

- Code : `AppBackground.vue` (un `<canvas>` derrière `NLayout`), motifs dans `utils/backgroundPatterns.ts`, bruit dans `utils/noise.ts`.
- Couleurs : accent et neutre du thème Naive UI, et les deux voisines de l'accent (teinte tournée de ±`NEIGHBOUR_HUE`, 40°).
- 25 images par seconde au plus ; cadence réduite si une image coûte trop (grand écran), jusqu'à 8 par seconde. Arrêté quand la fenêtre est cachée, une seule image fixe si le système demande moins de mouvement.
- Le rendu ne dépend pas de la taille de la fenêtre : grille et motif ancrés au centre de la fenêtre (arrondi au pixel), bruit et taille des points en pixels CSS, onde à vitesse fixe (400 px/s). Agrandir la fenêtre découvre plus de grille de chaque côté, sans rien étirer ; pendant qu'on la redimensionne, le motif suit le centre.
- La pastille d'état pulse au même rythme que l'onde (`PULSE_SECONDS`).
- **Valeurs du fond et du cadre** : vitesse 0,7, opacité du cadre 80 %, flou sous le cadre 10 px, pas de la grille 6 px (`DEFAULT_TUNING`, `stores/backgroundTuning.ts`). En développement seulement (`yarn tauri dev`, `yarn dev:mock`), des curseurs d'essai dans Réglages → Interface permettent de les ajuster, gardés dans la fenêtre ; ailleurs, les valeurs par défaut s'appliquent toujours.

## Formes et texte

- Police du système (`system-ui`, San Francisco, Segoe UI) ; police à chasse fixe du système pour les adresses, les secrets et le journal.
- Rayons : 16 px pour les panneaux, 10 px par défaut, 8 px pour les boutons, 6 px pour les étiquettes.
- Chaque réglage a son icône (Tabler), dans une pastille teintée de l'accent (`color-mix` avec `$accent-tint`).

## Dans le code

- Une couleur vient du thème (`useThemeVars()`), ou s'en calcule (`withAlpha`, `rotateHue`, `color-mix`) ; jamais en dur, jamais de variable CSS à part.
- Le fond de `NLayout` est transparent : le fond animé apparaît entre les panneaux. Sans lui, c'est le fond du thème (`bodyColor`).
