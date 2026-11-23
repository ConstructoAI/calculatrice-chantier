//! Les cloisons d'une pièce (section 25) : des murs intérieurs tracés sur le
//! plan, d'un seul trait ou en polyligne — des segments d'équerre ou à 45°,
//! aux coins d'onglet —, d'une épaisseur donnée : la leur, sinon celle que la
//! pièce donne à ses cloisons (5 po par défaut).

use serde::{Deserialize, Serialize};

use crate::dessin::{Axe, Sens};

/// L'épaisseur des cloisons d'une pièce qui n'en donne pas, en pouces.
pub const EPAISSEUR_PAR_DEFAUT: f64 = 5.0;
/// Un segment du plan, d'un bout à l'autre (pouces) : un mur, ou l'axe d'une
/// cloison.
pub type Segment = ((f64, f64), (f64, f64));

/// Une cloison redressée : son départ (pouces) et ses segments (direction,
/// longueur en pouces).
pub type Polyligne = ((f64, f64), Vec<(Sens, f64)>);

/// Longueur sous laquelle un trait ne fait pas une cloison, en pouces.
pub const LONGUEUR_MINIMALE: f64 = 6.0;
/// La largeur d'une porte de cloison sans largeur inscrite : 2 pi 8 po.
pub const LARGEUR_DE_PORTE_PAR_DEFAUT: f64 = 32.0;
/// La hauteur d'une porte de cloison : 6 pi 8 po.
pub const HAUTEUR_DE_PORTE: f64 = 80.0;

/// La porte d'une cloison : son segment, sa largeur et sa distance au début
/// de ce segment (le bout A), telles qu'inscrites — vides : 2 pi 8 po,
/// centrée —, et le côté où elle s'ouvre.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PorteDeCloison {
    pub largeur: String,
    pub position: String,
    /// Vrai si elle s'ouvre de l'autre côté de la cloison.
    pub inversee: bool,
    /// Le segment de la cloison qui la porte (le premier : 0).
    pub segment: usize,
}

/// La porte d'une cloison à ses mesures : son segment, ses bords en pouces
/// depuis le début de ce segment, et le côté où elle s'ouvre.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PorteMesuree {
    pub segment: usize,
    pub debut: f64,
    pub fin: f64,
    pub inversee: bool,
}

/// Un segment de plus d'une cloison en polyligne : sa direction et sa
/// longueur inscrite.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SegmentDeCloison {
    pub sens: Sens,
    pub longueur: String,
}

/// Une cloison : son départ (pouces, dans le repère du plan de sa pièce), son
/// premier segment (sa direction et sa longueur inscrite), les segments qui
/// le suivent, son épaisseur inscrite et sa porte.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Cloison {
    pub debut: (f64, f64),
    pub sens: Sens,
    pub longueur: String,
    /// Vide : celle des cloisons de la pièce.
    pub epaisseur: String,
    /// Sa porte, s'il y en a une.
    pub porte: Option<PorteDeCloison>,
    /// Les segments qui suivent le premier, d'une cloison en polyligne.
    pub suite: Vec<SegmentDeCloison>,
}

impl Default for Cloison {
    fn default() -> Self {
        Cloison {
            debut: (0.0, 0.0),
            sens: Sens::Est,
            longueur: String::new(),
            epaisseur: String::new(),
            porte: None,
            suite: Vec::new(),
        }
    }
}

impl Cloison {
    /// Le nombre de ses segments.
    pub fn nombre_de_segments(&self) -> usize {
        1 + self.suite.len()
    }

    /// Ses segments : leur direction et leur longueur inscrite.
    pub fn segments(&self) -> impl Iterator<Item = (Sens, &str)> {
        std::iter::once((self.sens, self.longueur.as_str())).chain(
            self.suite
                .iter()
                .map(|segment| (segment.sens, segment.longueur.as_str())),
        )
    }

    /// La longueur inscrite d'un de ses segments.
    pub fn longueur_du_segment(&self, segment: usize) -> Option<&str> {
        if segment == 0 {
            Some(&self.longueur)
        } else {
            self.suite.get(segment - 1).map(|s| s.longueur.as_str())
        }
    }

