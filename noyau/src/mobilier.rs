//! Les objets d'une pièce (section 25) : l'escalier et le mobilier, placés sur
//! le plan à l'échelle de ses mesures. Chacun a ses dimensions de départ, sa
//! hauteur pour la 3D et un symbole, décrit dans son rectangle (u le long de
//! sa largeur, v le long de sa profondeur, de 0 à 1).

use serde::{Deserialize, Serialize};

/// Ce qu'est un objet.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GenreDObjet {
    Escalier,
    Lit,
    #[default]
    Table,
    Sofa,
    Comptoir,
    Toilette,
    Bain,
    Douche,
    Laveuse,
    Refrigerateur,
    Cuisiniere,
}

impl GenreDObjet {
    /// Tous les genres, dans l'ordre des boutons.
    pub const TOUS: [GenreDObjet; 11] = [
        GenreDObjet::Escalier,
        GenreDObjet::Lit,
        GenreDObjet::Table,
        GenreDObjet::Sofa,
        GenreDObjet::Comptoir,
        GenreDObjet::Toilette,
        GenreDObjet::Bain,
        GenreDObjet::Douche,
        GenreDObjet::Laveuse,
        GenreDObjet::Refrigerateur,
        GenreDObjet::Cuisiniere,
    ];

    /// Largeur, profondeur et hauteur de départ, en pouces : des tailles
    /// courantes, à ajuster ensuite. L'escalier monte jusqu'au haut des murs.
    pub fn dimensions(self) -> (f64, f64, f64) {
        match self {
            GenreDObjet::Escalier => (36.0, 120.0, 0.0),
            GenreDObjet::Lit => (60.0, 80.0, 24.0),
            GenreDObjet::Table => (36.0, 60.0, 30.0),
            GenreDObjet::Sofa => (84.0, 36.0, 34.0),
            GenreDObjet::Comptoir => (96.0, 25.0, 36.0),
            GenreDObjet::Toilette => (20.0, 28.0, 30.0),
            GenreDObjet::Bain => (30.0, 60.0, 20.0),
            GenreDObjet::Douche => (36.0, 36.0, 80.0),
            GenreDObjet::Laveuse => (27.0, 30.0, 38.0),
            GenreDObjet::Refrigerateur => (30.0, 32.0, 68.0),
            GenreDObjet::Cuisiniere => (30.0, 26.0, 36.0),
        }
    }

    /// Le symbole de l'objet dans son rectangle, en plus de son contour.
    /// `profondeur` (pouces) donne le nombre de marches d'un escalier.
    pub fn symbole(self, profondeur: f64) -> Vec<Trait> {
        use Trait::{Cercle, Ligne, Rectangle};
        match self {
            GenreDObjet::Escalier => {
                // Une marche par 10 po de giron, et la flèche qui monte.
                let marches = (profondeur / 10.0).round().clamp(3.0, 20.0) as usize;
                let mut traits: Vec<Trait> = (1..marches)
                    .map(|k| {
                        let v = k as f64 / marches as f64;
                        Ligne((0.0, v), (1.0, v))
                    })
                    .collect();
                traits.push(Ligne((0.5, 0.94), (0.5, 0.06)));
                traits.push(Ligne((0.5, 0.06), (0.38, 0.16)));
                traits.push(Ligne((0.5, 0.06), (0.62, 0.16)));
                traits
            }
            GenreDObjet::Lit => vec![
                Rectangle((0.08, 0.04), (0.46, 0.18)),
                Rectangle((0.54, 0.04), (0.92, 0.18)),
                Ligne((0.0, 0.3), (1.0, 0.3)),
            ],
            GenreDObjet::Table => vec![
                Rectangle((0.25, -0.12), (0.45, 0.0)),
                Rectangle((0.55, -0.12), (0.75, 0.0)),
                Rectangle((0.25, 1.0), (0.45, 1.12)),
                Rectangle((0.55, 1.0), (0.75, 1.12)),
                Rectangle((-0.2, 0.4), (0.0, 0.6)),
                Rectangle((1.0, 0.4), (1.2, 0.6)),
            ],
            GenreDObjet::Sofa => vec![
                Rectangle((0.0, 0.0), (1.0, 0.25)),
                Rectangle((0.0, 0.25), (0.12, 1.0)),
                Rectangle((0.88, 0.25), (1.0, 1.0)),
            ],
            GenreDObjet::Comptoir => vec![
                Ligne((0.0, 0.85), (1.0, 0.85)),
                Rectangle((0.4, 0.15), (0.6, 0.7)),
            ],
            GenreDObjet::Toilette => vec![
                Rectangle((0.1, 0.0), (0.9, 0.28)),
                Cercle((0.5, 0.62), 0.36),
            ],
            GenreDObjet::Bain => vec![
                Rectangle((0.08, 0.06), (0.92, 0.94)),
                Cercle((0.5, 0.14), 0.06),
            ],
            GenreDObjet::Douche => vec![
                Ligne((0.0, 0.0), (1.0, 1.0)),
                Ligne((1.0, 0.0), (0.0, 1.0)),
                Cercle((0.5, 0.5), 0.06),
            ],
            GenreDObjet::Laveuse => vec![Cercle((0.5, 0.55), 0.36)],
            GenreDObjet::Refrigerateur => vec![Ligne((0.0, 0.85), (1.0, 0.85))],
            GenreDObjet::Cuisiniere => vec![
                Cercle((0.28, 0.3), 0.14),
                Cercle((0.72, 0.3), 0.14),
                Cercle((0.28, 0.72), 0.14),
                Cercle((0.72, 0.72), 0.14),
            ],
        }
    }
}

