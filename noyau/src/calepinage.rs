//! Le calepinage (évolution 19 de la section 27) : la pose de la céramique,
//! du plancher ou des feuilles de gypse du plafond, dessinée sur la pièce — le
//! mur de départ, au coin ou au centre, le sens, le joint et le décalage des
//! rangs —, avec le compte des pièces entières et coupées et la perte réelle.

use serde::{Deserialize, Serialize};

use crate::analyse::analyser_longueur;
use crate::unites::Format;

/// Ce qui se pose.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MateriauAPoser {
    #[default]
    Ceramique,
    /// Des lames de plancher.
    Plancher,
    /// Des feuilles de gypse, au plafond.
    Gypse,
}

impl MateriauAPoser {
    pub const TOUS: [MateriauAPoser; 3] = [
        MateriauAPoser::Ceramique,
        MateriauAPoser::Plancher,
        MateriauAPoser::Gypse,
    ];

    /// Ses formats courants : (largeur, longueur), en pouces. Le premier est
    /// celui d'une pose qui n'a pas le sien.
    pub fn formats(self) -> &'static [(f64, f64)] {
        match self {
            MateriauAPoser::Ceramique => &[
                (12.0, 24.0),
                (12.0, 12.0),
                (18.0, 18.0),
                (24.0, 24.0),
                (6.0, 24.0),
                (3.0, 6.0),
                (24.0, 48.0),
            ],
            MateriauAPoser::Plancher => &[(7.0, 48.0), (5.0, 48.0), (7.5, 50.0), (9.0, 60.0)],
            MateriauAPoser::Gypse => &[(48.0, 96.0), (48.0, 120.0), (48.0, 144.0), (54.0, 144.0)],
        }
    }

    /// Son joint de départ, en pouces : 1/8 po pour la céramique, aucun pour
    /// le plancher et le gypse.
    pub fn joint_de_depart(self) -> f64 {
        match self {
            MateriauAPoser::Ceramique => 0.125,
            MateriauAPoser::Plancher | MateriauAPoser::Gypse => 0.0,
        }
    }

    /// Le décalage de départ de ses rangs : alignés pour la céramique, au
    /// tiers pour les lames, à la moitié pour les feuilles de gypse.
    pub fn decalage_de_depart(self) -> Decalage {
        match self {
            MateriauAPoser::Ceramique => Decalage::Aligne,
            MateriauAPoser::Plancher => Decalage::Tiers,
            MateriauAPoser::Gypse => Decalage::Demi,
        }
    }
}

/// Les joints offerts, en pouces : aucun, puis de 1/16 po à 1/2 po.
pub const JOINTS: [f64; 7] = [0.0, 0.0625, 0.125, 0.1875, 0.25, 0.375, 0.5];

/// Au-delà de ce nombre de pièces, la pose ne se calcule pas : le format est
/// trop petit pour la pièce.
pub const PIECES_AU_PLUS: usize = 20_000;

/// Un morceau plus petit (en pouces carrés) ne se pose pas : le joint ou la
/// plinthe le couvre.
const AIRE_NEGLIGEABLE: f64 = 0.5;

/// D'où part la pose, sur le mur de départ.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Depart {
    /// Du premier coin du mur : une pièce entière dans le coin.
    #[default]
    Coin,
    /// Du centre de la pièce : une pièce centrée, les coupes égales de part
    /// et d'autre.
    Centre,
}

/// Le sens de la pose, par rapport au mur de départ.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SensDeLaPose {
    /// La longueur des pièces le long du mur.
    #[default]
    Droit,
    /// La longueur des pièces à angle droit du mur.
    Travers,
    /// En diagonale, à 45° du mur.
    Diagonale,
}

impl SensDeLaPose {
    pub const TOUS: [SensDeLaPose; 3] = [
        SensDeLaPose::Droit,
        SensDeLaPose::Travers,
        SensDeLaPose::Diagonale,
    ];
}

/// Le décalage d'un rang au suivant.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Decalage {
    /// Les joints alignés.
    #[default]
    Aligne,
    /// À la moitié d'une pièce (à coupe de pierre).
    Demi,
    /// Au tiers d'une pièce.
    Tiers,
}

impl Decalage {
    pub const TOUS: [Decalage; 3] = [Decalage::Aligne, Decalage::Demi, Decalage::Tiers];
}

