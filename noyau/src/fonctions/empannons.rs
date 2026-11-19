//! Empannons (section 12.3).

use super::triangle::Triangle;
use super::{MAX_ELEMENTS, SEUIL_LISTE, Sortie, atan_deg};
use crate::erreurs::Erreur;
use crate::etiquettes::Etiquette;
use crate::prefs::{EmpannonsIrreguliers, Ordre};
use crate::unites::EPS;
use crate::valeur::Valeur;

/// Données du calcul des empannons.
#[derive(Clone, Copy, Debug)]
pub struct DonneesEmpannons {
    pub triangle: Triangle,
    /// Ratio de la pente irrégulière, s'il y a lieu.
    pub pente_irreguliere: Option<f64>,
    /// Entraxe mémorisé.
    pub entraxe: Valeur,
    pub ordre: Ordre,
    pub mode_irregulier: EmpannonsIrreguliers,
    /// Versant irrégulier d'abord (`Conv` `Empannon`).
    pub irregulier_d_abord: bool,
}

/// Liste des longueurs d'un versant de chevron commun `l` et d'incrément `i`.
fn longueurs(l: f64, i: f64, ordre: Ordre) -> Result<Vec<f64>, Erreur> {
    if !(i.is_finite() && i > SEUIL_LISTE && l.is_finite()) {
        return Err(Erreur::Calcul);
    }
    let mut liste = Vec::new();
    let mut k = 1.0;
    loop {
        let longueur = match ordre {
            Ordre::Decroissant => {
                let reste = l - k * i;
                if reste <= SEUIL_LISTE {
                    break;
                }
                reste
            }
            Ordre::Croissant => {
                let cumul = k * i;
                if cumul > l + SEUIL_LISTE {
                    break;
                }
                cumul
            }
        };
        if liste.len() >= MAX_ELEMENTS {
            return Err(Erreur::Depassement);
        }
        liste.push(longueur);
        k += 1.0;
    }
    Ok(liste)
}

struct Versant {
    irregulier: bool,
    entraxe: f64,
    increment: f64,
    commun: f64,
    aplomb: f64,
    laterale: f64,
}

