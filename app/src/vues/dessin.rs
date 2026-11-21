//! Écran « Dessin » (section 25) : on trace le contour d'une pièce avec le
//! doigt ou la souris, puis on touche chaque côté pour y inscrire sa mesure.

use eframe::egui::{self, Align2, Color32, Pos2, Rect, RichText, Sense, Stroke, Ui, Vec2};
use noyau::cloison::{
    Cloison, CloisonMesuree, EPAISSEUR_PAR_DEFAUT, LARGEUR_DE_PORTE_PAR_DEFAUT, PorteDeCloison,
    Segment, redresser,
};
use noyau::dessin::{
    Dessin, Etat as EtatDeLaForme, GenreDOuverture, Mesure, Ouverture, Plan, Retrait, Sens,
    Solution,
};
use noyau::dessin::{EPAISSEUR_DES_MURS_PAR_DEFAUT, MorceauDeMur};
use noyau::metiers::Quantite;
use noyau::mobilier::{GenreDObjet, Objet, TraitSurLePlan};
use noyau::prefs::Preferences;
use noyau::unites::Format;

use super::{bouton_principal, cadre_de_saisie, carte, corps, en_tete};
use crate::textes;
use crate::theme::{self, Palette};

/// Marges entre le bord de la planche et la forme : la place des mesures, plus
/// large sur les côtés, où s'écrivent celles des côtés verticaux.
const MARGE_LATERALE: f32 = 64.0;
const MARGE_VERTICALE: f32 = 34.0;
/// Distance, en points, sous laquelle un toucher saisit une porte ou une fenêtre.
const PORTEE_D_UNE_OUVERTURE: f32 = 16.0;
/// Distance, en points, sous laquelle un toucher choisit un côté.
const PORTEE_DU_TOUCHER: f32 = 30.0;
/// Format d'un nombre inscrit sans unité : des pieds.
pub const FORMAT_PAR_DEFAUT: Format = Format::PiPo;

/// Zoom le plus faible et le plus fort de la planche (1 : la forme tient juste).
const ZOOM_MIN: f32 = 0.5;
const ZOOM_MAX: f32 = 8.0;

/// Le cadrage de la planche : le zoom, autour du centre de la planche, puis un
/// décalage en points de l'écran.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Cadrage {
    zoom: f32,
    decalage: Vec2,
}

impl Default for Cadrage {
    fn default() -> Self {
        Cadrage {
            zoom: 1.0,
            decalage: Vec2::ZERO,
        }
    }
}

impl Cadrage {
    /// Zoome d'un facteur autour d'un point de l'écran, qui reste en place.
    fn zoomer(&mut self, centre: Pos2, autour: Pos2, facteur: f32) {
        if !facteur.is_finite() || facteur <= 0.0 {
            return;
        }
        let zoom = (self.zoom * facteur).clamp(ZOOM_MIN, ZOOM_MAX);
        let k = zoom / self.zoom;
        self.decalage = (autour - centre) * (1.0 - k) + self.decalage * k;
        self.zoom = zoom;
    }

    /// Applique le cadrage à une échelle qui fait tenir la forme dans la planche.
    fn appliquer(self, echelle: Echelle, centre: Pos2) -> Echelle {
        Echelle {
            origine: centre + (echelle.origine - centre) * self.zoom + self.decalage,
            facteur: echelle.facteur * self.zoom,
        }
    }
}

/// Ce que l'écran retient d'une image à l'autre.
#[derive(Default)]
pub struct Etat {
    /// Le côté dont on inscrit la mesure.
    pub choisi: Option<usize>,
    /// Le tracé en cours, en points de l'écran.
    trace: Vec<Pos2>,
    /// L'autre mesure qu'on inscrit : une dimension d'un retrait, la hauteur
    /// des murs, ou une dimension d'une porte ou d'une fenêtre.
    autre_choisi: Option<AutreSaisie>,
    /// Vrai quand le champ de la mesure doit prendre le clavier.
    viser_le_champ: bool,
    /// Vrai quand la liste des outils qui peuvent recevoir la pièce est ouverte.
    outils_ouverts: bool,
    /// Les états précédents du dessin, pour le bouton « Annuler ».
    historique: Vec<Dessin>,
    /// Le zoom et le décalage de la planche.
    cadrage: Cadrage,
    /// Vrai quand les flèches dessinent en biais, à 45°.
    fleches_en_biais: bool,
    /// L'ouverture qui attend qu'on touche son mur pour s'y placer.
    ouverture_a_placer: Option<usize>,
    /// La porte ou la fenêtre que le doigt fait glisser le long de son mur :
    /// son rang, et la distance (pouces) entre le point saisi et son bord.
    ouverture_tiree: Option<(usize, f64)>,
    /// Où sont les ouvertures à l'écran, d'une image à la suivante : leur rang
    /// et leurs deux bords.
    ouvertures_a_l_ecran: Vec<(usize, Pos2, Pos2)>,
    /// L'objet qui attend qu'on touche la pièce pour s'y poser.
    objet_a_placer: Option<GenreDObjet>,
    /// L'objet choisi : il paraît en couleur.
    objet_choisi: Option<usize>,
    /// L'objet que le doigt déplace : son rang, et l'écart (pouces) entre le
    /// point saisi et son centre.
    objet_tire: Option<(usize, (f64, f64))>,
    /// Le passage du plan à l'écran à l'image précédente, pour saisir un objet.
    derniere_echelle: Option<Echelle>,
    /// Vrai quand une note attend qu'on touche le plan pour s'y poser.
    note_a_placer: bool,
    /// La note choisie : son cadre paraît en couleur.
    note_choisie: Option<usize>,
    /// La note que le doigt déplace : son rang, et l'écart (pouces) entre le
    /// point saisi et sa place.
    note_tiree: Option<(usize, (f64, f64))>,
    /// Où sont les notes à l'écran, d'une image à la suivante.
    notes_a_l_ecran: Vec<(usize, Rect)>,
    /// La note dont le champ doit prendre le clavier.
    viser_la_note: Option<usize>,
    /// La note qu'on écrit d'affilée : elle a déjà son étape d'« Annuler ».
    note_ecrite: Option<usize>,
    /// Vrai quand le prochain glissement sur le plan trace une cloison.
    cloison_a_tracer: bool,
    /// La cloison qu'on trace : son départ et le point atteint (pouces).
    cloison_en_trace: Option<Segment>,
    /// La cloison choisie.
    cloison_choisie: Option<usize>,
    /// La cloison que le doigt déplace : son rang, et l'écart (pouces) entre
    /// le point saisi et son départ.
    cloison_tiree: Option<(usize, (f64, f64))>,
    /// Vrai quand le nom de la pièce s'écrit sous la rangée des pièces.
    renommer: bool,
    /// Vrai quand ce champ doit prendre le clavier.
    viser_le_nom: bool,
    /// Vrai quand la planche montre la pièce en 3D.
    vue_3d: bool,
    /// La caméra de la vue 3D.
    orbite: super::maquette::Orbite,
    /// Vrai quand un geste à deux doigts a interrompu le geste en cours : rien
    /// n'est tracé ni déplacé jusqu'à ce que les doigts se lèvent.
    geste_annule: bool,
}

/// Une mesure d'une cloison ou de sa porte.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ChampDeCloison {
    Longueur,
    Epaisseur,
    LargeurDePorte,
    PositionDePorte,
}

/// Ce que demande un bouton de la ligne d'une cloison.
#[derive(Clone, Copy)]
enum GesteSurUneCloison {
    Enlever,
    AjouterUnePorte,
    Inverser,
    SansPorte,
}

/// Une mesure autre que celle d'un côté.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AutreSaisie {
    /// Une dimension d'un retrait : son rang, et vrai pour sa largeur.
    Retrait(usize, bool),
    /// La hauteur des murs.
    Hauteur,
    /// Une mesure d'une ouverture : son rang, et laquelle.
    Ouverture(usize, ChampDOuverture),
    /// Une dimension d'un objet : son rang, et vrai pour sa profondeur.
    Objet(usize, bool),
    /// Une mesure d'une cloison ou de sa porte : son rang, et laquelle.
    Cloison(usize, ChampDeCloison),
    /// L'épaisseur des murs extérieurs de la pièce.
    EpaisseurDesMurs,
    /// L'épaisseur des cloisons de la pièce.
    EpaisseurDesCloisons,
}

/// Une mesure d'une porte ou d'une fenêtre.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ChampDOuverture {
    Largeur,
    Hauteur,
    /// La hauteur du bas d'une fenêtre (son allège).
    Allege,
    /// La distance au premier coin de son mur.
    Position,
}

impl AutreSaisie {
    /// Vrai si la mesure existe encore dans le dessin.
    fn existe(self, dessin: &Dessin) -> bool {
        match self {
            AutreSaisie::Retrait(rang, _) => rang < dessin.retraits.len(),
            AutreSaisie::Hauteur => true,
            AutreSaisie::Ouverture(rang, _) => rang < dessin.ouvertures.len(),
            AutreSaisie::Objet(rang, _) => rang < dessin.objets.len(),
            AutreSaisie::Cloison(rang, champ) => dessin.cloisons.get(rang).is_some_and(|cloison| {
                matches!(champ, ChampDeCloison::Longueur | ChampDeCloison::Epaisseur)
                    || cloison.porte.is_some()
            }),
            AutreSaisie::EpaisseurDesMurs | AutreSaisie::EpaisseurDesCloisons => true,
        }
    }
}

/// Nombre d'étapes que le bouton « Annuler » peut remonter.
const PROFONDEUR_DE_L_HISTORIQUE: usize = 40;

impl Etat {
    /// Choisit un côté et place le curseur dans le champ de sa mesure.
    fn choisir(&mut self, rang: Option<usize>) {
        self.choisi = rang;
        self.autre_choisi = None;
        self.viser_le_champ = rang.is_some();
    }

    /// Une autre pièce vient à l'écran : rien n'y est choisi, et « Annuler »
    /// ne remonte pas dans la pièce précédente.
    fn changer_de_piece(&mut self) {
        self.choisi = None;
        self.autre_choisi = None;
        self.viser_le_champ = false;
        self.outils_ouverts = false;
        self.trace.clear();
        self.historique.clear();
        self.cadrage = Cadrage::default();
        self.geste_annule = false;
        self.ouverture_a_placer = None;
        self.ouverture_tiree = None;
        self.ouvertures_a_l_ecran.clear();
        self.vue_3d = false;
        self.renommer = false;
        self.viser_le_nom = false;
        self.orbite = super::maquette::Orbite::default();
        self.objet_a_placer = None;
        self.objet_choisi = None;
        self.objet_tire = None;
        self.derniere_echelle = None;
        self.note_a_placer = false;
        self.note_choisie = None;
        self.note_tiree = None;
        self.notes_a_l_ecran.clear();
        self.viser_la_note = None;
        self.note_ecrite = None;
        self.cloison_a_tracer = false;
        self.cloison_en_trace = None;
        self.cloison_choisie = None;
        self.cloison_tiree = None;
    }

    /// Choisit une autre mesure : un retrait, la hauteur des murs, une ouverture.
    fn choisir_autre(&mut self, lequel: Option<AutreSaisie>) {
        self.choisi = None;
        self.autre_choisi = lequel;
        self.viser_le_champ = lequel.is_some();
    }

    /// Retient le dessin tel qu'il est, avant un changement.
    fn memoriser(&mut self, dessin: &Dessin) {
        if self.historique.last() != Some(dessin) {
            self.historique.push(dessin.clone());
            if self.historique.len() > PROFONDEUR_DE_L_HISTORIQUE {
                self.historique.remove(0);
            }
        }
    }
}

/// Format d'export du plan.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormatDExport {
    Pdf,
    Dxf,
}

pub enum Action {
    /// Le dessin ou une mesure ont changé : à conserver.
    Modifie,
    Fermer,
    /// Inscrire la pièce au ruban : ses mesures, puis ses résultats.
    Consigner {
        titre: String,
        entrees: Vec<(String, String)>,
        resultats: Vec<(String, String)>,
    },
    /// Exporter le plan (toutes les pièces) en PDF ou en DXF.
    Exporter(FormatDExport),
    /// Ouvrir un outil de métier avec la longueur et la largeur de la pièce.
    VersOutil {
        id: &'static str,
        /// (rang du champ, ce qu'il faut y inscrire)
        champs: Vec<(usize, String)>,
    },
}

/// Passage du croquis (ou de la forme à ses mesures) à l'écran.
#[derive(Clone, Copy)]
struct Echelle {
    origine: Pos2,
    facteur: f32,
}

impl Echelle {
    fn vers_l_ecran(&self, (x, y): (f64, f64)) -> Pos2 {
        Pos2::new(
            self.origine.x + x as f32 * self.facteur,
            self.origine.y + y as f32 * self.facteur,
        )
    }

    /// Le point du plan (pouces) sous un point de l'écran.
    fn vers_le_plan(&self, point: Pos2) -> (f64, f64) {
        (
            f64::from((point.x - self.origine.x) / self.facteur),
            f64::from((point.y - self.origine.y) / self.facteur),
        )
    }
}

/// La cloison sous un point du plan (pouces), à `marge` près.
fn cloison_sous(solution: &Solution, point: (f64, f64), marge: f64) -> Option<usize> {
    solution
        .cloisons
        .iter()
        .enumerate()
        .rev()
        .find_map(|(rang, cloison)| match cloison {
            Some(c) if c.distance(point) <= c.epaisseur / 2.0 + marge => Some(rang),
            _ => None,
        })
}

/// Où s'arrête une cloison qu'on trace : les murs de la pièce et les autres
/// cloisons.
fn appuis_des_cloisons(sommets: &[(f64, f64)], solution: &Solution) -> Vec<Segment> {
    let mut appuis: Vec<_> = sommets.windows(2).map(|s| (s[0], s[1])).collect();
    appuis.extend(solution.cloisons.iter().flatten().map(|c| (c.debut, c.fin)));
    appuis
}

/// Le pas des déplacements sur le plan : le pouce, ou le centimètre en métrique.
fn pas_du_plan(solution: &Solution) -> f64 {
    if solution.modele.and_then(Format::systeme) == Some(noyau::unites::Systeme::Metrique) {
        1.0 / 2.54
    } else {
        1.0
    }
}

/// La distance (pouces) sous laquelle le bout d'une cloison va au mur : une
/// vingtaine de points à l'écran, quel que soit le zoom.
fn aimant_des_cloisons(echelle: &Echelle) -> f64 {
    f64::from(18.0 / echelle.facteur).max(3.0)
}

/// Une cloison sur le plan : pleine, avec sa longueur à côté, sa porte (le
/// battant et son arc) et, choisie, son bout A, d'où se mesure la porte.
fn dessiner_une_cloison(
    painter: &egui::Painter,
    p: &Palette,
    echelle: &Echelle,
    cloison: &CloisonMesuree,
    choisie: bool,
    longueur: String,
) {
    let encre = if choisie { p.barre_accent } else { p.primaire };
    for morceau in cloison.morceaux() {
        let coins: Vec<Pos2> = morceau
            .iter()
            .map(|coin| echelle.vers_l_ecran(*coin))
            .collect();
        painter.add(egui::Shape::convex_polygon(
            coins,
            encre.gamma_multiply(0.45),
            Stroke::new(1.4, encre),
        ));
    }
    if let Some(porte) = cloison.porte {
        let q0 = echelle.vers_l_ecran(cloison.point(porte.debut));
        let q1 = echelle.vers_l_ecran(cloison.point(porte.fin));
        let (nx, ny) = cloison.cote_de_la_porte();
        let cote = Vec2::new(nx as f32, ny as f32);
        dessiner_ouverture(painter, (q0, q1), cote, true, encre, encre, 0.0);
    }
    let (a, b) = (
        echelle.vers_l_ecran(cloison.debut),
        echelle.vers_l_ecran(cloison.fin),
    );
    let le_long = (b - a).normalized();
    // La longueur s'écrit au-dessus d'une cloison horizontale, à droite d'une
    // cloison verticale : deux cloisons qui se touchent ne mêlent pas leurs
    // longueurs.
    let normale = Vec2::new(-le_long.y, le_long.x);
    let a_cote = if normale.x - normale.y >= 0.0 {
        normale
    } else {
        -normale
    };
    let ecart = cloison.epaisseur as f32 * echelle.facteur / 2.0 + 10.0;
    let galere = painter.layout_no_wrap(longueur, theme::police_grasse(11.0), encre);
    let cadre = Rect::from_center_size(a + (b - a) * 0.5 + a_cote * ecart, galere.size())
        .expand2(Vec2::new(2.0, 1.0));
    painter.rect_filled(cadre, 3, p.carte.gamma_multiply(0.9));
    painter.galley(cadre.min + Vec2::new(2.0, 1.0), galere, encre);
    if choisie {
        // Le bout A, d'où se mesure la porte.
        painter.circle_stroke(a, 6.5, Stroke::new(1.6, p.barre_accent));
        painter.text(
            a - le_long * 15.0,
            Align2::CENTER_CENTER,
            "A",
            theme::police_grasse(12.0),
            p.barre_accent,
        );
    }
}

/// Un morceau d'un mur extérieur : plein, avec sa face extérieure et ses
/// jambages; en couleur s'il est du côté choisi.
fn dessiner_un_morceau_de_mur(
    painter: &egui::Painter,
    echelle: &Echelle,
    morceau: &MorceauDeMur,
    choisi: Option<usize>,
    p: &Palette,
) {
    let coins: Vec<Pos2> = morceau
        .coins
        .iter()
        .map(|coin| echelle.vers_l_ecran(*coin))
        .collect();
    let encre = if choisi == Some(morceau.cote) {
        p.barre_accent
    } else {
        p.primaire
    };
    painter.add(egui::Shape::convex_polygon(
        coins.clone(),
        encre.gamma_multiply(0.28),
        Stroke::NONE,
    ));
    let trait_ = Stroke::new(1.3, encre);
    painter.line_segment([coins[3], coins[2]], trait_);
    if morceau.jambage_au_debut {
        painter.line_segment([coins[0], coins[3]], trait_);
    }
    if morceau.jambage_a_la_fin {
        painter.line_segment([coins[1], coins[2]], trait_);
    }
}

