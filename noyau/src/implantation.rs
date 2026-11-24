//! Le plan d'implantation (évolution 29 de la section 27) : le terrain, les
//! marges de recul inscrites par l'utilisateur, la zone où bâtir, et pour
//! chaque objet du terrain ses distances aux limites; la surface du terrain,
//! celle des bâtiments et le taux d'implantation. Aucune règle d'urbanisme
//! n'est vérifiée : les marges sont celles que l'utilisateur inscrit.

use crate::analyse::analyser_longueur;
use crate::dessin::{Dessin, Solution};
use crate::mobilier::{Famille, GenreDObjet};
use crate::unites::Format;

/// La marge de recul de chaque côté du terrain, en pouces (0 sans marge). Un
/// nombre seul est en pieds, comme les autres mesures du dessin.
pub fn marges(dessin: &Dessin) -> Vec<f64> {
    (0..dessin.cotes.len())
        .map(|k| {
            dessin
                .marges_de_recul
                .get(k)
                .and_then(|texte| {
                    analyser_longueur(texte.trim(), Format::PiPo)
                        .ok()
                        .filter(|v| v.est_longueur() && v.base > 0.0 && v.base.is_finite())
                })
                .map_or(0.0, |v| v.base)
        })
        .collect()
}

/// Le contour sans son dernier sommet répété.
fn ouvert(mut contour: Vec<(f64, f64)>) -> Vec<(f64, f64)> {
    if contour.len() > 1 && contour.first() == contour.last() {
        contour.pop();
    }
    contour
}

/// L'aire signée d'un polygone : positive s'il tourne dans le sens des
/// aiguilles d'une montre à l'écran.
fn aire_signee(polygone: &[(f64, f64)]) -> f64 {
    let n = polygone.len();
    (0..n)
        .map(|k| {
            let (a, b) = (polygone[k], polygone[(k + 1) % n]);
            a.0 * b.1 - b.0 * a.1
        })
        .sum::<f64>()
        / 2.0
}

/// La zone où bâtir : le terrain, chaque côté repoussé vers l'intérieur de sa
/// marge de recul. `None` sans terrain complet, sans aucune marge, ou si les
/// marges ne laissent rien.
pub fn zone_constructible(dessin: &Dessin, solution: &Solution) -> Option<Vec<(f64, f64)>> {
    let marges = marges(dessin);
    if marges.iter().all(|&marge| marge <= 0.0) {
        return None;
    }
    let sommets = ouvert(dessin.contour_interieur(solution)?);
    let n = sommets.len();
    if n < 3 || marges.len() < n {
        return None;
    }
    // Une aire signée positive : le dedans est du côté de (−y, x).
    let sens = if aire_signee(&sommets) >= 0.0 {
        1.0
    } else {
        -1.0
    };
    // Chaque côté repoussé : un de ses points, et sa direction.
    let lignes: Vec<((f64, f64), (f64, f64))> = (0..n)
        .map(|k| {
            let (a, b) = (sommets[k], sommets[(k + 1) % n]);
            let longueur = (b.0 - a.0).hypot(b.1 - a.1).max(1e-12);
            let u = ((b.0 - a.0) / longueur, (b.1 - a.1) / longueur);
            let dedans = (-sens * u.1, sens * u.0);
            ((a.0 + dedans.0 * marges[k], a.1 + dedans.1 * marges[k]), u)
        })
        .collect();
    // Chaque coin : la rencontre des deux côtés voisins repoussés.
    let zone: Vec<(f64, f64)> = (0..n)
        .map(|k| {
            let (p, u) = lignes[(k + n - 1) % n];
            let (q, v) = lignes[k];
            let croise = u.0 * v.1 - u.1 * v.0;
            if croise.abs() < 1e-9 {
                q
            } else {
                let t = ((q.0 - p.0) * v.1 - (q.1 - p.1) * v.0) / croise;
                (p.0 + u.0 * t, p.1 + u.1 * t)
            }
        })
        .collect();
    // Des marges plus grandes que le terrain retournent la zone : un de ses
    // côtés va alors à l'envers du côté du terrain, et il n'en reste rien.
    let a_l_endroit = (0..n).all(|k| {
        let (a, b) = (zone[k], zone[(k + 1) % n]);
        let u = lignes[k].1;
        (b.0 - a.0) * u.0 + (b.1 - a.1) * u.1 > 1e-9
    });
    (a_l_endroit && aire_signee(&zone) * aire_signee(&sommets) > 0.0).then_some(zone)
}

/// La distance d'un objet du terrain à un côté : le côté, la distance, le
/// trait qui la mesure (de l'objet au côté), et vrai si l'objet entre dans la
/// marge de recul de ce côté.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DistanceALaLimite {
    pub cote: usize,
    pub distance: f64,
    pub de: (f64, f64),
    pub a: (f64, f64),
    pub dans_la_marge: bool,
}

