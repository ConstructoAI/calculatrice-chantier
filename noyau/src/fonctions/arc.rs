//! Cercle, arc, rayon et murs cintrés (section 12.8).

use std::f64::consts::PI;

use super::{MAX_ELEMENTS, SEUIL_LISTE, Sortie};
use crate::erreurs::Erreur;
use crate::etiquettes::Etiquette;
use crate::prefs::{MurCintre, Preferences};
use crate::unites::{EPS, Genre};
use crate::valeur::{Valeur, format_aire};

/// Arc saisi : un angle au centre (degrés) ou une longueur d'arc.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ArcSaisi {
    Angle(f64),
    Longueur(Valeur),
}

/// Données connues d'un cercle ou d'un arc.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DonneesArc {
    pub diametre: Option<Valeur>,
    pub arc: Option<ArcSaisi>,
    /// Corde : la course saisie.
    pub corde: Option<Valeur>,
    /// Flèche : la montée saisie.
    pub fleche: Option<Valeur>,
}

/// Arc résolu : rayon et angle au centre.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ArcResolu {
    pub rayon: f64,
    /// Angle au centre, en radians.
    pub theta: f64,
    /// Genre et format des longueurs calculées.
    modele: Valeur,
}

impl ArcResolu {
    pub fn longueur(&self, base: f64) -> Valeur {
        Valeur {
            base,
            ..self.modele
        }
    }

    pub fn longueur_arc(&self) -> f64 {
        self.rayon * self.theta
    }

    pub fn corde(&self) -> f64 {
        2.0 * self.rayon * (self.theta / 2.0).sin()
    }

    pub fn fleche(&self) -> f64 {
        self.rayon * (1.0 - (self.theta / 2.0).cos())
    }

    pub fn aire_segment(&self) -> f64 {
        self.rayon * self.rayon * (self.theta - self.theta.sin()) / 2.0
    }

    pub fn aire_secteur(&self) -> f64 {
        self.rayon * self.rayon * self.theta / 2.0
    }
}

/// Contrôle une donnée de cercle : longueur ou nombre nu, positive.
pub fn controler_longueur(v: &Valeur) -> Result<Valeur, Erreur> {
    super::triangle::controler_longueur(v)
}

/// Interprète la donnée de la touche Arc : un angle (nombre nu) ou une longueur d'arc.
pub fn interpreter_arc(donnee: &Valeur) -> Result<ArcSaisi, Erreur> {
    match donnee.genre {
        Genre::SansUnite => {
            if donnee.base.is_finite() && donnee.base > 0.0 && donnee.base < 360.0 {
                Ok(ArcSaisi::Angle(donnee.base))
            } else {
                Err(Erreur::Saisie)
            }
        }
        Genre::Longueur => {
            if donnee.base.is_finite() && donnee.base > 0.0 {
                Ok(ArcSaisi::Longueur(*donnee))
            } else {
                Err(Erreur::Saisie)
            }
        }
        _ => Err(Erreur::Unites),
    }
}

fn meme_genre(a: &Valeur, b: &Valeur) -> Result<(), Erreur> {
    if a.genre == b.genre {
        Ok(())
    } else {
        Err(Erreur::Unites)
    }
}

