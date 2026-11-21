//! Maquette 3D d'une pièce (section 25) : le plancher, les murs à leur
//! hauteur, et les portes et les fenêtres dans leurs murs, en faces planes
//! prêtes à projeter. Les coordonnées sont en pouces : x et z sur le plan,
//! y vers le haut.

use crate::dessin::{Dessin, Etat, Solution};
use crate::mobilier::GenreDObjet;

/// Hauteur des murs quand elle n'est pas inscrite : 8 pi.
pub const HAUTEUR_PAR_DEFAUT: f64 = 96.0;

/// Ce que représente une face.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GenreDeFace {
    Plancher,
    Mur,
    Vitre,
    Porte,
    /// Un bloc de l'escalier ou d'un meuble.
    Meuble(GenreDObjet),
    /// Un bloc d'une cloison.
    Cloison,
}

/// Une face plane : ses sommets `[x, y, z]` et sa normale — pour un mur ou
/// une ouverture, la direction de l'intérieur de la pièce; pour un meuble,
/// celle de l'extérieur du meuble.
#[derive(Clone, Debug, PartialEq)]
pub struct Face {
    pub genre: GenreDeFace,
    pub sommets: Vec<[f64; 3]>,
    pub normale: [f64; 3],
}

/// La maquette d'une pièce : ses faces, et les arêtes à tracer par-dessus
/// (le haut, le bas et les coins des murs, le cadre des ouvertures).
#[derive(Clone, Debug, PartialEq)]
pub struct Maquette {
    pub faces: Vec<Face>,
    pub aretes: Vec<[[f64; 3]; 2]>,
    /// Les blocs : les marches, les meubles et les cloisons.
    pub blocs: Vec<Bloc>,
}

/// Un bloc posé sur le plancher (une marche, un meuble, une cloison) : son
/// empreinte (x, z) et ses faces et ses arêtes, qui se suivent dans celles de
/// la maquette. Les blocs se peignent dans l'ordre que donne leur empreinte
/// vue de la caméra (`ordre_des_blocs`).
#[derive(Clone, Debug, PartialEq)]
pub struct Bloc {
    pub coins: [(f64, f64); 4],
    pub faces: std::ops::Range<usize>,
    pub aretes: std::ops::Range<usize>,
}

