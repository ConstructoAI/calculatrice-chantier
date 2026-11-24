//! Dessin d'une pièce (section 25) : un croquis tracé à main levée devient une
//! forme aux angles d'équerre — ou à 45° pour un mur en biais —, dont chaque
//! côté reçoit ensuite sa mesure. Un côté peut ensuite prendre n'importe quel
//! angle par rapport au précédent (section 25.1, point 32).
//!
//! Le croquis n'est pas à l'échelle : seules comptent la direction de chaque
//! côté et les mesures inscrites. Un côté manquant se déduit des autres quand
//! la fermeture de la forme le détermine (R-80).

use serde::{Deserialize, Serialize};

use crate::analyse::analyser_longueur;
use crate::etiquettes::symbole;
use crate::metiers::{Calculateur, Metier, Nature};
use crate::unites::{Format, PO_PAR_M};

/// Écart toléré, en pouces, entre des mesures qui devraient fermer la forme.
pub const TOLERANCE_PO: f64 = 1.0 / 16.0;
/// Part de la plus grande dimension du croquis sous laquelle la fin du tracé
/// est considérée comme revenue au point de départ.
const PROXIMITE_DE_FERMETURE: f64 = 0.16;
/// Part du plus long côté d'un tracé sous laquelle un côté est un tremblement.
const PART_NEGLIGEABLE: f64 = 0.10;
/// Finesse du redressement, en part de la diagonale du tracé.
const FINESSE: f64 = 0.07;

/// Direction d'un côté, comme sur l'écran : l'est à droite, le sud en bas. Les
/// quatre directions en biais sont à 45°; toute autre direction est libre.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Sens {
    Est,
    Sud,
    Ouest,
    Nord,
    NordEst,
    SudEst,
    SudOuest,
    NordOuest,
    /// Une direction qui n'est pas un multiple de 45° : son angle en millièmes
    /// de degré, de 0 (vers l'est) à 360 000, compté vers le sud comme à
    /// l'écran (section 25.1, point 32). Les dessins enregistrés sans elle se lisent
    /// tels quels.
    Libre(i32),
}

/// L'axe d'un côté : horizontal, vertical, l'une des deux diagonales à 45°, ou
/// un axe libre.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axe {
    Horizontal,
    Vertical,
    /// Du haut à gauche vers le bas à droite.
    Descendant,
    /// Du bas à gauche vers le haut à droite.
    Montant,
    /// Un autre axe : l'angle de son sens positif en millièmes de degré, de 0
    /// à 180 000 (exclu), jamais un multiple de 45 000.
    Libre(i32),
}

/// Millièmes de degré dans un demi-tour.
const DEMI_TOUR: i32 = 180_000;
/// Sous ce sinus (un demi-degré), deux axes sont trop près d'être parallèles
/// pour que la fermeture de la forme les départage.
const SINUS_MINIMAL: f64 = 0.0087;

/// Composante d'un déplacement unitaire à 45°.
const DEMI_RACINE: f64 = std::f64::consts::FRAC_1_SQRT_2;
/// Demi-largeur, en degrés, de la bande autour de 45° où un trait devient un
/// côté en biais; hors de cette bande, il est horizontal ou vertical.
const BANDE_DU_BIAIS: f64 = 12.0;

impl Axe {
    /// Le sens positif de l'axe.
    fn positif(self) -> Sens {
        match self {
            Axe::Horizontal => Sens::Est,
            Axe::Vertical => Sens::Sud,
            Axe::Descendant => Sens::SudEst,
            Axe::Montant => Sens::NordEst,
            Axe::Libre(angle) => Sens::Libre(angle),
        }
    }

    /// Le sens négatif de l'axe.
    fn negatif(self) -> Sens {
        match self {
            Axe::Horizontal => Sens::Ouest,
            Axe::Vertical => Sens::Nord,
            Axe::Descendant => Sens::NordOuest,
            Axe::Montant => Sens::SudOuest,
            Axe::Libre(angle) => Sens::Libre(angle + DEMI_TOUR),
        }
    }

    /// Le déplacement unitaire du sens positif.
    pub fn vecteur(self) -> (f64, f64) {
        self.positif().pas()
    }

    /// L'axe d'un déplacement : en biais s'il est à 45° à quelques degrés
    /// près, sinon horizontal ou vertical.
    pub(crate) fn du_deplacement(dx: f64, dy: f64) -> Axe {
        let angle = dy.atan2(dx).to_degrees().rem_euclid(180.0);
        if (angle - 45.0).abs() <= BANDE_DU_BIAIS {
            Axe::Descendant
        } else if (angle - 135.0).abs() <= BANDE_DU_BIAIS {
            Axe::Montant
        } else if !(45.0..=135.0).contains(&angle) {
            Axe::Horizontal
        } else {
            Axe::Vertical
        }
    }
}

impl Sens {
    /// La direction d'un angle en degrés, compté comme à l'écran : 0 vers la
    /// droite (l'est), 90 vers le bas (le sud). Un multiple de 45° donne l'une
    /// des huit directions; tout autre angle, une direction libre, au
    /// millième de degré.
    pub fn d_angle(degres: f64) -> Sens {
        let angle = ((degres * 1000.0).round() as i64).rem_euclid(2 * DEMI_TOUR as i64) as i32;
        match angle {
            0 => Sens::Est,
            45_000 => Sens::SudEst,
            90_000 => Sens::Sud,
            135_000 => Sens::SudOuest,
            180_000 => Sens::Ouest,
            225_000 => Sens::NordOuest,
            270_000 => Sens::Nord,
            315_000 => Sens::NordEst,
            angle => Sens::Libre(angle),
        }
    }

    /// L'angle de la direction en degrés, de 0 (vers la droite) à 360, compté
    /// vers le bas comme à l'écran.
    pub fn angle(self) -> f64 {
        match self {
            Sens::Est => 0.0,
            Sens::SudEst => 45.0,
            Sens::Sud => 90.0,
            Sens::SudOuest => 135.0,
            Sens::Ouest => 180.0,
            Sens::NordOuest => 225.0,
            Sens::Nord => 270.0,
            Sens::NordEst => 315.0,
            Sens::Libre(angle) => f64::from(angle) / 1000.0,
        }
    }

    /// La direction vue dans un miroir vertical : l'est et l'ouest échangés.
    pub fn en_miroir(self) -> Sens {
        match self {
            Sens::Est => Sens::Ouest,
            Sens::Ouest => Sens::Est,
            Sens::NordEst => Sens::NordOuest,
            Sens::NordOuest => Sens::NordEst,
            Sens::SudEst => Sens::SudOuest,
            Sens::SudOuest => Sens::SudEst,
            Sens::Nord => Sens::Nord,
            Sens::Sud => Sens::Sud,
            Sens::Libre(_) => Sens::d_angle(180.0 - self.angle()),
        }
    }

    /// L'axe du côté.
    pub fn axe(self) -> Axe {
        match self {
            Sens::Est | Sens::Ouest => Axe::Horizontal,
            Sens::Sud | Sens::Nord => Axe::Vertical,
            Sens::SudEst | Sens::NordOuest => Axe::Descendant,
            Sens::NordEst | Sens::SudOuest => Axe::Montant,
            // Lu d'un fichier, l'angle n'est peut-être pas ramené à son tour.
            Sens::Libre(_) => match Sens::d_angle(self.angle()) {
                Sens::Libre(angle) => Axe::Libre(angle % DEMI_TOUR),
                ramene => ramene.axe(),
            },
        }
    }

    /// Vrai pour un côté horizontal.
    pub fn horizontal(self) -> bool {
        self.axe() == Axe::Horizontal
    }

    /// Vrai pour un côté ni horizontal ni vertical : à 45° ou à un angle libre.
    pub fn en_biais(self) -> bool {
        !matches!(self.axe(), Axe::Horizontal | Axe::Vertical)
    }

    /// Vrai pour une direction libre, qui n'est pas un multiple de 45°.
    pub fn libre(self) -> bool {
        matches!(self.axe(), Axe::Libre(_))
    }

    /// Déplacement unitaire (x vers la droite, y vers le bas).
    pub fn pas(self) -> (f64, f64) {
        let d = DEMI_RACINE;
        match self {
            Sens::Est => (1.0, 0.0),
            Sens::Sud => (0.0, 1.0),
            Sens::Ouest => (-1.0, 0.0),
            Sens::Nord => (0.0, -1.0),
            Sens::NordEst => (d, -d),
            Sens::SudEst => (d, d),
            Sens::SudOuest => (-d, d),
            Sens::NordOuest => (-d, -d),
            Sens::Libre(_) => {
                let angle = self.angle().to_radians();
                (angle.cos(), angle.sin())
            }
        }
    }

    /// L'angle dont cette direction tourne depuis `avant`, en degrés, de −180
    /// à 180 : positif vers la droite de qui avance, comme du sud à l'ouest.
    pub fn virage_depuis(self, avant: Sens) -> f64 {
        (self.angle() - avant.angle() + 540.0).rem_euclid(360.0) - 180.0
    }

    /// +1 dans le sens positif de son axe, −1 dans l'autre.
    fn signe(self) -> f64 {
        if self == self.axe().positif() {
            1.0
        } else {
            -1.0
        }
    }

    pub(crate) fn selon(axe: Axe, longueur_signee: f64) -> Sens {
        if longueur_signee >= 0.0 {
            axe.positif()
        } else {
            axe.negatif()
        }
    }
}

/// Un côté de la forme.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Cote {
    pub sens: Sens,
    /// Longueur sur le croquis (sans unité : seule la proportion compte).
    pub croquis: f64,
    /// La mesure, telle que l'utilisateur l'a inscrite; vide s'il n'y en a pas.
    #[serde(default)]
    pub saisie: String,
}

impl Cote {
    fn signe(&self) -> f64 {
        self.sens.signe() * self.croquis
    }
}

/// L'épaisseur des murs extérieurs d'une pièce sans épaisseur inscrite, en
/// pouces. Les murs se dessinent hors des mesures, qui sont celles de
/// l'intérieur de la pièce.
pub const EPAISSEUR_DES_MURS_PAR_DEFAUT: f64 = 9.0;

/// Un morceau plein d'un mur extérieur, entre ses ouvertures : ses quatre
/// coins (pouces) — l'intérieur, de son début à sa fin, puis l'extérieur, de
/// sa fin à son début — et ses jambages (ses bouts contre une ouverture).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MorceauDeMur {
    pub cote: usize,
    pub coins: [(f64, f64); 4],
    pub jambage_au_debut: bool,
    pub jambage_a_la_fin: bool,
}

/// Le sens d'un côté et la direction de l'extérieur de la pièce.
#[derive(Clone, Copy)]
struct Repere {
    u: (f64, f64),
    dehors: (f64, f64),
}

/// Le repère de chaque côté d'un contour fermé.
fn reperes_des_cotes(sommets: &[(f64, f64)]) -> Vec<Repere> {
    let aire: f64 = sommets
        .windows(2)
        .map(|s| s[0].0 * s[1].1 - s[1].0 * s[0].1)
        .sum();
    sommets
        .windows(2)
        .map(|s| {
            let longueur = (s[1].0 - s[0].0).hypot(s[1].1 - s[0].1).max(1e-12);
            let u = ((s[1].0 - s[0].0) / longueur, (s[1].1 - s[0].1) / longueur);
            let dehors = if aire > 0.0 { (u.1, -u.0) } else { (-u.1, u.0) };
            Repere { u, dehors }
        })
        .collect()
}

/// Les sommets d'un contour fermé dont chaque côté est repoussé de
/// `distance` vers l'extérieur de la pièce (ou vers l'intérieur), chaque coin
/// à la rencontre de deux côtés repoussés; le contour reste fermé.
fn decaler(
    sommets: &[(f64, f64)],
    distance: f64,
    vers_l_exterieur: bool,
) -> Option<Vec<(f64, f64)>> {
    let n = sommets.len().checked_sub(1).filter(|n| *n >= 3)?;
    let e = if vers_l_exterieur {
        distance
    } else {
        -distance
    };
    let reperes = reperes_des_cotes(sommets);
    let mut contour: Vec<(f64, f64)> = (0..n)
        .map(|i| {
            let avant = (i + n - 1) % n;
            let (r1, r2) = (reperes[avant], reperes[i]);
            let p1 = (
                sommets[avant].0 + r1.dehors.0 * e,
                sommets[avant].1 + r1.dehors.1 * e,
            );
            let p2 = (
                sommets[i].0 + r2.dehors.0 * e,
                sommets[i].1 + r2.dehors.1 * e,
            );
            let croix = r1.u.0 * r2.u.1 - r1.u.1 * r2.u.0;
            if croix.abs() < 1e-9 {
                p2
            } else {
                let s = ((p2.0 - p1.0) * r2.u.1 - (p2.1 - p1.1) * r2.u.0) / croix;
                (p1.0 + r1.u.0 * s, p1.1 + r1.u.1 * s)
            }
        })
        .collect();
    contour.push(contour[0]);
    Some(contour)
}

/// La direction de l'extérieur de la pièce, pour chaque côté d'un contour
/// fermé (unitaire).
pub fn dehors_des_cotes(sommets: &[(f64, f64)]) -> Vec<(f64, f64)> {
    reperes_des_cotes(sommets)
        .into_iter()
        .map(|repere| repere.dehors)
        .collect()
}

/// L'aire (pouces carrés) et le périmètre (pouces) d'un contour fermé.
fn aire_et_perimetre(sommets: &[(f64, f64)]) -> (f64, f64) {
    let aire: f64 = sommets
        .windows(2)
        .map(|s| s[0].0 * s[1].1 - s[1].0 * s[0].1)
        .sum();
    let perimetre = sommets
        .windows(2)
        .map(|s| (s[1].0 - s[0].0).hypot(s[1].1 - s[0].1))
        .sum();
    (aire.abs() / 2.0, perimetre)
}

/// Une forme tirée de ses diagonales : ses sommets (pouces, le départ répété
/// à la fin), et l'écart entre les deux diagonales quand les deux sont
/// mesurées (section 25.1, point 33).
type FormeReelle = (Vec<(f64, f64)>, Option<f64>);

/// Un segment de cloison à coter : le rang de sa cloison, le sien, ses deux
/// bouts (pouces) et l'épaisseur de sa cloison.
type SegmentACoter = (usize, usize, (f64, f64), (f64, f64), f64);

/// Une chaîne de cotes (section 25.1, point 35) : ce qu'elle mesure, de
/// `debut` à `fin` (pouces, sur le plan), et ses points de mesure.
#[derive(Clone, Debug, PartialEq)]
pub struct ChaineDeCotes {
    pub debut: (f64, f64),
    pub fin: (f64, f64),
    /// Vers où la ligne de cote s'écarte de ce qu'elle mesure (unitaire) :
    /// l'extérieur, pour un mur; nul pour l'écart entre deux cloisons, qui se
    /// cote entre elles.
    pub dehors: (f64, f64),
    /// Les points de mesure, en pouces depuis `debut`, en ordre : le premier
    /// est 0, le dernier la longueur.
    pub points: Vec<f64>,
    /// Le côté dont elle cote le mur; `None` pour l'écart entre deux cloisons.
    pub mur: Option<usize>,
}

/// L'angle (radians) entre deux côtés d'un triangle, d'après leurs longueurs
/// et celle du côté qui leur fait face; `None` si ces trois longueurs ne font
/// pas un triangle, à 1/16 po près (un triangle aplati compte).
fn angle_du_triangle(cote: f64, autre: f64, en_face: f64) -> Option<f64> {
    if cote <= 0.0 || autre <= 0.0 || en_face <= 0.0 {
        return None;
    }
    if en_face > cote + autre + TOLERANCE_PO || en_face < (cote - autre).abs() - TOLERANCE_PO {
        return None;
    }
    let cosinus = (cote * cote + autre * autre - en_face * en_face) / (2.0 * cote * autre);
    Some(cosinus.clamp(-1.0, 1.0).acos())
}

/// Le point à `r1` du centre `c1` et à `r2` du centre `c2`, du côté `cote`
/// (+1 ou −1) de la droite de `c1` à `c2`; `None` si les deux cercles ne se
/// rencontrent pas, à 1/16 po près.
fn rencontre_de_cercles(
    c1: (f64, f64),
    r1: f64,
    c2: (f64, f64),
    r2: f64,
    cote: f64,
) -> Option<(f64, f64)> {
    let (dx, dy) = (c2.0 - c1.0, c2.1 - c1.1);
    let distance = dx.hypot(dy);
    if distance <= 1e-9
        || distance > r1 + r2 + TOLERANCE_PO
        || distance < (r1 - r2).abs() - TOLERANCE_PO
    {
        return None;
    }
    let le_long = (r1 * r1 - r2 * r2 + distance * distance) / (2.0 * distance);
    let de_travers = (r1 * r1 - le_long * le_long).max(0.0).sqrt() * cote;
    let (ux, uy) = (dx / distance, dy / distance);
    Some((
        c1.0 + ux * le_long - uy * de_travers,
        c1.1 + uy * le_long + ux * de_travers,
    ))
}

/// L'angle intérieur de chaque coin d'un contour fermé (le départ répété à la
/// fin), en degrés : 90 pour un coin d'équerre, plus de 180 pour un coin
/// rentrant. Le coin k est le début du côté k.
pub fn angles_des_coins(sommets: &[(f64, f64)]) -> Vec<f64> {
    let n = sommets.len().saturating_sub(1);
    if n < 3 {
        return Vec::new();
    }
    let aire: f64 = sommets
        .windows(2)
        .map(|s| s[0].0 * s[1].1 - s[1].0 * s[0].1)
        .sum();
    let signe = if aire < 0.0 { -1.0 } else { 1.0 };
    (0..n)
        .map(|i| {
            let (avant, ici, apres) = (sommets[(i + n - 1) % n], sommets[i], sommets[i + 1]);
            let u = (ici.0 - avant.0, ici.1 - avant.1);
            let v = (apres.0 - ici.0, apres.1 - ici.1);
            let virage = (u.0 * v.1 - u.1 * v.0)
                .atan2(u.0 * v.0 + u.1 * v.1)
                .to_degrees();
            180.0 - signe * virage
        })
        .collect()
}

/// L'écart d'un coin à l'équerre, mesuré à `sur` du coin : pour un coin de
/// cet angle (degrés), le bout d'un côté s'écarte de `sur` × tan(écart) de
/// l'équerre — « 3/8 po sur 4 pi » (section 25.1, point 33).
pub fn ecart_a_l_equerre(angle: f64, sur: f64) -> f64 {
    sur * (angle - 90.0).to_radians().tan().abs()
}

/// Une note écrite sur le plan : son texte, et sa place en pouces dans le
/// repère du plan (le départ du tracé en (0, 0), y vers le bas), comme un objet.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Note {
    pub texte: String,
    pub position: (f64, f64),
}

/// La forme dessinée : ses côtés dans l'ordre du tracé.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Dessin {
    pub cotes: Vec<Cote>,
    /// Vrai quand le dernier côté rejoint le premier.
    pub fermee: bool,
    /// Ce qui se retire de la surface : un îlot, une trémie, une colonne.
    pub retraits: Vec<Retrait>,
    /// La hauteur des murs, telle qu'elle a été inscrite; vide s'il n'y en a pas.
    pub hauteur_des_murs: String,
    /// Les portes et les fenêtres, retirées de la surface des murs.
    pub ouvertures: Vec<Ouverture>,
    /// L'escalier et le mobilier, posés sur le plan.
    pub objets: Vec<crate::mobilier::Objet>,
    /// Les notes écrites sur le plan.
    pub notes: Vec<Note>,
    /// Les traits à main levée, par-dessus le plan (évolution 30).
    pub croquis: Vec<TraitAMainLevee>,
    /// Les finis de la pièce : plancher, peinture, plafond, plinthes (évolution 27).
    pub finis: crate::finis::Finis,
    /// Les noms donnés aux espaces que ferment les cloisons (évolution 5).
    pub noms_des_espaces: Vec<crate::espaces::NomDEspace>,
    /// Vrai si l'ossature (les montants) paraît sur le plan (évolution 28).
    pub ossature_visible: bool,
    /// L'entraxe des montants de l'ossature.
    pub entraxe: crate::ossature::Entraxe,
    /// La pose de la céramique, du plancher ou du gypse (évolution 19).
    pub calepinage: crate::calepinage::Calepinage,
    /// Les cloisons : des murs intérieurs.
    pub cloisons: Vec<crate::cloison::Cloison>,
    /// L'épaisseur des murs extérieurs, telle qu'inscrite; vide : 9 po.
    pub epaisseur_des_murs: String,
    /// L'épaisseur des cloisons qui n'ont pas la leur; vide : 5 po.
    pub epaisseur_des_cloisons: String,
    /// Vrai si les mesures sont celles de l'extérieur des murs : les murs se
    /// dessinent alors vers l'intérieur.
    pub mesures_exterieures: bool,
    /// Les diagonales mesurées d'une pièce à quatre côtés (section 25.1,
    /// point 33), telles qu'inscrites : du coin 1 au coin 3, puis du coin 2 au
    /// coin 4 (le coin k est le début du côté k); vides si elles ne sont pas
    /// mesurées.
    pub diagonales: [String; 2],
    /// Le plafond (section 25.1, point 34) : plat, cathédrale ou en pente.
    pub plafond: crate::plafond::Plafond,
    /// Les retombées de plafond.
    pub retombees: Vec<crate::plafond::Retombee>,
    /// Vrai quand les lignes de cote se montrent, à l'écran et dans les
    /// exports (section 25.1, point 35).
    pub lignes_de_cote: bool,
}

/// Une ouverture dans les murs : une porte ou une fenêtre.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Ouverture {
    pub largeur: String,
    pub hauteur: String,
    /// Porte ou fenêtre. Absent d'un dessin de la version 0.8 : une ouverture
    /// de 6 pi ou plus de haut est alors une porte.
    pub genre: Option<GenreDOuverture>,
    /// Le rang du côté (du mur) où elle se trouve; `None` tant qu'elle n'est
    /// pas placée.
    pub cote: Option<usize>,
    /// La distance entre le premier coin de son mur et son bord, telle
    /// qu'elle a été inscrite; vide : l'ouverture est centrée.
    pub position: String,
    /// La hauteur du bas d'une fenêtre (son allège), telle qu'elle a été
    /// inscrite; vide : 3 pi. Une porte part du plancher.
    pub allege: String,
    /// Vrai si sa charnière (ou le côté où glisse son panneau) est au bout
    /// opposé, celui de sa fin (évolution 24).
    pub inversee: bool,
}

/// L'allège d'une fenêtre quand elle n'est pas inscrite : 3 pi.
pub const ALLEGE_PAR_DEFAUT: f64 = 36.0;
/// La hauteur d'une porte ou d'une fenêtre quand elle n'est pas lisible.
const HAUTEUR_DE_PORTE_PAR_DEFAUT: f64 = 80.0;
const HAUTEUR_DE_FENETRE_PAR_DEFAUT: f64 = 48.0;

/// Le genre d'une ouverture, pour la dessiner (évolution 24 : dix genres).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GenreDOuverture {
    /// Une porte battante, à une charnière.
    Porte,
    Fenetre,
    /// Deux vantaux battants, qui se rejoignent au milieu.
    PorteDouble,
    /// Deux panneaux qui glissent l'un devant l'autre, dans le mur.
    PorteCoulissante,
    /// Un panneau qui glisse sur un rail, devant le mur.
    PorteDeGrange,
    /// Des panneaux qui se plient en accordéon.
    PortePliante,
    /// Une porte-fenêtre (une porte-patio) : deux panneaux vitrés qui glissent.
    PorteFenetre,
    /// Une porte de garage, qui monte sous le plafond.
    PorteDeGarage,
    /// Une baie libre : une ouverture sans porte.
    BaieLibre,
    /// Une fenêtre en baie, qui avance hors du mur.
    FenetreEnBaie,
}

/// Ce qui remplit une ouverture en 3D.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Remplissage {
    Porte,
    Vitre,
    Rien,
}

impl GenreDOuverture {
    /// Tous les genres, dans l'ordre de la liste : les portes, la baie libre,
    /// puis les fenêtres.
    pub const TOUS: [GenreDOuverture; 10] = [
        GenreDOuverture::Porte,
        GenreDOuverture::PorteDouble,
        GenreDOuverture::PorteCoulissante,
        GenreDOuverture::PorteDeGrange,
        GenreDOuverture::PortePliante,
        GenreDOuverture::PorteFenetre,
        GenreDOuverture::PorteDeGarage,
        GenreDOuverture::BaieLibre,
        GenreDOuverture::Fenetre,
        GenreDOuverture::FenetreEnBaie,
    ];

    /// Vrai pour ce qui part du plancher, sans allège : les portes et la baie
    /// libre (un P au bordereau); faux pour les fenêtres (un F).
    pub fn est_une_porte(self) -> bool {
        !matches!(
            self,
            GenreDOuverture::Fenetre | GenreDOuverture::FenetreEnBaie
        )
    }

    /// Vrai pour une porte qui tourne sur ses charnières : son battant balaie
    /// un quart de disque (évolution 23).
    pub fn battante(self) -> bool {
        matches!(self, GenreDOuverture::Porte | GenreDOuverture::PorteDouble)
    }

    /// Vrai si « Inverser » la change : la charnière d'une porte, le côté où
    /// glisse un panneau.
    pub fn a_un_sens(self) -> bool {
        matches!(
            self,
            GenreDOuverture::Porte
                | GenreDOuverture::PorteCoulissante
                | GenreDOuverture::PorteDeGrange
                | GenreDOuverture::PorteFenetre
        )
    }

