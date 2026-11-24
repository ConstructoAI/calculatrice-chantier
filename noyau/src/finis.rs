//! Les finis d'une pièce (évolution 27 de la section 27) : son plancher, la
//! peinture de ses murs, son plafond et ses plinthes; la trame qui montre le
//! plancher sur le plan; les quantités de chaque fini, pour la pièce et pour
//! tout le plan.

use serde::{Deserialize, Serialize};

use crate::dessin::{Etat, Plan, Solution};
use crate::unites::Format;

/// Le revêtement d'un plancher.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Plancher {
    #[default]
    Aucun,
    BoisFranc,
    Flottant,
    Ceramique,
    Vinyle,
    Tapis,
    Beton,
}

impl Plancher {
    pub const TOUS: [Plancher; 7] = [
        Plancher::Aucun,
        Plancher::BoisFranc,
        Plancher::Flottant,
        Plancher::Ceramique,
        Plancher::Vinyle,
        Plancher::Tapis,
        Plancher::Beton,
    ];
}

/// Le fini (le lustre) de la peinture des murs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FiniDePeinture {
    #[default]
    Aucun,
    Mat,
    Velours,
    CoquilleDOeuf,
    Satine,
    SemiLustre,
}

impl FiniDePeinture {
    pub const TOUS: [FiniDePeinture; 6] = [
        FiniDePeinture::Aucun,
        FiniDePeinture::Mat,
        FiniDePeinture::Velours,
        FiniDePeinture::CoquilleDOeuf,
        FiniDePeinture::Satine,
        FiniDePeinture::SemiLustre,
    ];
}

/// Le fini d'un plafond.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FiniDePlafond {
    #[default]
    Aucun,
    GypsePeint,
    Suspendu,
    Crepi,
    Bois,
    BetonApparent,
}

impl FiniDePlafond {
    pub const TOUS: [FiniDePlafond; 6] = [
        FiniDePlafond::Aucun,
        FiniDePlafond::GypsePeint,
        FiniDePlafond::Suspendu,
        FiniDePlafond::Crepi,
        FiniDePlafond::Bois,
        FiniDePlafond::BetonApparent,
    ];
}

/// Les plinthes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Plinthe {
    #[default]
    Aucune,
    Mdf,
    Bois,
    Vinyle,
    Ceramique,
}

impl Plinthe {
    pub const TOUS: [Plinthe; 5] = [
        Plinthe::Aucune,
        Plinthe::Mdf,
        Plinthe::Bois,
        Plinthe::Vinyle,
        Plinthe::Ceramique,
    ];
}

/// Les finis d'une pièce; une pièce d'avant n'en a aucun.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Finis {
    pub plancher: Plancher,
    pub peinture: FiniDePeinture,
    /// La couleur de la peinture, telle qu'inscrite (un nom, un code).
    pub couleur: String,
    pub plafond: FiniDePlafond,
    pub plinthes: Plinthe,
}

impl Finis {
    /// Vrai si la pièce n'a aucun fini.
    pub fn est_vide(&self) -> bool {
        self.plancher == Plancher::Aucun
            && self.peinture == FiniDePeinture::Aucun
            && self.couleur.trim().is_empty()
            && self.plafond == FiniDePlafond::Aucun
            && self.plinthes == Plinthe::Aucune
    }

    /// Vrai si la peinture est choisie : un fini, ou une couleur.
    pub fn peinture_choisie(&self) -> bool {
        self.peinture != FiniDePeinture::Aucun || !self.couleur.trim().is_empty()
    }
}

/// Un trait de la trame d'un plancher : une ligne, ou un point (un tapis, du
/// béton), sur le plan (pouces).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TraitDeTrame {
    Ligne((f64, f64), (f64, f64)),
    Point((f64, f64)),
}

