//! Triangle rectangle : pente, montée, course, diagonale (section 12.1).

use serde::{Deserialize, Serialize};

use super::{Sortie, atan_deg};
use crate::erreurs::Erreur;
use crate::etiquettes::Etiquette;
use crate::unites::{EPS, Format, Genre, PO_PAR_M, Systeme};
use crate::valeur::Valeur;

/// Forme sous laquelle une pente a été saisie ou est montrée.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum FormePente {
    #[default]
    Pouces,
    Degres,
    Pourcent,
    Ratio,
}

impl FormePente {
    /// Ordre circulaire du cycle : pouces → degrés → pourcentage → ratio.
    fn suivante(self) -> FormePente {
        match self {
            FormePente::Pouces => FormePente::Degres,
            FormePente::Degres => FormePente::Pourcent,
            FormePente::Pourcent => FormePente::Ratio,
            FormePente::Ratio => FormePente::Pouces,
        }
    }
}

/// Pente mémorisée : le ratio montée ÷ course et sa forme d'origine.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PenteMemorisee {
    pub ratio: f64,
    pub origine: FormePente,
    /// Rang chronologique de la saisie; 0 : la plus ancienne (R-41).
    #[serde(skip)]
    pub ordre: u64,
}

/// Interprète une donnée comme une pente (section 12.1).
pub fn interpreter_pente(donnee: &Valeur, comme_ratio: bool) -> Result<(f64, FormePente), Erreur> {
    let (ratio, forme) = if comme_ratio {
        if !donnee.est_sans_unite() {
            return Err(Erreur::Unites);
        }
        (donnee.base, FormePente::Ratio)
    } else {
        match donnee.genre {
            Genre::Longueur => {
                if donnee.format.systeme() == Some(Systeme::Metrique) {
                    // Mètres de montée par mètre de course.
                    (donnee.base / PO_PAR_M, FormePente::Pourcent)
                } else {
                    (donnee.base / 12.0, FormePente::Pouces)
                }
            }
            Genre::SansUnite => {
                if donnee.format == Format::Pourcent {
                    (donnee.base, FormePente::Pourcent)
                } else {
                    let angle = donnee.base;
                    if !(angle > 0.0 && angle < 90.0) {
                        return Err(Erreur::Saisie);
                    }
                    (angle.to_radians().tan(), FormePente::Degres)
                }
            }
            _ => return Err(Erreur::Unites),
        }
    };
    if !(ratio.is_finite() && ratio > 0.0) {
        return Err(Erreur::Saisie);
    }
    Ok((ratio, forme))
}

/// Valeur d'une pente sous une forme donnée, avec son étiquette.
pub fn valeur_pente(ratio: f64, forme: FormePente) -> (Etiquette, Valeur) {
    match forme {
        FormePente::Pouces => (
            Etiquette::Pente,
            Valeur::en_base(ratio * 12.0, Format::PoFrac),
        ),
        FormePente::Degres => (Etiquette::Pente, Valeur::degres(atan_deg(ratio))),
        FormePente::Pourcent => (
            Etiquette::Pente,
            Valeur {
                base: ratio,
                genre: Genre::SansUnite,
                format: Format::Pourcent,
            },
        ),
        FormePente::Ratio => (Etiquette::RatioPente, Valeur::nombre(ratio)),
    }
}

/// Les quatre formes de la pente, en partant de `depart`.
pub fn cycle_pente(ratio: f64, depart: FormePente, memorise: bool) -> Vec<Sortie> {
    let mut forme = depart;
    let mut sorties = Vec::with_capacity(4);
    for _ in 0..4 {
        let (etiquette, valeur) = valeur_pente(ratio, forme);
        sorties.push(Sortie::nouvelle(etiquette, valeur).memorisee_si(memorise));
        forme = forme.suivante();
    }
    sorties
}

/// Côté du triangle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cote {
    Pente,
    Montee,
    Course,
    Diag,
}

/// Registres de saisie du triangle. Chaque longueur porte son rang chronologique.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RegistresTriangle {
    pub pente: Option<PenteMemorisee>,
    pub montee: Option<(Valeur, u64)>,
    pub course: Option<(Valeur, u64)>,
    pub diag: Option<(Valeur, u64)>,
}

/// Triangle résolu.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Triangle {
    /// Pente : montée ÷ course.
    pub ratio: f64,
    pub montee: Valeur,
    pub course: Valeur,
    pub diag: Valeur,
    /// Les deux valeurs saisies qui ont déterminé le triangle.
    pub paire: [Cote; 2],
    /// Genre et format des longueurs calculées.
    modele: Valeur,
}

