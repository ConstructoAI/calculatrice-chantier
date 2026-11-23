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
        Touche::Heures => "h",
        Touche::Minutes => "min",
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
        Touche::Heures => selon("Heures", "Hours"),
        Touche::Minutes => "Minutes",
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
pub const ONGLET_NOTES: Texte = t("Notes", "Notes");
pub const MENU_NOTES: Texte = t("Bloc-notes", "Notepad");
pub const NOUVELLE_NOTE: Texte = t("+ Nouvelle note", "+ New note");
pub const AUCUNE_NOTE: Texte = t("Aucune note pour l'instant.", "No notes yet.");
// Listes à cocher et recherche (section 26, point 9)
pub const PLUS_CASE: Texte = t("+ Case", "+ Checkbox");
pub const INFOBULLE_CASE: Texte = t(
    "Une ligne à cocher, au bout du texte",
    "A line to tick, at the end of the text",
);
pub const RECHERCHER_DANS_LES_NOTES: Texte = t("Rechercher dans les notes", "Search the notes");
pub const AUCUNE_NOTE_TROUVEE: Texte = t(
    "Aucune note ne contient ces mots.",
    "No note contains these words.",
);

// Le niveau et inclinomètre (section 22.22)
pub const NIVEAU: Texte = t("Niveau", "Level");
pub const NIVEAU_MENTION: Texte = t("Capteur", "Sensor");
pub const NIVEAU_RESUME_COURT: Texte = t(
    "Le téléphone posé sur un chevron, un tuyau ou une rampe : la pente en degrés, sur 12 et en %",
    "The phone laid on a rafter, a pipe or a ramp: the slope in degrees, in 12 and in %",
);
pub const NIVEAU_RESUME: Texte = t(
    "Posez le téléphone à plat sur la surface, l'écran vers le haut : il en donne la pente. Un toucher sur Envoyer met la valeur à l'afficheur de la calculatrice.",
    "Lay the phone flat on the surface, screen up: it gives the slope. A tap on Send puts the value on the calculator's display.",
);
pub const NIVEAU_EN_ATTENTE: Texte = t(
    "En attente du capteur d'inclinaison…",
    "Waiting for the tilt sensor…",
);
pub const NIVEAU_AUTORISER: Texte = t("Autoriser le capteur", "Allow the sensor");
pub const NIVEAU_DE_NIVEAU: Texte = t("De niveau", "Level");
pub const NIVEAU_ENVOYER: Texte = t("Envoyer", "Send");
pub const INFOBULLE_NIVEAU_ENVOYER: Texte = t(
    "À l'afficheur de la calculatrice",
    "To the calculator's display",
);
pub const NIVEAU_FIGER: Texte = t("Figer", "Hold");
pub const NIVEAU_REPRENDRE: Texte = t("Reprendre", "Resume");
pub const NIVEAU_ETALONNER: Texte = t("Étalonner", "Calibrate");
pub const INFOBULLE_NIVEAU_ETALONNER: Texte = t(
    "Le téléphone posé sur une surface de référence, de niveau : sa lecture devient le zéro",
    "With the phone on a level reference surface: its reading becomes zero",
);
pub const NIVEAU_SANS_ETALONNAGE: Texte = t("Effacer l'étalonnage", "Clear calibration");
pub const NIVEAU_AIDE: Texte = t(
    "Pour étalonner, posez le téléphone sur une surface de référence, de niveau, et touchez Étalonner : sa lecture devient le zéro. Un étui inégal fausse la lecture.",
    "To calibrate, lay the phone on a level reference surface and tap Calibrate: its reading becomes zero. An uneven case skews the reading.",
);
pub const AVIS_NIVEAU_ENVOYE: Texte = t(
    "Valeur envoyée à la calculatrice.",
    "Value sent to the calculator.",
);
pub const AVIS_NIVEAU_ETALONNE: Texte = t("Niveau étalonné.", "Level calibrated.");

/// « 6 sur 12 », la pente sur 12.
pub fn niveau_sur_douze(montee: &str) -> String {
    match langue::en_cours() {
        Langue::Francais => format!("{montee} sur 12"),
        Langue::Anglais => format!("{montee} in 12"),
    }
}

// Le carnet de nivellement (section 22.21)
pub const CARNET_DE_NIVELLEMENT: Texte = t("Carnet de nivellement", "Levelling book");
pub const NIVELLEMENT_MENTION: Texte = t("Laser", "Laser");
pub const NIVELLEMENT_RESUME_COURT: Texte = t(
    "Le repère et les lectures au niveau : les élévations, et ce qu'il faut creuser ou remblayer",
    "The benchmark and the level readings: the elevations, and what to cut or fill",
);
pub const NIVELLEMENT_RESUME: Texte = t(
    "Inscrivez l'élévation du repère et la lecture faite sur lui, puis la lecture de chaque point : la calculatrice donne son élévation et ce qu'il faut creuser ou remblayer pour atteindre l'élévation visée.",
    "Enter the benchmark elevation and the reading taken on it, then each point's reading: the calculator gives its elevation and what to cut or fill to reach the target elevation.",
);
pub const NIVELLEMENT_TITRE: Texte = t("Titre (facultatif)", "Title (optional)");
pub const NIVELLEMENT_TITRE_INDICE: Texte = t("ex. Entrée du garage", "e.g. Garage driveway");
pub const NIVELLEMENT_REPERE: Texte = t("Repère", "Benchmark");
pub const NIVELLEMENT_ELEVATION_DU_REPERE: Texte = t("Élévation du repère", "Benchmark elevation");
pub const NIVELLEMENT_LECTURE_SUR_LE_REPERE: Texte =
    t("Lecture sur le repère", "Reading on the benchmark");
