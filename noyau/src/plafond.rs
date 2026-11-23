//! Plafonds (section 25.1, point 34) : la hauteur du plafond en chaque point
//! d'une pièce — plat, cathédrale ou en pente — et ce qui en découle, au
//! pouce carré et au pouce cube près : la surface de chaque mur (un mur en
//! pignon monte jusqu'au faîte), celle du plafond et le volume de la pièce.
//!
//! Les points sont en pouces, dans le repère du plan (x vers la droite, y
//! vers le bas); un contour fermé répète son départ à la fin.

use serde::{Deserialize, Serialize};

/// La forme d'un plafond.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum FormeDePlafond {
    /// À la hauteur des murs partout.
    #[default]
    Plat,
    /// Deux pentes qui montent de la hauteur des murs jusqu'au faîte, au
    /// milieu de la pièce, parallèle au côté choisi : les deux murs qui
    /// croisent le faîte sont en pignon.
    Cathedrale,
    /// Une seule pente, du côté choisi (le haut) jusqu'au plus loin de la
    /// pièce (la hauteur des murs).
    EnPente,
}

/// Le plafond d'une pièce, tel qu'inscrit.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Plafond {
    pub forme: FormeDePlafond,
    /// La hauteur au faîte (cathédrale) ou du côté haut (en pente), telle
    /// qu'inscrite.
    pub haut: String,
    /// Le côté auquel le faîte est parallèle (cathédrale), ou le côté haut
    /// (en pente), à partir de 0.
    pub cote: usize,
}

/// Une retombée de plafond : un caisson qui descend sous le plafond (au-dessus
/// des armoires, autour d'une poutre ou d'un conduit), telle qu'inscrite.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Retombee {
    pub longueur: String,
    pub largeur: String,
    /// De combien elle descend sous le plafond.
    pub hauteur: String,
}

/// Le plafond à ses mesures : sa hauteur en chaque point du plan.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlafondMesure {
    pub forme: FormeDePlafond,
    /// La hauteur des murs (pouces) : celle d'un plafond plat, et du bas des
    /// pentes.
    pub bas: f64,
    /// La hauteur au faîte, ou du côté haut (pouces).
    pub haut: f64,
    /// La normale au côté choisi (unitaire) : la pente monte ou descend le
    /// long de `v = normale · point`.
    pub normale: (f64, f64),
    /// Le faîte (cathédrale) ou le côté haut (en pente), en `v`.
    pub ligne: f64,
    /// La distance, en `v`, du faîte ou du côté haut au plus loin de la pièce.
    pub portee: f64,
}

