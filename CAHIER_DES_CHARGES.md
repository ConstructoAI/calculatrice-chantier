# Cahier des charges — Calculatrice de chantier (Rust, français du Québec)

> **Version du document :** 1.11 — 2026-11-21 (1.11 : zoom du dessin, section 25.1; 1.10 : dessin complété — pavé en pieds, pouces et fractions, retraits, plusieurs pièces, ruban, envoi vers un outil, section 25; 1.9 : nom « Constructo AI » et logo, sections 1.3 et 16.1; dessin d'une pièce, section 25; 1.1 : identité visuelle aux couleurs de l'ERP Constructo AI, section 16.3; 1.2 : version Web et publication automatique, section 3.1; 1.3 : application Web installable et utilisable hors ligne, section 3.1; 1.4 : calculateurs par métier, section 22; 1.5 : estimation et soumission, section 22.3; 1.6 : clavier adapté au métier, section 23, et catalogue porté à 76 calculateurs pour onze métiers, section 22; 1.7 : licence Apache 2.0, section 3; manuel d'utilisation, section 15.4; coordonnées, version et révision au pied du menu, section 15.1; 1.8 : interface en français et en anglais, bouton de langue, manuel anglais, section 24)
> **Destinataire :** la prochaine session Claude Code qui réalisera le programme.
> **Dossier de travail :** `C:\Users\ThinkPad\Downloads\Calculatrice\`
> **Photos de référence :** `01.png` à `06.png` (même dossier).

Ce document est **autosuffisant** : il contient tout ce qu'il faut pour écrire, tester et livrer le programme sans autre source. Toutes les formules et tous les exemples chiffrés ont été vérifiés par calcul avant la rédaction.

---

## Table des matières

0. [Mode d'emploi de ce document](#0-mode-demploi-de-ce-document)
1. [Objectif, portée et principes](#1-objectif-portée-et-principes)
2. [Photos de référence — transcription](#2-photos-de-référence--transcription)
3. [Décisions techniques](#3-décisions-techniques)
4. [Terminologie québécoise et libellés](#4-terminologie-québécoise-et-libellés)
5. [Clavier — les 40 touches](#5-clavier--les-40-touches)
6. [Modèle de données](#6-modèle-de-données)
7. [Saisie des valeurs](#7-saisie-des-valeurs)
8. [Affichage et formatage](#8-affichage-et-formatage)
9. [Moteur de calcul](#9-moteur-de-calcul)
10. [Conversions](#10-conversions)
11. [Mémoires et registres](#11-mémoires-et-registres)
12. [Fonctions de construction](#12-fonctions-de-construction)
13. [Préférences et valeurs mémorisées](#13-préférences-et-valeurs-mémorisées)
14. [Ruban de calcul](#14-ruban-de-calcul)
15. [Écrans, menu et aide](#15-écrans-menu-et-aide)
16. [Interface utilisateur](#16-interface-utilisateur)
17. [Persistance](#17-persistance)
18. [Architecture du code](#18-architecture-du-code)
19. [Tests et vecteurs de validation](#19-tests-et-vecteurs-de-validation)
20. [Plan de réalisation par phases](#20-plan-de-réalisation-par-phases)
21. [Définition de « terminé »](#21-définition-de--terminé-)
22. [Calculateurs par métier](#22-calculateurs-par-métier)
23. [Clavier adapté au métier](#23-clavier-adapté-au-métier)
24. [Français et anglais](#24-français-et-anglais)

Annexes : [A. Constantes](#annexe-a--constantes) · [B. Textes d'aide](#annexe-b--textes-daide-français) · [C. Aide-mémoire eframe/egui 0.37](#annexe-c--aide-mémoire-eframeegui-037) · [D. Traçabilité photos → application](#annexe-d--traçabilité-photos--application) · [E. Hors périmètre](#annexe-e--hors-périmètre) · [F. Invite de démarrage](#annexe-f--invite-de-démarrage-de-la-prochaine-session) · [G. Hypothèses retenues](#annexe-g--hypothèses-retenues-à-confirmer-par-lutilisateur-aucune-nest-bloquante)

---

## 0. Mode d'emploi de ce document

**À lire en entier avant d'écrire la première ligne de code.** Ensuite :

1. Suivre le [plan par phases](#20-plan-de-réalisation-par-phases) dans l'ordre. Le noyau de calcul (sans interface) est écrit et testé **avant** l'interface.
2. Les règles numérotées (**R-xx**) sont normatives. Les mots « doit », « ne doit pas » sont des exigences; « peut » est une option.
3. Les [vecteurs de validation](#19-tests-et-vecteurs-de-validation) sont la vérité de référence : si le code et un vecteur divergent, c'est le code qui a tort (sauf erreur manifeste du document, à signaler à l'utilisateur plutôt qu'à contourner en silence).
4. Tout texte visible par l'utilisateur est en **français du Québec** et provient de la [section 4](#4-terminologie-québécoise-et-libellés) ou de l'[annexe B](#annexe-b--textes-daide-français). Ne pas improviser de terminologie de métier : en cas de besoin non couvert, suivre le style des termes existants.
5. L'API d'egui a beaucoup changé depuis la version 0.31. Lire l'[annexe C](#annexe-c--aide-mémoire-eframeegui-037) avant d'écrire l'interface et vérifier toute signature douteuse dans la documentation de la version réellement installée.
6. Ne jamais reproduire dans le code, les tests ou la documentation l'adresse courriel visible sur la photo `05.png` (renseignement personnel).

**Conventions typographiques du document**

| Notation | Sens |
|---|---|
| `[Pente]` | touche principale |
| `[Conv]` `[Pente]` | fonction secondaire (étiquette au-dessus de la touche) |
| `PENTE │ 7 po` | écran : étiquette, puis valeur |
| `MÉM.` | indicateur « valeur mémorisée » |
| R-12 | règle normative numéro 12 |

---

## 1. Objectif, portée et principes

### 1.1 Objectif

Réaliser en **Rust** une calculatrice de construction pour ordinateur (Windows d'abord), entièrement en **français du Québec**, destinée aux charpentiers-menuisiers, entrepreneurs, estimateurs et bricoleurs avertis. Elle calcule en pieds-pouces-fractions et en métrique, et résout les problèmes courants de chantier : pentes et chevrons, arêtiers, noues et empannons, escaliers, murs en pente et murs cintrés, arcs et cercles, onglets composés pour moulures, quantités de matériaux (blocs, béton de semelle, gypse, bardeaux, montants, bois en pmp), conversions et coûts.

### 1.2 Portée fonctionnelle

Le programme offre l'équivalent fonctionnel de tout ce que montrent les six photos de référence :

- le clavier de 40 touches avec ses fonctions principales et secondaires ([section 5](#5-clavier--les-40-touches));
- toutes les préférences et valeurs mémorisées des écrans de réglages ([section 13](#13-préférences-et-valeurs-mémorisées));
- le ruban de calcul et les rubans enregistrés ([section 14](#14-ruban-de-calcul));
- l'aide intégrée et le menu ([section 15](#15-écrans-menu-et-aide)).

S'y ajoutent, à la demande de l'utilisateur (2026-11-19), des **calculateurs par métier** — estimation et soumission, terrassement et aménagement, béton et fondations, maçonnerie et pavage, toiture et revêtement, gypse et isolation, finition, plomberie, électricité, ventilation et chauffage, acier et soudure — décrits à la [section 22](#22-calculateurs-par-métier), et un **clavier qui s'adapte au métier** choisi ([section 23](#23-clavier-adapté-au-métier)). L'utilisateur veut faire de l'application une référence gratuite pour les gens de métier, qu'ils utilisent tous les jours.

L'[annexe D](#annexe-d--traçabilité-photos--application) fait correspondre chaque élément visible sur les photos à son équivalent dans l'application. L'[annexe E](#annexe-e--hors-périmètre) liste ce qui est volontairement exclu.

### 1.3 Originalité (exigence ferme)

Les photos montrent une application commerciale d'un tiers. Elles servent **uniquement de référence fonctionnelle** (quelles fonctions, quelle disposition logique, quels réglages). Le programme à réaliser est une œuvre **originale** :

- **R-01** Ne pas utiliser le nom, les marques, le logo ni le site de l'application de référence, où que ce soit (interface, code, fichiers, « À propos »).
- **R-02** Ne pas copier ses textes d'aide, ses illustrations ni son habillage graphique. L'identité visuelle est définie à la [section 16](#16-interface-utilisateur); les textes d'aide sont ceux de l'[annexe B](#annexe-b--textes-daide-français).
- **R-03** Ne pas inclure les photos de référence dans le binaire ni dans un dépôt public.
- Les méthodes de calcul (trigonométrie, géométrie, règles de métier) et l'organisation d'un clavier de calculatrice sont du domaine fonctionnel et sont reprises librement.

**Nom :** « Constructo AI », en français comme en anglais — choisi par l'utilisateur le 2026-11-21 (auparavant « Calculatrice de chantier » et « Jobsite Calculator »). Il est défini à un seul endroit (`NOM_APPLICATION`). L'adresse du site, le nom du dépôt, celui de l'exécutable et le dossier des données (`CalculatriceChantier`) ne changent pas : personne ne perd ses réglages ni ses rubans.

### 1.4 Principes directeurs

1. **Exactitude d'abord.** Un résultat faux sur un chantier coûte cher. Chaque fonction est couverte par des tests chiffrés.
2. **Français impeccable.** Vocabulaire du métier tel qu'on l'emploie au Québec, typographie française (virgule décimale, espace des milliers, `12,50 $`, `75 %`).
3. **Impérial et métrique à égalité.** Le bois se mesure en pieds et en pouces, les plans officiels sont en métrique : les deux systèmes sont pleinement pris en charge.
4. **Aucune surprise.** Pas de réseau, pas de télémétrie, pas de compte. Les données restent sur le poste.
5. **Simplicité du code.** Noyau pur et testable, interface mince. Pas d'abstraction spéculative.

### 1.5 Avertissement à afficher dans « À propos »

> Cette calculatrice est un outil d'aide au calcul. Elle ne remplace ni les plans, ni le jugement d'une personne compétente, ni les exigences du Code de construction du Québec et de la réglementation municipale. Vérifiez toujours vos résultats avant de couper ou de commander.

Les valeurs par défaut (hauteur de contremarche, giron, échappée) sont des points de départ modifiables et **ne constituent pas** un avis de conformité au Code.

---

## 2. Photos de référence — transcription

Transcription fidèle, pour que la prochaine session n'ait pas à réinterpréter les images. Les libellés anglais ci-dessous sont ceux de l'application de référence; les libellés français retenus sont à la [section 5](#5-clavier--les-40-touches).

### 2.1 `01.png` — écran principal

De haut en bas :

1. **Barre supérieure** sombre : icône menu (☰), icône plein écran (quatre coins), titre, icône réglages (roue dentée).
2. **Afficheur** : grand rectangle de teinte claire, valeur `0.` alignée à droite, en gros chiffres.
3. **Clavier** : grille de **8 rangées × 5 colonnes**. Au-dessus de la plupart des touches, une petite étiquette en italique (fonction secondaire, activée par `Conv`).
4. **Barre inférieure** : trois onglets — *Home*, *Saved Tapes*, *Help*.

Grille (touche principale; étiquette secondaire entre parenthèses; « — » = aucune) :

| Rangée | Col. 1 | Col. 2 | Col. 3 | Col. 4 | Col. 5 |
|---|---|---|---|---|---|
| 1 | Pitch (Slope) | Rise (R/Wall) | Run (Polygon) | Diag (Roof) | Hip/V (Ir/Pitch) |
| 2 | Comp Miter (Spring Angle) | Stair (Riser Limited) | Arc (Radius) | Circ (Column/Cone) | Jack (Ir/Jack) |
| 3 | m (—) | Length (Blocks) | Width (Footing) | Height (Drywall) | % (x²) |
| 4 | Yds (—) | Feet (—) | Inch (—) | / (—) | Clear (√x) |
| 5 | Conv (—) | 7 (cm) | 8 (Bd Ft) | 9 (mm) | ÷ (1/x) |
| 6 | Store (Prefs) | 4 (lbs) | 5 (Studs) | 6 (tons) | × (Clear All) |
| 7 | Rcl (MC) | 1 (kg) | 2 (Acre) | 3 (met tons) | − (+/-) |
| 8 | M+ (M-) | 0 (Cost) | • (dms◄►deg) | = (Tape) | + (π) |

Familles de couleurs observées : touches de fonctions (rangées 1 et 4, `Store`, `Rcl`) dans une teinte ardoise; touches de construction et opérateurs dans une teinte charbon; chiffres en gris clair; `Conv` en orangé; `Clear` en rouge. Ces familles sont reprises **comme principe** (regrouper par couleur), avec une palette originale ([section 16.3](#163-identité-visuelle-originale)).

### 2.2 `02.png` à `05.png` — réglages (une seule liste défilante)

| Section | Réglage | Valeur affichée |
|---|---|---|
| General | Trig Mode | décoché |
| General | Legacy Mode | décoché |
| General | Fast Fractions | décoché |
| General | Haptic Feedback | décoché |
| General | Full Screen Mode | décoché |
| Reset | Reset calculator data and settings | (action) |
| Display Settings | Fractional Resolution | 1/16 |
| Display Settings | Fractional Mode | Standard |
| Display Settings | Exponential Mode | décoché |
| Display Settings | Area Display | Standard |
| Display Settings | Volume Display | Standard |
| Display Settings | Meters | 0.000 |
| Display Settings | Decimal Degrees | 0.00 |
| Display Settings | Comma Separator | coché |
| Display Settings | Unitless Decimal Display | Float |
| Function Settings | Rake Wall | Descending |
| Function Settings | Arched Wall | Outside |
| Function Settings | Jack Rafters | Descending |
| Function Settings | Irregular Jack Rafters | On-Center |
| Stored Settings | Headroom Height | 6 feet 8 inch |
| Stored Settings | Riser Height | 7 inch 1/2 |
| Stored Settings | Floor Thickness | 10 inch |
| Stored Settings | Tread Width | 10 inch |
| Stored Settings | Block Area | 128. square inch |
| Stored Settings | Block Length | 16 inch |
| Stored Settings | Spring Angle | 45.00° |
| Stored Settings | On-Center Spacing | 16 inch |
| Stored Settings | Footing Area | 264. square inch |
| Stored Settings | Weight/Volume | 1.5 ton/yard³ |
| Account Settings | Unlink Google Account Email | (adresse personnelle — **ne pas reproduire**) |

Un bouton de validation (crochet) flotte en bas à droite de l'écran de réglages.

### 2.3 `06.png` — menu latéral

Tiroir ouvert depuis l'icône ☰, par-dessus le clavier : *site Web de l'éditeur*, *Saved Tapes*, *Help*, *Welcome*, *Pro Tips*, *Like Us*, *Contact Us*, *Join Tool Crew*, *Demo Videos*, *Other CI Apps*, *About*.

---

## 3. Décisions techniques

Ces choix sont arrêtés. S'en écarter seulement en cas d'impossibilité technique constatée, et le signaler à l'utilisateur.

| Sujet | Décision |
|---|---|
| Langage | Rust stable, **édition 2024**. La machine a `cargo 1.99.1`; eframe 0.37 exige Rust ≥ 1.95. |
| Plateforme | **Windows 10/11 x64** (cible principale) et **navigateur Web** ([section 3.1](#31-version-web-ajoutée-le-2026-11-19-à-la-demande-de-lutilisateur)). Le code reste portable (Linux, macOS) : aucun appel propre à Windows hors blocs `cfg(windows)`. |
| Interface | **eframe / egui 0.37.x** (dernière version stable au 2026-11-19; licence MIT/Apache). Rendu par défaut (wgpu) sur le bureau, WebGL (`glow`) dans le navigateur. |
| Structure | Espace de travail Cargo à **deux crates** : `noyau` (bibliothèque de calcul, aucune dépendance graphique) et `app` (exécutable graphique). |
| Nombres | `f64` partout, valeurs stockées en unité de base ([section 6](#6-modèle-de-données)). Pas de bibliothèque de précision arbitraire. |
| Sérialisation | `serde` + `serde_json`, fichiers JSON lisibles. |
| Dates | `chrono` (heure locale) pour l'horodatage des rubans enregistrés. |
| Boîte « Enregistrer sous » | `rfd` (export du ruban en `.txt`), sur le bureau. Dans le navigateur, le fichier est téléchargé. |
| Réseau | **Aucun.** Aucune dépendance réseau, aucune télémétrie. La version Web est un site statique : une fois la page chargée, le programme ne fait aucune requête. Les liens du menu et de l'écran « À propos » (courriel, site, manuel, code source) s'ouvrent dans le navigateur ou le logiciel de courriel de l'utilisateur : le programme lui-même ne contacte rien. |
| Licence | **Apache 2.0**, choisie par l'utilisateur le 2026-11-19. Fichiers `LICENSE` (texte officiel, non modifié) et `NOTICE` (avis de droit d'auteur) à la racine; champ `license = "Apache-2.0"` des manifestes. Les polices de `app/polices/` gardent leur propre licence. |
| Langue | **Français du Québec et anglais du Canada**, au choix de l'utilisateur ([section 24](#24-français-et-anglais), ajoutée le 2026-11-19 à sa demande). Le français reste la langue de référence : c'est celle du présent cahier, du code et de ses commentaires, et celle des valeurs d'essai des sections 19, 22.14 et 23.9. |

**Dépendances autorisées (liste fermée)** : `eframe`, `serde`, `serde_json`, `chrono`, `rfd`; en `build-dependencies` facultative : `winresource` (icône de l'exécutable). Pour la cible Web seulement, l'accès au navigateur : `wasm-bindgen`, `wasm-bindgen-futures`, `js-sys` et `web-sys` (eframe les utilise déjà, sauf `wasm-bindgen-futures`, qui lance le programme dans la page). Le crate `noyau` ne dépend que de `serde` (derive). Toute autre dépendance doit être justifiée auprès de l'utilisateur.

**Commandes**

```powershell
cargo test --workspace            # tous les tests
cargo clippy --workspace -- -D warnings
cargo fmt --all
cargo run -p calculatrice-chantier              # lancer en mode débogage
cargo build -p calculatrice-chantier --release  # exécutable final : target\release\calculatrice-chantier.exe

# Version Web (section 3.1)
rustup target add wasm32-unknown-unknown        # une seule fois
cargo install --locked trunk                    # une seule fois
cargo clippy --target wasm32-unknown-unknown -p calculatrice-chantier -- -D warnings
trunk serve                                     # essayer : http://127.0.0.1:8080
trunk build --release                           # site final, dans dist\
```

**Profil de publication** (dans le `Cargo.toml` racine) :

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
```

**Particularités Windows**

- Masquer la console en version finale : `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` en tête de `app/src/main.rs`.
- Si le rendu wgpu échoue sur un poste (vieux pilote, machine virtuelle), activer la fonctionnalité `glow` d'eframe et choisir ce moteur de rendu en repli.
- Exécutable autonome : `.cargo/config.toml` lie la bibliothèque d'exécution C de façon statique (`-C target-feature=+crt-static` pour `x86_64-pc-windows-msvc`), pour que l'exécutable se lance sur un poste sans « Visual C++ Redistributable ».
- Emplacement du projet : créer l'espace de travail **à la racine du dossier de travail**, à côté des photos. Ajouter un `.gitignore` contenant `target/`, `dist/` et `/*.png` à la racine (les photos, à la racine du projet, ne sont pas versionnées; les icônes de `app/icones/` le sont). Ne pas initialiser de dépôt git ni faire de commit sans que l'utilisateur le demande.

### 3.1 Version Web (ajoutée le 2026-11-19, à la demande de l'utilisateur)

Le même programme se compile aussi pour le navigateur (cible `wasm32-unknown-unknown`). Les deux versions partagent tout le code : le noyau, l'interface, les textes et le thème. Seul ce qui dépend de la plateforme diffère, et il est isolé dans `app/src/plateforme/` : `bureau.rs` et `web.rs` offrent exactement les mêmes noms, et un seul des deux est compilé.

| Sujet | Bureau | Web |
|---|---|---|
| Rendu | wgpu (rendu par défaut d'eframe) | WebGL : eframe sans ses fonctionnalités par défaut, avec `glow` |
| Polices | Segoe UI du système, avec repli sur les polices intégrées d'egui | polices libres embarquées (`app/polices/`) : **Selawik**, qui a exactement les largeurs de Segoe UI (même mise en page), des extraits d'**Open Sans** (italique; signes `−` `√` `π` `′` `″` et espace insécable) et un extrait de **DejaVu Sans** (`⁻` `→` `↔` `◄` `►` `⌫`). Les polices intégrées d'egui ne sont pas embarquées. |
| Données (section 17) | fichiers du dossier de données | stockage local du navigateur |
| Export du ruban | boîte « Enregistrer sous » | téléchargement du fichier |
| Plein écran | préférence appliquée à la fenêtre; bouton; `F11` | bouton de la barre supérieure, affiché seulement si le navigateur permet le plein écran; pas de préférence, car le navigateur exige un geste de l'utilisateur; `F11` reste celui du navigateur |
| Menu ☰ | avec « Quitter » | sans « Quitter » (on ferme l'onglet) |
| Sauvegarde | au plus une écriture par seconde, et à la fermeture | à chaque changement, sans délai (une page se ferme sans préavis) |

- **Page hôte** : `app/index.html` contient le canevas où le programme se dessine, un écran d'attente et un message d'échec (navigateur sans WebAssembly ou sans WebGL).
- **Construction** : avec **trunk** (`Trunk.toml` à la racine). `trunk serve` pour essayer; `trunk build --release` produit un site statique dans `dist/`.
- **Hébergement** : GitHub Pages, à l'adresse `https://constructoai.github.io/calculatrice-chantier/`. L'automatisation `.github/workflows/web.yml` reconstruit et publie le site à chaque mise à jour de la branche `main`.
- **Exécutable de bureau** : l'automatisation `.github/workflows/bureau.yml` compile l'exécutable Windows et le joint à une publication GitHub à chaque étiquette de version (`v0.1.0`, etc.).
- **Garde-fou des polices** : le test `polices_web_sans_caractere_manquant` relève les caractères des textes du programme et échoue si l'un d'eux n'a pas de glyphe dans les polices embarquées (courant, gras, italique).
- **Application installable** (ajoutée le 2026-11-19, à la demande de l'utilisateur) : la version Web s'installe sur un téléphone, une tablette ou un ordinateur, puis s'ouvre comme une application, sans la barre du navigateur. Elle le doit à son manifeste (`app/manifest.webmanifest` : nom, couleurs, icônes, affichage `standalone`) et à ses icônes PNG (`app/icones/`, produites par `app/outils/icones.py` à partir du dessin de l'icône de fenêtre).
- **Hors ligne** : un service (`app/sw.js`) garde une copie du site dans l'appareil. La page est demandée au réseau d'abord, pour recevoir les mises à jour, puis prise dans la copie locale si le réseau ne répond pas en 4 secondes; les fichiers du programme, dont le nom porte l'empreinte, sont pris dans la copie locale d'abord. Le service ne porte aucun numéro de version : il lit dans la page la liste des fichiers du programme et retire les anciens.
- **Menu « Installer l'application »** (version Web, absent si l'application est déjà installée) : quand le navigateur sait installer (Chrome, Edge), l'entrée lance son invitation; sinon (Safari, Firefox), elle affiche la marche à suivre dans le menu du navigateur.
- **Non réalisé** : synchronisation des données entre appareils.

---

## 4. Terminologie québécoise et libellés

### 4.1 Règles de langue (normatives)

- **R-04** Séparateur décimal : la **virgule**. Séparateur de milliers : l'**espace insécable** (U+00A0). Exemples : `1 234,56`, `0,5`. (En anglais : le point et la virgule, `1,234.56` — section 24.3.)
- **R-05** Une espace insécable entre le nombre et son symbole : `12 pi`, `7 1/2 po`, `2,5 m`, `75 %`, `283,06 $`. Exception : le degré est collé (`36,87°`), de même que minutes et secondes d'angle (`23°42′39″`).
- **R-06** Le symbole du dollar suit le montant : `18,00 $`.
- **R-07** Les symboles d'unités ne prennent ni point ni marque du pluriel : `pi`, `po`, `vg`, `lb`, `pmp`.
- **R-08** Signe moins typographique `−` (U+2212) à l'affichage; `×` et `÷` pour les opérateurs.
- **R-09** Majuscules accentuées obligatoires : `MONTÉE`, `ARÊTIER`, `À propos`, `Échappée`.
- **R-10** Aucun mot anglais dans l'interface française (et aucun mot français dans l'interface anglaise, section 24). Les termes de chantier anglais courants (« stud », « jack », « footing », « pitch ») n'apparaissent que dans l'aide, entre guillemets, comme repère pour l'utilisateur.

### 4.2 Unités et symboles

| Unité | Symbole affiché | Nom complet (aide) |
|---|---|---|
| pied | `pi` | pied |
| pouce | `po` | pouce |
| verge | `vg` | verge |
| mètre, centimètre, millimètre | `m`, `cm`, `mm` | — |
| carré, cube | exposant `²`, `³` | `pi²` pied carré, `vg³` verge cube, `m³` mètre cube |
| pied mesure de planche | `pmp` | pied mesure de planche (pied-planche); `Mpmp` = mille pmp |
| acre | `acre` | acre |
| livre | `lb` | livre |
| kilogramme | `kg` | — |
| tonne courte (2 000 lb) | `tc` | tonne courte |
| tonne métrique (1 000 kg) | `t` | tonne métrique |
| degré, minute, seconde | `°`, `′`, `″` | — |

### 4.3 Lexique de métier (anglais de référence → français retenu)

| Référence | Français (Québec) | Remarque |
|---|---|---|
| Pitch | **pente** | exprimée « x po sur 12 » (ex. 7/12), en degrés, en % ou en ratio |
| Slope (ratio) | **ratio de pente** | montée ÷ course |
| Rise | **montée** | côté vertical |
| Run | **course** | côté horizontal |
| Diagonal / common rafter | **diagonale** / chevron commun | hypoténuse |
| Hip / Valley rafter | **arêtier** / **noue** | chevron d'arêtier, chevron de noue |
| Jack rafter | **empannon** | chevron raccourci appuyé sur l'arêtier ou la noue |
| Irregular pitch | **pente irrégulière** | seconde pente d'un toit à deux pentes |
| Plumb cut | **coupe d'aplomb** | coupe verticale |
| Level cut | **coupe de niveau** | coupe horizontale (assise) |
| Cheek cut | **coupe latérale** | biseau de la lame; « coupe de joue » |
| Incremental adjustment | **incrément** | différence commune entre empannons successifs |
| On-center spacing | **entraxe** | « centre à centre », noté c/c |
| Rake wall | **mur en pente** | mur dont la sablière suit la pente (mur rampant, mur pignon) |
| Arched wall | **mur cintré** | montants sous ou au-dessus d'un arc |
| Stud | **montant** | « colombage » dans la langue de chantier |
| Stair | **escalier** | |
| Riser / riser height | **contremarche** / hauteur de contremarche | |
| Tread / tread width | **marche** / **giron** | giron = profondeur utile, sans le nez |
| Stringer | **limon** | aussi « crémaillère » |
| Stairwell opening | **trémie** | ouverture dans le plancher |
| Headroom | **échappée** | hauteur libre au-dessus des marches |
| Floor thickness | **épaisseur du plancher** | |
| Incline angle | **inclinaison** | |
| Overage / underage | **écart** (surplus / déficit) | |
| Compound miter | **onglet composé** | coupe d'onglet + coupe en biseau |
| Miter / bevel | **onglet** / **biseau** | réglages de la scie à onglets |
| Spring (crown) angle | **angle de la moulure** | angle entre le dos de la moulure et le mur |
| Crown moulding | **moulure couronne** | aussi « corniche » |
| Corner angle | **angle du coin** | |
| Arc / chord / segment rise | **arc** / **corde** / **flèche** | |
| Segment area / pie slice | **segment** / **secteur** | aires |
| Radius / diameter / circumference | **rayon** / **diamètre** / **circonférence** | |
| Column / cone | **colonne** / **cône** | |
| Polygon | **polygone** | |
| Length / width / height | **longueur** / **largeur** / **hauteur** | |
| Area / perimeter / volume | **aire** / **périmètre** / **volume** | |
| Square-up | **équerrage** | diagonale de vérification |
| Wall area / room area | **aire des murs** / **aire de la pièce** | pièce = murs + plafond |
| Blocks / bricks | **blocs** / **briques** | |
| Footing | **semelle** | semelle de fondation |
| Drywall | **gypse** | panneaux (feuilles) de gypse |
| Roof (materials) | **toiture** | |
| Roofing square | **carré** | carré de toiture = 100 pi² |
| Bundle | **paquet** | paquet de bardeaux |
| Sheet (4×8) | **feuille** | feuille de contreplaqué, de gypse |
| Board feet | **pmp** | |
| Cost | **coût** | |
| Weight per volume | **masse volumique** | |
| Store / Recall | **Stocker** / **Rappel** | |
| Clear / Clear All | **Effacer** / **Effacer tout** | |
| Paperless tape | **ruban** (de calcul) | |
| Saved tapes | **rubans enregistrés** | |
| Preferences / Settings | **Préférences** | |
| Home / Help / About | **Accueil** / **Aide** / **À propos** | |
| Welcome / Pro Tips | **Bienvenue** / **Trucs de pro** | |
| Full screen | **plein écran** | |
| Fast fractions | **fractions rapides** | |
| Trig mode | **mode trigonométrique** | |

### 4.4 Étiquettes de l'afficheur

L'afficheur montre une **étiquette** au-dessus de la valeur. Liste fermée (énumération `Etiquette`), texte exact :

| Identifiant | Texte | Identifiant | Texte |
|---|---|---|---|
| `Pente` | PENTE | `Rayon` | RAYON |
| `RatioPente` | RATIO DE PENTE | `Diametre` | DIAMÈTRE |
| `PenteIrreguliere` | PENTE IRRÉGULIÈRE | `Aire` | AIRE |
| `Montee` | MONTÉE | `Circonference` | CIRCONFÉRENCE |
| `Course` | COURSE | `Arc` | ARC |
| `Diagonale` | DIAGONALE | `Corde` | CORDE |
| `CoupeAplomb` | COUPE D'APLOMB | `Segment` | SEGMENT |
| `CoupeNiveau` | COUPE DE NIVEAU | `Secteur` | SECTEUR |
| `CoupeLaterale` | COUPE LATÉRALE | `Fleche` | FLÈCHE |
| `CoupeLaterale1` | COUPE LATÉRALE 1 | `MurCintre(n)` | MUR CINTRÉ n |
| `CoupeLaterale2` | COUPE LATÉRALE 2 | `ColonneVolume` | COLONNE – VOLUME |
| `AretierNoue` | ARÊTIER/NOUE | `ColonneSurface` | COLONNE – SURFACE |
| `AretierNoueIrr` | ARÊTIER/NOUE IRRÉG. | `ConeVolume` | CÔNE – VOLUME |
| `Entraxe` | ENTRAXE | `ConeSurface` | CÔNE – SURFACE |
| `Increment` | INCRÉMENT | `AngleInterieur` | ANGLE INTÉRIEUR |
| `Empannon(n)` | EMPANNON n | `DemiAngle` | DEMI-ANGLE |
| `EmpannonIrr(n)` | EMPANNON IRRÉG. n | `Cote` | CÔTÉ |
| `Montant(n)` | MONTANT n | `Perimetre` | PÉRIMÈTRE |
| `Base` | BASE | `NbCotes` | NB CÔTÉS |
| `AngleMur` | ANGLE DU MUR | `Longueur` | LONGUEUR |
| `AireToit` | AIRE DU TOIT | `Largeur` | LARGEUR |
| `Carres` | CARRÉS | `Hauteur` | HAUTEUR |
| `Paquets` | PAQUETS | `Equerrage` | ÉQUERRAGE |
| `Paquet` | PAQUET | `Volume` | VOLUME |
| `Feuilles4x8` | FEUILLES 4×8 | `Murs` | MURS |
| `Feuilles4x9` | FEUILLES 4×9 | `Piece` | PIÈCE |
| `Feuilles4x10` | FEUILLES 4×10 | `Blocs` | BLOCS |
| `Feuilles4x12` | FEUILLES 4×12 | `AireBloc` | AIRE D'UN BLOC |
| `AirePlan` | AIRE EN PLAN | `LongueurBloc` | LONGUEUR D'UN BLOC |
| `Contremarche` | CONTREMARCHE | `Semelle` | SEMELLE |
| `NbContremarches` | NB CONTREMARCHES | `SectionSemelle` | SECTION DE SEMELLE |
| `EcartContremarches` | ÉCART CONTREMARCHES | `Montants` | MONTANTS |
| `Giron` | GIRON | `Cout` | COÛT |
| `NbMarches` | NB MARCHES | `MasseVolumique` | MASSE VOLUMIQUE |
| `EcartGirons` | ÉCART GIRONS | `MPlus` | M+ |
| `Tremie` | TRÉMIE | `MMoins` | M− |
| `Limon` | LIMON | `TotalM` | TOTAL M |
| `Inclinaison` | INCLINAISON | `Moyenne` | MOYENNE |
| `ContremarcheVisee` | CONTREMARCHE VISÉE | `Nombre` | NOMBRE |
| `GironVise` | GIRON VISÉ | `Registre(n)` | M1, M2, M3 |
| `Echappee` | ÉCHAPPÉE | `Resolution` | RÉSOLUTION |
| `EpaisseurPlancher` | ÉPAISSEUR PLANCHER | `Coin` | COIN |
| `Onglet` | ONGLET | `Biseau` | BISEAU |
| `AngleMoulure` | ANGLE MOULURE | | |

Les résultats des opérations arithmétiques, des conversions et des fonctions trigonométriques s'affichent **sans étiquette**.

### 4.5 Messages

| Situation | Texte exact |
|---|---|
| Dépassement de capacité | `ERREUR – dépassement` |
| Division par zéro, racine d'un nombre négatif | `ERREUR – calcul impossible` |
| Unités incompatibles | `ERREUR – unités incompatibles` |
| Saisie invalide | `ERREUR – saisie invalide` |
| Fonction trigonométrique sur une valeur avec unité, arc sinus hors de [−1, 1] | `ERREUR – trigonométrie` |
| Données insuffisantes pour une fonction | `AUCUNE VALEUR` |
| Après `Effacer tout` | `TOUT EFFACÉ` |
| Après effacement de la mémoire | `MÉMOIRE EFFACÉE` |
| Avertissement escalier (contremarche) | `Contremarche plus haute que la valeur visée` |
| Avertissement escalier (inclinaison) | `Escalier raide : inclinaison supérieure de plus de 10 % au rapport visé` |
| Confirmation `Effacer tout` | `Effacer toutes les valeurs mémorisées et les remettre aux valeurs par défaut? Les préférences sont conservées.` — boutons `Annuler`, `Effacer tout` |
| Confirmation réinitialisation | `Réinitialiser toutes les données et toutes les préférences?` — boutons `Annuler`, `Réinitialiser` |

---

## 5. Clavier — les 40 touches

Cette section décrit le **clavier de base** (charpente, toiture, escaliers). Quand un outil de métier est branché sur le clavier, les trois rangées du haut cèdent la place aux touches de l'outil : voir la [section 23](#23-clavier-adapté-au-métier).

### 5.1 Grille française (normative)

Même disposition logique que la référence. Libellé principal; libellé secondaire entre parenthèses.

| Rangée | Col. 1 | Col. 2 | Col. 3 | Col. 4 | Col. 5 |
|---|---|---|---|---|---|
| 1 | **Pente** (Ratio) | **Montée** (Mur en pente) | **Course** (Polygone) | **Diag** (Toiture) | **Arêtier Noue** (Pente irrég.) |
| 2 | **Onglet composé** (Angle moulure) | **Escalier** (Contrem. max.) | **Arc** (Rayon) | **Cercle** (Colonne/Cône) | **Empannon** (Emp. irrég.) |
| 3 | **m** | **Longueur** (Blocs) | **Largeur** (Semelle) | **Hauteur** (Gypse) | **%** (x²) |
| 4 | **Verges** | **Pieds** | **Pouces** | **/** | **Effacer** (√x) |
| 5 | **Conv** | **7** (cm) | **8** (pmp) | **9** (mm) | **÷** (1/x) |
| 6 | **Stocker** (Préf.) | **4** (lb) | **5** (Montants) | **6** (t courte) | **×** (Effacer tout) |
| 7 | **Rappel** (MC) | **1** (kg) | **2** (Acre) | **3** (t métr.) | **−** (+/−) |
| 8 | **M+** (M−) | **0** (Coût) | **,** (dms◄►déc) | **=** (Ruban) | **+** (π) |

Notes :

- La touche décimale porte une **virgule**.
- « Arêtier Noue » et « Onglet composé » s'écrivent sur deux lignes.
- **Mode trigonométrique** ([13.1](#131-général)) : les touches `Longueur`, `Largeur`, `Hauteur` deviennent `sin`, `cos`, `tan`, et leurs étiquettes secondaires deviennent `sin⁻¹`, `cos⁻¹`, `tan⁻¹`.
- Hors grille : une touche **⌫** (retour arrière) dans l'afficheur ([16.2](#162-zones-de-lécran-principal)) et, si l'option est active, la bande de **fractions rapides** ([7.6](#76-fractions-rapides)).

### 5.2 Table de répartition des touches

Identifiant = variante de l'énumération `Touche` du noyau. La colonne « Avec `Conv` » décrit la fonction secondaire; lorsqu'il n'y en a pas, `Conv` + unité déclenche une conversion ([section 10](#10-conversions)).

| # | Identifiant | Fonction principale | Avec `Conv` | Réf. |
|---|---|---|---|---|
| 1 | `Pente` | saisir / calculer la pente | saisir la pente comme **ratio** | 12.1 |
| 2 | `Montee` | saisir / calculer la montée (ou la flèche d'un arc) | **mur en pente** | 12.1, 12.4 |
| 3 | `Course` | saisir / calculer la course (ou la corde d'un arc) | **polygone** | 12.1, 12.10 |
| 4 | `Diag` | saisir / calculer la diagonale; coupes | **toiture** (matériaux) | 12.1, 12.5 |
| 5 | `AretierNoue` | arêtier/noue (régulier ou irrégulier) | saisir la **pente irrégulière** | 12.2 |
| 6 | `OngletCompose` | onglet et biseau d'une moulure | saisir / rappeler l'**angle de la moulure** | 12.7 |
| 7 | `Escalier` | calcul d'escalier | escalier à **contremarche maximale** | 12.6 |
| 8 | `Arc` | saisir / calculer l'arc; murs cintrés | saisir / calculer le **rayon** | 12.8 |
| 9 | `Cercle` | saisir le diamètre; aire, circonférence | **colonne / cône** | 12.8, 12.9 |
| 10 | `Empannon` | empannons (côté régulier d'abord) | empannons (côté **irrégulier** d'abord) | 12.3 |
| 11 | `Metres` | unité m | — (conversion) | 7, 10 |
| 12 | `Longueur` | saisir la longueur | **blocs** | 12.11, 12.12 |
| 13 | `Largeur` | saisir la largeur; aire, équerrage, périmètre | **semelle** | 12.11, 12.13 |
| 14 | `Hauteur` | saisir la hauteur; volume, murs, pièce | **gypse** | 12.11, 12.14 |
| 15 | `Pourcent` | pourcentage | **x²** | 9.6, 9.7 |
| 16 | `Verges` | unité vg | — (conversion) | 7, 10 |
| 17 | `Pieds` | unité pi | — (conversion) | 7, 10 |
| 18 | `Pouces` | unité po | — (conversion) | 7, 10 |
| 19 | `Fraction` | barre de fraction | — | 7.3 |
| 20 | `Effacer` | effacer (1 pression : affichage; 2 : valeurs temporaires) | **√x** | 9.9, 9.7 |
| 21 | `Conv` | préfixe : fonction secondaire / conversion | — | 9.3 |
| 22–24 | `Chiffre(7)`, `Chiffre(8)`, `Chiffre(9)` | chiffres | **cm**, **pmp**, **mm** | 10 |
| 25 | `Diviser` | ÷ | **1/x** | 9.7 |
| 26 | `Stocker` | préfixe : mise en mémoire | ouvrir les **Préférences** | 11 |
| 27–29 | `Chiffre(4)`, `Chiffre(5)`, `Chiffre(6)` | chiffres | **lb**, **montants**, **t courte** | 10, 12.15 |
| 30 | `Multiplier` | × | **Effacer tout** | 9.9 |
| 31 | `Rappel` | préfixe : rappel | **MC** (effacer la mémoire M+) | 11 |
| 32–34 | `Chiffre(1)`, `Chiffre(2)`, `Chiffre(3)` | chiffres | **kg**, **acre**, **t métr.** | 10 |
| 35 | `Soustraire` | − | **+/−** (changer le signe) | 9.7 |
| 36 | `MPlus` | ajouter à la mémoire | **M−** (soustraire de la mémoire) | 11 |
| 37 | `Chiffre(0)` | chiffre | **coût** | 12.17 |
| 38 | `Virgule` | séparateur décimal (et D:M:S) | **dms◄►déc** | 7.5 |
| 39 | `Egal` | = | afficher le **ruban** | 14 |
| 40 | `Additionner` | + | **π** | 9.7 |
| — | `RetourArriere` | supprimer la dernière frappe | — | 7.7 |
| — | `FractionRapide { num, den }` | fraction de pouce en une touche | — | 7.6 |

---

## 6. Modèle de données

### 6.1 Valeur

Toute quantité manipulée est une `Valeur` : un nombre en **unité de base**, un **genre** (dimension physique) et un **format** d'affichage.

```rust
pub enum Genre { SansUnite, Longueur, Aire, Volume, Masse }

pub enum Format {
    // SansUnite
    Nombre, Degres, Dms, Pourcent, Monnaie,
    // Longueur
    PiPo,   // pieds-pouces-fraction : 5 pi 7 1/2 po
    PiDec,  // pieds décimaux        : 5,625 pi
    PoFrac, // pouces et fraction    : 67 1/2 po
    PoDec,  // pouces décimaux       : 67,5 po
    Vg, M, Cm, Mm,
    // Aire
    Pi2, Po2, Vg2, M2, Cm2, Mm2, Acre,
    // Volume
    Pi3, Po3, Vg3, M3, Cm3, Mm3, Pmp,
    // Masse
    Lb, Kg, Tc, T,
}

pub struct Valeur { pub base: f64, pub genre: Genre, pub format: Format }
```

| Genre | Unité de base (`base`) | Pourquoi |
|---|---|---|
| `Longueur` | pouce | les fractions binaires (1/2 … 1/64) y sont exactes en `f64` |
| `Aire` | pouce carré | |
| `Volume` | pouce cube | |
| `Masse` | livre | |
| `SansUnite` | le nombre lui-même | `Pourcent` : la fraction (75 % → 0,75). `Degres` et `Dms` : degrés décimaux. `Monnaie` : dollars. |

- **R-11** Le format n'altère jamais `base`. Convertir une valeur, c'est changer son `format` seulement.
- **R-12** Tolérance de comparaison : `EPS = 1e-9` (relative à l'ordre de grandeur du pouce). Toute comparaison « ≤, <, = » sur des `f64` issus de calculs passe par des fonctions utilitaires avec tolérance.

### 6.2 Famille et système d'un format

Deux fonctions utilitaires servent aux règles de produit ([9.5](#95-arithmétique-dimensionnelle)) :

| Formats | Famille | Système |
|---|---|---|
| `PiPo`, `PiDec`, `Pi2`, `Pi3`, `Pmp`, `Acre` | Pieds | impérial |
| `PoFrac`, `PoDec`, `Po2`, `Po3` | Pouces | impérial |
| `Vg`, `Vg2`, `Vg3` | Verges | impérial |
| `M`, `M2`, `M3` | Mètres | métrique |
| `Cm`, `Cm2`, `Cm3` | Centimètres | métrique |
| `Mm`, `Mm2`, `Mm3` | Millimètres | métrique |

Formats « de la famille F » : linéaire par défaut (`PiPo`, `PoFrac`, `Vg`, `M`, `Cm`, `Mm`), carré (`Pi2`, `Po2`, …), cube (`Pi3`, `Po3`, …).

### 6.3 Facteurs de conversion (exacts)

Voir l'[annexe A](#annexe-a--constantes). Rappel : 1 po = 25,4 mm exactement; 1 pi = 12 po; 1 vg = 36 po; 1 lb = 0,453 592 37 kg exactement.

---

## 7. Saisie des valeurs

### 7.1 Nombres

- Chiffres `0`–`9` et virgule. Au plus **12 chiffres** par nombre; les frappes excédentaires sont ignorées.
- Une seconde virgule dans un nombre sans unité fait passer en saisie **D:M:S** ([7.5](#75-angles-en-degrés-minutes-secondes)).
- `Conv` `−` (+/−) change le signe de la saisie en cours.
- Une saisie commence toujours une **nouvelle valeur** : si l'afficheur montre un résultat ou une sortie de fonction, le premier chiffre le remplace.

### 7.2 Longueurs en pieds et en pouces

On saisit du plus grand au plus petit : pieds, pouces, fraction.

| Frappes | Valeur | Format |
|---|---|---|
| `5` `Pieds` `1` `Pouces` `1` `/` `2` | 5 pi 1 1/2 po | `PiPo` |
| `5` `Pieds` `6` | 5 pi 6 po (pouces implicites) | `PiPo` |
| `5` `Pieds` `1` `/` `2` | 5 pi 0 1/2 po | `PiPo` |
| `8` `Pouces` `1` `/` `8` | 8 1/8 po | `PoFrac` |
| `1` `/` `2` | 1/2 po (une fraction est toujours en pouces) | `PoFrac` |
| `17,32` `Pieds` | 17,32 pi | `PiDec` |
| `9,0625` `Pouces` | 9,0625 po | `PoDec` |
| `14` `Pieds` | 14 pi 0 po | `PiPo` |

Règles :

- **R-13** `Pieds` : le nombre tapé devient les pieds. S'il contient une virgule **et** qu'aucun pouce ni fraction ne suit, le format est `PiDec`; sinon `PiPo`.
- **R-14** `Pouces` : le nombre tapé devient les pouces (après d'éventuels pieds). Sans pieds : `PoDec` si le nombre contient une virgule et qu'aucune fraction ne suit, sinon `PoFrac`.
- **R-15** Après `Pieds`, un nombre laissé « en suspens » au moment de la validation compte pour des **pouces**. Après `Pouces`, un nombre en suspens qui n'est pas suivi de `/` est une erreur de saisie.
- **R-16** Deux fois `Pieds` avec un nombre entre les deux (ex. `5` `Pieds` `6` `Pieds`) : erreur de saisie. Même chose pour `Pouces`.

### 7.3 Fractions

- `/` : le nombre entier qui vient d'être tapé devient le **numérateur**; les chiffres suivants forment le **dénominateur**.
- **R-17** Si aucun dénominateur n'est tapé avant la validation, on prend la **résolution des fractions** en vigueur (`15` `/` `=` donne 15/16 à la résolution 1/16).
- Fractions impropres permises (`3` `/` `2` = 1 1/2 po). Numérateur et dénominateur : entiers de 1 à 4 chiffres; dénominateur nul : erreur de saisie; numérateur avec virgule : erreur de saisie.
- **R-18** Si le dénominateur tapé est une puissance de 2 **plus fine** que la résolution en vigueur (ex. `/64` alors que la résolution est 1/16), la **résolution temporaire** ([8.3](#83-fractions-de-pouce)) prend cette valeur jusqu'au prochain `Effacer`. Ainsi `44` `/` `64` `+` `1` `/` `64` `=` affiche `45/64 po`.

### 7.4 Unités simples, carrés et cubes

- `Verges`, `m`, et les unités secondaires `Conv` `7` (cm), `Conv` `9` (mm) étiquettent le nombre tapé : `5` `Verges` → 5 vg; `17,5` `m` → 17,5 m; `5` `Conv` `7` → 5 cm.
- **R-19** Carré et cube : appuyer **de nouveau sur la même touche d'unité**, immédiatement, fait défiler linéaire → carré → cube → linéaire. `100` `Pieds` `Pieds` → 100 pi²; `100` `Pieds` `Pieds` `Pieds` → 100 pi³; `5` `m` `m` → 5 m².
  - Pour cm et mm, après `Conv` `7` (ou `Conv` `9`), une nouvelle pression de `7` (ou `9`) **sans** `Conv` fait défiler l'exposant : `5` `Conv` `7` `7` → 5 cm².
  - Une saisie en pieds **et** pouces ne peut pas être mise au carré ou au cube de cette façon (R-19 ne s'applique qu'à une unité seule).
- Unités secondaires de masse et d'aire : `2500` `Conv` `4` → 2 500 lb; `Conv` `1` → kg; `Conv` `6` → tc; `Conv` `3` → t; `2` `Conv` `2` → 2 acres.
- **R-20** `Conv` `8` (pmp) sur un **nombre sans unité** l'interprète comme le produit « pouces × pouces × pieds » d'une pièce de bois et donne `N ÷ 12` pmp : `2` `×` `4` `×` `14` `Conv` `8` → 9,333333 pmp. Sur un volume, c'est une conversion ordinaire ([10.3](#103-volumes-masses-pmp-et-acres)).

### 7.5 Angles en degrés-minutes-secondes

- Saisie : degrés, virgule, minutes, virgule, secondes — `23` `,` `42` `,` `39` représente 23°42′39″ (format `Dms`). L'écho de saisie montre `23°42′39″` au fur et à mesure.
- `Conv` `,` (dms◄►déc) bascule l'affichage d'une valeur sans unité entre `Dms` et `Degres`.
- **R-21** Une saisie mal formée est normalisée à la validation : 30°89′ devient 31°29′00″.
- Les valeurs `Dms` s'additionnent et se soustraient entre elles (utile aussi pour des durées en heures-minutes-secondes).

### 7.6 Fractions rapides

Option des préférences ([13.1](#131-général)). Lorsqu'elle est active, une bande de touches apparaît entre l'afficheur et le clavier :

- résolution 1/16, 1/32 ou 1/64 : les quinze seizièmes, sur deux rangées — `1/16 1/8 3/16 1/4 5/16 3/8 7/16 1/2` puis `9/16 5/8 11/16 3/4 13/16 7/8 15/16`;
- résolution 1/8 : `1/8 1/4 3/8 1/2 5/8 3/4 7/8`; résolution 1/4 : `1/4 1/2 3/4`; résolution 1/2 : `1/2`.

Une touche de fraction rapide ajoute la fraction de pouce à la saisie en cours, sans passer par `Pouces` ni `/` :

- `5` `Pieds` `6` `[3/8]` → 5 pi 6 3/8 po;
- `6` `[3/8]` → 6 3/8 po;
- `[3/8]` seule → 3/8 po;
- si la saisie a déjà une fraction, elle est remplacée.

### 7.7 Retour arrière

`⌫` (ou la touche Retour arrière du clavier) annule la **dernière frappe** de la saisie en cours : dernier chiffre, virgule, étiquette d'unité, barre de fraction, exposant. Sans saisie en cours, la touche est sans effet.

### 7.8 Écho de saisie

Pendant la frappe, l'afficheur montre les éléments **dans l'ordre où ils sont tapés** : `5 pi 6 po 1/` puis `5 pi 6 po 1/2`. Dès la validation (opérateur, `=`, touche de fonction), la valeur est remise en forme canonique : `5 pi 6 1/2 po`.

### 7.9 Analyse d'un texte (dialogues de préférences et collage)

La fonction `analyser_longueur(texte, unite_par_defaut)` lit une longueur tapée au clavier d'ordinateur. Elle accepte, sans égard à la casse et aux espaces superflues :

```
longueur  := [signe] ( imperial | metrique | verges | nombre )
imperial  := [ nombre MARQUE_PI ] [ pouces ]            (au moins l'un des deux)
pouces    := ( entier (" " | "-") fraction | fraction | nombre ) [ MARQUE_PO ]
fraction  := entier "/" entier
metrique  := nombre ( "mm" | "cm" | "m" )
verges    := nombre ( "vg" | "verge" | "verges" )
MARQUE_PI := "pi" | "'" | "pied" | "pieds"
MARQUE_PO := "po" | "\"" | "''" | "pouce" | "pouces"
nombre    := chiffres [ ("," | ".") chiffres ]
```

Un nombre seul prend `unite_par_defaut`. Exemples (tous doivent passer en test) : `6'8"` → 80 po · `6 pi 8 po` → 80 po · `7 1/2 po` → 7,5 po · `7-1/2"` → 7,5 po · `5' 7 1/2"` → 67,5 po · `2,032 m` → 80 po · `1/2` (défaut po) → 0,5 po · `16` (défaut po) → 16 po. De même `analyser_aire` accepte `128 po²`, `128 po2`, `1,83 pi²`, `0,17 m²`.

---

## 8. Affichage et formatage

### 8.1 Structure de l'afficheur

Le noyau fournit à l'interface une structure `Ecran` :

```rust
pub struct Ecran {
    pub etiquette: Option<String>,      // « DIAGONALE », « EMPANNON 3 »…
    pub valeur: TexteValeur,            // segments typés (voir 8.9)
    pub memorise: bool,                 // indicateur MÉM.
    pub avertissement: Option<String>,  // texte d'avertissement (escalier)
    pub prefixe: Prefixe,               // Aucun | Conv | Stocker | Rappel
    pub operateur: Option<char>,        // opération en attente : + − × ÷
    pub memoire_active: bool,           // indicateur M (mémoire M+ non vide)
    pub message: Option<String>,        // erreur ou message plein écran
}
```

### 8.2 Nombres décimaux « flottants »

**R-22** Un nombre décimal s'affiche avec **7 chiffres au total, zéro de tête compris**; les zéros de fin sont supprimés.

Algorithme : `n` = nombre de chiffres de la partie entière de |x| (au moins 1); décimales = `max(0, 7 − n)`; formater avec l'arrondi standard de `format!("{:.d$}")`; si l'arrondi a ajouté un chiffre entier, recommencer; retirer les zéros de fin et la virgule orpheline; grouper la partie entière par trois avec l'espace insécable; un résultat nul n'a jamais de signe.

| Valeur | Affichage |
|---|---|
| 4,666666… | `4,666667` |
| 0,518518… | `0,518519` |
| 15,791666… | `15,79167` |
| 1133,980925 | `1 133,981` |
| 13006,4256 | `13 006,43` |
| 1234567,891 | `1 234 568` |
| 0,000123456 | `0,000123` |
| 9,9999999 | `10` |
| 0,1 + 0,2 | `0,3` |

Au-delà de 7 chiffres entiers, la partie entière s'affiche au complet, sans décimales (`12 345 679`).

### 8.3 Fractions de pouce

- **Résolution** : dénominateur maximal — 2, 4, 8, **16** (défaut), 32 ou 64 ([13.2](#132-affichage)).
- **R-23** Arrondi : `unites = floor(|pouces| × resolution + 0,5 + EPS)`, c'est-à-dire au plus proche, la demie vers le haut. Puis `pouces entiers = unites div resolution`, `reste = unites mod resolution`. La **retenue** se propage : 11,99 po donne `1 pi 0 po`, jamais `0 pi 12 po`.
- **Mode des fractions** : *Standard* — la fraction est réduite (8/16 → 1/2); *Constant* — le dénominateur reste celui de la résolution (8/16). Dans les deux modes, une fraction nulle n'est pas affichée.
- **R-24 Résolution temporaire.** Elle remplace la résolution des préférences pour l'affichage jusqu'au prochain `Effacer`. Deux sources : la saisie d'une fraction plus fine (R-18) et, sur une longueur affichée en `PiPo` ou `PoFrac`, les combinaisons `Conv` `1` → 1/16, `Conv` `2` → 1/2, `Conv` `3` → 1/32, `Conv` `4` → 1/4, `Conv` `6` → 1/64, `Conv` `8` → 1/8.

### 8.4 Longueurs

| Format | Rendu | Exemples |
|---|---|---|
| `PiPo` | `{pieds} pi {pouces}[ {num}/{den}] po` — pieds et pouces toujours présents | `24 pi 3 1/2 po`, `14 pi 0 po`, `0 pi 5 13/16 po`, `6 pi 0 1/2 po` |
| `PoFrac` | `{pouces}[ {num}/{den}] po`; si la partie entière est nulle et la fraction non nulle, la fraction seule | `49 po`, `8 1/8 po`, `45/64 po`, `0 po` |
| `PiDec` | flottant + `pi` | `15,79167 pi` |
| `PoDec` | flottant + `po` | `8,125 po` |
| `Vg` | flottant + `vg` | `4,666667 vg` |
| `M` | **3 décimales fixes** (préférence *Mètres* = `0,000`) ou flottant | `4,267 m` |
| `Cm`, `Mm` | flottant | `426,72 cm`, `4 267,2 mm` |

Les pieds se groupent par milliers (`1 250 pi 6 po`). Valeur négative : signe `−` en tête (`−5 pi 3 1/2 po`).

### 8.5 Aires, volumes, masses

Flottant (R-22) suivi du symbole : `60,59375 pi²`, `2 016 po²`, `5,146605 vg³`, `0,396436 m³`, `9,333333 pmp`, `0,229568 acre`, `1 133,981 kg`, `1,25 tc`, `1,133981 t`, `15 000 lb`.

### 8.6 Angles, pourcentages, monnaie, dénombrements

| Nature | Rendu | Exemples |
|---|---|---|
| `Degres` | 2 décimales fixes (préférence *Degrés décimaux* = `0,00`) ou flottant, puis `°` | `36,87°`, `45,00°` |
| `Dms` | `{d}°{mm}′{ss}″`, minutes et secondes sur deux chiffres, secondes arrondies à l'unité | `44°17′24″`, `19°01′53″` |
| `Pourcent` | flottant de la valeur × 100, puis ` %` | `75 %`, `58,78702 %` |
| `Monnaie` | 2 décimales fixes, puis ` $` | `283,06 $`, `18,00 $` |
| `Nombre` | selon la préférence *Décimales (valeurs sans unité)* : flottant (défaut) ou 0 à 4 décimales fixes | `52`, `4,615385` |
| Dénombrements entiers (contremarches, marches, montants, côtés, nombre M) | entier | `16` |
| Carrés, paquets, feuilles de toiture | **2 décimales fixes** | `2,00`, `6,01`, `6,26` |
| Blocs, feuilles de gypse | flottant | `169,5`, `4,6875` |

### 8.7 Grands et petits nombres

- **R-25** Capacité : |x| < 10¹² pour une valeur affichable en clair.
- Préférence *Notation exponentielle* **désactivée** (défaut) : une longueur, une aire ou un volume dont la valeur numérique affichée atteindrait 10⁹ passe automatiquement à l'**unité supérieure** de son système (mm → m; cm → m; po → pi → vg), autant de fois que nécessaire. Si la valeur reste ≥ 10¹² : `ERREUR – dépassement`.
- Préférence **activée** : toute valeur décimale dont |x| ≥ 10⁹ ou 0 < |x| < 10⁻⁶ s'affiche en notation scientifique, mantisse flottante et exposant : `2,5 E9`, `1,2 E−7`.

### 8.8 Zéro et saisie vide

Après effacement, l'afficheur montre `0`, sans étiquette.

### 8.9 Texte structuré

```rust
pub enum Segment {
    Texte(String),                  // chiffres, signes, « ° », « $ », « % »
    Fraction { num: u32, den: u32 },
    Unite(String),                  // « pi », « po », « m² »…
}
pub struct TexteValeur(pub Vec<Segment>);
impl TexteValeur { pub fn en_texte(&self) -> String }  // « 5 pi 7 1/2 po »
```

L'interface dessine les fractions **empilées** (numérateur au-dessus du dénominateur, en plus petit) et les unités en plus petit que les chiffres. `en_texte()` sert au ruban, au presse-papiers, à l'export et aux tests; il utilise l'espace insécable (R-04, R-05). Les tests comparent après remplacement des espaces insécables par des espaces ordinaires.

---

## 9. Moteur de calcul

### 9.1 État

```rust
pub struct Moteur {
    prefs: Preferences,                  // section 13
    memo: ValeursMemorisees,             // section 13.4
    saisie: Saisie,                      // saisie en cours (section 7)
    courant: Valeur,                     // valeur montrée
    provenance: Provenance,              // d'où vient la valeur montrée
    operation: Option<(Valeur, Operateur)>, // opération en attente
    operande_pret: bool,                 // un opérande droit a été fourni depuis l'opérateur
    prefixe: Prefixe,                    // Aucun | Conv | Stocker | Rappel
    triangle: RegistresTriangle,         // pente, montée, course, diagonale (12.1)
    toit: RegistresToit,                 // pente irrégulière, base de mur
    cercle: RegistresCercle,             // diamètre, arc saisi
    dimensions: RegistresDimensions,     // longueur, largeur, hauteur
    divers: RegistresDivers,             // nombre de côtés, angle du coin
    memoire: MemoireCumulative,          // M+ : somme et nombre
    registres: [Option<Valeur>; 3],      // M1, M2, M3
    cycle: Option<Cycle>,                // fonction en cours de défilement
    resolution_temp: Option<u32>,
    ruban: Ruban,
    compteur: u64,                       // ordre chronologique des saisies
}

pub enum Provenance { Zero, Saisie, Resultat, Sortie, Message }
```

| Provenance | Sens |
|---|---|
| `Zero` | afficheur vide (`0`) après effacement |
| `Saisie` | une saisie est en cours |
| `Resultat` | valeur « libre » : saisie validée, résultat de `=` ou de `%`, résultat d'une fonction mathématique, d'une conversion, valeur rappelée, constante π |
| `Sortie` | sortie d'une touche de fonction (avec étiquette) |
| `Message` | erreur ou message |

Interface publique minimale :

```rust
impl Moteur {
    pub fn nouveau(etat: EtatPersistant) -> Self;
    pub fn appuyer(&mut self, touche: Touche) -> Vec<Effet>;
    pub fn ecran(&self) -> Ecran;
    pub fn ruban(&self) -> &Ruban;
    pub fn effacer_tout(&mut self);              // après confirmation par l'interface
    pub fn etat_persistant(&self) -> EtatPersistant;
    pub fn preferences_mut(&mut self) -> &mut Preferences;
    pub fn memorises_mut(&mut self) -> &mut ValeursMemorisees;
}

pub enum Effet {                // actions qui relèvent de l'interface
    OuvrirPreferences,          // Conv Stocker
    BasculerRuban,              // Conv =
    ConfirmerEffacerTout,       // Conv ×
    EtatModifie,                // quelque chose à sauvegarder
}
```

- **R-26** `appuyer` ne panique jamais. Toute anomalie devient un `Erreur` affiché ([9.10](#910-erreurs)).

### 9.2 Cycles de sortie

Quand une touche de fonction **calcule**, le noyau produit d'un coup la liste complète de ses sorties :

```rust
pub struct Sortie {
    pub etiquette: Etiquette,
    pub valeur: Valeur,
    pub memorise: bool,                 // affiche MÉM.
    pub avertissement: Option<Avertissement>,
    pub rendu: Rendu,                   // Normal | Entier | DeuxDecimales (8.6)
}
pub struct Cycle { pub touche: Touche, pub secondaire: bool, pub sorties: Vec<Sortie>, pub index: usize }
```

- **R-27** Tant que l'utilisateur appuie sur **la même touche physique**, l'index avance d'un cran et **reboucle** au début après la dernière sortie. Une fonction lancée avec `Conv` se poursuit sans `Conv` (`Conv` `Escalier`, puis `Escalier`, `Escalier`…).
- **R-28** Toute autre touche met fin au cycle, **sauf une conversion d'unité** (10.1), qui ne change que le format de la sortie affichée. La sortie affichée reste utilisable comme opérande (voir 9.4).
- Chaque fonction de la [section 12](#12-fonctions-de-construction) est donc une **fonction pure** : entrées → `Result<Vec<Sortie>, Erreur>`. C'est ce qui est testé.

### 9.3 Préfixes `Conv`, `Stocker`, `Rappel`

- `Conv` arme le préfixe (indicateur `CONV` à l'écran). La touche suivante exécute sa fonction secondaire, ou une conversion s'il s'agit d'une unité. `Conv` une seconde fois désarme.
- `Stocker` et `Rappel` arment de même leur préfixe ([section 11](#11-mémoires-et-registres)).
- `Rappel` `Conv` `Arêtier Noue` est la seule séquence à deux préfixes (rappel de la pente irrégulière).
- `Effacer` désarme tout préfixe.

### 9.4 Opérateurs et chaînage

- **R-29 Exécution immédiate, de gauche à droite, sans priorité des opérateurs** : `2` `+` `3` `×` `4` `=` donne 20.
- Appui sur un opérateur : la saisie est validée. S'il y a une opération en attente **et** un opérande droit (`operande_pret`), elle est calculée et son résultat devient l'opérande gauche. Puis le nouvel opérateur est mis en attente. Sans opérande droit, l'opérateur en attente est simplement **remplacé**.
- `=` : valide la saisie en cours (remise en forme canonique), puis calcule l'opération en attente; le résultat est `Resultat`. Sans opération en attente : une saisie validée devient `Resultat`; une `Sortie` devient `Resultat` (**promotion** — utile pour la réutiliser dans une autre fonction, voir R-36); sinon, sans effet. Avec une opération en attente mais sans opérande droit, `=` annule l'opération et garde l'opérande gauche.
- Est un opérande droit valable : une saisie, une valeur rappelée, π, une sortie de fonction ou un résultat obtenus **depuis** l'appui sur l'opérateur.
- Après `=`, un chiffre commence un nouveau calcul; un opérateur poursuit avec le résultat.

### 9.5 Arithmétique dimensionnelle

**Addition et soustraction** — les deux opérandes doivent être du **même genre**; sinon `ERREUR – unités incompatibles` (y compris nombre sans unité ± longueur).

**R-30 Format du résultat d'une somme** : celui du **premier** opérande, sauf : pouces (`PoFrac`/`PoDec`) ± pieds (`PiPo`/`PiDec`) → `PiPo`. Entre valeurs sans unité : `Dms` si l'un des deux est `Dms`; sinon `Nombre`.

**Multiplication**

| Opérandes | Résultat |
|---|---|
| sans unité × X (ou X × sans unité) | X, dans le format de X |
| longueur × longueur | aire |
| longueur × aire (ou aire × longueur) | volume |
| toute autre combinaison | `ERREUR – unités incompatibles` |

**Division**

| Opérandes | Résultat |
|---|---|
| X ÷ sans unité | X, dans le format de X |
| X ÷ X (même genre) | sans unité |
| aire ÷ longueur | longueur |
| volume ÷ longueur | aire |
| volume ÷ aire | longueur |
| toute autre combinaison | `ERREUR – unités incompatibles` |
| diviseur nul | `ERREUR – calcul impossible` |

**R-31 Format « Standard » d'une aire ou d'un volume calculés** (préférences *Affichage des aires / des volumes* = Standard) :

| Cas | Aire | Volume |
|---|---|---|
| les opérandes sont de la **même famille** (6.2) | carré de cette famille | cube de cette famille |
| familles différentes, toutes impériales | `Pi2` | `Vg3` |
| familles différentes, toutes métriques | `M2` | `M3` |
| systèmes mélangés | selon le système du **premier** opérande : `Pi2` ou `M2` | `Vg3` ou `M3` |

Exemples : 5 pi 3 po × 11 pi 6 1/2 po → `60,59375 pi²` · 8 po × 16 po → `128 po²` · 10 pi × 6 po → `5 pi²` · 416,875 pi² × 4 po → `5,146605 vg³` · 3 pi × 1 pi 9 5/8 po × 2 pi 4 po → `12,61458 pi³` · 2 m × 50 cm → `1 m²` · 10 pi × 2 m → `65,6168 pi²`.

**R-32** Si la préférence d'affichage des aires (ou des volumes) n'est **pas** Standard, toute aire (ou tout volume) **calculée** — produit, x², sortie de fonction — prend l'unité imposée. Les valeurs saisies et les conversions explicites ne sont pas touchées.

**R-33 Format d'une longueur ou d'une aire issue d'une division** : même famille pour le dividende et le diviseur → format linéaire (ou carré) de cette famille; sinon `PiPo` (ou `Pi2`) si le dividende est impérial, `M` (ou `M2`) s'il est métrique.

### 9.6 Pourcentage

- **R-34** Avec une opération en attente `a ∘ b`, la touche `%` **termine** le calcul (comme `=`) :

| Frappes | Calcul | Exemple |
|---|---|---|
| `a` `+` `b` `%` | a × (1 + b/100) | `137` `Pieds` `Pieds` `+` `10` `%` → `150,7 pi²` |
| `a` `−` `b` `%` | a × (1 − b/100) | `552` `Pieds` `6` `Pouces` `−` `20` `%` → `442 pi 0 po` |
| `a` `×` `b` `%` | a × b/100 | `500` `Pieds` `×` `18` `%` → `90 pi 0 po` |
| `a` `÷` `b` `%` | a ÷ (b/100) | `350` `Verges` `Verges` `Verges` `÷` `80` `%` → `437,5 vg³` |

  `b` doit être sans unité. Le résultat garde le genre et le format de `a`.
- Sans opération en attente, `%` marque le nombre comme **pourcentage** : `75` `%` affiche `75 %` (base 0,75). Cette valeur sert à saisir une pente en pourcentage (`75` `%` `Pente`) et se comporte comme 0,75 dans les calculs.

### 9.7 Fonctions mathématiques

| Touches | Effet | Genres acceptés |
|---|---|---|
| `Conv` `%` (x²) | carré | sans unité → sans unité; longueur → aire (R-31); autre : erreur d'unités |
| `Conv` `Effacer` (√x) | racine carrée | sans unité ≥ 0; aire → longueur (format linéaire de la famille de l'aire); autre : erreur d'unités; négatif : calcul impossible |
| `Conv` `÷` (1/x) | inverse | sans unité non nul |
| `Conv` `−` (+/−) | change le signe de la saisie ou de la valeur montrée | tous |
| `Conv` `+` (π) | place π = 3,141592653589793 comme valeur libre | — |

Ces fonctions valident d'abord la saisie en cours; leur résultat est `Resultat`.

### 9.8 Touches de fonction : « saisir ou calculer »

- **R-35** Règle générale pour une touche de fonction (`Pente`, `Montée`, `Course`, `Diag`, `Longueur`, `Largeur`, `Hauteur`, `Cercle`, `Arc`, et les fonctions secondaires qui acceptent une donnée) :
  - provenance `Saisie` ou `Resultat` → la valeur montrée est une **donnée** : elle est affectée au registre de la touche (après contrôle du genre) et, en général, réaffichée avec son étiquette; la fonction prépare son cycle, dont la **première sortie** apparaîtra à la pression suivante de la même touche;
  - provenance `Zero`, `Sortie` ou `Message` → la touche **calcule** (ou réaffiche la valeur déjà saisie) : elle affiche la première sortie de son cycle, ou la suivante si le cycle est déjà en cours.
  - Quelques fonctions affichent directement leur première sortie même lorsqu'elles reçoivent une donnée (Toiture, Onglet composé, Polygone, Blocs, Semelle, Gypse, Montants) : c'est précisé à la [section 12](#12-fonctions-de-construction).
- **R-36** Une sortie de fonction n'est donc jamais réaffectée par mégarde à une autre fonction. Pour la réutiliser comme donnée, appuyer d'abord sur `=` (promotion, 9.4). Exemple : après `Largeur` `Largeur` `Largeur` (périmètre affiché), `=` puis `Conv` `Longueur` calcule les blocs pour ce périmètre.
- **R-37** Une valeur rappelée (`Rappel` + touche) est une valeur libre, mais appuyer aussitôt sur **sa touche d'origine** ne la réaffecte pas : la touche calcule.
- Les fonctions sans donnée (`Empannon`, `Arêtier Noue`, `Escalier`…) calculent toujours.
- Si une touche de fonction reçoit une donnée pendant qu'une opération est en attente, la donnée est affectée, l'opération reste en attente et `operande_pret` devient vrai.

### 9.9 Effacement

| Geste | Effet |
|---|---|
| `Effacer` (1re pression) | efface la saisie, l'erreur éventuelle et le préfixe; affiche `0` (`Zero`). L'opération en attente et tous les registres sont **conservés**. |
| `Effacer` (2e pression consécutive) | efface en plus : l'opération en attente; la montée, la course et la diagonale; la pente irrégulière et la base de mur; le diamètre et l'arc; la longueur, la largeur et la hauteur; le nombre de côtés et l'angle du coin; le cycle; la résolution temporaire. Ajoute un séparateur au ruban. |
| `Conv` `×` (Effacer tout) | après confirmation : tout ce qui précède, plus la mémoire M+, M1 à M3, la pente mémorisée, le ruban, et **remise aux valeurs par défaut des valeurs mémorisées** (13.4). Les préférences sont conservées. Message `TOUT EFFACÉ`. |
| *Réinitialiser* (Préférences) | `Effacer tout` + remise des préférences à leurs valeurs par défaut. |

**Conservés par `Effacer` ×2** : la **pente saisie** (elle reste mémorisée d'un calcul à l'autre et d'une séance à l'autre), M+, M1–M3, les valeurs mémorisées, les préférences.

### 9.10 Erreurs

```rust
pub enum Erreur { Depassement, Calcul, Unites, Saisie, Trigonometrie, AucuneValeur }
```

Une erreur s'affiche avec le texte de [4.5](#45-messages). **R-38** En état d'erreur, seule la touche `Effacer` est acceptée; elle efface l'erreur et rend la main. L'opération en attente est annulée par l'erreur; les registres ne sont pas modifiés.

---

## 10. Conversions

### 10.1 Règle générale

- **R-39** Quand la valeur montrée a une unité et n'est **pas** en cours de saisie (provenance `Resultat` ou `Sortie`), une touche d'unité — **avec ou sans `Conv`** — change le format d'affichage. Pendant une saisie, une touche d'unité seule étiquette le nombre ou fait défiler l'exposant (section 7); pour convertir, il faut `Conv` + unité, ce qui valide d'abord la saisie (`14` `Pieds` `Conv` `Verges`). `Conv` est toujours requis pour les unités secondaires (cm, mm, acre, pmp, masses).
- Sur un nombre **sans unité**, une touche d'unité (avec ou sans `Conv`) l'**étiquette** (`5` `×` `3` `=` `Pieds` → 15 pi 0 po).
- **R-39 bis** `Conv` + unité alors qu'une opération est en attente avec son opérande droit : l'opération est **d'abord calculée** (comme avec `=`), puis le résultat est converti. C'est ce qui permet `2` `×` `4` `×` `14` `Conv` `8`. (Les touches de fonction, elles, ne terminent pas l'opération : voir 9.8. La fonction Coût utilise l'opération en attente : voir 12.17.)
- La conversion d'une valeur libre donne une valeur libre (`Resultat`), sans étiquette. La conversion d'une **sortie de fonction** ne change que son format : l'étiquette, la provenance `Sortie` et le cycle en cours sont conservés. On peut ainsi afficher une diagonale en mètres (`Diag`, puis `m`), puis appuyer de nouveau sur `Diag` pour obtenir la coupe d'aplomb, sans que la valeur convertie soit prise pour une nouvelle donnée.

### 10.2 Longueurs

| Touche | Format obtenu |
|---|---|
| `Verges` | `Vg` |
| `Pieds` | depuis `PiPo` → `PiDec`; depuis `PiDec` → `PiPo`; depuis `PoDec` → `PiDec`; depuis tout autre format → `PiPo`. Pressions répétées : alternance `PiPo` ↔ `PiDec`. |
| `Pouces` | depuis `PoFrac` → `PoDec`; depuis `PoDec` → `PoFrac`; depuis `PiDec` → `PoDec`; depuis tout autre format → `PoFrac`. Pressions répétées : alternance. |
| `m` | `M` |
| `Conv` `7`, `Conv` `9` | `Cm`, `Mm` |

### 10.3 Volumes, masses, pmp et acres

| Valeur montrée | Touche | Résultat |
|---|---|---|
| aire | `Verges`, `Pieds`, `Pouces`, `m`, `Conv` `7`, `Conv` `9` | `Vg2`, `Pi2`, `Po2`, `M2`, `Cm2`, `Mm2` |
| aire | `Conv` `2` | `Acre` |
| volume | `Verges`, `Pieds`, `Pouces`, `m`, `Conv` `7`, `Conv` `9` | `Vg3`, `Pi3`, `Po3`, `M3`, `Cm3`, `Mm3` |
| volume | `Conv` `8` | `Pmp` (1 pmp = 144 po³) |
| volume | `Conv` `4`, `Conv` `1`, `Conv` `6`, `Conv` `3` | masse = volume × **masse volumique** mémorisée, en `Lb`, `Kg`, `Tc`, `T` |
| masse | `Conv` `4`, `Conv` `1`, `Conv` `6`, `Conv` `3` | `Lb`, `Kg`, `Tc`, `T` |
| masse | `Verges`, `Pieds`, `Pouces`, `m`, `Conv` `7`, `Conv` `9` | volume = masse ÷ masse volumique, au cube de l'unité |
| toute combinaison non listée | | `ERREUR – unités incompatibles` |

Exception documentée : sur une longueur en `PiPo` ou `PoFrac`, `Conv` `1`, `2`, `3`, `4`, `6`, `8` règlent la **résolution temporaire** (R-24) au lieu de produire une erreur.

---

## 11. Mémoires et registres

### 11.1 Mémoire cumulative M+

| Frappes | Effet | Écran |
|---|---|---|
| `M+` | ajoute la valeur montrée | `M+ │ valeur ajoutée` |
| `Conv` `M+` (M−) | la soustrait | `M− │ valeur soustraite` |
| `Rappel` `M+` | montre le total (sans effacer) | `TOTAL M │ total` |
| puis `M+`, `M+` | moyenne, puis nombre de valeurs (cycle total → moyenne → nombre) | `MOYENNE │ …`, `NOMBRE │ …` |
| `Rappel` `Rappel` | montre le total **et vide** la mémoire | `M+ │ total` |
| `Conv` `Rappel` (MC) | vide la mémoire | `MÉMOIRE EFFACÉE` |

- La première valeur ajoutée fixe le genre et le format; les suivantes doivent être du même genre, sinon erreur d'unités.
- Le nombre de valeurs compte les ajouts **et** les soustractions (100, 200, −50 → nombre 3, moyenne 83,33333).
- L'indicateur `M` est allumé tant que la mémoire n'est pas vide. La mémoire est conservée d'une séance à l'autre.
- **R-40** Juste après `Rappel` `M+`, la touche `M+` fait défiler (moyenne, nombre) au lieu d'ajouter.

### 11.2 `Stocker` + touche

| Touche | Registre | Donnée attendue |
|---|---|---|
| `1`, `2`, `3` | M1, M2, M3 (registres libres, permanents) | n'importe quelle valeur; `0` `Stocker` `1` vide M1 |
| `4` | bloc | une **aire** → aire d'un bloc; une **longueur** → longueur d'un bloc |
| `5` | entraxe | longueur > 0 |
| `6` | section de semelle | aire > 0 |
| `7` | hauteur de contremarche visée | longueur > 0 |
| `8` | épaisseur du plancher | longueur ≥ 0 |
| `9` | giron visé | longueur > 0 |
| `0` | masse volumique | nombre > 0, exprimé dans l'unité courante de masse volumique (tc/vg³ par défaut); l'unité elle-même se choisit dans les Préférences (13.4) |
| `Onglet composé` | angle de la moulure | nombre strictement entre 0 et 90 |

Après la mise en mémoire, l'écran montre l'étiquette correspondante, la valeur et `MÉM.` (ex. `ENTRAXE │ 18 po │ MÉM.`). Une donnée du mauvais genre : erreur d'unités. Toute autre touche après `Stocker` : erreur de saisie.

L'**échappée** et le **nombre de paquets par carré** se règlent seulement dans les Préférences ([13.4](#134-valeurs-mémorisées)).

### 11.3 `Rappel` + touche

| Touche | Valeur rappelée |
|---|---|
| `1`, `2`, `3` | M1, M2, M3 |
| `4` | aire d'un bloc; `Rappel` `4` de nouveau : longueur d'un bloc, et ainsi de suite en alternance |
| `5`, `6`, `7`, `8`, `9`, `0` | entraxe, section de semelle, contremarche visée, épaisseur du plancher, giron visé, masse volumique |
| `Onglet composé` | angle de la moulure |
| `Pente`, `Montée`, `Course`, `Diag` | dernière valeur saisie **ou calculée** du triangle |
| `Conv` `Arêtier Noue` | pente irrégulière |
| `Longueur`, `Largeur`, `Hauteur` | dimensions saisies |
| `Cercle`, `Arc` | diamètre saisi, arc saisi |
| `/` | résolution des fractions en vigueur (`RÉSOLUTION │ 1/16 po`, suivie de `Standard` ou `Constant` dans l'indicateur) |
| `M+`, `Rappel` | voir 11.1 |
| `=` | ouvre le ruban (comme `Conv` `=`) |

Une valeur rappelée est une valeur libre : elle peut servir d'opérande (`55` `Pieds` `×` `Rappel` `Course` `×` `2` `Pieds` `=`). Registre vide : `AUCUNE VALEUR`.

---

## 12. Fonctions de construction

Pour chaque fonction : les **données**, le **calcul**, puis les **sorties** dans l'ordre du cycle (R-27). Sauf indication contraire :

- quand la touche reçoit une donnée, l'écran montre d'abord **la donnée avec son étiquette**; la pression suivante affiche la première sortie du cycle, et le cycle reboucle;
- les longueurs calculées prennent le format de la première longueur fournie en donnée (`PiPo` pour une donnée en pieds-pouces); les aires et volumes suivent R-31 et R-32;
- les angles sont en `Degres`;
- `OC` désigne l'**entraxe mémorisé** (16 po par défaut);
- une « valeur **libre** à l'écran » est une saisie en cours ou une valeur de provenance `Resultat` (9.1) — par opposition à une sortie de fonction;
- une donnée négative ou nulle là où une grandeur positive est attendue : erreur de saisie.

### 12.1 Triangle rectangle : `Pente`, `Montée`, `Course`, `Diag`

**Registres.** `pente`, `montee`, `course`, `diag`. Chaque valeur **saisie** porte son numéro d'ordre (`compteur`). La pente saisie est permanente; après `Effacer` ×2 elle est conservée mais considérée comme la plus ancienne.

**Saisie de la pente** — la pente est mémorisée comme un ratio `t = montée ÷ course` :

| Donnée | Interprétation | Format d'origine |
|---|---|---|
| longueur impériale (`9` `Pouces` `Pente`) | t = pouces ÷ 12 | pouces |
| longueur métrique (`250` `Conv` `9` `Pente`) | t = mètres de montée par mètre de course (0,25) | pourcentage |
| nombre sans unité (`30` `Pente`), `Dms` compris | angle en degrés, 0 < a < 90; t = tan a | degrés |
| pourcentage (`75` `%` `Pente`) | t = 0,75 | pourcentage |
| nombre avec `Conv` `Pente` (`0,65` `Conv` `Pente`) | t = 0,65 | ratio |

**Résolution (R-41).** Parmi les quatre registres, on retient **les deux valeurs saisies les plus récentes**; elles déterminent les deux autres :

| Paire connue | Calcul |
|---|---|
| pente, montée | C = M ÷ t; D = √(M² + C²) |
| pente, course | M = C × t; D = √(M² + C²) |
| pente, diagonale | C = D ÷ √(1 + t²); M = C × t |
| montée, course | t = M ÷ C; D = √(M² + C²) |
| montée, diagonale | exige D > M; C = √(D² − M²); t = M ÷ C |
| course, diagonale | exige D > C; M = √(D² − C²); t = M ÷ C |

Moins de deux valeurs connues : `AUCUNE VALEUR`. Longueurs de genres différents (nombre nu et longueur) : erreur d'unités. Condition géométrique violée : calcul impossible. Une valeur **calculée** n'entre pas dans les registres de saisie (elle reste disponible par `Rappel`).

**Touches**

- `Pente` — avec donnée : mémorise et affiche dans le format d'origine. Sans donnée : affiche la pente (saisie si elle fait partie de la paire retenue, sinon calculée). Cycle sur quatre formats, dans l'ordre circulaire **pouces → degrés → pourcentage → ratio**, en partant du format d'origine (ou des pouces pour une pente calculée) :

  | Format | Écran |
  |---|---|
  | pouces | `PENTE │ 7 po` (valeur = t × 12, en `PoFrac`; l'interface ajoute la mention « sur 12 ») |
  | degrés | `PENTE │ 30,26°` |
  | pourcentage | `PENTE │ 58,33333 %` |
  | ratio | `RATIO DE PENTE │ 0,583333` |

- `Montée`, `Course` — avec donnée : mémorisent et affichent. Sans donnée : calculent (ou réaffichent la valeur saisie, avec `MÉM.`). Une seule sortie. Voir aussi le cas du cercle ([12.8](#128-cercle-arc-rayon-et-murs-cintrés)).
- `Diag` — avec donnée : mémorise et affiche. Sorties du cycle : `DIAGONALE`, `COUPE D'APLOMB` = atan t, `COUPE DE NIVEAU` = 90° − atan t.

La diagonale est la longueur « pointe à pointe » du chevron commun : elle n'inclut ni le débord ni la réduction pour la faîtière (à dire dans l'aide).

### 12.2 Arêtier et noue : `Arêtier Noue`

**Données.** Le triangle résolu (t₁, montée H, course R₁) et, s'il y a lieu, la **pente irrégulière** t₂.

- `Conv` `Arêtier Noue` avec donnée : mémorise t₂ (mêmes interprétations que la pente : pouces, degrés, pourcentage) et affiche `PENTE IRRÉGULIÈRE │ 8 po`. Sans donnée : la réaffiche, ou `AUCUNE VALEUR`. La pente irrégulière est temporaire (effacée par `Effacer` ×2).

**Toit régulier** (pas de pente irrégulière) :

- course en plan de l'arêtier = R₁ × √2; longueur = √(H² + 2 R₁²);
- aplomb = atan(H ÷ (R₁√2)); niveau = 90° − aplomb; latérale = 45°.

Sorties : `ARÊTIER/NOUE`, `COUPE D'APLOMB`, `COUPE DE NIVEAU`, `COUPE LATÉRALE`.

**Toit irrégulier** (deux pentes) : la montée H est commune aux deux versants.

- R₂ = H ÷ t₂ (course du versant irrégulier);
- course en plan = √(R₁² + R₂²); longueur = √(R₁² + R₂² + H²);
- aplomb = atan(H ÷ √(R₁² + R₂²)); niveau = 90° − aplomb;
- **latérale 1** = atan(R₂ ÷ R₁) — biseau des empannons du versant **irrégulier**;
- **latérale 2** = 90° − latérale 1 = atan(R₁ ÷ R₂) — biseau des empannons du versant **régulier**.

Sorties : `ARÊTIER/NOUE IRRÉG.`, `COUPE D'APLOMB`, `COUPE DE NIVEAU`, `COUPE LATÉRALE 1`, `COUPE LATÉRALE 2`.

Les longueurs sont théoriques (axe à axe), sans déduction pour l'épaisseur des pièces.

### 12.3 Empannons : `Empannon`

**Données.** Le triangle résolu, OC, la pente irrégulière éventuelle, et deux préférences : *Empannons* (décroissant / croissant) et *Empannons irréguliers* (entraxe constant / appariés).

**Calcul.** a₁ = atan t₁; chevron commun L₁ = R₁ ÷ cos a₁.

*Toit régulier* : e₁ = OC; incrément₁ = OC ÷ cos a₁; latérale = 45°.

*Toit irrégulier* : a₂ = atan t₂; R₂ = H ÷ t₂; L₂ = R₂ ÷ cos a₂.

- Entraxes des deux versants (e₁ régulier, e₂ irrégulier) :
  - **entraxe constant** : e₁ = e₂ = OC;
  - **appariés** (les empannons des deux versants se rencontrent sur l'arêtier) : l'entraxe mémorisé est le **maximum** permis et va au versant le plus pentu. Si R₁ ≥ R₂ : e₂ = OC et e₁ = OC × R₂ ÷ R₁; sinon e₁ = OC et e₂ = OC × R₁ ÷ R₂.
- incrément₁ = e₁ × (R₁ ÷ R₂) ÷ cos a₁; incrément₂ = e₂ × (R₂ ÷ R₁) ÷ cos a₂;
- latérale du versant régulier = atan(R₁ ÷ R₂); latérale du versant irrégulier = atan(R₂ ÷ R₁).

*Liste des longueurs* pour un versant de chevron commun L et d'incrément i :

- **décroissant** (tracé à partir du dernier chevron commun) : L − k·i pour k = 1, 2, 3… tant que le résultat est > 10⁻⁶ po;
- **croissant** (tracé à partir du coin) : k·i pour k = 1, 2, 3… tant que k·i ≤ L + 10⁻⁶ po.

Au plus 200 empannons par versant (au-delà : erreur de dépassement).

**Sorties d'un versant**, dans l'ordre : `ENTRAXE` (avec `MÉM.` s'il s'agit de l'entraxe mémorisé), `INCRÉMENT`, `EMPANNON 1` … `EMPANNON n` (ou `EMPANNON IRRÉG. n`), `COUPE D'APLOMB` (a du versant), `COUPE DE NIVEAU`, `COUPE LATÉRALE` (du versant).

- `Empannon` : versant régulier, puis versant irrégulier s'il existe.
- `Conv` `Empannon` : versant irrégulier d'abord, puis régulier. Sans pente irrégulière : `AUCUNE VALEUR`.

### 12.4 Mur en pente : `Conv` `Montée`

**Données.** Le triangle résolu (pente t, course C), OC, la **base** (0 par défaut), la préférence *Mur en pente* (décroissant / croissant).

- Si une **longueur** est en saisie au moment d'appuyer, elle devient la base : l'écran montre `BASE │ 5 pi 0 po`, et la pression suivante commence le cycle. La base est temporaire.

**Calcul.** Hauteur d'un montant à la distance d du point bas : d × t + base.

- **décroissant** : d = C − k·OC pour k = 1, 2… tant que d > 10⁻⁶ po;
- **croissant** : d = k·OC pour k = 1, 2… tant que d ≤ C + 10⁻⁶ po.

**Sorties** : `ENTRAXE` (`MÉM.`), `MONTANT 1` … `MONTANT n`, `BASE`, `ANGLE DU MUR` = atan t.

La base ainsi saisie sert aussi aux murs cintrés (12.8).

### 12.5 Toiture : `Conv` `Diag`

**Données.**

- La pente t : si les deux valeurs saisies les plus récentes du triangle sont deux longueurs (montée, course ou diagonale), t en est dérivé selon R-41; sinon c'est la pente saisie. Aucune des deux : `AUCUNE VALEUR`.
- L'aire en plan : une **aire** en donnée (`100` `Pieds` `Pieds` `Conv` `Diag`), sinon `longueur × largeur` des registres. Sinon `AUCUNE VALEUR`.
- Le nombre de **paquets par carré** mémorisé (3 par défaut).

**Calcul.** aire du toit = aire en plan ÷ cos(atan t). En pieds carrés : carrés = aire ÷ 100; paquets = carrés × paquets par carré; feuilles 4×8 = aire ÷ 32.

**Sorties** (affichées dès la première pression) :

| # | Étiquette | Valeur | Rendu |
|---|---|---|---|
| 1 | `AIRE DU TOIT` | aire | aire, même unité que l'aire en plan |
| 2 | `CARRÉS` | aire ÷ 100 pi² | 2 décimales |
| 3 | `PAQUETS` | carrés × paquets par carré | 2 décimales |
| 4 | `PAQUET` | 100 ÷ paquets par carré, en pi² (couverture d'un paquet) | 2 décimales, `MÉM.` |
| 5 | `FEUILLES 4×8` | aire ÷ 32 pi² | 2 décimales |
| 6 | `PENTE` | t × 12 en pouces | `MÉM.` si saisie |
| 7 | `AIRE EN PLAN` | | aire |

### 12.6 Escalier : `Escalier` et `Conv` `Escalier`

**Données.** La **montée** et/ou la **course** saisies (registres du triangle; la pente n'intervient pas), et quatre valeurs mémorisées : contremarche visée `cv` (7 1/2 po), giron visé `gv` (10 po), échappée `e` (6 pi 8 po), épaisseur du plancher `p` (10 po). Ni montée ni course : `AUCUNE VALEUR`.

**Arrondi de chantier (R-42).** Les dimensions unitaires sont arrondies à ce qu'on peut réellement tracer, et **tous les résultats suivants partent des valeurs arrondies** :

- si la donnée principale (la montée, sinon la course) est en impérial : arrondi à la résolution des fractions en vigueur;
- si elle est en métrique : arrondi au millimètre.

On note `arr(x)` cet arrondi (au plus proche, la demie vers le haut).

**Calcul**

| Cas | Nombre de contremarches n | Contremarche cm | Marches | Giron g | Complément |
|---|---|---|---|---|---|
| montée M seule | arrondi(M ÷ cv) | arr(M ÷ n) | n − 1 | arr(gv) | course C = marches × g |
| course C seule | marches + 1 | arr(cv) | max(1, arrondi(C ÷ gv)) | arr(C ÷ marches) | montée M = n × cm |
| montée et course | arrondi(M ÷ cv) | arr(M ÷ n) | n − 1 | arr(C ÷ marches) | — |

- « arrondi » = entier le plus proche, la demie vers le haut, au moins 1.
- **Contremarche maximale** (`Conv` `Escalier`) : n = plafond(M ÷ cv − EPS), de sorte que la contremarche ne dépasse jamais `cv`. Sans effet si la montée n'est pas saisie.
- Montée et course saisies avec n = 1 (aucune marche) : erreur de saisie.

**Sorties**

| # | Étiquette | Valeur |
|---|---|---|
| 1 | `CONTREMARCHE` | cm — avertissement si cm > cv (`MÉM.` dans le cas « course seule ») |
| 2 | `NB CONTREMARCHES` | n (entier) |
| 3 | `ÉCART CONTREMARCHES` | n × cm − M (positif : surplus; négatif : déficit) |
| 4 | `GIRON` | g (`MÉM.` dans le cas « montée seule ») |
| 5 | `NB MARCHES` | n − 1 (entier) |
| 6 | `ÉCART GIRONS` | marches × g − C |
| 7 | `TRÉMIE` | (e + p) ÷ cm × g |
| 8 | `LIMON` | marches × √(cm² + g²) |
| 9 | `INCLINAISON` | atan(cm ÷ g) — avertissement « escalier raide » si cm ÷ g > 1,10 × cv ÷ gv |
| 10 | `COURSE` | C (`MÉM.` si saisie) |
| 11 | `MONTÉE` | M (`MÉM.` si saisie) |
| 12 | `CONTREMARCHE VISÉE` | cv, `MÉM.` |
| 13 | `GIRON VISÉ` | gv, `MÉM.` |
| 14 | `ÉCHAPPÉE` | e, `MÉM.` |
| 15 | `ÉPAISSEUR PLANCHER` | p, `MÉM.` |

Formats — impérial : contremarche, giron et écarts en `PoFrac`; trémie, limon, course, montée en `PiPo`. Métrique : contremarche, giron et écarts en `Mm`; les autres en `M`. Les valeurs 12 à 15 gardent leur format de mémorisation.

Le giron calculé n'inclut pas le nez de marche (à dire dans l'aide).

### 12.7 Onglet composé : `Onglet composé`

**Données.** L'**angle de la moulure** S mémorisé (45° par défaut; `38` `Conv` `Onglet composé` ou `38` `Stocker` `Onglet composé` pour le changer; `Conv` `Onglet composé` sans donnée le réaffiche) et l'**angle du coin** C tapé avant la touche :

- donnée ≥ 25 et ≤ 180 : c'est l'angle du coin, en degrés;
- donnée < 25 : c'est un **nombre de côtés** n (entier ≥ 3) d'un polygone régulier; C = (n − 2) × 180 ÷ n;
- sans donnée : reprend le dernier coin saisi, sinon `AUCUNE VALEUR`.

**Calcul.**

- onglet = atan( sin S ÷ tan(C ÷ 2) );
- biseau = asin( cos S × cos(C ÷ 2) ).

**Sorties** (affichées dès la première pression) : avec un angle — `ONGLET`, `BISEAU`, `ANGLE MOULURE` (`MÉM.`), `COIN`; avec un nombre de côtés — `COIN`, `ONGLET`, `BISEAU`, `ANGLE MOULURE` (`MÉM.`).

### 12.8 Cercle, arc, rayon et murs cintrés

**Registres.** `diametre` (saisi par `Cercle`, ou deux fois le rayon saisi par `Conv` `Arc`); `arc` saisi (un **angle** si la donnée est sans unité, 0 < a < 360; une **longueur d'arc** si c'est une longueur); la **corde** = course saisie; la **flèche** = montée saisie.

**Résolution d'un arc.** Rayon R, angle au centre θ (radians), longueur d'arc s, corde c, flèche h. On applique le premier cas dont les données sont présentes :

| Ordre | Données | Calcul |
|---|---|---|
| 1 | R et angle | θ donné |
| 2 | R et longueur d'arc | θ = s ÷ R |
| 3 | R et corde | exige c ≤ 2R; θ = 2 asin(c ÷ 2R) |
| 4 | R et flèche | exige 0 < h ≤ 2R; θ = 2 acos(1 − h ÷ R) |
| 5 | corde et flèche | R = (c²/4 + h²) ÷ (2h); θ = 4 atan(2h ÷ c) |
| 6 | angle et corde | R = c ÷ (2 sin(θ/2)) |
| 7 | angle et flèche | R = h ÷ (1 − cos(θ/2)) |

Longueur d'arc avec corde, ou longueur d'arc avec flèche, sans rayon : pas de solution directe → `AUCUNE VALEUR`. Ensuite : s = Rθ; c = 2R sin(θ/2); h = R(1 − cos(θ/2)); aire du **segment** = R²(θ − sin θ) ÷ 2; aire du **secteur** = R²θ ÷ 2.

**Touches**

- `Cercle` — avec donnée (longueur ou nombre > 0) : mémorise le diamètre, affiche `DIAMÈTRE`; cycle : `AIRE` = πR², `CIRCONFÉRENCE` = 2πR, `DIAMÈTRE`. Sans donnée : mêmes sorties à partir du rayon connu ou résolu; sinon `AUCUNE VALEUR`.
- `Conv` `Arc` (Rayon) — avec donnée : mémorise le rayon, affiche `RAYON`. Sans donnée : calcule et affiche `RAYON`.
- `Arc` — avec donnée : mémorise l'arc, affiche `ARC │ donnée`. Cycle (avec ou sans donnée) :
  1. `ARC` — l'angle puis la longueur d'arc (si l'une des deux vient d'être affichée comme donnée, elle est sautée au premier tour);
  2. `CORDE`;
  3. `SEGMENT`;
  4. `SECTEUR`;
  5. `FLÈCHE`;
  6. `ENTRAXE` (`MÉM.`);
  7. `MUR CINTRÉ 1` … `MUR CINTRÉ n`.

  Si l'arc ne peut pas être résolu, la touche réaffiche seulement la donnée.
- `Montée` et `Course` **en présence d'un diamètre** (R-43) : si le diamètre est connu, que la corde est saisie et pas la flèche, `Montée` (sans donnée) calcule `FLÈCHE` = R − √(R² − c²/4); si la flèche est saisie et pas la corde, `Course` (sans donnée) calcule `CORDE` = 2√(2Rh − h²). Cette règle a priorité sur R-41. `Effacer` ×2 efface le diamètre et rétablit le comportement du triangle.

**Murs cintrés.** Longueurs des montants à l'entraxe OC, mesurées de part et d'autre du centre de l'arc; x = k·OC pour k = 1, 2… tant que x < c/2 − 10⁻⁶ po. Le montant central (x = 0) n'est pas listé : il vaut la base à l'extérieur, et la flèche plus la base à l'intérieur. Calculés seulement si θ ≤ 180°. Selon la préférence *Mur cintré* :

- **Extérieur** (défaut) — montant **au-dessus** de l'arc, jusqu'à l'horizontale passant par le sommet (cadrage d'une fenêtre cintrée) : R − √(R² − x²) + base;
- **Intérieur** — montant **sous** l'arc, à partir de la corde (mur à sablière cintrée) : √(R² − x²) − (R − h) + base.

La base est celle saisie par `Conv` `Montée` (12.4); 0 par défaut.

### 12.9 Colonne et cône : `Conv` `Cercle`

**Données.** Le diamètre (rayon r) et la **hauteur** h (registre `Hauteur`; en mode trigonométrique : la montée saisie). Donnée manquante : `AUCUNE VALEUR`.

**Sorties** (dès la première pression) :

| # | Étiquette | Valeur |
|---|---|---|
| 1 | `COLONNE – VOLUME` | π r² h |
| 2 | `COLONNE – SURFACE` | surface totale : 2π r (r + h) |
| 3 | `CÔNE – VOLUME` | π r² h ÷ 3 |
| 4 | `CÔNE – SURFACE` | surface totale : π r (r + √(r² + h²)) |

### 12.10 Polygone : `Conv` `Course`

**Données.** Le nombre de côtés n (entier ≥ 3, tapé avant la touche ou repris du dernier calcul) et le **rayon** R du cercle circonscrit (du diamètre saisi ou de `Conv` `Arc`). n invalide : erreur de saisie. Rayon absent : `AUCUNE VALEUR`.

**Sorties** (dès la première pression) : `ANGLE INTÉRIEUR` = (n − 2) × 180 ÷ n · `DEMI-ANGLE` = la moitié · `CÔTÉ` = 2R sin(180° ÷ n) · `PÉRIMÈTRE` = n × côté · `AIRE` = n R² sin(360° ÷ n) ÷ 2 · `RAYON` · `NB CÔTÉS`.

### 12.11 `Longueur`, `Largeur`, `Hauteur`

| Touche | Avec donnée | Cycle |
|---|---|---|
| `Longueur` | mémorise, affiche `LONGUEUR` | — (sans donnée : réaffiche avec `MÉM.`) |
| `Largeur` | mémorise, affiche `LARGEUR` | `AIRE` = L × l · `ÉQUERRAGE` = √(L² + l²) · `PÉRIMÈTRE` = 2(L + l) · `LONGUEUR` · `LARGEUR` |
| `Hauteur` | mémorise, affiche `HAUTEUR` | `VOLUME` = L × l × H · `MURS` = 2(L + l) × H · `PIÈCE` = murs + L × l (les quatre murs et le plafond) · `LONGUEUR` · `LARGEUR` · `HAUTEUR` |

S'il manque la longueur (pour `Largeur`) ou la longueur et la largeur (pour `Hauteur`), la touche réaffiche seulement sa propre valeur. En mode trigonométrique, ces trois touches n'existent pas (12.18).

### 12.12 Blocs et briques : `Conv` `Longueur`

**Donnée**, par ordre de priorité :

1. une **longueur** libre à l'écran → blocs = longueur ÷ longueur d'un bloc;
2. une **aire** libre à l'écran → blocs = aire ÷ aire d'un bloc;
3. les registres longueur **et** hauteur → blocs = (L × H) ÷ aire d'un bloc;
4. le registre longueur seul → blocs = L ÷ longueur d'un bloc;
5. sinon `AUCUNE VALEUR`.

**Sorties** : `BLOCS` (nombre décimal, non arrondi — l'utilisateur arrondit à l'unité supérieure et ajoute ses pertes avec `+` `5` `%`), puis la taille mémorisée utilisée : `LONGUEUR D'UN BLOC` ou `AIRE D'UN BLOC` (`MÉM.`). Le cycle se poursuit avec la touche `Longueur`.

Défauts : bloc de 8 × 16 po, soit 128 po² et 16 po. Pour des briques, mémoriser l'aire d'une brique avec son joint (`21` `Pouces` `Pouces` `Stocker` `4`).

### 12.13 Semelle : `Conv` `Largeur`

**Donnée** : une longueur libre à l'écran, sinon le registre longueur. **Calcul** : volume = longueur × **section de semelle** mémorisée (264 po² par défaut). Format : `Vg3` pour une longueur impériale, `M3` pour une longueur métrique (sauf préférence de volume imposée).

**Sorties** : `SEMELLE`, `SECTION DE SEMELLE` (`MÉM.`). Le cycle se poursuit avec `Largeur`.

### 12.14 Gypse : `Conv` `Hauteur`

**Donnée**, par ordre de priorité : une **aire** libre; une **longueur** libre (calcul linéaire); les registres longueur et hauteur (aire = L × H); le registre longueur seul (linéaire); sinon `AUCUNE VALEUR`.

**Calcul.** Pour une aire, en pieds carrés : feuilles = aire ÷ 32 (4×8), ÷ 36 (4×9), ÷ 40 (4×10), ÷ 48 (4×12). Pour une longueur : longueur ÷ 4 pi, quel que soit le format de feuille (feuilles posées debout).

**Sorties** : `FEUILLES 4×8`, `FEUILLES 4×9`, `FEUILLES 4×10`, `FEUILLES 4×12`, puis la donnée (`AIRE` ou `LONGUEUR`). Nombres décimaux non arrondis. Le cycle se poursuit avec `Hauteur`.

### 12.15 Montants : `Conv` `5`

**Donnée** : une longueur libre à l'écran, sinon le registre longueur. **Calcul** : montants = plafond(longueur ÷ OC − EPS) + 1 (le montant de départ s'ajoute aux intervalles). **Sortie unique** : `MONTANTS` (entier). L'entraxe se consulte par `Rappel` `5`.

Les fonctions placées sur des touches de chiffres (`Montants`, `Coût`, unités) n'ont **jamais** de cycle par répétition du chiffre : le chiffre suivant commence une saisie (seule exception : l'exposant de cm et mm, R-19).

### 12.16 Bois en pmp : `Conv` `8`

- Nombre sans unité : pmp = N ÷ 12 (R-20). Usage : `2` `×` `4` `×` `14` `Conv` `8` → 9,333333 pmp.
- Volume : conversion (1 pmp = 144 po³). `150` `Pieds` `Pieds` `Pieds` `Conv` `8` → 1 800 pmp.

### 12.17 Coût : `Conv` `0`

**Condition** : une multiplication en attente `quantité × prix`, le prix étant un nombre sans unité. Sinon erreur de saisie.

**Calcul** : coût = (valeur numérique de la quantité **dans son unité d'affichage**) × prix.

| Format de la quantité | Le prix s'entend… |
|---|---|
| `PiPo`, `PiDec` | par pied |
| `PoFrac`, `PoDec` | par pouce |
| toute autre unité (`Vg3`, `M2`, `Kg`…) | par unité affichée |
| `Pmp` | **par millier de pmp** (prix au Mpmp) : coût = pmp ÷ 1000 × prix |
| sans unité | par unité |

**Sortie unique** : `COÛT │ 283,06 $` (format `Monnaie`). Exemple : `5,146605 vg³` à l'écran, `×` `55` `Conv` `0` → `283,06 $`.

### 12.18 Mode trigonométrique

Quand la préférence est active, `Longueur`, `Largeur`, `Hauteur` deviennent `sin`, `cos`, `tan`; avec `Conv` : `sin⁻¹`, `cos⁻¹`, `tan⁻¹`. Les fonctions Blocs, Semelle, Gypse et les registres longueur/largeur/hauteur ne sont alors plus accessibles au clavier; Colonne/Cône lit sa hauteur dans `Montée`.

- `sin`, `cos`, `tan` : s'appliquent à la valeur sans unité montrée (degrés décimaux ou `Dms`), y compris une sortie de fonction en degrés (ex. après `Pente`). Résultat : `Nombre`.
- Fonctions inverses : résultat en `Degres`.
- Valeur avec unité, ou argument hors de [−1, 1] pour `sin⁻¹` et `cos⁻¹` : `ERREUR – trigonométrie`. Tangente de 90° : `ERREUR – calcul impossible`.

---

## 13. Préférences et valeurs mémorisées

Un seul écran défilant, intitulé **Préférences**, ouvert par la roue dentée ou par `Conv` `Stocker`. Les changements s'appliquent **immédiatement** et sont sauvegardés; un bouton **Terminé** ramène à la calculatrice. La valeur par défaut est marquée ★.

### 13.1 Général

| Préférence | Options | Effet |
|---|---|---|
| **Langue** | Français / English — au premier lancement, celle de l'appareil ★ | section 24; aussi par le bouton `FR \| EN` de la barre supérieure |
| **Mode trigonométrique** | désactivé ★ / activé | voir 12.18 |
| **Fractions rapides** | désactivé ★ / activé | affiche la bande de fractions (7.6) |
| **Mode plein écran** | désactivé ★ / activé | fenêtre en plein écran sans bordure (aussi `F11`) — version de bureau seulement (3.1) |
| **Thème** | Automatique ★ / Clair / Sombre | 16.3 |

### 13.2 Affichage

| Préférence | Options | Effet |
|---|---|---|
| **Résolution des fractions** | 1/2, 1/4, 1/8, 1/16 ★, 1/32, 1/64 | 8.3 |
| **Mode des fractions** | Standard ★ (fraction réduite) / Constant (dénominateur fixe) | 8.3 |
| **Notation exponentielle** | désactivée ★ / activée | 8.7 |
| **Affichage des aires** | Standard ★ / Pieds carrés / Verges carrées / Mètres carrés | R-31, R-32 |
| **Affichage des volumes** | Standard ★ / Verges cubes / Pieds cubes / Mètres cubes | R-31, R-32 |
| **Mètres (longueurs)** | 0,000 ★ / Flottant | 8.4 |
| **Degrés décimaux** | 0,00 ★ / Flottant | 8.6 |
| **Séparateur de milliers** | activé ★ (espace) / désactivé | R-04 |
| **Séparateur décimal** | Virgule ★ / Point | pour qui préfère le point; n'affecte que l'affichage et la touche décimale. En anglais, c'est toujours le point : la préférence n'est pas montrée, et son réglage français est conservé (24.3) |
| **Décimales (valeurs sans unité)** | Flottant ★ / 0 / 1 / 2 / 3 / 4 | 8.6 |

### 13.3 Fonctions

| Préférence | Options | Effet |
|---|---|---|
| **Mur en pente** | Décroissant ★ (du plus long au plus court) / Croissant | 12.4 |
| **Mur cintré** | Extérieur ★ (au-dessus de l'arc) / Intérieur (sous l'arc) | 12.8 |
| **Empannons** | Décroissant ★ / Croissant | 12.3 |
| **Empannons irréguliers** | Entraxe constant ★ / Appariés | 12.3 |

### 13.4 Valeurs mémorisées

Chaque ligne montre la valeur courante; un clic ouvre un petit dialogue de modification (champ de texte analysé par 7.9, boutons `Annuler` et `Enregistrer`, lien `Valeur par défaut`). Une saisie invalide affiche `Valeur non reconnue` sous le champ et n'est pas enregistrée.

| Valeur | Défaut | Aussi par | Sert à |
|---|---|---|---|
| **Échappée** | 6 pi 8 po | — | trémie d'escalier |
| **Hauteur de contremarche visée** | 7 1/2 po | `Stocker` `7` | escalier |
| **Épaisseur du plancher** | 10 po | `Stocker` `8` | trémie d'escalier |
| **Giron visé** | 10 po | `Stocker` `9` | escalier |
| **Aire d'un bloc** | 128 po² | `Stocker` `4` | blocs |
| **Longueur d'un bloc** | 16 po | `Stocker` `4` | blocs |
| **Angle de la moulure** | 45,00° | `Stocker` `Onglet composé` | onglet composé |
| **Entraxe** | 16 po | `Stocker` `5` | empannons, murs, montants |
| **Section de semelle** | 264 po² | `Stocker` `6` | semelle |
| **Masse volumique** | 1,5 tc/vg³ | `Stocker` `0` | volume ↔ masse; le dialogue offre le nombre et l'unité (tc/vg³, lb/vg³, lb/pi³, t/m³, kg/m³) |
| **Paquets de bardeaux par carré** | 3 | — | toiture (un paquet couvre 100 ÷ 3 = 33,33 pi²) |

### 13.5 Réinitialisation

Bouton **Réinitialiser les données et les préférences**, avec confirmation (texte en 4.5). Effet : 9.9.

---

## 14. Ruban de calcul

Le ruban garde la trace des calculs, pour relecture, vérification et partage.

### 14.1 Contenu

```rust
pub struct LigneRuban {
    pub genre: GenreLigne,          // Operande | SousTotal | Total | Sortie | Conversion | Memoire | Separateur
    pub operateur: Option<char>,    // + − × ÷ % devant l'opérande
    pub etiquette: Option<String>,  // pour les sorties de fonction
    pub texte: String,              // valeur formatée (en_texte)
}
```

| Événement | Ligne ajoutée |
|---|---|
| appui sur un opérateur | l'opérande qui vient d'être validé, précédé de l'opérateur qui le liait au précédent (rien pour le premier) |
| `=` | le dernier opérande, puis une ligne **Total** (`=` et le résultat) |
| `%` | l'opérande suivi de `%`, puis le total |
| donnée affectée à une fonction, ou sortie de fonction affichée pour la première fois | **Sortie** : étiquette et valeur |
| conversion | **Conversion** : `→` et la valeur convertie |
| `M+`, `M−`, rappel du total | **Mémoire** |
| `Effacer` ×2 | **Séparateur** |

Le ruban courant garde les **500** dernières lignes. Il est vidé par `Effacer tout`, pas par `Effacer` ×2.

### 14.2 Consultation

- `Conv` `=`, `Rappel` `=` ou `Ctrl+R` ouvre le ruban courant. Dans une fenêtre large (≥ 880 px), il s'affiche en permanence dans un panneau à droite de la calculatrice.
- Présentation : une ligne par entrée, valeurs alignées à droite, totaux en gras, séparateurs en filet. Défilement automatique vers la dernière ligne.
- Actions : **Enregistrer** (demande un nom; défaut `Ruban du 19 novembre 2026, 8 h 04`), **Copier** (texte dans le presse-papiers), **Exporter** (fichier `.txt` en UTF-8, boîte de dialogue native), **Vider** (avec confirmation).

### 14.3 Rubans enregistrés

Onglet **Rubans** : liste des rubans enregistrés, du plus récent au plus ancien (nom, date, nombre de lignes). Pour chacun : **Ouvrir** (lecture seule), **Renommer**, **Copier**, **Exporter**, **Supprimer** (avec confirmation). Au plus 200 rubans; au-delà, proposer de supprimer les plus anciens.

Format du texte copié ou exporté :

```
Ruban du 19 novembre 2026, 8 h 04
--------------------------------
        5 pi 4 1/2 po
+              8 po
+             3,5 vg
=     16 pi 6 1/2 po
```

---

## 15. Écrans, menu et aide

### 15.1 Navigation

| Élément | Contenu |
|---|---|
| **Barre supérieure** | ☰ (menu) · bouton plein écran (version Web : seulement si le navigateur le permet) · titre de l'application · bouton de langue `FR \| EN` (24.2) · ⚙ (Préférences) |
| **Barre inférieure** | quatre onglets : **Accueil** (la calculatrice), **Métiers** (section 22), **Rubans**, **Aide** |
| **Menu ☰** (tiroir à gauche) | Calculateurs par métier · Rubans enregistrés · Aide · Manuel d'utilisation (15.4) · Bienvenue · Trucs de pro · Préférences · À propos · Installer l'application (version Web seulement, 3.1) · Quitter (version de bureau seulement). **Au pied du tiroir**, de haut en bas : la mention « Gracieuseté de Sylvain Leduc de Constructo AI »; le courriel `info@constructoai.ca` (lien `mailto:`) et le site `constructoai.ca` (lien vers `https://constructoai.ca/`, ouvert dans un nouvel onglet), demandés par l'utilisateur le 2026-11-19; la version et la révision : `Version 0.5.0 · révision 1a2b3c4`. La révision est l'identifiant court du commit git, inscrit à la compilation par `app/build.rs`; sans dépôt git, seule la version s'affiche. Si le tiroir est trop bas pour tout montrer, les entrées du menu passent d'abord. |

`Échap` ferme le tiroir, un dialogue ou un écran secondaire et ramène à l'Accueil.

### 15.2 Écrans

| Écran | Contenu |
|---|---|
| **Accueil** | afficheur, bande de fractions rapides (si active), barre des métiers (23.2), clavier — celui de base ou celui d'un outil de métier (23.3) |
| **Rubans** | ruban courant et rubans enregistrés (section 14) |
| **Aide** | champ de recherche + liste des fiches; une fiche par fonction (annexe B) |
| **Préférences** | section 13 |
| **Bienvenue** | présentation en quelques lignes (annexe B.3); affichée automatiquement au premier lancement |
| **Trucs de pro** | liste d'astuces (annexe B.4) |
| **À propos** | nom, version, mention « Gracieuseté de Sylvain Leduc de Constructo AI » (demandée par l'utilisateur le 2026-11-19; elle figure aussi au pied du menu ☰ et sur l'écran d'attente de la version Web), le courriel et le site de Constructo AI (liens), avertissement (1.5), emplacement du dossier de données, licence du logiciel (Apache 2.0) avec un lien vers le code source, mention des bibliothèques libres utilisées et de leurs licences |

### 15.3 Aide contextuelle

- **Appui long** (≥ 0,6 s) ou **clic droit** sur une touche : ouvre la fiche d'aide de cette touche dans une fenêtre superposée, avec deux onglets si la touche a une fonction secondaire. L'appui long ne déclenche pas la touche.
- **Appui long** ou **clic droit** sur une touche d'un outil de métier : le mode d'emploi de l'outil (23.7).
- **Infobulle** au survol : nom complet de la fonction principale, de la fonction secondaire et raccourci clavier (ex. « Montée — avec Conv : Mur en pente — raccourci : T »).
- La recherche de l'écran Aide ignore la casse et les accents (« arete » trouve « Arêtier ») et cherche dans les titres, les mots-clés et le texte.

Chaque fiche comprend : un titre, « À quoi ça sert », « Comment faire » (étapes), un exemple chiffré tiré de la section 19, et au besoin une note.

### 15.4 Manuel d'utilisation

Ajouté le 2026-11-19, à la demande de l'utilisateur, qui voulait « un manuel d'utilisation complet ». C'est une page Web autonome, `app/manuel.html` : sans script ni ressource externe, lisible sur un téléphone, en thème clair ou sombre, et imprimable. Le manuel existe aussi **en anglais**, `app/manual.html` (24.7) : le menu ouvre celui de la langue en cours, et chacun porte un lien vers l'autre.

**Contenu.** Quinze chapitres : présentation et avertissement; installer et ouvrir; l'écran; taper une mesure et calculer; les mémoires; le clavier de charpente; le clavier de votre métier; le ruban; les préférences; les raccourcis clavier; les trucs de pro; les messages et le dépannage; le répertoire des touches et des fonctions; le répertoire des outils par métier; l'avertissement, la licence et les coordonnées.

**R-61** Le manuel ne s'écrit pas à la main : il est **produit** à partir d'un gabarit (`app/outils/manuel-gabarit.html`, qui porte la mise en page et les chapitres rédigés; `manuel-gabarit-en.html` pour l'anglais) et du contenu même du programme. Le générateur (`app/src/manuel.rs`, compilé pour les tests seulement) y insère :

- les trucs de pro (annexe B.4);
- toutes les fiches d'aide (annexe B), en trois groupes : touches principales, secondes fonctions, autres fonctions;
- pour chacun des métiers et des calculateurs de la section 22 : le nom, le résumé, le tableau des champs (libellé, libellé de la touche s'il diffère, unité, valeur de départ, options des listes), la note, et **un exemple calculé par le noyau** à partir d'une valeur d'essai de 22.14.

Trois tests gardent le manuel exact : chaque calculateur a un exemple qui donne des résultats; la page contient chaque métier, chaque calculateur et chaque fiche, et toutes ses ancres mènent quelque part; et **la page publiée est identique à ce que le générateur produit**. Ce dernier test échoue dès qu'une fiche, un calculateur ou le gabarit change : il faut alors refaire la page par `cargo test -p calculatrice-chantier regenerer_le_manuel -- --ignored`.

**Accès.** Entrée « Manuel d'utilisation » du menu ☰ :

| Version | Ce qui s'ouvre |
|---|---|
| Web | la page `manuel.html` du site (`manual.html` en anglais), dans un nouvel onglet. Les deux font partie des fichiers gardés hors ligne (`app/sw.js`) : installée, la calculatrice montre son manuel sans réseau. Le service hors ligne ne répond par la page de l'application qu'aux navigations vers cette page; une autre page du site — le manuel — est demandée au réseau d'abord, pour lire la version du jour, puis prise dans sa propre copie locale (sans quoi ouvrir le manuel hors ligne rouvrirait la calculatrice). |
| Bureau | les deux manuels sont embarqués dans l'exécutable. Celui de la langue en cours est copié dans le dossier de données (`manuel.html` ou `manual.html`), puis ouvert dans le navigateur par une adresse `file:` : il se lit sans réseau. Si la copie échoue, c'est la page publiée qui est ouverte. |

Tout ce qui diffère est dans `app/src/plateforme/` (`adresse_du_manuel`).

---

## 16. Interface utilisateur

### 16.1 Fenêtre

- Titre : le nom de l'application. Taille initiale **420 × 860** points logiques (format portrait, comme un appareil tenu en main); minimum 340 × 620; redimensionnable.
- Tout le contenu se met à l'échelle : `echelle = min(largeur ÷ 420, hauteur ÷ 860)` appliquée aux tailles de police, marges et rayons.
- **Fenêtre large** (largeur ≥ 880 points) : la calculatrice garde son format portrait à gauche (largeur = hauteur × 420 ÷ 860, plafonnée à 520) et le **ruban courant** occupe le reste, à droite.
- Icône de l'application : le **logo de Constructo AI** (celui du site de l'utilisateur, à sa demande du 2026-11-21) sur fond blanc aux coins arrondis. Le fichier source est `app/icones/logo-constructo-ai.png`; `app/outils/icones.py` en tire les icônes PNG de l'application Web installable (3.1) et `icone-96.rvba`, les pixels bruts lus par le programme pour l'icône de la fenêtre et la marque de la barre supérieure. (Jusqu'à la version 0.6.0 : une équerre orange dessinée par le code.)

### 16.2 Zones de l'écran principal

De haut en bas, en proportion de la hauteur utile :

| Zone | Hauteur | Détail |
|---|---|---|
| Barre supérieure | 48 pt | voir 15.1 |
| **Afficheur** | ≈ 20 % | voir ci-dessous |
| Bande de fractions rapides | 2 rangées de 34 pt (si active) | 7.6 |
| Barre des métiers | 34 pt | le métier choisi et ses outils, voir 23.2 |
| **Clavier** | le reste | grille 8 × 5, voir 16.4; avec un outil de métier : 7 ou 8 rangées, voir 23.3 |
| Barre inférieure | 56 pt | onglets |

**Afficheur** — un panneau arrondi, à fond clair et encre foncée, composé de trois lignes :

1. **Indicateurs** (petits, à gauche) : `CONV`, `STOCKER` ou `RAPPEL` quand un préfixe est armé; `M` quand la mémoire M+ n'est pas vide; l'opérateur en attente (`+ − × ÷`); `MÉM.` pour une valeur mémorisée; `⚠` en cas d'avertissement (le texte complet apparaît en infobulle et sur la ligne d'étiquette).
2. **Étiquette** (taille moyenne, à gauche) : texte de 4.4, ou le message d'erreur.
3. **Valeur** (grande, alignée à **droite**) : chiffres en gras; unités dans une taille réduite (≈ 55 %); fractions **empilées** (numérateur, barre, dénominateur) à ≈ 50 %. Si la valeur est trop large, la taille de police diminue jusqu'à ce qu'elle tienne.

Dans le coin inférieur gauche de l'afficheur : la touche **⌫**. Un double-clic sur la valeur la copie dans le presse-papiers (confirmation discrète « Copié »).

### 16.3 Identité visuelle (originale)

Thème « Constructo » : les couleurs de l'ERP Constructo AI, à la demande de l'utilisateur (2026-11-19). Il remplace la palette « Ardoise et ambre » de la version 1.0 de ce document. Les couleurs sont des constantes nommées dans `theme.rs`; aucun code de couleur ailleurs. Une flèche (→) indique un dégradé, du haut vers le bas.

| Jeton | Clair | Sombre |
|---|---|---|
| `fond_app` (dégradé en diagonale) | `#FFFFFF` → `#CFE2FC` | `#0F1923` → `#162340` |
| quadrillage du fond (pas de 24 points, ligne majeure aux 120 points) | bleu primaire, très transparent | blanc, très transparent |
| `barre_fond` (barre supérieure) / `barre_texte` | `#002B6B` → `#002050` / `#FFFFFF` | les mêmes |
| `pied_fond` (barre inférieure et tiroir du menu) | `#002050` → `#001833` | les mêmes |
| `barre_accent` (onglet actif, logo) | `#F97316` | `#F97316` |
| `ecran_fond` / `ecran_bord` | `#FFFFFF` → `#EAF2FF` / `#B4D0F0` | `#1E293B` → `#0F172A` / `#334155` |
| `ecran_encre` / `ecran_secondaire` / `ecran_etiquette` | `#002050` / `#605E5C` / `#005EA2` | `#F8FAFC` / `#A8B3C2` / `#6CB8F6` |
| `touche_fonction` (fond / texte) | `#0078D4` → `#005EA2` / `#FFFFFF` | les mêmes |
| `touche_construction` | `#002B6B` → `#002050` / `#FFFFFF` | `#003380` → `#002359` / `#FFFFFF` |
| `touche_chiffre` | `#FFFFFF` → `#EEF3F9` / `#002050` (bord `#C3CFDD`) | `#334155` → `#1E293B` / `#F8FAFC` (bord `#475569`) |
| `touche_conv` | `#FB923C` → `#F97316` / `#001833` | les mêmes |
| `touche_effacer` | `#DC2626` → `#B91C1C` / `#FFFFFF` | les mêmes |
| `etiquette_secondaire` | `#9A3412` | `#FDBA74` |
| `avertissement` / `erreur` | `#92400E` / `#B91C1C` | `#FCD34D` / `#F87171` |
| `primaire` / `titre` / `entete` | `#0078D4` / `#005EA2` / `#002050` | `#2B88D8` / `#6CB8F6` / `#FFFFFF` |
| `carte` / `carte_bord` | `#FFFFFF` / `#DCE4EE` | `#1F2937` / `#374151` |

Familles de touches :

| Famille | Touches |
|---|---|
| `touche_fonction` | rangée 1 (Pente … Arêtier Noue); Verges, Pieds, Pouces, `/`; Stocker, Rappel |
| `touche_construction` | rangée 2 (Onglet composé … Empannon); m, Longueur, Largeur, Hauteur, %; ÷, ×, −, +, =; M+ |
| `touche_chiffre` | 0 à 9 et la virgule |
| `touche_conv` | Conv |
| `touche_effacer` | Effacer |

Le thème **Automatique** suit le thème du système (clair ou sombre).

Formes : touches en rectangle arrondi (rayon 7 × échelle), face en dégradé vertical cernée d'un liseré d'un point, reflet clair sous l'arête supérieure, liseré plus foncé de 2 points au bas pour suggérer le relief, et ombre portée douce (teintée de la couleur de la touche pour les touches de couleur). Touche enfoncée : décalée de 1 point vers le bas et assombrie de 12 %. Survol : éclaircie de 6 %. L'afficheur est une carte en dégradé, avec un halo et un filet d'accent de la couleur primaire; ses indicateurs sont des pastilles. Le fond de l'application est un dégradé en diagonale recouvert d'un quadrillage façon plan. Pas d'images : dégradés, ombres et pictogrammes sont dessinés par le code.

Typographie : libellés de touches en **gras**; chiffres plus grands que les libellés; étiquettes secondaires en petit, en italique si la police le permet. Sous Windows, charger au démarrage Segoe UI Gras (`C:\Windows\Fonts\segoeuib.ttf`) comme famille « gras », avec repli sur la police par défaut d'egui si le fichier est absent. Vérifier à l'écran que tous ces caractères s'affichent : `é è ê à ç É È À Ç ² ³ √ π ÷ × − ° ′ ″ ◄ ► ⌫ ⚠ ☰ ⚙ ★`. Remplacer par un dessin vectoriel (lignes, cercles) tout pictogramme absent de la police. Dans la version Web, les polices sont embarquées (3.1).

### 16.4 Touches

Chaque case de la grille comprend une **zone d'étiquette secondaire** (≈ 24 % de la hauteur de la case, texte centré) et, en dessous, le **bouton**. Espacement de 6 × échelle entre les cases.

- Taille de police **uniforme par famille** : calculée pour que le plus long libellé de la famille tienne dans le bouton avec une marge de 6 points (ex. « Longueur », « Empannon », « Escalier »). Les libellés sur deux lignes (« Arêtier / Noue », « Onglet / composé ») utilisent la même taille.
- **`Conv` armé** : la touche `Conv` reste visuellement enfoncée et toutes les étiquettes secondaires passent en gras, pour montrer ce que la prochaine touche fera.
- **Clic sur une étiquette secondaire** : équivaut à `Conv` puis la touche (raccourci pour la souris).
- Cible minimale : 36 × 36 points à l'échelle 1.

### 16.5 Raccourcis clavier

Les raccourcis agissent quand l'écran Accueil est affiché et qu'aucun champ de texte n'a le focus.

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
| `E` | Escalier | | `Ctrl+V` | coller un nombre ou une longueur (analyse 7.9) |
| `B` | Arc | | `L` | Longueur |
| `I` | Cercle | | `G` | Largeur |
| `J` | Empannon | | `H` | Hauteur |

Lettres insensibles à la casse. Les raccourcis sont regroupés dans une seule table (`raccourcis.rs`) et rappelés dans les infobulles.

Dans la version Web, `F11` et `Ctrl+R` restent ceux du navigateur (plein écran, recharger la page); `F1` ouvre l'aide de la calculatrice et non celle du navigateur.

### 16.6 Comportement

- **Réactivité** : aucune animation continue; l'interface ne se redessine que sur un événement (egui est réactif par défaut). Pour l'appui long, demander un nouveau rendu après 0,6 s (`request_repaint_after`).
- **Démarrage** : la fenêtre apparaît en moins d'une seconde sur un poste courant; l'état précédent est restauré (section 17).
- **Fermeture** : l'état est sauvegardé.
- **Accessibilité** : contraste texte/fond d'au moins 4,5:1 pour les libellés; chaque touche porte un nom accessible en français (« Montée », « Conversion »…).
- **Mise à l'échelle Windows** (125 %, 150 %) : rendu net, rien de tronqué.

---

## 17. Persistance

### 17.1 Emplacement

Dossier de données de l'utilisateur : celui que retourne `eframe::storage_dir(APP_ID)` avec `APP_ID = "CalculatriceChantier"` (sous Windows, dans `%APPDATA%`). Si cette fonction est indisponible, utiliser `%APPDATA%\CalculatriceChantier\`. Le chemin réel est affiché dans « À propos ».

**Version Web** : les mêmes données sont conservées dans le stockage local du navigateur, sous les clés `CalculatriceChantier/etat.json` et `CalculatriceChantier/rubans.json`, en JSON compact (le stockage local est limité à environ 5 Mo par site). « À propos » indique que les données sont dans le navigateur, sur l'appareil, et qu'effacer les données de navigation du site les efface. Si le navigateur refuse le stockage, la calculatrice fonctionne quand même et le signale une fois.

### 17.2 Fichiers

| Fichier | Contenu |
|---|---|
| `etat.json` | version du format, préférences (13.1 à 13.3, dont la **langue**, section 24), valeurs mémorisées (13.4), pente mémorisée, mémoire M+ (somme, nombre, genre, format), registres M1–M3, ruban courant, indicateur « bienvenue déjà vue » |
| `rubans.json` | version du format, liste des rubans enregistrés (identifiant, nom, date ISO 8601, lignes) |
| `metiers.json` | version du format, dernières saisies de chaque calculateur par métier (section 22), identifiant de l'outil branché sur le clavier s'il y en a un (`outil`, section 23), et **langue dans laquelle les saisies sont écrites** (`langue`, 24.4). Simple commodité : une donnée illisible est ignorée. |

```rust
#[derive(Serialize, Deserialize)]
pub struct EtatPersistant {
    pub version: u32,                 // 1
    pub preferences: Preferences,
    pub memorises: ValeursMemorisees,
    pub pente: Option<PenteMemorisee>,
    pub memoire: MemoireCumulative,
    pub registres: [Option<Valeur>; 3],
    pub ruban: Ruban,
    pub bienvenue_vue: bool,
}
```

### 17.3 Règles

- **R-44** Écriture **atomique** : écrire dans `etat.json.tmp`, puis renommer. (Version Web : l'écriture dans le stockage local est déjà indivisible.)
- **R-45** Sauvegarder quand le moteur émet `Effet::EtatModifie` (au plus une écriture par seconde) et à la fermeture. (Version Web : sans délai à chaque changement; pas de sauvegarde à la fermeture.)
- **R-46** Fichier absent : valeurs par défaut. Fichier illisible ou d'une version inconnue : le renommer en `etat.corrompu.json`, repartir des valeurs par défaut et afficher une fois « Les réglages n'ont pas pu être lus; les valeurs par défaut ont été rétablies. »
- **R-47** Tous les champs sérialisés ont une valeur par défaut (`#[serde(default)]`), pour qu'un fichier d'une version antérieure reste lisible.

---

## 18. Architecture du code

### 18.1 Arborescence

```
Calculatrice/                      (dossier de travail)
├── CAHIER_DES_CHARGES.md          (ce document)
├── 01.png … 06.png                (photos de référence — hors dépôt, hors binaire)
├── Cargo.toml                     (espace de travail)
├── Trunk.toml                     (construction de la version Web, 3.1)
├── .cargo/config.toml             (exécutable Windows autonome)
├── .github/                       (publication automatique : site Web et exécutable)
├── .gitignore
├── README.md                      (les deux versions, compilation, raccourcis — en français)
├── LICENSE                        (licence Apache 2.0, texte officiel)
├── NOTICE                         (avis de droit d'auteur)
├── noyau/
│   ├── Cargo.toml                 (nom : calculatrice-noyau)
│   ├── src/
│   │   ├── lib.rs
│   │   ├── unites.rs              facteurs, Genre, Format, famille, système
│   │   ├── valeur.rs              Valeur + arithmétique dimensionnelle (9.5)
│   │   ├── format.rs              formatage (section 8), TexteValeur
│   │   ├── langue.rs              Langue, langue en cours, Texte : un texte dans les deux langues (24)
│   │   ├── analyse.rs             analyser_longueur, analyser_aire (7.9)
│   │   ├── saisie.rs              machine de saisie (section 7)
│   │   ├── touches.rs             enum Touche
│   │   ├── etiquettes.rs          enum Etiquette + textes (4.4)
│   │   ├── erreurs.rs             enum Erreur + textes (4.5)
│   │   ├── prefs.rs               Preferences, ValeursMemorisees, défauts
│   │   ├── moteur.rs              Moteur : états, préfixes, opérateurs, cycles (9)
│   │   ├── moteur/outils.rs       outils de métier au clavier (23)
│   │   ├── conversions.rs         section 10
│   │   ├── memoire.rs             M+, registres (11)
│   │   ├── metiers/               calculateurs par métier (22) : mod.rs, puis un fichier par métier
│   │   ├── ruban.rs               Ruban, LigneRuban, export texte (14)
│   │   ├── persistance.rs         EtatPersistant (17.2)
│   │   └── fonctions/
│   │       ├── mod.rs             Sortie, Cycle, Rendu
│   │       ├── triangle.rs        12.1
│   │       ├── aretier.rs         12.2
│   │       ├── empannons.rs       12.3
│   │       ├── mur_pente.rs       12.4
│   │       ├── toiture.rs         12.5
│   │       ├── escalier.rs        12.6
│   │       ├── onglet.rs          12.7
│   │       ├── arc.rs             12.8
│   │       ├── colonne_cone.rs    12.9
│   │       ├── polygone.rs        12.10
│   │       ├── dimensions.rs      12.11
│   │       ├── materiaux.rs       12.12 à 12.17
│   │       └── trigo.rs           12.18
│   └── tests/
│       ├── commun/mod.rs          outil de test : jouer une séquence (19.2)
│       ├── saisie_format.rs
│       ├── arithmetique.rs
│       ├── conversions.rs
│       ├── memoires.rs
│       ├── triangle_toit.rs
│       ├── escaliers.rs
│       ├── cercle_arc.rs
│       ├── materiaux.rs
│       ├── erreurs.rs
│       ├── metiers.rs             valeurs d'essai de la section 22
│       ├── metiers_suite.rs       valeurs d'essai des calculateurs ajoutés à la version 0.4 (22.14)
│       ├── outils.rs              outils de métier au clavier (23.9)
│       └── langue.rs              les deux langues : textes, nombres, saisies, changement de langue (24.8)
└── app/
    ├── Cargo.toml                 (nom : calculatrice-chantier — nom de l'exécutable)
    ├── build.rs                   inscrit la révision du code source (identifiant du commit) dans le programme
    ├── manuel.html                manuel d'utilisation, produit par src/manuel.rs (15.4)
    ├── manual.html                le même, en anglais (24.7)
    ├── index.html                 page hôte de la version Web (3.1)
    ├── manifest.webmanifest       manifeste de l'application Web installable (3.1)
    ├── sw.js                      service qui garde le site utilisable hors ligne (3.1)
    ├── icones/                    icônes PNG de l'application installable
    ├── outils/icones.py           script qui dessine ces icônes
    ├── outils/manuel-gabarit.html gabarit du manuel d'utilisation (15.4)
    ├── outils/manuel-gabarit-en.html  gabarit du manuel anglais (24.7)
    ├── polices/                   polices libres de la version Web, licences, LISEZMOI.md
    └── src/
        ├── main.rs                points d'entrée : fenêtre native ou page Web
        ├── application.rs         struct Application : impl eframe::App
        ├── textes.rs              NOM_APPLICATION, libellés, messages (français et anglais)
        ├── theme.rs               palettes, polices, échelle
        ├── clavier.rs             grille des touches, clavier d'un outil de métier (23.3), dessin, clics, appui long
        ├── afficheur.rs           dessin de l'afficheur (fractions empilées)
        ├── raccourcis.rs          table des raccourcis clavier
        ├── stockage.rs            lecture/écriture des données JSON (17)
        ├── plateforme.rs          ce qui diffère entre le bureau et le navigateur (3.1)
        ├── plateforme/
        │   ├── bureau.rs          dossier de données, « Enregistrer sous », plein écran de la fenêtre
        │   └── web.rs             stockage local, téléchargement, plein écran du navigateur, démarrage
        ├── aide.rs                contenu des fiches (annexe B), dans les deux langues, + recherche
        ├── manuel.rs              générateur du manuel d'utilisation et ses tests (15.4)
        └── vues/
            ├── preferences.rs
            ├── rubans.rs
            ├── aide.rs
            ├── bienvenue.rs
            ├── metiers.rs         écran des calculateurs par métier (22)
            ├── trucs.rs
            └── a_propos.rs
```

### 18.2 Règles de conception

- **R-48** `noyau` ne connaît ni egui ni le système de fichiers : il ne fait que du calcul et de la mise en forme. Il est entièrement testable par `cargo test -p calculatrice-noyau`, sans écran.
- **R-49** `app` ne contient **aucune formule**. Elle transmet des `Touche` au moteur, dessine l'`Ecran` qu'il retourne et exécute les `Effet`.
- **R-50** `#![forbid(unsafe_code)]` dans les deux crates. Dans `noyau` : pas de `unwrap()`, `expect()` ni `panic!` hors des tests; les erreurs passent par `Result<_, Erreur>`.
- **R-51** Identifiants en **français sans accents** (`Moteur`, `appuyer`, `hauteur_contremarche`), commentaires en français. Les chaînes visibles par l'utilisateur ont leurs accents.
- **R-52** Aucune chaîne visible par l'utilisateur en dehors de `noyau/src/etiquettes.rs`, `noyau/src/erreurs.rs`, `app/src/textes.rs`, `app/src/aide.rs` et, pour les libellés des calculateurs par métier, les fichiers de `noyau/src/metiers/`. Chaque texte y est écrit dans les deux langues, côte à côte (R-62). Le nom et le code de chaque langue (« Français », « English », `FR`, `EN`) sont dans `noyau/src/langue.rs`. Seule exception : la page hôte de la version Web (`app/index.html`) porte les textes affichés avant que le programme soit chargé (titre de la page, écran d'attente, message d'échec).
- **R-53** Chaque fonction de la section 12 est une fonction pure, sans accès au `Moteur` :

```rust
// Exemple de signature (escalier)
pub struct DonneesEscalier {
    pub montee: Option<Valeur>,
    pub course: Option<Valeur>,
    pub contremarche_visee: Valeur,
    pub giron_vise: Valeur,
    pub echappee: Valeur,
    pub epaisseur_plancher: Valeur,
    pub contremarche_max: bool,
    pub resolution: u32,
}
pub fn calculer_escalier(d: &DonneesEscalier) -> Result<Vec<Sortie>, Erreur>;
```

- **R-54** Compilation sans avertissement : `cargo clippy --workspace -- -D warnings`, `cargo clippy --target wasm32-unknown-unknown -p calculatrice-chantier -- -D warnings` et `cargo fmt --all --check` passent.

### 18.3 Boucle de l'application (schéma)

```
événement (clic, clavier)
   └─> Touche ──> Moteur::appuyer ──> Vec<Effet>
                      │                  ├─ OuvrirPreferences     → changer de vue
                      │                  ├─ BasculerRuban         → changer de vue / panneau
                      │                  ├─ ConfirmerEffacerTout  → dialogue → Moteur::effacer_tout
                      │                  └─ EtatModifie           → sauvegarde différée
                      └─> Moteur::ecran() ──> dessin de l'afficheur
```

---

## 19. Tests et vecteurs de validation

### 19.1 Stratégie

| Niveau | Quoi | Où |
|---|---|---|
| Unitaires | formatage, analyse de texte, arithmétique dimensionnelle, chaque fonction pure de la section 12 | `#[cfg(test)]` dans chaque module de `noyau` |
| Séquences | le moteur piloté touche par touche, écran comparé au résultat attendu | `noyau/tests/*.rs` |
| Robustesse | séquences aléatoires de touches : le moteur ne panique jamais et `ecran()` reste formatable | `noyau/tests/robustesse.rs` (générateur pseudo-aléatoire maison, sans dépendance; au moins 200 000 frappes, graine fixe) |
| Interface | vérification manuelle par l'utilisateur à l'aide de la liste 21.3 | — |

**Tous les vecteurs de 19.3 à 19.14 doivent devenir des tests automatisés.** Ils ont été calculés indépendamment avant la rédaction; ils reproduisent notamment les résultats classiques des manuels de calculatrices de construction.

### 19.2 Outil de test : jouer une séquence

Dans `noyau/tests/commun/mod.rs` :

```rust
/// Joue une séquence de jetons séparés par des espaces et retourne l'écran final.
pub fn jouer(moteur: &mut Moteur, sequence: &str) -> Ecran;
/// Texte comparable : « ÉTIQUETTE │ valeur » ou « valeur » s'il n'y a pas d'étiquette,
/// ou le message. Les espaces insécables sont remplacées par des espaces ordinaires.
pub fn texte(ecran: &Ecran) -> String;
pub fn moteur_neuf() -> Moteur;   // état par défaut, préférences par défaut
```

| Jeton | Touche |
|---|---|
| un nombre (`7`, `125`, `17,32`, `23,42,39`) | décomposé caractère par caractère en `Chiffre` et `Virgule` |
| `Pi` `Po` `Vg` `m` | `Pieds`, `Pouces`, `Verges`, `Metres` |
| `/` | `Fraction` |
| `+` `−` `×` `÷` `=` | opérateurs (variantes acceptées : `-`, `x`, `*`, `:`) |
| `%` | `Pourcent` |
| `Conv` `Sto` `Rap` `M+` `Eff` `⌫` | `Conv`, `Stocker`, `Rappel`, `MPlus`, `Effacer`, `RetourArriere` |
| `Pente` `Montée` `Course` `Diag` `Arêtier` `Onglet` `Escalier` `Arc` `Cercle` `Empannon` `Longueur` `Largeur` `Hauteur` | touches de fonction (jetons insensibles à la casse et aux accents) |
| `[3/8]` | `FractionRapide { num: 3, den: 8 }` |

Conventions des tableaux : chaque ligne part d'un **moteur neuf**, sauf les lignes commençant par « ↳ » qui **poursuivent** la ligne précédente. L'écran attendu s'écrit `ÉTIQUETTE │ valeur`; sans étiquette, la valeur seule. `MÉM.` et `⚠` sont notés quand ils doivent être présents.

### 19.3 Saisie et conversions

| Frappes | Écran attendu |
|---|---|
| `5 Pi 1 Po 1 / 2 =` | `5 pi 1 1/2 po` |
| `5 Pi 6 =` | `5 pi 6 po` |
| `5 Pi 1 / 2 =` | `5 pi 0 1/2 po` |
| `15 / =` | `15/16 po` |
| `3 / 2 =` | `1 1/2 po` |
| `5 Vg =` | `5 vg` |
| `14 Pi Conv Vg` | `4,666667 vg` |
| ↳ `Conv Pi` | `14 pi 0 po` |
| ↳ `Conv Po` | `168 po` |
| ↳ `Conv m` | `4,267 m` |
| ↳ `Conv 7` | `426,72 cm` |
| ↳ `Conv 9` | `4 267,2 mm` |
| `15 Pi 9 Po 1 / 2 Conv Pi` | `15,79167 pi` |
| ↳ `Pi` | `15 pi 9 1/2 po` |
| `17,32 Pi Conv Pi` | `17 pi 3 13/16 po` |
| ↳ `Pi` | `17,32 pi` |
| `8 Po 1 / 8 Conv Po` | `8,125 po` |
| ↳ `Pi` | `0,677083 pi` |
| ↳ `Po` | `8,125 po` |
| `9,0625 Po Conv Po` | `9 1/16 po` |
| ↳ `Pi` | `0 pi 9 1/16 po` |
| ↳ `Pi` | `0,755208 pi` |
| `14 Pi Pi Conv Po` | `2 016 po²` |
| ↳ `Conv Vg` | `1,555556 vg²` |
| ↳ `Conv m` | `1,300643 m²` |
| ↳ `Conv 7` | `13 006,43 cm²` |
| `14 Pi Pi Pi Conv Po` | `24 192 po³` |
| ↳ `Conv Vg` | `0,518519 vg³` |
| ↳ `Conv m` | `0,396436 m³` |
| `10 m Conv Pi` | `32 pi 9 11/16 po` |
| `2,4 m Conv Pi` | `7 pi 10 1/2 po` |
| `8 Pi Conv m` | `2,438 m` |
| `3 / 4 Conv 9` | `19,05 mm` |
| `5 Conv 7 7 =` | `5 cm²` |
| `2 Conv 2 Conv Pi` | `87 120 pi²` |
| ↳ `Conv m` | `8 093,713 m²` |
| `10000 Pi Pi Conv 2` | `0,229568 acre` |
| `5 × 3 = Pi` | `15 pi 0 po` |
| `5 Pi 6 [3/8] =` | `5 pi 6 3/8 po` |
| `12 3 ⌫ ⌫ 7 Po =` | `17 po` |

### 19.4 Arithmétique, pourcentages, fonctions mathématiques

| Frappes | Écran attendu |
|---|---|
| `2 + 3 × 4 =` | `20` |
| `11 Po + 2 Pi 1 Po =` | `3 pi 0 po` |
| `5 Pi 7 Po 1 / 2 + 18 Pi 8 Po =` | `24 pi 3 1/2 po` |
| `11 Pi 7 Po 1 / 2 − 3 Pi =` | `8 pi 7 1/2 po` |
| `81 Po − 32 Po =` | `49 po` |
| `5 Pi 4 Po 1 / 2 + 8 Po + 3,5 Vg =` | `16 pi 6 1/2 po` |
| `3,5 Vg + 2 Pi =` | `4,166667 vg` |
| `5 Pi 3 Po × 11 Pi 6 Po 1 / 2 =` | `60,59375 pi²` |
| `2 Pi 7 Po × 10 =` | `25 pi 10 po` |
| `30 Pi 4 Po ÷ 7 Po =` | `52` |
| `20 Pi 3 Po ÷ 9 =` | `2 pi 3 po` |
| `10 Pi ÷ 2 Pi 2 Po =` | `4,615385` |
| `2 Pi 5 Po × 3 =` | `7 pi 3 po` |
| ↳ `Pi` | `7,25 pi` |
| `13 Pi 8 Po ÷ 2 =` | `6 pi 10 po` |
| `12 Pi 6 Po × 15 Pi 8 Po =` | `195,8333 pi²` |
| `8 Po × 16 Po =` | `128 po²` |
| `10 Pi × 6 Po =` | `5 pi²` |
| `2 m × 50 Conv 7 =` | `1 m²` |
| `10 Pi × 2 m =` | `65,6168 pi²` |
| `3 Pi × 1 Pi 9 Po 5 / 8 × 2 Pi 4 Po =` | `12,61458 pi³` |
| ↳ `Conv Vg` | `0,467207 vg³` |
| `36 Pi 3 Po × 11 Pi 6 Po =` | `416,875 pi²` |
| ↳ `× 4 Po =` | `5,146605 vg³` |
| ↳ `× 55 Conv 0` | `COÛT │ 283,06 $` |
| `12 Pi 6 Po × 3,50 Conv 0` | `COÛT │ 43,75 $` |
| `500 Pi × 18 %` | `90 pi 0 po` |
| `137 Pi Pi + 10 %` | `150,7 pi²` |
| `552 Pi 6 Po − 20 %` | `442 pi 0 po` |
| `350 Vg Vg Vg ÷ 80 %` | `437,5 vg³` |
| `75 %` | `75 %` |
| `75 % × 200 =` | `150` |
| `7 Pi 4 Po Conv %` | `53,77778 pi²` |
| `100 Conv Eff` | `10` |
| `25 Pi Pi Conv Eff` | `5 pi 0 po` |
| `8 Conv ÷` | `0,125` |
| `Conv +` | `3,141593` |
| `5 Conv − =` | `−5` |
| `156 Po ÷ 5 Po 1 / 2 =` | `28,36364` |
| `114 Po ÷ 29 =` | `3 15/16 po` |
| `44 / 64 + 1 / 64 =` | `45/64 po` |
| ↳ `Conv 1` | `11/16 po` |
| ↳ `Conv 2` | `1/2 po` |
| ↳ `Conv 3` | `23/32 po` |
| ↳ `Conv 4` | `3/4 po` |
| ↳ `Conv 6` | `45/64 po` |
| ↳ `Conv 8` | `3/4 po` |
| `5 Pi + 3 Pi Conv m` | `2,438 m` |

### 19.5 Mémoires

| Frappes | Écran attendu |
|---|---|
| `100 M+` | `M+ │ 100` |
| ↳ `200 M+` | `M+ │ 200` |
| ↳ `50 Conv M+` | `M− │ 50` |
| ↳ `Rap M+` | `TOTAL M │ 250` |
| ↳ `M+` | `MOYENNE │ 83,33333` |
| ↳ `M+` | `NOMBRE │ 3` |
| ↳ `Rap Rap` | `M+ │ 250` (la mémoire est ensuite vide; indicateur `M` éteint) |
| `175 Sto 1 Eff Eff Rap 1` | `M1 │ 175` |
| `1575 Vg Vg Sto 2 Eff Eff Rap 2` | `M2 │ 1 575 vg²` |
| `18 Po Sto 5` | `ENTRAXE │ 18 po` `MÉM.` |
| `Rap 5` | `ENTRAXE │ 16 po` `MÉM.` |
| `Rap 7` | `CONTREMARCHE VISÉE │ 7 1/2 po` `MÉM.` |
| `Rap 4` | `AIRE D'UN BLOC │ 128 po²` `MÉM.` |
| ↳ `Rap 4` | `LONGUEUR D'UN BLOC │ 16 po` `MÉM.` |
| `Rap 0` | `MASSE VOLUMIQUE │ 1,5 tc/vg³` `MÉM.` |
| `Rap /` | `RÉSOLUTION │ 1/16 po` |
| `Rap 1` | `AUCUNE VALEUR` |

### 19.6 Longueur, largeur, hauteur

| Frappes | Écran attendu |
|---|---|
| `20 Pi 6 Po Longueur` | `LONGUEUR │ 20 pi 6 po` |
| ↳ `25 Pi 6 Po Largeur` | `LARGEUR │ 25 pi 6 po` |
| ↳ `Largeur` | `AIRE │ 522,75 pi²` |
| ↳ `Largeur` | `ÉQUERRAGE │ 32 pi 8 5/8 po` |
| ↳ `Largeur` | `PÉRIMÈTRE │ 92 pi 0 po` |
| ↳ `Largeur` | `LONGUEUR │ 20 pi 6 po` |
| ↳ `Largeur` | `LARGEUR │ 25 pi 6 po` |
| `15 Pi Longueur 20 Pi Largeur 12 Pi Hauteur` | `HAUTEUR │ 12 pi 0 po` |
| ↳ `Hauteur` | `VOLUME │ 3 600 pi³` |
| ↳ `Hauteur` | `MURS │ 840 pi²` |
| ↳ `Hauteur` | `PIÈCE │ 1 140 pi²` |
| `23 Pi 8 Po Longueur 45 Pi 6 Po Largeur Largeur Largeur` | `ÉQUERRAGE │ 51 pi 3 7/16 po` |

### 19.7 Masses, blocs, bois, semelle, gypse, montants

| Frappes | Écran attendu |
|---|---|
| `2500 Conv 4` | `2 500 lb` |
| ↳ `Conv 1` | `1 133,981 kg` |
| ↳ `Conv 6` | `1,25 tc` |
| ↳ `Conv 3` | `1,133981 t` |
| `5 Vg Vg Vg Conv 4` | `15 000 lb` |
| ↳ `Conv 6` | `7,5 tc` |
| ↳ `Conv 1` | `6 803,886 kg` |
| `3 Conv 6 Conv Vg` | `2 vg³` |
| `22 Pi + 15 Pi 8 Po = Longueur` | `LONGUEUR │ 37 pi 8 po` |
| ↳ `4 Pi Hauteur Conv Longueur` | `BLOCS │ 169,5` |
| ↳ `+ 5 %` | `177,975` |
| `300 Pi Pi Conv Longueur` | `BLOCS │ 337,5` |
| ↳ `+ 3 %` | `347,625` |
| `30 Pi Longueur 45 Pi Largeur Largeur Largeur Largeur` | `PÉRIMÈTRE │ 150 pi 0 po` |
| ↳ `= Conv Longueur` | `BLOCS │ 112,5` |
| ↳ `+ 3 %` | `115,875` |
| `20 Pi Conv Longueur` | `BLOCS │ 15` |
| ↳ `Longueur` | `LONGUEUR D'UN BLOC │ 16 po` `MÉM.` |
| `21 Po Po Sto 4` | `AIRE D'UN BLOC │ 21 po²` `MÉM.` |
| ↳ `40 Pi Longueur 8 Pi Hauteur Conv Longueur` | `BLOCS │ 2 194,286` |
| ↳ `+ 3 %` | `2 260,114` |
| `2 × 4 × 14 Conv 8` | `9,333333 pmp` |
| ↳ `M+ 2 × 10 × 16 Conv 8` | `26,66667 pmp` |
| ↳ `M+ 2 × 12 × 18 Conv 8` | `36 pmp` |
| ↳ `M+ Rap Rap` | `M+ │ 72 pmp` |
| ↳ `× 250 Conv 0` | `COÛT │ 18,00 $` |
| `150 Pi Pi Pi Conv 8` | `1 800 pmp` |
| `16 Po × 8 Po = Sto 6` | `SECTION DE SEMELLE │ 128 po²` `MÉM.` |
| ↳ `232 Pi 6 Po Conv Largeur` | `SEMELLE │ 7,654321 vg³` |
| ↳ `× 5 =` | `38,2716 vg³` |
| `100 Pi Conv Largeur` | `SEMELLE │ 6,790123 vg³` |
| ↳ `Largeur` | `SECTION DE SEMELLE │ 264 po²` `MÉM.` |
| `150 Pi Pi Conv Hauteur` | `FEUILLES 4×8 │ 4,6875` |
| ↳ `Hauteur` | `FEUILLES 4×9 │ 4,166667` |
| ↳ `Hauteur` | `FEUILLES 4×10 │ 3,75` |
| ↳ `Hauteur` | `FEUILLES 4×12 │ 3,125` |
| ↳ `Hauteur` | `AIRE │ 150 pi²` |
| `40 Pi Conv Hauteur` | `FEUILLES 4×8 │ 10` |
| `18 Pi 7 Po 1 / 2 Conv 5` | `MONTANTS │ 15` |
| `16 Pi Conv 5` | `MONTANTS │ 13` |
| `24 Po Sto 5 12 Pi Conv 5` | `MONTANTS │ 7` |

### 19.8 Pente et triangle rectangle

| Frappes | Écran attendu |
|---|---|
| `30,45 Pente` | `PENTE │ 30,45°` |
| ↳ `Pente` | `PENTE │ 58,78702 %` |
| ↳ `Pente` | `RATIO DE PENTE │ 0,58787` |
| ↳ `Pente` | `PENTE │ 7 1/16 po` |
| `47,25 % Pente` | `PENTE │ 47,25 %` |
| ↳ `Pente` | `RATIO DE PENTE │ 0,4725` |
| ↳ `Pente` | `PENTE │ 5 11/16 po` |
| ↳ `Pente` | `PENTE │ 25,29°` |
| `0,65 Conv Pente` | `RATIO DE PENTE │ 0,65` |
| ↳ `Pente` | `PENTE │ 7 13/16 po` |
| ↳ `Pente` | `PENTE │ 33,02°` |
| ↳ `Pente` | `PENTE │ 65 %` |
| `250 Conv 9 Pente` | `PENTE │ 25 %` |
| ↳ `Pente Pente` | `PENTE │ 3 po` |
| `7 Po Pente 14 Pi 4 Po ÷ 2 = Course` | `COURSE │ 7 pi 2 po` |
| ↳ `Diag` | `DIAGONALE │ 8 pi 3 9/16 po` |
| ↳ `Diag` | `COUPE D'APLOMB │ 30,26°` |
| ↳ `Diag` | `COUPE DE NIVEAU │ 59,74°` |
| ↳ `Diag` | `DIAGONALE │ 8 pi 3 9/16 po` |
| `6 Pi 11 Po 1 / 2 Montée 14 Pi 6 Po Course Diag` | `DIAGONALE │ 16 pi 1 po` |
| ↳ `Pente` | `PENTE │ 5 3/4 po` |
| ↳ `Pente` | `PENTE │ 25,64°` |
| `9 Pi Montée 12 Pi Course Diag` | `DIAGONALE │ 15 pi 0 po` |
| ↳ `Pente` | `PENTE │ 9 po` |
| ↳ `Pente` | `PENTE │ 36,87°` |
| `7 Po Pente 11 Pi 6 Po Course Montée` | `MONTÉE │ 6 pi 8 1/2 po` |
| `30 Pente 20 Pi 4 Po Course Montée` | `MONTÉE │ 11 pi 8 7/8 po` |
| ↳ `Diag` | `DIAGONALE │ 23 pi 5 3/4 po` |
| `6 Po Pente 4 Pi Diag Course` | `COURSE │ 3 pi 6 15/16 po` |
| ↳ `Montée` | `MONTÉE │ 1 pi 9 7/16 po` |
| `65 % Pente 2 Pi Montée Course` | `COURSE │ 3 pi 0 15/16 po` |
| `12 Pi Course 13 Pi Diag Montée` | `MONTÉE │ 5 pi 0 po` |
| ↳ `Pente` | `PENTE │ 5 po` |
| `23 Pi 8 Po Montée 45 Pi 6 Po Course Diag` | `DIAGONALE │ 51 pi 3 7/16 po` |
| `3 Montée 4 Course Diag` | `DIAGONALE │ 5` |
| `7 Po Pente Eff Eff 8 Pi Course Montée` | `MONTÉE │ 4 pi 8 po` (la pente survit à `Effacer` ×2) |
| `65 % Pente 2 Pi Montée Course 55 Pi × Rap Course × 2 Pi =` | `338,4615 pi³` |

*Note sur la dernière ligne* : `Rap Course` rappelle la course **calculée** (3 pi 0 15/16 po) et la fournit comme opérande. Les trois facteurs sont de la famille Pieds : le résultat est en pieds cubes (R-31), pas en verges cubes.

### 19.9 Arêtier, noue et empannons

**Toit régulier 7/12, course 8 pi 5 po, entraxe 16 po.** Frappes : `8 Pi 5 Po Course 7 Po Pente Diag` → `DIAGONALE │ 9 pi 8 15/16 po`.

Pressions successives de `Arêtier` :

1. `ARÊTIER/NOUE │ 12 pi 10 1/2 po`
2. `COUPE D'APLOMB │ 22,42°`
3. `COUPE DE NIVEAU │ 67,58°`
4. `COUPE LATÉRALE │ 45,00°`

Pressions successives de `Empannon` :

1. `ENTRAXE │ 16 po` `MÉM.`
2. `INCRÉMENT │ 1 pi 6 1/2 po`
3. `EMPANNON 1 │ 8 pi 2 3/8 po`
4. `EMPANNON 2 │ 6 pi 7 7/8 po`
5. `EMPANNON 3 │ 5 pi 1 3/8 po`
6. `EMPANNON 4 │ 3 pi 6 13/16 po`
7. `EMPANNON 5 │ 2 pi 0 5/16 po`
8. `EMPANNON 6 │ 0 pi 5 13/16 po`
9. `COUPE D'APLOMB │ 30,26°`
10. `COUPE DE NIVEAU │ 59,74°`
11. `COUPE LATÉRALE │ 45,00°`
12. (reboucle) `ENTRAXE │ 16 po`

**Toit régulier 9/12, course 6 pi 9 po, entraxe 18 po.** Frappes : `18 Po Sto 5 9 Po Pente 6 Pi 9 Po Course`, puis `Empannon` : `ENTRAXE │ 18 po` · `INCRÉMENT │ 1 pi 10 1/2 po` · `EMPANNON 1 │ 6 pi 6 3/4 po` · `EMPANNON 2 │ 4 pi 8 1/4 po` · `EMPANNON 3 │ 2 pi 9 3/4 po` · `EMPANNON 4 │ 0 pi 11 1/4 po` · `COUPE D'APLOMB │ 36,87°` · `COUPE DE NIVEAU │ 53,13°` · `COUPE LATÉRALE │ 45,00°`.

**Toit régulier 6/12, course 12 pi, entraxe 16 po.** `6 Po Pente 12 Pi Course Diag` → `DIAGONALE │ 13 pi 5 po`; `Arêtier` → `ARÊTIER/NOUE │ 18 pi 0 po`, `COUPE D'APLOMB │ 19,47°`, `COUPE DE NIVEAU │ 70,53°`. `Empannon` : incrément `1 pi 5 7/8 po`, puis **huit** empannons : `11 pi 11 1/8 po`, `10 pi 5 1/4 po`, `8 pi 11 5/16 po`, `7 pi 5 7/16 po`, `5 pi 11 9/16 po`, `4 pi 5 11/16 po`, `2 pi 11 3/4 po`, `1 pi 5 7/8 po` (le neuvième, de longueur nulle, n'est pas listé).

**Toit irrégulier 7/12 et 8/12, course 4 pi, entraxe constant, décroissant.** Frappes : `7 Po Pente 4 Pi Course Diag` → `DIAGONALE │ 4 pi 7 9/16 po`; `8 Po Conv Arêtier` → `PENTE IRRÉGULIÈRE │ 8 po`.

Pressions de `Arêtier` : `ARÊTIER/NOUE IRRÉG. │ 5 pi 9 11/16 po` · `COUPE D'APLOMB │ 23,70°` · `COUPE DE NIVEAU │ 66,30°` · `COUPE LATÉRALE 1 │ 41,19°` · `COUPE LATÉRALE 2 │ 48,81°`.

`Conv Empannon`, puis `Empannon` répété :

1. `ENTRAXE │ 16 po` `MÉM.`
2. `INCRÉMENT │ 1 pi 4 13/16 po`
3. `EMPANNON IRRÉG. 1 │ 2 pi 9 5/8 po`
4. `EMPANNON IRRÉG. 2 │ 1 pi 4 13/16 po`
5. `COUPE D'APLOMB │ 33,69°`
6. `COUPE DE NIVEAU │ 56,31°`
7. `COUPE LATÉRALE │ 41,19°`
8. `ENTRAXE │ 16 po` `MÉM.`
9. `INCRÉMENT │ 1 pi 9 3/16 po`
10. `EMPANNON 1 │ 2 pi 10 3/8 po`
11. `EMPANNON 2 │ 1 pi 1 1/4 po`
12. `COUPE D'APLOMB │ 30,26°`
13. `COUPE DE NIVEAU │ 59,74°`
14. `COUPE LATÉRALE │ 48,81°`

**Même toit, préférences *Empannons* = Croissant et *Empannons irréguliers* = Appariés.** `Conv Empannon`, puis `Empannon` répété :

1. `ENTRAXE │ 16 po` `MÉM.`
2. `INCRÉMENT │ 1 pi 4 13/16 po`
3. `EMPANNON IRRÉG. 1 │ 1 pi 4 13/16 po`
4. `EMPANNON IRRÉG. 2 │ 2 pi 9 5/8 po`
5. `EMPANNON IRRÉG. 3 │ 4 pi 2 1/2 po`
6. `COUPE D'APLOMB │ 33,69°` · 7. `COUPE DE NIVEAU │ 56,31°` · 8. `COUPE LATÉRALE │ 41,19°`
9. `ENTRAXE │ 14 po` (sans `MÉM.` : entraxe recalculé)
10. `INCRÉMENT │ 1 pi 6 1/2 po`
11. `EMPANNON 1 │ 1 pi 6 1/2 po`
12. `EMPANNON 2 │ 3 pi 1 1/16 po`
13. `EMPANNON 3 │ 4 pi 7 9/16 po`
14. `COUPE D'APLOMB │ 30,26°` · 15. `COUPE DE NIVEAU │ 59,74°` · 16. `COUPE LATÉRALE │ 48,81°`

### 19.10 Mur en pente et toiture

**Mur en pente, montée 4 pi, course 8 pi.** `4 Pi Montée 8 Pi Course Conv Montée`, puis `Montée` répété : `ENTRAXE │ 16 po` `MÉM.` · `MONTANT 1 │ 3 pi 4 po` · `MONTANT 2 │ 2 pi 8 po` · `MONTANT 3 │ 2 pi 0 po` · `MONTANT 4 │ 1 pi 4 po` · `MONTANT 5 │ 0 pi 8 po` · `BASE │ 0 pi 0 po` · `ANGLE DU MUR │ 26,57°`.

**Avec une base de 5 pi.** `4 Pi Montée 8 Pi Course 5 Pi Conv Montée` → `BASE │ 5 pi 0 po`; puis `Montée` répété : `ENTRAXE │ 16 po` · `MONTANT 1 │ 8 pi 4 po` · `7 pi 8 po` · `7 pi 0 po` · `6 pi 4 po` · `MONTANT 5 │ 5 pi 8 po` · `BASE │ 5 pi 0 po` · `ANGLE DU MUR │ 26,57°`.

**Sans base, préférence *Mur en pente* = Croissant** : six montants — `0 pi 8 po`, `1 pi 4 po`, `2 pi 0 po`, `2 pi 8 po`, `3 pi 4 po`, `4 pi 0 po`.

**Montée 4 pi, course 6 pi, décroissant** : `3 pi 1 5/16 po`, `2 pi 2 11/16 po`, `1 pi 4 po`, `0 pi 5 5/16 po`; angle `33,69°`.

**Toiture 10/12, plan 14 pi × 11 pi.** `10 Po Pente 14 Pi Longueur 11 Pi Largeur Conv Diag`, puis `Diag` répété : `AIRE DU TOIT │ 200,4631 pi²` · `CARRÉS │ 2,00` · `PAQUETS │ 6,01` · `PAQUET │ 33,33 pi²` `MÉM.` · `FEUILLES 4×8 │ 6,26` · `PENTE │ 10 po` `MÉM.` · `AIRE EN PLAN │ 154 pi²`.

**Toiture, montée 10 pi, course 15 pi, plan 50 pi × 30 pi.** `10 Pi Montée 15 Pi Course 50 Pi Longueur 30 Pi Largeur Conv Diag` : `AIRE DU TOIT │ 1 802,776 pi²` · `CARRÉS │ 18,03` · `PAQUETS │ 54,08` · `PAQUET │ 33,33 pi²` · `FEUILLES 4×8 │ 56,34` · `PENTE │ 8 po` · `AIRE EN PLAN │ 1 500 pi²`.

### 19.11 Escaliers

Valeurs mémorisées par défaut (7 1/2 po, 10 po, 6 pi 8 po, 10 po). Pressions successives de `Escalier` après les frappes indiquées; les sorties 12 à 15 (valeurs mémorisées) suivent toujours.

| # | Étiquette | A — `9 Pi 11 Po Montée` | B — `20 Pi Course` | C — `10 Pi 1 Po Montée 15 Pi 5 Po Course` | D — comme C, lancé par `Conv Escalier` |
|---|---|---|---|---|---|
| 1 | CONTREMARCHE | `7 7/16 po` | `7 1/2 po` `MÉM.` | `7 9/16 po` `⚠` | `7 1/8 po` |
| 2 | NB CONTREMARCHES | `16` | `25` | `16` | `17` |
| 3 | ÉCART CONTREMARCHES | `0 po` | `0 po` | `0 po` | `1/8 po` |
| 4 | GIRON | `10 po` `MÉM.` | `10 po` | `12 5/16 po` | `11 9/16 po` |
| 5 | NB MARCHES | `15` | `24` | `15` | `16` |
| 6 | ÉCART GIRONS | `0 po` | `0 po` | `−5/16 po` | `0 po` |
| 7 | TRÉMIE | `10 pi 1 po` | `10 pi 0 po` | `12 pi 2 1/2 po` | `12 pi 2 1/16 po` |
| 8 | LIMON | `15 pi 6 15/16 po` | `25 pi 0 po` | `18 pi 0 3/4 po` | `18 pi 1 5/16 po` |
| 9 | INCLINAISON | `36,64°` | `36,87°` | `31,56°` | `31,64°` |
| 10 | COURSE | `12 pi 6 po` | `20 pi 0 po` `MÉM.` | `15 pi 5 po` `MÉM.` | `15 pi 5 po` `MÉM.` |
| 11 | MONTÉE | `9 pi 11 po` `MÉM.` | `15 pi 7 1/2 po` | `10 pi 1 po` `MÉM.` | `10 pi 1 po` `MÉM.` |

**Escalier en métrique.** `2,75 m Montée`, puis `Escalier` répété : `CONTREMARCHE │ 196 mm` `⚠` · `NB CONTREMARCHES │ 14` · `ÉCART CONTREMARCHES │ −6 mm` · `GIRON │ 254 mm` `MÉM.` · `NB MARCHES │ 13` · `ÉCART GIRONS │ 0 mm` · `TRÉMIE │ 2,962 m` · `LIMON │ 4,171 m` · `INCLINAISON │ 37,66°` · `COURSE │ 3,302 m` · `MONTÉE │ 2,750 m` `MÉM.`.

Même montée avec `Conv Escalier` : `183 mm` · `15` · `−5 mm` · `254 mm` · `14` · `0 mm` · `3,173 m` · `4,383 m` · `35,77°` · `3,556 m` · `2,750 m`.

### 19.12 Cercle, arc, murs cintrés, colonne, cône, polygone, onglet

| Frappes | Écran attendu |
|---|---|
| `11 Po Cercle` | `DIAMÈTRE │ 11 po` |
| ↳ `Cercle` | `AIRE │ 95,03318 po²` |
| ↳ `Cercle` | `CIRCONFÉRENCE │ 34 9/16 po` |
| `5 Pi Cercle 85 Arc` | `ARC │ 85,00°` |
| ↳ `Arc` | `ARC │ 3 pi 8 1/2 po` |
| `3 Pi 6 Po Course 1 Pi 3 Po Montée Conv Arc` | `RAYON │ 1 pi 10 3/16 po` |
| ↳ `Arc` | `ARC │ 142,15°` |
| ↳ `Arc` | `ARC │ 4 pi 7 1/16 po` |
| `5 Pi Cercle 3 Pi 3 Po Arc` | `ARC │ 3 pi 3 po` |
| ↳ `Arc` | `ARC │ 74,48°` |
| ↳ `Arc` | `CORDE │ 3 pi 0 5/16 po` |
| ↳ `Arc` | `SEGMENT │ 1,051381 pi²` |
| ↳ `Arc` | `SECTEUR │ 4,0625 pi²` |
| ↳ `Arc` | `FLÈCHE │ 0 pi 6 1/8 po` |
| ↳ `Arc` | `ENTRAXE │ 16 po` `MÉM.` |
| ↳ `Arc` | `MUR CINTRÉ 1 │ 0 pi 4 5/8 po` |
| `10 Pi Cercle 6 Pi Course Montée` | `FLÈCHE │ 1 pi 0 po` |
| `10 Pi Cercle 1 Pi Montée Course` | `CORDE │ 6 pi 0 po` |

**Mur cintré extérieur (défaut).** `8 Pi 1 Po Course 2 Pi 10 Po 1 / 2 Montée`, puis `Arc` répété : `ARC │ 141,70°` · `ARC │ 10 pi 7 po` · `CORDE │ 8 pi 1 po` · `SEGMENT │ 16,96327 pi²` · `SECTEUR │ 22,63527 pi²` · `FLÈCHE │ 2 pi 10 1/2 po` · `ENTRAXE │ 16 po` · `MUR CINTRÉ 1 │ 0 pi 2 9/16 po` · `MUR CINTRÉ 2 │ 0 pi 11 3/16 po` · `MUR CINTRÉ 3 │ 2 pi 9 1/8 po`.

**Même arc avec une base de 2 pi** (`2 Pi Conv Montée` → `BASE │ 2 pi 0 po`, puis `Arc`…) : murs cintrés `2 pi 2 9/16 po`, `2 pi 11 3/16 po`, `4 pi 9 1/8 po`.

**Autre arc extérieur** — corde 9 pi 2 po, flèche 3 pi 11 1/2 po : `ARC │ 163,26°`, `ARC │ 13 pi 2 3/8 po`, `SEGMENT │ 27,48605 pi²`, `SECTEUR │ 30,57678 pi²`; murs cintrés `0 pi 2 3/8 po`, `0 pi 10 1/8 po`, `2 pi 3 9/16 po`.

**Mur cintré intérieur** (préférence *Mur cintré* = Intérieur). `15 Pi Course 5 Pi Montée`, puis `Arc` répété : `ARC │ 134,76°` · `ARC │ 19 pi 1 5/16 po` · `CORDE │ 15 pi 0 po` · `SEGMENT │ 54,19722 pi²` · `SECTEUR │ 77,63472 pi²` · `FLÈCHE │ 5 pi 0 po` · `ENTRAXE │ 16 po` · `MUR CINTRÉ 1 │ 4 pi 10 11/16 po` · `4 pi 6 5/8 po` · `3 pi 11 3/8 po` · `3 pi 0 1/16 po` · `MUR CINTRÉ 5 │ 1 pi 6 1/4 po`.

| Frappes | Écran attendu |
|---|---|
| `2 Pi 4 Po Cercle Cercle` | `AIRE │ 4,276057 pi²` |
| ↳ `4 Pi 6 Po Hauteur Conv Cercle` | `COLONNE – VOLUME │ 19,24226 pi³` |
| ↳ `Cercle` | `COLONNE – SURFACE │ 41,53884 pi²` |
| `3 Pi 6 Po Cercle 5 Pi Hauteur Conv Cercle Cercle Cercle` | `CÔNE – VOLUME │ 16,03521 pi³` |
| ↳ `Cercle` | `CÔNE – SURFACE │ 38,74513 pi²` |
| `1,75 Sto 0` | `MASSE VOLUMIQUE │ 1,75 tc/vg³` `MÉM.` |
| ↳ `3 Pi 4 Po 1 / 2 Cercle Cercle` | `AIRE │ 8,946176 pi²` |
| ↳ `11 Pi 6 Po Hauteur Conv Cercle` | `COLONNE – VOLUME │ 102,881 pi³` |
| ↳ `Conv Vg` | `COLONNE – VOLUME │ 3,810408 vg³` (conversion d'une sortie : l'étiquette reste) |
| ↳ `× 5 =` | `19,05204 vg³` |
| ↳ `Conv 6` | `33,34107 tc` |
| ↳ `Conv 4` | `66 682,14 lb` |
| ↳ `Conv 1` | `30 246,51 kg` |
| `15 Pi Conv Arc` | `RAYON │ 15 pi 0 po` |
| ↳ `6 Conv Course` | `ANGLE INTÉRIEUR │ 120,00°` |
| ↳ `Course` | `DEMI-ANGLE │ 60,00°` |
| ↳ `Course` | `CÔTÉ │ 15 pi 0 po` |
| ↳ `Course` | `PÉRIMÈTRE │ 90 pi 0 po` |
| ↳ `Course` | `AIRE │ 584,5671 pi²` |
| ↳ `Course` | `RAYON │ 15 pi 0 po` |
| ↳ `Course` | `NB CÔTÉS │ 6` |
| `10 Pi Conv Arc 8 Conv Course Course Course` | `CÔTÉ │ 7 pi 7 7/8 po` |
| ↳ `Course` | `PÉRIMÈTRE │ 61 pi 2 3/4 po` |
| ↳ `Course` | `AIRE │ 282,8427 pi²` |
| `38 Conv Onglet` | `ANGLE MOULURE │ 38,00°` `MÉM.` |
| ↳ `60 Onglet` | `ONGLET │ 46,84°` |
| ↳ `Onglet` | `BISEAU │ 43,03°` |
| ↳ `Onglet` | `ANGLE MOULURE │ 38,00°` `MÉM.` |
| ↳ `Onglet` | `COIN │ 60,00°` |
| `90 Onglet` | `ONGLET │ 35,26°` |
| ↳ `Onglet` | `BISEAU │ 30,00°` |
| `8 Onglet` | `COIN │ 135,00°` |
| ↳ `Onglet` | `ONGLET │ 16,32°` |
| ↳ `Onglet` | `BISEAU │ 15,70°` |
| `38 Sto Onglet 90 Onglet` | `ONGLET │ 31,62°` |
| ↳ `Onglet` | `BISEAU │ 33,86°` |
| `40 Sto Onglet 70 Onglet` | `ONGLET │ 42,55°` |
| ↳ `Onglet` | `BISEAU │ 38,87°` |

### 19.13 Angles et trigonométrie

| Frappes | Écran attendu |
|---|---|
| `23,42,39 Conv ,` | `23,71°` |
| `44,29 Conv ,` | `44°17′24″` |
| `7,45,33 + 11,16,20 =` | `19°01′53″` |
| `30,89, =` (la seconde virgule déclenche la saisie D:M:S) | `31°29′00″` |

Avec la préférence *Mode trigonométrique* activée (`Longueur` = sin, `Largeur` = cos, `Hauteur` = tan) :

| Frappes | Écran attendu |
|---|---|
| `30 Longueur` | `0,5` |
| `60 Largeur` | `0,5` |
| `45 Hauteur` | `1` |
| `0,5 Conv Longueur` | `30,00°` |
| `8 Po Pente Pente` | `PENTE │ 33,69°` |
| ↳ `Hauteur` | `0,666667` |
| `30,25 Hauteur` | `0,583183` |
| ↳ `Conv Pente` | `RATIO DE PENTE │ 0,583183` |
| ↳ `Pente` | `PENTE │ 7 po` |
| `6 Pi ÷ 10 Pi = Conv Largeur` | `53,13°` |
| ↳ `Conv ,` | `53°07′48″` |
| `35 % Conv Hauteur` | `19,29°` |
| ↳ `Conv ,` | `19°17′24″` |
| `1 Pi Longueur` | `ERREUR – trigonométrie` |

### 19.14 Erreurs et effacement

| Frappes | Écran attendu |
|---|---|
| `5 Pi + 3 =` | `ERREUR – unités incompatibles` |
| ↳ `7` | `ERREUR – unités incompatibles` (touche ignorée) |
| ↳ `Eff` | `0` |
| `5 ÷ 0 =` | `ERREUR – calcul impossible` |
| `5 Pi Pi × 2 Pi Pi =` | `ERREUR – unités incompatibles` |
| `4 Pi Conv Eff` | `ERREUR – unités incompatibles` |
| `9 Conv − Conv Eff` | `ERREUR – calcul impossible` |
| `5 Pi 6 Pi` | `ERREUR – saisie invalide` |
| `1 / 0 =` | `ERREUR – saisie invalide` |
| `Escalier` | `AUCUNE VALEUR` |
| `Diag` | `AUCUNE VALEUR` |
| `10 Pi Diag 12 Pi Montée Course` | `ERREUR – calcul impossible` |
| `5 Conv Course` (polygone sans rayon) | `AUCUNE VALEUR` |
| `55 Conv 0` (coût sans multiplication en attente) | `ERREUR – saisie invalide` |
| `8 Pi Course Eff Eff Course` | `AUCUNE VALEUR` (la course a été effacée) |
| `5 + 3 Eff 4 =` | `9` (la première pression d'`Effacer` garde l'opération en attente) |

### 19.15 Tests unitaires de formatage (appels directs, sans le moteur)

| Valeur (base, format) | Texte attendu |
|---|---|
| 11,99 po, `PiPo` | `1 pi 0 po` |
| 11,97 po, `PiPo` | `1 pi 0 po` |
| −63,5 po, `PiPo` | `−5 pi 3 1/2 po` |
| 0,5 po, `PoFrac` | `1/2 po` |
| 0,03 po, `PoFrac` | `0 po` |
| 0,04 po, `PoFrac` | `1/16 po` |
| 67,5 po, `PiPo`, mode Constant | `5 pi 7 8/16 po` |
| 67,5 po, `PiPo`, résolution 1/4 | `5 pi 7 1/2 po` |
| 15 006 po, `PiPo` | `1 250 pi 6 po` |
| 1234567,891, `Nombre` | `1 234 568` |
| 0,000123456, `Nombre` | `0,000123` |
| 9,9999999, `Nombre` | `10` |
| −0,00000001, `Nombre` | `0` |
| 2/3, `Nombre` | `0,666667` |
| 1000000/3, `Nombre` | `333 333,3` |
| 12345678,9, `Nombre` | `12 345 679` |
| 1234,5, `Nombre`, séparateur de milliers désactivé | `1234,5` |
| 1234,5, `Nombre`, séparateur décimal = point | `1 234.5` |
| 96 po, `M` | `2,438 m` |
| 96 po, `M`, préférence Flottant | `2,4384 m` |
| 44,29, `Dms` | `44°17′24″` |
| 0,4725, `Pourcent` | `47,25 %` |
| 283,0633, `Monnaie` | `283,06 $` |

Et l'analyse de texte de 7.9 (huit exemples de longueurs, quatre d'aires), plus au moins dix entrées invalides qui doivent retourner une erreur (`abc`, `5 pi 6 pi`, `1/0`, `--3`, chaîne vide…).

### 19.16 Propriétés à vérifier par boucle

- Conversion aller-retour : pour 1 000 longueurs pseudo-aléatoires, changer de format ne modifie jamais `base`.
- Pythagore : pour 1 000 couples (pente, course), `diagonale² ≈ montée² + course²` à 10⁻⁹ près en valeur relative.
- Cohérence des cycles : pour chaque fonction, appuyer `n + 1` fois (n = longueur du cycle) ramène à la première sortie.
- Symétrie des empannons : en toit régulier, décroissant, la somme `empannon k + k × incrément` égale le chevron commun.

---

## 20. Plan de réalisation par phases

Avancer dans l'ordre. **Ne pas commencer une phase tant que les tests de la précédente ne passent pas.** À la fin de chaque phase : `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test`, puis un court compte rendu à l'utilisateur.

| Phase | Contenu | Critère de sortie |
|---|---|---|
| **0. Mise en place** | Vérifier `rustc --version` (≥ 1.95). Créer l'espace de travail, les deux crates, le `.gitignore`, le `README.md`. Ajouter `eframe` et confirmer sa version exacte; parcourir l'annexe C et la documentation de cette version. Fenêtre vide qui s'ouvre avec le bon titre. | `cargo run -p calculatrice-chantier` ouvre une fenêtre; `cargo test` passe (zéro test). |
| **1. Unités, valeurs, formatage** | `unites.rs`, `valeur.rs`, `format.rs`, `analyse.rs`, `etiquettes.rs`, `erreurs.rs`, `prefs.rs`. | Vecteurs 19.15 et exemples 7.9 verts. |
| **2. Saisie et moteur de base** | `touches.rs`, `saisie.rs`, `moteur.rs` (préfixes, opérateurs, `%`, fonctions mathématiques, effacement, erreurs), `conversions.rs`, `memoire.rs`. Outil de test 19.2. | Vecteurs 19.3, 19.4, 19.5, 19.14 verts. |
| **3. Triangle et toit** | `triangle.rs`, `aretier.rs`, `empannons.rs`, `mur_pente.rs`, `toiture.rs`; cycles; `Rappel` des valeurs. | Vecteurs 19.8, 19.9, 19.10 verts. |
| **4. Escalier, cercle, polygone, onglet** | `escalier.rs`, `arc.rs`, `colonne_cone.rs`, `polygone.rs`, `onglet.rs`, `trigo.rs`. | Vecteurs 19.11, 19.12, 19.13 verts. |
| **5. Dimensions et matériaux** | `dimensions.rs`, `materiaux.rs` (blocs, semelle, gypse, montants, pmp, coût), conversions volume ↔ masse. | Vecteurs 19.6, 19.7 verts. |
| **6. Ruban, persistance, robustesse** | `ruban.rs`, `persistance.rs`; test de robustesse 19.1; propriétés 19.16. | Tous les tests du noyau verts; aucun avertissement. |
| **7. Interface : écran principal** | `theme.rs`, `afficheur.rs`, `clavier.rs`, `raccourcis.rs`, `application.rs` : afficheur (fractions empilées), grille de 40 touches, `Conv` armé, ⌫, bande de fractions rapides, mise à l'échelle, thèmes. | Les exemples de 19.4 et 19.11 se rejouent à la souris et au clavier, résultat identique. |
| **8. Interface : écrans secondaires** | Préférences (avec dialogues de valeurs mémorisées), Rubans (courant, enregistrés, copier, exporter), Aide (recherche, fiches, appui long, clic droit), Bienvenue, Trucs de pro, À propos, menu ☰, barre inférieure, panneau de ruban en fenêtre large; `stockage.rs`. | Chaque préférence modifie bien le comportement; l'état survit à un redémarrage. |
| **9. Finition** | Icône, plein écran, mode trigonométrique à l'écran, infobulles, vérification des caractères (16.3), mise à l'échelle Windows 125 % et 150 %, version finale `--release`, mise à jour du `README.md`. | Liste 21 entièrement cochée. |

Conseils d'exécution :

- En phase 2, écrire **d'abord** l'outil `jouer` et quelques vecteurs, puis le moteur : les vecteurs guident l'implémentation.
- Si un vecteur échoue d'un seizième de pouce, vérifier l'arrondi (R-23) et, pour l'escalier, la règle R-42 (les résultats partent des valeurs **arrondies**).
- En cas d'erreur de compilation liée à egui, consulter la documentation de la version installée (`cargo doc -p egui --open`, ou les sources sous `%USERPROFILE%\.cargo\registry\src\`) plutôt que d'essayer des signatures de mémoire.
- L'interface ne peut pas être vérifiée automatiquement de façon fiable : à la fin des phases 7 à 9, demander à l'utilisateur de lancer le programme et de parcourir la liste 21.3.

---

## 21. Définition de « terminé »

### 21.1 Code

- [ ] `cargo build --workspace --release` réussit sans avertissement.
- [ ] `cargo clippy --workspace -- -D warnings` et `cargo fmt --all --check` passent.
- [ ] Version Web : `cargo clippy --target wasm32-unknown-unknown -p calculatrice-chantier -- -D warnings` passe et `trunk build --release` réussit.
- [ ] `cargo test --workspace` : 100 % des tests verts, dont **tous** les vecteurs de la section 19.
- [ ] Aucune dépendance hors de la liste de la section 3.
- [ ] Aucune chaîne visible par l'utilisateur hors des fichiers prévus (R-52); aucun mot anglais à l'écran en français, aucun mot français à l'écran en anglais (R-10, R-62).
- [ ] Aucune mention du nom, de la marque ou du site de l'application de référence (R-01); l'adresse courriel de la photo `05.png` n'apparaît nulle part.

### 21.2 Fonctions

- [ ] Les 40 touches et leurs fonctions secondaires sont opérationnelles (table 5.2).
- [ ] Toutes les préférences de 13.1 à 13.3 agissent; les onze valeurs mémorisées de 13.4 sont modifiables et persistantes.
- [ ] Ruban courant, enregistrement, renommage, copie, export `.txt`, suppression.
- [ ] Aide : une fiche par fonction, recherche sans accents, appui long et clic droit.
- [ ] Écrans Bienvenue, Trucs de pro, À propos (avec l'avertissement de 1.5).
- [ ] Les 76 calculateurs par métier donnent les valeurs de 22.14; leurs résultats s'ajoutent au ruban; leurs saisies sont reprises à la réouverture.
- [ ] Le manuel d'utilisation est à jour (test `le_manuel_est_a_jour`) et s'ouvre depuis le menu (15.4); le pied du menu montre la mention, le courriel, le site, la version et la révision (15.1).
- [ ] Le clavier s'adapte au métier (section 23) : chaque calculateur se branche sur le clavier, se remplit avec les touches et donne les mêmes résultats que son formulaire; l'outil branché est repris à la réouverture.
- [ ] Les deux langues (section 24) : le bouton `FR | EN` change toute l'interface; les nombres et les unités suivent; les valeurs saisies restent les mêmes; les deux manuels sont à jour; la langue choisie est reprise à la réouverture.

### 21.3 Recette manuelle (à faire par l'utilisateur)

- [ ] La fenêtre s'ouvre en moins d'une seconde, en français (sur un appareil en français), avec la virgule sur la touche décimale.
- [ ] Le bouton `EN` met toute la calculatrice en anglais : touches (`Pitch`, `Rise`, `Run`, `Feet`, `Inch`), point sur la touche décimale, `5 ft 7 1/2 in + 18 ft 8 in =` affiche `24 ft 3 1/2 in`; le bouton `FR` la remet en français. Après fermeture et réouverture, la langue choisie est conservée.
- [ ] Sur un téléphone ou un ordinateur réglé en anglais, la première visite du site s'ouvre en anglais.
- [ ] `5 pi 7 1/2 po + 18 pi 8 po =` affiche `24 pi 3 1/2 po`, la fraction **empilée**.
- [ ] `7 po Pente`, `8 pi 5 po Course`, `Diag` : `9 pi 8 15/16 po`; `Arêtier Noue` : `12 pi 10 1/2 po`.
- [ ] `9 pi 11 po Montée`, `Escalier` ×11 : les valeurs de la colonne A de 19.11.
- [ ] `Conv` s'allume, les étiquettes secondaires passent en gras, puis s'éteint après la touche suivante.
- [ ] Un clic droit sur `Escalier` ouvre sa fiche d'aide.
- [ ] Changer la résolution à 1/8 modifie aussitôt l'affichage; après fermeture et réouverture, le réglage est conservé.
- [ ] `F11` bascule le plein écran; redimensionner la fenêtre garde tout lisible; au-delà de 880 points de large, le ruban apparaît à droite.
- [ ] Thèmes clair et sombre corrects; aucun caractère remplacé par un carré vide.
- [ ] Le ruban s'enregistre, se rouvre et s'exporte en `.txt` lisible dans le Bloc-notes (accents intacts).
- [ ] Version Web : la page s'ouvre sur ordinateur et sur téléphone; les mêmes calculs donnent les mêmes résultats; après rechargement de la page, les réglages et les rubans sont toujours là; un ruban s'exporte par téléchargement.
- [ ] Version Web : le menu « Installer l'application » l'installe (ou explique comment); une fois installée, elle s'ouvre et calcule en mode avion.
- [ ] Version Web installée, en mode avion : « Manuel d'utilisation » ouvre bien le manuel, et non une seconde calculatrice.
- [ ] Dans le menu ☰ : « Manuel d'utilisation » ouvre le manuel dans le navigateur; le courriel ouvre un nouveau message à `info@constructoai.ca`; le site ouvre `constructoai.ca`. (Ces ouvertures ne se vérifient pas par les tests automatisés.)
- [ ] Sur un téléphone : toucher un métier dans la barre au-dessus du clavier, puis `24` `Pieds` `Longueur` `20` `Pieds` `Largeur` `4` `Pouces` `Épaisseur` `Résultat` `Résultat` donne `5,93 vg³ (4,53 m³)` pour la dalle de béton; les touches restent assez grandes pour le doigt.

---

## 22. Calculateurs par métier

Ajoutés le 2026-11-19, à la demande de l'utilisateur, pour d'autres corps de métier que la charpente. Le catalogue a été porté le même jour à **76 calculateurs pour onze métiers** (version 0.4), quand l'utilisateur a demandé des outils « vraiment complets », à utiliser tous les jours. Chaque calculateur s'utilise de deux façons : en **formulaire** (onglet Métiers : des champs à remplir, des résultats qui se recalculent à chaque frappe) ou **au clavier** (ses champs deviennent les touches du haut de la calculatrice, [section 23](#23-clavier-adapté-au-métier)).

### 22.1 Principes

- **Géométrie et physique seulement.** Aucun calculateur ne reproduit une table de code ni ne se prononce sur la conformité. Là où un code s'applique (pente minimale d'un drain, chute de tension permise, calibre minimal), une note le rappelle sans donner la valeur.
- **Tout le calcul est dans `noyau`** (R-48, R-49) : `noyau/src/metiers/` décrit chaque calculateur par des données (identifiant, nom, résumé, champs, note) et une fonction pure. L'interface (`app/src/vues/metiers.rs`) dessine n'importe quel calculateur à partir de cette description, sans rien savoir de lui.
- **Un calculateur = un identifiant stable** (`dalle`, `chute-de-tension`…), qui sert à retrouver les dernières saisies.
- **Dix champs au plus** par calculateur, pour qu'il tienne sur deux rangées de touches (23.3). Un champ au libellé long porte aussi un **libellé court** pour sa touche : deux lignes au plus, de douze caractères au plus.
- **Les métiers**, dans l'ordre où l'interface les présente : Estimation et soumission · Terrassement et aménagement · Béton et fondations · Maçonnerie et pavage · Toiture et revêtement · Gypse et isolation · Finition · Plomberie · Électricité · Ventilation et chauffage · Acier et soudure. Chacun a un nom en un mot pour la barre des métiers : Estimation, Terrassement, Béton, Maçonnerie, Toiture, Gypse, Finition, Plomberie, Électricité, Ventilation, Acier.
- La charpente, les toits en pente et les escaliers restent au clavier de base : l'écran des métiers et la barre des métiers y renvoient.

### 22.2 Règles communes

**Champs.** Quatre natures :

| Nature | Saisie | Refus |
|---|---|---|
| Longueur | comme partout ailleurs (7.9) : `12 pi 6 po`, `30 po`, `3,8 m`, `90 mm`. Un nombre seul prend l'unité indiquée à côté du champ (pi ou po). | « Entrez une longueur, par exemple 12 pi 6 po, 30 po ou 3,8 m. » |
| Nombre | un nombre, virgule ou point; l'unité est écrite à côté du champ | « Entrez un nombre. » |
| Entier | un nombre entier | « Entrez un nombre entier. » |
| Choix | une option dans une liste | — |

Sauf mention contraire, une valeur doit être plus grande que zéro (« La valeur doit être plus grande que zéro. »); là où zéro a un sens (pertes, portes, fenêtres), seul le négatif est refusé (« La valeur ne peut pas être négative. »). Un champ vide n'est pas une erreur : tant qu'un champ obligatoire est vide, il n'y a simplement pas de résultat.

**Résultats.** Ils suivent le système de la **première longueur saisie** :

| Grandeur | Saisie impériale | Saisie métrique |
|---|---|---|
| Longueur | pieds-pouces à la résolution des préférences; pouces seuls sous un pied | l'unité de la saisie : m à 3 décimales, cm à 1 décimale, mm sans décimale |
| Aire | `480 pi² (44,6 m²)` | `44,6 m² (480 pi²)` |
| Volume de béton, de terre, de pierre | `5,93 vg³ (4,53 m³)` | `4,53 m³ (5,93 vg³)` |
| Volume d'un liquide | `24,7 L (6,53 gal US)` | idem |
| Masse | `403 lb (183 kg)` | `183 kg (403 lb)` |
| Masse en vrac | `13,3 tc (12,1 t)` | `12,1 t (13,3 tc)` |
| Deux écritures d'une même grandeur | `5,30 pi/s (1,62 m/s)`, `2,51 lb/pi (3,73 kg/m)` | idem, dans le même ordre |
| Angle | `26,57°` : le signe des degrés se colle au nombre | idem |
| Mot | le calibre d'un conducteur : `8 AWG` | idem |

Les nombres s'écrivent avec les **décimales utiles** : aucune à partir de 100, une à partir de 10, deux à partir de 1, trois en dessous. Les quantités à acheter (sacs, tuiles, boîtes, contenants) sont arrondies à l'entier supérieur. Le séparateur décimal et le groupement par milliers suivent les préférences.

**Ruban.** Le bouton `Ajouter au ruban` inscrit un bloc : un filet, le nom du calculateur (ligne de titre), les valeurs saisies remises au propre, les résultats précédés de `=`, un filet. Ce bloc s'enregistre, se copie et s'exporte comme le reste du ruban.

**Persistance.** Les dernières saisies de chaque calculateur sont conservées (`metiers.json`, section 17.2). `Recommencer` remet les valeurs par défaut.

**Constantes.** 1 gal US = 231 po³; 1 L = 61,0237 po³; eau : 1 kg/L et 9,806 65 kPa par mètre de hauteur; 1 lb/po² = 6,894 757 kPa; 1 HP = 746 W; 1 kW = 3 412,142 BTU/h; 1 tonne de réfrigération = 12 000 BTU/h; 1 pi³/min = 0,471 947 L/s; 1 po² = 645,16 mm²; chaleur massique de l'eau : 4,186 kJ par kilogramme et par degré Celsius; 1 RSI = 5,678 263 R; air aux conditions normales : 1,08 BTU/h par pi³/min et par degré Fahrenheit; acier 7 850 kg/m³, acier inoxydable 8 000 kg/m³, aluminium 2 700 kg/m³.

### 22.3 Estimation et soumission

Ajouté le 2026-11-19, à la demande de l'utilisateur, d'après la méthode de son profil d'entrepreneur général : le **coût majoré**. Au coût de base des travaux s'ajoutent l'administration, les imprévus et le profit, **tous trois calculés sur le coût de base** (ils s'additionnent, ils ne se composent pas); les taxes s'appliquent ensuite au **prix avant taxes**. Chaque montant est arrondi au cent, pour que les lignes affichées s'additionnent exactement. Les montants s'écrivent `1 234,56 $`.

Pourcentages de départ : administration 3 %, imprévus 12 %, profit 15 %, TPS 5 %, TVQ 9,975 %. Tous se modifient. Une note donne des repères pour l'administration et les imprévus : 3 % et 12 % en résidentiel neuf, 3 % et 15 % en rénovation résidentielle, 3 % et 10 % en commercial neuf. L'application ne contient **aucun prix** : ni coût au pied carré, ni taux horaire. L'utilisateur saisit les siens.

| Calculateur | Champs | Résultats |
|---|---|---|
| **Prix d'une soumission** `soumission` | Coût de base des travaux ($), Administration (% ; 3), Imprévus (% ; 12), Profit (% ; 15), TPS (% ; 5), TVQ (% ; 9,975) | Administration, Imprévus et Profit = coût de base × pourcentage · Prix avant taxes = somme · TPS et TVQ = prix avant taxes × taux · Prix, taxes comprises |
| **Budget au pied carré** `budget-au-pied-carre` | Superficie habitable (pi²), Coût de base ($/pi²), Superficie du garage (pi², facultatif), Coût de base du garage ($/pi², facultatif), puis les cinq pourcentages | Coût de base = superficie × coût (+ garage) · Administration, imprévus et profit · Prix avant taxes · TPS et TVQ · Prix, taxes comprises · Prix au pied carré, taxes comprises = prix ÷ superficie totale. Un seul des deux champs du garage rempli : « Pour le garage, donnez la superficie et le coût au pied carré. » |
| **Coût de la main-d'œuvre** `main-d-oeuvre` | Nombre d'ouvriers (entier ; 1), Heures par ouvrier (h), Taux horaire ($/h), Charges et avantages sociaux (% ; 0) | Heures totales · Coût horaire, charges comprises = taux × (1 + charges ÷ 100) · Coût de la main-d'œuvre |
| **Taxes du Québec** `taxes` | Montant ($), Ce montant est (avant taxes ★, taxes comprises), TPS (% ; 5), TVQ (% ; 9,975) | Montant avant taxes · TPS · TVQ · Total. À partir d'un total : montant avant taxes = total ÷ (1 + somme des taux), et la TVQ prend le reste, pour que les trois lignes redonnent le total saisi |
| **Majoration et marge** `majoration-et-marge` | Coût ($), Majoration sur le coût (%) | Prix de vente = coût × (1 + majoration ÷ 100) · Profit · Marge sur le prix de vente = profit ÷ prix |
| **Facture d'étape** `facturation-progressive` | Montant du contrat, avant taxes ($), Avancement des travaux (%), Retenue (% ; 10), Déjà facturé, avant taxes ($ ; 0), TPS (% ; 5), TVQ (% ; 9,975) | Travaux exécutés à ce jour = contrat × avancement · Retenue = exécutés × retenue · Cette facture, avant taxes = exécutés − retenue − déjà facturé · TPS · TVQ · Cette facture, taxes comprises. Refus : « L'avancement ne dépasse pas 100 %. », « La retenue doit être inférieure à 100 %. », « Le montant déjà facturé dépasse les travaux exécutés, retenue déduite. » |
| **Durée des travaux** `duree-des-travaux` | Quantité à réaliser, Rendement d'un ouvrier, à l'heure, Nombre d'ouvriers (entier ; 1), Heures par jour (h ; 8) | Heures de travail, en tout = quantité ÷ rendement · Durée, à cette équipe = heures ÷ ouvriers · Jours de travail = durée ÷ heures par jour. Plus de 24 h par jour : « Une journée compte 24 heures. » |

### 22.4 Béton et fondations

| Calculateur | Champs (unité par défaut; valeur de départ) | Résultats |
|---|---|---|
| **Dalle de béton** `dalle` | Longueur (pi), Largeur (pi), Épaisseur (po), Pertes (% ; 10) | Surface = L × l · Volume = L × l × e · À commander = volume × (1 + pertes ÷ 100) |
| **Semelle filante** `semelle` | Longueur totale (pi), Largeur (po), Épaisseur (po), Pertes (% ; 10) | Volume = L × l × e · À commander |
| **Mur de fondation** `mur-de-fondation` | Longueur totale (pi), Hauteur (pi), Épaisseur (po), Pertes (% ; 10) | Volume = L × h × e · À commander · Coffrage, deux faces = 2 × L × h |
| **Colonnes et pieux ronds** `colonnes` | Diamètre (po), Hauteur (pi), Nombre de colonnes (entier ; 1), Pertes (% ; 10), Rendement d'un sac (L ; 14) | Volume = π ÷ 4 × d² × h × n · À commander · Sacs = volume à commander en litres ÷ rendement, arrondi au-dessus |
| **Armature d'une dalle** `armature` | Longueur (pi), Largeur (pi), Espacement des barres (po ; 16), Barre (10M, 15M, 20M, 25M) | Barres dans le sens de la longueur = ⌊largeur ÷ espacement⌋ + 1 · Barres dans le sens de la largeur = ⌊longueur ÷ espacement⌋ + 1 · Longueur totale · Masse = longueur totale × masse linéique |
| **Béton en sacs** `sacs-de-beton` | Longueur (pi), Largeur (pi), Épaisseur (po), Pertes (% ; 10), Rendement d'un sac (L ; 14) | Volume · Volume à préparer, pertes comprises, en litres · Sacs = litres ÷ rendement, arrondi au-dessus |
| **Escalier de béton** `escalier-de-beton` | Nombre de marches (entier), Hauteur d'une contremarche (po), Giron (po), Largeur de l'escalier (pi), Épaisseur de la paillasse (po ; zéro permis), Pertes (% ; 10) | Volume = n × (h × g ÷ 2 + √(h² + g²) × e) × largeur : un prisme triangulaire par marche, plus la paillasse le long de la pente · À commander |
| **Treillis métallique** `treillis` | Longueur (pi), Largeur (pi), Longueur d'une feuille (pi ; 8), Largeur d'une feuille (pi ; 4), Chevauchement (po ; 6) | Surface · Feuilles = surface ÷ [(longueur de la feuille − chevauchement) × (largeur de la feuille − chevauchement)], arrondi au-dessus. « Le chevauchement dépasse la feuille. » |

Masse linéique des barres : 10M 0,785 kg/m; 15M 1,570; 20M 2,355; 25M 3,925. L'armature ne compte ni chevauchements ni enrobage des rives (une note le dit). Le mur ne déduit pas les ouvertures.

### 22.5 Plomberie

| Calculateur | Champs | Résultats |
|---|---|---|
| **Pente d'un drain** `pente-de-drain` | Longueur du tuyau (pi), Pente par pied de longueur (po ; 1/4) | Dénivelé = longueur en pieds × pente · Pente en % = pente ÷ 12 × 100 |
| **Déport de tuyauterie** `deport` | Déport (po), Angle des raccords (11,25° · 22,5° · 30° · 45° ★ · 60°) | Longueur de parcours = déport ÷ sin θ · Avance = déport ÷ tan θ |
| **Déport roulé** `deport-roule` | Déport horizontal (po), Déport vertical (po), Angle des raccords | Déport réel = √(h² + v²) · Longueur de parcours et Avance, comme ci-dessus, à partir du déport réel |
| **Contenance d'un tuyau** `contenance-de-tuyau` | Diamètre intérieur (po), Longueur (pi) | Contenance = π ÷ 4 × d² × L · Masse de l'eau (1 kg par litre) |
| **Pression d'une colonne d'eau** `pression-d-eau` | Hauteur d'eau (pi) | Pression = hauteur en mètres × 9,806 65 kPa, aussi en lb/po² |
| **Réservoir cylindrique** `reservoir` | Diamètre intérieur (po), Hauteur ou longueur (po) | Contenance = π ÷ 4 × d² × h |
| **Débit et vitesse de l'eau** `debit-et-vitesse` | Diamètre intérieur (po), Débit, Unité du débit (gal US/min ★, L/min, L/s) | Vitesse de l'eau = débit ÷ (π ÷ 4 × d²), en pi/s et en m/s · Débit, en gal US/min et en L/min |
| **Longueur de coupe d'un tuyau** `longueur-de-coupe` | Mesure de centre à centre (po), Retrait du premier raccord (po), Retrait du second raccord (po), Emboîtement à chaque bout (po ; 0) | Longueur de coupe = centre à centre − les deux retraits + 2 × emboîtement. Résultat nul ou négatif : « Les retraits dépassent la mesure de centre à centre. » |
| **Déports parallèles** `deports-paralleles` | Déport (po), Écart entre les tuyaux, de centre à centre (po), Angle des raccords | Longueur de parcours et Avance, comme pour le déport simple · Décalage d'un tuyau au suivant = écart × tan(θ ÷ 2) |
| **Dilatation d'un tuyau** `dilatation` | Longueur du tuyau (pi), Écart de température (°C), Coefficient de dilatation (mm/m/°C ; 0,017) | Allongement = coefficient × longueur en mètres × écart, en longueur · Allongement, en millimètres |
| **Chauffage de l'eau** `chauffe-eau` | Volume d'eau (L), Élévation de température (°C), Puissance de chauffe (kW), Rendement (% ; 100) | Énergie nécessaire = litres × 4,186 × écart ÷ 3 600, en kWh · Temps de chauffe = énergie ÷ (puissance × rendement ÷ 100), en minutes et en heures. « Le rendement ne dépasse pas 100 %. » |

Les déports se mesurent de centre à centre (une note rappelle de déduire l'emboîtement des raccords). La note de la dilatation donne des coefficients courants, en millimètres par mètre et par degré Celsius — cuivre 0,017; acier 0,012; PVC 0,054; CPVC 0,065; ABS 0,10; PEX de 0,15 à 0,20 — et précise que celui du fabricant fait foi.

### 22.6 Électricité

| Calculateur | Champs | Résultats |
|---|---|---|
| **Loi d'Ohm et puissance** `loi-d-ohm` | Tension (V), Courant (A), Résistance (Ω), Puissance (W) — tous facultatifs; il en faut **exactement deux** | Les quatre grandeurs : U = R·I, P = U·I. Sinon : « Remplissez exactement deux des quatre champs. » |
| **Chute de tension** `chute-de-tension` | Tension du circuit (V ; 120), Courant (A), Longueur du parcours, aller simple (pi), Calibre (14 AWG ★ à 4/0 AWG, 250, 350, 500 kcmil), Métal (Cuivre ★, Aluminium), Circuit (Monophasé ou continu ★, Triphasé) | Chute = F × K × I × L ÷ S, avec L en pieds, S en mils circulaires, K = 12,9 (cuivre) ou 21,2 (aluminium), F = 2 en monophasé ou √3 en triphasé · Chute en % · Tension à la charge. Si la chute dépasse la tension : « La chute dépasse la tension du circuit : vérifiez les valeurs. » |
| **Puissance d'un circuit** `puissance` | Tension (V), Courant (A), Facteur de puissance (1), Circuit (Monophasé ★, Triphasé) | Puissance apparente = U × I (× √3 en triphasé) en kVA · Puissance active = apparente × facteur · Équivalent en HP = watts ÷ 746. Facteur supérieur à 1 : « Le facteur de puissance se situe entre 0 et 1. » |
| **Cintrage d'un conduit : déport** `cintrage` | Hauteur du déport (po), Angle des coudes (10° · 22,5° · 30° ★ · 45° · 60°) | Distance entre les marques = hauteur ÷ sin θ · Raccourcissement = hauteur × tan(θ ÷ 2) |
| **Coût d'utilisation** `cout-d-energie` | Puissance de l'appareil (W), Utilisation par jour (h), Nombre de jours (entier ; 30), Tarif (¢/kWh) | Énergie = W × h × jours ÷ 1 000 · Coût = énergie × tarif ÷ 100. Plus de 24 h par jour : « Une journée compte 24 heures. » |
| **Calibre selon la chute de tension** `calibre-pour-chute` | Tension du circuit (V ; 120), Courant (A), Longueur du parcours, aller simple (pi), Chute de tension visée (% ; 3), Métal, Circuit (comme la chute de tension) | Calibre suffisant = le premier de la liste dont la section atteint la section minimale · Section minimale = F × K × I × L ÷ (U × chute visée ÷ 100), en kcmil · Chute avec ce calibre, en volts et en %. Au-delà de 500 kcmil : « Aucun calibre de la liste ne suffit : il faut plus gros que 500 kcmil, ou des conducteurs en parallèle. » Chute visée de 100 % ou plus : « La chute visée doit être inférieure à 100 %. » |
| **Résistances en série et en parallèle** `resistances` | Résistance 1 et Résistance 2 (Ω), Résistance 3 et Résistance 4 (Ω, facultatives) | En série = somme · En parallèle = 1 ÷ Σ(1 ÷ R) |
| **Courants d'un transformateur** `transformateur` | Puissance (kVA), Tension au primaire (V), Tension au secondaire (V), Circuit (Monophasé ★, Triphasé) | Courant au primaire et Courant au secondaire = kVA × 1 000 ÷ U (÷ √3 en triphasé) · Rapport de transformation = primaire ÷ secondaire |
| **Courant d'un moteur** `moteur` | Puissance du moteur (HP), Tension (V), Rendement (% ; 90), Facteur de puissance (0,85), Circuit (Monophasé, Triphasé ★) | Puissance à l'arbre = HP × 0,746 kW · Puissance absorbée = à l'arbre ÷ (rendement ÷ 100) · Courant à pleine charge = absorbée ÷ (U × facteur de puissance), divisé par √3 en triphasé. « Le rendement ne dépasse pas 100 %. » |
| **Remplissage d'un conduit** `remplissage-de-conduit` | Diamètre intérieur du conduit (po), Diamètre d'un conducteur, isolant compris (po), Nombre de conducteurs (entier) | Section du conduit et Section des conducteurs = π ÷ 4 × d², en po² et en mm² · Remplissage = conducteurs ÷ conduit, en %. « Les conducteurs n'entrent pas dans ce conduit. » |
| **Éclairage : nombre de luminaires** `eclairage` | Longueur et Largeur de la pièce (pi), Éclairement visé (lux), Flux d'un luminaire (lm), Facteur d'utilisation (0,6), Facteur de maintenance (0,8) | Surface · Flux lumineux nécessaire = lux × surface en m² ÷ (utilisation × maintenance) · Luminaires = flux ÷ flux d'un luminaire, arrondi au-dessus. Un facteur supérieur à 1 : « Les deux facteurs se situent entre 0 et 1. » |

Sections en mils circulaires : 14 AWG 4 110; 12 : 6 530; 10 : 10 380; 8 : 16 510; 6 : 26 240; 4 : 41 740; 3 : 52 620; 2 : 66 360; 1 : 83 690; 1/0 : 105 600; 2/0 : 133 100; 3/0 : 167 800; 4/0 : 211 600; puis 250 000, 350 000 et 500 000. K est la résistivité du conducteur à 75 °C, en ohms-mils circulaires par pied. Le calibre selon la chute de tension ne répond qu'à la chute : une note rappelle que le Code fixe aussi le calibre minimal d'après le courant admissible. De même, le remplissage permis d'un conduit et le courant d'un moteur selon les tables du Code ne sont pas donnés.

### 22.7 Finition

| Calculateur | Champs | Résultats |
|---|---|---|
| **Peinture d'une pièce** `peinture` | Longueur et Largeur de la pièce (pi), Hauteur des murs (pi ; 8), Portes (entier ; 1), Fenêtres (entier ; 1), Surfaces à peindre (Murs seulement ★, Murs et plafond), Couches (entier ; 2), Rendement d'un contenant de 3,78 L (pi² ; 350) | Surface = 2 × (L + l) × h − 21 pi² par porte − 15 pi² par fenêtre (+ L × l avec le plafond) · Peinture = surface × couches ÷ rendement × 3,78 L · Contenants, arrondi au-dessus. Surface nulle ou négative : « Les ouvertures couvrent toute la surface des murs. » |
| **Céramique et tuiles** `ceramique` | Longueur et Largeur de la surface (pi), Longueur et Largeur d'une tuile (po), Pertes (% ; 10), Tuiles par boîte (entier, facultatif) | Surface · Tuiles = surface × (1 + pertes ÷ 100) ÷ aire d'une tuile, arrondi au-dessus · Boîtes = tuiles ÷ tuiles par boîte, arrondi au-dessus (si le champ est rempli) |
| **Revêtement de sol en boîtes** `revetement-de-sol` | Longueur et Largeur du plancher (pi), Pertes (% ; 10), Couverture d'une boîte (pi²) | Surface · Surface à couvrir, pertes comprises · Boîtes, arrondi au-dessus |
| **Peinture d'une surface** `peinture-d-une-surface` | Longueur (pi), Hauteur ou largeur (pi), Couches (entier ; 2), Rendement d'un contenant de 3,78 L (pi² ; 350) | Surface à peindre · Peinture nécessaire · Contenants de 3,78 L, comme pour une pièce |
| **Plinthes et moulures** `moulures` | Longueur et Largeur de la pièce (pi), Portes (entier ; 1), Largeur d'une porte (po ; 32), Pertes (% ; 10), Longueur d'une moulure (pi ; 8) | Tour de la pièce = 2 × (L + l) · Longueur à poser = tour − portes × largeur d'une porte · Longueur à acheter, pertes comprises · Moulures, arrondi au-dessus. « Les portes couvrent tout le tour de la pièce. » |

### 22.8 Ventilation et chauffage

| Calculateur | Champs | Résultats |
|---|---|---|
| **Débit d'un conduit rond** `conduit-rond` | Diamètre (po), Vitesse de l'air (pi/min) | Section = π ÷ 4 × d² · Débit = section en pi² × vitesse, en pi³/min et en L/s |
| **Débit d'un conduit rectangulaire** `conduit-rectangulaire` | Largeur (po), Hauteur (po), Vitesse de l'air (pi/min) | Section · Débit · Conduit rond équivalent = 1,30 × (a·b)^0,625 ÷ (a + b)^0,25 (formule de Huebscher) |
| **Puissance de chauffage et de climatisation** `puissance-thermique` | Puissance, Unité (BTU/h ★, kW, tonnes de réfrigération) | La même puissance en BTU/h, en kW et en tonnes |
| **Conduit rond pour un débit** `conduit-pour-un-debit` | Débit d'air (pi³/min), Vitesse de l'air (pi/min) | Section nécessaire = débit ÷ vitesse · Diamètre du conduit rond = √(4 × section ÷ π) |
| **Changements d'air à l'heure** `changements-d-air` | Longueur et Largeur de la pièce (pi), Hauteur du plafond (pi ; 8), Changements d'air à l'heure | Volume de la pièce, en pi³ et en m³ · Débit = volume × changements ÷ 60, en pi³/min et en L/s |
| **Chaleur transportée par l'air** `chaleur-de-l-air` | Débit d'air (pi³/min), Écart de température, Unité de l'écart (°C ★, °F) | Puissance = 1,08 × débit × écart en °F, en BTU/h et en kW; un écart de 1 °C vaut 1,8 °F. Chaleur sensible seulement. |
| **Tuyau de plancher chauffant** `plancher-chauffant` | Longueur et Largeur de la surface (pi), Espacement des tuyaux (po ; 9), Longueur maximale d'une boucle (pi ; 300) | Surface · Longueur de tuyau = surface ÷ espacement · Boucles = longueur ÷ longueur maximale, arrondi au-dessus · Longueur par boucle |

### 22.9 Terrassement et aménagement

| Calculateur | Champs | Résultats |
|---|---|---|
| **Excavation et remblai** `excavation` | Longueur, Largeur, Profondeur (pi), Foisonnement (% ; 25) | Volume en place = L × l × p · Volume à transporter = volume × (1 + foisonnement ÷ 100) |
| **Pierre, gravier et sable** `granulats` | Longueur (pi), Largeur (pi), Épaisseur (po), Masse volumique (tc/vg³ ; 1,5) | Volume · Masse = volume en vg³ × masse volumique, en tonnes courtes et en tonnes |
| **Clôture** `cloture` | Longueur totale (pi), Espacement visé des poteaux (pi ; 8) | Sections = longueur ÷ espacement, arrondi au-dessus · Poteaux = sections + 1 · Espacement réel = longueur ÷ sections |
| **Tranchée** `tranchee` | Longueur (pi), Largeur au fond (pi), Largeur en haut (pi, facultative), Profondeur (pi), Foisonnement (% ; 25) | Volume en place = L × (fond + haut) ÷ 2 × profondeur; le haut vaut le fond si le champ est vide · Volume à transporter, foisonné. « La largeur en haut ne peut pas être plus petite que la largeur au fond. » |
| **Voyages de camion** `voyages-de-camion` | Volume à transporter (vg³), Capacité du camion (vg³) | Voyages = volume ÷ capacité, arrondi au-dessus · Charge moyenne d'un voyage |
| **Pente et dénivelé** `pente-de-terrain` | Distance horizontale (pi), Dénivelé (po), Pente (%) — tous facultatifs; il en faut **exactement deux** | Distance horizontale · Dénivelé · Pente = dénivelé ÷ distance × 100 · Angle = arctan(pente ÷ 100) · Longueur en pente = √(distance² + dénivelé²). Sinon : « Remplissez exactement deux des trois champs. » |
| **Terrasse : planches et vis** `terrasse` | Longueur, dans le sens des planches (pi), Largeur de la terrasse (pi), Largeur d'une planche (po ; 5 1/2), Jeu entre les planches (po ; 1/8), Espacement des solives (po ; 16), Pertes (% ; 10) | Surface · Rangs de planches = largeur ÷ (planche + jeu), arrondi au-dessus · Longueur de planches = rangs × longueur · Longueur à acheter, pertes comprises · Vis = rangs × solives × 2, avec solives = ⌈longueur ÷ espacement⌉ + 1 |
| **Tourbe en rouleaux** `tourbe` | Longueur (pi), Largeur (pi), Surface d'un rouleau (pi² ; 10), Pertes (% ; 5) | Surface · Rouleaux = surface × (1 + pertes ÷ 100) ÷ surface d'un rouleau, arrondi au-dessus |
| **Béton des poteaux** `poteaux` | Diamètre du trou (po), Profondeur du trou (pi), Poteau (Carré ★, Rond), Côté ou diamètre du poteau (po ; zéro permis), Nombre de poteaux (entier ; 1), Rendement d'un sac (L ; 14) | Volume = (section du trou − section du poteau) × profondeur × nombre · Volume, en litres · Sacs, arrondi au-dessus. « Le poteau n'entre pas dans le trou. » |

La note de la tranchée rappelle que l'inclinaison des parois est une question de sécurité (Code de sécurité pour les travaux de construction), sans donner de valeur.

### 22.10 Maçonnerie et pavage

| Calculateur | Champs | Résultats |
|---|---|---|
| **Briques et blocs** `briques-et-blocs` | Longueur du mur (pi), Hauteur du mur (pi), Ouvertures à déduire (pi² ; 0), Longueur d'une unité (po), Hauteur d'une unité (po), Joint de mortier (po ; 3/8), Pertes (% ; 5) | Surface du mur, ouvertures déduites · Unités au pied carré = 144 ÷ [(longueur + joint) × (hauteur + joint)], à trois décimales · Unités nécessaires, pertes comprises, arrondi au-dessus. « Les ouvertures couvrent toute la surface du mur. » |
| **Rangs de maçonnerie** `rangs-de-maconnerie` | Hauteur à atteindre (pi), Hauteur d'une unité (po), Joint de mortier (po ; 3/8) | Hauteur d'un rang, joint compris · Rangs = hauteur ÷ rang, arrondi au-dessus · Hauteur obtenue = rangs × rang |
| **Muret de blocs** `muret` | Longueur du muret (pi), Hauteur du muret (pi), Longueur d'un bloc (po), Hauteur d'un bloc (po) | Blocs par rang et Rangs, arrondis au-dessus · Blocs = les deux multipliés · Hauteur obtenue |
| **Pavés et dalles** `paves` | Longueur (pi), Largeur (pi), Longueur d'un pavé (po), Largeur d'un pavé (po), Pertes (% ; 5), Épaisseur du lit de sable (po ; 1) | Surface · Pavés = surface × (1 + pertes ÷ 100) ÷ aire d'un pavé, arrondi au-dessus · Sable du lit de pose = surface × épaisseur |

Une note donne la mesure d'un bloc de béton courant, sans le joint : 15 5/8 po sur 7 5/8 po (390 mm sur 190 mm). Le muret se pose à sec, sans joint; sa note renvoie aux directives du fabricant pour un muret de soutènement.

### 22.11 Toiture et revêtement

| Calculateur | Champs | Résultats |
|---|---|---|
| **Bardeaux d'asphalte** `bardeaux` | Longueur du toit (pi), Largeur au sol, débords compris (pi), Pente du toit (sur 12 ; zéro permis), Pertes (% ; 10), Couverture d'un paquet (pi² ; 33,3) | Surface du toit = longueur × largeur × √(1 + (pente ÷ 12)²) · Carrés de 100 pi², pertes comprises · Paquets de bardeaux, arrondi au-dessus |
| **Pente d'un toit** `pente-de-toit` | Pente, Unité (sur 12 ★, degrés, pour cent) | Pente, sur 12 · Angle · Pente, en pourcentage · Facteur de pente = √(1 + rapport²), où le rapport est la montée divisée par la course. 90° ou plus : « L'angle doit être inférieur à 90°. » |
| **Panneaux de contreplaqué ou d'OSB** `panneaux` | Longueur (pi), Largeur ou hauteur (pi), Panneau (4 pi × 8 pi ★, 4 pi × 9 pi, 4 pi × 10 pi), Pertes (% ; 10) | Surface · Panneaux, arrondi au-dessus |
| **Revêtement extérieur** `revetement-exterieur` | Longueur des murs (pi), Hauteur des murs (pi), Ouvertures à déduire (pi² ; 0), Pertes (% ; 10), Couverture d'une boîte (pi²) | Surface à couvrir · Surface à acheter, pertes comprises · Boîtes, arrondi au-dessus. « Les ouvertures couvrent toute la surface des murs. » |
| **Membrane et papier en rouleaux** `membrane` | Longueur et Largeur de la surface (pi), Largeur d'un rouleau (po ; 36), Longueur d'un rouleau (pi), Chevauchement (po ; zéro permis), Pertes (% ; 10) | Surface · Rouleaux = surface × (1 + pertes ÷ 100) ÷ [(largeur du rouleau − chevauchement) × longueur du rouleau], arrondi au-dessus. « Le chevauchement dépasse la largeur du rouleau. » |

Les bardeaux supposent un toit à deux versants de même pente; la note rappelle d'ajouter les bardeaux de départ et de faîte.

### 22.12 Gypse et isolation

| Calculateur | Champs | Résultats |
|---|---|---|
| **Panneaux de gypse d'une pièce** `gypse` | Longueur et Largeur de la pièce (pi), Hauteur des murs (pi ; 8), Portes (entier ; 1), Fenêtres (entier ; 1), Surfaces à couvrir (Murs seulement, Murs et plafond ★), Panneau (4 pi × 8 pi ★, 4 pi × 10 pi, 4 pi × 12 pi), Pertes (% ; 10) | Surface à couvrir, calculée comme pour la peinture d'une pièce · Panneaux = surface × (1 + pertes ÷ 100) ÷ aire d'un panneau, arrondi au-dessus |
| **Montants d'une cloison** `montants-de-cloison` | Longueur de la cloison (pi), Espacement des montants (po ; 16), Montants en plus (entier ; 0), Lisses (Une en bas, une en haut ★ · Une en bas, deux en haut) | Montants = ⌈longueur ÷ espacement⌉ + 1 + montants en plus · Longueur de lisses = longueur × 2 ou × 3 |
| **Isolant en nattes ou en panneaux** `isolant` | Longueur (pi), Hauteur ou largeur (pi), Pertes (% ; 5), Couverture d'un paquet (pi²) | Surface · Surface à couvrir, pertes comprises · Paquets, arrondi au-dessus |
| **Valeur R et valeur RSI** `valeur-r` | Résistance thermique, Unité (R (impérial) ★, RSI (métrique)) | Valeur R = RSI × 5,678 263, à une décimale · Valeur RSI, à deux décimales · Coefficient U = 1 ÷ RSI, en W/m²·K, à trois décimales |
| **Plafond suspendu** `plafond-suspendu` | Longueur et Largeur de la pièce (pi), Tuile (2 pi × 4 pi ★, 2 pi × 2 pi), Pertes (% ; 10) | Surface · Tuiles, arrondi au-dessus · Cornière murale = tour de la pièce |

### 22.13 Acier et soudure

| Calculateur | Champs | Résultats |
|---|---|---|
| **Masse d'une plaque** `plaque` | Longueur (pi), Largeur (pi), Épaisseur (po), Métal (Acier ★, Acier inoxydable, Aluminium), Nombre de pièces (entier ; 1) | Surface d'une face · Masse d'une pièce = volume × masse volumique · Masse totale |
| **Masse d'une barre ou d'un tube rond** `barre-ronde` | Diamètre extérieur (po), Épaisseur de la paroi (po, facultative : vide pour une barre pleine), Longueur (pi), Métal, Nombre de pièces | Masse linéique, en lb/pi et en kg/m · Masse d'une pièce · Masse totale. Section = π ÷ 4 × (D² − d²), avec d = D − 2 × paroi. « La paroi dépasse la moitié du diamètre. » |
| **Masse d'une barre plate ou d'un tube rectangulaire** `profile-rectangulaire` | Largeur (po), Hauteur (po), Épaisseur de la paroi (po, facultative), Longueur (pi), Métal, Nombre de pièces | Les mêmes trois lignes. Section = l × h − (l − 2e) × (h − 2e), coins vifs. « La paroi dépasse la moitié du plus petit côté. » |
| **Cordon de soudure d'angle** `cordon-de-soudure` | Côté du cordon (po), Longueur du cordon (pi), Rendement de dépôt (% ; 65) | Métal déposé = côté² ÷ 2 × longueur × masse volumique de l'acier · Électrodes ou fil à prévoir = métal déposé ÷ (rendement ÷ 100). « Le rendement ne dépasse pas 100 %. » |

Masses volumiques : acier 7 850 kg/m³; acier inoxydable 8 000 kg/m³; aluminium 2 700 kg/m³. La note du cordon donne des ordres de grandeur du rendement de dépôt (environ 60 à 65 % à l'électrode enrobée, plus de 90 % au fil plein) et rappelle de prévoir un surplus pour le bombé.

### 22.14 Valeurs d'essai

Calculées à part, hors du programme (comme les vecteurs de la section 19). Préférences par défaut. Le premier tableau est rejoué par `noyau/tests/metiers.rs`, le second par `noyau/tests/metiers_suite.rs`.

| Calculateur | Saisies | Résultats attendus |
|---|---|---|
| soumission | 100 000 · 3 · 12 · 15 · 5 · 9,975 | `3 000,00 $` · `12 000,00 $` · `15 000,00 $` · `130 000,00 $` · `6 500,00 $` · `12 967,50 $` · `149 467,50 $` |
| soumission | 251 126,40 · 3 · 12 · 15 · 5 · 9,975 | `7 533,79 $` · `30 135,17 $` · `37 668,96 $` · `326 464,32 $` · `16 323,22 $` · `32 564,82 $` · `375 352,36 $` |
| budget-au-pied-carre | 1 400 · 200 · 400 · 100 · 3 · 12 · 15 · 5 · 9,975 | `320 000,00 $` · `96 000,00 $` · `416 000,00 $` · `62 296,00 $` · `478 296,00 $` · `265,72 $/pi²` |
| main-d-oeuvre | 3 · 40 · 45,50 · 30 | `120,0 h` · `59,15 $/h` · `7 098,00 $` |
| taxes | 1 000 · avant taxes · 5 · 9,975 | `1 000,00 $` · `50,00 $` · `99,75 $` · `1 149,75 $` |
| taxes | 500 · taxes comprises · 5 · 9,975 | `434,88 $` · `21,74 $` · `43,38 $` · `500,00 $` |
| majoration-et-marge | 800 · 25 | `1 000,00 $` · `200,00 $` · `20,00 %` |
| dalle | 24 · 20 · 4 · 10 | `480 pi² (44,6 m²)` · `5,93 vg³ (4,53 m³)` · `6,52 vg³ (4,98 m³)` |
| dalle | 7,3 m · 6 m · 100 mm · 10 | `43,8 m² (471 pi²)` · `4,38 m³ (5,73 vg³)` · `4,82 m³ (6,30 vg³)` |
| semelle | 120 · 20 · 8 · 10 | `4,94 vg³ (3,78 m³)` · `5,43 vg³ (4,15 m³)` |
| mur-de-fondation | 120 · 8 · 8 · 10 | `23,7 vg³ (18,1 m³)` · `26,1 vg³ (19,9 m³)` · `1 920 pi² (178 m²)` |
| colonnes | 10 · 4 · 6 · 10 · 14 | `0,485 vg³ (0,371 m³)` · `0,533 vg³ (0,408 m³)` · `30 sacs` |
| armature | 24 · 20 · 16 · 10M | `16 barres` · `19 barres` · `764 pi 0 po` · `403 lb (183 kg)` |
| pente-de-drain | 40 · 1/4 | `10 po` · `2,08 %` |
| pente-de-drain | 12 m · 1/4 | `0,250 m` · `2,08 %` |
| deport | 12 · 45° | `1 pi 5 po` · `1 pi 0 po` |
| deport | 12 · 22,5° | `2 pi 7 3/8 po` · `2 pi 5 po` |
| deport-roule | 8 · 6 · 45° | `10 po` · `1 pi 2 1/8 po` · `10 po` |
| contenance-de-tuyau | 4 · 10 | `24,7 L (6,53 gal US)` · `54,5 lb (24,7 kg)` |
| pression-d-eau | 30 | `89,7 kPa (13,0 lb/po²)` |
| reservoir | 20 · 48 | `247 L (65,3 gal US)` |
| loi-d-ohm | 120 V · 15 A | `120 V` · `15,0 A` · `8,00 Ω` · `1 800 W` |
| chute-de-tension | 120 · 15 · 100 · 14 AWG · cuivre · monophasé | `9,42 V` · `7,85 %` · `110,6 V` |
| chute-de-tension | 600 · 100 · 300 · 1/0 AWG · aluminium · triphasé | `10,4 V` · `1,74 %` · `589,6 V` |
| puissance | 600 · 20 · 0,85 · triphasé | `20,8 kVA` · `17,7 kW` · `23,7 HP` |
| cintrage | 6 · 30° | `1 pi 0 po` · `1 5/8 po` |
| cout-d-energie | 1 500 · 8 · 30 · 10 | `360 kWh` · `36,00 $` |
| peinture | 12 · 10 · 8 · 1 · 1 · murs · 2 · 350 | `316 pi² (29,4 m²)` · `6,83 L` · `2 contenants` |
| ceramique | 10 · 8 · 24 · 12 · 10 · 8 | `80,0 pi² (7,43 m²)` · `44 tuiles` · `6 boîtes` |
| revetement-de-sol | 15 · 12 · 10 · 22,5 | `180 pi² (16,7 m²)` · `198 pi² (18,4 m²)` · `9 boîtes` |
| conduit-rond | 8 · 700 | `0,349 pi² (0,032 m²)` · `244 pi³/min (115 L/s)` |
| conduit-rectangulaire | 14 · 8 · 900 | `0,778 pi² (0,072 m²)` · `700 pi³/min (330 L/s)` · `11 7/16 po` |
| puissance-thermique | 36 000 · BTU/h | `36 000 BTU/h` · `10,6 kW` · `3,00 tonnes` |
| excavation | 30 · 20 · 4 · 25 | `88,9 vg³ (68,0 m³)` · `111 vg³ (85,0 m³)` |
| granulats | 40 · 12 · 6 · 1,5 | `8,89 vg³ (6,80 m³)` · `13,3 tc (12,1 t)` |
| cloture | 100 · 8 | `13 sections` · `14 poteaux` · `7 pi 8 5/16 po` |

Calculateurs ajoutés à la version 0.4 (un tiret : champ laissé vide) :

| Calculateur | Saisies | Résultats attendus |
|---|---|---|
| calibre-pour-chute | 120 · 20 · 100 · 3 · cuivre · monophasé | `8 AWG` · `14,3 kcmil` · `3,13 V` · `2,60 %` |
| calibre-pour-chute | 600 · 100 · 300 · 2 · aluminium · triphasé | `1/0 AWG` · `91,8 kcmil` · `10,4 V` · `1,74 %` |
| resistances | 100 · 220 · 470 · — | `790 Ω` · `60,0 Ω` |
| resistances | 8 · 8 · — · — | `16,0 Ω` · `4,00 Ω` |
| transformateur | 75 · 600 · 208 · triphasé | `72,2 A` · `208 A` · `2,88` |
| transformateur | 25 · 240 · 120 · monophasé | `104 A` · `208 A` · `2,00` |
| moteur | 10 · 575 · 90 · 0,85 · triphasé | `7,46 kW` · `8,29 kW` · `9,79 A` |
| moteur | 1 · 120 · 75 · 0,8 · monophasé | `0,746 kW` · `0,995 kW` · `10,4 A` |
| remplissage-de-conduit | 0,824 · 0,111 · 9 | `0,533 po² (344 mm²)` · `0,087 po² (56,2 mm²)` · `16,3 %` |
| eclairage | 30 · 20 · 300 · 4000 · 0,6 · 0,8 | `600 pi² (55,7 m²)` · `34 839 lm` · `9 luminaires` |
| debit-et-vitesse | 0,785 · 8 · gal US/min | `5,30 pi/s (1,62 m/s)` · `8,00 gal US/min (30,3 L/min)` |
| debit-et-vitesse | 1 · 20 · L/min | `2,16 pi/s (0,658 m/s)` · `5,28 gal US/min (20,0 L/min)` |
| debit-et-vitesse | 3/4 · 0,5 · L/s | `5,76 pi/s (1,75 m/s)` · `7,93 gal US/min (30,0 L/min)` |
| longueur-de-coupe | 48 · 1 1/8 · 1 1/8 · 3/4 | `3 pi 11 1/4 po` |
| longueur-de-coupe | 30 1/2 · 1 3/4 · 2 1/4 · 0 | `2 pi 2 1/2 po` |
| deports-paralleles | 12 · 6 · 45° | `1 pi 5 po` · `1 pi 0 po` · `2 1/2 po` |
| deports-paralleles | 10 · 8 · 22,5° | `2 pi 2 1/8 po` · `2 pi 0 1/8 po` · `1 9/16 po` |
| dilatation | 50 · 60 · 0,017 | `5/8 po` · `15,5 mm` |
| dilatation | 30 m · 50 · 0,054 | `0,081 m` · `81,0 mm` |
| chauffe-eau | 180 · 45 · 4,5 · 100 | `9,42 kWh` · `126 min` · `2,09 h` |
| chauffe-eau | 270 · 50 · 11 · 80 | `15,7 kWh` · `107 min` · `1,78 h` |
| sacs-de-beton | 4 · 4 · 4 · 10 · 14 | `0,198 vg³ (0,151 m³)` · `166 L` · `12 sacs` |
| escalier-de-beton | 5 · 7 · 11 · 4 · 5 · 10 | `0,533 vg³ (0,408 m³)` · `0,587 vg³ (0,449 m³)` |
| escalier-de-beton | 3 · 7 1/2 · 10 · 3 · 0 · 0 | `0,087 vg³ (0,066 m³)` · `0,087 vg³ (0,066 m³)` |
| treillis | 24 · 20 · 8 · 4 · 6 | `480 pi² (44,6 m²)` · `19 feuilles` |
| peinture-d-une-surface | 40 · 8 · 2 · 350 | `320 pi² (29,7 m²)` · `6,91 L` · `2 contenants` |
| moulures | 12 · 10 · 1 · 32 · 10 · 8 | `44 pi 0 po` · `41 pi 4 po` · `45 pi 5 5/8 po` · `6 moulures` |
| gypse | 12 · 10 · 8 · 1 · 1 · murs et plafond · 4 × 8 · 10 | `436 pi² (40,5 m²)` · `15 panneaux` |
| gypse | 12 · 10 · 8 · 1 · 1 · murs · 4 × 12 · 10 | `316 pi² (29,4 m²)` · `8 panneaux` |
| montants-de-cloison | 12 · 16 · 0 · trois lisses | `10 montants` · `36 pi 0 po` |
| montants-de-cloison | 12 pi 6 po · 16 · 2 · deux lisses | `13 montants` · `25 pi 0 po` |
| isolant | 40 · 8 · 5 · 78,3 | `320 pi² (29,7 m²)` · `336 pi² (31,2 m²)` · `5 paquets` |
| valeur-r | 24,5 · R | `24,5` · `4,31` · `0,232 W/m²·K` |
| valeur-r | 3,52 · RSI | `20,0` · `3,52` · `0,284 W/m²·K` |
| plafond-suspendu | 20 · 15 · 2 × 4 · 10 | `300 pi² (27,9 m²)` · `42 tuiles` · `70 pi 0 po` |
| plafond-suspendu | 20 · 15 · 2 × 2 · 5 | `300 pi² (27,9 m²)` · `79 tuiles` · `70 pi 0 po` |
| bardeaux | 40 · 28 · 6 · 10 · 33,3 | `1 252 pi² (116 m²)` · `13,8 carrés` · `42 paquets` |
| bardeaux | 40 · 28 · 0 · 0 · 33,3 | `1 120 pi² (104 m²)` · `11,2 carrés` · `34 paquets` |
| pente-de-toit | 6 · sur 12 | `6,00 sur 12` · `26,57°` · `50,0 %` · `1,1180` |
| pente-de-toit | 30 · degrés | `6,93 sur 12` · `30,00°` · `57,7 %` · `1,1547` |
| pente-de-toit | 75 · pour cent | `9,00 sur 12` · `36,87°` · `75,0 %` · `1,2500` |
| panneaux | 24 · 16 · 4 × 8 · 10 | `384 pi² (35,7 m²)` · `14 panneaux` |
| revetement-exterieur | 120 · 9 · 150 · 10 · 200 | `930 pi² (86,4 m²)` · `1 023 pi² (95,0 m²)` · `6 boîtes` |
| membrane | 40 · 14 · 36 · 65 · 4 · 10 | `560 pi² (52,0 m²)` · `4 rouleaux` |
| briques-et-blocs | 22 · 8 · 0 · 15 5/8 · 7 5/8 · 3/8 · 5 | `176 pi² (16,4 m²)` · `1,125` · `208 unités` |
| briques-et-blocs | 24 · 9 · 40 · 7 5/8 · 2 1/4 · 3/8 · 5 | `176 pi² (16,4 m²)` · `6,857` · `1 268 unités` |
| rangs-de-maconnerie | 8 pi 1 po · 7 5/8 · 3/8 | `8 po` · `13 rangs` · `8 pi 8 po` |
| rangs-de-maconnerie | 8 · 2 1/4 · 3/8 | `2 5/8 po` · `37 rangs` · `8 pi 1 1/8 po` |
| muret | 21 pi 6 po · 40 po · 12 · 6 | `22 blocs` · `7 rangs` · `154 blocs` · `3 pi 6 po` |
| paves | 20 · 12 · 7 7/8 · 3 15/16 · 5 · 1 | `240 pi² (22,3 m²)` · `1 171 pavés` · `0,741 vg³ (0,566 m³)` |
| plaque | 8 · 4 · 1/4 · acier · 1 | `32,0 pi² (2,97 m²)` · `327 lb (148 kg)` · `327 lb (148 kg)` |
| plaque | 8 · 4 · 1/4 · aluminium · 3 | `32,0 pi² (2,97 m²)` · `112 lb (51,0 kg)` · `337 lb (153 kg)` |
| barre-ronde | 1 · — · 12 · acier · 1 | `2,67 lb/pi (3,98 kg/m)` · `32,1 lb (14,5 kg)` · `32,1 lb (14,5 kg)` |
| barre-ronde | 2 · 1/8 · 20 · acier · 2 | `2,51 lb/pi (3,73 kg/m)` · `50,1 lb (22,7 kg)` · `100 lb (45,5 kg)` |
| profile-rectangulaire | 2 · 2 · 1/8 · 20 · acier · 1 | `3,19 lb/pi (4,75 kg/m)` · `63,8 lb (28,9 kg)` · `63,8 lb (28,9 kg)` |
| profile-rectangulaire | 2 · 1/4 · — · 10 · acier · 4 | `1,70 lb/pi (2,53 kg/m)` · `17,0 lb (7,72 kg)` · `68,1 lb (30,9 kg)` |
| cordon-de-soudure | 1/4 · 100 · 65 | `10,6 lb (4,82 kg)` · `16,4 lb (7,42 kg)` |
| conduit-pour-un-debit | 400 · 800 | `0,500 pi² (0,046 m²)` · `9 9/16 po` |
| changements-d-air | 12 · 10 · 8 · 6 | `960 pi³ (27,2 m³)` · `96,0 pi³/min (45,3 L/s)` |
| chaleur-de-l-air | 1000 · 20 · °F | `21 600 BTU/h (6,33 kW)` |
| chaleur-de-l-air | 1000 · 11 · °C | `21 384 BTU/h (6,27 kW)` |
| plancher-chauffant | 30 · 20 · 9 · 300 | `600 pi² (55,7 m²)` · `800 pi 0 po` · `3 boucles` · `266 pi 8 po` |
| tranchee | 100 · 2 · 6 · 4 · 25 | `59,3 vg³ (45,3 m³)` · `74,1 vg³ (56,6 m³)` |
| tranchee | 100 · 2 · — · 4 · 25 | `29,6 vg³ (22,7 m³)` · `37,0 vg³ (28,3 m³)` |
| voyages-de-camion | 74 · 12 | `7 voyages` · `10,6 vg³` |
| pente-de-terrain | 50 · — · 2 | `50 pi 0 po` · `1 pi 0 po` · `2,00 %` · `1,15°` · `50 pi 0 1/8 po` |
| pente-de-terrain | 50 · 9 · — | `50 pi 0 po` · `9 po` · `1,50 %` · `0,86°` · `50 pi 0 1/16 po` |
| pente-de-terrain | — · 6 · 1 | `50 pi 0 po` · `6 po` · `1,00 %` · `0,57°` · `50 pi 0 po` |
| terrasse | 16 · 12 · 5 1/2 · 1/8 · 16 · 10 | `192 pi² (17,8 m²)` · `26 rangs` · `416 pi 0 po` · `457 pi 7 3/16 po` · `676 vis` |
| tourbe | 50 · 30 · 10 · 5 | `1 500 pi² (139 m²)` · `158 rouleaux` |
| poteaux | 10 · 4 · carré · 3 1/2 · 12 · 14 | `0,818 vg³ (0,626 m³)` · `626 L` · `45 sacs` |
| poteaux | 12 · 4 · rond · 0 · 4 · 14 | `0,465 vg³ (0,356 m³)` · `356 L` · `26 sacs` |
| facturation-progressive | 100000 · 40 · 10 · 20000 · 5 · 9,975 | `40 000,00 $` · `4 000,00 $` · `16 000,00 $` · `800,00 $` · `1 596,00 $` · `18 396,00 $` |
| facturation-progressive | 87450,75 · 65 · 10 · 30000 · 5 · 9,975 | `56 842,99 $` · `5 684,30 $` · `21 158,69 $` · `1 057,93 $` · `2 110,58 $` · `24 327,20 $` |
| duree-des-travaux | 1200 · 50 · 2 · 8 | `24,0 h` · `12,0 h` · `1,5 jours` |
| duree-des-travaux | 350 · 12 · 3 · 7,5 | `29,2 h` · `9,7 h` · `1,3 jours` |

### 22.15 Écran

Onglet **Métiers** de la barre inférieure (pictogramme : un casque de chantier), aussi dans le menu ☰. Trois niveaux :

1. **Les métiers** : une carte par métier (nom, résumé, nombre de calculateurs), puis une carte « Charpente, toiture et escaliers » qui ramène au clavier de base, puis un rappel : ces calculateurs ne remplacent ni les plans ni les codes en vigueur.
2. **Les calculateurs d'un métier** : une carte par calculateur (nom, résumé). Bouton `Retour`.
3. **Le formulaire** : le résumé; les champs (libellé en gras, champ de saisie, unité à droite, message de refus dessous); les résultats (libellé en petit, valeur en gras); la note du calculateur; les boutons `Ajouter au ruban` (actif quand il y a des résultats), `Au clavier` (branche le calculateur sur le clavier de l'accueil, section 23) et `Recommencer`. Bouton `Retour`.

`Échap` quitte le champ en cours de saisie, puis remonte d'un niveau, puis ramène à l'Accueil. Dans une fenêtre large, le ruban reste visible à droite : les résultats ajoutés s'y inscrivent sous les yeux de l'utilisateur.

---

## 23. Clavier adapté au métier

Ajouté le 2026-11-19 (version 0.4). L'utilisateur s'attendait à ce que la calculatrice elle-même « s'adapte selon le métier », et pas seulement à des formulaires rangés dans un onglet. Le clavier de l'accueil prend donc les touches du métier choisi : un plombier y trouve ses déports et ses longueurs de coupe, un électricien sa chute de tension. Tout se tape avec les touches de la calculatrice (`Pieds`, `Pouces`, fractions), sans clavier d'ordinateur ni clavier de téléphone.

### 23.1 Principe

- Un **outil** est un calculateur de la section 22 **branché sur le clavier**. Chacun de ses champs devient une touche; une touche `Résultat` donne ses résultats, l'un après l'autre, au grand afficheur.
- **R-55** Le formulaire et le clavier sont deux façons de remplir le **même** calculateur : ils partagent les mêmes saisies et donnent toujours les mêmes résultats. Ce qui est rangé au clavier se retrouve dans le formulaire, et inversement.
- **R-56** Le calcul reste dans `noyau` (R-49) : l'application ne fait que transmettre des touches (`Touche::Outil`) et dessiner ce que le moteur retourne.
- Sans outil branché, le clavier est celui de la section 5 et rien d'autre ne change.

### 23.2 Barre des métiers

Entre la bande de fractions rapides et le clavier, sur 34 points de haut :

1. **La pastille du métier** (pleine, avec un chevron) : « Charpente » au clavier de base, sinon le nom en un mot du métier de l'outil branché. Elle ouvre la liste **Votre métier** : une carte « Charpente, toiture et escaliers » (le clavier de base), puis une carte par métier, avec son résumé; le métier en cours porte la mention « en cours ».
2. **Une rangée de pastilles, qui défile de côté** (au doigt, à la souris ou à la molette) :
   - au clavier de base, **les métiers** : un toucher adapte le clavier;
   - avec un métier, **ses outils** : celui du clavier est mis en évidence et amené en vue; un toucher branche un autre outil.

Choisir un métier branche le dernier outil de ce métier utilisé pendant la séance, sinon le premier de sa liste. L'outil branché est conservé (`metiers.json`, 17.2) et repris à la réouverture. Choisir « Charpente » débranche l'outil.

### 23.3 Disposition du clavier

Avec un outil de *n* champs (dix au plus, 22.1), de haut en bas :

| Rangée | Contenu |
|---|---|
| Champs | *n* ≤ 5 : une rangée; au-delà, deux rangées équilibrées (⌈*n* ÷ 2⌉ puis ⌊*n* ÷ 2⌋). Les touches d'une rangée se partagent toute la largeur. |
| Résultat | `m` · `Résultat` (large de trois colonnes) · `%` |
| Le reste | les cinq rangées du bas du clavier de base, inchangées : de `Verges` `Pieds` `Pouces` `/` `Effacer` à `M+` `0` `,` `=` `+` |

Le clavier compte donc sept ou huit rangées. Les deux rangées de charpente et les touches `Longueur`, `Largeur`, `Hauteur` ne sont pas montrées : un outil a souvent ses propres champs « Longueur » et « Largeur », et deux touches du même nom prêteraient à confusion. Leurs raccourcis clavier (16.5) sont alors sans effet.

**Touche d'un champ.** Son libellé est le libellé complet du champ s'il tient sur la touche, sinon son libellé court (22.1). Une touche bleue reçoit une valeur; une touche foncée fait défiler les options d'une liste. Au-dessus de la touche, là où le clavier de base écrit les fonctions secondaires : le **contenu du champ**, en gras; si le champ est vide, l'**unité** d'un nombre tapé sans unité, entre parenthèses — `(pi)`, `(po)`, `(A)`, `($)`. Un contenu trop long rapetisse, puis est coupé par des points de suspension.

**Touche `Résultat`.** De la couleur de `Conv`, pour se détacher. Son étiquette secondaire est `Recommencer`.

### 23.4 Touches des champs

- **R-57** Une valeur tapée ou calculée, puis la touche d'un champ : la valeur est **rangée** dans le champ. L'afficheur montre le libellé du champ, la valeur et l'indicateur `MÉM.`.
  - Une **longueur** est rangée telle que l'afficheur l'écrit (`24 pi 0 po`, `7,300 m`) : à la résolution des fractions, donc.
  - Un **nombre** est rangé avec toutes ses décimales (neuf au plus), sans séparateur de milliers : `1234567,89` garde ses cents, même si l'afficheur l'arrondit.
  - Un **nombre seul** prend l'unité du champ : `24` sur un champ en pieds donne `24 pi 0 po`.
  - Une **aire**, un **volume** ou une **masse** va dans un champ de la même unité : `40 pi × 35 pi =` puis `Superficie` (en pi²) range `1400`. Toute autre combinaison — une longueur dans un champ de nombre, des mètres carrés dans un champ en pieds carrés — donne `ERREUR – unités incompatibles`.
  - Une valeur que le champ refuse (zéro, valeur négative, nombre non entier) donne `ERREUR – saisie invalide`; le champ garde son contenu.
- La touche d'un champ, **sans rien taper** : elle **rappelle** son contenu. Champ vide : `AUCUNE VALEUR`, sans bloquer.
- Une touche de **liste** passe à l'option suivante à chaque pression; l'afficheur montre l'option et son rang (`2 de 5`).
- **R-58** `Conv` puis la touche d'un champ le remet à sa **valeur de départ**. S'il n'en a pas, il est vidé : `CHAMP VIDÉ`.

### 23.5 Touche `Résultat`

- Elle calcule. À la première pression, le calcul est **inscrit au ruban** — le même bloc que le bouton `Ajouter au ruban` du formulaire (22.2) — et le premier résultat s'affiche, avec son rang : `VOLUME DE BÉTON · 2 de 3`. Chaque nouvelle pression montre le résultat suivant, puis revient au premier.
- Toute autre touche met fin au défilement : la pression suivante sur `Résultat` recalcule et inscrit un nouveau bloc.
- **Il manque une valeur** : `IL MANQUE : LARGEUR` nomme le premier champ obligatoire resté vide. Ce n'est pas une erreur : la touche suivante est acceptée.
- **Les valeurs ne se prêtent pas au calcul** : le message du calculateur s'affiche tel quel (« Remplissez exactement deux des quatre champs. »).
- **Une saisie du formulaire est refusée** : `À CORRIGER : LONGUEUR`.
- **R-59** Un résultat affiché est une **valeur** : `×` `2` `=`, `M+` et `Stocker` s'en servent. Une longueur reste une longueur, à sa pleine précision; une aire est en pi² ou en m², un volume en vg³ ou en m³, une masse en lb ou en kg, selon le système de la saisie; les autres résultats sont des nombres (la première de deux écritures). Un mot (`8 AWG`) n'a pas de valeur.
- `Conv` `Résultat` — ou un toucher sur l'étiquette `Recommencer` — remet **tous** les champs à leur valeur de départ : `CHAMPS REMIS AU DÉPART`. `Effacer tout` le fait aussi; `Effacer`, non.

### 23.6 Changer d'outil

- **R-60** La valeur affichée **suit** d'un outil à l'autre, comme une valeur ordinaire, sans le libellé de l'outil quitté : le volume foisonné d'une tranchée (`74,1 vg³`) se range d'une pression dans le champ « Volume à transporter » des voyages de camion; les mesures d'une dalle servent à son armature.
- Un message de l'outil quitté, une option de liste ou un mot ne suivent pas : l'afficheur revient à `0`.
- Revenir à l'accueil reprend ce que le formulaire contient : une saisie modifiée dans l'onglet Métiers se retrouve au clavier.

### 23.7 Aide

- **Appui long** ou **clic droit** sur une touche de l'outil : son mode d'emploi — nom et résumé du calculateur, cinq points « Au clavier », la note du calculateur, et les boutons `Formulaire` (ouvre ce calculateur dans l'onglet Métiers) et `Fermer`.

  > 1. **Rangez une valeur** : tapez-la, puis appuyez sur la touche du champ. Un nombre seul prend l'unité montrée entre parenthèses au-dessus de la touche.
  > 2. **Une touche foncée** fait défiler les options d'une liste.
  > 3. **La touche Résultat** calcule et inscrit le calcul au ruban. Appuyez de nouveau pour voir le résultat suivant.
  > 4. **Rappelez ou effacez** : la touche d'un champ, sans rien taper, rappelle sa valeur; `Conv` puis la touche la remet à sa valeur de départ.
  > 5. **Poursuivez le calcul** : un résultat affiché sert à la suite (`×` `2` `=`, `M+`) et vous suit quand vous changez d'outil.

- **Infobulles** : le nom complet du champ, son contenu et son mode d'emploi; celle de `Résultat` rappelle le défilement et `Conv`.
- La fiche « Clavier adapté au métier » de l'écran Aide (annexe B) et le cinquième point de l'écran Bienvenue (B.3) présentent la barre des métiers.

### 23.8 Réalisation

| Où | Quoi |
|---|---|
| `noyau/src/touches.rs` | `Touche::Outil(u8)` : le rang d'un champ (0 à 9) ou celui de `Résultat` (10) |
| `noyau/src/moteur/outils.rs` | `CHAMPS_D_OUTIL` (10), `TOUCHE_RESULTAT` (10); `Moteur::choisir_outil(id, saisies)`, `outil()`, `saisies_d_outil()`, `touches_d_outil()` → `ToucheOutil { libelle, nom, valeur, unite, choix, facultatif }`; le traitement des touches |
| `noyau/src/metiers/mod.rs` | `Champ::court` et `libelle_court()`; `Evaluation.quantites` et `.modele`; `Quantite::valeur()` (la grandeur comme valeur de la calculatrice) et `texte_libre()` |
| `noyau/src/moteur.rs` | libellé libre, texte libre et mention de l'afficheur; messages `IL MANQUE` et `À CORRIGER` |
| `app/src/clavier.rs` | `disposer()` : la disposition de 23.3; dessin des touches d'un champ |
| `app/src/application.rs` | la barre des métiers, les dialogues « Votre métier » et « mode d'emploi de l'outil », le va-et-vient des saisies entre le moteur et les formulaires |
| `app/src/vues/mod.rs` | `pastille()` |

### 23.9 Valeurs d'essai

Rejouées par `noyau/tests/outils.rs`. Entre crochets : une touche de l'outil. Préférences par défaut.

| Outil | Frappes | Afficheur |
|---|---|---|
| dalle | `24` `Pieds` `[Longueur]` `20` `Pieds` `[Largeur]` `4` `Pouces` `[Épaisseur]` `[Résultat]` ×3 | `SURFACE · 1 de 3 │ 480 pi² (44,6 m²)` · `VOLUME DE BÉTON · 2 de 3 │ 5,93 vg³ (4,53 m³)` · `À COMMANDER, PERTES COMPRISES · 3 de 3 │ 6,52 vg³ (4,98 m³)`; le bloc « Dalle de béton » est inscrit une fois au ruban |
| dalle | `24` `[Longueur]` · `4` `[Épaisseur]` · `7,3` `m` `[Longueur]` | `LONGUEUR │ 24 pi 0 po` · `ÉPAISSEUR │ 4 po` · `LONGUEUR │ 7,300 m` |
| dalle | `24` `Pieds` `[Longueur]` `[Résultat]` | `IL MANQUE : LARGEUR` |
| dalle | `5` `[Pertes]` `Conv` `[Pertes]` | `PERTES │ 5 %` puis `PERTES │ 10 %` |
| dalle | … `[Résultat]` `[Résultat]` `×` `200` `=` | `1 185,185 vg³` (5,925 926 vg³ × 200) |
| deport | `12` `Pouces` `[Déport]` `[Résultat]` `[Résultat]` | `LONGUEUR DE PARCOURS · 1 de 2 │ 1 pi 5 po` · `AVANCE │ 1 pi 0 po` |
| deport | `[Angle]` ×3, depuis 45° | `60°` (5 de 5) · `11,25°` (1 de 5) · `22,5°` |
| deport | `12` `Pouces` `[Déport]` `[Résultat]` `×` `2` `=` | `2 pi 9 15/16 po` : la longueur exacte (16,97 po) est doublée, pas son arrondi |
| chute-de-tension | `15` `[Courant]` `100` `Pieds` `[Longueur]` `[Résultat]` ×3 | `9,42 V` · `7,85 %` · `110,6 V` |
| chute-de-tension | `5` `Pieds` `[Courant]` · `0` `[Courant]` | `ERREUR – unités incompatibles` · `ERREUR – saisie invalide` |
| loi-d-ohm | `120` `[Tension]` `[Résultat]` | `Remplissez exactement deux des quatre champs.` |
| loi-d-ohm | `120` `[Tension]` `15` `[Courant]` `Conv` `[Courant]` | `CHAMP VIDÉ` |
| soumission | `1234567,89` `[Coût de base]` `[Résultat]` | le champ contient `1234567,89`; `ADMINISTRATION · 1 de 7 │ 37 037,04 $` |
| budget-au-pied-carre | `40` `Pieds` `×` `35` `Pieds` `=` `[Superficie]` | `SUPERFICIE HABITABLE │ 1 400 pi²`; le champ contient `1400` |
| calibre-pour-chute | `20` `[Courant]` `100` `Pieds` `[Longueur]` `[Résultat]` | `CALIBRE SUFFISANT · 1 de 4 │ 8 AWG` |
| tranchee, puis voyages-de-camion | `100` `Pieds` `[Longueur]` `2` `Pieds` `[Largeur au fond]` `6` `Pieds` `[Largeur en haut]` `4` `Pieds` `[Profondeur]` `[Résultat]` `[Résultat]`, changement d'outil, `[Volume]` `12` `[Camion]` `[Résultat]` | `VOLUME À TRANSPORTER, FOISONNÉ │ 74,1 vg³ (56,6 m³)`, puis `74,07407 vg³`, `VOLUME À TRANSPORTER │ 74,07407 vg³` et `VOYAGES │ 7 voyages` |

Un test de robustesse joue des milliers de touches au hasard sur chacun des 76 outils : aucune suite de touches ne fait planter le moteur, et l'afficheur reste toujours présentable. Un autre vérifie que chaque calculateur a dix champs au plus et que chaque libellé court tient sur une touche, dans les deux langues (24.8).

---

## 24. Français et anglais

Ajoutée le 2026-11-19, à la demande de l'utilisateur : « le bouton fr/en avec la traduction en anglais ». Toute la calculatrice se lit en **français du Québec** ou en **anglais du Canada**, et on passe de l'un à l'autre d'un toucher.

### 24.1 Principe

- **R-62** Chaque texte visible existe dans les deux langues, **écrit au même endroit** : un `Texte { fr, en }` (`noyau/src/langue.rs`), créé par `t("français", "English")` — ou par `tel("kW")` quand il s'écrit de la même façon dans les deux langues. Il se lit par `traduit()`, dans la langue en cours. Un texte sans version anglaise ne compile pas : il n'y a ni table de traduction à part, ni texte oublié. Dans une fonction qui choisit un texte (un `match`), on écrit `selon("français", "English")`.
- **R-63** Le français est la langue de référence. Sans réglage, tout le programme parle français : **aucun** test ni aucune valeur d'essai des sections 19, 22.14 et 23.9 ne change.
- **R-64** Changer de langue ne change **aucune valeur** : ni les mémoires, ni les registres, ni ce qui est saisi dans les outils. Seule l'écriture change.
- L'anglais est celui du Canada : *metre*, *millimetre*, *centre*, *labour*, *mitre*, *moulding*, *licence* (le nom), *tonne* (métrique). Les termes sont ceux des chantiers : `Pitch`, `Rise`, `Run`, `Hip/Valley`, `Jack`, `Stud`, `On centre`, `Plumb cut`, `Riser`, `Tread`, `Stringer`, `Drywall`, `Footing`, `Waste` (les pertes), `Tape` (le ruban).
- Le nom de l'application est le même dans les deux langues, « **Constructo AI** » (titre de la fenêtre et de la page, barre supérieure, « About », manuel).
- La règle R-01 vaut en anglais comme en français : aucun nom, aucune marque et aucun texte de l'application de référence.

### 24.2 Choisir la langue

| Où | Quoi |
|---|---|
| **Barre supérieure** | une pastille à deux moitiés, `FR` et `EN`, entre le titre et la roue dentée. La langue en cours est en évidence (fond clair, texte foncé). Toucher l'autre moitié change la langue sur-le-champ; chaque moitié se touche sur toute la hauteur de la barre. Nom accessible de chaque moitié : « Français », « English ». |
| **Préférences** | première ligne de « Général » : **Langue**, avec les choix « Français » et « English » (chaque langue est nommée dans sa propre langue). |
| **Premier lancement** | la langue de l'appareil, si c'est le français ou l'anglais : la première des langues préférées du système (bureau) ou du navigateur (Web) qui est l'une des deux; sinon le français. Ensuite, la langue choisie est conservée avec les préférences (`etat.json`). Un état d'une version précédente, sans langue, reste en français. |
| **Réinitialisation** | « Réinitialiser les données et les préférences » garde la langue en cours. |

Le changement est immédiat et complet : titre de la fenêtre (ou de la page, avec son attribut `lang`), barres, clavier, afficheur, barre des métiers, formulaires, aide, dialogues, date des rubans.

### 24.3 Nombres et unités

| | Français | Anglais |
|---|---|---|
| Séparateur décimal | la virgule, ou le point si la préférence le demande (13.2) | **toujours le point**; la préférence n'est pas montrée, son réglage français est conservé |
| Milliers | l'espace insécable : `1 234 567` | la virgule : `1,234,567` |
| Touche décimale | `,` (ou `.`) | `.` |
| Pieds, pouces, verges | `pi`, `po`, `vg` — `5 pi 7 1/2 po` | `ft`, `in`, `yd` — `5 ft 7 1/2 in` |
| Aires et volumes | `pi²`, `po²`, `vg²`, `pi³`, `po³`, `vg³` | `ft²`, `in²`, `yd²`, `ft³`, `in³`, `yd³` |
| Pied mesure de planche | `pmp` | `bd ft` |
| Tonne courte, tonne métrique | `tc`, `t` | `ton`, `t` |
| Masse volumique | `tc/vg³`, `lb/vg³`, `lb/pi³` | `ton/yd³`, `lb/yd³`, `lb/ft³` |
| Monnaie | `1 283,06 $` | `$1,283.06` (le signe moins devant : `−$0.13`) |
| Pourcentage | `47,25 %` | `47.25%` |
| Unités des outils | `pi/min`, `pi³/min`, `gal US/min`, `lb/po²`, `pi/s`, `lb/pi`, `$/pi²`, `sur 12` | `ft/min`, `CFM`, `US gal/min`, `psi`, `ft/s`, `lb/ft`, `$/ft²`, `in 12` |
| Rang d'un résultat | `2 de 3` | `2 of 3` |
| Date d'un ruban | `19 novembre 2026, 8 h 04` | `November 19, 2026, 8:04 AM` |

Les unités métriques, les degrés et les fractions s'écrivent de la même façon dans les deux langues. Ce qui est déjà au ruban y reste tel qu'il a été écrit.

**Lecture d'une saisie tapée** (formulaires, valeurs mémorisées, collage) :

- les **unités se lisent dans les deux langues**, quelle que soit la langue en cours : `6 pi 8 po`, `6 ft 8 in`, `6'8"`, `3 vg`, `3 yd`, `128 po²`, `128 in²`, `2 sq ft`;
- la **virgule** est le séparateur décimal en français (`2,032 m`). En anglais, une virgule suivie d'exactement trois chiffres, après un à trois chiffres dont le premier n'est pas zéro, **sépare les milliers** : `1,500` vaut 1500, `25,000` vaut 25 000, `1,234.5` vaut 1234,5. Toute autre virgule reste un séparateur décimal (`1,5` vaut 1,5; `0,500` vaut 0,5). Le point est un séparateur décimal dans les deux langues.

### 24.4 Les saisies changent d'écriture, pas de valeur

Une virgule n'a pas le même sens dans les deux langues : `3,500 m` vaut trois mètres et demi en français, trois mille cinq cents mètres en anglais.

- **R-65** Au changement de langue, toutes les saisies des outils — celles des formulaires comme celles de l'outil branché sur le clavier — sont **récrites** comme elles se tapent dans la nouvelle langue (`noyau::analyse::transposer`) : le séparateur décimal, les milliers et les symboles d'unités changent, la valeur reste. `3,500 m` devient `3.500 m`, `12 pi 6 1/2 po` devient `12 ft 6 1/2 in`, `1 250 pi 6 po` devient `1250 ft 6 in`, et inversement (`1,500` redevient `1500`). Une saisie qui ne se lit pas est laissée telle quelle; le rang d'un choix dans une liste ne change pas.
- `metiers.json` porte la langue dans laquelle ses saisies sont écrites : si elle n'est pas celle de la calculatrice à l'ouverture, elles sont récrites.
- Les valeurs de départ des outils sont écrites à la française dans le catalogue (`9,975`); elles sont proposées à l'anglaise en anglais (`9.975`).
- **L'afficheur** : une étiquette de charpente (`PENTE` → `PITCH`), un message de la calculatrice et une valeur changent de langue sur-le-champ. Un **résultat d'outil** montré est récrit dans la nouvelle langue, au même rang (`SURFACE · 1 de 3` → `AREA · 1 of 3`), sans être inscrit de nouveau au ruban. Le contenu d'un champ ou un message d'outil laissent la place à la valeur seule (ou à zéro), comme en changeant d'outil (23.6).

### 24.5 Textes anglais

| Texte | Où |
|---|---|
| Étiquettes de l'afficheur, symboles d'unités, messages et erreurs | `noyau/src/etiquettes.rs`, `noyau/src/erreurs.rs` |
| Métiers, calculateurs, champs, libellés courts, options, unités, résultats, avis et notes des 76 outils | `noyau/src/metiers/` |
| Touches, menus, dialogues, préférences, écrans | `app/src/textes.rs` |
| Fiches d'aide | `app/src/aide.rs` |
| Chapitres du manuel | `app/outils/manuel-gabarit-en.html` |
| Écran d'attente et message d'échec de la version Web | attributs `data-en` de `app/index.html` (24.6) |

- Un libellé court de touche respecte la même limite dans les deux langues : deux lignes au plus, de douze caractères au plus (23.4). Il peut n'exister que dans une langue : `t("", "Tread")`.
- Dans les fiches d'aide, les touches portent leur nom anglais (`Pitch`, `Feet`, `Inch`, `Store`, `Recall`, `Clear`, `Result`) et les exemples sont écrits à l'anglaise.
- Les mots-clés de recherche d'une fiche réunissent les mots français et anglais : la recherche répond aux deux, quelle que soit la langue de l'écran.
- Clavier anglais : `Pitch`, `Rise`, `Run`, `Diag`, `Hip Valley`, `Compound Mitre`, `Stair`, `Arc`, `Circle`, `Jack`, `m`, `Length`, `Width`, `Height`, `%`, `Yards`, `Feet`, `Inch`, `/`, `Clear`, `Conv`, `Store`, `Recall`, `M+`; secondes fonctions : `Ratio`, `Rake wall`, `Polygon`, `Roof`, `Irreg. pitch`, `Crown angle`, `Riser limit`, `Radius`, `Column/Cone`, `Irreg. jack`, `Blocks`, `Footing`, `Drywall`, `bd ft`, `Prefs`, `Studs`, `ton`, `Clear all`, `tonne`, `Cost`, `dms◄►dec`, `Tape`.
- Les onze métiers : Estimating and bidding, Earthwork and landscaping, Concrete and foundations, Masonry and paving, Roofing and siding, Drywall and insulation, Finishing, Plumbing, Electrical, Ventilation and heating (pastille : HVAC), Steel and welding; le clavier de base : Framing.

### 24.6 Ce qui ne change pas

- Les **raccourcis clavier** (16.5) sont les mêmes dans les deux langues.
- Les **identifiants** des calculateurs, les noms de fichiers de données et le format des données.
- Le **manifeste** de l'application installable (`app/manifest.webmanifest`) : un manifeste n'a qu'une langue; il reste en français, et sa description dit que la calculatrice est offerte en français et en anglais.
- La **page hôte** de la version Web (`app/index.html`) est écrite en français; l'attribut `data-en` d'un élément porte son texte anglais. Avant que le programme soit chargé, un court script met ces textes en place quand la calculatrice est en anglais : la langue conservée dans `CalculatriceChantier/etat.json` ou, à la première visite, celle du navigateur — la même règle que le programme (24.2). Le message « JavaScript requis » est écrit dans les deux langues.

### 24.7 Manuel anglais

`app/manual.html` est produit par le même générateur que `app/manuel.html` (R-61), à partir de `app/outils/manuel-gabarit-en.html` et des textes anglais du programme; ses exemples sont calculés par le noyau en anglais.

- **R-66** Les deux gabarits se suivent : la même feuille de style, les mêmes chapitres et les mêmes ancres, les mêmes repères, autant de touches et de lignes, et **les mêmes nombres** (seule leur écriture change). Un test le vérifie; toute retouche d'un gabarit se fait dans les deux.
- Chaque gabarit cite les messages de l'afficheur de sa langue, tels que le programme les écrit (un test le vérifie).
- Chaque manuel porte, dans son en-tête, un lien vers l'autre. Les ancres des métiers sont les mêmes dans les deux (`#metier-beton`).
- Version Web : une navigation vers un manuel suit la règle « le réseau d'abord », comme la page de l'application, puis se rabat sur la copie locale de ce manuel (15.4). Vérifié le 2026-11-19, serveur arrêté : la calculatrice et les deux manuels s'ouvrent.
- Les deux pages font partie des fichiers gardés hors ligne par la version Web.

### 24.8 Réalisation et essais

| Où | Quoi |
|---|---|
| `noyau/src/langue.rs` | `Langue { Francais, Anglais }` (`code()`, `nom()`, `autre()`, `d_apres("fr-CA")`); la langue en cours, propre au fil d'exécution : `en_cours()`, `choisir()`, `pour_la_portee()` (la langue précédente revient à la fin de la portée); `Texte`, `t()`, `tel()`, `selon()` |
| `noyau/src/prefs.rs` | `Preferences.langue` |
| `noyau/src/format.rs` | `separateur_decimal()`, `separateur_de_milliers()`, `nombre_et_unite()`; `TexteValeur::espaces()` (le dollar devant, le pourcentage collé) |
| `noyau/src/analyse.rs` | unités des deux langues, virgule des milliers en anglais, `transposer()` |
| `noyau/src/metiers/mod.rs` | `Champ::depart()`, `Champ::transposer()`, `Calculateur::transposer_les_saisies()` |
| `noyau/src/moteur/outils.rs` | `Moteur::changer_de_langue()` |
| `app/src/application.rs` | `bouton_de_langue()`, `changer_de_langue()`, la langue du premier lancement, le titre de la fenêtre |
| `app/src/plateforme/` | `langue_de_l_appareil()`, `regler_la_page()`, `adresse_du_manuel()` |
| `app/src/stockage.rs` | `Chargement.premiere_fois`; la langue des saisies dans `metiers.json` |

- **R-67** La langue en cours est propre au fil d'exécution. L'application la règle d'après les préférences au début de chaque image; un test qui a besoin de l'anglais le demande par `langue::pour_la_portee(Langue::Anglais)`, sans nuire aux autres tests.

Essais (`noyau/tests/langue.rs`, et dans `app` : `textes.rs`, `aide.rs`, `manuel.rs`, `stockage.rs`) :

| Essai | Ce qu'il garantit |
|---|---|
| chaque texte du catalogue existe dans les deux langues | aucun libellé, option ou note sans version anglaise |
| aucun texte anglais n'est resté en français | ni lettre accentuée, ni symbole `pi`, `po`, `vg`, `tc` dans ce que les 76 outils écrivent en anglais |
| les libellés tiennent sur les touches | dans les deux langues |
| les valeurs de départ se lisent dans les deux langues | et donnent les mêmes grandeurs |
| changer de langue garde les valeurs | `3,500 m`, `1 250 pi 6 po`, `4,5 po`, `7,5` ↔ `3.500 m`, `1250 ft 6 in`, `4.5 in`, `7.5`; le même résultat, récrit |
| la virgule sépare les milliers en anglais seulement | `1,500` : 1,5 en français, 1500 en anglais |
| les fiches d'aide et les gabarits du manuel citent les mêmes nombres et les mêmes touches dans les deux langues | une erreur de transcription d'un exemple est détectée |
| les deux manuels sont à jour | `manuel.html` et `manual.html` sont identiques à ce que le générateur produit |

Valeurs d'essai en anglais (préférences par défaut) :

| Outil ou touches | Résultat |
|---|---|
| dalle : `24`, `20`, `4`, `10` | `Area │ 480 ft² (44.6 m²)` · `Concrete volume │ 5.93 yd³ (4.53 m³)` · `To order, waste included │ 6.52 yd³ (4.98 m³)` |
| soumission : `100,000`, `3`, `12`, `15`, `5`, `9.975` | `Administration │ $3,000.00` · `Contingency │ $12,000.00` · `Profit │ $15,000.00` · `Price before taxes │ $130,000.00` · `GST │ $6,500.00` · `QST │ $12,967.50` · `Price, taxes included │ $149,467.50` |
| pression-d-eau : `30` | `Pressure │ 89.7 kPa (13.0 psi)` |
| conduit-rond : `8`, `900` | `Cross-section │ 0.349 ft² (0.032 m²)` · `Airflow │ 314 CFM (148 L/s)` |
| rangs-de-maconnerie : `8`, `7 5/8`, `3/8` | `Course height, joint included │ 8 in` · `Courses │ 12 courses` · `Height reached │ 8 ft 0 in` |
| `7` `Inch` `Pitch` · `7` `Feet` `2` `Inch` `Run` · `Diag` ×3 | `PITCH │ 7 in` (in 12) · `RUN │ 7 ft 2 in` · `DIAGONAL │ 8 ft 3 9/16 in` · `PLUMB CUT │ 30.26°` · `LEVEL CUT │ 59.74°` |
| dalle au clavier : `24` `Feet` `[Length]` `[Result]` | `MISSING: WIDTH` |

---

## 25. Dessin d'une pièce

Ajouté le 2026-11-21 (version 0.7.0), à la demande de l'utilisateur : « dessiner avec mon doigt ou [la] souris sur ma calculatrice les dimensions de la pièce ou [de la] forme », puis toucher chaque ligne et y inscrire la mesure. L'utilisateur a confirmé la description avant la réalisation. Complété le même jour, avec son accord (« ok pour tout »), par les versions 0.7.1 à 0.8.0, puis, à sa demande (« ajouter pieds, pouces, fraction pour les dessins »), par la version 0.8.1.

### 25.1 Principe

1. L'écran **Dessin** s'ouvre par l'onglet « Dessin » de la barre du bas (le troisième de cinq) ou par le menu ☰ → « Dessiner une pièce ».
2. Sur la planche, l'utilisateur trace le contour avec le doigt ou la souris : d'un seul geste, ou un côté à la fois (chaque nouveau tracé repart du bout du précédent).
3. Le tracé est un **croquis** : il n'est pas à l'échelle. Chaque trait est redressé à l'horizontale ou à la verticale; les tremblements sont écartés. **Tous les angles sont d'équerre** (hypothèse 26).
4. Quand le tracé revient près de son point de départ, la forme se ferme d'elle-même. Le bouton « Fermer la forme » ajoute le ou les côtés qui manquent.
5. L'utilisateur touche un côté, puis inscrit sa mesure avec le **pavé de touches** qui s'ouvre sous le dessin : une ou deux rangées de **fractions de pouce** (au seizième, ou à la précision choisie dans les Préférences, comme la bande de fractions du clavier), les chiffres, `Pieds`, `Pouces`, `m` pour les mètres, la barre de fraction, l'espace et l'effacement. Une fraction se place après les pouces : `12` `Pieds` `6` `1/2` donne « 12 pi 6 1/2 po »; toucher une autre fraction la remplace. Sur le bureau, la mesure peut aussi se taper au clavier; un nombre seul est en pieds. « Suivant » (ou Entrée) passe au prochain côté sans mesure. Pendant la saisie, la planche rapetisse pour que le pavé reste visible en entier.
6. Les mesures s'écrivent sur le dessin **en pieds, pouces et fractions** (ou en mètres, si elles ont été inscrites ainsi), à côté de leur côté. Dès qu'il ne manque qu'un côté dans un sens (gauche-droite ou haut-bas), il est **déduit** des autres et écrit en gris.
7. Quand tout est connu, la forme est redessinée à l'échelle, et l'écran donne la **surface** et le **périmètre**, dans le système des mesures inscrites, l'autre entre parenthèses; pour un rectangle, aussi la **diagonale d'équerre** à mesurer sur place.
8. **Retraits** : des rectangles (longueur × largeur) à retirer de la surface — îlot, trémie d'escalier, colonne. L'écran donne alors la **surface nette**.
9. **Plusieurs pièces** : « + Pièce » en ajoute une (30 au plus); chaque pièce a son nom, son dessin et ses retraits; l'écran donne le **total** des surfaces nettes des pièces complètes.
10. « **Ajouter au ruban** » inscrit les mesures des côtés et les résultats, sous le nom de la pièce. « **Envoyer vers un outil…** » ouvre la liste des outils par métier qui partent d'une longueur et d'une largeur au sol (sauf l'acier); l'outil choisi s'ouvre au clavier avec un rectangle de **même surface (nette) et de même périmètre** que la pièce, ce qui lui donne le même résultat que la pièce dessinée.
11. « Annuler » remonte jusqu'à 40 étapes de la pièce à l'écran; « Retirer le dernier côté » et « Tout effacer » complètent les boutons.
12. **Zoom** (version 0.8.2, à la demande de l'utilisateur : « zoom/dézoomer pour le dessin ») : les boutons `+` et `−` du coin de la planche zooment autour de son centre (de ×0,5 à ×8); « Ajuster » apparaît quand la vue a bougé et fait de nouveau tenir la forme. Deux doigts zooment et déplacent la vue (le tracé en cours s'annule); sur le bureau, la molette zoome autour du pointeur et le bouton droit (ou du milieu) déplace la vue. Une forme fermée, qui ne prend plus de tracé, se déplace aussi au doigt. Le quadrillage suit le zoom; les mesures des côtés hors de la vue ne s'écrivent pas. Changer de pièce ou tout effacer remet le cadrage à zéro.

### 25.2 Règles

- **R-68** Le calcul est dans `noyau/src/dessin.rs`, sans interface : redressement du tracé, fermeture, lecture et déduction des mesures, aire (formule des lacets), périmètre, retraits (`Retrait`), pièces (`Piece`, `Plan`), rectangle équivalent et liste des outils d'une pièce. L'écran est `app/src/vues/dessin.rs`.
- **R-69** Une mesure inscrite n'est jamais modifiée par le programme. Si les mesures d'un sens ne s'accordent pas à 1/16 po près, l'écran le dit et donne l'écart; il ne calcule pas de surface.
- **R-70** Si, une fois mis à leurs mesures, des côtés se croisent, l'écran le dit et ne calcule pas de surface.
- **R-71** Le plan est conservé (`dessin.json` : `version`, `plan`, `langue`) et retrouvé à l'ouverture; un fichier de la version 0.7 (une seule pièce, `dessin`) est lu comme un plan d'une pièce. Changer de langue récrit les mesures des côtés et des retraits sans changer leur valeur (R-64).
- **R-72** Un retrait vide ne compte pas. Un retrait incomplet, ou des retraits plus grands que la pièce, sont signalés; la surface nette n'est alors pas donnée.
- **R-73** L'outil reçoit le rectangle de même surface et de même périmètre que la pièce; s'il n'existe pas, le carré de même surface, et l'écran le dit. Les deux mesures sont écrites en pieds décimaux (ou en mètres) dans les champs `Longueur` et `Largeur` de l'outil.
- **R-74** Le geste part de l'endroit où le doigt s'est posé, et toutes les positions reçues pendant une image sont retenues : un tracé rapide ne perd pas son début.
- **R-75** Le pavé n'appelle pas le clavier du téléphone : dans un navigateur, le champ de la mesure ne prend pas le curseur de lui-même.
- **R-76** Le zoom ne change que la vue : le croquis, les mesures et les résultats restent les mêmes. Un tracé fait en vue zoomée est ramené à l'échelle du croquis.

### 25.3 Vecteurs de validation

| Forme | Mesures inscrites | Déduites | Surface | Périmètre |
|---|---|---|---|---|
| Rectangle | 12 pi; 10 pi 6 po | 12 pi; 10 pi 6 po | 126 pi² (11,7 m²) | 45 pi 0 po |
| Pièce en L | 20; 8; 11; 10 (pieds) | 9 pi; 18 pi | 250 pi² (23,2 m²) | 76 pi 0 po |
| Rectangle | 4 m; 3 m | 4 m; 3 m | 12 m² | 14 m |
| Rectangle | 12; 10; 11; 10 (pieds) | — | aucune : écart de 1 pi 0 po | — |
| Rectangle 20 × 15 pi, retraits 3 × 6 pi et 12 × 12 po | — | — | nette : 281 pi² | 70 pi 0 po |
| Deux pièces : 12 × 10 pi et 20 × 15 pi | — | — | total : 420 pi² | — |

Autres valeurs : la diagonale d'équerre d'un rectangle de 12 pi × 10 pi 6 po est de 15 pi 11 5/16 po; la pièce en L de 250 pi² et de 76 pi de périmètre s'envoie à un outil comme un rectangle de 29,5357 pi × 8,4643 pi; au pavé, `12` `Pieds` `6` `1/2` donne 12 pi 6 1/2 po, soit 150,5 po, et `1/4` ensuite donne 12 pi 6 1/4 po.

### 25.4 Limites connues

Les murs en biais (prévus, avec l'accord de l'utilisateur, pour la fin), la surface des murs moins les portes et les fenêtres (prévue), les courbes.

## Annexe A — Constantes

### A.1 Facteurs de conversion (vers l'unité de base)

| Unité | Valeur en unité de base | Exactitude |
|---|---|---|
| 1 pi | 12 po | exact |
| 1 vg | 36 po | exact |
| 1 mm | 1 ÷ 25,4 po | exact (1 po = 25,4 mm) |
| 1 cm | 1 ÷ 2,54 po | exact |
| 1 m | 1 ÷ 0,0254 po ≈ 39,370 078 74 po | exact |
| 1 pi² | 144 po² | exact |
| 1 vg² | 1 296 po² | exact |
| 1 acre | 43 560 pi² = 6 272 640 po² | exact |
| 1 m² | 1 ÷ 0,0254² po² ≈ 1 550,003 1 po² | exact |
| 1 pi³ | 1 728 po³ | exact |
| 1 vg³ | 46 656 po³ | exact |
| 1 pmp | 144 po³ | exact |
| 1 m³ | 1 ÷ 0,0254³ po³ ≈ 61 023,744 09 po³ | exact |
| 1 kg | 1 ÷ 0,453 592 37 lb ≈ 2,204 622 622 lb | exact (1 lb = 0,453 592 37 kg) |
| 1 tc | 2 000 lb | exact |
| 1 t | 1 000 kg ≈ 2 204,622 622 lb | exact |

Toujours écrire les facteurs sous leur forme exacte dans le code (`1.0 / 0.0254`, `0.453_592_37`), jamais une valeur tronquée.

### A.2 Constantes de métier

| Constante | Valeur |
|---|---|
| π | `std::f64::consts::PI` |
| Base de la pente | 12 po de course |
| Carré de toiture | 100 pi² |
| Feuilles 4×8, 4×9, 4×10, 4×12 | 32, 36, 40, 48 pi²; largeur 4 pi |
| Seuil « nombre de côtés » de l'onglet composé | donnée < 25 |
| Seuil d'avertissement « escalier raide » | rapport contremarche/giron > 1,10 × rapport visé |
| Durée de l'appui long | 0,6 s |
| Nombre maximal d'éléments d'une liste (empannons, montants) | 200 |
| Lignes du ruban courant / rubans enregistrés | 500 / 200 |

### A.3 Masse volumique

```rust
pub enum UniteMasseVolumique { TcParVg3, LbParVg3, LbParPi3, TParM3, KgParM3 }
pub struct MasseVolumique { pub valeur: f64, pub unite: UniteMasseVolumique }
```

Symboles affichés : `tc/vg³`, `lb/vg³`, `lb/pi³`, `t/m³`, `kg/m³`. Conversion en lb par po³ : tc/vg³ × 2000 ÷ 46 656; lb/vg³ ÷ 46 656; lb/pi³ ÷ 1 728; t/m³ × 2 204,622 622 × 0,0254³; kg/m³ × 2,204 622 622 × 0,0254³. Repère : 1,5 tc/vg³ = 3 000 lb/vg³ = 111,1111 lb/pi³ = 1,779829 t/m³ = 1 779,829 kg/m³.

### A.4 Valeurs par défaut

Préférences : voir les ★ de la section 13. Valeurs mémorisées : échappée 80 po (`PiPo`); contremarche visée 7,5 po (`PoFrac`); épaisseur du plancher 10 po; giron visé 10 po; aire d'un bloc 128 po² (`Po2`); longueur d'un bloc 16 po; angle de la moulure 45°; entraxe 16 po; section de semelle 264 po²; masse volumique 1,5 tc/vg³; paquets par carré 3.

---

## Annexe B — Textes d'aide (français)

Textes originaux, à reprendre tels quels dans `app/src/aide.rs`. Leur version anglaise est écrite à côté de chacun, dans le même fichier (section 24) : elle n'est pas reproduite ici. Chaque fiche : **titre**, *résumé*, mode d'emploi. L'exemple chiffré de chaque fiche est tiré de la section 19 (même numéro de sous-section que la fonction). Les mots-clés entre crochets servent à la recherche et ne sont pas affichés.

### B.1 Touches principales

**Pente** — *La pente d'un toit ou d'un plan incliné.* Elle s'exprime en pouces de montée pour 12 pouces de course (7 po = pente 7/12), en degrés, en pourcentage ou en ratio. Pour la saisir : `7` `Pouces` `Pente` (7/12), `30` `Pente` (30 degrés) ou `75` `%` `Pente` (75 %). Pour la calculer : saisissez deux valeurs parmi la montée, la course et la diagonale, puis appuyez sur `Pente`. Appuyez de nouveau pour la voir sous ses autres formes. La pente saisie reste en mémoire jusqu'à ce que vous en saisissiez une autre. [toit, inclinaison, degrés, pourcentage, « pitch »]

**Montée** — *Le côté vertical d'un triangle rectangle.* C'est la hauteur d'un toit de la sablière au faîte, la hauteur totale d'un escalier d'un plancher fini à l'autre, ou la flèche d'un arc. Tapez la mesure puis `Montée` pour la saisir. Pour la calculer, saisissez deux autres valeurs (pente, course ou diagonale) et appuyez sur `Montée`. [hauteur, vertical, « rise »]

**Course** — *Le côté horizontal d'un triangle rectangle.* C'est la demi-portée d'un toit (du mur extérieur au centre du faîte), la longueur au sol d'un escalier, ou la corde d'un arc. Tapez la mesure puis `Course`, ou calculez-la à partir de deux autres valeurs. [horizontal, portée, « run »]

**Diag** — *La diagonale (hypoténuse).* C'est la longueur du chevron commun, ou la diagonale qui sert à vérifier l'équerrage d'une fondation. Appuyez de nouveau pour obtenir la coupe d'aplomb et la coupe de niveau. La longueur est mesurée de pointe à pointe : ajoutez le débord du toit et retranchez la demi-épaisseur de la faîtière au besoin. [chevron commun, hypoténuse, équerre]

**Arêtier Noue** — *La longueur du chevron d'arêtier ou de noue et ses coupes.* Saisissez la pente et la course du chevron commun, puis appuyez sur `Arêtier Noue`. Appuyez de nouveau pour la coupe d'aplomb, la coupe de niveau et la coupe latérale (biseau). Pour un toit dont les deux versants ont des pentes différentes, saisissez d'abord la pente irrégulière. Les longueurs sont théoriques, d'axe en axe. [croupe, « hip », « valley »]

**Onglet composé** — *Les réglages de la scie à onglets pour une moulure couronne coupée à plat.* Tapez l'angle du coin (90 pour un coin d'équerre), puis `Onglet composé` : vous obtenez l'angle d'onglet (rotation de la table) et l'angle de biseau (inclinaison de la lame). Pour un ouvrage à plusieurs côtés égaux, tapez plutôt le nombre de côtés (toute valeur inférieure à 25 est prise pour un nombre de côtés). Vérifiez d'abord que l'angle de la moulure mémorisé correspond à votre moulure. [corniche, moulure, biseau, scie]

**Escalier** — *Le calcul complet d'un escalier droit.* Saisissez la hauteur à monter avec `Montée`, la longueur disponible avec `Course`, ou les deux, puis appuyez sur `Escalier` à répétition : hauteur et nombre de contremarches, giron et nombre de marches, écarts d'arrondi, longueur de la trémie, longueur du limon, inclinaison. La hauteur de contremarche et le giron visés, l'échappée et l'épaisseur du plancher se règlent dans les Préférences. Le giron n'inclut pas le nez de marche. Un symbole d'avertissement signale une contremarche plus haute que la valeur visée. Vérifiez toujours les exigences du Code de construction. [marche, contremarche, giron, limon, trémie]

**Arc** — *Tout ce qui concerne un arc de cercle.* Deux données suffisent : le diamètre et l'angle, le diamètre et la longueur d'arc, ou encore la corde (touche `Course`) et la flèche (touche `Montée`). Appuyez sur `Arc` à répétition : angle, longueur d'arc, corde, aire du segment, aire du secteur, flèche, puis la longueur de chaque montant d'un mur cintré à l'entraxe mémorisé. [cintre, arche, fenêtre cintrée, corde, flèche]

**Cercle** — *Aire et circonférence d'un cercle.* Tapez le diamètre, puis `Cercle`; appuyez de nouveau pour l'aire, puis la circonférence. [diamètre, rond]

**Empannon** — *La longueur de chaque empannon.* Les empannons sont les chevrons raccourcis qui s'appuient sur l'arêtier ou la noue. Saisissez la pente et la course, puis appuyez sur `Empannon` à répétition : entraxe, incrément (différence de longueur d'un empannon au suivant), chaque longueur, puis les coupes. L'ordre (décroissant ou croissant) se choisit dans les Préférences. [chevron, « jack »]

**m** — *Mètres.* Tapez un nombre puis `m`. Deux pressions donnent des mètres carrés, trois des mètres cubes. Sur une valeur déjà affichée, `m` la convertit en mètres. [métrique]

**Longueur** — *Mémorise une longueur* pour les calculs d'aire, de volume et de matériaux (blocs, semelle, gypse, toiture). [dimension]

**Largeur** — *Mémorise la largeur*, puis donne, à chaque pression : l'aire, la diagonale d'équerrage et le périmètre du rectangle. [aire, équerrage, périmètre]

**Hauteur** — *Mémorise la hauteur*, puis donne, à chaque pression : le volume, l'aire des quatre murs et l'aire de la pièce (murs et plafond), utile pour la peinture et le gypse. [volume, murs]

**%** — *Pourcentages.* `+` `10` `%` ajoute 10 % à la valeur, `−` `20` `%` en retranche 20 %, `×` `18` `%` en calcule 18 %, `÷` `80` `%` la divise par 0,80. Fonctionne aussi avec des mesures. Sert enfin à saisir une pente en pourcentage : `2` `%` `Pente`. [pertes, rabais, majoration]

**Verges, Pieds, Pouces** — *Les unités impériales.* Tapez le nombre, puis l'unité : `5` `Pieds` `7` `Pouces`. Deux pressions de la même unité donnent des unités carrées, trois des unités cubes. Sur une valeur affichée, la touche convertit; `Pieds` alterne entre pieds-pouces et pieds décimaux, `Pouces` entre fraction et décimales. [impérial, conversion]

**/** — *La barre de fraction.* `1` `/` `2` donne un demi-pouce. Si vous ne tapez pas le dénominateur, la résolution en vigueur est utilisée : `15` `/` vaut 15/16. [fraction]

**Effacer** — *Une pression* efface l'affichage et l'erreur éventuelle. *Deux pressions* effacent aussi les valeurs temporaires (montée, course, longueur, diamètre…). La pente saisie, les mémoires et les valeurs mémorisées sont conservées. [remise à zéro]

**Conv** — *La touche de conversion et de seconde fonction.* Suivie d'une touche, elle exécute la fonction écrite en petit au-dessus de cette touche. Suivie d'une unité, elle convertit la valeur affichée. Vous pouvez aussi cliquer directement sur la petite étiquette. [convertir, seconde fonction]

**Stocker** — *Met une valeur en mémoire.* `Stocker` `1`, `2` ou `3` : trois registres libres. `Stocker` `4` à `9` et `0` : les valeurs mémorisées de la calculatrice (bloc, entraxe, section de semelle, contremarche visée, épaisseur du plancher, giron visé, masse volumique). [mémoire]

**Rappel** — *Rappelle une valeur.* Suivie d'une touche, elle affiche ce que cette touche a en mémoire : `Rappel` `5` montre l'entraxe, `Rappel` `Course` la dernière course. La valeur rappelée peut servir dans un calcul. [mémoire]

**M+** — *La mémoire cumulative.* Chaque pression ajoute la valeur affichée au total. `Rappel` `M+` montre le total; `M+` ensuite montre la moyenne, puis le nombre de valeurs. `Rappel` `Rappel` montre le total et vide la mémoire. [total, somme, moyenne]

**=** — *Termine le calcul.* Après une touche de fonction, `=` rend le résultat disponible pour une autre fonction. [égal, résultat]

**⌫** — *Retour arrière* : annule la dernière frappe de la saisie en cours. [corriger]

### B.2 Fonctions secondaires (avec `Conv`)

**Ratio** (`Conv` `Pente`) — *Saisit la pente comme un ratio* montée ÷ course : `0,65` `Conv` `Pente`.

**Mur en pente** (`Conv` `Montée`) — *La longueur de chaque montant d'un mur dont le dessus suit la pente du toit* (mur pignon, mur rampant). Saisissez la montée et la course (ou la pente et l'une des deux), puis `Conv` `Montée` et `Montée` à répétition : entraxe, chaque montant, base, angle du mur. Si le mur en pente repose sur un mur de hauteur constante, tapez cette hauteur juste avant `Conv` `Montée` : elle s'ajoute à chaque montant. [pignon, rampant, colombage]

**Polygone** (`Conv` `Course`) — *Une figure régulière à plusieurs côtés* (patio hexagonal, gloriette octogonale). Saisissez le rayon (`15` `Pieds` `Conv` `Arc`), tapez le nombre de côtés, puis `Conv` `Course` : angle intérieur, demi-angle (réglage de coupe), longueur d'un côté, périmètre, aire. [hexagone, octogone]

**Toiture** (`Conv` `Diag`) — *Les matériaux de couverture.* Saisissez la pente, puis la longueur et la largeur du bâtiment en plan (ou tapez directement une aire en plan), puis `Conv` `Diag` : aire réelle du toit, nombre de carrés (100 pi²), nombre de paquets de bardeaux, nombre de feuilles de 4 × 8. Les quantités ne sont pas arrondies et n'incluent ni les pertes, ni les débords, ni les noues. [bardeaux, contreplaqué, carré, paquet]

**Pente irrég.** (`Conv` `Arêtier Noue`) — *La seconde pente d'un toit irrégulier*, dont deux versants de pentes différentes se rencontrent sur un arêtier ou une noue : `8` `Pouces` `Conv` `Arêtier Noue`. Elle est effacée par deux pressions d'`Effacer`.

**Angle moulure** (`Conv` `Onglet composé`) — *L'angle entre le dos de la moulure et le mur.* Les moulures couronnes courantes sont à 38° ou à 45°. `38` `Conv` `Onglet composé` le mémorise; sans nombre, la touche l'affiche.

**Contrem. max.** (`Conv` `Escalier`) — *Le même calcul d'escalier, avec une hauteur de contremarche plafonnée.* La contremarche ne dépasse jamais la hauteur visée : la calculatrice ajoute une contremarche au besoin. À utiliser quand une hauteur maximale est imposée.

**Rayon** (`Conv` `Arc`) — *Saisit le rayon d'un cercle*, ou le calcule à partir de la corde et de la flèche d'un arc.

**Colonne/Cône** (`Conv` `Cercle`) — *Volume et surface d'une colonne, puis d'un cône.* Saisissez le diamètre (`Cercle`) et la hauteur (`Hauteur`), puis `Conv` `Cercle` et `Cercle` à répétition. Pratique pour le béton des colonnes et des coffrages tubulaires. [cylindre, pilier, béton]

**Emp. irrég.** (`Conv` `Empannon`) — *Les empannons du versant à pente irrégulière d'abord.* Dans les Préférences, choisissez de garder le même entraxe sur les deux versants ou d'apparier les empannons pour qu'ils se rencontrent sur l'arêtier.

**Blocs** (`Conv` `Longueur`) — *Le nombre de blocs de béton ou de briques.* À partir de la longueur et de la hauteur du mur saisies, d'une aire affichée, ou d'une longueur seule (une rangée). Bloc par défaut : 8 × 16 po, joint compris. Pour des briques, mémorisez l'aire d'une brique avec `Stocker` `4`. Le résultat n'est pas arrondi : ajoutez vos pertes avec `+` `5` `%`. [maçonnerie, brique]

**Semelle** (`Conv` `Largeur`) — *Le volume de béton d'une semelle.* Tapez la longueur de la semelle, puis `Conv` `Largeur`. La section de la semelle (largeur × épaisseur) est mémorisée : `16` `Pouces` `×` `8` `Pouces` `=` `Stocker` `6`. [fondation, béton, « footing »]

**Gypse** (`Conv` `Hauteur`) — *Le nombre de feuilles de gypse* de 4 × 8, 4 × 9, 4 × 10 et 4 × 12 pour une aire, ou pour une longueur de mur si les feuilles sont posées debout. Le résultat n'est pas arrondi et ne tient pas compte des ouvertures. [placoplâtre, panneau, feuille]

**x²** (`Conv` `%`) — *Le carré.* Le carré d'une longueur donne une aire.

**√x** (`Conv` `Effacer`) — *La racine carrée.* La racine d'une aire donne une longueur.

**cm, mm** (`Conv` `7`, `Conv` `9`) — *Centimètres et millimètres* : pour saisir (`250` `Conv` `9`) ou pour convertir la valeur affichée.

**pmp** (`Conv` `8`) — *Le pied mesure de planche* (144 pouces cubes). Tapez les dimensions nominales d'une pièce en pouces, pouces et pieds : `2` `×` `4` `×` `14` `Conv` `8`. Convertit aussi un volume. [bois d'œuvre, planche]

**1/x** (`Conv` `÷`) — *L'inverse* d'un nombre.

**lb, kg, t courte, t métr.** (`Conv` `4`, `1`, `6`, `3`) — *Les masses.* Pour saisir une masse ou convertir la valeur affichée. Un volume est converti en masse au moyen de la masse volumique mémorisée (et l'inverse). La tonne courte vaut 2 000 lb; la tonne métrique, 1 000 kg. [poids, béton, gravier]

**Montants** (`Conv` `5`) — *Le nombre de montants d'un mur* pour une longueur donnée, à l'entraxe mémorisé, montant de départ compris. N'inclut ni les montants doublés, ni les coins, ni les cadres d'ouverture. [colombage, « stud »]

**Acre** (`Conv` `2`) — *L'acre* (43 560 pi²), pour saisir une superficie ou convertir une aire affichée. [terrain, superficie]

**Coût** (`Conv` `0`) — *Le coût total.* La quantité étant affichée : `×`, le prix unitaire, puis `Conv` `0`. Le prix s'applique à l'unité affichée (par pied, par verge cube, par mètre carré…). Pour le bois en pmp, le prix est celui du millier de pmp. [prix, estimation]

**Effacer tout** (`Conv` `×`) — *Remet la calculatrice à neuf* : efface les mémoires et rétablit les valeurs mémorisées par défaut. Les préférences sont conservées. Une confirmation est demandée.

**+/−** (`Conv` `−`) — *Change le signe* de la valeur.

**π** (`Conv` `+`) — *La constante π* (3,141593).

**dms◄►déc** (`Conv` `,`) — *Convertit un angle* entre les degrés décimaux et les degrés-minutes-secondes. Pour saisir 23° 42′ 39″ : `23` `,` `42` `,` `39`. [angle, arpentage]

**Ruban** (`Conv` `=`) — *Affiche le ruban de calcul* : la liste de tout ce qui a été saisi et calculé. Vous pouvez l'enregistrer, le copier ou l'exporter. [historique]

**Préf.** (`Conv` `Stocker`) — *Ouvre les Préférences.*

**MC** (`Conv` `Rappel`) — *Vide la mémoire cumulative* M+.

**M−** (`Conv` `M+`) — *Soustrait* la valeur affichée de la mémoire cumulative.

**sin, cos, tan** (mode trigonométrique) — *Les fonctions trigonométriques*, en degrés. Avec `Conv` : les fonctions inverses, qui donnent un angle. Dans ce mode, elles remplacent les touches `Longueur`, `Largeur` et `Hauteur`.

**Fractions rapides** — *Une bande de touches* qui saisit une fraction de pouce d'un seul geste : `5` `Pieds` `6` puis `3/8`. S'active dans les Préférences.

**Clavier adapté au métier** (barre des métiers) — *Le clavier prend les touches de votre métier :* touchez le nom du métier, au-dessus du clavier, puis un de ses outils. Les champs de l'outil deviennent les touches du haut : tapez une valeur, puis appuyez sur la touche du champ où la ranger. `Résultat` calcule et inscrit le calcul au ruban; appuyez de nouveau pour voir le résultat suivant. `Conv` puis la touche d'un champ le remet à sa valeur de départ. Maintenez une touche de l'outil enfoncée pour lire son mode d'emploi. Exemple — Béton, Dalle de béton : `24` `Pieds` `Longueur` `20` `Pieds` `Largeur` `4` `Pouces` `Épaisseur` `Résultat` `Résultat` → `VOLUME DE BÉTON 5,93 vg³ (4,53 m³)`. Note : l'onglet Métiers offre les mêmes calculateurs en formulaires à remplir. [métier outil plombier électricien béton maçon couvreur gypse acier soudure terrassement estimation soumission]

### B.3 Écran « Bienvenue »

> **Bienvenue!**
>
> Cette calculatrice parle le langage du chantier : pieds, pouces et fractions, mais aussi mètres et millimètres.
>
> 1. **Tapez une mesure comme vous la dites** : `5` `Pieds` `7` `Pouces` `1` `/` `2`.
> 2. **Les touches du haut résolvent le triangle** : donnez deux valeurs parmi Pente, Montée, Course et Diag, puis appuyez sur la troisième.
> 3. **La touche Conv** donne accès aux fonctions écrites en petit au-dessus des touches, et convertit les unités.
> 4. **Besoin d'aide?** Maintenez une touche enfoncée, ou faites un clic droit, pour afficher sa fiche.
> 5. **Plombier, électricien, maçon, couvreur?** Touchez le nom du métier, au-dessus du clavier : les touches du haut deviennent celles de vos outils. L'onglet Métiers offre les mêmes calculateurs en formulaires.
>
> Bouton : **Commencer**

### B.4 Écran « Trucs de pro »

1. **Faites défiler les résultats.** Appuyez de nouveau sur une touche de fonction pour voir tous ses résultats : `Diag` donne la longueur, puis la coupe d'aplomb, puis la coupe de niveau.
2. **La pente reste en mémoire.** Inutile de la retaper pour le chevron suivant.
3. **Repartez à neuf.** Avant un nouveau problème, appuyez deux fois sur `Effacer`.
4. **Réutilisez un résultat.** Pour envoyer un résultat dans une autre fonction, appuyez d'abord sur `=`.
5. **Vérifiez l'équerre.** `Longueur`, `Largeur`, puis `Largeur` deux fois : la diagonale à mesurer sur le terrain.
6. **Ajoutez vos pertes d'un geste.** `+` `10` `%`.
7. **Convertissez sans effort.** Sur une valeur affichée, appuyez sur l'unité voulue. `Pieds` alterne entre pieds-pouces et pieds décimaux.
8. **Cliquez sur l'étiquette.** Un clic sur la petite étiquette au-dessus d'une touche équivaut à `Conv` puis la touche.
9. **Gardez votre liste de coupe.** Enregistrez le ruban après le calcul d'un escalier ou d'un toit.
10. **Plus de précision?** Réglez la résolution des fractions dans les Préférences, ou temporairement : `Conv` `3` pour le 1/32.
11. **Arrondissez à la hausse.** Les quantités de matériaux sont exactes, non arrondies : commandez toujours l'unité supérieure.
12. **Au clavier.** L'apostrophe pour les pieds, le guillemet pour les pouces, la barre d'espace pour `Conv`.
13. **Votre métier au bout des doigts.** Touchez le nom du métier, au-dessus du clavier, pour que les touches du haut deviennent celles de vos outils.
14. **D'un outil à l'autre.** Un résultat affiché vous suit quand vous changez d'outil : appuyez sur la touche du champ où le ranger.
15. **En français ou en anglais.** Le bouton FR | EN, en haut de l'écran, change la langue sur-le-champ : les nombres et les unités suivent (1 234,5 pi² devient 1,234.5 ft²), et vos valeurs restent les mêmes.

---

## Annexe C — Aide-mémoire eframe/egui 0.37

Faits **vérifiés le 2026-11-19 dans la documentation en ligne d'eframe 0.37.0 et d'egui 0.37.0** (publiées le 2026-10-15). En cas d'écart avec la version installée, la documentation de la version installée fait foi. Les lignes marquées « à vérifier » n'ont pas été confirmées mot pour mot.

### C.1 Ce qui a changé depuis egui 0.31 (pièges)

| Ancien (ne compile plus) | Actuel |
|---|---|
| `impl eframe::App { fn update(&mut self, ctx: &Context, frame: &mut Frame) }` | `fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame)` (méthode obligatoire); `fn logic(&mut self, ctx, frame)` facultative, appelée avant `ui`, sans dessin |
| `CentralPanel::default().show(ctx, …)` | `CentralPanel::default().show(ui, …)` — prend un `&mut Ui`; doit être ajouté **après** les autres panneaux |
| `TopBottomPanel::top(…)`, `SidePanel::left(…)` | `egui::Panel::top(id)`, `::bottom(id)`, `::left(id)`, `::right(id)`, puis `.show(ui, …)`; taille fixe : `.exact_size(48.0)`; `.resizable(false)`; `.frame(…)` |
| `Rounding` | `CornerRadius` (`CornerRadius::same(10)`; champs en `u8`; conversions depuis `u8` et `f32`) |
| `painter.rect_stroke(rect, r, stroke)` | un quatrième paramètre `StrokeKind` (aussi pour `painter.rect`) |
| `ctx.style()`, `ctx.set_style(…)` | `ctx.global_style()`, `ctx.set_global_style(…)`, `ctx.global_style_mut(…)` |
| `ctx.screen_rect()` | `ctx.content_rect()` ou `ctx.viewport_rect()` |
| `Id::new("x")` | déconseillé : `Id::unique("x")`, ou `ui.scope_id().with("x")`; les paramètres `id_salt` acceptent toute valeur `Hash + Debug` |
| `ui.id()` | `ui.scope_id()` |
| `frame.close()` | `ctx.send_viewport_cmd(egui::ViewportCommand::Close)` |
| modificateurs lus dans `RawInput` | `i.modifiers` dans `ctx.input(…)`, ou le champ `modifiers` de `Event::Key` |
| `ui.button("x")` prenant `impl Into<WidgetText>` | prend `impl IntoAtoms` (une chaîne convient toujours) |
| `SelectableLabel`, `ImageButton` | supprimés ou déconseillés |

`Ui` se déréférence en `Context` : `ui.input(…)`, `ui.request_repaint()`, `ui.copy_text(…)` fonctionnent directement.

### C.2 Paquet et fonctionnalités

- eframe 0.37.0 : `edition = "2024"`, `rust-version = "1.95"`.
- Fonctionnalités par défaut : `accesskit`, `clipboard`, `default_fonts`, `links`, `system_fonts`, `wayland`, `web_screen_reader`, `wgpu`, `x11`. **Non** par défaut : `persistence`, `glow`.
- `system_fonts` (par défaut) : repli sur les polices du système pour les caractères absents des polices intégrées.
- La persistance intégrée d'eframe n'est **pas** utilisée (fichiers JSON maison, section 17) : ne pas activer `persistence`.
- Version Web : `eframe = { default-features = false, features = ["glow"] }` (rendu WebGL, sans `default_fonts`). Avec `glow`, la méthode `App::on_exit` prend un paramètre de plus (`Option<&glow::Context>`) : l'application ne la définit donc que pour le bureau. Démarrage : `eframe::WebRunner::new().start(canevas, eframe::WebOptions::default(), créateur).await`, lancé par `wasm_bindgen_futures::spawn_local`.
- Version Web : `std::time::Instant` n'existe pas dans le navigateur; mesurer le temps avec l'horloge d'egui (`ctx.input(|i| i.time)`).
- Compagnon pour les boîtes de dialogue de fichiers : `rfd` (API à vérifier à l'usage : `rfd::FileDialog::new().set_file_name(…).add_filter(…).save_file()`).

### C.3 Squelette de départ (à valider à la compilation)

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![forbid(unsafe_code)]

use eframe::egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(textes::NOM_APPLICATION)
            .with_inner_size([420.0, 860.0])
            .with_min_inner_size([340.0, 620.0])
            .with_app_id("CalculatriceChantier"),
        ..Default::default()
    };
    eframe::run_native(
        textes::NOM_APPLICATION,
        options,
        Box::new(|cc| Ok(Box::new(Application::nouvelle(cc)))),
    )
}

impl eframe::App for Application {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.traiter_clavier(ui);
        egui::Panel::top("barre_superieure").exact_size(48.0).show(ui, |ui| { /* … */ });
        egui::Panel::bottom("barre_inferieure").exact_size(56.0).show(ui, |ui| { /* … */ });
        egui::CentralPanel::default().show(ui, |ui| { /* afficheur + clavier */ });
    }
}
```

`Application::nouvelle(cc: &eframe::CreationContext<'_>)` : c'est là qu'on installe les polices (`cc.egui_ctx.set_fonts(…)`), le style (`cc.egui_ctx.set_global_style(…)`) et qu'on charge l'état.

### C.4 Signatures utiles (vérifiées)

```rust
// Disposition et interaction
ui.allocate_exact_size(desired_size: Vec2, sense: Sense) -> (Rect, Response)
ui.allocate_rect(rect: Rect, sense: Sense) -> Response
ui.interact(rect: Rect, id: Id, sense: Sense) -> Response
ui.available_size() -> Vec2
ui.max_rect() -> Rect
ui.painter() -> &Painter
Sense::click()   Sense::hover()   Sense::click_and_drag()
// Response : `rect` est un CHAMP public (response.rect)
response.clicked()  .secondary_clicked()  .double_clicked()  .hovered()
response.is_pointer_button_down_on()  .long_touched()  .on_hover_text(text)

// Dessin
painter.rect_filled(rect, corner_radius: impl Into<CornerRadius>, fill: impl Into<Color32>)
painter.rect_stroke(rect, corner_radius, stroke: impl Into<Stroke>, stroke_kind: StrokeKind)
painter.text(pos: Pos2, anchor: Align2, text: impl ToString, font_id: FontId, color: Color32) -> Rect
painter.layout_no_wrap(text: String, font_id: FontId, color: Color32) -> Arc<Galley>
painter.galley(pos: Pos2, galley: Arc<Galley>, fallback_color: Color32)
painter.line_segment(points: [Pos2; 2], stroke: impl Into<Stroke>)
painter.circle_filled(center: Pos2, radius: f32, fill: impl Into<Color32>)
Color32::from_rgb(r, g, b)      Color32::from_hex("#F2A900") -> Result<Color32, _>
Stroke::new(width: f32, color)  FontId::proportional(size)  FontId::new(size, family)

// Entrées
ui.input(|i| { i.events /* Vec<Event> */; i.modifiers; i.time /* f64 */; i.key_pressed(Key::F11) })
Event::Text(String)
Event::Key { key: Key, physical_key: Option<Key>, pressed: bool, repeat: bool, modifiers: Modifiers }
Event::Paste(String)   Event::Copy
// Key : Num0..Num9, A..Z, Enter, Escape, Backspace, Delete, Insert, Space, Tab,
//       Comma, Period, Slash, Plus, Minus, Equals, Quote, Colon, F1..F35

// Fenêtre et système
ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true))
ctx.copy_text(text: String)
ctx.request_repaint_after(duration: std::time::Duration)
ctx.system_theme() -> Option<Theme>      ctx.set_theme(…)      ctx.set_visuals(visuals)
ViewportBuilder::with_title / with_inner_size / with_min_inner_size / with_icon(impl Into<Arc<IconData>>)
              / with_resizable / with_fullscreen / with_app_id
eframe::icon_data::from_png_bytes(png: &[u8]) -> Result<IconData, _>

// Conteneurs
egui::Frame::new().fill(color).inner_margin(m).corner_radius(r).stroke(s).show(ui, |ui| …)
egui::ScrollArea::vertical().id_salt("…").auto_shrink(false).show(ui, |ui| …)
```

### C.5 Recettes

**Touche dessinée sur mesure**

```rust
let (rect, reponse) = ui.allocate_exact_size(taille, egui::Sense::click());
let enfoncee = reponse.is_pointer_button_down_on();
let peintre = ui.painter();
peintre.rect_filled(rect, egui::CornerRadius::same(10), couleur_fond);
peintre.text(rect.center(), egui::Align2::CENTER_CENTER, libelle,   // constante Align2 : à vérifier
             egui::FontId::proportional(taille_police), couleur_texte);
if reponse.clicked() { /* envoyer la Touche au moteur */ }
if reponse.secondary_clicked() { /* ouvrir l'aide */ }
```

**Appui long à la souris** : mémoriser `ui.input(|i| i.time)` quand `is_pointer_button_down_on()` devient vrai; tant qu'il le reste, appeler `request_repaint_after(100 ms)`; à 0,6 s, ouvrir l'aide et ignorer le `clicked()` qui suivra le relâchement.

**Clavier** : parcourir `i.events`; utiliser `Event::Text` pour les caractères imprimables (chiffres, `,`, `.`, `'`, `"`, `+`, `-`, `*`, `/`, `:`, `%`, `=`, lettres) — c'est indépendant de la disposition du clavier (français canadien, multilingue, etc.) — et `Event::Key { pressed: true, .. }` pour `Enter`, `Escape`, `Backspace`, `Delete`, `Insert`, `Space`, `F1`, `F11` et les combinaisons avec `Ctrl`. Ne pas traiter le même appui deux fois.

**Polices** : `FontDefinitions::default()`, insérer `Arc::new(FontData::from_owned(octets))` dans `font_data` (les valeurs sont des `Arc<FontData>`), ajouter le nom dans `families`, puis `ctx.set_fonts(…)`. Famille nommée pour le gras : `FontFamily::Name("gras".into())` (à vérifier).

**Fractions empilées** : mesurer chaque morceau avec `painter.layout_no_wrap(…)` (la taille est dans `galley.size()`), calculer la largeur totale, puis dessiner de droite à gauche avec `painter.galley(…)`; tracer la barre de fraction avec `line_segment`.

**Moteur de rendu de repli** : fonctionnalité `glow` d'eframe et `renderer: eframe::Renderer::Glow` dans `NativeOptions` (à vérifier).

---

## Annexe D — Traçabilité photos → application

| Photo | Élément de la référence | Dans l'application | Section |
|---|---|---|---|
| 01 | Barre supérieure : menu, plein écran, titre, réglages | identique en fonction; titre = nom original | 15.1 |
| 01 | Afficheur | afficheur à trois lignes, fractions empilées | 16.2 |
| 01 | 40 touches et leurs étiquettes secondaires | grille française | 5.1, 5.2 |
| 01 | Onglets Home / Saved Tapes / Help | Accueil / Rubans / Aide | 15.1 |
| 02 | Trig Mode | Mode trigonométrique | 13.1, 12.18 |
| 02 | Legacy Mode | **non repris** : il sert, dans la référence, à retrouver une ancienne disposition munie d'un retour arrière; ici le retour arrière (⌫) est toujours présent | 7.7 |
| 02 | Fast Fractions | Fractions rapides | 13.1, 7.6 |
| 02 | Haptic Feedback | **sans objet** sur ordinateur (pas de moteur de vibration); retour visuel à l'enfoncement | 16.3 |
| 02 | Full Screen Mode | Mode plein écran | 13.1 |
| 02 | Reset calculator data and settings | Réinitialiser les données et les préférences | 13.5 |
| 02 | Fractional Resolution | Résolution des fractions | 13.2 |
| 02 | Fractional Mode | Mode des fractions | 13.2 |
| 02 | Exponential Mode | Notation exponentielle | 13.2 |
| 02–03 | Area Display, Volume Display | Affichage des aires, des volumes | 13.2 |
| 03 | Meters | Mètres (longueurs) | 13.2 |
| 03 | Decimal Degrees | Degrés décimaux | 13.2 |
| 03 | Comma Separator | Séparateur de milliers (une espace, selon l'usage français) | 13.2 |
| 03 | Unitless Decimal Display | Décimales (valeurs sans unité) | 13.2 |
| 03 | Rake Wall | Mur en pente | 13.3 |
| 03 | Arched Wall | Mur cintré | 13.3 |
| 03 | Jack Rafters | Empannons | 13.3 |
| 03 | Irregular Jack Rafters | Empannons irréguliers | 13.3 |
| 03–05 | Headroom Height … Weight/Volume (dix valeurs) | Valeurs mémorisées (dix valeurs équivalentes) | 13.4 |
| 05 | Account Settings (compte en ligne) | **exclu** : aucun compte, aucune donnée en ligne | annexe E |
| 06 | Saved Tapes, Help, Welcome, Pro Tips, About | Rubans enregistrés, Aide, Bienvenue, Trucs de pro, À propos | 15 |
| 06 | Site Web, Like Us, Contact Us, Join Tool Crew, Demo Videos, Other CI Apps | **exclus** : liens promotionnels propres à l'éditeur de la référence | annexe E |

**Ajouts par rapport à la référence**, pour l'usage au Québec et sur ordinateur : séparateur décimal au choix, thème clair/sombre, feuilles de gypse 4 × 10, nombre de paquets de bardeaux par carré réglable, escalier arrondi au millimètre quand la donnée est métrique, modification des valeurs mémorisées par saisie de texte, raccourcis clavier, clic sur les étiquettes secondaires, panneau de ruban en fenêtre large, copie et export du ruban.

---

## Annexe E — Hors périmètre

Ne pas réaliser, même partiellement :

- compte utilisateur, synchronisation, toute communication réseau, télémétrie, publicité, achats;
- assistant conversationnel, notes de projet, vidéos, illustrations des calculateurs (les calculateurs par métier de la section 22 sont des formulaires sans dessin);
- tout ce qui relève d'un code plutôt que du calcul : tables d'intensité admissible, calibre minimal, remplissage des conduits et des boîtes, facteurs d'alimentation de plomberie, portées des pièces de charpente;
- connexion à des appareils de mesure (télémètres laser);
- applications mobiles natives (la version Web, section 3.1, sert sur téléphone et sur tablette);
- autres langues que le français;
- programme d'installation (l'exécutable `--release` suffit);
- retour haptique et sons;
- vérification automatique de la conformité au Code de construction.

---

## Annexe F — Invite de démarrage de la prochaine session

À coller telle quelle dans une nouvelle session Claude Code ouverte dans ce dossier :

```text
Lis en entier le fichier CAHIER_DES_CHARGES.md qui se trouve dans ce dossier, puis
regarde les six photos 01.png à 06.png (référence fonctionnelle seulement).

Réalise ensuite le programme décrit, en Rust, en suivant le plan par phases de la
section 20, dans l'ordre. Écris et fais passer les tests du noyau (section 19) avant
de commencer l'interface. Respecte les règles R-01 à R-76, la terminologie de la
section 4 et les textes de l'annexe B. Avant d'écrire du code egui, relis l'annexe C
et vérifie les signatures dans la documentation de la version installée.

À la fin de chaque phase, lance cargo fmt, cargo clippy -- -D warnings et cargo test,
puis fais-moi un court compte rendu en français. Ne fais aucun commit git sans que je
le demande. Si une règle du cahier des charges te paraît erronée ou ambiguë, signale-le
au lieu de la contourner.
```

---

## Annexe G — Hypothèses retenues (à confirmer par l'utilisateur; aucune n'est bloquante)

Ces choix ont été faits faute d'indication contraire. Pour en changer un, modifier la section indiquée **avant** de lancer la réalisation.

| # | Hypothèse | Section |
|---|---|---|
| 1 | Cible : application de bureau **Windows** (portable Linux/macOS). **Version Web ajoutée à la demande de l'utilisateur le 2026-11-19**; pas d'application mobile native. | 3, 3.1 |
| 2 | Interface avec **eframe/egui**. | 3 |
| 3 | Nom : « **Constructo AI** » — tranché par l'utilisateur le 2026-11-21. | 1.3 |
| 4 | Un nombre sans unité additionné à une longueur est une **erreur** (plutôt qu'une unité devinée). | 9.5 |
| 5 | Surface de la colonne et du cône : **surface totale** (bases comprises). | 12.9 |
| 6 | Feuilles de gypse : ajout du format **4 × 10**. | 12.14 |
| 7 | Bardeaux : **3 paquets par carré** par défaut, réglable. | 12.5 |
| 8 | Masse volumique par défaut : **1,5 tc/vg³**. | 13.4 |
| 9 | Valeurs d'escalier par défaut : contremarche 7 1/2 po, giron 10 po, échappée 6 pi 8 po, plancher 10 po. | 13.4 |
| 10 | Terminologie : « montée / course », « empannon », « coupe latérale », « mur en pente », « gypse », « ruban ». | 4.3 |
| 11 | Les listes d'empannons n'affichent pas d'élément final de longueur nulle. | 12.3 |
| 12 | Palette aux couleurs de l'ERP Constructo AI — **choisie par l'utilisateur le 2026-11-19** (elle remplace « Ardoise et ambre »). | 16.3 |
| 13 | Calculateurs par métier : l'utilisateur a demandé « plomberie, électricien, fondation, etc. », puis des calculateurs « vraiment complets », à utiliser tous les jours; la liste des 76 calculateurs, le découpage en onze métiers et les valeurs de départ sont un choix de Claude. | 22 |
| 14 | Les calculateurs par métier ne reproduisent aucune table de code (intensités admissibles, calibres, pentes minimales) : géométrie et physique seulement. | 22.1 |
| 15 | Estimation : la méthode du coût majoré (administration, imprévus et profit sur le coût de base; taxes sur le prix avant taxes) et ses pourcentages de départ viennent du profil d'entrepreneur général fourni par l'utilisateur le 2026-11-19. Les coûts au pied carré et les taux horaires de ce profil ne sont **pas** repris : l'utilisateur de l'application saisit les siens. À confirmer : faut-il les offrir en choix prédéfinis? | 22.3 |
| 16 | Clavier adapté au métier : l'utilisateur s'attendait à ce que « la calculatrice s'adapte selon le métier ». Le principe retenu — les champs d'un calculateur deviennent les touches du haut, une touche `Résultat` fait défiler les résultats, une barre des métiers au-dessus du clavier — est un choix de Claude. | 23 |
| 17 | Valeurs de départ des calculateurs ajoutés à la version 0.4 : retenue de 10 %, paquet de bardeaux de 33,3 pi², rouleau de tourbe de 10 pi², espacement de 9 po et boucle de 300 pi pour un plancher chauffant, rendement de dépôt de 65 %, rendement de 90 % et facteur de puissance de 0,85 pour un moteur, facteurs d'éclairage de 0,6 et de 0,8. Ce sont des points de départ modifiables, pas des recommandations. | 22 |
| 18 | Constantes physiques citées en note : coefficients de dilatation des tuyaux, masses volumiques des métaux, 1,08 BTU/h par pi³/min et par degré Fahrenheit. Valeurs courantes des manuels; les données du fabricant font foi. | 22.5, 22.8, 22.13 |
| 19 | Licence : l'utilisateur a choisi **Apache 2.0** le 2026-11-19. Le titulaire du droit d'auteur inscrit dans `NOTICE` et dans le README — « Sylvain Leduc, Constructo AI » — reprend la mention publique de l'auteur; à confirmer (la personne ou l'entreprise). | 3 |
| 20 | « Révision » au pied du menu : l'utilisateur a demandé « la révision »; Claude affiche la version du programme suivie de l'identifiant court du commit git. | 15.1 |
| 21 | Manuel d'utilisation : sa forme (une page Web produite à partir du programme, ouverte depuis le menu) et son plan sont un choix de Claude. | 15.4 |
| 22 | Anglais : l'utilisateur a demandé un bouton de langue « avec la traduction en anglais ». L'anglais retenu est celui **du Canada** (*metre*, *centre*, *labour*, *mitre*, *moulding*), le nom de l'application y est le même qu'en français (« **Constructo AI** », depuis le 2026-11-21), et les taxes s'y appellent GST et QST. | 24.1, 24.5 |
| 23 | En anglais, les nombres s'écrivent à l'anglaise (`1,234.5`, `$1,283.06`, `47.25%`) et les unités impériales portent leurs symboles anglais (`ft`, `in`, `yd`, `ft²`, `yd³`, `bd ft`, `ton`). Une virgule tapée entre des groupes de trois chiffres y sépare les milliers. | 24.3 |
| 24 | Au premier lancement, la langue est celle de l'appareil si c'est le français ou l'anglais, sinon le français. Un utilisateur qui avait déjà la calculatrice (en français) la garde en français. | 24.2 |
| 25 | Les raccourcis clavier restent les mêmes dans les deux langues (`P` pour `Pitch`, `T` pour `Rise`, `C` pour `Run`…) : ils ne sont pas refaits d'après les mots anglais. Le manifeste de l'application installable reste en français (un manifeste n'a qu'une langue). | 24.6 |
| 26 | Dessin d'une pièce : tous les angles sont d'équerre « pour le moment » (réponse de l'utilisateur du 2026-11-21); il a ensuite accepté (« ok pour tout ») les ajouts proposés : ruban, envoi vers un outil, saisie au pavé, retraits, plusieurs pièces, puis les murs en biais, à faire à la fin. Choix de Claude, non confirmés : un nombre inscrit sans unité est en pieds; l'outil reçoit le rectangle de même surface et de même périmètre. | 25 |