pub const NIVELLEMENT_VISEE: Texte = t("Élévation visée", "Target elevation");
pub const NIVELLEMENT_INDICE_LECTURE: Texte = t("ex. 4 pi 6 1/4 po", "e.g. 4 ft 6 1/4 in");
pub const NIVELLEMENT_INDICE_VISEE: Texte = t("ex. 99 pi 6 po", "e.g. 99 ft 6 in");
pub const NIVELLEMENT_AIDE: Texte = t(
    "Un nombre seul est en pieds; le mètre s'écrit 1,25 m. Une ligne peut avoir sa propre élévation visée.",
    "A number alone is in feet; metres are written 1.25 m. A line can have its own target elevation.",
);
pub const NIVELLEMENT_HAUTEUR: Texte = t("Hauteur de l'instrument", "Height of instrument");
pub const NIVELLEMENT_POINTS: Texte = t("Points", "Points");
pub const NIVELLEMENT_PLUS_POINT: Texte = t("+ Point", "+ Point");
pub const NIVELLEMENT_COLONNE_POINT: Texte = t("Point", "Point");
pub const NIVELLEMENT_COLONNE_LECTURE: Texte = t("Lecture", "Reading");
pub const NIVELLEMENT_COLONNE_VISEE: Texte = t("Visée", "Target");
pub const NIVELLEMENT_COLONNE_ELEVATION: Texte = t("Élévation", "Elevation");
pub const NIVELLEMENT_COLONNE_CREUSER: Texte = t("Creuser", "Cut");
pub const NIVELLEMENT_COLONNE_REMBLAYER: Texte = t("Remblayer", "Fill");
pub const NIVELLEMENT_AU_NIVEAU: Texte = t("Au niveau", "On grade");
pub const NIVELLEMENT_POINT_ILLISIBLE: Texte = t(
    "La lecture ou la visée de ce point ne se lit pas.",
    "This point's reading or target cannot be read.",
);
pub const NIVELLEMENT_REPERE_ILLISIBLE: Texte = t(
    "L'élévation du repère ne se lit pas.",
    "The benchmark elevation cannot be read.",
);
pub const NIVELLEMENT_LECTURE_ILLISIBLE: Texte = t(
    "La lecture sur le repère ne se lit pas.",
    "The reading on the benchmark cannot be read.",
);
pub const NIVELLEMENT_VISEE_ILLISIBLE: Texte = t(
    "L'élévation visée ne se lit pas.",
    "The target elevation cannot be read.",
);
pub const NIVELLEMENT_EXPORTER: Texte = t("Exporter (CSV)", "Export (CSV)");
pub const INFOBULLE_NIVELLEMENT_EXPORTER: Texte = t(
    "Le carnet dans un fichier pour un tableur",
    "The book in a file for a spreadsheet",
);
pub const NIVELLEMENT_VIDER: Texte = t("Vider le carnet", "Clear the book");
pub const FILTRE_CSV: Texte = t("Tableur CSV", "CSV spreadsheet");
pub const AVIS_CARNET_EXPORTE: Texte = t("Carnet exporté.", "Book exported.");
pub const AVIS_CARNET_EXPORT_ECHEC: Texte = t(
    "Le carnet n'a pas pu être enregistré.",
    "The book could not be saved.",
);

/// « Hauteur de l'instrument : 105 pi 0 po ».
pub fn nivellement_hauteur(hauteur: &str) -> String {
    match langue::en_cours() {
        Langue::Francais => format!("Hauteur de l'instrument : {hauteur}"),
        Langue::Anglais => format!("Height of instrument: {hauteur}"),
    }
}

/// « Élévation 98 pi 9 po ».
pub fn nivellement_elevation(elevation: &str) -> String {
    match langue::en_cours() {
        Langue::Francais => format!("Élévation {elevation}"),
        Langue::Anglais => format!("Elevation {elevation}"),
    }
}

/// « Creuser 2 pi 0 po ».
pub fn nivellement_creuser(ecart: &str) -> String {
    match langue::en_cours() {
        Langue::Francais => format!("Creuser {ecart}"),
        Langue::Anglais => format!("Cut {ecart}"),
    }
}

/// « Remblayer 3 po ».
pub fn nivellement_remblayer(ecart: &str) -> String {
    match langue::en_cours() {
        Langue::Francais => format!("Remblayer {ecart}"),
        Langue::Anglais => format!("Fill {ecart}"),
    }
}

/// Le nom du fichier du carnet : « nivellement-2026-11-23 ».
pub fn nom_du_carnet(jour: &str) -> String {
    match langue::en_cours() {
        Langue::Francais => format!("nivellement-{jour}"),
        Langue::Anglais => format!("levelling-{jour}"),
    }
}

// Le calcul d'heures et la feuille de temps (section 22.20)
pub const HEURES: Texte = t("Heures", "Hours");
pub const HEURES_RESUME: Texte = t(
    "Des heures et des minutes au clavier, comme des pieds et des pouces, et la feuille de temps de la semaine",
    "Hours and minutes on the keypad, like feet and inches, and the weekly timesheet",
);
pub const FEUILLE_DE_TEMPS: Texte = t("Feuille de temps", "Timesheet");
pub const FEUILLE_DE_TEMPS_MENTION: Texte = t("Semaine", "Week");
pub const FEUILLE_DE_TEMPS_RESUME_COURT: Texte = t(
    "L'arrivée, le départ et le dîner de chaque jour : les heures de la semaine",
    "Arrival, departure and lunch each day: the hours of the week",
);
pub const FEUILLE_DE_TEMPS_RESUME: Texte = t(
    "L'arrivée, le départ et le dîner de chaque jour : les heures de chaque jour et le total de la semaine. Une heure s'écrit 7:30 ou 7 h 30, sur 24 heures; un dîner, en minutes (30) ou en heures (1 h).",
    "Arrival, departure and lunch each day: the hours of each day and the week total. A time is written 7:30 or 7 h 30, on the 24-hour clock; a lunch, in minutes (30) or in hours (1 h).",
);
pub const COLONNE_ARRIVEE: Texte = t("Arrivée", "In");
pub const COLONNE_DEPART: Texte = t("Départ", "Out");
pub const COLONNE_DINER: Texte = t("Dîner", "Lunch");
pub const COLONNE_HEURES: Texte = t("Heures", "Hours");
pub const TOTAL_DE_LA_SEMAINE: Texte = t("Total de la semaine", "Week total");
pub const VIDER_LA_SEMAINE: Texte = t("Vider la semaine", "Clear the week");
pub const JOURS_COURTS: [Texte; 7] = [
    t("Lun", "Mon"),
    t("Mar", "Tue"),
    t("Mer", "Wed"),
    t("Jeu", "Thu"),
    t("Ven", "Fri"),
    t("Sam", "Sat"),
    t("Dim", "Sun"),
];
pub const JOURS: [Texte; 7] = [
    t("Lundi", "Monday"),
    t("Mardi", "Tuesday"),
    t("Mercredi", "Wednesday"),
    t("Jeudi", "Thursday"),
    t("Vendredi", "Friday"),
    t("Samedi", "Saturday"),
    t("Dimanche", "Sunday"),
];

