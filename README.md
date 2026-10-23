# Baskul

Baskul partage **le clavier, la souris et le presse-papiers** d'une **machine principale** (celle où sont branchés le clavier et la souris) avec une ou plusieurs **machines pilotées**. Pour passer sur une pilotée, il suffit d'amener le curseur au bord de l'écran, ou d'appuyer sur un raccourci clavier (**Maj + F12** par défaut).

Windows et macOS peuvent jouer l'un ou l'autre rôle. Linux est [expérimental](#linux-expérimental) : il se connecte et partage le presse-papiers, mais pas encore le clavier ni la souris.

- **Plusieurs pilotées**, placées sur une **grille** comme vos écrans sur le bureau. On passe d'une machine à sa voisine par leur bord commun, ou directement par un raccourci (dernière pilotée, machine suivante, un raccourci par pilotée), par le menu « Aller sur… » ou par une [commande](#commandes-pour-dautres-applications).
- **Découverte automatique** (Bonjour) : les pilotées trouvent la principale toutes seules sur le réseau local. Le **choix de la carte réseau** permet d'imposer l'Ethernet plutôt que le Wi-Fi, par exemple.
- Le **clavier est transparent** : Baskul transmet la *position physique* de chaque touche, jamais un caractère. C'est la disposition clavier de la pilotée qui produit les caractères, comme si le clavier y était branché. Aucune combinaison n'est convertie, et **pas de bug des touches mortes**, contrairement à Deskflow / Synergy (voir [plus bas](#pourquoi-pas-de-bug-des-touches-mortes)) : `^` puis `e` donne `ê`.
- Les **2 boutons latéraux de la souris (4/5)** sont transmis. Sur une pilotée macOS, ils peuvent aussi devenir un geste « précédent / suivant » (Safari, Finder, Xcode…), et la molette peut défiler **de façon fluide**, comme avec Mac Mouse Fix.
- Le **presse-papiers** est partagé entre toutes les machines : texte, texte mis en forme (HTML), **images dans leur format d'origine** (un PNG reste un PNG, sans conversion), **fichiers et dossiers** (copiés sur la machine où vous les collez). Les mots de passe copiés depuis un gestionnaire de mots de passe sont ignorés.
- Une **bulle d'arrivée** affiche un instant le nom de la machine près du curseur quand le clavier et la souris arrivent dessus.
- L'**activité est partagée** : les machines restent éveillées (et « actives » dans Teams, Discord…) tant que vous travaillez sur l'une d'elles.
- **Mises à jour intégrées** : Baskul télécharge les nouvelles versions en arrière-plan ; « Redémarrer pour mettre à jour » les installe.
- La liaison est **chiffrée** (protocole Noise) et protégée par un secret partagé. Baskul se reconnecte automatiquement.
- Tous les réglages se changent **sans recompiler**, dans la fenêtre de Baskul (ou directement dans le fichier texte).

> Baskul s'appelait **Bascule** jusqu'à la version 0.6. Au premier lancement, ses réglages sont repris automatiquement.

---

## 1. Obtenir les programmes

Les versions publiées sont dans la page [**Releases**](https://github.com/StarbreakersNet/baskul/releases) :

- `Baskul-Windows.zip` : `baskul.exe` et son app de réglages `baskul-settings.exe` ;
- `Baskul-macOS.zip` : `Baskul.app`, universelle (Apple Silicon et Intel), app de réglages incluse ;
- `Baskul-Linux.tar.gz` : la [version Linux](#linux-expérimental) ;
- `SHA256SUMS` : les empreintes des fichiers, vérifiées par la mise à jour intégrée.

Une fois installé, Baskul se met à jour tout seul (voir [Mises à jour](#5-mises-à-jour)).

### Publier une version (GitHub Actions)

1. Mettez à jour `version` dans la section `[workspace.package]` de `Cargo.toml` (c'est la version affichée par les apps), puis, au choix :
   - onglet **Actions** → **release** → **Run workflow** : le tag (la version de `Cargo.toml`, par exemple `0.7.0`) est créé avec la Release ;
   - ou créez un tag et poussez-le :
     ```bash
     git tag 0.7.0
     git push origin 0.7.0
     ```
2. À la fin, la **Release** contient les trois archives et `SHA256SUMS`.

Pour compiler sans publier : onglet **Actions** → **build** → **Run workflow**. Les programmes sont alors dans les *artifacts*, en bas de la page de l'exécution. Le workflow **release** réutilise ce workflow **build** (tests puis compilation), puis publie.

### Compiler soi-même

Il faut [Rust](https://rustup.rs) et [Node.js](https://nodejs.org) 22 (pour l'app de réglages). Yarn 4 est fourni par `corepack`, inclus dans Node.js.

**macOS**

```bash
xcode-select --install                                           # outils de compilation Apple
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh  # Rust (choix par défaut)
source ~/.cargo/env
./scripts/build-macos.sh      # produit dist/Baskul.app
```

**Windows** : rustup propose d'installer les *Visual Studio Build Tools* ; acceptez. Ensuite :

```powershell
powershell -ExecutionPolicy Bypass -File scripts\build-windows.ps1   # produit dist\Baskul\
```

**Linux** : voir [Linux](#linux-expérimental).

---

## 2. Mise en route (environ 5 minutes)

Quittez tout autre logiciel de partage de souris (CursorHop, Deskflow…) avant de lancer Baskul : deux logiciels qui interceptent la souris en même temps se gênent.

### Installer Baskul sur chaque machine

**macOS**

1. Décompressez `Baskul-macOS.zip` et glissez **Baskul.app** dans **Applications**.
2. Au premier lancement, macOS bloque l'app car elle n'est pas signée par un développeur identifié.
   - Faites un clic droit sur l'app, puis **Ouvrir**.
   - Sur macOS 15 et suivants, allez dans *Réglages Système → Confidentialité et sécurité*, puis cliquez sur **Ouvrir quand même** en bas de la page.
   - Autre solution, dans le Terminal : `xattr -dr com.apple.quarantine /Applications/Baskul.app`
3. macOS demande l'autorisation **Accessibilité** (et, sur une machine principale, **Surveillance de l'entrée**). Ouvrez *Réglages Système → Confidentialité et sécurité* et activez **Baskul**. Acceptez aussi l'accès au **réseau local** et, sur une machine principale, les **connexions entrantes**.

**Windows**

1. Décompressez `Baskul-Windows.zip` dans un dossier définitif, par exemple `%LOCALAPPDATA%\Programs\Baskul`. Gardez `baskul.exe` et `baskul-settings.exe` ensemble. Si SmartScreen s'affiche, cliquez sur **Informations complémentaires**, puis **Exécuter quand même**.
2. Lancez `baskul.exe`. Sur une machine principale, le pare-feu de Windows demande d'autoriser Baskul : acceptez pour les **réseaux privés**.

Au premier lancement, Baskul crée sa configuration et ouvre ses **réglages**.

### Relier les machines

1. **Rôle.** Sur la machine qui a le clavier et la souris, choisissez **Principale** ; sur les autres, **Pilotée**. Donnez à chacune un **nom affiché** parlant (« Bureau », « Atelier »…) : c'est lui qui apparaît dans les menus et la bulle d'arrivée.
2. **Secret partagé.** Copiez celui de la principale (bouton « Copier ») et collez-le dans les réglages de chaque pilotée. Il doit être **identique** partout.
3. **Connexion.** Rien à saisir : chaque pilotée trouve la principale sur le réseau local (« Machines principales trouvées », onglet Connexion). Si elle ne la trouve pas (réseau qui bloque Bonjour, sous-réseaux différents…), indiquez l'**adresse de la principale** dans « Se connecter à » : elle est affichée dans l'onglet Connexion de la principale.
4. Cliquez sur **Enregistrer et redémarrer** sur chaque machine.
5. **Disposition.** Une pilotée qui se connecte pour la première fois est placée à côté de la principale (à droite par défaut). Dans les réglages de la principale, onglet **Changer de machine**, faites glisser les pilotées sur la grille pour reproduire la place réelle des écrans.

L'icône de Baskul (barre des menus ou zone de notification) devient **verte** une fois connectée. Appuyez sur **Maj + F12**, ou amenez la souris au bord de l'écran du côté d'une pilotée : le curseur y passe. 🎉

Facultatif : dans le menu de l'icône, cochez **Lancer au démarrage de Windows** ou **Ouvrir à l'ouverture de session** (macOS).

### Depuis la version 0.6

Jusqu'à la 0.6, une seule machine pilotée était possible, et c'était souvent la principale qui se connectait à la pilotée. Désormais, **ce sont les pilotées qui se connectent à la principale**. Après la mise à jour :

- sur la pilotée, laissez « Se connecter à » vide (découverte automatique) ou indiquez-y l'adresse de la principale ;
- sur la principale, autorisez les connexions entrantes (pare-feu de Windows, ou de macOS) ;
- sur macOS, l'app a changé d'identité (nouveau nom) : réactivez Baskul dans *Accessibilité* (et *Surveillance de l'entrée* sur une principale).

---

## 3. Utilisation

| Action | Comment |
|---|---|
| Passer sur une pilotée | Pousser le curseur contre le bord commun avec elle (sauf dans les 10 px des coins) ; **Maj + F12** pour la dernière pilotée utilisée (le curseur reprend là où il était) ; son propre raccourci, s'il y en a un ; menu de l'icône « Aller sur « … » » ; bouton « Aller sur » de l'accueil |
| Passer d'une pilotée à une autre | Par leur bord commun dans la grille, ou par raccourci |
| Revenir sur la principale | **Maj + F11**, ou ramener le curseur par le bord commun ; menu « Ramener le curseur sur cette machine » |
| Faire le tour des machines | Raccourci « Machine suivante » (à définir) : la principale, puis les pilotées dans l'ordre de la grille |
| Copier / coller entre machines | Copier normalement sur une machine, coller sur une autre : texte, images, fichiers |
| Rester sur une machine | Raccourci « Verrouiller le curseur » (par exemple Arrêt défil.) |

Les raccourcis se changent dans les réglages de la principale. Le même raccourci pour l'aller et le retour permet de passer d'une machine à l'autre avec une seule combinaison. Baskul ne les intercepte que lorsqu'une pilotée est connectée. Attention, **Maj + F12** sert aussi dans VS Code (« Rechercher toutes les références ») : choisissez-en un autre si vous en avez besoin.

Pour ne passer d'une machine à l'autre **qu'au clavier**, désactivez « Bascule par les bords de l'écran ». Une pilotée qui ne touche aucune autre machine sur la grille n'est accessible que par un raccourci : l'éditeur de la grille le signale.

Si la connexion avec la pilotée en cours tombe, Baskul attend quelques secondes qu'elle revienne (redémarrage de Baskul sur la pilotée…), puis le curseur revient tout seul sur la principale. Tout ce qui était enfoncé est relâché.

**Bulle d'arrivée.** Quand le clavier et la souris arrivent sur une machine, son nom s'affiche un instant près du curseur, avec le logo de son système, puis s'efface. Elle ne prend pas le focus et laisse passer les clics. Réglage « Bulle d'arrivée », onglet Interface, sur chaque machine.

**Presse-papiers.** Les images partent dans leur format d'origine (PNG, JPEG, GIF, WebP, HEIC…) ; seul le TIFF interne de macOS devient un PNG. Pour les formats que le système d'arrivée lit mal (HEIC, WebP…), un PNG est ajouté. Les **fichiers et dossiers** copiés dans le Finder ou l'Explorateur sont transférés en arrière-plan, puis collés depuis un dossier temporaire de Baskul ; une nouvelle copie annule un transfert en cours. Copier une seule image en tant que fichier permet aussi de la coller comme image. La principale relaie le presse-papiers entre les pilotées. Les limites de taille se règlent dans l'onglet Presse-papiers.

**Activité partagée.** Tant que vous êtes actif sur une machine, les autres restent actives elles aussi : pas de veille de l'écran ni d'économiseur, et le statut « actif » de Teams, Discord, Slack… est conservé. Quand vous vous absentez, les signaux cessent et chaque machine compte son inactivité comme d'habitude. Chaque machine décide pour elle-même (réglage « Activité partagée », onglet Connexion, activé par défaut).

### Commandes pour d'autres applications

D'autres applications (Keyboard Maestro, Raccourcis, AutoHotkey, scripts…) peuvent dire à Baskul où envoyer le clavier et la souris, depuis n'importe quelle machine :

| Commande | Effet |
|---|---|
| `baskul switch primary` | Clavier et souris sur la machine principale |
| `baskul switch secondary` | … sur la dernière pilotée utilisée |
| `baskul switch Atelier` | … sur la pilotée « Atelier » (son nom affiché) |
| `baskul switch here` | … sur la machine où la commande est lancée |
| `baskul switch other` | Depuis la principale : la dernière pilotée ; depuis une pilotée : la principale |
| `baskul switch next` | … sur la machine suivante (la principale, puis les pilotées dans l'ordre de la grille) |
| `baskul switch toggle` | Aller-retour entre la principale et la dernière pilotée (ou cette pilotée) |
| `baskul status` | État de Baskul, une information par ligne (`role`, `connected`, `peers`, `active`, `locked`) |

Baskul doit être lancé sur la machine où la commande s'exécute ; lancée sur une pilotée, la commande est transmise à la principale. En cas d'erreur (Baskul pas lancé, machines pas connectées…), un message s'affiche sur la sortie d'erreur et le code de sortie vaut 1. Le chemin complet de la commande, prêt à copier, est dans les réglages (onglet Avancé). Sur macOS : `/Applications/Baskul.app/Contents/MacOS/baskul switch here`, par exemple dans une action « Execute Shell Script » de Keyboard Maestro. Sous Windows, `baskul.exe` est une application sans console : dans un script, lancez-la avec `start /wait` pour attendre son code de sortie.

### Correspondance des touches (position physique)

Les touches gardent leur place, quel que soit le sens :

| Clavier de PC | Clavier Apple |
|---|---|
| Ctrl | Control ⌃ |
| Windows ⊞ | Commande ⌘ |
| Alt | Option ⌥ |
| AltGr | Option ⌥ (droite) |
| Impr. écran / Arrêt défil. / Pause | F13 / F14 / F15 |

Avec un clavier de PC sur une pilotée macOS, **copier** se fait donc avec **⊞ + C**, puisque la touche Windows est à la place de ⌘. Les caractères suivent la disposition de la pilotée : avec un Mac en AZERTY, `@` est sur la touche à gauche du `1`, et `Option ⌥` donne les caractères spéciaux du Mac.

Pour avoir ⌘ à côté de la barre d'espace, comme sur un clavier Apple, ajoutez dans la configuration **de la pilotée macOS** :

```toml
[keyboard.remap]
AltLeft = "MetaLeft"     # Alt du PC     → ⌘
MetaLeft = "AltLeft"     # Windows du PC → ⌥
```

---

## 4. Réglages (sans recompiler)

Dans le menu de Baskul, choisissez **Réglages…** : la fenêtre de Baskul s'ouvre. Relancer Baskul alors qu'il tourne déjà (double-clic sur l'app) l'ouvre aussi. Le menu de gauche mène à :

- **Accueil** : l'état de Baskul en direct (machines connectées, celle que pilotent le clavier et la souris, raccourcis), avec des boutons « Aller sur » / « Revenir ici » ;
- **Réglages** : modifiez, puis cliquez sur **Enregistrer et redémarrer** ; Baskul redémarre et la fenêtre affiche aussitôt le nouvel état ;
- **Journal** : les dernières lignes du journal de Baskul, en direct ;
- **À propos** : versions, mises à jour, fichier de configuration, licence.

Dans les réglages :

- Les réglages sont rangés par onglets. Ceux qui ne concernent pas le rôle de cette machine sont masqués ; ceux qui ne servent que sur un système (par exemple « macOS uniquement ») sont grisés ailleurs.
- Pour un raccourci, cliquez sur « Modifier » puis tapez la combinaison. Sur la machine principale, c'est Baskul lui-même qui la relève : elle ne déclenche alors aucun raccourci, et toutes les touches fonctionnent (F11, F12…).
- Le bouton « Ouvrir le fichier » donne accès au fichier texte, par exemple pour le remappage de touches. Quand les réglages sont enregistrés, les commentaires du fichier sont conservés.
- Si le fichier de configuration contient une erreur, Baskul ouvre ses réglages pour la corriger.
- `baskul --settings` ouvre les réglages.

Le fichier complet et commenté est dans [`docs/config.exemple.toml`](docs/config.exemple.toml). Après une modification à la main, faites **Redémarrer Baskul** dans le menu.

| Section | Réglage | Effet |
|---|---|---|
| `[network]` | `secret` | Secret partagé, identique sur toutes les machines |
| | `connect` (pilotée) | Adresse de la principale ; vide : trouvée automatiquement |
| | `listen` (principale) | Adresse et port d'écoute (`0.0.0.0:24870`) |
| | `interface` | Carte réseau à utiliser (`"en0"`, `"Ethernet"`…) ; vide : automatique |
| | `discovery` | Découverte automatique (Bonjour) : la principale s'annonce, les pilotées la cherchent |
| `[layout]` (principale) | `machines = [{ name = "Atelier", x = 1, y = 0, hotkey = "Ctrl+Alt+1" }]` | Grille des pilotées : la principale est en (0, 0), `x` vers la droite, `y` vers le bas ; `hotkey` facultatif |
| `[screen]` (principale) | `edge_switching` | Bascule par les bords de l'écran (`false` : seulement au clavier) |
| | `secondary_position` | Côté où se place une nouvelle pilotée |
| | `corner_size` | Coins neutres, en pixels |
| `[hotkeys]` (principale) | `switch_to_secondary`, `return_to_primary` | Aller sur la dernière pilotée (`"Shift+F12"`), revenir sur la principale (`"Shift+F11"`) |
| | `next_machine` | Machine suivante |
| | `toggle_lock` | Verrouillage du curseur (ex. `"ScrollLock"`) |
| `[keyboard]` (principale) | `media_keys = "cursor"` / `"primary"` / `"secondary"` | Les touches volume et lecture agissent sur la machine du curseur, toujours sur la principale, ou sur la dernière pilotée utilisée |
| `[keyboard]` (pilotée) | `[keyboard.remap]` | Remappage de touches physiques |
| | `iso_swap = "auto"` (macOS) | Correction des touches `²` et `<` des claviers européens ; à forcer seulement si elles restent inversées |
| `[mouse]` (pilotée) | `speed = 1.0` | Vitesse du curseur |
| `[scroll]` (pilotée) | `mode = "lines"` / `"pixels"` / `"smooth"` | Par lignes, comme une molette Windows ; au pixel (précis et identique dans toutes les apps) ; ou fluide, à la manière de Mac Mouse Fix (macOS) |
| | `lines_per_notch`, `pixels_per_notch` | Vitesse en mode lignes ou pixels (par cran de molette) |
| | `smooth_speed`, `smoothness`, `smooth_precision`, `trackpad_simulation` | Mode fluide : mêmes réglages que Mac Mouse Fix (vitesse, douceur, précision, simulation du trackpad) |
| | `invert_vertical`, `invert_horizontal` | Inverser le sens |
| `[buttons]` (pilotée macOS) | `side_buttons = "buttons"` / `"swipe"` | Boutons 4/5 bruts (Chrome, Firefox, VS Code…) ou geste précédent/suivant (Safari, Finder, Xcode…) |
| `[clipboard]` | `enabled`, `images`, `html`, `files` | Ce qui est partagé |
| | `max_size_mb`, `max_files_mb` | Tailles maximales (texte et images ; fichiers) |
| | `skip_sensitive` | Ignorer les mots de passe |
| `[activity]` | `mirror = true` | Activité partagée |
| `[ui]` | `theme`, `language`, `tray_icon` | Thème et langue de la fenêtre, icône en couleurs ou monochrome |
| | `arrival_bubble = true` | Bulle d'arrivée |
| `[updates]` | `check = true` | Recherche et téléchargement automatiques des mises à jour |
| `[log]` | `level = "debug"` | Journal détaillé, utile pour un diagnostic |

Pour le défilement d'une pilotée macOS, commencez par `mode = "lines"`. Si la vitesse varie trop d'une app à l'autre (Safari vs Chrome), essayez `mode = "pixels"` avec `pixels_per_notch` entre 60 et 120. Pour retrouver le défilement de Mac Mouse Fix, choisissez `mode = "smooth"` avec la même vitesse et la même douceur que dans Mac Mouse Fix.

> **Mac Mouse Fix** et les logiciels du même genre ne s'activent que si une souris est branchée sur le Mac : ils n'agissent donc pas sur la souris de la principale. (D'après son code source, si une souris, même inutilisée, est branchée sur le Mac, Mac Mouse Fix traite aussi les défilements envoyés par Baskul en mode `"lines"` : réglez alors `lines_per_notch = 1.0`, un cran de Baskul valant un cran pour Mac Mouse Fix. Les modes `"pixels"` et `"smooth"` lui échappent.) Le mode `"smooth"` de Baskul reprend leur algorithme : la vitesse de rotation de la molette est mesurée (les crans rapprochés vont plus loin), plusieurs gestes rapides enchaînés accélèrent encore, et chaque cran est animé à vitesse constante puis freiné jusqu'à l'arrêt ; un changement de sens arrête net le défilement. Avec la simulation du trackpad, la phase à vitesse constante est envoyée comme un geste à deux doigts et le freinage comme l'élan qui suit. La vitesse « macOS » de Mac Mouse Fix n'existe pas dans Baskul : elle reprend l'accélération que macOS applique à une souris branchée sur le Mac, que Baskul ne reçoit pas.

---

## 5. Mises à jour

Peu après son démarrage, puis toutes les six heures, Baskul cherche une nouvelle version dans les [Releases](https://github.com/StarbreakersNet/baskul/releases). S'il en trouve une, il la télécharge en arrière-plan et vérifie son empreinte SHA-256. Il ne reste plus qu'à cliquer sur **Redémarrer pour mettre à jour** : dans le menu de l'icône, dans le bandeau de la fenêtre de Baskul, ou dans la page À propos (qui permet aussi de chercher tout de suite). Baskul remplace ses programmes et redémarre dans la nouvelle version ; lancée depuis la fenêtre de Baskul, la mise à jour la rouvre ensuite.

- Mettez à jour **toutes les machines** : des versions trop différentes ne se connectent pas (le menu affiche « versions incompatibles »).
- macOS : Baskul.app est remplacée là où elle se trouve ; il faut donc pouvoir écrire dans son dossier (c'est le cas d'un compte administrateur dans Applications). L'app n'étant pas signée par un développeur identifié, macOS peut redemander l'autorisation **Accessibilité** après une mise à jour (voir [Dépannage](#6-dépannage)).
- Windows : les programmes sont remplacés dans leur dossier (les anciens, renommés en `.old`, sont effacés au démarrage suivant).
- Linux : la nouvelle version est signalée ; installez-la à la main.
- Pour ne plus chercher automatiquement : « Mises à jour automatiques », onglet Avancé (`[updates] check = false`).

---

## 6. Dépannage

**L'icône reste grise ou rouge, sans connexion**
- Le `secret` doit être **identique** sur toutes les machines. En cas d'erreur, le menu affiche « secret incorrect ».
- Il faut **une seule** principale ; les autres machines sont pilotées. Le menu signale l'erreur sinon.
- Toutes les machines doivent avoir la **même version** de Baskul. Sinon, le menu affiche « versions incompatibles ».
- La principale doit accepter les connexions entrantes (port TCP 24870 par défaut) : pare-feu de Windows (*Autoriser une application via le pare-feu*, réseaux privés) ou de macOS (*Réglages → Réseau → Pare-feu → Options*).
- Les machines doivent être sur le même réseau. Si la découverte automatique ne trouve pas la principale (Wi-Fi invité, réseau d'entreprise, sous-réseaux différents…), indiquez son adresse dans « Se connecter à », et pensez à lui donner une IP fixe dans votre box (bail DHCP statique).
- Plusieurs cartes réseau (Ethernet et Wi-Fi, VPN…) : choisissez la bonne dans « Carte réseau », onglet Connexion.
- Deux pilotées ne peuvent pas porter le même nom : la seconde est refusée et son menu l'indique. Changez son « Nom affiché ».

**Une pilotée n'est pas là où je l'attends.** Dans les réglages de la principale, onglet Changer de machine, déplacez-la sur la grille.

**Les touches `²` / `@` et `<` / `>` sont inversées.** Réglez `[keyboard] iso_swap = "on"` (ou `"off"`) sur la pilotée macOS, puis redémarrez Baskul.

**Le défilement va dans le mauvais sens, trop vite ou trop lentement.** Voir la section `[scroll]` ci-dessus.

**Précédent / suivant (boutons 4/5) ne fonctionne pas dans Safari ou le Finder.** Réglez `side_buttons = "swipe"`.

**Après une mise à jour, macOS ne réagit plus.** Une app non signée par un développeur identifié change d'identité à chaque version. Dans *Accessibilité* (et *Surveillance de l'entrée* sur une principale), supprimez Baskul (bouton **−**), puis ajoutez-la à nouveau (**+**).

Pour éviter cette manipulation avec vos compilations locales, signez-les avec un certificat local :
1. Ouvrez *Trousseaux d'accès → Assistant de certification → Créer un certificat*.
2. Indiquez le nom `Baskul Local`, le type *Racine auto-signée* et le type de certificat *Signature de code*.
3. Compilez avec `CODESIGN_IDENTITY="Baskul Local" ./scripts/build-macos.sh`.

**Win + L verrouille la principale Windows, même quand le curseur est sur une pilotée.** C'est une limite de Windows, comme Ctrl + Alt + Suppr : ces combinaisons ne peuvent pas être interceptées. Baskul relâche alors tout proprement.

**Une fenêtre lancée en administrateur sur une principale Windows (Gestionnaire des tâches…).** Windows interdit à un programme normal d'intercepter le clavier quand une telle fenêtre est active. Baskul reprend le premier plan quand vous passez sur une pilotée. Pour couvrir aussi ce cas, lancez Baskul en administrateur.

**Les réglages ne s'ouvrent pas sous Windows.** L'app de réglages utilise WebView2, inclus dans Windows 11 et installé par les mises à jour de Windows 10. Sinon, installez le *WebView2 Runtime* de Microsoft. En attendant, le menu « Ouvrir le dossier de Baskul » donne accès au fichier `config.toml`.

**Journaux**
- Windows : `%APPDATA%\Baskul\baskul.log`
- macOS : `~/Library/Logs/Baskul/baskul.log`
- Linux : `~/.config/Baskul/baskul.log`

Le menu propose aussi **Ouvrir le journal**. Avec `[log] level = "debug"`, chaque touche ignorée et chaque changement d'écran sont détaillés.

**Configuration**
- Windows : `%APPDATA%\Baskul\config.toml`
- macOS : `~/Library/Application Support/Baskul/config.toml`
- Linux : `~/.config/Baskul/config.toml`

---

## Linux (expérimental)

Sous Linux, Baskul se connecte aux autres machines, partage le presse-papiers (texte surtout) et a les mêmes réglages. En revanche, il **ne partage ni le clavier ni la souris** : cette version sert surtout à tester l'app de réglages et la connexion.

```bash
sudo apt install libwebkit2gtk-4.1-0     # pour l'app de réglages, s'il manque
tar xzf Baskul-Linux.tar.gz
./baskul/baskul
```

Au premier lancement, les réglages s'ouvrent. Il n'y a pas d'icône : pour les rouvrir, relancez `./baskul/baskul` (ou `./baskul/baskul --settings`). Pour tester la liaison, choisissez « Pilotée » sous Linux avec le secret de la principale : les deux machines affichent « Connecté », et un texte copié d'un côté se colle de l'autre.

Pour compiler (Debian, Ubuntu) :

```bash
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev librsvg2-dev libxdo-dev libssl-dev
./scripts/build-linux.sh      # produit dist/baskul/ et dist/Baskul-Linux.tar.gz
```

---

## Développer l'interface (Tauri + Vue 3)

Les réglages sont une app [Tauri 2](https://tauri.app) à part, `baskul-settings`, dans `app/` : Vue 3, TypeScript, Vite, Yarn 4 et [Naive UI](https://www.naiveui.com). Baskul (le daemon) la lance et lui répond par une API locale :

```
Vue (app/src)  →  commandes Tauri (app/src-tauri)  →  API locale du daemon (src/api.rs)
```

Le daemon écrit le port et un jeton de son API dans `api.json`, à côté de `config.toml`, lisible par l'utilisateur seulement. L'app le relit à chaque appel, ce qui suit les redémarrages du daemon.

```bash
corepack enable          # une fois : active Yarn 4 (champ packageManager)
cd app && yarn install
cargo run                # à la racine : lance Baskul (le daemon)
cd app && yarn tauri dev # fenêtre de réglages, avec rechargement à chaud
```

- `src/types.ts` reprend la configuration de `src/config.rs`. Pour un nouveau réglage : `config.rs` (structure, modèle commenté, validation), puis `types.ts` et la section concernée dans `src/sections/`.
- `src/composables/useSettings.ts` gère l'état : chargement, état en direct, enregistrement, attente du redémarrage. `useUpdate.ts` gère les mises à jour.
- `src/components/LayoutEditor.vue` : la grille des machines (glisser-déposer au pointeur, ou sélection puis clic).
- Pages (`src/pages/`) et menu latéral (`src/layouts/AppLayout.vue`) : vue-router. Icônes : [Tabler Icons](https://tabler.io/icons) (`@tabler/icons-vue`).
- **Traductions** : `locales/fr.json` et `locales/en.json`, à la racine du dépôt. La section `app` est lue par [vue-i18n](https://vue-i18n.intlify.dev) (`$t("…")` dans les composants), la section `daemon` par Baskul (`src/i18n.rs`, menus et état de la connexion). Pour une nouvelle langue : un fichier de plus, avec les mêmes clés.
- `yarn build` vérifie les types (`vue-tsc`) puis construit le front, qui est intégré à l'exécutable par `cargo build -p baskul-settings --features custom-protocol`.

---

## Pourquoi pas de bug des touches mortes ?

Deskflow et Synergy **traduisent les frappes en caractères sur la machine qui a le clavier** (fonction Windows `ToUnicode`). Ils tentent ensuite de retrouver, sur l'autre machine, quelle touche produirait ce caractère. Or `ToUnicode` *consomme* l'état des touches mortes de Windows. Résultat : `^` puis `e` donne `^e`, `ê` en double, ou rien du tout. Il faut aussi deviner des correspondances entre des dispositions qui ne sont pas les mêmes (AZERTY PC et AZERTY Mac diffèrent).

Baskul ne traduit **rien**. Sur la principale, un hook bas niveau (Windows) ou un `CGEventTap` (macOS) lit le **code physique** de chaque touche (scancode, keycode). Baskul le convertit en code **USB HID**, puis en code de la pilotée, et l'injecte. La composition `^` + `e` → `ê` est faite par le système de la pilotée, avec sa propre disposition. L'état des touches mortes de la principale n'est jamais touché.

Deux détails de Windows sont gérés :
- **AltGr** : Windows génère un faux « Ctrl gauche » en même temps qu'AltGr. Baskul le filtre, et la pilotée ne reçoit que la touche AltGr (⌥ droite sur macOS).
- **Claviers ISO** (européens) : les touches `²` et `<` sont échangées comme le fait macOS pour ses claviers ISO. Le réglage `iso_swap` le contrôle.

---

## Architecture

```
src/
  main.rs          démarrage, arguments, instance unique, reprise des réglages de Bascule
  config.rs        configuration TOML commentée (créée au 1er lancement)
  protocol.rs      messages (postcard)
  crypto.rs        chiffrement Noise NNpsk0 (clé dérivée du secret)
  net.rs           connexions TCP (une par pilotée), reconnexion, priorité souris/clavier > presse-papiers
  discovery.rs     découverte automatique (Bonjour / mDNS)
  iface.rs         cartes réseau
  keymap.rs        table USB HID ↔ scancode Windows ↔ keycode Mac (données Chromium)
  geometry.rs      écrans, bords, conversion des positions
  primary.rs       logique de la principale (grille, bascule, routage des touches…)
  secondary.rs     logique d'une pilotée (position, sortie par le bord…)
  clipboard.rs     presse-papiers (texte, HTML, images d'origine, fichiers en flux), relais, anti-écho
  hotkey.rs        raccourcis
  capture.rs       saisie d'un raccourci pour l'app (crochet clavier)
  smooth.rs        défilement fluide (algorithme de Mac Mouse Fix, simulation du trackpad)
  update.rs        mises à jour (Releases GitHub, SHA-256, installation, redémarrage)
  icon.rs          icône (couleur ou monochrome)
  i18n.rs          traductions des menus et de l'état (locales/*.json)
  api.rs           API locale (127.0.0.1 + jeton) pour l'app de réglages et les commandes
  cli.rs           commandes `baskul switch …` et `baskul status`
  platform/
    windows.rs     hooks bas niveau, masquage du curseur, SendInput, presse-papiers, bulle, icône
    macos.rs       CGEventTap, CGEventPost, gestes, Verr. Maj (IOKit), presse-papiers, bulle, barre des menus
    linux.rs       version minimale (connexion, presse-papiers, réglages)
locales/           traductions (fr, en), partagées par Baskul et l'app
app/               app de Baskul : Tauri 2 + Vue 3 + Naive UI
  src/             front (pages, sections, composants, état)
  src-tauri/       commandes Tauri, client de l'API du daemon
```

Tests : `cargo test` (dont des tests de bout en bout sur de vraies connexions TCP, avec plusieurs pilotées). Le code Windows et macOS se vérifie aussi depuis Linux avec `./scripts/xcheck.sh`.
