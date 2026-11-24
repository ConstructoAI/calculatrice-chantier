//! La pièce en 3D (section 25) : la maquette du noyau, vue par une caméra qui
//! tourne autour d'elle, peinte face par face, de la plus éloignée à la plus
//! proche. Les murs vus de l'extérieur sont translucides : on voit dedans.
//! La visite (évolution 10) met l'œil dans la pièce, à 5 pi 6 po du plancher;
//! chaque face et chaque arête sont alors coupées au plan proche.

use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};
use noyau::maquette::{GenreDeFace, Maquette};
use noyau::mobilier::Matiere;

use crate::theme::{self, Palette};

/// Où regarde la caméra : son lacet (autour de la verticale), son tangage
/// (au-dessus de l'horizon), en radians, et un zoom.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Orbite {
    pub lacet: f32,
    pub tangage: f32,
    pub zoom: f32,
    /// La visite à hauteur d'œil, quand elle est en cours (évolution 10).
    pub visite: Option<Visite>,
}

/// La hauteur de l'œil pendant la visite : 5 pi 6 po (1,68 m).
pub const HAUTEUR_D_OEIL: f64 = 66.0;

/// Le plan proche de la visite : ce qui est à moins de 4 po devant l'œil est
/// coupé.
const PLAN_PROCHE: f64 = 4.0;

/// La visite : où est l'œil sur le plancher (x, z), où il regarde (son lacet,
/// en radians), et l'étendue où il peut aller.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Visite {
    pub x: f64,
    pub z: f64,
    pub lacet: f64,
    bornes: [f64; 4],
}

impl Default for Orbite {
    fn default() -> Self {
        Orbite {
            lacet: 0.65,
            tangage: 0.62,
            zoom: 1.0,
            visite: None,
        }
    }
}

