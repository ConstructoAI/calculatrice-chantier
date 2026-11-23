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

    /// Les parties du meuble en 3D, chacune de sa matière. L'escalier se bâtit
    /// à part, en marches jusqu'au haut des murs.
    pub fn volumes(self) -> Vec<Volume> {
        use Matiere::*;
        match self {
            GenreDObjet::Escalier => Vec::new(),
            // La tête, la base, le matelas, la couverture et deux oreillers.
            GenreDObjet::Lit => vec![
                volume((0.0, 1.0), (0.0, 0.03), 0.0, 40.0, BoisFonce),
                volume((0.0, 1.0), (0.03, 1.0), 0.0, 12.0, BoisFonce),
                volume((0.02, 0.98), (0.03, 0.99), 12.0, 22.0, Literie),
                volume((0.02, 0.98), (0.32, 0.99), 22.0, 23.0, Tissu),
                volume((0.08, 0.46), (0.05, 0.18), 22.0, 26.0, Coussin),
                volume((0.54, 0.92), (0.05, 0.18), 22.0, 26.0, Coussin),
            ],
            // Le plateau, ses quatre pattes, et les chaises de son symbole.
            GenreDObjet::Table => {
                let mut parties = vec![volume((0.0, 1.0), (0.0, 1.0), 28.5, 30.0, Bois)];
                for u in [(0.03, 0.09), (0.91, 0.97)] {
                    for v in [(0.03, 0.07), (0.93, 0.97)] {
                        parties.push(volume(u, v, 0.0, 28.5, Bois));
                    }
                }
                for (u, v, dossier) in [
                    ((0.25, 0.45), (-0.12, 0.0), (0.0, -1.0)),
                    ((0.55, 0.75), (-0.12, 0.0), (0.0, -1.0)),
                    ((0.25, 0.45), (1.0, 1.12), (0.0, 1.0)),
                    ((0.55, 0.75), (1.0, 1.12), (0.0, 1.0)),
                    ((-0.2, 0.0), (0.4, 0.6), (-1.0, 0.0)),
                    ((1.0, 1.2), (0.4, 0.6), (1.0, 0.0)),
                ] {
                    parties.extend(chaise(u, v, dossier));
                }
                parties
            }
            // Le dossier, les bras, l'assise, et les coussins du siège et du dos.
            GenreDObjet::Sofa => vec![
                volume((0.0, 1.0), (0.0, 0.15), 0.0, 34.0, Tissu),
                volume((0.0, 0.12), (0.15, 1.0), 0.0, 25.0, Tissu),
                volume((0.88, 1.0), (0.15, 1.0), 0.0, 25.0, Tissu),
                volume((0.12, 0.88), (0.15, 1.0), 0.0, 15.0, Tissu),
                volume((0.12, 0.5), (0.25, 0.98), 15.0, 19.0, Coussin),
                volume((0.5, 0.88), (0.25, 0.98), 15.0, 19.0, Coussin),
                volume((0.12, 0.5), (0.15, 0.25), 15.0, 31.0, Coussin),
                volume((0.5, 0.88), (0.15, 0.25), 15.0, 31.0, Coussin),
            ],
            // Le socle, les armoires et leurs portes, le dessus, l'évier et
            // son robinet; le devant est du côté v = 1.
            GenreDObjet::Comptoir => vec![
                volume((0.0, 1.0), (0.0, 0.88), 0.0, 4.0, Noir),
                volume((0.0, 1.0), (0.0, 0.94), 4.0, 35.0, Armoire),
                volume((0.248, 0.252), (0.94, 0.945), 6.0, 33.0, Noir),
                volume((0.498, 0.502), (0.94, 0.945), 6.0, 33.0, Noir),
                volume((0.748, 0.752), (0.94, 0.945), 6.0, 33.0, Noir),
                volume((0.0, 1.0), (0.0, 1.0), 35.0, 36.5, Comptoir),
                volume((0.4, 0.6), (0.15, 0.7), 36.5, 36.7, Metal),
                volume((0.48, 0.52), (0.05, 0.12), 36.5, 44.0, Metal),
            ],
            // Le réservoir, la cuvette et le siège.
            GenreDObjet::Toilette => vec![
                volume((0.1, 0.9), (0.0, 0.26), 0.0, 30.0, Porcelaine),
                volume((0.22, 0.78), (0.26, 0.94), 0.0, 14.5, Porcelaine),
                volume((0.15, 0.85), (0.26, 1.0), 14.5, 16.0, Porcelaine),
            ],
            // Le fond et les quatre rebords (le bain est creux), et le robinet.
            GenreDObjet::Bain => vec![
                volume((0.08, 0.92), (0.06, 0.94), 0.0, 5.0, Porcelaine),
                volume((0.0, 0.08), (0.0, 1.0), 0.0, 20.0, Porcelaine),
                volume((0.92, 1.0), (0.0, 1.0), 0.0, 20.0, Porcelaine),
                volume((0.08, 0.92), (0.0, 0.06), 0.0, 20.0, Porcelaine),
                volume((0.08, 0.92), (0.94, 1.0), 0.0, 20.0, Porcelaine),
                volume((0.45, 0.55), (0.01, 0.05), 20.0, 24.0, Metal),
            ],
            // La base, deux parois de verre et la pomme.
            GenreDObjet::Douche => vec![
                volume((0.0, 1.0), (0.0, 1.0), 0.0, 4.0, Porcelaine),
                volume((0.0, 1.0), (0.98, 1.0), 4.0, 78.0, Verre),
                volume((0.98, 1.0), (0.0, 0.98), 4.0, 78.0, Verre),
                volume((0.45, 0.55), (0.0, 0.05), 70.0, 74.0, Metal),
            ],
            // La caisse, la console et son afficheur, et le couvercle.
            GenreDObjet::Laveuse => vec![
                volume((0.0, 1.0), (0.0, 1.0), 0.0, 36.0, Electromenager),
                volume((0.0, 1.0), (0.0, 0.15), 36.0, 42.0, Electromenager),
                volume((0.4, 0.6), (0.15, 0.16), 38.0, 40.0, Noir),
                volume((0.15, 0.85), (0.25, 0.9), 36.0, 36.4, Metal),
            ],
            // La caisse, le joint des portes et leurs poignées.
            GenreDObjet::Refrigerateur => vec![
                volume((0.0, 1.0), (0.0, 1.0), 0.0, 68.0, Metal),
                volume((0.0, 1.0), (1.0, 1.005), 44.0, 44.6, Noir),
                volume((0.85, 0.9), (1.0, 1.04), 47.0, 62.0, Noir),
                volume((0.85, 0.9), (1.0, 1.04), 20.0, 40.0, Noir),
            ],
            // La caisse, le dessus et ses quatre brûleurs, le dosseret, et le
            // hublot et la poignée du four.
            GenreDObjet::Cuisiniere => {
                let mut parties = vec![
                    volume((0.0, 1.0), (0.0, 1.0), 0.0, 35.6, Electromenager),
                    volume((0.0, 1.0), (0.0, 1.0), 35.6, 36.0, Noir),
                    volume((0.0, 1.0), (0.0, 0.1), 36.0, 42.0, Electromenager),
                    volume((0.15, 0.85), (1.0, 1.01), 8.0, 24.0, Noir),
                    volume((0.1, 0.9), (1.0, 1.04), 28.0, 30.0, Metal),
                ];
                for u in [(0.16, 0.40), (0.60, 0.84)] {
                    for v in [(0.16, 0.44), (0.58, 0.86)] {
                        parties.push(volume(u, v, 36.0, 36.3, Metal));
                    }
                }
                parties
            }
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

/// Ce dont une partie d'un meuble est faite, pour sa couleur en 3D.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Matiere {
    Bois,
    BoisFonce,
    Tissu,
    Coussin,
    Literie,
    Porcelaine,
    Metal,
    Electromenager,
    Noir,
    Verre,
    Comptoir,
    Armoire,
}

/// Une partie d'un meuble en 3D : un bloc, de `u` et `v` (en part de la
/// largeur et de la profondeur du meuble, comme son symbole) et de `bas` à
/// `haut` (pouces). Deux parties d'un même meuble ne se chevauchent pas.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Volume {
    pub u: (f64, f64),
    pub v: (f64, f64),
    pub bas: f64,
    pub haut: f64,
    pub matiere: Matiere,
}