/// Le calepinage d'une pièce, tel que choisi.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Calepinage {
    /// Vrai s'il paraît sur le plan.
    pub visible: bool,
    pub materiau: MateriauAPoser,
    /// La largeur et la longueur d'une pièce, telles qu'inscrites; vides : le
    /// premier format du matériau.
    pub largeur: String,
    pub longueur: String,
    /// Le joint, en pouces.
    pub joint: f64,
    /// Le rang du mur de départ : la pose le suit, depuis son premier coin.
    pub mur: usize,
    pub depart: Depart,
    pub sens: SensDeLaPose,
    pub decalage: Decalage,
}

impl Default for Calepinage {
    fn default() -> Self {
        Calepinage {
            visible: false,
            materiau: MateriauAPoser::Ceramique,
            largeur: String::new(),
            longueur: String::new(),
            joint: MateriauAPoser::Ceramique.joint_de_depart(),
            mur: 0,
            depart: Depart::Coin,
            sens: SensDeLaPose::Droit,
            decalage: Decalage::Aligne,
        }
    }
}

impl Calepinage {
    /// La largeur et la longueur d'une pièce, en pouces : celles qui sont
    /// inscrites (un nombre seul est en pieds, comme les autres mesures du
    /// dessin), sinon le premier format du matériau.
    pub fn dimensions(&self) -> (f64, f64) {
        let (largeur, longueur) = self.materiau.formats()[0];
        let lire = |texte: &str, defaut: f64| {
            analyser_longueur(texte.trim(), Format::PiPo)
                .ok()
                .filter(|v| v.est_longueur() && v.base > 0.0 && v.base.is_finite())
                .map_or(defaut, |v| v.base)
        };
        (lire(&self.largeur, largeur), lire(&self.longueur, longueur))
    }

    /// Change de matériau : la pose prend son format, son joint et son
    /// décalage de départ.
    pub fn changer_de_materiau(&mut self, materiau: MateriauAPoser) {
        self.materiau = materiau;
        self.largeur.clear();
        self.longueur.clear();
        self.joint = materiau.joint_de_depart();
        self.decalage = materiau.decalage_de_depart();
    }
}

/// Une pièce posée : son contour dans la pièce, sur le plan (pouces), et vrai
/// si elle est entière.
#[derive(Clone, Debug, PartialEq)]
pub struct PiecePosee {
    pub contour: Vec<(f64, f64)>,
    pub entiere: bool,
}

/// Un trait du plan, d'un point à un autre (pouces).
pub type TraitDuPlan = ((f64, f64), (f64, f64));

/// La pose calculée.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Pose {
    pub pieces: Vec<PiecePosee>,
    /// Les bords des pièces, coupés à la pièce : les traits à dessiner.
    pub traits: Vec<TraitDuPlan>,
    /// Le point de départ, sur le plan : le coin du mur ou le centre.
    pub depart: (f64, f64),
    pub entieres: usize,
    pub coupees: usize,
    /// La part du matériau qui tombe, de 0 à 1 : les chutes des pièces
    /// coupées, sans réemploi.
    pub perte: f64,
    /// La plus petite coupe droite, en pouces : la largeur ou la longueur qui
    /// reste à la plus étroite des pièces coupées.
    pub plus_petite_coupe: Option<f64>,
    /// Vrai si le format est trop petit pour la pièce (plus de
    /// `PIECES_AU_PLUS` pièces) : rien ne se calcule alors.
    pub trop_de_pieces: bool,
}

