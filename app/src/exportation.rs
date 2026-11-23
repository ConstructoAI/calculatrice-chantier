//! Export du plan dessiné (section 25) : en PDF, pour l'imprimer ou le
//! transmettre, et en DXF, pour l'ouvrir dans un logiciel de dessin (AutoCAD,
//! LibreCAD, QCAD…). Les deux fichiers sont écrits ici, sans bibliothèque
//! (R-77), en texte ASCII : le PDF code ses accents en WinAnsi, le DXF en
//! `\U+XXXX`.

use noyau::dessin::{
    Dessin, EchelleDuPlan, Etat as EtatDeLaForme, Mesure, MiseEnPage, MorceauDeMur, Plan,
};
use noyau::metiers::Quantite;
use noyau::prefs::Preferences;
use noyau::unites::{Format, Systeme};

use crate::textes;
use crate::vues::dessin::{
    FORMAT_PAR_DEFAUT, longueur_en_clair, nom_de_la_piece, resultats_de_la_piece, saisie_en_clair,
};

/// Ce qu'on exporte d'une pièce.
struct PieceExportee {
    nom: String,
    /// Sommets en pouces, y vers le bas. Une forme fermée répète son départ.
    sommets: Vec<(f64, f64)>,
    /// Vrai si la forme est à l'échelle de ses mesures (pièce complète).
    a_l_echelle: bool,
    /// Pour chaque côté : sa mesure en clair, et vrai si elle est déduite des
    /// autres.
    cotes: Vec<(String, bool)>,
    /// Les résultats, comme à l'écran : (libellé, valeur).
    resultats: Vec<(String, String)>,
    /// Surface nette (pouces carrés) et périmètre (pouces), si la pièce est complète.
    aire_nette: Option<f64>,
    perimetre: Option<f64>,
    modele: Option<Format>,
    /// Les portes et les fenêtres : leur mur, leurs bords en part de la
    /// longueur du mur (de 0 à 1), et vrai pour une porte.
    ouvertures: Vec<(usize, f64, f64, bool)>,
    /// Les objets d'une pièce complète : leur nom, leur centre et leurs traits,
    /// sur le plan (pouces).
    objets: Vec<(String, (f64, f64), Vec<noyau::mobilier::TraitSurLePlan>)>,
    /// Les notes écrites d'une pièce complète : leur texte et leur place.
    notes: Vec<(String, (f64, f64))>,
    /// Les cloisons d'une pièce complète : leurs morceaux pleins (quatre
    /// coins, en pouces), et leurs portes.
    cloisons: Vec<noyau::cloison::MorceauDeCloison>,
    portes_de_cloisons: Vec<PorteExportee>,
    /// Les murs extérieurs d'une pièce complète, à leur épaisseur, et le
    /// contour de leur extérieur; vides sans épaisseur.
    murs_exterieurs: Vec<MorceauDeMur>,
    contour_exterieur: Vec<(f64, f64)>,
    /// L'épaisseur des murs extérieurs (pouces) : positive s'ils vont vers
    /// l'extérieur des mesures, négative vers l'intérieur, zéro sans murs.
    epaisseur_des_murs: f64,
    /// Les lignes de cote, si elles se montrent (section 25.1, point 35) :
    /// chaque chaîne et le texte de chacun de ses morceaux.
    chaines: Vec<(noyau::dessin::ChaineDeCotes, Vec<String>)>,
}

/// La normale extérieure d'un côté `ab` : perpendiculaire, unitaire, et
/// tournée à l'opposé du centre de la forme.
fn normale_exterieure(a: (f64, f64), b: (f64, f64), centre: (f64, f64)) -> (f64, f64) {
    let longueur = (b.0 - a.0).hypot(b.1 - a.1).max(1e-12);
    let (mut nx, mut ny) = (-(b.1 - a.1) / longueur, (b.0 - a.0) / longueur);
    let milieu = ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0);
    if (milieu.0 - centre.0) * nx + (milieu.1 - centre.1) * ny < 0.0 {
        (nx, ny) = (-nx, -ny);
    }
    (nx, ny)
}

/// Les espaces insécables des valeurs affichées deviennent des espaces.
fn sans_insecables(texte: &str) -> String {
    texte
        .chars()
        .map(|c| if c as u32 == 0xA0 { ' ' } else { c })
        .collect()
}

/// Les pièces dessinées du plan; une pièce vide n'est pas exportée.
fn preparer(plan: &Plan, prefs: &Preferences) -> Vec<PieceExportee> {
    plan.pieces
        .iter()
        .enumerate()
        .filter(|(_, piece)| !piece.dessin.est_vide())
        .map(|(rang, piece)| preparer_la_piece(&piece.dessin, &nom_de_la_piece(plan, rang), prefs))
        .collect()
}

fn preparer_la_piece(dessin: &Dessin, nom: &str, prefs: &Preferences) -> PieceExportee {
    let solution = dessin.resoudre(FORMAT_PAR_DEFAUT);
    let complete = matches!(solution.etat, EtatDeLaForme::Complete { .. });
    let sommets = if complete {
        dessin
            .sommets_mesures(&solution)
            .unwrap_or_else(|| dessin.sommets_du_croquis())
    } else {
        dessin.sommets_du_croquis()
    };
    let cotes = dessin
        .cotes
        .iter()
        .zip(&solution.mesures)
        .map(|(cote, mesure)| {
            let (texte, deduite) = match mesure {
                Mesure::Lue(_) => (
                    saisie_en_clair(&cote.saisie, prefs)
                        .unwrap_or_else(|| cote.saisie.trim().to_string()),
                    false,
                ),
                Mesure::Deduite(pouces) => (longueur_en_clair(*pouces, &solution, prefs), true),
                Mesure::Manquante => ("?".to_string(), false),
                Mesure::Invalide => (cote.saisie.trim().to_string(), false),
            };
            (sans_insecables(&texte), deduite)
        })
        .collect();
    let (aire_nette, perimetre) = match solution.etat {
        EtatDeLaForme::Complete { perimetre, .. } => (solution.aire_nette(), Some(perimetre)),
        _ => (None, None),
    };
    let ouvertures = solution
        .placements
        .iter()
        .flatten()
        .filter_map(|placement| {
            let longueur = solution.mesures.get(placement.cote)?.pouces()?;
            (longueur > 0.0).then(|| {
                (
                    placement.cote,
                    placement.debut / longueur,
                    placement.fin / longueur,
                    placement.porte,
                )
            })
        })
        .collect();
    let objets = if complete {
        dessin
            .objets
            .iter()
            .zip(&solution.objets)
            .enumerate()
            .map(|(rang, (objet, &dimensions))| {
                let contremarches = solution
                    .escaliers
                    .get(rang)
                    .copied()
                    .flatten()
                    .map(|calcul| calcul.contremarches);
                (
                    textes::objet(objet.genre).traduit().to_string(),
                    objet.centre,
                    objet.traits_sur_le_plan(dimensions, contremarches),
                )
            })
            .collect()
    } else {
        Vec::new()
    };
    let notes = if complete {
        dessin
            .notes
            .iter()
            .filter(|note| !note.texte.trim().is_empty())
            .map(|note| (note.texte.trim().to_string(), note.position))
            .collect()
    } else {
        Vec::new()
    };
    let (cloisons, portes_de_cloisons) = if complete {
        let mesurees: Vec<_> = solution.cloisons.iter().flatten().cloned().collect();
        let portes = mesurees
            .iter()
            .filter_map(|cloison| {
                let porte = cloison.porte?;
                Some(PorteExportee {
                    gonds: cloison.point(porte.segment, porte.debut),
                    autre_bord: cloison.point(porte.segment, porte.fin),
                    cote: cloison.cote_de_la_porte(),
                })
            })
            .collect();
        (
            mesurees
                .iter()
                .flat_map(|cloison| cloison.morceaux())
                .collect(),
            portes,
        )
    } else {
        (Vec::new(), Vec::new())
    };
    let murs_exterieurs = dessin.murs_exterieurs(&solution);
    let (contour_exterieur, epaisseur_des_murs) = if murs_exterieurs.is_empty() {
        (Vec::new(), 0.0)
    } else {
        (
            dessin.contour_exterieur(&solution).unwrap_or_default(),
            if dessin.mesures_exterieures {
                -solution.epaisseur_des_murs
            } else {
                solution.epaisseur_des_murs
            },
        )
    };
    PieceExportee {
        nom: sans_insecables(nom),
        sommets,
        a_l_echelle: complete,
        cotes,
        resultats: resultats_de_la_piece(dessin, &solution, prefs, true)
            .into_iter()
            .map(|(libelle, valeur)| (sans_insecables(&libelle), sans_insecables(&valeur)))
            .collect(),
        aire_nette,
        perimetre,
        modele: solution.modele,
        ouvertures,
        objets,
        notes,
        cloisons,
        portes_de_cloisons,
        murs_exterieurs,
        contour_exterieur,
        epaisseur_des_murs,
        chaines: if dessin.lignes_de_cote {
            dessin
                .chaines_de_cotes(&solution)
                .into_iter()
                .map(|chaine| {
                    let textes = chaine
                        .points
                        .windows(2)
                        .map(|paire| {
                            sans_insecables(&longueur_en_clair(
                                paire[1] - paire[0],
                                &solution,
                                prefs,
                            ))
                        })
                        .collect();
                    (chaine, textes)
                })
                .collect()
        } else {
            Vec::new()
        },
    }
}

/// Où écrire le texte d'une cote ou d'une mesure, à `ecart` du point `milieu`
/// dans la direction `(nx, ny)` (y vers le haut) : au-dessus ou au-dessous
/// d'un trait horizontal, à côté d'un trait vertical, en coin d'un trait de
/// biais. `hauteur` est la hauteur du texte.
fn place_du_texte(
    milieu: (f64, f64),
    (nx, ny): (f64, f64),
    ecart: f64,
    hauteur: f64,
) -> ((f64, f64), Alignement) {
    let alignement_de_cote = if nx > 0.0 {
        Alignement::Gauche
    } else {
        Alignement::Droite
    };
    if ny.abs() > 0.9 {
        let y = if ny > 0.0 {
            milieu.1 + ecart
        } else {
            milieu.1 - ecart - hauteur
        };
        ((milieu.0, y), Alignement::Centre)
    } else if nx.abs() > 0.9 {
        (
            (milieu.0 + nx.signum() * ecart, milieu.1 - hauteur / 3.0),
            alignement_de_cote,
        )
    } else {
        let y = if ny > 0.0 {
            milieu.1 + ecart
        } else {
            milieu.1 - ecart - hauteur
        };
        ((milieu.0 + nx.signum() * ecart, y), alignement_de_cote)
    }
}

/// Un morceau d'une chaîne de cotes posée : son milieu, sa longueur dans le
/// repère de sortie, et la direction où écrire son texte.
type MorceauDeCote = ((f64, f64), f64, (f64, f64));

/// Une chaîne de cotes posée sur une page ou dans un DXF (y vers le haut) :
/// sa ligne, ses traits à 45°, ses lignes d'attache et la place du texte de
/// chaque morceau (`poser_une_chaine`).
struct CoteAPoser {
    ligne: [(f64, f64); 2],
    traits: Vec<[(f64, f64); 2]>,
    attaches: Vec<[(f64, f64); 2]>,
    morceaux: Vec<MorceauDeCote>,
}

/// Pose une chaîne de cotes : `vers` passe du plan au repère de sortie;
/// `decalage` écarte la ligne de ce qu'elle mesure, `attache` est l'écart des
/// lignes d'attache au mur, `trait_` la longueur d'un trait de cote.
fn poser_une_chaine(
    chaine: &noyau::dessin::ChaineDeCotes,
    vers: &dyn Fn((f64, f64)) -> (f64, f64),
    decalage: f64,
    attache: f64,
    trait_: f64,
    centre: (f64, f64),
) -> Option<CoteAPoser> {
    let longueur = (chaine.fin.0 - chaine.debut.0).hypot(chaine.fin.1 - chaine.debut.1);
    let (a0, b0) = (vers(chaine.debut), vers(chaine.fin));
    let longueur_de_sortie = (b0.0 - a0.0).hypot(b0.1 - a0.1);
    if longueur <= 1e-9 || longueur_de_sortie <= 1e-9 {
        return None;
    }
    // Le repère de sortie a l'axe des y vers le haut.
    let dehors = (chaine.dehors.0, -chaine.dehors.1);
    let ecart = if dehors == (0.0, 0.0) { 0.0 } else { decalage };
    let le_long = (
        (b0.0 - a0.0) / longueur_de_sortie,
        (b0.1 - a0.1) / longueur_de_sortie,
    );
    let decale = |q: (f64, f64), d: f64| (q.0 + dehors.0 * d, q.1 + dehors.1 * d);
    let (a, b) = (decale(a0, ecart), decale(b0, ecart));
    let au = |t: f64| {
        let part = t / longueur;
        (a.0 + (b.0 - a.0) * part, a.1 + (b.1 - a.1) * part)
    };
    let racine = std::f64::consts::FRAC_1_SQRT_2 * trait_;
    let oblique = (
        (le_long.0 - le_long.1) * racine,
        (le_long.1 + le_long.0) * racine,
    );
    let mut traits = Vec::new();
    let mut attaches = Vec::new();
    for &t in &chaine.points {
        let q = au(t);
        traits.push([
            (q.0 - oblique.0, q.1 - oblique.1),
            (q.0 + oblique.0, q.1 + oblique.1),
        ]);
        if ecart > 0.0 {
            let part = t / longueur;
            let sur_le_mur = (a0.0 + (b0.0 - a0.0) * part, a0.1 + (b0.1 - a0.1) * part);
            attaches.push([decale(sur_le_mur, attache), decale(q, trait_ * 0.6)]);
        }
    }
    // La direction du texte : vers l'extérieur, ou, pour l'écart entre deux
    // cloisons, à l'opposé du centre de la pièce.
    let texte_vers = if ecart > 0.0 {
        dehors
    } else {
        normale_exterieure(a, b, centre)
    };
    let morceaux = chaine
        .points
        .windows(2)
        .map(|paire| {
            let (q0, q1) = (au(paire[0]), au(paire[1]));
            (
                ((q0.0 + q1.0) / 2.0, (q0.1 + q1.1) / 2.0),
                (q1.0 - q0.0).hypot(q1.1 - q0.1),
                texte_vers,
            )
        })
        .collect();
    Some(CoteAPoser {
        ligne: [a, b],
        traits,
        attaches,
        morceaux,
    })
}

