//! Tous les textes de l'interface (français du Québec).
//!
//! Aucune chaîne visible par l'utilisateur ne doit être écrite ailleurs que dans
//! ce fichier et dans `aide.rs` (R-52).

use noyau::Touche;
use noyau::prefs::{
    AffichageAires, AffichageVolumes, EmpannonsIrreguliers, ModeFractions, MurCintre, Ordre,
    SeparateurDecimal, Theme,
};
use noyau::unites::UniteMasseVolumique;

/// Nom de l'application. Défini ici seulement, pour être changé facilement.
pub const NOM_APPLICATION: &str = "Calculatrice de chantier";

/// Identifiant du dossier de données de l'utilisateur.
pub const APP_ID: &str = "CalculatriceChantier";

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// ----------------------------------------------------------------------
// Clavier (section 5.1)
// ----------------------------------------------------------------------

/// Libellé principal d'une touche. Un saut de ligne sépare les libellés sur deux lignes.
pub fn libelle(touche: Touche, trigo: bool, point_decimal: bool) -> String {
    let fixe = match touche {
        Touche::Pente => "Pente",
        Touche::Montee => "Montée",
        Touche::Course => "Course",
        Touche::Diag => "Diag",
        Touche::AretierNoue => "Arêtier\nNoue",
        Touche::OngletCompose => "Onglet\ncomposé",
        Touche::Escalier => "Escalier",
        Touche::Arc => "Arc",
        Touche::Cercle => "Cercle",
        Touche::Empannon => "Empannon",
        Touche::Metres => "m",
        Touche::Longueur if trigo => "sin",
        Touche::Largeur if trigo => "cos",
        Touche::Hauteur if trigo => "tan",
        Touche::Longueur => "Longueur",
        Touche::Largeur => "Largeur",
        Touche::Hauteur => "Hauteur",
        Touche::Pourcent => "%",
        Touche::Verges => "Verges",
        Touche::Pieds => "Pieds",
        Touche::Pouces => "Pouces",
        Touche::Fraction => "/",
        Touche::Effacer => "Effacer",
        Touche::Conv => "Conv",
        Touche::Chiffre(c) => return c.to_string(),
        Touche::Diviser => "÷",
        Touche::Stocker => "Stocker",
        Touche::Multiplier => "×",
        Touche::Rappel => "Rappel",
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
        Touche::Pente => "Ratio",
        Touche::Montee => "Mur en pente",
        Touche::Course => "Polygone",
        Touche::Diag => "Toiture",
        Touche::AretierNoue => "Pente irrég.",
        Touche::OngletCompose => "Angle moulure",
        Touche::Escalier => "Contrem. max.",
        Touche::Arc => "Rayon",
        Touche::Cercle => "Colonne/Cône",
        Touche::Empannon => "Emp. irrég.",
        Touche::Longueur if trigo => "sin⁻¹",
        Touche::Largeur if trigo => "cos⁻¹",
        Touche::Hauteur if trigo => "tan⁻¹",
        Touche::Longueur => "Blocs",
        Touche::Largeur => "Semelle",
        Touche::Hauteur => "Gypse",
        Touche::Pourcent => "x²",
        Touche::Effacer => "√x",
        Touche::Chiffre(7) => "cm",
        Touche::Chiffre(8) => "pmp",
        Touche::Chiffre(9) => "mm",
        Touche::Diviser => "1/x",
        Touche::Stocker => "Préf.",
        Touche::Chiffre(4) => "lb",
        Touche::Chiffre(5) => "Montants",
        Touche::Chiffre(6) => "t courte",
        Touche::Multiplier => "Effacer tout",
        Touche::Rappel => "MC",
        Touche::Chiffre(1) => "kg",
        Touche::Chiffre(2) => "Acre",
        Touche::Chiffre(3) => "t métr.",
        Touche::Soustraire => "+/−",
        Touche::MPlus => "M−",
        Touche::Chiffre(0) => "Coût",
        Touche::Virgule => "dms◄►déc",
        Touche::Egal => "Ruban",
        Touche::Additionner => "π",
        _ => "",
    }
}

