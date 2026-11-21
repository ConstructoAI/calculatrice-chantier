//! Export du plan dessiné (section 25) : en PDF, pour l'imprimer ou le
//! transmettre, et en DXF, pour l'ouvrir dans un logiciel de dessin (AutoCAD,
//! LibreCAD, QCAD…). Les deux fichiers sont écrits ici, sans bibliothèque
//! (R-77), en texte ASCII : le PDF code ses accents en WinAnsi, le DXF en
//! `\U+XXXX`.

use noyau::dessin::{Dessin, Etat as EtatDeLaForme, Mesure, Plan};
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
    }
}

// ---------------------------------------------------------------------- PDF

/// Format Lettre, en points.
const LARGEUR_PAGE: f64 = 612.0;
const HAUTEUR_PAGE: f64 = 792.0;
const MARGE: f64 = 54.0;

type Couleur = (f64, f64, f64);
const NOIR: Couleur = (0.10, 0.12, 0.16);
const GRIS: Couleur = (0.45, 0.48, 0.52);
const BLEU: Couleur = (0.0, 0.40, 0.80);
const ORANGE: Couleur = (0.85, 0.38, 0.05);
const FILET: Couleur = (0.80, 0.83, 0.87);

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
    page.texte((MARGE, 742.0), 18.0, true, Alignement::Gauche, NOIR, titre);
    page.texte((MARGE, 724.0), 9.5, false, Alignement::Gauche, GRIS, entete);
    page.trait_(&[(MARGE, 714.0), (LARGEUR_PAGE - MARGE, 714.0)], 0.8, FILET);
}

fn pied_de_page(page: &mut Page, numero: usize, total: usize) {
    page.trait_(&[(MARGE, 50.0), (LARGEUR_PAGE - MARGE, 50.0)], 0.6, FILET);
    page.texte(
        (MARGE, 36.0),
        8.0,
        false,
        Alignement::Gauche,
        GRIS,
        textes::DESSIN_EXPORT_PIED.traduit(),
    );
    page.texte(
        (LARGEUR_PAGE - MARGE, 36.0),
        8.0,
        false,
        Alignement::Droite,
        GRIS,
        &textes::dessin_export_page(numero, total),
    );
}

/// La page d'une pièce : sa forme et ses mesures, ses résultats, ses côtés.
fn page_de_piece(piece: &PieceExportee, entete: &str, numero: usize, total: usize) -> Page {
    let mut page = Page::default();
    en_tete_de_page(&mut page, &piece.nom, entete);

    // La forme, ajustée à son cadre; la place des mesures autour.
    let cadre = (MARGE + 84.0, 372.0, LARGEUR_PAGE - MARGE - 84.0, 684.0);
    let vers_la_page = ajuster(&piece.sommets, cadre);
    let points: Vec<(f64, f64)> = piece.sommets.iter().map(|s| vers_la_page(*s)).collect();
    page.trait_(&points, 1.8, BLEU);
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
    if !piece.a_l_echelle {
        page.texte(
            (MARGE, 350.0),
            9.5,
            false,
            Alignement::Gauche,
            ORANGE,
            textes::DESSIN_EXPORT_CROQUIS.traduit(),
        );
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
        let contenu = decalages.len() + 2;
        ajouter_objet(
            &mut sortie,
            &mut decalages,
            &format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {LARGEUR_PAGE} {HAUTEUR_PAGE}] /Resources << /Font << /F1 3 0 R /F2 4 0 R >> >> /Contents {contenu} 0 R >>"
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
    for piece in &pieces {
        let numero = pages.len() + 1;
        pages.push(page_de_piece(piece, &entete, numero, total));
    }
    assembler_pdf(&pages, &textes::dessin_export_plan(pieces.len()))
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
        let gauche = points.iter().map(|q| q.0).fold(f64::MAX, f64::min);
        let droite = points.iter().map(|q| q.0).fold(f64::MIN, f64::max);
        let bas = points.iter().map(|q| q.1).fold(f64::MAX, f64::min);
        let haut = points.iter().map(|q| q.1).fold(f64::MIN, f64::max);
        let decalage = depart - gauche;
        let points: Vec<(f64, f64)> = points.iter().map(|&(x, y)| (x + decalage, y)).collect();
        let (largeur, hauteur) = (droite - gauche, haut - bas);
        let taille = (largeur.max(hauteur) * 0.025).clamp(3.0 * facteur, 12.0 * facteur);
        for segment in points.windows(2) {
            ligne_dxf(&mut entites, "MURS", segment[0], segment[1]);
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
    let calques = [("0", "7"), ("MURS", "5"), ("COTES", "7"), ("PIECES", "3")];
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
