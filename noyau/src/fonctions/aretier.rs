//! Arêtier et noue (section 12.2).

use super::triangle::Triangle;
use super::{Sortie, atan_deg};
use crate::erreurs::Erreur;
use crate::etiquettes::Etiquette;

/// Calcule l'arêtier ou la noue d'un toit régulier ou irrégulier.
///
/// `pente_irreguliere` : ratio de la seconde pente, s'il y a lieu.
pub fn calculer_aretier(
    triangle: &Triangle,
    pente_irreguliere: Option<f64>,
) -> Result<Vec<Sortie>, Erreur> {
    let h = triangle.montee.base;
    let r1 = triangle.course.base;
    let sorties = match pente_irreguliere {
        None => {
            let en_plan = r1 * std::f64::consts::SQRT_2;
            let longueur = (h * h + en_plan * en_plan).sqrt();
            let aplomb = atan_deg(h / en_plan);
            vec![
                Sortie::nouvelle(Etiquette::AretierNoue, triangle.longueur(longueur)),
                Sortie::angle(Etiquette::CoupeAplomb, aplomb),
                Sortie::angle(Etiquette::CoupeNiveau, 90.0 - aplomb),
                Sortie::angle(Etiquette::CoupeLaterale, 45.0),
            ]
        }
        Some(t2) => {
            if !(t2.is_finite() && t2 > 0.0) {
                return Err(Erreur::Saisie);
            }
            // La montée est commune aux deux versants.
            let r2 = h / t2;
            let en_plan = r1.hypot(r2);
            let longueur = (en_plan * en_plan + h * h).sqrt();
            let aplomb = atan_deg(h / en_plan);
            let laterale_1 = atan_deg(r2 / r1);
            vec![
                Sortie::nouvelle(Etiquette::AretierNoueIrr, triangle.longueur(longueur)),
                Sortie::angle(Etiquette::CoupeAplomb, aplomb),
                Sortie::angle(Etiquette::CoupeNiveau, 90.0 - aplomb),
                Sortie::angle(Etiquette::CoupeLaterale1, laterale_1),
                Sortie::angle(Etiquette::CoupeLaterale2, 90.0 - laterale_1),
            ]
        }
    };
    if sorties.iter().any(|s| !s.valeur.base.is_finite()) {
        return Err(Erreur::Calcul);
    }
    Ok(sorties)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fonctions::triangle::{FormePente, PenteMemorisee, RegistresTriangle};
    use crate::unites::Format;
    use crate::valeur::Valeur;

    fn triangle(pouces_de_pente: f64, course: f64) -> Triangle {
        RegistresTriangle {
            pente: Some(PenteMemorisee {
                ratio: pouces_de_pente / 12.0,
                origine: FormePente::Pouces,
                ordre: 1,
            }),
            course: Some((Valeur::en_base(course, Format::PiPo), 2)),
            ..RegistresTriangle::default()
        }
        .resoudre()
        .unwrap()
    }

    #[test]
    fn toit_regulier_7_sur_12() {
        let s = calculer_aretier(&triangle(7.0, 101.0), None).unwrap();
        assert!((s[0].valeur.base - 154.5096).abs() < 1e-3);
        assert!((s[1].valeur.base - 22.4157).abs() < 1e-3);
        assert!((s[2].valeur.base - 67.5843).abs() < 1e-3);
        assert_eq!(s[3].valeur.base, 45.0);
    }

    #[test]
    fn toit_irregulier_7_et_8_sur_12() {
        let s = calculer_aretier(&triangle(7.0, 48.0), Some(8.0 / 12.0)).unwrap();
        assert_eq!(s[0].etiquette, Etiquette::AretierNoueIrr);
        assert!((s[0].valeur.base - 4852f64.sqrt()).abs() < 1e-9);
        assert!((s[1].valeur.base - 23.70).abs() < 5e-3);
        assert!((s[3].valeur.base - 41.19).abs() < 5e-3);
        assert!((s[4].valeur.base - 48.81).abs() < 5e-3);
    }
}