/// La note sous un point de l'écran (la dernière écrite, si elles se
/// chevauchent), d'après leurs cadres de l'image précédente.
fn note_sous(etat: &Etat, point: Pos2) -> Option<usize> {
    etat.notes_a_l_ecran
        .iter()
        .rev()
        .find(|(_, cadre)| cadre.expand(4.0).contains(point))
        .map(|(rang, _)| *rang)
}

/// L'objet sous un point du plan (le plus haut dans la liste, s'ils se
/// chevauchent).
fn objet_sous(dessin: &Dessin, solution: &Solution, point: (f64, f64)) -> Option<usize> {
    dessin
        .objets
        .iter()
        .zip(&solution.objets)
        .rposition(|(objet, dimensions)| objet.contient(point, *dimensions))
}

/// Cadre la forme dans la planche. Sans forme : un point par unité, depuis le coin.
fn cadrer(planche: Rect, sommets: &[(f64, f64)]) -> Echelle {
    let (mut x0, mut x1, mut y0, mut y1) = (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
    for &(x, y) in sommets {
        x0 = x0.min(x);
        x1 = x1.max(x);
        y0 = y0.min(y);
        y1 = y1.max(y);
    }
    if sommets.len() < 2 {
        return Echelle {
            origine: planche.min,
            facteur: 1.0,
        };
    }
    let (largeur, hauteur) = ((x1 - x0) as f32, (y1 - y0) as f32);
    let utile = planche.shrink2(Vec2::new(MARGE_LATERALE, MARGE_VERTICALE));
    let facteur = (utile.width() / largeur.max(1e-6))
        .min(utile.height() / hauteur.max(1e-6))
        .max(1e-6);
    let centre = planche.center();
    Echelle {
        origine: Pos2::new(
            centre.x - ((x0 + x1) / 2.0) as f32 * facteur,
            centre.y - ((y0 + y1) / 2.0) as f32 * facteur,
        ),
        facteur,
    }
}

/// Où écrire la mesure d'un côté : la direction de l'extérieur de la forme, et
/// l'ancrage du texte — au-dessus ou au-dessous d'un côté horizontal, à côté
/// d'un côté vertical, en coin d'un côté en biais.
fn place_de_la_mesure(a: Pos2, b: Pos2, centre: Pos2) -> (Vec2, Align2) {
    let le_long = (b - a).normalized();
    let mut dehors = Vec2::new(-le_long.y, le_long.x);
    let milieu = a + (b - a) * 0.5;
    if (milieu - centre).dot(dehors) < 0.0 {
        dehors = -dehors;
    }
    let ancre = if dehors.y.abs() > 0.9 {
        if dehors.y > 0.0 {
            Align2::CENTER_TOP
        } else {
            Align2::CENTER_BOTTOM
        }
    } else if dehors.x.abs() > 0.9 {
        if dehors.x > 0.0 {
            Align2::LEFT_CENTER
        } else {
            Align2::RIGHT_CENTER
        }
    } else {
        match (dehors.x > 0.0, dehors.y > 0.0) {
            (true, true) => Align2::LEFT_TOP,
            (true, false) => Align2::LEFT_BOTTOM,
            (false, true) => Align2::RIGHT_TOP,
            (false, false) => Align2::RIGHT_BOTTOM,
        }
    };
    (dehors, ancre)
}

/// Distance d'un point à un segment.
fn distance_au_segment(p: Pos2, a: Pos2, b: Pos2) -> f32 {
    let ab = b - a;
    let longueur2 = ab.length_sq();
    if longueur2 <= f32::EPSILON {
        return p.distance(a);
    }
    let t = ((p - a).dot(ab) / longueur2).clamp(0.0, 1.0);
    p.distance(a + ab * t)
}

/// Texte d'une longueur en pouces, dans le système des mesures inscrites.
pub(crate) fn longueur_en_clair(pouces: f64, solution: &Solution, prefs: &Preferences) -> String {
    Quantite::Longueur(pouces).en_clair(solution.modele, prefs)
}

/// Une mesure telle qu'elle a été inscrite, remise au propre dans son système :
/// « 20 » devient « 20 pi 0 po », « 3,81 m » devient « 3,810 m ». `None` si
/// elle est illisible.
pub(crate) fn saisie_en_clair(saisie: &str, prefs: &Preferences) -> Option<String> {
    let valeur = noyau::analyse::analyser_longueur(saisie.trim(), FORMAT_PAR_DEFAUT)
        .ok()
        .filter(|v| v.est_longueur() && v.base > 0.0 && v.base.is_finite())?;
    Some(Quantite::Longueur(valeur.base).en_clair(Some(valeur.format), prefs))
}

/// Premier côté sans mesure (ni inscrite, ni déduite), s'il y en a un.
fn premier_cote_a_mesurer(solution: &Solution) -> Option<usize> {
    solution
        .mesures
        .iter()
        .position(|m| matches!(m, Mesure::Manquante | Mesure::Invalide))
}

/// Nom d'une pièce : celui que l'utilisateur a donné, sinon « Pièce 2 ».
pub(crate) fn nom_de_la_piece(plan: &Plan, rang: usize) -> String {
    match plan.pieces.get(rang).map(|piece| piece.nom.trim()) {
        Some(nom) if !nom.is_empty() => nom.to_string(),
        _ => textes::dessin_piece(rang + 1),
    }
}

pub fn afficher(
    ui: &mut Ui,
    p: &Palette,
    plan: &mut Plan,
    etat: &mut Etat,
    prefs: &Preferences,
) -> Option<Action> {
    let mut action = None;
    if en_tete(
        ui,
        p,
        textes::DESSIN_TITRE.traduit(),
        textes::FERMER.traduit(),
    ) {
        action = Some(Action::Fermer);
    }
    plan.normaliser();
    if barre_des_pieces(ui, p, plan, etat) {
        action = Some(Action::Modifie);
    }
    let courante = plan.courante;
    let titre = textes::avec_tiret(
        textes::DESSIN_TITRE.traduit(),
        &nom_de_la_piece(plan, courante),
    );
    {
        let dessin = &mut plan.pieces[courante].dessin;
        if etat.choisi.is_some_and(|i| i >= dessin.cotes.len()) {
            etat.choisi = None;
        }
        if etat.autre_choisi.is_some_and(|autre| !autre.existe(dessin)) {
            etat.autre_choisi = None;
        }
        let solution = dessin.resoudre(FORMAT_PAR_DEFAUT);

        // La planche à dessin : hors de la zone qui défile, pour que le doigt
        // trace au lieu de faire défiler l'écran.
        let largeur = ui.available_width() - 20.0;
        // Pendant qu'on inscrit une mesure, la planche rapetisse pour laisser
        // voir le pavé de touches en entier, même sur un téléphone.
        let reserve = if etat.choisi.is_some() || etat.autre_choisi.is_some() {
            410.0
        } else {
            250.0
        };
        let hauteur = (ui.available_height() - reserve).clamp(150.0, largeur.max(150.0));
        ui.add_space(6.0);
        let (planche, reponse) = ui
            .horizontal(|ui| {
                ui.add_space(10.0);
                ui.allocate_exact_size(Vec2::new(largeur, hauteur), Sense::click_and_drag())
            })
            .inner;
        reponse.widget_info(|| {
            egui::WidgetInfo::labeled(egui::Role::Canvas, true, textes::DESSIN_PLANCHE.traduit())
        });
        if planche_et_gestes(ui, p, planche, &reponse, dessin, etat, &solution, prefs) {
            action = Some(Action::Modifie);
        }
    }

    corps(ui, "dessin", |ui| {
        {
            let dessin = &mut plan.pieces[courante].dessin;
            // Les mesures peuvent avoir changé pendant cette image.
            let solution = dessin.resoudre(FORMAT_PAR_DEFAUT);
            if boutons(ui, dessin, etat) {
                action = Some(Action::Modifie);
            }
            ui.add_space(6.0);
            if mesure_du_cote(ui, p, dessin, etat, &solution, prefs) {
                action = Some(Action::Modifie);
            }
            let solution = dessin.resoudre(FORMAT_PAR_DEFAUT);
            match bilan(ui, p, dessin, etat, &solution, prefs) {
                Some(Action::Consigner {
                    entrees, resultats, ..
                }) => {
                    action = Some(Action::Consigner {
                        titre: titre.clone(),
                        entrees,
                        resultats,
                    });
                }
                Some(autre) => action = Some(autre),
                None => {}
            }
            if dessin.fermee && retraits(ui, p, dessin, etat, &solution, prefs) {
                action = Some(Action::Modifie);
            }
            if dessin.fermee && murs(ui, p, dessin, etat, &solution, prefs) {
                action = Some(Action::Modifie);
            }
            if matches!(solution.etat, EtatDeLaForme::Complete { .. })
                && cloisons(ui, p, dessin, etat, &solution, prefs)
            {
                action = Some(Action::Modifie);
            }
            if matches!(solution.etat, EtatDeLaForme::Complete { .. })
                && objets(ui, p, dessin, etat, &solution, prefs)
            {
                action = Some(Action::Modifie);
            }
            if matches!(solution.etat, EtatDeLaForme::Complete { .. }) && notes(ui, p, dessin, etat)
            {
                action = Some(Action::Modifie);
            }
        }
        let (modifie, export) = la_piece_et_le_total(ui, p, plan, etat, prefs);
        if modifie {
            action = Some(Action::Modifie);
        }
        if let Some(format) = export {
            action = Some(Action::Exporter(format));
        }
        ui.add_space(10.0);
    });
    action
}

/// La rangée des pièces, sous l'en-tête : une pastille par pièce, et « + Pièce ».
/// Retourne vrai si le plan a changé.
fn barre_des_pieces(ui: &mut Ui, p: &Palette, plan: &mut Plan, etat: &mut Etat) -> bool {
    let mut modifie = false;
    ui.add_space(6.0);
    egui::ScrollArea::horizontal()
        .id_salt("pieces-du-plan")
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(10.0);
                for rang in 0..plan.pieces.len() {
                    let bouton = egui::Button::new(nom_de_la_piece(plan, rang))
                        .selected(rang == plan.courante);
                    if ui.add(bouton).clicked() {
                        if rang == plan.courante {
                            // Toucher la pièce choisie ouvre son nom.
                            etat.renommer = !etat.renommer;
                            etat.viser_le_nom = etat.renommer;
                        } else {
                            plan.courante = rang;
                            etat.changer_de_piece();
                            modifie = true;
                        }
                    }
                }
                if plan.pieces.len() < noyau::dessin::MAX_PIECES
                    && ui
                        .add(bouton_principal(
                            p,
                            textes::DESSIN_AJOUTER_UNE_PIECE.traduit(),
                            13.0,
                        ))
                        .clicked()
                    && plan.ajouter()
                {
                    etat.changer_de_piece();
                    modifie = true;
                }
                let renommer =
                    egui::Button::new(textes::DESSIN_RENOMMER.traduit()).selected(etat.renommer);
                if ui.add(renommer).clicked() {
                    etat.renommer = !etat.renommer;
                    etat.viser_le_nom = etat.renommer;
                }
                ui.add_space(10.0);
            });
        });
    // Le nom de la pièce choisie, sous la rangée, le temps de le changer.
    if etat.renommer {
        let courante = plan.courante;
        ui.horizontal(|ui| {
            ui.add_space(10.0);
            let champ = egui::TextEdit::singleline(&mut plan.pieces[courante].nom)
                .desired_width((ui.available_width() - 80.0).clamp(140.0, 300.0))
                .char_limit(40)
                .hint_text(textes::dessin_piece(courante + 1));
            let reponse = ui.add(champ);
            cadre_de_saisie(ui, p, &reponse);
            if etat.viser_le_nom {
                etat.viser_le_nom = false;
                reponse.request_focus();
            }
            if reponse.changed() {
                modifie = true;
            }
            let entree = reponse.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            if ui
                .add(bouton_principal(p, textes::OK.traduit(), 13.0))
                .clicked()
                || entree
            {
                etat.renommer = false;
            }
        });
    }
    modifie
}

/// Le nom de la pièce à l'écran, sa suppression, et le total des pièces.
fn la_piece_et_le_total(
    ui: &mut Ui,
    p: &Palette,
    plan: &mut Plan,
    etat: &mut Etat,
    prefs: &Preferences,
) -> (bool, Option<FormatDExport>) {
    let mut modifie = false;
    let courante = plan.courante;
    let nombre = plan.pieces.len();
    ui.add_space(10.0);
    let libelle = ui.label(
        RichText::new(textes::DESSIN_NOM_DE_LA_PIECE.traduit())
            .font(theme::police_grasse(13.0))
            .color(p.titre),
    );
    let mut supprimer = false;
    ui.horizontal_wrapped(|ui| {
        let indice = textes::dessin_piece(courante + 1);
        let champ = egui::TextEdit::singleline(&mut plan.pieces[courante].nom)
            .desired_width((ui.available_width() - 190.0).clamp(110.0, 240.0))
            .char_limit(40)
            .hint_text(indice);
        let reponse = ui.add(champ).labelled_by(libelle.id);
        cadre_de_saisie(ui, p, &reponse);
        if reponse.changed() {
            modifie = true;
        }
        let vide = nombre == 1 && plan.pieces[0] == noyau::dessin::Piece::default();
        if ui
            .add_enabled(
                !vide,
                egui::Button::new(textes::DESSIN_SUPPRIMER_LA_PIECE.traduit()),
            )
            .clicked()
        {
            supprimer = true;
        }
    });
    if supprimer {
        plan.supprimer_la_courante();
        etat.changer_de_piece();
        modifie = true;
    }
    if plan.pieces.len() > 1 {
        let (aire, completes) = plan.total(FORMAT_PAR_DEFAUT);
        // Le total s'écrit dans le système de la première mesure du plan.
        let modele = plan
            .pieces
            .iter()
            .find_map(|piece| piece.dessin.resoudre(FORMAT_PAR_DEFAUT).modele);
        ui.add_space(8.0);
        carte(ui, p, |ui| {
            ui.label(
                RichText::new(textes::dessin_total(completes, plan.pieces.len()))
                    .font(theme::police_grasse(13.5))
                    .color(p.titre),
            );
            ui.label(
                RichText::new(Quantite::Aire(aire).en_clair(modele, prefs))
                    .font(theme::police_grasse(17.0))
                    .color(p.texte),
            );
        });
    }
    // L'export du plan, toutes pièces comprises.
    let mut export = None;
    let une_piece_dessinee = plan.pieces.iter().any(|piece| !piece.dessin.est_vide());
    let une_piece_complete = plan.pieces.iter().any(|piece| {
        matches!(
            piece.dessin.resoudre(FORMAT_PAR_DEFAUT).etat,
            EtatDeLaForme::Complete { .. }
        )
    });
    ui.add_space(10.0);
    ui.label(
        RichText::new(textes::DESSIN_EXPORTER.traduit())
            .font(theme::police_grasse(13.0))
            .color(p.titre),
    );
    ui.horizontal_wrapped(|ui| {
        if ui
            .add_enabled(
                une_piece_dessinee,
                egui::Button::new(textes::DESSIN_EXPORTER_PDF.traduit()),
            )
            .clicked()
        {
            export = Some(FormatDExport::Pdf);
        }
        if ui
            .add_enabled(
                une_piece_complete,
                egui::Button::new(textes::DESSIN_EXPORTER_DXF.traduit()),
            )
            .on_disabled_hover_text(textes::DESSIN_DXF_IMPOSSIBLE.traduit())
            .clicked()
        {
            export = Some(FormatDExport::Dxf);
        }
    });
    (modifie, export)
}

