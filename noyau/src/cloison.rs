//! Les cloisons d'une pièce (section 25) : des murs intérieurs tracés sur le
//! plan, d'équerre ou à 45°, d'une épaisseur donnée — la leur, sinon celle
//! que la pièce donne à ses cloisons (5 po par défaut).

use serde::{Deserialize, Serialize};

use crate::dessin::{Axe, Sens};

/// L'épaisseur des cloisons d'une pièce qui n'en donne pas, en pouces.
pub const EPAISSEUR_PAR_DEFAUT: f64 = 5.0;
/// Un segment du plan, d'un bout à l'autre (pouces) : un mur, ou l'axe d'une
/// cloison.
pub type Segment = ((f64, f64), (f64, f64));

/// Longueur sous laquelle un trait ne fait pas une cloison, en pouces.
pub const LONGUEUR_MINIMALE: f64 = 6.0;
/// La largeur d'une porte de cloison sans largeur inscrite : 2 pi 8 po.
pub const LARGEUR_DE_PORTE_PAR_DEFAUT: f64 = 32.0;
/// La hauteur d'une porte de cloison : 6 pi 8 po.
pub const HAUTEUR_DE_PORTE: f64 = 80.0;

/// La porte d'une cloison : sa largeur et sa distance au bout A (le départ de
/// la cloison), telles qu'inscrites — vides : 2 pi 8 po, centrée —, et le
/// côté où elle s'ouvre.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PorteDeCloison {
    pub largeur: String,
    pub position: String,
    /// Vrai si elle s'ouvre de l'autre côté de la cloison.
    pub inversee: bool,
}

/// La porte d'une cloison à ses mesures : ses bords, en pouces depuis le
/// bout A, et le côté où elle s'ouvre.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PorteMesuree {
    pub debut: f64,
    pub fin: f64,
    pub inversee: bool,
}

/// Une cloison : son départ (pouces, dans le repère du plan de sa pièce), sa
/// direction, et sa longueur et son épaisseur telles qu'inscrites.
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
}

impl Default for Cloison {
    fn default() -> Self {
        Cloison {
            debut: (0.0, 0.0),
            sens: Sens::Est,
            longueur: String::new(),
            epaisseur: String::new(),
            porte: None,
        }
    }
}

/// Une cloison mise à ses mesures, en pouces.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CloisonMesuree {
    /// Les deux bouts de son axe.
    pub debut: (f64, f64),
    pub fin: (f64, f64),
    pub longueur: f64,
    pub epaisseur: f64,
    pub porte: Option<PorteMesuree>,
}

impl CloisonMesuree {
    /// La direction de son axe, de son départ à sa fin.
    fn sens(&self) -> (f64, f64) {
        let longueur = self.longueur.max(1e-12);
        (
            (self.fin.0 - self.debut.0) / longueur,
            (self.fin.1 - self.debut.1) / longueur,
        )
    }

    /// Le point de son axe à `t` pouces de son départ (le bout A).
    pub fn point(&self, t: f64) -> (f64, f64) {
        let u = self.sens();
        (self.debut.0 + u.0 * t, self.debut.1 + u.1 * t)
    }

    /// Le rectangle de la cloison de `t0` à `t1` pouces de son départ.
    pub fn rectangle(&self, t0: f64, t1: f64) -> [(f64, f64); 4] {
        let (ux, uy) = self.sens();
        let (nx, ny) = (-uy * self.epaisseur / 2.0, ux * self.epaisseur / 2.0);
        let (a, b) = (self.point(t0), self.point(t1));
        [
            (a.0 + nx, a.1 + ny),
            (b.0 + nx, b.1 + ny),
            (b.0 - nx, b.1 - ny),
            (a.0 - nx, a.1 - ny),
        ]
    }

    /// Les quatre coins de la cloison, de part et d'autre de son axe.
    pub fn coins(&self) -> [(f64, f64); 4] {
        self.rectangle(0.0, self.longueur)
    }

    /// Ses parties pleines, sans le vide de sa porte.
    pub fn morceaux(&self) -> Vec<[(f64, f64); 4]> {
        match self.porte {
            None => vec![self.coins()],
            Some(porte) => [(0.0, porte.debut), (porte.fin, self.longueur)]
                .into_iter()
                .filter(|(t0, t1)| t1 - t0 > 1e-6)
                .map(|(t0, t1)| self.rectangle(t0, t1))
                .collect(),
        }
    }

