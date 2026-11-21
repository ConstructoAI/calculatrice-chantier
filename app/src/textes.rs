//! Tous les textes de l'interface, en français du Québec et en anglais du
//! Canada (section 24).
//!
//! Aucune chaîne visible par l'utilisateur ne doit être écrite ailleurs que dans
//! ce fichier et dans `aide.rs` (R-52). Un texte fixe est un `Texte` : il porte
//! ses deux versions, et se lit dans la langue en cours par `traduit()`.

use noyau::Touche;
use noyau::langue::{self, Langue, Texte, selon, t, tel};
use noyau::moteur::TOUCHE_RESULTAT;
use noyau::prefs::{
    AffichageAires, AffichageVolumes, EmpannonsIrreguliers, ModeFractions, MurCintre, Ordre,
    SeparateurDecimal, Theme,
};
use noyau::unites::UniteMasseVolumique;

/// Nom de l'application. Défini ici seulement, pour être changé facilement.
pub const NOM_APPLICATION: Texte = tel("Constructo AI");

/// Identifiant du dossier de données de l'utilisateur.
pub const APP_ID: &str = "CalculatriceChantier";

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
/// Révision du code source : l'identifiant court du commit, inscrit à la
/// compilation par `app/build.rs`. Vide quand elle n'est pas connue.
pub const REVISION: &str = env!("REVISION");

// ----------------------------------------------------------------------
// Langue (section 24)
// ----------------------------------------------------------------------

/// Préférences : le choix de la langue.
pub const LANGUE: Texte = t("Langue", "Language");

/// Infobulle du bouton de langue : la langue en cours, et ce que fait le bouton.
pub fn infobulle_de_langue(en_cours: Langue) -> String {
    match en_cours {
        Langue::Francais => {
            "Langue : français — appuyez sur EN pour l'anglais (English)".to_string()
        }
        Langue::Anglais => "Language: English — press FR for French (français)".to_string(),
    }
}

// ----------------------------------------------------------------------
// Clavier (section 5.1)
// ----------------------------------------------------------------------

/// Libellé principal d'une touche. Un saut de ligne sépare les libellés sur deux lignes.
pub fn libelle(touche: Touche, trigo: bool, point_decimal: bool) -> String {
    let fixe = match touche {
        // Les touches des champs d'un outil de métier tiennent leur libellé
        // de l'outil (section 23).
        Touche::Outil(TOUCHE_RESULTAT) => RESULTAT.traduit(),
        Touche::Outil(_) => "",
        Touche::Pente => selon("Pente", "Pitch"),
        Touche::Montee => selon("Montée", "Rise"),
        Touche::Course => selon("Course", "Run"),
        Touche::Diag => "Diag",
        Touche::AretierNoue => selon("Arêtier\nNoue", "Hip\nValley"),
        Touche::OngletCompose => selon("Onglet\ncomposé", "Compound\nMitre"),
        Touche::Escalier => selon("Escalier", "Stair"),
        Touche::Arc => "Arc",
        Touche::Cercle => selon("Cercle", "Circle"),
        Touche::Empannon => selon("Empannon", "Jack"),
        Touche::Metres => "m",
        Touche::Longueur if trigo => "sin",
        Touche::Largeur if trigo => "cos",
        Touche::Hauteur if trigo => "tan",
        Touche::Longueur => selon("Longueur", "Length"),
        Touche::Largeur => selon("Largeur", "Width"),
        Touche::Hauteur => selon("Hauteur", "Height"),
        Touche::Pourcent => "%",
        Touche::Verges => selon("Verges", "Yards"),
        Touche::Pieds => selon("Pieds", "Feet"),
        Touche::Pouces => selon("Pouces", "Inch"),
        Touche::Fraction => "/",
        Touche::Effacer => selon("Effacer", "Clear"),
        Touche::Conv => "Conv",
        Touche::Chiffre(c) => return c.to_string(),
        Touche::Diviser => "÷",
        Touche::Stocker => selon("Stocker", "Store"),
        Touche::Multiplier => "×",
        Touche::Rappel => selon("Rappel", "Recall"),
        Touche::Soustraire => "−",
        Touche::MPlus => "M+",
        Touche::Virgule if point_decimal => ".",
        Touche::Virgule => ",",
        Touche::Egal => "=",
        Touche::Additionner => "+",
        Touche::RetourArriere => "⌫",
        Touche::FractionRapide { num, den } => return format!("{num}/{den}"),
    };
    fixe.to_string()
}

/// Étiquette secondaire (fonction obtenue avec `Conv`); vide s'il n'y en a pas.
pub fn libelle_secondaire(touche: Touche, trigo: bool) -> &'static str {
    match touche {
        Touche::Outil(TOUCHE_RESULTAT) => RECOMMENCER.traduit(),
        Touche::Pente => "Ratio",
        Touche::Montee => selon("Mur en pente", "Rake wall"),
        Touche::Course => selon("Polygone", "Polygon"),
        Touche::Diag => selon("Toiture", "Roof"),
        Touche::AretierNoue => selon("Pente irrég.", "Irreg. pitch"),
        Touche::OngletCompose => selon("Angle moulure", "Crown angle"),
        Touche::Escalier => selon("Contrem. max.", "Riser limit"),
        Touche::Arc => selon("Rayon", "Radius"),
        Touche::Cercle => selon("Colonne/Cône", "Column/Cone"),
        Touche::Empannon => selon("Emp. irrég.", "Irreg. jack"),
        Touche::Longueur if trigo => "sin⁻¹",
        Touche::Largeur if trigo => "cos⁻¹",
        Touche::Hauteur if trigo => "tan⁻¹",
        Touche::Longueur => selon("Blocs", "Blocks"),
        Touche::Largeur => selon("Semelle", "Footing"),
        Touche::Hauteur => selon("Gypse", "Drywall"),
        Touche::Pourcent => "x²",
        Touche::Effacer => "√x",
        Touche::Chiffre(7) => "cm",
        Touche::Chiffre(8) => selon("pmp", "bd ft"),
        Touche::Chiffre(9) => "mm",
        Touche::Diviser => "1/x",
        Touche::Stocker => selon("Préf.", "Prefs"),
        Touche::Chiffre(4) => "lb",
        Touche::Chiffre(5) => selon("Montants", "Studs"),
        Touche::Chiffre(6) => selon("t courte", "ton"),
        Touche::Multiplier => selon("Effacer tout", "Clear all"),
        Touche::Rappel => "MC",
        Touche::Chiffre(1) => "kg",
        Touche::Chiffre(2) => "Acre",
        Touche::Chiffre(3) => selon("t métr.", "tonne"),
        Touche::Soustraire => "+/−",
        Touche::MPlus => "M−",
        Touche::Chiffre(0) => selon("Coût", "Cost"),
        Touche::Virgule => selon("dms◄►déc", "dms◄►dec"),
        Touche::Egal => selon("Ruban", "Tape"),
        Touche::Additionner => "π",
        _ => "",
    }
}

/// Nom complet de la fonction principale (infobulles et accessibilité).
pub fn nom(touche: Touche, trigo: bool) -> String {
    let fixe = match touche {
        Touche::Outil(TOUCHE_RESULTAT) => RESULTAT.traduit(),
        Touche::Outil(_) => "",
        Touche::Pente => selon("Pente", "Pitch"),
        Touche::Montee => selon("Montée", "Rise"),
        Touche::Course => selon("Course", "Run"),
        Touche::Diag => selon("Diagonale", "Diagonal"),
        Touche::AretierNoue => selon("Arêtier ou noue", "Hip or valley"),
        Touche::OngletCompose => selon("Onglet composé", "Compound mitre"),
        Touche::Escalier => selon("Escalier", "Stair"),
        Touche::Arc => "Arc",
        Touche::Cercle => selon("Cercle", "Circle"),
        Touche::Empannon => selon("Empannons", "Jack rafters"),
        Touche::Metres => selon("Mètres", "Metres"),
        Touche::Longueur if trigo => selon("Sinus", "Sine"),
        Touche::Largeur if trigo => selon("Cosinus", "Cosine"),
        Touche::Hauteur if trigo => selon("Tangente", "Tangent"),
        Touche::Longueur => selon("Longueur", "Length"),
        Touche::Largeur => selon("Largeur", "Width"),
        Touche::Hauteur => selon("Hauteur", "Height"),
        Touche::Pourcent => selon("Pourcentage", "Percent"),
        Touche::Verges => selon("Verges", "Yards"),
        Touche::Pieds => selon("Pieds", "Feet"),
        Touche::Pouces => selon("Pouces", "Inches"),
        Touche::Fraction => selon("Barre de fraction", "Fraction bar"),
        Touche::Effacer => selon("Effacer", "Clear"),
        Touche::Conv => selon("Conversion", "Convert"),
        Touche::Chiffre(c) => return format!("{} {c}", selon("Chiffre", "Digit")),
        Touche::Diviser => selon("Diviser", "Divide"),
        Touche::Stocker => selon("Stocker", "Store"),
        Touche::Multiplier => selon("Multiplier", "Multiply"),
        Touche::Rappel => selon("Rappel", "Recall"),
        Touche::Soustraire => selon("Soustraire", "Subtract"),
        Touche::MPlus => selon("Ajouter à la mémoire", "Add to memory"),
        Touche::Virgule => selon("Séparateur décimal", "Decimal point"),
        Touche::Egal => selon("Égal", "Equals"),
        Touche::Additionner => selon("Additionner", "Add"),
        Touche::RetourArriere => selon("Retour arrière", "Backspace"),
        Touche::FractionRapide { num, den } => {
            return format!("{} {num}/{den}", selon("Fraction rapide", "Quick fraction"));
        }
    };
    fixe.to_string()
}