    /// La longueur inscrite d'un de ses segments, à modifier.
    pub fn longueur_du_segment_mut(&mut self, segment: usize) -> Option<&mut String> {
        if segment == 0 {
            Some(&mut self.longueur)
        } else {
            self.suite.get_mut(segment - 1).map(|s| &mut s.longueur)
        }
    }

    /// La direction d'un de ses segments.
    pub fn direction_du_segment(&self, segment: usize) -> Option<Sens> {
        if segment == 0 {
            Some(self.sens)
        } else {
            self.suite.get(segment - 1).map(|s| s.sens)
        }
    }

    /// L'angle, en degrés, dont le segment `segment` tourne par rapport au
    /// précédent, à gauche ou à droite (section 25.1, point 32); `None` pour le premier.
    pub fn virage(&self, segment: usize) -> Option<f64> {
        let avant = self.direction_du_segment(segment.checked_sub(1)?)?;
        Some(
            self.direction_du_segment(segment)?
                .virage_depuis(avant)
                .abs(),
        )
    }

    /// Tourne le segment `segment` pour qu'il fasse l'angle `virage` (en
    /// degrés, plus de 0 et moins de 180) avec le précédent, du même côté
    /// qu'il tournait. Les segments suivants gardent leur direction. Faux si
    /// rien n'a changé ou si c'est impossible (le premier segment n'a pas de
    /// précédent).
    pub fn orienter(&mut self, segment: usize, virage: f64) -> bool {
        if !(virage > 0.0 && virage < 180.0) {
            return false;
        }
        let Some(avant) = segment
            .checked_sub(1)
            .and_then(|k| self.direction_du_segment(k))
        else {
            return false;
        };
        let Some(actuel) = self.direction_du_segment(segment) else {
            return false;
        };
        let ecart = actuel.virage_depuis(avant);
        let signe = if ecart < -1e-9 && ecart > -180.0 + 1e-9 {
            -1.0
        } else {
            1.0
        };
        let sens = Sens::d_angle(avant.angle() + signe * virage);
        if sens == actuel {
            return false;
        }
        self.suite[segment - 1].sens = sens;
        true
    }
}

/// Un morceau plein d'une cloison : ses quatre coins — un côté de son début
/// à sa fin, puis l'autre de sa fin à son début — et ses bouts libres (le
/// bout de la cloison, ou le bord de sa porte), qui se tracent; le coin
/// d'onglet entre deux segments ne se trace pas.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MorceauDeCloison {
    pub coins: [(f64, f64); 4],
    pub bout_au_debut: bool,
    pub bout_a_la_fin: bool,
}

/// Une cloison mise à ses mesures, en pouces.
#[derive(Clone, Debug, PartialEq)]
pub struct CloisonMesuree {
    /// Les sommets de son axe : son départ, puis la fin de chaque segment.
    pub sommets: Vec<(f64, f64)>,
    pub epaisseur: f64,
    pub porte: Option<PorteMesuree>,
}

impl CloisonMesuree {
    /// Une cloison à ses mesures, de son départ et de ses segments (direction,
    /// longueur en pouces).
    pub fn depuis(
        debut: (f64, f64),
        segments: &[(Sens, f64)],
        epaisseur: f64,
        porte: Option<PorteMesuree>,
    ) -> CloisonMesuree {
        let mut sommets = vec![debut];
        let mut point = debut;
        for &(sens, longueur) in segments {
            let (ux, uy) = sens.pas();
            point = (point.0 + ux * longueur, point.1 + uy * longueur);
            sommets.push(point);
        }
        CloisonMesuree {
            sommets,
            epaisseur,
            porte,
        }
    }

    /// Son départ.
    pub fn debut(&self) -> (f64, f64) {
        self.sommets[0]
    }

    /// Le nombre de ses segments.
    pub fn nombre_de_segments(&self) -> usize {
        self.sommets.len().saturating_sub(1)
    }

