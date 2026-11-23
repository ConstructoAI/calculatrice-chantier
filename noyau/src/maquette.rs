//! Maquette 3D d'une pièce (section 25) : le plancher, les murs jusqu'au
//! plafond (un mur en pignon monte au faîte, section 25.1, point 34), et les
//! portes et les fenêtres dans leurs murs, en faces planes prêtes à projeter. Les coordonnées sont en pouces : x et z sur le plan,
//! y vers le haut.

use crate::dessin::{Dessin, Etat, Placement, Solution};
use crate::mobilier::{GenreDObjet, Matiere};

/// Hauteur des murs quand elle n'est pas inscrite : 8 pi.
pub const HAUTEUR_PAR_DEFAUT: f64 = 96.0;

/// Ce que représente une face.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GenreDeFace {
    Plancher,
    Mur,
    Vitre,
    Porte,
    /// Une partie de l'escalier ou d'un meuble, de sa matière.
    Meuble(Matiere),
    /// Un bloc d'une cloison.
    Cloison,
    /// Le dessus d'un mur, à son épaisseur.
    DessusDeMur,
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
    /// Sa hauteur, de son dessous à son dessus (pouces).
    pub bas: f64,
    pub haut: f64,
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
    // Les murs se bâtissent sur leur face intérieure : les mesures, ou, si
    // elles sont celles de l'extérieur, le contour repoussé vers l'intérieur.
    let sommets = dessin.contour_interieur(solution)?;
    let mesurees = dessin.sommets_mesures(solution)?;
    let contour = &sommets[..sommets.len().saturating_sub(1)];
    if contour.len() < 3 {
        return None;
    }
    let hauteur = solution.hauteur.unwrap_or(HAUTEUR_PAR_DEFAUT);
    // La hauteur du plafond en un point du plan (point 34).
    let plafond = solution.plafond;
    let haut_au = |p: (f64, f64)| plafond.map_or(hauteur, |au_plafond| au_plafond.hauteur_au(p));
    let aire_signee: f64 = sommets
        .windows(2)
        .map(|s| s[0].0 * s[1].1 - s[1].0 * s[0].1)
        .sum();
    let mut faces = Vec::new();
    let mut aretes = Vec::new();
    let exterieur = dessin.contour_exterieur(solution);

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
        // Le haut du mur suit le plafond; il se brise où le mur croise le faîte.
        let haut_du_mur = |t: f64| haut_au((a.0 + u.0 * t, a.1 + u.1 * t));
        let brisure = plafond.and_then(|au_plafond| au_plafond.brisure(a, b));
        let mut cretes = vec![0.0, longueur];
        cretes.extend(brisure);
        cretes.sort_by(f64::total_cmp);
        // Le bas et le haut du mur, et le coin où il commence.
        aretes.push([point(0.0, 0.0), point(longueur, 0.0)]);
        for paire in cretes.windows(2) {
            aretes.push([
                point(paire[0], haut_du_mur(paire[0])),
                point(paire[1], haut_du_mur(paire[1])),
            ]);
        }
        aretes.push([point(0.0, 0.0), point(0.0, haut_du_mur(0.0))]);
        // Le dessus du mur, à son épaisseur; il se range avec son mur.
        if let Some(exterieur) = &exterieur
            && solution.epaisseur_des_murs > 1e-9
        {
            let (wa, wb) = (exterieur[cote], exterieur[cote + 1]);
            let dehors = |t: f64| {
                let part = t / longueur;
                let p = (wa.0 + (wb.0 - wa.0) * part, wa.1 + (wb.1 - wa.1) * part);
                [p.0, haut_au(p), p.1]
            };
            for paire in cretes.windows(2) {
                let (t0, t1) = (paire[0], paire[1]);
                faces.push(Face {
                    genre: GenreDeFace::DessusDeMur,
                    sommets: vec![
                        point(t0, haut_du_mur(t0)),
                        point(t1, haut_du_mur(t1)),
                        dehors(t1),
                        dehors(t0),
                    ],
                    normale: dedans,
                });
                aretes.push([dehors(t0), dehors(t1)]);
            }
        }
        // Les ouvertures de ce mur, à leur place sur sa face intérieure (elles
        // se mesurent sur la ligne tracée).
        let decalage =
            (sommets[cote].0 - mesurees[cote].0) * u.0 + (sommets[cote].1 - mesurees[cote].1) * u.1;
        let ouvertures: Vec<Placement> = solution
            .placements
            .iter()
            .flatten()
            .filter(|placement| placement.cote == cote)
            .map(|placement| Placement {
                debut: placement.debut - decalage,
                fin: placement.fin - decalage,
                ..*placement
            })
            .collect();
        // Les bornes le long du mur : ses deux coins, le faîte et les bords
        // des ouvertures.
        let mut bornes = vec![0.0, longueur];
        bornes.extend(brisure);
        for placement in &ouvertures {
            bornes.push(placement.debut.clamp(0.0, longueur));
            bornes.push(placement.fin.clamp(0.0, longueur));
        }
        bornes.sort_by(f64::total_cmp);
        bornes.dedup_by(|x, y| (*x - *y).abs() < 1e-6);
        for paire in bornes.windows(2) {
            let (t0, t1) = (paire[0], paire[1]);
            let (h0, h1) = (haut_du_mur(t0), haut_du_mur(t1));
            let plafond_du_troncon = h0.min(h1);
            let milieu = (t0 + t1) / 2.0;
            // Ce que les ouvertures retirent de ce tronçon, en hauteur.
            let mut vides: Vec<(f64, f64)> = ouvertures
                .iter()
                .filter(|placement| placement.debut <= milieu && milieu <= placement.fin)
                .map(|placement| {
                    (
                        placement.bas.clamp(0.0, plafond_du_troncon),
                        placement.haut.clamp(0.0, plafond_du_troncon),
                    )
                })
                .filter(|(bas, haut)| haut > bas)
                .collect();
            vides.sort_by(|x, y| x.0.total_cmp(&y.0));
            let mut depuis = 0.0;
            for (bas, haut) in vides {
                if bas > depuis + 1e-6 {
                    faces.push(Face {
                        genre: GenreDeFace::Mur,
                        sommets: rectangle(t0, t1, depuis, bas),
                        normale: dedans,
                    });
                }
                depuis = f64::max(depuis, haut);
            }
            // Jusqu'au plafond, peut-être en pente.
            if h0.max(h1) > depuis + 1e-6 {
                faces.push(Face {
                    genre: GenreDeFace::Mur,
                    sommets: vec![
                        point(t0, depuis),
                        point(t1, depuis),
                        point(t1, h1.max(depuis)),
                        point(t0, h0.max(depuis)),
                    ],
                    normale: dedans,
                });
            }
        }
        for placement in &ouvertures {
            let dessus = haut_du_mur(placement.debut).min(haut_du_mur(placement.fin));
            let (bas, haut) = (
                placement.bas.clamp(0.0, dessus),
                placement.haut.clamp(0.0, dessus),
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
    // Les objets : chacun en ses parties; l'escalier, une marche par bloc, de
    // la flèche jusqu'au haut des murs.
    let mut blocs = Vec::new();
    for (rang, (objet, &dimensions)) in dessin.objets.iter().zip(&solution.objets).enumerate() {
        let coins = |u: (f64, f64), v: (f64, f64)| {
            [(u.0, v.0), (u.1, v.0), (u.1, v.1), (u.0, v.1)]
                .map(|coin| objet.vers_le_plan(coin, dimensions))
        };
        if objet.genre == GenreDObjet::Escalier {
            // Ses marches et ses paliers, selon sa forme et son calcul (R-101).
            let genre = GenreDeFace::Meuble(Matiere::Bois);
            let calcul = solution.escaliers.get(rang).copied().flatten().or_else(|| {
                crate::mobilier::calculer_l_escalier(objet.forme, dimensions, hauteur, None)
            });
            for marche in calcul
                .map(|calcul| objet.marches_en_3d(dimensions, &calcul))
                .unwrap_or_default()
            {
                bloc(
                    &mut faces,
                    &mut aretes,
                    &mut blocs,
                    marche.coins,
                    (marche.bas, marche.haut),
                    genre,
                    marche.sans_l_avant,
                );
            }
        } else {
            for partie in objet.genre.volumes() {
                let genre = GenreDeFace::Meuble(partie.matiere);
                let coins = coins(partie.u, partie.v);
                let hauteurs = (partie.bas, partie.haut);
                bloc(
                    &mut faces,
                    &mut aretes,
                    &mut blocs,
                    coins,
                    hauteurs,
                    genre,
                    false,
                );
            }
        }
    }
    // Les cloisons, jusqu'au haut des murs, interrompues par leur porte, que
    // surmonte un linteau.
    // Sous un plafond en pente, chaque morceau monte jusqu'au plus bas du
    // plafond au-dessus de lui.
    let sous_le_plafond = |coins: &[(f64, f64); 4]| {
        coins
            .iter()
            .map(|coin| haut_au(*coin))
            .fold(f64::INFINITY, f64::min)
    };
    for cloison in solution.cloisons.iter().flatten() {
        let genre = GenreDeFace::Cloison;
        for coins in cloison.morceaux().into_iter().map(|morceau| morceau.coins) {
            let haut = sous_le_plafond(&coins);
            bloc(
                &mut faces,
                &mut aretes,
                &mut blocs,
                coins,
                (0.0, haut),
                genre,
                false,
            );
        }
        // Au-dessus de chaque porte et de chaque passage, un linteau.
        for porte in &cloison.ouvertures {
            let coins = cloison.rectangle(porte.segment, porte.debut, porte.fin);
            let haut = sous_le_plafond(&coins);
            if haut <= crate::cloison::HAUTEUR_DE_PORTE {
                continue;
            }
            let hauteurs = (crate::cloison::HAUTEUR_DE_PORTE, haut);
            bloc(
                &mut faces,
                &mut aretes,
                &mut blocs,
                coins,
                hauteurs,
                genre,
                false,
            );
        }
    }
    Some(Maquette {
        faces,
        aretes,
        blocs,
    })
}

/// Un bloc, de son empreinte `coins` (x, z) et de `bas` à `haut` : son
/// dessus et ses côtés, tournés vers l'extérieur; `sans_l_avant` omet le
/// côté du premier au deuxième coin.
fn bloc(
    faces: &mut Vec<Face>,
    aretes: &mut Vec<[[f64; 3]; 2]>,
    blocs: &mut Vec<Bloc>,
    coins: [(f64, f64); 4],
    (bas, haut): (f64, f64),
    genre: GenreDeFace,
    sans_l_avant: bool,
) {
    let (premiere_face, premiere_arete) = (faces.len(), aretes.len());
    let dessous = coins.map(|(x, z)| [x, bas, z]);
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
            sommets: vec![dessous[a], dessous[b], dessus[b], dessus[a]],
            normale: [nx / longueur, 0.0, nz / longueur],
        });
    }
    blocs.push(Bloc {
        coins,
        bas,
        haut,
        faces: premiere_face..faces.len(),
        aretes: premiere_arete..aretes.len(),
    });
}

