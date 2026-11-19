//! Fonctions de construction (section 12) : fonctions pures, sans accès au moteur (R-53).

pub mod arc;
pub mod aretier;
pub mod colonne_cone;
pub mod dimensions;
pub mod empannons;
pub mod escalier;
pub mod materiaux;
pub mod mur_pente;
pub mod onglet;
pub mod polygone;
pub mod toiture;
pub mod triangle;
pub mod trigo;

use crate::erreurs::Avertissement;
use crate::etiquettes::Etiquette;
use crate::format::Rendu;
use crate::touches::Touche;
use crate::valeur::Valeur;

/// Nombre maximal d'éléments d'une liste d'empannons ou de montants (annexe A.2).
pub const MAX_ELEMENTS: usize = 200;
/// Longueur en deçà de laquelle un élément de liste est tenu pour nul (en pouces).
pub const SEUIL_LISTE: f64 = 1e-6;

/// Une sortie de fonction : ce que l'afficheur montre à une pression de touche.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sortie {
    pub etiquette: Etiquette,
    pub valeur: Valeur,
    /// Affiche l'indicateur MÉM.
    pub memorise: bool,
    pub avertissement: Option<Avertissement>,
    pub rendu: Rendu,
}

impl Sortie {
    pub fn nouvelle(etiquette: Etiquette, valeur: Valeur) -> Sortie {
        Sortie {
            etiquette,
            valeur,
            memorise: false,
            avertissement: None,
            rendu: Rendu::Normal,
        }
    }

    /// Angle en degrés décimaux.
    pub fn angle(etiquette: Etiquette, degres: f64) -> Sortie {
        Sortie::nouvelle(etiquette, Valeur::degres(degres))
    }

    /// Dénombrement entier.
    pub fn entier(etiquette: Etiquette, n: f64) -> Sortie {
        Sortie::nouvelle(etiquette, Valeur::nombre(n)).rendue(Rendu::Entier)
    }

    pub fn memorisee(mut self) -> Sortie {
        self.memorise = true;
        self
    }

    pub fn memorisee_si(mut self, condition: bool) -> Sortie {
        self.memorise = condition;
        self
    }

    pub fn rendue(mut self, rendu: Rendu) -> Sortie {
        self.rendu = rendu;
        self
    }

    pub fn avertie(mut self, avertissement: Option<Avertissement>) -> Sortie {
        self.avertissement = avertissement;
        self
    }
}

/// Fonction en cours de défilement (section 9.2).
#[derive(Clone, Debug, PartialEq)]
pub struct Cycle {
    pub touche: Touche,
    /// La fonction a été lancée avec `Conv`.
    pub secondaire: bool,
    pub sorties: Vec<Sortie>,
    /// Indice de la prochaine sortie à montrer.
    pub index: usize,
    /// Sortie à sauter une fois (donnée déjà montrée, section 12.8).
    pub a_sauter: Option<usize>,
    /// Sorties déjà inscrites au ruban.
    pub vues: Vec<bool>,
}

impl Cycle {
    pub fn nouveau(touche: Touche, secondaire: bool, sorties: Vec<Sortie>) -> Cycle {
        let n = sorties.len();
        Cycle {
            touche,
            secondaire,
            sorties,
            index: 0,
            a_sauter: None,
            vues: vec![false; n],
        }
    }

    /// Retourne la prochaine sortie et fait avancer l'index, qui reboucle (R-27).
    /// Le second élément dit si la sortie est montrée pour la première fois.
    pub fn suivante(&mut self) -> Option<(Sortie, bool)> {
        let n = self.sorties.len();
        if n == 0 {
            return None;
        }
        let mut i = self.index % n;
        if self.a_sauter == Some(i) {
            self.a_sauter = None;
            i = (i + 1) % n;
        }
        self.index = (i + 1) % n;
        let premiere_fois = !std::mem::replace(&mut self.vues[i], true);
        Some((self.sorties[i], premiere_fois))
    }

    /// Marque une sortie comme déjà montrée et place l'index juste après elle.
    pub fn reprendre_apres(&mut self, i: usize) {
        let n = self.sorties.len();
        if n == 0 {
            return;
        }
        let i = i % n;
        self.vues[i] = true;
        self.index = (i + 1) % n;
    }
}

/// Angle en degrés de l'arc tangente d'un rapport.
pub(crate) fn atan_deg(rapport: f64) -> f64 {
    rapport.atan().to_degrees()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_cycle_reboucle_et_saute_une_fois() {
        let s = |n: f64| Sortie::nouvelle(Etiquette::Nombre, Valeur::nombre(n));
        let mut c = Cycle::nouveau(Touche::Arc, false, vec![s(1.0), s(2.0), s(3.0)]);
        c.a_sauter = Some(1);
        let suite: Vec<f64> = (0..5)
            .map(|_| c.suivante().unwrap().0.valeur.base)
            .collect();
        assert_eq!(suite, vec![1.0, 3.0, 1.0, 2.0, 3.0]);
        let mut vide = Cycle::nouveau(Touche::Arc, false, Vec::new());
        assert!(vide.suivante().is_none());
    }

    #[test]
    fn premiere_fois_signalee_une_seule_fois() {
        let s = Sortie::nouvelle(Etiquette::Nombre, Valeur::nombre(1.0));
        let mut c = Cycle::nouveau(Touche::Diag, false, vec![s, s]);
        assert!(c.suivante().unwrap().1);
        assert!(c.suivante().unwrap().1);
        assert!(!c.suivante().unwrap().1);
    }
}