impl PlafondMesure {
    /// Le plafond d'une pièce au contour `sommets` (fermé). Un plafond plat
    /// ne dépend que de `bas`; les autres, du côté `cote` et de `haut`.
    pub fn nouveau(
        forme: FormeDePlafond,
        bas: f64,
        haut: f64,
        sommets: &[(f64, f64)],
        cote: usize,
    ) -> Option<PlafondMesure> {
        let n = sommets.len().checked_sub(1).filter(|n| *n >= 3)?;
        let plat = PlafondMesure {
            forme: FormeDePlafond::Plat,
            bas,
            haut: bas,
            normale: (0.0, 1.0),
            ligne: 0.0,
            portee: 1.0,
        };
        if forme == FormeDePlafond::Plat {
            return Some(plat);
        }
        let k = cote % n;
        let (a, b) = (sommets[k], sommets[k + 1]);
        let longueur = (b.0 - a.0).hypot(b.1 - a.1);
        if longueur <= 1e-9 {
            return None;
        }
        let normale = (-(b.1 - a.1) / longueur, (b.0 - a.0) / longueur);
        let v = |p: (f64, f64)| normale.0 * p.0 + normale.1 * p.1;
        let (bas_v, haut_v) = sommets[..n]
            .iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), p| {
                (lo.min(v(*p)), hi.max(v(*p)))
            });
        let (ligne, portee) = match forme {
            FormeDePlafond::Cathedrale => ((bas_v + haut_v) / 2.0, (haut_v - bas_v) / 2.0),
            _ => {
                let cote_haut = v(a);
                (
                    cote_haut,
                    (haut_v - cote_haut).abs().max((cote_haut - bas_v).abs()),
                )
            }
        };
        (portee > 1e-9).then_some(PlafondMesure {
            forme,
            bas,
            haut,
            normale,
            ligne,
            portee,
        })
    }

    fn v(&self, p: (f64, f64)) -> f64 {
        self.normale.0 * p.0 + self.normale.1 * p.1
    }

    /// La pente (pouces de hauteur par pouce en plan).
    fn pente(&self) -> f64 {
        if self.forme == FormeDePlafond::Plat {
            0.0
        } else {
            (self.haut - self.bas) / self.portee
        }
    }

    /// La hauteur du plafond au point `p` du plan : jamais sous la hauteur des
    /// murs d'une pente qui monte, même hors de la pièce.
    pub fn hauteur_au(&self, p: (f64, f64)) -> f64 {
        if self.forme == FormeDePlafond::Plat {
            return self.bas;
        }
        let part = ((self.v(p) - self.ligne).abs() / self.portee).min(1.0);
        self.haut - (self.haut - self.bas) * part
    }

    /// La plus grande hauteur du plafond.
    pub fn plus_haut(&self) -> f64 {
        self.bas.max(self.haut)
    }

    /// Où un mur de `a` à `b` croise le faîte ou le côté haut, en pouces
    /// depuis `a`, s'il le croise entre ses deux bouts.
    pub fn brisure(&self, a: (f64, f64), b: (f64, f64)) -> Option<f64> {
        if self.forme == FormeDePlafond::Plat {
            return None;
        }
        let (va, vb) = (self.v(a) - self.ligne, self.v(b) - self.ligne);
        let longueur = (b.0 - a.0).hypot(b.1 - a.1);
        (va * vb < 0.0).then(|| longueur * va / (va - vb))
    }

    /// La surface d'un mur de `a` à `b`, du plancher au plafond.
    pub fn surface_du_mur(&self, a: (f64, f64), b: (f64, f64)) -> f64 {
        let longueur = (b.0 - a.0).hypot(b.1 - a.1);
        if longueur <= 1e-12 {
            return 0.0;
        }
        let point = |t: f64| {
            (
                a.0 + (b.0 - a.0) * t / longueur,
                a.1 + (b.1 - a.1) * t / longueur,
            )
        };
        let mut bornes = vec![0.0, longueur];
        bornes.extend(self.brisure(a, b));
        bornes.sort_by(f64::total_cmp);
        bornes
            .windows(2)
            .map(|t| {
                (t[1] - t[0]) * (self.hauteur_au(point(t[0])) + self.hauteur_au(point(t[1]))) / 2.0
            })
            .sum()
    }

    /// La surface du plafond au-dessus du contour `sommets` : son aire en
    /// plan, allongée par la pente.
    pub fn surface(&self, sommets: &[(f64, f64)]) -> f64 {
        aire_et_moment(sommets, self.normale).0 * self.pente().hypot(1.0)
    }

    /// Le volume sous le plafond, au-dessus du contour `sommets` : le contour
    /// se coupe au faîte, et de chaque côté la hauteur varie en ligne droite,
    /// si bien que le volume s'y tire de l'aire et du premier moment.
    pub fn volume(&self, sommets: &[(f64, f64)]) -> f64 {
        if self.forme == FormeDePlafond::Plat {
            return self.bas * aire_et_moment(sommets, self.normale).0;
        }
        let pente = self.pente();
        [(true, -1.0), (false, 1.0)]
            .into_iter()
            .map(|(au_dela, signe)| {
                let morceau = couper(sommets, self.normale, self.ligne, au_dela);
                let (aire, moment) = aire_et_moment(&morceau, self.normale);
                // Au-delà du faîte, h = haut − pente·(v − ligne); en deçà,
                // h = haut + pente·(v − ligne).
                self.haut * aire + signe * pente * (moment - self.ligne * aire)
            })
            .sum()
    }
}