    /// La longueur d'un de ses segments.
    pub fn longueur_du_segment(&self, segment: usize) -> f64 {
        let (a, b) = (self.sommets[segment], self.sommets[segment + 1]);
        (b.0 - a.0).hypot(b.1 - a.1)
    }

    /// Sa longueur, tous segments mis bout à bout.
    pub fn longueur(&self) -> f64 {
        (0..self.nombre_de_segments())
            .map(|k| self.longueur_du_segment(k))
            .sum()
    }

    /// La direction d'un de ses segments (unitaire).
    fn sens_du_segment(&self, segment: usize) -> (f64, f64) {
        let (a, b) = (self.sommets[segment], self.sommets[segment + 1]);
        let longueur = (b.0 - a.0).hypot(b.1 - a.1).max(1e-12);
        ((b.0 - a.0) / longueur, (b.1 - a.1) / longueur)
    }

    /// Le point d'un segment à `t` pouces de son début.
    pub fn point(&self, segment: usize, t: f64) -> (f64, f64) {
        let (ux, uy) = self.sens_du_segment(segment);
        let a = self.sommets[segment];
        (a.0 + ux * t, a.1 + uy * t)
    }

    /// Les axes de ses segments.
    pub fn axes(&self) -> Vec<Segment> {
        self.sommets.windows(2).map(|s| (s[0], s[1])).collect()
    }

    /// Le rectangle d'un segment de `t0` à `t1` pouces de son début, aux bouts
    /// carrés.
    pub fn rectangle(&self, segment: usize, t0: f64, t1: f64) -> [(f64, f64); 4] {
        let (ux, uy) = self.sens_du_segment(segment);
        let (nx, ny) = (-uy * self.epaisseur / 2.0, ux * self.epaisseur / 2.0);
        let (a, b) = (self.point(segment, t0), self.point(segment, t1));
        [
            (a.0 + nx, a.1 + ny),
            (b.0 + nx, b.1 + ny),
            (b.0 - nx, b.1 - ny),
            (a.0 - nx, a.1 - ny),
        ]
    }

    /// Le coin d'onglet au sommet `sommet` (entre deux segments), d'un côté de
    /// l'axe (`cote` : 1 ou −1) : la rencontre des faces des deux segments.
    fn coin_d_onglet(&self, sommet: usize, cote: f64) -> (f64, f64) {
        let (u1, u2) = (
            self.sens_du_segment(sommet - 1),
            self.sens_du_segment(sommet),
        );
        let h = cote * self.epaisseur / 2.0;
        let s = self.sommets[sommet];
        let p1 = (s.0 - u1.1 * h, s.1 + u1.0 * h);
        let p2 = (s.0 - u2.1 * h, s.1 + u2.0 * h);
        let croix = u1.0 * u2.1 - u1.1 * u2.0;
        if croix.abs() < 1e-9 {
            return p2;
        }
        let t = ((p2.0 - p1.0) * u2.1 - (p2.1 - p1.1) * u2.0) / croix;
        (p1.0 + u1.0 * t, p1.1 + u1.1 * t)
    }

