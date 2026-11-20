//! Erreurs, messages et avertissements (sections 4.5 et 9.10), dans les deux
//! langues (section 24).

use crate::langue::selon;

/// Anomalie signalée à l'afficheur.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Erreur {
    Depassement,
    Calcul,
    Unites,
    Saisie,
    Trigonometrie,
    AucuneValeur,
}

impl Erreur {
    /// Texte exact affiché (section 4.5), dans la langue en cours.
    pub fn texte(self) -> &'static str {
        match self {
            Erreur::Depassement => selon("ERREUR – dépassement", "ERROR – overflow"),
            Erreur::Calcul => selon("ERREUR – calcul impossible", "ERROR – cannot calculate"),
            Erreur::Unites => selon("ERREUR – unités incompatibles", "ERROR – unit mismatch"),
            Erreur::Saisie => selon("ERREUR – saisie invalide", "ERROR – invalid entry"),
            Erreur::Trigonometrie => selon("ERREUR – trigonométrie", "ERROR – trigonometry"),
            Erreur::AucuneValeur => selon("AUCUNE VALEUR", "NO VALUE"),
        }
    }

    /// Une erreur bloquante n'accepte que la touche Effacer (R-38).
    /// « AUCUNE VALEUR » est un simple message : la touche suivante est acceptée.
    pub fn est_bloquante(self) -> bool {
        !matches!(self, Erreur::AucuneValeur)
    }
}

/// Message d'information plein écran.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Information {
    ToutEfface,
    MemoireEffacee,
    /// Un champ d'un outil de métier vient d'être vidé (section 23).
    ChampVide,
    /// Les champs d'un outil de métier ont repris leurs valeurs de départ.
    OutilRecommence,
}

impl Information {
    pub fn texte(self) -> &'static str {
        match self {
            Information::ToutEfface => selon("TOUT EFFACÉ", "ALL CLEARED"),
            Information::MemoireEffacee => selon("MÉMOIRE EFFACÉE", "MEMORY CLEARED"),
            Information::ChampVide => selon("CHAMP VIDÉ", "FIELD CLEARED"),
            Information::OutilRecommence => selon("CHAMPS REMIS AU DÉPART", "FIELDS RESET"),
        }
    }
}

/// Message d'un outil de métier (section 23.5) : il manque une valeur.
/// Le libellé d'un champ peut tenir sur deux lignes : il s'écrit ici sur une seule.
pub fn valeur_manquante(libelle: &str) -> String {
    format!(
        "{} {}",
        selon("IL MANQUE :", "MISSING:"),
        en_capitales(libelle)
    )
}

/// Message d'un outil de métier : un champ contient une saisie refusée.
pub fn valeur_a_corriger(libelle: &str) -> String {
    format!(
        "{} {}",
        selon("À CORRIGER :", "PLEASE FIX:"),
        en_capitales(libelle)
    )
}

fn en_capitales(libelle: &str) -> String {
    libelle.replace('\n', " ").to_uppercase()
}

/// Avertissement attaché à une sortie de fonction (escalier).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Avertissement {
    ContremarcheHaute,
    EscalierRaide,
}

impl Avertissement {
    pub fn texte(self) -> &'static str {
        match self {
            Avertissement::ContremarcheHaute => selon(
                "Contremarche plus haute que la valeur visée",
                "Riser higher than the target value",
            ),
            Avertissement::EscalierRaide => selon(
                "Escalier raide : inclinaison supérieure de plus de 10 % au rapport visé",
                "Steep stair: incline more than 10% above the target ratio",
            ),
        }
    }
}
