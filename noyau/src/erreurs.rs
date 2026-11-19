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
}

impl Information {
    pub fn texte(self) -> &'static str {
        match self {
            Information::ToutEfface => "TOUT EFFACÉ",
            Information::MemoireEffacee => "MÉMOIRE EFFACÉE",
        }
    }
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
