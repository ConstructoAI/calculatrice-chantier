//! Escalier (section 12.6).

use super::{Sortie, atan_deg};
use crate::erreurs::{Avertissement, Erreur};
use crate::etiquettes::Etiquette;
use crate::format::arrondir_a_la_fraction;
use crate::unites::{EPS, Format, Genre, PO_PAR_MM, Systeme};
use crate::valeur::Valeur;

/// Seuil d'avertissement « escalier raide » (annexe A.2).
const FACTEUR_RAIDE: f64 = 1.10;

/// Données du calcul d'un escalier.
#[derive(Clone, Copy, Debug)]
pub struct DonneesEscalier {
    pub montee: Option<Valeur>,
    pub course: Option<Valeur>,
    pub contremarche_visee: Valeur,
    pub giron_vise: Valeur,
    pub echappee: Valeur,
    pub epaisseur_plancher: Valeur,
    /// La contremarche ne dépasse jamais la valeur visée (`Conv` `Escalier`).
    pub contremarche_max: bool,
    /// Résolution des fractions en vigueur (arrondi de chantier, R-42).
    pub resolution: u32,
}

/// Entier le plus proche, la demie vers le haut, au moins 1.
fn arrondi_entier(x: f64) -> f64 {
    (x + 0.5 + EPS).floor().max(1.0)
}