// ---------------------------------------------------------------------- PDF

/// Format Lettre, en points.
const LARGEUR_PAGE: f64 = 612.0;
const HAUTEUR_PAGE: f64 = 792.0;
const MARGE: f64 = 54.0;

type Couleur = (f64, f64, f64);
const NOIR: Couleur = (0.10, 0.12, 0.16);
const GRIS: Couleur = (0.45, 0.48, 0.52);
/// Le plein des cloisons.
const BLEU_PALE: Couleur = (0.82, 0.87, 0.95);
/// Le plein des murs extérieurs.
const MUR_PLEIN: Couleur = (0.74, 0.80, 0.90);
const BLEU: Couleur = (0.0, 0.40, 0.80);
const ORANGE: Couleur = (0.85, 0.38, 0.05);
const FILET: Couleur = (0.80, 0.83, 0.87);
const BLANC: Couleur = (1.0, 1.0, 1.0);

/// Une ouverture sur un contour : ses deux bords, le sens du mur, celui de
/// l'intérieur de la pièce et sa largeur.
struct Geometrie {
    q0: (f64, f64),
    q1: (f64, f64),
    u: (f64, f64),
    dedans: (f64, f64),
    largeur: f64,
}

/// La géométrie d'une ouverture; `aire_signee` donne le sens du contour.
fn geometrie_d_une_ouverture(
    points: &[(f64, f64)],
    aire_signee: f64,
    (cote, t0, t1): (usize, f64, f64),
) -> Option<Geometrie> {
    let (a, b) = (points.get(cote)?, points.get(cote + 1)?);
    let le_long = (b.0 - a.0, b.1 - a.1);
    let q0 = (a.0 + le_long.0 * t0, a.1 + le_long.1 * t0);
    let q1 = (a.0 + le_long.0 * t1, a.1 + le_long.1 * t1);
    let largeur = (q1.0 - q0.0).hypot(q1.1 - q0.1);
    if largeur <= 1e-9 {
        return None;
    }
    let u = ((q1.0 - q0.0) / largeur, (q1.1 - q0.1) / largeur);
    let dedans = if aire_signee > 0.0 {
        (-u.1, u.0)
    } else {
        (u.1, -u.0)
    };
    Some(Geometrie {
        q0,
        q1,
        u,
        dedans,
        largeur,
    })
}

fn aire_signee(points: &[(f64, f64)]) -> f64 {
    points
        .windows(2)
        .map(|s| s[0].0 * s[1].1 - s[1].0 * s[0].1)
        .sum()
}

/// L'arc d'ouverture d'une porte : un quart de cercle autour de son montant.
fn arc_de_porte(q0: (f64, f64), u: (f64, f64), dedans: (f64, f64), rayon: f64) -> Vec<(f64, f64)> {
    (0..=16)
        .map(|i| {
            let angle = std::f64::consts::FRAC_PI_2 * f64::from(i) / 16.0;
            (
                q0.0 + (u.0 * angle.cos() + dedans.0 * angle.sin()) * rayon,
                q0.1 + (u.1 * angle.cos() + dedans.1 * angle.sin()) * rayon,
            )
        })
        .collect()
}

/// Les portes et les fenêtres sur la page : le mur s'interrompt, puis le
/// battant et son arc, ou le trait double d'une fenêtre.
fn ouvertures_sur_la_page(
    page: &mut Page,
    points: &[(f64, f64)],
    ouvertures: &[(usize, f64, f64, bool)],
    epaisseur: f64,
) {
    let sens = aire_signee(points);
    for &(cote, t0, t1, porte) in ouvertures {
        let Some(Geometrie {
            q0,
            q1,
            u,
            dedans,
            largeur,
        }) = geometrie_d_une_ouverture(points, sens, (cote, t0, t1))
        else {
            continue;
        };
        // Un mur sans épaisseur s'interrompt en blanc; un mur à son
        // épaisseur l'est déjà.
        if epaisseur.abs() <= 1e-9 {
            page.trait_(&[q0, q1], 4.2, BLANC);
        }
        if porte {
            // Un mur vers l'intérieur : la porte s'ouvre de sa face intérieure.
            let q0 = if epaisseur < 0.0 {
                (q0.0 - dedans.0 * epaisseur, q0.1 - dedans.1 * epaisseur)
            } else {
                q0
            };
            let battant = (q0.0 + dedans.0 * largeur, q0.1 + dedans.1 * largeur);
            page.trait_(&[q0, battant], 1.2, NOIR);
            page.trait_(&arc_de_porte(q0, u, dedans, largeur), 0.6, GRIS);
        } else if epaisseur.abs() > 1e-9 {
            // La fenêtre traverse le mur : ses deux faces et son vitrage.
            for (part, largeur_du_trait) in [(0.0, 0.5), (0.5, 0.8), (1.0, 0.5)] {
                let k = -epaisseur * part;
                page.trait_(
                    &[
                        (q0.0 + dedans.0 * k, q0.1 + dedans.1 * k),
                        (q1.0 + dedans.0 * k, q1.1 + dedans.1 * k),
                    ],
                    largeur_du_trait,
                    BLEU,
                );
            }
        } else {
            for decalage in [-1.6, 1.6] {
                page.trait_(
                    &[
                        (q0.0 + dedans.0 * decalage, q0.1 + dedans.1 * decalage),
                        (q1.0 + dedans.0 * decalage, q1.1 + dedans.1 * decalage),
                    ],
                    0.6,
                    BLEU,
                );
            }
            page.trait_(&[q0, q1], 0.4, BLEU);
        }
    }
}

/// Alignement d'un texte par rapport à son point d'ancrage.
#[derive(Clone, Copy)]
enum Alignement {
    Gauche,
    Centre,
    Droite,
}

/// Largeurs des caractères ASCII 32 à 126 dans la police Helvetica, en
/// millièmes de la taille (métriques standard des polices PDF).
const LARGEURS_HELVETICA: [u16; 95] = [
    278, 278, 355, 556, 556, 889, 667, 191, 333, 333, 389, 584, 278, 333, 278, 278, //
    556, 556, 556, 556, 556, 556, 556, 556, 556, 556, 278, 278, 584, 584, 584, 556, //
    1015, 667, 667, 722, 722, 667, 611, 778, 722, 278, 500, 667, 556, 833, 722, 778, //
    667, 778, 722, 667, 611, 722, 667, 944, 667, 667, 611, 278, 278, 278, 469, 556, //
    333, 556, 556, 500, 556, 556, 278, 556, 556, 222, 222, 500, 222, 833, 556, 556, //
    556, 556, 333, 500, 278, 556, 500, 722, 500, 500, 500, 334, 260, 334, 584,
];

/// L'octet WinAnsi d'un caractère; « ? » s'il n'y en a pas.
fn octet_winansi(c: char) -> u8 {
    match c as u32 {
        0x20..=0x7E => c as u8,
        // L'espace insécable s'écrit comme une espace.
        0xA0 => b' ',
        0xA1..=0xFF => c as u32 as u8,
        // Le signe moins s'écrit avec le trait d'union.
        0x2212 => b'-',
        0x2013 => 0x96,
        0x2014 => 0x97,
        0x2018 => 0x91,
        0x2019 => 0x92,
        0x201C => 0x93,
        0x201D => 0x94,
        0x2022 => 0x95,
        0x2026 => 0x85,
        0x20AC => 0x80,
        0x0152 => 0x8C,
        0x0153 => 0x9C,
        0x00..=0x1F => b' ',
        _ => b'?',
    }
}

/// Largeur d'un octet WinAnsi en Helvetica (approchée pour les lettres accentuées).
fn largeur_octet(octet: u8) -> u32 {
    match octet {
        0x20..=0x7E => u32::from(LARGEURS_HELVETICA[usize::from(octet - 0x20)]),
        0xB2 | 0xB3 | 0xB9 => 333,
        0xB0 => 400,
        0xB7 | 0x91 | 0x92 => 278,
        0xD7 => 584,
        0xAB | 0xBB | 0x96 => 556,
        0x97 => 1000,
        0xC0..=0xDE => 680,
        0xDF..=0xFF => 556,
        _ => 556,
    }
}

/// Largeur d'un texte, en millièmes de la taille de la police.
fn largeur_du_texte(texte: &str, gras: bool) -> f64 {
    let somme: u32 = texte.chars().map(|c| largeur_octet(octet_winansi(c))).sum();
    f64::from(somme) * if gras { 1.06 } else { 1.0 }
}

/// Un texte écrit comme une chaîne PDF : WinAnsi, parenthèses et barres
/// obliques inverses protégées, octets hors ASCII en octal.
fn chaine_pdf(texte: &str) -> String {
    let mut chaine = String::with_capacity(texte.len() + 8);
    for c in texte.chars() {
        let octet = octet_winansi(c);
        match octet {
            b'(' | b')' | b'\\' => {
                chaine.push('\\');
                chaine.push(char::from(octet));
            }
            0x20..=0x7E => chaine.push(char::from(octet)),
            autre => chaine.push_str(&format!("\\{autre:03o}")),
        }
    }
    chaine
}

/// Une page PDF en construction : ses opérateurs de dessin.
#[derive(Default)]
struct Page {
    operateurs: String,
    /// Sa largeur et sa hauteur (points); `None` : Lettre en portrait.
    taille: Option<(f64, f64)>,
}

impl Page {
    /// Sa largeur et sa hauteur (points).
    fn dimensions(&self) -> (f64, f64) {
        self.taille.unwrap_or((LARGEUR_PAGE, HAUTEUR_PAGE))
    }
}

impl Page {
    /// Une ligne brisée, de l'épaisseur et de la couleur données.
    fn trait_(&mut self, points: &[(f64, f64)], epaisseur: f64, couleur: Couleur) {
        let Some((premier, suite)) = points.split_first() else {
            return;
        };
        self.operateurs.push_str(&format!(
            "{:.3} {:.3} {:.3} RG {epaisseur:.2} w 1 j 1 J {:.2} {:.2} m",
            couleur.0, couleur.1, couleur.2, premier.0, premier.1
        ));
        for point in suite {
            self.operateurs
                .push_str(&format!(" {:.2} {:.2} l", point.0, point.1));
        }
        self.operateurs.push_str(" S\n");
    }

    /// Un polygone plein, de la couleur donnée.
    fn polygone_plein(&mut self, points: &[(f64, f64)], couleur: Couleur) {
        let Some((premier, suite)) = points.split_first() else {
            return;
        };
        self.operateurs.push_str(&format!(
            "{:.3} {:.3} {:.3} rg {:.2} {:.2} m",
            couleur.0, couleur.1, couleur.2, premier.0, premier.1
        ));
        for point in suite {
            self.operateurs
                .push_str(&format!(" {:.2} {:.2} l", point.0, point.1));
        }
        self.operateurs.push_str(" h f\n");
    }

    /// Un texte, ancré à gauche, au centre ou à droite de sa position.
    fn texte(
        &mut self,
        (x, y): (f64, f64),
        taille: f64,
        gras: bool,
        alignement: Alignement,
        couleur: Couleur,
        texte: &str,
    ) {
        let largeur = largeur_du_texte(texte, gras) * taille / 1000.0;
        let x = match alignement {
            Alignement::Gauche => x,
            Alignement::Centre => x - largeur / 2.0,
            Alignement::Droite => x - largeur,
        };
        let police = if gras { "F2" } else { "F1" };
        self.operateurs.push_str(&format!(
            "BT /{police} {taille:.1} Tf {:.3} {:.3} {:.3} rg {x:.2} {y:.2} Td ({}) Tj ET\n",
            couleur.0,
            couleur.1,
            couleur.2,
            chaine_pdf(texte)
        ));
    }
}

/// La porte d'une cloison, sur le plan (pouces) : le bord de ses gonds, son
/// autre bord, et le côté où elle s'ouvre.
#[derive(Clone, Copy, Debug)]
struct PorteExportee {
    gonds: (f64, f64),
    autre_bord: (f64, f64),
    cote: (f64, f64),
}

/// La géométrie d'une porte de cloison une fois sur la page ou dans le DXF :
/// `q0` et `q1` sont ses bords, `bout` l'image du point à un pouce de ses
/// gonds, du côté où elle s'ouvre.
fn geometrie_d_une_porte_de_cloison(
    q0: (f64, f64),
    q1: (f64, f64),
    bout: (f64, f64),
) -> Option<Geometrie> {
    let largeur = (q1.0 - q0.0).hypot(q1.1 - q0.1);
    let (dx, dy) = (bout.0 - q0.0, bout.1 - q0.1);
    let norme = dx.hypot(dy);
    if largeur <= 1e-9 || norme <= 1e-12 {
        return None;
    }
    Some(Geometrie {
        q0,
        q1,
        u: ((q1.0 - q0.0) / largeur, (q1.1 - q0.1) / largeur),
        dedans: (dx / norme, dy / norme),
        largeur,
    })
}

/// Les angles (degrés, dans le sens trigonométrique) de l'arc d'une porte,
/// du mur à son battant ouvert.
fn angles_d_un_arc_de_porte(u: (f64, f64), dedans: (f64, f64)) -> (f64, f64) {
    let vers_le_mur = u.1.atan2(u.0).to_degrees();
    let vers_le_battant = dedans.1.atan2(dedans.0).to_degrees();
    let angles = if ((vers_le_battant - vers_le_mur).rem_euclid(360.0) - 90.0).abs() < 1.0 {
        (vers_le_mur, vers_le_battant)
    } else {
        (vers_le_battant, vers_le_mur)
    };
    (angles.0.rem_euclid(360.0), angles.1.rem_euclid(360.0))
}

/// Les points extrêmes d'un trait d'objet, pour le cadrage.
fn etendue_d_un_trait(trait_: &noyau::mobilier::TraitSurLePlan) -> Vec<(f64, f64)> {
    match *trait_ {
        noyau::mobilier::TraitSurLePlan::Ligne(a, b) => vec![a, b],
        noyau::mobilier::TraitSurLePlan::Cercle((x, y), r) => vec![(x - r, y - r), (x + r, y + r)],
    }
}

