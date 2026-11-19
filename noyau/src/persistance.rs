//! État conservé d'une séance à l'autre (section 17.2).
//!
//! Le noyau ne lit ni n'écrit de fichier (R-48) : il fournit seulement la
//! structure sérialisable; l'application s'occupe du disque.

use serde::{Deserialize, Serialize};

use crate::fonctions::triangle::PenteMemorisee;
use crate::memoire::MemoireCumulative;
use crate::prefs::{Preferences, ValeursMemorisees};
use crate::ruban::Ruban;
use crate::valeur::Valeur;

/// Version du format de `etat.json`.
pub const VERSION_FORMAT: u32 = 1;

/// Tout ce que la calculatrice conserve entre deux séances.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EtatPersistant {
    pub version: u32,
    pub preferences: Preferences,
    pub memorises: ValeursMemorisees,
    pub pente: Option<PenteMemorisee>,
    pub memoire: MemoireCumulative,
    pub registres: [Option<Valeur>; 3],
    pub ruban: Ruban,
    pub bienvenue_vue: bool,
}

impl Default for EtatPersistant {
    fn default() -> Self {
        EtatPersistant {
            version: VERSION_FORMAT,
            preferences: Preferences::default(),
            memorises: ValeursMemorisees::default(),
            pente: None,
            memoire: MemoireCumulative::default(),
            registres: [None; 3],
            ruban: Ruban::default(),
            bienvenue_vue: false,
        }
    }
}

impl EtatPersistant {
    /// Vrai si la version du fichier est connue (R-46).
    pub fn version_connue(&self) -> bool {
        self.version == VERSION_FORMAT
    }

    /// Remet en état les valeurs incohérentes d'un fichier modifié à la main.
    pub fn assaini(mut self) -> EtatPersistant {
        self.memorises.assainir();
        self.memoire.assainir();
        self.pente = self.pente.filter(|p| p.ratio.is_finite() && p.ratio > 0.0);
        for registre in &mut self.registres {
            *registre = registre.and_then(Valeur::assainie);
        }
        if self.ruban.lignes.len() > crate::ruban::MAX_LIGNES {
            let surplus = self.ruban.lignes.len() - crate::ruban::MAX_LIGNES;
            self.ruban.lignes.drain(..surplus);
        }
        self
    }
}