/// Nom complet de la fonction secondaire; vide s'il n'y en a pas.
pub fn nom_secondaire(touche: Touche, trigo: bool) -> &'static str {
    match touche {
        Touche::Outil(TOUCHE_RESULTAT) => selon(
            "Remettre les champs à leur valeur de départ",
            "Reset the fields to their starting values",
        ),
        Touche::Pente => selon("Ratio de pente", "Pitch ratio"),
        Touche::Montee => selon("Mur en pente", "Rake wall"),
        Touche::Course => selon("Polygone", "Polygon"),
        Touche::Diag => selon("Toiture", "Roof"),
        Touche::AretierNoue => selon("Pente irrégulière", "Irregular pitch"),
        Touche::OngletCompose => selon("Angle de la moulure", "Crown moulding angle"),
        Touche::Escalier => selon(
            "Escalier à contremarche maximale",
            "Stair with a maximum riser",
        ),
        Touche::Arc => selon("Rayon", "Radius"),
        Touche::Cercle => selon("Colonne et cône", "Column and cone"),
        Touche::Empannon => selon("Empannons irréguliers", "Irregular jack rafters"),
        Touche::Longueur if trigo => selon("Arc sinus", "Arcsine"),
        Touche::Largeur if trigo => selon("Arc cosinus", "Arccosine"),
        Touche::Hauteur if trigo => selon("Arc tangente", "Arctangent"),
        Touche::Longueur => selon("Blocs", "Blocks"),
        Touche::Largeur => selon("Semelle", "Footing"),
        Touche::Hauteur => selon("Gypse", "Drywall"),
        Touche::Pourcent => selon("Carré", "Square"),
        Touche::Effacer => selon("Racine carrée", "Square root"),
        Touche::Chiffre(7) => selon("Centimètres", "Centimetres"),
        Touche::Chiffre(8) => selon("Pieds mesure de planche", "Board feet"),
        Touche::Chiffre(9) => selon("Millimètres", "Millimetres"),
        Touche::Diviser => selon("Inverse", "Reciprocal"),
        Touche::Stocker => selon("Préférences", "Preferences"),
        Touche::Chiffre(4) => selon("Livres", "Pounds"),
        Touche::Chiffre(5) => selon("Montants", "Studs"),
        Touche::Chiffre(6) => selon("Tonnes courtes", "Short tons"),
        Touche::Multiplier => selon("Effacer tout", "Clear all"),
        Touche::Rappel => selon("Effacer la mémoire", "Clear memory"),
        Touche::Chiffre(1) => selon("Kilogrammes", "Kilograms"),
        Touche::Chiffre(2) => "Acres",
        Touche::Chiffre(3) => selon("Tonnes métriques", "Metric tonnes"),
        Touche::Soustraire => selon("Changer le signe", "Change sign"),
        Touche::MPlus => selon("Soustraire de la mémoire", "Subtract from memory"),
        Touche::Chiffre(0) => selon("Coût", "Cost"),
        Touche::Virgule => selon(
            "Degrés décimaux ou degrés-minutes-secondes",
            "Decimal degrees or degrees-minutes-seconds",
        ),
        Touche::Egal => selon("Ruban de calcul", "Calculation tape"),
        Touche::Additionner => "Pi",
        _ => "",
    }
}

/// Infobulle d'une touche : « Montée — avec Conv : Mur en pente — raccourci : T ».
pub fn infobulle(touche: Touche, trigo: bool, raccourci: Option<&str>) -> String {
    if touche == Touche::Outil(TOUCHE_RESULTAT) {
        return INFOBULLE_RESULTAT.to_string();
    }
    let mut texte = nom(touche, trigo);
    let secondaire = nom_secondaire(touche, trigo);
    if !secondaire.is_empty() {
        texte.push_str(selon(" — avec Conv : ", " — with Conv: "));
        texte.push_str(secondaire);
    }
    if let Some(r) = raccourci {
        texte.push_str(selon(" — raccourci : ", " — shortcut: "));
        texte.push_str(r);
    }
    texte
}

// ----------------------------------------------------------------------
// Afficheur (section 16.2)
// ----------------------------------------------------------------------

pub const INDICATEUR_CONV: Texte = tel("CONV");
pub const INDICATEUR_STOCKER: Texte = t("STOCKER", "STORE");
pub const INDICATEUR_RAPPEL: Texte = t("RAPPEL", "RECALL");
pub const INDICATEUR_MEMOIRE: Texte = tel("M");
pub const INDICATEUR_MEMORISE: Texte = t("MÉM.", "MEM");
pub const COPIE: Texte = t("Copié", "Copied");
pub const INFOBULLE_RETOUR_ARRIERE: Texte = t(
    "Retour arrière — raccourci : Retour arrière",
    "Backspace — shortcut: Backspace",
);
pub const INFOBULLE_VALEUR: Texte = t(
    "Double-clic : copier la valeur",
    "Double-click: copy the value",
);

// ----------------------------------------------------------------------
// Navigation (section 15.1)
// ----------------------------------------------------------------------

pub const ONGLET_ACCUEIL: Texte = t("Accueil", "Home");
pub const ONGLET_RUBANS: Texte = t("Rubans", "Tapes");
pub const ONGLET_AIDE: Texte = t("Aide", "Help");

pub const MENU_RUBANS: Texte = t("Rubans enregistrés", "Saved tapes");
pub const MENU_AIDE: Texte = t("Aide", "Help");
pub const MENU_MANUEL: Texte = t("Manuel d'utilisation", "User manual");
pub const MENU_BIENVENUE: Texte = t("Bienvenue", "Welcome");
pub const MENU_TRUCS: Texte = t("Trucs de pro", "Pro tips");
pub const MENU_PREFERENCES: Texte = t("Préférences", "Preferences");
pub const MENU_A_PROPOS: Texte = t("À propos", "About");
pub const MENU_QUITTER: Texte = t("Quitter", "Quit");
pub const MENU_METIERS: Texte = t("Calculateurs par métier", "Trade calculators");
/// Version Web : installer l'application sur le téléphone, la tablette ou l'ordinateur.
pub const MENU_INSTALLER: Texte = t("Installer l'application", "Install the app");
// Les espaces insécables gardent chaque guillemet collé à son texte.
pub const AVIS_INSTALLER: Texte = t(
    "Pour installer la calculatrice sur cet appareil, passez par le menu de votre navigateur.\n\nSur iPhone ou iPad (Safari) : touchez Partager, puis «\u{00A0}Sur l'écran d'accueil\u{00A0}».\n\nSur Android (Chrome) : touchez les trois points, puis «\u{00A0}Installer l'application\u{00A0}» ou «\u{00A0}Ajouter à l'écran d'accueil\u{00A0}».\n\nUne fois installée, elle s'ouvre comme une application et fonctionne sans réseau.",
    "To install the calculator on this device, use your browser's menu.\n\nOn iPhone or iPad (Safari): tap Share, then \"Add to Home Screen\".\n\nOn Android (Chrome): tap the three dots, then \"Install app\" or \"Add to Home screen\".\n\nOnce installed, it opens like an app and works offline.",
);

pub const INFOBULLE_MENU: Texte = tel("Menu");
pub const NOM_MENU: Texte = tel("Menu");
pub const NOM_PLEIN_ECRAN: Texte = t("Plein écran", "Full screen");
pub const INFOBULLE_PLEIN_ECRAN: Texte = t(
    "Plein écran — raccourci : F11",
    "Full screen — shortcut: F11",
);
pub const INFOBULLE_PREFERENCES: Texte = t(
    "Préférences — raccourci : Ctrl+,",
    "Preferences — shortcut: Ctrl+,",
);

pub const TERMINE: Texte = t("Terminé", "Done");
pub const FERMER: Texte = t("Fermer", "Close");

// --- Écran « Dessin » (section 25)
pub const ONGLET_DESSIN: Texte = t("Dessin", "Sketch");
pub const MENU_DESSIN: Texte = t("Dessiner une pièce", "Sketch a room");
pub const DESSIN_TITRE: Texte = t("Dessin d'une pièce", "Room sketch");
pub const DESSIN_PLANCHE: Texte = t("Planche à dessin", "Drawing board");
pub const DESSIN_INVITE: Texte = t(
    "Tracez le contour ici, ou avec les flèches",
    "Draw the outline here, or with the arrows",
);
pub const DESSIN_FLECHE_GAUCHE: Texte = t("Côté vers la gauche", "Side to the left");
pub const DESSIN_FLECHE_HAUT: Texte = t("Côté vers le haut", "Side upward");
pub const DESSIN_FLECHE_BAS: Texte = t("Côté vers le bas", "Side downward");
pub const DESSIN_FLECHE_DROITE: Texte = t("Côté vers la droite", "Side to the right");
pub const DESSIN_FLECHE_HAUT_GAUCHE: Texte = t(
    "Côté en biais vers le haut et la gauche",
    "Angled side up and to the left",
);
pub const DESSIN_FLECHE_HAUT_DROITE: Texte = t(
    "Côté en biais vers le haut et la droite",
    "Angled side up and to the right",
);
pub const DESSIN_FLECHE_BAS_GAUCHE: Texte = t(
    "Côté en biais vers le bas et la gauche",
    "Angled side down and to the left",
);
pub const DESSIN_FLECHE_BAS_DROITE: Texte = t(
    "Côté en biais vers le bas et la droite",
    "Angled side down and to the right",
);
pub const DESSIN_FLECHES_EN_BIAIS: Texte = t("Flèches en biais, à 45°", "Angled arrows, at 45°");
pub const DESSIN_45_DEGRES: Texte = tel("45°");
pub const DESSIN_FERMER_LA_FORME: Texte = t("Fermer la forme", "Close the shape");
pub const DESSIN_RETIRER: Texte = t("Retirer le dernier côté", "Remove last side");
pub const DESSIN_EFFACER: Texte = t("Tout effacer", "Clear all");
pub const DESSIN_ANNULER: Texte = t("Annuler", "Undo");
pub const DESSIN_AU_RUBAN: Texte = t("Ajouter au ruban", "Add to tape");
pub const DESSIN_VERS_UN_OUTIL: Texte = t("Envoyer vers un outil…", "Send to a tool…");
pub const DESSIN_RETRAITS: Texte = t(
    "Retraits (îlot, trémie, colonne)",
    "Deductions (island, stairwell, column)",
);
pub const DESSIN_AJOUTER_UN_RETRAIT: Texte = t("Ajouter un retrait", "Add a deduction");
pub const DESSIN_ENLEVER: Texte = t("Enlever", "Remove");
pub const DESSIN_LONGUEUR: Texte = t("longueur", "length");
pub const DESSIN_LARGEUR: Texte = t("largeur", "width");
pub const DESSIN_SURFACE_NETTE: Texte = t("Surface nette", "Net area");
pub const DESSIN_MURS: Texte = t("Murs (peinture, gypse)", "Walls (paint, drywall)");
pub const DESSIN_HAUTEUR_DES_MURS: Texte = t("Hauteur des murs", "Wall height");
pub const DESSIN_HAUTEUR: Texte = t("hauteur", "height");
pub const DESSIN_HAUTEUR_EXEMPLE: Texte = t("8 pi 0 po", "8 ft 0 in");
pub const DESSIN_AJOUTER_PORTE: Texte = t("+ Porte", "+ Door");
pub const DESSIN_AJOUTER_FENETRE: Texte = t("+ Fenêtre", "+ Window");
pub const DESSIN_SURFACE_DES_MURS: Texte = t("Surface des murs", "Wall area");
pub const DESSIN_PORTES_ET_FENETRES: Texte = t("Portes et fenêtres", "Doors and windows");
pub const DESSIN_MURS_NETS: Texte = t("Surface nette des murs", "Net wall area");
pub const DESSIN_HAUTEUR_INVALIDE: Texte = t(
    "Hauteur illisible. Exemple : 8 pi 0 po.",
    "Unreadable height. Example: 8 ft 0 in.",
);
pub const DESSIN_OUVERTURE_INCOMPLETE: Texte = t(
    "Une porte ou une fenêtre est incomplète : inscrivez sa largeur et sa hauteur.",
    "A door or window is incomplete: enter its width and its height.",
);
pub const DESSIN_OUVERTURES_TROP_GRANDES: Texte = t(
    "Les portes et les fenêtres dépassent la surface des murs.",
    "The doors and windows exceed the wall area.",
);

