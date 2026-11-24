//! Les espaces d'une pièce (évolution 5 de la section 27) : ce que ses
//! cloisons y ferment — une chambre et sa garde-robe, par exemple. Ce sont
//! les faces du graphe que forment la face intérieure des murs et l'axe des
//! cloisons, coupés à leurs croisements; l'aire de chacune se compte jusqu'à
//! la face des cloisons.

use serde::{Deserialize, Serialize};

use crate::cloison::CloisonMesuree;

/// La distance (pouces) sous laquelle deux points se confondent, et à laquelle
/// le bout d'une cloison touche un mur ou une autre cloison.
const TOLERANCE: f64 = 0.75;

/// Un segment du graphe : ses deux bouts, et sa demi-épaisseur (nulle pour
/// un mur, dont on suit la face; celle d'une cloison, dont on suit l'axe).
type SegmentDuGraphe = ((f64, f64), (f64, f64), f64);

/// Un espace fermé de la pièce.
#[derive(Clone, Debug, PartialEq)]
pub struct Espace {
    /// Son contour (pouces, sur le plan), le long des murs et de l'axe des
    /// cloisons.
    pub contour: Vec<(f64, f64)>,
    /// Son aire nette (pouces carrés), jusqu'à la face des cloisons.
    pub aire: f64,
    /// Un point bien à l'intérieur, pour y écrire son nom.
    pub repere: (f64, f64),
}

/// Le nom donné à un espace : il s'attache au point où on l'a nommé, et va à
/// l'espace qui le contient.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct NomDEspace {
    pub point: (f64, f64),
    pub nom: String,
}