    /// Le côté où s'ouvre sa porte : à gauche de son axe, ou de l'autre côté
    /// si elle est inversée (une direction unitaire).
    pub fn cote_de_la_porte(&self) -> (f64, f64) {
        let (ux, uy) = self.sens();
        if self.porte.is_some_and(|porte| porte.inversee) {
            (uy, -ux)
        } else {
            (-uy, ux)
        }
    }

    /// La distance d'un point (pouces) à l'axe de la cloison.
    pub fn distance(&self, (x, y): (f64, f64)) -> f64 {
        let (dx, dy) = (self.fin.0 - self.debut.0, self.fin.1 - self.debut.1);
        let carre = dx * dx + dy * dy;
        let t = if carre <= 1e-12 {
            0.0
        } else {
            (((x - self.debut.0) * dx + (y - self.debut.1) * dy) / carre).clamp(0.0, 1.0)
        };
        (x - self.debut.0 - t * dx).hypot(y - self.debut.1 - t * dy)
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
    let croisements: Vec<f64> = appuis
        .iter()
        .filter_map(|appui| croisement(debut, u, *appui))
        .collect();
    let le_plus_proche = |cible: f64, permis: &dyn Fn(f64) -> bool| {
        croisements
            .iter()
            .copied()
            .filter(|t| permis(*t) && (t - cible).abs() <= aimant)
            .min_by(|a, b| (a - cible).abs().total_cmp(&(b - cible).abs()))
    };
    let t_debut = le_plus_proche(0.0, &|t| t <= longueur - LONGUEUR_MINIMALE).unwrap_or(0.0);
    let t_fin = le_plus_proche(longueur, &|t| t >= t_debut + LONGUEUR_MINIMALE).unwrap_or(longueur);
    let longueur = t_fin - t_debut;
    (longueur >= LONGUEUR_MINIMALE).then_some((
        sens,
        (debut.0 + u.0 * t_debut, debut.1 + u.1 * t_debut),
        longueur,
    ))
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
    fn les_morceaux_et_la_porte_d_une_cloison() {
        let mut cloison = CloisonMesuree {
            debut: (48.0, 0.0),
            fin: (48.0, 120.0),
            longueur: 120.0,
            epaisseur: 5.0,
            porte: Some(PorteMesuree {
                debut: 44.0,
                fin: 76.0,
                inversee: false,
            }),
        };
        // Deux morceaux pleins, de part et d'autre de la porte.
        let morceaux = cloison.morceaux();
        assert_eq!(morceaux.len(), 2);
        assert!(morceaux[0].iter().all(|c| c.1 <= 44.0 + 1e-9));
        assert!(morceaux[1].iter().all(|c| c.1 >= 76.0 - 1e-9));
        assert_eq!(cloison.point(44.0), (48.0, 44.0));
        // Elle s'ouvre à gauche de son axe, ou de l'autre côté une fois inversée.
        assert_eq!(cloison.cote_de_la_porte(), (-1.0, 0.0));
        cloison.porte = Some(PorteMesuree {
            debut: 44.0,
            fin: 76.0,
            inversee: true,
        });
        assert_eq!(cloison.cote_de_la_porte(), (1.0, 0.0));
        // Une porte contre le bout A ne laisse qu'un morceau.
        cloison.porte = Some(PorteMesuree {
            debut: 0.0,
            fin: 32.0,
            inversee: false,
        });
        assert_eq!(cloison.morceaux().len(), 1);
    }

    #[test]
    fn les_coins_et_la_distance_d_une_cloison() {
        let cloison = CloisonMesuree {
            debut: (48.0, 0.0),
            fin: (48.0, 120.0),
            longueur: 120.0,
            epaisseur: 4.5,
            porte: None,
        };
        let coins = cloison.coins();
        let xs: Vec<f64> = coins.iter().map(|c| c.0).collect();
        assert!(
            xs.iter()
                .all(|x| (x - 45.75).abs() < 1e-9 || (x - 50.25).abs() < 1e-9)
        );
        assert!((cloison.distance((50.0, 60.0)) - 2.0).abs() < 1e-9);
        assert!((cloison.distance((48.0, 130.0)) - 10.0).abs() < 1e-9);
    }
}