/// Nom complet de la fonction principale (infobulles et accessibilité).
pub fn nom(touche: Touche, trigo: bool) -> String {
    let fixe = match touche {
        Touche::Pente => "Pente",
        Touche::Montee => "Montée",
        Touche::Course => "Course",
        Touche::Diag => "Diagonale",
        Touche::AretierNoue => "Arêtier ou noue",
        Touche::OngletCompose => "Onglet composé",
        Touche::Escalier => "Escalier",
        Touche::Arc => "Arc",
        Touche::Cercle => "Cercle",
        Touche::Empannon => "Empannons",
        Touche::Metres => "Mètres",
        Touche::Longueur if trigo => "Sinus",
        Touche::Largeur if trigo => "Cosinus",
        Touche::Hauteur if trigo => "Tangente",
        Touche::Longueur => "Longueur",
        Touche::Largeur => "Largeur",
        Touche::Hauteur => "Hauteur",
        Touche::Pourcent => "Pourcentage",
        Touche::Verges => "Verges",
        Touche::Pieds => "Pieds",
        Touche::Pouces => "Pouces",
        Touche::Fraction => "Barre de fraction",
        Touche::Effacer => "Effacer",
        Touche::Conv => "Conversion",
        Touche::Chiffre(c) => return format!("Chiffre {c}"),
        Touche::Diviser => "Diviser",
        Touche::Stocker => "Stocker",
        Touche::Multiplier => "Multiplier",
        Touche::Rappel => "Rappel",
        Touche::Soustraire => "Soustraire",
        Touche::MPlus => "Ajouter à la mémoire",
        Touche::Virgule => "Séparateur décimal",
        Touche::Egal => "Égal",
        Touche::Additionner => "Additionner",
        Touche::RetourArriere => "Retour arrière",
        Touche::FractionRapide { num, den } => return format!("Fraction rapide {num}/{den}"),
    };
    fixe.to_string()
}

/// Nom complet de la fonction secondaire; vide s'il n'y en a pas.
pub fn nom_secondaire(touche: Touche, trigo: bool) -> &'static str {
    match touche {
        Touche::Pente => "Ratio de pente",
        Touche::Montee => "Mur en pente",
        Touche::Course => "Polygone",
        Touche::Diag => "Toiture",
        Touche::AretierNoue => "Pente irrégulière",
        Touche::OngletCompose => "Angle de la moulure",
        Touche::Escalier => "Escalier à contremarche maximale",
        Touche::Arc => "Rayon",
        Touche::Cercle => "Colonne et cône",
        Touche::Empannon => "Empannons irréguliers",
        Touche::Longueur if trigo => "Arc sinus",
        Touche::Largeur if trigo => "Arc cosinus",
        Touche::Hauteur if trigo => "Arc tangente",
        Touche::Longueur => "Blocs",
        Touche::Largeur => "Semelle",
        Touche::Hauteur => "Gypse",
        Touche::Pourcent => "Carré",
        Touche::Effacer => "Racine carrée",
        Touche::Chiffre(7) => "Centimètres",
        Touche::Chiffre(8) => "Pieds mesure de planche",
        Touche::Chiffre(9) => "Millimètres",
        Touche::Diviser => "Inverse",
        Touche::Stocker => "Préférences",
        Touche::Chiffre(4) => "Livres",
        Touche::Chiffre(5) => "Montants",
        Touche::Chiffre(6) => "Tonnes courtes",
        Touche::Multiplier => "Effacer tout",
        Touche::Rappel => "Effacer la mémoire",
        Touche::Chiffre(1) => "Kilogrammes",
        Touche::Chiffre(2) => "Acres",
        Touche::Chiffre(3) => "Tonnes métriques",
        Touche::Soustraire => "Changer le signe",
        Touche::MPlus => "Soustraire de la mémoire",
        Touche::Chiffre(0) => "Coût",
        Touche::Virgule => "Degrés décimaux ou degrés-minutes-secondes",
        Touche::Egal => "Ruban de calcul",
        Touche::Additionner => "Pi",
        _ => "",
    }
}

/// Infobulle d'une touche : « Montée — avec Conv : Mur en pente — raccourci : T ».
pub fn infobulle(touche: Touche, trigo: bool, raccourci: Option<&str>) -> String {
    let mut texte = nom(touche, trigo);
    let secondaire = nom_secondaire(touche, trigo);
    if !secondaire.is_empty() {
        texte.push_str(" — avec Conv : ");
        texte.push_str(secondaire);
    }
    if let Some(r) = raccourci {
        texte.push_str(" — raccourci : ");
        texte.push_str(r);
    }
    texte
}