/// Place des sommets (pouces, y vers le bas) dans un cadre de la page (points,
/// y vers le haut), à la plus grande échelle qui les y fait tenir.
fn ajuster(
    sommets: &[(f64, f64)],
    (x0, y0, x1, y1): (f64, f64, f64, f64),
) -> impl Fn((f64, f64)) -> (f64, f64) {
    let (mut gauche, mut droite, mut haut, mut bas) = (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
    for &(x, y) in sommets {
        gauche = gauche.min(x);
        droite = droite.max(x);
        haut = haut.min(y);
        bas = bas.max(y);
    }
    if sommets.is_empty() {
        (gauche, droite, haut, bas) = (0.0, 1.0, 0.0, 1.0);
    }
    let facteur = ((x1 - x0) / (droite - gauche).max(1e-9)).min((y1 - y0) / (bas - haut).max(1e-9));
    let (cx, cy) = ((x0 + x1) / 2.0, (y0 + y1) / 2.0);
    let (mx, my) = ((gauche + droite) / 2.0, (haut + bas) / 2.0);
    move |(x, y)| (cx + (x - mx) * facteur, cy - (y - my) * facteur)
}

fn en_tete_de_page(page: &mut Page, titre: &str, entete: &str) {
    let (largeur, hauteur) = page.dimensions();
    page.texte(
        (MARGE, hauteur - 50.0),
        18.0,
        true,
        Alignement::Gauche,
        NOIR,
        titre,
    );
    page.texte(
        (MARGE, hauteur - 68.0),
        9.5,
        false,
        Alignement::Gauche,
        GRIS,
        entete,
    );
    page.trait_(
        &[(MARGE, hauteur - 78.0), (largeur - MARGE, hauteur - 78.0)],
        0.8,
        FILET,
    );
}

fn pied_de_page(page: &mut Page, numero: usize, total: usize) {
    let (largeur, _) = page.dimensions();
    page.trait_(&[(MARGE, 50.0), (largeur - MARGE, 50.0)], 0.6, FILET);
    page.texte(
        (MARGE, 36.0),
        8.0,
        false,
        Alignement::Gauche,
        GRIS,
        textes::DESSIN_EXPORT_PIED.traduit(),
    );
    page.texte(
        (largeur - MARGE, 36.0),
        8.0,
        false,
        Alignement::Droite,
        GRIS,
        &textes::dessin_export_page(numero, total),
    );
}

/// La page d'une pièce : sa forme et ses mesures, ses résultats, ses côtés.
fn page_de_piece(
    piece: &PieceExportee,
    entete: &str,
    numero: usize,
    total: usize,
    avis: Option<&str>,
) -> Page {
    let mut page = Page::default();
    en_tete_de_page(&mut page, &piece.nom, entete);

    // La forme, ajustée à son cadre; la place des mesures autour.
    let cadre = (MARGE + 84.0, 372.0, LARGEUR_PAGE - MARGE - 84.0, 684.0);
    let points_a_cadrer = a_cadrer(piece);
    let vers_la_page = ajuster(&points_a_cadrer, cadre);
    dessiner_le_plan(&mut page, piece, &vers_la_page);
    if !piece.a_l_echelle {
        page.texte(
            (MARGE, 350.0),
            9.5,
            false,
            Alignement::Gauche,
            ORANGE,
            textes::DESSIN_EXPORT_CROQUIS.traduit(),
        );
    } else if let Some(avis) = avis {
        page.texte((MARGE, 350.0), 9.5, false, Alignement::Gauche, ORANGE, avis);
    }

    // Les résultats.
    let mut y = 318.0;
    let largeur_des_libelles = piece
        .resultats
        .iter()
        .map(|(libelle, _)| largeur_du_texte(libelle, true) * 11.0 / 1000.0)
        .fold(0.0, f64::max);
    for (libelle, valeur) in &piece.resultats {
        page.texte((MARGE, y), 11.0, true, Alignement::Gauche, NOIR, libelle);
        page.texte(
            (MARGE + largeur_des_libelles + 12.0, y),
            11.0,
            false,
            Alignement::Gauche,
            NOIR,
            valeur,
        );
        y -= 17.0;
    }

    // Les côtés, sur deux colonnes s'il y en a beaucoup.
    y -= 8.0;
    page.texte(
        (MARGE, y),
        11.0,
        true,
        Alignement::Gauche,
        NOIR,
        textes::DESSIN_EXPORT_COTES.traduit(),
    );
    y -= 15.0;
    let par_colonne = if piece.cotes.len() > 8 {
        piece.cotes.len().div_ceil(2)
    } else {
        piece.cotes.len().max(1)
    };
    for (i, (texte, deduite)) in piece.cotes.iter().enumerate() {
        let x = MARGE + (i / par_colonne) as f64 * 250.0;
        let y_ligne = y - (i % par_colonne) as f64 * 13.5;
        if y_ligne < 60.0 {
            continue;
        }
        let ligne = if *deduite {
            format!(
                "{}. {}   ({})",
                i + 1,
                texte,
                textes::DESSIN_EXPORT_DEDUITE.traduit()
            )
        } else {
            format!("{}. {}", i + 1, texte)
        };
        let couleur = if *deduite { GRIS } else { NOIR };
        page.texte(
            (x, y_ligne),
            10.0,
            false,
            Alignement::Gauche,
            couleur,
            &ligne,
        );
    }
    pied_de_page(&mut page, numero, total);
    page
}

/// Ce qu'il faut faire tenir d'une pièce sur une page (pouces) : sa forme,
/// ses murs, ses objets, ses notes et ses cloisons.
fn a_cadrer(piece: &PieceExportee) -> Vec<(f64, f64)> {
    let mut points = piece.sommets.clone();
    for (_, _, traits) in &piece.objets {
        points.extend(traits.iter().flat_map(etendue_d_un_trait));
    }
    points.extend(piece.notes.iter().map(|(_, position)| *position));
    points.extend(piece.cloisons.iter().flat_map(|morceau| morceau.coins));
    points.extend(piece.contour_exterieur.iter().copied());
    points
}

/// L'étendue de ces points : (x0, y0, x1, y1).
fn etendue(points: &[(f64, f64)]) -> (f64, f64, f64, f64) {
    points.iter().fold(
        (f64::MAX, f64::MAX, f64::MIN, f64::MIN),
        |(x0, y0, x1, y1), p| (x0.min(p.0), y0.min(p.1), x1.max(p.0), y1.max(p.1)),
    )
}

/// Le plan d'une pièce sur une page : ses murs, ses ouvertures, ses objets,
/// ses cloisons et leurs portes, ses notes, les mesures de ses côtés et ses
/// lignes de cote; `vers_la_page` passe du plan (pouces) à la page.
fn dessiner_le_plan(
    page: &mut Page,
    piece: &PieceExportee,
    vers_la_page: &dyn Fn((f64, f64)) -> (f64, f64),
) {
    let points: Vec<(f64, f64)> = piece.sommets.iter().map(|s| vers_la_page(*s)).collect();
    // L'échelle de la page, en points par pouce.
    let origine = vers_la_page((0.0, 0.0));
    let un_pouce = vers_la_page((1.0, 0.0));
    let echelle = (un_pouce.0 - origine.0).hypot(un_pouce.1 - origine.1);
    let epaisseur_des_murs = piece.epaisseur_des_murs * echelle;
    // Les murs à leur épaisseur, pleins : l'intérieur, l'extérieur et les
    // jambages; sans épaisseur, le contour.
    if piece.murs_exterieurs.is_empty() {
        page.trait_(&points, 1.8, BLEU);
    } else {
        for morceau in &piece.murs_exterieurs {
            let [a, b, c, d] = morceau.coins.map(&vers_la_page);
            page.polygone_plein(&[a, b, c, d], MUR_PLEIN);
            page.trait_(&[a, b], 1.4, BLEU);
            page.trait_(&[d, c], 1.0, BLEU);
            if morceau.jambage_au_debut {
                page.trait_(&[a, d], 1.0, BLEU);
            }
            if morceau.jambage_a_la_fin {
                page.trait_(&[b, c], 1.0, BLEU);
            }
        }
    }
    ouvertures_sur_la_page(page, &points, &piece.ouvertures, epaisseur_des_murs);
    // Les objets, à la même échelle que la forme.
    for (nom, centre, traits) in &piece.objets {
        for trait_ in traits {
            match *trait_ {
                noyau::mobilier::TraitSurLePlan::Ligne(a, b) => {
                    page.trait_(&[vers_la_page(a), vers_la_page(b)], 0.7, GRIS);
                }
                noyau::mobilier::TraitSurLePlan::Cercle(c, rayon) => {
                    let c = vers_la_page(c);
                    let cercle: Vec<(f64, f64)> = (0..=24)
                        .map(|k| {
                            let angle = std::f64::consts::TAU * f64::from(k) / 24.0;
                            (
                                c.0 + rayon * echelle * angle.cos(),
                                c.1 + rayon * echelle * angle.sin(),
                            )
                        })
                        .collect();
                    page.trait_(&cercle, 0.7, GRIS);
                }
            }
        }
        page.texte(
            vers_la_page(*centre),
            7.0,
            false,
            Alignement::Centre,
            GRIS,
            nom,
        );
    }
    // Les cloisons, pleines.
    // Leurs faces et leurs bouts se tracent; leurs coins d'onglet, non.
    for morceau in &piece.cloisons {
        let [a, b, c, d] = morceau.coins.map(&vers_la_page);
        page.polygone_plein(&[a, b, c, d], BLEU_PALE);
        page.trait_(&[a, b], 0.9, BLEU);
        page.trait_(&[d, c], 0.9, BLEU);
        if morceau.bout_au_debut {
            page.trait_(&[a, d], 0.9, BLEU);
        }
        if morceau.bout_a_la_fin {
            page.trait_(&[b, c], 0.9, BLEU);
        }
    }
    // Les portes des cloisons : le battant et son arc.
    for porte in &piece.portes_de_cloisons {
        let (q0, q1) = (vers_la_page(porte.gonds), vers_la_page(porte.autre_bord));
        let bout = vers_la_page((porte.gonds.0 + porte.cote.0, porte.gonds.1 + porte.cote.1));
        let Some(Geometrie {
            q0,
            u,
            dedans,
            largeur,
            ..
        }) = geometrie_d_une_porte_de_cloison(q0, q1, bout)
        else {
            continue;
        };
        let battant = (q0.0 + dedans.0 * largeur, q0.1 + dedans.1 * largeur);
        page.trait_(&[q0, battant], 1.2, NOIR);
        page.trait_(&arc_de_porte(q0, u, dedans, largeur), 0.6, GRIS);
    }
    // Les notes, par-dessus.
    for (texte, position) in &piece.notes {
        page.texte(
            vers_la_page(*position),
            8.0,
            false,
            Alignement::Centre,
            NOIR,
            texte,
        );
    }
    let nombre = points.len().saturating_sub(1).max(1);
    let centre = (
        points.iter().take(nombre).map(|q| q.0).sum::<f64>() / nombre as f64,
        points.iter().take(nombre).map(|q| q.1).sum::<f64>() / nombre as f64,
    );
    for (i, segment) in points.windows(2).enumerate() {
        let Some((texte, deduite)) = piece.cotes.get(i) else {
            continue;
        };
        if texte.is_empty() {
            continue;
        }
        let milieu = (
            (segment[0].0 + segment[1].0) / 2.0,
            (segment[0].1 + segment[1].1) / 2.0,
        );
        // La page a l'axe des y vers le haut.
        let (nx, ny) = normale_exterieure(segment[0], segment[1], centre);
        // La mesure s'écrit hors du mur, à son épaisseur.
        let milieu = (
            milieu.0 + nx * epaisseur_des_murs.max(0.0),
            milieu.1 + ny * epaisseur_des_murs.max(0.0),
        );
        let (position, alignement) = if ny.abs() > 0.9 {
            if ny > 0.0 {
                ((milieu.0, milieu.1 + 5.0), Alignement::Centre)
            } else {
                ((milieu.0, milieu.1 - 12.0), Alignement::Centre)
            }
        } else if nx.abs() > 0.9 {
            if nx > 0.0 {
                ((milieu.0 + 6.0, milieu.1 - 3.0), Alignement::Gauche)
            } else {
                ((milieu.0 - 6.0, milieu.1 - 3.0), Alignement::Droite)
            }
        } else {
            let y = if ny > 0.0 {
                milieu.1 + 4.0
            } else {
                milieu.1 - 11.0
            };
            if nx > 0.0 {
                ((milieu.0 + 5.0, y), Alignement::Gauche)
            } else {
                ((milieu.0 - 5.0, y), Alignement::Droite)
            }
        };
        let couleur = if *deduite { GRIS } else { NOIR };
        page.texte(position, 9.0, false, alignement, couleur, texte);
    }
    // Les lignes de cote (point 35), au-delà des mesures.
    let mur = epaisseur_des_murs.max(0.0);
    for (chaine, textes_des_cotes) in &piece.chaines {
        // Au-delà de la mesure du mur, qui s'écrit à côté d'un mur vertical.
        let largeur_de_la_mesure = chaine
            .mur
            .and_then(|k| piece.cotes.get(k))
            .map_or(0.0, |(texte, _)| {
                largeur_du_texte(texte, false) * 9.0 / 1000.0
            });
        let horizontal = chaine.dehors.1.abs() > 0.9;
        let decalage = if horizontal {
            mur + 24.0
        } else {
            (mur + 24.0).max(mur + 6.0 + largeur_de_la_mesure + 8.0)
        };
        let Some(cote) = poser_une_chaine(chaine, &vers_la_page, decalage, mur + 2.0, 3.0, centre)
        else {
            continue;
        };
        page.trait_(&cote.ligne, 0.5, GRIS);
        for attache in &cote.attaches {
            page.trait_(attache, 0.3, GRIS);
        }
        for trait_ in &cote.traits {
            page.trait_(trait_, 0.8, NOIR);
        }
        for (&(milieu, longueur, vers), texte) in cote.morceaux.iter().zip(textes_des_cotes) {
            // Un morceau trop court garde ses traits, sans texte.
            let largeur = largeur_du_texte(texte, false) * 7.0 / 1000.0;
            let horizontal = vers.1.abs() > 0.9;
            if (horizontal && largeur + 3.0 > longueur) || (!horizontal && 9.0 > longueur) {
                continue;
            }
            let (position, alignement) = place_du_texte(milieu, vers, 2.5, 7.0);
            page.texte(position, 7.0, false, alignement, NOIR, texte);
        }
    }
}

/// Une feuille de papier : son papier, son sens, sa largeur et sa hauteur
/// (points).
#[derive(Clone, Copy)]
struct Feuille {
    papier: noyau::langue::Texte,
    paysage: bool,
    largeur: f64,
    hauteur: f64,
}

/// Les feuilles, de la plus petite à la plus grande : Lettre, puis 11 × 17.
const FEUILLES: [Feuille; 4] = [
    Feuille {
        papier: textes::PAPIER_LETTRE,
        paysage: false,
        largeur: 612.0,
        hauteur: 792.0,
    },
    Feuille {
        papier: textes::PAPIER_LETTRE,
        paysage: true,
        largeur: 792.0,
        hauteur: 612.0,
    },
    Feuille {
        papier: textes::PAPIER_11_17,
        paysage: false,
        largeur: 792.0,
        hauteur: 1224.0,
    },
    Feuille {
        papier: textes::PAPIER_11_17,
        paysage: true,
        largeur: 1224.0,
        hauteur: 792.0,
    },
];
/// La colonne de droite d'une page à l'échelle : les résultats, l'échelle
/// graphique et le cartouche (points).
const COLONNE: f64 = 216.0;
/// La place laissée autour d'un plan à l'échelle, pour ses mesures et ses
/// cotes (points).
const MARGE_DU_PLAN: f64 = 40.0;
/// La hauteur du cartouche (points).
const HAUTEUR_DU_CARTOUCHE: f64 = 104.0;

/// La zone du plan sur une feuille (x0, y0, x1, y1), y vers le haut : à gauche
/// de la colonne, entre l'en-tête et le pied.
fn zone_du_plan(feuille: &Feuille) -> (f64, f64, f64, f64) {
    (
        MARGE,
        66.0,
        feuille.largeur - MARGE - COLONNE - 18.0,
        feuille.hauteur - 92.0,
    )
}

/// La plus petite feuille où tient un plan de cette étendue (pouces) à cette
/// échelle (points par pouce) : Lettre avant 11 × 17, et pour chacune le sens
/// du plan d'abord.
fn feuille_pour(etendue: (f64, f64), points_par_pouce: f64) -> Option<Feuille> {
    let (l, h) = (
        etendue.0 * points_par_pouce + 2.0 * MARGE_DU_PLAN,
        etendue.1 * points_par_pouce + 2.0 * MARGE_DU_PLAN,
    );
    let tient = |feuille: &Feuille| {
        let (x0, y0, x1, y1) = zone_du_plan(feuille);
        l <= x1 - x0 && h <= y1 - y0
    };
    FEUILLES.chunks(2).find_map(|paire| {
        let (premiere, seconde) = if l > h {
            (paire[1], paire[0])
        } else {
            (paire[0], paire[1])
        };
        [premiere, seconde].into_iter().find(tient)
    })
}

/// Pour l'écran : sur quelle feuille tiendrait la pièce à cette échelle;
/// `None` pour une échelle ajustée ou une pièce incomplète.
pub fn avis_de_l_echelle(
    dessin: &Dessin,
    echelle: EchelleDuPlan,
    prefs: &Preferences,
) -> Option<String> {
    let points_par_pouce = echelle.points_par_pouce()?;
    let piece = preparer_la_piece(dessin, "", prefs);
    if !piece.a_l_echelle {
        return None;
    }
    let (x0, y0, x1, y1) = etendue(&a_cadrer(&piece));
    Some(match feuille_pour((x1 - x0, y1 - y0), points_par_pouce) {
        Some(feuille) => textes::dessin_tient_sur(feuille.papier.traduit(), feuille.paysage),
        None => textes::DESSIN_NE_TIENT_PAS.traduit().to_string(),
    })
}

/// Ce que la page d'une pièce à l'échelle reçoit du plan : sa mise en page,
/// la date de l'export et le séparateur décimal.
struct MiseEnPageDuPdf {
    mise_en_page: MiseEnPage,
    date: String,
    decimal: char,
}

/// La page d'une pièce complète à l'échelle choisie (section 25.1, point 36) :
/// sur la plus petite feuille où elle tient, le plan à gauche, et à droite
/// ses résultats, l'échelle graphique et le cartouche. `None` si elle ne
/// tient sur aucune feuille.
fn page_de_piece_a_l_echelle(
    piece: &PieceExportee,
    entete: &str,
    numero: usize,
    total: usize,
    mise: &MiseEnPageDuPdf,
) -> Option<Page> {
    let echelle = mise.mise_en_page.echelle;
    let points_par_pouce = echelle.points_par_pouce()?;
    let (x0, y0, x1, y1) = etendue(&a_cadrer(piece));
    let feuille = feuille_pour((x1 - x0, y1 - y0), points_par_pouce)?;
    let mut page = Page {
        taille: Some((feuille.largeur, feuille.hauteur)),
        ..Page::default()
    };
    en_tete_de_page(&mut page, &piece.nom, entete);
    let (zx0, zy0, zx1, zy1) = zone_du_plan(&feuille);
    let (cx, cy) = ((zx0 + zx1) / 2.0, (zy0 + zy1) / 2.0);
    let (mx, my) = ((x0 + x1) / 2.0, (y0 + y1) / 2.0);
    let vers_la_page = |(x, y): (f64, f64)| {
        (
            cx + (x - mx) * points_par_pouce,
            cy - (y - my) * points_par_pouce,
        )
    };
    dessiner_le_plan(&mut page, piece, &vers_la_page);
    // La colonne de droite : les résultats, puis, au bas, l'échelle graphique
    // et le cartouche.
    let x = feuille.largeur - MARGE - COLONNE;
    let bas_des_resultats = 66.0 + HAUTEUR_DU_CARTOUCHE + 56.0;
    let mut y = feuille.hauteur - 104.0;
    for (libelle, valeur) in &piece.resultats {
        if y - 11.0 < bas_des_resultats {
            break;
        }
        page.texte((x, y), 8.0, true, Alignement::Gauche, GRIS, libelle);
        page.texte(
            (x, y - 11.0),
            10.0,
            false,
            Alignement::Gauche,
            NOIR,
            &tronquer(valeur, 10.0, COLONNE),
        );
        y -= 25.0;
    }
    echelle_graphique(
        &mut page,
        (x, 66.0 + HAUTEUR_DU_CARTOUCHE + 26.0),
        echelle,
        points_par_pouce,
        mise.decimal,
    );
    cartouche(&mut page, (x, 66.0), mise, &feuille);
    pied_de_page(&mut page, numero, total);
    Some(page)
}

/// L'échelle graphique : des blocs noirs et blancs entre ses repères, leurs
/// nombres dessous et son unité, sous le nom de l'échelle.
fn echelle_graphique(
    page: &mut Page,
    (x, y): (f64, f64),
    echelle: EchelleDuPlan,
    points_par_pouce: f64,
    decimal: char,
) {
    let (reperes, pouces_par_unite, metrique) = echelle.reperes();
    page.texte(
        (x, y + 12.0),
        8.0,
        true,
        Alignement::Gauche,
        NOIR,
        &format!(
            "{} : {}",
            textes::DESSIN_ECHELLE.traduit(),
            textes::nom_de_l_echelle(echelle).traduit()
        ),
    );
    let a_la_page = |repere: f64| x + repere * pouces_par_unite * points_par_pouce;
    for (k, paire) in reperes.windows(2).enumerate() {
        let (a, b) = (a_la_page(paire[0]), a_la_page(paire[1]));
        let bloc = [(a, y), (b, y), (b, y + 5.0), (a, y + 5.0)];
        if k % 2 == 0 {
            page.polygone_plein(&bloc, NOIR);
        }
        page.trait_(&[bloc[0], bloc[1], bloc[2], bloc[3], bloc[0]], 0.5, NOIR);
    }
    for &repere in reperes {
        let texte = if repere.fract() == 0.0 {
            format!("{repere:.0}")
        } else {
            format!("{repere}").replace('.', &decimal.to_string())
        };
        page.texte(
            (a_la_page(repere), y - 9.0),
            7.0,
            false,
            Alignement::Centre,
            NOIR,
            &texte,
        );
    }
    let unite = if metrique {
        "m"
    } else {
        noyau::etiquettes::symbole(Format::PiDec)
    };
    if let Some(&dernier) = reperes.last() {
        page.texte(
            (a_la_page(dernier) + 8.0, y - 1.0),
            7.5,
            false,
            Alignement::Gauche,
            NOIR,
            unite,
        );
    }
}

/// Le cartouche : le projet, l'adresse, la date, le dessinateur, l'échelle et
/// le papier, chacun sur sa ligne.
fn cartouche(page: &mut Page, (x, y): (f64, f64), mise: &MiseEnPageDuPdf, feuille: &Feuille) {
    let lignes = [
        (
            textes::DESSIN_PROJET,
            mise.mise_en_page.projet.trim().to_string(),
        ),
        (
            textes::DESSIN_ADRESSE,
            mise.mise_en_page.adresse.trim().to_string(),
        ),
        (textes::DESSIN_DATE, mise.date.clone()),
        (
            textes::DESSIN_DESSINE_PAR,
            mise.mise_en_page.dessine_par.trim().to_string(),
        ),
        (
            textes::DESSIN_ECHELLE,
            textes::nom_de_l_echelle(mise.mise_en_page.echelle)
                .traduit()
                .to_string(),
        ),
        (
            textes::DESSIN_PAPIER,
            textes::dessin_papier(feuille.papier.traduit(), feuille.paysage),
        ),
    ];
    let hauteur_de_ligne = HAUTEUR_DU_CARTOUCHE / lignes.len() as f64;
    let (x1, y1) = (x + COLONNE, y + HAUTEUR_DU_CARTOUCHE);
    page.trait_(&[(x, y), (x1, y), (x1, y1), (x, y1), (x, y)], 0.8, NOIR);
    for (k, (libelle, valeur)) in lignes.iter().enumerate() {
        let haut = y1 - k as f64 * hauteur_de_ligne;
        if k > 0 {
            page.trait_(&[(x, haut), (x1, haut)], 0.3, GRIS);
        }
        let base = haut - hauteur_de_ligne + 5.5;
        page.texte(
            (x + 5.0, base),
            7.0,
            false,
            Alignement::Gauche,
            GRIS,
            libelle.traduit(),
        );
        page.texte(
            (x + 66.0, base),
            9.0,
            false,
            Alignement::Gauche,
            NOIR,
            &tronquer(&sans_insecables(valeur), 9.0, COLONNE - 71.0),
        );
    }
}

/// Un texte raccourci pour tenir dans cette largeur (points), avec « … ».
fn tronquer(texte: &str, taille: f64, largeur: f64) -> String {
    let largeur_de = |t: &str| largeur_du_texte(t, false) * taille / 1000.0;
    if largeur_de(texte) <= largeur {
        return texte.to_string();
    }
    let mut court = String::new();
    for c in texte.chars() {
        if largeur_de(&format!("{court}{c}…")) > largeur {
            break;
        }
        court.push(c);
    }
    format!("{}…", court.trim_end())
}

/// La page du plan, quand il a plusieurs pièces : chacune, puis le total.
fn page_du_plan(
    pieces: &[PieceExportee],
    prefs: &Preferences,
    entete: &str,
    numero: usize,
    total: usize,
) -> Page {
    let mut page = Page::default();
    en_tete_de_page(&mut page, &textes::dessin_export_plan(pieces.len()), entete);
    let colonnes = [MARGE, 300.0, 440.0];
    let mut y = 688.0;
    for (x, titre) in colonnes.iter().zip([
        textes::DESSIN_EXPORT_PIECE,
        textes::DESSIN_SURFACE_NETTE,
        textes::DESSIN_PERIMETRE,
    ]) {
        page.texte(
            (*x, y),
            10.0,
            true,
            Alignement::Gauche,
            GRIS,
            titre.traduit(),
        );
    }
    y -= 20.0;
    let mut somme = 0.0;
    for piece in pieces {
        if y < 90.0 {
            break;
        }
        page.texte(
            (colonnes[0], y),
            11.0,
            false,
            Alignement::Gauche,
            NOIR,
            &piece.nom,
        );
        match (piece.aire_nette, piece.perimetre) {
            (Some(aire), Some(perimetre)) => {
                somme += aire;
                let aire = sans_insecables(&Quantite::Aire(aire).en_clair(piece.modele, prefs));
                let perimetre =
                    sans_insecables(&Quantite::Longueur(perimetre).en_clair(piece.modele, prefs));
                page.texte(
                    (colonnes[1], y),
                    11.0,
                    false,
                    Alignement::Gauche,
                    NOIR,
                    &aire,
                );
                page.texte(
                    (colonnes[2], y),
                    11.0,
                    false,
                    Alignement::Gauche,
                    NOIR,
                    &perimetre,
                );
            }
            _ => page.texte(
                (colonnes[1], y),
                11.0,
                false,
                Alignement::Gauche,
                ORANGE,
                textes::DESSIN_EXPORT_INCOMPLETE.traduit(),
            ),
        }
        y -= 18.0;
    }
    y -= 4.0;
    page.trait_(
        &[(MARGE, y + 12.0), (LARGEUR_PAGE - MARGE, y + 12.0)],
        0.6,
        FILET,
    );
    let modele = pieces.iter().find_map(|piece| piece.modele);
    page.texte(
        (colonnes[0], y - 4.0),
        11.5,
        true,
        Alignement::Gauche,
        NOIR,
        textes::DESSIN_EXPORT_TOTAL.traduit(),
    );
    page.texte(
        (colonnes[1], y - 4.0),
        11.5,
        true,
        Alignement::Gauche,
        NOIR,
        &sans_insecables(&Quantite::Aire(somme).en_clair(modele, prefs)),
    );
    pied_de_page(&mut page, numero, total);
    page
}

/// Ajoute un objet numéroté au fichier PDF et retient où il commence.
fn ajouter_objet(sortie: &mut String, decalages: &mut Vec<usize>, corps: &str) {
    decalages.push(sortie.len());
    let numero = decalages.len();
    sortie.push_str(&format!("{numero} 0 obj\n{corps}\nendobj\n"));
}

/// Assemble les pages en un fichier PDF 1.4 : catalogue, pages, polices
/// standard, renseignements, puis la table des renvois.
fn assembler_pdf(pages: &[Page], titre: &str) -> String {
    let mut sortie = String::from("%PDF-1.4\n");
    let mut decalages = Vec::new();
    let enfants: Vec<String> = (0..pages.len())
        .map(|k| format!("{} 0 R", 6 + 2 * k))
        .collect();
    ajouter_objet(
        &mut sortie,
        &mut decalages,
        "<< /Type /Catalog /Pages 2 0 R >>",
    );
    ajouter_objet(
        &mut sortie,
        &mut decalages,
        &format!(
            "<< /Type /Pages /Kids [{}] /Count {} >>",
            enfants.join(" "),
            pages.len()
        ),
    );
    for police in ["Helvetica", "Helvetica-Bold"] {
        ajouter_objet(
            &mut sortie,
            &mut decalages,
            &format!(
                "<< /Type /Font /Subtype /Type1 /BaseFont /{police} /Encoding /WinAnsiEncoding >>"
            ),
        );
    }
    ajouter_objet(
        &mut sortie,
        &mut decalages,
        &format!(
            "<< /Title ({}) /Producer (Constructo AI {}) >>",
            chaine_pdf(titre),
            textes::VERSION
        ),
    );
    for page in pages {
        let (largeur, hauteur) = page.dimensions();
        let contenu = decalages.len() + 2;
        ajouter_objet(
            &mut sortie,
            &mut decalages,
            &format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {largeur} {hauteur}] /Resources << /Font << /F1 3 0 R /F2 4 0 R >> >> /Contents {contenu} 0 R >>"
            ),
        );
        ajouter_objet(
            &mut sortie,
            &mut decalages,
            &format!(
                "<< /Length {} >>\nstream\n{}endstream",
                page.operateurs.len(),
                page.operateurs
            ),
        );
    }
    let renvois = sortie.len();
    sortie.push_str(&format!(
        "xref\n0 {}\n0000000000 65535 f \n",
        decalages.len() + 1
    ));
    for decalage in &decalages {
        sortie.push_str(&format!("{decalage:010} 00000 n \n"));
    }
    sortie.push_str(&format!(
        "trailer\n<< /Size {} /Root 1 0 R /Info 5 0 R >>\nstartxref\n{renvois}\n%%EOF\n",
        decalages.len() + 1
    ));
    sortie
}