/// « Lundi : « 7:75 » ne se lit pas. »
pub fn heure_illisible(jour: &str, texte: &str) -> String {
    match langue::en_cours() {
        Langue::Francais => format!("{jour} : « {texte} » ne se lit pas."),
        Langue::Anglais => format!("{jour}: \"{texte}\" cannot be read."),
    }
}

/// « Lundi : le dîner dépasse la journée. »
pub fn diner_trop_long(jour: &str) -> String {
    match langue::en_cours() {
        Langue::Francais => format!("{jour} : le dîner dépasse la journée."),
        Langue::Anglais => format!("{jour}: lunch is longer than the day."),
    }
}

/// « 7:30 – 16:30 · dîner 30 », dans le ruban.
pub fn journee_en_clair(arrivee: &str, depart: &str, diner: &str) -> String {
    if diner.is_empty() {
        return format!("{arrivee} – {depart}");
    }
    match langue::en_cours() {
        Langue::Francais => format!("{arrivee} – {depart} · dîner {diner}"),
        Langue::Anglais => format!("{arrivee} – {depart} · lunch {diner}"),
    }
}

// La liste de coupe (section 22.19)
pub const LISTE_DE_COUPE: Texte = t("Liste de coupe", "Cut list");
pub const LISTE_DE_COUPE_MENTION: Texte = t("Optimisée", "Optimized");
pub const LISTE_DE_COUPE_RESUME_COURT: Texte = t(
    "Bois, moulures, conduits, tuyaux : les pièces à acheter et leur plan de coupe",
    "Lumber, mouldings, conduit, pipe: the lengths to buy and how to cut them",
);
pub const LISTE_DE_COUPE_RESUME: Texte = t(
    "Inscrivez les morceaux à couper et les longueurs offertes : la calculatrice donne les pièces à acheter et le plan de coupe de chacune, trait de scie compris, avec la chute.",
    "Enter the pieces to cut and the stock lengths: the calculator gives the lengths to buy and how to cut each one, saw kerf included, with the offcut.",
);
pub const COUPE_MORCEAUX: Texte = t("Morceaux à couper", "Pieces to cut");
pub const COUPE_PLUS_MORCEAU: Texte = t("+ Morceau", "+ Piece");
pub const COUPE_INDICE_LONGUEUR: Texte = t("ex. 4 pi 6 po ou 1,2 m", "e.g. 4 ft 6 in or 1.2 m");
pub const COUPE_STOCK: Texte = t("Stock", "Stock");
pub const COUPE_OFFERTES: Texte = t("Longueurs offertes", "Stock lengths");
pub const COUPE_OFFERTES_INDICE: Texte = t("ex. 8; 10; 12; 16", "e.g. 8; 10; 12; 16");
pub const COUPE_OFFERTES_AIDE: Texte = t(
    "Un nombre seul est en pieds; séparez les longueurs par des points-virgules.",
    "A number alone is in feet; separate the lengths with semicolons.",
);
pub const COUPE_TRAIT: Texte = t("Trait de scie", "Saw kerf");
pub const COUPE_TRAIT_INDICE: Texte = t("ex. 1/8 po", "e.g. 1/8 in");
pub const COUPE_MATERIAU: Texte = t("Matériau (facultatif)", "Material (optional)");
pub const COUPE_MATERIAU_INDICE: Texte = t("ex. 2 × 4 SPF", "e.g. 2 × 4 SPF");
pub const COUPE_PLAN: Texte = t("Plan de coupe", "Cutting plan");
pub const COUPE_A_ACHETER: Texte = t("À acheter", "To buy");
pub const COUPE_LONGUEUR_ACHETEE: Texte = t("Longueur achetée", "Length bought");
pub const COUPE_CHUTE: Texte = t("Chute", "Offcut");
pub const COUPE_PIECE: Texte = t("Pièce", "Length");
pub const COUPE_PIECES_DE_STOCK: Texte = t("Pièces de stock", "Stock lengths");
pub const COUPE_AUCUNE_OFFERTE: Texte = t(
    "Inscrivez au moins une longueur offerte.",
    "Enter at least one stock length.",
);
pub const COUPE_TRAIT_ILLISIBLE: Texte = t(
    "Le trait de scie ne se lit pas.",
    "The saw kerf cannot be read.",
);

/// « La longueur offerte 2 ne se lit pas. »
pub fn coupe_offerte_illisible(rang: usize) -> String {
    match langue::en_cours() {
        Langue::Francais => format!("La longueur offerte {rang} ne se lit pas."),
        Langue::Anglais => format!("Stock length {rang} cannot be read."),
    }
}

/// « Ligne 3 : la longueur ne se lit pas. »
pub fn coupe_longueur_illisible(ligne: usize) -> String {
    match langue::en_cours() {
        Langue::Francais => format!("Ligne {ligne} : la longueur ne se lit pas."),
        Langue::Anglais => format!("Line {ligne}: the length cannot be read."),
    }
}

/// « Ligne 3 : la quantité doit être un nombre entier de 1 à 999. »
pub fn coupe_quantite_illisible(ligne: usize) -> String {
    match langue::en_cours() {
        Langue::Francais => {
            format!("Ligne {ligne} : la quantité doit être un nombre entier de 1 à 999.")
        }
        Langue::Anglais => {
            format!("Line {ligne}: the quantity must be a whole number from 1 to 999.")
        }
    }
}

/// « Il y a plus de 2000 morceaux. »
pub fn coupe_trop_de_morceaux(maximum: usize) -> String {
    match langue::en_cours() {
        Langue::Francais => format!("Pas plus de {maximum} morceaux, en tout."),
        Langue::Anglais => format!("No more than {maximum} pieces in all."),
    }
}

/// Les morceaux qu'aucune longueur offerte ne peut donner.
pub fn coupe_trop_longs(liste: &str) -> String {
    match langue::en_cours() {
        Langue::Francais => {
            format!("Plus longs que toutes les longueurs offertes, donc hors du plan : {liste}.")
        }
        Langue::Anglais => {
            format!("Longer than every stock length, so left out of the plan: {liste}.")
        }
    }
}