// ----------------------------------------------------------------------
// Afficheur (section 16.2)
// ----------------------------------------------------------------------

pub const INDICATEUR_CONV: &str = "CONV";
pub const INDICATEUR_STOCKER: &str = "STOCKER";
pub const INDICATEUR_RAPPEL: &str = "RAPPEL";
pub const INDICATEUR_MEMOIRE: &str = "M";
pub const INDICATEUR_MEMORISE: &str = "MÉM.";
pub const COPIE: &str = "Copié";
pub const INFOBULLE_RETOUR_ARRIERE: &str = "Retour arrière — raccourci : Retour arrière";
pub const INFOBULLE_VALEUR: &str = "Double-clic : copier la valeur";

// ----------------------------------------------------------------------
// Navigation (section 15.1)
// ----------------------------------------------------------------------

pub const ONGLET_ACCUEIL: &str = "Accueil";
pub const ONGLET_RUBANS: &str = "Rubans";
pub const ONGLET_AIDE: &str = "Aide";

pub const MENU_RUBANS: &str = "Rubans enregistrés";
pub const MENU_AIDE: &str = "Aide";
pub const MENU_BIENVENUE: &str = "Bienvenue";
pub const MENU_TRUCS: &str = "Trucs de pro";
pub const MENU_PREFERENCES: &str = "Préférences";
pub const MENU_A_PROPOS: &str = "À propos";
pub const MENU_QUITTER: &str = "Quitter";
pub const MENU_METIERS: &str = "Calculateurs par métier";
/// Version Web : installer l'application sur le téléphone, la tablette ou l'ordinateur.
pub const MENU_INSTALLER: &str = "Installer l'application";
// Les espaces insécables gardent chaque guillemet collé à son texte.
pub const AVIS_INSTALLER: &str = "Pour installer la calculatrice sur cet appareil, passez par le menu de votre navigateur.\n\nSur iPhone ou iPad (Safari) : touchez Partager, puis «\u{00A0}Sur l'écran d'accueil\u{00A0}».\n\nSur Android (Chrome) : touchez les trois points, puis «\u{00A0}Installer l'application\u{00A0}» ou «\u{00A0}Ajouter à l'écran d'accueil\u{00A0}».\n\nUne fois installée, elle s'ouvre comme une application et fonctionne sans réseau.";

pub const INFOBULLE_MENU: &str = "Menu";
pub const NOM_MENU: &str = "Menu";
pub const NOM_PLEIN_ECRAN: &str = "Plein écran";
pub const INFOBULLE_PLEIN_ECRAN: &str = "Plein écran — raccourci : F11";
pub const INFOBULLE_PREFERENCES: &str = "Préférences — raccourci : Ctrl+,";

pub const TERMINE: &str = "Terminé";
pub const FERMER: &str = "Fermer";
pub const RETOUR: &str = "Retour";
pub const ANNULER: &str = "Annuler";
pub const ENREGISTRER: &str = "Enregistrer";
pub const OUVRIR: &str = "Ouvrir";
pub const RENOMMER: &str = "Renommer";
pub const COPIER: &str = "Copier";
pub const EXPORTER: &str = "Exporter";
pub const SUPPRIMER: &str = "Supprimer";
pub const VIDER: &str = "Vider";
pub const COMMENCER: &str = "Commencer";

// ----------------------------------------------------------------------
// Messages et confirmations (section 4.5)
// ----------------------------------------------------------------------

pub const TITRE_EFFACER_TOUT: &str = "Effacer tout";
pub const CONFIRMER_EFFACER_TOUT: &str = "Effacer toutes les valeurs mémorisées et les remettre aux valeurs par défaut? Les préférences sont conservées.";
pub const BOUTON_EFFACER_TOUT: &str = "Effacer tout";

pub const TITRE_REINITIALISER: &str = "Réinitialisation";
pub const CONFIRMER_REINITIALISER: &str =
    "Réinitialiser toutes les données et toutes les préférences?";
pub const BOUTON_REINITIALISER: &str = "Réinitialiser";

pub const AVIS_REGLAGES_ILLISIBLES: &str =
    "Les réglages n'ont pas pu être lus; les valeurs par défaut ont été rétablies.";