/// Dessine la planche et traite les gestes. Retourne vrai si le dessin a changé.
#[allow(clippy::too_many_arguments)]
fn planche_et_gestes(
    ui: &mut Ui,
    p: &Palette,
    planche: Rect,
    reponse: &egui::Response,
    dessin: &mut Dessin,
    etat: &mut Etat,
    solution: &Solution,
    prefs: &Preferences,
) -> bool {
    let mut modifie = false;
    let painter = ui.painter_at(planche);
    painter.rect_filled(planche, 10, p.carte);
    painter.rect_stroke(
        planche,
        10,
        Stroke::new(1.0, p.carte_bord),
        egui::StrokeKind::Inside,
    );
    // La vue 3D, quand elle est choisie et que la pièce est complète.
    if etat.vue_3d {
        if let Some(maquette) = noyau::maquette::maquette(dessin, solution) {
            if reponse.dragged() {
                etat.orbite.tourner(reponse.drag_delta());
            }
            let (pincement, molette, survol) =
                ui.input(|i| (i.zoom_delta(), i.smooth_scroll_delta, i.pointer.hover_pos()));
            if survol.is_some_and(|q| planche.contains(q)) {
                let facteur = if (pincement - 1.0).abs() > 1e-4 {
                    pincement
                } else {
                    (molette.y * 0.002).exp()
                };
                etat.orbite.zoomer(facteur);
            }
            super::maquette::dessiner(&painter, planche.shrink(12.0), &maquette, &etat.orbite, p);
            let consigne = if solution.hauteur.is_none() {
                format!(
                    "{} · {}",
                    textes::DESSIN_TOURNER.traduit(),
                    textes::DESSIN_MURS_PAR_DEFAUT.traduit()
                )
            } else {
                textes::DESSIN_TOURNER.traduit().to_string()
            };
            painter.text(
                planche.left_top() + Vec2::new(12.0, 12.0),
                Align2::LEFT_TOP,
                consigne,
                theme::police(12.5),
                p.texte_attenue,
            );
            // Le zoom, et le retour au plan.
            let cote = 34.0;
            let a_droite = planche.right() - 8.0 - cote;
            let plus =
                Rect::from_min_size(Pos2::new(a_droite, planche.top() + 8.0), Vec2::splat(cote));
            if bouton_de_planche(
                ui,
                &painter,
                plus,
                "+",
                textes::DESSIN_ZOOMER.traduit(),
                false,
                p,
            ) {
                etat.orbite.zoomer(1.25);
            }
            let moins = plus.translate(Vec2::new(0.0, cote + 6.0));
            if bouton_de_planche(
                ui,
                &painter,
                moins,
                "\u{2212}",
                textes::DESSIN_DEZOOMER.traduit(),
                false,
                p,
            ) {
                etat.orbite.zoomer(0.8);
            }
            let ajuster = Rect::from_min_size(
                Pos2::new(planche.right() - 8.0 - 74.0, moins.bottom() + 6.0),
                Vec2::new(74.0, cote),
            );
            if etat.orbite != super::maquette::Orbite::default()
                && bouton_de_planche(
                    ui,
                    &painter,
                    ajuster,
                    textes::DESSIN_AJUSTER.traduit(),
                    textes::DESSIN_AJUSTER.traduit(),
                    false,
                    p,
                )
            {
                etat.orbite = super::maquette::Orbite::default();
            }
            let plan = Rect::from_min_size(
                Pos2::new(planche.left() + 8.0, planche.bottom() - 8.0 - cote),
                Vec2::new(52.0, cote),
            );
            if bouton_de_planche(
                ui,
                &painter,
                plan,
                textes::DESSIN_2D.traduit(),
                textes::DESSIN_REVENIR_AU_PLAN.traduit(),
                true,
                p,
            ) {
                etat.vue_3d = false;
            }
            return modifie;
        }
        etat.vue_3d = false;
    }

    // Un quadrillage léger, comme une feuille de chantier. Il suit le zoom et
    // le décalage, avec un pas toujours lisible.
    let mut pas = 26.0 * etat.cadrage.zoom;
    while pas > 52.0 {
        pas /= 2.0;
    }
    while pas < 13.0 {
        pas *= 2.0;
    }
    let repere = planche.center() + etat.cadrage.decalage;
    let mut x = planche.left() + (repere.x - planche.left()).rem_euclid(pas);
    while x < planche.right() {
        painter.line_segment(
            [Pos2::new(x, planche.top()), Pos2::new(x, planche.bottom())],
            Stroke::new(1.0, p.grille),
        );
        x += pas;
    }
    let mut y = planche.top() + (repere.y - planche.top()).rem_euclid(pas);
    while y < planche.bottom() {
        painter.line_segment(
            [Pos2::new(planche.left(), y), Pos2::new(planche.right(), y)],
            Stroke::new(1.0, p.grille),
        );
        y += pas;
    }

    // La forme à ses mesures quand elles sont toutes connues, sinon le croquis.
    let a_l_echelle = dessin.sommets_mesures(solution);
    let sommets_croquis = dessin.sommets_du_croquis();
    let sommets = a_l_echelle.as_deref().unwrap_or(&sommets_croquis);
    // Tant que la forme est ouverte, les flèches occupent le bas de la planche :
    // la forme se cadre au-dessus d'elles.
    let zone_de_la_forme = if dessin.fermee {
        planche
    } else {
        Rect::from_min_max(
            planche.min,
            Pos2::new(planche.max.x, planche.max.y - HAUTEUR_DES_FLECHES),
        )
    };
    let centre_de_la_planche = zone_de_la_forme.center();

    // --- Zoom et déplacement
    let (deux_doigts, pincement, molette, survol) = ui.input(|i| {
        (
            i.multi_touch(),
            i.zoom_delta(),
            i.smooth_scroll_delta,
            i.pointer.hover_pos(),
        )
    });
    if let Some(toucher) = deux_doigts.filter(|m| planche.contains(m.center_pos)) {
        // Deux doigts : ils zooment et déplacent, et le tracé en cours s'annule.
        etat.cadrage
            .zoomer(centre_de_la_planche, toucher.center_pos, toucher.zoom_delta);
        etat.cadrage.decalage += toucher.translation_delta;
        etat.trace.clear();
        etat.cloison_en_trace = None;
        etat.geste_annule = true;
    } else if let Some(pointeur) = survol.filter(|q| planche.contains(*q)) {
        // La molette, ou le pincement d'un pavé tactile, zoome autour du pointeur.
        let facteur = if (pincement - 1.0).abs() > 1e-4 {
            pincement
        } else {
            (molette.y * 0.002).exp()
        };
        if (facteur - 1.0).abs() > 1e-4 {
            etat.cadrage.zoomer(centre_de_la_planche, pointeur, facteur);
        }
    }
    // Glisser avec le bouton droit ou celui du milieu déplace toujours la vue;
    // avec le doigt ou le bouton gauche, quand la forme est fermée (elle ne
    // prend plus de tracé).
    // Une porte ou une fenêtre saisie au doigt glisse le long de son mur, au
    // lieu de déplacer la vue.
    let saisie = if dessin.fermee && reponse.drag_started_by(egui::PointerButton::Primary) {
        ui.input(|i| i.pointer.press_origin()).and_then(|origine| {
            etat.ouvertures_a_l_ecran
                .iter()
                .copied()
                .find(|(_, q0, q1)| {
                    distance_au_segment(origine, *q0, *q1) <= PORTEE_D_UNE_OUVERTURE
                })
                .map(|ouverture| (origine, ouverture))
        })
    } else {
        None
    };
    if let Some((origine, (rang, q0, q1))) = saisie
        && let Some(Some(placement)) = solution.placements.get(rang)
    {
        let longueur_a_l_ecran = q0.distance(q1).max(1e-3);
        let pouces_par_point = (placement.fin - placement.debut) / f64::from(longueur_a_l_ecran);
        let decalage =
            f64::from((origine - q0).dot((q1 - q0) / longueur_a_l_ecran)) * pouces_par_point;
        etat.memoriser(dessin);
        etat.ouverture_tiree = Some((rang, decalage));
        etat.choisir_autre(Some(AutreSaisie::Ouverture(
            rang,
            ChampDOuverture::Position,
        )));
    }
    // Un objet saisi au doigt se déplace, lui aussi, au lieu de la vue.
    let complete = matches!(solution.etat, EtatDeLaForme::Complete { .. });
    // Une cloison se trace d'un glissement, de son départ à son arrivée.
    if complete
        && etat.cloison_a_tracer
        && reponse.drag_started_by(egui::PointerButton::Primary)
        && let Some(origine) = ui.input(|i| i.pointer.press_origin())
        && let Some(echelle) = etat.derniere_echelle
    {
        let depart = echelle.vers_le_plan(origine);
        etat.cloison_en_trace = Some((depart, depart));
    }
    // Une note saisie au doigt se déplace aussi; elle passe avant les objets,
    // qu'elle recouvre.
    let note_saisie = if complete
        && etat.cloison_en_trace.is_none()
        && etat.ouverture_tiree.is_none()
        && reponse.drag_started_by(egui::PointerButton::Primary)
        && let Some(origine) = ui.input(|i| i.pointer.press_origin())
        && let Some(echelle) = etat.derniere_echelle
        && let Some(rang) = note_sous(etat, origine)
        && rang < dessin.notes.len()
    {
        let point = echelle.vers_le_plan(origine);
        let position = dessin.notes[rang].position;
        Some((rang, (point.0 - position.0, point.1 - position.1)))
    } else {
        None
    };
    if let Some((rang, ecart)) = note_saisie {
        etat.memoriser(dessin);
        etat.note_tiree = Some((rang, ecart));
        etat.note_choisie = Some(rang);
    }
    // Une cloison saisie au doigt se déplace d'un bloc.
    let cloison_saisie = if complete
        && etat.cloison_en_trace.is_none()
        && etat.ouverture_tiree.is_none()
        && etat.note_tiree.is_none()
        && reponse.drag_started_by(egui::PointerButton::Primary)
        && let Some(origine) = ui.input(|i| i.pointer.press_origin())
        && let Some(echelle) = etat.derniere_echelle
        && let Some(rang) = cloison_sous(
            solution,
            echelle.vers_le_plan(origine),
            f64::from(8.0 / echelle.facteur),
        )
        && rang < dessin.cloisons.len()
    {
        let point = echelle.vers_le_plan(origine);
        let debut = dessin.cloisons[rang].debut;
        Some((rang, (point.0 - debut.0, point.1 - debut.1)))
    } else {
        None
    };
    if let Some((rang, ecart)) = cloison_saisie {
        etat.memoriser(dessin);
        etat.cloison_tiree = Some((rang, ecart));
        etat.cloison_choisie = Some(rang);
    }
    let objet_saisi = if complete
        && etat.cloison_en_trace.is_none()
        && etat.cloison_tiree.is_none()
        && etat.ouverture_tiree.is_none()
        && etat.note_tiree.is_none()
        && reponse.drag_started_by(egui::PointerButton::Primary)
        && let Some(origine) = ui.input(|i| i.pointer.press_origin())
        && let Some(echelle) = etat.derniere_echelle
    {
        let point = echelle.vers_le_plan(origine);
        objet_sous(dessin, solution, point).map(|rang| {
            let centre = dessin.objets[rang].centre;
            (rang, (point.0 - centre.0, point.1 - centre.1))
        })
    } else {
        None
    };
    if let Some((rang, ecart)) = objet_saisi {
        etat.memoriser(dessin);
        etat.objet_tire = Some((rang, ecart));
        etat.objet_choisi = Some(rang);
    }
    let deplacer = reponse.dragged_by(egui::PointerButton::Secondary)
        || reponse.dragged_by(egui::PointerButton::Middle)
        || (dessin.fermee
            && reponse.dragged_by(egui::PointerButton::Primary)
            && etat.ouverture_tiree.is_none()
            && etat.objet_tire.is_none()
            && etat.note_tiree.is_none()
            && etat.cloison_tiree.is_none()
            && etat.cloison_en_trace.is_none());
    if deplacer && !etat.geste_annule {
        etat.cadrage.decalage += reponse.drag_delta();
    }

    // Une pièce complète se cadre à l'extérieur de ses murs.
    let contour_exterieur = if complete {
        dessin.contour_exterieur(solution)
    } else {
        None
    };
    let a_cadrer = contour_exterieur.as_deref().unwrap_or(sommets);
    let echelle = etat
        .cadrage
        .appliquer(cadrer(zone_de_la_forme, a_cadrer), centre_de_la_planche);
    // L'épaisseur des murs à l'écran : ils se dessinent hors des mesures, et
    // toute leur épaisseur choisit leur côté.
    let epaisseur_des_murs_a_l_ecran = if complete {
        solution.epaisseur_des_murs as f32 * echelle.facteur
    } else {
        0.0
    };
    // Le tracé, lui, se mesure toujours à l'échelle du croquis.
    let echelle_du_croquis = etat.cadrage.appliquer(
        cadrer(zone_de_la_forme, &sommets_croquis),
        centre_de_la_planche,
    );
    let points: Vec<Pos2> = sommets.iter().map(|s| echelle.vers_l_ecran(*s)).collect();

    // La porte ou la fenêtre qui glisse : sa nouvelle distance au coin A,
    // arrondie au pouce (au centimètre en métrique).
    if let Some((rang, decalage)) = etat.ouverture_tiree {
        if reponse.dragged()
            && let Some(pointeur) = reponse.interact_pointer_pos()
            && let Some(Some(placement)) = solution.placements.get(rang)
            && let (Some(a), Some(b)) = (points.get(placement.cote), points.get(placement.cote + 1))
            && let Some(longueur) = solution
                .mesures
                .get(placement.cote)
                .and_then(|m| m.pouces())
            && rang < dessin.ouvertures.len()
        {
            let le_long = *b - *a;
            let t = f64::from((pointeur - *a).dot(le_long) / le_long.length_sq().max(1e-6));
            let largeur = placement.fin - placement.debut;
            let metrique =
                solution.modele.and_then(Format::systeme) == Some(noyau::unites::Systeme::Metrique);
            let pas = if metrique { 1.0 / 2.54 } else { 1.0 };
            let debut = (((t * longueur - decalage) / pas).round() * pas)
                .clamp(0.0, (longueur - largeur).max(0.0));
            dessin.ouvertures[rang].position =
                sans_insecables(&Quantite::Longueur(debut).en_clair(solution.modele, prefs));
            modifie = true;
        }
        if !reponse.dragged() {
            etat.ouverture_tiree = None;
        }
    }

    // L'objet qui se déplace : son nouveau centre, au pouce près (au
    // centimètre en métrique).
    if let Some((rang, ecart)) = etat.objet_tire {
        if reponse.dragged()
            && let Some(pointeur) = reponse.interact_pointer_pos()
            && rang < dessin.objets.len()
        {
            let point = echelle.vers_le_plan(pointeur);
            let metrique =
                solution.modele.and_then(Format::systeme) == Some(noyau::unites::Systeme::Metrique);
            let pas = if metrique { 1.0 / 2.54 } else { 1.0 };
            let centre = (
                ((point.0 - ecart.0) / pas).round() * pas,
                ((point.1 - ecart.1) / pas).round() * pas,
            );
            if dessin.objets[rang].centre != centre {
                dessin.objets[rang].centre = centre;
                modifie = true;
            }
        }
        if !reponse.dragged() {
            etat.objet_tire = None;
        }
    }
    // La cloison qu'on trace suit le doigt; elle naît quand il se lève.
    if let Some((depart, atteint)) = etat.cloison_en_trace {
        let atteint = reponse
            .interact_pointer_pos()
            .map_or(atteint, |pointeur| echelle.vers_le_plan(pointeur));
        etat.cloison_en_trace = Some((depart, atteint));
        if !reponse.dragged() {
            etat.cloison_en_trace = None;
            let appuis = appuis_des_cloisons(sommets, solution);
            if let Some((sens, debut, longueur)) = redresser(
                depart,
                atteint,
                &appuis,
                aimant_des_cloisons(&echelle),
                pas_du_plan(solution),
            ) {
                etat.memoriser(dessin);
                dessin.cloisons.push(Cloison {
                    debut,
                    sens,
                    longueur: sans_insecables(
                        &Quantite::Longueur(longueur).en_clair(solution.modele, prefs),
                    ),
                    epaisseur: String::new(),
                    porte: None,
                });
                etat.cloison_choisie = Some(dessin.cloisons.len() - 1);
                etat.cloison_a_tracer = false;
                modifie = true;
            }
        }
    }
    // La cloison qui se déplace, au pouce près (au centimètre en métrique).
    if let Some((rang, ecart)) = etat.cloison_tiree {
        if reponse.dragged()
            && let Some(pointeur) = reponse.interact_pointer_pos()
            && rang < dessin.cloisons.len()
        {
            let point = echelle.vers_le_plan(pointeur);
            let pas = pas_du_plan(solution);
            let debut = (
                ((point.0 - ecart.0) / pas).round() * pas,
                ((point.1 - ecart.1) / pas).round() * pas,
            );
            if dessin.cloisons[rang].debut != debut {
                dessin.cloisons[rang].debut = debut;
                modifie = true;
            }
        }
        if !reponse.dragged() {
            etat.cloison_tiree = None;
        }
    }
    // La note qui se déplace, au pouce près.
    if let Some((rang, ecart)) = etat.note_tiree {
        if reponse.dragged()
            && let Some(pointeur) = reponse.interact_pointer_pos()
            && rang < dessin.notes.len()
        {
            let point = echelle.vers_le_plan(pointeur);
            let position = ((point.0 - ecart.0).round(), (point.1 - ecart.1).round());
            if dessin.notes[rang].position != position {
                dessin.notes[rang].position = position;
                modifie = true;
            }
        }
        if !reponse.dragged() {
            etat.note_tiree = None;
        }
    }
    etat.derniere_echelle = complete.then_some(echelle);

    // --- Gestes
    let trace_permise = !dessin.fermee && !etat.geste_annule;
    if reponse.drag_started_by(egui::PointerButton::Primary) && trace_permise {
        etat.trace.clear();
        // Le geste part de l'endroit où le doigt s'est posé, pas de celui où
        // le glissement a été reconnu : sinon, un tracé rapide perd son début.
        if let Some(origine) = ui.input(|i| i.pointer.press_origin()) {
            etat.trace.push(origine);
        }
    }
    if (reponse.dragged_by(egui::PointerButton::Primary)
        || reponse.drag_stopped_by(egui::PointerButton::Primary))
        && trace_permise
    {
        // Toutes les positions de cette image, pas seulement la dernière : un
        // geste rapide en apporte plusieurs à la fois.
        let positions: Vec<Pos2> = ui.input(|i| {
            i.events
                .iter()
                .filter_map(|evenement| match evenement {
                    egui::Event::PointerMoved(position) => Some(*position),
                    egui::Event::PointerButton {
                        pos,
                        pressed: false,
                        ..
                    } => Some(*pos),
                    _ => None,
                })
                .collect()
        });
        for position in positions.into_iter().chain(reponse.interact_pointer_pos()) {
            if etat.trace.last().is_none_or(|d| d.distance(position) > 1.5) {
                etat.trace.push(position);
            }
        }
    }
    if reponse.drag_stopped() {
        // Un geste interrompu par deux doigts ne trace rien.
        let annule = std::mem::take(&mut etat.geste_annule);
        if annule || !reponse.drag_stopped_by(egui::PointerButton::Primary) {
            etat.trace.clear();
        }
        let trace: Vec<(f64, f64)> = etat
            .trace
            .iter()
            .map(|point| {
                (
                    f64::from(
                        (point.x - echelle_du_croquis.origine.x) / echelle_du_croquis.facteur,
                    ),
                    f64::from(
                        (point.y - echelle_du_croquis.origine.y) / echelle_du_croquis.facteur,
                    ),
                )
            })
            .collect();
        etat.trace.clear();
        let avant = dessin.clone();
        if trace.len() > 1 && dessin.ajouter_trace(&trace) {
            etat.memoriser(&avant);
            modifie = true;
            if dessin.fermee {
                let solution = dessin.resoudre(FORMAT_PAR_DEFAUT);
                etat.choisir(premier_cote_a_mesurer(&solution).or(Some(0)));
            }
        }
    } else if reponse.clicked() && etat.cloison_a_tracer {
        // Une cloison se trace en glissant : un simple toucher n'y fait rien.
    } else if reponse.clicked()
        && let Some(position) = reponse.interact_pointer_pos()
        && complete
        && etat.note_a_placer
    {
        // La note se pose là où le plan a été touché, et son champ prend le
        // clavier.
        let (x, y) = echelle.vers_le_plan(position);
        etat.memoriser(dessin);
        dessin.notes.push(noyau::dessin::Note {
            texte: String::new(),
            position: (x.round(), y.round()),
        });
        let rang = dessin.notes.len() - 1;
        etat.note_a_placer = false;
        etat.note_choisie = Some(rang);
        etat.viser_la_note = Some(rang);
        etat.choisir(None);
        modifie = true;
    } else if reponse.clicked()
        && let Some(position) = reponse.interact_pointer_pos()
        && complete
        && etat.ouverture_a_placer.is_none()
        && etat.objet_a_placer.is_none()
        && let Some(rang) = note_sous(etat, position)
    {
        // Toucher une note la choisit et ouvre son champ.
        etat.note_choisie = Some(rang);
        etat.viser_la_note = Some(rang);
        etat.choisir(None);
    } else if reponse.clicked()
        && let Some(position) = reponse.interact_pointer_pos()
        && complete
        && let Some(genre) = etat.objet_a_placer
    {
        // L'objet se pose là où la pièce a été touchée.
        let (x, y) = echelle.vers_le_plan(position);
        etat.memoriser(dessin);
        dessin
            .objets
            .push(Objet::nouveau(genre, (x.round(), y.round())));
        etat.objet_a_placer = None;
        etat.objet_choisi = Some(dessin.objets.len() - 1);
        modifie = true;
    } else if reponse.clicked()
        && let Some(position) = reponse.interact_pointer_pos()
        && etat.ouverture_a_placer.is_none()
        && let Some((rang, _, _)) = etat
            .ouvertures_a_l_ecran
            .iter()
            .copied()
            .find(|(_, q0, q1)| distance_au_segment(position, *q0, *q1) <= PORTEE_D_UNE_OUVERTURE)
    {
        // Toucher une porte ou une fenêtre ouvre sa distance au coin A.
        etat.choisir_autre(Some(AutreSaisie::Ouverture(
            rang,
            ChampDOuverture::Position,
        )));
    } else if reponse.clicked()
        && let Some(position) = reponse.interact_pointer_pos()
        && complete
        && etat.ouverture_a_placer.is_none()
        && let Some(rang) = cloison_sous(
            solution,
            echelle.vers_le_plan(position),
            f64::from(8.0 / echelle.facteur),
        )
    {
        // Toucher une cloison la choisit.
        etat.cloison_choisie = Some(rang);
        etat.objet_choisi = None;
        etat.note_choisie = None;
    } else if reponse.clicked()
        && let Some(position) = reponse.interact_pointer_pos()
        && complete
        && etat.ouverture_a_placer.is_none()
        && let Some(rang) = objet_sous(dessin, solution, echelle.vers_le_plan(position))
    {
        // Toucher un objet le choisit.
        etat.objet_choisi = Some(rang);
        etat.note_choisie = None;
        etat.cloison_choisie = None;
    } else if reponse.clicked()
        && let Some(position) = reponse.interact_pointer_pos()
    {
        etat.objet_choisi = None;
        etat.note_choisie = None;
        etat.cloison_choisie = None;
        let plus_proche = points
            .windows(2)
            .enumerate()
            .map(|(i, s)| (i, distance_au_segment(position, s[0], s[1])))
            .min_by(|a, b| a.1.total_cmp(&b.1));
        match plus_proche {
            Some((i, distance)) if distance <= PORTEE_DU_TOUCHER + epaisseur_des_murs_a_l_ecran => {
                if let Some(rang) = etat.ouverture_a_placer.take()
                    && rang < dessin.ouvertures.len()
                {
                    // L'ouverture se centre là où le mur a été touché.
                    etat.memoriser(dessin);
                    let (a, b) = (points[i], points[i + 1]);
                    let le_long = b - a;
                    let t = ((position - a).dot(le_long) / le_long.length_sq().max(1e-6))
                        .clamp(0.0, 1.0);
                    let longueur = solution.mesures.get(i).and_then(|m| m.pouces());
                    let largeur = pouces_de(&dessin.ouvertures[rang].largeur);
                    let ouverture = &mut dessin.ouvertures[rang];
                    ouverture.cote = Some(i);
                    ouverture.position = match (longueur, largeur) {
                        (Some(longueur), Some(largeur)) if largeur <= longueur => {
                            let debut = (f64::from(t) * longueur - largeur / 2.0)
                                .clamp(0.0, longueur - largeur);
                            sans_insecables(
                                &Quantite::Longueur(debut).en_clair(solution.modele, prefs),
                            )
                        }
                        _ => String::new(),
                    };
                    modifie = true;
                } else {
                    etat.choisir(Some(i));
                }
            }
            _ => {
                if etat.ouverture_a_placer.is_none() {
                    etat.choisir(None);
                }
            }
        }
    }

    // --- Dessin
    if dessin.est_vide() && etat.trace.is_empty() {
        painter.text(
            planche.center(),
            Align2::CENTER_CENTER,
            textes::DESSIN_INVITE.traduit(),
            theme::police(14.5),
            p.texte_attenue,
        );
    }
    // Les objets : leur rectangle, plein, leur symbole et leur nom.
    if complete {
        for (rang, (objet, &dimensions)) in dessin.objets.iter().zip(&solution.objets).enumerate() {
            let choisi = etat.objet_choisi == Some(rang)
                || etat.objet_tire.is_some_and(|(r, _)| r == rang)
                || matches!(etat.autre_choisi, Some(AutreSaisie::Objet(r, _)) if r == rang);
            let encre = if choisi {
                p.barre_accent
            } else {
                p.texte_attenue
            };
            let coins: Vec<Pos2> = objet
                .coins(dimensions)
                .iter()
                .map(|coin| echelle.vers_l_ecran(*coin))
                .collect();
            painter.add(egui::Shape::convex_polygon(
                coins.clone(),
                theme::melanger(p.carte, p.texte, 0.06),
                Stroke::NONE,
            ));
            for trait_ in objet.traits_sur_le_plan(dimensions) {
                match trait_ {
                    TraitSurLePlan::Ligne(a, b) => {
                        painter.line_segment(
                            [echelle.vers_l_ecran(a), echelle.vers_l_ecran(b)],
                            Stroke::new(1.3, encre),
                        );
                    }
                    TraitSurLePlan::Cercle(centre, rayon) => {
                        painter.circle_stroke(
                            echelle.vers_l_ecran(centre),
                            rayon as f32 * echelle.facteur,
                            Stroke::new(1.3, encre),
                        );
                    }
                }
            }
            let plus_petit = coins[0].distance(coins[1]).min(coins[1].distance(coins[2]));
            if plus_petit > 34.0 {
                painter.text(
                    echelle.vers_l_ecran(objet.centre),
                    Align2::CENTER_CENTER,
                    textes::objet(objet.genre).traduit(),
                    theme::police(10.5),
                    encre,
                );
            }
        }
    }
    // Les cloisons, et celle qu'on trace.
    if complete {
        for (rang, cloison) in solution.cloisons.iter().enumerate() {
            let Some(cloison) = cloison else {
                continue;
            };
            let choisie = etat.cloison_choisie == Some(rang)
                || etat.cloison_tiree.is_some_and(|(r, _)| r == rang)
                || matches!(etat.autre_choisi, Some(AutreSaisie::Cloison(r, _)) if r == rang);
            let longueur = longueur_en_clair(cloison.longueur, solution, prefs);
            dessiner_une_cloison(&painter, p, &echelle, cloison, choisie, longueur);
        }
        if let Some((depart, atteint)) = etat.cloison_en_trace
            && let Some((sens, debut, longueur)) = redresser(
                depart,
                atteint,
                &appuis_des_cloisons(sommets, solution),
                aimant_des_cloisons(&echelle),
                pas_du_plan(solution),
            )
        {
            let (ux, uy) = sens.pas();
            let apercu = CloisonMesuree {
                debut,
                fin: (debut.0 + ux * longueur, debut.1 + uy * longueur),
                longueur,
                epaisseur: solution.epaisseur_des_cloisons,
                porte: None,
            };
            let texte = longueur_en_clair(longueur, solution, prefs);
            dessiner_une_cloison(&painter, p, &echelle, &apercu, true, texte);
        }
    }
    // Les murs extérieurs, à leur épaisseur, hors des mesures : pleins, avec
    // leur face extérieure et leurs jambages; le côté choisi en couleur.
    if complete {
        for morceau in dessin.murs_exterieurs(solution) {
            dessiner_un_morceau_de_mur(&painter, &echelle, &morceau, etat.choisi, p);
        }
    }
    for (i, s) in points.windows(2).enumerate() {
        let choisi = etat.choisi == Some(i);
        let (epaisseur, couleur) = if choisi {
            (5.0, p.barre_accent)
        } else {
            (3.0, p.primaire)
        };
        painter.line_segment([s[0], s[1]], Stroke::new(epaisseur, couleur));
    }
    // Les sommets, dès qu'il y a un côté (une planche vide n'en montre aucun).
    if points.len() > 1 {
        for point in &points {
            painter.circle_filled(*point, 3.5, p.primaire);
        }
    }
    if !dessin.fermee
        && let (Some(depart), Some(fin)) = (points.first(), points.last())
        && points.len() > 1
    {
        // Le départ à rejoindre, et le bout d'où repart le tracé.
        painter.circle_stroke(*depart, 8.0, Stroke::new(2.0, p.barre_accent));
        painter.circle_filled(*fin, 5.5, p.barre_accent);
    }
    // Les portes et les fenêtres, sur leur mur, ouvertes vers l'intérieur.
    etat.ouvertures_a_l_ecran.clear();
    if dessin.fermee && points.len() > 3 {
        let aire_signee: f32 = points
            .windows(2)
            .map(|s| s[0].x * s[1].y - s[1].x * s[0].y)
            .sum();
        for (rang, placement) in solution.placements.iter().enumerate() {
            let Some(placement) = placement else {
                continue;
            };
            let (Some(a), Some(b), Some(longueur)) = (
                points.get(placement.cote),
                points.get(placement.cote + 1),
                solution
                    .mesures
                    .get(placement.cote)
                    .and_then(|m| m.pouces()),
            ) else {
                continue;
            };
            let le_long = *b - *a;
            let q0 = *a + le_long * (placement.debut / longueur) as f32;
            let q1 = *a + le_long * (placement.fin / longueur) as f32;
            let unite = le_long.normalized();
            let dedans = if aire_signee > 0.0 {
                Vec2::new(-unite.y, unite.x)
            } else {
                Vec2::new(unite.y, -unite.x)
            };
            etat.ouvertures_a_l_ecran.push((rang, q0, q1));
            let choisie = etat.ouverture_a_placer == Some(rang)
                || etat.ouverture_tiree.is_some_and(|(r, _)| r == rang)
                || matches!(etat.autre_choisi, Some(AutreSaisie::Ouverture(r, _)) if r == rang);
            let encre = if choisie { p.barre_accent } else { p.texte };
            if choisie {
                // Les deux distances jusqu'aux coins, à l'intérieur de la pièce,
                // et le coin A, d'où se mesure la distance au coin.
                let trait_ = Stroke::new(1.0, p.barre_accent);
                for (depuis, jusqu_a, distance) in [
                    (*a, q0, placement.debut),
                    (q1, *b, longueur - placement.fin),
                ] {
                    if distance < 0.5 {
                        continue;
                    }
                    painter.line_segment([depuis + dedans * 16.0, jusqu_a + dedans * 16.0], trait_);
                    for bout in [depuis, jusqu_a] {
                        painter.line_segment([bout + dedans * 10.0, bout + dedans * 22.0], trait_);
                    }
                    painter.text(
                        depuis + (jusqu_a - depuis) * 0.5 + dedans * 28.0,
                        Align2::CENTER_CENTER,
                        longueur_en_clair(distance, solution, prefs),
                        theme::police_grasse(11.5),
                        p.barre_accent,
                    );
                }
                painter.circle_stroke(*a, 6.5, Stroke::new(1.6, p.barre_accent));
                painter.text(
                    *a + dedans * 14.0 + unite * 12.0,
                    Align2::CENTER_CENTER,
                    "A",
                    theme::police_grasse(12.0),
                    p.barre_accent,
                );
            }
            // Le mur s'interrompt.
            painter.line_segment([q0, q1], Stroke::new(7.0, p.carte));
            dessiner_ouverture(
                &painter,
                (q0, q1),
                dedans,
                placement.porte,
                encre,
                p.primaire,
                epaisseur_des_murs_a_l_ecran,
            );
        }
    }

    // Les mesures, à côté de chaque côté, vers l'extérieur de la forme.
    let centre = if points.is_empty() {
        planche.center()
    } else {
        let somme = points.iter().fold(Vec2::ZERO, |s, q| s + q.to_vec2());
        (somme / points.len() as f32).to_pos2()
    };
    for (i, s) in points.windows(2).enumerate() {
        let (Some(cote), Some(mesure)) = (dessin.cotes.get(i), solution.mesures.get(i)) else {
            continue;
        };
        let milieu = s[0] + (s[1] - s[0]) * 0.5;
        // Un côté hors de la vue (après un zoom) n'écrit pas sa mesure.
        if !planche.expand(4.0).contains(milieu) {
            continue;
        }
        let (dehors, ancre) = place_de_la_mesure(s[0], s[1], centre);
        let (texte, couleur) = match mesure {
            Mesure::Lue(_) => (
                saisie_en_clair(&cote.saisie, prefs)
                    .unwrap_or_else(|| cote.saisie.trim().to_string()),
                p.texte,
            ),
            Mesure::Deduite(pouces) => {
                (longueur_en_clair(*pouces, solution, prefs), p.texte_attenue)
            }
            Mesure::Manquante => ("?".to_string(), p.barre_accent),
            Mesure::Invalide => (cote.saisie.trim().to_string(), p.erreur),
        };
        // Hors du mur, à son épaisseur.
        let position = milieu + dehors * (7.0 + epaisseur_des_murs_a_l_ecran);
        let galere = painter.layout_no_wrap(texte, theme::police_grasse(12.5), couleur);
        let cadre = ancre
            .anchor_size(position, galere.size())
            .expand2(Vec2::new(3.0, 1.0));
        // La mesure reste dans la planche, même pour un côté collé au bord.
        let decalage = Vec2::new(
            (planche.left() + 3.0 - cadre.left()).max(0.0)
                - (cadre.right() - planche.right() + 3.0).max(0.0),
            (planche.top() + 3.0 - cadre.top()).max(0.0)
                - (cadre.bottom() - planche.bottom() + 3.0).max(0.0),
        );
        let cadre = cadre.translate(decalage);
        painter.rect_filled(cadre, 4, p.carte.gamma_multiply(0.92));
        painter.galley(cadre.min + Vec2::new(3.0, 1.0), galere, couleur);
    }
    // Les notes, par-dessus tout : leur texte dans un cadre clair.
    etat.notes_a_l_ecran.clear();
    if complete {
        for (rang, note) in dessin.notes.iter().enumerate() {
            let choisie =
                etat.note_choisie == Some(rang) || etat.note_tiree.is_some_and(|(r, _)| r == rang);
            let (texte, couleur) = if note.texte.trim().is_empty() {
                (textes::DESSIN_NOTE.traduit().to_string(), p.texte_attenue)
            } else {
                (note.texte.trim().to_string(), p.texte)
            };
            let galere = painter.layout(texte, theme::police(12.5), couleur, 180.0);
            let cadre = Rect::from_center_size(echelle.vers_l_ecran(note.position), galere.size())
                .expand2(Vec2::new(5.0, 3.0));
            painter.rect_filled(cadre, 4, p.carte.gamma_multiply(0.94));
            let bord = if choisie {
                Stroke::new(1.8, p.barre_accent)
            } else {
                Stroke::new(1.0, p.carte_bord)
            };
            painter.rect_stroke(cadre, 4, bord, egui::StrokeKind::Inside);
            painter.galley(cadre.min + Vec2::new(5.0, 3.0), galere, couleur);
            etat.notes_a_l_ecran.push((rang, cadre));
        }
    }
    // Le tracé en cours.
    if etat.trace.len() > 1 {
        painter.add(egui::Shape::line(
            etat.trace.clone(),
            Stroke::new(3.0, p.barre_accent),
        ));
    }
    // Une cloison attend son glissement.
    if etat.cloison_a_tracer {
        painter.text(
            planche.left_top() + Vec2::new(12.0, 12.0),
            Align2::LEFT_TOP,
            textes::DESSIN_TRACER_LA_CLOISON.traduit(),
            theme::police_grasse(13.0),
            p.barre_accent,
        );
    }
    // Une note attend qu'on touche le plan.
    if etat.note_a_placer {
        painter.text(
            planche.left_top() + Vec2::new(12.0, 12.0),
            Align2::LEFT_TOP,
            textes::DESSIN_TOUCHER_POUR_LA_NOTE.traduit(),
            theme::police_grasse(13.0),
            p.barre_accent,
        );
    }
    // Un objet attend qu'on touche la pièce.
    if let Some(genre) = etat.objet_a_placer {
        painter.text(
            planche.left_top() + Vec2::new(12.0, 12.0),
            Align2::LEFT_TOP,
            textes::dessin_toucher_pour_placer(textes::objet(genre).traduit()),
            theme::police_grasse(13.0),
            p.barre_accent,
        );
    }
    // Une ouverture attend qu'on touche son mur.
    if let Some(ouverture) = etat
        .ouverture_a_placer
        .and_then(|rang| dessin.ouvertures.get(rang))
    {
        let porte = ouverture.est_une_porte(pouces_de(&ouverture.hauteur));
        painter.text(
            planche.left_top() + Vec2::new(12.0, 12.0),
            Align2::LEFT_TOP,
            textes::dessin_toucher_le_mur(porte),
            theme::police_grasse(13.0),
            p.barre_accent,
        );
    }
    boutons_du_zoom(ui, p, &painter, planche, centre_de_la_planche, etat);
    // La vue 3D, pour une pièce complète.
    let bouton_3d = Rect::from_min_size(
        Pos2::new(planche.left() + 8.0, planche.bottom() - 42.0),
        Vec2::new(52.0, 34.0),
    );
    if dessin.fermee
        && matches!(solution.etat, EtatDeLaForme::Complete { .. })
        && bouton_de_planche(
            ui,
            &painter,
            bouton_3d,
            textes::DESSIN_3D.traduit(),
            textes::DESSIN_VOIR_EN_3D.traduit(),
            false,
            p,
        )
    {
        etat.vue_3d = true;
    }
    if !dessin.fermee && fleches(ui, p, &painter, planche, dessin, etat) {
        modifie = true;
    }
    modifie
}