const fn volume(u: (f64, f64), v: (f64, f64), bas: f64, haut: f64, matiere: Matiere) -> Volume {
    Volume {
        u,
        v,
        bas,
        haut,
        matiere,
    }
}

/// Une chaise de la table : son siège, et son dossier du côté loin de la
/// table (`dossier` : la direction de ce côté, en u ou en v).
fn chaise(u: (f64, f64), v: (f64, f64), dossier: (f64, f64)) -> [Volume; 2] {
    // Le dossier occupe le sixième de la chaise, du côté de `dossier`.
    let couper = |(a, b): (f64, f64), sens: f64| {
        let part = (b - a) / 6.0;
        if sens > 0.0 {
            ((a, b - part), (b - part, b))
        } else if sens < 0.0 {
            ((a + part, b), (a, a + part))
        } else {
            ((a, b), (a, b))
        }
    };
    let (siege_u, dos_u) = couper(u, dossier.0);
    let (siege_v, dos_v) = couper(v, dossier.1);
    [
        volume(siege_u, siege_v, 0.0, 18.0, Matiere::Bois),
        volume(dos_u, dos_v, 0.0, 34.0, Matiere::Bois),
    ]
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
    /// Ses degrés de rotation en plus de ses quarts de tour, dans le même sens
    /// (par pas de 15°).
    #[serde(default)]
    pub angle: f64,
    /// La forme d'un escalier (point 30); sans objet pour un meuble.
    #[serde(default)]
    pub forme: FormeDEscalier,
    /// La hauteur à monter d'un escalier, telle qu'inscrite; vide : celle des murs.
    #[serde(default)]
    pub hauteur: String,
    /// Le nombre de contremarches d'un escalier, tel qu'inscrit; vide : calculé
    /// pour des contremarches d'au plus 7 1/2 po.
    #[serde(default)]
    pub contremarches: String,
    /// Vrai s'il est retourné comme dans un miroir, sa gauche et sa droite
    /// échangées (point 31) : un escalier tournant tourne de l'autre côté.
    #[serde(default)]
    pub retourne: bool,
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
            angle: 0.0,
            forme: FormeDEscalier::Droit,
            hauteur: String::new(),
            contremarches: String::new(),
            retourne: false,
        }
    }

    /// Sa largeur et sa profondeur de départ, en pouces : celles de son genre,
    /// ou, pour un escalier, celles de sa forme.
    pub fn dimensions_de_depart(&self) -> (f64, f64) {
        if self.genre == GenreDObjet::Escalier {
            self.forme.dimensions()
        } else {
            let (largeur, profondeur, _) = self.genre.dimensions();
            (largeur, profondeur)
        }
    }

    /// Son angle, en degrés de 0 à 360, dans le sens des aiguilles d'une montre.
    pub fn angle_total(&self) -> f64 {
        (f64::from(self.rotation) * 90.0 + self.angle).rem_euclid(360.0)
    }

    /// Le fait tourner de `degres` (négatif : dans l'autre sens).
    pub fn pivoter(&mut self, degres: f64) {
        self.angle = (self.angle + degres).rem_euclid(360.0);
    }

    /// Un point de son rectangle (u, v), sur le plan; `dimensions` est sa
    /// largeur et sa profondeur en pouces.
    pub fn vers_le_plan(
        &self,
        (u, v): (f64, f64),
        (largeur, profondeur): (f64, f64),
    ) -> (f64, f64) {
        // Retourné : sa gauche et sa droite échangées.
        let u = if self.retourne { 1.0 - u } else { u };
        let (dx, dy) = ((u - 0.5) * largeur, (v - 0.5) * profondeur);
        let angle = self.angle_total();
        let quarts = angle / 90.0;
        // Un quart de tour exact garde des coins exacts.
        let (dx, dy) = if (quarts - quarts.round()).abs() < 1e-9 {
            match (quarts.round() as i64).rem_euclid(4) {
                0 => (dx, dy),
                1 => (-dy, dx),
                2 => (-dx, -dy),
                _ => (dy, -dx),
            }
        } else {
            let (sinus, cosinus) = angle.to_radians().sin_cos();
            (dx * cosinus - dy * sinus, dx * sinus + dy * cosinus)
        };
        (self.centre.0 + dx, self.centre.1 + dy)
    }

    /// Les quatre coins de son rectangle, sur le plan.
    pub fn coins(&self, dimensions: (f64, f64)) -> [(f64, f64); 4] {
        [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]
            .map(|coin| self.vers_le_plan(coin, dimensions))
    }

    /// Son contour et son symbole, sur le plan. `contremarches` : celles d'un
    /// escalier calculé (sinon, une marche par 10 po de profondeur).
    pub fn traits_sur_le_plan(
        &self,
        dimensions: (f64, f64),
        contremarches: Option<usize>,
    ) -> Vec<TraitSurLePlan> {
        let coins = self.coins(dimensions);
        let escalier = self.genre == GenreDObjet::Escalier;
        // Un colimaçon n'a pas de contour carré : son cercle en tient lieu.
        let mut traits: Vec<TraitSurLePlan> = if escalier && self.forme == FormeDEscalier::Colimacon
        {
            Vec::new()
        } else {
            (0..4)
                .map(|k| TraitSurLePlan::Ligne(coins[k], coins[(k + 1) % 4]))
                .collect()
        };
        let plus_petit = dimensions.0.min(dimensions.1);
        let symbole = if escalier {
            let marches =
                contremarches.unwrap_or(((dimensions.1 / 10.0).round().clamp(3.0, 20.0)) as usize);
            symbole_d_escalier(self.forme, dimensions, marches)
        } else {
            self.genre.symbole(dimensions.1)
        };
        for trait_ in symbole {
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

// ----------------------------------------------------------------------
// Escaliers (point 30, R-101) : forme, calcul des contremarches, symbole et
// marches en 3D, dans le rectangle de l'escalier (u, v de 0 à 1; le bas en
// v = 1, la montée vers v = 0).
// ----------------------------------------------------------------------

/// La forme d'un escalier.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FormeDEscalier {
    #[default]
    Droit,
    /// En L : deux volées et un palier dans le coin.
    EnL,
    /// En U : deux volées côte à côte et un palier qui les relie.
    EnU,
    /// Quart tournant : deux volées et deux marches dansantes dans le coin.
    Tournant,
    /// Colimaçon : des marches en éventail autour d'un poteau.
    Colimacon,
}

impl FormeDEscalier {
    /// Toutes les formes, dans l'ordre du choix.
    pub const TOUTES: [FormeDEscalier; 5] = [
        FormeDEscalier::Droit,
        FormeDEscalier::EnL,
        FormeDEscalier::EnU,
        FormeDEscalier::Tournant,
        FormeDEscalier::Colimacon,
    ];

    /// Largeur et profondeur de départ de son emprise, en pouces.
    pub fn dimensions(self) -> (f64, f64) {
        match self {
            FormeDEscalier::Droit => (36.0, 120.0),
            FormeDEscalier::EnL | FormeDEscalier::Tournant => (72.0, 96.0),
            FormeDEscalier::EnU => (84.0, 84.0),
            FormeDEscalier::Colimacon => (60.0, 60.0),
        }
    }
}

/// Contremarche visée d'un escalier du dessin, en pouces : la valeur de départ
/// du calcul d'escalier (section 13.4). Ce n'est pas une norme.
pub const CONTREMARCHE_VISEE: f64 = 7.5;
/// Largeur d'une volée d'un escalier en L, en U ou tournant, en pouces.
pub const LARGEUR_DE_VOLEE: f64 = 36.0;
/// Nombre de contremarches permis.
const CONTREMARCHES_PERMISES: std::ops::RangeInclusive<usize> = 3..=40;

/// Un escalier calculé.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EscalierCalcule {
    /// La hauteur à monter, en pouces.
    pub hauteur: f64,
    pub contremarches: usize,
    /// La hauteur d'une contremarche, en pouces.
    pub contremarche: f64,
    /// Le giron d'un escalier droit : sa profondeur partagée entre ses marches.
    pub giron: Option<f64>,
}