pub const AVIS_ECRITURE_IMPOSSIBLE: &str =
    "Les données n'ont pas pu être enregistrées sur le disque.";
/// Version Web : le stockage local du navigateur est plein ou refusé.
pub const AVIS_ECRITURE_IMPOSSIBLE_WEB: &str = "Les données n'ont pas pu être enregistrées dans ce navigateur. La calculatrice fonctionne, mais vos réglages et vos rubans ne seront pas conservés.";
pub const AVIS_COLLAGE_REFUSE: &str = "Le texte collé n'est ni un nombre ni une longueur.";
pub const AVIS_EXPORT_REUSSI: &str = "Ruban exporté.";
pub const AVIS_EXPORT_ECHEC: &str = "Le fichier n'a pas pu être écrit.";
pub const AVIS_RUBAN_ENREGISTRE: &str = "Ruban enregistré.";
pub const AVIS_RUBAN_COPIE: &str = "Ruban copié dans le presse-papiers.";
pub const AVIS_RUBAN_VIDE: &str = "Le ruban est vide.";
pub const OK: &str = "OK";

// ----------------------------------------------------------------------
// Préférences (section 13)
// ----------------------------------------------------------------------

pub const PREFERENCES: &str = "Préférences";
pub const SECTION_GENERAL: &str = "Général";
pub const SECTION_AFFICHAGE: &str = "Affichage";
pub const SECTION_FONCTIONS: &str = "Fonctions";
pub const SECTION_MEMORISEES: &str = "Valeurs mémorisées";
pub const SECTION_REINITIALISATION: &str = "Réinitialisation";

pub const PREF_TRIGO: &str = "Mode trigonométrique";
pub const PREF_TRIGO_AIDE: &str = "Longueur, Largeur et Hauteur deviennent sin, cos et tan.";
pub const PREF_FRACTIONS_RAPIDES: &str = "Fractions rapides";
pub const PREF_FRACTIONS_RAPIDES_AIDE: &str =
    "Affiche une bande de fractions de pouce au-dessus du clavier.";
pub const PREF_PLEIN_ECRAN: &str = "Mode plein écran";
pub const PREF_PLEIN_ECRAN_AIDE: &str = "Fenêtre en plein écran, sans bordure (aussi F11).";
pub const PREF_THEME: &str = "Thème";

pub const PREF_RESOLUTION: &str = "Résolution des fractions";
pub const PREF_MODE_FRACTIONS: &str = "Mode des fractions";
pub const PREF_EXPONENTIELLE: &str = "Notation exponentielle";
pub const PREF_EXPONENTIELLE_AIDE: &str =
    "Affiche les très grands et les très petits nombres en notation scientifique.";
pub const PREF_AIRES: &str = "Affichage des aires";
pub const PREF_VOLUMES: &str = "Affichage des volumes";
pub const PREF_METRES: &str = "Mètres (longueurs)";
pub const PREF_DEGRES: &str = "Degrés décimaux";
pub const PREF_MILLIERS: &str = "Séparateur de milliers";
pub const PREF_MILLIERS_AIDE: &str = "Une espace sépare les groupes de trois chiffres : 1 234,56.";
pub const PREF_DECIMAL: &str = "Séparateur décimal";
pub const PREF_DECIMALES: &str = "Décimales (valeurs sans unité)";

pub const PREF_MUR_PENTE: &str = "Mur en pente";
pub const PREF_MUR_CINTRE: &str = "Mur cintré";
pub const PREF_EMPANNONS: &str = "Empannons";
pub const PREF_EMPANNONS_IRREGULIERS: &str = "Empannons irréguliers";

pub const FLOTTANT: &str = "Flottant";
pub const METRES_FIXES: &str = "0,000";
pub const DEGRES_FIXES: &str = "0,00";

pub fn theme(t: Theme) -> &'static str {
    match t {
        Theme::Automatique => "Automatique",
        Theme::Clair => "Clair",
        Theme::Sombre => "Sombre",
    }
}

pub fn resolution(denominateur: u32) -> String {
    format!("1/{denominateur}")
}

pub fn mode_fractions(m: ModeFractions) -> &'static str {
    match m {
        ModeFractions::Standard => "Standard (fraction réduite)",
        ModeFractions::Constant => "Constant (dénominateur fixe)",
    }
}