impl Triangle {
    pub fn est_saisi(&self, cote: Cote) -> bool {
        self.paire.contains(&cote)
    }

    /// Longueur calculée, dans le format de la première longueur fournie.
    pub fn longueur(&self, base: f64) -> Valeur {
        Valeur {
            base,
            ..self.modele
        }
    }

    /// Genre des longueurs du triangle (longueur, ou nombre nu).
    pub fn genre(&self) -> Genre {
        self.modele.genre
    }
}

/// Contrôle une longueur destinée au triangle : positive, longueur ou nombre nu.
pub fn controler_longueur(v: &Valeur) -> Result<Valeur, Erreur> {
    let valeur = match v.genre {
        Genre::Longueur => *v,
        Genre::SansUnite => Valeur::nombre(v.base),
        _ => return Err(Erreur::Unites),
    };
    if !(valeur.base.is_finite() && valeur.base > 0.0) {
        return Err(Erreur::Saisie);
    }
    Ok(valeur)
}

impl RegistresTriangle {
    /// Les deux valeurs saisies les plus récentes (R-41).
    pub fn paire(&self) -> Option<[Cote; 2]> {
        let mut presents: Vec<(Cote, u64)> = Vec::with_capacity(4);
        if let Some(p) = &self.pente {
            presents.push((Cote::Pente, p.ordre));
        }
        if let Some((_, o)) = self.montee {
            presents.push((Cote::Montee, o));
        }
        if let Some((_, o)) = self.course {
            presents.push((Cote::Course, o));
        }
        if let Some((_, o)) = self.diag {
            presents.push((Cote::Diag, o));
        }
        if presents.len() < 2 {
            return None;
        }
        presents.sort_by_key(|p| std::cmp::Reverse(p.1));
        Some([presents[0].0, presents[1].0])
    }

    /// Vrai si les deux valeurs retenues sont deux longueurs (section 12.5).
    pub fn paire_de_longueurs(&self) -> bool {
        self.paire().is_some_and(|p| !p.contains(&Cote::Pente))
    }

    fn longueur(&self, cote: Cote) -> Option<(Valeur, u64)> {
        match cote {
            Cote::Montee => self.montee,
            Cote::Course => self.course,
            Cote::Diag => self.diag,
            Cote::Pente => None,
        }
    }

    /// Résout le triangle à partir des deux valeurs retenues.
    pub fn resoudre(&self) -> Result<Triangle, Erreur> {
        let paire = self.paire().ok_or(Erreur::AucuneValeur)?;
        let a = |c: Cote| paire.contains(&c);
        // Longueurs de la paire, de la plus ancienne à la plus récente.
        let mut longueurs: Vec<(Valeur, u64)> =
            paire.iter().filter_map(|c| self.longueur(*c)).collect();
        longueurs.sort_by_key(|l| l.1);
        let modele = longueurs.first().map(|l| l.0).ok_or(Erreur::AucuneValeur)?;
        if longueurs.iter().any(|l| l.0.genre != modele.genre) {
            return Err(Erreur::Unites);
        }
        let calcule = |base: f64| Valeur { base, ..modele };
        let ratio_saisi = self.pente.map(|p| p.ratio).unwrap_or(0.0);
        let m = self.montee.map(|x| x.0);
        let c = self.course.map(|x| x.0);
        let d = self.diag.map(|x| x.0);
        let (ratio, montee, course, diag) = match (
            a(Cote::Pente),
            a(Cote::Montee),
            a(Cote::Course),
            a(Cote::Diag),
        ) {
            (true, true, _, _) => {
                let montee = m.ok_or(Erreur::AucuneValeur)?;
                let t = ratio_saisi;
                let course = montee.base / t;
                (
                    t,
                    montee,
                    calcule(course),
                    calcule(montee.base.hypot(course)),
                )
            }
            (true, _, true, _) => {
                let course = c.ok_or(Erreur::AucuneValeur)?;
                let t = ratio_saisi;
                let montee = course.base * t;
                (
                    t,
                    calcule(montee),
                    course,
                    calcule(montee.hypot(course.base)),
                )
            }
            (true, _, _, true) => {
                let diag = d.ok_or(Erreur::AucuneValeur)?;
                let t = ratio_saisi;
                let course = diag.base / (1.0 + t * t).sqrt();
                (t, calcule(course * t), calcule(course), diag)
            }
            (_, true, true, _) => {
                let montee = m.ok_or(Erreur::AucuneValeur)?;
                let course = c.ok_or(Erreur::AucuneValeur)?;
                let t = montee.base / course.base;
                (t, montee, course, calcule(montee.base.hypot(course.base)))
            }
            (_, true, _, true) => {
                let montee = m.ok_or(Erreur::AucuneValeur)?;
                let diag = d.ok_or(Erreur::AucuneValeur)?;
                if diag.base <= montee.base * (1.0 + EPS) {
                    return Err(Erreur::Calcul);
                }
                let course = (diag.base * diag.base - montee.base * montee.base).sqrt();
                (montee.base / course, montee, calcule(course), diag)
            }
            (_, _, true, true) => {
                let course = c.ok_or(Erreur::AucuneValeur)?;
                let diag = d.ok_or(Erreur::AucuneValeur)?;
                if diag.base <= course.base * (1.0 + EPS) {
                    return Err(Erreur::Calcul);
                }
                let montee = (diag.base * diag.base - course.base * course.base).sqrt();
                (montee / course.base, calcule(montee), course, diag)
            }
            _ => return Err(Erreur::AucuneValeur),
        };
        let fini = [ratio, montee.base, course.base, diag.base]
            .iter()
            .all(|x| x.is_finite() && *x > 0.0);
        if !fini {
            return Err(Erreur::Calcul);
        }
        Ok(Triangle {
            ratio,
            montee,
            course,
            diag,
            paire,
            modele,
        })
    }

