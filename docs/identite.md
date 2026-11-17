# Identité visuelle

Baskul fait passer le clavier et la souris d'un écran à l'autre : le logo montre deux écrans reliés par une flèche, et l'interface reprend un dégradé bleu indigo, du verre dépoli et des coins arrondis.

## Logo

- Source : `app/app-icon.svg`, copiée dans `app/src/assets/logo.svg` pour la fenêtre.
- Carré arrondi en dégradé (`#5b8cff` → `#4338ca`), un reflet en haut ; deux écrans blancs, celui de gauche en contour, celui de droite plein ; une flèche en arc de l'un à l'autre.
- Icônes des systèmes (`app/src-tauri/icons/`) régénérées depuis le SVG :

```bash
cd app && yarn tauri icon app-icon.svg
```

## Couleurs

Le thème Naive UI est dans `app/src/utils/naiveTheme.ts` ; les couleurs que Naive UI ne dessine pas (fond, verre, dégradé) sont des variables CSS de `app/src/assets/brand.scss`.

| Rôle | Clair | Sombre |
|---|---|---|
| Accent (`primary`, `info`) | `#4c5fef` | `#8c9bff` |
| Succès | `#16a34a` | `#4ade80` |
| Avertissement | `#d97706` | `#fbbf24` |
| Erreur | `#dc2626` | `#f87171` |
| Fond | `#f4f6fb` | `#0d1020` |
| Cartes | `#ffffff` | `#151a2e` |
| Texte | `#151a2d` | `rgba(236, 239, 255, 0.94)` |

Variables CSS (`brand.scss`), redéfinies en thème sombre sur `:root[data-theme="dark"]` :

| Variable | Usage |
|---|---|
| `--brand-gradient` | Dégradé du logo, éclairci en thème sombre : nom "Baskul" (`.brand-wordmark`) |
| `--brand-glow-start`, `--brand-glow-end` | Deux lueurs dans les coins du fond de la fenêtre |
| `--glass`, `--glass-border` | Barre latérale, en-tête et pied de page (`.glass`, flou d'arrière-plan) |
| `--brand-tint` | Fond teinté de l'accent : icônes des réglages, machine active |

## Formes et texte

- Police du système (`system-ui`, San Francisco, Segoe UI) ; police à chasse fixe du système pour les adresses, les secrets et le journal.
- Rayons : 14 px pour les cartes, 10 px par défaut, 8 px pour les boutons, 6 px pour les étiquettes.
- Chaque réglage a son icône (Tabler), dans une pastille teintée de l'accent.

## Dans le code

- Une couleur passe par le thème (`useThemeVars()`) ou par une variable de `brand.scss`, jamais en dur.
- Les fonds de `NLayout` sont transparents : les lueurs du fond passent sous le cadre.