/// Le plan en PDF, format Lettre : une page de résumé s'il a plusieurs
/// pièces, puis une page par pièce. `date` est écrite en tête des pages.
pub fn pdf_du_plan(plan: &Plan, prefs: &Preferences, date: &str) -> String {
    let pieces = preparer(plan, prefs);
    let entete = textes::dessin_export_entete(date);
    let resume = pieces.len() != 1;
    let total = pieces.len() + usize::from(resume);
    let mut pages = Vec::with_capacity(total);
    if resume {
        pages.push(page_du_plan(&pieces, prefs, &entete, 1, total));
    }
    let mise = MiseEnPageDuPdf {
        mise_en_page: plan.mise_en_page.clone(),
        date: date.to_string(),
        decimal: noyau::format::separateur_decimal(prefs),
    };
    for piece in &pieces {
        let numero = pages.len() + 1;
        // À l'échelle choisie, si la pièce est complète; ajustée sinon, ou si
        // elle ne tient sur aucune feuille (et la page le dit).
        let page = match plan.mise_en_page.echelle.points_par_pouce() {
            Some(_) if piece.a_l_echelle => page_de_piece_a_l_echelle(
                piece, &entete, numero, total, &mise,
            )
            .unwrap_or_else(|| {
                let avis = textes::dessin_export_ne_tient_pas(
                    textes::nom_de_l_echelle(plan.mise_en_page.echelle).traduit(),
                );
                page_de_piece(piece, &entete, numero, total, Some(&avis))
            }),
            _ => page_de_piece(piece, &entete, numero, total, None),
        };
        pages.push(page);
    }
    assembler_pdf(&pages, &textes::dessin_export_plan(pieces.len()))
}