    /// Ses morceaux pleins : un par segment, aux coins d'onglet, et deux pour
    /// le segment de sa porte, de part et d'autre d'elle.
    pub fn morceaux(&self) -> Vec<MorceauDeCloison> {
        let n = self.nombre_de_segments();
        let h = self.epaisseur / 2.0;
        let mut morceaux = Vec::new();
        for k in 0..n {
            let longueur = self.longueur_du_segment(k);
            let (ux, uy) = self.sens_du_segment(k);
            let carre = |t: f64, cote: f64| {
                let p = self.point(k, t);
                (p.0 - uy * h * cote, p.1 + ux * h * cote)
            };
            let au_debut = |cote: f64| {
                if k == 0 {
                    carre(0.0, cote)
                } else {
                    self.coin_d_onglet(k, cote)
                }
            };
            let a_la_fin = |cote: f64| {
                if k + 1 == n {
                    carre(longueur, cote)
                } else {
                    self.coin_d_onglet(k + 1, cote)
                }
            };
            // Les parties pleines du segment, de part et d'autre de la porte.
            let pleines = match self.porte {
                Some(porte) if porte.segment == k => {
                    vec![(0.0, porte.debut), (porte.fin, longueur)]
                }
                _ => vec![(0.0, longueur)],
            };
            for (t0, t1) in pleines {
                if t1 - t0 <= 1e-6 {
                    continue;
                }
                let (g0, d0, bout_au_debut) = if t0 <= 1e-9 {
                    (au_debut(1.0), au_debut(-1.0), k == 0)
                } else {
                    (carre(t0, 1.0), carre(t0, -1.0), true)
                };
                let (g1, d1, bout_a_la_fin) = if t1 >= longueur - 1e-9 {
                    (a_la_fin(1.0), a_la_fin(-1.0), k + 1 == n)
                } else {
                    (carre(t1, 1.0), carre(t1, -1.0), true)
                };
                morceaux.push(MorceauDeCloison {
                    coins: [g0, g1, d1, d0],
                    bout_au_debut,
                    bout_a_la_fin,
                });
            }
        }
        morceaux
    }

    /// La distance d'un point (pouces) à l'axe de la cloison.
    pub fn distance(&self, (x, y): (f64, f64)) -> f64 {
        self.axes()
            .into_iter()
            .map(|(a, b)| {
                let (dx, dy) = (b.0 - a.0, b.1 - a.1);
                let carre = dx * dx + dy * dy;
                let t = if carre <= 1e-12 {
                    0.0
                } else {
                    (((x - a.0) * dx + (y - a.1) * dy) / carre).clamp(0.0, 1.0)
                };
                (x - a.0 - t * dx).hypot(y - a.1 - t * dy)
            })
            .fold(f64::MAX, f64::min)
    }

    /// Le côté où s'ouvre sa porte : à gauche de l'axe de son segment, ou de
    /// l'autre côté si elle est inversée (une direction unitaire).
    pub fn cote_de_la_porte(&self) -> (f64, f64) {
        let segment = self.porte.map_or(0, |porte| porte.segment);
        let (ux, uy) =
            self.sens_du_segment(segment.min(self.nombre_de_segments().saturating_sub(1)));
        if self.porte.is_some_and(|porte| porte.inversee) {
            (uy, -ux)
        } else {
            (-uy, ux)
        }
    }
}

/// Où la droite `depart + t × u` croise le segment `ab` : la valeur de t, ou
/// `None` si elle lui est parallèle ou passe à côté.
fn croisement(depart: (f64, f64), u: (f64, f64), (a, b): Segment) -> Option<f64> {
    let w = (b.0 - a.0, b.1 - a.1);
    let denominateur = u.0 * w.1 - u.1 * w.0;
    if denominateur.abs() < 1e-9 {
        return None;
    }
    let (px, py) = (a.0 - depart.0, a.1 - depart.1);
    let t = (px * w.1 - py * w.0) / denominateur;
    let s = (px * u.1 - py * u.0) / denominateur;
    (-1e-9..=1.0 + 1e-9).contains(&s).then_some(t)
}

/// Le croisement avec un appui le plus proche de `cible` sur la droite
/// `depart + t × u`, à moins de `aimant` pouces et permis par `permis`.
fn appui_proche(
    depart: (f64, f64),
    u: (f64, f64),
    appuis: &[Segment],
    cible: f64,
    aimant: f64,
    permis: &dyn Fn(f64) -> bool,
) -> Option<f64> {
    appuis
        .iter()
        .filter_map(|appui| croisement(depart, u, *appui))
        .filter(|t| permis(*t) && (t - cible).abs() <= aimant)
        .min_by(|a, b| (a - cible).abs().total_cmp(&(b - cible).abs()))
}