/// Le point du segment ab le plus proche de p.
fn le_plus_proche(p: (f64, f64), (a, b): ((f64, f64), (f64, f64))) -> (f64, f64) {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let carre = dx * dx + dy * dy;
    let t = if carre < 1e-18 {
        0.0
    } else {
        (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / carre).clamp(0.0, 1.0)
    };
    (a.0 + dx * t, a.1 + dy * t)
}

/// Les distances aux limites de chaque objet du terrain (la famille de
/// l'implantation) : le rang de l'objet, et sa distance à chaque côté.
pub fn distances_aux_limites(
    dessin: &Dessin,
    solution: &Solution,
) -> Vec<(usize, Vec<DistanceALaLimite>)> {
    let Some(sommets) = dessin.contour_interieur(solution).map(ouvert) else {
        return Vec::new();
    };
    let n = sommets.len();
    if n < 3 {
        return Vec::new();
    }
    let marges = marges(dessin);
    dessin
        .objets
        .iter()
        .enumerate()
        .filter(|(_, objet)| objet.genre.famille() == Famille::Implantation)
        .filter_map(|(rang, objet)| {
            let coins = objet.coins(*solution.objets.get(rang)?);
            let distances = (0..n)
                .map(|cote| {
                    let limite = (sommets[cote], sommets[(cote + 1) % n]);
                    // Le plus court trait entre l'objet et le côté : d'un coin
                    // de l'objet au côté, ou d'un bout du côté à l'objet.
                    let mut meilleur = (f64::MAX, (0.0, 0.0), (0.0, 0.0));
                    for &coin in &coins {
                        let pied = le_plus_proche(coin, limite);
                        let d = (pied.0 - coin.0).hypot(pied.1 - coin.1);
                        if d < meilleur.0 {
                            meilleur = (d, coin, pied);
                        }
                    }
                    for bout in [limite.0, limite.1] {
                        for k in 0..4 {
                            let sur_l_objet = le_plus_proche(bout, (coins[k], coins[(k + 1) % 4]));
                            let d = (bout.0 - sur_l_objet.0).hypot(bout.1 - sur_l_objet.1);
                            if d < meilleur.0 {
                                meilleur = (d, sur_l_objet, bout);
                            }
                        }
                    }
                    let (distance, de, a) = meilleur;
                    let marge = marges.get(cote).copied().unwrap_or(0.0);
                    DistanceALaLimite {
                        cote,
                        distance,
                        de,
                        a,
                        dans_la_marge: marge > 0.0 && distance < marge - 1e-6,
                    }
                })
                .collect();
            Some((rang, distances))
        })
        .collect()
}

/// Les distances d'un bâtiment à montrer sur le plan : toutes pour la maison;
/// les deux plus courtes pour un autre bâtiment, que de longs traits à
/// travers le terrain n'éclaireraient pas.
pub fn distances_a_montrer(
    genre: GenreDObjet,
    mut distances: Vec<DistanceALaLimite>,
) -> Vec<DistanceALaLimite> {
    if genre != GenreDObjet::Maison {
        distances.sort_by(|a, b| a.distance.total_cmp(&b.distance));
        distances.truncate(2);
    }
    distances
}

/// Le bilan d'un terrain : sa surface, celle des bâtiments (maison, garage,
/// cabanon) et leur part du terrain, et celle des surfaces dures (terrasse,
/// entrée, piscine), en pouces carrés.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BilanDuTerrain {
    pub terrain: f64,
    pub batiments: f64,
    /// La part du terrain que couvrent les bâtiments, de 0 à 1.
    pub taux: f64,
    pub surfaces_dures: f64,
}

/// Vrai pour un bâtiment du terrain.
pub fn est_un_batiment(genre: GenreDObjet) -> bool {
    matches!(
        genre,
        GenreDObjet::Maison | GenreDObjet::Garage | GenreDObjet::Cabanon
    )
}