/// La maquette d'une pièce complète; `None` tant que ses mesures ne la
/// déterminent pas. La hauteur des murs est celle qui est inscrite, sinon
/// 8 pi.
pub fn maquette(dessin: &Dessin, solution: &Solution) -> Option<Maquette> {
    if !matches!(solution.etat, Etat::Complete { .. }) {
        return None;
    }
    let sommets = dessin.sommets_mesures(solution)?;
    let contour = &sommets[..sommets.len().saturating_sub(1)];
    if contour.len() < 3 {
        return None;
    }
    let hauteur = solution.hauteur.unwrap_or(HAUTEUR_PAR_DEFAUT);
    let aire_signee: f64 = sommets
        .windows(2)
        .map(|s| s[0].0 * s[1].1 - s[1].0 * s[0].1)
        .sum();
    let mut faces = Vec::new();
    let mut aretes = Vec::new();

    // Le plancher, en triangles (la pièce peut avoir des coins rentrants).
    for triangle in trianguler(contour) {
        faces.push(Face {
            genre: GenreDeFace::Plancher,
            sommets: triangle.iter().map(|&(x, z)| [x, 0.0, z]).collect(),
            normale: [0.0, 1.0, 0.0],
        });
    }

    // Les murs, interrompus par leurs ouvertures, puis les ouvertures.
    for (cote, segment) in sommets.windows(2).enumerate() {
        let (a, b) = (segment[0], segment[1]);
        let longueur = (b.0 - a.0).hypot(b.1 - a.1);
        if longueur <= 1e-9 {
            continue;
        }
        let u = ((b.0 - a.0) / longueur, (b.1 - a.1) / longueur);
        let dedans = if aire_signee > 0.0 {
            [-u.1, 0.0, u.0]
        } else {
            [u.1, 0.0, -u.0]
        };
        let point = |t: f64, y: f64| [a.0 + u.0 * t, y, a.1 + u.1 * t];
        let rectangle = |t0: f64, t1: f64, y0: f64, y1: f64| {
            vec![point(t0, y0), point(t1, y0), point(t1, y1), point(t0, y1)]
        };
        // Le bas et le haut du mur, et le coin où il commence.
        aretes.push([point(0.0, 0.0), point(longueur, 0.0)]);
        aretes.push([point(0.0, hauteur), point(longueur, hauteur)]);
        aretes.push([point(0.0, 0.0), point(0.0, hauteur)]);
        let ouvertures: Vec<_> = solution
            .placements
            .iter()
            .flatten()
            .filter(|placement| placement.cote == cote)
            .collect();
        // Les bornes le long du mur : ses deux coins et les bords des ouvertures.
        let mut bornes = vec![0.0, longueur];
        for placement in &ouvertures {
            bornes.push(placement.debut.clamp(0.0, longueur));
            bornes.push(placement.fin.clamp(0.0, longueur));
        }
        bornes.sort_by(f64::total_cmp);
        bornes.dedup_by(|x, y| (*x - *y).abs() < 1e-6);
        for paire in bornes.windows(2) {
            let (t0, t1) = (paire[0], paire[1]);
            let milieu = (t0 + t1) / 2.0;
            // Ce que les ouvertures retirent de ce tronçon, en hauteur.
            let mut vides: Vec<(f64, f64)> = ouvertures
                .iter()
                .filter(|placement| placement.debut <= milieu && milieu <= placement.fin)
                .map(|placement| {
                    (
                        placement.bas.clamp(0.0, hauteur),
                        placement.haut.clamp(0.0, hauteur),
                    )
                })
                .filter(|(bas, haut)| haut > bas)
                .collect();
            vides.sort_by(|x, y| x.0.total_cmp(&y.0));
            let mut depuis = 0.0;
            for (bas, haut) in vides.into_iter().chain([(hauteur, hauteur)]) {
                if bas > depuis + 1e-6 {
                    faces.push(Face {
                        genre: GenreDeFace::Mur,
                        sommets: rectangle(t0, t1, depuis, bas),
                        normale: dedans,
                    });
                }
                depuis = f64::max(depuis, haut);
            }
        }
        for placement in &ouvertures {
            let (bas, haut) = (
                placement.bas.clamp(0.0, hauteur),
                placement.haut.clamp(0.0, hauteur),
            );
            if haut > bas {
                let cadre = rectangle(placement.debut, placement.fin, bas, haut);
                for k in 0..4 {
                    aretes.push([cadre[k], cadre[(k + 1) % 4]]);
                }
                faces.push(Face {
                    genre: if placement.porte {
                        GenreDeFace::Porte
                    } else {
                        GenreDeFace::Vitre
                    },
                    sommets: rectangle(placement.debut, placement.fin, bas, haut),
                    normale: dedans,
                });
            }
        }
    }
    // Les objets : des blocs à leur hauteur; l'escalier, une marche par bloc,
    // de la flèche jusqu'au haut des murs.
    let mut blocs = Vec::new();
    for (objet, &dimensions) in dessin.objets.iter().zip(&solution.objets) {
        let genre = GenreDeFace::Meuble(objet.genre);
        let coins = |v0: f64, v1: f64| {
            [(0.0, v0), (1.0, v0), (1.0, v1), (0.0, v1)]
                .map(|coin| objet.vers_le_plan(coin, dimensions))
        };
        if objet.genre == GenreDObjet::Escalier {
            let marches = (dimensions.1 / 10.0).round().clamp(3.0, 20.0) as usize;
            for k in 0..marches {
                let v0 = 1.0 - (k + 1) as f64 / marches as f64;
                let v1 = 1.0 - k as f64 / marches as f64;
                let haut = hauteur * (k + 1) as f64 / marches as f64;
                // Le devant d'une marche est caché par la suivante, plus haute.
                let cache = k + 1 < marches;
                let coins = coins(v0, v1);
                bloc(
                    &mut faces,
                    &mut aretes,
                    &mut blocs,
                    coins,
                    haut,
                    genre,
                    cache,
                );
            }
        } else {
            let (_, _, haut) = objet.genre.dimensions();
            let coins = coins(0.0, 1.0);
            bloc(
                &mut faces,
                &mut aretes,
                &mut blocs,
                coins,
                haut,
                genre,
                false,
            );
        }
    }
    // Les cloisons, jusqu'au haut des murs.
    for cloison in solution.cloisons.iter().flatten() {
        let genre = GenreDeFace::Cloison;
        let coins = cloison.coins();
        bloc(
            &mut faces,
            &mut aretes,
            &mut blocs,
            coins,
            hauteur,
            genre,
            false,
        );
    }
    Some(Maquette {
        faces,
        aretes,
        blocs,
    })
}

