//! Le plan d'étage (évolution 4 de la section 27) : les pièces placées les
//! unes par rapport aux autres dans le repère de l'étage, leurs murs qui
//! s'aimantent l'un contre l'autre, et la surface de l'étage.

use serde::{Deserialize, Serialize};

use crate::dessin::{Dessin, GenreDOuverture, MorceauDeMur, Placement, Plan};
use crate::unites::Format;

/// La place d'une pièce dans l'étage : le point de l'étage où va l'origine de
/// son plan, et sa rotation autour de ce point, en degrés, dans le sens des
/// aiguilles d'une montre à l'écran (y vers le bas).
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Place {
    pub x: f64,
    pub y: f64,
    pub rotation: f64,
}

impl Place {
    /// Un point du plan de la pièce, dans l'étage.
    pub fn vers_l_etage(&self, (x, y): (f64, f64)) -> (f64, f64) {
        let (s, c) = self.rotation.to_radians().sin_cos();
        (self.x + x * c - y * s, self.y + x * s + y * c)
    }

    /// Un point de l'étage, dans le plan de la pièce.
    pub fn vers_la_piece(&self, (x, y): (f64, f64)) -> (f64, f64) {
        let (s, c) = self.rotation.to_radians().sin_cos();
        let (dx, dy) = (x - self.x, y - self.y);
        (dx * c + dy * s, -dx * s + dy * c)
    }

    /// La même place, déplacée de (dx, dy).
    pub fn deplacee(self, (dx, dy): (f64, f64)) -> Place {
        Place {
            x: self.x + dx,
            y: self.y + dy,
            ..self
        }
    }
}

/// Une pièce complète, vue dans l'étage.
#[derive(Clone, Debug, PartialEq)]
pub struct PieceDansLEtage {
    /// Son rang dans le plan.
    pub rang: usize,
    pub place: Place,
    /// La face intérieure de ses murs, et leur face extérieure, dans l'étage.
    pub interieur: Vec<(f64, f64)>,
    pub exterieur: Vec<(f64, f64)>,
    /// L'épaisseur de ses murs (pouces).
    pub epaisseur: f64,
    /// Sa surface nette (pouces carrés).
    pub aire: f64,
}

/// L'écart entre deux pièces placées d'office, côte à côte : 3 pi.
const ECART_D_OFFICE: f64 = 36.0;

/// Les deux murs qui se touchent après l'aimant : l'autre pièce (son rang),
/// le côté de la pièce tirée et celui de l'autre pièce.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Contact {
    pub autre: usize,
    pub cote: usize,
    pub cote_de_l_autre: usize,
}

/// Les contours d'un dessin complet, dans son propre plan, sans leur dernier
/// sommet répété : la face intérieure des murs et leur face extérieure, avec
/// l'épaisseur des murs et la surface nette.
struct Contours {
    interieur: Vec<(f64, f64)>,
    exterieur: Vec<(f64, f64)>,
    epaisseur: f64,
    aire: f64,
}

fn contours(dessin: &Dessin, defaut: Format) -> Option<Contours> {
    let solution = dessin.resoudre(defaut);
    let aire = solution.aire_nette()?;
    let ouvert = |mut contour: Vec<(f64, f64)>| {
        if contour.len() > 1 && contour.first() == contour.last() {
            contour.pop();
        }
        contour
    };
    let interieur = ouvert(dessin.contour_interieur(&solution)?);
    let exterieur = dessin
        .contour_exterieur(&solution)
        .map(ouvert)
        .unwrap_or_else(|| interieur.clone());
    (interieur.len() >= 3).then_some(Contours {
        interieur,
        exterieur,
        epaisseur: solution.epaisseur_des_murs,
        aire,
    })
}