/// La pose d'un calepinage dans le polygone de la pièce (pouces).
pub fn poser(calepinage: &Calepinage, polygone: &[(f64, f64)]) -> Pose {
    let mut polygone = polygone.to_vec();
    if polygone.len() > 1 && polygone.first() == polygone.last() {
        polygone.pop();
    }
    let n = polygone.len();
    if n < 3 {
        return Pose::default();
    }
    let (largeur, longueur) = calepinage.dimensions();
    let joint = calepinage.joint.max(0.0);
    // Le repère de la pose : l'origine au premier coin du mur de départ, x le
    // long des pièces, y d'un rang au suivant, vers le dedans de la pièce.
    let mur = calepinage.mur % n;
    let a = polygone[mur];
    let b = polygone[(mur + 1) % n];
    let longueur_du_mur = (b.0 - a.0).hypot(b.1 - a.1);
    if longueur_du_mur < 1e-9 {
        return Pose::default();
    }
    let u = ((b.0 - a.0) / longueur_du_mur, (b.1 - a.1) / longueur_du_mur);
    // Une aire signée positive : le dedans est du côté de (−y, x).
    let vers_le_dedans = if aire_signee(&polygone) >= 0.0 {
        (-u.1, u.0)
    } else {
        (u.1, -u.0)
    };
    let r = std::f64::consts::FRAC_1_SQRT_2;
    let (axe_x, axe_y) = match calepinage.sens {
        SensDeLaPose::Droit => (u, vers_le_dedans),
        SensDeLaPose::Travers => (vers_le_dedans, u),
        SensDeLaPose::Diagonale => (
            ((u.0 + vers_le_dedans.0) * r, (u.1 + vers_le_dedans.1) * r),
            ((vers_le_dedans.0 - u.0) * r, (vers_le_dedans.1 - u.1) * r),
        ),
    };
    let vers_la_pose = |p: (f64, f64)| {
        let (dx, dy) = (p.0 - a.0, p.1 - a.1);
        (dx * axe_x.0 + dy * axe_x.1, dx * axe_y.0 + dy * axe_y.1)
    };
    let vers_le_plan = |(x, y): (f64, f64)| {
        (
            a.0 + x * axe_x.0 + y * axe_y.0,
            a.1 + x * axe_x.1 + y * axe_y.1,
        )
    };
    let salle: Vec<(f64, f64)> = polygone.iter().map(|&p| vers_la_pose(p)).collect();
    let (x0, x1, y0, y1) = etendue(&salle);
    let (pas_x, pas_y) = (longueur + joint, largeur + joint);
    let estimees = ((x1 - x0) / pas_x + 3.0) * ((y1 - y0) / pas_y + 3.0);
    if estimees > PIECES_AU_PLUS as f64 {
        return Pose {
            trop_de_pieces: true,
            ..Pose::default()
        };
    }
    let (origine, depart) = match calepinage.depart {
        Depart::Coin => ((0.0, 0.0), (0.0, 0.0)),
        Depart::Centre => {
            let centre = ((x0 + x1) / 2.0, (y0 + y1) / 2.0);
            (
                (centre.0 - longueur / 2.0, centre.1 - largeur / 2.0),
                centre,
            )
        }
    };
    let mut pose = Pose {
        depart: vers_le_plan(depart),
        ..Pose::default()
    };
    let aire_d_une_piece = longueur * largeur;
    let (mut achetee, mut posee) = (0.0, 0.0);
    let premier_rang = ((y0 - origine.1) / pas_y).floor() as i64 - 1;
    let dernier_rang = ((y1 - origine.1) / pas_y).ceil() as i64 + 1;
    for rang in premier_rang..=dernier_rang {
        let y = origine.1 + rang as f64 * pas_y;
        let decalage = match calepinage.decalage {
            Decalage::Aligne => 0.0,
            Decalage::Demi => rang.rem_euclid(2) as f64 * pas_x / 2.0,
            Decalage::Tiers => rang.rem_euclid(3) as f64 * pas_x / 3.0,
        };
        let premiere = ((x0 - origine.0 - decalage) / pas_x).floor() as i64 - 1;
        let derniere = ((x1 - origine.0 - decalage) / pas_x).ceil() as i64 + 1;
        for colonne in premiere..=derniere {
            let x = origine.0 + decalage + colonne as f64 * pas_x;
            let piece = [
                (x, y),
                (x + longueur, y),
                (x + longueur, y + largeur),
                (x, y + largeur),
            ];
            let contour = sans_points_doubles(couper(&salle, &piece));
            let aire = aire_signee(&contour).abs();
            if aire < AIRE_NEGLIGEABLE {
                continue;
            }
            let entiere = aire_d_une_piece - aire < 1e-6 * aire_d_une_piece;
            if entiere {
                pose.entieres += 1;
            } else {
                pose.coupees += 1;
                // Sa coupe : la longueur ou la largeur qui lui reste.
                let (cx0, cx1, cy0, cy1) = etendue(&contour);
                for (reste, pleine) in [(cx1 - cx0, longueur), (cy1 - cy0, largeur)] {
                    if reste < pleine - 1e-6 {
                        pose.plus_petite_coupe =
                            Some(pose.plus_petite_coupe.map_or(reste, |c: f64| c.min(reste)));
                    }
                }
            }
            achetee += aire_d_une_piece;
            posee += aire;
            for k in 0..4 {
                for (p, q) in segments_dedans(piece[k], piece[(k + 1) % 4], &salle) {
                    pose.traits.push((vers_le_plan(p), vers_le_plan(q)));
                }
            }
            pose.pieces.push(PiecePosee {
                contour: contour.into_iter().map(vers_le_plan).collect(),
                entiere,
            });
        }
    }
    sans_doublons(&mut pose.traits);
    pose.perte = if achetee > 0.0 {
        (achetee - posee) / achetee
    } else {
        0.0
    };
    pose
}