/// « 4 pi 3 po (7 %) » : la chute et sa part de la longueur achetée.
pub fn coupe_chute(longueur: &str, pourcent: f64) -> String {
    match langue::en_cours() {
        Langue::Francais => format!("{longueur} ({pourcent:.0}\u{a0}%)"),
        Langue::Anglais => format!("{longueur} ({pourcent:.0}%)"),
    }
}

/// Les coupes d'une pièce, puis sa chute s'il en reste une.
pub fn coupe_detail(coupes: &str, chute: &str, avec_chute: bool) -> String {
    if !avec_chute {
        return coupes.to_string();
    }
    match langue::en_cours() {
        Langue::Francais => format!("{coupes} · chute {chute}"),
        Langue::Anglais => format!("{coupes} · offcut {chute}"),
    }
}

// Le rapport d'un calcul en PDF (section 22.18)
pub const BOUTON_PDF: Texte = tel("PDF");
pub const INFOBULLE_RAPPORT_PDF: Texte = t(
    "Un rapport de ce calcul en PDF : la date, le projet, les données et les résultats",
    "A PDF report of this calculation: date, project, inputs and results",
);
pub const TITRE_RAPPORT_PDF: Texte = t("Rapport de calcul en PDF", "Calculation report as PDF");
pub const PROJET_DU_RAPPORT: Texte = t("Projet (facultatif)", "Project (optional)");
pub const PROJET_INDICE: Texte = t(
    "ex. Agrandissement, 12 rue des Érables",
    "e.g. Addition, 12 Maple Street",
);
pub const RAPPORT_EXPORTER: Texte = t("Exporter en PDF", "Export as PDF");
pub const RAPPORT_DONNEES: Texte = t("Données", "Inputs");
pub const AVIS_RAPPORT_EXPORTE: Texte = t("Rapport exporté.", "Report exported.");
pub const AVIS_RAPPORT_EXPORT_ECHEC: Texte = t(
    "Le rapport n'a pas pu être enregistré.",
    "The report could not be saved.",
);

/// L'en-tête d'un rapport de calcul : « Agrandissement · Rapport de calcul ·
/// 23 novembre 2026, 10 h 51 ».
pub fn rapport_entete(date: &str, projet: &str) -> String {
    let rapport = match langue::en_cours() {
        Langue::Francais => format!("Rapport de calcul · {date}"),
        Langue::Anglais => format!("Calculation report · {date}"),
    };
    if projet.trim().is_empty() {
        rapport
    } else {
        format!("{} · {rapport}", projet.trim())
    }
}

/// Le nom du fichier d'un rapport : « rapport-dalle-2026-11-23 ».
pub fn nom_du_rapport(id: &str, jour: &str) -> String {
    match langue::en_cours() {
        Langue::Francais => format!("rapport-{id}-{jour}"),
        Langue::Anglais => format!("report-{id}-{jour}"),
    }
}

// La liste d'achat (section 26, point 10)
pub const LISTE_D_ACHAT: Texte = t("Liste d'achat", "Shopping list");
pub const AJOUTER_A_LA_LISTE_D_ACHAT: Texte =
    t("Ajouter à la liste d'achat", "Add to shopping list");
pub const INFOBULLE_LISTE_D_ACHAT: Texte = t(
    "Les sacs, les panneaux, les boîtes… dans la note « Liste d'achat » du bloc-notes",
    "Bags, boards, boxes… in the Shopping list note of the notepad",
);
pub const AVIS_AJOUTE_A_LA_LISTE: Texte = t(
    "Ajouté à la liste d'achat, dans le bloc-notes.",
    "Added to the shopping list, in the notepad.",
);

/// « 3 cochées sur 5 », sous le titre d'une note qui a des cases à cocher.
pub fn cases_cochees(cochees: usize, total: usize) -> String {
    match langue::en_cours() {
        Langue::Francais => {
            let mot = if cochees > 1 { "cochées" } else { "cochée" };
            format!("{cochees} {mot} sur {total}")
        }
        Langue::Anglais => format!("{cochees} of {total} ticked"),
    }
}
pub const NOTE_SANS_TITRE: Texte = t("Note sans titre", "Untitled note");
pub const NOTE_TITRE_INDICE: Texte = t("Titre", "Title");
pub const NOTE_TEXTE_INDICE: Texte = t("Écrivez votre note…", "Write your note…");
pub const AVIS_NOTE_COPIEE: Texte = t("Note copiée.", "Note copied.");
pub const AVIS_TROP_DE_NOTES: Texte = t(
    "Le bloc-notes est plein (500 notes) : supprimez-en quelques-unes.",
    "The notepad is full (500 notes): delete a few.",
);
pub const TITRE_SUPPRIMER_NOTE: Texte = t("Supprimer la note", "Delete the note");
pub const CONFIRMER_SUPPRIMER_NOTE: Texte = t(
    "La note sera supprimée pour de bon, avec ses pièces jointes.",
    "The note will be deleted for good, with its attachments.",
);