/// « Ouverture 1 : largeur ».
pub fn dessin_ouverture(rang: usize, dimension: &str) -> String {
    match langue::en_cours() {
        Langue::Francais => format!("Ouverture {rang} : {dimension}"),
        Langue::Anglais => format!("Opening {rang}: {dimension}"),
    }
}

pub const DESSIN_PORTE: Texte = t("Porte", "Door");
pub const DESSIN_FENETRE: Texte = t("Fenêtre", "Window");
pub const DESSIN_PLACER: Texte = t("Placer", "Place");
pub const DESSIN_PAS_PLACEE: Texte = t("pas placée", "not placed");
pub const DESSIN_CENTREE: Texte = t("centrée", "centred");
pub const DESSIN_POSITION: Texte = t("distance du coin A", "distance from corner A");
pub const DESSIN_POSITION_EXEMPLE: Texte = t("vide : centrée", "empty: centred");
pub const DESSIN_ALLEGE: Texte = t("allège", "sill height");
pub const DESSIN_ALLEGE_DEFAUT: Texte = t("3 pi 0 po", "3 ft 0 in");
pub const DESSIN_ALLEGE_EXEMPLE: Texte = t("3 pi 0 po (vide : 3 pi)", "3 ft 0 in (empty: 3 ft)");
pub const DESSIN_3D: Texte = tel("3D");
pub const DESSIN_2D: Texte = tel("2D");
pub const DESSIN_VOIR_EN_3D: Texte = t("Voir la pièce en 3D", "View the room in 3D");
pub const DESSIN_REVENIR_AU_PLAN: Texte = t("Revenir au plan", "Back to the plan");
pub const DESSIN_TOURNER: Texte = t("Glissez pour tourner", "Drag to turn");
pub const DESSIN_MURS_PAR_DEFAUT: Texte = t("murs de 8 pi par défaut", "8 ft walls by default");

/// « allège 3 pi 0 po » : le bas d'une fenêtre.
pub fn dessin_allege(hauteur: &str) -> String {
    match langue::en_cours() {
        Langue::Francais => format!("allège {hauteur}"),
        Langue::Anglais => format!("sill {hauteur}"),
    }
}

/// « côté 3 » : le mur d'une ouverture.
pub fn dessin_sur_le_cote(rang: usize) -> String {
    match langue::en_cours() {
        Langue::Francais => format!("côté {rang}"),
        Langue::Anglais => format!("side {rang}"),
    }
}

/// « à 3 pi 0 po du coin ».
pub fn dessin_a_du_coin(distance: &str) -> String {
    match langue::en_cours() {
        Langue::Francais => format!("à {distance} du coin A"),
        Langue::Anglais => format!("{distance} from corner A"),
    }
}

/// La consigne affichée sur la planche quand une ouverture attend son mur.
pub fn dessin_toucher_le_mur(porte: bool) -> &'static str {
    match (langue::en_cours(), porte) {
        (Langue::Francais, true) => "Touchez le mur de la porte",
        (Langue::Francais, false) => "Touchez le mur de la fenêtre",
        (Langue::Anglais, true) => "Tap the wall for the door",
        (Langue::Anglais, false) => "Tap the wall for the window",
    }
}
pub const DESSIN_AJOUTER_UNE_PIECE: Texte = t("+ Pièce", "+ Room");
pub const DESSIN_NOM_DE_LA_PIECE: Texte = t("Nom de la pièce", "Room name");
pub const DESSIN_SUPPRIMER_LA_PIECE: Texte = t("Supprimer la pièce", "Delete the room");

/// « Pièce 2 » : le nom d'une pièce qui n'en a pas reçu.
pub fn dessin_piece(rang: usize) -> String {
    match langue::en_cours() {
        Langue::Francais => format!("Pièce {rang}"),
        Langue::Anglais => format!("Room {rang}"),
    }
}

/// Titre du total des pièces.
pub fn dessin_total(completes: usize, nombre: usize) -> String {
    match langue::en_cours() {
        Langue::Francais if completes == nombre => format!("Total des {nombre} pièces"),
        Langue::Francais => format!("Total des pièces complètes ({completes} de {nombre})"),
        Langue::Anglais if completes == nombre => format!("Total of the {nombre} rooms"),
        Langue::Anglais => format!("Total of the complete rooms ({completes} of {nombre})"),
    }
}
pub const DESSIN_RETRAIT_INCOMPLET: Texte = t(
    "Un retrait est incomplet : inscrivez sa longueur et sa largeur.",
    "A deduction is incomplete: enter its length and its width.",
);
pub const DESSIN_RETRAITS_TROP_GRANDS: Texte = t(
    "Les retraits dépassent la surface de la pièce.",
    "The deductions exceed the area of the room.",
);

/// « Retrait 1 : longueur ».
pub fn dessin_retrait(rang: usize, en_largeur: bool) -> String {
    let dimension = if en_largeur {
        DESSIN_LARGEUR
    } else {
        DESSIN_LONGUEUR
    };
    match langue::en_cours() {
        Langue::Francais => format!("Retrait {rang} : {}", dimension.traduit()),
        Langue::Anglais => format!("Deduction {rang}: {}", dimension.traduit()),
    }
}

/// Ce que l'outil choisi va recevoir.
pub fn dessin_rectangle_equivalent(longueur: &str, largeur: &str, exact: bool) -> String {
    match (langue::en_cours(), exact) {
        (Langue::Francais, true) => format!(
            "L'outil reçoit un rectangle de même surface et de même périmètre que la pièce : {longueur} sur {largeur}. Choisissez l'outil :"
        ),
        (Langue::Francais, false) => format!(
            "L'outil reçoit un carré de même surface que la pièce : {longueur} sur {largeur}. Son périmètre n'est pas celui de la pièce. Choisissez l'outil :"
        ),
        (Langue::Anglais, true) => format!(
            "The tool gets a rectangle with the same area and perimeter as the room: {longueur} by {largeur}. Pick the tool:"
        ),
        (Langue::Anglais, false) => format!(
            "The tool gets a square with the same area as the room: {longueur} by {largeur}. Its perimeter is not the room's. Pick the tool:"
        ),
    }
}
pub const DESSIN_DIAGONALE: Texte = t("Diagonale d'équerre", "Square-up diagonal");
pub const DESSIN_EXEMPLE: Texte = t("12 pi 6 po", "12 ft 6 in");
pub const DESSIN_COTE_SUIVANT: Texte = t("Suivant", "Next");
pub const DESSIN_TOUCHE_ESPACE: Texte = t("esp.", "space");
pub const DESSIN_TOUCHE_PIEDS: Texte = t("Pieds", "Feet");
pub const DESSIN_ZOOMER: Texte = t("Zoomer", "Zoom in");
pub const DESSIN_DEZOOMER: Texte = t("Dézoomer", "Zoom out");
pub const DESSIN_AJUSTER: Texte = t("Ajuster", "Fit");
pub const DESSIN_EXPORTER: Texte = t("Exporter le plan", "Export the plan");
pub const DESSIN_EXPORTER_PDF: Texte = t("Exporter en PDF", "Export as PDF");
pub const DESSIN_EXPORTER_DXF: Texte = t("Exporter en DXF", "Export as DXF");
pub const DESSIN_DXF_IMPOSSIBLE: Texte = t(
    "Le DXF demande au moins une pièce complète, à l'échelle de ses mesures.",
    "DXF needs at least one complete room, drawn to its measurements.",
);
pub const TITRE_EXPORTER_DESSIN: Texte = t("Exporter le plan", "Export the plan");
pub const FILTRE_PDF: Texte = t("Document PDF", "PDF document");
pub const FILTRE_DXF: Texte = t("Dessin DXF", "DXF drawing");
pub const AVIS_DESSIN_EXPORTE: Texte = t("Plan exporté.", "Plan exported.");
pub const AVIS_DESSIN_EXPORT_ECHEC: Texte = t(
    "Le plan n'a pas pu être exporté.",
    "The plan could not be exported.",
);
pub const DESSIN_EXPORT_CROQUIS: Texte = t(
    "Croquis : mesures incomplètes, pas à l'échelle.",
    "Sketch: measurements incomplete, not to scale.",
);
pub const DESSIN_EXPORT_COTES: Texte = t("Côtés", "Sides");
pub const DESSIN_EXPORT_DEDUITE: Texte = t("déduite", "worked out");
pub const DESSIN_EXPORT_PIECE: Texte = t("Pièce", "Room");
pub const DESSIN_EXPORT_INCOMPLETE: Texte = t("incomplète", "incomplete");
pub const DESSIN_EXPORT_TOTAL: Texte = t("Total des surfaces nettes", "Total net area");
pub const DESSIN_EXPORT_PIED: Texte = tel("Constructo AI · constructoai.ca");