/// Calcule un escalier : `contremarches` inscrites, ou le plus petit nombre
/// qui donne des contremarches d'au plus 7 1/2 po. Rien si la hauteur n'en
/// est pas une.
pub fn calculer_l_escalier(
    forme: FormeDEscalier,
    (_, profondeur): (f64, f64),
    hauteur: f64,
    contremarches: Option<usize>,
) -> Option<EscalierCalcule> {
    if !(hauteur.is_finite() && hauteur > 0.0) {
        return None;
    }
    let calcule = (hauteur / CONTREMARCHE_VISEE - 1e-9).ceil() as usize;
    let n = contremarches
        .filter(|n| CONTREMARCHES_PERMISES.contains(n))
        .unwrap_or(calcule.clamp(
            *CONTREMARCHES_PERMISES.start(),
            *CONTREMARCHES_PERMISES.end(),
        ));
    Some(EscalierCalcule {
        hauteur,
        contremarches: n,
        contremarche: hauteur / n as f64,
        // Un escalier droit a n − 1 marches sur sa profondeur.
        giron: (forme == FormeDEscalier::Droit).then(|| profondeur / (n - 1) as f64),
    })
}

/// La largeur d'une volée, en part de la largeur et de la profondeur de
/// l'escalier (`volees` côte à côte dans la largeur).
fn parts_de_volee((largeur, profondeur): (f64, f64), volees: f64) -> (f64, f64) {
    let volee = LARGEUR_DE_VOLEE
        .min(largeur / volees)
        .min(profondeur)
        .max(1.0);
    (volee / largeur.max(1.0), volee / profondeur.max(1.0))
}

/// Le nombre de marches de chaque volée : (première, seconde), sur `marches`
/// marches au total.
fn partager(marches: usize) -> (usize, usize) {
    let premiere = marches.div_ceil(2);
    (premiere, marches - premiere)
}

/// Un point d'un colimaçon : à `r` (part du rayon) et `angle` (radians) du centre.
fn point_du_colimacon((largeur, profondeur): (f64, f64), r: f64, angle: f64) -> (f64, f64) {
    let rayon = largeur.min(profondeur) / 2.0;
    (
        0.5 + r * rayon / largeur.max(1.0) * angle.cos(),
        0.5 + r * rayon / profondeur.max(1.0) * angle.sin(),
    )
}

/// L'angle de la k-ième marche d'un colimaçon de `marches` marches : elles
/// font 330° en tout, à partir du bas du cercle, dans le sens horaire.
fn angle_du_colimacon(k: usize, marches: usize) -> f64 {
    (90.0 - 330.0 * k as f64 / marches.max(1) as f64).to_radians()
}

