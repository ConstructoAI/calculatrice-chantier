//! Dessin d'une pièce (section 25) : un croquis tracé à main levée devient une
//! forme aux angles d'équerre, dont chaque côté reçoit ensuite sa mesure.
//!
//! Le croquis n'est pas à l'échelle : seules comptent la direction de chaque
//! côté et les mesures inscrites. Quand il ne manque qu'un côté dans un sens
//! (gauche-droite ou haut-bas), il se déduit des autres.

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

/// Direction d'un côté, comme sur l'écran : l'est à droite, le sud en bas.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Sens {
    Est,
    Sud,
    Ouest,
    Nord,
}

impl Sens {
    /// Vrai pour un côté horizontal.
    pub fn horizontal(self) -> bool {
        matches!(self, Sens::Est | Sens::Ouest)
    }

    /// Déplacement unitaire (x vers la droite, y vers le bas).
    pub fn pas(self) -> (f64, f64) {
        match self {
            Sens::Est => (1.0, 0.0),
            Sens::Sud => (0.0, 1.0),
            Sens::Ouest => (-1.0, 0.0),
            Sens::Nord => (0.0, -1.0),
        }
    }

    fn signe(self) -> f64 {
        match self {
            Sens::Est | Sens::Sud => 1.0,
            Sens::Ouest | Sens::Nord => -1.0,
        }
    }