// ------------------------------------------------------------ Rapport de calcul

/// Ce qui ne s'écrit pas en WinAnsi, récrit pour le PDF : l'ohm, le prime et
/// le double prime des angles, l'exposant 9.
fn pour_le_pdf(texte: &str) -> String {
    let mut sortie = String::with_capacity(texte.len());
    for c in texte.chars() {
        // Par leur code : ces signes ne s'affichent jamais à l'écran.
        match u32::from(c) {
            0x03A9 => sortie.push_str("ohms"),
            0x2032 => sortie.push('\''),
            0x2033 => sortie.push('"'),
            0x2079 => sortie.push_str("^9"),
            _ => sortie.push(c),
        }
    }
    sortie
}

/// Coupe un texte en lignes d'au plus `largeur` points, entre les mots.
fn couper_en_lignes(texte: &str, taille: f64, largeur: f64) -> Vec<String> {
    let mut lignes = Vec::new();
    for paragraphe in texte.split('\n') {
        let mut ligne = String::new();
        for mot in paragraphe.split_whitespace() {
            let essai = if ligne.is_empty() {
                mot.to_string()
            } else {
                format!("{ligne} {mot}")
            };
            if !ligne.is_empty() && largeur_du_texte(&essai, false) * taille / 1000.0 > largeur {
                lignes.push(std::mem::replace(&mut ligne, mot.to_string()));
            } else {
                ligne = essai;
            }
        }
        lignes.push(ligne);
    }
    lignes
}

/// Le rapport d'un calcul en PDF (section 22.18), format Lettre : le nom du
/// calculateur, l'en-tête (projet et date), les données entrées, les
/// résultats et la note du calculateur; « Constructo AI · constructoai.ca »
/// au bas, comme les plans.
pub fn pdf_d_un_calcul(
    titre: &str,
    entete: &str,
    entrees: &[(&str, String)],
    resultats: &[(&str, String)],
    note: &str,
) -> String {
    let mut page = Page::default();
    en_tete_de_page(&mut page, &pour_le_pdf(titre), &pour_le_pdf(entete));
    let largeur = LARGEUR_PAGE - 2.0 * MARGE;
    let droite = LARGEUR_PAGE - MARGE;
    let mut y = 688.0;
    for (section, lignes, gras) in [
        (textes::RAPPORT_DONNEES.traduit(), entrees, false),
        (textes::RESULTATS.traduit(), resultats, true),
    ] {
        if lignes.is_empty() {
            continue;
        }
        page.texte((MARGE, y), 12.0, true, Alignement::Gauche, BLEU, section);
        y -= 8.0;
        page.trait_(&[(MARGE, y), (droite, y)], 0.6, FILET);
        y -= 16.0;
        for (libelle, valeur) in lignes {
            let (libelle, valeur) = (pour_le_pdf(libelle), pour_le_pdf(valeur));
            page.texte((MARGE, y), 10.0, false, Alignement::Gauche, GRIS, &libelle);
            let place = (largeur_du_texte(&libelle, false) * 10.0
                + largeur_du_texte(&valeur, gras) * 10.5)
                / 1000.0
                + 18.0;
            if place > largeur {
                // Trop long pour une ligne : la valeur passe dessous.
                y -= 14.0;
            }
            page.texte((droite, y), 10.5, gras, Alignement::Droite, NOIR, &valeur);
            y -= 7.0;
            page.trait_(&[(MARGE, y), (droite, y)], 0.3, FILET);
            y -= 14.0;
        }
        y -= 16.0;
    }
    // La note du calculateur : sa précision ou sa mise en garde.
    for ligne in couper_en_lignes(&pour_le_pdf(note), 9.0, largeur) {
        if y < 70.0 {
            break;
        }
        page.texte((MARGE, y), 9.0, false, Alignement::Gauche, GRIS, &ligne);
        y -= 12.0;
    }
    pied_de_page(&mut page, 1, 1);
    assembler_pdf(&[page], &pour_le_pdf(titre))
}

// ------------------------------------------------------------ Photo cotée

/// Une cote à dessiner sur la photo d'un PDF : ses bouts, en part de la photo
/// (de 0 à 1, y vers le bas), et son texte (« 32 po », « ~ 2 pi 0 po »).
pub struct CoteExportee {
    pub a: (f64, f64),
    pub b: (f64, f64),
    pub texte: String,
}

/// La photo cotée en PDF, format Lettre (R-100) : son nom et la note en tête,
/// la photo ajustée à la page, ses cotes par-dessus. `jpeg` est la photo déjà
/// tournée, insérée telle quelle (filtre DCTDecode).
pub fn pdf_d_une_photo(
    titre: &str,
    entete: &str,
    jpeg: &[u8],
    (largeur, hauteur): (u32, u32),
    cotes: &[CoteExportee],
) -> Vec<u8> {
    let mut page = Page::default();
    en_tete_de_page(&mut page, titre, entete);
    // La place de la photo, entre l'en-tête et le pied de page.
    let (x0, y0, x1, y1) = (MARGE, 66.0, LARGEUR_PAGE - MARGE, 700.0);
    let (lp, hp) = (f64::from(largeur.max(1)), f64::from(hauteur.max(1)));
    let echelle = ((x1 - x0) / lp).min((y1 - y0) / hp);
    let (l, h) = (lp * echelle, hp * echelle);
    let (gauche, bas) = (x0 + ((x1 - x0) - l) / 2.0, y1 - h);
    page.operateurs.push_str(&format!(
        "q {l:.2} 0 0 {h:.2} {gauche:.2} {bas:.2} cm /Im1 Do Q\n"
    ));
    let sur_la_page = |(u, v): (f64, f64)| (gauche + u * l, bas + (1.0 - v) * h);
    // Les traits d'abord, puis les textes : aucun trait ne cache un texte.
    for cote in cotes {
        let (a, b) = (sur_la_page(cote.a), sur_la_page(cote.b));
        page.trait_(&[a, b], 1.6, ORANGE);
        // Un trait de bout à chaque extrémité, en travers de la cote.
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let longueur = dx.hypot(dy).max(1e-9);
        let (nx, ny) = (-dy / longueur * 5.0, dx / longueur * 5.0);
        for bout in [a, b] {
            page.trait_(
                &[(bout.0 - nx, bout.1 - ny), (bout.0 + nx, bout.1 + ny)],
                1.6,
                ORANGE,
            );
        }
    }
    for cote in cotes {
        let (a, b) = (sur_la_page(cote.a), sur_la_page(cote.b));
        // Son texte au milieu, sur fond blanc.
        let milieu = ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0);
        let taille = 9.0;
        let etendue = largeur_du_texte(&cote.texte, true) * taille / 1000.0;
        let (gx, by) = (milieu.0 - etendue / 2.0 - 3.0, milieu.1 - 4.0);
        page.polygone_plein(
            &[
                (gx, by),
                (gx + etendue + 6.0, by),
                (gx + etendue + 6.0, by + 13.0),
                (gx, by + 13.0),
            ],
            BLANC,
        );
        page.texte(
            (milieu.0, milieu.1 - 1.0),
            taille,
            true,
            Alignement::Centre,
            NOIR,
            &cote.texte,
        );
    }
    pied_de_page(&mut page, 1, 1);
    assembler_pdf_d_une_photo(&page, titre, jpeg, (largeur, hauteur))
}

fn ajouter_objet_binaire(sortie: &mut Vec<u8>, decalages: &mut Vec<usize>, corps: &[u8]) {
    decalages.push(sortie.len());
    let numero = decalages.len();
    sortie.extend_from_slice(format!("{numero} 0 obj\n").as_bytes());
    sortie.extend_from_slice(corps);
    sortie.extend_from_slice(b"\nendobj\n");
}

/// Assemble une page et sa photo en un fichier PDF 1.4 : catalogue, pages,
/// polices standard, renseignements, la page, son contenu, puis la photo.
fn assembler_pdf_d_une_photo(
    page: &Page,
    titre: &str,
    jpeg: &[u8],
    (largeur, hauteur): (u32, u32),
) -> Vec<u8> {
    // La seconde ligne annonce un fichier binaire.
    let mut sortie: Vec<u8> = b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n".to_vec();
    let mut decalages = Vec::new();
    let mut objet = |sortie: &mut Vec<u8>, corps: &[u8]| {
        ajouter_objet_binaire(sortie, &mut decalages, corps);
    };
    objet(&mut sortie, b"<< /Type /Catalog /Pages 2 0 R >>");
    objet(&mut sortie, b"<< /Type /Pages /Kids [6 0 R] /Count 1 >>");
    for police in ["Helvetica", "Helvetica-Bold"] {
        objet(
            &mut sortie,
            format!(
                "<< /Type /Font /Subtype /Type1 /BaseFont /{police} /Encoding /WinAnsiEncoding >>"
            )
            .as_bytes(),
        );
    }
    objet(
        &mut sortie,
        format!(
            "<< /Title ({}) /Producer (Constructo AI {}) >>",
            chaine_pdf(titre),
            textes::VERSION
        )
        .as_bytes(),
    );
    objet(
        &mut sortie,
        format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {LARGEUR_PAGE} {HAUTEUR_PAGE}] /Resources << /Font << /F1 3 0 R /F2 4 0 R >> /XObject << /Im1 8 0 R >> >> /Contents 7 0 R >>"
        )
        .as_bytes(),
    );
    objet(
        &mut sortie,
        format!(
            "<< /Length {} >>\nstream\n{}endstream",
            page.operateurs.len(),
            page.operateurs
        )
        .as_bytes(),
    );
    let mut image = format!(
        "<< /Type /XObject /Subtype /Image /Width {largeur} /Height {hauteur} /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /DCTDecode /Length {} >>\nstream\n",
        jpeg.len()
    )
    .into_bytes();
    image.extend_from_slice(jpeg);
    image.extend_from_slice(b"\nendstream");
    objet(&mut sortie, &image);
    let renvois = sortie.len();
    sortie.extend_from_slice(
        format!("xref\n0 {}\n0000000000 65535 f \n", decalages.len() + 1).as_bytes(),
    );
    for decalage in &decalages {
        sortie.extend_from_slice(format!("{decalage:010} 00000 n \n").as_bytes());
    }
    sortie.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R /Info 5 0 R >>\nstartxref\n{renvois}\n%%EOF\n",
            decalages.len() + 1
        )
        .as_bytes(),
    );
    sortie
}

// ---------------------------------------------------------------------- DXF

/// Un texte écrit pour le DXF : ASCII, les autres caractères en `\U+XXXX`.
fn texte_dxf(texte: &str) -> String {
    let mut sortie = String::with_capacity(texte.len() + 8);
    for c in texte.chars() {
        match c as u32 {
            0x20..=0x7E => sortie.push(c),
            0xA0 => sortie.push(' '),
            0x2212 => sortie.push('-'),
            0x00..=0x1F => sortie.push(' '),
            code => sortie.push_str(&format!("\\U+{code:04X}")),
        }
    }
    sortie
}

/// Ajoute un couple code-valeur au DXF.
fn groupe(dxf: &mut String, code: u16, valeur: &str) {
    dxf.push_str(&format!("{code:>3}\r\n{valeur}\r\n"));
}

fn nombre_dxf(x: f64) -> String {
    format!("{x:.4}")
}