/// Le morceau d'un contour fermé où `normale · p ≥ seuil` (`au_dela`) ou
/// `≤ seuil`, fermé lui aussi; vide s'il n'en reste rien. Un contour à coins
/// rentrants peut donner des côtés qui se replient le long de la coupe : ils
/// ne changent ni l'aire ni le moment.
fn couper(
    sommets: &[(f64, f64)],
    normale: (f64, f64),
    seuil: f64,
    au_dela: bool,
) -> Vec<(f64, f64)> {
    let signe = if au_dela { 1.0 } else { -1.0 };
    let cote = |p: (f64, f64)| signe * (normale.0 * p.0 + normale.1 * p.1 - seuil);
    let mut morceau = Vec::new();
    for s in sommets.windows(2) {
        let (p, q) = (s[0], s[1]);
        let (cp, cq) = (cote(p), cote(q));
        if cp >= 0.0 {
            morceau.push(p);
        }
        if (cp >= 0.0) != (cq >= 0.0) {
            let t = cp / (cp - cq);
            morceau.push((p.0 + (q.0 - p.0) * t, p.1 + (q.1 - p.1) * t));
        }
    }
    if let Some(&depart) = morceau.first() {
        morceau.push(depart);
    }
    morceau
}

/// L'aire (positive) d'un contour fermé, et le premier moment de sa surface
/// le long de `normale` : l'intégrale de `normale · p` sur la surface.
fn aire_et_moment(sommets: &[(f64, f64)], normale: (f64, f64)) -> (f64, f64) {
    let (mut double_aire, mut mx, mut my) = (0.0, 0.0, 0.0);
    for s in sommets.windows(2) {
        let (p, q) = (s[0], s[1]);
        let croix = p.0 * q.1 - q.0 * p.1;
        double_aire += croix;
        mx += (p.0 + q.0) * croix;
        my += (p.1 + q.1) * croix;
    }
    let signe = if double_aire < 0.0 { -1.0 } else { 1.0 };
    (
        double_aire.abs() / 2.0,
        signe * (normale.0 * mx + normale.1 * my) / 6.0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Une pièce de 12 × 10 pi, en pouces : le premier côté vers l'est.
    fn rectangle() -> Vec<(f64, f64)> {
        vec![
            (0.0, 0.0),
            (144.0, 0.0),
            (144.0, 120.0),
            (0.0, 120.0),
            (0.0, 0.0),
        ]
    }

    fn proche(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6 * b.abs().max(1.0)
    }

    #[test]
    fn un_plafond_plat() {
        let s = rectangle();
        let plafond = PlafondMesure::nouveau(FormeDePlafond::Plat, 96.0, 0.0, &s, 0).unwrap();
        assert!(proche(plafond.volume(&s), 144.0 * 120.0 * 96.0));
        assert!(proche(plafond.surface(&s), 144.0 * 120.0));
        assert!(proche(plafond.surface_du_mur(s[0], s[1]), 144.0 * 96.0));
        assert_eq!(plafond.brisure(s[1], s[2]), None);
        assert_eq!(plafond.hauteur_au((50.0, 50.0)), 96.0);
    }

    #[test]
    fn un_plafond_cathedrale() {
        // Murs de 8 pi, faîte à 12 pi, parallèle au côté de 12 pi.
        let s = rectangle();
        let plafond =
            PlafondMesure::nouveau(FormeDePlafond::Cathedrale, 96.0, 144.0, &s, 0).unwrap();
        // 120 pi² × 8 pi + le prisme : 4 pi × 60 pi² → 1 200 pi³.
        assert!(proche(plafond.volume(&s), 1200.0 * 1728.0));
        // Deux pentes de 5 pi sur 4 : 120 pi² × √(1 + 0,64).
        assert!(proche(plafond.surface(&s), 120.0 * 144.0 * 1.64f64.sqrt()));
        // Deux murs de 12 × 8 pi, deux pignons de 10 × 8 pi plus un triangle
        // de 10 × 4 pi.
        let murs: Vec<f64> = s
            .windows(2)
            .map(|m| plafond.surface_du_mur(m[0], m[1]))
            .collect();
        assert!(proche(murs[0], 96.0 * 144.0));
        assert!(proche(murs[1], (80.0 + 20.0) * 144.0));
        assert!(proche(murs[2], 96.0 * 144.0));
        assert!(proche(murs[3], (80.0 + 20.0) * 144.0));
        assert!(proche(plafond.brisure(s[1], s[2]).unwrap(), 60.0));
        assert!(proche(plafond.hauteur_au((72.0, 60.0)), 144.0));
        assert!(proche(plafond.hauteur_au((72.0, 0.0)), 96.0));
        assert!(proche(plafond.hauteur_au((72.0, -50.0)), 96.0));
        assert_eq!(plafond.plus_haut(), 144.0);
    }

    #[test]
    fn un_plafond_en_pente() {
        // Le côté haut à 10 pi (le premier), le côté opposé à 8 pi.
        let s = rectangle();
        let plafond = PlafondMesure::nouveau(FormeDePlafond::EnPente, 96.0, 120.0, &s, 0).unwrap();
        assert!(proche(plafond.volume(&s), 120.0 * 9.0 * 1728.0));
        assert!(proche(plafond.surface(&s), 120.0 * 144.0 * 1.04f64.sqrt()));
        let murs: Vec<f64> = s
            .windows(2)
            .map(|m| plafond.surface_du_mur(m[0], m[1]))
            .collect();
        assert!(proche(murs[0], 120.0 * 144.0));
        assert!(proche(murs[1], 90.0 * 144.0));
        assert!(proche(murs[2], 96.0 * 144.0));
        assert!(proche(murs[3], 90.0 * 144.0));
        // Le côté haut peut être un autre côté : le deuxième, à droite.
        let autre = PlafondMesure::nouveau(FormeDePlafond::EnPente, 96.0, 120.0, &s, 1).unwrap();
        assert!(proche(autre.hauteur_au((144.0, 30.0)), 120.0));
        assert!(proche(autre.hauteur_au((0.0, 30.0)), 96.0));
        assert!(proche(autre.volume(&s), 120.0 * 9.0 * 1728.0));
    }

    #[test]
    fn le_volume_d_une_piece_en_l_sous_une_cathedrale() {
        // Une pièce en L : le volume exact s'accorde avec une somme en grille.
        let s = vec![
            (0.0, 0.0),
            (240.0, 0.0),
            (240.0, 96.0),
            (108.0, 96.0),
            (108.0, 216.0),
            (0.0, 216.0),
            (0.0, 0.0),
        ];
        let plafond =
            PlafondMesure::nouveau(FormeDePlafond::Cathedrale, 96.0, 150.0, &s, 3).unwrap();
        let dedans = |x: f64, y: f64| (y < 96.0 && x < 240.0) || (x < 108.0 && y < 216.0);
        let pas = 0.5;
        let mut somme = 0.0;
        let mut x = pas / 2.0;
        while x < 240.0 {
            let mut y = pas / 2.0;
            while y < 216.0 {
                if dedans(x, y) {
                    somme += plafond.hauteur_au((x, y)) * pas * pas;
                }
                y += pas;
            }
            x += pas;
        }
        let volume = plafond.volume(&s);
        assert!((volume - somme).abs() / volume < 1e-4, "{volume} {somme}");
        // Le contour parcouru dans l'autre sens : le même volume.
        let mut inverse = s.clone();
        inverse.reverse();
        assert!(proche(plafond.volume(&inverse), volume));
    }
}