/// L'ordre où peindre les blocs, du plus loin au plus proche, vus d'un œil
/// placé en `oeil` (x, y, z) : un bloc passe avant un autre quand un plan
/// sépare les deux — une droite entre leurs empreintes, ou l'horizontale
/// entre deux blocs empilés — et que l'œil est du côté de l'autre. Sans
/// plan qui les sépare, le plus loin de l'œil passe avant.
pub fn ordre_des_blocs(blocs: &[Bloc], oeil: [f64; 3]) -> Vec<usize> {
    let n = blocs.len();
    let distance = |bloc: &Bloc| {
        let x = bloc.coins.iter().map(|c| c.0).sum::<f64>() / 4.0;
        let z = bloc.coins.iter().map(|c| c.1).sum::<f64>() / 4.0;
        (x - oeil[0]).hypot(z - oeil[2])
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
/// aucun plan ne les sépare.
fn devant(a: &Bloc, b: &Bloc, oeil: [f64; 3]) -> Option<bool> {
    let etendue = |coins: &[(f64, f64); 4], axe: (f64, f64)| {
        coins
            .iter()
            .map(|c| c.0 * axe.0 + c.1 * axe.1)
            .fold((f64::MAX, f64::MIN), |(bas, haut), v| {
                (bas.min(v), haut.max(v))
            })
    };
    // Une droite entre les empreintes : un plan vertical.
    for coins in [&a.coins, &b.coins] {
        for k in 0..4 {
            let (p, q) = (coins[k], coins[(k + 1) % 4]);
            let axe = (p.1 - q.1, q.0 - p.0);
            let norme = axe.0.hypot(axe.1);
            if norme < 1e-9 {
                continue;
            }
            let ((a0, a1), (b0, b1)) = (etendue(&a.coins, axe), etendue(&b.coins, axe));
            let oeil_sur_l_axe = oeil[0] * axe.0 + oeil[2] * axe.1;
            let marge = 1e-6 * norme;
            if a1 <= b0 + marge {
                return Some(oeil_sur_l_axe < (a1 + b0) / 2.0);
            }
            if b1 <= a0 + marge {
                return Some(oeil_sur_l_axe > (b1 + a0) / 2.0);
            }
        }
    }
    // Deux blocs empilés : le plan horizontal entre eux.
    if a.bas >= b.haut - 1e-6 {
        return Some(oeil[1] > (a.bas + b.haut) / 2.0);
    }
    if b.bas >= a.haut - 1e-6 {
        return Some(oeil[1] < (b.bas + a.haut) / 2.0);
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
        let maquette = maquette(&dessin, &solution).unwrap();
        // Le lit en ses parties, et l'escalier : 8 pi à monter (la hauteur par
        // défaut des murs), soit 13 contremarches de 7 3/8 po.
        assert_eq!(maquette.blocs.len(), GenreDObjet::Lit.volumes().len() + 13);
        let calcul = solution.escaliers[1].unwrap();
        assert_eq!(calcul.contremarches, 13);
        assert!((calcul.contremarche - 96.0 / 13.0).abs() < 1e-9);
        assert!(maquette.blocs.iter().any(|b| (b.haut - 96.0).abs() < 1e-9));
        let faces = &maquette.faces;
        let dessus = |matiere: Matiere| -> Vec<&Face> {
            faces
                .iter()
                .filter(|f| f.genre == GenreDeFace::Meuble(matiere) && f.normale[1] > 0.5)
                .collect()
        };
        // Le matelas, à 22 po.
        let matelas = dessus(Matiere::Literie);
        assert_eq!(matelas.len(), 1);
        assert!(
            matelas[0]
                .sommets
                .iter()
                .all(|s| (s[1] - 22.0).abs() < 1e-9)
        );
        // La tête du lit monte à 40 po.
        let tete = dessus(Matiere::BoisFonce)
            .iter()
            .flat_map(|f| f.sommets.iter().map(|s| s[1]))
            .fold(0.0, f64::max);
        assert!((tete - 40.0).abs() < 1e-9);
        // Les marches montent jusqu'au haut des murs (8 pi).
        let marches = dessus(Matiere::Bois);
        assert_eq!(marches.len(), 13);
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
            porte: None,
            suite: Vec::new(),
            ouvertures: Vec::new(),
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
        assert!((aire(dessus) - 5.0 * 120.0).abs() < 1e-6);
        assert!(dessus.sommets.iter().all(|s| (s[1] - 96.0).abs() < 1e-9));
    }

    #[test]
    fn la_porte_d_une_cloison_sous_son_linteau() {
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
            porte: Some(crate::cloison::PorteDeCloison::default()),
            ..Default::default()
        }];
        let solution = dessin.resoudre(Format::PiPo);
        let maquette = maquette(&dessin, &solution).unwrap();
        // Deux morceaux jusqu'au haut des murs, et le linteau sur la porte.
        assert_eq!(maquette.blocs.len(), 3);
        let linteau = maquette.blocs.iter().find(|b| b.bas > 0.0).unwrap();
        assert_eq!((linteau.bas, linteau.haut), (80.0, 96.0));
    }

    #[test]
    fn des_murs_vers_l_interieur_en_3d() {
        let mut dessin = piece(&[
            (Sens::Est, "12"),
            (Sens::Sud, "10"),
            (Sens::Ouest, ""),
            (Sens::Nord, ""),
        ]);
        dessin.mesures_exterieures = true;
        let solution = dessin.resoudre(Format::PiPo);
        let faces = maquette(&dessin, &solution).unwrap().faces;
        let somme = |genre: GenreDeFace| -> f64 {
            faces.iter().filter(|f| f.genre == genre).map(aire).sum()
        };
        // Le plancher : la pièce utile; le dessus des murs : le reste.
        assert!((somme(GenreDeFace::Plancher) - 126.0 * 102.0).abs() < 1e-6);
        assert!((somme(GenreDeFace::DessusDeMur) - (144.0 * 120.0 - 126.0 * 102.0)).abs() < 1e-6);
    }

    #[test]
    fn une_cloison_en_l_en_deux_blocs() {
        let mut dessin = piece(&[
            (Sens::Est, "12"),
            (Sens::Sud, "10"),
            (Sens::Ouest, ""),
            (Sens::Nord, ""),
        ]);
        dessin.cloisons = vec![Cloison {
            debut: (0.0, 50.0),
            sens: Sens::Est,
            longueur: "5".to_string(),
            suite: vec![crate::cloison::SegmentDeCloison {
                sens: Sens::Sud,
                longueur: "70 po".to_string(),
            }],
            ..Default::default()
        }];
        let solution = dessin.resoudre(Format::PiPo);
        let maquette = maquette(&dessin, &solution).unwrap();
        // Un bloc par segment, qui se rejoignent en onglet.
        assert_eq!(maquette.blocs.len(), 2);
        assert_eq!(maquette.blocs[0].coins[1], maquette.blocs[1].coins[0]);
    }

    #[test]
    fn le_dessus_des_murs_a_leur_epaisseur() {
        let dessin = piece(&[
            (Sens::Est, "12"),
            (Sens::Sud, "10"),
            (Sens::Ouest, ""),
            (Sens::Nord, ""),
        ]);
        let solution = dessin.resoudre(Format::PiPo);
        let faces = maquette(&dessin, &solution).unwrap().faces;
        let dessus: Vec<&Face> = faces
            .iter()
            .filter(|f| f.genre == GenreDeFace::DessusDeMur)
            .collect();
        assert_eq!(dessus.len(), 4);
        assert!(
            dessus
                .iter()
                .all(|f| f.sommets.iter().all(|s| (s[1] - 96.0).abs() < 1e-9))
        );
        // Le pourtour, à 9 po hors des mesures : (162 × 138) − (144 × 120) po².
        let aire_totale: f64 = dessus.iter().map(|f| aire(f)).sum();
        assert!((aire_totale - (162.0 * 138.0 - 144.0 * 120.0)).abs() < 1e-6);
    }

    #[test]
    fn les_blocs_se_peignent_du_plus_loin_au_plus_proche() {
        let bloc = |coins: [(f64, f64); 4], (bas, haut): (f64, f64)| Bloc {
            coins,
            bas,
            haut,
            faces: 0..0,
            aretes: 0..0,
        };
        let carre = |x: f64, z: f64, hauteurs: (f64, f64)| {
            bloc(
                [(x, z), (x + 10.0, z), (x + 10.0, z + 10.0), (x, z + 10.0)],
                hauteurs,
            )
        };
        // Deux blocs côte à côte : vu de la droite, celui de gauche d'abord.
        let blocs = [carre(20.0, 0.0, (0.0, 10.0)), carre(0.0, 0.0, (0.0, 10.0))];
        assert_eq!(ordre_des_blocs(&blocs, [100.0, 50.0, 5.0]), vec![1, 0]);
        assert_eq!(ordre_des_blocs(&blocs, [-100.0, 50.0, 5.0]), vec![0, 1]);
        // Une longue cloison et un petit bloc devant son bout : vu de devant,
        // la cloison d'abord, même si son milieu est plus proche de l'œil.
        let cloison = bloc(
            [(0.0, 0.0), (200.0, 0.0), (200.0, 4.0), (0.0, 4.0)],
            (0.0, 96.0),
        );
        let petit = carre(190.0, 10.0, (0.0, 30.0));
        assert_eq!(
            ordre_des_blocs(&[petit, cloison], [20.0, 120.0, 300.0]),
            vec![1, 0]
        );
        // Un plateau sur sa patte : vu d'en haut, la patte d'abord; vu d'en
        // dessous, le plateau d'abord.
        let plateau = carre(0.0, 0.0, (28.5, 30.0));
        let patte = bloc(
            [(1.0, 1.0), (2.0, 1.0), (2.0, 2.0), (1.0, 2.0)],
            (0.0, 28.5),
        );
        let pieces = [plateau, patte];
        assert_eq!(ordre_des_blocs(&pieces, [5.0, 100.0, 50.0]), vec![1, 0]);
        assert_eq!(ordre_des_blocs(&pieces, [5.0, 10.0, 50.0]), vec![0, 1]);
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