/// Place que prennent les flèches au bas de la planche.
const HAUTEUR_DES_FLECHES: f32 = 46.0;

/// Les flèches, au bas de la planche : chacune ajoute un côté dans sa direction
/// (ou allonge le dernier), puis ouvre le pavé pour sa mesure. Retourne vrai
/// si le dessin a changé.
fn fleches(
    ui: &Ui,
    p: &Palette,
    painter: &egui::Painter,
    planche: Rect,
    dessin: &mut Dessin,
    etat: &mut Etat,
) -> bool {
    let mut modifie = false;
    // « Fermer la forme » se place à droite; les flèches prennent la place qui
    // reste à gauche, sans la chevaucher, même sur un écran étroit.
    let libelle_fermer = textes::DESSIN_FERMER_LA_FORME.traduit();
    let police_fermer = theme::police_grasse(13.0);
    let largeur_fermer = painter
        .layout_no_wrap(
            libelle_fermer.to_string(),
            police_fermer.clone(),
            p.selection_texte,
        )
        .size()
        .x
        + 20.0;
    // Quatre flèches et le bouton « 45° ».
    let largeur_fleche =
        ((planche.width() - 16.0 - largeur_fermer - 8.0 - 4.0 * 6.0) / 5.0).clamp(30.0, 44.0);
    let taille = Vec2::new(largeur_fleche, 34.0);
    let d = std::f32::consts::FRAC_1_SQRT_2;
    let en_biais = [
        (
            Sens::NordOuest,
            textes::DESSIN_FLECHE_HAUT_GAUCHE,
            Vec2::new(-d, -d),
        ),
        (
            Sens::NordEst,
            textes::DESSIN_FLECHE_HAUT_DROITE,
            Vec2::new(d, -d),
        ),
        (
            Sens::SudOuest,
            textes::DESSIN_FLECHE_BAS_GAUCHE,
            Vec2::new(-d, d),
        ),
        (
            Sens::SudEst,
            textes::DESSIN_FLECHE_BAS_DROITE,
            Vec2::new(d, d),
        ),
    ];
    let d_equerre = [
        (
            Sens::Ouest,
            textes::DESSIN_FLECHE_GAUCHE,
            Vec2::new(-1.0, 0.0),
        ),
        (Sens::Nord, textes::DESSIN_FLECHE_HAUT, Vec2::new(0.0, -1.0)),
        (Sens::Sud, textes::DESSIN_FLECHE_BAS, Vec2::new(0.0, 1.0)),
        (Sens::Est, textes::DESSIN_FLECHE_DROITE, Vec2::new(1.0, 0.0)),
    ];
    let fleches = if etat.fleches_en_biais {
        en_biais
    } else {
        d_equerre
    };
    for (rang, (sens, nom, direction)) in fleches.into_iter().enumerate() {
        let zone = Rect::from_min_size(
            Pos2::new(
                planche.left() + 8.0 + rang as f32 * (taille.x + 6.0),
                planche.bottom() - 8.0 - taille.y,
            ),
            taille,
        );
        let reponse = ui.interact(
            zone,
            egui::Id::unique(("fleche-du-dessin", rang)),
            Sense::click(),
        );
        reponse.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, nom.traduit()));
        let fond = if reponse.hovered() {
            theme::melanger(p.bouton, p.primaire, 0.12)
        } else {
            p.bouton
        };
        painter.rect_filled(zone, 8, fond.gamma_multiply(0.94));
        painter.rect_stroke(
            zone,
            8,
            Stroke::new(1.0, p.bouton_bord),
            egui::StrokeKind::Inside,
        );
        // La flèche : une hampe et une pointe.
        let centre = zone.center();
        let pointe = centre + direction * 9.0;
        let travers = Vec2::new(-direction.y, direction.x);
        painter.line_segment(
            [centre - direction * 9.0, pointe - direction * 4.0],
            Stroke::new(2.6, p.texte),
        );
        painter.add(egui::Shape::convex_polygon(
            vec![
                pointe,
                pointe - direction * 7.0 + travers * 5.5,
                pointe - direction * 7.0 - travers * 5.5,
            ],
            p.texte,
            Stroke::NONE,
        ));
        if reponse.clicked() {
            let avant = dessin.clone();
            if dessin.ajouter_cote(sens) {
                etat.memoriser(&avant);
                modifie = true;
                // Le côté ajouté attend sa mesure; une forme qui vient de se
                // fermer attend celle de son premier côté sans mesure.
                let rang = if dessin.fermee {
                    premier_cote_a_mesurer(&dessin.resoudre(FORMAT_PAR_DEFAUT)).unwrap_or(0)
                } else {
                    dessin.cotes.len().saturating_sub(1)
                };
                etat.choisir(Some(rang));
            }
        }
        reponse.on_hover_text(nom.traduit());
    }
    // Le bouton « 45° » : les flèches passent en biais, ou reviennent d'équerre.
    let zone = Rect::from_min_size(
        Pos2::new(
            planche.left() + 8.0 + 4.0 * (taille.x + 6.0),
            planche.bottom() - 8.0 - taille.y,
        ),
        taille,
    );
    let reponse = ui.interact(zone, egui::Id::unique("fleches-en-biais"), Sense::click());
    reponse.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::Role::CheckBox,
            true,
            etat.fleches_en_biais,
            textes::DESSIN_FLECHES_EN_BIAIS.traduit(),
        )
    });
    let (fond, encre) = if etat.fleches_en_biais {
        (p.selection, p.selection_texte)
    } else if reponse.hovered() {
        (theme::melanger(p.bouton, p.primaire, 0.12), p.texte)
    } else {
        (p.bouton, p.texte)
    };
    painter.rect_filled(zone, 8, fond.gamma_multiply(0.94));
    painter.rect_stroke(
        zone,
        8,
        Stroke::new(1.0, p.bouton_bord),
        egui::StrokeKind::Inside,
    );
    painter.text(
        zone.center(),
        Align2::CENTER_CENTER,
        textes::DESSIN_45_DEGRES.traduit(),
        theme::police_grasse(12.5),
        encre,
    );
    if reponse.clicked() {
        etat.fleches_en_biais = !etat.fleches_en_biais;
    }
    reponse.on_hover_text(textes::DESSIN_FLECHES_EN_BIAIS.traduit());
    // « Fermer la forme », au bas de la planche, à droite des flèches.
    if dessin.cotes.len() >= 2 {
        let zone = Rect::from_min_size(
            Pos2::new(
                planche.right() - 8.0 - largeur_fermer,
                planche.bottom() - 8.0 - taille.y,
            ),
            Vec2::new(largeur_fermer, taille.y),
        );
        let reponse = ui.interact(zone, egui::Id::unique("fermer-la-forme"), Sense::click());
        reponse.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, libelle_fermer));
        let fond = if reponse.hovered() {
            theme::melanger(p.selection, p.texte, 0.15)
        } else {
            p.selection
        };
        painter.rect_filled(zone, 8, fond);
        painter.text(
            zone.center(),
            Align2::CENTER_CENTER,
            libelle_fermer,
            police_fermer,
            p.selection_texte,
        );
        if reponse.clicked() {
            let avant = dessin.clone();
            if dessin.fermer() {
                etat.memoriser(&avant);
                modifie = true;
                let solution = dessin.resoudre(FORMAT_PAR_DEFAUT);
                etat.choisir(premier_cote_a_mesurer(&solution).or(Some(0)));
            }
        }
    }
    modifie
}