    /// Ce qui la remplit en 3D : du verre, une porte, ou rien.
    pub fn remplissage(self) -> Remplissage {
        match self {
            GenreDOuverture::Fenetre
            | GenreDOuverture::FenetreEnBaie
            | GenreDOuverture::PorteFenetre => Remplissage::Vitre,
            GenreDOuverture::BaieLibre => Remplissage::Rien,
            _ => Remplissage::Porte,
        }
    }

    /// Ses formats courants (largeur, hauteur, en pouces), le premier de la
    /// liste étant celui de départ.
    pub fn formats(self) -> &'static [(f64, f64)] {
        match self {
            GenreDOuverture::Porte => &[
                (32.0, 80.0),
                (24.0, 80.0),
                (28.0, 80.0),
                (30.0, 80.0),
                (34.0, 80.0),
                (36.0, 80.0),
            ],
            GenreDOuverture::PorteDouble => &[(60.0, 80.0), (48.0, 80.0), (72.0, 80.0)],
            GenreDOuverture::PorteCoulissante => {
                &[(60.0, 80.0), (48.0, 80.0), (72.0, 80.0), (96.0, 80.0)]
            }
            GenreDOuverture::PorteDeGrange => &[(36.0, 84.0), (30.0, 84.0), (42.0, 84.0)],
            GenreDOuverture::PortePliante => &[
                (36.0, 80.0),
                (24.0, 80.0),
                (30.0, 80.0),
                (48.0, 80.0),
                (60.0, 80.0),
                (72.0, 80.0),
            ],
            GenreDOuverture::PorteFenetre => &[(72.0, 80.0), (60.0, 80.0), (96.0, 80.0)],
            GenreDOuverture::PorteDeGarage => {
                &[(108.0, 84.0), (96.0, 84.0), (192.0, 84.0), (216.0, 84.0)]
            }
            GenreDOuverture::BaieLibre => &[(36.0, 84.0), (30.0, 80.0), (48.0, 84.0), (60.0, 84.0)],
            GenreDOuverture::Fenetre => &[
                (36.0, 48.0),
                (24.0, 36.0),
                (30.0, 48.0),
                (48.0, 48.0),
                (60.0, 48.0),
                (72.0, 48.0),
            ],
            GenreDOuverture::FenetreEnBaie => &[(72.0, 54.0), (84.0, 54.0), (96.0, 60.0)],
        }
    }
}

/// Le style d'un trait du symbole d'une ouverture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StyleDOuverture {
    /// Épais : un battant, un panneau fermé.
    Panneau,
    /// Fin : l'arc d'un battant, le contour d'un panneau.
    Fin,
    /// En tirets : un rail, une porte de garage levée, une baie libre.
    Tirets,
    /// Le vitrage d'une fenêtre ou d'une porte-fenêtre.
    Vitrage,
}

/// Un trait du symbole d'une ouverture : une ligne brisée, et son style.
#[derive(Clone, Debug, PartialEq)]
pub struct TraitDOuverture {
    pub points: Vec<(f64, f64)>,
    pub style: StyleDOuverture,
}

/// Le symbole des nouveaux genres d'ouverture (évolution 24), dans les unités
/// de l'appelant (pouces, points de l'écran ou de la page) : `q0` et `q1`,
/// ses bords sur la ligne des mesures; `dedans`, unitaire, vers la pièce;
/// `epaisseur`, celle du mur, négative s'il est vers l'intérieur (des mesures
/// prises à l'extérieur); `course`, la profondeur que prend une porte de
/// garage levée; `pouce`, la longueur d'un pouce dans ces unités. La porte
/// battante et la fenêtre ont leur dessin à part.
pub fn symbole_d_ouverture(
    genre: GenreDOuverture,
    inversee: bool,
    (q0, q1): ((f64, f64), (f64, f64)),
    dedans: (f64, f64),
    epaisseur: f64,
    (course, pouce): (f64, f64),
) -> Vec<TraitDOuverture> {
    let largeur = (q1.0 - q0.0).hypot(q1.1 - q0.1);
    if largeur < 1e-9 {
        return Vec::new();
    }
    let u = ((q1.0 - q0.0) / largeur, (q1.1 - q0.1) / largeur);
    let n = dedans;
    // Les faces du mur, comptées le long de `dedans` depuis la ligne des mesures.
    let interieure = (-epaisseur).max(0.0);
    let exterieure = interieure - epaisseur.abs();
    let mince = epaisseur.abs() < 1e-9;
    let point = |le_long: f64, vers_dedans: f64| {
        (
            q0.0 + u.0 * le_long + n.0 * vers_dedans,
            q0.1 + u.1 * le_long + n.1 * vers_dedans,
        )
    };
    let trait_ = |points: Vec<(f64, f64)>, style| TraitDOuverture { points, style };
    // Un panneau mince entre deux points le long du mur, à une profondeur.
    let panneau = |de: f64, a: f64, milieu: f64, demi: f64| {
        vec![
            point(de, milieu - demi),
            point(a, milieu - demi),
            point(a, milieu + demi),
            point(de, milieu + demi),
            point(de, milieu - demi),
        ]
    };
    // L'épaisseur d'un panneau, à l'échelle de l'ouverture.
    let demi = (largeur * 0.02).max(epaisseur.abs() * 0.08);
    let mut traits = Vec::new();
    match genre {
        GenreDOuverture::PorteDouble => {
            // Deux battants, chacun de la moitié de la largeur, et leurs arcs.
            for (charniere, sens) in [(0.0, 1.0), (largeur, -1.0)] {
                let base = point(charniere, interieure);
                let rayon = largeur / 2.0;
                traits.push(trait_(
                    vec![base, (base.0 + n.0 * rayon, base.1 + n.1 * rayon)],
                    StyleDOuverture::Panneau,
                ));
                traits.push(trait_(
                    (0..=12)
                        .map(|i| {
                            let angle = std::f64::consts::FRAC_PI_2 * f64::from(i) / 12.0;
                            let (c, s) = (angle.cos(), angle.sin());
                            (
                                base.0 + (u.0 * sens * c + n.0 * s) * rayon,
                                base.1 + (u.1 * sens * c + n.1 * s) * rayon,
                            )
                        })
                        .collect(),
                    StyleDOuverture::Fin,
                ));
            }
        }
        GenreDOuverture::PorteCoulissante | GenreDOuverture::PorteFenetre => {
            // Deux panneaux qui se chevauchent, dans l'épaisseur du mur.
            let (a, b) = if mince {
                (-demi * 1.6, demi * 1.6)
            } else {
                (
                    exterieure + epaisseur.abs() / 3.0,
                    exterieure + 2.0 * epaisseur.abs() / 3.0,
                )
            };
            let (devant, derriere) = if inversee { (b, a) } else { (a, b) };
            for (de, fin, milieu) in [
                (0.0, largeur * 0.55, devant),
                (largeur * 0.45, largeur, derriere),
            ] {
                traits.push(trait_(panneau(de, fin, milieu, demi), StyleDOuverture::Fin));
                if genre == GenreDOuverture::PorteFenetre {
                    traits.push(trait_(
                        vec![point(de, milieu), point(fin, milieu)],
                        StyleDOuverture::Vitrage,
                    ));
                }
            }
        }
        GenreDOuverture::PorteDeGrange => {
            // Le panneau, devant la face intérieure, et son rail, qui file du
            // côté où il glisse.
            let milieu = interieure + demi * 2.0;
            traits.push(trait_(
                panneau(-largeur * 0.04, largeur * 1.04, milieu, demi),
                StyleDOuverture::Panneau,
            ));
            let rail = if inversee {
                vec![
                    point(-largeur * 1.04, milieu + demi * 2.5),
                    point(largeur * 1.04, milieu + demi * 2.5),
                ]
            } else {
                vec![
                    point(-largeur * 0.04, milieu + demi * 2.5),
                    point(largeur * 2.04, milieu + demi * 2.5),
                ]
            };
            traits.push(trait_(rail, StyleDOuverture::Tirets));
        }
        GenreDOuverture::PortePliante => {
            // Les panneaux en accordéon : un V, ou deux pour une large porte.
            // Une porte de 4 pi ou plus se plie des deux côtés.
            if largeur >= 47.9 * pouce {
                let creux = largeur / 4.0 * 0.8;
                traits.push(trait_(
                    vec![
                        point(0.0, interieure),
                        point(largeur / 4.0, interieure + creux),
                        point(largeur / 2.0, interieure),
                    ],
                    StyleDOuverture::Panneau,
                ));
                traits.push(trait_(
                    vec![
                        point(largeur / 2.0, interieure),
                        point(largeur * 0.75, interieure + creux),
                        point(largeur, interieure),
                    ],
                    StyleDOuverture::Panneau,
                ));
            } else {
                let creux = largeur / 2.0 * 0.6;
                traits.push(trait_(
                    vec![
                        point(0.0, interieure),
                        point(largeur / 2.0, interieure + creux),
                        point(largeur, interieure),
                    ],
                    StyleDOuverture::Panneau,
                ));
            }
        }
        GenreDOuverture::PorteDeGarage => {
            // La porte fermée, à la face intérieure, et levée, en tirets sous
            // le plafond.
            traits.push(trait_(
                vec![point(0.0, interieure), point(largeur, interieure)],
                StyleDOuverture::Panneau,
            ));
            let fond = interieure + course;
            traits.push(trait_(
                vec![
                    point(0.0, interieure),
                    point(0.0, fond),
                    point(largeur, fond),
                    point(largeur, interieure),
                ],
                StyleDOuverture::Tirets,
            ));
        }
        GenreDOuverture::BaieLibre => {
            traits.push(trait_(
                vec![point(0.0, interieure), point(largeur, interieure)],
                StyleDOuverture::Tirets,
            ));
        }
        GenreDOuverture::FenetreEnBaie => {
            // Le vitrage dans le mur, et la baie qui avance au dehors, ses
            // côtés à 45°.
            traits.push(trait_(
                vec![point(0.0, exterieure), point(largeur, exterieure)],
                StyleDOuverture::Fin,
            ));
            // La baie avance du quart de sa largeur.
            let avance = largeur / 4.0;
            let baie = vec![
                point(0.0, exterieure),
                point(avance, exterieure - avance),
                point(largeur - avance, exterieure - avance),
                point(largeur, exterieure),
            ];
            traits.push(trait_(baie, StyleDOuverture::Fin));
            traits.push(trait_(
                vec![
                    point(avance, exterieure - avance * 0.85),
                    point(largeur - avance, exterieure - avance * 0.85),
                ],
                StyleDOuverture::Vitrage,
            ));
        }
        GenreDOuverture::Porte | GenreDOuverture::Fenetre => {}
    }
    // Sans épaisseur, des montants marquent les bords d'une ouverture.
    if mince && genre != GenreDOuverture::FenetreEnBaie {
        for bout in [0.0, largeur] {
            traits.push(trait_(
                vec![point(bout, -demi * 2.5), point(bout, demi * 2.5)],
                StyleDOuverture::Fin,
            ));
        }
    }
    traits
}

impl Ouverture {
    /// Vrai pour une porte. Sans genre, une ouverture de 6 pi (72 po) ou plus
    /// de haut en est une; `hauteur` est sa hauteur en pouces, si elle est lisible.
    pub fn est_une_porte(&self, hauteur: Option<f64>) -> bool {
        self.genre_ou_deduit(hauteur).est_une_porte()
    }

    /// Son genre; sans genre (un dessin de la version 0.8), une porte si elle
    /// a 6 pi (72 po) ou plus de haut, une fenêtre sinon.
    pub fn genre_ou_deduit(&self, hauteur: Option<f64>) -> GenreDOuverture {
        self.genre.unwrap_or(if hauteur.is_some_and(|h| h >= 72.0) {
            GenreDOuverture::Porte
        } else {
            GenreDOuverture::Fenetre
        })
    }
}

/// Où se trouve une ouverture, pour la dessiner : son mur, et ses deux bords
/// en pouces depuis le premier coin de ce mur.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placement {
    pub cote: usize,
    pub debut: f64,
    pub fin: f64,
    /// Vrai pour ce qui part du plancher (une porte, une baie libre).
    pub porte: bool,
    pub genre: GenreDOuverture,
    pub inversee: bool,
    /// Le bas et le haut de l'ouverture, en pouces depuis le plancher.
    pub bas: f64,
    pub haut: f64,
}

/// La surface des murs d'une pièce, en pouces carrés.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Murs {
    /// Périmètre × hauteur des murs.
    pub brute: f64,
    /// Les portes et les fenêtres.
    pub ouvertures: f64,
    /// Ce qui reste à peindre ou à couvrir.
    pub nette: f64,
}

/// Un rectangle à retirer de la surface de la pièce.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Retrait {
    pub longueur: String,
    pub largeur: String,
}

/// Une pièce du plan : son nom et son dessin.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Piece {
    /// Le nom donné par l'utilisateur; vide tant qu'il n'en a pas donné.
    pub nom: String,
    pub dessin: Dessin,
}

/// Le crayon d'un trait à main levée (évolution 30) : sa couleur, ou le
/// surligneur, large et transparent, qui passe sous le plan.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Crayon {
    #[default]
    Rouge,
    Bleu,
    Vert,
    Noir,
    Surligneur,
}

impl Crayon {
    /// Les crayons, dans l'ordre de la palette.
    pub const TOUS: [Crayon; 5] = [
        Crayon::Rouge,
        Crayon::Bleu,
        Crayon::Vert,
        Crayon::Noir,
        Crayon::Surligneur,
    ];
}

/// Un trait à main levée par-dessus le plan (évolution 30) : ses points
/// (pouces, sur le plan) et son crayon.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TraitAMainLevee {
    pub points: Vec<(f64, f64)>,
    #[serde(default)]
    pub crayon: Crayon,
}

/// Allège un trait à main levée (méthode de Ramer-Douglas-Peucker) : ne garde
/// que les points qui s'écartent de plus de `tolerance` (pouces) du trait
/// simplifié; le premier et le dernier restent.
pub fn simplifier_le_trait(points: &[(f64, f64)], tolerance: f64) -> Vec<(f64, f64)> {
    if points.len() < 3 {
        return points.to_vec();
    }
    let mut gardes = vec![false; points.len()];
    gardes[0] = true;
    gardes[points.len() - 1] = true;
    let mut a_voir = vec![(0, points.len() - 1)];
    while let Some((debut, fin)) = a_voir.pop() {
        let (a, b) = (points[debut], points[fin]);
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let longueur = dx.hypot(dy);
        let ecart = |p: (f64, f64)| {
            if longueur < 1e-12 {
                (p.0 - a.0).hypot(p.1 - a.1)
            } else {
                ((p.0 - a.0) * dy - (p.1 - a.1) * dx).abs() / longueur
            }
        };
        let plus_loin =
            (debut + 1..fin).max_by(|i, j| ecart(points[*i]).total_cmp(&ecart(points[*j])));
        if let Some(i) = plus_loin
            && ecart(points[i]) > tolerance
        {
            gardes[i] = true;
            a_voir.push((debut, i));
            a_voir.push((i, fin));
        }
    }
    points
        .iter()
        .zip(gardes)
        .filter(|(_, garde)| *garde)
        .map(|(point, _)| *point)
        .collect()
}

/// Une ligne du bordereau des portes et des fenêtres (évolution 24).
#[derive(Clone, Debug, PartialEq)]
pub struct LigneDuBordereau {
    /// Son repère : P1, P2… pour les portes et les baies, F1… pour les fenêtres.
    pub repere: String,
    pub genre: GenreDOuverture,
    /// Sa largeur et sa hauteur (pouces), si elles se lisent.
    pub largeur: Option<f64>,
    pub hauteur: Option<f64>,
    /// Pour une porte battante placée sur son mur : vrai si elle est poussant
    /// gauche (vu du dehors de la pièce, en la poussant, la charnière à
    /// gauche), faux si elle est poussant droite.
    pub poussant_gauche: Option<bool>,
    /// Le nom de sa pièce, et le rang de la pièce et de l'ouverture.
    pub piece: String,
    pub rang_de_la_piece: usize,
    pub rang: usize,
}

impl Plan {
    /// Le repère de chaque ouverture de chaque pièce (évolution 24) : les
    /// portes et les baies libres P1, P2…, les fenêtres F1, F2…, numérotées à
    /// la suite d'une pièce à l'autre.
    pub fn reperes_des_ouvertures(&self) -> Vec<Vec<String>> {
        let (mut portes, mut fenetres) = (0, 0);
        self.pieces
            .iter()
            .map(|piece| {
                piece
                    .dessin
                    .ouvertures
                    .iter()
                    .map(|ouverture| {
                        let hauteur = crate::analyse::analyser_longueur(
                            ouverture.hauteur.trim(),
                            Format::PiPo,
                        )
                        .ok()
                        .filter(|v| v.est_longueur())
                        .map(|v| v.base);
                        if ouverture.est_une_porte(hauteur) {
                            portes += 1;
                            format!("P{portes}")
                        } else {
                            fenetres += 1;
                            format!("F{fenetres}")
                        }
                    })
                    .collect()
            })
            .collect()
    }

    /// Le bordereau des portes et des fenêtres de tout le plan (évolution
    /// 24), dans l'ordre des pièces : les portes d'abord, puis les fenêtres.
    pub fn bordereau(&self, format: Format) -> Vec<LigneDuBordereau> {
        let reperes = self.reperes_des_ouvertures();
        let mut lignes = Vec::new();
        for (rang_de_la_piece, piece) in self.pieces.iter().enumerate() {
            let dessin = &piece.dessin;
            let solution = dessin.resoudre(format);
            let sommets = dessin.sommets_mesures(&solution);
            let aire: f64 = sommets
                .as_ref()
                .map(|s| {
                    s.windows(2)
                        .map(|p| p[0].0 * p[1].1 - p[1].0 * p[0].1)
                        .sum()
                })
                .unwrap_or(0.0);
            for (rang, ouverture) in dessin.ouvertures.iter().enumerate() {
                let lire = |texte: &str| {
                    crate::analyse::analyser_longueur(texte.trim(), format)
                        .ok()
                        .filter(|v| v.est_longueur() && v.base > 0.0)
                        .map(|v| v.base)
                };
                let placement = solution.placements.get(rang).copied().flatten();
                let hauteur = lire(&ouverture.hauteur);
                let genre = ouverture.genre_ou_deduit(hauteur);
                // Le côté de la charnière, vu du dehors : à gauche de qui
                // regarde vers la pièce.
                let poussant_gauche = match (placement, &sommets) {
                    (Some(p), Some(sommets)) if genre == GenreDOuverture::Porte => {
                        let a = sommets.get(p.cote).copied();
                        let b = sommets.get(p.cote + 1).or(sommets.first()).copied();
                        a.zip(b).and_then(|(a, b)| {
                            let longueur = (b.0 - a.0).hypot(b.1 - a.1);
                            (longueur > 1e-9).then(|| {
                                let u = ((b.0 - a.0) / longueur, (b.1 - a.1) / longueur);
                                let dedans = if aire > 0.0 { (-u.1, u.0) } else { (u.1, -u.0) };
                                let gauche = (dedans.1, -dedans.0);
                                let charniere = if p.inversee { p.fin } else { p.debut };
                                let decalage = charniere - (p.debut + p.fin) / 2.0;
                                (u.0 * gauche.0 + u.1 * gauche.1) * decalage > 0.0
                            })
                        })
                    }
                    _ => None,
                };
                lignes.push(LigneDuBordereau {
                    repere: reperes[rang_de_la_piece][rang].clone(),
                    genre,
                    largeur: placement
                        .map(|p| p.fin - p.debut)
                        .or(lire(&ouverture.largeur)),
                    hauteur: placement.map(|p| p.haut - p.bas).or(hauteur),
                    poussant_gauche,
                    piece: piece.nom.clone(),
                    rang_de_la_piece,
                    rang,
                });
            }
        }
        // Les portes, puis les fenêtres, chacune dans l'ordre de son numéro.
        let numero = |ligne: &LigneDuBordereau| {
            (
                !ligne.repere.starts_with('P'),
                ligne.repere[1..].parse::<usize>().unwrap_or(0),
            )
        };
        lignes.sort_by_key(numero);
        lignes
    }
}

impl Dessin {
    /// Les espaces que ferment les cloisons dans la pièce complète (évolution
    /// 5); vide s'il n'y en a qu'un.
    pub fn espaces(&self, solution: &Solution) -> Vec<crate::espaces::Espace> {
        let Some(contour) = self.contour_interieur(solution) else {
            return Vec::new();
        };
        let cloisons: Vec<crate::cloison::CloisonMesuree> =
            solution.cloisons.iter().flatten().cloned().collect();
        crate::espaces::espaces(&contour, &cloisons)
    }

    /// Le nom donné à un espace : celui dont le point est en lui.
    pub fn nom_de_l_espace(&self, espace: &crate::espaces::Espace) -> Option<&str> {
        self.noms_des_espaces
            .iter()
            .find(|nom| crate::espaces::dedans(nom.point, &espace.contour))
            .map(|nom| nom.nom.as_str())
    }

    /// Nomme un espace (un nom vide lui retire le sien).
    pub fn nommer_l_espace(&mut self, espace: &crate::espaces::Espace, nom: &str) {
        self.noms_des_espaces
            .retain(|ancien| !crate::espaces::dedans(ancien.point, &espace.contour));
        if !nom.trim().is_empty() {
            self.noms_des_espaces.push(crate::espaces::NomDEspace {
                point: espace.repere,
                nom: nom.to_string(),
            });
        }
    }
}

/// Un mur vu de face (évolution 18) : sa longueur, le profil de son haut, et
/// ses portes et ses fenêtres à leur hauteur.
#[derive(Clone, Debug, PartialEq)]
pub struct Elevation {
    /// Le rang du mur (de son côté).
    pub cote: usize,
    pub longueur: f64,
    /// Le haut du mur, de son début à sa fin : (distance depuis le début,
    /// hauteur), en pouces; deux points, ou trois pour un mur en pignon.
    pub haut: Vec<(f64, f64)>,
    pub ouvertures: Vec<OuvertureEnElevation>,
}

/// Une porte ou une fenêtre d'un mur vu de face : son rang parmi les
/// ouvertures, ses bords depuis le début du mur, son bas et son haut (pouces).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OuvertureEnElevation {
    pub rang: usize,
    pub debut: f64,
    pub fin: f64,
    pub bas: f64,
    pub haut: f64,
    pub genre: GenreDOuverture,
}

impl Elevation {
    /// Sa plus grande hauteur.
    pub fn plus_haut(&self) -> f64 {
        self.haut.iter().map(|&(_, h)| h).fold(0.0, f64::max)
    }

    /// La chaîne de ses cotes au bas du mur : les coins et les bords de ses
    /// ouvertures, dans l'ordre, sans doublon.
    pub fn chaine(&self) -> Vec<f64> {
        let mut points = vec![0.0, self.longueur];
        for ouverture in &self.ouvertures {
            points.push(ouverture.debut.clamp(0.0, self.longueur));
            points.push(ouverture.fin.clamp(0.0, self.longueur));
        }
        points.sort_by(f64::total_cmp);
        points.dedup_by(|a, b| (*a - *b).abs() < 0.5);
        points
    }
}

impl Dessin {
    /// Chaque mur de la pièce complète vu de face (évolution 18), de la
    /// hauteur des murs ou sous son plafond; vide sans hauteur de murs.
    pub fn elevations(&self, solution: &Solution) -> Vec<Elevation> {
        if !matches!(solution.etat, Etat::Complete { .. }) {
            return Vec::new();
        }
        let Some(sommets) = self.sommets_mesures(solution) else {
            return Vec::new();
        };
        let hauteur = solution.hauteur;
        if hauteur.is_none() && solution.plafond.is_none() {
            return Vec::new();
        }
        (0..self.cotes.len())
            .filter_map(|cote| {
                let a = *sommets.get(cote)?;
                let b = *sommets.get(cote + 1).or(sommets.first())?;
                let longueur = (b.0 - a.0).hypot(b.1 - a.1);
                if longueur < 1e-9 {
                    return None;
                }
                let haut = match &solution.plafond {
                    Some(plafond) => {
                        let mut haut = vec![(0.0, plafond.hauteur_au(a))];
                        if let Some(t) = plafond.brisure(a, b) {
                            let point = (
                                a.0 + (b.0 - a.0) * t / longueur,
                                a.1 + (b.1 - a.1) * t / longueur,
                            );
                            haut.push((t, plafond.hauteur_au(point)));
                        }
                        haut.push((longueur, plafond.hauteur_au(b)));
                        haut
                    }
                    None => {
                        let h = hauteur?;
                        vec![(0.0, h), (longueur, h)]
                    }
                };
                let ouvertures = solution
                    .placements
                    .iter()
                    .enumerate()
                    .filter_map(|(rang, placement)| {
                        let placement = placement.as_ref()?;
                        (placement.cote == cote).then_some(OuvertureEnElevation {
                            rang,
                            debut: placement.debut,
                            fin: placement.fin,
                            bas: placement.bas,
                            haut: placement.haut,
                            genre: placement.genre,
                        })
                    })
                    .collect();
                Some(Elevation {
                    cote,
                    longueur,
                    haut,
                    ouvertures,
                })
            })
            .collect()
    }
}

/// Ce qui gêne un objet de la pièce (évolution 23).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RaisonDuConflit {
    /// Le battant de la porte de rang donné (parmi les ouvertures des murs).
    PorteDuMur(usize),
    /// Le battant d'une porte d'une cloison : la cloison, et l'ouverture.
    PorteDeCloison(usize, usize),
    /// L'objet traverse un mur, ou sort de la pièce.
    Mur,
    /// L'objet traverse la cloison de rang donné.
    Cloison(usize),
    /// L'objet en chevauche un autre, de rang plus grand.
    Objet(usize),
}

/// Un conflit : l'objet gêné (son rang), et ce qui le gêne.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Conflit {
    pub objet: usize,
    pub raison: RaisonDuConflit,
}