// Pièces jointes du bloc-notes (section 26)
pub const PIECES_JOINTES: Texte = t("Pièces jointes", "Attachments");
pub const PIECES_JOINTES_DEUX_POINTS: Texte = t("Pièces jointes :", "Attachments:");
pub const AJOUTER_PHOTO: Texte = t("+ Photo", "+ Photo");
pub const AJOUTER_FICHIER: Texte = t("+ Fichier", "+ File");
pub const RETIRER: Texte = t("Retirer", "Remove");
pub const CHARGEMENT: Texte = t("Chargement…", "Loading…");
// Titre et filtre de la boîte « Ouvrir » du bureau (le navigateur n'en a pas).
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
pub const TITRE_CHOISIR_PHOTOS: Texte = t("Joindre des photos", "Attach photos");
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
pub const TITRE_CHOISIR_FICHIERS: Texte = t("Joindre des fichiers", "Attach files");
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
pub const FILTRE_PHOTOS: Texte = t("Photos (JPEG, PNG)", "Photos (JPEG, PNG)");
pub const FILTRE_PIECE: Texte = t("Pièce jointe", "Attachment");
pub const TITRE_ENREGISTRER_PIECE: Texte = t("Enregistrer la pièce jointe", "Save the attachment");
pub const AVIS_PIECE_ENREGISTREE: Texte = t("Pièce jointe enregistrée.", "Attachment saved.");
pub const AVIS_PIECE_EXPORT_ECHEC: Texte = t(
    "La pièce jointe n'a pas pu être enregistrée à cet endroit.",
    "The attachment could not be saved there.",
);
pub const AVIS_TROP_DE_PIECES: Texte = t(
    "Une note peut avoir 50 pièces jointes au plus.",
    "A note can have at most 50 attachments.",
);
pub const AVIS_FICHIER_TROP_GROS: Texte = t(
    "Un fichier de plus de 25 Mo ne peut pas être joint.",
    "A file larger than 25 MB cannot be attached.",
);
pub const AVIS_IMAGE_ILLISIBLE: Texte = t(
    "Cette image n'a pas pu être lue (JPEG et PNG seulement) : elle est jointe comme un fichier.",
    "This image could not be read (JPEG and PNG only): it is attached as a file.",
);
pub const AVIS_ECHEC_PIECE: Texte = t(
    "Une pièce jointe n'a pas pu être gardée : l'espace de l'appareil est peut-être plein.",
    "An attachment could not be kept: the device may be out of space.",
);
pub const AVIS_PIECE_ABSENTE: Texte = t(
    "Cette pièce jointe est introuvable sur l'appareil.",
    "This attachment cannot be found on the device.",
);
pub const AVIS_OUVERTURE_ECHEC: Texte = t(
    "Aucun programme n'a pu ouvrir cette pièce jointe.",
    "No program could open this attachment.",
);
pub const TITRE_RETIRER_PIECE: Texte = t("Retirer la pièce jointe", "Remove the attachment");
pub const CONFIRMER_RETIRER_PIECE: Texte = t(
    "La pièce jointe sera effacée de l'appareil pour de bon.",
    "The attachment will be erased from the device for good.",
);

// Mesures sur une photo du bloc-notes (section 26)
pub const AJOUTER_COTE: Texte = t("+ Cote", "+ Dimension");
pub const COTE_CONSIGNE: Texte = t(
    "Glissez sur la photo d'un bout à l'autre de ce que vous avez mesuré.",
    "Drag across the photo from one end to the other of what you measured.",
);
pub const MESURE_DE_LA_COTE: Texte = t("Mesure", "Measurement");
pub const MESURE_INDICE: Texte = t("ex. : 32 po", "e.g. 32 in");
pub const REFERENCE: Texte = t("Référence", "Reference");
pub const PREFIXE_REFERENCE: Texte = t("Réf.", "Ref.");
pub const COTES_EXPLICATION: Texte = t(
    "Inscrivez la mesure prise au ruban. Une cote laissée vide est estimée d'après la cote de référence et marquée ~ : la perspective fausse les longueurs qui ne sont pas dans le plan de la référence.",
    "Enter the measurement taken with the tape. A dimension left empty is estimated from the reference dimension and marked ~: perspective distorts lengths that are not in the plane of the reference.",
);
pub const TITRE_EXPORTER_PHOTO: Texte =
    t("Exporter la photo cotée", "Export the dimensioned photo");
pub const AVIS_PHOTO_EXPORTEE: Texte = t("Photo cotée exportée.", "Dimensioned photo exported.");
pub const AVIS_PHOTO_EXPORT_ECHEC: Texte = t(
    "La photo n'a pas pu être exportée.",
    "The photo could not be exported.",
);

// Sauvegarde complète (section 17.4)
pub const MENU_MODE_CHANTIER: Texte = t("Mode chantier", "Job-site mode");
// Les sections du menu latéral
pub const MENU_SECTION_OUTILS: Texte = t("Outils", "Tools");
pub const MENU_SECTION_REGLAGES: Texte = t("Réglages", "Settings");
pub const MENU_SECTION_DONNEES: Texte = t("Mes données", "My data");
pub const MENU_SECTION_AIDE: Texte = t("Aide et informations", "Help and information");
pub const MENU_SECTION_APPLICATION: Texte = t("Application", "App");
// Calculateurs épinglés et récents (section 22.16)
pub const EPINGLER: Texte = t("Épingler", "Pin");
pub const DESEPINGLER: Texte = t("Désépingler", "Unpin");
pub const INFOBULLE_EPINGLER: Texte = t(
    "En tête de l'onglet Métiers et de la barre des métiers de l'accueil",
    "At the head of the Trades tab and of the home screen's trade bar",
);
pub const EPINGLES: Texte = t("Épinglés", "Pinned");
pub const RECENTS: Texte = t("Récents", "Recent");
pub const TOUS_LES_METIERS: Texte = t("Tous les métiers", "All trades");
pub const AVIS_TROP_D_EPINGLES: Texte = t(
    "Douze calculateurs au plus peuvent être épinglés : désépinglez-en un d'abord.",
    "Up to twelve calculators can be pinned: unpin one first.",
);
pub const MENU_QUITTER_MODE_CHANTIER: Texte = t("Quitter le mode chantier", "Leave job-site mode");
pub const MENU_EXPORTER_DONNEES: Texte = t("Exporter mes données", "Export my data");
pub const MENU_IMPORTER_DONNEES: Texte = t("Importer des données", "Import data");
pub const TITRE_EXPORTER_DONNEES: Texte = t("Exporter mes données", "Export my data");
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
pub const TITRE_IMPORTER_DONNEES: Texte = t("Importer des données", "Import data");
pub const FILTRE_SAUVEGARDE: Texte = t(
    "Sauvegarde de Constructo AI (JSON)",
    "Constructo AI backup (JSON)",
);
pub const AVIS_DONNEES_EXPORTEES: Texte = t("Données exportées.", "Data exported.");
pub const AVIS_DONNEES_EXPORT_ECHEC: Texte = t(
    "Les données n'ont pas pu être exportées à cet endroit.",
    "The data could not be exported there.",
);
pub const AVIS_SAUVEGARDE_ILLISIBLE: Texte = t(
    "Ce fichier n'est pas une sauvegarde de Constructo AI, ou il est abîmé.",
    "This file is not a Constructo AI backup, or it is damaged.",
);
pub const TITRE_RESTAURER: Texte = t("Importer des données", "Import data");
pub const CONFIRMER_RESTAURER: Texte = t(
    "Les données de la calculatrice — réglages, mémoires, rubans, dessins, notes et photos — seront remplacées par celles de la sauvegarde.",
    "The calculator's data — settings, memories, tapes, drawings, notes and photos — will be replaced by those of the backup.",
);
pub const REMPLACER: Texte = t("Remplacer", "Replace");
pub const AVIS_DONNEES_IMPORTEES: Texte = t("Données importées.", "Data imported.");
pub const AVIS_DONNEES_IMPORT_ECHEC: Texte = t(
    "Les données n'ont pas pu être importées : l'espace de l'appareil est peut-être plein.",
    "The data could not be imported: the device may be out of space.",
);