/// « Retrait 2 », dans la liste des résultats d'une pièce exportée.
pub fn dessin_retrait_numero(rang: usize) -> String {
    match langue::en_cours() {
        Langue::Francais => format!("Retrait {rang}"),
        Langue::Anglais => format!("Deduction {rang}"),
    }
}

/// En-tête des pages du plan exporté.
pub fn dessin_export_entete(date: &str) -> String {
    match langue::en_cours() {
        Langue::Francais => format!("Constructo AI · Dessin des pièces · {date}"),
        Langue::Anglais => format!("Constructo AI · Room sketch · {date}"),
    }
}

/// « Page 2 de 3 ».
pub fn dessin_export_page(numero: usize, total: usize) -> String {
    match langue::en_cours() {
        Langue::Francais => format!("Page {numero} de {total}"),
        Langue::Anglais => format!("Page {numero} of {total}"),
    }
}

/// Titre du plan exporté : « Plan : 3 pièces ».
pub fn dessin_export_plan(pieces: usize) -> String {
    match (langue::en_cours(), pieces) {
        (Langue::Francais, 0 | 1) => format!("Plan : {pieces} pièce"),
        (Langue::Francais, _) => format!("Plan : {pieces} pièces"),
        (Langue::Anglais, 1) => "Plan: 1 room".to_string(),
        (Langue::Anglais, _) => format!("Plan: {pieces} rooms"),
    }
}

/// Nom proposé pour le fichier du plan exporté (sans l'extension).
pub fn dessin_nom_de_fichier(jour: &str) -> String {
    match langue::en_cours() {
        Langue::Francais => format!("Plan {jour}"),
        Langue::Anglais => format!("Plan {jour}"),
    }
}
pub const DESSIN_TOUCHE_POUCES: Texte = t("Pouces", "Inches");
pub const DESSIN_TOUCHE_RECULER: Texte = t("eff.", "del");
pub const DESSIN_SURFACE: Texte = t("Surface", "Area");
pub const DESSIN_PERIMETRE: Texte = t("Périmètre", "Perimeter");
pub const DESSIN_MESURE_INVALIDE: Texte = t(
    "Mesure illisible. Exemples : 12 pi 6 po, 150 po, 3,81 m.",
    "Unreadable measurement. Examples: 12 ft 6 in, 150 in, 3.81 m.",
);
pub const DESSIN_MESURE_DEDUITE: Texte = t(
    "Déduite des autres côtés. Inscrivez-la pour la vérifier.",
    "Worked out from the other sides. Enter it to double-check.",
);
pub const DESSIN_ETAPE_TRACER: Texte = t(
    "Tracez le contour de la pièce avec le doigt ou la souris, d'un seul geste ou un côté à la fois, ou avec les flèches du bas de la planche : chacune ajoute un côté dans sa direction. Le croquis n'a pas besoin d'être à l'échelle : les angles sont mis d'équerre, ou à 45° pour un mur en biais (le bouton 45° fait passer les flèches en biais).",
    "Draw the outline of the room with your finger or the mouse, in one stroke or one side at a time, or with the arrows at the bottom of the board: each one adds a side in its direction. The sketch does not need to be to scale: corners are squared up, or set at 45° for an angled wall (the 45° button switches the arrows to angled).",
);
pub const DESSIN_ETAPE_FERMER: Texte = t(
    "Continuez le tracé jusqu'au point de départ (le cercle orange), ou touchez « Fermer la forme ».",
    "Keep drawing back to the starting point (the orange circle), or tap \"Close the shape\".",
);
pub const DESSIN_SE_CROISE: Texte = t(
    "Avec ces mesures, des côtés se croisent. Vérifiez-les.",
    "With these measurements, some sides cross. Please check them.",
);

/// « Côté 2 de 6 ».
pub fn dessin_cote(rang: usize, nombre: usize) -> String {
    match langue::en_cours() {
        Langue::Francais => format!("Côté {rang} de {nombre}"),
        Langue::Anglais => format!("Side {rang} of {nombre}"),
    }
}

/// Consigne quand il manque encore des mesures.
pub fn dessin_etape_mesurer(manquantes: usize) -> String {
    match langue::en_cours() {
        Langue::Francais => format!(
            "Touchez un côté, puis inscrivez sa mesure. Un nombre seul est en pieds. Mesures qui manquent encore : {manquantes}."
        ),
        Langue::Anglais => format!(
            "Tap a side, then enter its measurement. A number alone is in feet. Measurements still missing: {manquantes}."
        ),
    }
}

/// Message quand les mesures inscrites ne ferment pas la forme.
pub fn dessin_ne_ferme_pas(ecart: &str) -> String {
    match langue::en_cours() {
        Langue::Francais => format!(
            "Les mesures ne ferment pas la forme : il y a un écart de {ecart} entre des côtés opposés."
        ),
        Langue::Anglais => {
            format!("The measurements do not close the shape: opposite sides are off by {ecart}.")
        }
    }
}
pub const RETOUR: Texte = t("Retour", "Back");
pub const ANNULER: Texte = t("Annuler", "Cancel");
pub const ENREGISTRER: Texte = t("Enregistrer", "Save");
pub const OUVRIR: Texte = t("Ouvrir", "Open");
pub const RENOMMER: Texte = t("Renommer", "Rename");
pub const COPIER: Texte = t("Copier", "Copy");
pub const EXPORTER: Texte = t("Exporter", "Export");
pub const SUPPRIMER: Texte = t("Supprimer", "Delete");
pub const VIDER: Texte = t("Vider", "Clear");
pub const COMMENCER: Texte = t("Commencer", "Get started");

// ----------------------------------------------------------------------
// Messages et confirmations (section 4.5)
// ----------------------------------------------------------------------

pub const TITRE_EFFACER_TOUT: Texte = t("Effacer tout", "Clear all");
pub const CONFIRMER_EFFACER_TOUT: Texte = t(
    "Effacer toutes les valeurs mémorisées et les remettre aux valeurs par défaut? Les préférences sont conservées.",
    "Clear all stored values and restore their defaults? Preferences are kept.",
);
pub const BOUTON_EFFACER_TOUT: Texte = t("Effacer tout", "Clear all");

pub const TITRE_REINITIALISER: Texte = t("Réinitialisation", "Reset");
pub const CONFIRMER_REINITIALISER: Texte = t(
    "Réinitialiser toutes les données et toutes les préférences?",
    "Reset all data and all preferences?",
);
pub const BOUTON_REINITIALISER: Texte = t("Réinitialiser", "Reset");

pub const AVIS_REGLAGES_ILLISIBLES: Texte = t(
    "Les réglages n'ont pas pu être lus; les valeurs par défaut ont été rétablies.",
    "The settings could not be read; the defaults have been restored.",
);
pub const AVIS_ECRITURE_IMPOSSIBLE: Texte = t(
    "Les données n'ont pas pu être enregistrées sur le disque.",
    "The data could not be saved to disk.",
);
/// Version Web : le stockage local du navigateur est plein ou refusé.
pub const AVIS_ECRITURE_IMPOSSIBLE_WEB: Texte = t(
    "Les données n'ont pas pu être enregistrées dans ce navigateur. La calculatrice fonctionne, mais vos réglages et vos rubans ne seront pas conservés.",
    "The data could not be saved in this browser. The calculator works, but your settings and tapes will not be kept.",
);
pub const AVIS_COLLAGE_REFUSE: Texte = t(
    "Le texte collé n'est ni un nombre ni une longueur.",
    "The pasted text is neither a number nor a length.",
);
pub const AVIS_EXPORT_REUSSI: Texte = t("Ruban exporté.", "Tape exported.");
pub const AVIS_EXPORT_ECHEC: Texte = t(
    "Le fichier n'a pas pu être écrit.",
    "The file could not be written.",
);
pub const AVIS_RUBAN_ENREGISTRE: Texte = t("Ruban enregistré.", "Tape saved.");
pub const AVIS_RUBAN_COPIE: Texte = t(
    "Ruban copié dans le presse-papiers.",
    "Tape copied to the clipboard.",
);
pub const AVIS_RUBAN_VIDE: Texte = t("Le ruban est vide.", "The tape is empty.");
pub const OK: Texte = tel("OK");

// ----------------------------------------------------------------------
// Préférences (section 13)
// ----------------------------------------------------------------------

pub const PREFERENCES: Texte = t("Préférences", "Preferences");
pub const SECTION_GENERAL: Texte = t("Général", "General");
pub const SECTION_AFFICHAGE: Texte = t("Affichage", "Display");
pub const SECTION_FONCTIONS: Texte = t("Fonctions", "Functions");
pub const SECTION_MEMORISEES: Texte = t("Valeurs mémorisées", "Stored values");
pub const SECTION_REINITIALISATION: Texte = t("Réinitialisation", "Reset");