impl Dessin {
    /// Le battant de chaque porte, ouvert à angle droit (évolution 23) : la
    /// porte et le quart de disque que balaie son battant (un polygone
    /// convexe, sur le plan). Les portes des murs s'ouvrent vers l'intérieur,
    /// depuis la face intérieure; celles des cloisons, du côté choisi.
    pub fn battants_des_portes(
        &self,
        solution: &Solution,
    ) -> Vec<(RaisonDuConflit, Vec<(f64, f64)>)> {
        let quart = |charniere: (f64, f64), le_long: (f64, f64), dedans: (f64, f64), rayon: f64| {
            let mut polygone = vec![charniere];
            polygone.extend((0..=8).map(|i| {
                let angle = std::f64::consts::FRAC_PI_2 * f64::from(i) / 8.0;
                let (c, s) = (angle.cos(), angle.sin());
                (
                    charniere.0 + (le_long.0 * c + dedans.0 * s) * rayon,
                    charniere.1 + (le_long.1 * c + dedans.1 * s) * rayon,
                )
            }));
            polygone
        };
        let mut battants = Vec::new();
        if let Some(sommets) = self.sommets_mesures(solution)
            && sommets.len() > 2
        {
            let aire: f64 = sommets
                .windows(2)
                .map(|s| s[0].0 * s[1].1 - s[1].0 * s[0].1)
                .sum();
            for (rang, placement) in solution.placements.iter().enumerate() {
                let Some(placement) = placement.filter(|p| p.genre.battante()) else {
                    continue;
                };
                let (Some(&a), Some(&b)) = (
                    sommets.get(placement.cote),
                    sommets.get(placement.cote + 1).or(sommets.first()),
                ) else {
                    continue;
                };
                let longueur = (b.0 - a.0).hypot(b.1 - a.1);
                let rayon = placement.fin - placement.debut;
                if longueur < 1e-9 || rayon < 1e-9 {
                    continue;
                }
                let le_long = ((b.0 - a.0) / longueur, (b.1 - a.1) / longueur);
                let dedans = if aire > 0.0 {
                    (-le_long.1, le_long.0)
                } else {
                    (le_long.1, -le_long.0)
                };
                // Mesurée à l'extérieur, la porte s'ouvre de la face intérieure.
                let retrait = if self.mesures_exterieures {
                    solution.epaisseur_des_murs
                } else {
                    0.0
                };
                let charniere = |t: f64| {
                    (
                        a.0 + le_long.0 * t + dedans.0 * retrait,
                        a.1 + le_long.1 * t + dedans.1 * retrait,
                    )
                };
                let a_rebours = (-le_long.0, -le_long.1);
                let raison = RaisonDuConflit::PorteDuMur(rang);
                if placement.genre == GenreDOuverture::PorteDouble {
                    // Deux vantaux de la moitié de la largeur.
                    battants.push((
                        raison,
                        quart(charniere(placement.debut), le_long, dedans, rayon / 2.0),
                    ));
                    battants.push((
                        raison,
                        quart(charniere(placement.fin), a_rebours, dedans, rayon / 2.0),
                    ));
                } else if placement.inversee {
                    battants.push((
                        raison,
                        quart(charniere(placement.fin), a_rebours, dedans, rayon),
                    ));
                } else {
                    battants.push((
                        raison,
                        quart(charniere(placement.debut), le_long, dedans, rayon),
                    ));
                }
            }
        }
        for (rang, cloison) in solution.cloisons.iter().enumerate() {
            let Some(cloison) = cloison else {
                continue;
            };
            for (k, porte) in cloison.ouvertures.iter().enumerate() {
                if porte.passage {
                    continue;
                }
                let (q0, q1) = (
                    cloison.point(porte.segment, porte.debut),
                    cloison.point(porte.segment, porte.fin),
                );
                let rayon = (q1.0 - q0.0).hypot(q1.1 - q0.1);
                if rayon < 1e-9 {
                    continue;
                }
                let le_long = ((q1.0 - q0.0) / rayon, (q1.1 - q0.1) / rayon);
                battants.push((
                    RaisonDuConflit::PorteDeCloison(rang, k),
                    quart(q0, le_long, cloison.cote_de_l_ouverture(porte), rayon),
                ));
            }
        }
        battants
    }

    /// Ce qui gêne les objets qui occupent le plancher (évolution 23) : le
    /// battant d'une porte qui les frappe, un mur ou une cloison qu'ils
    /// traversent, un autre objet qu'ils chevauchent. Un symbole (une prise,
    /// une armoire du haut) ne gêne et n'est gêné par rien.
    pub fn conflits(&self, solution: &Solution) -> Vec<Conflit> {
        let mut conflits = Vec::new();
        let contour = self.contour_interieur(solution);
        let battants = self.battants_des_portes(solution);
        let morceaux: Vec<Vec<[(f64, f64); 4]>> = solution
            .cloisons
            .iter()
            .map(|cloison| {
                cloison
                    .as_ref()
                    .map(|c| c.morceaux().into_iter().map(|m| m.coins).collect())
                    .unwrap_or_default()
            })
            .collect();
        let encombrants: Vec<(usize, [(f64, f64); 4])> = self
            .objets
            .iter()
            .zip(&solution.objets)
            .enumerate()
            .filter(|(_, (objet, _))| objet.genre.encombre())
            .map(|(rang, (objet, dimensions))| (rang, objet.coins(*dimensions)))
            .collect();
        for (i, &(rang, coins)) in encombrants.iter().enumerate() {
            for (raison, polygone) in &battants {
                if crate::mobilier::se_chevauchent(polygone, &coins) {
                    conflits.push(Conflit {
                        objet: rang,
                        raison: *raison,
                    });
                }
            }
            let dans_la_piece = crate::mobilier::Encombrement {
                obstacles: Vec::new(),
                piece: contour.as_deref(),
            };
            if contour.is_some() && !dans_la_piece.libre(&coins) {
                conflits.push(Conflit {
                    objet: rang,
                    raison: RaisonDuConflit::Mur,
                });
            }
            for (c, pieces) in morceaux.iter().enumerate() {
                if pieces
                    .iter()
                    .any(|piece| crate::mobilier::se_chevauchent(piece, &coins))
                {
                    conflits.push(Conflit {
                        objet: rang,
                        raison: RaisonDuConflit::Cloison(c),
                    });
                }
            }
            for &(autre, coins_de_l_autre) in &encombrants[i + 1..] {
                if crate::mobilier::se_chevauchent(&coins, &coins_de_l_autre) {
                    conflits.push(Conflit {
                        objet: rang,
                        raison: RaisonDuConflit::Objet(autre),
                    });
                }
            }
        }
        conflits
    }
}
/// Les pièces dessinées : toujours au moins une, dont celle qui est à l'écran.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Plan {
    pub pieces: Vec<Piece>,
    /// Rang de la pièce à l'écran.
    pub courante: usize,
    /// L'échelle du plan au PDF, et son cartouche (section 25.1, point 36).
    pub mise_en_page: MiseEnPage,
}

/// L'échelle d'un plan au PDF (section 25.1, point 36).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum EchelleDuPlan {
    /// Ajusté à la page, comme avant.
    #[default]
    Ajustee,
    /// 1/4 po = 1 pi (1:48).
    QuartDePouce,
    /// 1/8 po = 1 pi (1:96).
    HuitiemeDePouce,
    UnCinquante,
    UnCent,
}

impl EchelleDuPlan {
    pub const TOUTES: [EchelleDuPlan; 5] = [
        EchelleDuPlan::Ajustee,
        EchelleDuPlan::QuartDePouce,
        EchelleDuPlan::HuitiemeDePouce,
        EchelleDuPlan::UnCinquante,
        EchelleDuPlan::UnCent,
    ];

    /// Les points du PDF (1/72 po) pour un pouce réel; `None` : ajusté à la
    /// page.
    pub fn points_par_pouce(self) -> Option<f64> {
        match self {
            EchelleDuPlan::Ajustee => None,
            EchelleDuPlan::QuartDePouce => Some(72.0 / 48.0),
            EchelleDuPlan::HuitiemeDePouce => Some(72.0 / 96.0),
            EchelleDuPlan::UnCinquante => Some(72.0 / 50.0),
            EchelleDuPlan::UnCent => Some(72.0 / 100.0),
        }
    }

    /// Les repères de l'échelle graphique, dans son unité, le nombre de
    /// pouces de cette unité, et vrai si elle est métrique.
    pub fn reperes(self) -> (&'static [f64], f64, bool) {
        match self {
            EchelleDuPlan::Ajustee | EchelleDuPlan::QuartDePouce => {
                (&[0.0, 1.0, 2.0, 4.0, 8.0], 12.0, false)
            }
            EchelleDuPlan::HuitiemeDePouce => (&[0.0, 2.0, 4.0, 8.0, 16.0], 12.0, false),
            EchelleDuPlan::UnCinquante => (&[0.0, 0.5, 1.0, 2.0, 3.0], PO_PAR_M, true),
            EchelleDuPlan::UnCent => (&[0.0, 1.0, 2.0, 4.0, 6.0], PO_PAR_M, true),
        }
    }
}

/// La mise en page d'un plan au PDF : son échelle, et les champs de son
/// cartouche (la date est celle de l'export).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MiseEnPage {
    pub echelle: EchelleDuPlan,
    pub projet: String,
    pub adresse: String,
    pub dessine_par: String,
}

impl Default for Plan {
    fn default() -> Self {
        Plan {
            pieces: vec![Piece::default()],
            courante: 0,
            mise_en_page: MiseEnPage::default(),
        }
    }
}

/// Nombre maximal de pièces d'un plan.
pub const MAX_PIECES: usize = 30;

impl Plan {
    /// Un plan d'une seule pièce.
    pub fn d_une_piece(dessin: Dessin) -> Plan {
        Plan {
            pieces: vec![Piece {
                nom: String::new(),
                dessin,
            }],
            courante: 0,
            mise_en_page: MiseEnPage::default(),
        }
    }

    /// Remet le plan dans ses règles : au moins une pièce, et une pièce
    /// courante qui existe.
    pub fn normaliser(&mut self) {
        if self.pieces.is_empty() {
            self.pieces.push(Piece::default());
        }
        self.courante = self.courante.min(self.pieces.len() - 1);
        // La porte unique d'une cloison d'avant devient sa première ouverture.
        for piece in &mut self.pieces {
            for cloison in &mut piece.dessin.cloisons {
                cloison.normaliser();
            }
        }
    }

    /// La pièce à l'écran.
    pub fn piece(&mut self) -> &mut Piece {
        self.normaliser();
        &mut self.pieces[self.courante]
    }

    /// Ajoute une pièce vide et la met à l'écran. Retourne faux si le plan est plein.
    pub fn ajouter(&mut self) -> bool {
        if self.pieces.len() >= MAX_PIECES {
            return false;
        }
        self.pieces.push(Piece::default());
        self.courante = self.pieces.len() - 1;
        true
    }

    /// Ajoute une copie de la pièce à l'écran, sous ce nom, juste après elle;
    /// la copie passe à l'écran. Faux si le plan est plein.
    pub fn dupliquer(&mut self, nom: String) -> bool {
        self.normaliser();
        if self.pieces.len() >= MAX_PIECES {
            return false;
        }
        let mut copie = self.pieces[self.courante].clone();
        copie.nom = nom;
        self.pieces.insert(self.courante + 1, copie);
        self.courante += 1;
        true
    }

    /// Supprime la pièce à l'écran; la dernière pièce est seulement vidée.
    pub fn supprimer_la_courante(&mut self) {
        self.normaliser();
        if self.pieces.len() == 1 {
            self.pieces[0] = Piece::default();
        } else {
            self.pieces.remove(self.courante);
        }
        self.normaliser();
    }

    /// Somme des surfaces nettes des pièces complètes (en pouces carrés), et
    /// leur nombre. Une pièce incomplète ne compte pas.
    pub fn total(&self, defaut: Format) -> (f64, usize) {
        self.pieces
            .iter()
            .filter_map(|piece| piece.dessin.resoudre(defaut).aire_nette())
            .fold((0.0, 0), |(somme, nombre), aire| (somme + aire, nombre + 1))
    }
}

/// Ce que l'on sait d'un côté.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mesure {
    /// Inscrite par l'utilisateur (en pouces).
    Lue(f64),
    /// Déduite des autres côtés (en pouces).
    Deduite(f64),
    /// Pas encore inscrite.
    Manquante,
    /// Inscrite, mais illisible ou nulle.
    Invalide,
}

impl Mesure {
    /// La longueur en pouces, si elle est connue.
    pub fn pouces(self) -> Option<f64> {
        match self {
            Mesure::Lue(x) | Mesure::Deduite(x) => Some(x),
            Mesure::Manquante | Mesure::Invalide => None,
        }
    }
}

/// Où en est la forme.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Etat {
    /// Rien n'est tracé.
    Vide,
    /// Le tracé ne revient pas encore à son point de départ.
    Ouverte,
    /// Une mesure est illisible.
    MesureInvalide,
    /// Il manque encore ce nombre de mesures.
    Incomplete(usize),
    /// Les mesures ne ferment pas la forme : l'écart, en pouces.
    NeFermePas(f64),
    /// Les côtés se croisent une fois mis à leurs mesures.
    SeCroise,
    /// La diagonale (0 : du coin 1 au coin 3; 1 : du coin 2 au coin 4) est
    /// trop longue ou trop courte pour les côtés.
    DiagonaleImpossible(usize),
    /// Tout est connu : aire en pouces carrés, périmètre en pouces.
    Complete { aire: f64, perimetre: f64 },
}

/// Résultat de la lecture des mesures.
#[derive(Clone, Debug, PartialEq)]
pub struct Solution {
    /// Une mesure par côté, dans l'ordre.
    pub mesures: Vec<Mesure>,
    pub etat: Etat,
    /// Format de la première mesure inscrite : il sert de modèle aux résultats.
    pub modele: Option<Format>,
    /// Aire de chaque retrait, en pouces carrés : zéro s'il est vide, `None`
    /// s'il est incomplet ou illisible.
    pub retraits: Vec<Option<f64>>,
    /// La hauteur des murs (pouces), si elle est inscrite et lisible.
    pub hauteur: Option<f64>,
    /// Vrai si une hauteur est inscrite mais illisible.
    pub hauteur_invalide: bool,
    /// Aire de chaque ouverture (pouces carrés), comme pour les retraits.
    pub ouvertures: Vec<Option<f64>>,
    /// Où dessiner chaque ouverture : `None` si elle n'est pas placée, si la
    /// longueur de son mur n'est pas connue ou si elle est plus large que lui.
    pub placements: Vec<Option<Placement>>,
    /// La largeur et la profondeur de chaque objet, en pouces (celles de
    /// départ si elles ne sont pas lisibles).
    pub objets: Vec<(f64, f64)>,
    /// Pour chaque objet qui est un escalier : son calcul (contremarches,
    /// giron), d'après sa hauteur à monter, ou celle des murs (point 30).
    pub escaliers: Vec<Option<crate::mobilier::EscalierCalcule>>,
    /// Chaque cloison à ses mesures; `None` si sa longueur est illisible.
    pub cloisons: Vec<Option<crate::cloison::CloisonMesuree>>,
    /// L'épaisseur des murs extérieurs (pouces); zéro : pas de mur dessiné.
    pub epaisseur_des_murs: f64,
    /// L'épaisseur des cloisons qui n'ont pas la leur (pouces).
    pub epaisseur_des_cloisons: f64,
    /// La surface brute, à l'extérieur des murs, quand les mesures sont celles
    /// de l'extérieur (la surface de l'état est alors celle de l'intérieur).
    pub aire_exterieure: Option<f64>,
    /// Les diagonales lues d'une pièce à quatre côtés (point 33).
    pub diagonales: [Mesure; 2],
    /// La vraie forme de la pièce d'après ses quatre côtés et ses diagonales :
    /// ses sommets (pouces), le départ répété à la fin. `None` quand aucune
    /// diagonale ne sert : la forme suit alors les directions de ses côtés.
    pub forme_reelle: Option<Vec<(f64, f64)>>,
    /// Quand les deux diagonales sont mesurées : l'écart (pouces) entre la
    /// diagonale du coin 2 au coin 4 et celle de la forme, que donne l'autre.
    pub ecart_des_diagonales: Option<f64>,
    /// Le plafond à ses mesures, sur la face intérieure des murs, pour une
    /// pièce complète (point 34) : à la hauteur des murs inscrite, sinon à
    /// 8 pi. Un plafond qui n'est pas plat, sans hauteur haute lisible, reste
    /// plat.
    pub plafond: Option<crate::plafond::PlafondMesure>,
    /// La hauteur haute d'un plafond qui n'est pas plat manque, ou est
    /// illisible.
    pub plafond_incomplet: bool,
    pub plafond_invalide: bool,
    /// La surface de chaque mur, sur sa face intérieure, du plancher au
    /// plafond (pouces carrés); vide sans hauteur des murs inscrite, ou avec un
    /// plafond incomplet ou illisible.
    pub surfaces_des_murs: Vec<f64>,
    /// La surface du plafond, côtés des retombées compris (pouces carrés), et
    /// le volume de la pièce, retombées déduites (pouces cubes) : `None` dans
    /// les mêmes cas, ou si une retombée est incomplète.
    pub surface_du_plafond: Option<f64>,
    pub volume: Option<f64>,
    /// Chaque retombée : son volume et la surface de ses côtés; zéro si elle
    /// est vide, `None` si elle est incomplète ou illisible.
    pub retombees: Vec<Option<(f64, f64)>>,
}

impl Solution {
    /// La longueur des cloisons lisibles, en pouces.
    pub fn longueur_des_cloisons(&self) -> f64 {
        self.cloisons.iter().flatten().map(|c| c.longueur()).sum()
    }

    /// La surface des cloisons, leurs deux faces, si la hauteur des murs est
    /// connue (pouces carrés).
    pub fn surface_des_cloisons(&self) -> Option<f64> {
        let hauteur = self.hauteur?;
        // Une porte ou un passage s'enlève des deux faces, sur sa hauteur.
        let hauteur_de_porte = hauteur.min(crate::cloison::HAUTEUR_DE_PORTE);
        let une_face: f64 = self
            .cloisons
            .iter()
            .flatten()
            .map(|c| {
                c.longueur() * hauteur
                    - c.ouvertures
                        .iter()
                        .map(|porte| (porte.fin - porte.debut) * hauteur_de_porte)
                        .sum::<f64>()
            })
            .sum();
        Some(2.0 * une_face)
    }

    /// Somme des retraits, s'ils sont tous lisibles.
    pub fn aire_des_retraits(&self) -> Option<f64> {
        self.retraits.iter().copied().sum()
    }

    /// Somme des portes et des fenêtres, si elles sont toutes lisibles.
    pub fn aire_des_ouvertures(&self) -> Option<f64> {
        self.ouvertures.iter().copied().sum()
    }

    /// La surface des murs : `None` si la pièce n'est pas complète, si la
    /// hauteur manque, si une ouverture est incomplète, ou si les ouvertures
    /// dépassent les murs.
    pub fn murs(&self) -> Option<Murs> {
        if !matches!(self.etat, Etat::Complete { .. }) || self.surfaces_des_murs.is_empty() {
            return None;
        }
        // Chaque mur du plancher au plafond : un mur en pignon monte au faîte.
        let brute: f64 = self.surfaces_des_murs.iter().sum();
        let ouvertures = self.aire_des_ouvertures()?;
        let nette = brute - ouvertures;
        (nette > 0.0).then_some(Murs {
            brute,
            ouvertures,
            nette,
        })
    }

    /// Surface de la pièce moins les retraits; `None` si la forme n'est pas
    /// complète, si un retrait est incomplet ou si les retraits la dépassent.
    pub fn aire_nette(&self) -> Option<f64> {
        let Etat::Complete { aire, .. } = self.etat else {
            return None;
        };
        let nette = aire - self.aire_des_retraits()?;
        (nette > 0.0).then_some(nette)
    }
}

/// Rectangle de même aire et de même périmètre qu'une pièce : sa longueur, sa
/// largeur, et vrai s'il existe. Les outils qui ne dépendent que de l'aire et
/// du périmètre (dalle, plancher, peinture) donnent alors le même résultat
/// que pour la pièce. S'il n'existe pas, c'est le carré de même aire.
pub fn rectangle_equivalent(aire: f64, perimetre: f64) -> (f64, f64, bool) {
    let demi = perimetre / 2.0;
    let discriminant = demi * demi - 4.0 * aire;
    if discriminant >= -1e-9 {
        let racine = discriminant.max(0.0).sqrt();
        ((demi + racine) / 2.0, (demi - racine) / 2.0, true)
    } else {
        (aire.sqrt(), aire.sqrt(), false)
    }
}

/// Une longueur en pouces, écrite comme elle se tape dans un champ : en pieds
/// décimaux ou en mètres, avec le séparateur décimal donné.
pub fn longueur_a_taper(pouces: f64, metrique: bool, decimal: char) -> String {
    let (valeur, format) = if metrique {
        (pouces / PO_PAR_M, Format::M)
    } else {
        (pouces / 12.0, Format::PiDec)
    };
    let nombre = format!("{valeur:.4}");
    let nombre = nombre.trim_end_matches('0').trim_end_matches('.');
    format!(
        "{} {}",
        nombre.replace('.', &decimal.to_string()),
        symbole(format)
    )
}

/// Les outils par métier qui partent d'une longueur et d'une largeur au sol :
/// l'outil, le rang de ces deux champs, et celui de la hauteur s'il en a une
/// (la peinture d'une pièce, par exemple). Un champ se reconnaît à son
/// libellé court (« Longueur ») ou, s'il n'en a pas, à son libellé.
pub fn outils_d_une_piece() -> Vec<(&'static Calculateur, usize, usize, Option<usize>)> {
    let rang = |c: &Calculateur, nom: &str| {
        c.champs.iter().position(|champ| {
            let libelle = if champ.court.est_vide() {
                champ.libelle.fr
            } else {
                champ.court.fr
            };
            libelle == nom && matches!(champ.nature, Nature::Longueur(_))
        })
    };
    Metier::TOUS
        .iter()
        .filter(|metier| **metier != Metier::Acier)
        .flat_map(|metier| metier.calculateurs())
        .filter_map(|c| {
            Some((
                c,
                rang(c, "Longueur")?,
                rang(c, "Largeur")?,
                rang(c, "Hauteur"),
            ))
        })
        .collect()
}