/// Le symbole d'un escalier de `contremarches` contremarches : ses marches,
/// ses paliers et la flèche qui monte.
pub fn symbole_d_escalier(
    forme: FormeDEscalier,
    dimensions: (f64, f64),
    contremarches: usize,
) -> Vec<Trait> {
    use Trait::Ligne;
    let n = contremarches.max(2);
    let fleche = |traits: &mut Vec<Trait>, points: &[(f64, f64)]| {
        for paire in points.windows(2) {
            traits.push(Ligne(paire[0], paire[1]));
        }
        // La pointe, dans le sens du dernier trait.
        if let [.., a, b] = points {
            let (dx, dy) = (b.0 - a.0, b.1 - a.1);
            let longueur = dx.hypot(dy).max(1e-9);
            let (ux, uy) = (dx / longueur * 0.07, dy / longueur * 0.07);
            traits.push(Ligne(*b, (b.0 - ux - uy * 0.8, b.1 - uy + ux * 0.8)));
            traits.push(Ligne(*b, (b.0 - ux + uy * 0.8, b.1 - uy - ux * 0.8)));
        }
    };
    let mut traits = Vec::new();
    match forme {
        FormeDEscalier::Droit => {
            for k in 1..n {
                let v = k as f64 / n as f64;
                traits.push(Ligne((0.0, v), (1.0, v)));
            }
            fleche(&mut traits, &[(0.5, 0.94), (0.5, 0.06)]);
        }
        FormeDEscalier::EnL | FormeDEscalier::Tournant => {
            let (su, sv) = parts_de_volee(dimensions, 1.0);
            let dansantes = if forme == FormeDEscalier::Tournant {
                2
            } else {
                1
            };
            let (t1, t2) = partager(n.saturating_sub(1 + dansantes).max(2));
            for k in 1..t1 {
                let v = 1.0 - (1.0 - sv) * k as f64 / t1 as f64;
                traits.push(Ligne((0.0, v), (su, v)));
            }
            for k in 1..t2 {
                let u = su + (1.0 - su) * k as f64 / t2 as f64;
                traits.push(Ligne((u, 0.0), (u, sv)));
            }
            // Le coin : un palier, ou deux marches dansantes.
            traits.push(Ligne((0.0, sv), (su, sv)));
            traits.push(Ligne((su, 0.0), (su, sv)));
            if forme == FormeDEscalier::Tournant {
                traits.push(Ligne((0.0, 0.0), (su, sv)));
            }
            traits.push(Ligne((su, sv), (su, 1.0)));
            traits.push(Ligne((su, sv), (1.0, sv)));
            fleche(
                &mut traits,
                &[(su / 2.0, 0.95), (su / 2.0, sv / 2.0), (0.96, sv / 2.0)],
            );
        }
        FormeDEscalier::EnU => {
            let (su, sv) = parts_de_volee(dimensions, 2.0);
            let (t1, t2) = partager(n.saturating_sub(2).max(2));
            for k in 1..t1 {
                let v = 1.0 - (1.0 - sv) * k as f64 / t1 as f64;
                traits.push(Ligne((0.0, v), (su, v)));
            }
            for k in 1..t2 {
                let v = sv + (1.0 - sv) * k as f64 / t2 as f64;
                traits.push(Ligne((1.0 - su, v), (1.0, v)));
            }
            traits.push(Ligne((0.0, sv), (su, sv)));
            traits.push(Ligne((1.0 - su, sv), (1.0, sv)));
            traits.push(Ligne((su, sv), (su, 1.0)));
            traits.push(Ligne((1.0 - su, sv), (1.0 - su, 1.0)));
            fleche(
                &mut traits,
                &[
                    (su / 2.0, 0.95),
                    (su / 2.0, sv / 2.0),
                    (1.0 - su / 2.0, sv / 2.0),
                    (1.0 - su / 2.0, 0.95),
                ],
            );
        }
        FormeDEscalier::Colimacon => {
            let marches = n - 1;
            traits.push(Trait::Cercle((0.5, 0.5), 0.5));
            traits.push(Trait::Cercle((0.5, 0.5), 0.08));
            for k in 0..=marches {
                let angle = angle_du_colimacon(k, marches);
                traits.push(Ligne(
                    point_du_colimacon(dimensions, 0.16, angle),
                    point_du_colimacon(dimensions, 1.0, angle),
                ));
            }
            // La flèche suit le cercle, à mi-rayon.
            let arc: Vec<(f64, f64)> = (0..=10)
                .map(|k| {
                    let angle = angle_du_colimacon(k * marches / 10, marches);
                    point_du_colimacon(dimensions, 0.58, angle)
                })
                .collect();
            fleche(&mut traits, &arc);
        }
    }
    traits
}

/// Une marche (ou un palier) en 3D : ses quatre coins sur le plan, son bas et
/// son haut (pouces), et vrai si sa face avant est cachée par la suivante.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MarcheEn3d {
    pub coins: [(f64, f64); 4],
    pub bas: f64,
    pub haut: f64,
    pub sans_l_avant: bool,
}