pub fn affichage_aires(a: AffichageAires) -> &'static str {
    match a {
        AffichageAires::Standard => "Standard",
        AffichageAires::PiedsCarres => "Pieds carrés",
        AffichageAires::VergesCarrees => "Verges carrées",
        AffichageAires::MetresCarres => "Mètres carrés",
    }
}

pub fn affichage_volumes(v: AffichageVolumes) -> &'static str {
    match v {
        AffichageVolumes::Standard => "Standard",
        AffichageVolumes::VergesCubes => "Verges cubes",
        AffichageVolumes::PiedsCubes => "Pieds cubes",
        AffichageVolumes::MetresCubes => "Mètres cubes",
    }
}

pub fn separateur_decimal(s: SeparateurDecimal) -> &'static str {
    match s {
        SeparateurDecimal::Virgule => "Virgule",
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
        Ordre::Decroissant => "Décroissant (du plus long au plus court)",
        Ordre::Croissant => "Croissant (du plus court au plus long)",
    }
}

pub fn mur_cintre(m: MurCintre) -> &'static str {
    match m {
        MurCintre::Exterieur => "Extérieur (au-dessus de l'arc)",
        MurCintre::Interieur => "Intérieur (sous l'arc)",
    }
}

pub fn empannons_irreguliers(e: EmpannonsIrreguliers) -> &'static str {
    match e {
        EmpannonsIrreguliers::EntraxeConstant => "Entraxe constant",
        EmpannonsIrreguliers::Apparies => "Appariés",
    }
}

pub fn unite_masse_volumique(u: UniteMasseVolumique) -> &'static str {
    noyau::etiquettes::symbole_masse_volumique(u)
}

// Valeurs mémorisées (section 13.4)
pub const MEMO_ECHAPPEE: &str = "Échappée";
pub const MEMO_CONTREMARCHE: &str = "Hauteur de contremarche visée";
pub const MEMO_PLANCHER: &str = "Épaisseur du plancher";
pub const MEMO_GIRON: &str = "Giron visé";
pub const MEMO_AIRE_BLOC: &str = "Aire d'un bloc";
pub const MEMO_LONGUEUR_BLOC: &str = "Longueur d'un bloc";
pub const MEMO_ANGLE_MOULURE: &str = "Angle de la moulure";
pub const MEMO_ENTRAXE: &str = "Entraxe";
pub const MEMO_SECTION_SEMELLE: &str = "Section de semelle";
pub const MEMO_MASSE_VOLUMIQUE: &str = "Masse volumique";
pub const MEMO_PAQUETS: &str = "Paquets de bardeaux par carré";

pub const MEMO_USAGE_ECHAPPEE: &str = "Trémie d'escalier";
pub const MEMO_USAGE_CONTREMARCHE: &str = "Escalier — aussi par Stocker 7";
pub const MEMO_USAGE_PLANCHER: &str = "Trémie d'escalier — aussi par Stocker 8";
pub const MEMO_USAGE_GIRON: &str = "Escalier — aussi par Stocker 9";
pub const MEMO_USAGE_AIRE_BLOC: &str = "Blocs — aussi par Stocker 4";
pub const MEMO_USAGE_LONGUEUR_BLOC: &str = "Blocs — aussi par Stocker 4";
pub const MEMO_USAGE_ANGLE_MOULURE: &str = "Onglet composé — aussi par Stocker Onglet composé";
pub const MEMO_USAGE_ENTRAXE: &str = "Empannons, murs, montants — aussi par Stocker 5";
pub const MEMO_USAGE_SECTION_SEMELLE: &str = "Semelle — aussi par Stocker 6";
pub const MEMO_USAGE_MASSE_VOLUMIQUE: &str = "Volume ↔ masse — aussi par Stocker 0";
pub const MEMO_USAGE_PAQUETS: &str = "Toiture";

pub const VALEUR_PAR_DEFAUT: &str = "Valeur par défaut";
pub const VALEUR_NON_RECONNUE: &str = "Valeur non reconnue";
pub const MODIFIER: &str = "Modifier";
pub const UNITE: &str = "Unité";
pub const EXEMPLES_LONGUEUR: &str = "Exemples : 6'8\", 7 1/2 po, 2,032 m";
pub const EXEMPLES_AIRE: &str = "Exemples : 128 po², 1,83 pi², 0,17 m²";
pub const EXEMPLES_ANGLE: &str = "Un angle en degrés, entre 0 et 90";
pub const EXEMPLES_NOMBRE: &str = "Un nombre plus grand que zéro";