/// L'aire signée d'un polygone : positive s'il tourne dans le sens des
/// aiguilles d'une montre à l'écran (y vers le bas).
fn aire_signee(polygone: &[(f64, f64)]) -> f64 {
    let n = polygone.len();
    if n < 3 {
        return 0.0;
    }
    (0..n)
        .map(|k| {
            let (a, b) = (polygone[k], polygone[(k + 1) % n]);
            a.0 * b.1 - b.0 * a.1
        })
        .sum::<f64>()
        / 2.0
}

/// L'étendue d'un polygone : (x min, x max, y min, y max).
fn etendue(polygone: &[(f64, f64)]) -> (f64, f64, f64, f64) {
    polygone.iter().fold(
        (f64::MAX, f64::MIN, f64::MAX, f64::MIN),
        |(x0, x1, y0, y1), &(x, y)| (x0.min(x), x1.max(x), y0.min(y), y1.max(y)),
    )
}

/// Vrai si le point est dans le polygone (règle pair-impair).
fn dedans((x, y): (f64, f64), polygone: &[(f64, f64)]) -> bool {
    let n = polygone.len();
    let mut interieur = false;
    for k in 0..n {
        let (a, b) = (polygone[k], polygone[(k + 1) % n]);
        if (a.1 > y) != (b.1 > y) && x < a.0 + (y - a.1) * (b.0 - a.0) / (b.1 - a.1) {
            interieur = !interieur;
        }
    }
    interieur
}

/// Le polygone `sujet` coupé au rectangle `fenetre` (méthode de
/// Sutherland-Hodgman : la fenêtre est convexe, le sujet peut ne pas l'être).
fn couper(sujet: &[(f64, f64)], fenetre: &[(f64, f64); 4]) -> Vec<(f64, f64)> {
    let mut sortie = sujet.to_vec();
    // La fenêtre tourne dans le sens des aiguilles d'une montre (y vers le
    // bas) : le dedans est à droite de chacun de ses côtés.
    for k in 0..4 {
        if sortie.is_empty() {
            break;
        }
        let (a, b) = (fenetre[k], fenetre[(k + 1) % 4]);
        let (ex, ey) = (b.0 - a.0, b.1 - a.1);
        let cote = |p: (f64, f64)| ex * (p.1 - a.1) - ey * (p.0 - a.0);
        let entree = std::mem::take(&mut sortie);
        for i in 0..entree.len() {
            let (precedent, courant) = (entree[(i + entree.len() - 1) % entree.len()], entree[i]);
            let (c0, c1) = (cote(precedent), cote(courant));
            let croisement = || {
                let t = c0 / (c0 - c1);
                (
                    precedent.0 + (courant.0 - precedent.0) * t,
                    precedent.1 + (courant.1 - precedent.1) * t,
                )
            };
            match (c1 >= -1e-9, c0 >= -1e-9) {
                (true, true) => sortie.push(courant),
                (true, false) => {
                    sortie.push(croisement());
                    sortie.push(courant);
                }
                (false, true) => sortie.push(croisement()),
                (false, false) => {}
            }
        }
    }
    sortie
}