/// Calcule les empannons d'un ou de deux versants.
pub fn calculer_empannons(d: &DonneesEmpannons) -> Result<Vec<Sortie>, Erreur> {
    let t = &d.triangle;
    if d.entraxe.genre != t.genre() {
        return Err(Erreur::Unites);
    }
    let oc = d.entraxe.base;
    if !(oc.is_finite() && oc > 0.0) {
        return Err(Erreur::Saisie);
    }
    let h = t.montee.base;
    let r1 = t.course.base;
    let a1 = t.ratio.atan();
    let l1 = r1 / a1.cos();

    let versants: Vec<Versant> = match d.pente_irreguliere {
        None => {
            if d.irregulier_d_abord {
                return Err(Erreur::AucuneValeur);
            }
            vec![Versant {
                irregulier: false,
                entraxe: oc,
                increment: oc / a1.cos(),
                commun: l1,
                aplomb: a1.to_degrees(),
                laterale: 45.0,
            }]
        }
        Some(t2) => {
            if !(t2.is_finite() && t2 > 0.0) {
                return Err(Erreur::Saisie);
            }
            let a2 = t2.atan();
            let r2 = h / t2;
            let l2 = r2 / a2.cos();
            let (e1, e2) = match d.mode_irregulier {
                EmpannonsIrreguliers::EntraxeConstant => (oc, oc),
                // Appariés : l'entraxe mémorisé va au versant le plus pentu.
                EmpannonsIrreguliers::Apparies => {
                    if r1 >= r2 {
                        (oc * r2 / r1, oc)
                    } else {
                        (oc, oc * r1 / r2)
                    }
                }
            };
            let regulier = Versant {
                irregulier: false,
                entraxe: e1,
                increment: e1 * (r1 / r2) / a1.cos(),
                commun: l1,
                aplomb: a1.to_degrees(),
                laterale: atan_deg(r1 / r2),
            };
            let irregulier = Versant {
                irregulier: true,
                entraxe: e2,
                increment: e2 * (r2 / r1) / a2.cos(),
                commun: l2,
                aplomb: a2.to_degrees(),
                laterale: atan_deg(r2 / r1),
            };
            if d.irregulier_d_abord {
                vec![irregulier, regulier]
            } else {
                vec![regulier, irregulier]
            }
        }
    };

    let mut sorties = Vec::new();
    for v in &versants {
        let entraxe = Valeur {
            base: v.entraxe,
            ..d.entraxe
        };
        let memorise = (v.entraxe - oc).abs() <= EPS * oc.max(1.0);
        sorties.push(Sortie::nouvelle(Etiquette::Entraxe, entraxe).memorisee_si(memorise));
        sorties.push(Sortie::nouvelle(
            Etiquette::Increment,
            t.longueur(v.increment),
        ));
        for (k, longueur) in longueurs(v.commun, v.increment, d.ordre)?
            .into_iter()
            .enumerate()
        {
            let n = (k + 1) as u32;
            let etiquette = if v.irregulier {
                Etiquette::EmpannonIrr(n)
            } else {
                Etiquette::Empannon(n)
            };
            sorties.push(Sortie::nouvelle(etiquette, t.longueur(longueur)));
        }
        sorties.push(Sortie::angle(Etiquette::CoupeAplomb, v.aplomb));
        sorties.push(Sortie::angle(Etiquette::CoupeNiveau, 90.0 - v.aplomb));
        sorties.push(Sortie::angle(Etiquette::CoupeLaterale, v.laterale));
    }
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

    fn donnees(pouces_de_pente: f64, course: f64) -> DonneesEmpannons {
        let triangle = RegistresTriangle {
            pente: Some(PenteMemorisee {
                ratio: pouces_de_pente / 12.0,
                origine: FormePente::Pouces,
                ordre: 1,
            }),
            course: Some((Valeur::en_base(course, Format::PiPo), 2)),
            ..RegistresTriangle::default()
        }
        .resoudre()
        .unwrap();
        DonneesEmpannons {
            triangle,
            pente_irreguliere: None,
            entraxe: Valeur::en_base(16.0, Format::PoFrac),
            ordre: Ordre::Decroissant,
            mode_irregulier: EmpannonsIrreguliers::EntraxeConstant,
            irregulier_d_abord: false,
        }
    }

    fn nb_empannons(s: &[Sortie]) -> usize {
        s.iter()
            .filter(|x| {
                matches!(
                    x.etiquette,
                    Etiquette::Empannon(_) | Etiquette::EmpannonIrr(_)
                )
            })
            .count()
    }

    #[test]
    fn toit_regulier_7_sur_12() {
        let s = calculer_empannons(&donnees(7.0, 101.0)).unwrap();
        assert_eq!(nb_empannons(&s), 6);
        assert!(s[0].memorise);
        assert!((s[1].valeur.base - 18.5231).abs() < 1e-3);
        assert!((s[2].valeur.base - 98.4040).abs() < 1e-3);
        assert_eq!(s.len(), 11);
    }

    #[test]
    fn le_dernier_empannon_nul_n_est_pas_liste() {
        let s = calculer_empannons(&donnees(6.0, 144.0)).unwrap();
        assert_eq!(nb_empannons(&s), 8);
    }

    #[test]
    fn symetrie_decroissante() {
        // Propriété 19.16 : empannon k + k × incrément = chevron commun.
        let d = donnees(9.0, 81.0);
        let s = calculer_empannons(&d).unwrap();
        let increment = s[1].valeur.base;
        let commun = d.triangle.diag.base;
        for x in &s {
            if let Etiquette::Empannon(k) = x.etiquette {
                assert!((x.valeur.base + f64::from(k) * increment - commun).abs() < 1e-9);
            }
        }
    }

    #[test]
    fn toit_irregulier_apparie_et_croissant() {
        let mut d = donnees(7.0, 48.0);
        d.pente_irreguliere = Some(8.0 / 12.0);
        d.irregulier_d_abord = true;
        d.ordre = Ordre::Croissant;
        d.mode_irregulier = EmpannonsIrreguliers::Apparies;
        let s = calculer_empannons(&d).unwrap();
        assert_eq!(s[0].valeur.base, 16.0);
        assert!(s[0].memorise);
        assert_eq!(s[2].etiquette, Etiquette::EmpannonIrr(1));
        assert_eq!(nb_empannons(&s), 6);
        let second = s
            .iter()
            .position(|x| x.etiquette == Etiquette::Entraxe && !x.memorise)
            .unwrap();
        assert!((s[second].valeur.base - 14.0).abs() < 1e-9);
    }

    #[test]
    fn sans_pente_irreguliere_la_fonction_secondaire_n_a_rien() {
        let mut d = donnees(7.0, 48.0);
        d.irregulier_d_abord = true;
        assert_eq!(
            calculer_empannons(&d).map(|_| ()),
            Err(Erreur::AucuneValeur)
        );
    }
}