impl DonneesArc {
    /// Résout l'arc : premier cas dont les données sont présentes (section 12.8).
    pub fn resoudre(&self) -> Result<ArcResolu, Erreur> {
        let resolu = |rayon: f64, theta: f64, modele: Valeur| -> Result<ArcResolu, Erreur> {
            if rayon.is_finite()
                && rayon > 0.0
                && theta.is_finite()
                && theta > 0.0
                && theta <= 2.0 * PI * (1.0 + EPS)
            {
                Ok(ArcResolu {
                    rayon,
                    theta,
                    modele,
                })
            } else {
                Err(Erreur::Calcul)
            }
        };
        if let Some(d) = &self.diametre {
            let r = d.base / 2.0;
            match (&self.arc, &self.corde, &self.fleche) {
                (Some(ArcSaisi::Angle(a)), _, _) => return resolu(r, a.to_radians(), *d),
                (Some(ArcSaisi::Longueur(s)), _, _) => {
                    meme_genre(d, s)?;
                    return resolu(r, s.base / r, *d);
                }
                (None, Some(c), _) => {
                    meme_genre(d, c)?;
                    let rapport = c.base / (2.0 * r);
                    if rapport > 1.0 + EPS {
                        return Err(Erreur::Calcul);
                    }
                    return resolu(r, 2.0 * rapport.min(1.0).asin(), *d);
                }
                (None, None, Some(h)) => {
                    meme_genre(d, h)?;
                    if h.base > 2.0 * r * (1.0 + EPS) {
                        return Err(Erreur::Calcul);
                    }
                    return resolu(r, 2.0 * (1.0 - h.base / r).clamp(-1.0, 1.0).acos(), *d);
                }
                (None, None, None) => {}
            }
        }
        match (&self.arc, &self.corde, &self.fleche) {
            (_, Some(c), Some(h)) => {
                meme_genre(c, h)?;
                let r = (c.base * c.base / 4.0 + h.base * h.base) / (2.0 * h.base);
                resolu(r, 4.0 * (2.0 * h.base / c.base).atan(), *c)
            }
            (Some(ArcSaisi::Angle(a)), Some(c), None) => {
                let theta = a.to_radians();
                resolu(c.base / (2.0 * (theta / 2.0).sin()), theta, *c)
            }
            (Some(ArcSaisi::Angle(a)), None, Some(h)) => {
                let theta = a.to_radians();
                resolu(h.base / (1.0 - (theta / 2.0).cos()), theta, *h)
            }
            _ => Err(Erreur::AucuneValeur),
        }
    }

    /// Rayon connu (diamètre saisi) ou résolu, avec le modèle des longueurs.
    pub fn rayon(&self) -> Result<(f64, Valeur), Erreur> {
        match &self.diametre {
            Some(d) => Ok((d.base / 2.0, *d)),
            None => {
                let a = self.resoudre()?;
                Ok((a.rayon, a.modele))
            }
        }
    }

    /// Sortie RAYON.
    pub fn sortie_rayon(&self) -> Result<Sortie, Erreur> {
        let (r, modele) = self.rayon()?;
        Ok(Sortie::nouvelle(
            Etiquette::Rayon,
            Valeur { base: r, ..modele },
        ))
    }

    /// Cycle de la touche Cercle : aire, circonférence, diamètre.
    pub fn cycle_cercle(&self, prefs: &Preferences) -> Result<Vec<Sortie>, Erreur> {
        let (r, modele) = self.rayon()?;
        let longueur = |base: f64| Valeur { base, ..modele };
        Ok(vec![
            Sortie::nouvelle(Etiquette::Aire, aire(PI * r * r, &modele, prefs)),
            Sortie::nouvelle(Etiquette::Circonference, longueur(2.0 * PI * r)),
            Sortie::nouvelle(Etiquette::Diametre, longueur(2.0 * r))
                .memorisee_si(self.diametre.is_some()),
        ])
    }