impl Objet {
    /// Les marches et les paliers d'un escalier, en blocs, du bas vers le haut.
    pub fn marches_en_3d(
        &self,
        dimensions: (f64, f64),
        calcul: &EscalierCalcule,
    ) -> Vec<MarcheEn3d> {
        let n = calcul.contremarches.max(2);
        let r = calcul.hauteur / n as f64;
        let rectangle = |(u0, u1): (f64, f64), (v0, v1): (f64, f64)| {
            [(u0, v0), (u1, v0), (u1, v1), (u0, v1)].map(|c| self.vers_le_plan(c, dimensions))
        };
        let quad = |points: [(f64, f64); 4]| points.map(|c| self.vers_le_plan(c, dimensions));
        let marche = |coins, niveau: usize, sans_l_avant| MarcheEn3d {
            coins,
            bas: 0.0,
            haut: r * niveau as f64,
            sans_l_avant,
        };
        let mut marches = Vec::new();
        match self.forme {
            FormeDEscalier::Droit => {
                for k in 0..n {
                    let v0 = 1.0 - (k + 1) as f64 / n as f64;
                    let v1 = 1.0 - k as f64 / n as f64;
                    marches.push(marche(rectangle((0.0, 1.0), (v0, v1)), k + 1, k + 1 < n));
                }
            }
            FormeDEscalier::EnL | FormeDEscalier::Tournant => {
                let (su, sv) = parts_de_volee(dimensions, 1.0);
                let tournant = self.forme == FormeDEscalier::Tournant;
                let dansantes = if tournant { 2 } else { 1 };
                let (t1, t2) = partager(n.saturating_sub(1 + dansantes).max(2));
                for k in 0..t1 {
                    let v0 = 1.0 - (1.0 - sv) * (k + 1) as f64 / t1 as f64;
                    let v1 = 1.0 - (1.0 - sv) * k as f64 / t1 as f64;
                    marches.push(marche(rectangle((0.0, su), (v0, v1)), k + 1, false));
                }
                if tournant {
                    // Deux marches dansantes, de part et d'autre de la diagonale du
                    // coin (un triangle, avec un quatrième point sur la diagonale).
                    let milieu = (su / 2.0, sv / 2.0);
                    marches.push(marche(
                        quad([(0.0, sv), (0.0, 0.0), milieu, (su, sv)]),
                        t1 + 1,
                        false,
                    ));
                    marches.push(marche(
                        quad([(0.0, 0.0), (su, 0.0), (su, sv), milieu]),
                        t1 + 2,
                        false,
                    ));
                } else {
                    marches.push(marche(rectangle((0.0, su), (0.0, sv)), t1 + 1, false));
                }
                for k in 0..t2 {
                    let u0 = su + (1.0 - su) * k as f64 / t2 as f64;
                    let u1 = su + (1.0 - su) * (k + 1) as f64 / t2 as f64;
                    marches.push(marche(
                        rectangle((u0, u1), (0.0, sv)),
                        t1 + dansantes + k + 1,
                        false,
                    ));
                }
            }
            FormeDEscalier::EnU => {
                let (su, sv) = parts_de_volee(dimensions, 2.0);
                let (t1, t2) = partager(n.saturating_sub(2).max(2));
                for k in 0..t1 {
                    let v0 = 1.0 - (1.0 - sv) * (k + 1) as f64 / t1 as f64;
                    let v1 = 1.0 - (1.0 - sv) * k as f64 / t1 as f64;
                    marches.push(marche(rectangle((0.0, su), (v0, v1)), k + 1, false));
                }
                marches.push(marche(rectangle((0.0, 1.0), (0.0, sv)), t1 + 1, false));
                for k in 0..t2 {
                    let v0 = sv + (1.0 - sv) * k as f64 / t2 as f64;
                    let v1 = sv + (1.0 - sv) * (k + 1) as f64 / t2 as f64;
                    marches.push(marche(
                        rectangle((1.0 - su, 1.0), (v0, v1)),
                        t1 + 2 + k,
                        false,
                    ));
                }
            }
            FormeDEscalier::Colimacon => {
                let tours = n - 1;
                for k in 0..tours {
                    let (a0, a1) = (
                        angle_du_colimacon(k, tours),
                        angle_du_colimacon(k + 1, tours),
                    );
                    let coins = quad([
                        point_du_colimacon(dimensions, 0.08, a0),
                        point_du_colimacon(dimensions, 1.0, a0),
                        point_du_colimacon(dimensions, 1.0, a1),
                        point_du_colimacon(dimensions, 0.08, a1),
                    ]);
                    marches.push(marche(coins, k + 1, false));
                }
                // Le poteau, jusqu'en haut.
                let (pu, pv) = parts_de_volee(dimensions, 1.0);
                let poteau = (pu.min(pv) * 0.12).max(0.01);
                marches.push(MarcheEn3d {
                    coins: rectangle((0.5 - poteau, 0.5 + poteau), (0.5 - poteau, 0.5 + poteau)),
                    bas: 0.0,
                    haut: calcul.hauteur,
                    sans_l_avant: false,
                });
            }
        }
        marches
    }
}

// ----------------------------------------------------------------------
// Collisions (R-98) : un objet ne chevauche ni un autre objet, ni une cloison,
// ni les murs de sa pièce.
// ----------------------------------------------------------------------

/// Tolérance des collisions, en pouces : deux formes qui se touchent ne se
/// chevauchent pas.
const TOLERANCE: f64 = 1e-6;

/// Vrai si deux polygones convexes (en pouces, sur le plan) se chevauchent,
/// au sens strict : se toucher par un côté ou un coin n'est pas chevaucher.
/// Un segment (deux points) compte comme un polygone aplati. Méthode des axes
/// séparateurs.
pub fn se_chevauchent(a: &[(f64, f64)], b: &[(f64, f64)]) -> bool {
    let axes = |forme: &[(f64, f64)]| -> Vec<(f64, f64)> {
        (0..forme.len())
            .filter_map(|k| {
                let (p, q) = (forme[k], forme[(k + 1) % forme.len()]);
                let (dx, dy) = (q.0 - p.0, q.1 - p.1);
                let longueur = dx.hypot(dy);
                (longueur > TOLERANCE).then(|| (-dy / longueur, dx / longueur))
            })
            .collect()
    };
    let projeter = |forme: &[(f64, f64)], axe: (f64, f64)| {
        forme.iter().fold((f64::MAX, f64::MIN), |(bas, haut), p| {
            let x = p.0 * axe.0 + p.1 * axe.1;
            (bas.min(x), haut.max(x))
        })
    };
    axes(a).into_iter().chain(axes(b)).all(|axe| {
        let (a0, a1) = projeter(a, axe);
        let (b0, b1) = projeter(b, axe);
        // Un segment n'a pas d'épaisseur sur sa normale : il traverse l'autre
        // forme s'il tombe strictement dedans.
        if a1 - a0 <= TOLERANCE {
            b0 < a0 - TOLERANCE && a1 + TOLERANCE < b1
        } else if b1 - b0 <= TOLERANCE {
            a0 < b0 - TOLERANCE && b1 + TOLERANCE < a1
        } else {
            a1.min(b1) - a0.max(b0) > TOLERANCE
        }
    })
}

/// Vrai si le point est dans le polygone (règle des croisements).
fn dans_le_polygone((x, y): (f64, f64), polygone: &[(f64, f64)]) -> bool {
    let mut dedans = false;
    for k in 0..polygone.len() {
        let (a, b) = (polygone[k], polygone[(k + 1) % polygone.len()]);
        if (a.1 > y) != (b.1 > y) && x < a.0 + (y - a.1) * (b.0 - a.0) / (b.1 - a.1) {
            dedans = !dedans;
        }
    }
    dedans
}

/// Ce qui encombre une pièce pour un objet qu'on déplace : les autres objets
/// et les morceaux de cloison (des quadrilatères convexes), et le contour
/// intérieur des murs, s'il est connu.
pub struct Encombrement<'a> {
    pub obstacles: Vec<[(f64, f64); 4]>,
    pub piece: Option<&'a [(f64, f64)]>,
}