/// Les morceaux du segment pq qui sont dans le polygone.
fn segments_dedans(p: (f64, f64), q: (f64, f64), polygone: &[(f64, f64)]) -> Vec<TraitDuPlan> {
    let (dx, dy) = (q.0 - p.0, q.1 - p.1);
    let mut coupures = vec![0.0, 1.0];
    let n = polygone.len();
    for k in 0..n {
        let (a, b) = (polygone[k], polygone[(k + 1) % n]);
        let (ex, ey) = (b.0 - a.0, b.1 - a.1);
        let denominateur = dx * ey - dy * ex;
        if denominateur.abs() < 1e-12 {
            continue;
        }
        let t = ((a.0 - p.0) * ey - (a.1 - p.1) * ex) / denominateur;
        let s = ((a.0 - p.0) * dy - (a.1 - p.1) * dx) / denominateur;
        if t > 0.0 && t < 1.0 && (-1e-9..=1.0 + 1e-9).contains(&s) {
            coupures.push(t);
        }
    }
    coupures.sort_by(f64::total_cmp);
    // Les intervalles dans le polygone, ceux qui se touchent réunis.
    let mut intervalles: Vec<(f64, f64)> = Vec::new();
    for paire in coupures.windows(2) {
        let (t0, t1) = (paire[0], paire[1]);
        let milieu = (t0 + t1) / 2.0;
        let point = (p.0 + dx * milieu, p.1 + dy * milieu);
        if t1 - t0 < 1e-9 || !dedans(point, polygone) || sur_le_bord(point, polygone) {
            continue;
        }
        match intervalles.last_mut() {
            Some(dernier) if (dernier.1 - t0).abs() < 1e-9 => dernier.1 = t1,
            _ => intervalles.push((t0, t1)),
        }
    }
    intervalles
        .into_iter()
        .map(|(t0, t1)| {
            (
                (p.0 + dx * t0, p.1 + dy * t0),
                (p.0 + dx * t1, p.1 + dy * t1),
            )
        })
        .collect()
}

/// Vrai si le point est sur un côté du polygone.
fn sur_le_bord(point: (f64, f64), polygone: &[(f64, f64)]) -> bool {
    let n = polygone.len();
    (0..n).any(|k| {
        let (a, b) = (polygone[k], polygone[(k + 1) % n]);
        let (ex, ey) = (b.0 - a.0, b.1 - a.1);
        let carre = ex * ex + ey * ey;
        if carre < 1e-18 {
            return false;
        }
        let t = (((point.0 - a.0) * ex + (point.1 - a.1) * ey) / carre).clamp(0.0, 1.0);
        (a.0 + ex * t - point.0).hypot(a.1 + ey * t - point.1) < 1e-6
    })
}

/// Le contour sans ses points doubles, le dernier compris s'il redonne le
/// premier.
fn sans_points_doubles(contour: Vec<(f64, f64)>) -> Vec<(f64, f64)> {
    let proches =
        |a: (f64, f64), b: (f64, f64)| (a.0 - b.0).abs() < 1e-7 && (a.1 - b.1).abs() < 1e-7;
    let mut sortie: Vec<(f64, f64)> = Vec::with_capacity(contour.len());
    for point in contour {
        if sortie
            .last()
            .is_none_or(|&dernier| !proches(dernier, point))
        {
            sortie.push(point);
        }
    }
    while sortie.len() > 1 && proches(sortie[0], sortie[sortie.len() - 1]) {
        sortie.pop();
    }
    sortie
}