pub const BOUTON_REINITIALISER_TOUT: &str = "Réinitialiser les données et les préférences";

// ----------------------------------------------------------------------
// Rubans (section 14)
// ----------------------------------------------------------------------

pub const RUBAN_COURANT: &str = "Ruban courant";
pub const RUBANS_ENREGISTRES: &str = "Rubans enregistrés";
pub const RUBAN_VIDE: &str = "Le ruban est vide. Les calculs s'y inscriront au fur et à mesure.";
pub const AUCUN_RUBAN_ENREGISTRE: &str = "Aucun ruban enregistré.";
pub const NOM_DU_RUBAN: &str = "Nom du ruban";
pub const TITRE_ENREGISTRER_RUBAN: &str = "Enregistrer le ruban";
pub const TITRE_RENOMMER_RUBAN: &str = "Renommer le ruban";
pub const TITRE_VIDER_RUBAN: &str = "Vider le ruban";
pub const CONFIRMER_VIDER_RUBAN: &str = "Vider le ruban courant?";
pub const TITRE_SUPPRIMER_RUBAN: &str = "Supprimer le ruban";
pub const CONFIRMER_SUPPRIMER_RUBAN: &str = "Supprimer définitivement ce ruban enregistré?";
pub const TROP_DE_RUBANS: &str = "La limite de 200 rubans enregistrés est atteinte. Supprimer les plus anciens pour faire de la place?";
pub const SUPPRIMER_LES_PLUS_ANCIENS: &str = "Supprimer les plus anciens";
pub const LECTURE_SEULE: &str = "Lecture seule";
pub const FILTRE_TEXTE: &str = "Fichier texte";
pub const TITRE_EXPORTER: &str = "Exporter le ruban";

const MOIS: [&str; 12] = [
    "janvier",
    "février",
    "mars",
    "avril",
    "mai",
    "juin",
    "juillet",
    "août",
    "septembre",
    "octobre",
    "novembre",
    "décembre",
];

/// Date en clair : « 19 novembre 2026, 8 h 04 ». `mois` va de 1 à 12.
pub fn date_en_clair(annee: i32, mois: u32, jour: u32, heure: u32, minute: u32) -> String {
    let nom_mois = MOIS
        .get(mois.saturating_sub(1) as usize)
        .copied()
        .unwrap_or("");
    let quantieme = if jour == 1 {
        "1er".to_string()
    } else {
        jour.to_string()
    };
    format!("{quantieme} {nom_mois} {annee}, {heure} h {minute:02}")
}

/// Nom proposé pour un ruban : « Ruban du 19 novembre 2026, 8 h 04 ».
pub fn nom_de_ruban(date: &str) -> String {
    format!("Ruban du {date}")
}

/// Nom de fichier proposé quand le ruban n'a pas de nom utilisable.
pub const FICHIER_PAR_DEFAUT: &str = "ruban";

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
        0 => "aucune ligne".to_string(),
        1 => "1 ligne".to_string(),
        _ => format!("{n} lignes"),
    }
}

// ----------------------------------------------------------------------
// Aide (section 15.3)
// ----------------------------------------------------------------------

pub const AIDE: &str = "Aide";
/// Pictogramme de l'onglet d'aide.
pub const SYMBOLE_AIDE: &str = "?";
pub const RECHERCHER: &str = "Rechercher";
pub const AUCUNE_FICHE: &str = "Aucune fiche ne correspond à la recherche.";
pub const EXEMPLE: &str = "Exemple";
pub const NOTE: &str = "Note";
pub const FONCTION_PRINCIPALE: &str = "Touche";
pub const FONCTION_SECONDAIRE: &str = "Avec Conv";
pub const TOUCHES_PRINCIPALES: &str = "Touches principales";
pub const FONCTIONS_SECONDAIRES: &str = "Fonctions secondaires (avec Conv)";

// ----------------------------------------------------------------------
// Bienvenue (annexe B.3)
// ----------------------------------------------------------------------