    /// Cycle de la touche Arc : angle, longueur d'arc, corde, segment, secteur,
    /// flèche, puis l'entraxe et les montants du mur cintré.
    pub fn cycle_arc(
        &self,
        entraxe: &Valeur,
        base: Option<Valeur>,
        mur: MurCintre,
        prefs: &Preferences,
    ) -> Result<Vec<Sortie>, Erreur> {
        let a = self.resoudre()?;
        let r = a.rayon;
        let corde = a.corde();
        let fleche = a.fleche();
        let mut sorties = vec![
            Sortie::angle(Etiquette::Arc, a.theta.to_degrees()),
            Sortie::nouvelle(Etiquette::Arc, a.longueur(a.longueur_arc())),
            Sortie::nouvelle(Etiquette::Corde, a.longueur(corde)),
            Sortie::nouvelle(Etiquette::Segment, aire(a.aire_segment(), &a.modele, prefs)),
            Sortie::nouvelle(Etiquette::Secteur, aire(a.aire_secteur(), &a.modele, prefs)),
            Sortie::nouvelle(Etiquette::Fleche, a.longueur(fleche)),
        ];
        // Murs cintrés : seulement pour de vraies longueurs.
        if entraxe.genre == a.modele.genre && entraxe.base > SEUIL_LISTE {
            sorties.push(Sortie::nouvelle(Etiquette::Entraxe, *entraxe).memorisee());
            if a.theta <= PI * (1.0 + EPS) {
                let hauteur_base = match base {
                    Some(b) if b.genre == a.modele.genre => b.base,
                    _ => 0.0,
                };
                let mut k = 1.0;
                let mut n = 0u32;
                loop {
                    let x = k * entraxe.base;
                    if x >= corde / 2.0 - SEUIL_LISTE {
                        break;
                    }
                    if n as usize >= MAX_ELEMENTS {
                        return Err(Erreur::Depassement);
                    }
                    n += 1;
                    let sous_arc = (r * r - x * x).max(0.0).sqrt();
                    let hauteur = match mur {
                        // Au-dessus de l'arc, jusqu'à l'horizontale du sommet.
                        MurCintre::Exterieur => r - sous_arc + hauteur_base,
                        // Sous l'arc, à partir de la corde.
                        MurCintre::Interieur => sous_arc - (r - fleche) + hauteur_base,
                    };
                    sorties.push(Sortie::nouvelle(
                        Etiquette::MurCintre(n),
                        a.longueur(hauteur),
                    ));
                    k += 1.0;
                }
            }
        }
        if sorties.iter().any(|s| !s.valeur.base.is_finite()) {
            return Err(Erreur::Calcul);
        }
        Ok(sorties)
    }
}

/// Aire dans l'unité qui convient aux longueurs du modèle.
fn aire(base: f64, modele: &Valeur, prefs: &Preferences) -> Valeur {
    if modele.genre == Genre::Longueur {
        Valeur::en_base(base, format_aire(&[modele.format], prefs))
    } else {
        Valeur::nombre(base)
    }
}

/// Flèche d'un arc de diamètre et de corde connus (R-43).
pub fn fleche_depuis_corde(diametre: &Valeur, corde: &Valeur) -> Result<Valeur, Erreur> {
    meme_genre(diametre, corde)?;
    let r = diametre.base / 2.0;
    let demi = corde.base / 2.0;
    if demi > r * (1.0 + EPS) {
        return Err(Erreur::Calcul);
    }
    Ok(Valeur {
        base: r - (r * r - demi * demi).max(0.0).sqrt(),
        ..*diametre
    })
}