pub const PREF_TRIGO: Texte = t("Mode trigonométrique", "Trigonometric mode");
pub const PREF_TRIGO_AIDE: Texte = t(
    "Longueur, Largeur et Hauteur deviennent sin, cos et tan.",
    "Length, Width and Height become sin, cos and tan.",
);
pub const PREF_FRACTIONS_RAPIDES: Texte = t("Fractions rapides", "Quick fractions");
pub const PREF_FRACTIONS_RAPIDES_AIDE: Texte = t(
    "Affiche une bande de fractions de pouce au-dessus du clavier.",
    "Shows a strip of inch fractions above the keypad.",
);
pub const PREF_PLEIN_ECRAN: Texte = t("Mode plein écran", "Full-screen mode");
pub const PREF_PLEIN_ECRAN_AIDE: Texte = t(
    "Fenêtre en plein écran, sans bordure (aussi F11).",
    "Borderless full-screen window (also F11).",
);
pub const PREF_THEME: Texte = t("Thème", "Theme");

pub const PREF_RESOLUTION: Texte = t("Résolution des fractions", "Fraction resolution");
pub const PREF_MODE_FRACTIONS: Texte = t("Mode des fractions", "Fraction mode");
pub const PREF_EXPONENTIELLE: Texte = t("Notation exponentielle", "Exponential notation");
pub const PREF_EXPONENTIELLE_AIDE: Texte = t(
    "Affiche les très grands et les très petits nombres en notation scientifique.",
    "Shows very large and very small numbers in scientific notation.",
);
pub const PREF_AIRES: Texte = t("Affichage des aires", "Area display");
pub const PREF_VOLUMES: Texte = t("Affichage des volumes", "Volume display");
pub const PREF_METRES: Texte = t("Mètres (longueurs)", "Metres (lengths)");
pub const PREF_DEGRES: Texte = t("Degrés décimaux", "Decimal degrees");
pub const PREF_MILLIERS: Texte = t("Séparateur de milliers", "Thousands separator");
pub const PREF_MILLIERS_AIDE: Texte = t(
    "Une espace sépare les groupes de trois chiffres : 1 234,56.",
    "A comma separates groups of three digits: 1,234.56.",
);
pub const PREF_DECIMAL: Texte = t("Séparateur décimal", "Decimal separator");
pub const PREF_DECIMALES: Texte = t(
    "Décimales (valeurs sans unité)",
    "Decimals (unitless values)",
);

pub const PREF_MUR_PENTE: Texte = t("Mur en pente", "Rake wall");
pub const PREF_MUR_CINTRE: Texte = t("Mur cintré", "Arched wall");
pub const PREF_EMPANNONS: Texte = t("Empannons", "Jack rafters");
pub const PREF_EMPANNONS_IRREGULIERS: Texte = t("Empannons irréguliers", "Irregular jack rafters");

pub const FLOTTANT: Texte = t("Flottant", "Floating");
pub const METRES_FIXES: Texte = t("0,000", "0.000");
pub const DEGRES_FIXES: Texte = t("0,00", "0.00");

pub fn theme(theme: Theme) -> &'static str {
    match theme {
        Theme::Automatique => selon("Automatique", "Automatic"),
        Theme::Clair => selon("Clair", "Light"),
        Theme::Sombre => selon("Sombre", "Dark"),
    }
}

pub fn resolution(denominateur: u32) -> String {
    format!("1/{denominateur}")
}

pub fn mode_fractions(m: ModeFractions) -> &'static str {
    match m {
        ModeFractions::Standard => {
            selon("Standard (fraction réduite)", "Standard (reduced fraction)")
        }
        ModeFractions::Constant => selon(
            "Constant (dénominateur fixe)",
            "Constant (fixed denominator)",
        ),
    }
}

pub fn affichage_aires(a: AffichageAires) -> &'static str {
    match a {
        AffichageAires::Standard => "Standard",
        AffichageAires::PiedsCarres => selon("Pieds carrés", "Square feet"),
        AffichageAires::VergesCarrees => selon("Verges carrées", "Square yards"),
        AffichageAires::MetresCarres => selon("Mètres carrés", "Square metres"),
    }
}

pub fn affichage_volumes(v: AffichageVolumes) -> &'static str {
    match v {
        AffichageVolumes::Standard => "Standard",
        AffichageVolumes::VergesCubes => selon("Verges cubes", "Cubic yards"),
        AffichageVolumes::PiedsCubes => selon("Pieds cubes", "Cubic feet"),
        AffichageVolumes::MetresCubes => selon("Mètres cubes", "Cubic metres"),
    }
}

pub fn separateur_decimal(s: SeparateurDecimal) -> &'static str {
    match s {
        SeparateurDecimal::Virgule => selon("Virgule", "Comma"),
        SeparateurDecimal::Point => "Point",
    }
}

pub fn decimales(d: Option<u8>) -> String {
    match d {
        None => FLOTTANT.to_string(),
        Some(n) => n.to_string(),
    }
}

pub fn ordre(o: Ordre) -> &'static str {
    match o {
        Ordre::Decroissant => selon(
            "Décroissant (du plus long au plus court)",
            "Descending (longest to shortest)",
        ),
        Ordre::Croissant => selon(
            "Croissant (du plus court au plus long)",
            "Ascending (shortest to longest)",
        ),
    }
}

pub fn mur_cintre(m: MurCintre) -> &'static str {
    match m {
        MurCintre::Exterieur => selon("Extérieur (au-dessus de l'arc)", "Outside (above the arc)"),
        MurCintre::Interieur => selon("Intérieur (sous l'arc)", "Inside (below the arc)"),
    }
}

pub fn empannons_irreguliers(e: EmpannonsIrreguliers) -> &'static str {
    match e {
        EmpannonsIrreguliers::EntraxeConstant => {
            selon("Entraxe constant", "Constant on-centre spacing")
        }
        EmpannonsIrreguliers::Apparies => selon("Appariés", "Mated"),
    }
}

pub fn unite_masse_volumique(u: UniteMasseVolumique) -> &'static str {
    noyau::etiquettes::symbole_masse_volumique(u)
}

// Valeurs mémorisées (section 13.4)
pub const MEMO_ECHAPPEE: Texte = t("Échappée", "Headroom");
pub const MEMO_CONTREMARCHE: Texte = t("Hauteur de contremarche visée", "Target riser height");
pub const MEMO_PLANCHER: Texte = t("Épaisseur du plancher", "Floor thickness");
pub const MEMO_GIRON: Texte = t("Giron visé", "Target tread depth");
pub const MEMO_AIRE_BLOC: Texte = t("Aire d'un bloc", "Block area");
pub const MEMO_LONGUEUR_BLOC: Texte = t("Longueur d'un bloc", "Block length");
pub const MEMO_ANGLE_MOULURE: Texte = t("Angle de la moulure", "Crown moulding angle");
pub const MEMO_ENTRAXE: Texte = t("Entraxe", "On-centre spacing");
pub const MEMO_SECTION_SEMELLE: Texte = t("Section de semelle", "Footing section");
pub const MEMO_MASSE_VOLUMIQUE: Texte = t("Masse volumique", "Density");
pub const MEMO_PAQUETS: Texte = t(
    "Paquets de bardeaux par carré",
    "Shingle bundles per square",
);

pub const MEMO_USAGE_ECHAPPEE: Texte = t("Trémie d'escalier", "Stairwell opening");
pub const MEMO_USAGE_CONTREMARCHE: Texte = t(
    "Escalier — aussi par Stocker 7",
    "Stair — also with Store 7",
);
pub const MEMO_USAGE_PLANCHER: Texte = t(
    "Trémie d'escalier — aussi par Stocker 8",
    "Stairwell opening — also with Store 8",
);
pub const MEMO_USAGE_GIRON: Texte = t(
    "Escalier — aussi par Stocker 9",
    "Stair — also with Store 9",
);
pub const MEMO_USAGE_AIRE_BLOC: Texte =
    t("Blocs — aussi par Stocker 4", "Blocks — also with Store 4");
pub const MEMO_USAGE_LONGUEUR_BLOC: Texte =
    t("Blocs — aussi par Stocker 4", "Blocks — also with Store 4");
pub const MEMO_USAGE_ANGLE_MOULURE: Texte = t(
    "Onglet composé — aussi par Stocker Onglet composé",
    "Compound mitre — also with Store Compound Mitre",
);
pub const MEMO_USAGE_ENTRAXE: Texte = t(
    "Empannons, murs, montants — aussi par Stocker 5",
    "Jack rafters, walls, studs — also with Store 5",
);
pub const MEMO_USAGE_SECTION_SEMELLE: Texte = t(
    "Semelle — aussi par Stocker 6",
    "Footing — also with Store 6",
);
pub const MEMO_USAGE_MASSE_VOLUMIQUE: Texte = t(
    "Volume ↔ masse — aussi par Stocker 0",
    "Volume ↔ weight — also with Store 0",
);
pub const MEMO_USAGE_PAQUETS: Texte = t("Toiture", "Roof");

pub const VALEUR_PAR_DEFAUT: Texte = t("Valeur par défaut", "Default value");
pub const VALEUR_NON_RECONNUE: Texte = t("Valeur non reconnue", "Value not recognized");
pub const MODIFIER: Texte = t("Modifier", "Edit");
pub const UNITE: Texte = t("Unité", "Unit");
pub const EXEMPLES_LONGUEUR: Texte = t(
    "Exemples : 6'8\", 7 1/2 po, 2,032 m",
    "Examples: 6'8\", 7 1/2 in, 2.032 m",
);
pub const EXEMPLES_AIRE: Texte = t(
    "Exemples : 128 po², 1,83 pi², 0,17 m²",
    "Examples: 128 in², 1.83 ft², 0.17 m²",
);
pub const EXEMPLES_ANGLE: Texte = t(
    "Un angle en degrés, entre 0 et 90",
    "An angle in degrees, between 0 and 90",
);
pub const EXEMPLES_NOMBRE: Texte = t(
    "Un nombre plus grand que zéro",
    "A number greater than zero",
);