    fn selon(horizontal: bool, longueur_signee: f64) -> Sens {
        match (horizontal, longueur_signee >= 0.0) {
            (true, true) => Sens::Est,
            (true, false) => Sens::Ouest,
            (false, true) => Sens::Sud,
            (false, false) => Sens::Nord,
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

/// La forme dessinée : ses côtés dans l'ordre du tracé.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Dessin {
    pub cotes: Vec<Cote>,
    /// Vrai quand le dernier côté rejoint le premier.
    pub fermee: bool,
    /// Ce qui se retire de la surface : un îlot, une trémie, une colonne.
    pub retraits: Vec<Retrait>,
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
}

impl Solution {
    /// Somme des retraits, s'ils sont tous lisibles.
    pub fn aire_des_retraits(&self) -> Option<f64> {
        self.retraits.iter().copied().sum()
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
/// l'outil, puis le rang de ces deux champs.
pub fn outils_d_une_piece() -> Vec<(&'static Calculateur, usize, usize)> {
    let rang = |c: &Calculateur, nom: &str| {
        c.champs.iter().position(|champ| {
            champ.libelle.fr == nom && matches!(champ.nature, Nature::Longueur(_))
        })
    };
    Metier::TOUS
        .iter()
        .filter(|metier| **metier != Metier::Acier)
        .flat_map(|metier| metier.calculateurs())
        .filter_map(|c| Some((c, rang(c, "Longueur")?, rang(c, "Largeur")?)))
        .collect()
}

/// Redresse un tracé à main levée : une suite de déplacements horizontaux et
/// verticaux, chacun avec sa longueur signée (positive vers la droite ou vers
/// le bas). Les tremblements sont écartés.
pub fn redresser(points: &[(f64, f64)]) -> Vec<(bool, f64)> {
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
    let mut pas: Vec<(bool, f64)> = Vec::new();
    for paire in sommets.windows(2) {
        let (dx, dy) = (paire[1].0 - paire[0].0, paire[1].1 - paire[0].1);
        let horizontal = dx.abs() >= dy.abs();
        pas.push((horizontal, if horizontal { dx } else { dy }));
    }
    let pas = fusionner(pas);
    let plus_long = pas.iter().map(|(_, l)| l.abs()).fold(0.0, f64::max);
    let utiles = pas
        .into_iter()
        .filter(|(_, l)| l.abs() >= plus_long * PART_NEGLIGEABLE)
        .collect();
    fusionner(utiles)
}

/// Réunit les déplacements consécutifs de même sens (horizontal ou vertical).
fn fusionner(pas: Vec<(bool, f64)>) -> Vec<(bool, f64)> {
    let mut fusion: Vec<(bool, f64)> = Vec::new();
    for (horizontal, longueur) in pas {
        match fusion.last_mut() {
            Some((h, l)) if *h == horizontal => *l += longueur,
            _ => fusion.push((horizontal, longueur)),
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
            Some(dernier) if dernier.sens.horizontal() == sens.horizontal() => {
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
        if self.cotes.len() >= 4 {
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
        for (horizontal, longueur) in pas {
            self.ajouter_pas(horizontal, longueur);
        }
        if self.cotes.len() >= 4 {
            let (dx, dy) = self.fin();
            if dx.hypot(dy) <= self.etendue() * PROXIMITE_DE_FERMETURE {
                self.fermer();
            }
        }
        true
    }

    fn ajouter_pas(&mut self, horizontal: bool, longueur: f64) {
        if let Some(dernier) = self.cotes.last_mut()
            && dernier.sens.horizontal() == horizontal
        {
            let somme = dernier.signe() + longueur;
            if somme.abs() > 0.0 {
                dernier.sens = Sens::selon(horizontal, somme);
                dernier.croquis = somme.abs();
            } else {
                self.cotes.pop();
            }
            return;
        }
        self.cotes.push(Cote {
            sens: Sens::selon(horizontal, longueur),
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
        // Le retour au départ : d'abord dans le sens que le dernier côté n'a pas.
        let dernier_horizontal = self.cotes.last().is_some_and(|c| c.sens.horizontal());
        let retours = if dernier_horizontal {
            [(false, -dy), (true, -dx)]
        } else {
            [(true, -dx), (false, -dy)]
        };
        for (horizontal, longueur) in retours {
            if longueur.abs() > seuil {
                self.ajouter_pas(horizontal, longueur);
            }
        }
        // Le dernier côté et le premier se suivent : s'ils vont dans le même
        // sens, ils n'en font qu'un.
        while self.cotes.len() > 2
            && self.cotes[0].sens.horizontal() == self.cotes[self.cotes.len() - 1].sens.horizontal()
        {
            let Some(dernier) = self.cotes.pop() else {
                break;
            };
            let somme = self.cotes[0].signe() + dernier.signe();
            if somme.abs() > 0.0 {
                self.cotes[0].sens = Sens::selon(self.cotes[0].sens.horizontal(), somme);
                self.cotes[0].croquis = somme.abs();
                if self.cotes[0].saisie.is_empty() {
                    self.cotes[0].saisie = dernier.saisie;
                }
            } else {
                self.cotes.remove(0);
            }
        }
        // Le petit écart qui reste est repris par le dernier côté de chaque sens.
        let (dx, dy) = self.fin();
        let mut valide = self.cotes.len() >= 4 && self.cotes.len().is_multiple_of(2);
        for (horizontal, ecart) in [(true, dx), (false, dy)] {
            if let Some(cote) = self
                .cotes
                .iter_mut()
                .rev()
                .find(|c| c.sens.horizontal() == horizontal)
            {
                let corrige = cote.signe() - ecart;
                if corrige.abs() > 0.0 && (corrige > 0.0) == (cote.signe() > 0.0) {
                    cote.croquis = corrige.abs();
                } else {
                    valide = false;
                }
            } else {
                valide = false;
            }
        }
        if !valide {
            *self = avant;
            return false;
        }
        self.fermee = true;
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
        Solution {
            mesures,
            etat,
            modele,
            retraits,
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
        let mut manquantes = 0;
        let mut ecart_maximal: f64 = 0.0;
        for horizontal in [true, false] {
            let rangs: Vec<usize> = (0..self.cotes.len())
                .filter(|&i| self.cotes[i].sens.horizontal() == horizontal)
                .collect();
            let inconnus: Vec<usize> = rangs
                .iter()
                .copied()
                .filter(|&i| mesures[i] == Mesure::Manquante)
                .collect();
            // Somme signée des côtés connus : elle doit s'annuler.
            let somme: f64 = rangs
                .iter()
                .filter_map(|&i| mesures[i].pouces().map(|x| x * self.cotes[i].sens.signe()))
                .sum();
            match inconnus.as_slice() {
                [] => ecart_maximal = ecart_maximal.max(somme.abs()),
                [seul] => {
                    let longueur = -somme * self.cotes[*seul].sens.signe();
                    if longueur > TOLERANCE_PO {
                        mesures[*seul] = Mesure::Deduite(longueur);
                    } else {
                        ecart_maximal = ecart_maximal.max(TOLERANCE_PO.max(-longueur));
                    }
                }
                plusieurs => manquantes += plusieurs.len() - 1,
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
    let dans = |a: f64, b: f64, x: f64| x >= a.min(b) - eps && x <= a.max(b) + eps;
    for i in 0..n {
        for j in (i + 2)..n {
            if i == 0 && j == n - 1 {
                continue;
            }
            let (a, b, c, d) = (sommets[i], sommets[i + 1], sommets[j], sommets[j + 1]);
            // Les boîtes des deux côtés se recouvrent-elles ? Pour des côtés
            // horizontaux ou verticaux, cela suffit.
            let en_x = dans(a.0, b.0, c.0)
                || dans(a.0, b.0, d.0)
                || dans(c.0, d.0, a.0)
                || dans(c.0, d.0, b.0);
            let en_y = dans(a.1, b.1, c.1)
                || dans(a.1, b.1, d.1)
                || dans(c.1, d.1, a.1)
                || dans(c.1, d.1, b.1);
            if en_x && en_y {
                return true;
            }
        }
    }
    false
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
        assert!(pas[0].0, "horizontal");
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
                .any(|(c, l, w)| c.id == "dalle" && *l == 0 && *w == 1)
        );
        assert!(outils.len() >= 8);
        assert!(outils.iter().all(|(c, _, _)| c.metier != Metier::Acier));
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