/// Un bloc posé sur le plancher, de son empreinte `coins` (x, z) jusqu'à
/// `haut` : son dessus et ses côtés, tournés vers l'extérieur;
/// `sans_l_avant` omet le côté du premier au deuxième coin.
fn bloc(
    faces: &mut Vec<Face>,
    aretes: &mut Vec<[[f64; 3]; 2]>,
    blocs: &mut Vec<Bloc>,
    coins: [(f64, f64); 4],
    haut: f64,
    genre: GenreDeFace,
    sans_l_avant: bool,
) {
    let (premiere_face, premiere_arete) = (faces.len(), aretes.len());
    let bas = coins.map(|(x, z)| [x, 0.0, z]);
    let dessus = coins.map(|(x, z)| [x, haut, z]);
    let centre = (
        coins.iter().map(|c| c.0).sum::<f64>() / 4.0,
        coins.iter().map(|c| c.1).sum::<f64>() / 4.0,
    );
    faces.push(Face {
        genre,
        sommets: dessus.to_vec(),
        normale: [0.0, 1.0, 0.0],
    });
    for k in 0..4 {
        let (a, b) = (k, (k + 1) % 4);
        aretes.push([dessus[a], dessus[b]]);
        if k == 0 && sans_l_avant {
            continue;
        }
        let milieu = (
            (coins[a].0 + coins[b].0) / 2.0,
            (coins[a].1 + coins[b].1) / 2.0,
        );
        let (nx, nz) = (milieu.0 - centre.0, milieu.1 - centre.1);
        let longueur = nx.hypot(nz).max(1e-12);
        faces.push(Face {
            genre,
            sommets: vec![bas[a], bas[b], dessus[b], dessus[a]],
            normale: [nx / longueur, 0.0, nz / longueur],
        });
    }
    blocs.push(Bloc {
        coins,
        faces: premiere_face..faces.len(),
        aretes: premiere_arete..aretes.len(),
    });
}

/// L'ordre où peindre les blocs, du plus loin au plus proche, vus d'un œil
/// placé en `oeil` (x, z) : un bloc passe avant un autre quand une droite
/// sépare leurs empreintes et que l'œil est du côté de l'autre. Sans droite
/// qui les sépare (ils se chevauchent), le plus loin de l'œil passe avant.
pub fn ordre_des_blocs(blocs: &[Bloc], oeil: (f64, f64)) -> Vec<usize> {
    let n = blocs.len();
    let distance = |bloc: &Bloc| {
        let x = bloc.coins.iter().map(|c| c.0).sum::<f64>() / 4.0;
        let z = bloc.coins.iter().map(|c| c.1).sum::<f64>() / 4.0;
        (x - oeil.0).hypot(z - oeil.1)
    };
    // derriere[i] : les blocs à peindre avant i.
    let mut derriere: Vec<Vec<usize>> = vec![Vec::new(); n];
    for i in 0..n {
        for j in i + 1..n {
            match devant(&blocs[i], &blocs[j], oeil) {
                Some(true) => derriere[i].push(j),
                Some(false) => derriere[j].push(i),
                None => {}
            }
        }
    }
    let mut peint = vec![false; n];
    let mut ordre = Vec::with_capacity(n);
    while ordre.len() < n {
        let restants = (0..n).filter(|&i| !peint[i]);
        let prets: Vec<usize> = restants
            .clone()
            .filter(|&i| derriere[i].iter().all(|&j| peint[j]))
            .collect();
        // Un cercle (rare) : le plus loin des restants.
        let candidats: Vec<usize> = if prets.is_empty() {
            restants.collect()
        } else {
            prets
        };
        let Some(choisi) = candidats
            .into_iter()
            .max_by(|&a, &b| distance(&blocs[a]).total_cmp(&distance(&blocs[b])))
        else {
            break;
        };
        peint[choisi] = true;
        ordre.push(choisi);
    }
    ordre
}