/// L'étendue d'une suite de points : (x min, x max, y min, y max).
fn etendue(points: &[(f64, f64)]) -> (f64, f64, f64, f64) {
    points.iter().fold(
        (f64::MAX, f64::MIN, f64::MAX, f64::MIN),
        |(x0, x1, y0, y1), &(x, y)| (x0.min(x), x1.max(x), y0.min(y), y1.max(y)),
    )
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

impl Plan {
    /// Donne une place aux pièces complètes qui n'en ont pas : à la suite des
    /// autres, vers la droite, leurs hauts alignés. Vrai si une pièce a reçu
    /// sa place.
    pub fn placer_les_pieces(&mut self, defaut: Format) -> bool {
        let mut droite: Option<f64> = None;
        let mut haut: Option<f64> = None;
        for piece in &self.pieces {
            if let (Some(place), Some(Contours { exterieur, .. })) =
                (piece.place, contours(&piece.dessin, defaut))
            {
                let (_, x1, y0, _) = etendue(
                    &exterieur
                        .iter()
                        .map(|&p| place.vers_l_etage(p))
                        .collect::<Vec<_>>(),
                );
                droite = Some(droite.map_or(x1, |d: f64| d.max(x1)));
                haut = Some(haut.map_or(y0, |h: f64| h.min(y0)));
            }
        }
        let mut x = droite.map_or(0.0, |d| d + ECART_D_OFFICE);
        let y = haut.unwrap_or(0.0);
        let mut placee = false;
        for piece in &mut self.pieces {
            if piece.place.is_some() {
                continue;
            }
            let Some(Contours { exterieur, .. }) = contours(&piece.dessin, defaut) else {
                continue;
            };
            let (x0, x1, y0, _) = etendue(&exterieur);
            piece.place = Some(Place {
                x: x - x0,
                y: y - y0,
                rotation: 0.0,
            });
            x += x1 - x0 + ECART_D_OFFICE;
            placee = true;
        }
        placee
    }

    /// Les pièces complètes et placées, dans l'étage.
    pub fn pieces_dans_l_etage(&self, defaut: Format) -> Vec<PieceDansLEtage> {
        self.pieces
            .iter()
            .enumerate()
            .filter_map(|(rang, piece)| {
                let place = piece.place?;
                let Contours {
                    interieur,
                    exterieur,
                    epaisseur,
                    aire,
                } = contours(&piece.dessin, defaut)?;
                let vers = |contour: Vec<(f64, f64)>| {
                    contour.into_iter().map(|p| place.vers_l_etage(p)).collect()
                };
                Some(PieceDansLEtage {
                    rang,
                    place,
                    interieur: vers(interieur),
                    exterieur: vers(exterieur),
                    epaisseur,
                    aire,
                })
            })
            .collect()
    }

    /// Tourne la pièce de rang `rang` de `degres` autour du centre de son
    /// étendue, qui reste en place. Faux si elle n'est pas placée.
    pub fn tourner_dans_l_etage(&mut self, rang: usize, degres: f64, defaut: Format) -> bool {
        let Some(piece) = self.pieces.get_mut(rang) else {
            return false;
        };
        let (Some(place), Some(Contours { interieur, .. })) =
            (piece.place, contours(&piece.dessin, defaut))
        else {
            return false;
        };
        let (x0, x1, y0, y1) = etendue(&interieur);
        let centre = ((x0 + x1) / 2.0, (y0 + y1) / 2.0);
        let dans_l_etage = place.vers_l_etage(centre);
        let mut tournee = Place {
            rotation: (place.rotation + degres).rem_euclid(360.0),
            ..place
        };
        let apres = tournee.vers_l_etage(centre);
        tournee = tournee.deplacee((dans_l_etage.0 - apres.0, dans_l_etage.1 - apres.1));
        piece.place = Some(tournee);
        true
    }
}

/// Un côté d'un contour dans l'étage : son rang dans le contour, son début,
/// sa direction, la normale qui sort de la pièce, et sa longueur.
#[derive(Clone, Copy, Debug)]
struct CoteSortant {
    rang: usize,
    debut: (f64, f64),
    direction: (f64, f64),
    sortie: (f64, f64),
    longueur: f64,
}

/// Les côtés d'un contour, sauf ceux de longueur nulle.
fn cotes_sortants(contour: &[(f64, f64)]) -> Vec<CoteSortant> {
    let sens = if aire_signee(contour) >= 0.0 {
        1.0
    } else {
        -1.0
    };
    (0..contour.len())
        .filter_map(|rang| {
            let (a, b) = (contour[rang], contour[(rang + 1) % contour.len()]);
            let longueur = (b.0 - a.0).hypot(b.1 - a.1);
            (longueur > 1e-6).then(|| {
                let direction = ((b.0 - a.0) / longueur, (b.1 - a.1) / longueur);
                CoteSortant {
                    rang,
                    debut: a,
                    direction,
                    sortie: (sens * direction.1, -sens * direction.0),
                    longueur,
                }
            })
        })
        .collect()
}

/// Deux côtés qui se font face : l'écart entre leurs lignes, le long de la
/// sortie du premier, et l'étendue du second le long du premier, depuis son
/// début. `None` s'ils ne se font pas face.
fn en_face(a: &CoteSortant, b: &CoteSortant) -> Option<(f64, f64, f64)> {
    if a.sortie.0 * b.sortie.0 + a.sortie.1 * b.sortie.1 > -0.9995 {
        return None;
    }
    let (dx, dy) = (b.debut.0 - a.debut.0, b.debut.1 - a.debut.1);
    let ecart = dx * a.sortie.0 + dy * a.sortie.1;
    let debut = dx * a.direction.0 + dy * a.direction.1;
    let fin = debut + b.longueur * (b.direction.0 * a.direction.0 + b.direction.1 * a.direction.1);
    Some((ecart, debut.min(fin), debut.max(fin)))
}

/// La place de la pièce de rang `rang` corrigée par l'aimant : si l'un de ses
/// murs arrive à moins de `seuil` pouces de sa place contre un mur d'une autre
/// pièce, en face, les deux pièces se rangent de part et d'autre d'un seul mur
/// (de l'épaisseur du plus épais des deux), et leurs coins s'alignent s'ils
/// sont à moins de `seuil` l'un de l'autre.
pub fn aimanter(
    plan: &Plan,
    rang: usize,
    place: Place,
    seuil: f64,
    defaut: Format,
) -> (Place, Option<Contact>) {
    let Some(Contours {
        interieur,
        epaisseur,
        ..
    }) = plan
        .pieces
        .get(rang)
        .and_then(|piece| contours(&piece.dessin, defaut))
    else {
        return (place, None);
    };
    let tiree: Vec<(f64, f64)> = interieur.iter().map(|&p| place.vers_l_etage(p)).collect();
    let cotes_de_la_tiree = cotes_sortants(&tiree);
    // Le meilleur appui : l'écart à combler, le côté de la tirée, le
    // glissement le long du mur et les deux murs qui se touchent.
    let mut meilleur: Option<(f64, CoteSortant, f64, Contact)> = None;
    for autre in plan
        .pieces_dans_l_etage(defaut)
        .iter()
        .filter(|autre| autre.rang != rang)
    {
        let mur = epaisseur.max(autre.epaisseur);
        for cote in &cotes_de_la_tiree {
            for vis_a_vis in cotes_sortants(&autre.interieur) {
                let Some((ecart, b0, b1)) = en_face(cote, &vis_a_vis) else {
                    continue;
                };
                let ecart = ecart - mur;
                // Assez près, et les deux côtés se chevauchent, à `seuil` près.
                if ecart.abs() > seuil || b1.min(cote.longueur) - b0.max(0.0) < -seuil {
                    continue;
                }
                // Les coins alignés, s'ils sont proches.
                let glissement = [b0, b1 - cote.longueur]
                    .into_iter()
                    .filter(|s| s.abs() <= seuil)
                    .min_by(|s, r| s.abs().total_cmp(&r.abs()))
                    .unwrap_or(0.0);
                if meilleur.is_none_or(|(e, ..)| ecart.abs() < e.abs()) {
                    meilleur = Some((
                        ecart,
                        *cote,
                        glissement,
                        Contact {
                            autre: autre.rang,
                            cote: cote.rang,
                            cote_de_l_autre: vis_a_vis.rang,
                        },
                    ));
                }
            }
        }
    }
    let Some((ecart, cote, glissement, contact)) = meilleur else {
        return (place, None);
    };
    let corrigee = place.deplacee((
        cote.sortie.0 * ecart + cote.direction.0 * glissement,
        cote.sortie.1 * ecart + cote.direction.1 * glissement,
    ));
    (corrigee, Some(contact))
}

/// Un mur commun à deux pièces voisines (évolution 4) : les deux pièces, leurs
/// côtés, et le morceau qu'elles partagent, dans l'étage, sur la face
/// intérieure de la première.
#[derive(Clone, Debug, PartialEq)]
pub struct MurCommun {
    pub pieces: (usize, usize),
    pub cotes: (usize, usize),
    pub debut: (f64, f64),
    pub fin: (f64, f64),
    /// L'épaisseur du mur : celle du plus épais des deux.
    pub epaisseur: f64,
}

impl MurCommun {
    /// La longueur du morceau partagé (pouces).
    pub fn longueur(&self) -> f64 {
        (self.fin.0 - self.debut.0).hypot(self.fin.1 - self.debut.1)
    }
}

/// L'écart toléré pour reconnaître un mur commun, en pouces.
const TOLERANCE_DU_MUR_COMMUN: f64 = 0.75;
/// Le plus court morceau qui fait un mur commun, en pouces.
const PLUS_COURT_MUR_COMMUN: f64 = 1.0;

impl Plan {
    /// Les murs communs de l'étage : deux pièces placées dont les faces
    /// intérieures se font face, à l'épaisseur du plus épais de leurs murs.
    pub fn murs_communs(&self, defaut: Format) -> Vec<MurCommun> {
        let pieces = self.pieces_dans_l_etage(defaut);
        let mut murs = Vec::new();
        for (i, une) in pieces.iter().enumerate() {
            let cotes = cotes_sortants(&une.interieur);
            for autre in &pieces[i + 1..] {
                let mur = une.epaisseur.max(autre.epaisseur);
                for cote in &cotes {
                    for vis_a_vis in cotes_sortants(&autre.interieur) {
                        let Some((ecart, b0, b1)) = en_face(cote, &vis_a_vis) else {
                            continue;
                        };
                        let (t0, t1) = (b0.max(0.0), b1.min(cote.longueur));
                        if (ecart - mur).abs() > TOLERANCE_DU_MUR_COMMUN
                            || t1 - t0 < PLUS_COURT_MUR_COMMUN
                        {
                            continue;
                        }
                        let point = |t: f64| {
                            (
                                cote.debut.0 + cote.direction.0 * t,
                                cote.debut.1 + cote.direction.1 * t,
                            )
                        };
                        murs.push(MurCommun {
                            pieces: (une.rang, autre.rang),
                            cotes: (cote.rang, vis_a_vis.rang),
                            debut: point(t0),
                            fin: point(t1),
                            epaisseur: mur,
                        });
                    }
                }
            }
        }
        murs
    }

    /// Les passages qu'ouvrent les portes et les fenêtres des pièces voisines
    /// dans les murs communs de la pièce de rang `rang` : (côté, début, fin),
    /// en pouces le long de ses mesures.
    pub fn passages_des_voisins(&self, rang: usize, defaut: Format) -> Vec<(usize, f64, f64)> {
        let Some(moi) = self.pieces.get(rang) else {
            return Vec::new();
        };
        let (Some(ma_place), Some(mes_sommets)) = (
            moi.place,
            moi.dessin.sommets_mesures(&moi.dessin.resoudre(defaut)),
        ) else {
            return Vec::new();
        };
        let mut passages = Vec::new();
        for mur in self.murs_communs(defaut) {
            let (mon_cote, autre, son_cote) = if mur.pieces.0 == rang {
                (mur.cotes.0, mur.pieces.1, mur.cotes.1)
            } else if mur.pieces.1 == rang {
                (mur.cotes.1, mur.pieces.0, mur.cotes.0)
            } else {
                continue;
            };
            let voisine = &self.pieces[autre];
            let solution = voisine.dessin.resoudre(defaut);
            let (Some(sa_place), Some(ses_sommets)) =
                (voisine.place, voisine.dessin.sommets_mesures(&solution))
            else {
                continue;
            };
            let (Some(&a), Some(&b), Some(&c), Some(&d)) = (
                mes_sommets.get(mon_cote),
                mes_sommets.get(mon_cote + 1),
                ses_sommets.get(son_cote),
                ses_sommets.get(son_cote + 1),
            ) else {
                continue;
            };
            let (longueur, sa_longueur) =
                ((b.0 - a.0).hypot(b.1 - a.1), (d.0 - c.0).hypot(d.1 - c.1));
            if longueur < 1e-9 || sa_longueur < 1e-9 {
                continue;
            }
            let u = ((b.0 - a.0) / longueur, (b.1 - a.1) / longueur);
            let v = ((d.0 - c.0) / sa_longueur, (d.1 - c.1) / sa_longueur);
            // Un point de l'étage, le long de mon côté.
            let le_long = |point: (f64, f64)| {
                let (x, y) = ma_place.vers_la_piece(point);
                (x - a.0) * u.0 + (y - a.1) * u.1
            };
            let (p, q) = (le_long(mur.debut), le_long(mur.fin));
            let (s0, s1) = (p.min(q), p.max(q));
            for placement in solution
                .placements
                .iter()
                .flatten()
                .filter(|placement| placement.cote == son_cote)
            {
                let bout = |t: f64| le_long(sa_place.vers_l_etage((c.0 + v.0 * t, c.1 + v.1 * t)));
                let (p, q) = (bout(placement.debut), bout(placement.fin));
                let (t0, t1) = (p.min(q).max(s0), p.max(q).min(s1));
                if t1 - t0 > 0.5 {
                    passages.push((mon_cote, t0, t1));
                }
            }
        }
        passages
    }

    /// Les murs de la pièce de rang `rang`, dans l'étage, à leur épaisseur :
    /// interrompus par ses portes et ses fenêtres, et par celles de ses
    /// voisines dans leurs murs communs.
    pub fn murs_dans_l_etage(&self, rang: usize, defaut: Format) -> Vec<MorceauDeMur> {
        let Some(piece) = self.pieces.get(rang) else {
            return Vec::new();
        };
        let Some(place) = piece.place else {
            return Vec::new();
        };
        let mut solution = piece.dessin.resoudre(defaut);
        for (cote, debut, fin) in self.passages_des_voisins(rang, defaut) {
            solution.placements.push(Some(Placement {
                cote,
                debut,
                fin,
                porte: true,
                genre: GenreDOuverture::BaieLibre,
                inversee: false,
                bas: 0.0,
                haut: 0.0,
            }));
        }
        piece
            .dessin
            .murs_exterieurs(&solution)
            .into_iter()
            .map(|morceau| MorceauDeMur {
                coins: morceau.coins.map(|coin| place.vers_l_etage(coin)),
                ..morceau
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dessin::{Cote, Piece, Sens};

    /// Une pièce rectangulaire, mesurée à l'intérieur, aux murs de 9 po.
    fn rectangle(longueur: &str, largeur: &str) -> Dessin {
        let cote = |sens, saisie: &str| Cote {
            sens,
            croquis: 100.0,
            saisie: saisie.to_string(),
        };
        Dessin {
            cotes: vec![
                cote(Sens::Est, longueur),
                cote(Sens::Sud, largeur),
                cote(Sens::Ouest, ""),
                cote(Sens::Nord, ""),
            ],
            fermee: true,
            ..Dessin::default()
        }
    }

    fn plan_de_deux_pieces() -> Plan {
        let mut plan = Plan::d_une_piece(rectangle("12", "10"));
        plan.pieces.push(Piece {
            dessin: rectangle("10", "8"),
            ..Piece::default()
        });
        plan
    }

    #[test]
    fn une_place_et_son_retour() {
        let place = Place {
            x: 100.0,
            y: 50.0,
            rotation: 90.0,
        };
        // Tournée de 90° à l'écran : l'est devient le sud.
        let (x, y) = place.vers_l_etage((10.0, 0.0));
        assert!((x - 100.0).abs() < 1e-9 && (y - 60.0).abs() < 1e-9);
        let (u, v) = place.vers_la_piece((x, y));
        assert!((u - 10.0).abs() < 1e-9 && v.abs() < 1e-9);
    }

    #[test]
    fn les_pieces_se_placent_d_office_cote_a_cote() {
        let mut plan = plan_de_deux_pieces();
        assert!(plan.pieces_dans_l_etage(Format::PiPo).is_empty());
        assert!(plan.placer_les_pieces(Format::PiPo));
        // La première, le dehors de ses murs au coin (0, 0); la seconde 3 pi
        // plus loin, leurs hauts alignés.
        let premiere = plan.pieces[0].place.unwrap();
        assert!((premiere.x - 9.0).abs() < 1e-9 && (premiere.y - 9.0).abs() < 1e-9);
        let seconde = plan.pieces[1].place.unwrap();
        assert!((seconde.x - (9.0 + 144.0 + 9.0 + 36.0 + 9.0)).abs() < 1e-9);
        assert!((seconde.y - 9.0).abs() < 1e-9);
        // Rien de plus à placer.
        assert!(!plan.placer_les_pieces(Format::PiPo));
        let pieces = plan.pieces_dans_l_etage(Format::PiPo);
        assert_eq!(pieces.len(), 2);
        assert!((pieces[0].aire - 144.0 * 120.0).abs() < 1e-6);
    }

    #[test]
    fn un_mur_s_aimante_contre_un_autre() {
        let mut plan = plan_de_deux_pieces();
        plan.placer_les_pieces(Format::PiPo);
        let premiere = plan.pieces[0].place.unwrap();
        // La seconde tirée à 3 po d'un seul mur de 9 po à droite de la
        // première, 2 po plus bas que son haut.
        let proposee = Place {
            x: premiere.x + 144.0 + 9.0 + 3.0,
            y: premiere.y + 2.0,
            rotation: 0.0,
        };
        let (place, contact) = aimanter(&plan, 1, proposee, 6.0, Format::PiPo);
        assert!(
            (place.x - (premiere.x + 144.0 + 9.0)).abs() < 1e-9,
            "{place:?}"
        );
        assert!((place.y - premiere.y).abs() < 1e-9, "{place:?}");
        let contact = contact.unwrap();
        assert_eq!(
            (contact.autre, contact.cote, contact.cote_de_l_autre),
            (0, 3, 1)
        );
        // Trop loin : aucun aimant.
        let loin = proposee.deplacee((20.0, 0.0));
        assert_eq!(aimanter(&plan, 1, loin, 6.0, Format::PiPo), (loin, None));
    }

    #[test]
    fn une_piece_tourne_sur_place() {
        let mut plan = plan_de_deux_pieces();
        plan.placer_les_pieces(Format::PiPo);
        let centre = |plan: &Plan| {
            let piece = &plan.pieces_dans_l_etage(Format::PiPo)[0];
            let (x0, x1, y0, y1) = etendue(&piece.interieur);
            ((x0 + x1) / 2.0, (y0 + y1) / 2.0, x1 - x0, y1 - y0)
        };
        let avant = centre(&plan);
        assert!(plan.tourner_dans_l_etage(0, 90.0, Format::PiPo));
        let apres = centre(&plan);
        // Même centre; la largeur et la profondeur échangées.
        assert!((avant.0 - apres.0).abs() < 1e-6 && (avant.1 - apres.1).abs() < 1e-6);
        assert!((apres.2 - 120.0).abs() < 1e-6 && (apres.3 - 144.0).abs() < 1e-6);
        for _ in 0..3 {
            plan.tourner_dans_l_etage(0, 90.0, Format::PiPo);
        }
        let place = plan.pieces[0].place.unwrap();
        assert!(
            place.rotation.abs() < 1e-9
                && (place.x - 9.0).abs() < 1e-6
                && (place.y - 9.0).abs() < 1e-6
        );
    }
    /// Le salon, et la chambre rangée contre son mur de droite, sa porte de
    /// 30 po dans leur mur commun.
    fn salon_et_chambre() -> Plan {
        let mut plan = plan_de_deux_pieces();
        plan.pieces[1].dessin.ouvertures = vec![crate::dessin::Ouverture {
            largeur: "30 po".into(),
            hauteur: "80 po".into(),
            cote: Some(3),
            position: "2".into(),
            ..crate::dessin::Ouverture::default()
        }];
        plan.placer_les_pieces(Format::PiPo);
        let premiere = plan.pieces[0].place.unwrap();
        plan.pieces[1].place = Some(Place {
            x: premiere.x + 144.0 + 9.0,
            y: premiere.y,
            rotation: 0.0,
        });
        plan
    }

    #[test]
    fn deux_pieces_voisines_partagent_leur_mur() {
        let plan = salon_et_chambre();
        let murs = plan.murs_communs(Format::PiPo);
        assert_eq!(murs.len(), 1);
        let mur = &murs[0];
        assert_eq!((mur.pieces, mur.cotes), ((0, 1), (1, 3)));
        assert!((mur.longueur() - 96.0).abs() < 1e-6 && (mur.epaisseur - 9.0).abs() < 1e-9);
        // La porte de la chambre traverse le mur du salon : de 42 à 72 po le
        // long de son côté 2.
        let passages = plan.passages_des_voisins(0, Format::PiPo);
        assert_eq!(passages.len(), 1, "{passages:?}");
        let (cote, debut, fin) = passages[0];
        assert_eq!(cote, 1);
        assert!(
            (debut - 42.0).abs() < 1e-6 && (fin - 72.0).abs() < 1e-6,
            "{passages:?}"
        );
        // Le mur du salon s'interrompt là : un morceau de plus.
        let salon = &plan.pieces[0].dessin;
        let seuls = salon.murs_exterieurs(&salon.resoudre(Format::PiPo)).len();
        assert_eq!(plan.murs_dans_l_etage(0, Format::PiPo).len(), seuls + 1);
        // Éloignées l'une de l'autre, plus de mur commun.
        let mut loin = plan.clone();
        loin.pieces[1].place = loin.pieces[1]
            .place
            .map(|place| place.deplacee((24.0, 0.0)));
        assert!(loin.murs_communs(Format::PiPo).is_empty());
        assert!(loin.passages_des_voisins(0, Format::PiPo).is_empty());
    }
}