/// Un trait d'un symbole, dans le rectangle de l'objet.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Trait {
    Ligne((f64, f64), (f64, f64)),
    Rectangle((f64, f64), (f64, f64)),
    /// Un cercle : son centre, et son rayon en part du plus petit côté.
    Cercle((f64, f64), f64),
}

/// Un trait d'un symbole, une fois sur le plan (pouces).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TraitSurLePlan {
    Ligne((f64, f64), (f64, f64)),
    Cercle((f64, f64), f64),
}

/// Un objet posé dans une pièce.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Objet {
    pub genre: GenreDObjet,
    /// Son centre, en pouces, dans le repère du plan (le départ du tracé en
    /// (0, 0), y vers le bas).
    pub centre: (f64, f64),
    /// Sa largeur et sa profondeur, telles qu'inscrites; vides : celles de départ.
    #[serde(default)]
    pub largeur: String,
    #[serde(default)]
    pub profondeur: String,
    /// Ses quarts de tour, dans le sens des aiguilles d'une montre (0 à 3).
    #[serde(default)]
    pub rotation: u8,
}

impl Objet {
    /// Un objet neuf, à ses dimensions de départ.
    pub fn nouveau(genre: GenreDObjet, centre: (f64, f64)) -> Objet {
        Objet {
            genre,
            centre,
            largeur: String::new(),
            profondeur: String::new(),
            rotation: 0,
        }
    }

    /// Un point de son rectangle (u, v), sur le plan; `dimensions` est sa
    /// largeur et sa profondeur en pouces.
    pub fn vers_le_plan(
        &self,
        (u, v): (f64, f64),
        (largeur, profondeur): (f64, f64),
    ) -> (f64, f64) {
        let (dx, dy) = ((u - 0.5) * largeur, (v - 0.5) * profondeur);
        let (dx, dy) = match self.rotation % 4 {
            0 => (dx, dy),
            1 => (-dy, dx),
            2 => (-dx, -dy),
            _ => (dy, -dx),
        };
        (self.centre.0 + dx, self.centre.1 + dy)
    }

    /// Les quatre coins de son rectangle, sur le plan.
    pub fn coins(&self, dimensions: (f64, f64)) -> [(f64, f64); 4] {
        [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]
            .map(|coin| self.vers_le_plan(coin, dimensions))
    }

