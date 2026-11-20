# Calculatrice de chantier

> **English** — *Jobsite Calculator* is a free construction calculator for every trade:
> feet-inches-fractions and metric, a framer's keypad, and 76 tools for eleven other
> trades. The whole app is available in English: tap **EN** in the top bar.
> [Open the calculator](https://constructoai.github.io/calculatrice-chantier/) ·
> [User manual](https://constructoai.github.io/calculatrice-chantier/manual.html)

Calculatrice de construction en français du Québec et en anglais, pour tous les corps de
métier. Elle calcule en pieds-pouces-fractions et en métrique, et résout les problèmes
courants de chantier :

- **au clavier**, comme une calculatrice de charpentier : pentes et chevrons, arêtiers,
  noues et empannons, escaliers, murs en pente et murs cintrés, arcs et cercles, onglets
  composés, quantités de matériaux (blocs, béton de semelle, gypse, bardeaux, montants,
  bois en pmp), conversions et coûts;
- **au clavier de votre métier** : touchez « Plomberie », « Électricité » ou « Béton »
  au-dessus du clavier, et les touches du haut deviennent celles de vos outils —
  76 calculateurs pour onze corps de métier;
- **par formulaires**, dans l'onglet *Métiers* : les mêmes calculateurs, à remplir d'un
  coup d'œil.

Le même programme existe en deux versions :

| Version | Pour l'utiliser |
|---|---|
| **Web** — ordinateur, tablette, téléphone | Ouvrir <https://constructoai.github.io/calculatrice-chantier/>. Elle s'utilise telle quelle, ou s'installe sur l'appareil pour servir hors ligne (voir plus bas). |
| **Bureau** — Windows 10 ou 11 (64 bits) | Télécharger `calculatrice-chantier-windows-x64.exe` depuis la [dernière publication](https://github.com/ConstructoAI/calculatrice-chantier/releases/latest), puis le lancer : rien à installer. |

L'exécutable de bureau n'est pas signé. Au premier lancement, Windows peut afficher
« Windows a protégé votre ordinateur » : choisir *Informations complémentaires*, puis
*Exécuter quand même*.

La calculatrice est gratuite, sans publicité et sans compte : une gracieuseté de
Sylvain Leduc de [Constructo AI](https://constructoai.ca/). C'est un logiciel libre, sous
licence Apache 2.0.

**Le [manuel d'utilisation](https://constructoai.github.io/calculatrice-chantier/manuel.html)**
explique tout, exemples à l'appui : il s'ouvre aussi depuis le menu ☰ de la calculatrice
et se lit sans réseau. Il existe aussi
[en anglais](https://constructoai.github.io/calculatrice-chantier/manual.html). Le
comportement complet du programme est décrit dans `CAHIER_DES_CHARGES.md`.

> Cette calculatrice est un outil d'aide au calcul. Elle ne remplace ni les plans, ni le
> jugement d'une personne compétente, ni les exigences du Code de construction du Québec
> et de la réglementation municipale. Vérifiez toujours vos résultats avant de couper ou
> de commander.

## Utilisation rapide

- Tapez une mesure comme vous la dites : `5` `Pieds` `7` `Pouces` `1` `/` `2`.
- Les touches du haut résolvent le triangle : donnez deux valeurs parmi Pente, Montée,
  Course et Diag, puis appuyez sur la troisième.
- `Conv` donne accès aux fonctions écrites en petit au-dessus des touches et convertit
  les unités. Un clic sur la petite étiquette fait la même chose.
- Appuyez de nouveau sur une touche de fonction pour faire défiler tous ses résultats.
- Clic droit ou appui long sur une touche : sa fiche d'aide.
- Votre métier n'est pas la charpente? Touchez son nom dans la barre au-dessus du
  clavier (voir ci-dessous).

## En français ou en anglais

Le bouton **FR | EN**, dans la barre du haut, fait passer toute la calculatrice d'une
langue à l'autre : les touches, l'afficheur, les 76 outils, l'aide et le manuel. Le choix
se fait aussi dans les Préférences, et il est conservé.

| | Français | English |
|---|---|---|
| Touches | `Pente` `Montée` `Course` `Pieds` `Pouces` | `Pitch` `Rise` `Run` `Feet` `Inch` |
| Mesures | `5 pi 7 1/2 po`, `480 pi²`, `5,93 vg³` | `5 ft 7 1/2 in`, `480 ft²`, `5.93 yd³` |
| Nombres | `1 234,56` | `1,234.56` |
| Montants | `1 283,06 $` | `$1,283.06` |

Vos valeurs ne changent pas avec la langue : ce qui est tapé dans un outil est récrit dans
la nouvelle langue (`2,032 m` devient `2.032 m`). Les unités se tapent dans les deux
langues, quelle que soit celle de l'écran : `12 pi 6 po` comme `12 ft 6 in`. Au premier
lancement, la calculatrice prend la langue de l'appareil si c'est le français ou
l'anglais.

## Le clavier de votre métier

Au-dessus du clavier, une barre montre les métiers. Touchez le vôtre : les touches du
haut deviennent celles de ses outils, et la barre vous les présente — un toucher pour
passer de l'un à l'autre.

Exemple, pour une dalle de béton de 24 pi sur 20 pi, épaisse de 4 po : touchez *Béton*,
puis

`24` `Pieds` `Longueur` · `20` `Pieds` `Largeur` · `4` `Pouces` `Épaisseur` · `Résultat`

L'afficheur donne la surface; `Résultat` de nouveau, le volume (`5,93 vg³ (4,53 m³)`),
puis le volume à commander, pertes comprises. Le calcul complet s'inscrit au ruban.

- **Une touche de champ** range la valeur tapée; la valeur s'écrit au-dessus de la
  touche. Sans rien taper, la touche rappelle sa valeur.
- **Une touche foncée** fait défiler les options d'une liste (l'angle d'un raccord, le
  calibre d'un conducteur).
- **`Résultat`** calcule, puis fait défiler les résultats. Un résultat affiché sert à la
  suite : `×` `2` `=`, `M+`… et vous suit quand vous changez d'outil.
- **`Conv` puis une touche de champ** la remet à sa valeur de départ; l'étiquette
  *Recommencer*, au-dessus de `Résultat`, remet tous les champs au départ.
- **Appui long** sur une touche de l'outil : son mode d'emploi.

L'onglet *Métiers* offre les mêmes calculateurs en formulaires : tous les résultats d'un
coup d'œil, et le bouton *Au clavier* pour brancher un calculateur sur les touches.

| Métier | Outils |
|---|---|
| Estimation et soumission | prix d'une soumission (administration, imprévus, profit, TPS et TVQ), budget au pied carré, coût de la main-d'œuvre, taxes du Québec, majoration et marge, facture d'étape (avancement et retenue), durée des travaux |
| Terrassement et aménagement | excavation et remblai, tranchée, pierre, gravier et sable, voyages de camion, pente et dénivelé, clôture, terrasse (planches et vis), tourbe, béton des poteaux |
| Béton et fondations | dalle, semelle filante, mur de fondation, colonnes et pieux ronds, armature d'une dalle, treillis métallique, béton en sacs, escalier de béton |
| Maçonnerie et pavage | briques et blocs, rangs de maçonnerie, muret de blocs, pavés et dalles |
| Toiture et revêtement | bardeaux d'asphalte, pente d'un toit (sur 12, degrés, pour cent), panneaux de contreplaqué ou d'OSB, revêtement extérieur, membrane et papier en rouleaux |
| Gypse et isolation | panneaux de gypse d'une pièce, montants d'une cloison, isolant, valeur R et valeur RSI, plafond suspendu |
| Finition | peinture d'une pièce, peinture d'une surface, céramique et tuiles, revêtement de sol en boîtes, plinthes et moulures |
| Plomberie | pente d'un drain, déport, déport roulé, déports parallèles, longueur de coupe d'un tuyau, débit et vitesse de l'eau, dilatation d'un tuyau, contenance d'un tuyau, pression d'une colonne d'eau, réservoir cylindrique, chauffage de l'eau |
| Électricité | loi d'Ohm et puissance, chute de tension, calibre selon la chute de tension, puissance d'un circuit, courants d'un transformateur, courant d'un moteur, résistances en série et en parallèle, remplissage d'un conduit, cintrage d'un conduit, éclairage (nombre de luminaires), coût d'utilisation |
| Ventilation et chauffage | débit d'un conduit rond ou rectangulaire, conduit rond pour un débit, changements d'air à l'heure, chaleur transportée par l'air, conversion BTU/h, kW et tonnes, tuyau de plancher chauffant |
| Acier et soudure | masse d'une plaque, d'une barre ou d'un tube rond, d'une barre plate ou d'un tube rectangulaire, cordon de soudure d'angle |

Ces outils font de la géométrie, de la physique et de l'arithmétique. Ils ne reproduisent
aucune table de code (intensités admissibles, calibres minimaux, pentes minimales,
remplissage permis), ne contiennent aucun prix (chacun saisit ses coûts et ses taux) et ne
remplacent ni les plans ni les codes en vigueur.

## Installer sur un téléphone ou une tablette

La version Web s'installe comme une application : elle a alors son icône, s'ouvre en
plein écran et fonctionne sans réseau, même au fond d'un sous-sol.

- **Android (Chrome)** : ouvrir le site, puis menu ☰ de la calculatrice → *Installer
  l'application*. Ou, dans le menu ⋮ de Chrome : *Installer l'application*.
- **iPhone et iPad (Safari)** : ouvrir le site, toucher *Partager*, puis *Sur l'écran
  d'accueil*.
- **Ordinateur (Chrome ou Edge)** : menu ☰ de la calculatrice → *Installer
  l'application*, ou l'icône d'installation dans la barre d'adresse.

Les mises à jour arrivent d'elles-mêmes : à chaque ouverture avec du réseau, la
calculatrice prend la dernière version.

## Les deux versions en bref

| | Bureau | Web |
|---|---|---|
| Données (réglages, mémoires, rubans) | fichiers JSON dans `%APPDATA%\CalculatriceChantier\` | stockage local du navigateur, sur l'appareil |
| Export d'un ruban en `.txt` | boîte « Enregistrer sous » | téléchargement |
| Plein écran | préférence, bouton ou `F11` | bouton (là où le navigateur le permet) ou `F11` du navigateur |
| Police | Segoe UI (Windows) | Selawik, police libre aux mêmes largeurs |
| Menu « Quitter » | oui | non : on ferme l'onglet |
| Installation | un fichier `.exe` à lancer | menu « Installer l'application »; utilisable hors ligne ensuite |

Dans les deux cas, aucune donnée n'est envoyée sur Internet : pas de télémétrie, pas de
compte. Dans la version Web, effacer les données de navigation du site efface aussi les
réglages et les rubans enregistrés.

## Apparence

Thème clair ou sombre, au choix dans les Préférences, aux couleurs de Constructo AI :
barres bleu marine, bleu primaire, accent orange et fond façon plan quadrillé. Les
dégradés, les ombres, les pictogrammes et l'icône sont dessinés par le code, sans aucun
fichier image.

## Raccourcis clavier

| Touche du clavier | Action | | Touche du clavier | Action |
|---|---|---|---|---|
| `0` – `9` | chiffres | | `'` | Pieds |
| `,` ou `.` | virgule | | `"` | Pouces |
| `+` `-` `*` | + − × | | `V` | Verges |
| `:` | ÷ | | `M` | m |
| `/` | barre de fraction | | `%` | % |
| `Entrée` ou `=` | = | | `Espace` | Conv |
| `Retour arrière` | ⌫ | | `S` | Stocker |
| `Échap` ou `Suppr` | Effacer | | `R` | Rappel |
| `P` | Pente | | `Inser` | M+ |
| `T` | Montée | | `F1` | Aide |
| `C` | Course | | `F11` | plein écran |
| `D` | Diag | | `Ctrl+,` | Préférences |
| `A` | Arêtier Noue | | `Ctrl+R` | ruban |
| `O` | Onglet composé | | `Ctrl+C` | copier la valeur affichée |
| `E` | Escalier | | `Ctrl+V` | coller un nombre ou une longueur |
| `B` | Arc | | `L` | Longueur |
| `I` | Cercle | | `G` | Largeur |
| `J` | Empannon | | `H` | Hauteur |

Dans la version Web, `F11` et `Ctrl+R` restent ceux du navigateur (plein écran,
recharger la page). Les raccourcis sont les mêmes en anglais.

## Compiler soi-même

Il faut Rust stable 1.95 ou plus récent (`rustup update stable`).

### Version de bureau

```powershell
cargo run -p calculatrice-chantier                  # lancer en mode débogage
cargo build -p calculatrice-chantier --release      # exécutable final
```

L'exécutable final se trouve dans `target\release\calculatrice-chantier.exe`. Il est
autonome : un seul fichier, aucune installation. Le code reste portable sous Linux et
macOS.

### Version Web

```powershell
rustup target add wasm32-unknown-unknown    # une seule fois
cargo install --locked trunk                # une seule fois
trunk serve                                 # essayer : http://127.0.0.1:8080
trunk build --release                       # site final, dans dist\
```

Le dossier `dist\` contient un site statique : on peut le déposer tel quel chez
n'importe quel hébergeur.

### Vérifier

```powershell
cargo test --workspace                    # tous les tests
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy --target wasm32-unknown-unknown -p calculatrice-chantier -- -D warnings
cargo fmt --all --check
```

Les tests du noyau rejouent, touche par touche, tous les exemples chiffrés de la
section 19 du cahier des charges, et vérifient chaque calculateur par métier contre des
valeurs calculées à part (section 22.14). Ils rejouent aussi les outils de métier au
clavier (section 23.9) et soumettent chacun des 76 outils à des milliers de touches au
hasard.

Les deux langues ont leurs propres tests (`noyau/tests/langue.rs`) : chaque texte existe
en français et en anglais, les libellés tiennent sur les touches dans les deux langues,
et changer de langue ne change aucune valeur.

Le manuel d'utilisation (`app/manuel.html`, et `app/manual.html` en anglais) est produit
à partir d'un gabarit (`app/outils/manuel-gabarit.html`, `manuel-gabarit-en.html`) et du
contenu même du programme : fiches d'aide, calculateurs, exemples calculés par le noyau.
Un test échoue s'il n'est plus à jour; pour refaire les deux pages :

```powershell
cargo test -p calculatrice-chantier regenerer_le_manuel -- --ignored
```

## Publication automatique

Deux automatisations GitHub (dossier `.github/workflows/`) tiennent les deux versions à
jour :

- **`web.yml`** — à chaque mise à jour de la branche `main`, le site est reconstruit et
  publié sur GitHub Pages.
- **`bureau.yml`** — à chaque étiquette de version, l'exécutable Windows est compilé et
  joint à une publication. Pour publier la version 0.2.0, par exemple : mettre
  `version = "0.2.0"` dans le `Cargo.toml` racine, pousser, puis

  ```powershell
  git tag v0.2.0
  git push origin v0.2.0
  ```

## Organisation du code

| Dossier | Rôle |
|---|---|
| `noyau/` | Bibliothèque de calcul (`calculatrice-noyau`) : unités, saisie, formatage, moteur, fonctions de construction, calculateurs par métier (`noyau/src/metiers/`) et leur branchement sur le clavier (`noyau/src/moteur/outils.rs`). Aucune dépendance graphique; entièrement testée sans écran. |
| `noyau/src/langue.rs` | Les deux langues : chaque texte du programme est écrit en français et en anglais au même endroit, `t("Longueur", "Length")`, et se lit dans la langue en cours. Un texte sans version anglaise ne compile pas. |
| `app/` | Interface graphique (`calculatrice-chantier`) avec eframe/egui : clavier, afficheur, préférences, rubans, aide. Ne contient aucune formule. Ses textes sont dans `app/src/textes.rs` et `app/src/aide.rs`. |
| `app/src/plateforme/` | Tout ce qui diffère entre le bureau (`bureau.rs`) et le navigateur (`web.rs`) : stockage, export, plein écran, démarrage. |
| `app/index.html` | Page hôte de la version Web. |
| `app/manifest.webmanifest`, `app/sw.js` | Ce qui rend la version Web installable et utilisable hors ligne. |
| `app/icones/` | Icônes de l'application installable, dessinées par `app/outils/icones.py`. |
| `app/polices/` | Polices libres de la version Web, avec leurs licences (`LISEZMOI.md`). |
| `app/manuel.html`, `app/manual.html` | Manuel d'utilisation, en français et en anglais : deux pages du site, aussi embarquées dans l'exécutable de bureau. Produites par `app/src/manuel.rs` à partir des gabarits de `app/outils/`. |
| `app/build.rs` | Inscrit dans le programme la révision du code source (l'identifiant du commit), affichée au bas du menu. |
| `LICENSE`, `NOTICE` | Licence Apache 2.0 et avis de droit d'auteur. |

## Licence

La Calculatrice de chantier est un logiciel libre, distribué sous la
[licence Apache, version 2.0](LICENSE). Chacun peut l'utiliser, la copier, la modifier et
la redistribuer, y compris à des fins commerciales, à condition de joindre une copie de la
licence, de conserver les avis de droit d'auteur et le contenu du fichier [`NOTICE`](NOTICE),
et de signaler les fichiers modifiés. La licence ne donne aucun droit sur les noms et les
marques. Le logiciel est fourni « tel quel », sans garantie d'aucune sorte.

Copyright 2026 Sylvain Leduc, Constructo AI — <https://constructoai.ca/> ·
<info@constructoai.ca>

Les polices du dossier `app/polices/` viennent d'autres auteurs et gardent leur propre
licence (SIL OFL 1.1 pour Selawik et Open Sans, licence Bitstream Vera pour DejaVu Sans) :
voir `app/polices/LISEZMOI.md`.

## Nom de l'application

Le nom affiché est défini par la constante `NOM_APPLICATION` de `app/src/textes.rs` :
« Calculatrice de chantier » en français, « Jobsite Calculator » en anglais. La page hôte
de la version Web (`app/index.html`) le répète dans son titre et sur son écran d'attente.