fn ligne_dxf(dxf: &mut String, calque: &str, a: (f64, f64), b: (f64, f64)) {
    groupe(dxf, 0, "LINE");
    groupe(dxf, 8, calque);
    for (code, valeur) in [
        (10, a.0),
        (20, a.1),
        (30, 0.0),
        (11, b.0),
        (21, b.1),
        (31, 0.0),
    ] {
        groupe(dxf, code, &nombre_dxf(valeur));
    }
}

fn cercle_dxf(dxf: &mut String, calque: &str, centre: (f64, f64), rayon: f64) {
    groupe(dxf, 0, "CIRCLE");
    groupe(dxf, 8, calque);
    for (code, valeur) in [(10, centre.0), (20, centre.1), (30, 0.0), (40, rayon)] {
        groupe(dxf, code, &nombre_dxf(valeur));
    }
}

fn arc_dxf(dxf: &mut String, calque: &str, centre: (f64, f64), rayon: f64, angles: (f64, f64)) {
    groupe(dxf, 0, "ARC");
    groupe(dxf, 8, calque);
    for (code, valeur) in [
        (10, centre.0),
        (20, centre.1),
        (30, 0.0),
        (40, rayon),
        (50, angles.0),
        (51, angles.1),
    ] {
        groupe(dxf, code, &nombre_dxf(valeur));
    }
}

/// Les portes (battant et arc) et les fenêtres (trait double et vitrage) sur
/// le calque OUVERTURES. Le DXF a l'axe des y vers le haut.
fn ouvertures_dans_le_dxf(
    dxf: &mut String,
    points: &[(f64, f64)],
    ouvertures: &[(usize, f64, f64, bool)],
    facteur: f64,
    epaisseur: f64,
) {
    let sens = aire_signee(points);
    for &(cote, t0, t1, porte) in ouvertures {
        let Some(Geometrie {
            q0,
            q1,
            u,
            dedans,
            largeur,
        }) = geometrie_d_une_ouverture(points, sens, (cote, t0, t1))
        else {
            continue;
        };
        if porte {
            // Un mur vers l'intérieur : la porte s'ouvre de sa face intérieure.
            let q0 = if epaisseur < 0.0 {
                (q0.0 - dedans.0 * epaisseur, q0.1 - dedans.1 * epaisseur)
            } else {
                q0
            };
            let battant = (q0.0 + dedans.0 * largeur, q0.1 + dedans.1 * largeur);
            ligne_dxf(dxf, "OUVERTURES", q0, battant);
            // L'arc va, dans le sens trigonométrique, du mur au battant.
            arc_dxf(
                dxf,
                "OUVERTURES",
                q0,
                largeur,
                angles_d_un_arc_de_porte(u, dedans),
            );
        } else if epaisseur.abs() > 1e-9 {
            // La fenêtre traverse le mur : ses deux faces et son vitrage.
            for part in [0.0, 0.5, 1.0] {
                let k = -epaisseur * part;
                ligne_dxf(
                    dxf,
                    "OUVERTURES",
                    (q0.0 + dedans.0 * k, q0.1 + dedans.1 * k),
                    (q1.0 + dedans.0 * k, q1.1 + dedans.1 * k),
                );
            }
        } else {
            let ecart = 2.0 * facteur;
            for decalage in [-ecart, ecart] {
                ligne_dxf(
                    dxf,
                    "OUVERTURES",
                    (q0.0 + dedans.0 * decalage, q0.1 + dedans.1 * decalage),
                    (q1.0 + dedans.0 * decalage, q1.1 + dedans.1 * decalage),
                );
            }
            ligne_dxf(dxf, "OUVERTURES", q0, q1);
        }
    }
}

/// Un texte, ancré (horizontalement) à gauche, au centre ou à droite, et
/// centré verticalement sur son point.
fn texte_dans_le_dxf(
    dxf: &mut String,
    calque: &str,
    (x, y): (f64, f64),
    hauteur: f64,
    alignement: Alignement,
    texte: &str,
) {
    let horizontal = match alignement {
        Alignement::Gauche => "0",
        Alignement::Centre => "1",
        Alignement::Droite => "2",
    };
    groupe(dxf, 0, "TEXT");
    groupe(dxf, 8, calque);
    for (code, valeur) in [(10, x), (20, y), (30, 0.0), (40, hauteur)] {
        groupe(dxf, code, &nombre_dxf(valeur));
    }
    groupe(dxf, 1, &texte_dxf(texte));
    groupe(dxf, 72, horizontal);
    for (code, valeur) in [(11, x), (21, y), (31, 0.0)] {
        groupe(dxf, code, &nombre_dxf(valeur));
    }
    groupe(dxf, 73, "2");
}