/// La trame d'un plancher dans le polygone de la pièce (pouces) : des lames
/// de bois franc, les lames d'un plancher flottant et leurs joints décalés,
/// le quadrillage de la céramique, la diagonale du vinyle, les points d'un
/// tapis ou du béton.
pub fn trame(plancher: Plancher, polygone: &[(f64, f64)]) -> Vec<TraitDeTrame> {
    let polygone = ouvert(polygone);
    if polygone.len() < 3 {
        return Vec::new();
    }
    let lignes = |angle: f64, ecart: f64| {
        hachures(&polygone, angle, ecart)
            .into_iter()
            .map(|(a, b)| TraitDeTrame::Ligne(a, b))
            .collect::<Vec<_>>()
    };
    match plancher {
        Plancher::Aucun => Vec::new(),
        Plancher::BoisFranc => lignes(0.0, 4.0),
        Plancher::Flottant => {
            // Des lames de 8 po, et leurs joints, décalés d'une rangée à l'autre.
            let mut traits = lignes(0.0, 8.0);
            let (x0, x1, y0, y1) = etendue(&polygone);
            let mut rangee = 0;
            let mut y = y0;
            while y < y1 {
                let decalage = if rangee % 2 == 0 { 0.0 } else { 24.0 };
                let mut x = x0 + 24.0 + decalage;
                while x < x1 {
                    let milieu = (x, y + 4.0);
                    if dedans(milieu, &polygone) {
                        traits.push(TraitDeTrame::Ligne((x, y + 0.5), (x, y + 7.5)));
                    }
                    x += 48.0;
                }
                y += 8.0;
                rangee += 1;
            }
            traits
        }
        Plancher::Ceramique => {
            let mut traits = lignes(0.0, 12.0);
            traits.extend(lignes(90.0, 12.0));
            traits
        }
        Plancher::Vinyle => {
            let mut traits = lignes(45.0, 12.0);
            traits.extend(lignes(135.0, 12.0));
            traits
        }
        Plancher::Tapis => points(&polygone, 6.0),
        Plancher::Beton => points(&polygone, 12.0),
    }
}

/// Le polygone sans son premier sommet répété à la fin.
fn ouvert(polygone: &[(f64, f64)]) -> Vec<(f64, f64)> {
    let mut sommets = polygone.to_vec();
    if sommets.len() > 1 && sommets.first() == sommets.last() {
        sommets.pop();
    }
    sommets
}

fn etendue(polygone: &[(f64, f64)]) -> (f64, f64, f64, f64) {
    polygone.iter().fold(
        (f64::MAX, f64::MIN, f64::MAX, f64::MIN),
        |(x0, x1, y0, y1), &(x, y)| (x0.min(x), x1.max(x), y0.min(y), y1.max(y)),
    )
}

/// Vrai si le point est dans le polygone (règle pair-impair).
fn dedans((x, y): (f64, f64), polygone: &[(f64, f64)]) -> bool {
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

/// Des lignes parallèles à `angle` degrés, à `ecart` pouces l'une de l'autre,
/// coupées au polygone (règle pair-impair).
pub fn hachures(polygone: &[(f64, f64)], angle: f64, ecart: f64) -> Vec<((f64, f64), (f64, f64))> {
    let (sinus, cosinus) = angle.to_radians().sin_cos();
    // Le polygone tourné de −angle : les hachures y sont horizontales.
    let tourne: Vec<(f64, f64)> = polygone
        .iter()
        .map(|&(x, y)| (x * cosinus + y * sinus, -x * sinus + y * cosinus))
        .collect();
    let retour = |(x, y): (f64, f64)| (x * cosinus - y * sinus, x * sinus + y * cosinus);
    let (_, _, y0, y1) = etendue(&tourne);
    let n = tourne.len();
    let mut morceaux = Vec::new();
    let mut y = (y0 / ecart).floor() * ecart + ecart / 2.0;
    while y < y1 {
        let mut croisements: Vec<f64> = (0..n)
            .filter_map(|k| {
                let (a, b) = (tourne[k], tourne[(k + 1) % n]);
                ((a.1 <= y && y < b.1) || (b.1 <= y && y < a.1))
                    .then(|| a.0 + (y - a.1) * (b.0 - a.0) / (b.1 - a.1))
            })
            .collect();
        croisements.sort_by(f64::total_cmp);
        for &[debut, fin] in croisements.as_chunks::<2>().0 {
            if fin - debut > 1e-6 {
                morceaux.push((retour((debut, y)), retour((fin, y))));
            }
        }
        y += ecart;
    }
    morceaux
}

/// Une grille de points à `ecart` pouces, dans le polygone.
fn points(polygone: &[(f64, f64)], ecart: f64) -> Vec<TraitDeTrame> {
    let (x0, x1, y0, y1) = etendue(polygone);
    let mut traits = Vec::new();
    let mut y = (y0 / ecart).floor() * ecart + ecart / 2.0;
    while y < y1 {
        let mut x = (x0 / ecart).floor() * ecart + ecart / 2.0;
        while x < x1 {
            if dedans((x, y), polygone) {
                traits.push(TraitDeTrame::Point((x, y)));
            }
            x += ecart;
        }
        y += ecart;
    }
    traits
}

/// Les quantités des finis d'une pièce complète : la surface du plancher et
/// celle du plafond, la surface nette des murs (pouces carrés), et la longueur
/// des plinthes, le périmètre moins les portes et les baies (pouces).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct QuantitesDesFinis {
    pub plancher: Option<f64>,
    pub murs: Option<f64>,
    pub plafond: Option<f64>,
    pub plinthes: Option<f64>,
}