/// Redresse un tracé à main levée : une suite de déplacements horizontaux,
/// verticaux ou à 45°, chacun avec sa longueur signée le long de son axe
/// (positive dans le sens positif de l'axe). Les tremblements sont écartés.
pub fn redresser(points: &[(f64, f64)]) -> Vec<(Axe, f64)> {
    if points.len() < 2 {
        return Vec::new();
    }
    let (mut x0, mut x1, mut y0, mut y1) = (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
    for &(x, y) in points {
        x0 = x0.min(x);
        x1 = x1.max(x);
        y0 = y0.min(y);
        y1 = y1.max(y);
    }
    let diagonale = (x1 - x0).hypot(y1 - y0);
    if !diagonale.is_finite() || diagonale <= 0.0 {
        return Vec::new();
    }
    let sommets = simplifier(points, diagonale * FINESSE);
    let mut pas: Vec<(Axe, f64)> = Vec::new();
    for paire in sommets.windows(2) {
        let (dx, dy) = (paire[1].0 - paire[0].0, paire[1].1 - paire[0].1);
        let axe = Axe::du_deplacement(dx, dy);
        let (ux, uy) = axe.vecteur();
        pas.push((axe, dx * ux + dy * uy));
    }
    let pas = fusionner(pas);
    let plus_long = pas.iter().map(|(_, l)| l.abs()).fold(0.0, f64::max);
    let utiles = pas
        .into_iter()
        .filter(|(_, l)| l.abs() >= plus_long * PART_NEGLIGEABLE)
        .collect();
    fusionner(utiles)
}

/// Réunit les déplacements consécutifs d'un même axe.
fn fusionner(pas: Vec<(Axe, f64)>) -> Vec<(Axe, f64)> {
    let mut fusion: Vec<(Axe, f64)> = Vec::new();
    for (axe, longueur) in pas {
        match fusion.last_mut() {
            Some((a, l)) if *a == axe => *l += longueur,
            _ => fusion.push((axe, longueur)),
        }
    }
    fusion.retain(|(_, l)| l.abs() > 0.0);
    fusion
}

/// Simplification d'une ligne brisée (Douglas-Peucker).
fn simplifier(points: &[(f64, f64)], seuil: f64) -> Vec<(f64, f64)> {
    let dernier = points.len() - 1;
    let mut garde = vec![false; points.len()];
    garde[0] = true;
    garde[dernier] = true;
    let mut pile = vec![(0usize, dernier)];
    while let Some((a, b)) = pile.pop() {
        if b <= a + 1 {
            continue;
        }
        let (ax, ay) = points[a];
        let (bx, by) = points[b];
        let longueur = (bx - ax).hypot(by - ay);
        let mut pire = (0.0, a);
        for (i, &(x, y)) in points.iter().enumerate().take(b).skip(a + 1) {
            let ecart = if longueur > 0.0 {
                ((bx - ax) * (ay - y) - (ax - x) * (by - ay)).abs() / longueur
            } else {
                (x - ax).hypot(y - ay)
            };
            if ecart > pire.0 {
                pire = (ecart, i);
            }
        }
        if pire.0 > seuil {
            garde[pire.1] = true;
            pile.push((a, pire.1));
            pile.push((pire.1, b));
        }
    }
    points
        .iter()
        .zip(garde)
        .filter_map(|(p, g)| g.then_some(*p))
        .collect()
}

impl Dessin {
    /// Déduit le côté inconnu d'un axe, s'il est seul : `le_long` est sa
    /// longueur signée le long de l'axe. Plusieurs côtés inconnus sur un même
    /// axe se partagent cette longueur : il manque alors des mesures.
    fn deduire_sur_l_axe(
        &self,
        mesures: &mut [Mesure],
        rangs: &[usize],
        le_long: f64,
        ecart: &mut f64,
        manquantes: &mut usize,
    ) {
        match rangs {
            [seul] => {
                let longueur = le_long * self.cotes[*seul].sens.signe();
                if longueur > TOLERANCE_PO {
                    mesures[*seul] = Mesure::Deduite(longueur);
                } else {
                    *ecart = ecart.max(TOLERANCE_PO.max(-longueur));
                }
            }
            plusieurs => *manquantes += plusieurs.len().saturating_sub(1),
        }
    }

    /// Vrai si rien n'est tracé.
    pub fn est_vide(&self) -> bool {
        self.cotes.is_empty()
    }

    /// Efface tout.
    pub fn effacer(&mut self) {
        *self = Dessin::default();
    }

    /// Longueur de croquis d'un côté ajouté aux flèches : la moyenne des côtés
    /// déjà tracés, ou 100 pour le premier.
    fn pas_des_fleches(&self) -> f64 {
        if self.cotes.is_empty() {
            100.0
        } else {
            self.cotes.iter().map(|c| c.croquis).sum::<f64>() / self.cotes.len() as f64
        }
    }

    /// Ajoute un côté dans une direction, comme avec les flèches de la planche
    /// (section 25). Une flèche perpendiculaire au dernier côté en ajoute un;
    /// dans l'axe du dernier côté, elle l'allonge ou le raccourcit d'un pas,
    /// sans jamais le faire disparaître. La forme se ferme d'elle-même si elle
    /// revient à son départ. Retourne vrai si le dessin a changé.
    pub fn ajouter_cote(&mut self, sens: Sens) -> bool {
        if self.fermee {
            return false;
        }
        let pas = self.pas_des_fleches();
        match self.cotes.last_mut() {
            Some(dernier) if dernier.sens == sens => dernier.croquis += pas,
            Some(dernier) if dernier.sens.axe() == sens.axe() => {
                let raccourci = (dernier.croquis - pas).max(pas * 0.25);
                if (raccourci - dernier.croquis).abs() < 1e-9 {
                    return false;
                }
                dernier.croquis = raccourci;
            }
            _ => self.cotes.push(Cote {
                sens,
                croquis: pas,
                saisie: String::new(),
            }),
        }
        if self.cotes.len() >= 3 {
            let (dx, dy) = self.fin();
            if dx.hypot(dy) <= self.etendue() * PROXIMITE_DE_FERMETURE {
                self.fermer();
            }
        }
        true
    }

    /// Ajoute un tracé à main levée à la suite de la forme. Si le tracé
    /// revient près du point de départ, la forme est fermée. Retourne vrai si
    /// quelque chose a été ajouté.
    pub fn ajouter_trace(&mut self, points: &[(f64, f64)]) -> bool {
        if self.fermee {
            return false;
        }
        let pas = redresser(points);
        if pas.is_empty() {
            return false;
        }
        for (axe, longueur) in pas {
            self.ajouter_pas(axe, longueur);
        }
        if self.cotes.len() >= 3 {
            let (dx, dy) = self.fin();
            if dx.hypot(dy) <= self.etendue() * PROXIMITE_DE_FERMETURE {
                self.fermer();
            }
        }
        true
    }

    fn ajouter_pas(&mut self, axe: Axe, longueur: f64) {
        if let Some(dernier) = self.cotes.last_mut()
            && dernier.sens.axe() == axe
        {
            let somme = dernier.signe() + longueur;
            if somme.abs() > 0.0 {
                dernier.sens = Sens::selon(axe, somme);
                dernier.croquis = somme.abs();
            } else {
                self.cotes.pop();
            }
            return;
        }
        self.cotes.push(Cote {
            sens: Sens::selon(axe, longueur),
            croquis: longueur.abs(),
            saisie: String::new(),
        });
    }

    /// Position de la fin du tracé par rapport au départ, sur le croquis.
    fn fin(&self) -> (f64, f64) {
        self.cotes.iter().fold((0.0, 0.0), |(x, y), c| {
            let (px, py) = c.sens.pas();
            (x + px * c.croquis, y + py * c.croquis)
        })
    }

    /// Plus grande dimension du croquis.
    fn etendue(&self) -> f64 {
        let (mut x0, mut x1, mut y0, mut y1) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
        for (x, y) in self.sommets_du_croquis() {
            x0 = x0.min(x);
            x1 = x1.max(x);
            y0 = y0.min(y);
            y1 = y1.max(y);
        }
        (x1 - x0).max(y1 - y0)
    }

    /// Ferme la forme : ramène le tracé au point de départ, en ajoutant au
    /// besoin un ou deux côtés. Retourne faux si la forme ne peut pas être
    /// fermée (moins de deux côtés, ou côtés qui s'annulent).
    pub fn fermer(&mut self) -> bool {
        if self.fermee {
            return true;
        }
        if self.cotes.len() < 2 {
            return false;
        }
        let avant = self.clone();
        let (dx, dy) = self.fin();
        let seuil = self.etendue() * PART_NEGLIGEABLE;
        // Le retour au départ. La fermeture n'invente pas de côté en biais : un
        // tracé d'équerre un peu de travers ne doit pas recevoir de coin coupé.
        // Si le dernier côté est en biais et que le départ est dans son
        // prolongement, il s'allonge jusqu'au départ; sinon, le retour se fait
        // d'équerre, d'abord dans l'axe que le dernier côté n'a pas.
        let retour = Axe::du_deplacement(-dx, -dy);
        let dernier_axe = self.cotes.last().map(|c| c.sens.axe());
        if matches!(retour, Axe::Descendant | Axe::Montant) && dernier_axe == Some(retour) {
            let (ux, uy) = retour.vecteur();
            let longueur = -dx * ux - dy * uy;
            if longueur.abs() > seuil {
                self.ajouter_pas(retour, longueur);
            }
        } else {
            let dernier_horizontal = self.cotes.last().is_some_and(|c| c.sens.horizontal());
            let retours = if dernier_horizontal {
                [(Axe::Vertical, -dy), (Axe::Horizontal, -dx)]
            } else {
                [(Axe::Horizontal, -dx), (Axe::Vertical, -dy)]
            };
            for (axe, longueur) in retours {
                if longueur.abs() > seuil {
                    self.ajouter_pas(axe, longueur);
                }
            }
        }
        // Le dernier côté et le premier se suivent : s'ils ont le même axe, ils
        // n'en font qu'un.
        while self.cotes.len() > 2
            && self.cotes[0].sens.axe() == self.cotes[self.cotes.len() - 1].sens.axe()
        {
            let Some(dernier) = self.cotes.pop() else {
                break;
            };
            let somme = self.cotes[0].signe() + dernier.signe();
            if somme.abs() > 0.0 {
                self.cotes[0].sens = Sens::selon(self.cotes[0].sens.axe(), somme);
                self.cotes[0].croquis = somme.abs();
                if self.cotes[0].saisie.is_empty() {
                    self.cotes[0].saisie = dernier.saisie;
                }
            } else {
                self.cotes.remove(0);
            }
        }
        // Le petit écart qui reste est repris par les deux derniers côtés.
        if self.cotes.len() < 3 || !self.reprendre_l_ecart() {
            *self = avant;
            return false;
        }
        self.fermee = true;
        true
    }

    /// Reprend le petit écart entre la fin du croquis et son départ sur les
    /// deux derniers côtés (ils n'ont jamais le même axe), sans changer leur
    /// sens. Retourne faux si c'est impossible.
    fn reprendre_l_ecart(&mut self) -> bool {
        let n = self.cotes.len();
        n >= 2 && self.reprendre_l_ecart_sur(n - 2, n - 1, 0.0)
    }

    /// Reprend l'écart du croquis sur les côtés `i` et `j`, qui gardent leur
    /// sens et au moins la longueur `minimum`; ne change rien si c'est
    /// impossible.
    fn reprendre_l_ecart_sur(&mut self, i: usize, j: usize, minimum: f64) -> bool {
        let (dx, dy) = self.fin();
        let u = self.cotes[i].sens.axe().vecteur();
        let v = self.cotes[j].sens.axe().vecteur();
        let determinant = u.0 * v.1 - u.1 * v.0;
        if determinant.abs() < SINUS_MINIMAL {
            return false;
        }
        let (wx, wy) = (-dx, -dy);
        let mut nouvelles = [0.0; 2];
        for (k, (rang, correction)) in [
            (i, (wx * v.1 - wy * v.0) / determinant),
            (j, (u.0 * wy - u.1 * wx) / determinant),
        ]
        .into_iter()
        .enumerate()
        {
            let avant = self.cotes[rang].signe();
            let apres = avant + correction;
            if apres.abs() <= minimum || (apres > 0.0) != (avant > 0.0) {
                return false;
            }
            nouvelles[k] = apres.abs();
        }
        self.cotes[i].croquis = nouvelles[0];
        self.cotes[j].croquis = nouvelles[1];
        true
    }

    /// Le côté qui précède le côté `rang` : le dernier pour le premier côté
    /// d'une forme fermée; aucun pour celui d'une forme ouverte.
    fn precedent(&self, rang: usize) -> Option<usize> {
        let n = self.cotes.len();
        if rang >= n || n < 2 {
            None
        } else if rang > 0 {
            Some(rang - 1)
        } else if self.fermee {
            Some(n - 1)
        } else {
            None
        }
    }

    /// L'angle, en degrés, dont le côté `rang` tourne par rapport au côté
    /// précédent, à gauche ou à droite : 90 d'équerre, 45 en biais, 0 en
    /// ligne droite (section 25.1, point 32). `None` pour le premier côté d'une forme
    /// ouverte.
    pub fn virage(&self, rang: usize) -> Option<f64> {
        let precedent = self.precedent(rang)?;
        Some(
            self.cotes[rang]
                .sens
                .virage_depuis(self.cotes[precedent].sens)
                .abs(),
        )
    }

    /// Tourne le côté `rang` pour qu'il fasse l'angle `virage` (en degrés,
    /// plus de 0 et moins de 180) avec le côté précédent, du même côté qu'il
    /// tournait. Les autres côtés gardent leur direction; une forme fermée se
    /// referme sur le croquis par deux autres côtés, et ses mesures déduites
    /// se déduisent à nouveau. Retourne faux si rien n'a changé ou si c'est
    /// impossible.
    pub fn orienter(&mut self, rang: usize, virage: f64) -> bool {
        if !(virage > 0.0 && virage < 180.0) {
            return false;
        }
        let Some(precedent) = self.precedent(rang) else {
            return false;
        };
        let avant = self.cotes[precedent].sens;
        let actuel = self.cotes[rang].sens.virage_depuis(avant);
        let signe = if actuel.abs() > 1e-9 && actuel.abs() < 180.0 - 1e-9 {
            actuel.signum()
        } else {
            self.orientation()
        };
        let sens = Sens::d_angle(avant.angle() + signe * virage);
        if sens == self.cotes[rang].sens {
            return false;
        }
        let ancien = self.clone();
        self.cotes[rang].sens = sens;
        if self.fermee && !self.refermer_le_croquis(rang) {
            *self = ancien;
            return false;
        }
        true
    }

    /// Le sens de rotation du croquis : 1 s'il tourne vers la droite de qui
    /// avance (l'aire signée est positive), −1 sinon.
    fn orientation(&self) -> f64 {
        let sommets = self.sommets_du_croquis();
        let aire: f64 = sommets
            .windows(2)
            .map(|s| s[0].0 * s[1].1 - s[1].0 * s[0].1)
            .sum();
        if aire < 0.0 { -1.0 } else { 1.0 }
    }

    /// Ramène le croquis d'une forme fermée à son départ après que le côté
    /// `tourne` a changé de direction : deux autres côtés reprennent l'écart,
    /// les plus proches après lui d'abord, sans devenir minuscules.
    fn refermer_le_croquis(&mut self, tourne: usize) -> bool {
        let n = self.cotes.len();
        let minimum = self.etendue() * 0.02;
        let autres: Vec<usize> = (1..n).map(|k| (tourne + k) % n).collect();
        for (a, &i) in autres.iter().enumerate() {
            for &j in &autres[a + 1..] {
                if self.reprendre_l_ecart_sur(i, j, minimum) {
                    return true;
                }
            }
        }
        false
    }

    /// Retire le dernier côté tracé; rouvre d'abord une forme fermée.
    /// La pièce retournée comme dans un miroir, la gauche et la droite
    /// échangées : ses côtés, ses notes, ses cloisons et leurs portes, et ses
    /// objets, retournés eux aussi (un escalier tournant tourne de l'autre
    /// côté). Ses mesures, ses ouvertures (placées depuis le début de leur
    /// côté) et ses retraits restent les mêmes; deux miroirs la redonnent.
    pub fn miroir(&mut self) {
        for cote in &mut self.cotes {
            cote.sens = cote.sens.en_miroir();
        }
        for objet in &mut self.objets {
            objet.centre.0 = -objet.centre.0;
            let angle = (360.0 - objet.angle_total()).rem_euclid(360.0);
            objet.rotation = 0;
            objet.angle = angle;
            objet.retourne = !objet.retourne;
        }
        for note in &mut self.notes {
            note.position.0 = -note.position.0;
        }
        for trait_ in &mut self.croquis {
            for point in &mut trait_.points {
                point.0 = -point.0;
            }
        }
        for nom in &mut self.noms_des_espaces {
            nom.point.0 = -nom.point.0;
        }
        for cloison in &mut self.cloisons {
            cloison.debut.0 = -cloison.debut.0;
            cloison.sens = cloison.sens.en_miroir();
            for segment in &mut cloison.suite {
                segment.sens = segment.sens.en_miroir();
            }
            // Ses portes s'ouvrent du même côté de la pièce, donc de l'autre
            // côté de leur segment, dont le sens est retourné.
            for porte in cloison.toutes_les_ouvertures_mut() {
                porte.inversee = !porte.inversee;
            }
        }
    }

    pub fn retirer_le_dernier(&mut self) {
        self.fermee = false;
        self.cotes.pop();
    }

    /// Sommets du croquis, le départ en (0, 0). Une forme fermée revient au départ.
    pub fn sommets_du_croquis(&self) -> Vec<(f64, f64)> {
        let longueurs: Vec<f64> = self.cotes.iter().map(|c| c.croquis).collect();
        self.sommets(&longueurs)
    }

    fn sommets(&self, longueurs: &[f64]) -> Vec<(f64, f64)> {
        let mut sommets = vec![(0.0, 0.0)];
        let (mut x, mut y) = (0.0, 0.0);
        for (cote, longueur) in self.cotes.iter().zip(longueurs) {
            let (px, py) = cote.sens.pas();
            x += px * longueur;
            y += py * longueur;
            sommets.push((x, y));
        }
        sommets
    }

    /// Sommets à l'échelle des mesures (en pouces), quand toutes sont connues.
    pub fn sommets_mesures(&self, solution: &Solution) -> Option<Vec<(f64, f64)>> {
        if let Some(forme) = &solution.forme_reelle {
            return Some(forme.clone());
        }
        let longueurs: Option<Vec<f64>> = solution.mesures.iter().map(|m| m.pouces()).collect();
        let longueurs = longueurs?;
        (longueurs.len() == self.cotes.len()).then(|| self.sommets(&longueurs))
    }

    /// Les coins de l'extérieur des murs, un par sommet (le contour fermé) :
    /// les mesures elles-mêmes si elles sont prises à l'extérieur, sinon les
    /// côtés repoussés vers l'extérieur de l'épaisseur des murs. `None` tant
    /// que la forme n'est pas complète.
    pub fn contour_exterieur(&self, solution: &Solution) -> Option<Vec<(f64, f64)>> {
        if !matches!(solution.etat, Etat::Complete { .. }) {
            return None;
        }
        let sommets = self.sommets_mesures(solution)?;
        if self.mesures_exterieures {
            Some(sommets)
        } else {
            decaler(&sommets, solution.epaisseur_des_murs, true)
        }
    }

    /// Les coins de l'intérieur des murs : les mesures elles-mêmes, ou, si
    /// elles sont prises à l'extérieur, les côtés repoussés vers l'intérieur
    /// de l'épaisseur des murs. `None` tant que la forme n'est pas complète.
    pub fn contour_interieur(&self, solution: &Solution) -> Option<Vec<(f64, f64)>> {
        if !matches!(solution.etat, Etat::Complete { .. }) {
            return None;
        }
        let sommets = self.sommets_mesures(solution)?;
        if self.mesures_exterieures {
            decaler(&sommets, solution.epaisseur_des_murs, false)
        } else {
            Some(sommets)
        }
    }

    /// Les murs extérieurs à leur épaisseur, d'un côté ou de l'autre des
    /// mesures : leurs morceaux pleins, interrompus par les portes et les
    /// fenêtres. Vide tant que la forme n'est pas complète, ou si l'épaisseur
    /// des murs est nulle.
    pub fn murs_exterieurs(&self, solution: &Solution) -> Vec<MorceauDeMur> {
        let e = solution.epaisseur_des_murs;
        let decale = if self.mesures_exterieures {
            self.contour_interieur(solution)
        } else {
            self.contour_exterieur(solution)
        };
        let (Some(sommets), Some(decale)) = (self.sommets_mesures(solution), decale) else {
            return Vec::new();
        };
        if e <= 1e-9 {
            return Vec::new();
        }
        let mut morceaux = Vec::new();
        for (cote, repere) in reperes_des_cotes(&sommets).into_iter().enumerate() {
            let (a, b) = (sommets[cote], sommets[cote + 1]);
            let longueur = (b.0 - a.0).hypot(b.1 - a.1);
            let Repere { u, dehors } = repere;
            // Le mur est du côté de l'extérieur des mesures, ou de l'intérieur.
            let vers_le_mur = if self.mesures_exterieures {
                (-dehors.0, -dehors.1)
            } else {
                dehors
            };
            // Un point de la ligne mesurée, ou de l'autre face du mur (ses coins
            // sont ceux du contour repoussé).
            let face = |t: f64, l_autre: bool| {
                if !l_autre {
                    (a.0 + u.0 * t, a.1 + u.1 * t)
                } else if t <= 1e-9 {
                    decale[cote]
                } else if t >= longueur - 1e-9 {
                    decale[cote + 1]
                } else {
                    (
                        a.0 + u.0 * t + vers_le_mur.0 * e,
                        a.1 + u.1 * t + vers_le_mur.1 * e,
                    )
                }
            };
            let interieur = |t: f64| face(t, self.mesures_exterieures);
            let exterieur = |t: f64| face(t, !self.mesures_exterieures);
            let mut vides: Vec<(f64, f64)> = solution
                .placements
                .iter()
                .flatten()
                .filter(|placement| placement.cote == cote)
                .map(|placement| {
                    (
                        placement.debut.clamp(0.0, longueur),
                        placement.fin.clamp(0.0, longueur),
                    )
                })
                .collect();
            vides.sort_by(|x, y| x.0.total_cmp(&y.0));
            let mut depuis = 0.0_f64;
            for (t0, t1) in vides.into_iter().chain([(longueur, longueur)]) {
                if t0 > depuis + 1e-6 {
                    morceaux.push(MorceauDeMur {
                        cote,
                        coins: [
                            interieur(depuis),
                            interieur(t0),
                            exterieur(t0),
                            exterieur(depuis),
                        ],
                        jambage_au_debut: depuis > 1e-9,
                        jambage_a_la_fin: t0 < longueur - 1e-9,
                    });
                }
                depuis = depuis.max(t1);
            }
        }
        morceaux
    }

    /// Déplace le côté `cote` de `delta` pouces vers l'extérieur de la pièce
    /// (négatif : vers l'intérieur), parallèlement à lui-même : ses deux
    /// voisins s'allongent ou raccourcissent le long de leur direction. Les
    /// mesures inscrites suivent (`ecrire` les écrit); les déduites se
    /// déduisent encore. Les portes et les fenêtres du voisin d'après (leur
    /// coin A recule), les objets, les notes et les cloisons gardent leur
    /// place. `None` si le geste est impossible : un voisin parallèle au côté,
    /// ou une longueur qui deviendrait plus courte qu'un pouce.
    pub fn etirer(
        &self,
        cote: usize,
        delta: f64,
        solution: &Solution,
        ecrire: &dyn Fn(f64) -> String,
    ) -> Option<Dessin> {
        // Une forme tirée de ses diagonales ne s'étire pas : ses côtés et ses
        // diagonales ne s'accorderaient plus.
        if !matches!(solution.etat, Etat::Complete { .. }) || solution.forme_reelle.is_some() {
            return None;
        }
        let n = self.cotes.len();
        if n < 3 || cote >= n {
            return None;
        }
        let longueurs: Vec<f64> = solution
            .mesures
            .iter()
            .map(|m| m.pouces())
            .collect::<Option<_>>()?;
        let sommets = self.sommets(&longueurs);
        let reperes = reperes_des_cotes(&sommets);
        let (avant, apres) = ((cote + n - 1) % n, (cote + 1) % n);
        let scalaire = |a: (f64, f64), b: (f64, f64)| a.0 * b.0 + a.1 * b.1;
        let dehors = reperes[cote].dehors;
        let (c_avant, c_apres) = (
            scalaire(reperes[avant].u, dehors),
            scalaire(reperes[apres].u, dehors),
        );
        if c_avant.abs() < 0.2 || c_apres.abs() < 0.2 {
            return None;
        }
        // Le début du côté glisse le long du voisin d'avant, sa fin le long du
        // voisin d'après.
        let (glisse_avant, glisse_apres) = (delta / c_avant, delta / c_apres);
        let mut nouvelles = longueurs.clone();
        nouvelles[avant] += glisse_avant;
        nouvelles[apres] -= glisse_apres;
        nouvelles[cote] += glisse_apres * scalaire(reperes[apres].u, reperes[cote].u)
            - glisse_avant * scalaire(reperes[avant].u, reperes[cote].u);
        if nouvelles.iter().any(|longueur| *longueur < 1.0) {
            return None;
        }
        let mut etire = self.clone();
        for k in [avant, cote, apres] {
            if (nouvelles[k] - longueurs[k]).abs() > 1e-9 && !self.cotes[k].saisie.trim().is_empty()
            {
                etire.cotes[k].saisie = ecrire(nouvelles[k]);
            }
        }
        // Les ouvertures du voisin d'après restent en place : son coin A a
        // reculé d'autant que le voisin s'est allongé.
        let recul = nouvelles[apres] - longueurs[apres];
        for (ouverture, placement) in etire.ouvertures.iter_mut().zip(&solution.placements) {
            if ouverture.cote == Some(apres)
                && !ouverture.position.trim().is_empty()
                && let Some(placement) = placement
            {
                ouverture.position = ecrire((placement.debut + recul).max(0.0));
            }
        }
        // Un sommet qui ne bouge pas : ce qui est posé sur le plan le suit.
        let nouveaux = self.sommets(&nouvelles);
        let fixe = (cote + 2) % n;
        let (dx, dy) = (
            nouveaux[fixe].0 - sommets[fixe].0,
            nouveaux[fixe].1 - sommets[fixe].1,
        );
        if dx.abs() > 1e-12 || dy.abs() > 1e-12 {
            for objet in &mut etire.objets {
                objet.centre = (objet.centre.0 + dx, objet.centre.1 + dy);
            }
            for note in &mut etire.notes {
                note.position = (note.position.0 + dx, note.position.1 + dy);
            }
            for cloison in &mut etire.cloisons {
                cloison.debut = (cloison.debut.0 + dx, cloison.debut.1 + dy);
            }
        }
        Some(etire)
    }

    /// La pièce dont la cloison `rang` a son segment `segment` déplacé de
    /// `delta` pouces parallèlement à lui-même (`Cloison::etirer`, section
    /// 25.1, point 38). `None` si le geste est impossible.
    pub fn etirer_une_cloison(
        &self,
        rang: usize,
        segment: usize,
        delta: f64,
        solution: &Solution,
        ecrire: &dyn Fn(f64) -> String,
    ) -> Option<Dessin> {
        let mesuree = solution.cloisons.get(rang)?.as_ref()?;
        let etiree = self
            .cloisons
            .get(rang)?
            .etirer(segment, delta, mesuree, ecrire)?;
        let mut dessin = self.clone();
        dessin.cloisons[rang] = etiree;
        Some(dessin)
    }

    /// Les chaînes de cotes d'une pièce complète (section 25.1, point 35) :
    /// le long de chaque mur, ses coins, les bords de ses portes et de ses
    /// fenêtres et les deux faces des cloisons qui le touchent (un mur sans
    /// rien de tel n'en a pas : sa mesure suffit); entre deux segments de
    /// cloison parallèles qui se font face, leur écart, de face à face.
    pub fn chaines_de_cotes(&self, solution: &Solution) -> Vec<ChaineDeCotes> {
        if !matches!(solution.etat, Etat::Complete { .. }) {
            return Vec::new();
        }
        let Some(sommets) = self.sommets_mesures(solution) else {
            return Vec::new();
        };
        let mut chaines = Vec::new();
        // Une cloison touche un mur par un de ses bouts, sur la face
        // intérieure du mur : à l'épaisseur du mur près de la ligne mesurée.
        let portee = solution.epaisseur_des_murs.max(0.0) + 2.0;
        for (cote, (mur, repere)) in sommets
            .windows(2)
            .zip(reperes_des_cotes(&sommets))
            .enumerate()
        {
            let (a, b) = (mur[0], mur[1]);
            let longueur = (b.0 - a.0).hypot(b.1 - a.1);
            if longueur <= 1e-6 {
                continue;
            }
            let u = repere.u;
            let mut points = vec![0.0, longueur];
            for placement in solution.placements.iter().flatten() {
                if placement.cote == cote {
                    points.push(placement.debut.clamp(0.0, longueur));
                    points.push(placement.fin.clamp(0.0, longueur));
                }
            }
            for cloison in solution.cloisons.iter().flatten() {
                let n = cloison.sommets.len();
                if n < 2 {
                    continue;
                }
                for (bout, voisin) in [
                    (cloison.sommets[0], cloison.sommets[1]),
                    (cloison.sommets[n - 1], cloison.sommets[n - 2]),
                ] {
                    let (dx, dy) = (bout.0 - a.0, bout.1 - a.1);
                    let le_long = dx * u.0 + dy * u.1;
                    let de_travers = (dx * u.1 - dy * u.0).abs();
                    if de_travers > portee || le_long < -0.5 || le_long > longueur + 0.5 {
                        continue;
                    }
                    let (vx, vy) = (voisin.0 - bout.0, voisin.1 - bout.1);
                    let l = vx.hypot(vy);
                    if l <= 1e-9 {
                        continue;
                    }
                    // Une cloison de biais occupe plus de place le long du mur.
                    let sinus = ((u.0 * vy - u.1 * vx) / l).abs();
                    if sinus < 0.2 {
                        continue;
                    }
                    let demi = cloison.epaisseur / 2.0 / sinus;
                    points.push((le_long - demi).clamp(0.0, longueur));
                    points.push((le_long + demi).clamp(0.0, longueur));
                }
            }
            points.sort_by(f64::total_cmp);
            points.dedup_by(|x, y| (*x - *y).abs() < TOLERANCE_PO);
            if points.len() > 2 {
                chaines.push(ChaineDeCotes {
                    debut: a,
                    fin: b,
                    dehors: repere.dehors,
                    points,
                    mur: Some(cote),
                });
            }
        }
        // Entre deux segments de cloison parallèles qui se font face.
        let segments: Vec<SegmentACoter> = solution
            .cloisons
            .iter()
            .enumerate()
            .filter_map(|(rang, c)| c.as_ref().map(|c| (rang, c)))
            .flat_map(|(rang, c)| {
                c.sommets
                    .windows(2)
                    .enumerate()
                    .map(move |(k, s)| (rang, k, s[0], s[1], c.epaisseur))
            })
            .collect();
        let unitaire = |p: (f64, f64), q: (f64, f64)| {
            let l = (q.0 - p.0).hypot(q.1 - p.1);
            (l > 1e-9).then(|| ((q.0 - p.0) / l, (q.1 - p.1) / l, l))
        };
        // L'écart signé de `q` à la droite de `p` dans la direction `d`, et sa
        // place le long de cette droite.
        let ecart_de = |p: (f64, f64), d: (f64, f64), q: (f64, f64)| {
            (
                d.0 * (q.1 - p.1) - d.1 * (q.0 - p.0),
                d.0 * (q.0 - p.0) + d.1 * (q.1 - p.1),
            )
        };
        for (i, &(rang_1, k_1, p0, p1, e1)) in segments.iter().enumerate() {
            let Some((dx, dy, l1)) = unitaire(p0, p1) else {
                continue;
            };
            for &(rang_2, k_2, q0, q1, e2) in &segments[i + 1..] {
                if rang_1 == rang_2 && k_1.abs_diff(k_2) < 2 {
                    continue;
                }
                let Some((ex, ey, _)) = unitaire(q0, q1) else {
                    continue;
                };
                if (dx * ey - dy * ex).abs() > SINUS_MINIMAL {
                    continue;
                }
                let ((ecart, t0), (_, t1)) =
                    (ecart_de(p0, (dx, dy), q0), ecart_de(p0, (dx, dy), q1));
                let (debut, fin) = (t0.min(t1).max(0.0), t0.max(t1).min(l1));
                if ecart.abs() < 1.0 || fin - debut < 6.0 {
                    continue;
                }
                let milieu = (debut + fin) / 2.0;
                // Un autre segment parallèle entre les deux : il se cote avec
                // chacun, pas par-dessus.
                let entre = segments.iter().any(|&(r, k, s0, s1, _)| {
                    if (r, k) == (rang_1, k_1) || (r, k) == (rang_2, k_2) {
                        return false;
                    }
                    let Some((sx, sy, _)) = unitaire(s0, s1) else {
                        return false;
                    };
                    let ((e, a), (_, b)) = (ecart_de(p0, (dx, dy), s0), ecart_de(p0, (dx, dy), s1));
                    (dx * sy - dy * sx).abs() <= SINUS_MINIMAL
                        && e * ecart.signum() > 0.0
                        && e.abs() < ecart.abs()
                        && a.min(b) <= milieu
                        && milieu <= a.max(b)
                });
                let jour = ecart.abs() - (e1 + e2) / 2.0;
                if entre || jour <= 0.5 {
                    continue;
                }
                let n = (-dy * ecart.signum(), dx * ecart.signum());
                let base = (p0.0 + dx * milieu, p0.1 + dy * milieu);
                let face = |d: f64| (base.0 + n.0 * d, base.1 + n.1 * d);
                chaines.push(ChaineDeCotes {
                    debut: face(e1 / 2.0),
                    fin: face(ecart.abs() - e2 / 2.0),
                    dehors: (0.0, 0.0),
                    points: vec![0.0, jour],
                    mur: None,
                });
            }
        }
        chaines
    }

    /// Lit les mesures inscrites, déduit celles qui peuvent l'être et calcule
    /// l'aire et le périmètre quand la forme est complète. Un nombre sans
    /// unité est lu dans le format `defaut`.
    pub fn resoudre(&self, defaut: Format) -> Solution {
        let mut modele = None;
        let mut mesures: Vec<Mesure> = self
            .cotes
            .iter()
            .map(|cote| {
                let texte = cote.saisie.trim();
                if texte.is_empty() {
                    return Mesure::Manquante;
                }
                match analyser_longueur(texte, defaut) {
                    Ok(v) if v.est_longueur() && v.base > 0.0 && v.base.is_finite() => {
                        modele.get_or_insert(v.format);
                        Mesure::Lue(v.base)
                    }
                    _ => Mesure::Invalide,
                }
            })
            .collect();
        let etat = self.etat(&mut mesures);
        let lire = |texte: &str| match analyser_longueur(texte.trim(), defaut) {
            Ok(v) if v.est_longueur() && v.base > 0.0 && v.base.is_finite() => Some(v.base),
            _ => None,
        };
        let retraits = self
            .retraits
            .iter()
            .map(|retrait| {
                if retrait.longueur.trim().is_empty() && retrait.largeur.trim().is_empty() {
                    Some(0.0)
                } else {
                    Some(lire(&retrait.longueur)? * lire(&retrait.largeur)?)
                }
            })
            .collect();
        // Les diagonales d'une pièce à quatre côtés : avec les quatre côtés
        // mesurés, elles donnent sa vraie forme (point 33).
        let diagonales = [0, 1].map(|k| {
            let texte = self.diagonales[k].trim();
            if texte.is_empty() {
                Mesure::Manquante
            } else {
                lire(texte).map_or(Mesure::Invalide, Mesure::Lue)
            }
        });
        let (etat, reelle) = self.selon_les_diagonales(etat, &mesures, &diagonales);
        let (forme_reelle, ecart_des_diagonales) = match reelle {
            Some((sommets, ecart)) => (Some(sommets), ecart),
            None => (None, None),
        };
        let texte_de_la_hauteur = self.hauteur_des_murs.trim();
        let hauteur = if texte_de_la_hauteur.is_empty() {
            None
        } else {
            lire(texte_de_la_hauteur)
        };
        let ouvertures = self
            .ouvertures
            .iter()
            .map(|ouverture| {
                if ouverture.largeur.trim().is_empty() && ouverture.hauteur.trim().is_empty() {
                    Some(0.0)
                } else {
                    Some(lire(&ouverture.largeur)? * lire(&ouverture.hauteur)?)
                }
            })
            .collect();
        // Une distance au coin peut être nulle : la porte est contre le coin.
        let lire_ou_zero = |texte: &str| match analyser_longueur(texte.trim(), defaut) {
            Ok(v) if v.est_longueur() && v.base >= 0.0 && v.base.is_finite() => Some(v.base),
            _ => None,
        };
        let placements = self
            .ouvertures
            .iter()
            .map(|ouverture| {
                let cote = ouverture.cote?;
                let longueur_du_mur = mesures.get(cote)?.pouces()?;
                let largeur = lire(&ouverture.largeur)?;
                if largeur > longueur_du_mur + TOLERANCE_PO {
                    return None;
                }
                let largeur = largeur.min(longueur_du_mur);
                let libre = longueur_du_mur - largeur;
                let debut = if ouverture.position.trim().is_empty() {
                    libre / 2.0
                } else {
                    lire_ou_zero(&ouverture.position)?.min(libre)
                };
                let hauteur = lire(&ouverture.hauteur);
                let genre = ouverture.genre_ou_deduit(hauteur);
                let porte = genre.est_une_porte();
                let bas = if porte {
                    0.0
                } else if ouverture.allege.trim().is_empty() {
                    ALLEGE_PAR_DEFAUT
                } else {
                    lire_ou_zero(&ouverture.allege).unwrap_or(ALLEGE_PAR_DEFAUT)
                };
                let hauteur = hauteur.unwrap_or(if porte {
                    HAUTEUR_DE_PORTE_PAR_DEFAUT
                } else {
                    HAUTEUR_DE_FENETRE_PAR_DEFAUT
                });
                Some(Placement {
                    cote,
                    debut,
                    fin: debut + largeur,
                    porte,
                    genre,
                    inversee: ouverture.inversee,
                    bas,
                    haut: bas + hauteur,
                })
            })
            .collect();
        let objets = self
            .objets
            .iter()
            .map(|objet| {
                let (largeur, profondeur) = objet.dimensions_de_depart();
                (
                    lire(&objet.largeur).unwrap_or(largeur),
                    lire(&objet.profondeur).unwrap_or(profondeur),
                )
            })
            .collect::<Vec<_>>();
        // Les escaliers : leur hauteur à monter, celle des murs s'ils n'en ont
        // pas d'inscrite (8 pi sans murs), et leurs contremarches.
        let escaliers = self
            .objets
            .iter()
            .zip(&objets)
            .map(|(objet, &dimensions)| {
                if objet.genre != crate::mobilier::GenreDObjet::Escalier {
                    return None;
                }
                let a_monter = lire(&objet.hauteur)
                    .or(hauteur)
                    .unwrap_or(crate::maquette::HAUTEUR_PAR_DEFAUT);
                let voulues = objet.contremarches.trim().parse::<usize>().ok();
                crate::mobilier::calculer_l_escalier(objet.forme, dimensions, a_monter, voulues)
            })
            .collect();
        // Une épaisseur sans unité se lit en pouces (en millimètres si les
        // mesures sont métriques) : « 9 » fait un mur de 9 po, pas de 9 pi.
        let format_des_epaisseurs =
            if modele.and_then(Format::systeme) == Some(crate::unites::Systeme::Metrique) {
                Format::Mm
            } else {
                Format::PoFrac
            };
        let lire_une_epaisseur = |texte: &str, nulle_permise: bool| match analyser_longueur(
            texte.trim(),
            format_des_epaisseurs,
        ) {
            Ok(v)
                if v.est_longueur()
                    && v.base.is_finite()
                    && (v.base > 0.0 || (nulle_permise && v.base >= 0.0)) =>
            {
                Some(v.base)
            }
            _ => None,
        };
        let epaisseur_des_murs = lire_une_epaisseur(&self.epaisseur_des_murs, true)
            .unwrap_or(EPAISSEUR_DES_MURS_PAR_DEFAUT);
        let epaisseur_des_cloisons = lire_une_epaisseur(&self.epaisseur_des_cloisons, false)
            .unwrap_or(crate::cloison::EPAISSEUR_PAR_DEFAUT);
        let cloisons = self
            .cloisons
            .iter()
            .map(|cloison| {
                // Tous ses segments doivent avoir une longueur lisible.
                let segments: Vec<(Sens, f64)> = cloison
                    .segments()
                    .map(|(sens, texte)| lire(texte).map(|longueur| (sens, longueur)))
                    .collect::<Option<_>>()?;
                let epaisseur =
                    lire_une_epaisseur(&cloison.epaisseur, false).unwrap_or(epaisseur_des_cloisons);
                // Ses portes et ses passages, chacun sur l'un de ses segments :
                // centrés sans distance inscrite; jamais plus larges que leur
                // segment, ni hors de lui. Une largeur sans unité se lit en
                // pouces.
                let ouvertures = cloison.toutes_les_ouvertures().map(|porte| {
                    let segment = porte.segment.min(segments.len() - 1);
                    let longueur = segments[segment].1;
                    let largeur = lire_une_epaisseur(&porte.largeur, false)
                        .unwrap_or(crate::cloison::LARGEUR_DE_PORTE_PAR_DEFAUT)
                        .min(longueur);
                    let libre = longueur - largeur;
                    let debut = if porte.position.trim().is_empty() {
                        libre / 2.0
                    } else {
                        lire_ou_zero(&porte.position)
                            .unwrap_or(libre / 2.0)
                            .min(libre)
                    };
                    crate::cloison::PorteMesuree {
                        segment,
                        debut,
                        fin: debut + largeur,
                        inversee: porte.inversee,
                        passage: porte.passage,
                    }
                });
                let ouvertures = ouvertures.collect();
                Some(
                    crate::cloison::CloisonMesuree::depuis(
                        cloison.debut,
                        &segments,
                        epaisseur,
                        None,
                    )
                    .avec_ouvertures(ouvertures),
                )
            })
            .collect();
        // Des mesures prises à l'extérieur des murs : la surface et le
        // périmètre sont ceux de la face intérieure (ce qu'on couvre et qu'on
        // peint); la surface brute reste, à part.
        let (etat, aire_exterieure) = match etat {
            Etat::Complete { aire, .. } if self.mesures_exterieures && epaisseur_des_murs > 0.0 => {
                let longueurs: Option<Vec<f64>> = mesures.iter().map(|m| m.pouces()).collect();
                let sommets = forme_reelle
                    .clone()
                    .or_else(|| longueurs.map(|l| self.sommets(&l)));
                let interieur = sommets
                    .and_then(|s| decaler(&s, epaisseur_des_murs, false))
                    .map(|contour| aire_et_perimetre(&contour));
                match interieur {
                    Some((aire_interieure, perimetre)) if aire_interieure > 0.0 => (
                        Etat::Complete {
                            aire: aire_interieure,
                            perimetre,
                        },
                        Some(aire),
                    ),
                    _ => (etat, None),
                }
            }
            _ => (etat, None),
        };
        // Le plafond, sur la face intérieure des murs (point 34) : la surface
        // de chaque mur, celle du plafond et le volume de la pièce.
        let interieur: Option<Vec<(f64, f64)>> = match etat {
            Etat::Complete { .. } => {
                let longueurs: Option<Vec<f64>> = mesures.iter().map(|m| m.pouces()).collect();
                let sommets = forme_reelle
                    .clone()
                    .or_else(|| longueurs.map(|l| self.sommets(&l)));
                if self.mesures_exterieures && epaisseur_des_murs > 0.0 {
                    sommets.and_then(|s| decaler(&s, epaisseur_des_murs, false))
                } else {
                    sommets
                }
            }
            _ => None,
        };
        let texte_du_haut = self.plafond.haut.trim();
        let haut_du_plafond = if texte_du_haut.is_empty() {
            None
        } else {
            lire(texte_du_haut)
        };
        let plat = self.plafond.forme == crate::plafond::FormeDePlafond::Plat;
        let plafond_incomplet = !plat && texte_du_haut.is_empty();
        let plafond_invalide = !plat && !texte_du_haut.is_empty() && haut_du_plafond.is_none();
        let bas = hauteur.unwrap_or(crate::maquette::HAUTEUR_PAR_DEFAUT);
        let plafond = interieur.as_ref().and_then(|sommets| {
            let forme = if haut_du_plafond.is_some() {
                self.plafond.forme
            } else {
                crate::plafond::FormeDePlafond::Plat
            };
            crate::plafond::PlafondMesure::nouveau(
                forme,
                bas,
                haut_du_plafond.unwrap_or(bas),
                sommets,
                self.plafond.cote,
            )
        });
        let retombees_lues: Vec<Option<(f64, f64)>> = self
            .retombees
            .iter()
            .map(|retombee| {
                let textes = [&retombee.longueur, &retombee.largeur, &retombee.hauteur];
                if textes.iter().all(|texte| texte.trim().is_empty()) {
                    return Some((0.0, 0.0));
                }
                let (longueur, largeur, descente) = (
                    lire(&retombee.longueur)?,
                    lire(&retombee.largeur)?,
                    lire(&retombee.hauteur)?,
                );
                Some((
                    longueur * largeur * descente,
                    2.0 * (longueur + largeur) * descente,
                ))
            })
            .collect();
        let (surfaces_des_murs, surface_du_plafond, volume) = match (&plafond, &interieur) {
            (Some(au_plafond), Some(sommets))
                if hauteur.is_some() && !plafond_incomplet && !plafond_invalide =>
            {
                let murs = sommets
                    .windows(2)
                    .map(|mur| au_plafond.surface_du_mur(mur[0], mur[1]))
                    .collect();
                let retombees: Option<Vec<(f64, f64)>> = retombees_lues.iter().copied().collect();
                match retombees {
                    Some(retombees) => (
                        murs,
                        Some(
                            au_plafond.surface(sommets)
                                + retombees.iter().map(|r| r.1).sum::<f64>(),
                        ),
                        Some(
                            au_plafond.volume(sommets) - retombees.iter().map(|r| r.0).sum::<f64>(),
                        )
                        .filter(|volume| *volume > 0.0),
                    ),
                    None => (murs, None, None),
                }
            }
            _ => (Vec::new(), None, None),
        };
        Solution {
            mesures,
            etat,
            modele,
            retraits,
            hauteur,
            hauteur_invalide: !texte_de_la_hauteur.is_empty() && hauteur.is_none(),
            ouvertures,
            placements,
            objets,
            escaliers,
            cloisons,
            epaisseur_des_murs,
            epaisseur_des_cloisons,
            aire_exterieure,
            diagonales,
            forme_reelle,
            ecart_des_diagonales,
            plafond,
            plafond_incomplet,
            plafond_invalide,
            surfaces_des_murs,
            surface_du_plafond,
            volume,
            retombees: retombees_lues,
        }
    }

    /// L'état d'une pièce à quatre côtés fermée, ses quatre côtés mesurés et
    /// au moins une diagonale aussi : sa vraie forme, et l'écart entre les
    /// diagonales si les deux sont mesurées. Sinon, l'état reste celui de la
    /// fermeture (une diagonale illisible le rend illisible).
    fn selon_les_diagonales(
        &self,
        etat: Etat,
        mesures: &[Mesure],
        diagonales: &[Mesure; 2],
    ) -> (Etat, Option<FormeReelle>) {
        if !self.fermee || self.cotes.len() != 4 || matches!(etat, Etat::MesureInvalide) {
            return (etat, None);
        }
        if diagonales.contains(&Mesure::Invalide) {
            return (Etat::MesureInvalide, None);
        }
        let lues = diagonales.map(|m| match m {
            Mesure::Lue(x) => Some(x),
            _ => None,
        });
        let cotes: Option<Vec<f64>> = mesures
            .iter()
            .map(|m| match m {
                Mesure::Lue(x) => Some(*x),
                _ => None,
            })
            .collect();
        let (Some(cotes), true) = (cotes, lues.iter().any(Option::is_some)) else {
            return (etat, None);
        };
        match self.forme_d_apres_les_diagonales([cotes[0], cotes[1], cotes[2], cotes[3]], lues) {
            Ok((sommets, ecart)) => {
                if se_croise(&sommets) {
                    return (Etat::SeCroise, None);
                }
                let (aire, perimetre) = aire_et_perimetre(&sommets);
                (Etat::Complete { aire, perimetre }, Some((sommets, ecart)))
            }
            Err(diagonale) => (Etat::DiagonaleImpossible(diagonale), None),
        }
    }

    /// La vraie forme d'une pièce à quatre côtés (`a`, `b`, `c`, `d`, du coin
    /// 1 au coin 1) d'après une diagonale ou les deux : du coin 1 au coin 3
    /// (`p`), et du coin 2 au coin 4 (`q`). Le premier côté garde sa direction;
    /// chaque coin se place du même côté qu'il est sur le croquis. Avec les
    /// deux diagonales, la forme suit `p`, et l'écart avec `q` est rendu.
    /// `Err(k)` si la diagonale `k` ne va pas avec les côtés.
    fn forme_d_apres_les_diagonales(
        &self,
        [a, b, c, d]: [f64; 4],
        [p, q]: [Option<f64>; 2],
    ) -> Result<FormeReelle, usize> {
        let croquis = self.sommets_du_croquis();
        // De quel côté de la droite de `o` à `v` est le point `w` : +1 ou −1.
        let cote_de = |o: (f64, f64), v: (f64, f64), w: (f64, f64)| {
            if (v.0 - o.0) * (w.1 - o.1) - (v.1 - o.1) * (w.0 - o.0) < 0.0 {
                -1.0
            } else {
                1.0
            }
        };
        // Un point à `longueur` de l'origine, dans la direction `u` tournée de
        // `angle` (radians, vers le bas comme à l'écran).
        let vers = |u: (f64, f64), angle: f64, longueur: f64| {
            let (sin, cos) = angle.sin_cos();
            (
                (u.0 * cos - u.1 * sin) * longueur,
                (u.0 * sin + u.1 * cos) * longueur,
            )
        };
        let [ka, kb, kc, kd] = [croquis[0], croquis[1], croquis[2], croquis[3]];
        let u = self.cotes[0].sens.pas();
        let pa = (0.0, 0.0);
        let pb = vers(u, 0.0, a);
        let (pc, pd) = if let Some(p) = p {
            // Les triangles 1-2-3 et 1-3-4, de part et d'autre de la diagonale
            // (ou du même côté, pour un coin rentrant).
            let alpha = angle_du_triangle(a, p, b).ok_or(0usize)?;
            let beta = angle_du_triangle(p, d, c).ok_or(0usize)?;
            let pc = vers(u, cote_de(ka, kb, kc) * alpha, p);
            let vers_c = (pc.0 / p, pc.1 / p);
            let pd = vers(vers_c, cote_de(ka, kc, kd) * beta, d);
            (pc, pd)
        } else {
            let q = q.ok_or(1usize)?;
            // Le triangle 1-2-4, puis le coin 3 à la rencontre de ses deux côtés.
            let alpha = angle_du_triangle(a, d, q).ok_or(1usize)?;
            let pd = vers(u, cote_de(ka, kb, kd) * alpha, d);
            let pc = rencontre_de_cercles(pb, b, pd, c, cote_de(kb, kd, kc)).ok_or(1usize)?;
            (pc, pd)
        };
        let ecart = match (p, q) {
            (Some(_), Some(q)) => Some(((pd.0 - pb.0).hypot(pd.1 - pb.1) - q).abs()),
            _ => None,
        };
        Ok((vec![pa, pb, pc, pd, pa], ecart))
    }

    fn etat(&self, mesures: &mut [Mesure]) -> Etat {
        if self.cotes.is_empty() {
            return Etat::Vide;
        }
        if mesures.contains(&Mesure::Invalide) {
            return Etat::MesureInvalide;
        }
        if !self.fermee {
            return Etat::Ouverte;
        }
        // Ce que donnent les côtés connus, et les côtés inconnus groupés par axe.
        let (mut rx, mut ry) = (0.0, 0.0);
        let mut groupes: Vec<(Axe, Vec<usize>)> = Vec::new();
        for (i, cote) in self.cotes.iter().enumerate() {
            if let Some(longueur) = mesures[i].pouces() {
                let (ux, uy) = cote.sens.pas();
                rx += longueur * ux;
                ry += longueur * uy;
            } else {
                let axe = cote.sens.axe();
                match groupes.iter_mut().find(|(a, _)| *a == axe) {
                    Some((_, rangs)) => rangs.push(i),
                    None => groupes.push((axe, vec![i])),
                }
            }
        }
        // Les côtés inconnus doivent ramener la forme à son départ. Sur un ou
        // deux axes, ce retour se décompose d'une seule façon; sur trois axes
        // ou plus, il ne détermine aucun côté.
        let mut manquantes = 0;
        let mut ecart_maximal: f64 = 0.0;
        match groupes.as_slice() {
            [] => ecart_maximal = rx.abs().max(ry.abs()),
            [(axe, rangs)] => {
                let (ux, uy) = axe.vecteur();
                // Ce que cet axe ne peut pas reprendre : l'écart de travers.
                ecart_maximal = (rx * uy - ry * ux).abs();
                let le_long = -(rx * ux + ry * uy);
                self.deduire_sur_l_axe(
                    mesures,
                    rangs,
                    le_long,
                    &mut ecart_maximal,
                    &mut manquantes,
                );
            }
            [(a, rangs_a), (b, rangs_b)]
                if {
                    let (u, v) = (a.vecteur(), b.vecteur());
                    (u.0 * v.1 - u.1 * v.0).abs() >= SINUS_MINIMAL
                } =>
            {
                let (u, v) = (a.vecteur(), b.vecteur());
                let determinant = u.0 * v.1 - u.1 * v.0;
                let (wx, wy) = (-rx, -ry);
                let le_long_de_a = (wx * v.1 - wy * v.0) / determinant;
                let le_long_de_b = (u.0 * wy - u.1 * wx) / determinant;
                self.deduire_sur_l_axe(
                    mesures,
                    rangs_a,
                    le_long_de_a,
                    &mut ecart_maximal,
                    &mut manquantes,
                );
                self.deduire_sur_l_axe(
                    mesures,
                    rangs_b,
                    le_long_de_b,
                    &mut ecart_maximal,
                    &mut manquantes,
                );
            }
            _ => {
                // Sur trois axes ou plus, il faut mesurer jusqu'à ce qu'il ne
                // reste que deux axes; sur deux axes presque parallèles,
                // jusqu'à ce qu'il n'en reste qu'un.
                let inconnus: usize = groupes.iter().map(|(_, rangs)| rangs.len()).sum();
                let determinables = if groupes.len() == 2 { 1 } else { 2 };
                manquantes += inconnus - determinables;
            }
        }
        if ecart_maximal > TOLERANCE_PO {
            return Etat::NeFermePas(ecart_maximal);
        }
        if manquantes > 0 {
            return Etat::Incomplete(manquantes);
        }
        let longueurs: Vec<f64> = mesures.iter().filter_map(|m| m.pouces()).collect();
        let sommets = self.sommets(&longueurs);
        if se_croise(&sommets) {
            return Etat::SeCroise;
        }
        let aire = sommets
            .windows(2)
            .map(|s| s[0].0 * s[1].1 - s[1].0 * s[0].1)
            .sum::<f64>()
            .abs()
            / 2.0;
        Etat::Complete {
            aire,
            perimetre: longueurs.iter().sum(),
        }
    }
}

/// Vrai si deux côtés non voisins d'un contour fermé se touchent ou se croisent.
/// `sommets` répète le point de départ à la fin.
fn se_croise(sommets: &[(f64, f64)]) -> bool {
    let n = sommets.len().saturating_sub(1);
    let eps = TOLERANCE_PO / 4.0;
    for i in 0..n {
        for j in (i + 2)..n {
            if i == 0 && j == n - 1 {
                continue;
            }
            if se_touchent(sommets[i], sommets[i + 1], sommets[j], sommets[j + 1], eps) {
                return true;
            }
        }
    }
    false
}

/// Vrai si les segments `ab` et `cd` se touchent ou se croisent, à `eps` près.
fn se_touchent(a: (f64, f64), b: (f64, f64), c: (f64, f64), d: (f64, f64), eps: f64) -> bool {
    // Leurs boîtes doivent se recouvrir.
    let recouvre =
        |p: f64, q: f64, r: f64, s: f64| p.min(q) <= r.max(s) + eps && r.min(s) <= p.max(q) + eps;
    if !recouvre(a.0, b.0, c.0, d.0) || !recouvre(a.1, b.1, c.1, d.1) {
        return false;
    }
    let orientation = |p: (f64, f64), q: (f64, f64), r: (f64, f64)| {
        (q.0 - p.0) * (r.1 - p.1) - (q.1 - p.1) * (r.0 - p.0)
    };
    // Deux points nettement du même côté d'un segment : pas de contact.
    let d_un_seul_cote = |p: (f64, f64), q: (f64, f64), r: (f64, f64), s: (f64, f64)| {
        let tolerance = eps * (q.0 - p.0).hypot(q.1 - p.1);
        let (o1, o2) = (orientation(p, q, r), orientation(p, q, s));
        (o1 > tolerance && o2 > tolerance) || (o1 < -tolerance && o2 < -tolerance)
    };
    !d_un_seul_cote(c, d, a, b) && !d_un_seul_cote(a, b, c, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_miroir_est_un_vrai_reflet() {
        use crate::mobilier::{FormeDEscalier, GenreDObjet, Objet, TraitSurLePlan};
        let mut dessin = Dessin::default();
        dessin.ajouter_trace(&[
            (0.0, 0.0),
            (100.0, 0.0),
            (100.0, 60.0),
            (0.0, 60.0),
            (0.0, 4.0),
        ]);
        dessin.cotes[0].saisie = "12".to_string();
        dessin.cotes[1].saisie = "10".to_string();
        let mut en_l = Objet::nouveau(GenreDObjet::Escalier, (40.0, 30.0));
        en_l.forme = FormeDEscalier::EnL;
        en_l.pivoter(30.0);
        let mut tournant = Objet::nouveau(GenreDObjet::Escalier, (100.0, 50.0));
        tournant.forme = FormeDEscalier::Tournant;
        tournant.rotation = 1;
        let mut colimacon = Objet::nouveau(GenreDObjet::Escalier, (70.0, 90.0));
        colimacon.forme = FormeDEscalier::Colimacon;
        dessin.objets.extend([en_l, tournant, colimacon]);
        dessin.cloisons.push(crate::cloison::Cloison {
            debut: (0.0, 72.0),
            sens: Sens::Est,
            longueur: "6".to_string(),
            porte: Some(crate::cloison::PorteDeCloison::default()),
            ..crate::cloison::Cloison::default()
        });
        let reflet = |(x, y): (f64, f64)| (-x, y);
        let proches =
            |a: (f64, f64), b: (f64, f64)| (a.0 - b.0).abs() < 1e-9 && (a.1 - b.1).abs() < 1e-9;
        let dimensions = (36.0, 120.0);
        let traits = |dessin: &Dessin| -> Vec<Vec<TraitSurLePlan>> {
            dessin
                .objets
                .iter()
                .map(|objet| objet.traits_sur_le_plan(dimensions, Some(14)))
                .collect()
        };
        let porte = |dessin: &Dessin| {
            dessin.resoudre(Format::PiPo).cloisons[0]
                .as_ref()
                .expect("cloison mesurée")
                .cote_de_l_ouverture(
                    &dessin.resoudre(Format::PiPo).cloisons[0]
                        .as_ref()
                        .expect("cloison mesurée")
                        .ouvertures[0],
                )
        };
        let (traits_avant, porte_avant) = (traits(&dessin), porte(&dessin));
        dessin.miroir();
        assert!(dessin.objets.iter().all(|objet| objet.retourne));
        // Chaque trait de chaque escalier est le reflet de celui d'avant.
        for (avant, apres) in traits_avant.iter().zip(traits(&dessin)) {
            assert_eq!(avant.len(), apres.len());
            for (a, b) in avant.iter().zip(&apres) {
                match (a, b) {
                    (TraitSurLePlan::Ligne(a0, a1), TraitSurLePlan::Ligne(b0, b1)) => {
                        assert!(proches(reflet(*a0), *b0) && proches(reflet(*a1), *b1));
                    }
                    (TraitSurLePlan::Cercle(a0, ra), TraitSurLePlan::Cercle(b0, rb)) => {
                        assert!(proches(reflet(*a0), *b0) && (ra - rb).abs() < 1e-9);
                    }
                    _ => panic!("des traits de genres différents"),
                }
            }
        }
        // La porte de la cloison s'ouvre toujours du même côté de la pièce.
        assert!(proches(reflet(porte_avant), porte(&dessin)));
        dessin.miroir();
        assert!(dessin.objets.iter().all(|objet| !objet.retourne));
        assert!(proches(porte_avant, porte(&dessin)));
    }

    #[test]
    fn le_miroir_d_une_piece_et_sa_copie() {
        let mut dessin = Dessin::default();
        dessin.ajouter_trace(&[
            (0.0, 0.0),
            (100.0, 0.0),
            (100.0, 60.0),
            (0.0, 60.0),
            (0.0, 4.0),
        ]);
        assert!(dessin.fermee);
        dessin.cotes[0].saisie = "12".to_string();
        dessin.cotes[1].saisie = "10".to_string();
        let mut lit =
            crate::mobilier::Objet::nouveau(crate::mobilier::GenreDObjet::Lit, (30.0, 40.0));
        lit.pivoter(15.0);
        dessin.objets.push(lit);
        dessin.notes.push(Note {
            texte: "Prise".to_string(),
            position: (10.0, 12.0),
        });
        dessin.cloisons.push(crate::cloison::Cloison {
            debut: (0.0, 72.0),
            sens: Sens::Est,
            longueur: "6".to_string(),
            suite: vec![crate::cloison::SegmentDeCloison {
                sens: Sens::Sud,
                longueur: "4".to_string(),
            }],
            ..crate::cloison::Cloison::default()
        });
        let origine = dessin.clone();
        let avant = dessin.resoudre(Format::PiPo);
        dessin.miroir();
        let sens: Vec<Sens> = dessin.cotes.iter().map(|c| c.sens).collect();
        assert_eq!(sens, [Sens::Ouest, Sens::Sud, Sens::Est, Sens::Nord]);
        assert_eq!(dessin.objets[0].centre, (-30.0, 40.0));
        assert!((dessin.objets[0].angle_total() - 345.0).abs() < 1e-9);
        assert_eq!(dessin.notes[0].position, (-10.0, 12.0));
        assert_eq!(dessin.cloisons[0].sens, Sens::Ouest);
        assert_eq!(dessin.cloisons[0].suite[0].sens, Sens::Sud);
        // Les mesures et les résultats ne changent pas.
        let apres = dessin.resoudre(Format::PiPo);
        assert_eq!(avant.aire_nette(), apres.aire_nette());
        assert_eq!(avant.etat, apres.etat);
        // Deux miroirs redonnent la pièce.
        dessin.miroir();
        assert_eq!(dessin.cotes, origine.cotes);
        assert_eq!(dessin.objets[0].centre, origine.objets[0].centre);
        assert!((dessin.objets[0].angle_total() - 15.0).abs() < 1e-9);
        assert_eq!(dessin.cloisons, origine.cloisons);
        // La copie d'une pièce vient juste après elle, à l'écran.
        let mut plan = Plan::d_une_piece(origine);
        plan.ajouter();
        plan.courante = 0;
        assert!(plan.dupliquer("Chambre (copie)".to_string()));
        assert_eq!(plan.pieces.len(), 3);
        assert_eq!(plan.courante, 1);
        assert_eq!(plan.pieces[1].nom, "Chambre (copie)");
        assert_eq!(plan.pieces[1].dessin, plan.pieces[0].dessin);
    }

    /// Un trait à main levée de `a` à `b`, avec un léger tremblement.
    fn trait_tremble(a: (f64, f64), b: (f64, f64)) -> Vec<(f64, f64)> {
        (0..=20)
            .map(|i| {
                let t = f64::from(i) / 20.0;
                let tremble = if i % 2 == 0 { 0.6 } else { -0.6 };
                (
                    a.0 + (b.0 - a.0) * t + tremble,
                    a.1 + (b.1 - a.1) * t - tremble,
                )
            })
            .collect()
    }

    fn contour(coins: &[(f64, f64)]) -> Vec<(f64, f64)> {
        coins
            .windows(2)
            .flat_map(|c| trait_tremble(c[0], c[1]))
            .collect()
    }

    fn inscrire(dessin: &mut Dessin, saisies: &[&str]) {
        for (cote, saisie) in dessin.cotes.iter_mut().zip(saisies) {
            cote.saisie = (*saisie).to_string();
        }
    }

    #[test]
    fn un_trait_devient_un_cote() {
        let pas = redresser(&trait_tremble((10.0, 50.0), (200.0, 58.0)));
        assert_eq!(pas.len(), 1);
        assert_eq!(pas[0].0, Axe::Horizontal);
        assert!((pas[0].1 - 190.0).abs() < 3.0);
        assert!(redresser(&[(5.0, 5.0)]).is_empty());
        assert!(redresser(&[(5.0, 5.0), (5.0, 5.0)]).is_empty());
    }

    #[test]
    fn un_rectangle_trace_d_un_seul_geste() {
        let mut dessin = Dessin::default();
        // Le tracé s'arrête un peu avant le point de départ.
        let trace = contour(&[
            (20.0, 20.0),
            (220.0, 24.0),
            (216.0, 140.0),
            (24.0, 136.0),
            (22.0, 34.0),
        ]);
        assert!(dessin.ajouter_trace(&trace));
        assert!(dessin.fermee);
        let sens: Vec<Sens> = dessin.cotes.iter().map(|c| c.sens).collect();
        assert_eq!(sens, [Sens::Est, Sens::Sud, Sens::Ouest, Sens::Nord]);
        // Le croquis fermé revient exactement au départ.
        let fin = *dessin.sommets_du_croquis().last().unwrap();
        assert!(fin.0.abs() < 1e-9 && fin.1.abs() < 1e-9);

        // Deux mesures suffisent : les côtés opposés se déduisent.
        inscrire(&mut dessin, &["12 pi", "10 pi 6 po"]);
        let solution = dessin.resoudre(Format::PiPo);
        assert_eq!(solution.mesures[0], Mesure::Lue(144.0));
        assert_eq!(solution.mesures[2], Mesure::Deduite(144.0));
        assert_eq!(solution.mesures[3], Mesure::Deduite(126.0));
        assert_eq!(
            solution.etat,
            Etat::Complete {
                aire: 144.0 * 126.0,
                perimetre: 2.0 * (144.0 + 126.0)
            }
        );
        assert_eq!(solution.modele, Some(Format::PiPo));
    }

    #[test]
    fn une_piece_en_l_cote_par_cote() {
        let mut dessin = Dessin::default();
        let coins = [
            (0.0, 0.0),
            (200.0, 0.0),
            (200.0, 80.0),
            (90.0, 80.0),
            (90.0, 180.0),
            (0.0, 180.0),
        ];
        for paire in coins.windows(2) {
            assert!(dessin.ajouter_trace(&trait_tremble(paire[0], paire[1])));
        }
        assert!(!dessin.fermee);
        assert_eq!(dessin.resoudre(Format::PiPo).etat, Etat::Ouverte);
        // Le bouton « Fermer » ajoute le côté qui manque.
        assert!(dessin.fermer());
        assert_eq!(dessin.cotes.len(), 6);
        assert_eq!(dessin.cotes[5].sens, Sens::Nord);

        // 20 pi × 18 pi, avec un recoin de 11 pi × 10 pi : 360 − 110 = 250 pi².
        inscrire(&mut dessin, &["20", "8", "11", "10"]);
        let solution = dessin.resoudre(Format::PiPo);
        assert_eq!(solution.mesures[4], Mesure::Deduite(9.0 * 12.0));
        assert_eq!(solution.mesures[5], Mesure::Deduite(18.0 * 12.0));
        match solution.etat {
            Etat::Complete { aire, perimetre } => {
                assert!((aire - 250.0 * 144.0).abs() < 1e-6);
                assert!((perimetre - 76.0 * 12.0).abs() < 1e-6);
            }
            autre => panic!("forme incomplète : {autre:?}"),
        }
    }

    #[test]
    fn mesures_manquantes_invalides_ou_incoherentes() {
        let mut dessin = Dessin::default();
        dessin.ajouter_trace(&contour(&[
            (0.0, 0.0),
            (100.0, 0.0),
            (100.0, 60.0),
            (0.0, 60.0),
            (0.0, 5.0),
        ]));
        assert!(dessin.fermee);
        assert_eq!(dessin.resoudre(Format::PiPo).etat, Etat::Incomplete(2));
        inscrire(&mut dessin, &["12"]);
        assert_eq!(dessin.resoudre(Format::PiPo).etat, Etat::Incomplete(1));
        inscrire(&mut dessin, &["12", "abc"]);
        assert_eq!(dessin.resoudre(Format::PiPo).etat, Etat::MesureInvalide);
        // Les deux côtés opposés ne s'accordent pas : 12 pi et 11 pi.
        inscrire(&mut dessin, &["12", "10", "11", "10"]);
        assert_eq!(dessin.resoudre(Format::PiPo).etat, Etat::NeFermePas(12.0));
        // En métrique : 4 m × 3 m.
        inscrire(&mut dessin, &["4 m", "3 m", "", ""]);
        let solution = dessin.resoudre(Format::PiPo);
        assert_eq!(solution.modele, Some(Format::M));
        match solution.etat {
            Etat::Complete { aire, .. } => {
                let m2 = aire / (39.370_078_740_157_48f64).powi(2);
                assert!((m2 - 12.0).abs() < 1e-6, "{m2}");
            }
            autre => panic!("{autre:?}"),
        }
    }

    #[test]
    fn un_cote_trop_court_fait_croiser_la_forme() {
        // Un U : la branche de droite descend plus bas que le fond ne le permet.
        let mut dessin = Dessin::default();
        let coins = [
            (0.0, 0.0),
            (60.0, 0.0),
            (60.0, 100.0),
            (140.0, 100.0),
            (140.0, 0.0),
            (200.0, 0.0),
            (200.0, 160.0),
            (0.0, 160.0),
        ];
        for paire in coins.windows(2) {
            dessin.ajouter_trace(&trait_tremble(paire[0], paire[1]));
        }
        assert!(dessin.fermer());
        assert_eq!(dessin.cotes.len(), 8);
        inscrire(&mut dessin, &["6", "10", "8", "10", "6", "16"]);
        assert!(matches!(
            dessin.resoudre(Format::PiPo).etat,
            Etat::Complete { .. }
        ));
        // Le creux (20 pi) dépasse la hauteur de la pièce (16 pi).
        inscrire(&mut dessin, &["6", "20", "8", "20", "6", "16"]);
        assert_eq!(dessin.resoudre(Format::PiPo).etat, Etat::SeCroise);
    }

    #[test]
    fn rectangle_equivalent_et_outils() {
        // Un rectangle est son propre équivalent.
        let (l, w, exact) = rectangle_equivalent(144.0 * 126.0, 2.0 * (144.0 + 126.0));
        assert!(exact && (l - 144.0).abs() < 1e-9 && (w - 126.0).abs() < 1e-9);
        // La pièce en L de 250 pi² et de 76 pi de périmètre.
        let (l, w, exact) = rectangle_equivalent(250.0 * 144.0, 76.0 * 12.0);
        assert!(exact);
        assert!((l * w - 250.0 * 144.0).abs() < 1e-6);
        assert!((2.0 * (l + w) - 76.0 * 12.0).abs() < 1e-9);
        // Un disque n'a pas d'équivalent : le carré de même aire.
        assert_eq!(rectangle_equivalent(100.0, 20.0), (10.0, 10.0, false));

        // Ce qui est écrit se relit à la même valeur, dans les deux systèmes.
        for (pouces, metrique, decimal) in [
            (126.0, false, ','),
            (277.128, false, '.'),
            (150.0, true, ','),
        ] {
            let texte = longueur_a_taper(pouces, metrique, decimal);
            let relu = if decimal == ',' {
                analyser_longueur(&texte, Format::PiPo).unwrap().base
            } else {
                let _anglais = crate::langue::pour_la_portee(crate::Langue::Anglais);
                analyser_longueur(&texte, Format::PiPo).unwrap().base
            };
            assert!((relu - pouces).abs() < 0.01, "{texte} -> {relu}");
        }
        assert_eq!(longueur_a_taper(126.0, false, ','), "10,5 pi");

        let outils = outils_d_une_piece();
        assert!(
            outils
                .iter()
                .any(|(c, l, w, h)| c.id == "dalle" && *l == 0 && *w == 1 && h.is_none())
        );
        // La peinture d'une pièce reçoit aussi la hauteur des murs.
        assert!(
            outils
                .iter()
                .any(|(c, l, w, h)| c.id == "peinture" && *l == 0 && *w == 1 && *h == Some(2))
        );
        assert!(outils.len() >= 8);
        assert!(outils.iter().all(|(c, _, _, _)| c.metier != Metier::Acier));
    }

    #[test]
    fn retraits_et_surface_nette() {
        let mut dessin = Dessin::default();
        dessin.ajouter_trace(&contour(&[
            (0.0, 0.0),
            (100.0, 0.0),
            (100.0, 60.0),
            (0.0, 60.0),
            (0.0, 5.0),
        ]));
        inscrire(&mut dessin, &["20", "15"]);
        let brute = 300.0 * 144.0;
        assert_eq!(dessin.resoudre(Format::PiPo).aire_nette(), Some(brute));
        // Un îlot de 3 pi × 6 pi et une colonne de 12 po × 12 po : 300 − 18 − 1 = 281 pi².
        dessin.retraits.push(Retrait {
            longueur: "3".into(),
            largeur: "6".into(),
        });
        dessin.retraits.push(Retrait {
            longueur: "12 po".into(),
            largeur: "12 po".into(),
        });
        // Un retrait vide ne compte pas.
        dessin.retraits.push(Retrait::default());
        let solution = dessin.resoudre(Format::PiPo);
        assert_eq!(solution.aire_des_retraits(), Some(19.0 * 144.0));
        assert_eq!(solution.aire_nette(), Some(281.0 * 144.0));
        // Incomplet : pas de surface nette.
        dessin.retraits[2].longueur = "4".into();
        assert_eq!(dessin.resoudre(Format::PiPo).aire_nette(), None);
        // Plus grand que la pièce : pas de surface nette non plus.
        dessin.retraits[2].largeur = "100".into();
        let solution = dessin.resoudre(Format::PiPo);
        assert!(solution.aire_des_retraits().is_some());
        assert_eq!(solution.aire_nette(), None);
    }

    #[test]
    fn plusieurs_pieces_et_leur_total() {
        let rectangle = |longueur: &str, largeur: &str| {
            let mut dessin = Dessin::default();
            dessin.ajouter_trace(&contour(&[
                (0.0, 0.0),
                (100.0, 0.0),
                (100.0, 60.0),
                (0.0, 60.0),
                (0.0, 5.0),
            ]));
            inscrire(&mut dessin, &[longueur, largeur]);
            dessin
        };
        let mut plan = Plan::default();
        assert_eq!(plan.pieces.len(), 1);
        assert_eq!(plan.total(Format::PiPo), (0.0, 0));
        plan.piece().dessin = rectangle("12", "10");
        plan.piece().nom = "Cuisine".into();
        assert!(plan.ajouter());
        assert_eq!(plan.courante, 1);
        plan.piece().dessin = rectangle("20", "15");
        // Une troisième pièce, encore sans mesures : elle ne compte pas.
        assert!(plan.ajouter());
        plan.piece().dessin = rectangle("", "");
        assert_eq!(plan.total(Format::PiPo), (420.0 * 144.0, 2));
        // Supprimer la pièce à l'écran.
        plan.courante = 0;
        plan.supprimer_la_courante();
        assert_eq!(plan.pieces.len(), 2);
        assert_eq!(plan.total(Format::PiPo), (300.0 * 144.0, 1));
        // La dernière pièce ne disparaît pas : elle est vidée.
        plan.supprimer_la_courante();
        plan.supprimer_la_courante();
        assert_eq!(plan, Plan::default());
        // Un plan abîmé est remis dans ses règles.
        let mut abime = Plan {
            pieces: Vec::new(),
            courante: 7,
            ..Plan::default()
        };
        abime.normaliser();
        assert_eq!(abime, Plan::default());
        for _ in 0..MAX_PIECES + 5 {
            abime.ajouter();
        }
        assert_eq!(abime.pieces.len(), MAX_PIECES);
    }

    #[test]
    fn dessiner_avec_les_fleches() {
        // Un rectangle : quatre flèches, et la forme se ferme d'elle-même.
        let mut dessin = Dessin::default();
        for sens in [Sens::Est, Sens::Sud, Sens::Ouest] {
            assert!(dessin.ajouter_cote(sens));
            assert!(!dessin.fermee);
        }
        assert!(dessin.ajouter_cote(Sens::Nord));
        assert!(dessin.fermee);
        assert_eq!(dessin.cotes.len(), 4);
        // Une forme fermée ne prend plus de côté.
        assert!(!dessin.ajouter_cote(Sens::Est));
        inscrire(&mut dessin, &["12", "10"]);
        assert_eq!(
            dessin.resoudre(Format::PiPo).etat,
            Etat::Complete {
                aire: 144.0 * 120.0,
                perimetre: 2.0 * (144.0 + 120.0)
            }
        );

        // Dans l'axe du dernier côté : il s'allonge, ou raccourcit sans disparaître.
        let mut dessin = Dessin::default();
        dessin.ajouter_cote(Sens::Est);
        assert!(dessin.ajouter_cote(Sens::Est));
        assert_eq!(dessin.cotes.len(), 1);
        assert!((dessin.cotes[0].croquis - 200.0).abs() < 1e-9);
        assert!(dessin.ajouter_cote(Sens::Ouest));
        assert!(dessin.ajouter_cote(Sens::Ouest));
        assert_eq!(dessin.cotes.len(), 1);
        assert!(dessin.cotes[0].croquis > 0.0);
        assert_eq!(dessin.cotes[0].sens, Sens::Est);

        // Une pièce en L : cinq flèches, puis « Fermer la forme ».
        let mut dessin = Dessin::default();
        for sens in [Sens::Est, Sens::Sud, Sens::Ouest, Sens::Sud, Sens::Ouest] {
            dessin.ajouter_cote(sens);
        }
        assert!(!dessin.fermee);
        assert!(dessin.fermer());
        assert_eq!(dessin.cotes.len(), 6);
        inscrire(&mut dessin, &["20", "8", "11", "10"]);
        match dessin.resoudre(Format::PiPo).etat {
            Etat::Complete { aire, .. } => assert!((aire - 250.0 * 144.0).abs() < 1e-6),
            autre => panic!("{autre:?}"),
        }
    }

    #[test]
    fn surface_des_murs_moins_les_ouvertures() {
        let mut dessin = Dessin::default();
        dessin.ajouter_trace(&contour(&[
            (0.0, 0.0),
            (100.0, 0.0),
            (100.0, 60.0),
            (0.0, 60.0),
            (0.0, 5.0),
        ]));
        inscrire(&mut dessin, &["12", "10"]);
        assert_eq!(dessin.resoudre(Format::PiPo).murs(), None);
        // 44 pi de périmètre × 8 pi de hauteur = 352 pi².
        dessin.hauteur_des_murs = "8".into();
        let murs = dessin.resoudre(Format::PiPo).murs().unwrap();
        assert!((murs.brute - 352.0 * 144.0).abs() < 1e-6);
        assert_eq!((murs.ouvertures, murs.nette), (0.0, murs.brute));
        // Une porte de 2 pi 8 po × 6 pi 8 po et une fenêtre de 3 pi × 4 pi.
        dessin.ouvertures.push(Ouverture {
            largeur: "2 pi 8 po".into(),
            hauteur: "6 pi 8 po".into(),
            ..Ouverture::default()
        });
        dessin.ouvertures.push(Ouverture {
            largeur: "3".into(),
            hauteur: "4".into(),
            ..Ouverture::default()
        });
        let murs = dessin.resoudre(Format::PiPo).murs().unwrap();
        assert!((murs.ouvertures - (32.0 * 80.0 + 36.0 * 48.0)).abs() < 1e-6);
        assert!((murs.nette - (352.0 * 144.0 - 32.0 * 80.0 - 36.0 * 48.0)).abs() < 1e-6);
        // Une ouverture incomplète, ou une hauteur illisible : pas de surface des murs.
        dessin.ouvertures.push(Ouverture {
            largeur: "3".into(),
            hauteur: String::new(),
            ..Ouverture::default()
        });
        assert_eq!(dessin.resoudre(Format::PiPo).murs(), None);
        dessin.ouvertures.pop();
        dessin.hauteur_des_murs = "abc".into();
        let solution = dessin.resoudre(Format::PiPo);
        assert!(solution.hauteur_invalide && solution.murs().is_none());
    }

    #[test]
    fn un_trait_a_45_degres_devient_un_cote_en_biais() {
        let biais = |b: (f64, f64)| redresser(&[(0.0, 0.0), b]);
        let pas = biais((100.0, 100.0));
        assert_eq!(pas[0].0, Axe::Descendant);
        assert!((pas[0].1 - 100.0 * 2f64.sqrt()).abs() < 1e-9);
        let pas = biais((100.0, -100.0));
        assert_eq!(pas[0].0, Axe::Montant);
        assert!(pas[0].1 > 0.0);
        let pas = biais((-100.0, 100.0));
        assert_eq!(pas[0].0, Axe::Montant);
        assert!(pas[0].1 < 0.0);
        // À 17° de l'horizontale ou à 70°, le trait reste d'équerre.
        assert_eq!(biais((100.0, 30.0))[0].0, Axe::Horizontal);
        assert_eq!(biais((36.0, 100.0))[0].0, Axe::Vertical);
    }

    fn piece(cotes: &[(Sens, f64, &str)]) -> Dessin {
        Dessin {
            cotes: cotes
                .iter()
                .map(|(sens, croquis, saisie)| Cote {
                    sens: *sens,
                    croquis: *croquis,
                    saisie: (*saisie).to_string(),
                })
                .collect(),
            fermee: true,
            ..Dessin::default()
        }
    }

    #[test]
    fn les_murs_exterieurs_a_leur_epaisseur() {
        // Une chambre de 12 pi × 10 pi, une porte de 2 pi 8 po au mur du bas.
        let mut dessin = piece(&[
            (Sens::Est, 200.0, "12"),
            (Sens::Sud, 150.0, "10"),
            (Sens::Ouest, 200.0, ""),
            (Sens::Nord, 150.0, ""),
        ]);
        dessin.ouvertures = vec![Ouverture {
            largeur: "2 pi 8 po".into(),
            hauteur: "6 pi 8 po".into(),
            genre: Some(GenreDOuverture::Porte),
            cote: Some(2),
            position: "1".into(),
            allege: String::new(),
            inversee: false,
        }];
        let solution = dessin.resoudre(Format::PiPo);
        // 9 po par défaut, hors des mesures.
        assert_eq!(solution.epaisseur_des_murs, 9.0);
        assert_eq!(
            dessin.contour_exterieur(&solution).unwrap(),
            vec![
                (-9.0, -9.0),
                (153.0, -9.0),
                (153.0, 129.0),
                (-9.0, 129.0),
                (-9.0, -9.0)
            ]
        );
        // Le mur du bas s'interrompt à la porte : deux morceaux, chacun avec
        // son jambage contre elle.
        let morceaux = dessin.murs_exterieurs(&solution);
        assert_eq!(morceaux.len(), 5);
        let du_bas: Vec<&MorceauDeMur> = morceaux.iter().filter(|m| m.cote == 2).collect();
        assert_eq!(du_bas.len(), 2);
        assert!(du_bas[0].jambage_a_la_fin && !du_bas[0].jambage_au_debut);
        assert!(du_bas[1].jambage_au_debut && !du_bas[1].jambage_a_la_fin);
        // Une épaisseur sans unité se lit en pouces; nulle, pas de mur.
        dessin.epaisseur_des_murs = "6".into();
        assert_eq!(dessin.resoudre(Format::PiPo).epaisseur_des_murs, 6.0);
        dessin.epaisseur_des_murs = "0".into();
        let solution = dessin.resoudre(Format::PiPo);
        assert!(dessin.murs_exterieurs(&solution).is_empty());
        // Les cloisons prennent l'épaisseur que la pièce leur donne, sauf la leur.
        dessin.cloisons = vec![
            crate::cloison::Cloison {
                longueur: "10".into(),
                ..Default::default()
            },
            crate::cloison::Cloison {
                longueur: "10".into(),
                epaisseur: "4".into(),
                ..Default::default()
            },
        ];
        let solution = dessin.resoudre(Format::PiPo);
        assert_eq!(solution.cloisons[0].as_ref().unwrap().epaisseur, 5.0);
        dessin.epaisseur_des_cloisons = "6 po".into();
        let solution = dessin.resoudre(Format::PiPo);
        assert_eq!(solution.cloisons[0].as_ref().unwrap().epaisseur, 6.0);
        assert_eq!(solution.cloisons[1].as_ref().unwrap().epaisseur, 4.0);
    }

    #[test]
    fn la_porte_d_une_cloison() {
        let mut dessin = piece(&[
            (Sens::Est, 200.0, "12"),
            (Sens::Sud, 150.0, "10"),
            (Sens::Ouest, 200.0, ""),
            (Sens::Nord, 150.0, ""),
        ]);
        dessin.hauteur_des_murs = "8".into();
        dessin.cloisons = vec![crate::cloison::Cloison {
            debut: (48.0, 0.0),
            sens: Sens::Sud,
            longueur: "10".into(),
            porte: Some(crate::cloison::PorteDeCloison::default()),
            ..Default::default()
        }];
        let solution = dessin.resoudre(Format::PiPo);
        // 2 pi 8 po, centrée.
        let porte = solution.cloisons[0].as_ref().unwrap().ouvertures[0];
        assert_eq!((porte.debut, porte.fin), (44.0, 76.0));
        // Les deux faces de la cloison, moins la porte (6 pi 8 po de haut).
        assert_eq!(
            solution.surface_des_cloisons(),
            Some(2.0 * (120.0 * 96.0 - 32.0 * 80.0))
        );
        // Une largeur sans unité se lit en pouces; la distance au bout A se lit
        // comme les autres mesures.
        let inscrite = dessin.cloisons[0].porte.as_mut().unwrap();
        inscrite.largeur = "36".into();
        inscrite.position = "1".into();
        let porte = dessin.resoudre(Format::PiPo).cloisons[0]
            .as_ref()
            .unwrap()
            .ouvertures[0];
        assert_eq!((porte.debut, porte.fin), (12.0, 48.0));
        // Trop loin, elle s'arrête au bout de la cloison.
        dessin.cloisons[0].porte.as_mut().unwrap().position = "20".into();
        let porte = dessin.resoudre(Format::PiPo).cloisons[0]
            .as_ref()
            .unwrap()
            .ouvertures[0];
        assert_eq!((porte.debut, porte.fin), (84.0, 120.0));
    }

    #[test]
    fn des_mesures_prises_a_l_exterieur_des_murs() {
        let mut dessin = piece(&[
            (Sens::Est, 200.0, "12"),
            (Sens::Sud, 150.0, "10"),
            (Sens::Ouest, 200.0, ""),
            (Sens::Nord, 150.0, ""),
        ]);
        dessin.mesures_exterieures = true;
        let solution = dessin.resoudre(Format::PiPo);
        // La pièce utile perd 9 po de chaque côté : 10 pi 6 po × 8 pi 6 po.
        assert_eq!(
            solution.etat,
            Etat::Complete {
                aire: 126.0 * 102.0,
                perimetre: 2.0 * (126.0 + 102.0)
            }
        );
        assert_eq!(solution.aire_exterieure, Some(144.0 * 120.0));
        // Les murs vont vers l'intérieur : la ligne tracée est leur face
        // extérieure.
        assert_eq!(dessin.contour_exterieur(&solution).unwrap()[0], (0.0, 0.0));
        assert_eq!(dessin.contour_interieur(&solution).unwrap()[0], (9.0, 9.0));
        let morceaux = dessin.murs_exterieurs(&solution);
        assert_eq!(morceaux.len(), 4);
        assert_eq!(morceaux[0].coins[0], (9.0, 9.0));
        assert_eq!(morceaux[0].coins[3], (0.0, 0.0));
        // Les mesures de l'intérieur ne changent rien à la surface.
        dessin.mesures_exterieures = false;
        let solution = dessin.resoudre(Format::PiPo);
        assert_eq!(solution.aire_exterieure, None);
        assert!(matches!(solution.etat, Etat::Complete { aire, .. } if aire == 144.0 * 120.0));
    }

    #[test]
    fn etirer_un_mur_par_son_milieu() {
        // Une chambre de 12 pi × 10 pi; le mur de droite recule de 1 pi.
        let mut dessin = piece(&[
            (Sens::Est, 200.0, "12"),
            (Sens::Sud, 150.0, "10"),
            (Sens::Ouest, 200.0, ""),
            (Sens::Nord, 150.0, ""),
        ]);
        dessin.objets = vec![crate::mobilier::Objet::nouveau(
            crate::mobilier::GenreDObjet::Lit,
            (100.0, 40.0),
        )];
        dessin.ouvertures = vec![Ouverture {
            largeur: "3 pi".into(),
            hauteur: "4 pi".into(),
            genre: Some(GenreDOuverture::Fenetre),
            cote: Some(2),
            position: "2".into(),
            allege: String::new(),
            inversee: false,
        }];
        let solution = dessin.resoudre(Format::PiPo);
        let ecrire = |pouces: f64| format!("{pouces} po");
        let etire = dessin.etirer(1, 12.0, &solution, &ecrire).unwrap();
        // Le haut s'allonge de 1 pi; le bas, déduit, suit.
        assert_eq!(etire.cotes[0].saisie, "156 po");
        assert_eq!(etire.cotes[2].saisie, "");
        let s = etire.resoudre(Format::PiPo);
        assert!(
            matches!(s.etat, Etat::Complete { aire, .. } if (aire - 156.0 * 120.0).abs() < 1e-9)
        );
        // La fenêtre du bas reste en place : sa distance au coin A grandit de 1 pi.
        assert_eq!(etire.ouvertures[0].position, "36 po");
        // Le lit ne bouge pas.
        assert_eq!(etire.objets[0].centre, (100.0, 40.0));
        // Le mur du haut monte de 6 po : le reste garde sa place.
        let etire = dessin.etirer(0, 6.0, &solution, &ecrire).unwrap();
        assert_eq!(etire.cotes[1].saisie, "126 po");
        assert_eq!(etire.objets[0].centre, (100.0, 46.0));
        // Un mur ne passe pas par-dessus son vis-à-vis.
        assert!(dessin.etirer(1, -150.0, &solution, &ecrire).is_none());
    }

    #[test]
    fn une_cloison_en_polyligne_et_sa_porte() {
        let mut dessin = piece(&[
            (Sens::Est, 200.0, "12"),
            (Sens::Sud, 150.0, "10"),
            (Sens::Ouest, 200.0, ""),
            (Sens::Nord, 150.0, ""),
        ]);
        dessin.hauteur_des_murs = "8".into();
        // Une cloison en L : 5 pi vers l'est, puis 70 po vers le sud; sa porte
        // sur le second segment, centrée.
        dessin.cloisons = vec![crate::cloison::Cloison {
            debut: (0.0, 50.0),
            sens: Sens::Est,
            longueur: "5".into(),
            suite: vec![crate::cloison::SegmentDeCloison {
                sens: Sens::Sud,
                longueur: "70 po".into(),
            }],
            porte: Some(crate::cloison::PorteDeCloison {
                segment: 1,
                ..Default::default()
            }),
            ..Default::default()
        }];
        let solution = dessin.resoudre(Format::PiPo);
        let cloison = solution.cloisons[0].as_ref().unwrap();
        assert_eq!(
            cloison.sommets,
            vec![(0.0, 50.0), (60.0, 50.0), (60.0, 120.0)]
        );
        assert!((solution.longueur_des_cloisons() - 130.0).abs() < 1e-9);
        let porte = cloison.ouvertures[0];
        assert_eq!((porte.segment, porte.debut, porte.fin), (1, 19.0, 51.0));
        // Les deux faces, moins la porte.
        assert_eq!(
            solution.surface_des_cloisons(),
            Some(2.0 * (130.0 * 96.0 - 32.0 * 80.0))
        );
        // Un segment illisible cache la cloison.
        dessin.cloisons[0].suite[0].longueur = "?".into();
        assert!(dessin.resoudre(Format::PiPo).cloisons[0].is_none());
    }

    #[test]
    fn un_coin_coupe_a_45_degres() {
        // Un rectangle de 12 pi × 10 pi dont un coin est coupé de 2 pi × 2 pi.
        let mut dessin = piece(&[
            (Sens::Est, 120.0, "12"),
            (Sens::Sud, 80.0, "8"),
            (Sens::SudOuest, 28.0, ""),
            (Sens::Ouest, 100.0, "10"),
            (Sens::Nord, 100.0, "10"),
        ]);
        let solution = dessin.resoudre(Format::PiPo);
        let biais = solution.mesures[2].pouces().unwrap();
        assert!((biais - 24.0 * 2f64.sqrt()).abs() < 1e-6, "{biais}");
        match solution.etat {
            Etat::Complete { aire, perimetre } => {
                assert!((aire - (144.0 * 120.0 - 24.0 * 24.0 / 2.0)).abs() < 1e-6);
                assert!((perimetre - (480.0 + 24.0 * 2f64.sqrt())).abs() < 1e-6);
            }
            autre => panic!("{autre:?}"),
        }
        // Le côté en biais mesuré, c'est un côté d'équerre qui se déduit.
        dessin.cotes[2].saisie = "2 pi 9,941 po".into();
        dessin.cotes[3].saisie.clear();
        let solution = dessin.resoudre(Format::PiPo);
        assert!((solution.mesures[3].pouces().unwrap() - 120.0).abs() < 0.01);
        // Des mesures qui ne permettent pas de fermer la forme.
        dessin.cotes[2].saisie = "5".into();
        dessin.cotes[3].saisie = "10".into();
        assert!(matches!(
            dessin.resoudre(Format::PiPo).etat,
            Etat::NeFermePas(_)
        ));
    }

    #[test]
    fn fermer_par_un_cote_en_biais() {
        // Est, Sud, Ouest, puis un trait en biais vers le départ : la fermeture
        // le prolonge jusqu'au départ.
        let mut dessin = Dessin::default();
        for (a, b) in [
            ((0.0, 0.0), (100.0, 0.0)),
            ((100.0, 0.0), (100.0, 60.0)),
            ((100.0, 60.0), (60.0, 60.0)),
            ((60.0, 60.0), (30.0, 30.0)),
        ] {
            dessin.ajouter_trace(&trait_tremble(a, b));
        }
        assert!(!dessin.fermee);
        assert!(dessin.fermer());
        assert_eq!(dessin.cotes.len(), 4);
        assert_eq!(dessin.cotes[3].sens, Sens::NordOuest);
        let fin = *dessin.sommets_du_croquis().last().unwrap();
        assert!(fin.0.abs() < 1e-6 && fin.1.abs() < 1e-6);
        // Un triangle aux flèches : Est, Sud, puis en biais vers le haut à gauche.
        let mut triangle = Dessin::default();
        for sens in [Sens::Est, Sens::Sud, Sens::NordOuest] {
            triangle.ajouter_cote(sens);
        }
        assert!(triangle.fermee || triangle.fermer());
        assert_eq!(triangle.cotes.len(), 3);
        inscrire(&mut triangle, &["10", "10"]);
        match triangle.resoudre(Format::PiPo).etat {
            Etat::Complete { aire, .. } => assert!((aire - 50.0 * 144.0).abs() < 1e-6),
            autre => panic!("{autre:?}"),
        }
    }

    #[test]
    fn la_fermeture_n_invente_pas_de_biais() {
        // Est, Sud, Ouest : l'écart avec le départ est à 45°, mais le retour se
        // fait d'équerre, comme avant les murs en biais.
        let mut dessin = Dessin::default();
        for (a, b) in [
            ((0.0, 0.0), (100.0, 0.0)),
            ((100.0, 0.0), (100.0, 60.0)),
            ((100.0, 60.0), (60.0, 60.0)),
        ] {
            dessin.ajouter_trace(&trait_tremble(a, b));
        }
        assert!(dessin.fermer());
        assert!(dessin.cotes.iter().all(|cote| !cote.sens.en_biais()));
    }

    #[test]
    fn les_directions_libres() {
        // Un multiple de 45° redonne l'une des huit directions.
        assert_eq!(Sens::d_angle(45.0), Sens::SudEst);
        assert_eq!(Sens::d_angle(-90.0), Sens::Nord);
        assert_eq!(Sens::d_angle(720.0), Sens::Est);
        // Tout autre angle est libre, ramené à son tour, au millième de degré.
        assert_eq!(Sens::d_angle(30.0), Sens::Libre(30_000));
        assert_eq!(Sens::d_angle(390.0004), Sens::Libre(30_000));
        assert_eq!(Sens::d_angle(-22.5), Sens::Libre(337_500));
        assert!((Sens::Libre(30_000).angle() - 30.0).abs() < 1e-12);
        let (x, y) = Sens::Libre(30_000).pas();
        assert!((x - 3f64.sqrt() / 2.0).abs() < 1e-12 && (y - 0.5).abs() < 1e-12);
        // Son axe, son miroir, et le virage d'une direction à l'autre.
        assert_eq!(Sens::Libre(210_000).axe(), Axe::Libre(30_000));
        assert_eq!(Sens::Libre(30_000).axe(), Sens::Libre(210_000).axe());
        assert_eq!(Sens::Libre(30_000).en_miroir(), Sens::Libre(150_000));
        assert!(Sens::Libre(30_000).en_biais() && Sens::Libre(30_000).libre());
        assert!(!Sens::SudEst.libre() && Sens::SudEst.en_biais());
        assert!((Sens::Sud.virage_depuis(Sens::Est) - 90.0).abs() < 1e-12);
        assert!((Sens::Est.virage_depuis(Sens::Sud) + 90.0).abs() < 1e-12);
        assert!((Sens::Libre(10_000).virage_depuis(Sens::Libre(350_000)) - 20.0).abs() < 1e-9);
        // Lu d'un fichier sans être ramené : le même axe.
        assert_eq!(Sens::Libre(390_000).axe(), Axe::Libre(30_000));
        assert_eq!(Sens::Libre(405_000).axe(), Axe::Descendant);
    }

    #[test]
    fn un_mur_a_60_degres_du_precedent() {
        // Un rectangle dont le deuxième côté tourne de 60° au lieu de 90° : un
        // trapèze. Le premier côté et le dernier sont mesurés; les deux autres
        // se déduisent de la fermeture.
        let mut dessin = piece(&[
            (Sens::Est, 100.0, "12'"),
            (Sens::Sud, 80.0, ""),
            (Sens::Ouest, 100.0, ""),
            (Sens::Nord, 80.0, "8'"),
        ]);
        assert_eq!(dessin.virage(0), Some(90.0));
        assert_eq!(dessin.virage(1), Some(90.0));
        assert!(dessin.orienter(1, 60.0));
        assert_eq!(dessin.cotes[1].sens, Sens::Libre(60_000));
        assert!((dessin.virage(1).unwrap() - 60.0).abs() < 1e-9);
        assert!((dessin.virage(2).unwrap() - 120.0).abs() < 1e-9);
        // Le croquis reste fermé.
        let (dx, dy) = dessin.fin();
        assert!(dx.hypot(dy) < 1e-9, "{dx} {dy}");
        let solution = dessin.resoudre(Format::PiPo);
        let cote_2 = 96.0 / 60f64.to_radians().sin();
        let cote_3 = 144.0 + cote_2 * 0.5;
        let Etat::Complete { aire, perimetre } = solution.etat else {
            panic!("{:?}", solution.etat);
        };
        assert!(matches!(solution.mesures[1], Mesure::Deduite(x) if (x - cote_2).abs() < 1e-6));
        assert!(matches!(solution.mesures[2], Mesure::Deduite(x) if (x - cote_3).abs() < 1e-6));
        assert!((aire - (144.0 + cote_3) / 2.0 * 96.0).abs() < 1e-6);
        assert!((perimetre - (144.0 + cote_2 + cote_3 + 96.0)).abs() < 1e-6);
        // Le même angle ne change rien; un angle hors de 0 à 180 est refusé.
        assert!(!dessin.orienter(1, 60.0));
        assert!(!dessin.orienter(1, 0.0) && !dessin.orienter(1, 180.0));
        assert!(!dessin.orienter(1, f64::NAN));
        // Le miroir garde la surface.
        let mut miroir = dessin.clone();
        miroir.miroir();
        assert_eq!(miroir.cotes[1].sens, Sens::Libre(120_000));
        let Etat::Complete {
            aire: aire_du_miroir,
            ..
        } = miroir.resoudre(Format::PiPo).etat
        else {
            panic!("miroir");
        };
        assert!((aire_du_miroir - aire).abs() < 1e-6);
        // Revenir à 90° redonne le rectangle.
        assert!(dessin.orienter(1, 90.0));
        assert_eq!(dessin.cotes[1].sens, Sens::Sud);
    }

    #[test]
    fn le_premier_cote_d_une_forme_ouverte_n_a_pas_de_precedent() {
        let mut dessin = Dessin::default();
        dessin.ajouter_cote(Sens::Est);
        dessin.ajouter_cote(Sens::Sud);
        assert_eq!(dessin.virage(0), None);
        assert!(!dessin.orienter(0, 30.0));
        // Le deuxième tourne vers la droite de qui avance : il le garde.
        assert!(dessin.orienter(1, 30.0));
        assert_eq!(dessin.cotes[1].sens, Sens::Libre(30_000));
    }

    #[test]
    fn une_piece_hors_d_equerre_d_apres_ses_diagonales() {
        // 12 × 10 pi, mais la diagonale du coin 1 au coin 3 mesure 15 pi 7 po
        // au lieu de 15 pi 7 7/16 po : un parallélogramme.
        let mut dessin = piece(&[
            (Sens::Est, 120.0, "12'"),
            (Sens::Sud, 100.0, "10'"),
            (Sens::Ouest, 120.0, "12'"),
            (Sens::Nord, 100.0, "10'"),
        ]);
        dessin.diagonales[0] = "15' 7\"".to_string();
        let solution = dessin.resoudre(Format::PiPo);
        let (a, b, p) = (144.0f64, 120.0f64, 187.0f64);
        let angle_b = ((a * a + b * b - p * p) / (2.0 * a * b))
            .acos()
            .to_degrees();
        let Etat::Complete { aire, perimetre } = solution.etat else {
            panic!("{:?}", solution.etat);
        };
        assert!((aire - a * b * angle_b.to_radians().sin()).abs() < 1e-6);
        assert!((perimetre - 528.0).abs() < 1e-6);
        let forme = solution.forme_reelle.clone().expect("vraie forme");
        assert_eq!(dessin.sommets_mesures(&solution), Some(forme.clone()));
        // Les côtés gardent leurs mesures; le premier, sa direction.
        for (k, longueur) in [a, b, a, b].into_iter().enumerate() {
            let (s, t) = (forme[k], forme[k + 1]);
            assert!(((t.0 - s.0).hypot(t.1 - s.1) - longueur).abs() < 1e-9);
        }
        assert!(forme[1].1.abs() < 1e-12 && forme[1].0 > 0.0);
        let coins = angles_des_coins(&forme);
        for (k, attendu) in [180.0 - angle_b, angle_b, 180.0 - angle_b, angle_b]
            .into_iter()
            .enumerate()
        {
            assert!((coins[k] - attendu).abs() < 1e-9, "coin {k} : {coins:?}");
        }
        // Un peu plus de 1/4 po sur 4 pi à chaque coin.
        let ecart = ecart_a_l_equerre(coins[0], 48.0);
        assert!((ecart - 48.0 * (90.0 - angle_b).to_radians().tan()).abs() < 1e-12);
        assert!((ecart - 0.232).abs() < 0.001, "{ecart}");
        assert_eq!(solution.ecart_des_diagonales, None);
        // L'autre diagonale seule donne la même forme.
        let q = (2.0 * (a * a + b * b) - p * p).sqrt();
        let mut par_l_autre = dessin.clone();
        par_l_autre.diagonales = [String::new(), format!("{q} po")];
        let autre = par_l_autre
            .resoudre(Format::PiPo)
            .forme_reelle
            .expect("forme");
        for (s, t) in autre.iter().zip(&forme) {
            assert!(
                (s.0 - t.0).abs() < 1e-6 && (s.1 - t.1).abs() < 1e-6,
                "{autre:?}"
            );
        }
        // Les deux : la forme suit la première, et l'écart avec la seconde se lit.
        let mut les_deux = dessin.clone();
        les_deux.diagonales[1] = format!("{} po", q + 0.5);
        let ecart = les_deux.resoudre(Format::PiPo).ecart_des_diagonales;
        assert!(ecart.is_some_and(|e| (e - 0.5).abs() < 1e-6), "{ecart:?}");
        // Une forme tirée de ses diagonales ne s'étire pas.
        assert!(
            dessin
                .etirer(1, 6.0, &solution, &|x| format!("{x}"))
                .is_none()
        );
    }

    #[test]
    fn des_diagonales_qui_ne_vont_pas() {
        let mut dessin = piece(&[
            (Sens::Est, 120.0, "12'"),
            (Sens::Sud, 100.0, "10'"),
            (Sens::Ouest, 120.0, "12'"),
            (Sens::Nord, 100.0, "10'"),
        ]);
        // Plus longue que deux côtés mis bout à bout.
        dessin.diagonales[0] = "23'".to_string();
        assert_eq!(
            dessin.resoudre(Format::PiPo).etat,
            Etat::DiagonaleImpossible(0)
        );
        dessin.diagonales = [String::new(), "1'".to_string()];
        assert_eq!(
            dessin.resoudre(Format::PiPo).etat,
            Etat::DiagonaleImpossible(1)
        );
        // Illisible.
        dessin.diagonales = ["abc".to_string(), String::new()];
        assert_eq!(dessin.resoudre(Format::PiPo).etat, Etat::MesureInvalide);
        // Un côté sans mesure : la diagonale attend, la fermeture le déduit.
        dessin.diagonales = ["15' 7\"".to_string(), String::new()];
        dessin.cotes[3].saisie.clear();
        let solution = dessin.resoudre(Format::PiPo);
        assert!(matches!(solution.etat, Etat::Complete { .. }));
        assert_eq!(solution.forme_reelle, None);
        assert_eq!(solution.diagonales[0], Mesure::Lue(187.0));
    }

    #[test]
    fn un_trapeze_retrouve_par_sa_diagonale() {
        // Le trapèze d'un rectangle dont le deuxième côté tourne de 60°,
        // croqué comme un rectangle : sa diagonale le redonne.
        let cote_2 = 96.0 / 60f64.to_radians().sin();
        let cote_3 = 144.0 + cote_2 * 0.5;
        let mut dessin = piece(&[
            (Sens::Est, 120.0, "144 po"),
            (Sens::Sud, 100.0, &format!("{cote_2} po")),
            (Sens::Ouest, 120.0, &format!("{cote_3} po")),
            (Sens::Nord, 100.0, "96 po"),
        ]);
        dessin.diagonales[0] = format!("{} po", cote_3.hypot(96.0));
        let forme = dessin.resoudre(Format::PiPo).forme_reelle.expect("forme");
        let coins = angles_des_coins(&forme);
        for (k, attendu) in [90.0, 120.0, 60.0, 90.0].into_iter().enumerate() {
            assert!((coins[k] - attendu).abs() < 1e-6, "{coins:?}");
        }
        // Le croquis en miroir donne la forme en miroir, aux mêmes angles.
        let mut miroir = dessin.clone();
        miroir.miroir();
        let forme_miroir = miroir.resoudre(Format::PiPo).forme_reelle.expect("miroir");
        for (s, t) in forme_miroir.iter().zip(&forme) {
            assert!((s.0 + t.0).abs() < 1e-6 && (s.1 - t.1).abs() < 1e-6);
        }
        assert!(ecart_a_l_equerre(90.0, 48.0).abs() < 1e-12);
        assert!(
            (ecart_a_l_equerre(90.0 + (0.375f64 / 48.0).atan().to_degrees(), 48.0) - 0.375).abs()
                < 1e-12
        );
    }

    #[test]
    fn une_piece_sous_un_plafond_cathedrale() {
        use crate::plafond::{FormeDePlafond, Plafond, Retombee};
        // 12 × 10 pi, murs de 8 pi, faîte à 12 pi parallèle au premier côté.
        let mut dessin = piece(&[
            (Sens::Est, 120.0, "12'"),
            (Sens::Sud, 100.0, "10'"),
            (Sens::Ouest, 120.0, ""),
            (Sens::Nord, 100.0, ""),
        ]);
        dessin.hauteur_des_murs = "8'".to_string();
        dessin.epaisseur_des_murs = "0".to_string();
        dessin.plafond = Plafond {
            forme: FormeDePlafond::Cathedrale,
            haut: "12'".to_string(),
            cote: 0,
        };
        let solution = dessin.resoudre(Format::PiPo);
        let (pi2, pi3) = (144.0, 1728.0);
        assert!((solution.volume.unwrap() - 1200.0 * pi3).abs() < 1e-6);
        assert!((solution.surface_du_plafond.unwrap() - 120.0 * pi2 * 1.64f64.sqrt()).abs() < 1e-6);
        // Deux murs de 96 pi² et deux pignons de 100 pi² : 392 pi².
        let murs = solution.murs().unwrap();
        assert!(
            (murs.brute - 392.0 * pi2).abs() < 1e-6,
            "{}",
            murs.brute / pi2
        );
        // Une retombée de 4 × 2 pi qui descend de 1 pi.
        dessin.retombees.push(Retombee {
            longueur: "4'".to_string(),
            largeur: "2'".to_string(),
            hauteur: "1'".to_string(),
        });
        let solution = dessin.resoudre(Format::PiPo);
        assert!((solution.volume.unwrap() - 1192.0 * pi3).abs() < 1e-6);
        let attendue = 120.0 * pi2 * 1.64f64.sqrt() + 12.0 * pi2;
        assert!((solution.surface_du_plafond.unwrap() - attendue).abs() < 1e-6);
        // Le pignon monte au faîte dans la 3D.
        let maquette = crate::maquette::maquette(&dessin, &solution).unwrap();
        let plus_haut = maquette
            .faces
            .iter()
            .filter(|face| face.genre == crate::maquette::GenreDeFace::Mur)
            .flat_map(|face| face.sommets.iter().map(|s| s[1]))
            .fold(0.0, f64::max);
        assert!((plus_haut - 144.0).abs() < 1e-9, "{plus_haut}");
        // Une retombée incomplète : ni volume ni plafond.
        dessin.retombees[0].hauteur.clear();
        let solution = dessin.resoudre(Format::PiPo);
        assert_eq!((solution.volume, solution.surface_du_plafond), (None, None));
        assert!(solution.murs().is_some());
        // Sans hauteur au faîte : le plafond reste plat pour la 3D, et rien
        // ne se calcule.
        dessin.retombees.clear();
        dessin.plafond.haut.clear();
        let solution = dessin.resoudre(Format::PiPo);
        assert!(solution.plafond_incomplet && !solution.plafond_invalide);
        assert_eq!(
            solution.plafond.map(|p| p.forme),
            Some(FormeDePlafond::Plat)
        );
        assert!(solution.murs().is_none() && solution.volume.is_none());
        // Un plafond plat : comme avant, périmètre × hauteur.
        dessin.plafond.forme = FormeDePlafond::Plat;
        let solution = dessin.resoudre(Format::PiPo);
        assert!((solution.murs().unwrap().brute - 44.0 * 12.0 * 96.0).abs() < 1e-6);
        assert!((solution.volume.unwrap() - 960.0 * pi3).abs() < 1e-6);
    }

    #[test]
    fn les_chaines_de_cotes() {
        use crate::cloison::Cloison;
        // 12 × 10 pi : une porte de 32 po à 2 pi du coin 1 sur le premier mur,
        // une fenêtre de 36 po à 30 po du coin 2 sur le deuxième.
        let mut dessin = piece(&[
            (Sens::Est, 120.0, "12'"),
            (Sens::Sud, 100.0, "10'"),
            (Sens::Ouest, 120.0, ""),
            (Sens::Nord, 100.0, ""),
        ]);
        dessin.ouvertures = vec![
            Ouverture {
                largeur: "32 po".to_string(),
                hauteur: "80 po".to_string(),
                genre: Some(GenreDOuverture::Porte),
                cote: Some(0),
                position: "2'".to_string(),
                ..Ouverture::default()
            },
            Ouverture {
                largeur: "36 po".to_string(),
                hauteur: "48 po".to_string(),
                genre: Some(GenreDOuverture::Fenetre),
                cote: Some(1),
                position: "30 po".to_string(),
                ..Ouverture::default()
            },
        ];
        let solution = dessin.resoudre(Format::PiPo);
        let chaines = dessin.chaines_de_cotes(&solution);
        assert_eq!(chaines.len(), 2, "{chaines:?}");
        assert_eq!(chaines[0].points, vec![0.0, 24.0, 56.0, 144.0]);
        assert_eq!(chaines[0].debut, (0.0, 0.0));
        assert_eq!(chaines[0].dehors, (0.0, -1.0));
        assert_eq!(chaines[1].points, vec![0.0, 30.0, 66.0, 120.0]);
        // Deux cloisons parallèles depuis le mur de gauche, de 5 po : leurs
        // faces sur ce mur, et leur écart de face à face.
        let cloison = |y: f64, longueur: &str| Cloison {
            debut: (0.0, y),
            longueur: longueur.to_string(),
            epaisseur: "5 po".to_string(),
            ..Cloison::default()
        };
        dessin.cloisons = vec![cloison(48.0, "4'"), cloison(96.0, "5'")];
        let solution = dessin.resoudre(Format::PiPo);
        let chaines = dessin.chaines_de_cotes(&solution);
        assert_eq!(chaines.len(), 4, "{chaines:?}");
        // Le mur de gauche va de (0, 120) à (0, 0).
        assert_eq!(chaines[2].points, vec![0.0, 21.5, 26.5, 69.5, 74.5, 120.0]);
        let ecart = &chaines[3];
        assert_eq!(ecart.points, vec![0.0, 43.0]);
        assert_eq!((ecart.debut, ecart.fin), ((24.0, 50.5), (24.0, 93.5)));
        assert_eq!(ecart.dehors, (0.0, 0.0));
        // Une troisième entre les deux : chacune se cote avec sa voisine.
        dessin.cloisons.push(cloison(72.0, "4'"));
        let solution = dessin.resoudre(Format::PiPo);
        let ecarts: Vec<f64> = dessin
            .chaines_de_cotes(&solution)
            .iter()
            .filter(|chaine| chaine.dehors == (0.0, 0.0))
            .map(|chaine| chaine.points[1])
            .collect();
        assert_eq!(ecarts, vec![19.0, 19.0]);
        // Une pièce incomplète n'a pas de cotes.
        dessin.cotes[0].saisie.clear();
        assert!(
            dessin
                .chaines_de_cotes(&dessin.resoudre(Format::PiPo))
                .is_empty()
        );
    }

    #[test]
    fn deux_cotes_inconnus_presque_paralleles() {
        // Deux côtés inconnus à un dixième de degré l'un de l'autre : la
        // fermeture ne les départage pas, il manque une mesure.
        let dessin = piece(&[
            (Sens::Est, 100.0, "10'"),
            (Sens::Libre(89_900), 80.0, ""),
            (Sens::Ouest, 100.0, "10'"),
            (Sens::Nord, 80.0, ""),
        ]);
        assert_eq!(dessin.resoudre(Format::PiPo).etat, Etat::Incomplete(1));
    }

    #[test]
    fn des_cotes_en_biais_qui_se_croisent() {
        // Un « sablier » : les deux côtés en biais se croisent.
        let dessin = piece(&[
            (Sens::Est, 100.0, "10"),
            (Sens::SudOuest, 141.0, "14,1421"),
            (Sens::Est, 100.0, "10"),
            (Sens::NordOuest, 141.0, ""),
        ]);
        assert_eq!(dessin.resoudre(Format::PiPo).etat, Etat::SeCroise);
    }

    #[test]
    fn les_ouvertures_se_placent_sur_leur_mur() {
        let mut dessin = piece(&[
            (Sens::Est, 120.0, "12"),
            (Sens::Sud, 100.0, "10"),
            (Sens::Ouest, 120.0, ""),
            (Sens::Nord, 100.0, ""),
        ]);
        let ouverture =
            |largeur: &str, hauteur: &str, cote: Option<usize>, position: &str| Ouverture {
                largeur: largeur.into(),
                hauteur: hauteur.into(),
                genre: None,
                cote,
                position: position.into(),
                allege: String::new(),
                inversee: false,
            };
        dessin.ouvertures = vec![
            // Une porte centrée sur le premier mur : (144 − 32) / 2 = 56 po.
            ouverture("2 pi 8 po", "6 pi 8 po", Some(0), ""),
            // Une fenêtre à 2 pi du coin, sur le deuxième mur.
            ouverture("3", "4", Some(1), "2"),
            // Contre le coin, puis trop loin : elle reste dans son mur.
            ouverture("3", "4", Some(2), "0"),
            ouverture("3", "4", Some(3), "20"),
            // Pas placée; plus large que son mur; sur un mur qui n'existe pas.
            ouverture("3", "4", None, ""),
            ouverture("13", "4", Some(1), ""),
            ouverture("3", "4", Some(9), ""),
        ];
        let p = dessin.resoudre(Format::PiPo).placements;
        assert_eq!(
            p[0],
            Some(Placement {
                cote: 0,
                debut: 56.0,
                fin: 88.0,
                porte: true,
                genre: GenreDOuverture::Porte,
                inversee: false,
                bas: 0.0,
                haut: 80.0
            })
        );
        assert_eq!(
            p[1],
            Some(Placement {
                cote: 1,
                debut: 24.0,
                fin: 60.0,
                porte: false,
                genre: GenreDOuverture::Fenetre,
                inversee: false,
                bas: 36.0,
                haut: 84.0
            })
        );
        assert_eq!(p[2].map(|p| (p.debut, p.fin)), Some((0.0, 36.0)));
        assert_eq!(p[3].map(|p| (p.debut, p.fin)), Some((84.0, 120.0)));
        assert_eq!(&p[4..], &[None, None, None]);
        // Le genre inscrit l'emporte sur la hauteur.
        dessin.ouvertures[0].genre = Some(GenreDOuverture::Fenetre);
        assert!(!dessin.resoudre(Format::PiPo).placements[0].unwrap().porte);
        // Tant que la longueur d'un mur n'est pas connue, rien n'y est dessiné.
        dessin.cotes[0].saisie.clear();
        dessin.cotes[2].saisie.clear();
        assert_eq!(dessin.resoudre(Format::PiPo).placements[0], None);
    }

    #[test]
    fn retirer_et_effacer() {
        let mut dessin = Dessin::default();
        assert!(!dessin.fermer());
        dessin.ajouter_trace(&trait_tremble((0.0, 0.0), (100.0, 0.0)));
        dessin.ajouter_trace(&trait_tremble((100.0, 0.0), (100.0, 50.0)));
        // Un trait dans le même sens allonge le côté; en sens contraire, il le raccourcit.
        dessin.ajouter_trace(&trait_tremble((100.0, 50.0), (100.0, 90.0)));
        assert_eq!(dessin.cotes.len(), 2);
        assert!((dessin.cotes[1].croquis - 90.0).abs() < 3.0);
        assert!(dessin.fermer());
        assert_eq!(dessin.cotes.len(), 4);
        // Une forme fermée n'accepte plus de tracé.
        assert!(!dessin.ajouter_trace(&trait_tremble((0.0, 0.0), (50.0, 0.0))));
        dessin.retirer_le_dernier();
        assert!(!dessin.fermee);
        assert_eq!(dessin.cotes.len(), 3);
        dessin.effacer();
        assert!(dessin.est_vide());
        assert_eq!(dessin.resoudre(Format::PiPo).etat, Etat::Vide);
    }
    #[test]
    fn une_porte_qui_frappe_un_lit_et_un_lit_qui_traverse_un_mur() {
        // Une pièce de 12 pi × 10 pi, une porte de 32 po à 1 pi du coin, sur
        // le mur du haut.
        let mut dessin = Dessin::default();
        for (sens, saisie) in [
            (Sens::Est, "12"),
            (Sens::Sud, "10"),
            (Sens::Ouest, ""),
            (Sens::Nord, ""),
        ] {
            dessin.cotes.push(Cote {
                sens,
                croquis: 100.0,
                saisie: saisie.to_string(),
            });
        }
        dessin.fermee = true;
        dessin.ouvertures.push(Ouverture {
            largeur: "32 po".to_string(),
            hauteur: "80 po".to_string(),
            genre: Some(GenreDOuverture::Porte),
            cote: Some(0),
            position: "1".to_string(),
            ..Ouverture::default()
        });
        // Un lit sous le battant, et une table bien à l'écart.
        dessin.objets = vec![
            crate::mobilier::Objet::nouveau(crate::mobilier::GenreDObjet::Lit, (40.0, 50.0)),
            crate::mobilier::Objet::nouveau(crate::mobilier::GenreDObjet::Table, (110.0, 80.0)),
        ];
        let solution = dessin.resoudre(Format::PiPo);
        let battants = dessin.battants_des_portes(&solution);
        assert_eq!(battants.len(), 1);
        // Le battant : la charnière à 12 po du coin, 32 po vers l'intérieur.
        let (raison, polygone) = &battants[0];
        assert_eq!(*raison, RaisonDuConflit::PorteDuMur(0));
        assert!(
            polygone
                .iter()
                .any(|p| (p.0 - 12.0).abs() < 1e-9 && (p.1 - 32.0).abs() < 1e-9)
        );
        assert_eq!(
            dessin.conflits(&solution),
            vec![Conflit {
                objet: 0,
                raison: RaisonDuConflit::PorteDuMur(0),
            }]
        );
        // Le lit poussé à travers le mur de gauche, contre la table : deux
        // conflits de plus.
        dessin.objets[0].centre = (10.0, 90.0);
        dessin.objets[1].centre = (40.0, 90.0);
        let solution = dessin.resoudre(Format::PiPo);
        let conflits = dessin.conflits(&solution);
        assert!(conflits.contains(&Conflit {
            objet: 0,
            raison: RaisonDuConflit::Mur,
        }));
        assert!(conflits.contains(&Conflit {
            objet: 0,
            raison: RaisonDuConflit::Objet(1),
        }));
    }
    #[test]
    fn un_croquis_a_main_levee() {
        // Un trait presque droit s'allège; un coin reste.
        let presque_droit: Vec<(f64, f64)> = (0..=20)
            .map(|k| (f64::from(k), if k % 2 == 0 { 0.0 } else { 0.1 }))
            .collect();
        assert_eq!(
            simplifier_le_trait(&presque_droit, 0.5),
            vec![(0.0, 0.0), (20.0, 0.0)]
        );
        let coin = [
            (0.0, 0.0),
            (5.0, 0.0),
            (10.0, 0.0),
            (10.0, 5.0),
            (10.0, 10.0),
        ];
        assert_eq!(
            simplifier_le_trait(&coin, 0.5),
            vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0)]
        );
        // Un dessin d'avant n'a pas de croquis; le miroir le retourne aussi.
        let mut dessin = Dessin::default();
        assert!(dessin.croquis.is_empty());
        dessin.croquis.push(TraitAMainLevee {
            points: vec![(10.0, 5.0), (30.0, 5.0)],
            crayon: Crayon::Surligneur,
        });
        dessin.miroir();
        assert_eq!(dessin.croquis[0].points, vec![(-10.0, 5.0), (-30.0, 5.0)]);
    }
    #[test]
    fn les_genres_d_ouverture_et_le_bordereau() {
        // Les genres : ce qui part du plancher, ce qui bat, ce qui a un sens.
        assert!(GenreDOuverture::BaieLibre.est_une_porte());
        assert!(!GenreDOuverture::FenetreEnBaie.est_une_porte());
        assert!(GenreDOuverture::PorteDouble.battante());
        assert!(!GenreDOuverture::PorteCoulissante.battante());
        assert_eq!(GenreDOuverture::BaieLibre.remplissage(), Remplissage::Rien);
        assert_eq!(
            GenreDOuverture::PorteFenetre.remplissage(),
            Remplissage::Vitre
        );
        for genre in GenreDOuverture::TOUS {
            assert!(!genre.formats().is_empty());
        }
        // Une pièce de 12 pi × 10 pi : une porte sur le mur du haut, une
        // porte double et une fenêtre.
        let mut dessin = Dessin::default();
        for (sens, saisie) in [
            (Sens::Est, "12"),
            (Sens::Sud, "10"),
            (Sens::Ouest, ""),
            (Sens::Nord, ""),
        ] {
            dessin.cotes.push(Cote {
                sens,
                croquis: 100.0,
                saisie: saisie.to_string(),
            });
        }
        dessin.fermee = true;
        let ouverture = |genre, largeur: &str, cote, position: &str, inversee| Ouverture {
            largeur: largeur.to_string(),
            hauteur: "6 pi 8 po".to_string(),
            genre: Some(genre),
            cote: Some(cote),
            position: position.to_string(),
            inversee,
            ..Ouverture::default()
        };
        dessin.ouvertures = vec![
            ouverture(GenreDOuverture::Fenetre, "3", 1, "", false),
            ouverture(GenreDOuverture::Porte, "32 po", 0, "1", false),
            ouverture(GenreDOuverture::PorteDouble, "5", 2, "", false),
        ];
        dessin.ouvertures[0].hauteur = "4".to_string();
        let solution = dessin.resoudre(Format::PiPo);
        assert_eq!(
            solution.placements[2].unwrap().genre,
            GenreDOuverture::PorteDouble
        );
        // Deux battants pour la porte double, un pour la porte.
        assert_eq!(dessin.battants_des_portes(&solution).len(), 3);
        let plan = Plan::d_une_piece(dessin.clone());
        assert_eq!(plan.reperes_des_ouvertures(), vec![vec!["F1", "P1", "P2"]]);
        let bordereau = plan.bordereau(Format::PiPo);
        let reperes: Vec<&str> = bordereau.iter().map(|l| l.repere.as_str()).collect();
        assert_eq!(reperes, vec!["P1", "P2", "F1"]);
        // La porte du mur du haut, sa charnière à l'ouest : vu du nord, en la
        // poussant, la charnière est à droite.
        assert_eq!(bordereau[0].poussant_gauche, Some(false));
        assert_eq!(bordereau[0].largeur, Some(32.0));
        assert_eq!(bordereau[2].hauteur, Some(48.0));
        assert_eq!(bordereau[1].poussant_gauche, None);
        // Inversée, elle devient poussant gauche, et son battant part de l'autre bout.
        let mut inversee = dessin.clone();
        inversee.ouvertures[1].inversee = true;
        let plan = Plan::d_une_piece(inversee.clone());
        assert_eq!(plan.bordereau(Format::PiPo)[0].poussant_gauche, Some(true));
        let solution = inversee.resoudre(Format::PiPo);
        let (_, battant) = &inversee.battants_des_portes(&solution)[0];
        assert!(battant[0].0 > 40.0);
        // Le symbole d'une porte coulissante dans un mur de 9 po : deux
        // panneaux; d'une baie libre : un trait de tirets.
        let coulissante = symbole_d_ouverture(
            GenreDOuverture::PorteCoulissante,
            false,
            ((0.0, 0.0), (60.0, 0.0)),
            (0.0, 1.0),
            9.0,
            (0.0, 1.0),
        );
        assert_eq!(coulissante.len(), 2);
        let baie = symbole_d_ouverture(
            GenreDOuverture::BaieLibre,
            false,
            ((0.0, 0.0), (36.0, 0.0)),
            (0.0, 1.0),
            9.0,
            (0.0, 1.0),
        );
        assert_eq!(baie.len(), 1);
        assert_eq!(baie[0].style, StyleDOuverture::Tirets);
    }
    #[test]
    fn les_elevations_des_murs() {
        let mut dessin = Dessin::default();
        for (sens, saisie) in [
            (Sens::Est, "12"),
            (Sens::Sud, "10"),
            (Sens::Ouest, ""),
            (Sens::Nord, ""),
        ] {
            dessin.cotes.push(Cote {
                sens,
                croquis: 100.0,
                saisie: saisie.to_string(),
            });
        }
        dessin.fermee = true;
        dessin.hauteur_des_murs = "8".into();
        dessin.ouvertures = vec![
            Ouverture {
                largeur: "32 po".into(),
                hauteur: "80 po".into(),
                genre: Some(GenreDOuverture::Porte),
                cote: Some(0),
                position: "1".into(),
                ..Ouverture::default()
            },
            Ouverture {
                largeur: "3".into(),
                hauteur: "4".into(),
                genre: Some(GenreDOuverture::Fenetre),
                cote: Some(1),
                ..Ouverture::default()
            },
        ];
        let elevations = dessin.elevations(&dessin.resoudre(Format::PiPo));
        assert_eq!(elevations.len(), 4);
        // Le premier mur : 12 pi de long, 8 pi de haut, la porte à 1 pi du coin.
        let mur = &elevations[0];
        assert_eq!(
            (mur.longueur, mur.haut.clone()),
            (144.0, vec![(0.0, 96.0), (144.0, 96.0)])
        );
        assert_eq!(mur.ouvertures.len(), 1);
        assert_eq!(
            (mur.ouvertures[0].debut, mur.ouvertures[0].fin),
            (12.0, 44.0)
        );
        assert_eq!(mur.chaine(), vec![0.0, 12.0, 44.0, 144.0]);
        // La fenêtre, centrée sur le deuxième mur, à son allège de 3 pi.
        let fenetre = elevations[1].ouvertures[0];
        assert_eq!(
            (fenetre.debut, fenetre.bas, fenetre.haut),
            (42.0, 36.0, 84.0)
        );
        // Sous un plafond cathédrale, les murs en pignon montent au faîte.
        dessin.plafond = crate::plafond::Plafond {
            forme: crate::plafond::FormeDePlafond::Cathedrale,
            haut: "12".into(),
            ..Default::default()
        };
        let elevations = dessin.elevations(&dessin.resoudre(Format::PiPo));
        assert!(
            elevations
                .iter()
                .any(|mur| mur.haut.len() == 3 && (mur.plus_haut() - 144.0).abs() < 1e-6)
        );
    }
}