/// Les espaces que les cloisons ferment dans la pièce de `contour` (la face
/// intérieure de ses murs). Vide s'il n'y en a qu'un : la pièce elle-même.
pub fn espaces(contour: &[(f64, f64)], cloisons: &[CloisonMesuree]) -> Vec<Espace> {
    let mut contour = contour.to_vec();
    if contour.len() > 1 && contour.first() == contour.last() {
        contour.pop();
    }
    if contour.len() < 3 || cloisons.is_empty() {
        return Vec::new();
    }
    // Les segments : les murs, puis l'axe des cloisons et leur demi-épaisseur.
    let mut segments: Vec<SegmentDuGraphe> = (0..contour.len())
        .map(|k| (contour[k], contour[(k + 1) % contour.len()], 0.0))
        .collect();
    for cloison in cloisons {
        for paire in cloison.sommets.windows(2) {
            segments.push((paire[0], paire[1], cloison.epaisseur / 2.0));
        }
    }
    // Chaque segment se coupe là où un autre le croise ou le touche.
    let mut coupures: Vec<Vec<f64>> = vec![vec![0.0, 1.0]; segments.len()];
    for i in 0..segments.len() {
        for j in i + 1..segments.len() {
            let (a0, a1, _) = segments[i];
            let (b0, b1, _) = segments[j];
            for (bout, sur) in [(b0, i), (b1, i), (a0, j), (a1, j)] {
                let (s0, s1, _) = segments[sur];
                if let Some(t) = projection_proche(bout, s0, s1) {
                    coupures[sur].push(t);
                }
            }
            if let Some((t, u)) = croisement(a0, a1, b0, b1) {
                coupures[i].push(t);
                coupures[j].push(u);
            }
        }
    }
    // Les sommets du graphe, confondus à la tolérance près, et ses arêtes.
    let mut sommets: Vec<(f64, f64)> = Vec::new();
    let mut sommet = |p: (f64, f64)| -> usize {
        match sommets
            .iter()
            .position(|q| (q.0 - p.0).hypot(q.1 - p.1) < TOLERANCE)
        {
            Some(rang) => rang,
            None => {
                sommets.push(p);
                sommets.len() - 1
            }
        }
    };
    let mut aretes: Vec<(usize, usize, f64)> = Vec::new();
    for (rang, &(a, b, demi)) in segments.iter().enumerate() {
        let mut ts = coupures[rang].clone();
        ts.sort_by(f64::total_cmp);
        ts.dedup_by(|x, y| (*x - *y).abs() < 1e-9);
        let point = |t: f64| (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
        for paire in ts.windows(2) {
            let (u, v) = (sommet(point(paire[0])), sommet(point(paire[1])));
            if u != v
                && !aretes
                    .iter()
                    .any(|&(x, y, _)| (x, y) == (u, v) || (x, y) == (v, u))
            {
                aretes.push((u, v, demi));
            }
        }
    }
    // Ce qui pend (une cloison qui ne ferme rien) ne borde aucun espace.
    loop {
        let mut degre = vec![0_usize; sommets.len()];
        for &(u, v, _) in &aretes {
            degre[u] += 1;
            degre[v] += 1;
        }
        let avant = aretes.len();
        aretes.retain(|&(u, v, _)| degre[u] > 1 && degre[v] > 1);
        if aretes.len() == avant {
            break;
        }
    }
    // Les voisins de chaque sommet, dans l'ordre de leur angle.
    let mut voisins: Vec<Vec<usize>> = vec![Vec::new(); sommets.len()];
    for &(u, v, _) in &aretes {
        voisins[u].push(v);
        voisins[v].push(u);
    }
    for (s, liste) in voisins.iter_mut().enumerate() {
        let centre = sommets[s];
        liste.sort_by(|&a, &b| {
            let angle = |p: (f64, f64)| (p.1 - centre.1).atan2(p.0 - centre.0);
            angle(sommets[a]).total_cmp(&angle(sommets[b]))
        });
    }
    // Les faces : chaque demi-arête, suivie en tournant toujours du même côté.
    let mut vue = std::collections::HashSet::new();
    let mut faces: Vec<Vec<usize>> = Vec::new();
    for &(u0, v0, _) in &aretes {
        for depart in [(u0, v0), (v0, u0)] {
            if vue.contains(&depart) {
                continue;
            }
            let mut face = Vec::new();
            let (mut u, mut v) = depart;
            loop {
                if !vue.insert((u, v)) {
                    break;
                }
                face.push(u);
                let liste = &voisins[v];
                let ici = liste.iter().position(|&w| w == u).unwrap_or(0);
                let suivant = liste[(ici + liste.len() - 1) % liste.len()];
                (u, v) = (v, suivant);
                if (u, v) == depart {
                    break;
                }
            }
            if face.len() >= 3 {
                faces.push(face);
            }
        }
    }
    // La face du dehors est la plus grande; les autres sont les espaces.
    let aire_signee = |face: &[usize]| {
        let n = face.len();
        (0..n)
            .map(|k| {
                let (a, b) = (sommets[face[k]], sommets[face[(k + 1) % n]]);
                a.0 * b.1 - b.0 * a.1
            })
            .sum::<f64>()
            / 2.0
    };
    let Some(dehors) = (0..faces.len()).max_by(|&a, &b| {
        aire_signee(&faces[a])
            .abs()
            .total_cmp(&aire_signee(&faces[b]).abs())
    }) else {
        return Vec::new();
    };
    let demi_epaisseur = |u: usize, v: usize| {
        aretes
            .iter()
            .find(|&&(x, y, _)| (x, y) == (u, v) || (x, y) == (v, u))
            .map_or(0.0, |&(_, _, demi)| demi)
    };
    let mut resultat: Vec<Espace> = faces
        .iter()
        .enumerate()
        .filter(|&(rang, face)| rang != dehors && aire_signee(face).abs() > 1.0)
        .map(|(_, face)| {
            let contour: Vec<(f64, f64)> = face.iter().map(|&s| sommets[s]).collect();
            // L'aire jusqu'à la face des cloisons : la moitié de leur épaisseur
            // en moins, le long de chaque arête qui en est une.
            let n = face.len();
            let retrait: f64 = (0..n)
                .map(|k| {
                    let (u, v) = (face[k], face[(k + 1) % n]);
                    let (a, b) = (sommets[u], sommets[v]);
                    (b.0 - a.0).hypot(b.1 - a.1) * demi_epaisseur(u, v)
                })
                .sum();
            Espace {
                repere: point_interieur(&contour),
                aire: (aire_signee(face).abs() - retrait).max(0.0),
                contour,
            }
        })
        .collect();
    if resultat.len() < 2 {
        return Vec::new();
    }
    // De haut en bas, puis de gauche à droite.
    resultat.sort_by(|a, b| {
        (a.repere.1 / 12.0)
            .round()
            .total_cmp(&(b.repere.1 / 12.0).round())
            .then(a.repere.0.total_cmp(&b.repere.0))
    });
    resultat
}

/// Où le point `p` se projette sur le segment `ab`, en part de sa longueur,
/// s'il en est à la tolérance près sans être à l'un de ses bouts.
fn projection_proche(p: (f64, f64), a: (f64, f64), b: (f64, f64)) -> Option<f64> {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let longueur2 = dx * dx + dy * dy;
    if longueur2 < 1e-12 {
        return None;
    }
    let t = ((p.0 - a.0) * dx + (p.1 - a.1) * dy) / longueur2;
    let longueur = longueur2.sqrt();
    if t * longueur <= TOLERANCE || (1.0 - t) * longueur <= TOLERANCE {
        return None;
    }
    let distance = (p.0 - a.0 - dx * t).hypot(p.1 - a.1 - dy * t);
    (distance < TOLERANCE).then_some(t)
}

/// Où deux segments se croisent franchement : leurs deux parts.
fn croisement(
    a0: (f64, f64),
    a1: (f64, f64),
    b0: (f64, f64),
    b1: (f64, f64),
) -> Option<(f64, f64)> {
    let (r, s) = ((a1.0 - a0.0, a1.1 - a0.1), (b1.0 - b0.0, b1.1 - b0.1));
    let denominateur = r.0 * s.1 - r.1 * s.0;
    if denominateur.abs() < 1e-9 {
        return None;
    }
    let (qx, qy) = (b0.0 - a0.0, b0.1 - a0.1);
    let t = (qx * s.1 - qy * s.0) / denominateur;
    let u = (qx * r.1 - qy * r.0) / denominateur;
    let marge = 1e-6;
    (t > marge && t < 1.0 - marge && u > marge && u < 1.0 - marge).then_some((t, u))
}

/// Vrai si le point est dans le polygone (règle pair-impair).
pub fn dedans((x, y): (f64, f64), polygone: &[(f64, f64)]) -> bool {
    let n = polygone.len();
    let mut dedans = false;
    for k in 0..n {
        let (a, b) = (polygone[k], polygone[(k + 1) % n]);
        if (a.1 > y) != (b.1 > y) && x < a.0 + (y - a.1) * (b.0 - a.0) / (b.1 - a.1) {
            dedans = !dedans;
        }
    }
    dedans
}

/// Un point bien à l'intérieur d'un polygone, même concave : parmi une grille
/// de points, celui qui est le plus loin de ses côtés. Le nom d'une pièce
/// en L s'y écrit, dans l'étage, au lieu du milieu de son étendue.
pub fn point_interieur(polygone: &[(f64, f64)]) -> (f64, f64) {
    let (x0, x1, y0, y1) = polygone.iter().fold(
        (f64::MAX, f64::MIN, f64::MAX, f64::MIN),
        |(x0, x1, y0, y1), &(x, y)| (x0.min(x), x1.max(x), y0.min(y), y1.max(y)),
    );
    let distance_aux_cotes = |p: (f64, f64)| {
        let n = polygone.len();
        (0..n)
            .map(|k| {
                let (a, b) = (polygone[k], polygone[(k + 1) % n]);
                let (dx, dy) = (b.0 - a.0, b.1 - a.1);
                let longueur2 = (dx * dx + dy * dy).max(1e-12);
                let t = (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / longueur2).clamp(0.0, 1.0);
                (p.0 - a.0 - dx * t).hypot(p.1 - a.1 - dy * t)
            })
            .fold(f64::MAX, f64::min)
    };
    let mut meilleur = ((x0 + x1) / 2.0, (y0 + y1) / 2.0);
    let mut plus_loin = f64::MIN;
    for i in 0..=16 {
        for j in 0..=16 {
            let p = (
                x0 + (x1 - x0) * f64::from(i) / 16.0,
                y0 + (y1 - y0) * f64::from(j) / 16.0,
            );
            if dedans(p, polygone) {
                let d = distance_aux_cotes(p);
                if d > plus_loin {
                    plus_loin = d;
                    meilleur = p;
                }
            }
        }
    }
    meilleur
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Une pièce de 12 pi × 10 pi (pouces, y vers le bas).
    const PIECE: [(f64, f64); 4] = [(0.0, 0.0), (144.0, 0.0), (144.0, 120.0), (0.0, 120.0)];

    fn cloison(sommets: &[(f64, f64)]) -> CloisonMesuree {
        CloisonMesuree {
            sommets: sommets.to_vec(),
            epaisseur: 4.0,
            ouvertures: Vec::new(),
        }
    }

    #[test]
    fn une_cloison_droite_ferme_deux_espaces() {
        // D'un mur à l'autre, à 4 pi du mur de gauche.
        let espaces = espaces(&PIECE, &[cloison(&[(48.0, 0.0), (48.0, 120.0)])]);
        assert_eq!(espaces.len(), 2);
        let mut aires: Vec<f64> = espaces.iter().map(|e| e.aire).collect();
        aires.sort_by(f64::total_cmp);
        // Chacun perd 2 po de cloison sur 10 pi.
        assert!((aires[0] - (48.0 * 120.0 - 2.0 * 120.0)).abs() < 1e-6);
        assert!((aires[1] - (96.0 * 120.0 - 2.0 * 120.0)).abs() < 1e-6);
        // Le repère de chacun est en lui.
        for espace in &espaces {
            assert!(dedans(espace.repere, &espace.contour));
        }
        // Une cloison qui ne touche qu'un mur ne ferme rien.
        assert!(espaces_de(&[cloison(&[(48.0, 0.0), (48.0, 60.0)])]).is_empty());
    }

    fn espaces_de(cloisons: &[CloisonMesuree]) -> Vec<Espace> {
        espaces(&PIECE, cloisons)
    }

    #[test]
    fn des_cloisons_en_l_en_t_a_45_degres_et_en_croix() {
        // En L : une garde-robe de 4 pi × 3 pi dans le coin.
        let l = espaces_de(&[cloison(&[(48.0, 0.0), (48.0, 36.0), (0.0, 36.0)])]);
        assert_eq!(l.len(), 2);
        let petite = l.iter().map(|e| e.aire).fold(f64::MAX, f64::min);
        assert!((petite - (48.0 * 36.0 - 2.0 * (36.0 + 48.0))).abs() < 1e-6);
        // En T : une cloison d'un mur à l'autre, et une autre qui la rejoint.
        let t = espaces_de(&[
            cloison(&[(72.0, 0.0), (72.0, 120.0)]),
            cloison(&[(0.0, 60.0), (72.0, 60.0)]),
        ]);
        assert_eq!(t.len(), 3);
        // À 45° : le coin coupé en triangle.
        let biais = espaces_de(&[cloison(&[(48.0, 0.0), (0.0, 48.0)])]);
        assert_eq!(biais.len(), 2);
        let triangle = biais.iter().map(|e| e.aire).fold(f64::MAX, f64::min);
        let longueur = 48.0 * 2.0_f64.sqrt();
        assert!((triangle - (48.0 * 48.0 / 2.0 - 2.0 * longueur)).abs() < 1e-6);
        // En croix : quatre espaces.
        let croix = espaces_de(&[
            cloison(&[(72.0, 0.0), (72.0, 120.0)]),
            cloison(&[(0.0, 60.0), (144.0, 60.0)]),
        ]);
        assert_eq!(croix.len(), 4);
        let total: f64 = croix.iter().map(|e| e.aire).sum();
        // Toute la pièce, moins les cloisons (le croisement compté deux fois,
        // un écart de 4 po² au plus).
        let cloisons = 4.0 * (120.0 + 144.0);
        assert!((total - (144.0 * 120.0 - cloisons)).abs() < 8.0);
        // Rangés de haut en bas, puis de gauche à droite.
        assert!(croix[0].repere.0 < croix[1].repere.0 && croix[0].repere.1 < croix[2].repere.1);
    }
}