/// Les boutons du zoom, dans le coin de la planche : « + », « − » et, quand la
/// vue a bougé, « Ajuster », qui fait de nouveau tenir la forme dans la planche.
fn boutons_du_zoom(
    ui: &Ui,
    p: &Palette,
    painter: &egui::Painter,
    planche: Rect,
    centre: Pos2,
    etat: &mut Etat,
) {
    let cote = 34.0;
    let mut haut = planche.top() + 8.0;
    let mut boutons = vec![
        ("+", textes::DESSIN_ZOOMER, 1.25_f32, cote),
        ("\u{2212}", textes::DESSIN_DEZOOMER, 0.8, cote),
    ];
    if etat.cadrage != Cadrage::default() {
        boutons.push((
            textes::DESSIN_AJUSTER.traduit(),
            textes::DESSIN_AJUSTER,
            0.0,
            74.0,
        ));
    }
    for (rang, (signe, nom, facteur, largeur)) in boutons.into_iter().enumerate() {
        let zone = Rect::from_min_size(
            Pos2::new(planche.right() - 8.0 - largeur, haut),
            Vec2::new(largeur, cote),
        );
        haut += cote + 6.0;
        let reponse = ui.interact(
            zone,
            egui::Id::unique(("zoom-du-dessin", rang)),
            Sense::click(),
        );
        reponse.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, nom.traduit()));
        let fond = if reponse.hovered() {
            theme::melanger(p.bouton, p.primaire, 0.12)
        } else {
            p.bouton
        };
        painter.rect_filled(zone, 8, fond.gamma_multiply(0.94));
        painter.rect_stroke(
            zone,
            8,
            Stroke::new(1.0, p.bouton_bord),
            egui::StrokeKind::Inside,
        );
        let police = if signe.chars().count() > 1 {
            theme::police_grasse(13.0)
        } else {
            theme::police_grasse(20.0)
        };
        painter.text(zone.center(), Align2::CENTER_CENTER, signe, police, p.texte);
        if reponse.clicked() {
            if facteur > 0.0 {
                etat.cadrage.zoomer(centre, centre, facteur);
            } else {
                etat.cadrage = Cadrage::default();
            }
        }
        reponse.on_hover_text(nom.traduit());
    }
}

/// Les boutons sous la planche. Retourne vrai si le dessin a changé.
fn boutons(ui: &mut Ui, dessin: &mut Dessin, etat: &mut Etat) -> bool {
    let mut modifie = false;
    // « Fermer la forme » est sur la planche, à côté des flèches : cette
    // rangée garde la même hauteur, et le pavé ne se déplace pas.
    ui.horizontal_wrapped(|ui| {
        let plein = !dessin.est_vide();
        if ui
            .add_enabled(
                !etat.historique.is_empty(),
                egui::Button::new(textes::DESSIN_ANNULER.traduit()),
            )
            .clicked()
            && let Some(precedent) = etat.historique.pop()
        {
            *dessin = precedent;
            etat.choisir(None);
            modifie = true;
        }
        if ui
            .add_enabled(plein, egui::Button::new(textes::DESSIN_RETIRER.traduit()))
            .clicked()
        {
            etat.memoriser(dessin);
            dessin.retirer_le_dernier();
            etat.choisir(None);
            modifie = true;
        }
        if ui
            .add_enabled(plein, egui::Button::new(textes::DESSIN_EFFACER.traduit()))
            .clicked()
        {
            etat.memoriser(dessin);
            dessin.effacer();
            etat.cadrage = Cadrage::default();
            etat.choisir(None);
            modifie = true;
        }
    });
    modifie
}

/// Le champ de la mesure du côté choisi. Retourne vrai si une mesure a changé.
fn mesure_du_cote(
    ui: &mut Ui,
    p: &Palette,
    dessin: &mut Dessin,
    etat: &mut Etat,
    solution: &Solution,
    prefs: &Preferences,
) -> bool {
    if let Some(autre) = etat.autre_choisi {
        return autre_mesure(ui, p, dessin, etat, autre, prefs);
    }
    let Some(rang) = etat.choisi else {
        return false;
    };
    let nombre = dessin.cotes.len();
    let Some(cote) = dessin.cotes.get_mut(rang) else {
        return false;
    };
    let mut modifie = false;
    let mut suivant = false;
    carte(ui, p, |ui| {
        let libelle = ui.label(
            RichText::new(textes::dessin_cote(rang + 1, nombre)).font(theme::police_grasse(13.5)),
        );
        ui.horizontal(|ui| {
            // Une mesure déduite se lit dans le champ vide, en gris.
            let indice = match solution.mesures.get(rang) {
                Some(Mesure::Deduite(pouces)) => longueur_en_clair(*pouces, solution, prefs),
                _ => textes::DESSIN_EXEMPLE.traduit().to_string(),
            };
            let largeur = ui.available_width().min(360.0);
            let champ = egui::TextEdit::singleline(&mut cote.saisie)
                .desired_width(largeur)
                .hint_text(indice);
            let reponse = ui.add(champ).labelled_by(libelle.id);
            cadre_de_saisie(ui, p, &reponse);
            if etat.viser_le_champ {
                etat.viser_le_champ = false;
                // Sur un téléphone, viser le champ ouvrirait le clavier du
                // système par-dessus le pavé de touches.
                if !crate::plateforme::NAVIGATEUR {
                    reponse.request_focus();
                }
            }
            if reponse.changed() {
                modifie = true;
            }
            if reponse.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                suivant = true;
            }
        });
        ui.add_space(4.0);
        match pave_de_touches(ui, p, &mut cote.saisie, prefs) {
            Some(Frappe::Modifie) => modifie = true,
            Some(Frappe::Suivant) => suivant = true,
            None => {}
        }
        if matches!(solution.mesures.get(rang), Some(Mesure::Invalide)) {
            ui.label(
                RichText::new(textes::DESSIN_MESURE_INVALIDE.traduit())
                    .size(12.5)
                    .color(p.erreur),
            );
        } else if matches!(solution.mesures.get(rang), Some(Mesure::Deduite(_))) {
            ui.label(
                RichText::new(textes::DESSIN_MESURE_DEDUITE.traduit())
                    .size(12.5)
                    .color(p.texte_attenue),
            );
        }
    });
    if suivant && nombre > 0 {
        // Le prochain côté sans mesure, sinon simplement le suivant.
        let solution = dessin.resoudre(FORMAT_PAR_DEFAUT);
        let prochain = (1..=nombre)
            .map(|k| (rang + k) % nombre)
            .find(|&i| matches!(solution.mesures.get(i), Some(Mesure::Manquante)));
        etat.choisir(Some(prochain.unwrap_or((rang + 1) % nombre)));
    }
    modifie
}