/// Retire les traits en double : le bord commun de deux pièces voisines.
fn sans_doublons(traits: &mut Vec<TraitDuPlan>) {
    let cle = |&(p, q): &TraitDuPlan| {
        let arrondi = |v: f64| (v * 1e4).round() as i64;
        let (p, q) = ((arrondi(p.0), arrondi(p.1)), (arrondi(q.0), arrondi(q.1)));
        if q < p { (q, p) } else { (p, q) }
    };
    traits.sort_by_key(cle);
    traits.dedup_by_key(|t| cle(t));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Une pièce de 12 pi × 10 pi, tracée dans le sens des aiguilles d'une
    /// montre à l'écran.
    const PIECE: [(f64, f64); 4] = [(0.0, 0.0), (144.0, 0.0), (144.0, 120.0), (0.0, 120.0)];

    fn ceramique(largeur: &str, longueur: &str, joint: f64) -> Calepinage {
        Calepinage {
            largeur: largeur.into(),
            longueur: longueur.into(),
            joint,
            ..Calepinage::default()
        }
    }

    fn aire_posee(pose: &Pose) -> f64 {
        pose.pieces
            .iter()
            .map(|piece| aire_signee(&piece.contour).abs())
            .sum()
    }

    #[test]
    fn les_dimensions_d_une_piece() {
        // Rien d'inscrit : le premier format du matériau; un nombre seul est
        // en pieds, comme les autres mesures du dessin.
        assert_eq!(Calepinage::default().dimensions(), (12.0, 24.0));
        assert_eq!(ceramique("12 po", "2", 0.0).dimensions(), (12.0, 24.0));
        let mut gypse = Calepinage::default();
        gypse.changer_de_materiau(MateriauAPoser::Gypse);
        assert_eq!(gypse.dimensions(), (48.0, 96.0));
        assert_eq!((gypse.joint, gypse.decalage), (0.0, Decalage::Demi));
    }

    #[test]
    fn une_ceramique_de_12_po_depuis_le_coin() {
        // Des carreaux de 12 po, joints de 1/8 po : 11 × 9 entiers, une
        // rangée et une colonne coupées, et le coin qui reste.
        let pose = poser(&ceramique("12 po", "12 po", 0.125), &PIECE);
        assert_eq!(pose.entieres, 11 * 9);
        assert_eq!(pose.coupees, 11 + 9 + 1);
        // La colonne coupée garde 144 − 11 × 12 1/8 = 10 5/8 po.
        assert!((pose.plus_petite_coupe.unwrap() - 10.625).abs() < 1e-9);
        assert!(pose.perte > 0.0 && pose.perte < 0.1);
        // Le premier carreau, entier, dans le coin de départ.
        assert_eq!(pose.depart, (0.0, 0.0));
        assert!(pose.pieces.iter().any(|piece| {
            piece.entiere
                && piece
                    .contour
                    .iter()
                    .any(|&(x, y)| x.abs() < 1e-9 && y.abs() < 1e-9)
        }));
        // Les traits restent dans la pièce, sans doublon.
        for &((x0, y0), (x1, y1)) in &pose.traits {
            for (x, y) in [(x0, y0), (x1, y1)] {
                assert!((-1e-6..=144.0 + 1e-6).contains(&x) && (-1e-6..=120.0 + 1e-6).contains(&y));
            }
        }
        let mut traits = pose.traits.clone();
        sans_doublons(&mut traits);
        assert_eq!(traits.len(), pose.traits.len());
    }

    #[test]
    fn depuis_le_centre_les_coupes_s_equilibrent() {
        // Un carreau centré : 6 po de coupe contre chacun des quatre murs.
        let pose = poser(
            &Calepinage {
                depart: Depart::Centre,
                ..ceramique("12 po", "12 po", 0.0)
            },
            &PIECE,
        );
        assert_eq!(pose.entieres, 11 * 9);
        assert_eq!(pose.coupees, 2 * 11 + 2 * 9 + 4);
        assert!((pose.plus_petite_coupe.unwrap() - 6.0).abs() < 1e-9);
        assert_eq!(pose.depart, (72.0, 60.0));
        // Sans joint, la pose couvre toute la pièce.
        assert!((aire_posee(&pose) - 144.0 * 120.0).abs() < 1e-6);
    }

    #[test]
    fn depuis_le_coin_d_un_autre_mur() {
        // Le mur 3 part du coin (144, 120) : le premier carreau y est entier.
        let pose = poser(
            &Calepinage {
                mur: 2,
                ..ceramique("12 po", "12 po", 0.125)
            },
            &PIECE,
        );
        assert_eq!(pose.depart, (144.0, 120.0));
        assert!(pose.pieces.iter().any(|piece| {
            piece.entiere
                && piece
                    .contour
                    .iter()
                    .any(|&(x, y)| (x - 144.0).abs() < 1e-9 && (y - 120.0).abs() < 1e-9)
        }));
        assert_eq!(pose.entieres, 11 * 9);
    }

    #[test]
    fn des_feuilles_de_gypse_et_des_lames_decalees() {
        // Des feuilles de 4 pi × 8 pi : deux entières, la pose couvre tout.
        let gypse = Calepinage {
            materiau: MateriauAPoser::Gypse,
            largeur: "4".into(),
            longueur: "8".into(),
            joint: 0.0,
            ..Calepinage::default()
        };
        let pose = poser(&gypse, &PIECE);
        assert_eq!((pose.entieres, pose.coupees), (2, 4));
        assert!((aire_posee(&pose) - 144.0 * 120.0).abs() < 1e-6);
        // En travers, les feuilles de 8 pi vont d'un long mur vers l'autre.
        let travers = poser(
            &Calepinage {
                sens: SensDeLaPose::Travers,
                ..gypse.clone()
            },
            &PIECE,
        );
        assert_eq!((travers.entieres, travers.coupees), (3, 3));
        // Des lames de 5 po × 4 pi décalées de moitié : 24 rangs, les rangs
        // impairs commencent et finissent par une demi-lame.
        let lames = Calepinage {
            materiau: MateriauAPoser::Plancher,
            largeur: "5 po".into(),
            longueur: "4".into(),
            joint: 0.0,
            decalage: Decalage::Demi,
            ..Calepinage::default()
        };
        let pose = poser(&lames, &PIECE);
        assert_eq!(pose.entieres, 12 * 3 + 12 * 2);
        assert_eq!(pose.coupees, 12 * 2);
        assert!((pose.plus_petite_coupe.unwrap() - 24.0).abs() < 1e-9);
        assert!((aire_posee(&pose) - 144.0 * 120.0).abs() < 1e-6);
    }

    #[test]
    fn en_diagonale_et_dans_une_piece_en_l() {
        // Des carreaux de 12 po sans joint tombent juste : aucune perte; en
        // diagonale, des coupes partout.
        let droit = poser(&ceramique("12 po", "12 po", 0.0), &PIECE);
        assert_eq!((droit.entieres, droit.coupees), (120, 0));
        // Ses traits : les joints entre les carreaux, sans ceux des murs.
        assert_eq!(droit.traits.len(), 11 * 10 + 9 * 12);
        // Chaque carreau coupé se découpe en triangles qui le couvrent.
        for piece in &poser(&diagonale_de_12(), &PIECE).pieces {
            let triangles = crate::maquette::trianguler(&piece.contour);
            let aire: f64 = triangles.iter().map(|t| aire_signee(t).abs()).sum();
            assert!((aire - aire_signee(&piece.contour).abs()).abs() < 1e-6);
        }
        assert!(droit.perte.abs() < 1e-9);
        let diagonale = Calepinage {
            sens: SensDeLaPose::Diagonale,
            ..ceramique("12 po", "12 po", 0.0)
        };
        let pose = poser(&diagonale, &PIECE);
        assert!(pose.coupees > 0 && pose.perte > 0.05);
        // Une pièce en L de 10 pi, sans son coin de 5 pi : la pose en
        // diagonale la couvre, à quelques miettes près, et ses traits
        // n'entrent pas dans le coin retiré.
        let l = [
            (0.0, 0.0),
            (120.0, 0.0),
            (120.0, 60.0),
            (60.0, 60.0),
            (60.0, 120.0),
            (0.0, 120.0),
        ];
        let pose = poser(&diagonale, &l);
        assert!((aire_posee(&pose) - (120.0 * 120.0 - 60.0 * 60.0)).abs() < 3.0);
        for &(p, q) in &pose.traits {
            let milieu = ((p.0 + q.0) / 2.0, (p.1 + q.1) / 2.0);
            assert!(
                !(milieu.0 > 60.0 + 1e-6 && milieu.1 > 60.0 + 1e-6),
                "{milieu:?}"
            );
        }
    }

    fn diagonale_de_12() -> Calepinage {
        Calepinage {
            sens: SensDeLaPose::Diagonale,
            ..ceramique("12 po", "12 po", 0.125)
        }
    }

    #[test]
    fn un_format_trop_petit_pour_la_piece() {
        // Des pièces de 1 po dans une pièce de 100 pi de côté : trop.
        let grande = [(0.0, 0.0), (1200.0, 0.0), (1200.0, 1200.0), (0.0, 1200.0)];
        let pose = poser(&ceramique("1 po", "1 po", 0.0), &grande);
        assert!(pose.trop_de_pieces && pose.pieces.is_empty());
    }
}