/// « Sauvegarde du 23 novembre 2026, 15 h 00 », dans la confirmation d'une importation.
pub fn sauvegarde_du(date: &str) -> String {
    match langue::en_cours() {
        Langue::Francais => format!("Sauvegarde du {date}"),
        Langue::Anglais => format!("Backup of {date}"),
    }
}

/// Nom du fichier d'une sauvegarde complète : « constructo-ai-sauvegarde-2026-11-23 ».
pub fn nom_de_sauvegarde(jour: &str) -> String {
    match langue::en_cours() {
        Langue::Francais => format!("constructo-ai-sauvegarde-{jour}"),
        Langue::Anglais => format!("constructo-ai-backup-{jour}"),
    }
}

// Copier, dupliquer, miroir et rétablir (section 25.1, point 31)
pub const DESSIN_RETABLIR: Texte = t("Rétablir", "Redo");
pub const DESSIN_DUPLIQUER: Texte = t("Dupliquer", "Duplicate");
pub const DESSIN_MIROIR: Texte = t("Miroir", "Mirror");
pub const AVIS_DESSIN_DUPLIQUE: Texte = t("Dessin dupliqué.", "Drawing duplicated.");

/// « Chambre (copie) », le nom d'une copie.
pub fn nom_de_copie(nom: &str) -> String {
    match langue::en_cours() {
        Langue::Francais => format!("{nom} (copie)"),
        Langue::Anglais => format!("{nom} (copy)"),
    }
}

// Mes dessins : la bibliothèque des dessins (section 25)
pub const MES_DESSINS: Texte = t("Mes dessins", "My drawings");
pub const NOUVEAU_DESSIN: Texte = t("+ Nouveau dessin", "+ New drawing");
pub const OUVRIR_UN_FICHIER: Texte = t("Ouvrir un fichier…", "Open a file…");
pub const ENREGISTRER_LE_FICHIER: Texte = t("Enregistrer le fichier", "Save as a file");
pub const DESSIN_OUVERT: Texte = t("ouvert", "open");
pub const DESSINS_EXPLICATION: Texte = t(
    "Chaque dessin garde ses pièces. Pour le rouvrir sur un autre appareil, enregistrez-le en fichier, envoyez ce fichier (courriel, nuage, câble), puis touchez « Ouvrir un fichier… » sur l'autre appareil.",
    "Each drawing keeps its rooms. To reopen it on another device, save it as a file, send that file (email, cloud, cable), then tap Open a file… on the other device.",
);
pub const TITRE_NOMMER_DESSIN: Texte = t("Renommer le dessin", "Rename the drawing");
pub const NOM_DU_DESSIN: Texte = t("Nom du dessin", "Drawing name");
pub const TITRE_SUPPRIMER_DESSIN: Texte = t("Supprimer le dessin", "Delete the drawing");
pub const CONFIRMER_SUPPRIMER_DESSIN: Texte = t(
    "Le dessin sera supprimé pour de bon, avec toutes ses pièces.",
    "The drawing will be deleted for good, with all its rooms.",
);
pub const TITRE_ENREGISTRER_DESSIN: Texte = t("Enregistrer le dessin", "Save the drawing");
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
pub const TITRE_OUVRIR_DESSIN: Texte = t("Ouvrir un dessin", "Open a drawing");
pub const FILTRE_DESSIN: Texte = t(
    "Dessin de Constructo AI (JSON)",
    "Constructo AI drawing (JSON)",
);
pub const AVIS_DESSIN_ENREGISTRE: Texte = t("Dessin enregistré.", "Drawing saved.");
pub const AVIS_DESSIN_ENREGISTRE_ECHEC: Texte = t(
    "Le dessin n'a pas pu être enregistré à cet endroit.",
    "The drawing could not be saved there.",
);
pub const AVIS_DESSIN_ILLISIBLE: Texte = t(
    "Ce fichier n'est pas un dessin de Constructo AI, ou il est abîmé.",
    "This file is not a Constructo AI drawing, or it is damaged.",
);
pub const AVIS_TROP_DE_DESSINS: Texte = t(
    "La calculatrice garde 100 dessins au plus : supprimez-en quelques-uns.",
    "The calculator keeps at most 100 drawings: delete a few.",
);
pub const AVIS_DESSIN_IMPORTE: Texte = t("Dessin ouvert.", "Drawing opened.");

/// « Dessin 3 », le nom d'un dessin neuf.
pub fn dessin_par_defaut(numero: usize) -> String {
    match langue::en_cours() {
        Langue::Francais => format!("Dessin {numero}"),
        Langue::Anglais => format!("Drawing {numero}"),
    }
}

/// « 3 pièces », sous le nom d'un dessin de la liste.
pub fn nombre_de_pieces_du_dessin(nombre: usize) -> String {
    match (langue::en_cours(), nombre) {
        (Langue::Francais, 1) => "1 pièce".to_string(),
        (Langue::Francais, n) => format!("{n} pièces"),
        (Langue::Anglais, 1) => "1 room".to_string(),
        (Langue::Anglais, n) => format!("{n} rooms"),
    }
}

/// « 3 pièces jointes », sous le titre d'une note de la liste.
pub fn nombre_de_pieces(nombre: usize) -> String {
    match (langue::en_cours(), nombre) {
        (Langue::Francais, 1) => "1 pièce jointe".to_string(),
        (Langue::Francais, n) => format!("{n} pièces jointes"),
        (Langue::Anglais, 1) => "1 attachment".to_string(),
        (Langue::Anglais, n) => format!("{n} attachments"),
    }
}