/// Le bilan d'un terrain complet.
pub fn bilan(dessin: &Dessin, solution: &Solution) -> Option<BilanDuTerrain> {
    let terrain = solution.aire_nette()?;
    let aire = |filtre: fn(GenreDObjet) -> bool| -> f64 {
        dessin
            .objets
            .iter()
            .zip(&solution.objets)
            .filter(|(objet, _)| filtre(objet.genre))
            .map(|(_, &(largeur, profondeur))| largeur * profondeur)
            .sum()
    };
    let batiments = aire(est_un_batiment);
    let surfaces_dures = aire(|genre| {
        matches!(
            genre,
            GenreDObjet::Terrasse | GenreDObjet::Entree | GenreDObjet::Piscine
        )
    });
    Some(BilanDuTerrain {
        terrain,
        batiments,
        taux: if terrain > 0.0 {
            batiments / terrain
        } else {
            0.0
        },
        surfaces_dures,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dessin::{Cote, Sens};
    use crate::mobilier::Objet;

    /// Un terrain de 60 pi × 100 pi, la rue en haut : 20 pi de recul à
    /// l'avant, 5 pi de chaque côté, 25 pi à l'arrière.
    fn terrain() -> Dessin {
        let cote = |sens, saisie: &str| Cote {
            sens,
            croquis: 100.0,
            saisie: saisie.to_string(),
        };
        Dessin {
            cotes: vec![
                cote(Sens::Est, "60"),
                cote(Sens::Sud, "100"),
                cote(Sens::Ouest, ""),
                cote(Sens::Nord, ""),
            ],
            fermee: true,
            terrain: true,
            marges_de_recul: vec!["20".into(), "5".into(), "25".into(), "5".into()],
            ..Dessin::default()
        }
    }

    #[test]
    fn la_zone_ou_batir() {
        let dessin = terrain();
        let mut dessin = dessin;
        dessin.hauteur_des_murs = "8".into();
        let solution = dessin.resoudre(Format::PiPo);
        // Un terrain n'a pas de murs : ni épaisseur, ni hauteur, ni surface de
        // murs, ni murs en 3D.
        assert_eq!(solution.epaisseur_des_murs, 0.0);
        assert!(solution.hauteur.is_none() && solution.murs().is_none());
        let maquette = crate::maquette::maquette(&dessin, &solution).unwrap();
        assert!(
            maquette
                .faces
                .iter()
                .all(|face| face.genre != crate::maquette::GenreDeFace::Mur)
        );
        assert_eq!(marges(&dessin), vec![240.0, 60.0, 300.0, 60.0]);
        let zone = zone_constructible(&dessin, &solution).unwrap();
        for coin in [(60.0, 240.0), (660.0, 240.0), (660.0, 900.0), (60.0, 900.0)] {
            assert!(
                zone.iter()
                    .any(|p| (p.0 - coin.0).abs() < 1e-6 && (p.1 - coin.1).abs() < 1e-6),
                "{zone:?}"
            );
        }
        // Sans marge, ou avec des marges qui ne laissent rien : aucune zone.
        let mut sans = dessin.clone();
        sans.marges_de_recul.clear();
        assert!(zone_constructible(&sans, &solution).is_none());
        let mut trop = dessin.clone();
        trop.marges_de_recul = vec!["60".into(), "40".into(), "60".into(), "40".into()];
        assert!(zone_constructible(&trop, &solution).is_none());
    }

    #[test]
    fn les_distances_aux_limites_et_le_bilan() {
        let mut dessin = terrain();
        // Une maison de 40 pi × 28 pi, à 19 pi 4 po de la rue : dans la marge
        // d'avant; et un cabanon tout au fond.
        dessin.objets = vec![
            Objet::nouveau(GenreDObjet::Maison, (360.0, 400.0)),
            Objet::nouveau(GenreDObjet::Cabanon, (600.0, 1100.0)),
            Objet::nouveau(GenreDObjet::Lit, (100.0, 100.0)),
        ];
        let solution = dessin.resoudre(Format::PiPo);
        let distances = distances_aux_limites(&dessin, &solution);
        // Le lit n'est pas un objet du terrain.
        assert_eq!(distances.len(), 2);
        let (rang, maison) = &distances[0];
        assert_eq!(*rang, 0);
        let attendues = [
            (232.0, true),
            (120.0, false),
            (632.0, false),
            (120.0, false),
        ];
        for (distance, (attendue, dans_la_marge)) in maison.iter().zip(attendues) {
            assert!((distance.distance - attendue).abs() < 1e-6, "{distance:?}");
            assert_eq!(distance.dans_la_marge, dans_la_marge, "{distance:?}");
        }
        // Le trait d'une distance va de l'objet au côté.
        assert!((maison[0].a.1).abs() < 1e-6 && (maison[0].de.1 - 232.0).abs() < 1e-6);
        let bilan = bilan(&dessin, &solution).unwrap();
        assert!((bilan.terrain - 720.0 * 1200.0).abs() < 1e-6);
        assert!((bilan.batiments - (480.0 * 336.0 + 120.0 * 144.0)).abs() < 1e-6);
        assert!((bilan.taux - bilan.batiments / bilan.terrain).abs() < 1e-12);
        assert_eq!(bilan.surfaces_dures, 0.0);
    }
}