pub const BOUTON_REINITIALISER_TOUT: Texte = t(
    "Réinitialiser les données et les préférences",
    "Reset data and preferences",
);

// ----------------------------------------------------------------------
// Rubans (section 14)
// ----------------------------------------------------------------------

pub const RUBAN_COURANT: Texte = t("Ruban courant", "Current tape");
pub const RUBANS_ENREGISTRES: Texte = t("Rubans enregistrés", "Saved tapes");
pub const RUBAN_VIDE: Texte = t(
    "Le ruban est vide. Les calculs s'y inscriront au fur et à mesure.",
    "The tape is empty. Calculations are recorded here as you go.",
);
pub const AUCUN_RUBAN_ENREGISTRE: Texte = t("Aucun ruban enregistré.", "No saved tapes.");
pub const NOM_DU_RUBAN: Texte = t("Nom du ruban", "Tape name");
pub const TITRE_ENREGISTRER_RUBAN: Texte = t("Enregistrer le ruban", "Save the tape");
pub const TITRE_RENOMMER_RUBAN: Texte = t("Renommer le ruban", "Rename the tape");
pub const TITRE_VIDER_RUBAN: Texte = t("Vider le ruban", "Clear the tape");
pub const CONFIRMER_VIDER_RUBAN: Texte = t("Vider le ruban courant?", "Clear the current tape?");
pub const TITRE_SUPPRIMER_RUBAN: Texte = t("Supprimer le ruban", "Delete the tape");
pub const CONFIRMER_SUPPRIMER_RUBAN: Texte = t(
    "Supprimer définitivement ce ruban enregistré?",
    "Permanently delete this saved tape?",
);
pub const TROP_DE_RUBANS: Texte = t(
    "La limite de 200 rubans enregistrés est atteinte. Supprimer les plus anciens pour faire de la place?",
    "The limit of 200 saved tapes has been reached. Delete the oldest ones to make room?",
);
pub const SUPPRIMER_LES_PLUS_ANCIENS: Texte = t("Supprimer les plus anciens", "Delete the oldest");
pub const LECTURE_SEULE: Texte = t("Lecture seule", "Read-only");
pub const FILTRE_TEXTE: Texte = t("Fichier texte", "Text file");
pub const TITRE_EXPORTER: Texte = t("Exporter le ruban", "Export the tape");

const MOIS: [Texte; 12] = [
    t("janvier", "January"),
    t("février", "February"),
    t("mars", "March"),
    t("avril", "April"),
    t("mai", "May"),
    t("juin", "June"),
    t("juillet", "July"),
    t("août", "August"),
    t("septembre", "September"),
    t("octobre", "October"),
    t("novembre", "November"),
    t("décembre", "December"),
];

/// Date en clair : « 19 novembre 2026, 8 h 04 »; en anglais, « November 19,
/// 2026, 8:04 AM ». `mois` va de 1 à 12.
pub fn date_en_clair(annee: i32, mois: u32, jour: u32, heure: u32, minute: u32) -> String {
    let nom_mois = MOIS
        .get(mois.saturating_sub(1) as usize)
        .map_or("", |mois| mois.traduit());
    match langue::en_cours() {
        Langue::Francais => {
            let quantieme = if jour == 1 {
                "1er".to_string()
            } else {
                jour.to_string()
            };
            format!("{quantieme} {nom_mois} {annee}, {heure} h {minute:02}")
        }
        Langue::Anglais => {
            // Horloge de douze heures : minuit et midi s'écrivent 12.
            let (douze, moitie) = match heure {
                0 => (12, "AM"),
                1..=11 => (heure, "AM"),
                12 => (12, "PM"),
                _ => (heure - 12, "PM"),
            };
            format!("{nom_mois} {jour}, {annee}, {douze}:{minute:02} {moitie}")
        }
    }
}

/// Nom proposé pour un ruban : « Ruban du 19 novembre 2026, 8 h 04 ».
pub fn nom_de_ruban(date: &str) -> String {
    format!("{} {date}", selon("Ruban du", "Tape of"))
}

/// Nom de fichier proposé quand le ruban n'a pas de nom utilisable.
pub const FICHIER_PAR_DEFAUT: Texte = t("ruban", "tape");

/// Deux éléments séparés par un tiret : « Escalier — 19 novembre 2026 ».
pub fn avec_tiret(a: &str, b: &str) -> String {
    format!("{a} — {b}")
}

/// Résultat d'un exemple, précédé d'une flèche.
pub fn resultat(texte: &str) -> String {
    format!("→ {texte}")
}

/// Résumé d'un ruban enregistré : « 12 lignes ».
pub fn nombre_de_lignes(n: usize) -> String {
    match n {
        0 => selon("aucune ligne", "no lines").to_string(),
        1 => selon("1 ligne", "1 line").to_string(),
        _ => format!("{n} {}", selon("lignes", "lines")),
    }
}

// ----------------------------------------------------------------------
// Aide (section 15.3)
// ----------------------------------------------------------------------

pub const AIDE: Texte = t("Aide", "Help");
/// Pictogramme de l'onglet d'aide.
pub const SYMBOLE_AIDE: &str = "?";
pub const RECHERCHER: Texte = t("Rechercher", "Search");
pub const AUCUNE_FICHE: Texte = t(
    "Aucune fiche ne correspond à la recherche.",
    "No help topic matches your search.",
);
pub const EXEMPLE: Texte = t("Exemple", "Example");
pub const NOTE: Texte = tel("Note");
pub const FONCTION_PRINCIPALE: Texte = t("Touche", "Key");
pub const FONCTION_SECONDAIRE: Texte = t("Avec Conv", "With Conv");
pub const TOUCHES_PRINCIPALES: Texte = t("Touches principales", "Main keys");
pub const FONCTIONS_SECONDAIRES: Texte = t(
    "Fonctions secondaires (avec Conv)",
    "Second functions (with Conv)",
);

// ----------------------------------------------------------------------
// Bienvenue (annexe B.3)
// ----------------------------------------------------------------------

pub const BIENVENUE_TITRE: Texte = t("Bienvenue!", "Welcome!");
pub const BIENVENUE_INTRO: Texte = t(
    "Cette calculatrice parle le langage du chantier : pieds, pouces et fractions, mais aussi mètres et millimètres.",
    "This calculator speaks the language of the jobsite: feet, inches and fractions, as well as metres and millimetres.",
);
/// Chaque point : (titre en gras, suite du texte).
pub const BIENVENUE_POINTS: [(Texte, Texte); 5] = [
    (
        t(
            "Tapez une mesure comme vous la dites",
            "Enter a measurement the way you say it",
        ),
        t(": 5 Pieds 7 Pouces 1 / 2.", ": 5 Feet 7 Inch 1 / 2."),
    ),
    (
        t(
            "Les touches du haut résolvent le triangle",
            "The top keys solve the triangle",
        ),
        t(
            ": donnez deux valeurs parmi Pente, Montée, Course et Diag, puis appuyez sur la troisième.",
            ": enter two values among Pitch, Rise, Run and Diag, then press the third.",
        ),
    ),
    (
        t("La touche Conv", "The Conv key"),
        t(
            "donne accès aux fonctions écrites en petit au-dessus des touches, et convertit les unités.",
            "gives access to the functions written in small type above the keys, and converts units.",
        ),
    ),
    (
        t("Besoin d'aide?", "Need help?"),
        t(
            "Maintenez une touche enfoncée, ou faites un clic droit, pour afficher sa fiche.",
            "Press and hold a key, or right-click it, to open its help topic.",
        ),
    ),
    (
        t(
            "Plombier, électricien, maçon, couvreur?",
            "Plumber, electrician, mason, roofer?",
        ),
        t(
            "Touchez le nom du métier, au-dessus du clavier : les touches du haut deviennent celles de vos outils. L'onglet Métiers offre les mêmes calculateurs en formulaires.",
            "Tap the trade name above the keypad: the top keys become those of your tools. The Trades tab offers the same calculators as forms.",
        ),
    ),
];

// ----------------------------------------------------------------------
// Calculateurs par métier (section 22)
// ----------------------------------------------------------------------

pub const METIERS: Texte = t("Métiers", "Trades");
pub const METIERS_INTRODUCTION: Texte = t(
    "Des calculateurs à remplir, par corps de métier. Le bouton «\u{a0}Au clavier\u{a0}» d'un calculateur le branche sur les touches de l'accueil.",
    "Fill-in calculators, by trade. A calculator's \"On keypad\" button plugs it into the keys of the home screen.",
);
pub const METIERS_CHARPENTE: Texte = t(
    "Charpente, toiture et escaliers",
    "Framing, roofing and stairs",
);
pub const METIERS_CHARPENTE_RESUME: Texte = t(
    "Pentes, chevrons, arêtiers, escaliers, arcs, matériaux : au clavier principal.",
    "Pitches, rafters, hips, stairs, arcs, materials: on the main keypad.",
);
pub const METIERS_CHARPENTE_MENTION: Texte = t("Clavier", "Keypad");
pub const METIERS_AVERTISSEMENT: Texte = t(
    "Ces calculateurs font de la géométrie, de la physique et de l'arithmétique. Ils ne remplacent ni les plans ni les codes en vigueur : vérifiez toujours les exigences qui s'appliquent à vos travaux.",
    "These calculators do geometry, physics and arithmetic. They do not replace the drawings or the codes in force: always check the requirements that apply to your work.",
);
pub const RESULTATS: Texte = t("Résultats", "Results");
pub const AJOUTER_AU_RUBAN: Texte = t("Ajouter au ruban", "Add to tape");
pub const RECOMMENCER: Texte = t("Recommencer", "Start over");
pub const AVIS_AJOUTE_AU_RUBAN: Texte = t("Ajouté au ruban.", "Added to the tape.");