    /// Son contour et son symbole, sur le plan.
    pub fn traits_sur_le_plan(&self, dimensions: (f64, f64)) -> Vec<TraitSurLePlan> {
        let coins = self.coins(dimensions);
        let mut traits: Vec<TraitSurLePlan> = (0..4)
            .map(|k| TraitSurLePlan::Ligne(coins[k], coins[(k + 1) % 4]))
            .collect();
        let plus_petit = dimensions.0.min(dimensions.1);
        for trait_ in self.genre.symbole(dimensions.1) {
            match trait_ {
                Trait::Ligne(a, b) => traits.push(TraitSurLePlan::Ligne(
                    self.vers_le_plan(a, dimensions),
                    self.vers_le_plan(b, dimensions),
                )),
                Trait::Rectangle((u0, v0), (u1, v1)) => {
                    let r = [(u0, v0), (u1, v0), (u1, v1), (u0, v1)]
                        .map(|c| self.vers_le_plan(c, dimensions));
                    for k in 0..4 {
                        traits.push(TraitSurLePlan::Ligne(r[k], r[(k + 1) % 4]));
                    }
                }
                Trait::Cercle(centre, rayon) => traits.push(TraitSurLePlan::Cercle(
                    self.vers_le_plan(centre, dimensions),
                    rayon * plus_petit,
                )),
            }
        }
        traits
    }

    /// Vrai si le point (sur le plan) est dans son rectangle.
    pub fn contient(&self, point: (f64, f64), dimensions: (f64, f64)) -> bool {
        let coins = self.coins(dimensions);
        let signes: Vec<f64> = (0..4)
            .map(|k| {
                let (a, b) = (coins[k], coins[(k + 1) % 4]);
                (b.0 - a.0) * (point.1 - a.1) - (b.1 - a.1) * (point.0 - a.0)
            })
            .collect();
        signes.iter().all(|s| *s >= 0.0) || signes.iter().all(|s| *s <= 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn un_objet_pivote_d_un_quart_de_tour() {
        let mut lit = Objet::nouveau(GenreDObjet::Lit, (100.0, 50.0));
        let (largeur, profondeur, _) = GenreDObjet::Lit.dimensions();
        let dimensions = (largeur, profondeur);
        // Sans rotation : 60 po de large (x), 80 po de profond (y).
        assert_eq!(lit.coins(dimensions)[0], (70.0, 10.0));
        assert_eq!(lit.coins(dimensions)[2], (130.0, 90.0));
        assert!(lit.contient((100.0, 85.0), dimensions));
        assert!(!lit.contient((100.0, 95.0), dimensions));
        // Un quart de tour : 80 po en x, 60 po en y.
        lit.rotation = 1;
        let xs: Vec<f64> = lit.coins(dimensions).iter().map(|c| c.0).collect();
        let ys: Vec<f64> = lit.coins(dimensions).iter().map(|c| c.1).collect();
        let etendue = |v: &[f64]| {
            v.iter().cloned().fold(f64::MIN, f64::max) - v.iter().cloned().fold(f64::MAX, f64::min)
        };
        assert!((etendue(&xs) - 80.0).abs() < 1e-9 && (etendue(&ys) - 60.0).abs() < 1e-9);
        assert!(lit.contient((135.0, 50.0), dimensions));
        assert!(!lit.contient((100.0, 85.0), dimensions));
    }

    #[test]
    fn le_symbole_d_un_escalier() {
        // 10 pi de long : 12 marches, donc 11 lignes de marche et la flèche.
        let traits = GenreDObjet::Escalier.symbole(120.0);
        assert_eq!(traits.len(), 11 + 3);
        let escalier = Objet::nouveau(GenreDObjet::Escalier, (0.0, 0.0));
        // Le contour (4 traits) et le symbole.
        assert_eq!(escalier.traits_sur_le_plan((36.0, 120.0)).len(), 4 + 14);
        // Chaque genre a des dimensions et un symbole.
        for genre in GenreDObjet::TOUS {
            let (largeur, profondeur, hauteur) = genre.dimensions();
            assert!(largeur > 0.0 && profondeur > 0.0 && hauteur >= 0.0);
            assert!(!genre.symbole(profondeur).is_empty());
        }
    }
}