/// Taille d'un fichier, en Ko ou en Mo (1 Ko = 1 024 octets) : « 2,4 Mo », « 2.4 MB ».
pub fn taille_en_clair(octets: u64) -> String {
    const KO: f64 = 1024.0;
    let octets = octets as f64;
    let (valeur, unite_fr, unite_en) = if octets < KO * KO {
        ((octets / KO).ceil(), "Ko", "KB")
    } else {
        (((octets / (KO * KO)) * 10.0).round() / 10.0, "Mo", "MB")
    };
    match langue::en_cours() {
        Langue::Francais => format!("{} {unite_fr}", valeur.to_string().replace('.', ",")),
        Langue::Anglais => format!("{valeur} {unite_en}"),
    }
}
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

pub const DESSIN_OBJETS: Texte = t("Objets (escalier, mobilier)", "Objects (stairs, furniture)");
pub const DESSIN_CLOISONS: Texte = t("Cloisons (murs intérieurs)", "Partitions (interior walls)");
pub const DESSIN_CLOISON: Texte = t("Cloison", "Partition");
pub const DESSIN_AJOUTER_UNE_CLOISON: Texte = t("+ Cloison", "+ Partition");
pub const DESSIN_EPAISSEUR: Texte = t("épaisseur", "thickness");
/// Devant l'épaisseur d'une cloison, dans sa ligne.
pub const DESSIN_EP: Texte = t("ép.", "thk.");
pub const DESSIN_EPAISSEUR_DES_MURS: Texte = t("Épaisseur des murs", "Wall thickness");
/// Devant le bouton qui dit de quel côté des murs sont les mesures.
pub const DESSIN_MESURES: Texte = t("Mesures", "Measurements");
pub const DESSIN_INTERIEUR: Texte = t("Intérieur", "Interior");
pub const DESSIN_EXTERIEUR: Texte = t("Extérieur", "Exterior");
pub const DESSIN_SURFACE_EXTERIEURE: Texte =
    t("Surface extérieure (brute)", "Exterior area (gross)");
pub const DESSIN_LARGEUR_DE_LA_PORTE: Texte = t("largeur de la porte", "door width");
pub const DESSIN_DISTANCE_DU_BOUT_A: Texte = t("distance du bout A", "distance from end A");
pub const DESSIN_INVERSER: Texte = t("Inverser", "Flip");
pub const DESSIN_SANS_PORTE: Texte = t("Sans porte", "No door");

/// « segment 2 » : un segment d'une cloison en polyligne.
pub fn dessin_segment(numero: usize) -> String {
    format!("segment {numero}")
}

/// « sur le segment 2 » : le segment qui porte la porte d'une cloison.
pub fn dessin_sur_le_segment(numero: usize) -> String {
    match langue::en_cours() {
        Langue::Francais => format!("sur le segment {numero}"),
        Langue::Anglais => format!("on segment {numero}"),
    }
}

/// « à 3 pi 0 po du bout A » : la place de la porte d'une cloison.
pub fn dessin_du_bout_a(distance: &str) -> String {
    match langue::en_cours() {
        Langue::Francais => format!("à {distance} du bout A"),
        Langue::Anglais => format!("{distance} from end A"),
    }
}
pub const DESSIN_EPAISSEUR_DES_CLOISONS: Texte = t("Épaisseur des cloisons", "Partition thickness");
pub const DESSIN_TRACER_LA_CLOISON: Texte = t(
    "Glissez d'un bout à l'autre de la cloison",
    "Drag from one end of the partition to the other",
);
pub const DESSIN_LONGUEUR_DES_CLOISONS: Texte = t("Longueur des cloisons", "Partition length");
pub const DESSIN_SURFACE_DES_CLOISONS: Texte = t("Cloisons (2 faces)", "Partitions (2 faces)");
pub const DESSIN_NOTES: Texte = t("Notes sur le plan", "Notes on the plan");
pub const DESSIN_AJOUTER_UNE_NOTE: Texte = t("+ Note", "+ Note");
/// Ce que montre sur le plan une note encore vide.
pub const DESSIN_NOTE: Texte = t("Note", "Note");
pub const DESSIN_NOTE_INDICE: Texte = t("Écrivez la note", "Type the note");
pub const DESSIN_TOUCHER_POUR_LA_NOTE: Texte = t(
    "Touchez le plan pour placer la note",
    "Tap the plan to place the note",
);
pub const DESSIN_PIVOTER: Texte = t("Pivoter", "Rotate");
pub const DESSIN_PLUS_15: Texte = tel("+15°");
pub const DESSIN_MOINS_15: Texte = tel("−15°");
pub const DESSIN_PLUS_90: Texte = tel("+90°");
pub const DESSIN_PROFONDEUR: Texte = t("profondeur", "depth");

/// Le nom d'un objet de la pièce.
/// Le nom d'une forme d'escalier (point 30).
pub fn forme_d_escalier(forme: noyau::mobilier::FormeDEscalier) -> Texte {
    use noyau::mobilier::FormeDEscalier;
    match forme {
        FormeDEscalier::Droit => t("Droit", "Straight"),
        FormeDEscalier::EnL => t("En L", "L-shaped"),
        FormeDEscalier::EnU => t("En U", "U-shaped"),
        FormeDEscalier::Tournant => t("Quart tournant", "Quarter-turn"),
        FormeDEscalier::Colimacon => t("Colimaçon", "Spiral"),
    }
}

pub const DESSIN_HAUTEUR_A_MONTER: Texte = t("Hauteur à monter", "Rise");
pub const DESSIN_CONTREMARCHES: Texte = t("Nombre de contremarches", "Number of risers");
pub const DESSIN_HAUTEUR_COURT: Texte = t("Hauteur", "Rise");

/// « 13 contremarches de 7 3/8 po », sous un escalier.
pub fn contremarches_en_clair(nombre: usize, contremarche: &str) -> String {
    match langue::en_cours() {
        Langue::Francais => format!("{nombre} contremarches de {contremarche}"),
        Langue::Anglais => format!("{nombre} risers of {contremarche}"),
    }
}

/// « giron 10 po », le giron d'un escalier droit.
pub fn giron_en_clair(giron: &str) -> String {
    match langue::en_cours() {
        Langue::Francais => format!("giron {giron}"),
        Langue::Anglais => format!("tread {giron}"),
    }
}