/// Le champ d'une autre mesure (retrait, hauteur des murs, ouverture), avec le
/// pavé de touches.
fn autre_mesure(
    ui: &mut Ui,
    p: &Palette,
    dessin: &mut Dessin,
    etat: &mut Etat,
    autre: AutreSaisie,
    prefs: &Preferences,
) -> bool {
    let (saisie, titre, indice, ensuite) = match autre {
        AutreSaisie::Retrait(rang, en_largeur) => {
            let Some(retrait) = dessin.retraits.get_mut(rang) else {
                return false;
            };
            let saisie = if en_largeur {
                &mut retrait.largeur
            } else {
                &mut retrait.longueur
            };
            (
                saisie,
                textes::dessin_retrait(rang + 1, en_largeur),
                textes::DESSIN_EXEMPLE.traduit().to_string(),
                (!en_largeur).then_some(AutreSaisie::Retrait(rang, true)),
            )
        }
        AutreSaisie::Hauteur => (
            &mut dessin.hauteur_des_murs,
            textes::DESSIN_HAUTEUR_DES_MURS.traduit().to_string(),
            textes::DESSIN_HAUTEUR_EXEMPLE.traduit().to_string(),
            None,
        ),
        AutreSaisie::EpaisseurDesMurs => (
            &mut dessin.epaisseur_des_murs,
            textes::DESSIN_EPAISSEUR_DES_MURS.traduit().to_string(),
            sans_insecables(
                &Quantite::Longueur(EPAISSEUR_DES_MURS_PAR_DEFAUT).en_clair(None, prefs),
            ),
            None,
        ),
        AutreSaisie::EpaisseurDesCloisons => (
            &mut dessin.epaisseur_des_cloisons,
            textes::DESSIN_EPAISSEUR_DES_CLOISONS.traduit().to_string(),
            sans_insecables(&Quantite::Longueur(EPAISSEUR_PAR_DEFAUT).en_clair(None, prefs)),
            None,
        ),
        AutreSaisie::Ouverture(rang, champ) => {
            let Some(ouverture) = dessin.ouvertures.get_mut(rang) else {
                return false;
            };
            let porte = ouverture.est_une_porte(pouces_de(&ouverture.hauteur));
            let (saisie, dimension, indice, ensuite) = match champ {
                ChampDOuverture::Largeur => (
                    &mut ouverture.largeur,
                    textes::DESSIN_LARGEUR,
                    textes::DESSIN_EXEMPLE.traduit(),
                    Some(ChampDOuverture::Hauteur),
                ),
                ChampDOuverture::Hauteur => (
                    &mut ouverture.hauteur,
                    textes::DESSIN_HAUTEUR,
                    textes::DESSIN_EXEMPLE.traduit(),
                    Some(if porte {
                        ChampDOuverture::Position
                    } else {
                        ChampDOuverture::Allege
                    }),
                ),
                ChampDOuverture::Allege => (
                    &mut ouverture.allege,
                    textes::DESSIN_ALLEGE,
                    textes::DESSIN_ALLEGE_EXEMPLE.traduit(),
                    Some(ChampDOuverture::Position),
                ),
                ChampDOuverture::Position => (
                    &mut ouverture.position,
                    textes::DESSIN_POSITION,
                    textes::DESSIN_POSITION_EXEMPLE.traduit(),
                    None,
                ),
            };
            (
                saisie,
                textes::dessin_ouverture(rang + 1, dimension.traduit()),
                indice.to_string(),
                ensuite.map(|champ| AutreSaisie::Ouverture(rang, champ)),
            )
        }
        AutreSaisie::Objet(rang, en_profondeur) => {
            let Some(objet) = dessin.objets.get_mut(rang) else {
                return false;
            };
            let nom = textes::objet(objet.genre).traduit();
            let (largeur, profondeur, _) = objet.genre.dimensions();
            let (saisie, dimension, depart) = if en_profondeur {
                (&mut objet.profondeur, textes::DESSIN_PROFONDEUR, profondeur)
            } else {
                (&mut objet.largeur, textes::DESSIN_LARGEUR, largeur)
            };
            // L'indice : la dimension de départ, en pieds et pouces.
            let indice = sans_insecables(&Quantite::Longueur(depart).en_clair(None, prefs));
            (
                saisie,
                format!("{nom} : {}", dimension.traduit()),
                indice,
                (!en_profondeur).then_some(AutreSaisie::Objet(rang, true)),
            )
        }
        AutreSaisie::Cloison(rang, champ) => {
            // L'indice de l'épaisseur : celle que la pièce donne à ses cloisons.
            let defaut = dessin.resoudre(FORMAT_PAR_DEFAUT).epaisseur_des_cloisons;
            let Some(cloison) = dessin.cloisons.get_mut(rang) else {
                return false;
            };
            let (saisie, dimension, indice) = match champ {
                ChampDeCloison::Longueur => (
                    &mut cloison.longueur,
                    textes::DESSIN_LONGUEUR,
                    textes::DESSIN_EXEMPLE.traduit().to_string(),
                ),
                ChampDeCloison::Epaisseur => (
                    &mut cloison.epaisseur,
                    textes::DESSIN_EPAISSEUR,
                    sans_insecables(&Quantite::Longueur(defaut).en_clair(None, prefs)),
                ),
                ChampDeCloison::LargeurDePorte => {
                    let Some(porte) = cloison.porte.as_mut() else {
                        return false;
                    };
                    (
                        &mut porte.largeur,
                        textes::DESSIN_LARGEUR_DE_LA_PORTE,
                        sans_insecables(
                            &Quantite::Longueur(LARGEUR_DE_PORTE_PAR_DEFAUT).en_clair(None, prefs),
                        ),
                    )
                }
                ChampDeCloison::PositionDePorte => {
                    let Some(porte) = cloison.porte.as_mut() else {
                        return false;
                    };
                    (
                        &mut porte.position,
                        textes::DESSIN_DISTANCE_DU_BOUT_A,
                        textes::DESSIN_CENTREE.traduit().to_string(),
                    )
                }
            };
            (
                saisie,
                format!(
                    "{} {} : {}",
                    textes::DESSIN_CLOISON.traduit(),
                    rang + 1,
                    dimension.traduit()
                ),
                indice,
                None,
            )
        }
    };
    let mut modifie = false;
    let mut suivant = false;
    carte(ui, p, |ui| {
        let libelle = ui.label(RichText::new(titre).font(theme::police_grasse(13.5)));
        let champ = egui::TextEdit::singleline(saisie)
            .desired_width((ui.available_width() - 20.0).clamp(90.0, 260.0))
            .hint_text(indice);
        let reponse = ui.add(champ).labelled_by(libelle.id);
        cadre_de_saisie(ui, p, &reponse);
        if etat.viser_le_champ {
            etat.viser_le_champ = false;
            if !crate::plateforme::NAVIGATEUR {
                reponse.request_focus();
            }
        }
        if reponse.changed() {
            modifie = true;
        }
        if reponse.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            suivant = true;
        }
        ui.add_space(4.0);
        match pave_de_touches(ui, p, saisie, prefs) {
            Some(Frappe::Modifie) => modifie = true,
            Some(Frappe::Suivant) => suivant = true,
            None => {}
        }
    });
    if suivant {
        // De la première dimension à la seconde, puis l'éditeur se referme.
        etat.choisir_autre(ensuite);
    }
    modifie
}

/// La liste des retraits : ce qui se retire de la surface de la pièce.
fn retraits(
    ui: &mut Ui,
    p: &Palette,
    dessin: &mut Dessin,
    etat: &mut Etat,
    solution: &Solution,
    prefs: &Preferences,
) -> bool {
    let mut modifie = false;
    let mut a_enlever = None;
    ui.add_space(8.0);
    ui.label(
        RichText::new(textes::DESSIN_RETRAITS.traduit())
            .font(theme::police_grasse(13.0))
            .color(p.titre),
    );
    for rang in 0..dessin.retraits.len() {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(format!("{}.", rang + 1)).font(theme::police_grasse(13.5)));
            for en_largeur in [false, true] {
                let retrait = &dessin.retraits[rang];
                let saisie = if en_largeur {
                    retrait.largeur.trim()
                } else {
                    retrait.longueur.trim()
                };
                let texte = if saisie.is_empty() {
                    if en_largeur {
                        textes::DESSIN_LARGEUR.traduit().to_string()
                    } else {
                        textes::DESSIN_LONGUEUR.traduit().to_string()
                    }
                } else {
                    saisie_en_clair(saisie, prefs).unwrap_or_else(|| saisie.to_string())
                };
                let cible = AutreSaisie::Retrait(rang, en_largeur);
                let bouton = egui::Button::new(texte).selected(etat.autre_choisi == Some(cible));
                if ui.add(bouton).clicked() {
                    etat.choisir_autre(Some(cible));
                }
                if !en_largeur {
                    ui.label("×");
                }
            }
            match solution.retraits.get(rang) {
                Some(Some(aire)) if *aire > 0.0 => {
                    ui.label(
                        RichText::new(format!(
                            "= {}",
                            Quantite::Aire(*aire).en_clair(solution.modele, prefs)
                        ))
                        .color(p.texte_attenue),
                    );
                }
                _ => {}
            }
            if ui
                .add(egui::Button::new(textes::DESSIN_ENLEVER.traduit()))
                .clicked()
            {
                a_enlever = Some(rang);
            }
        });
    }
    if let Some(rang) = a_enlever {
        etat.memoriser(dessin);
        dessin.retraits.remove(rang);
        etat.choisir_autre(None);
        modifie = true;
    }
    if ui
        .add(egui::Button::new(
            textes::DESSIN_AJOUTER_UN_RETRAIT.traduit(),
        ))
        .clicked()
    {
        etat.memoriser(dessin);
        dessin.retraits.push(Retrait::default());
        etat.choisir_autre(Some(AutreSaisie::Retrait(dessin.retraits.len() - 1, false)));
        modifie = true;
    }
    modifie
}

/// Un petit bouton peint sur la planche; retourne vrai s'il est touché.
fn bouton_de_planche(
    ui: &Ui,
    painter: &egui::Painter,
    zone: Rect,
    texte: &str,
    nom: &str,
    mis_en_avant: bool,
    p: &Palette,
) -> bool {
    let reponse = ui.interact(
        zone,
        egui::Id::unique(("bouton-de-planche", nom)),
        Sense::click(),
    );
    reponse.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, nom));
    let (fond, encre) = if mis_en_avant {
        (p.selection, p.selection_texte)
    } else if reponse.hovered() {
        (theme::melanger(p.bouton, p.primaire, 0.12), p.texte)
    } else {
        (p.bouton, p.texte)
    };
    painter.rect_filled(zone, 8, fond.gamma_multiply(0.94));
    painter.rect_stroke(
        zone,
        8,
        Stroke::new(1.0, p.bouton_bord),
        egui::StrokeKind::Inside,
    );
    let police = if texte.chars().count() > 1 {
        theme::police_grasse(13.0)
    } else {
        theme::police_grasse(20.0)
    };
    painter.text(zone.center(), Align2::CENTER_CENTER, texte, police, encre);
    let clic = reponse.clicked();
    reponse.on_hover_text(nom);
    clic
}

/// La valeur en pouces d'une mesure inscrite, si elle est lisible.
fn pouces_de(saisie: &str) -> Option<f64> {
    noyau::analyse::analyser_longueur(saisie.trim(), FORMAT_PAR_DEFAUT)
        .ok()
        .filter(|v| v.est_longueur() && v.base > 0.0 && v.base.is_finite())
        .map(|v| v.base)
}

/// Les espaces insécables d'un texte affiché deviennent des espaces, pour
/// qu'il se relise comme une mesure tapée.
fn sans_insecables(texte: &str) -> String {
    texte
        .chars()
        .map(|c| if c as u32 == 0xA0 { ' ' } else { c })
        .collect()
}

/// Le symbole d'une porte — ses montants, son battant ouvert à angle droit et
/// l'arc de son ouverture — ou d'une fenêtre — un trait double dans le mur —,
/// entre `q0` et `q1`; `dedans` pointe vers l'intérieur de la pièce. Dans un
/// mur à son `epaisseur` (points), les jambages sont déjà tracés, et la
/// fenêtre montre ses deux faces et son vitrage.
fn dessiner_ouverture(
    painter: &egui::Painter,
    (q0, q1): (Pos2, Pos2),
    dedans: Vec2,
    porte: bool,
    encre: Color32,
    mur: Color32,
    epaisseur: f32,
) {
    let largeur = q0.distance(q1);
    if largeur < 1.0 {
        return;
    }
    let unite = (q1 - q0) / largeur;
    let a_l_epaisseur = epaisseur >= 2.0;
    if porte {
        if !a_l_epaisseur {
            for q in [q0, q1] {
                painter.line_segment([q - dedans * 3.5, q + dedans * 3.5], Stroke::new(2.0, mur));
            }
        }
        let battant = q0 + dedans * largeur;
        painter.line_segment([q0, battant], Stroke::new(2.2, encre));
        let arc: Vec<Pos2> = (0..=16)
            .map(|i| {
                let angle = std::f32::consts::FRAC_PI_2 * i as f32 / 16.0;
                q0 + (unite * angle.cos() + dedans * angle.sin()) * largeur
            })
            .collect();
        painter.add(egui::Shape::line(arc, Stroke::new(1.2, encre)));
    } else if a_l_epaisseur {
        for (part, largeur_du_trait) in [(0.0, 1.0), (0.5, 1.5), (1.0, 1.0)] {
            let decalage = -dedans * epaisseur * part;
            painter.line_segment(
                [q0 + decalage, q1 + decalage],
                Stroke::new(largeur_du_trait, encre),
            );
        }
    } else {
        for decalage in [-3.0, 3.0] {
            painter.line_segment(
                [q0 + dedans * decalage, q1 + dedans * decalage],
                Stroke::new(1.4, encre),
            );
        }
        painter.line_segment([q0, q1], Stroke::new(1.0, encre));
        for q in [q0, q1] {
            painter.line_segment([q - dedans * 3.5, q + dedans * 3.5], Stroke::new(2.0, mur));
        }
    }
}

/// Une mesure inscrite, en clair pour un bouton, ou le mot qui la nomme si
/// elle est vide.
fn mesure_du_bouton(saisie: &str, vide: &str, prefs: &Preferences) -> String {
    let saisie = saisie.trim();
    if saisie.is_empty() {
        vide.to_string()
    } else {
        saisie_en_clair(saisie, prefs).unwrap_or_else(|| saisie.to_string())
    }
}

/// Les murs : leur hauteur, puis les portes et les fenêtres à retirer. La
/// surface des murs s'ajoute aux résultats de la pièce.
fn murs(
    ui: &mut Ui,
    p: &Palette,
    dessin: &mut Dessin,
    etat: &mut Etat,
    solution: &Solution,
    prefs: &Preferences,
) -> bool {
    let mut modifie = false;
    ui.add_space(8.0);
    ui.label(
        RichText::new(textes::DESSIN_MURS.traduit())
            .font(theme::police_grasse(13.0))
            .color(p.titre),
    );
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(textes::DESSIN_HAUTEUR_DES_MURS.traduit()).size(13.5));
        let texte = mesure_du_bouton(
            &dessin.hauteur_des_murs,
            textes::DESSIN_HAUTEUR_EXEMPLE.traduit(),
            prefs,
        );
        let bouton =
            egui::Button::new(texte).selected(etat.autre_choisi == Some(AutreSaisie::Hauteur));
        if ui.add(bouton).clicked() {
            etat.choisir_autre(Some(AutreSaisie::Hauteur));
        }
    });
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(textes::DESSIN_EPAISSEUR_DES_MURS.traduit()).size(13.5));
        let texte = longueur_en_clair(solution.epaisseur_des_murs, solution, prefs);
        let cible = AutreSaisie::EpaisseurDesMurs;
        let bouton = egui::Button::new(texte).selected(etat.autre_choisi == Some(cible));
        if ui.add(bouton).clicked() {
            etat.choisir_autre(Some(cible));
        }
    });
    if solution.hauteur_invalide {
        ui.label(
            RichText::new(textes::DESSIN_HAUTEUR_INVALIDE.traduit())
                .size(12.5)
                .color(p.erreur),
        );
    }
    let mut a_enlever = None;
    for rang in 0..dessin.ouvertures.len() {
        ui.horizontal_wrapped(|ui| {
            let ouverture = &dessin.ouvertures[rang];
            let nom = if ouverture.est_une_porte(pouces_de(&ouverture.hauteur)) {
                textes::DESSIN_PORTE
            } else {
                textes::DESSIN_FENETRE
            };
            ui.label(
                RichText::new(format!("{}. {}", rang + 1, nom.traduit()))
                    .font(theme::police_grasse(13.5)),
            );
            // Son mur, et le bouton pour la placer sur un mur en le touchant.
            let (ou, couleur) = match ouverture.cote {
                Some(cote) => (textes::dessin_sur_le_cote(cote + 1), p.texte_attenue),
                None => (
                    textes::DESSIN_PAS_PLACEE.traduit().to_string(),
                    p.barre_accent,
                ),
            };
            ui.label(RichText::new(ou).color(couleur));
            let en_placement = etat.ouverture_a_placer == Some(rang);
            if ui
                .add(egui::Button::new(textes::DESSIN_PLACER.traduit()).selected(en_placement))
                .clicked()
            {
                etat.ouverture_a_placer = if en_placement { None } else { Some(rang) };
                etat.objet_a_placer = None;
                etat.note_a_placer = false;
                etat.cloison_a_tracer = false;
            }
            for champ in [ChampDOuverture::Largeur, ChampDOuverture::Hauteur] {
                let ouverture = &dessin.ouvertures[rang];
                let (saisie, vide) = if champ == ChampDOuverture::Hauteur {
                    (&ouverture.hauteur, textes::DESSIN_HAUTEUR.traduit())
                } else {
                    (&ouverture.largeur, textes::DESSIN_LARGEUR.traduit())
                };
                let cible = AutreSaisie::Ouverture(rang, champ);
                let bouton = egui::Button::new(mesure_du_bouton(saisie, vide, prefs))
                    .selected(etat.autre_choisi == Some(cible));
                if ui.add(bouton).clicked() {
                    etat.choisir_autre(Some(cible));
                }
                if champ == ChampDOuverture::Largeur {
                    ui.label("×");
                }
            }
            // L'allège d'une fenêtre : la hauteur de son bas.
            let ouverture = &dessin.ouvertures[rang];
            if !ouverture.est_une_porte(pouces_de(&ouverture.hauteur)) {
                let cible = AutreSaisie::Ouverture(rang, ChampDOuverture::Allege);
                let texte = textes::dessin_allege(&mesure_du_bouton(
                    &ouverture.allege,
                    textes::DESSIN_ALLEGE_DEFAUT.traduit(),
                    prefs,
                ));
                if ui
                    .add(egui::Button::new(texte).selected(etat.autre_choisi == Some(cible)))
                    .clicked()
                {
                    etat.choisir_autre(Some(cible));
                }
            }
            // La distance au premier coin de son mur; vide : centrée.
            let position = dessin.ouvertures[rang].position.trim();
            let texte = if position.is_empty() {
                textes::DESSIN_CENTREE.traduit().to_string()
            } else {
                textes::dessin_a_du_coin(&mesure_du_bouton(position, "", prefs))
            };
            let cible = AutreSaisie::Ouverture(rang, ChampDOuverture::Position);
            if ui
                .add(egui::Button::new(texte).selected(etat.autre_choisi == Some(cible)))
                .clicked()
            {
                etat.choisir_autre(Some(cible));
            }
            if let Some(Some(aire)) = solution.ouvertures.get(rang)
                && *aire > 0.0
            {
                ui.label(
                    RichText::new(format!(
                        "= {}",
                        Quantite::Aire(*aire).en_clair(solution.modele, prefs)
                    ))
                    .color(p.texte_attenue),
                );
            }
            if ui
                .add(egui::Button::new(textes::DESSIN_ENLEVER.traduit()))
                .clicked()
            {
                a_enlever = Some(rang);
            }
        });
    }
    if let Some(rang) = a_enlever {
        etat.memoriser(dessin);
        dessin.ouvertures.remove(rang);
        etat.choisir_autre(None);
        etat.ouverture_a_placer = None;
        modifie = true;
    }
    // Ce qui empêche de calculer la surface des murs.
    if solution.hauteur.is_some() && matches!(solution.etat, EtatDeLaForme::Complete { .. }) {
        let message = match (solution.aire_des_ouvertures(), solution.murs()) {
            (None, _) => Some(textes::DESSIN_OUVERTURE_INCOMPLETE),
            (Some(_), None) => Some(textes::DESSIN_OUVERTURES_TROP_GRANDES),
            _ => None,
        };
        if let Some(message) = message {
            ui.label(RichText::new(message.traduit()).size(12.5).color(p.erreur));
        }
    }
    // Une porte (32 po × 80 po) ou une fenêtre (3 pi × 4 pi) à ajuster ensuite.
    let pieds = noyau::etiquettes::symbole(Format::PiDec);
    let pouces = noyau::etiquettes::symbole(Format::PoDec);
    ui.horizontal_wrapped(|ui| {
        let modeles = [
            (
                textes::DESSIN_AJOUTER_PORTE,
                GenreDOuverture::Porte,
                format!("2 {pieds} 8 {pouces}"),
                format!("6 {pieds} 8 {pouces}"),
            ),
            (
                textes::DESSIN_AJOUTER_FENETRE,
                GenreDOuverture::Fenetre,
                format!("3 {pieds}"),
                format!("4 {pieds}"),
            ),
        ];
        for (nom, genre, largeur, hauteur) in modeles {
            if ui.add(egui::Button::new(nom.traduit())).clicked() {
                etat.memoriser(dessin);
                // Sur le côté choisi, s'il y en a un, centrée; puis on peut
                // toucher un autre mur pour l'y placer.
                let cote = etat.choisi;
                dessin.ouvertures.push(Ouverture {
                    largeur,
                    hauteur,
                    genre: Some(genre),
                    cote,
                    position: String::new(),
                    allege: String::new(),
                });
                let rang = dessin.ouvertures.len() - 1;
                etat.choisir_autre(Some(AutreSaisie::Ouverture(rang, ChampDOuverture::Largeur)));
                etat.ouverture_a_placer = Some(rang);
                etat.objet_a_placer = None;
                etat.note_a_placer = false;
                etat.cloison_a_tracer = false;
                modifie = true;
            }
        }
    });
    modifie
}

