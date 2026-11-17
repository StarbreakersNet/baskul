# Styles

Les règles marquées **`lint`** sont vérifiées par `yarn lint`.

Dans l'ordre de préférence :

1. **Un composant `h-*`** pour la mise en page : `<h-flex vertical>`, `<h-grid>`, plutôt qu'un `display: flex` écrit à la main.
2. **Une classe partagée** de `assets/main.scss` pour ce qui revient partout : `.mono` (police à chasse fixe), `.help-text` (texte d'aide sous un libellé), `.small-text`, `.break-all`.
3. **Une classe nommée dans `<style lang="scss" scoped>`** pour tout le reste : marges, largeurs, états (`:hover`), animations, media queries. **`lint`** (`lang="scss"`)

Et :

- **Pas d'attribut `style` en dur** : `class="advanced-select"` et `.advanced-select { width: 220px; }`, plutôt que `style="width: 220px"`. Un `:style` ne sert qu'aux valeurs calculées au moment de l'affichage (position de la pilotée qu'on fait glisser, nombre de colonnes de la grille).
- Valeurs du design (hauteur des barres, écarts, tailles de texte, police à chasse fixe) : les variables de `assets/variables.scss`, avec `@use "@/assets/variables";` puis `variables.$bar-height`. Une valeur qui revient devient une variable.
- Rayons : ceux du thème Naive UI, jamais recopiés en SCSS. `v-bind("themeVars.borderRadius")` (ou `borderRadiusSmall`) avec `useThemeVars()` ; pour le rayon des panneaux flottants, qui n'est pas dans les variables communes, `PANEL_RADIUS` de `utils/naiveTheme.ts`.
- Couleurs : celles du thème Naive UI, avec `useThemeVars()` et `v-bind()` dans le style. Pas de couleur en dur qui ne marche que dans un thème. Une teinte de la couleur d'accent s'écrit `color-mix(in srgb, v-bind("colors.accent") variables.$accent-tint, transparent)`.
- Couleurs de l'identité que Naive UI ne dessine pas (nom en dégradé, fond animé) : calculées depuis le thème (`withAlpha`, `rotateHue` de `utils/color.ts`), jamais de variable CSS à part. Voir [`docs/identite.md`](../identite.md).
- `scoped` toujours. Les classes partagées vivent dans `assets/main.scss`.
- Un `<style>` non scoped dans un composant ne sert qu'à cibler du contenu téléporté hors du composant (menu déroulant, modale), avec un commentaire qui le dit.
- `:deep()` sert dans les wrappers à ajuster l'intérieur d'un composant Naive UI. Ailleurs, c'est le signe qu'il manque une prop ou une variante au wrapper.
- Classes en kebab-case, préfixées par le nom du composant (`.layout-tile`, `.layout-tile--on`), ou imbriquées sous sa classe racine.

## Piège : `NGrid` en mode `responsive="self"`

`NGrid` en mode `responsive="self"` ne reçoit pas l'attribut des styles scoped du parent : une classe scoped posée sur `<h-grid responsive="self">` n'a aucun effet. On pose la classe sur un conteneur autour (voir la grille des machines de `HomeView`).
