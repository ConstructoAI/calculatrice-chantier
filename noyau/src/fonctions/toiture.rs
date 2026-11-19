//! Matériaux de toiture (section 12.5).

use super::Sortie;
use crate::erreurs::Erreur;
use crate::etiquettes::Etiquette;
use crate::format::Rendu;
use crate::unites::{Format, Genre, PO2_PAR_PI2};
use crate::valeur::Valeur;

/// Carré de toiture, en pieds carrés (annexe A.2).
const PI2_PAR_CARRE: f64 = 100.0;
/// Feuille de 4 × 8, en pieds carrés.
const PI2_PAR_FEUILLE: f64 = 32.0;

/// Données du calcul de toiture.
#[derive(Clone, Copy, Debug)]
pub struct DonneesToiture {
    /// Pente : montée ÷ course.
    pub ratio: f64,
    /// La pente provient d'une saisie (indicateur MÉM.).
    pub pente_saisie: bool,
    pub aire_plan: Valeur,
    pub paquets_par_carre: f64,
}

/// Calcule l'aire du toit et les quantités de matériaux.
pub fn calculer_toiture(d: &DonneesToiture) -> Result<Vec<Sortie>, Erreur> {
    if d.aire_plan.genre != Genre::Aire {
        return Err(Erreur::Unites);
    }
    if !(d.aire_plan.base > 0.0 && d.ratio > 0.0 && d.paquets_par_carre > 0.0) {
        return Err(Erreur::Saisie);
    }
    // aire ÷ cos(atan t) = aire × √(1 + t²)
    let aire = d.aire_plan.base * (1.0 + d.ratio * d.ratio).sqrt();
    if !aire.is_finite() {
        return Err(Erreur::Depassement);
    }
    let pi2 = aire / PO2_PAR_PI2;
    let carres = pi2 / PI2_PAR_CARRE;
    let deux = |etiquette: Etiquette, x: f64| {
        Sortie::nouvelle(etiquette, Valeur::nombre(x)).rendue(Rendu::DeuxDecimales)
    };
    Ok(vec![
        Sortie::nouvelle(
            Etiquette::AireToit,
            Valeur {
                base: aire,
                ..d.aire_plan
            },
        ),
        deux(Etiquette::Carres, carres),
        deux(Etiquette::Paquets, carres * d.paquets_par_carre),
        Sortie::nouvelle(
            Etiquette::Paquet,
            Valeur::avec_format(PI2_PAR_CARRE / d.paquets_par_carre, Format::Pi2),
        )
        .rendue(Rendu::DeuxDecimales)
        .memorisee(),
        deux(Etiquette::Feuilles4x8, pi2 / PI2_PAR_FEUILLE),
        Sortie::nouvelle(
            Etiquette::Pente,
            Valeur::en_base(d.ratio * 12.0, Format::PoFrac),
        )
        .memorisee_si(d.pente_saisie),
        Sortie::nouvelle(Etiquette::AirePlan, d.aire_plan),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toiture_10_sur_12() {
        let s = calculer_toiture(&DonneesToiture {
            ratio: 10.0 / 12.0,
            pente_saisie: true,
            aire_plan: Valeur::avec_format(154.0, Format::Pi2),
            paquets_par_carre: 3.0,
        })
        .unwrap();
        assert!((s[0].valeur.numerique() - 200.4631).abs() < 1e-4);
        assert!((s[1].valeur.base - 2.004_631).abs() < 1e-6);
        assert!((s[2].valeur.base - 6.013_893).abs() < 1e-6);
        assert!((s[3].valeur.numerique() - 33.333_333).abs() < 1e-6);
        assert!((s[4].valeur.base - 6.264_472).abs() < 1e-6);
        assert!(s[5].memorise && s[5].valeur.base == 10.0);
        assert_eq!(s[6].valeur.numerique(), 154.0);
    }

    #[test]
    fn aire_en_plan_obligatoire() {
        let d = DonneesToiture {
            ratio: 0.5,
            pente_saisie: true,
            aire_plan: Valeur::en_base(100.0, Format::PiPo),
            paquets_par_carre: 3.0,
        };
        assert_eq!(calculer_toiture(&d).map(|_| ()), Err(Erreur::Unites));
    }
}