/// Le plan en DXF (R12, ASCII) : les pièces complètes, à l'échelle réelle,
/// côte à côte — calques MURS, COTES et PIECES. L'unité est le pouce, ou le
/// millimètre si les mesures sont métriques. `None` s'il n'y a aucune pièce
/// complète (R-78).
pub fn dxf_du_plan(plan: &Plan, prefs: &Preferences) -> Option<String> {
    let pieces: Vec<PieceExportee> = preparer(plan, prefs)
        .into_iter()
        .filter(|piece| piece.a_l_echelle)
        .collect();
    if pieces.is_empty() {
        return None;
    }
    let metrique = pieces
        .iter()
        .find_map(|piece| piece.modele)
        .and_then(Format::systeme)
        == Some(Systeme::Metrique);
    let (facteur, unites) = if metrique { (25.4, "4") } else { (1.0, "1") };

    let mut entites = String::new();
    let (mut min, mut max) = ((f64::MAX, f64::MAX), (f64::MIN, f64::MIN));
    let mut depart = 0.0;
    for piece in &pieces {
        // Le DXF a l'axe des y vers le haut.
        let points: Vec<(f64, f64)> = piece
            .sommets
            .iter()
            .map(|&(x, y)| (x * facteur, -y * facteur))
            .collect();
        let etendue: Vec<(f64, f64)> = piece
            .objets
            .iter()
            .flat_map(|(_, _, traits)| traits.iter().flat_map(etendue_d_un_trait))
            .chain(piece.notes.iter().map(|(_, position)| *position))
            .chain(piece.cloisons.iter().flat_map(|morceau| morceau.coins))
            .chain(piece.contour_exterieur.iter().copied())
            .map(|(x, y)| (x * facteur, -y * facteur))
            .chain(points.iter().copied())
            .collect();
        let gauche = etendue.iter().map(|q| q.0).fold(f64::MAX, f64::min);
        let droite = etendue.iter().map(|q| q.0).fold(f64::MIN, f64::max);
        let bas = etendue.iter().map(|q| q.1).fold(f64::MAX, f64::min);
        let haut = etendue.iter().map(|q| q.1).fold(f64::MIN, f64::max);
        let decalage = depart - gauche;
        let points: Vec<(f64, f64)> = points.iter().map(|&(x, y)| (x + decalage, y)).collect();
        let (largeur, hauteur) = (droite - gauche, haut - bas);
        let taille = (largeur.max(hauteur) * 0.025).clamp(3.0 * facteur, 12.0 * facteur);
        // Sans épaisseur, le mur est un trait, interrompu aux portes et aux
        // fenêtres; à son épaisseur, il se trace plus bas.
        let sans_epaisseur = piece.murs_exterieurs.is_empty();
        for (cote, segment) in points.windows(2).enumerate().filter(|_| sans_epaisseur) {
            // Le mur s'interrompt aux portes et aux fenêtres.
            let point = |t: f64| {
                (
                    segment[0].0 + (segment[1].0 - segment[0].0) * t,
                    segment[0].1 + (segment[1].1 - segment[0].1) * t,
                )
            };
            let mut coupures: Vec<(f64, f64)> = piece
                .ouvertures
                .iter()
                .filter(|ouverture| ouverture.0 == cote)
                .map(|ouverture| (ouverture.1, ouverture.2))
                .collect();
            coupures.sort_by(|a, b| a.0.total_cmp(&b.0));
            let mut depuis = 0.0_f64;
            for (t0, t1) in coupures.into_iter().chain([(1.0, 1.0)]) {
                if t0 > depuis + 1e-9 {
                    ligne_dxf(&mut entites, "MURS", point(depuis), point(t0));
                }
                depuis = depuis.max(t1);
            }
        }
        ouvertures_dans_le_dxf(
            &mut entites,
            &points,
            &piece.ouvertures,
            facteur,
            piece.epaisseur_des_murs * facteur,
        );
        // Les objets, sur le calque MOBILIER.
        let vers_le_dxf = |(x, y): (f64, f64)| (x * facteur + decalage, -y * facteur);
        for (nom, centre, traits) in &piece.objets {
            for trait_ in traits {
                match *trait_ {
                    noyau::mobilier::TraitSurLePlan::Ligne(a, b) => {
                        ligne_dxf(&mut entites, "MOBILIER", vers_le_dxf(a), vers_le_dxf(b));
                    }
                    noyau::mobilier::TraitSurLePlan::Cercle(c, rayon) => {
                        cercle_dxf(&mut entites, "MOBILIER", vers_le_dxf(c), rayon * facteur);
                    }
                }
            }
            texte_dans_le_dxf(
                &mut entites,
                "MOBILIER",
                vers_le_dxf(*centre),
                taille * 0.8,
                Alignement::Centre,
                nom,
            );
        }
        // Les murs à leur épaisseur : l'intérieur, l'extérieur et les jambages.
        for morceau in &piece.murs_exterieurs {
            let [a, b, c, d] = morceau.coins.map(vers_le_dxf);
            ligne_dxf(&mut entites, "MURS", a, b);
            ligne_dxf(&mut entites, "MURS", d, c);
            if morceau.jambage_au_debut {
                ligne_dxf(&mut entites, "MURS", a, d);
            }
            if morceau.jambage_a_la_fin {
                ligne_dxf(&mut entites, "MURS", b, c);
            }
        }
        // Les cloisons, sur le calque CLOISONS.
        for morceau in &piece.cloisons {
            let [a, b, c, d] = morceau.coins.map(vers_le_dxf);
            ligne_dxf(&mut entites, "CLOISONS", a, b);
            ligne_dxf(&mut entites, "CLOISONS", d, c);
            if morceau.bout_au_debut {
                ligne_dxf(&mut entites, "CLOISONS", a, d);
            }
            if morceau.bout_a_la_fin {
                ligne_dxf(&mut entites, "CLOISONS", b, c);
            }
        }
        for porte in &piece.portes_de_cloisons {
            let (q0, q1) = (vers_le_dxf(porte.gonds), vers_le_dxf(porte.autre_bord));
            let bout = vers_le_dxf((porte.gonds.0 + porte.cote.0, porte.gonds.1 + porte.cote.1));
            let Some(Geometrie {
                q0,
                u,
                dedans,
                largeur,
                ..
            }) = geometrie_d_une_porte_de_cloison(q0, q1, bout)
            else {
                continue;
            };
            let battant = (q0.0 + dedans.0 * largeur, q0.1 + dedans.1 * largeur);
            ligne_dxf(&mut entites, "CLOISONS", q0, battant);
            arc_dxf(
                &mut entites,
                "CLOISONS",
                q0,
                largeur,
                angles_d_un_arc_de_porte(u, dedans),
            );
        }
        // Les notes, sur le calque NOTES.
        for (texte, position) in &piece.notes {
            texte_dans_le_dxf(
                &mut entites,
                "NOTES",
                vers_le_dxf(*position),
                taille * 0.8,
                Alignement::Centre,
                texte,
            );
        }
        let nombre = points.len().saturating_sub(1).max(1);
        let centre = (
            points.iter().take(nombre).map(|q| q.0).sum::<f64>() / nombre as f64,
            points.iter().take(nombre).map(|q| q.1).sum::<f64>() / nombre as f64,
        );
        for (i, segment) in points.windows(2).enumerate() {
            let Some((texte, _)) = piece.cotes.get(i) else {
                continue;
            };
            let milieu = (
                (segment[0].0 + segment[1].0) / 2.0,
                (segment[0].1 + segment[1].1) / 2.0,
            );
            let (nx, ny) = normale_exterieure(segment[0], segment[1], centre);
            // La mesure s'écrit hors du mur, à son épaisseur.
            let epaisseur = piece.epaisseur_des_murs.max(0.0) * facteur;
            let milieu = (milieu.0 + nx * epaisseur, milieu.1 + ny * epaisseur);
            let (position, alignement) = if ny.abs() > 0.9 {
                (
                    (milieu.0, milieu.1 + ny.signum() * taille * 1.2),
                    Alignement::Centre,
                )
            } else if nx.abs() > 0.9 {
                if nx > 0.0 {
                    ((milieu.0 + taille * 0.8, milieu.1), Alignement::Gauche)
                } else {
                    ((milieu.0 - taille * 0.8, milieu.1), Alignement::Droite)
                }
            } else {
                let position = (milieu.0 + nx * taille, milieu.1 + ny * taille);
                if nx > 0.0 {
                    (position, Alignement::Gauche)
                } else {
                    (position, Alignement::Droite)
                }
            };
            texte_dans_le_dxf(&mut entites, "COTES", position, taille, alignement, texte);
        }
        // Les lignes de cote (point 35), sur le calque COTES aussi.
        let mur = piece.epaisseur_des_murs.max(0.0) * facteur;
        for (chaine, textes_des_cotes) in &piece.chaines {
            // Au-delà de la mesure du mur, qui s'écrit à côté d'un mur vertical.
            let largeur_de_la_mesure = chaine
                .mur
                .and_then(|k| piece.cotes.get(k))
                .map_or(0.0, |(texte, _)| {
                    largeur_du_texte(texte, false) * taille / 1000.0
                });
            let decalage = if chaine.dehors.1.abs() > 0.9 {
                mur + taille * 3.0
            } else {
                (mur + taille * 3.0).max(mur + taille * 1.3 + largeur_de_la_mesure)
            };
            let Some(cote) = poser_une_chaine(
                chaine,
                &vers_le_dxf,
                decalage,
                mur + taille * 0.3,
                taille * 0.35,
                centre,
            ) else {
                continue;
            };
            ligne_dxf(&mut entites, "COTES", cote.ligne[0], cote.ligne[1]);
            for [a, b] in cote.attaches.iter().chain(&cote.traits) {
                ligne_dxf(&mut entites, "COTES", *a, *b);
            }
            let hauteur = taille * 0.7;
            for (&(milieu, longueur, vers), texte) in cote.morceaux.iter().zip(textes_des_cotes) {
                let largeur = largeur_du_texte(texte, false) * hauteur / 1000.0;
                let horizontal = vers.1.abs() > 0.9;
                if (horizontal && largeur > longueur) || (!horizontal && hauteur * 1.2 > longueur) {
                    continue;
                }
                // Le texte du DXF se place par son milieu, en hauteur.
                let ((x, y), alignement) = place_du_texte(milieu, vers, taille * 0.3, hauteur);
                texte_dans_le_dxf(
                    &mut entites,
                    "COTES",
                    (x, y + hauteur / 2.0),
                    hauteur,
                    alignement,
                    texte,
                );
            }
        }
        // Le nom de la pièce et sa surface nette, sous la pièce.
        let etiquette = match piece.aire_nette {
            Some(aire) => format!(
                "{} : {}",
                piece.nom,
                sans_insecables(&Quantite::Aire(aire).en_clair(piece.modele, prefs))
            ),
            None => piece.nom.clone(),
        };
        let position = (depart + largeur / 2.0, bas - taille * 3.5);
        texte_dans_le_dxf(
            &mut entites,
            "PIECES",
            position,
            taille * 1.4,
            Alignement::Centre,
            &etiquette,
        );
        min = (
            min.0.min(depart - taille * 12.0),
            min.1.min(bas - taille * 5.0),
        );
        max = (
            max.0.max(depart + largeur + taille * 12.0),
            max.1.max(haut + taille * 3.0),
        );
        depart += largeur + (largeur.max(hauteur) * 0.3).max(48.0 * facteur);
    }

    let mut dxf = String::new();
    groupe(&mut dxf, 0, "SECTION");
    groupe(&mut dxf, 2, "HEADER");
    groupe(&mut dxf, 9, "$ACADVER");
    groupe(&mut dxf, 1, "AC1009");
    groupe(&mut dxf, 9, "$DWGCODEPAGE");
    groupe(&mut dxf, 3, "ANSI_1252");
    groupe(&mut dxf, 9, "$INSUNITS");
    groupe(&mut dxf, 70, unites);
    for (variable, (x, y)) in [("$EXTMIN", min), ("$EXTMAX", max)] {
        groupe(&mut dxf, 9, variable);
        groupe(&mut dxf, 10, &nombre_dxf(x));
        groupe(&mut dxf, 20, &nombre_dxf(y));
        groupe(&mut dxf, 30, "0.0");
    }
    groupe(&mut dxf, 0, "ENDSEC");

    groupe(&mut dxf, 0, "SECTION");
    groupe(&mut dxf, 2, "TABLES");
    groupe(&mut dxf, 0, "TABLE");
    groupe(&mut dxf, 2, "LTYPE");
    groupe(&mut dxf, 70, "1");
    groupe(&mut dxf, 0, "LTYPE");
    groupe(&mut dxf, 2, "CONTINUOUS");
    groupe(&mut dxf, 70, "0");
    groupe(&mut dxf, 3, "Solid line");
    groupe(&mut dxf, 72, "65");
    groupe(&mut dxf, 73, "0");
    groupe(&mut dxf, 40, "0.0");
    groupe(&mut dxf, 0, "ENDTAB");
    let calques = [
        ("0", "7"),
        ("MURS", "5"),
        ("COTES", "7"),
        ("PIECES", "3"),
        ("OUVERTURES", "1"),
        ("MOBILIER", "6"),
        ("NOTES", "2"),
        ("CLOISONS", "5"),
    ];
    groupe(&mut dxf, 0, "TABLE");
    groupe(&mut dxf, 2, "LAYER");
    groupe(&mut dxf, 70, &calques.len().to_string());
    for (nom, couleur) in calques {
        groupe(&mut dxf, 0, "LAYER");
        groupe(&mut dxf, 2, nom);
        groupe(&mut dxf, 70, "0");
        groupe(&mut dxf, 62, couleur);
        groupe(&mut dxf, 6, "CONTINUOUS");
    }
    groupe(&mut dxf, 0, "ENDTAB");
    groupe(&mut dxf, 0, "ENDSEC");

    groupe(&mut dxf, 0, "SECTION");
    groupe(&mut dxf, 2, "ENTITIES");
    dxf.push_str(&entites);
    groupe(&mut dxf, 0, "ENDSEC");
    groupe(&mut dxf, 0, "EOF");
    Some(dxf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_rapport_d_un_calcul() {
        let entrees = vec![
            ("Longueur", "24 pi 0 po".to_string()),
            ("Pertes", "10 %".to_string()),
        ];
        let resultats = vec![
            ("Volume de beton", "5,93 vg³ (4,53 m³)".to_string()),
            ("Resistance", "47 Ω".to_string()),
        ];
        let pdf = pdf_d_un_calcul(
            "Dalle de beton",
            "Agrandissement Tremblay · Rapport de calcul · 23 novembre 2026, 10 h 51",
            &entrees,
            &resultats,
            "Une note assez longue pour passer a la ligne. "
                .repeat(6)
                .trim(),
        );
        assert!(pdf.starts_with("%PDF-1.4"));
        assert!(pdf.trim_end().ends_with("%%EOF"));
        assert!(pdf.contains("/Count 1"));
        for attendu in [
            "Dalle de beton",
            "Agrandissement Tremblay",
            "Longueur",
            "24 pi 0 po",
            "Volume de beton",
            "47 ohms",
            "constructoai.ca",
        ] {
            assert!(pdf.contains(attendu), "{attendu}");
        }
        // Le mètre cube s'écrit en WinAnsi, l'ohm en toutes lettres.
        assert!(pdf.contains("m\\263"));
        assert!(!pdf.contains('Ω'));
    }

    #[test]
    fn couper_une_note_en_lignes() {
        let lignes = couper_en_lignes("un deux trois quatre cinq six sept", 10.0, 60.0);
        assert!(lignes.len() > 1);
        assert_eq!(lignes.join(" "), "un deux trois quatre cinq six sept");
        assert!(
            lignes
                .iter()
                .all(|ligne| largeur_du_texte(ligne, false) * 10.0 / 1000.0 <= 60.0)
        );
        assert_eq!(couper_en_lignes("", 10.0, 60.0), [""]);
    }
    use noyau::dessin::{Cote, Piece, Retrait, Sens};

    fn rectangle(longueur: &str, largeur: &str) -> Dessin {
        let cote = |sens, croquis, saisie: &str| Cote {
            sens,
            croquis,
            saisie: saisie.to_string(),
        };
        Dessin {
            cotes: vec![
                cote(Sens::Est, 200.0, longueur),
                cote(Sens::Sud, 150.0, largeur),
                cote(Sens::Ouest, 200.0, ""),
                cote(Sens::Nord, 150.0, ""),
            ],
            fermee: true,
            // Des murs sans épaisseur : un trait par côté.
            epaisseur_des_murs: "0".to_string(),
            ..Dessin::default()
        }
    }

    fn plan(pieces: Vec<(&str, Dessin)>) -> Plan {
        Plan {
            pieces: pieces
                .into_iter()
                .map(|(nom, dessin)| Piece {
                    nom: nom.to_string(),
                    dessin,
                })
                .collect(),
            courante: 0,
            ..Plan::default()
        }
    }

    #[test]
    fn les_chaines_pdf_sont_en_winansi() {
        assert_eq!(chaine_pdf("é (a) \\ − ²"), "\\351 \\(a\\) \\\\ - \\262");
        assert_eq!(chaine_pdf("12 pi 6 1/2 po"), "12 pi 6 1/2 po");
        // Un caractère sans équivalent devient un point d'interrogation.
        assert_eq!(chaine_pdf("中"), "?");
        assert!((largeur_du_texte("Pi", false) - 889.0).abs() < 1e-9);
    }

    #[test]
    fn le_pdf_d_une_photo_cotee_est_bien_forme() {
        // Un faux JPEG suffit : le PDF le recopie tel quel.
        let jpeg: Vec<u8> = vec![0xFF, 0xD8, 0x00, 0x0A, b'e', b'n', b'd', 0xFF, 0xD9];
        let cotes = [CoteExportee {
            a: (0.1, 0.5),
            b: (0.6, 0.5),
            texte: "Réf. 32 po".to_string(),
        }];
        let pdf = pdf_d_une_photo(
            "fenetre.jpg",
            "Fenêtres du salon · 23 novembre 2026",
            &jpeg,
            (2048, 1536),
            &cotes,
        );
        assert!(pdf.starts_with(b"%PDF-1.4\n") && pdf.ends_with(b"%%EOF\n"));
        let texte = String::from_utf8_lossy(&pdf);
        assert!(texte.contains("/Subtype /Image /Width 2048 /Height 1536"));
        assert!(texte.contains("/Filter /DCTDecode /Length 9 >>"));
        assert!(texte.contains("/Im1 Do"));
        assert!(texte.contains("(R\\351f. 32 po) Tj"));
        // La photo est recopiée octet pour octet, entre « stream » et « endstream ».
        let debut = pdf
            .windows(jpeg.len())
            .position(|w| w == jpeg.as_slice())
            .unwrap();
        assert!(pdf[debut + jpeg.len()..].starts_with(b"\nendstream"));
        // Chaque objet est là où la table des renvois le dit.
        let depart: usize = texte
            .rsplit("startxref\n")
            .next()
            .and_then(|fin| fin.lines().next())
            .and_then(|nombre| nombre.parse().ok())
            .unwrap();
        assert!(pdf[depart..].starts_with(b"xref\n"));
        let table = String::from_utf8_lossy(&pdf[depart..]).to_string();
        let renvois: Vec<usize> = table
            .lines()
            .skip(3)
            .take_while(|ligne| ligne.ends_with(" n "))
            .map(|ligne| ligne[..10].parse().unwrap())
            .collect();
        assert_eq!(renvois.len(), 8);
        for (i, decalage) in renvois.iter().enumerate() {
            assert!(
                pdf[*decalage..].starts_with(format!("{} 0 obj", i + 1).as_bytes()),
                "objet {}",
                i + 1
            );
        }
    }

    #[test]
    fn le_pdf_est_bien_forme() {
        let mut cuisine = rectangle("12", "10 pi 6 po");
        cuisine.retraits.push(Retrait {
            longueur: "3".into(),
            largeur: "6".into(),
        });
        let plan = plan(vec![
            ("Cuisine", cuisine),
            ("Salon", rectangle("20", "15")),
            ("Vide", Dessin::default()),
        ]);
        let pdf = pdf_du_plan(&plan, &Preferences::default(), "21 novembre 2026, 14 h 05");
        assert!(pdf.is_ascii());
        assert!(pdf.starts_with("%PDF-1.4\n") && pdf.ends_with("%%EOF\n"));
        // Une page de résumé, puis une par pièce dessinée (la pièce vide n'y est pas).
        assert!(pdf.contains("/Count 3"));
        // Chaque objet est là où la table des renvois le dit.
        let debut: usize = pdf
            .rsplit("startxref\n")
            .next()
            .and_then(|fin| fin.lines().next())
            .and_then(|nombre| nombre.parse().ok())
            .unwrap();
        assert!(pdf[debut..].starts_with("xref\n"));
        let renvois: Vec<usize> = pdf[debut..]
            .lines()
            .skip(3)
            .take_while(|ligne| ligne.ends_with(" n "))
            .map(|ligne| ligne[..10].parse().unwrap())
            .collect();
        assert_eq!(renvois.len(), 5 + 2 * 3);
        for (i, decalage) in renvois.iter().enumerate() {
            assert!(
                pdf[*decalage..].starts_with(&format!("{} 0 obj", i + 1)),
                "objet {}",
                i + 1
            );
        }
        // La longueur annoncée de chaque contenu est la bonne.
        for morceau in pdf.split("<< /Length ").skip(1) {
            let (longueur, reste) = morceau.split_once(" >>\nstream\n").unwrap();
            let longueur: usize = longueur.parse().unwrap();
            assert!(reste[longueur..].starts_with("endstream"));
        }
        // Les textes : accents en WinAnsi, mesures en pieds-pouces.
        assert!(pdf.contains("(Cuisine)"));
        assert!(pdf.contains("(10 pi 6 po)"));
        assert!(pdf.contains("(P\\351rim\\350tre)"));
        assert!(pdf.contains("(126 pi\\262 \\(11,7 m\\262\\))"));
        assert!(pdf.contains("(108 pi\\262 \\(10,0 m\\262\\))"));
        // Le total du résumé : 108 + 300 = 408 pi².
        assert!(pdf.contains("(408 pi\\262 \\(37,9 m\\262\\))"));
        // Une seule pièce : pas de page de résumé.
        let seule = pdf_du_plan(
            &plan_d_une_piece(rectangle("12", "10")),
            &Preferences::default(),
            "",
        );
        assert!(seule.contains("/Count 1"));
    }

    fn plan_d_une_piece(dessin: Dessin) -> Plan {
        Plan::d_une_piece(dessin)
    }

    #[test]
    fn les_portes_et_les_fenetres_sont_dessinees() {
        let mut chambre = rectangle("12", "10");
        chambre.ouvertures = vec![
            noyau::dessin::Ouverture {
                largeur: "2 pi 8 po".into(),
                hauteur: "6 pi 8 po".into(),
                genre: Some(noyau::dessin::GenreDOuverture::Porte),
                cote: Some(0),
                position: String::new(),
                allege: String::new(),
            },
            noyau::dessin::Ouverture {
                largeur: "3".into(),
                hauteur: "4".into(),
                genre: Some(noyau::dessin::GenreDOuverture::Fenetre),
                cote: Some(1),
                position: "2".into(),
                allege: String::new(),
            },
        ];
        let plan = Plan::d_une_piece(chambre);
        let dxf = dxf_du_plan(&plan, &Preferences::default()).unwrap();
        assert!(dxf.contains("\r\nOUVERTURES\r\n"));
        // Les murs interrompus : deux tronçons pour chacun des deux premiers murs.
        let murs = dxf.matches("\r\nLINE\r\n  8\r\nMURS\r\n").count();
        assert_eq!(murs, 6);
        // La porte : son battant et son arc; la fenêtre : trois traits.
        let ouvertures = dxf.matches("\r\nLINE\r\n  8\r\nOUVERTURES\r\n").count();
        assert_eq!(ouvertures, 4);
        assert_eq!(dxf.matches("\r\nARC\r\n").count(), 1);
        // Le battant mesure la largeur de la porte : 32 po.
        assert!(dxf.contains("\r\n 40\r\n32.0000\r\n"));
        // Le PDF interrompt les murs, en blanc.
        let pdf = pdf_du_plan(&plan, &Preferences::default(), "");
        assert!(pdf.contains("1.000 1.000 1.000 RG"));
    }

    #[test]
    fn les_objets_paraissent_dans_le_pdf_et_le_dxf() {
        let mut chambre = rectangle("12", "10");
        chambre.objets = vec![
            noyau::mobilier::Objet::nouveau(noyau::mobilier::GenreDObjet::Lit, (72.0, 60.0)),
            noyau::mobilier::Objet::nouveau(noyau::mobilier::GenreDObjet::Laveuse, (20.0, 20.0)),
        ];
        let plan = Plan::d_une_piece(chambre);
        let dxf = dxf_du_plan(&plan, &Preferences::default()).unwrap();
        assert!(dxf.contains("\r\nMOBILIER\r\n"));
        // Le lit : son contour, ses deux oreillers et le pli du drap; la laveuse : son tambour.
        let lignes = dxf.matches("\r\nLINE\r\n  8\r\nMOBILIER\r\n").count();
        assert_eq!(lignes, 4 + 8 + 1 + 4);
        assert_eq!(dxf.matches("\r\nCIRCLE\r\n").count(), 1);
        assert!(dxf.contains("\r\nLit\r\n"));
        let pdf = pdf_du_plan(&plan, &Preferences::default(), "");
        assert!(pdf.contains("(Lit)") && pdf.contains("(Laveuse)"));
    }

    #[test]
    fn les_lignes_de_cote_dans_le_pdf_et_le_dxf() {
        // Une cloison de 5 po depuis le mur de gauche, à 4 pi du haut.
        let mut chambre = rectangle("12", "10");
        chambre.cloisons = vec![noyau::cloison::Cloison {
            debut: (0.0, 48.0),
            sens: Sens::Est,
            longueur: "4".to_string(),
            epaisseur: "5 po".to_string(),
            porte: None,
            suite: Vec::new(),
        }];
        let lignes_de_cote = |dxf: &str| dxf.matches("\r\nLINE\r\n  8\r\nCOTES\r\n").count();
        // Sans le bouton « Cotes », aucune ligne de cote.
        let sans = dxf_du_plan(&Plan::d_une_piece(chambre.clone()), &Preferences::default());
        assert_eq!(lignes_de_cote(&sans.unwrap()), 0);
        chambre.lignes_de_cote = true;
        let plan = Plan::d_une_piece(chambre);
        // Le mur de gauche : de 0 à 69 1/2 po, la cloison, puis 45 1/2 po; une
        // ligne, quatre traits et quatre lignes d'attache.
        let dxf = dxf_du_plan(&plan, &Preferences::default()).unwrap();
        assert_eq!(lignes_de_cote(&dxf), 9);
        assert!(dxf.contains("\r\n5 pi 9 1/2 po\r\n"), "{dxf}");
        assert!(dxf.contains("\r\n3 pi 9 1/2 po\r\n"));
        let pdf = pdf_du_plan(&plan, &Preferences::default(), "");
        assert!(pdf.contains("(5 pi 9 1/2 po)"));
        assert!(pdf.contains("(3 pi 9 1/2 po)"));
    }

    #[test]
    fn la_feuille_d_un_plan_a_l_echelle() {
        let nom =
            |feuille: Option<Feuille>| feuille.map(|f| (f.papier.traduit().to_string(), f.paysage));
        // 12 × 10 pi et ses murs de 9 po, à 1/4 po = 1 pi : Lettre en paysage.
        assert_eq!(
            nom(feuille_pour((162.0, 138.0), 1.5)),
            Some(("Lettre".to_string(), true))
        );
        // Une pièce en longueur : en portrait d'abord, sur 11 × 17.
        assert_eq!(
            nom(feuille_pour((138.0, 378.0), 1.5)),
            Some(("11 × 17".to_string(), false))
        );
        // Trop grande pour 11 × 17 à 1/4 po; elle tient à 1/8 po.
        assert_eq!(nom(feuille_pour((738.0, 498.0), 1.5)), None);
        assert!(feuille_pour((738.0, 498.0), 0.75).is_some());
    }

    #[test]
    fn le_pdf_a_l_echelle_et_son_cartouche() {
        let mut chambre = rectangle("12", "10");
        chambre.epaisseur_des_murs = "9 po".to_string();
        let mut plan = Plan::d_une_piece(chambre);
        plan.mise_en_page = MiseEnPage {
            echelle: EchelleDuPlan::QuartDePouce,
            projet: "Agrandissement".to_string(),
            adresse: "12 rue des Pins".to_string(),
            dessine_par: "S. L.".to_string(),
        };
        let pdf = pdf_du_plan(&plan, &Preferences::default(), "2026-11-23");
        // Lettre en paysage, le cartouche et l'échelle graphique.
        assert!(pdf.contains("/MediaBox [0 0 792 612]"), "{pdf}");
        for texte in [
            "(Agrandissement)",
            "(12 rue des Pins)",
            "(S. L.)",
            "(2026-11-23)",
            "(1/4 po = 1 pi)",
            "(Lettre, paysage)",
        ] {
            assert!(pdf.contains(texte), "{texte}");
        }
        // Ajusté : Lettre en portrait, comme avant.
        plan.mise_en_page.echelle = EchelleDuPlan::Ajustee;
        let pdf = pdf_du_plan(&plan, &Preferences::default(), "2026-11-23");
        assert!(pdf.contains("/MediaBox [0 0 612 792]"));
        assert!(!pdf.contains("(Agrandissement)"));
        // Une pièce trop grande pour 11 × 17 à 1/4 po : ajustée, et la page le dit.
        let mut grande = Plan::d_une_piece(rectangle("60", "40"));
        grande.mise_en_page.echelle = EchelleDuPlan::QuartDePouce;
        let pdf = pdf_du_plan(&grande, &Preferences::default(), "");
        assert!(pdf.contains("/MediaBox [0 0 612 792]"));
        assert!(pdf.contains("(Le plan ne tient pas "), "{pdf}");
        // L'avis de l'écran, pour la pièce de 12 × 10 pi.
        assert_eq!(
            avis_de_l_echelle(
                &plan.pieces[0].dessin,
                EchelleDuPlan::QuartDePouce,
                &Preferences::default()
            )
            .as_deref(),
            Some("À cette échelle, la pièce tient sur une feuille Lettre, paysage.")
        );
        assert_eq!(
            avis_de_l_echelle(
                &plan.pieces[0].dessin,
                EchelleDuPlan::Ajustee,
                &Preferences::default()
            ),
            None
        );
    }

    #[test]
    fn les_notes_paraissent_dans_le_pdf_et_le_dxf() {
        let mut chambre = rectangle("12", "10");
        chambre.notes = vec![
            noyau::dessin::Note {
                texte: "Prise 220 V ici".to_string(),
                position: (30.0, 40.0),
            },
            noyau::dessin::Note {
                texte: "   ".to_string(),
                position: (60.0, 60.0),
            },
        ];
        let plan = Plan::d_une_piece(chambre);
        let dxf = dxf_du_plan(&plan, &Preferences::default()).unwrap();
        // La note écrite, sur le calque NOTES; la note vide ne s'exporte pas.
        assert_eq!(dxf.matches("\r\nTEXT\r\n  8\r\nNOTES\r\n").count(), 1);
        assert!(dxf.contains("\r\nPrise 220 V ici\r\n"));
        let pdf = pdf_du_plan(&plan, &Preferences::default(), "");
        assert!(pdf.contains("(Prise 220 V ici)"));
    }

    #[test]
    fn les_cloisons_paraissent_dans_le_pdf_et_le_dxf() {
        let mut chambre = rectangle("12", "10");
        chambre.cloisons = vec![noyau::cloison::Cloison {
            debut: (48.0, 0.0),
            sens: noyau::dessin::Sens::Sud,
            longueur: "10".to_string(),
            epaisseur: String::new(),
            porte: None,
            suite: Vec::new(),
        }];
        let plan = Plan::d_une_piece(chambre);
        let dxf = dxf_du_plan(&plan, &Preferences::default()).unwrap();
        assert_eq!(dxf.matches("\r\nLINE\r\n  8\r\nCLOISONS\r\n").count(), 4);
        let pdf = pdf_du_plan(&plan, &Preferences::default(), "");
        assert_eq!(pdf.matches("0.820 0.870 0.950 rg").count(), 1);
    }

    #[test]
    fn les_murs_a_leur_epaisseur_dans_le_pdf_et_le_dxf() {
        let mut chambre = rectangle("12", "10");
        chambre.epaisseur_des_murs = String::new();
        chambre.ouvertures = vec![noyau::dessin::Ouverture {
            largeur: "3 pi".into(),
            hauteur: "4 pi".into(),
            genre: Some(noyau::dessin::GenreDOuverture::Fenetre),
            cote: Some(0),
            position: String::new(),
            allege: String::new(),
        }];
        let plan = Plan::d_une_piece(chambre);
        let dxf = dxf_du_plan(&plan, &Preferences::default()).unwrap();
        // Cinq morceaux de mur (le mur du haut coupé par la fenêtre) : leur
        // intérieur, leur extérieur, et les deux jambages de la fenêtre.
        assert_eq!(
            dxf.matches("\r\nLINE\r\n  8\r\nMURS\r\n").count(),
            5 + 5 + 2
        );
        // Le coin extérieur, à 9 po hors des mesures : la pièce commence à
        // x = 0 par l'extérieur de son mur (le DXF a les y vers le haut).
        assert!(dxf.contains("\r\n 10\r\n0.0000\r\n 20\r\n9.0000\r\n"));
        // La fenêtre traverse le mur : trois traits.
        assert_eq!(dxf.matches("\r\nLINE\r\n  8\r\nOUVERTURES\r\n").count(), 3);
        let pdf = pdf_du_plan(&plan, &Preferences::default(), "");
        assert_eq!(pdf.matches("0.740 0.800 0.900 rg").count(), 5);
        // Les murs pleins n'ont pas besoin d'être interrompus en blanc.
        assert!(!pdf.contains("1.000 1.000 1.000 RG"));
    }

    #[test]
    fn la_porte_d_une_cloison_dans_le_pdf_et_le_dxf() {
        let mut chambre = rectangle("12", "10");
        chambre.cloisons = vec![noyau::cloison::Cloison {
            debut: (48.0, 0.0),
            sens: noyau::dessin::Sens::Sud,
            longueur: "10".to_string(),
            porte: Some(noyau::cloison::PorteDeCloison::default()),
            ..Default::default()
        }];
        let plan = Plan::d_une_piece(chambre);
        let dxf = dxf_du_plan(&plan, &Preferences::default()).unwrap();
        // Deux morceaux de cloison (quatre traits chacun), et le battant.
        assert_eq!(
            dxf.matches("\r\nLINE\r\n  8\r\nCLOISONS\r\n").count(),
            2 * 4 + 1
        );
        assert_eq!(dxf.matches("\r\nARC\r\n  8\r\nCLOISONS\r\n").count(), 1);
        // Le battant mesure 2 pi 8 po.
        assert!(dxf.contains("\r\n 40\r\n32.0000\r\n"));
        let pdf = pdf_du_plan(&plan, &Preferences::default(), "");
        assert_eq!(pdf.matches("0.820 0.870 0.950 rg").count(), 2);
    }

    #[test]
    fn des_murs_vers_l_interieur_dans_le_dxf() {
        let mut chambre = rectangle("12", "10");
        chambre.epaisseur_des_murs = String::new();
        chambre.mesures_exterieures = true;
        let plan = Plan::d_une_piece(chambre);
        let dxf = dxf_du_plan(&plan, &Preferences::default()).unwrap();
        // Quatre morceaux de mur : leur intérieur et leur extérieur.
        assert_eq!(dxf.matches("\r\nLINE\r\n  8\r\nMURS\r\n").count(), 8);
        // L'intérieur du premier mur commence à 9 po dans la pièce (le DXF a
        // les y vers le haut).
        assert!(dxf.contains("\r\n 10\r\n9.0000\r\n 20\r\n-9.0000\r\n"));
    }

    #[test]
    fn une_cloison_en_l_dans_le_dxf() {
        let mut chambre = rectangle("12", "10");
        chambre.cloisons = vec![noyau::cloison::Cloison {
            debut: (0.0, 50.0),
            sens: noyau::dessin::Sens::Est,
            longueur: "5".to_string(),
            suite: vec![noyau::cloison::SegmentDeCloison {
                sens: noyau::dessin::Sens::Sud,
                longueur: "70 po".to_string(),
            }],
            ..Default::default()
        }];
        let plan = Plan::d_une_piece(chambre);
        let dxf = dxf_du_plan(&plan, &Preferences::default()).unwrap();
        // Deux morceaux : leurs deux faces et le bout libre de chacun; le coin
        // d'onglet ne se trace pas.
        assert_eq!(
            dxf.matches("\r\nLINE\r\n  8\r\nCLOISONS\r\n").count(),
            2 * 3
        );
    }

    #[test]
    fn le_dxf_est_a_l_echelle() {
        let esquisse = Dessin {
            fermee: false,
            ..rectangle("", "")
        };
        let plan = plan(vec![
            ("Cuisine", rectangle("12", "10 pi 6 po")),
            ("Esquisse", esquisse.clone()),
        ]);
        let dxf = dxf_du_plan(&plan, &Preferences::default()).unwrap();
        assert!(dxf.is_ascii());
        assert!(dxf.contains("AC1009"));
        assert!(dxf.trim_end().ends_with("EOF"));
        // Quatre murs pour le rectangle; la pièce incomplète n'est pas exportée.
        assert_eq!(dxf.matches("\r\nLINE\r\n").count(), 4);
        // Les coordonnées sont en pouces : 12 pi = 144 po, 10 pi 6 po = 126 po.
        assert!(dxf.contains("\r\n144.0000\r\n"));
        assert!(dxf.contains("\r\n-126.0000\r\n"));
        // Le nom et la surface; l'exposant s'écrit en \U+XXXX.
        assert!(dxf.contains("Cuisine : 126 pi\\U+00B2 (11,7 m\\U+00B2)"));
        // En métrique, l'unité est le millimètre : 4 m = 4000 mm.
        let metrique = dxf_du_plan(
            &plan_d_une_piece(rectangle("4 m", "3 m")),
            &Preferences::default(),
        )
        .unwrap();
        assert!(metrique.contains("\r\n4000.0000\r\n"));
        // Sans pièce complète, pas de DXF.
        assert!(dxf_du_plan(&plan_d_une_piece(esquisse), &Preferences::default()).is_none());
    }
}
