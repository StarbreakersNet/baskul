# Identité visuelle

Baskul fait passer le clavier et la souris d'un écran à l'autre : le logo montre deux écrans reliés par une flèche, et l'interface, en verre façon *Liquid Glass*, laisse voir les couleurs de la marque.

## Logo

- Source : `app/app-icon.svg`, copiée dans `app/src/assets/logo.svg` pour la fenêtre.
- Carré arrondi en dégradé (`#5b8cff` → `#4338ca`), un reflet en haut ; deux écrans blancs, celui de gauche en contour, celui de droite plein ; une flèche en arc de l'un à l'autre.
- Icônes des systèmes (`app/src-tauri/icons/`) régénérées depuis le SVG :

```bash
cd app && yarn tauri icon app-icon.svg
```

## Style : Liquid Glass

Les surfaces sont du verre : translucides, elles floutent et saturent ce qui passe dessous, avec un reflet clair sur le bord haut et une ombre douce.

- **Fond** : quatre formes colorées de la marque (indigo, bleu ciel, violet), fixes, sous toute la fenêtre. Sans elles, le verre n'aurait rien à montrer.
- **Panneaux flottants** : la barre latérale, l'en-tête et le pied de page sont détachés des bords de la fenêtre ; le contenu défile dessous.
- **Formes** : cartes à 22 px de rayon, boutons, étiquettes et onglets en pilule, champs à 12 px.
- Une carte posée dans une autre carte (`embedded`) ne refait pas le flou.

## Couleurs

Tout passe par le thème Naive UI (`app/src/utils/naiveTheme.ts`) : surfaces translucides (`cardColor`, `modalColor`, `popoverColor`, barres de `Layout`…), couleurs d'état, bordures, rayons.

| Rôle | Clair | Sombre |
|---|---|---|
| Accent (`primary`, `info`) | `#4c5fef` | `#8c9bff` |
| Succès | `#16a34a` | `#4ade80` |
| Avertissement | `#d97706` | `#fbbf24` |
| Erreur | `#dc2626` | `#f87171` |
| Fond | `#eef1fa` | `#0a0d1c` |
| Cartes (verre) | blanc à 52 % | `#222848` à 42 % |
| Texte | `#151a2d` | blanc bleuté à 95 % |

`app/src/assets/brand.scss` ne contient que ce que Naive UI ne sait pas dessiner, sans variable CSS :

| Classe ou règle | Usage |
|---|---|
| `body` | Les formes colorées du fond |
| `.glass` | Flou, saturation, reflet et ombre du verre : cartes (`h-card`), barre latérale, en-tête, pied de page |
| `.brand-wordmark` | Nom "Baskul" en dégradé, éclairci en thème sombre |

## Formes et texte

- Police du système (`system-ui`, San Francisco, Segoe UI) ; police à chasse fixe du système pour les adresses, les secrets et le journal.
- Chaque réglage a son icône (Tabler), dans une pastille teintée de l'accent (`color-mix` avec la couleur du thème).

## Dans le code

- Une couleur vient du thème (`useThemeVars()`), jamais en dur ni d'une variable CSS ajoutée.
- Une nouvelle surface en verre prend la classe `.glass` et une couleur translucide du thème.
- Les fonds de `NLayout` sont transparents : les formes du fond passent sous le cadre.