    /// Pente en vigueur : (ratio, saisie?, forme d'origine). `AucuneValeur` s'il n'y en a pas.
    pub fn pente_en_vigueur(&self) -> Result<(f64, bool, FormePente), Erreur> {
        match self.paire() {
            Some(paire) => {
                let triangle = self.resoudre()?;
                let saisie = paire.contains(&Cote::Pente);
                let forme = if saisie {
                    self.pente.map(|p| p.origine).unwrap_or_default()
                } else {
                    FormePente::Pouces
                };
                Ok((triangle.ratio, saisie, forme))
            }
            None => match &self.pente {
                Some(p) => Ok((p.ratio, true, p.origine)),
                None => Err(Erreur::AucuneValeur),
            },
        }
    }

    /// Sortie d'un côté : la valeur saisie (avec MÉM.) ou la valeur calculée.
    pub fn sortie_cote(&self, cote: Cote) -> Result<Sortie, Erreur> {
        let etiquette = match cote {
            Cote::Montee => Etiquette::Montee,
            Cote::Course => Etiquette::Course,
            Cote::Diag | Cote::Pente => Etiquette::Diagonale,
        };
        match self.resoudre() {
            Ok(t) => {
                let valeur = match cote {
                    Cote::Montee => t.montee,
                    Cote::Course => t.course,
                    Cote::Diag | Cote::Pente => t.diag,
                };
                Ok(Sortie::nouvelle(etiquette, valeur).memorisee_si(t.est_saisi(cote)))
            }
            // Triangle incomplet : on réaffiche la valeur saisie, s'il y en a une.
            Err(Erreur::AucuneValeur) => match self.longueur(cote) {
                Some((valeur, _)) => Ok(Sortie::nouvelle(etiquette, valeur).memorisee()),
                None => Err(Erreur::AucuneValeur),
            },
            Err(e) => Err(e),
        }
    }