/// Vrai si `a` est devant `b` pour l'œil, faux s'il est derrière; `None` si
/// aucune droite ne sépare leurs empreintes.
fn devant(a: &Bloc, b: &Bloc, oeil: (f64, f64)) -> Option<bool> {
    let etendue = |coins: &[(f64, f64); 4], axe: (f64, f64)| {
        coins
            .iter()
            .map(|c| c.0 * axe.0 + c.1 * axe.1)
            .fold((f64::MAX, f64::MIN), |(bas, haut), v| {
                (bas.min(v), haut.max(v))
            })
    };
    for coins in [&a.coins, &b.coins] {
        for k in 0..4 {
            let (p, q) = (coins[k], coins[(k + 1) % 4]);
            let axe = (p.1 - q.1, q.0 - p.0);
            let norme = axe.0.hypot(axe.1);
            if norme < 1e-9 {
                continue;
            }
            let ((a0, a1), (b0, b1)) = (etendue(&a.coins, axe), etendue(&b.coins, axe));
            let oeil_sur_l_axe = oeil.0 * axe.0 + oeil.1 * axe.1;
            let marge = 1e-6 * norme;
            if a1 <= b0 + marge {
                return Some(oeil_sur_l_axe < (a1 + b0) / 2.0);
            }
            if b1 <= a0 + marge {
                return Some(oeil_sur_l_axe > (b1 + a0) / 2.0);
            }
        }
    }
    None
}

