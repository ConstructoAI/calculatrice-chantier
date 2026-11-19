//! Mur en pente (section 12.4).

use super::triangle::Triangle;
use super::{MAX_ELEMENTS, SEUIL_LISTE, Sortie, atan_deg};
use crate::erreurs::Erreur;
use crate::etiquettes::Etiquette;
use crate::prefs::Ordre;
use crate::valeur::Valeur;

/// Données du calcul d'un mur en pente.
#[derive(Clone, Copy, Debug)]
pub struct DonneesMurPente {
    pub triangle: Triangle,
    /// Entraxe mémorisé.
    pub entraxe: Valeur,
    /// Hauteur du mur de base, s'il y en a un.
    pub base: Option<Valeur>,
    pub ordre: Ordre,
}

/// Calcule la hauteur de chaque montant d'un mur en pente.
pub fn calculer_mur_pente(d: &DonneesMurPente) -> Result<Vec<Sortie>, Erreur> {
    let t = &d.triangle;
    if d.entraxe.genre != t.genre() {
        return Err(Erreur::Unites);
    }
    let base = match d.base {
        Some(b) if b.genre != t.genre() => return Err(Erreur::Unites),
        Some(b) => b,
        None => t.longueur(0.0),
    };
    let oc = d.entraxe.base;
    if !(oc.is_finite() && oc > SEUIL_LISTE) {
        return Err(Erreur::Saisie);
    }
    let course = t.course.base;
    let mut sorties = vec![Sortie::nouvelle(Etiquette::Entraxe, d.entraxe).memorisee()];
    let mut k = 1.0;
    let mut n = 0u32;
    loop {
        let distance = match d.ordre {
            Ordre::Decroissant => {
                let x = course - k * oc;
                if x <= SEUIL_LISTE {
                    break;
                }
                x
            }
            Ordre::Croissant => {
                let x = k * oc;
                if x > course + SEUIL_LISTE {
                    break;
                }
                x
            }
        };
        if n as usize >= MAX_ELEMENTS {
            return Err(Erreur::Depassement);
        }
        n += 1;
        let hauteur = distance * t.ratio + base.base;
        sorties.push(Sortie::nouvelle(Etiquette::Montant(n), t.longueur(hauteur)));
        k += 1.0;
    }
    sorties.push(Sortie::nouvelle(Etiquette::Base, base));
    sorties.push(Sortie::angle(Etiquette::AngleMur, atan_deg(t.ratio)));
    Ok(sorties)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fonctions::triangle::RegistresTriangle;
    use crate::unites::Format;

    fn donnees(montee: f64, course: f64) -> DonneesMurPente {
        let triangle = RegistresTriangle {
            montee: Some((Valeur::en_base(montee, Format::PiPo), 1)),
            course: Some((Valeur::en_base(course, Format::PiPo), 2)),
            ..RegistresTriangle::default()
        }
        .resoudre()
        .unwrap();
        DonneesMurPente {
            triangle,
            entraxe: Valeur::en_base(16.0, Format::PoFrac),
            base: None,
            ordre: Ordre::Decroissant,
        }
    }

    fn montants(s: &[Sortie]) -> Vec<f64> {
        s.iter()
            .filter(|x| matches!(x.etiquette, Etiquette::Montant(_)))
            .map(|x| x.valeur.base)
            .collect()
    }

    #[test]
    fn decroissant_sans_base() {
        let s = calculer_mur_pente(&donnees(48.0, 96.0)).unwrap();
        assert_eq!(montants(&s), vec![40.0, 32.0, 24.0, 16.0, 8.0]);
        assert_eq!(s[s.len() - 2].valeur.base, 0.0);
        assert!((s[s.len() - 1].valeur.base - 26.565).abs() < 1e-3);
    }

    #[test]
    fn croissant_et_base() {
        let mut d = donnees(48.0, 96.0);
        d.ordre = Ordre::Croissant;
        assert_eq!(
            montants(&calculer_mur_pente(&d).unwrap()),
            vec![8.0, 16.0, 24.0, 32.0, 40.0, 48.0]
        );
        d.ordre = Ordre::Decroissant;
        d.base = Some(Valeur::en_base(60.0, Format::PiPo));
        assert_eq!(
            montants(&calculer_mur_pente(&d).unwrap()),
            vec![100.0, 92.0, 84.0, 76.0, 68.0]
        );
    }
}