/// Redresse un trait du doigt (pouces) en cloison : sa direction devient la
/// plus proche des huit (d'équerre ou à 45°), son départ et sa longueur
/// s'arrondissent au `pas` (le pouce, ou le centimètre en métrique), et chacun
/// de ses bouts va jusqu'au mur ou à la cloison la plus proche (`appuis`) s'il
/// en est à moins de `aimant` pouces. Donne sa direction, son départ et sa
/// longueur; `None` pour un trait trop court.
pub fn redresser(
    depart: (f64, f64),
    arrivee: (f64, f64),
    appuis: &[Segment],
    aimant: f64,
    pas: f64,
) -> Option<(Sens, (f64, f64), f64)> {
    let (dx, dy) = (arrivee.0 - depart.0, arrivee.1 - depart.1);
    let axe = Axe::du_deplacement(dx, dy);
    let v = axe.vecteur();
    let signee = dx * v.0 + dy * v.1;
    let sens = Sens::selon(axe, signee);
    let u = sens.pas();
    let arrondir = |x: f64| (x / pas).round() * pas;
    let debut = (arrondir(depart.0), arrondir(depart.1));
    let longueur = arrondir(signee.abs());
    // Les bouts vont aux appuis tout proches, sans se rejoindre.
    let t_debut = appui_proche(debut, u, appuis, 0.0, aimant, &|t| {
        t <= longueur - LONGUEUR_MINIMALE
    })
    .unwrap_or(0.0);
    let t_fin = appui_proche(debut, u, appuis, longueur, aimant, &|t| {
        t >= t_debut + LONGUEUR_MINIMALE
    })
    .unwrap_or(longueur);
    let longueur = t_fin - t_debut;
    (longueur >= LONGUEUR_MINIMALE).then_some((
        sens,
        (debut.0 + u.0 * t_debut, debut.1 + u.1 * t_debut),
        longueur,
    ))
}

