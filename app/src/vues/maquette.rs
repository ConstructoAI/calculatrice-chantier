//! La pièce en 3D (section 25) : la maquette du noyau, vue par une caméra qui
//! tourne autour d'elle, peinte face par face, de la plus éloignée à la plus
//! proche. Les murs vus de l'extérieur sont translucides : on voit dedans.

use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};
use noyau::maquette::{GenreDeFace, Maquette};
use noyau::mobilier::GenreDObjet;

use crate::theme::{self, Palette};

/// Où regarde la caméra : son lacet (autour de la verticale), son tangage
/// (au-dessus de l'horizon), en radians, et un zoom.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Orbite {
    pub lacet: f32,
    pub tangage: f32,
    pub zoom: f32,
}

impl Default for Orbite {
    fn default() -> Self {
        Orbite {
            lacet: 0.65,
            tangage: 0.62,
            zoom: 1.0,
        }
    }
}

impl Orbite {
    /// Tourne la caméra d'un glissement du doigt, en points de l'écran.
    pub fn tourner(&mut self, glissement: Vec2) {
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

/// La couleur d'un meuble dans la vue 3D.
fn couleur_d_un_meuble(genre: GenreDObjet) -> Color32 {
    match genre {
        GenreDObjet::Escalier => Color32::from_rgb(176, 132, 92),
        GenreDObjet::Lit => Color32::from_rgb(128, 148, 182),
        GenreDObjet::Table => Color32::from_rgb(160, 114, 76),
        GenreDObjet::Sofa => Color32::from_rgb(136, 136, 148),
        GenreDObjet::Comptoir => Color32::from_rgb(204, 204, 210),
        GenreDObjet::Toilette | GenreDObjet::Bain => Color32::from_rgb(246, 246, 250),
        GenreDObjet::Douche => Color32::from_rgb(176, 214, 236),
        GenreDObjet::Laveuse | GenreDObjet::Refrigerateur | GenreDObjet::Cuisiniere => {
            Color32::from_rgb(220, 224, 230)
        }
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

    // La caméra, sur une sphère autour de la pièce.
    let (lacet, tangage) = (f64::from(orbite.lacet), f64::from(orbite.tangage));
    let distance = rayon * 2.8;
    let oeil = [
        centre[0] + distance * tangage.cos() * lacet.sin(),
        centre[1] + distance * tangage.sin(),
        centre[2] + distance * tangage.cos() * lacet.cos(),
    ];
    let devant = unitaire(moins(centre, oeil));
    let droite = unitaire(vectoriel(devant, [0.0, 1.0, 0.0]));
    let dessus = vectoriel(droite, devant);
    let projeter = |sommet: V3| {
        let v = moins(sommet, oeil);
        let z = scalaire(v, devant);
        (z > 1e-6).then(|| (scalaire(v, droite) / z, -scalaire(v, dessus) / z, z))
    };
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
        let mut points = Vec::with_capacity(face.sommets.len());
        let mut profondeur = 0.0;
        let mut milieu = [0.0; 3];
        for &sommet in &face.sommets {
            let Some((x, y, z)) = projeter(sommet) else {
                break;
            };
            points.push((x, y));
            profondeur += z;
            for k in 0..3 {
                milieu[k] += sommet[k];
            }
        }
        if points.len() != face.sommets.len() || points.len() < 3 {
            continue;
        }
        let n = points.len() as f64;
        milieu = [milieu[0] / n, milieu[1] / n, milieu[2] / n];
        // Les faces d'un bloc tournées vers l'arrière ne se voient pas.
        let bloc = bloc_de_la_face[indice];
        if bloc.is_some() && scalaire(face.normale, moins(oeil, milieu)) <= 0.0 {
            continue;
        }
        projetees.push(Projetee {
            bloc,
            genre: face.genre,
            points,
            profondeur: profondeur / n,
            de_l_interieur: scalaire(face.normale, moins(oeil, milieu)) > 0.0,
            lumiere: if face.normale[1].abs() > 0.5 {
                1.0
            } else {
                (0.5 + 0.5 * scalaire(face.normale, lumiere).abs()) as f32
            },
        });
    }
    // La maquette tient dans la zone.
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
    let echelle = 0.86
        * (f64::from(zone.width()) / (x1 - x0).max(1e-9))
            .min(f64::from(zone.height()) / (y1 - y0).max(1e-9))
        * f64::from(orbite.zoom);
    let (mx, my) = ((x0 + x1) / 2.0, (y0 + y1) / 2.0);
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
    let ordre = noyau::maquette::ordre_des_blocs(&maquette.blocs, (oeil[0], oeil[2]));
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
    let tracer = |arete: &[[f64; 3]; 2]| {
        if let (Some((xa, ya, _)), Some((xb, yb, _))) = (projeter(arete[0]), projeter(arete[1])) {
            painter.line_segment([vers_l_ecran((xa, ya)), vers_l_ecran((xb, yb))], trait_);
        }
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
            GenreDeFace::Meuble(genre) => {
                let base = couleur_d_un_meuble(genre);
                theme::melanger(base, Color32::BLACK, 0.30 * (1.0 - face.lumiere))
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