/// Découpe un contour simple en triangles (méthode des oreilles). Le contour
/// ne répète pas son premier sommet.
pub fn trianguler(contour: &[(f64, f64)]) -> Vec<[(f64, f64); 3]> {
    let mut rangs: Vec<usize> = (0..contour.len()).collect();
    let aire: f64 = (0..contour.len())
        .map(|i| {
            let (a, b) = (contour[i], contour[(i + 1) % contour.len()]);
            a.0 * b.1 - b.0 * a.1
        })
        .sum();
    if aire < 0.0 {
        rangs.reverse();
    }
    let produit = |a: (f64, f64), b: (f64, f64), c: (f64, f64)| {
        (b.0 - a.0) * (c.1 - b.1) - (b.1 - a.1) * (c.0 - b.0)
    };
    // Vrai si p est dans le triangle abc ou sur son bord.
    let dans = |p: (f64, f64), a: (f64, f64), b: (f64, f64), c: (f64, f64)| {
        let (d1, d2, d3) = (produit(a, b, p), produit(b, c, p), produit(c, a, p));
        d1 >= -1e-9 && d2 >= -1e-9 && d3 >= -1e-9
    };
    let mut triangles = Vec::new();
    while rangs.len() > 3 {
        let n = rangs.len();
        let oreille = (0..n).find(|&i| {
            let (ia, ib, ic) = (rangs[(i + n - 1) % n], rangs[i], rangs[(i + 1) % n]);
            let (a, b, c) = (contour[ia], contour[ib], contour[ic]);
            produit(a, b, c) > 1e-9
                && !rangs
                    .iter()
                    .any(|&j| j != ia && j != ib && j != ic && dans(contour[j], a, b, c))
        });
        let Some(i) = oreille else {
            // Un contour dégénéré : on s'arrête là.
            return triangles;
        };
        let (ia, ib, ic) = (rangs[(i + n - 1) % n], rangs[i], rangs[(i + 1) % n]);
        triangles.push([contour[ia], contour[ib], contour[ic]]);
        rangs.remove(i);
    }
    if let [a, b, c] = rangs[..] {
        triangles.push([contour[a], contour[b], contour[c]]);
    }
    triangles
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cloison::Cloison;
    use crate::dessin::{Cote, GenreDOuverture, Ouverture, Sens};
    use crate::mobilier::Objet;
    use crate::unites::Format;

    fn piece(cotes: &[(Sens, &str)]) -> Dessin {
        Dessin {
            cotes: cotes
                .iter()
                .map(|(sens, saisie)| Cote {
                    sens: *sens,
                    croquis: 100.0,
                    saisie: (*saisie).to_string(),
                })
                .collect(),
            fermee: true,
            ..Dessin::default()
        }
    }

    /// Aire d'une face plane (pouces carrés), par le produit vectoriel.
    fn aire(face: &Face) -> f64 {
        let s = &face.sommets;
        let mut total = [0.0; 3];
        for i in 1..s.len() - 1 {
            let (u, v) = (
                [s[i][0] - s[0][0], s[i][1] - s[0][1], s[i][2] - s[0][2]],
                [
                    s[i + 1][0] - s[0][0],
                    s[i + 1][1] - s[0][1],
                    s[i + 1][2] - s[0][2],
                ],
            );
            total[0] += u[1] * v[2] - u[2] * v[1];
            total[1] += u[2] * v[0] - u[0] * v[2];
            total[2] += u[0] * v[1] - u[1] * v[0];
        }
        (total[0].hypot(total[1])).hypot(total[2]) / 2.0
    }

    fn total(faces: &[Face], genre: GenreDeFace) -> f64 {
        faces.iter().filter(|f| f.genre == genre).map(aire).sum()
    }

    #[test]
    fn une_chambre_avec_sa_porte_et_sa_fenetre() {
        let mut dessin = piece(&[
            (Sens::Est, "12"),
            (Sens::Sud, "10"),
            (Sens::Ouest, ""),
            (Sens::Nord, ""),
        ]);
        let solution = dessin.resoudre(Format::PiPo);
        let faces = maquette(&dessin, &solution).unwrap().faces;
        // Sans hauteur inscrite : des murs de 8 pi.
        assert!((total(&faces, GenreDeFace::Plancher) - 120.0 * 144.0).abs() < 1e-6);
        assert!((total(&faces, GenreDeFace::Mur) - 44.0 * 12.0 * 96.0).abs() < 1e-6);
        // Une porte de 32 po × 80 po et une fenêtre de 36 po × 48 po, sur 9 pi.
        dessin.hauteur_des_murs = "9".into();
        dessin.ouvertures = vec![
            Ouverture {
                largeur: "2 pi 8 po".into(),
                hauteur: "6 pi 8 po".into(),
                genre: Some(GenreDOuverture::Porte),
                cote: Some(0),
                ..Ouverture::default()
            },
            Ouverture {
                largeur: "3".into(),
                hauteur: "4".into(),
                genre: Some(GenreDOuverture::Fenetre),
                cote: Some(1),
                allege: "2 pi 6 po".into(),
                ..Ouverture::default()
            },
        ];
        let solution = dessin.resoudre(Format::PiPo);
        let faces = maquette(&dessin, &solution).unwrap().faces;
        let murs = 44.0 * 12.0 * 108.0 - 32.0 * 80.0 - 36.0 * 48.0;
        assert!((total(&faces, GenreDeFace::Mur) - murs).abs() < 1e-6);
        assert!((total(&faces, GenreDeFace::Porte) - 32.0 * 80.0).abs() < 1e-6);
        assert!((total(&faces, GenreDeFace::Vitre) - 36.0 * 48.0).abs() < 1e-6);
        // La fenêtre est à 30 po du plancher.
        let vitre = faces
            .iter()
            .find(|f| f.genre == GenreDeFace::Vitre)
            .unwrap();
        let bas = vitre.sommets.iter().map(|s| s[1]).fold(f64::MAX, f64::min);
        assert!((bas - 30.0).abs() < 1e-9);
        // Les murs regardent vers l'intérieur : le mur du haut, vers le bas du plan.
        let mur_du_haut = faces
            .iter()
            .find(|f| f.genre == GenreDeFace::Mur && f.sommets.iter().all(|s| s[2].abs() < 1e-9))
            .unwrap();
        assert!(mur_du_haut.normale[2] > 0.0);
        // Une pièce incomplète n'a pas de maquette.
        dessin.cotes[1].saisie.clear();
        assert!(maquette(&dessin, &dessin.resoudre(Format::PiPo)).is_none());
    }

    #[test]
    fn un_lit_et_un_escalier_en_blocs() {
        let mut dessin = piece(&[
            (Sens::Est, "12"),
            (Sens::Sud, "10"),
            (Sens::Ouest, ""),
            (Sens::Nord, ""),
        ]);
        dessin.objets = vec![
            Objet::nouveau(GenreDObjet::Lit, (72.0, 60.0)),
            Objet::nouveau(GenreDObjet::Escalier, (24.0, 60.0)),
        ];
        let solution = dessin.resoudre(Format::PiPo);
        let faces = maquette(&dessin, &solution).unwrap().faces;
        let meubles: Vec<&Face> = faces
            .iter()
            .filter(|f| f.genre == GenreDeFace::Meuble(GenreDObjet::Lit))
            .collect();
        // Le lit : un dessus et quatre côtés, de 24 po de haut.
        assert_eq!(meubles.len(), 5);
        let dessus = meubles
            .iter()
            .find(|f| f.normale == [0.0, 1.0, 0.0])
            .unwrap();
        assert!((aire(dessus) - 60.0 * 80.0).abs() < 1e-6);
        assert!(dessus.sommets.iter().all(|s| (s[1] - 24.0).abs() < 1e-9));
        // L'escalier de 10 pi : 12 marches, jusqu'au haut des murs (8 pi).
        let marches: Vec<&Face> = faces
            .iter()
            .filter(|f| f.genre == GenreDeFace::Meuble(GenreDObjet::Escalier) && f.normale[1] > 0.5)
            .collect();
        assert_eq!(marches.len(), 12);
        let plus_haut = marches
            .iter()
            .flat_map(|f| f.sommets.iter().map(|s| s[1]))
            .fold(0.0, f64::max);
        assert!((plus_haut - 96.0).abs() < 1e-9);
    }

    #[test]
    fn une_cloison_monte_jusqu_au_haut_des_murs() {
        let mut dessin = piece(&[
            (Sens::Est, "12"),
            (Sens::Sud, "10"),
            (Sens::Ouest, ""),
            (Sens::Nord, ""),
        ]);
        dessin.cloisons = vec![Cloison {
            debut: (48.0, 0.0),
            sens: Sens::Sud,
            longueur: "10".to_string(),
            epaisseur: String::new(),
        }];
        let solution = dessin.resoudre(Format::PiPo);
        let maquette = maquette(&dessin, &solution).unwrap();
        assert_eq!(maquette.blocs.len(), 1);
        let bloc = &maquette.blocs[0];
        assert_eq!(bloc.faces.len(), 5);
        assert_eq!(bloc.aretes.len(), 4);
        assert!(
            maquette.faces[bloc.faces.clone()]
                .iter()
                .all(|f| f.genre == GenreDeFace::Cloison)
        );
        let dessus = &maquette.faces[bloc.faces.start];
        assert!((aire(dessus) - 4.5 * 120.0).abs() < 1e-6);
        assert!(dessus.sommets.iter().all(|s| (s[1] - 96.0).abs() < 1e-9));
    }

    #[test]
    fn les_blocs_se_peignent_du_plus_loin_au_plus_proche() {
        let bloc = |coins: [(f64, f64); 4]| Bloc {
            coins,
            faces: 0..0,
            aretes: 0..0,
        };
        let carre =
            |x: f64, z: f64| bloc([(x, z), (x + 10.0, z), (x + 10.0, z + 10.0), (x, z + 10.0)]);
        // Deux blocs côte à côte : vu de la droite, celui de gauche d'abord.
        let blocs = [carre(20.0, 0.0), carre(0.0, 0.0)];
        assert_eq!(ordre_des_blocs(&blocs, (100.0, 5.0)), vec![1, 0]);
        assert_eq!(ordre_des_blocs(&blocs, (-100.0, 5.0)), vec![0, 1]);
        // Une longue cloison et un petit bloc devant son bout : vu de devant,
        // la cloison d'abord, même si son milieu est plus proche de l'œil.
        let cloison = bloc([(0.0, 0.0), (200.0, 0.0), (200.0, 4.0), (0.0, 4.0)]);
        let petit = carre(190.0, 10.0);
        assert_eq!(
            ordre_des_blocs(&[petit, cloison], (20.0, 300.0)),
            vec![1, 0]
        );
    }

    #[test]
    fn le_plancher_d_une_piece_en_l() {
        let contour = [
            (0.0, 0.0),
            (240.0, 0.0),
            (240.0, 96.0),
            (108.0, 96.0),
            (108.0, 216.0),
            (0.0, 216.0),
        ];
        let triangles = trianguler(&contour);
        assert_eq!(triangles.len(), 4);
        let aire: f64 = triangles
            .iter()
            .map(|[a, b, c]| ((b.0 - a.0) * (c.1 - a.1) - (c.0 - a.0) * (b.1 - a.1)).abs() / 2.0)
            .sum();
        assert!((aire - 250.0 * 144.0).abs() < 1e-6);
        // Dans l'autre sens de parcours, le même plancher.
        let mut inverse = contour;
        inverse.reverse();
        assert_eq!(trianguler(&inverse).len(), 4);
    }
}