/// Les quantités des finis d'une pièce.
pub fn quantites(solution: &Solution) -> QuantitesDesFinis {
    let Etat::Complete { aire, perimetre } = solution.etat else {
        return QuantitesDesFinis::default();
    };
    let portes: f64 = solution
        .placements
        .iter()
        .flatten()
        .filter(|placement| placement.porte)
        .map(|placement| placement.fin - placement.debut)
        .sum();
    QuantitesDesFinis {
        plancher: solution.aire_nette(),
        murs: solution.murs().map(|murs| murs.nette),
        // Sans hauteur de murs, un plafond plat couvre toute la pièce.
        plafond: solution.surface_du_plafond.or(Some(aire)),
        plinthes: Some((perimetre - portes).max(0.0)),
    }
}

/// Un fini, pour en faire le total d'une pièce à l'autre.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Fini {
    Plancher(Plancher),
    /// Le fini de la peinture, et sa couleur.
    Peinture(FiniDePeinture, String),
    Plafond(FiniDePlafond),
    Plinthe(Plinthe),
}

impl Fini {
    /// Vrai si sa quantité est une longueur (des plinthes), faux pour une aire.
    pub fn en_longueur(&self) -> bool {
        matches!(self, Fini::Plinthe(_))
    }
}

/// Le tableau des finis d'un plan : chaque pièce et ses finis, puis la
/// quantité totale de chaque fini, dans l'ordre où il paraît.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TableauDesFinis {
    /// Le rang de la pièce, ses finis et leurs quantités.
    pub pieces: Vec<(usize, Finis, QuantitesDesFinis)>,
    /// Chaque fini et sa quantité (pouces carrés, ou pouces pour les plinthes).
    pub totaux: Vec<(Fini, f64)>,
}