impl Orbite {
    /// Commence la visite : l'œil à hauteur d'œil, contre un mur, regardant
    /// dans le sens le plus long du plancher; libre d'aller partout sur le
    /// plancher, à 1 pi près des murs.
    pub fn commencer_la_visite(&mut self, maquette: &Maquette) {
        let planchers: Vec<&noyau::maquette::Face> = maquette
            .faces
            .iter()
            .filter(|face| face.genre == GenreDeFace::Plancher)
            .collect();
        let faces: Vec<&noyau::maquette::Face> = if planchers.is_empty() {
            maquette.faces.iter().collect()
        } else {
            planchers
        };
        let (mut x0, mut x1, mut z0, mut z1) = (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
        for sommet in faces.iter().flat_map(|face| face.sommets.iter()) {
            x0 = x0.min(sommet[0]);
            x1 = x1.max(sommet[0]);
            z0 = z0.min(sommet[2]);
            z1 = z1.max(sommet[2]);
        }
        if x0 > x1 {
            return;
        }
        let (largeur, profondeur) = (x1 - x0, z1 - z0);
        let marge = 12.0_f64.min(largeur / 4.0).min(profondeur / 4.0);
        let recul = 18.0_f64.min(largeur.max(profondeur) / 4.0);
        let (x, z, lacet) = if largeur > profondeur {
            (x0 + recul, (z0 + z1) / 2.0, std::f64::consts::FRAC_PI_2)
        } else {
            ((x0 + x1) / 2.0, z0 + recul, 0.0)
        };
        self.visite = Some(Visite {
            x,
            z,
            lacet,
            bornes: [x0 + marge, x1 - marge, z0 + marge, z1 - marge],
        });
    }

    /// Tourne la caméra d'un glissement du doigt, en points de l'écran. Pendant
    /// la visite, glisser de côté tourne la tête, glisser vers le haut avance
    /// (vers le bas, recule).
    pub fn tourner(&mut self, glissement: Vec2) {
        if let Some(visite) = &mut self.visite {
            visite.lacet -= f64::from(glissement.x) * 0.006;
            let pas = -f64::from(glissement.y) * 1.2;
            let [x0, x1, z0, z1] = visite.bornes;
            visite.x = (visite.x + visite.lacet.sin() * pas).clamp(x0, x1);
            visite.z = (visite.z + visite.lacet.cos() * pas).clamp(z0, z1);
            return;
        }
        self.lacet -= glissement.x * 0.01;
        self.tangage = (self.tangage + glissement.y * 0.01).clamp(0.12, 1.45);
    }

    pub fn zoomer(&mut self, facteur: f32) {
        if facteur.is_finite() && facteur > 0.0 {
            self.zoom = (self.zoom * facteur).clamp(0.4, 6.0);
        }
    }
}

type V3 = [f64; 3];

fn moins(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scalaire(a: V3, b: V3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn vectoriel(a: V3, b: V3) -> V3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn unitaire(a: V3) -> V3 {
    let n = scalaire(a, a).sqrt().max(1e-12);
    [a[0] / n, a[1] / n, a[2] / n]
}

/// Un polygone (dans le repère de l'œil : x à droite, y en haut, z devant)
/// coupé au plan proche : ce qui reste devant lui.
fn couper_au_plan_proche(polygone: &[V3]) -> Vec<V3> {
    let mut reste = Vec::with_capacity(polygone.len() + 2);
    for (i, &courant) in polygone.iter().enumerate() {
        let precedent = polygone[(i + polygone.len() - 1) % polygone.len()];
        let (devant, avant) = (courant[2] >= PLAN_PROCHE, precedent[2] >= PLAN_PROCHE);
        if devant != avant {
            let t = (PLAN_PROCHE - precedent[2]) / (courant[2] - precedent[2]);
            reste.push([
                precedent[0] + (courant[0] - precedent[0]) * t,
                precedent[1] + (courant[1] - precedent[1]) * t,
                PLAN_PROCHE,
            ]);
        }
        if devant {
            reste.push(courant);
        }
    }
    reste
}

/// Une face projetée : ses points (avant mise à l'échelle), sa profondeur.
struct Projetee {
    genre: GenreDeFace,
    points: Vec<(f64, f64)>,
    profondeur: f64,
    /// Vrai si la caméra voit le côté intérieur de la face.
    de_l_interieur: bool,
    /// Éclairage d'un mur, de 0 à 1, selon sa direction.
    lumiere: f32,
    /// Le bloc de la face (une marche, un meuble, une cloison), s'il y en a un.
    bloc: Option<usize>,
}

/// La couleur d'une matière de meuble dans la vue 3D.
fn couleur_d_une_matiere(matiere: Matiere) -> Color32 {
    match matiere {
        Matiere::Bois => Color32::from_rgb(176, 132, 92),
        Matiere::BoisFonce => Color32::from_rgb(120, 82, 54),
        Matiere::Tissu => Color32::from_rgb(118, 128, 146),
        Matiere::Coussin => Color32::from_rgb(150, 162, 182),
        Matiere::Literie => Color32::from_rgb(240, 240, 246),
        Matiere::Porcelaine => Color32::from_rgb(246, 246, 250),
        Matiere::Metal => Color32::from_rgb(196, 202, 210),
        Matiere::Electromenager => Color32::from_rgb(232, 234, 238),
        Matiere::Noir => Color32::from_rgb(52, 54, 58),
        Matiere::Verre => Color32::from_rgb(170, 212, 236),
        Matiere::Comptoir => Color32::from_rgb(186, 186, 192),
        Matiere::Armoire => Color32::from_rgb(214, 198, 174),
        Matiere::Cuivre => Color32::from_rgb(190, 118, 70),
    }
}

/// Peint la maquette dans la zone.
pub fn dessiner(
    painter: &egui::Painter,
    zone: Rect,
    maquette: &Maquette,
    orbite: &Orbite,
    p: &Palette,
) {
    let tous = maquette.faces.iter().flat_map(|face| face.sommets.iter());
    let (mut min, mut max) = ([f64::MAX; 3], [f64::MIN; 3]);
    for sommet in tous {
        for k in 0..3 {
            min[k] = min[k].min(sommet[k]);
            max[k] = max[k].max(sommet[k]);
        }
    }
    if min[0] > max[0] {
        return;
    }
    let centre = [
        (min[0] + max[0]) / 2.0,
        (min[1] + max[1]) / 2.0,
        (min[2] + max[2]) / 2.0,
    ];
    let rayon = (max[0] - min[0])
        .hypot(max[1] - min[1])
        .hypot(max[2] - min[2])
        .max(1.0)
        / 2.0;

    // La caméra : sur une sphère autour de la pièce, ou, pendant la visite,
    // à hauteur d'œil dans la pièce, regardant à l'horizontale.
    let (oeil, devant) = match orbite.visite {
        // Le regard un peu baissé, de 8° : le plancher et les meubles se
        // voient mieux.
        Some(visite) => {
            let (sinus, cosinus) = (-8.0_f64).to_radians().sin_cos();
            (
                [visite.x, HAUTEUR_D_OEIL, visite.z],
                [
                    visite.lacet.sin() * cosinus,
                    sinus,
                    visite.lacet.cos() * cosinus,
                ],
            )
        }
        None => {
            let (lacet, tangage) = (f64::from(orbite.lacet), f64::from(orbite.tangage));
            let distance = rayon * 2.8;
            let oeil = [
                centre[0] + distance * tangage.cos() * lacet.sin(),
                centre[1] + distance * tangage.sin(),
                centre[2] + distance * tangage.cos() * lacet.cos(),
            ];
            (oeil, unitaire(moins(centre, oeil)))
        }
    };
    let droite = unitaire(vectoriel(devant, [0.0, 1.0, 0.0]));
    let dessus = vectoriel(droite, devant);
    // Un sommet dans le repère de l'œil, puis sur l'écran.
    let vers_l_oeil = |sommet: V3| {
        let v = moins(sommet, oeil);
        [
            scalaire(v, droite),
            scalaire(v, dessus),
            scalaire(v, devant),
        ]
    };
    let projeter = |[x, y, z]: V3| (x / z, -y / z, z);
    let lumiere = unitaire([0.45, 0.0, 0.75]);

    // Le bloc de chaque face, et les arêtes qui appartiennent à un bloc.
    let mut bloc_de_la_face = vec![None; maquette.faces.len()];
    let mut arete_d_un_bloc = vec![false; maquette.aretes.len()];
    for (rang, bloc) in maquette.blocs.iter().enumerate() {
        for face in bloc.faces.clone() {
            if let Some(case) = bloc_de_la_face.get_mut(face) {
                *case = Some(rang);
            }
        }
        for arete in bloc.aretes.clone() {
            if let Some(case) = arete_d_un_bloc.get_mut(arete) {
                *case = true;
            }
        }
    }
    let mut projetees = Vec::with_capacity(maquette.faces.len());
    for (indice, face) in maquette.faces.iter().enumerate() {
        // La face, coupée au plan proche : ce qui passe derrière l'œil ne se
        // voit pas.
        let devant_l_oeil = couper_au_plan_proche(
            &face
                .sommets
                .iter()
                .map(|&s| vers_l_oeil(s))
                .collect::<Vec<_>>(),
        );
        if devant_l_oeil.len() < 3 {
            continue;
        }
        let points: Vec<(f64, f64)> = devant_l_oeil
            .iter()
            .map(|&sommet| {
                let (x, y, _) = projeter(sommet);
                (x, y)
            })
            .collect();
        let profondeur: f64 = devant_l_oeil.iter().map(|sommet| sommet[2]).sum();
        let n = face.sommets.len() as f64;
        let mut milieu = [0.0; 3];
        for sommet in &face.sommets {
            for k in 0..3 {
                milieu[k] += sommet[k] / n;
            }
        }
        // Les faces d'un bloc tournées vers l'arrière ne se voient pas.
        let bloc = bloc_de_la_face[indice];
        if bloc.is_some() && scalaire(face.normale, moins(oeil, milieu)) <= 0.0 {
            continue;
        }
        projetees.push(Projetee {
            bloc,
            genre: face.genre,
            profondeur: profondeur / points.len() as f64,
            points,
            de_l_interieur: scalaire(face.normale, moins(oeil, milieu)) > 0.0,
            lumiere: if face.normale[1].abs() > 0.5 {
                1.0
            } else {
                (0.5 + 0.5 * scalaire(face.normale, lumiere).abs()) as f32
            },
        });
    }
    // La maquette tient dans la zone; pendant la visite, une focale fixe (un
    // champ de 75° en largeur, resserré par le zoom), centrée.
    let (mut x0, mut x1, mut y0, mut y1) = (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
    for &(x, y) in projetees.iter().flat_map(|face| face.points.iter()) {
        x0 = x0.min(x);
        x1 = x1.max(x);
        y0 = y0.min(y);
        y1 = y1.max(y);
    }
    if x0 > x1 {
        return;
    }
    let (echelle, mx, my) = if orbite.visite.is_some() {
        (
            f64::from(zone.width()) / 2.0 / (37.5_f64.to_radians()).tan() * f64::from(orbite.zoom),
            0.0,
            0.0,
        )
    } else {
        (
            0.86 * (f64::from(zone.width()) / (x1 - x0).max(1e-9))
                .min(f64::from(zone.height()) / (y1 - y0).max(1e-9))
                * f64::from(orbite.zoom),
            (x0 + x1) / 2.0,
            (y0 + y1) / 2.0,
        )
    };
    let centre_ecran = zone.center();
    let vers_l_ecran = |(x, y): (f64, f64)| {
        Pos2::new(
            centre_ecran.x + ((x - mx) * echelle) as f32,
            centre_ecran.y + ((y - my) * echelle) as f32,
        )
    };

    // Le plancher d'abord, puis les murs vus de l'intérieur (ils sont derrière
    // tout ce qui est dans la pièce), puis les blocs, enfin les murs vus de
    // l'extérieur, translucides, devant. Les blocs se suivent dans l'ordre que
    // donne leur empreinte vue de l'œil : une longue cloison triée par son
    // seul milieu passerait devant le meuble qui la cache. Le reste va du plus
    // loin au plus proche.
    let ordre = noyau::maquette::ordre_des_blocs(&maquette.blocs, oeil);
    let mut place_du_bloc = vec![0; maquette.blocs.len()];
    for (place, &bloc) in ordre.iter().enumerate() {
        place_du_bloc[bloc] = place;
    }
    let rang = |face: &Projetee| match (face.genre, face.bloc) {
        (GenreDeFace::Plancher, _) => 0,
        (_, Some(_)) => 2,
        _ if face.de_l_interieur => 1,
        _ => 3,
    };
    let place = |face: &Projetee| face.bloc.map(|bloc| place_du_bloc[bloc]);
    projetees.sort_by(|a, b| {
        rang(a)
            .cmp(&rang(b))
            .then(place(a).cmp(&place(b)))
            .then(b.profondeur.total_cmp(&a.profondeur))
    });
    let mur = theme::melanger(p.carte, p.texte, 0.17);
    let plancher = Color32::from_rgb(214, 186, 148);
    let vitre = Color32::from_rgb(150, 205, 240);
    let porte = Color32::from_rgb(150, 104, 66);
    let trait_ = Stroke::new(1.2, p.texte.gamma_multiply(0.55));
    // Une arête, coupée elle aussi au plan proche.
    let tracer = |arete: &[[f64; 3]; 2]| {
        let (mut a, mut b) = (vers_l_oeil(arete[0]), vers_l_oeil(arete[1]));
        if a[2] < PLAN_PROCHE && b[2] < PLAN_PROCHE {
            return;
        }
        if a[2] < PLAN_PROCHE || b[2] < PLAN_PROCHE {
            let t = (PLAN_PROCHE - a[2]) / (b[2] - a[2]);
            let coupe = [
                a[0] + (b[0] - a[0]) * t,
                a[1] + (b[1] - a[1]) * t,
                PLAN_PROCHE,
            ];
            if a[2] < PLAN_PROCHE {
                a = coupe;
            } else {
                b = coupe;
            }
        }
        let ((xa, ya, _), (xb, yb, _)) = (projeter(a), projeter(b));
        painter.line_segment([vers_l_ecran((xa, ya)), vers_l_ecran((xb, yb))], trait_);
    };
    for (k, face) in projetees.iter().enumerate() {
        let couleur = match face.genre {
            GenreDeFace::Plancher => plancher,
            GenreDeFace::Mur => {
                let base = theme::melanger(mur, p.texte, 0.28 * (1.0 - face.lumiere));
                if face.de_l_interieur {
                    base
                } else {
                    base.gamma_multiply(0.28)
                }
            }
            GenreDeFace::Vitre => {
                vitre.gamma_multiply(if face.de_l_interieur { 0.75 } else { 0.35 })
            }
            GenreDeFace::Porte => {
                porte.gamma_multiply(if face.de_l_interieur { 1.0 } else { 0.35 })
            }
            GenreDeFace::Meuble(matiere) => {
                let base = theme::melanger(
                    couleur_d_une_matiere(matiere),
                    Color32::BLACK,
                    0.30 * (1.0 - face.lumiere),
                );
                // Le verre d'une douche laisse voir derrière lui.
                if matiere == Matiere::Verre {
                    base.gamma_multiply(0.45)
                } else {
                    base
                }
            }
            GenreDeFace::Cloison => theme::melanger(mur, p.texte, 0.28 * (1.0 - face.lumiere)),
            GenreDeFace::DessusDeMur => {
                let base = theme::melanger(mur, Color32::WHITE, 0.35);
                if face.de_l_interieur {
                    base
                } else {
                    base.gamma_multiply(0.28)
                }
            }
        };
        // Un maillage plein, sans adoucissement : pas de joint entre deux
        // morceaux d'un même mur.
        let mut maillage = egui::Mesh::default();
        for &point in &face.points {
            maillage.colored_vertex(vers_l_ecran(point), couleur);
        }
        for k in 1..face.points.len() as u32 - 1 {
            maillage.add_triangle(0, k, k + 1);
        }
        painter.add(egui::Shape::mesh(maillage));
        // Après la dernière face d'un bloc, ses arêtes : le bloc suivant, s'il
        // est devant, les cache.
        if let Some(bloc) = face.bloc
            && projetees
                .get(k + 1)
                .is_none_or(|suivante| suivante.bloc != Some(bloc))
            && let Some(bloc) = maquette.blocs.get(bloc)
        {
            for arete in &maquette.aretes[bloc.aretes.clone()] {
                tracer(arete);
            }
        }
    }
    // Les arêtes des murs et des ouvertures, par-dessus.
    for (arete, d_un_bloc) in maquette.aretes.iter().zip(&arete_d_un_bloc) {
        if !d_un_bloc {
            tracer(arete);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn une_face_se_coupe_au_plan_proche() {
        // Un carré qui passe derrière l'œil : il en reste ce qui est devant le
        // plan proche, toujours quatre coins.
        let carre = [
            [-10.0, 0.0, -20.0],
            [10.0, 0.0, -20.0],
            [10.0, 0.0, 20.0],
            [-10.0, 0.0, 20.0],
        ];
        let reste = couper_au_plan_proche(&carre);
        assert_eq!(reste.len(), 4);
        assert!(reste.iter().all(|s| s[2] >= PLAN_PROCHE - 1e-9));
        // Tout derrière : rien; tout devant : la face entière.
        assert!(
            couper_au_plan_proche(&[[0.0, 0.0, -5.0], [1.0, 0.0, -5.0], [0.0, 1.0, -5.0]])
                .is_empty()
        );
        let devant = [[0.0, 0.0, 50.0], [1.0, 0.0, 50.0], [0.0, 1.0, 60.0]];
        assert_eq!(couper_au_plan_proche(&devant), devant.to_vec());
    }

    #[test]
    fn la_visite_reste_dans_la_piece() {
        let mut orbite = Orbite {
            visite: Some(Visite {
                x: 0.0,
                z: 0.0,
                lacet: 0.0,
                bornes: [-10.0, 10.0, -10.0, 10.0],
            }),
            ..Orbite::default()
        };
        // Glisser vers le haut avance, sans sortir de la pièce.
        orbite.tourner(Vec2::new(0.0, -1000.0));
        let visite = orbite.visite.unwrap();
        assert!((visite.z - 10.0).abs() < 1e-9 && visite.x.abs() < 1e-9);
        // Glisser de côté tourne la tête.
        orbite.tourner(Vec2::new(100.0, 0.0));
        assert!((orbite.visite.unwrap().lacet + 0.6).abs() < 1e-9);
        // La visite commence au milieu de la maquette, à hauteur d'œil.
        let maquette = Maquette {
            faces: vec![noyau::maquette::Face {
                genre: GenreDeFace::Plancher,
                sommets: vec![[0.0, 0.0, 0.0], [144.0, 0.0, 0.0], [144.0, 0.0, 120.0]],
                normale: [0.0, 1.0, 0.0],
            }],
            aretes: Vec::new(),
            blocs: Vec::new(),
        };
        let mut orbite = Orbite::default();
        orbite.commencer_la_visite(&maquette);
        // Contre le mur de gauche, regardant dans le sens le plus long.
        let visite = orbite.visite.unwrap();
        assert_eq!((visite.x, visite.z), (18.0, 60.0));
        assert!((visite.lacet - std::f64::consts::FRAC_PI_2).abs() < 1e-12);
    }
}