// Outils de métier au clavier (section 23).
/// Barre des métiers : le clavier de base.
pub const METIER_CHARPENTE: Texte = t("Charpente", "Framing");
pub const CHARPENTE_NOM: Texte = t(
    "Charpente, toiture et escaliers",
    "Framing, roofing and stairs",
);
pub const CHARPENTE_RESUME: Texte = t(
    "Le clavier de base : pentes, chevrons, escaliers, arcs, surfaces et volumes",
    "The basic keypad: pitches, rafters, stairs, arcs, areas and volumes",
);
pub const INFOBULLE_METIER: Texte = t(
    "Changer de métier : le clavier s'adapte",
    "Change trade: the keypad adapts",
);
pub const TITRE_CHOISIR_METIER: Texte = t("Votre métier", "Your trade");
pub const CHOISIR_METIER_EXPLICATION: Texte = t(
    "Le clavier s'adapte au métier choisi : ses outils s'ouvrent d'un toucher, et leurs champs deviennent les touches du haut.",
    "The keypad adapts to the trade you choose: its tools open with one tap, and their fields become the top keys.",
);
/// Mention à côté du métier en cours, dans la liste des métiers.
pub const METIER_ACTUEL: Texte = t("en cours", "current");
/// Touche qui donne les résultats de l'outil.
pub const RESULTAT: Texte = t("Résultat", "Result");
pub const INFOBULLE_RESULTAT: Texte = t(
    "Résultat : calcule et inscrit le calcul au ruban; appuyez de nouveau pour le résultat suivant — avec Conv : remet les champs à leur valeur de départ",
    "Result: calculates and records the calculation on the tape; press again for the next result — with Conv: resets the fields to their starting values",
);
/// Bouton d'un formulaire : brancher ce calculateur sur le clavier.
pub const AU_CLAVIER: Texte = t("Au clavier", "On keypad");
pub const INFOBULLE_AU_CLAVIER: Texte = t(
    "Brancher ce calculateur sur le clavier : ses champs deviennent les touches du haut",
    "Plug this calculator into the keypad: its fields become the top keys",
);
/// Aide d'un outil branché sur le clavier (appui long sur une de ses touches).
pub const AIDE_OUTIL_TITRE: Texte = t("Au clavier", "On the keypad");
/// Chaque point : (titre en gras, suite du texte).
pub const AIDE_OUTIL: [(Texte, Texte); 5] = [
    (
        t("Rangez une valeur", "Store a value"),
        t(
            ": tapez-la, puis appuyez sur la touche du champ. Un nombre seul prend l'unité montrée entre parenthèses au-dessus de la touche.",
            ": type it, then press the field's key. A bare number takes the unit shown in parentheses above the key.",
        ),
    ),
    (
        t("Une touche foncée", "A dark key"),
        t(
            "fait défiler les options d'une liste.",
            "cycles through the options of a list.",
        ),
    ),
    (
        t("La touche Résultat", "The Result key"),
        t(
            "calcule et inscrit le calcul au ruban. Appuyez de nouveau pour voir le résultat suivant.",
            "calculates and records the calculation on the tape. Press again to see the next result.",
        ),
    ),
    (
        t("Rappelez ou effacez", "Recall or reset"),
        t(
            ": la touche d'un champ, sans rien taper, rappelle sa valeur; `Conv` puis la touche la remet à sa valeur de départ.",
            ": a field's key, with nothing typed, recalls its value; `Conv` then the key resets it to its starting value.",
        ),
    ),
    (
        t("Poursuivez le calcul", "Keep calculating"),
        t(
            ": un résultat affiché sert à la suite (`×` `2` `=`, `M+`) et vous suit quand vous changez d'outil.",
            ": a displayed result can be used right away (`×` `2` `=`, `M+`) and follows you when you change tools.",
        ),
    ),
];
pub const OUVRIR_LE_FORMULAIRE: Texte = t("Formulaire", "Form");
pub const INFOBULLE_OUVRIR_LE_FORMULAIRE: Texte = t(
    "Ouvrir ce calculateur en formulaire : tous les résultats d'un coup d'œil",
    "Open this calculator as a form: all the results at a glance",
);

/// Infobulle de la touche d'un champ d'outil : son nom, son contenu et son mode d'emploi.
pub fn infobulle_d_outil(nom: &str, valeur: &str, choix: bool) -> String {
    let emploi = if choix {
        selon(
            "appuyez pour passer à l'option suivante",
            "press to go to the next option",
        )
    } else {
        selon(
            "tapez une valeur, puis appuyez pour la ranger ici",
            "type a value, then press to store it here",
        )
    };
    if valeur.is_empty() {
        format!("{nom} — {emploi}")
    } else {
        // Le deux-points est précédé d'une espace en français seulement.
        format!("{nom}{} {valeur} — {emploi}", selon(" :", ":"))
    }
}
/// Exemples montrés dans un champ de longueur vide.
pub const INDICE_PIEDS: Texte = t("ex. 12 pi 6 po ou 3,8 m", "e.g. 12 ft 6 in or 3.8 m");
pub const INDICE_POUCES: Texte = t("ex. 3 1/2 ou 90 mm", "e.g. 3 1/2 or 90 mm");

/// Mention d'un métier : « 5 calculateurs ».
pub fn nombre_de_calculateurs(nombre: usize) -> String {
    if nombre > 1 {
        format!("{nombre} {}", selon("calculateurs", "calculators"))
    } else {
        format!("{nombre} {}", selon("calculateur", "calculator"))
    }
}

// ----------------------------------------------------------------------
// Trucs de pro (annexe B.4)
// ----------------------------------------------------------------------

pub const TRUCS_TITRE: Texte = t("Trucs de pro", "Pro tips");
/// Chaque truc : (titre en gras, texte).
pub const TRUCS: [(Texte, Texte); 15] = [
    (
        t(
            "Faites défiler les résultats.",
            "Cycle through the results.",
        ),
        t(
            "Appuyez de nouveau sur une touche de fonction pour voir tous ses résultats : Diag donne la longueur, puis la coupe d'aplomb, puis la coupe de niveau.",
            "Press a function key again to see all its results: Diag gives the length, then the plumb cut, then the level cut.",
        ),
    ),
    (
        t("La pente reste en mémoire.", "The pitch stays in memory."),
        t(
            "Inutile de la retaper pour le chevron suivant.",
            "No need to enter it again for the next rafter.",
        ),
    ),
    (
        t("Repartez à neuf.", "Start fresh."),
        t(
            "Avant un nouveau problème, appuyez deux fois sur Effacer.",
            "Before a new problem, press Clear twice.",
        ),
    ),
    (
        t("Réutilisez un résultat.", "Reuse a result."),
        t(
            "Pour envoyer un résultat dans une autre fonction, appuyez d'abord sur =.",
            "To send a result to another function, press = first.",
        ),
    ),
    (
        t("Vérifiez l'équerre.", "Check for square."),
        t(
            "Longueur, Largeur, puis Largeur deux fois : la diagonale à mesurer sur le terrain.",
            "Length, Width, then Width twice: the diagonal to measure on site.",
        ),
    ),
    (
        t(
            "Ajoutez vos pertes d'un geste.",
            "Add your waste in one move.",
        ),
        tel("+ 10 %."),
    ),
    (
        t("Convertissez sans effort.", "Convert effortlessly."),
        t(
            "Sur une valeur affichée, appuyez sur l'unité voulue. Pieds alterne entre pieds-pouces et pieds décimaux.",
            "With a value displayed, press the unit you want. Feet toggles between feet-inches and decimal feet.",
        ),
    ),
    (
        t("Cliquez sur l'étiquette.", "Click the label."),
        t(
            "Un clic sur la petite étiquette au-dessus d'une touche équivaut à Conv puis la touche.",
            "Clicking the small label above a key is the same as Conv then the key.",
        ),
    ),
    (
        t("Gardez votre liste de coupe.", "Keep your cut list."),
        t(
            "Enregistrez le ruban après le calcul d'un escalier ou d'un toit.",
            "Save the tape after calculating a stair or a roof.",
        ),
    ),
    (
        t("Plus de précision?", "Need more precision?"),
        t(
            "Réglez la résolution des fractions dans les Préférences, ou temporairement : Conv 3 pour le 1/32.",
            "Set the fraction resolution in Preferences, or temporarily: Conv 3 for 1/32.",
        ),
    ),
    (
        t("Arrondissez à la hausse.", "Round up."),
        t(
            "Les quantités de matériaux sont exactes, non arrondies : commandez toujours l'unité supérieure.",
            "Material quantities are exact, not rounded: always order the next whole unit.",
        ),
    ),
    (
        t("Au clavier.", "On a keyboard."),
        t(
            "L'apostrophe pour les pieds, le guillemet pour les pouces, la barre d'espace pour Conv.",
            "The apostrophe for feet, the quotation mark for inches, the space bar for Conv.",
        ),
    ),
    (
        t(
            "Votre métier au bout des doigts.",
            "Your trade at your fingertips.",
        ),
        t(
            "Touchez le nom du métier, au-dessus du clavier, pour que les touches du haut deviennent celles de vos outils.",
            "Tap the trade name above the keypad so the top keys become those of your tools.",
        ),
    ),
    (
        t("D'un outil à l'autre.", "From one tool to the next."),
        t(
            "Un résultat affiché vous suit quand vous changez d'outil : appuyez sur la touche du champ où le ranger.",
            "A displayed result follows you when you change tools: press the key of the field where you want to store it.",
        ),
    ),
    (
        t("En français ou en anglais.", "In English or in French."),
        t(
            "Le bouton FR | EN, en haut de l'écran, change la langue sur-le-champ : les nombres et les unités suivent (1 234,5 pi² devient 1,234.5 ft²), et vos valeurs restent les mêmes.",
            "The FR | EN button at the top of the screen switches the language instantly: numbers and units follow (1 234,5 pi² becomes 1,234.5 ft²), and your values stay the same.",
        ),
    ),
];

// ----------------------------------------------------------------------
// À propos (sections 1.5 et 15.2)
// ----------------------------------------------------------------------