pub const BIENVENUE_TITRE: &str = "Bienvenue!";
pub const BIENVENUE_INTRO: &str = "Cette calculatrice parle le langage du chantier : pieds, pouces et fractions, mais aussi mètres et millimètres.";
/// Chaque point : (titre en gras, suite du texte).
pub const BIENVENUE_POINTS: [(&str, &str); 5] = [
    (
        "Tapez une mesure comme vous la dites",
        ": 5 Pieds 7 Pouces 1 / 2.",
    ),
    (
        "Les touches du haut résolvent le triangle",
        ": donnez deux valeurs parmi Pente, Montée, Course et Diag, puis appuyez sur la troisième.",
    ),
    (
        "La touche Conv",
        "donne accès aux fonctions écrites en petit au-dessus des touches, et convertit les unités.",
    ),
    (
        "Besoin d'aide?",
        "Maintenez une touche enfoncée, ou faites un clic droit, pour afficher sa fiche.",
    ),
    (
        "Plombier, électricien, cimentier?",
        "L'onglet Métiers réunit des calculateurs à remplir, par corps de métier, et de quoi préparer une soumission.",
    ),
];

// ----------------------------------------------------------------------
// Calculateurs par métier (section 22)
// ----------------------------------------------------------------------

pub const METIERS: &str = "Métiers";
pub const METIERS_INTRODUCTION: &str =
    "Des calculateurs à remplir, par corps de métier. Les résultats s'ajoutent au ruban.";
pub const METIERS_CHARPENTE: &str = "Charpente, toiture et escaliers";
pub const METIERS_CHARPENTE_RESUME: &str =
    "Pentes, chevrons, arêtiers, escaliers, arcs, matériaux : au clavier principal.";
pub const METIERS_CHARPENTE_MENTION: &str = "Clavier";
pub const METIERS_AVERTISSEMENT: &str = "Ces calculateurs font de la géométrie, de la physique et de l'arithmétique. Ils ne remplacent ni les plans ni les codes en vigueur : vérifiez toujours les exigences qui s'appliquent à vos travaux.";
pub const RESULTATS: &str = "Résultats";
pub const AJOUTER_AU_RUBAN: &str = "Ajouter au ruban";
pub const RECOMMENCER: &str = "Recommencer";
pub const AVIS_AJOUTE_AU_RUBAN: &str = "Ajouté au ruban.";
/// Exemples montrés dans un champ de longueur vide.
pub const INDICE_PIEDS: &str = "ex. 12 pi 6 po ou 3,8 m";
pub const INDICE_POUCES: &str = "ex. 3 1/2 ou 90 mm";

/// Mention d'un métier : « 5 calculateurs ».
pub fn nombre_de_calculateurs(nombre: usize) -> String {
    if nombre > 1 {
        format!("{nombre} calculateurs")
    } else {
        format!("{nombre} calculateur")
    }
}

// ----------------------------------------------------------------------
// Trucs de pro (annexe B.4)
// ----------------------------------------------------------------------

pub const TRUCS_TITRE: &str = "Trucs de pro";
/// Chaque truc : (titre en gras, texte).
pub const TRUCS: [(&str, &str); 12] = [
    (
        "Faites défiler les résultats.",
        "Appuyez de nouveau sur une touche de fonction pour voir tous ses résultats : Diag donne la longueur, puis la coupe d'aplomb, puis la coupe de niveau.",
    ),
    (
        "La pente reste en mémoire.",
        "Inutile de la retaper pour le chevron suivant.",
    ),
    (
        "Repartez à neuf.",
        "Avant un nouveau problème, appuyez deux fois sur Effacer.",
    ),
    (
        "Réutilisez un résultat.",
        "Pour envoyer un résultat dans une autre fonction, appuyez d'abord sur =.",
    ),
    (
        "Vérifiez l'équerre.",
        "Longueur, Largeur, puis Largeur deux fois : la diagonale à mesurer sur le terrain.",
    ),
    ("Ajoutez vos pertes d'un geste.", "+ 10 %."),
    (
        "Convertissez sans effort.",
        "Sur une valeur affichée, appuyez sur l'unité voulue. Pieds alterne entre pieds-pouces et pieds décimaux.",
    ),
    (
        "Cliquez sur l'étiquette.",
        "Un clic sur la petite étiquette au-dessus d'une touche équivaut à Conv puis la touche.",
    ),
    (
        "Gardez votre liste de coupe.",
        "Enregistrez le ruban après le calcul d'un escalier ou d'un toit.",
    ),
    (
        "Plus de précision?",
        "Réglez la résolution des fractions dans les Préférences, ou temporairement : Conv 3 pour le 1/32.",
    ),
    (
        "Arrondissez à la hausse.",
        "Les quantités de matériaux sont exactes, non arrondies : commandez toujours l'unité supérieure.",
    ),
    (
        "Au clavier.",
        "L'apostrophe pour les pieds, le guillemet pour les pouces, la barre d'espace pour Conv.",
    ),
];