impl Plan {
    /// Le tableau des finis du plan (évolution 27) : les pièces qui ont au
    /// moins un fini, et les totaux par fini.
    pub fn tableau_des_finis(&self, format: Format) -> TableauDesFinis {
        let mut tableau = TableauDesFinis::default();
        for (rang, piece) in self.pieces.iter().enumerate() {
            let finis = &piece.dessin.finis;
            if finis.est_vide() {
                continue;
            }
            let solution = piece.dessin.resoudre(format);
            let quantites = quantites(&solution);
            let mut ajouter = |fini: Fini, quantite: Option<f64>| {
                let Some(quantite) = quantite else {
                    return;
                };
                match tableau.totaux.iter_mut().find(|(f, _)| *f == fini) {
                    Some((_, total)) => *total += quantite,
                    None => tableau.totaux.push((fini, quantite)),
                }
            };
            if finis.plancher != Plancher::Aucun {
                ajouter(Fini::Plancher(finis.plancher), quantites.plancher);
            }
            if finis.peinture_choisie() {
                ajouter(
                    Fini::Peinture(finis.peinture, finis.couleur.trim().to_string()),
                    quantites.murs,
                );
            }
            if finis.plafond != FiniDePlafond::Aucun {
                ajouter(Fini::Plafond(finis.plafond), quantites.plafond);
            }
            if finis.plinthes != Plinthe::Aucune {
                ajouter(Fini::Plinthe(finis.plinthes), quantites.plinthes);
            }
            tableau.pieces.push((rang, finis.clone(), quantites));
        }
        tableau
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dessin::{Cote, Dessin, Ouverture, Piece, Sens};

    fn rectangle(largeur: &str, profondeur: &str) -> Dessin {
        let mut dessin = Dessin::default();
        for (sens, saisie) in [
            (Sens::Est, largeur),
            (Sens::Sud, profondeur),
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
        dessin
    }

    #[test]
    fn les_hachures_et_les_trames() {
        let carre = [(0.0, 0.0), (48.0, 0.0), (48.0, 48.0), (0.0, 48.0)];
        // Douze lames de 4 po, chacune de toute la largeur.
        let lames = hachures(&carre, 0.0, 4.0);
        assert_eq!(lames.len(), 12);
        assert!(lames.iter().all(|(a, b)| (b.0 - a.0).abs() - 48.0 < 1e-9));
        // La céramique : quatre lignes dans chaque sens; le tapis : 64 points.
        assert_eq!(trame(Plancher::Ceramique, &carre).len(), 4 + 4);
        assert_eq!(trame(Plancher::Tapis, &carre).len(), 64);
        // Une pièce en L : les hachures s'arrêtent au coin rentrant.
        let en_l = [
            (0.0, 0.0),
            (48.0, 0.0),
            (48.0, 24.0),
            (24.0, 24.0),
            (24.0, 48.0),
            (0.0, 48.0),
        ];
        let bas = hachures(&en_l, 0.0, 12.0);
        assert_eq!(bas.len(), 4);
        assert!((bas[3].1.0 - bas[3].0.0 - 24.0).abs() < 1e-9);
        // Le premier sommet répété à la fin ne change rien.
        let mut ferme = carre.to_vec();
        ferme.push(carre[0]);
        assert_eq!(trame(Plancher::BoisFranc, &ferme).len(), 12);
        assert!(trame(Plancher::Aucun, &carre).is_empty());
    }

    #[test]
    fn les_quantites_et_le_tableau_des_finis() {
        // Un salon de 12 pi × 10 pi, une porte de 32 po; une chambre de 10 pi
        // × 10 pi, au même plancher.
        let mut salon = rectangle("12", "10");
        salon.hauteur_des_murs = "8".into();
        salon.ouvertures.push(Ouverture {
            largeur: "32 po".into(),
            hauteur: "80 po".into(),
            genre: Some(crate::dessin::GenreDOuverture::Porte),
            cote: Some(0),
            ..Ouverture::default()
        });
        salon.finis = Finis {
            plancher: Plancher::BoisFranc,
            peinture: FiniDePeinture::CoquilleDOeuf,
            couleur: "OC-17".into(),
            plafond: FiniDePlafond::GypsePeint,
            plinthes: Plinthe::Mdf,
        };
        let solution = salon.resoudre(Format::PiPo);
        let q = quantites(&solution);
        assert_eq!(q.plancher, Some(144.0 * 120.0));
        // 44 pi de périmètre, moins la porte de 32 po.
        assert_eq!(q.plinthes, Some(44.0 * 12.0 - 32.0));
        assert_eq!(q.plafond, Some(144.0 * 120.0));
        let mut chambre = rectangle("10", "10");
        chambre.finis.plancher = Plancher::BoisFranc;
        let plan = Plan {
            pieces: vec![
                Piece {
                    nom: "Salon".into(),
                    dessin: salon,
                    ..Piece::default()
                },
                Piece {
                    nom: "Chambre".into(),
                    dessin: chambre,
                    ..Piece::default()
                },
                Piece::default(),
            ],
            ..Plan::default()
        };
        let tableau = plan.tableau_des_finis(Format::PiPo);
        assert_eq!(tableau.pieces.len(), 2);
        // Le bois franc des deux pièces : 120 + 100 pi².
        assert_eq!(
            tableau.totaux[0],
            (
                Fini::Plancher(Plancher::BoisFranc),
                (144.0 * 120.0) + (120.0 * 120.0)
            )
        );
        assert!(tableau.totaux.iter().any(
            |(fini, _)| *fini == Fini::Peinture(FiniDePeinture::CoquilleDOeuf, "OC-17".into())
        ));
        assert!(tableau.totaux.iter().any(|(fini, _)| fini.en_longueur()));
    }
}