/// Corde d'un arc de diamètre et de flèche connus (R-43).
pub fn corde_depuis_fleche(diametre: &Valeur, fleche: &Valeur) -> Result<Valeur, Erreur> {
    meme_genre(diametre, fleche)?;
    let r = diametre.base / 2.0;
    let h = fleche.base;
    if h > 2.0 * r * (1.0 + EPS) {
        return Err(Erreur::Calcul);
    }
    Ok(Valeur {
        base: 2.0 * (2.0 * r * h - h * h).max(0.0).sqrt(),
        ..*diametre
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unites::Format;

    fn pi_po(po: f64) -> Valeur {
        Valeur::en_base(po, Format::PiPo)
    }

    fn entraxe() -> Valeur {
        Valeur::en_base(16.0, Format::PoFrac)
    }

    fn murs(s: &[Sortie]) -> Vec<f64> {
        s.iter()
            .filter(|x| matches!(x.etiquette, Etiquette::MurCintre(_)))
            .map(|x| x.valeur.base)
            .collect()
    }

    #[test]
    fn rayon_et_angle() {
        let d = DonneesArc {
            diametre: Some(pi_po(60.0)),
            arc: Some(ArcSaisi::Angle(85.0)),
            ..DonneesArc::default()
        };
        let a = d.resoudre().unwrap();
        assert!((a.longueur_arc() - 44.506).abs() < 1e-3);
    }

    #[test]
    fn corde_et_fleche() {
        let d = DonneesArc {
            corde: Some(pi_po(42.0)),
            fleche: Some(pi_po(15.0)),
            ..DonneesArc::default()
        };
        let a = d.resoudre().unwrap();
        assert!((a.rayon - 22.2).abs() < 1e-9);
        assert!((a.theta.to_degrees() - 142.15).abs() < 5e-3);
        assert!((a.longueur_arc() - 55.078).abs() < 1e-3);
        assert!((d.sortie_rayon().unwrap().valeur.base - 22.2).abs() < 1e-9);
    }

    #[test]
    fn rayon_et_longueur_d_arc() {
        let d = DonneesArc {
            diametre: Some(pi_po(60.0)),
            arc: Some(ArcSaisi::Longueur(pi_po(39.0))),
            ..DonneesArc::default()
        };
        let p = Preferences::default();
        let s = d
            .cycle_arc(&entraxe(), None, MurCintre::Exterieur, &p)
            .unwrap();
        assert!((s[0].valeur.base - 74.48).abs() < 5e-3);
        assert!((s[2].valeur.base - 36.3112).abs() < 1e-3);
        assert!((s[3].valeur.numerique() - 1.051_381).abs() < 1e-6);
        assert!((s[4].valeur.numerique() - 4.0625).abs() < 1e-9);
        assert!((s[5].valeur.base - 6.1175).abs() < 1e-3);
        assert!(s[6].memorise);
        let m = murs(&s);
        assert_eq!(m.len(), 1);
        assert!((m[0] - 4.623).abs() < 1e-3);
    }

    #[test]
    fn murs_cintres_exterieurs_et_interieurs() {
        let p = Preferences::default();
        let ext = DonneesArc {
            corde: Some(pi_po(97.0)),
            fleche: Some(pi_po(34.5)),
            ..DonneesArc::default()
        };
        let m = murs(
            &ext.cycle_arc(&entraxe(), None, MurCintre::Exterieur, &p)
                .unwrap(),
        );
        assert_eq!(m.len(), 3);
        assert!(
            (m[0] - 2.557).abs() < 1e-3
                && (m[1] - 11.192).abs() < 1e-3
                && (m[2] - 33.124).abs() < 1e-2
        );
        let avec_base = murs(
            &ext.cycle_arc(&entraxe(), Some(pi_po(24.0)), MurCintre::Exterieur, &p)
                .unwrap(),
        );
        assert!((avec_base[0] - m[0] - 24.0).abs() < 1e-9);
        let int = DonneesArc {
            corde: Some(pi_po(180.0)),
            fleche: Some(pi_po(60.0)),
            ..DonneesArc::default()
        };
        let m = murs(
            &int.cycle_arc(&entraxe(), None, MurCintre::Interieur, &p)
                .unwrap(),
        );
        assert_eq!(m.len(), 5);
        assert!((m[0] - 58.678).abs() < 1e-3 && (m[4] - 18.234).abs() < 1e-2);
    }

    #[test]
    fn fleche_et_corde_avec_diametre() {
        let f = fleche_depuis_corde(&pi_po(120.0), &pi_po(72.0)).unwrap();
        assert!((f.base - 12.0).abs() < 1e-9);
        let c = corde_depuis_fleche(&pi_po(120.0), &pi_po(12.0)).unwrap();
        assert!((c.base - 72.0).abs() < 1e-9);
        assert_eq!(
            fleche_depuis_corde(&pi_po(60.0), &pi_po(72.0)),
            Err(Erreur::Calcul)
        );
    }

    #[test]
    fn cas_sans_solution() {
        let d = DonneesArc {
            arc: Some(ArcSaisi::Longueur(pi_po(39.0))),
            corde: Some(pi_po(30.0)),
            ..DonneesArc::default()
        };
        assert_eq!(d.resoudre().map(|_| ()), Err(Erreur::AucuneValeur));
        assert_eq!(
            DonneesArc::default()
                .cycle_cercle(&Preferences::default())
                .map(|_| ()),
            Err(Erreur::AucuneValeur)
        );
    }
}
