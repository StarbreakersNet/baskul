# Bascule

Bascule partage **le clavier, la souris et le presse-papiers** de votre PC Windows avec votre Mac. Pour piloter le Mac, il suffit d'amener le curseur au bord de l'écran du PC.

- Le **clavier est transparent** : Bascule transmet la *position physique* de chaque touche. Il ne transmet jamais de caractère. C'est la disposition clavier du Mac qui produit les caractères, comme si le clavier était branché sur le Mac. **Ctrl reste Ctrl.** Aucune combinaison n'est convertie.
- **Pas de bug des touches mortes**, contrairement à Deskflow / Synergy (voir [plus bas](#pourquoi-pas-de-bug-des-touches-mortes)). `^` puis `e` donne `ê`, et `¨` puis `i` donne `ï`.
- Les **2 boutons latéraux de la souris (4/5)** sont transmis. Ils peuvent aussi devenir un geste « précédent / suivant » compris par Safari, le Finder, Xcode…
- La **molette** est réglable : défilement par lignes ou au pixel, vitesse, inversion.
- Le **presse-papiers** est partagé dans les deux sens : texte, texte mis en forme (HTML), images. Les mots de passe copiés depuis un gestionnaire de mots de passe sont ignorés.
- La liaison est **chiffrée** (protocole Noise) et protégée par un secret partagé. Bascule se reconnecte automatiquement.
- Tous les réglages se trouvent dans un fichier texte et se changent **sans recompiler**.

> Le sens Mac → PC (le Mac pilote le PC) est aussi prévu, mais il est **expérimental** (voir [Sens inverse](#sens-inverse-mac--pc-expérimental)).

---

## 1. Obtenir les programmes

### Option A : GitHub Actions (recommandé, rien à installer)

1. Créez un dépôt **privé** sur GitHub et poussez-y ce dossier.
2. L'onglet **Actions** lance automatiquement le workflow `build` (5 à 10 minutes).
3. Ouvrez l'exécution terminée et téléchargez les deux *artifacts* :
   - **Bascule-Windows** contient `bascule.exe`.
   - **Bascule-macOS** contient `Bascule-macOS.zip`, une app universelle pour Apple Silicon et Intel.

### Option B : compiler soi-même

**Sur le Mac**

```bash
xcode-select --install                                           # outils de compilation Apple
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh  # Rust (choix par défaut)
source ~/.cargo/env
./scripts/build-macos.sh      # produit dist/Bascule.app
```

**Sur le PC**, installez Rust avec [rustup](https://rustup.rs). Il propose d'installer les *Visual Studio Build Tools* : acceptez. Ensuite :

```powershell
powershell -ExecutionPolicy Bypass -File scripts\build-windows.ps1   # produit dist\Bascule.exe
```

---

## 2. Mise en route (environ 5 minutes)

La configuration conseillée : **le Mac attend la connexion, le PC se connecte au Mac.** Le pare-feu de Windows n'a ainsi rien à autoriser.

Quittez CursorHop (ou tout autre logiciel de partage de souris) avant de lancer Bascule : deux logiciels qui interceptent la souris en même temps se gênent.

### Sur le Mac

1. Décompressez `Bascule-macOS.zip` et glissez **Bascule.app** dans **Applications**.
2. Au premier lancement, macOS bloque l'app car elle n'est pas signée par un développeur identifié.
   - Faites un clic droit sur l'app, puis **Ouvrir**.
   - Sur macOS 15 et suivants, allez dans *Réglages Système → Confidentialité et sécurité*, puis cliquez sur **Ouvrir quand même** en bas de la page.
   - Autre solution, dans le Terminal : `xattr -dr com.apple.quarantine /Applications/Bascule.app`
3. Bascule crée sa configuration et l'ouvre. `role = "secondary"` est déjà réglé. **Copiez la ligne `secret = "…"`**, vous en aurez besoin sur le PC. Ne touchez à rien d'autre.
4. macOS demande l'autorisation **Accessibilité**. Ouvrez *Réglages Système → Confidentialité et sécurité → Accessibilité* et activez **Bascule**. Si macOS demande aussi d'accepter les connexions entrantes ou l'accès au réseau local, cliquez sur **Autoriser**.
5. Notez l'adresse IP du Mac. Elle se trouve dans *Réglages Système → Wi-Fi (ou Réseau) → Détails*. Vous pouvez aussi taper `ipconfig getifaddr en0` dans le Terminal.

L'icône ⌨︎ dans la barre des menus indique l'état. Son menu propose aussi « Autoriser Bascule (Accessibilité)… » si l'autorisation manque.

### Sur le PC

1. Rangez `bascule.exe` dans un dossier définitif, par exemple `%LOCALAPPDATA%\Programs\Bascule\Bascule.exe`. Si SmartScreen s'affiche, cliquez sur **Informations complémentaires**, puis **Exécuter quand même**.
2. Lancez-le. La configuration s'ouvre dans le Bloc-notes avec `role = "primary"`. Modifiez :
   ```toml
   connect = "192.168.1.30"          # l'adresse IP du Mac
   secret = "…"                      # EXACTEMENT le même secret que sur le Mac
   ```
   Si le Mac n'est pas à droite du PC, réglez aussi `[screen] secondary_position` sur `"left"`, `"top"` ou `"bottom"`.
3. Enregistrez. Faites un clic droit sur l'icône de Bascule dans la zone de notification (à côté de l'horloge), puis **Redémarrer Bascule**.
4. L'icône devient **verte** une fois connectée. Amenez la souris au bord droit de l'écran : le curseur passe sur le Mac. 🎉
5. Facultatif : dans le menu, cochez **Lancer au démarrage de Windows**. Sur le Mac, cochez **Ouvrir à l'ouverture de session**.

---

## 3. Utilisation

| Action | Comment |
|---|---|
| Passer sur le Mac | Pousser le curseur contre le bord de l'écran du PC (sauf dans les 10 px des coins) |
| Revenir sur le PC | Ramener le curseur par le même bord, côté Mac |
| Retour d'urgence | **Ctrl + Alt + Maj + B**, ou le menu « Ramener le curseur sur ce PC » |
| Copier / coller entre machines | Copier normalement d'un côté, coller de l'autre |

Si la connexion tombe, le curseur revient tout seul sur le PC. Tout ce qui était enfoncé est relâché côté Mac.

### Correspondance des touches (position physique)

| Clavier du PC | Mac |
|---|---|
| Ctrl | Control ⌃ |
| Windows ⊞ | Commande ⌘ |
| Alt | Option ⌥ |
| AltGr | Option ⌥ (droite) |
| Impr. écran / Arrêt défil. / Pause | F13 / F14 / F15 |

Sur le Mac, **copier** se fait donc avec **⊞ + C**, puisque la touche Windows est à la place de ⌘. Les caractères suivent la disposition du Mac. Avec un Mac en AZERTY, `@` est sur la touche à gauche du `1`, et `Option ⌥` donne les caractères spéciaux du Mac.

Pour avoir ⌘ à côté de la barre d'espace, comme sur un clavier Apple, ajoutez dans la configuration **du Mac** :

```toml
[keyboard.remap]
AltLeft = "MetaLeft"     # Alt du PC     → ⌘
MetaLeft = "AltLeft"     # Windows du PC → ⌥
```

---

## 4. Réglages (sans recompiler)

Ouvrez le fichier via le menu, **Modifier la configuration…**, puis faites **Redémarrer Bascule**. Le fichier complet et commenté est dans [`docs/config.exemple.toml`](docs/config.exemple.toml).

| Section | Réglage | Effet |
|---|---|---|
| `[scroll]` (Mac) | `mode = "lines"` / `"pixels"` | Par lignes, comme une molette Windows, ou au pixel (précis et identique dans toutes les apps) |
| | `lines_per_notch = 3.0` | Vitesse en mode lignes (par cran de molette) |
| | `pixels_per_notch = 60.0` | Vitesse en mode pixels |
| | `invert_vertical`, `invert_horizontal` | Inverser le sens |
| `[buttons]` (Mac) | `side_buttons = "buttons"` / `"swipe"` | Boutons 4/5 bruts (Chrome, Firefox, VS Code…) ou geste précédent/suivant (Safari, Finder, Xcode…) |
| | `swap_side_buttons` | Inverser les boutons 4 et 5 |
| `[mouse]` (Mac) | `speed = 1.0` | Vitesse du curseur sur le Mac |
| `[keyboard]` (Mac) | `iso_swap = "auto"` / `"on"` / `"off"` | Touches `²` et `<` inversées ? Forcez `on` ou `off` |
| | `[keyboard.remap]` | Remappage de touches physiques |
| `[keyboard]` (PC) | `media_keys = "remote"` / `"local"` | Les touches volume et lecture agissent sur la machine où est le curseur, ou toujours sur le PC |
| `[screen]` (PC) | `secondary_position`, `corner_size` | Position du Mac, et coins neutres |
| `[hotkeys]` (PC) | `return_to_primary`, `toggle_lock` | Raccourcis : retour sur le PC, verrouillage du curseur (ex. `"ScrollLock"`) |
| `[clipboard]` | `enabled`, `images`, `html`, `max_size_mb`, `skip_sensitive` | Presse-papiers partagé |
| `[log]` | `level = "debug"` | Journal détaillé, utile pour un diagnostic |

Pour le défilement, commencez par `mode = "lines"`. Si la vitesse varie trop d'une app à l'autre (Safari vs Chrome), essayez `mode = "pixels"` avec `pixels_per_notch` entre 60 et 120.

---

## 5. Dépannage

**L'icône reste grise ou rouge, sans connexion**
- Le `secret` doit être **identique** des deux côtés. En cas d'erreur, le menu affiche « secret incorrect ».
- Une seule machine doit avoir `connect` rempli, avec l'IP de l'autre.
- Les rôles doivent être `primary` sur le PC et `secondary` sur le Mac. Le menu signale l'erreur si ce n'est pas le cas.
- Si le pare-feu du Mac est actif, autorisez Bascule : *Réglages → Réseau → Pare-feu → Options*.
- Les deux machines doivent être sur le même réseau. Pensez à donner une IP fixe au Mac dans votre box (bail DHCP statique).

**Les touches `²` / `@` et `<` / `>` sont inversées.** Réglez `[keyboard] iso_swap = "on"` (ou `"off"`) sur le Mac, puis redémarrez Bascule.

**Le défilement va dans le mauvais sens, trop vite ou trop lentement.** Voir la section `[scroll]` ci-dessus.

**Précédent / suivant (boutons 4/5) ne fonctionne pas dans Safari ou le Finder.** Réglez `side_buttons = "swipe"`.

**Après une mise à jour, le Mac ne réagit plus.** Une app non signée par un développeur change d'identité à chaque compilation. Dans *Accessibilité*, supprimez Bascule (bouton **−**), puis ajoutez-la à nouveau (**+**).

Pour éviter cette manipulation, signez vos compilations locales avec un certificat local :
1. Ouvrez *Trousseaux d'accès → Assistant de certification → Créer un certificat*.
2. Indiquez le nom `Bascule Local`, le type *Racine auto-signée* et le type de certificat *Signature de code*.
3. Compilez avec `CODESIGN_IDENTITY="Bascule Local" ./scripts/build-macos.sh`.

**Win + L verrouille le PC, même quand le curseur est sur le Mac.** C'est une limite de Windows, comme Ctrl + Alt + Suppr : ces combinaisons ne peuvent pas être interceptées. Bascule relâche alors tout proprement.

**Une fenêtre lancée en administrateur sur le PC (Gestionnaire des tâches…).** Windows interdit à un programme normal d'intercepter le clavier quand une telle fenêtre est active. Bascule reprend le premier plan quand vous passez sur le Mac. Pour couvrir aussi ce cas, lancez Bascule en administrateur.

**Journaux**
- PC : `%APPDATA%\Bascule\bascule.log`
- Mac : `~/Library/Logs/Bascule/bascule.log`

Le menu propose aussi **Ouvrir le journal**. Avec `[log] level = "debug"`, chaque touche ignorée et chaque changement d'écran sont détaillés.

**Configuration**
- PC : `%APPDATA%\Bascule\config.toml`
- Mac : `~/Library/Application Support/Bascule/config.toml`

---

## Sens inverse (Mac → PC, expérimental)

Inversez simplement les rôles : `role = "primary"` sur le Mac, `role = "secondary"` sur le PC. Sur le Mac, il faut alors les autorisations **Accessibilité** et **Surveillance de l'entrée**. Le code est complet, mais il a été moins éprouvé que le sens PC → Mac.

---

## Pourquoi pas de bug des touches mortes ?

Deskflow et Synergy **traduisent les frappes en caractères sur le PC** (fonction Windows `ToUnicode`). Ils tentent ensuite de retrouver, sur l'autre machine, quelle touche produirait ce caractère. Or `ToUnicode` *consomme* l'état des touches mortes de Windows. Résultat : `^` puis `e` donne `^e`, `ê` en double, ou rien du tout. Il faut aussi deviner des correspondances entre des dispositions qui ne sont pas les mêmes (AZERTY PC et AZERTY Mac diffèrent).

Bascule ne traduit **rien**. Sur le PC, un hook bas niveau lit le **scancode** de chaque touche, c'est-à-dire sa position physique. Bascule le convertit en code **USB HID**, puis en **keycode Mac**, et l'injecte. La composition `^` + `e` → `ê` est faite par macOS lui-même, avec la disposition choisie sur le Mac. L'état des touches mortes de Windows n'est jamais touché.

Deux détails de Windows sont gérés :
- **AltGr** : Windows génère un faux « Ctrl gauche » en même temps qu'AltGr. Bascule le filtre, et le Mac ne reçoit que ⌥ droite.
- **Claviers ISO** (européens) : les touches `²` et `<` sont échangées comme le fait macOS pour ses claviers ISO. Le réglage `iso_swap` le contrôle.

---

## Architecture

```
src/
  main.rs          démarrage, arguments, instance unique
  config.rs        configuration TOML commentée (créée au 1er lancement)
  protocol.rs      messages (postcard)
  crypto.rs        chiffrement Noise NNpsk0 (clé dérivée du secret)
  net.rs           connexion TCP, reconnexion, priorité souris/clavier > presse-papiers
  keymap.rs        table USB HID ↔ scancode Windows ↔ keycode Mac (données Chromium)
  geometry.rs      écrans, bords, conversion des positions
  primary.rs       logique de la machine qui pilote (bascule, routage des touches…)
  secondary.rs     logique de la machine pilotée (position, sortie par le bord…)
  clipboard.rs     presse-papiers (texte, HTML, PNG), anti-écho
  hotkey.rs        raccourcis
  platform/
    windows.rs     hooks bas niveau, masquage du curseur, SendInput, icône
    macos.rs       CGEventPost, gestes, Verr. Maj (IOKit), CGEventTap, barre des menus
```

Tests : `cargo test` (54 tests, dont un test de bout en bout sur une vraie connexion TCP). Le code Windows et macOS se vérifie aussi depuis Linux avec `./scripts/xcheck.sh`.