pub const A_PROPOS: Texte = t("À propos", "About");
pub const A_PROPOS_VERSION: Texte = tel("Version");
/// Mention demandée par l'auteur : l'application est offerte gratuitement.
/// La page hôte de la version Web (`app/index.html`) la répète sur son écran d'attente.
pub const GRACIEUSETE: Texte = t(
    "Gracieuseté de Sylvain Leduc de Constructo AI",
    "Courtesy of Sylvain Leduc of Constructo AI",
);
/// Coordonnées publiques de Constructo AI, à la demande de l'auteur : pied du
/// menu et écran « À propos ».
pub const COURRIEL_CONSTRUCTO: &str = "info@constructoai.ca";
pub const LIEN_COURRIEL_CONSTRUCTO: &str = "mailto:info@constructoai.ca";
pub const INFOBULLE_COURRIEL: Texte = t("Écrire à Constructo AI", "Email Constructo AI");
pub const SITE_CONSTRUCTO: &str = "constructoai.ca";
pub const LIEN_SITE_CONSTRUCTO: &str = "https://constructoai.ca/";
pub const INFOBULLE_SITE: Texte = t(
    "Ouvrir le site de Constructo AI",
    "Open the Constructo AI website",
);

/// « Version 0.5.0 · révision 1a2b3c4 » : pied du menu et écran « À propos ».
pub fn version_et_revision() -> String {
    if REVISION.is_empty() {
        format!("{A_PROPOS_VERSION} {VERSION}")
    } else {
        format!(
            "{A_PROPOS_VERSION} {VERSION} · {} {REVISION}",
            selon("révision", "revision")
        )
    }
}

/// Le manuel d'utilisation, dans chaque langue : nom de la page, et adresse de
/// la page publiée (pour la version de bureau, si la copie locale ne peut pas
/// être écrite).
pub const FICHIER_DU_MANUEL: Texte = t("manuel.html", "manual.html");
// Dans un navigateur, le manuel est toujours la page voisine : l'adresse complète ne sert pas.
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
pub const ADRESSE_DU_MANUEL: Texte = t(
    "https://constructoai.github.io/calculatrice-chantier/manuel.html",
    "https://constructoai.github.io/calculatrice-chantier/manual.html",
);

pub const LICENCE_TITRE: Texte = tel("Licence");
pub const LICENCE: Texte = t(
    "Constructo AI est un logiciel libre, distribué sous la licence Apache 2.0. Il est fourni tel quel, sans garantie.",
    "Constructo AI is free software, distributed under the Apache License 2.0. It is provided as is, without warranty.",
);
pub const CODE_SOURCE: Texte = t(
    "Code source : github.com/ConstructoAI/calculatrice-chantier",
    "Source code: github.com/ConstructoAI/calculatrice-chantier",
);
pub const LIEN_CODE_SOURCE: &str = "https://github.com/ConstructoAI/calculatrice-chantier";

pub const A_PROPOS_DESCRIPTION: Texte = t(
    "Calculatrice de construction en pieds-pouces-fractions et en métrique, dont le clavier s'adapte au métier : des outils pour onze corps de métier, pour le chantier et l'estimation. Gratuite, sans publicité, sans compte. En français et en anglais.",
    "A construction calculator in feet-inches-fractions and metric, whose keypad adapts to your trade: tools for eleven trades, for the jobsite and for estimating. Free, no ads, no account. In English and in French.",
);
pub const AVERTISSEMENT_TITRE: Texte = t("Avertissement", "Disclaimer");
pub const AVERTISSEMENT: Texte = t(
    "Cette calculatrice est un outil d'aide au calcul. Elle ne remplace ni les plans, ni le jugement d'une personne compétente, ni les exigences du Code de construction du Québec et de la réglementation municipale. Vérifiez toujours vos résultats avant de couper ou de commander.",
    "This calculator is a calculation aid. It does not replace the drawings, the judgment of a qualified person, or the requirements of the building code in force where you work (in Québec, the Construction Code) and of municipal regulations. Always check your results before cutting or ordering.",
);
pub const AVERTISSEMENT_DEFAUTS: Texte = t(
    "Les valeurs par défaut (hauteur de contremarche, giron, échappée) sont des points de départ modifiables et ne constituent pas un avis de conformité au Code.",
    "The default values (riser height, tread depth, headroom) are starting points that you can change; they are not a statement of code compliance.",
);
pub const DOSSIER_DE_DONNEES: Texte = t("Dossier de données", "Data folder");
pub const CONFIDENTIALITE: Texte = t(
    "Aucune donnée ne quitte ce poste : pas de réseau, pas de télémétrie, pas de compte.",
    "No data leaves this computer: no network, no telemetry, no account.",
);
/// Version Web : où sont les données, et ce que cela implique.
pub const DONNEES_WEB_TITRE: Texte = t("Données", "Data");
pub const DONNEES_WEB: Texte = t(
    "Vos réglages, vos mémoires et vos rubans sont conservés dans ce navigateur, sur cet appareil.",
    "Your settings, memories and tapes are kept in this browser, on this device.",
);
pub const CONFIDENTIALITE_WEB: Texte = t(
    "Rien n'est envoyé sur Internet : pas de télémétrie, pas de compte. Si vous effacez les données de navigation de ce site, elles sont effacées aussi.",
    "Nothing is sent over the Internet: no telemetry, no account. If you clear this site's browsing data, they are erased too.",
);
pub const BIBLIOTHEQUES_WEB: Texte = t(
    "Ce logiciel utilise les bibliothèques libres eframe et egui, serde, serde_json, chrono, wasm-bindgen, js-sys et web-sys (licences MIT ou Apache 2.0).",
    "This software uses the open-source libraries eframe and egui, serde, serde_json, chrono, wasm-bindgen, js-sys and web-sys (MIT or Apache 2.0 licences).",
);
pub const POLICES_WEB: Texte = t(
    "Il affiche les polices libres Selawik, Open Sans (licence SIL OFL 1.1) et DejaVu Sans (licence Bitstream Vera).",
    "It displays the open fonts Selawik, Open Sans (SIL OFL 1.1 licence) and DejaVu Sans (Bitstream Vera licence).",
);
pub const BIBLIOTHEQUES_TITRE: Texte = t("Bibliothèques libres", "Open-source libraries");
pub const BIBLIOTHEQUES: Texte = t(
    "Ce logiciel utilise les bibliothèques libres eframe et egui, serde, serde_json et chrono (licences MIT ou Apache 2.0) ainsi que rfd (licence MIT).",
    "This software uses the open-source libraries eframe and egui, serde, serde_json and chrono (MIT or Apache 2.0 licences) as well as rfd (MIT licence).",
);

// ----------------------------------------------------------------------
// Raccourcis (section 16.5) : noms des touches du clavier d'ordinateur
// ----------------------------------------------------------------------

pub const TOUCHE_ENTREE: Texte = t("Entrée", "Enter");
pub const TOUCHE_ESPACE: Texte = t("Espace", "Space");
pub const TOUCHE_ECHAP: Texte = t("Échap", "Esc");
pub const TOUCHE_SUPPR: Texte = t("Suppr", "Del");
pub const TOUCHE_INSER: Texte = t("Inser", "Ins");
pub const TOUCHE_RETOUR_ARRIERE: Texte = t("Retour arrière", "Backspace");

/// Deux raccourcis équivalents : « Entrée ou = ».
pub fn l_un_ou_l_autre(a: &str, b: &str) -> String {
    format!("{a} {} {b}", selon("ou", "or"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_date_s_ecrit_dans_chaque_langue() {
        assert_eq!(
            date_en_clair(2026, 11, 19, 8, 4),
            "19 novembre 2026, 8 h 04"
        );
        assert_eq!(date_en_clair(2026, 3, 1, 15, 30), "1er mars 2026, 15 h 30");
        assert_eq!(
            nom_de_ruban("19 novembre 2026, 8 h 04"),
            "Ruban du 19 novembre 2026, 8 h 04"
        );
        let _anglais = langue::pour_la_portee(Langue::Anglais);
        assert_eq!(
            date_en_clair(2026, 11, 19, 8, 4),
            "November 19, 2026, 8:04 AM"
        );
        assert_eq!(date_en_clair(2026, 3, 1, 15, 30), "March 1, 2026, 3:30 PM");
        assert_eq!(date_en_clair(2026, 1, 2, 0, 5), "January 2, 2026, 12:05 AM");
        assert_eq!(
            date_en_clair(2026, 1, 2, 12, 0),
            "January 2, 2026, 12:00 PM"
        );
        assert_eq!(nombre_de_lignes(0), "no lines");
        assert_eq!(nombre_de_lignes(3), "3 lines");
        assert_eq!(nombre_de_calculateurs(1), "1 calculator");
        assert_eq!(l_un_ou_l_autre("Enter", "="), "Enter or =");
    }

    #[test]
    fn les_touches_ont_un_libelle_dans_chaque_langue() {
        for langue in Langue::TOUTES {
            let _langue = langue::pour_la_portee(langue);
            for touche in Touche::GRILLE.iter().flatten().copied() {
                assert!(!libelle(touche, false, false).is_empty(), "{touche:?}");
                assert!(!nom(touche, false).is_empty(), "{touche:?}");
                if touche.a_une_fonction_secondaire() {
                    assert!(!libelle_secondaire(touche, false).is_empty(), "{touche:?}");
                    assert!(!nom_secondaire(touche, false).is_empty(), "{touche:?}");
                }
            }
        }
        let _anglais = langue::pour_la_portee(Langue::Anglais);
        assert_eq!(libelle(Touche::Pente, false, true), "Pitch");
        assert_eq!(libelle(Touche::Virgule, false, true), ".");
        assert_eq!(
            infobulle(Touche::Montee, false, Some("T")),
            "Rise — with Conv: Rake wall — shortcut: T"
        );
        assert_eq!(
            infobulle_d_outil("Length", "24 ft 0 in", false),
            "Length: 24 ft 0 in — type a value, then press to store it here"
        );
    }
}