/// Les objets de la pièce : l'escalier et le mobilier, leurs dimensions, et
/// les boutons pour en ajouter. Retourne vrai si le dessin a changé.
fn objets(
    ui: &mut Ui,
    p: &Palette,
    dessin: &mut Dessin,
    etat: &mut Etat,
    solution: &Solution,
    prefs: &Preferences,
) -> bool {
    let mut modifie = false;
    ui.add_space(8.0);
    ui.label(
        RichText::new(textes::DESSIN_OBJETS.traduit())
            .font(theme::police_grasse(13.0))
            .color(p.titre),
    );
    let (mut a_pivoter, mut a_enlever) = (None, None);
    for rang in 0..dessin.objets.len() {
        ui.horizontal_wrapped(|ui| {
            let objet = &dessin.objets[rang];
            let choisi = etat.objet_choisi == Some(rang);
            let nom = RichText::new(format!(
                "{}. {}",
                rang + 1,
                textes::objet(objet.genre).traduit()
            ))
            .font(theme::police_grasse(13.5));
            if ui.add(egui::Button::new(nom).selected(choisi)).clicked() {
                etat.objet_choisi = if choisi { None } else { Some(rang) };
            }
            let (largeur, profondeur) = solution.objets.get(rang).copied().unwrap_or_default();
            for (en_profondeur, saisie, valeur) in [
                (false, &objet.largeur, largeur),
                (true, &objet.profondeur, profondeur),
            ] {
                let texte = if saisie.trim().is_empty() {
                    longueur_en_clair(valeur, solution, prefs)
                } else {
                    mesure_du_bouton(saisie, "", prefs)
                };
                let cible = AutreSaisie::Objet(rang, en_profondeur);
                if ui
                    .add(egui::Button::new(texte).selected(etat.autre_choisi == Some(cible)))
                    .clicked()
                {
                    etat.choisir_autre(Some(cible));
                    etat.objet_choisi = Some(rang);
                }
                if !en_profondeur {
                    ui.label("×");
                }
            }
            if ui
                .add(egui::Button::new(textes::DESSIN_PIVOTER.traduit()))
                .clicked()
            {
                a_pivoter = Some(rang);
            }
            if ui
                .add(egui::Button::new(textes::DESSIN_ENLEVER.traduit()))
                .clicked()
            {
                a_enlever = Some(rang);
            }
        });
    }
    if let Some(rang) = a_pivoter {
        etat.memoriser(dessin);
        let objet = &mut dessin.objets[rang];
        objet.rotation = (objet.rotation + 1) % 4;
        etat.objet_choisi = Some(rang);
        modifie = true;
    }
    if let Some(rang) = a_enlever {
        etat.memoriser(dessin);
        dessin.objets.remove(rang);
        etat.objet_choisi = None;
        etat.objet_tire = None;
        etat.choisir_autre(None);
        modifie = true;
    }
    // Un bouton par genre : on touche ensuite la pièce pour y poser l'objet.
    ui.horizontal_wrapped(|ui| {
        for genre in GenreDObjet::TOUS {
            let en_attente = etat.objet_a_placer == Some(genre);
            let bouton = egui::Button::new(format!("+ {}", textes::objet(genre).traduit()))
                .selected(en_attente);
            if ui.add(bouton).clicked() {
                etat.objet_a_placer = if en_attente { None } else { Some(genre) };
                etat.ouverture_a_placer = None;
                etat.note_a_placer = false;
                etat.cloison_a_tracer = false;
            }
        }
    });
    modifie
}

/// Les cloisons de la pièce : leur longueur, leur épaisseur et leur porte, et
/// le bouton pour en tracer une. Retourne vrai si le dessin a changé.
fn cloisons(
    ui: &mut Ui,
    p: &Palette,
    dessin: &mut Dessin,
    etat: &mut Etat,
    solution: &Solution,
    prefs: &Preferences,
) -> bool {
    let mut modifie = false;
    ui.add_space(8.0);
    ui.label(
        RichText::new(textes::DESSIN_CLOISONS.traduit())
            .font(theme::police_grasse(13.0))
            .color(p.titre),
    );
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(textes::DESSIN_EPAISSEUR_DES_CLOISONS.traduit()).size(13.5));
        let texte = longueur_en_clair(solution.epaisseur_des_cloisons, solution, prefs);
        let cible = AutreSaisie::EpaisseurDesCloisons;
        let bouton = egui::Button::new(texte).selected(etat.autre_choisi == Some(cible));
        if ui.add(bouton).clicked() {
            etat.choisir_autre(Some(cible));
        }
    });
    let mut geste = None;
    for rang in 0..dessin.cloisons.len() {
        let cloison = &dessin.cloisons[rang];
        let mesuree = solution.cloisons.get(rang).copied().flatten();
        let epaisseur = mesuree.map_or(solution.epaisseur_des_cloisons, |c| c.epaisseur);
        ui.horizontal_wrapped(|ui| {
            let choisie = etat.cloison_choisie == Some(rang);
            let nom = RichText::new(format!(
                "{}. {}",
                rang + 1,
                textes::DESSIN_CLOISON.traduit()
            ))
            .font(theme::police_grasse(13.5));
            if ui.add(egui::Button::new(nom).selected(choisie)).clicked() {
                etat.cloison_choisie = if choisie { None } else { Some(rang) };
            }
            let boutons = [
                (
                    ChampDeCloison::Longueur,
                    mesure_du_bouton(&cloison.longueur, "?", prefs),
                ),
                (
                    ChampDeCloison::Epaisseur,
                    longueur_en_clair(epaisseur, solution, prefs),
                ),
            ];
            for (champ, texte) in boutons {
                if champ == ChampDeCloison::Epaisseur {
                    ui.label(textes::DESSIN_EP.traduit());
                }
                let cible = AutreSaisie::Cloison(rang, champ);
                if ui
                    .add(egui::Button::new(texte).selected(etat.autre_choisi == Some(cible)))
                    .clicked()
                {
                    etat.choisir_autre(Some(cible));
                    etat.cloison_choisie = Some(rang);
                }
            }
            if cloison.porte.is_none()
                && ui
                    .add(egui::Button::new(textes::DESSIN_AJOUTER_PORTE.traduit()))
                    .clicked()
            {
                geste = Some((rang, GesteSurUneCloison::AjouterUnePorte));
            }
            if ui
                .add(egui::Button::new(textes::DESSIN_ENLEVER.traduit()))
                .clicked()
            {
                geste = Some((rang, GesteSurUneCloison::Enlever));
            }
        });
        // Sa porte : sa largeur, sa place depuis le bout A, son côté.
        if let Some(porte) = &cloison.porte {
            let mesuree = mesuree.and_then(|c| c.porte);
            ui.horizontal_wrapped(|ui| {
                ui.add_space(16.0);
                ui.label(RichText::new(textes::DESSIN_PORTE.traduit()).size(13.5));
                let largeur = mesuree.map_or(LARGEUR_DE_PORTE_PAR_DEFAUT, |q| q.fin - q.debut);
                let place = match mesuree {
                    Some(q) if !porte.position.trim().is_empty() => {
                        textes::dessin_du_bout_a(&longueur_en_clair(q.debut, solution, prefs))
                    }
                    _ => textes::DESSIN_CENTREE.traduit().to_string(),
                };
                for (champ, texte) in [
                    (
                        ChampDeCloison::LargeurDePorte,
                        longueur_en_clair(largeur, solution, prefs),
                    ),
                    (ChampDeCloison::PositionDePorte, place),
                ] {
                    let cible = AutreSaisie::Cloison(rang, champ);
                    if ui
                        .add(egui::Button::new(texte).selected(etat.autre_choisi == Some(cible)))
                        .clicked()
                    {
                        etat.choisir_autre(Some(cible));
                        etat.cloison_choisie = Some(rang);
                    }
                }
                if ui
                    .add(egui::Button::new(textes::DESSIN_INVERSER.traduit()))
                    .clicked()
                {
                    geste = Some((rang, GesteSurUneCloison::Inverser));
                }
                if ui
                    .add(egui::Button::new(textes::DESSIN_SANS_PORTE.traduit()))
                    .clicked()
                {
                    geste = Some((rang, GesteSurUneCloison::SansPorte));
                }
            });
        }
    }
    if let Some((rang, geste)) = geste {
        etat.memoriser(dessin);
        match geste {
            GesteSurUneCloison::Enlever => {
                dessin.cloisons.remove(rang);
                etat.cloison_choisie = None;
                etat.cloison_tiree = None;
                etat.choisir_autre(None);
            }
            GesteSurUneCloison::AjouterUnePorte => {
                dessin.cloisons[rang].porte = Some(PorteDeCloison::default());
                etat.cloison_choisie = Some(rang);
            }
            GesteSurUneCloison::Inverser => {
                if let Some(porte) = dessin.cloisons[rang].porte.as_mut() {
                    porte.inversee = !porte.inversee;
                }
                etat.cloison_choisie = Some(rang);
            }
            GesteSurUneCloison::SansPorte => {
                dessin.cloisons[rang].porte = None;
                etat.choisir_autre(None);
            }
        }
        modifie = true;
    }
    let bouton = egui::Button::new(textes::DESSIN_AJOUTER_UNE_CLOISON.traduit())
        .selected(etat.cloison_a_tracer);
    if ui.add(bouton).clicked() {
        etat.cloison_a_tracer = !etat.cloison_a_tracer;
        etat.objet_a_placer = None;
        etat.ouverture_a_placer = None;
        etat.note_a_placer = false;
    }
    modifie
}

/// Les notes écrites sur le plan : leur texte, et le bouton pour en ajouter.
/// Retourne vrai si le dessin a changé.
fn notes(ui: &mut Ui, p: &Palette, dessin: &mut Dessin, etat: &mut Etat) -> bool {
    let mut modifie = false;
    ui.add_space(8.0);
    ui.label(
        RichText::new(textes::DESSIN_NOTES.traduit())
            .font(theme::police_grasse(13.0))
            .color(p.titre),
    );
    let mut a_enlever = None;
    for rang in 0..dessin.notes.len() {
        ui.horizontal(|ui| {
            let numero =
                ui.label(RichText::new(format!("{}.", rang + 1)).font(theme::police_grasse(13.5)));
            let avant = dessin.notes[rang].texte.clone();
            let champ = egui::TextEdit::singleline(&mut dessin.notes[rang].texte)
                .desired_width((ui.available_width() - 90.0).clamp(120.0, 320.0))
                .char_limit(120)
                .hint_text(textes::DESSIN_NOTE_INDICE.traduit());
            let reponse = ui.add(champ).labelled_by(numero.id);
            cadre_de_saisie(ui, p, &reponse);
            if etat.viser_la_note == Some(rang) {
                etat.viser_la_note = None;
                reponse.request_focus();
            }
            if reponse.gained_focus() {
                etat.note_choisie = Some(rang);
                etat.choisir(None);
            }
            if reponse.changed() {
                // Une seule étape d'« Annuler » pour ce qui s'écrit d'affilée.
                if etat.note_ecrite != Some(rang) {
                    let mut precedent = dessin.clone();
                    precedent.notes[rang].texte = avant;
                    etat.memoriser(&precedent);
                    etat.note_ecrite = Some(rang);
                }
                modifie = true;
            }
            if reponse.lost_focus() && etat.note_ecrite == Some(rang) {
                etat.note_ecrite = None;
            }
            if ui
                .add(egui::Button::new(textes::DESSIN_ENLEVER.traduit()))
                .clicked()
            {
                a_enlever = Some(rang);
            }
        });
    }
    if let Some(rang) = a_enlever {
        etat.memoriser(dessin);
        dessin.notes.remove(rang);
        etat.note_choisie = None;
        etat.note_tiree = None;
        etat.note_ecrite = None;
        modifie = true;
    }
    let bouton =
        egui::Button::new(textes::DESSIN_AJOUTER_UNE_NOTE.traduit()).selected(etat.note_a_placer);
    if ui.add(bouton).clicked() {
        etat.note_a_placer = !etat.note_a_placer;
        etat.objet_a_placer = None;
        etat.ouverture_a_placer = None;
        etat.cloison_a_tracer = false;
    }
    modifie
}

/// Les résultats d'une pièce complète : surface, périmètre, diagonale d'un
/// rectangle, surface nette, puis les murs. `detail` ajoute une ligne par
/// retrait et la hauteur des murs (pour l'export).
pub(crate) fn resultats_de_la_piece(
    dessin: &Dessin,
    solution: &Solution,
    prefs: &Preferences,
    detail: bool,
) -> Vec<(String, String)> {
    let EtatDeLaForme::Complete { aire, perimetre } = solution.etat else {
        return Vec::new();
    };
    let aire_en_clair = |po2: f64| Quantite::Aire(po2).en_clair(solution.modele, prefs);
    let longueur = |pouces: f64| longueur_en_clair(pouces, solution, prefs);
    let mut lignes = vec![
        (
            textes::DESSIN_SURFACE.traduit().to_string(),
            aire_en_clair(aire),
        ),
        (
            textes::DESSIN_PERIMETRE.traduit().to_string(),
            longueur(perimetre),
        ),
    ];
    // Un rectangle est d'équerre quand ses deux diagonales ont cette longueur.
    if let [Some(a), Some(b), _, _] = solution
        .mesures
        .iter()
        .map(|m| m.pouces())
        .collect::<Vec<_>>()[..]
        && dessin.cotes.iter().all(|cote| !cote.sens.en_biais())
    {
        lignes.push((
            textes::DESSIN_DIAGONALE.traduit().to_string(),
            longueur(a.hypot(b)),
        ));
    }
    if detail {
        for (rang, (retrait, aire_du_retrait)) in
            dessin.retraits.iter().zip(&solution.retraits).enumerate()
        {
            let Some(aire_du_retrait) = aire_du_retrait.filter(|a| *a > 0.0) else {
                continue;
            };
            lignes.push((
                textes::dessin_retrait_numero(rang + 1),
                format!(
                    "{} × {} = {}",
                    mesure_du_bouton(&retrait.longueur, "?", prefs),
                    mesure_du_bouton(&retrait.largeur, "?", prefs),
                    aire_en_clair(aire_du_retrait)
                ),
            ));
        }
    }
    if let (Some(nette), Some(retiree)) = (solution.aire_nette(), solution.aire_des_retraits())
        && retiree > 0.0
    {
        lignes.push((
            textes::DESSIN_SURFACE_NETTE.traduit().to_string(),
            aire_en_clair(nette),
        ));
    }
    if let Some(murs) = solution.murs() {
        if detail && let Some(hauteur) = solution.hauteur {
            lignes.push((
                textes::DESSIN_HAUTEUR_DES_MURS.traduit().to_string(),
                longueur(hauteur),
            ));
        }
        lignes.push((
            textes::DESSIN_SURFACE_DES_MURS.traduit().to_string(),
            aire_en_clair(murs.brute),
        ));
        if murs.ouvertures > 0.0 {
            lignes.push((
                textes::DESSIN_PORTES_ET_FENETRES.traduit().to_string(),
                aire_en_clair(murs.ouvertures),
            ));
            lignes.push((
                textes::DESSIN_MURS_NETS.traduit().to_string(),
                aire_en_clair(murs.nette),
            ));
        }
    }
    let longueur_des_cloisons = solution.longueur_des_cloisons();
    if longueur_des_cloisons > 0.0 {
        lignes.push((
            textes::DESSIN_LONGUEUR_DES_CLOISONS.traduit().to_string(),
            longueur(longueur_des_cloisons),
        ));
        if let Some(surface) = solution.surface_des_cloisons() {
            lignes.push((
                textes::DESSIN_SURFACE_DES_CLOISONS.traduit().to_string(),
                aire_en_clair(surface),
            ));
        }
    }
    lignes
}