    /// Cycle de la touche Diag : diagonale, coupe d'aplomb, coupe de niveau.
    pub fn cycle_diag(&self) -> Result<Vec<Sortie>, Erreur> {
        let diagonale = self.sortie_cote(Cote::Diag)?;
        match self.resoudre() {
            Ok(t) => {
                let aplomb = atan_deg(t.ratio);
                Ok(vec![
                    diagonale,
                    Sortie::angle(Etiquette::CoupeAplomb, aplomb),
                    Sortie::angle(Etiquette::CoupeNiveau, 90.0 - aplomb),
                ])
            }
            Err(_) => Ok(vec![diagonale]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pi_po(po: f64) -> Valeur {
        Valeur::en_base(po, Format::PiPo)
    }

    fn pente(ratio: f64, ordre: u64) -> Option<PenteMemorisee> {
        Some(PenteMemorisee {
            ratio,
            origine: FormePente::Pouces,
            ordre,
        })
    }

    #[test]
    fn interpretations_de_la_pente() {
        let (t, f) = interpreter_pente(&Valeur::en_base(9.0, Format::PoFrac), false).unwrap();
        assert!((t - 0.75).abs() < 1e-12 && f == FormePente::Pouces);
        let (t, f) = interpreter_pente(&Valeur::avec_format(250.0, Format::Mm), false).unwrap();
        assert!((t - 0.25).abs() < 1e-12 && f == FormePente::Pourcent);
        let (t, f) = interpreter_pente(&Valeur::nombre(45.0), false).unwrap();
        assert!((t - 1.0).abs() < 1e-12 && f == FormePente::Degres);
        let (t, f) = interpreter_pente(&Valeur::nombre(0.65), true).unwrap();
        assert!((t - 0.65).abs() < 1e-12 && f == FormePente::Ratio);
        assert_eq!(
            interpreter_pente(&Valeur::nombre(90.0), false),
            Err(Erreur::Saisie)
        );
        assert_eq!(
            interpreter_pente(&Valeur::avec_format(2.0, Format::Pi2), false),
            Err(Erreur::Unites)
        );
    }

    #[test]
    fn les_deux_saisies_les_plus_recentes_gagnent() {
        let r = RegistresTriangle {
            pente: pente(7.0 / 12.0, 1),
            montee: Some((pi_po(108.0), 2)),
            course: Some((pi_po(144.0), 3)),
            diag: None,
        };
        let t = r.resoudre().unwrap();
        assert_eq!(t.paire, [Cote::Course, Cote::Montee]);
        assert!((t.ratio - 0.75).abs() < 1e-12);
        assert!((t.diag.base - 180.0).abs() < 1e-9);
        assert!(!t.est_saisi(Cote::Pente));
    }

    #[test]
    fn pente_et_diagonale() {
        let r = RegistresTriangle {
            pente: pente(0.5, 1),
            diag: Some((pi_po(48.0), 2)),
            ..RegistresTriangle::default()
        };
        let t = r.resoudre().unwrap();
        assert!((t.course.base - 48.0 / 1.25f64.sqrt()).abs() < 1e-9);
        assert!((t.montee.base - t.course.base / 2.0).abs() < 1e-9);
    }

    #[test]
    fn geometrie_impossible_et_donnees_insuffisantes() {
        let r = RegistresTriangle {
            montee: Some((pi_po(144.0), 2)),
            diag: Some((pi_po(120.0), 1)),
            ..RegistresTriangle::default()
        };
        assert_eq!(r.resoudre().map(|_| ()), Err(Erreur::Calcul));
        let seul = RegistresTriangle {
            course: Some((pi_po(96.0), 1)),
            ..RegistresTriangle::default()
        };
        assert_eq!(seul.resoudre().map(|_| ()), Err(Erreur::AucuneValeur));
        assert!(seul.sortie_cote(Cote::Course).unwrap().memorise);
        assert_eq!(
            seul.sortie_cote(Cote::Montee).map(|_| ()),
            Err(Erreur::AucuneValeur)
        );
    }

    #[test]
    fn pythagore_sur_mille_couples() {
        // Propriété 19.16 : diagonale² ≈ montée² + course².
        let mut graine: u64 = 0x9E37_79B9_7F4A_7C15;
        let mut suivant = || {
            graine ^= graine << 13;
            graine ^= graine >> 7;
            graine ^= graine << 17;
            (graine >> 11) as f64 / (1u64 << 53) as f64
        };
        for _ in 0..1000 {
            let t = 0.02 + suivant() * 3.0;
            let course = 1.0 + suivant() * 2000.0;
            let r = RegistresTriangle {
                pente: pente(t, 1),
                course: Some((pi_po(course), 2)),
                ..RegistresTriangle::default()
            };
            let tr = r.resoudre().unwrap();
            let gauche = tr.diag.base * tr.diag.base;
            let droite = tr.montee.base * tr.montee.base + tr.course.base * tr.course.base;
            assert!((gauche - droite).abs() <= 1e-9 * droite);
        }
    }

    #[test]
    fn cycle_de_la_pente_dans_l_ordre_circulaire() {
        let c = cycle_pente(0.65, FormePente::Ratio, false);
        assert_eq!(c[0].etiquette, Etiquette::RatioPente);
        assert_eq!(c[1].valeur.format, Format::PoFrac);
        assert_eq!(c[2].valeur.format, Format::Degres);
        assert_eq!(c[3].valeur.format, Format::Pourcent);
    }
}