/// Redresse un tracé du doigt (pouces), qui peut tourner des coins, en
/// cloison en polyligne : des segments d'équerre ou à 45°, comme le contour
/// d'une pièce, leurs longueurs arrondies au `pas`; son premier bout va au mur
/// ou à la cloison tout proche le long de son premier segment, et son dernier
/// bout de même le long du dernier. Donne son départ et ses segments
/// (direction, longueur); `None` pour un tracé trop court.
pub fn redresser_le_trace(
    points: &[(f64, f64)],
    appuis: &[Segment],
    aimant: f64,
    pas: f64,
) -> Option<Polyligne> {
    let premier = *points.first()?;
    let arrondir = |x: f64| (x / pas).round() * pas;
    let mut segments: Vec<(Sens, f64)> = crate::dessin::redresser(points)
        .into_iter()
        .map(|(axe, longueur)| (Sens::selon(axe, longueur), arrondir(longueur.abs())))
        .filter(|(_, longueur)| *longueur >= pas.max(1.0))
        .collect();
    if segments.is_empty() {
        return None;
    }
    let mut debut = (arrondir(premier.0), arrondir(premier.1));
    // Le premier bout, le long du premier segment.
    let u = segments[0].0.pas();
    let longueur = segments[0].1;
    if let Some(t) = appui_proche(debut, u, appuis, 0.0, aimant, &|t| t <= longueur - 1.0) {
        debut = (debut.0 + u.0 * t, debut.1 + u.1 * t);
        segments[0].1 -= t;
    }
    // Le dernier bout, le long du dernier segment.
    let fin = CloisonMesuree::depuis(debut, &segments, 0.0, None)
        .sommets
        .last()
        .copied()
        .unwrap_or(debut);
    let dernier = segments.len() - 1;
    let v = segments[dernier].0.pas();
    let longueur = segments[dernier].1;
    if let Some(t) = appui_proche(fin, v, appuis, 0.0, aimant, &|t| t >= 1.0 - longueur) {
        segments[dernier].1 += t;
    }
    let total: f64 = segments.iter().map(|(_, longueur)| longueur).sum();
    (total >= LONGUEUR_MINIMALE && segments.iter().all(|(_, longueur)| *longueur >= 1.0))
        .then_some((debut, segments))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Les murs d'une pièce de 12 pi × 10 pi, le départ en haut à gauche.
    fn murs() -> Vec<Segment> {
        let s = [
            (0.0, 0.0),
            (144.0, 0.0),
            (144.0, 120.0),
            (0.0, 120.0),
            (0.0, 0.0),
        ];
        s.windows(2).map(|w| (w[0], w[1])).collect()
    }

    #[test]
    fn un_trait_devient_une_cloison_d_un_mur_a_l_autre() {
        // Un trait hésitant, presque horizontal, qui part près du mur de
        // gauche et s'arrête près de celui de droite.
        let (sens, debut, longueur) =
            redresser((3.4, 50.3), (139.0, 54.0), &murs(), 8.0, 1.0).unwrap();
        assert_eq!(sens, Sens::Est);
        assert_eq!(debut, (0.0, 50.0));
        assert!((longueur - 144.0).abs() < 1e-9);
        // Une seconde cloison, verticale, part de la première et va au mur du bas.
        let mut appuis = murs();
        appuis.push(((0.0, 50.0), (144.0, 50.0)));
        let (sens, debut, longueur) =
            redresser((60.2, 55.0), (61.0, 116.0), &appuis, 8.0, 1.0).unwrap();
        assert_eq!(sens, Sens::Sud);
        assert_eq!(debut, (60.0, 50.0));
        assert!((longueur - 70.0).abs() < 1e-9);
        // Loin des murs, la longueur s'arrondit au pouce.
        let (_, debut, longueur) =
            redresser((30.0, 80.0), (30.4, 100.6), &murs(), 8.0, 1.0).unwrap();
        assert_eq!(debut, (30.0, 80.0));
        assert!((longueur - 21.0).abs() < 1e-9);
        // Un trait trop court ne fait pas de cloison.
        assert!(redresser((30.0, 30.0), (32.0, 31.0), &murs(), 8.0, 1.0).is_none());
    }

    #[test]
    fn un_trace_qui_tourne_un_coin_devient_une_polyligne() {
        // Un tracé en L : du mur de gauche vers la droite, puis vers le bas,
        // jusqu'au mur du bas.
        let mut points: Vec<(f64, f64)> =
            (0..=19).map(|k| (3.0 + 3.0 * f64::from(k), 50.4)).collect();
        points.extend((1..=22).map(|k| (60.2, 50.4 + 3.05 * f64::from(k))));
        let (debut, segments) = redresser_le_trace(&points, &murs(), 8.0, 1.0).unwrap();
        assert_eq!(debut, (0.0, 50.0));
        assert_eq!(segments, vec![(Sens::Est, 60.0), (Sens::Sud, 70.0)]);
        // Un trait droit donne un seul segment.
        let droit = [(30.0, 80.0), (30.2, 90.0), (30.4, 100.6)];
        let (debut, segments) = redresser_le_trace(&droit, &murs(), 8.0, 1.0).unwrap();
        assert_eq!(debut, (30.0, 80.0));
        assert_eq!(segments, vec![(Sens::Sud, 21.0)]);
    }

    #[test]
    fn les_coins_d_onglet_d_une_cloison_en_l() {
        let cloison = CloisonMesuree::depuis(
            (0.0, 50.0),
            &[(Sens::Est, 60.0), (Sens::Sud, 70.0)],
            5.0,
            None,
        );
        assert!((cloison.longueur() - 130.0).abs() < 1e-9);
        let morceaux = cloison.morceaux();
        assert_eq!(morceaux.len(), 2);
        // Le coin : les faces des deux segments se rencontrent en onglet.
        assert_eq!(morceaux[0].coins[1], (57.5, 52.5));
        assert_eq!(morceaux[0].coins[2], (62.5, 47.5));
        assert_eq!(morceaux[1].coins[0], (57.5, 52.5));
        assert_eq!(morceaux[1].coins[3], (62.5, 47.5));
        // Seuls les bouts de la cloison se tracent.
        assert!(morceaux[0].bout_au_debut && !morceaux[0].bout_a_la_fin);
        assert!(!morceaux[1].bout_au_debut && morceaux[1].bout_a_la_fin);
        assert!((cloison.distance((60.0, 90.0)) - 0.0).abs() < 1e-9);
        assert!((cloison.distance((70.0, 90.0)) - 10.0).abs() < 1e-9);
    }

    #[test]
    fn les_morceaux_et_la_porte_d_une_cloison() {
        let mut cloison = CloisonMesuree::depuis(
            (48.0, 0.0),
            &[(Sens::Sud, 120.0)],
            5.0,
            Some(PorteMesuree {
                segment: 0,
                debut: 44.0,
                fin: 76.0,
                inversee: false,
            }),
        );
        // Deux morceaux pleins, de part et d'autre de la porte.
        let morceaux = cloison.morceaux();
        assert_eq!(morceaux.len(), 2);
        assert!(morceaux[0].coins.iter().all(|c| c.1 <= 44.0 + 1e-9));
        assert!(morceaux[1].coins.iter().all(|c| c.1 >= 76.0 - 1e-9));
        assert!(morceaux[0].bout_a_la_fin && morceaux[1].bout_au_debut);
        assert_eq!(cloison.point(0, 44.0), (48.0, 44.0));
        // Elle s'ouvre à gauche de son axe, ou de l'autre côté une fois inversée.
        assert_eq!(cloison.cote_de_la_porte(), (-1.0, 0.0));
        cloison.porte = Some(PorteMesuree {
            segment: 0,
            debut: 44.0,
            fin: 76.0,
            inversee: true,
        });
        assert_eq!(cloison.cote_de_la_porte(), (1.0, 0.0));
        // Une porte contre le bout A ne laisse qu'un morceau.
        cloison.porte = Some(PorteMesuree {
            segment: 0,
            debut: 0.0,
            fin: 32.0,
            inversee: false,
        });
        assert_eq!(cloison.morceaux().len(), 1);
    }

    #[test]
    fn les_coins_et_la_distance_d_une_cloison() {
        let cloison = CloisonMesuree::depuis((48.0, 0.0), &[(Sens::Sud, 120.0)], 4.5, None);
        let coins = cloison.morceaux()[0].coins;
        assert!(
            coins
                .iter()
                .all(|c| (c.0 - 45.75).abs() < 1e-9 || (c.0 - 50.25).abs() < 1e-9)
        );
        assert!((cloison.distance((50.0, 60.0)) - 2.0).abs() < 1e-9);
        assert!((cloison.distance((48.0, 130.0)) - 10.0).abs() < 1e-9);
    }

    #[test]
    fn l_angle_d_un_segment_de_cloison() {
        // Une cloison en L : vers l'est, puis vers le sud.
        let mut cloison = Cloison {
            suite: vec![SegmentDeCloison {
                sens: Sens::Sud,
                longueur: "5'".to_string(),
            }],
            longueur: "4'".to_string(),
            ..Cloison::default()
        };
        assert_eq!(cloison.virage(0), None);
        assert_eq!(cloison.virage(1), Some(90.0));
        assert!(!cloison.orienter(0, 30.0));
        // Le deuxième segment tourne de 30° au lieu de 90°, toujours vers le sud.
        assert!(cloison.orienter(1, 30.0));
        assert_eq!(cloison.suite[0].sens, Sens::Libre(30_000));
        assert!((cloison.virage(1).unwrap() - 30.0).abs() < 1e-9);
        assert!(cloison.orienter(1, 45.0));
        assert_eq!(cloison.suite[0].sens, Sens::SudEst);
        assert!(!cloison.orienter(1, 45.0));
        // À ses mesures, la cloison suit sa direction libre.
        assert!(cloison.orienter(1, 60.0));
        let mesuree = CloisonMesuree::depuis(
            (0.0, 0.0),
            &[(Sens::Est, 48.0), (cloison.suite[0].sens, 60.0)],
            0.0,
            None,
        );
        let fin = mesuree.point(1, 60.0);
        assert!((fin.0 - 78.0).abs() < 1e-9, "{fin:?}");
        assert!((fin.1 - 60.0 * 60f64.to_radians().sin()).abs() < 1e-9);
    }
}