/// Calcule un escalier droit.
pub fn calculer_escalier(d: &DonneesEscalier) -> Result<Vec<Sortie>, Erreur> {
    let principale = d.montee.or(d.course).ok_or(Erreur::AucuneValeur)?;
    for v in [d.montee, d.course].into_iter().flatten() {
        if v.genre != Genre::Longueur {
            return Err(Erreur::Unites);
        }
        if !(v.base.is_finite() && v.base > 0.0) {
            return Err(Erreur::Saisie);
        }
    }
    let cv = d.contremarche_visee.base;
    let gv = d.giron_vise.base;
    if !(cv > 0.0 && gv > 0.0) {
        return Err(Erreur::Saisie);
    }
    let metrique = principale.format.systeme() == Some(Systeme::Metrique);
    // Arrondi de chantier (R-42) : au millimètre en métrique, à la fraction sinon.
    let arr = |x: f64| -> f64 {
        if metrique {
            (x / PO_PAR_MM + 0.5 + EPS).floor() * PO_PAR_MM
        } else {
            arrondir_a_la_fraction(x, d.resolution)
        }
    };
    let nb_contremarches = |montee: f64| -> f64 {
        if d.contremarche_max {
            (montee / cv - EPS).ceil().max(1.0)
        } else {
            arrondi_entier(montee / cv)
        }
    };

    let (n, cm, marches, g, montee, course) = match (d.montee, d.course) {
        (Some(m), None) => {
            let n = nb_contremarches(m.base);
            let cm = arr(m.base / n);
            let marches = n - 1.0;
            let g = arr(gv);
            (n, cm, marches, g, m.base, marches * g)
        }
        (None, Some(c)) => {
            let marches = arrondi_entier(c.base / gv);
            let n = marches + 1.0;
            let cm = arr(cv);
            let g = arr(c.base / marches);
            (n, cm, marches, g, n * cm, c.base)
        }
        (Some(m), Some(c)) => {
            let n = nb_contremarches(m.base);
            if n < 2.0 {
                return Err(Erreur::Saisie);
            }
            let cm = arr(m.base / n);
            let marches = n - 1.0;
            let g = arr(c.base / marches);
            (n, cm, marches, g, m.base, c.base)
        }
        (None, None) => return Err(Erreur::AucuneValeur),
    };
    if !(cm > 0.0 && g > 0.0 && n.is_finite() && n < 1e9) {
        return Err(Erreur::Calcul);
    }

    let (petit, grand) = if metrique {
        (Format::Mm, Format::M)
    } else {
        (Format::PoFrac, Format::PiPo)
    };
    let court = |base: f64| Valeur::en_base(base, petit);
    let long = |base: f64| Valeur::en_base(base, grand);

    let montee_saisie = d.montee.is_some();
    let course_saisie = d.course.is_some();
    let trop_haute = cm > cv * (1.0 + EPS);
    let raide = cm / g > FACTEUR_RAIDE * cv / gv;
    let tremie = (d.echappee.base + d.epaisseur_plancher.base) / cm * g;
    let limon = marches * cm.hypot(g);

    let sorties = vec![
        Sortie::nouvelle(Etiquette::Contremarche, court(cm))
            .memorisee_si(!montee_saisie)
            .avertie(trop_haute.then_some(Avertissement::ContremarcheHaute)),
        Sortie::entier(Etiquette::NbContremarches, n),
        Sortie::nouvelle(Etiquette::EcartContremarches, court(n * cm - montee)),
        Sortie::nouvelle(Etiquette::Giron, court(g)).memorisee_si(!course_saisie),
        Sortie::entier(Etiquette::NbMarches, marches),
        Sortie::nouvelle(Etiquette::EcartGirons, court(marches * g - course)),
        Sortie::nouvelle(Etiquette::Tremie, long(tremie)),
        Sortie::nouvelle(Etiquette::Limon, long(limon)),
        Sortie::angle(Etiquette::Inclinaison, atan_deg(cm / g))
            .avertie(raide.then_some(Avertissement::EscalierRaide)),
        Sortie::nouvelle(Etiquette::Course, long(course)).memorisee_si(course_saisie),
        Sortie::nouvelle(Etiquette::Montee, long(montee)).memorisee_si(montee_saisie),
        Sortie::nouvelle(Etiquette::ContremarcheVisee, d.contremarche_visee).memorisee(),
        Sortie::nouvelle(Etiquette::GironVise, d.giron_vise).memorisee(),
        Sortie::nouvelle(Etiquette::Echappee, d.echappee).memorisee(),
        Sortie::nouvelle(Etiquette::EpaisseurPlancher, d.epaisseur_plancher).memorisee(),
    ];
    if sorties.iter().any(|s| !s.valeur.base.is_finite()) {
        return Err(Erreur::Calcul);
    }
    Ok(sorties)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn donnees(montee: Option<Valeur>, course: Option<Valeur>, max: bool) -> DonneesEscalier {
        DonneesEscalier {
            montee,
            course,
            contremarche_visee: Valeur::en_base(7.5, Format::PoFrac),
            giron_vise: Valeur::en_base(10.0, Format::PoFrac),
            echappee: Valeur::en_base(80.0, Format::PiPo),
            epaisseur_plancher: Valeur::en_base(10.0, Format::PoFrac),
            contremarche_max: max,
            resolution: 16,
        }
    }

    fn pi_po(po: f64) -> Option<Valeur> {
        Some(Valeur::en_base(po, Format::PiPo))
    }

    #[test]
    fn montee_seule() {
        let s = calculer_escalier(&donnees(pi_po(119.0), None, false)).unwrap();
        assert_eq!(s[0].valeur.base, 7.4375);
        assert_eq!(s[1].valeur.base, 16.0);
        assert_eq!(s[2].valeur.base, 0.0);
        assert!(s[3].memorise && s[3].valeur.base == 10.0);
        assert_eq!(s[4].valeur.base, 15.0);
        assert!((s[6].valeur.base - 121.008).abs() < 1e-3);
        assert!((s[7].valeur.base - 186.942).abs() < 1e-2);
        assert!((s[8].valeur.base - 36.64).abs() < 5e-3);
        assert_eq!(s[9].valeur.base, 150.0);
        assert!(s[10].memorise);
        assert!(s[0].avertissement.is_none());
        assert_eq!(s.len(), 15);
    }

    #[test]
    fn montee_et_course_avec_avertissement() {
        let s = calculer_escalier(&donnees(pi_po(121.0), pi_po(185.0), false)).unwrap();
        assert_eq!(s[0].valeur.base, 7.5625);
        assert_eq!(s[0].avertissement, Some(Avertissement::ContremarcheHaute));
        assert_eq!(s[3].valeur.base, 12.3125);
        assert_eq!(s[5].valeur.base, -0.3125);
        let max = calculer_escalier(&donnees(pi_po(121.0), pi_po(185.0), true)).unwrap();
        assert_eq!(max[1].valeur.base, 17.0);
        assert_eq!(max[0].valeur.base, 7.125);
        assert_eq!(max[2].valeur.base, 0.125);
        assert!(max[0].avertissement.is_none());
    }

    #[test]
    fn course_seule() {
        let s = calculer_escalier(&donnees(None, pi_po(240.0), false)).unwrap();
        assert!(s[0].memorise && s[0].valeur.base == 7.5);
        assert_eq!(s[1].valeur.base, 25.0);
        assert_eq!(s[10].valeur.base, 187.5);
        assert!(s[9].memorise && !s[10].memorise);
    }

    #[test]
    fn escalier_metrique_arrondi_au_millimetre() {
        let montee = Some(Valeur::avec_format(2.75, Format::M));
        let s = calculer_escalier(&donnees(montee, None, false)).unwrap();
        let mm = |v: &Valeur| v.base / PO_PAR_MM;
        assert!((mm(&s[0].valeur) - 196.0).abs() < 1e-9);
        assert_eq!(s[0].valeur.format, Format::Mm);
        assert!((mm(&s[2].valeur) + 6.0).abs() < 1e-9);
        assert!((mm(&s[3].valeur) - 254.0).abs() < 1e-9);
        assert_eq!(s[6].valeur.format, Format::M);
        assert!((mm(&s[6].valeur) - 2962.47).abs() < 0.01);
    }

    #[test]
    fn donnees_manquantes_ou_invalides() {
        assert_eq!(
            calculer_escalier(&donnees(None, None, false)).map(|_| ()),
            Err(Erreur::AucuneValeur)
        );
        let nombre = Some(Valeur::nombre(100.0));
        assert_eq!(
            calculer_escalier(&donnees(nombre, None, false)).map(|_| ()),
            Err(Erreur::Unites)
        );
        let basse = pi_po(3.0);
        assert_eq!(
            calculer_escalier(&donnees(basse, pi_po(100.0), false)).map(|_| ()),
            Err(Erreur::Saisie)
        );
    }
}