// ----------------------------------------------------------------------
// À propos (sections 1.5 et 15.2)
// ----------------------------------------------------------------------

pub const A_PROPOS: &str = "À propos";
pub const A_PROPOS_VERSION: &str = "Version";
/// Mention demandée par l'auteur : l'application est offerte gratuitement.
/// La page hôte de la version Web (`app/index.html`) la répète sur son écran d'attente.
pub const GRACIEUSETE: &str = "Gracieuseté de Sylvain Leduc de Constructo AI";
pub const A_PROPOS_DESCRIPTION: &str = "Calculatrice de construction en pieds-pouces-fractions et en métrique, avec des calculateurs par métier, pour le chantier et l'estimation. Gratuite, sans publicité, sans compte.";
pub const AVERTISSEMENT_TITRE: &str = "Avertissement";
pub const AVERTISSEMENT: &str = "Cette calculatrice est un outil d'aide au calcul. Elle ne remplace ni les plans, ni le jugement d'une personne compétente, ni les exigences du Code de construction du Québec et de la réglementation municipale. Vérifiez toujours vos résultats avant de couper ou de commander.";
pub const AVERTISSEMENT_DEFAUTS: &str = "Les valeurs par défaut (hauteur de contremarche, giron, échappée) sont des points de départ modifiables et ne constituent pas un avis de conformité au Code.";
pub const DOSSIER_DE_DONNEES: &str = "Dossier de données";
pub const CONFIDENTIALITE: &str =
    "Aucune donnée ne quitte ce poste : pas de réseau, pas de télémétrie, pas de compte.";
/// Version Web : où sont les données, et ce que cela implique.
pub const DONNEES_WEB_TITRE: &str = "Données";
pub const DONNEES_WEB: &str =
    "Vos réglages, vos mémoires et vos rubans sont conservés dans ce navigateur, sur cet appareil.";
pub const CONFIDENTIALITE_WEB: &str = "Rien n'est envoyé sur Internet : pas de télémétrie, pas de compte. Si vous effacez les données de navigation de ce site, elles sont effacées aussi.";
pub const BIBLIOTHEQUES_WEB: &str = "Ce logiciel utilise les bibliothèques libres eframe et egui, serde, serde_json, chrono, wasm-bindgen, js-sys et web-sys (licences MIT ou Apache 2.0).";
pub const POLICES_WEB: &str = "Il affiche les polices libres Selawik, Open Sans (licence SIL OFL 1.1) et DejaVu Sans (licence Bitstream Vera).";
pub const BIBLIOTHEQUES_TITRE: &str = "Bibliothèques libres";
pub const BIBLIOTHEQUES: &str = "Ce logiciel utilise les bibliothèques libres eframe et egui, serde, serde_json et chrono (licences MIT ou Apache 2.0) ainsi que rfd (licence MIT).";

// ----------------------------------------------------------------------
// Raccourcis (section 16.5) : noms des touches du clavier d'ordinateur
// ----------------------------------------------------------------------

pub const TOUCHE_ENTREE: &str = "Entrée";
pub const TOUCHE_ESPACE: &str = "Espace";
pub const TOUCHE_ECHAP: &str = "Échap";
pub const TOUCHE_SUPPR: &str = "Suppr";
pub const TOUCHE_INSER: &str = "Inser";
pub const TOUCHE_RETOUR_ARRIERE: &str = "Retour arrière";

/// Deux raccourcis équivalents : « Entrée ou = ».
pub fn l_un_ou_l_autre(a: &str, b: &str) -> String {
    format!("{a} ou {b}")
}
