//! Dessin d'une pièce (section 25) : un croquis tracé à main levée devient une
//! forme aux angles d'équerre — ou à 45° pour un mur en biais —, dont chaque
//! côté reçoit ensuite sa mesure.
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
/// quatre directions en biais sont à 45°.
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
}

/// L'axe d'un côté : horizontal, vertical, ou l'une des deux diagonales à 45°.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axe {
    Horizontal,
    Vertical,
    /// Du haut à gauche vers le bas à droite.
    Descendant,
    /// Du bas à gauche vers le haut à droite.
    Montant,
}

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
        }
    }

    /// Le sens négatif de l'axe.
    fn negatif(self) -> Sens {
        match self {
            Axe::Horizontal => Sens::Ouest,
            Axe::Vertical => Sens::Nord,
            Axe::Descendant => Sens::NordOuest,
            Axe::Montant => Sens::SudOuest,
        }
    }

    /// Le déplacement unitaire du sens positif.
    pub fn vecteur(self) -> (f64, f64) {
        self.positif().pas()
    }

    /// L'axe d'un déplacement : en biais s'il est à 45° à quelques degrés
    /// près, sinon horizontal ou vertical.
    fn du_deplacement(dx: f64, dy: f64) -> Axe {
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
    /// L'axe du côté.
    pub fn axe(self) -> Axe {
        match self {
            Sens::Est | Sens::Ouest => Axe::Horizontal,
            Sens::Sud | Sens::Nord => Axe::Vertical,
            Sens::SudEst | Sens::NordOuest => Axe::Descendant,
            Sens::NordEst | Sens::SudOuest => Axe::Montant,
        }
    }

    /// Vrai pour un côté horizontal.
    pub fn horizontal(self) -> bool {
        self.axe() == Axe::Horizontal
    }

    /// Vrai pour un côté en biais, à 45°.
    pub fn en_biais(self) -> bool {
        matches!(self.axe(), Axe::Descendant | Axe::Montant)
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
        }
    }

    /// +1 dans le sens positif de son axe, −1 dans l'autre.
    fn signe(self) -> f64 {
        if self == self.axe().positif() {
            1.0
        } else {
            -1.0
        }
    }

    fn selon(axe: Axe, longueur_signee: f64) -> Sens {
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
}

/// L'allège d'une fenêtre quand elle n'est pas inscrite : 3 pi.
pub const ALLEGE_PAR_DEFAUT: f64 = 36.0;
/// La hauteur d'une porte ou d'une fenêtre quand elle n'est pas lisible.
const HAUTEUR_DE_PORTE_PAR_DEFAUT: f64 = 80.0;
const HAUTEUR_DE_FENETRE_PAR_DEFAUT: f64 = 48.0;

/// Le genre d'une ouverture, pour la dessiner.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GenreDOuverture {
    Porte,
    Fenetre,
}

impl Ouverture {
    /// Vrai pour une porte. Sans genre, une ouverture de 6 pi (72 po) ou plus
    /// de haut en est une; `hauteur` est sa hauteur en pouces, si elle est lisible.
    pub fn est_une_porte(&self, hauteur: Option<f64>) -> bool {
        match self.genre {
            Some(genre) => genre == GenreDOuverture::Porte,
            None => hauteur.is_some_and(|h| h >= 72.0),
        }
    }
}

/// Où se trouve une ouverture, pour la dessiner : son mur, et ses deux bords
/// en pouces depuis le premier coin de ce mur.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placement {
    pub cote: usize,
    pub debut: f64,
    pub fin: f64,
    pub porte: bool,
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

/// Les pièces dessinées : toujours au moins une, dont celle qui est à l'écran.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Plan {
    pub pieces: Vec<Piece>,
    /// Rang de la pièce à l'écran.
    pub courante: usize,
}

impl Default for Plan {
    fn default() -> Self {
        Plan {
            pieces: vec![Piece::default()],
            courante: 0,
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
        }
    }

    /// Remet le plan dans ses règles : au moins une pièce, et une pièce
    /// courante qui existe.
    pub fn normaliser(&mut self) {
        if self.pieces.is_empty() {
            self.pieces.push(Piece::default());
        }
        self.courante = self.courante.min(self.pieces.len() - 1);
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
}

impl Solution {
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
        let Etat::Complete { perimetre, .. } = self.etat else {
            return None;
        };
        let brute = perimetre * self.hauteur?;
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
        if n < 2 {
            return false;
        }
        let (dx, dy) = self.fin();
        let u = self.cotes[n - 2].sens.axe().vecteur();
        let v = self.cotes[n - 1].sens.axe().vecteur();
        let determinant = u.0 * v.1 - u.1 * v.0;
        if determinant.abs() < 1e-9 {
            return false;
        }
        let (wx, wy) = (-dx, -dy);
        let corrections = [
            (n - 2, (wx * v.1 - wy * v.0) / determinant),
            (n - 1, (u.0 * wy - u.1 * wx) / determinant),
        ];
        for (rang, correction) in corrections {
            let avant = self.cotes[rang].signe();
            let apres = avant + correction;
            if apres.abs() <= 0.0 || (apres > 0.0) != (avant > 0.0) {
                return false;
            }
            self.cotes[rang].croquis = apres.abs();
        }
        true
    }

    /// Retire le dernier côté tracé; rouvre d'abord une forme fermée.
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
        let longueurs: Option<Vec<f64>> = solution.mesures.iter().map(|m| m.pouces()).collect();
        let longueurs = longueurs?;
        (longueurs.len() == self.cotes.len()).then(|| self.sommets(&longueurs))
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
                let porte = ouverture.est_une_porte(hauteur);
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
                    bas,
                    haut: bas + hauteur,
                })
            })
            .collect();
        let objets = self
            .objets
            .iter()
            .map(|objet| {
                let (largeur, profondeur, _) = objet.genre.dimensions();
                (
                    lire(&objet.largeur).unwrap_or(largeur),
                    lire(&objet.profondeur).unwrap_or(profondeur),
                )
            })
            .collect();
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
        }
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
            [(a, rangs_a), (b, rangs_b)] => {
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
                let inconnus: usize = groupes.iter().map(|(_, rangs)| rangs.len()).sum();
                manquantes += inconnus - 2;
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
}
