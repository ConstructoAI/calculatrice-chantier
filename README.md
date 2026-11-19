# Calculatrice de chantier

Calculatrice de construction en français du Québec. Elle calcule en
pieds-pouces-fractions et en métrique, et résout les problèmes courants de chantier :
pentes et chevrons, arêtiers, noues et empannons, escaliers, murs en pente et murs
cintrés, arcs et cercles, onglets composés, quantités de matériaux (blocs, béton de
semelle, gypse, bardeaux, montants, bois en pmp), conversions et coûts.

Le même programme existe en deux versions :

| Version | Pour l'utiliser |
|---|---|
| **Web** — ordinateur, tablette, téléphone | Ouvrir <https://constructoai.github.io/calculatrice-chantier/> : rien à installer. |
| **Bureau** — Windows 10 ou 11 (64 bits) | Télécharger `calculatrice-chantier-windows-x64.exe` depuis la [dernière publication](https://github.com/ConstructoAI/calculatrice-chantier/releases/latest), puis le lancer : rien à installer. |

L'exécutable de bureau n'est pas signé. Au premier lancement, Windows peut afficher
« Windows a protégé votre ordinateur » : choisir *Informations complémentaires*, puis
*Exécuter quand même*.

Le comportement complet est décrit dans `CAHIER_DES_CHARGES.md`.

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

## Les deux versions en bref

| | Bureau | Web |
|---|---|---|
| Données (réglages, mémoires, rubans) | fichiers JSON dans `%APPDATA%\CalculatriceChantier\` | stockage local du navigateur, sur l'appareil |
| Export d'un ruban en `.txt` | boîte « Enregistrer sous » | téléchargement |
| Plein écran | préférence, bouton ou `F11` | bouton (là où le navigateur le permet) ou `F11` du navigateur |
| Police | Segoe UI (Windows) | Selawik, police libre aux mêmes largeurs |
| Menu « Quitter » | oui | non : on ferme l'onglet |

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
recharger la page).

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
section 19 du cahier des charges.

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
| `noyau/` | Bibliothèque de calcul (`calculatrice-noyau`) : unités, saisie, formatage, moteur, fonctions de construction. Aucune dépendance graphique; entièrement testée sans écran. |
| `app/` | Interface graphique (`calculatrice-chantier`) avec eframe/egui : clavier, afficheur, préférences, rubans, aide. Ne contient aucune formule. |
| `app/src/plateforme/` | Tout ce qui diffère entre le bureau (`bureau.rs`) et le navigateur (`web.rs`) : stockage, export, plein écran, démarrage. |
| `app/index.html` | Page hôte de la version Web. |
| `app/polices/` | Polices libres de la version Web, avec leurs licences (`LISEZMOI.md`). |

## Nom de l'application

Le nom affiché est défini par la constante `NOM_APPLICATION` de `app/src/textes.rs`. La
page hôte de la version Web (`app/index.html`) le répète dans son titre et sur son écran
d'attente.