/// Où en est la forme : ce qu'il reste à faire, ou la surface et le périmètre.
fn bilan(
    ui: &mut Ui,
    p: &Palette,
    dessin: &Dessin,
    etat: &mut Etat,
    solution: &Solution,
    prefs: &Preferences,
) -> Option<Action> {
    let mut action = None;
    ui.add_space(8.0);
    let consigne = |ui: &mut Ui, texte: &str, couleur: Color32| {
        ui.label(RichText::new(texte).size(13.5).color(couleur));
    };
    match solution.etat {
        EtatDeLaForme::Vide => consigne(ui, textes::DESSIN_ETAPE_TRACER.traduit(), p.texte),
        EtatDeLaForme::Ouverte => consigne(ui, textes::DESSIN_ETAPE_FERMER.traduit(), p.texte),
        EtatDeLaForme::MesureInvalide => {
            consigne(ui, textes::DESSIN_MESURE_INVALIDE.traduit(), p.erreur);
        }
        EtatDeLaForme::Incomplete(manquantes) => {
            consigne(ui, &textes::dessin_etape_mesurer(manquantes), p.texte);
        }
        EtatDeLaForme::NeFermePas(ecart) => consigne(
            ui,
            &textes::dessin_ne_ferme_pas(&longueur_en_clair(ecart, solution, prefs)),
            p.erreur,
        ),
        EtatDeLaForme::SeCroise => consigne(ui, textes::DESSIN_SE_CROISE.traduit(), p.erreur),
        EtatDeLaForme::Complete { aire, perimetre } => {
            let resultats = resultats_de_la_piece(dessin, solution, prefs, false);
            // Les retraits : ce qui empêche de calculer la surface nette.
            let nette = solution.aire_nette();
            let aire_utile = nette.unwrap_or(aire);
            if !dessin.retraits.is_empty() {
                match (nette, solution.aire_des_retraits()) {
                    (None, Some(_)) => {
                        consigne(ui, textes::DESSIN_RETRAITS_TROP_GRANDS.traduit(), p.erreur);
                    }
                    (None, None) => {
                        consigne(ui, textes::DESSIN_RETRAIT_INCOMPLET.traduit(), p.erreur);
                    }
                    _ => {}
                }
            }
            carte(ui, p, |ui| {
                for (libelle, valeur) in &resultats {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(libelle)
                                .font(theme::police_grasse(13.5))
                                .color(p.titre),
                        );
                        ui.label(
                            RichText::new(valeur)
                                .font(theme::police_grasse(17.0))
                                .color(p.texte),
                        );
                    });
                }
                ui.add_space(4.0);
                if ui
                    .add(bouton_principal(p, textes::DESSIN_AU_RUBAN.traduit(), 13.5))
                    .clicked()
                {
                    let entrees = solution
                        .mesures
                        .iter()
                        .enumerate()
                        .filter_map(|(i, m)| {
                            m.pouces().map(|pouces| {
                                (
                                    textes::dessin_cote(i + 1, dessin.cotes.len()),
                                    longueur_en_clair(pouces, solution, prefs),
                                )
                            })
                        })
                        .collect();
                    action = Some(Action::Consigner {
                        titre: String::new(),
                        entrees,
                        resultats: resultats.clone(),
                    });
                }
                if ui
                    .add(egui::Button::new(textes::DESSIN_VERS_UN_OUTIL.traduit()))
                    .clicked()
                {
                    etat.outils_ouverts = !etat.outils_ouverts;
                }
                if etat.outils_ouverts {
                    let (longueur, largeur, exact) =
                        noyau::dessin::rectangle_equivalent(aire_utile, perimetre);
                    let metrique = solution.modele.and_then(Format::systeme)
                        == Some(noyau::unites::Systeme::Metrique);
                    let decimal = noyau::format::separateur_decimal(prefs);
                    let a_taper =
                        |pouces: f64| noyau::dessin::longueur_a_taper(pouces, metrique, decimal);
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(textes::dessin_rectangle_equivalent(
                            &a_taper(longueur),
                            &a_taper(largeur),
                            exact,
                        ))
                        .size(12.5)
                        .color(p.texte_attenue),
                    );
                    ui.add_space(2.0);
                    for (c, rang_longueur, rang_largeur, rang_hauteur) in
                        noyau::dessin::outils_d_une_piece()
                    {
                        let libelle = textes::avec_tiret(c.metier.nom_court(), c.nom.traduit());
                        if ui.add(egui::Button::new(libelle)).clicked() {
                            etat.outils_ouverts = false;
                            let mut champs = vec![
                                (rang_longueur, a_taper(longueur)),
                                (rang_largeur, a_taper(largeur)),
                            ];
                            // La hauteur des murs, si l'outil en a une.
                            if let (Some(rang), Some(hauteur)) = (rang_hauteur, solution.hauteur) {
                                champs.push((rang, a_taper(hauteur)));
                            }
                            action = Some(Action::VersOutil { id: c.id, champs });
                        }
                    }
                }
            });
        }
    }
    action
}

/// Ce qu'une touche du pavé a fait.
enum Frappe {
    Modifie,
    Suivant,
}

/// Une touche du pavé de mesure.
#[derive(Clone, Copy)]
enum ToucheDeMesure {
    Chiffre(char),
    Decimale,
    /// Une unité : son symbole dans la langue en cours.
    Unite(&'static str),
    Barre,
    /// Une fraction de pouce : numérateur, dénominateur.
    Fraction(u32, u32),
    Espace,
    Reculer,
    Suivant,
}

/// Applique une touche à la mesure en cours d'écriture.
fn frapper(saisie: &mut String, touche: ToucheDeMesure, decimal: char) {
    match touche {
        ToucheDeMesure::Chiffre(c) => saisie.push(c),
        ToucheDeMesure::Decimale => saisie.push(decimal),
        ToucheDeMesure::Barre => saisie.push('/'),
        ToucheDeMesure::Espace => {
            if !saisie.is_empty() && !saisie.ends_with(' ') {
                saisie.push(' ');
            }
        }
        ToucheDeMesure::Unite(symbole) => {
            // « 12 » puis « pi » donne « 12 pi », prêt pour la suite.
            let debut = saisie.trim_end().to_string();
            if !debut.is_empty() {
                *saisie = format!("{debut} {symbole} ");
            }
        }
        ToucheDeMesure::Fraction(num, den) => {
            // « 12 pi 6 » puis « 1/2 » donne « 12 pi 6 1/2 po » : une fraction
            // est une fraction de pouce, et elle se place après les pouces.
            let pouces = noyau::etiquettes::symbole(Format::PoDec);
            let mut debut = saisie.trim_end().to_string();
            if let Some(sans) = debut.strip_suffix(pouces) {
                debut = sans.trim_end().to_string();
            }
            // Une fraction déjà inscrite est remplacée : on corrige d'un toucher.
            let dernier_mot = debut.rfind(' ').map_or(0, |i| i + 1);
            if debut[dernier_mot..].contains('/') {
                debut.truncate(dernier_mot);
                debut = debut.trim_end().to_string();
            }
            *saisie = if debut.is_empty() {
                format!("{num}/{den} {pouces} ")
            } else {
                format!("{debut} {num}/{den} {pouces} ")
            };
        }
        ToucheDeMesure::Reculer => {
            let debut = saisie.trim_end();
            // Une unité s'efface d'un coup, un chiffre à la fois.
            let coupe = match debut.rfind(' ') {
                Some(i) if debut[i + 1..].chars().all(char::is_alphabetic) => i,
                None if !debut.is_empty() && debut.chars().all(char::is_alphabetic) => 0,
                _ => debut.char_indices().next_back().map_or(0, |(i, _)| i),
            };
            *saisie = debut[..coupe].trim_end().to_string();
        }
        ToucheDeMesure::Suivant => {}
    }
}

/// Le pavé de touches sous le champ : les fractions de pouce, puis les chiffres,
/// `Pieds`, `Pouces` et les mètres. On inscrit une mesure sans le clavier du
/// téléphone.
fn pave_de_touches(
    ui: &mut Ui,
    p: &Palette,
    saisie: &mut String,
    prefs: &Preferences,
) -> Option<Frappe> {
    use ToucheDeMesure::{Barre, Chiffre, Decimale, Espace, Fraction, Reculer, Suivant, Unite};
    let decimal = noyau::format::separateur_decimal(prefs);
    let pieds = noyau::etiquettes::symbole(Format::PiDec);
    let pouces = noyau::etiquettes::symbole(Format::PoDec);
    let mut frappe = None;
    // Les tailles se calculent une fois, sur la largeur offerte : chaque touche
    // reçoit exactement la sienne, et aucune rangée ne déborde de l'écran.
    let disponible = ui.available_width();
    // Les fractions de pouce, à la précision choisie dans les Préférences, huit
    // par rangée comme la bande de fractions du clavier (cinq sur un écran étroit).
    let fractions = noyau::touches::fractions_rapides(prefs.resolution_valide());
    let par_rangee = if (disponible - 7.0 * 4.0) / 8.0 >= 34.0 {
        8
    } else {
        5
    };
    let largeur_fraction =
        ((disponible - 4.0 * (par_rangee as f32 - 1.0)) / par_rangee as f32).clamp(28.0, 64.0);
    let police_fraction = (largeur_fraction * 0.31).clamp(10.0, 13.0);
    for rangee in fractions.chunks(par_rangee) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            ui.spacing_mut().button_padding = Vec2::new(1.0, 1.0);
            for &(num, den) in rangee {
                let libelle = format!("{num}/{den}");
                let bouton = egui::Button::new(
                    RichText::new(&libelle).font(theme::police_grasse(police_fraction)),
                );
                if ui
                    .add_sized(Vec2::new(largeur_fraction, 32.0), bouton)
                    .clicked()
                {
                    frapper(saisie, Fraction(num, den), decimal);
                    frappe = Some(Frappe::Modifie);
                }
            }
        });
        ui.add_space(1.0);
    }
    ui.add_space(3.0);
    let rangees: [&[(ToucheDeMesure, f32)]; 4] = [
        &[
            (Chiffre('7'), 1.0),
            (Chiffre('8'), 1.0),
            (Chiffre('9'), 1.0),
            (Unite(pieds), 1.0),
            (Reculer, 1.0),
        ],
        &[
            (Chiffre('4'), 1.0),
            (Chiffre('5'), 1.0),
            (Chiffre('6'), 1.0),
            (Unite(pouces), 1.0),
            (Unite("m"), 1.0),
        ],
        &[
            (Chiffre('1'), 1.0),
            (Chiffre('2'), 1.0),
            (Chiffre('3'), 1.0),
            (Barre, 1.0),
            (Espace, 1.0),
        ],
        &[(Chiffre('0'), 1.0), (Decimale, 1.0), (Suivant, 3.0)],
    ];
    let ecart = 5.0;
    let largeur = ((disponible - 4.0 * ecart) / 5.0).clamp(36.0, 84.0);
    for rangee in rangees {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = ecart;
            ui.spacing_mut().button_padding = Vec2::new(2.0, 1.0);
            for &(touche, parts) in rangee {
                let libelle = match touche {
                    Chiffre(c) => c.to_string(),
                    Decimale => decimal.to_string(),
                    Unite(symbole) if symbole == pieds => {
                        textes::DESSIN_TOUCHE_PIEDS.traduit().to_string()
                    }
                    Unite(symbole) if symbole == pouces => {
                        textes::DESSIN_TOUCHE_POUCES.traduit().to_string()
                    }
                    Unite(symbole) => symbole.to_string(),
                    Barre => "/".to_string(),
                    Fraction(num, den) => format!("{num}/{den}"),
                    Espace => textes::DESSIN_TOUCHE_ESPACE.traduit().to_string(),
                    Reculer => textes::DESSIN_TOUCHE_RECULER.traduit().to_string(),
                    Suivant => textes::DESSIN_COTE_SUIVANT.traduit().to_string(),
                };
                let taille = Vec2::new(largeur * parts + ecart * (parts - 1.0), 38.0);
                let bouton = if matches!(touche, Suivant) {
                    bouton_principal(p, &libelle, 14.5)
                } else {
                    // Les mots (Pieds, Pouces) un peu plus petits que les chiffres.
                    let taille = if libelle.chars().count() > 2 {
                        14.0
                    } else {
                        15.5
                    };
                    egui::Button::new(RichText::new(&libelle).font(theme::police_grasse(taille)))
                };
                if ui.add_sized(taille, bouton).clicked() {
                    if matches!(touche, Suivant) {
                        frappe = Some(Frappe::Suivant);
                    } else {
                        frapper(saisie, touche, decimal);
                        frappe = Some(Frappe::Modifie);
                    }
                }
            }
        });
        ui.add_space(1.0);
    }
    frappe
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_pave_ecrit_une_mesure_lisible() {
        use ToucheDeMesure::{Barre, Chiffre, Decimale, Espace, Fraction, Reculer, Unite};
        let mut saisie = String::new();
        // Une unité seule n'écrit rien.
        frapper(&mut saisie, Unite("pi"), ',');
        assert_eq!(saisie, "");
        for touche in [
            Chiffre('1'),
            Chiffre('2'),
            Unite("pi"),
            Chiffre('6'),
            Espace,
            Chiffre('1'),
            Barre,
            Chiffre('2'),
            Unite("po"),
        ] {
            frapper(&mut saisie, touche, ',');
        }
        assert_eq!(saisie, "12 pi 6 1/2 po ");
        let lue = noyau::analyse::analyser_longueur(&saisie, FORMAT_PAR_DEFAUT).unwrap();
        assert!((lue.base - 150.5).abs() < 1e-9);
        // Reculer : l'unité d'un coup, puis un caractère à la fois.
        frapper(&mut saisie, Reculer, ',');
        assert_eq!(saisie, "12 pi 6 1/2");
        frapper(&mut saisie, Reculer, ',');
        assert_eq!(saisie, "12 pi 6 1/");
        for _ in 0..4 {
            frapper(&mut saisie, Reculer, ',');
        }
        assert_eq!(saisie, "12");
        frapper(&mut saisie, Reculer, ',');
        frapper(&mut saisie, Reculer, ',');
        frapper(&mut saisie, Reculer, ',');
        assert_eq!(saisie, "");
        // En métrique, avec la virgule décimale.
        for touche in [
            Chiffre('3'),
            Decimale,
            Chiffre('8'),
            Chiffre('1'),
            Unite("m"),
        ] {
            frapper(&mut saisie, touche, ',');
        }
        assert_eq!(saisie, "3,81 m ");
        let lue = noyau::analyse::analyser_longueur(&saisie, FORMAT_PAR_DEFAUT).unwrap();
        assert!((lue.base - 150.0).abs() < 0.001);

        // Les fractions de pouce se placent après les pouces.
        let lire = |texte: &str| {
            noyau::analyse::analyser_longueur(texte, FORMAT_PAR_DEFAUT)
                .unwrap()
                .base
        };
        saisie.clear();
        frapper(&mut saisie, Fraction(3, 8), ',');
        assert_eq!(saisie, "3/8 po ");
        assert!((lire(&saisie) - 0.375).abs() < 1e-9);
        saisie.clear();
        for touche in [
            Chiffre('1'),
            Chiffre('2'),
            Unite("pi"),
            Chiffre('6'),
            Fraction(1, 2),
        ] {
            frapper(&mut saisie, touche, ',');
        }
        assert_eq!(saisie, "12 pi 6 1/2 po ");
        assert!((lire(&saisie) - 150.5).abs() < 1e-9);
        // Une autre fraction remplace la première.
        frapper(&mut saisie, Fraction(1, 4), ',');
        assert_eq!(saisie, "12 pi 6 1/4 po ");
        assert!((lire(&saisie) - 150.25).abs() < 1e-9);
        // Après « Pouces », la fraction se place devant le symbole.
        saisie = "12 pi 6 po ".to_string();
        frapper(&mut saisie, Fraction(7, 8), ',');
        assert_eq!(saisie, "12 pi 6 7/8 po ");
        // Des pieds, puis une fraction : une fraction de pouce.
        saisie = "12 pi ".to_string();
        frapper(&mut saisie, Fraction(1, 2), ',');
        assert_eq!(saisie, "12 pi 1/2 po ");
        assert!((lire(&saisie) - 144.5).abs() < 1e-9);
        // En anglais, avec les symboles anglais.
        let _anglais = noyau::langue::pour_la_portee(noyau::langue::Langue::Anglais);
        saisie = "12 ft 6".to_string();
        frapper(&mut saisie, Fraction(1, 2), '.');
        assert_eq!(saisie, "12 ft 6 1/2 in ");
        assert!((lire(&saisie) - 150.5).abs() < 1e-9);
    }

    #[test]
    fn le_zoom_garde_en_place_le_point_vise() {
        let centre = Pos2::new(200.0, 150.0);
        let ajustee = Echelle {
            origine: Pos2::new(60.0, 40.0),
            facteur: 2.0,
        };
        let mut cadrage = Cadrage::default();
        // Sans zoom, l'échelle reste celle qui fait tenir la forme.
        let vue = cadrage.appliquer(ajustee, centre);
        assert_eq!(
            (vue.origine, vue.facteur),
            (ajustee.origine, ajustee.facteur)
        );
        // Le point du croquis qui est sous le pointeur y reste après le zoom.
        let pointeur = Pos2::new(300.0, 90.0);
        let avant = cadrage.appliquer(ajustee, centre);
        let sous_le_pointeur = (
            f64::from((pointeur.x - avant.origine.x) / avant.facteur),
            f64::from((pointeur.y - avant.origine.y) / avant.facteur),
        );
        cadrage.zoomer(centre, pointeur, 2.5);
        let apres = cadrage
            .appliquer(ajustee, centre)
            .vers_l_ecran(sous_le_pointeur);
        assert!(apres.distance(pointeur) < 1e-3, "{apres:?}");
        assert!((cadrage.zoom - 2.5).abs() < 1e-6);
        // Les bornes du zoom.
        cadrage.zoomer(centre, centre, 100.0);
        assert!((cadrage.zoom - ZOOM_MAX).abs() < 1e-6);
        cadrage.zoomer(centre, centre, 1e-6);
        assert!((cadrage.zoom - ZOOM_MIN).abs() < 1e-6);
        // Un facteur absurde ne change rien.
        let avant = cadrage;
        cadrage.zoomer(centre, centre, f32::NAN);
        cadrage.zoomer(centre, centre, 0.0);
        assert_eq!(cadrage, avant);
    }

    #[test]
    fn les_mesures_s_affichent_en_pieds_pouces_et_fractions() {
        let prefs = Preferences::default();
        // Les espaces insécables du texte affiché sont lus comme des espaces.
        let en_clair =
            |texte: &str| saisie_en_clair(texte, &prefs).map(|s| s.replace('\u{a0}', " "));
        assert_eq!(en_clair("20").as_deref(), Some("20 pi 0 po"));
        assert_eq!(
            en_clair("12 pi 6 1/2 po").as_deref(),
            Some("12 pi 6 1/2 po")
        );
        assert_eq!(en_clair("150 po").as_deref(), Some("12 pi 6 po"));
        assert_eq!(en_clair("3/8 po").as_deref(), Some("3/8 po"));
        assert_eq!(en_clair("3,81 m").as_deref(), Some("3,810 m"));
        assert_eq!(en_clair("abc"), None);
        assert_eq!(en_clair("0"), None);
    }
}