impl Encombrement<'_> {
    /// Vrai si un objet de ces quatre coins est à sa place : dans la pièce, et
    /// sans chevaucher d'obstacle (il peut les toucher).
    pub fn libre(&self, coins: &[(f64, f64); 4]) -> bool {
        if let Some(piece) = self.piece {
            // Aucun mur ne traverse l'objet, et l'objet est du bon côté.
            let traverse = (0..piece.len())
                .any(|k| se_chevauchent(&[piece[k], piece[(k + 1) % piece.len()]], coins));
            let centre = (
                coins.iter().map(|c| c.0).sum::<f64>() / 4.0,
                coins.iter().map(|c| c.1).sum::<f64>() / 4.0,
            );
            if traverse || !dans_le_polygone(centre, piece) {
                return false;
            }
        }
        !self.obstacles.iter().any(|o| se_chevauchent(o, coins))
    }
}

/// Les coins de l'objet s'il avait ce centre.
fn coins_au_centre(objet: &Objet, dimensions: (f64, f64), centre: (f64, f64)) -> [(f64, f64); 4] {
    let mut deplace = objet.clone();
    deplace.centre = centre;
    deplace.coins(dimensions)
}

/// Où l'objet peut aller, au plus près du centre `voulu` : il avance pas à pas
/// (`pas`, en pouces) et s'arrête contre ce qui l'encombre, puis glisse le long
/// (le mouvement se fait en x puis en y, ou en y puis en x : le plus proche du
/// but l'emporte). Un objet qui chevauche déjà quelque chose (un plan d'avant
/// les collisions) va où l'on veut, pour qu'on puisse le dégager.
pub fn deplacer_sans_chevaucher(
    objet: &Objet,
    dimensions: (f64, f64),
    voulu: (f64, f64),
    encombrement: &Encombrement<'_>,
    pas: f64,
) -> (f64, f64) {
    let depart = objet.centre;
    let libre =
        |centre: (f64, f64)| encombrement.libre(&coins_au_centre(objet, dimensions, centre));
    // Un objet qui chevauche déjà quelque chose va où l'on veut; un chemin
    // direct libre se prend tel quel.
    if !libre(depart) || trajet_libre(depart, voulu, pas, &libre) {
        return voulu;
    }
    // Avancer en ligne droite jusqu'au dernier pas libre.
    let avancer = |de: (f64, f64), vers: (f64, f64)| -> (f64, f64) {
        let (dx, dy) = (vers.0 - de.0, vers.1 - de.1);
        let n = nombre_de_pas(dx, dy, pas);
        let mut dernier = de;
        for k in 1..=n {
            let t = k as f64 / n as f64;
            let point = (de.0 + dx * t, de.1 + dy * t);
            if !libre(point) {
                break;
            }
            dernier = point;
        }
        dernier
    };
    // En x puis en y, ou en y puis en x : l'objet glisse le long de l'obstacle.
    let x_d_abord = avancer(depart, (voulu.0, depart.1));
    let par_x = avancer(x_d_abord, (x_d_abord.0, voulu.1));
    let y_d_abord = avancer(depart, (depart.0, voulu.1));
    let par_y = avancer(y_d_abord, (voulu.0, y_d_abord.1));
    let distance = |p: (f64, f64)| (p.0 - voulu.0).hypot(p.1 - voulu.1);
    if distance(par_x) <= distance(par_y) {
        par_x
    } else {
        par_y
    }
}

/// Le nombre de pas d'au plus `pas` pouces pour aller de (0, 0) à (dx, dy)
/// (sur la grille, un déplacement d'un nombre entier de pas en garde le compte).
fn nombre_de_pas(dx: f64, dy: f64, pas: f64) -> usize {
    ((dx.abs().max(dy.abs()) / pas) - 1e-9).ceil().max(1.0) as usize
}

/// Vrai si l'objet passe de `de` à `vers` sans rien traverser (pas à pas).
fn trajet_libre(
    de: (f64, f64),
    vers: (f64, f64),
    pas: f64,
    libre: &dyn Fn((f64, f64)) -> bool,
) -> bool {
    let (dx, dy) = (vers.0 - de.0, vers.1 - de.1);
    let n = nombre_de_pas(dx, dy, pas);
    (1..=n).all(|k| {
        let t = k as f64 / n as f64;
        libre((de.0 + dx * t, de.1 + dy * t))
    })
}