pub fn objet(genre: noyau::mobilier::GenreDObjet) -> Texte {
    use noyau::mobilier::GenreDObjet;
    match genre {
        GenreDObjet::Escalier => t("Escalier", "Stairs"),
        GenreDObjet::Lit => t("Lit", "Bed"),
        GenreDObjet::Table => t("Table", "Table"),
        GenreDObjet::Sofa => t("Sofa", "Sofa"),
        GenreDObjet::Comptoir => t("Comptoir", "Counter"),
        GenreDObjet::Toilette => t("Toilette", "Toilet"),
        GenreDObjet::Bain => t("Bain", "Bathtub"),
        GenreDObjet::Douche => t("Douche", "Shower"),
        GenreDObjet::Laveuse => t("Laveuse", "Washer"),
        GenreDObjet::Refrigerateur => t("Réfrigérateur", "Fridge"),
        GenreDObjet::Cuisiniere => t("Cuisinière", "Range"),
    }
}

/// La consigne affichée sur la planche quand un objet attend sa place.
pub fn dessin_toucher_pour_placer(nom: &str) -> String {
    match langue::en_cours() {
        Langue::Francais => format!("Touchez la pièce pour placer : {nom}"),
        Langue::Anglais => format!("Tap the room to place: {nom}"),
    }
}

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
pub const DESSIN_RENOMMER: Texte = t("Renommer", "Rename");
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
// L'angle d'un côté ou d'un segment de cloison (section 25.1, point 32)
pub const DESSIN_ANGLE_DU_COTE: Texte = t(
    "Angle par rapport au côté précédent",
    "Angle from the previous side",
);
pub const DESSIN_ANGLE_DU_SEGMENT: Texte = t(
    "Angle par rapport au segment précédent",
    "Angle from the previous segment",
);
pub const DESSIN_ANGLE_APPLIQUER: Texte = t("Appliquer", "Apply");
pub const INFOBULLE_DESSIN_ANGLE: Texte = t(
    "90° : d'équerre; 45° : en biais. Tapez un autre angle au besoin. Les autres côtés gardent leur direction; les mesures déduites suivent.",
    "90°: square; 45°: angled. Type another angle if needed. The other sides keep their direction; worked-out measurements follow.",
);
pub const DESSIN_ANGLE_INVALIDE: Texte = t(
    "Tapez un angle de plus de 0° et de moins de 180°.",
    "Type an angle over 0° and under 180°.",
);
pub const DESSIN_ANGLE_REFUSE: Texte = t(
    "Avec cet angle, la forme ne se referme pas : essayez-en un autre.",
    "With this angle, the shape does not close: try another one.",
);
// Les diagonales d'une pièce à quatre côtés (section 25.1, point 33)
pub const DESSIN_DIAGONALES: Texte = t(
    "Diagonales (pièce hors d'équerre)",
    "Diagonals (out-of-square room)",
);
pub const DESSIN_DIAGONALES_AIDE: Texte = t(
    "Mesurez les diagonales d'un coin à l'autre : la pièce se redessine telle qu'elle est, avec l'angle de chaque coin et son écart à l'équerre.",
    "Measure the diagonals corner to corner: the room is redrawn as it really is, with each corner's angle and how far it is out of square.",
);
pub const DESSIN_DIAGONALES_ATTENTE: Texte = t(
    "Les diagonales servent quand les quatre côtés sont mesurés.",
    "The diagonals are used once all four sides are measured.",
);
pub const DESSIN_D_EQUERRE: Texte = t("d'équerre", "square");

/// Le titre d'une diagonale : 0 du coin 1 au coin 3, 1 du coin 2 au coin 4.
pub fn dessin_diagonale(diagonale: usize) -> String {
    let (de, a) = (diagonale + 1, diagonale + 3);
    match langue::en_cours() {
        Langue::Francais => format!("Diagonale du coin {de} au coin {a}"),
        Langue::Anglais => format!("Diagonal from corner {de} to corner {a}"),
    }
}

/// Une diagonale en bref : « 1 → 3 ».
pub fn dessin_diagonale_courte(diagonale: usize) -> String {
    format!("{} → {}", diagonale + 1, diagonale + 3)
}

/// Une diagonale qui ne va pas avec les côtés.
pub fn dessin_diagonale_impossible(diagonale: usize) -> String {
    let (de, a) = (diagonale + 1, diagonale + 3);
    match langue::en_cours() {
        Langue::Francais => format!(
            "La diagonale du coin {de} au coin {a} ne va pas avec ces côtés : elle est trop longue ou trop courte. Vérifiez les mesures."
        ),
        Langue::Anglais => format!(
            "The diagonal from corner {de} to corner {a} does not fit these sides: it is too long or too short. Check the measurements."
        ),
    }
}

/// Deux diagonales qui ne s'accordent pas.
pub fn dessin_diagonales_en_desaccord(ecart: &str) -> String {
    match langue::en_cours() {
        Langue::Francais => format!(
            "Les deux diagonales diffèrent de {ecart} de ce que donnent les côtés : mesurez-les de nouveau. La forme suit la diagonale 1 → 3."
        ),
        Langue::Anglais => format!(
            "The two diagonals are {ecart} off from what the sides give: measure them again. The shape follows diagonal 1 → 3."
        ),
    }
}

/// Le nom d'un coin.
pub fn dessin_coin(rang: usize) -> String {
    match langue::en_cours() {
        Langue::Francais => format!("Coin {rang}"),
        Langue::Anglais => format!("Corner {rang}"),
    }
}

/// L'écart d'un coin à l'équerre : « 3/8 po sur 4 pi ».
pub fn dessin_ecart_sur(ecart: &str, metrique: bool) -> String {
    match (langue::en_cours(), metrique) {
        (Langue::Francais, false) => format!("{ecart} sur 4 pi"),
        (Langue::Francais, true) => format!("{ecart} sur 1 m"),
        (Langue::Anglais, false) => format!("{ecart} over 4 ft"),
        (Langue::Anglais, true) => format!("{ecart} over 1 m"),
    }
}

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
pub const PREF_MODE_CHANTIER: Texte = t("Mode chantier", "Job-site mode");
pub const PREF_MODE_CHANTIER_AIDE: Texte = t(
    "Contraste élevé, lisible en plein soleil. Sur l'écran de la calculatrice, les touches des chiffres grossissent et le clavier descend jusqu'au bas de l'écran, pour les gants et pour taper d'une main; le menu mène aux autres écrans.",
    "High contrast, readable in full sun. On the calculator screen, the number keys get bigger and the keypad goes down to the bottom of the screen, for gloves and one-handed typing; the menu leads to the other screens.",
);

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
