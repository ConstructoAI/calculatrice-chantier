//! Erreurs, messages et avertissements (sections 4.5 et 9.10).

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
    /// Texte exact affiché (section 4.5).
    pub fn texte(self) -> &'static str {
        match self {
            Erreur::Depassement => "ERREUR – dépassement",
            Erreur::Calcul => "ERREUR – calcul impossible",
            Erreur::Unites => "ERREUR – unités incompatibles",
            Erreur::Saisie => "ERREUR – saisie invalide",
            Erreur::Trigonometrie => "ERREUR – trigonométrie",
            Erreur::AucuneValeur => "AUCUNE VALEUR",
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
            Information::ToutEfface => "TOUT EFFACÉ",
            Information::MemoireEffacee => "MÉMOIRE EFFACÉE",
            Information::ChampVide => "CHAMP VIDÉ",
            Information::OutilRecommence => "CHAMPS REMIS AU DÉPART",
        }
    }
}

/// Message d'un outil de métier (section 23.5) : il manque une valeur.
/// Le libellé d'un champ peut tenir sur deux lignes : il s'écrit ici sur une seule.
pub fn valeur_manquante(libelle: &str) -> String {
    format!("IL MANQUE : {}", en_capitales(libelle))
}

/// Message d'un outil de métier : un champ contient une saisie refusée.
pub fn valeur_a_corriger(libelle: &str) -> String {
    format!("À CORRIGER : {}", en_capitales(libelle))
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
            Avertissement::ContremarcheHaute => "Contremarche plus haute que la valeur visée",
            Avertissement::EscalierRaide => {
                "Escalier raide : inclinaison supérieure de plus de 10 % au rapport visé"
            }
        }
    }
}