/// Le centre libre le plus proche de celui de l'objet, à `rayon` pouces au
/// plus, sur la grille de `pas` pouces; rien s'il n'y en a pas. L'objet déjà
/// libre reste où il est.
pub fn position_libre_proche(
    objet: &Objet,
    dimensions: (f64, f64),
    encombrement: &Encombrement<'_>,
    pas: f64,
    rayon: f64,
) -> Option<(f64, f64)> {
    let (cx, cy) = objet.centre;
    let libre =
        |centre: (f64, f64)| encombrement.libre(&coins_au_centre(objet, dimensions, centre));
    if libre((cx, cy)) {
        return Some((cx, cy));
    }
    let anneaux = (rayon / pas).ceil() as i64;
    for anneau in 1..=anneaux {
        // Les points de l'anneau, du plus proche au plus loin du centre.
        let mut points: Vec<(f64, f64)> = Vec::new();
        for i in -anneau..=anneau {
            for j in -anneau..=anneau {
                if i.abs().max(j.abs()) == anneau {
                    points.push((cx + i as f64 * pas, cy + j as f64 * pas));
                }
            }
        }
        points.sort_by(|a, b| {
            let da = (a.0 - cx).hypot(a.1 - cy);
            let db = (b.0 - cx).hypot(b.1 - cy);
            da.total_cmp(&db)
        });
        if let Some(point) = points.into_iter().find(|p| libre(*p)) {
            return Some(point);
        }
    }
    None
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
    fn les_parties_d_un_meuble_ne_se_chevauchent_pas() {
        // Deux parties se séparent en u, en v ou en hauteur : la 3D les peint
        // dans le bon ordre.
        let disjoints = |a: (f64, f64), b: (f64, f64)| a.1 <= b.0 + 1e-9 || b.1 <= a.0 + 1e-9;
        for genre in GenreDObjet::TOUS {
            let parties = genre.volumes();
            assert_eq!(
                parties.is_empty(),
                genre == GenreDObjet::Escalier,
                "{genre:?}"
            );
            for (i, a) in parties.iter().enumerate() {
                assert!(
                    a.u.0 < a.u.1 && a.v.0 < a.v.1 && a.bas < a.haut,
                    "{genre:?} {i}"
                );
                for b in &parties[i + 1..] {
                    assert!(
                        disjoints(a.u, b.u)
                            || disjoints(a.v, b.v)
                            || disjoints((a.bas, a.haut), (b.bas, b.haut)),
                        "{genre:?} : {a:?} et {b:?}"
                    );
                }
            }
        }
        // Une table : son plateau, quatre pattes, et six chaises de deux parties.
        assert_eq!(GenreDObjet::Table.volumes().len(), 1 + 4 + 12);
    }

    #[test]
    fn un_objet_pivote_au_15_degres() {
        let mut table = Objet::nouveau(GenreDObjet::Table, (100.0, 100.0));
        let dimensions = (36.0, 60.0);
        table.pivoter(15.0);
        table.pivoter(15.0);
        assert!((table.angle_total() - 30.0).abs() < 1e-9);
        // Le coin (0, 0) tourne de 30° autour du centre.
        let (dx, dy) = (-18.0_f64, -30.0_f64);
        let (sinus, cosinus) = 30.0_f64.to_radians().sin_cos();
        let attendu = (
            100.0 + dx * cosinus - dy * sinus,
            100.0 + dx * sinus + dy * cosinus,
        );
        let coin = table.coins(dimensions)[0];
        assert!((coin.0 - attendu.0).abs() < 1e-9 && (coin.1 - attendu.1).abs() < 1e-9);
        assert!(table.contient((100.0, 100.0), dimensions));
        // Six pas de 15° font un quart de tour exact.
        let mut lit = Objet::nouveau(GenreDObjet::Lit, (0.0, 0.0));
        for _ in 0..6 {
            lit.pivoter(15.0);
        }
        assert_eq!(lit.coins((60.0, 80.0))[0], (40.0, -30.0));
        // −15° revient en arrière.
        lit.pivoter(-15.0);
        assert!((lit.angle_total() - 75.0).abs() < 1e-9);
    }

    #[test]
    fn le_symbole_d_un_escalier() {
        // 10 pi de long : 12 marches, donc 11 lignes de marche et la flèche.
        let traits = GenreDObjet::Escalier.symbole(120.0);
        assert_eq!(traits.len(), 11 + 3);
        let escalier = Objet::nouveau(GenreDObjet::Escalier, (0.0, 0.0));
        // Le contour (4 traits) et le symbole.
        assert_eq!(
            escalier.traits_sur_le_plan((36.0, 120.0), None).len(),
            4 + 14
        );
        // Chaque genre a des dimensions et un symbole.
        for genre in GenreDObjet::TOUS {
            let (largeur, profondeur, hauteur) = genre.dimensions();
            assert!(largeur > 0.0 && profondeur > 0.0 && hauteur >= 0.0);
            assert!(!genre.symbole(profondeur).is_empty());
        }
    }

    /// Une pièce de 20 pi sur 12 pi (en pouces, y vers le bas).
    const PIECE: [(f64, f64); 4] = [(0.0, 0.0), (240.0, 0.0), (240.0, 144.0), (0.0, 144.0)];

    fn table_en(centre: (f64, f64)) -> (Objet, (f64, f64)) {
        (Objet::nouveau(GenreDObjet::Table, centre), (36.0, 60.0))
    }

    #[test]
    fn se_toucher_n_est_pas_chevaucher() {
        let a = [(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)];
        let colle = [(10.0, 0.0), (20.0, 0.0), (20.0, 10.0), (10.0, 10.0)];
        let mord = [(9.5, 2.0), (20.0, 2.0), (20.0, 8.0), (9.5, 8.0)];
        assert!(!se_chevauchent(&a, &colle));
        assert!(se_chevauchent(&a, &mord));
        // Un carré tourné de 45° qui ne touche que la pointe… ou qui mord.
        let losange = |dx: f64| {
            [
                (15.0 + dx, 5.0),
                (20.0 + dx, 0.0),
                (25.0 + dx, 5.0),
                (20.0 + dx, 10.0),
            ]
        };
        assert!(!se_chevauchent(&a, &losange(-5.0)));
        assert!(se_chevauchent(&a, &losange(-6.0)));
        // Un segment le long d'un côté ne traverse pas; en travers, il traverse.
        assert!(!se_chevauchent(&[(0.0, 10.0), (30.0, 10.0)], &a));
        assert!(se_chevauchent(&[(5.0, -5.0), (5.0, 15.0)], &a));
    }

    #[test]
    fn un_meuble_s_arrete_contre_un_autre_et_glisse_le_long() {
        // Une table de 36 po sur 60 po contre un lit posé à droite.
        let lit = Objet::nouveau(GenreDObjet::Lit, (150.0, 72.0));
        let encombrement = Encombrement {
            obstacles: vec![lit.coins((60.0, 80.0))],
            piece: Some(&PIECE),
        };
        let (table, dimensions) = table_en((40.0, 72.0));
        // Le lit occupe x de 120 à 180 : la table (demi-largeur 18) s'arrête à x = 102.
        let arrivee =
            deplacer_sans_chevaucher(&table, dimensions, (150.0, 72.0), &encombrement, 1.0);
        assert_eq!(arrivee, (102.0, 72.0));
        // En visant en biais, elle avance en y le long du lit.
        let arrivee =
            deplacer_sans_chevaucher(&table, dimensions, (130.0, 90.0), &encombrement, 1.0);
        assert_eq!(arrivee, (102.0, 90.0));
        // Un chemin libre se prend tel quel.
        let arrivee =
            deplacer_sans_chevaucher(&table, dimensions, (60.0, 80.0), &encombrement, 1.0);
        assert_eq!(arrivee, (60.0, 80.0));
    }

    #[test]
    fn un_meuble_reste_dans_la_piece_et_ne_traverse_pas_une_cloison() {
        // Une cloison de 5 po d'épaisseur, de haut en bas, à x de 118 à 123.
        let cloison = [(118.0, 0.0), (123.0, 0.0), (123.0, 144.0), (118.0, 144.0)];
        let encombrement = Encombrement {
            obstacles: vec![cloison],
            piece: Some(&PIECE),
        };
        let (table, dimensions) = table_en((40.0, 72.0));
        // Un geste rapide vers l'autre côté s'arrête contre la cloison, sans la sauter.
        let arrivee =
            deplacer_sans_chevaucher(&table, dimensions, (200.0, 72.0), &encombrement, 1.0);
        assert_eq!(arrivee, (100.0, 72.0));
        // Vers le mur du haut : la table (demi-profondeur 30) s'arrête à y = 30.
        let arrivee =
            deplacer_sans_chevaucher(&table, dimensions, (40.0, -50.0), &encombrement, 1.0);
        assert_eq!(arrivee, (40.0, 30.0));
        // Dehors, elle n'est pas libre.
        assert!(!encombrement.libre(&table_en((-100.0, 72.0)).0.coins(dimensions)));
    }

    #[test]
    fn une_piece_en_l_retient_un_meuble_dans_son_coin_rentrant() {
        // Une pièce en L : le coin de (120, 72) à (240, 144) est dehors.
        let en_l = [
            (0.0, 0.0),
            (240.0, 0.0),
            (240.0, 72.0),
            (120.0, 72.0),
            (120.0, 144.0),
            (0.0, 144.0),
        ];
        let encombrement = Encombrement {
            obstacles: Vec::new(),
            piece: Some(&en_l),
        };
        let (table, dimensions) = table_en((60.0, 100.0));
        // Vers la droite, le mur du coin rentrant l'arrête (x de 120 moins 18).
        let arrivee =
            deplacer_sans_chevaucher(&table, dimensions, (200.0, 100.0), &encombrement, 1.0);
        assert_eq!(arrivee, (102.0, 100.0));
        // À cheval sur le coin rentrant : pas libre, même si ses coins sont dedans.
        let travers = Objet::nouveau(GenreDObjet::Sofa, (130.0, 60.0));
        assert!(!encombrement.libre(&travers.coins((84.0, 36.0))));
    }

    #[test]
    fn un_meuble_qui_chevauche_deja_se_degage_librement() {
        let lit = Objet::nouveau(GenreDObjet::Lit, (60.0, 72.0));
        let encombrement = Encombrement {
            obstacles: vec![lit.coins((60.0, 80.0))],
            piece: Some(&PIECE),
        };
        // Une table posée sur le lit (un plan d'avant les collisions).
        let (table, dimensions) = table_en((60.0, 72.0));
        assert!(!encombrement.libre(&table.coins(dimensions)));
        let arrivee =
            deplacer_sans_chevaucher(&table, dimensions, (65.0, 72.0), &encombrement, 1.0);
        assert_eq!(arrivee, (65.0, 72.0));
        // La place libre la plus proche est juste à côté du lit.
        let proche = position_libre_proche(&table, dimensions, &encombrement, 1.0, 120.0).unwrap();
        assert!(encombrement.libre(&table_en(proche).0.coins(dimensions)));
        assert!(
            (proche.0 - 60.0).abs() + (proche.1 - 72.0).abs() <= 50.0,
            "{proche:?}"
        );
        // Sans place assez proche : rien.
        assert!(position_libre_proche(&table, dimensions, &encombrement, 1.0, 3.0).is_none());
    }

    #[test]
    fn le_calcul_d_un_escalier() {
        // 8 pi 10 po à monter : 15 contremarches de 7 1/15 po (106 / 15).
        let droit = calculer_l_escalier(FormeDEscalier::Droit, (36.0, 140.0), 106.0, None).unwrap();
        assert_eq!(droit.contremarches, 15);
        assert!((droit.contremarche - 106.0 / 15.0).abs() < 1e-9);
        // 14 marches sur 140 po : un giron de 10 po.
        assert!((droit.giron.unwrap() - 10.0).abs() < 1e-9);
        // Une hauteur juste divisible ne prend pas de contremarche de trop.
        let juste = calculer_l_escalier(FormeDEscalier::Droit, (36.0, 120.0), 90.0, None).unwrap();
        assert_eq!(juste.contremarches, 12);
        // Le nombre inscrit l'emporte; un nombre déraisonnable est ignoré.
        let inscrit =
            calculer_l_escalier(FormeDEscalier::EnL, (72.0, 96.0), 96.0, Some(14)).unwrap();
        assert_eq!((inscrit.contremarches, inscrit.giron), (14, None));
        let ignore =
            calculer_l_escalier(FormeDEscalier::Droit, (36.0, 120.0), 96.0, Some(500)).unwrap();
        assert_eq!(ignore.contremarches, 13);
        assert!(calculer_l_escalier(FormeDEscalier::Droit, (36.0, 120.0), 0.0, None).is_none());
    }

    #[test]
    fn chaque_forme_d_escalier_a_son_symbole_et_ses_marches() {
        for forme in FormeDEscalier::TOUTES {
            let mut escalier = Objet::nouveau(GenreDObjet::Escalier, (100.0, 100.0));
            escalier.forme = forme;
            let dimensions = escalier.dimensions_de_depart();
            assert_eq!(dimensions, forme.dimensions());
            let calcul = calculer_l_escalier(forme, dimensions, 96.0, None).unwrap();
            let traits = escalier.traits_sur_le_plan(dimensions, Some(calcul.contremarches));
            assert!(traits.len() >= 8, "{forme:?} : {} traits", traits.len());
            let marches = escalier.marches_en_3d(dimensions, &calcul);
            // Chaque bloc a des côtés de longueur non nulle, et le plus haut
            // atteint presque la hauteur à monter.
            for marche in &marches {
                for k in 0..4 {
                    let (a, b) = (marche.coins[k], marche.coins[(k + 1) % 4]);
                    assert!((a.0 - b.0).hypot(a.1 - b.1) > 1e-6, "{forme:?}");
                }
            }
            let plus_haut = marches.iter().map(|m| m.haut).fold(0.0, f64::max);
            assert!(
                plus_haut >= 96.0 * 12.0 / 13.0 - 1e-9,
                "{forme:?} : {plus_haut}"
            );
            // Les marches restent dans l'emprise de l'escalier.
            for marche in &marches {
                for (x, y) in marche.coins {
                    assert!((x - 100.0).abs() <= dimensions.0 / 2.0 + 1e-6, "{forme:?}");
                    assert!((y - 100.0).abs() <= dimensions.1 / 2.0 + 1e-6, "{forme:?}");
                }
            }
        }
        // Le droit : une marche par contremarche; le colimaçon : son poteau en plus.
        let droit = Objet::nouveau(GenreDObjet::Escalier, (0.0, 0.0));
        let calcul = calculer_l_escalier(FormeDEscalier::Droit, (36.0, 120.0), 96.0, None).unwrap();
        assert_eq!(droit.marches_en_3d((36.0, 120.0), &calcul).len(), 13);
        let mut colimacon = droit.clone();
        colimacon.forme = FormeDEscalier::Colimacon;
        assert_eq!(colimacon.marches_en_3d((60.0, 60.0), &calcul).len(), 12 + 1);
    }
}
