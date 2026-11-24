//! Écran « Dessin » (section 25) : on trace le contour d'une pièce avec le
//! doigt ou la souris, puis on touche chaque côté pour y inscrire sa mesure.

use eframe::egui::{self, Align2, Color32, Pos2, Rect, RichText, Sense, Stroke, Ui, Vec2};
use noyau::cloison::{
    Cloison, CloisonMesuree, EPAISSEUR_PAR_DEFAUT, LARGEUR_DE_PORTE_PAR_DEFAUT, PorteDeCloison,
    Segment, SegmentDeCloison, redresser_le_trace,
};
use noyau::dessin::{
    Dessin, Etat as EtatDeLaForme, GenreDOuverture, Mesure, Ouverture, Plan, Retrait, Sens,
    Solution,
};
use noyau::dessin::{EPAISSEUR_DES_MURS_PAR_DEFAUT, EchelleDuPlan, MorceauDeMur};
use noyau::metiers::Quantite;
use noyau::mobilier::{
    self, Contour, Encombrement, Famille, FormeDEscalier, GenreDObjet, Objet, TraitSurLePlan,
};
use noyau::plafond::{FormeDePlafond, Retombee};
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
/// Distance, en points, sous laquelle un toucher saisit la poignée d'un mur.
const PORTEE_D_UNE_POIGNEE: f32 = 16.0;
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

/// Le mur qu'on étire par son milieu.
/// Un segment de cloison qu'on étire par son milieu (point 38), comme un mur.
struct CloisonEtiree {
    rang: usize,
    segment: usize,
    /// Le dessin au début du geste : chaque image repart de lui.
    avant: Dessin,
    /// Le point de l'écran où le doigt s'est posé, et la direction dans
    /// laquelle le segment avance à l'écran (unitaire).
    origine: Pos2,
    normale: Vec2,
    /// Des pouces par point de l'écran.
    pouces_par_point: f64,
    /// Le cadrage du plan au début du geste, gardé jusqu'à sa fin.
    echelle: Echelle,
}

struct MurTire {
    cote: usize,
    /// Le dessin au début du geste : chaque image repart de lui.
    avant: Dessin,
    /// Le point de l'écran où le doigt s'est posé, et la direction de
    /// l'extérieur du mur à l'écran (unitaire).
    origine: Pos2,
    dehors: Vec2,
    /// Des pouces par point de l'écran.
    pouces_par_point: f64,
    /// Le cadrage du plan au début du geste, gardé jusqu'à sa fin.
    echelle: Echelle,
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
    /// Les états précédents du dessin, pour le bouton « Annuler »; et ceux
    /// qu'« Annuler » a défaits, pour « Rétablir ».
    historique: Vec<Dessin>,
    futur: Vec<Dessin>,
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
    /// La famille d'objets dont la section montre les boutons (point 40) :
    /// le choix dure le temps de la séance, comme le pliage des sections.
    famille: Famille,
    /// L'outil « Mesurer » (évolution 23) : `None` s'il est fermé, sinon ses
    /// points (pouces, sur le plan), deux au plus.
    mesure: Option<Vec<(f64, f64)>>,
    /// Vrai quand le crayon du croquis à main levée est sorti (évolution
    /// 30) : un glissement sur le plan trace alors un trait.
    crayon_sorti: bool,
    /// Le crayon choisi dans la palette.
    crayon: noyau::dessin::Crayon,
    /// Le trait que le doigt trace (pouces, sur le plan).
    trait_en_cours: Option<Vec<(f64, f64)>>,
    /// Le repère de chaque ouverture de la pièce au bordereau : P1, F1…
    /// (évolution 24), recalculé à chaque image d'après le plan.
    reperes_des_ouvertures: Vec<String>,
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
    /// La cloison qu'on trace : les points du doigt (pouces), coins compris.
    cloison_en_trace: Option<Vec<(f64, f64)>>,
    /// La cloison choisie.
    cloison_choisie: Option<usize>,
    /// La cloison que le doigt déplace : son rang, et l'écart (pouces) entre
    /// le point saisi et son départ.
    cloison_tiree: Option<(usize, (f64, f64))>,
    /// Le mur qu'on étire par son milieu.
    mur_tire: Option<MurTire>,
    /// Le segment de cloison qu'on étire par son milieu (point 38).
    cloison_etiree: Option<CloisonEtiree>,
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
    /// L'angle qu'on tape pour le côté ou le segment choisi (section 25.1, point 32).
    angle_saisi: String,
    /// Ce qui n'a pas marché avec cet angle.
    avis_d_angle: Option<noyau::langue::Texte>,
    /// Vrai quand la planche prend presque tout l'écran (point 37).
    planche_agrandie: bool,
    /// Les sections pliées sous le plan.
    sections_pliees: Vec<Section>,
    /// Vrai quand la planche montre l'étage entier (évolution 4).
    vue_etage: bool,
    /// La pièce à l'écran quand l'étage s'est ouvert.
    courante_avant_l_etage: usize,
    /// Le cadrage de la vue de l'étage.
    cadrage_etage: Cadrage,
    /// La pièce qu'on glisse dans l'étage.
    piece_tiree: Option<PieceTiree>,
    /// Les deux murs qui se touchent pendant qu'elle glisse.
    contact: Option<noyau::etage::Contact>,
    /// Les places d'avant, pour « Annuler » dans l'étage.
    places_d_avant: Vec<Vec<Option<noyau::etage::Place>>>,
}

/// La pièce qu'on glisse dans l'étage (évolution 4).
struct PieceTiree {
    rang: usize,
    /// Sa place au début du geste, et le chemin parcouru depuis (pouces).
    depart: noyau::etage::Place,
    chemin: (f64, f64),
    /// Le cadrage de l'étage au début du geste, gardé jusqu'à sa fin.
    echelle: Echelle,
}

/// Une section repliable sous le plan (point 37).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Section {
    Diagonales,
    Retraits,
    Murs,
    Cloisons,
    Objets,
    Notes,
    Exporter,
    Bordereau,
    Finis,
    TableauDesFinis,
    Espaces,
    Elevations,
    Ossature,
    Calepinage,
}

/// Le titre d'une section repliable : un triangle, pointé vers le bas si elle
/// est dépliée, et son nom; un toucher la plie ou la déplie. Retourne vrai
/// si elle est dépliée.
fn titre_de_section(
    ui: &mut Ui,
    p: &Palette,
    etat: &mut Etat,
    section: Section,
    titre: &str,
) -> bool {
    let galere = ui
        .painter()
        .layout_no_wrap(titre.to_owned(), theme::police_grasse(13.0), p.titre);
    let (zone, reponse) = ui.allocate_exact_size(
        Vec2::new(galere.size().x + 22.0, galere.size().y.max(18.0) + 6.0),
        Sense::click(),
    );
    let pliee = etat.sections_pliees.contains(&section);
    reponse.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, titre));
    let centre = Pos2::new(zone.left() + 7.0, zone.center().y);
    let pointes = if pliee {
        vec![
            centre + Vec2::new(-3.0, -5.0),
            centre + Vec2::new(4.0, 0.0),
            centre + Vec2::new(-3.0, 5.0),
        ]
    } else {
        vec![
            centre + Vec2::new(-5.0, -3.0),
            centre + Vec2::new(5.0, -3.0),
            centre + Vec2::new(0.0, 4.0),
        ]
    };
    let painter = ui.painter();
    painter.add(egui::Shape::convex_polygon(pointes, p.titre, Stroke::NONE));
    painter.galley(
        Pos2::new(zone.left() + 18.0, zone.center().y - galere.size().y / 2.0),
        galere,
        p.titre,
    );
    if reponse.clicked() {
        if pliee {
            etat.sections_pliees.retain(|s| *s != section);
        } else {
            etat.sections_pliees.push(section);
        }
        return pliee;
    }
    !pliee
}

/// Une mesure d'une cloison ou de sa porte.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ChampDeCloison {
    /// La longueur d'un de ses segments.
    Longueur(usize),
    Epaisseur,
    /// La largeur et la place d'une de ses ouvertures (point 39).
    LargeurDOuverture(usize),
    PositionDOuverture(usize),
}

/// Ce que demande un bouton de la ligne d'une cloison.
#[derive(Clone, Copy)]
enum GesteSurUneCloison {
    Enlever,
    /// Une porte, ou un passage sans porte (vrai).
    AjouterUneOuverture(bool),
    /// La porte s'ouvre de l'autre côté.
    Inverser(usize),
    EnleverLOuverture(usize),
    /// L'ouverture passe au segment suivant.
    SegmentSuivant(usize),
    Copier,
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
    /// Une mesure d'un objet : son rang, et laquelle.
    Objet(usize, ChampDObjet),
    /// Une mesure d'une cloison ou de sa porte : son rang, et laquelle.
    Cloison(usize, ChampDeCloison),
    /// L'épaisseur des murs extérieurs de la pièce.
    EpaisseurDesMurs,
    /// L'épaisseur des cloisons de la pièce.
    EpaisseurDesCloisons,
    /// Une diagonale d'une pièce à quatre côtés : 0 du coin 1 au coin 3, 1
    /// du coin 2 au coin 4 (section 25.1, point 33).
    Diagonale(usize),
    /// La hauteur haute d'un plafond qui n'est pas plat (point 34).
    HautDuPlafond,
    /// Une mesure d'une retombée de plafond : son rang, et laquelle.
    Retombee(usize, ChampDeRetombee),
    /// Une dimension d'une pièce du calepinage : vrai pour sa longueur.
    Calepinage(bool),
}

/// Une mesure d'une retombée de plafond.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ChampDeRetombee {
    Longueur,
    Largeur,
    /// De combien elle descend sous le plafond.
    Descente,
}

/// Une mesure d'un objet : ses dimensions, et pour un escalier sa hauteur à
/// monter et son nombre de contremarches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ChampDObjet {
    Largeur,
    Profondeur,
    Hauteur,
    Contremarches,
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
            AutreSaisie::Cloison(rang, champ) => {
                dessin
                    .cloisons
                    .get(rang)
                    .is_some_and(|cloison| match champ {
                        ChampDeCloison::Longueur(segment) => segment < cloison.nombre_de_segments(),
                        ChampDeCloison::Epaisseur => true,
                        ChampDeCloison::LargeurDOuverture(k)
                        | ChampDeCloison::PositionDOuverture(k) => k < cloison.ouvertures.len(),
                    })
            }
            AutreSaisie::EpaisseurDesMurs | AutreSaisie::EpaisseurDesCloisons => true,
            AutreSaisie::Diagonale(k) => k < 2 && dessin.fermee && dessin.cotes.len() == 4,
            AutreSaisie::HautDuPlafond => dessin.plafond.forme != FormeDePlafond::Plat,
            AutreSaisie::Retombee(rang, _) => rang < dessin.retombees.len(),
            AutreSaisie::Calepinage(_) => true,
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
        self.angle_saisi.clear();
        self.avis_d_angle = None;
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
        self.futur.clear();
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
        self.crayon_sorti = false;
        self.cloison_en_trace = None;
        self.cloison_choisie = None;
        self.cloison_tiree = None;
        self.mur_tire = None;
        self.cloison_etiree = None;
    }

    /// Choisit une autre mesure : un retrait, la hauteur des murs, une ouverture.
    fn choisir_autre(&mut self, lequel: Option<AutreSaisie>) {
        self.choisi = None;
        self.autre_choisi = lequel;
        self.viser_le_champ = lequel.is_some();
        self.angle_saisi.clear();
        self.avis_d_angle = None;
    }

    /// Retient le dessin tel qu'il est, avant un changement.
    fn memoriser(&mut self, dessin: &Dessin) {
        // Un changement nouveau : ce qu'« Annuler » avait défait ne revient plus.
        self.futur.clear();
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
    /// Ouvrir la liste des dessins.
    MesDessins,
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
/// Le segment d'une cloison le plus proche d'un point du plan (pouces).
fn segment_le_plus_proche(cloison: &CloisonMesuree, (x, y): (f64, f64)) -> usize {
    let distance = |(a, b): ((f64, f64), (f64, f64))| {
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let longueur = dx * dx + dy * dy;
        let t = if longueur > 0.0 {
            (((x - a.0) * dx + (y - a.1) * dy) / longueur).clamp(0.0, 1.0)
        } else {
            0.0
        };
        (x - (a.0 + dx * t)).hypot(y - (a.1 + dy * t))
    };
    cloison
        .axes()
        .into_iter()
        .enumerate()
        .min_by(|(_, s), (_, t)| distance(*s).total_cmp(&distance(*t)))
        .map_or(0, |(segment, _)| segment)
}

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
    appuis.extend(solution.cloisons.iter().flatten().flat_map(|c| c.axes()));
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

/// Une cloison sur le plan : pleine, aux coins d'onglet, avec la longueur de
/// chaque segment à côté de lui, sa porte (le battant et son arc) et, choisie,
/// le bout A du segment de sa porte, d'où se mesure la porte.
fn dessiner_une_cloison(
    painter: &egui::Painter,
    p: &Palette,
    echelle: &Echelle,
    cloison: &CloisonMesuree,
    choisie: bool,
    ecrire: &dyn Fn(f64) -> String,
) {
    let encre = if choisie {
        p.accent_du_plan()
    } else {
        p.primaire
    };
    let morceaux = cloison.morceaux();
    // Le plein d'abord, puis les faces et les bouts : les coins d'onglet ne
    // se tracent pas.
    for morceau in &morceaux {
        let coins: Vec<Pos2> = morceau
            .coins
            .iter()
            .map(|coin| echelle.vers_l_ecran(*coin))
            .collect();
        painter.add(egui::Shape::convex_polygon(
            coins,
            encre.gamma_multiply(0.45),
            Stroke::NONE,
        ));
    }
    let trait_ = Stroke::new(1.4, encre);
    for morceau in &morceaux {
        let [a, b, c, d] = morceau.coins.map(|coin| echelle.vers_l_ecran(coin));
        painter.line_segment([a, b], trait_);
        painter.line_segment([d, c], trait_);
        if morceau.bout_au_debut {
            painter.line_segment([a, d], trait_);
        }
        if morceau.bout_a_la_fin {
            painter.line_segment([b, c], trait_);
        }
    }
    // Ses portes, avec leur battant; ses passages, un trait de tirets sur
    // l'axe, comme le linteau qui les surmonte (point 39).
    for porte in &cloison.ouvertures {
        let q0 = echelle.vers_l_ecran(cloison.point(porte.segment, porte.debut));
        let q1 = echelle.vers_l_ecran(cloison.point(porte.segment, porte.fin));
        if porte.passage {
            painter.extend(egui::Shape::dashed_line(
                &[q0, q1],
                Stroke::new(1.2, encre),
                4.0,
                3.0,
            ));
        } else {
            let (nx, ny) = cloison.cote_de_l_ouverture(porte);
            let cote = Vec2::new(nx as f32, ny as f32);
            dessiner_ouverture(
                painter,
                (q0, q1),
                cote,
                (GenreDOuverture::Porte, false, 0.0, 1.0),
                encre,
                encre,
                0.0,
            );
        }
    }
    // La longueur de chaque segment s'écrit au-dessus d'un segment horizontal,
    // à droite d'un segment vertical : deux cloisons qui se touchent ne mêlent
    // pas leurs longueurs.
    for k in 0..cloison.nombre_de_segments() {
        let a = echelle.vers_l_ecran(cloison.sommets[k]);
        let b = echelle.vers_l_ecran(cloison.sommets[k + 1]);
        let le_long = (b - a).normalized();
        let normale = Vec2::new(-le_long.y, le_long.x);
        let a_cote = if normale.x - normale.y >= 0.0 {
            normale
        } else {
            -normale
        };
        let ecart = cloison.epaisseur as f32 * echelle.facteur / 2.0 + 10.0;
        let texte = ecrire(cloison.longueur_du_segment(k));
        let galere = painter.layout_no_wrap(texte, theme::police_grasse(11.0), encre);
        let cadre = Rect::from_center_size(a + (b - a) * 0.5 + a_cote * ecart, galere.size())
            .expand2(Vec2::new(2.0, 1.0));
        painter.rect_filled(cadre, 3, p.carte.gamma_multiply(0.9));
        painter.galley(cadre.min + Vec2::new(2.0, 1.0), galere, encre);
    }
    if choisie && cloison.nombre_de_segments() > 0 {
        // Le bout A : le début du segment de sa première ouverture (ou de la
        // cloison).
        let segment = cloison
            .ouvertures
            .first()
            .map_or(0, |porte| porte.segment)
            .min(cloison.nombre_de_segments() - 1);
        let a = echelle.vers_l_ecran(cloison.sommets[segment]);
        let b = echelle.vers_l_ecran(cloison.sommets[segment + 1]);
        let le_long = (b - a).normalized();
        painter.circle_stroke(a, 6.5, Stroke::new(1.6, p.accent_du_plan()));
        painter.text(
            a - le_long * 15.0,
            Align2::CENTER_CENTER,
            "A",
            theme::police_grasse(12.0),
            p.accent_du_plan(),
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
        p.accent_du_plan()
    } else {
        p.primaire
    };
    painter.add(egui::Shape::convex_polygon(
        coins.clone(),
        encre.gamma_multiply(0.28),
        Stroke::NONE,
    ));
    let trait_ = Stroke::new(1.3, encre);
    // Ses deux faces : celle de la ligne mesurée se double du côté, plus épais.
    painter.line_segment([coins[0], coins[1]], trait_);
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

/// Le pas d'un objet qu'on déplace : le pouce, ou le centimètre en métrique.
fn pas_des_objets(solution: &Solution) -> f64 {
    let metrique =
        solution.modele.and_then(Format::systeme) == Some(noyau::unites::Systeme::Metrique);
    if metrique { 1.0 / 2.54 } else { 1.0 }
}

/// Ce qui encombre la pièce pour l'objet de rang `sauf` : les autres objets
/// qui occupent le plancher, les morceaux des cloisons et les murs (R-98).
/// Rien n'encombre un symbole, qui se pose n'importe où (point 40).
fn encombrement<'a>(
    dessin: &Dessin,
    solution: &Solution,
    sauf: usize,
    contour: Option<&'a [(f64, f64)]>,
) -> Encombrement<'a> {
    if dessin
        .objets
        .get(sauf)
        .is_some_and(|objet| !objet.genre.encombre())
    {
        return Encombrement {
            obstacles: Vec::new(),
            piece: None,
        };
    }
    let mut obstacles: Vec<[(f64, f64); 4]> = dessin
        .objets
        .iter()
        .zip(&solution.objets)
        .enumerate()
        .filter(|(rang, (objet, _))| *rang != sauf && objet.genre.encombre())
        .map(|(_, (objet, dimensions))| objet.coins(*dimensions))
        .collect();
    obstacles.extend(
        solution
            .cloisons
            .iter()
            .flatten()
            .flat_map(|cloison| cloison.morceaux())
            .map(|morceau| morceau.coins),
    );
    Encombrement {
        obstacles,
        piece: contour,
    }
}

/// Où s'attache le sigle d'un objet (sur le plan), et, s'il va à côté de son
/// symbole, la direction de son devant (un vecteur unitaire, y vers le bas)
/// (point 40).
pub(crate) fn attache_du_sigle(
    objet: &Objet,
    dimensions: (f64, f64),
) -> ((f64, f64), Option<(f64, f64)>) {
    let attache = objet.vers_le_plan(objet.genre.place_du_sigle(), dimensions);
    if !objet.genre.sigle_a_cote() {
        return (attache, None);
    }
    let (dos, devant) = (
        objet.vers_le_plan((0.5, 0.0), dimensions),
        objet.vers_le_plan((0.5, 1.0), dimensions),
    );
    let (dx, dy) = (devant.0 - dos.0, devant.1 - dos.1);
    let norme = dx.hypot(dy).max(1e-9);
    (attache, Some((dx / norme, dy / norme)))
}

/// Le centre d'un texte de taille `taille` (largeur, hauteur) écrit à côté
/// de `attache`, dans la direction `devant` (un vecteur unitaire) : poussé de
/// la moitié de son étendue dans cette direction, et de `ecart`, il ne
/// chevauche pas le symbole (point 40).
pub(crate) fn texte_a_cote(
    attache: (f64, f64),
    devant: (f64, f64),
    (largeur, hauteur): (f64, f64),
    ecart: f64,
) -> (f64, f64) {
    let etendue = devant.0.abs() * largeur / 2.0 + devant.1.abs() * hauteur / 2.0;
    (
        attache.0 + devant.0 * (etendue + ecart),
        attache.1 + devant.1 * (etendue + ecart),
    )
}

/// Où s'accroche un point de l'outil « Mesurer » (évolution 23) : au coin le
/// plus proche (de la pièce, d'une cloison ou d'un objet) à 14 points au plus,
/// sinon au point le plus proche d'une face de mur ou de cloison à 10 points
/// au plus, sinon là où l'on touche, au pas du plan.
fn point_a_mesurer(
    position: Pos2,
    echelle: &Echelle,
    dessin: &Dessin,
    solution: &Solution,
) -> (f64, f64) {
    let contour = dessin.contour_interieur(solution);
    let morceaux: Vec<[(f64, f64); 4]> = solution
        .cloisons
        .iter()
        .flatten()
        .flat_map(|cloison| cloison.morceaux())
        .map(|morceau| morceau.coins)
        .collect();
    let mut coins: Vec<(f64, f64)> = contour.clone().unwrap_or_default();
    coins.extend(dessin.contour_exterieur(solution).unwrap_or_default());
    coins.extend(morceaux.iter().flatten().copied());
    coins.extend(
        dessin
            .objets
            .iter()
            .zip(&solution.objets)
            .flat_map(|(objet, dimensions)| objet.coins(*dimensions)),
    );
    let ecart = |point: (f64, f64)| echelle.vers_l_ecran(point).distance(position);
    if let Some(coin) = coins
        .iter()
        .copied()
        .min_by(|a, b| ecart(*a).total_cmp(&ecart(*b)))
        && ecart(coin) <= 14.0
    {
        return coin;
    }
    let (x, y) = echelle.vers_le_plan(position);
    let sur_une_face = mobilier::faces_d_appui(contour.as_deref(), &morceaux)
        .into_iter()
        .filter_map(|(a, b)| {
            let (dx, dy) = (b.0 - a.0, b.1 - a.1);
            let longueur2 = dx * dx + dy * dy;
            if longueur2 < 1e-12 {
                return None;
            }
            let t = (((x - a.0) * dx + (y - a.1) * dy) / longueur2).clamp(0.0, 1.0);
            Some((a.0 + dx * t, a.1 + dy * t))
        })
        .min_by(|a, b| ecart(*a).total_cmp(&ecart(*b)));
    if let Some(point) = sur_une_face
        && ecart(point) <= 10.0
    {
        return point;
    }
    let pas = pas_des_objets(solution);
    ((x / pas).round() * pas, (y / pas).round() * pas)
}

/// Ce que frappe un meuble qui avance (évolution 23) : les faces des murs et
/// des cloisons, et les côtés des autres objets qui occupent le plancher.
fn obstacles_des_degagements(
    dessin: &Dessin,
    solution: &Solution,
    sauf: usize,
) -> Vec<mobilier::Face> {
    let contour = dessin.contour_interieur(solution);
    let morceaux: Vec<[(f64, f64); 4]> = solution
        .cloisons
        .iter()
        .flatten()
        .flat_map(|cloison| cloison.morceaux())
        .map(|morceau| morceau.coins)
        .collect();
    let mut obstacles = mobilier::faces_d_appui(contour.as_deref(), &morceaux);
    for (rang, (objet, dimensions)) in dessin.objets.iter().zip(&solution.objets).enumerate() {
        if rang != sauf && objet.genre.encombre() {
            let coins = objet.coins(*dimensions);
            obstacles.extend((0..4).map(|k| (coins[k], coins[(k + 1) % 4])));
        }
    }
    obstacles
}

/// Une distance écrite sur le plan, entre `a` et `b` (à l'écran) : un trait
/// fin terminé par deux petits traits, et sa longueur dans une pastille.
fn distance_sur_le_plan(
    painter: &egui::Painter,
    p: &Palette,
    (a, b): (Pos2, Pos2),
    texte: String,
    pointille: bool,
) {
    let longueur = a.distance(b);
    if longueur < 1.0 {
        return;
    }
    let couleur = p.accent_du_plan();
    if pointille {
        painter.extend(egui::Shape::dashed_line(
            &[a, b],
            Stroke::new(1.3, couleur),
            4.0,
            3.0,
        ));
    } else {
        painter.line_segment([a, b], Stroke::new(2.0, couleur));
    }
    let unite = (b - a) / longueur;
    let travers = Vec2::new(-unite.y, unite.x);
    for bout in [a, b] {
        painter.line_segment(
            [bout - travers * 5.0, bout + travers * 5.0],
            Stroke::new(1.3, couleur),
        );
    }
    let ecrit = painter.layout_no_wrap(texte, theme::police_grasse(11.0), couleur);
    let centre = a + (b - a) * 0.5 + travers * (ecrit.size().y * 0.5 + 6.0);
    let cadre = Rect::from_center_size(centre, ecrit.size() + Vec2::new(8.0, 3.0));
    painter.rect_filled(cadre, 5.0, p.carte);
    painter.rect_stroke(
        cadre,
        5.0,
        Stroke::new(1.0, couleur),
        egui::StrokeKind::Inside,
    );
    painter.galley(centre - ecrit.size() / 2.0, ecrit, couleur);
}

/// Un trait à main levée sur la planche (évolution 30) : au crayon, un trait
/// de couleur; au surligneur, une large bande transparente.
fn trait_a_main_levee(
    painter: &egui::Painter,
    p: &Palette,
    trait_: &noyau::dessin::TraitAMainLevee,
    echelle: &Echelle,
) {
    let points: Vec<Pos2> = trait_
        .points
        .iter()
        .map(|point| echelle.vers_l_ecran(*point))
        .collect();
    if points.len() < 2 {
        return;
    }
    let largeur = if trait_.crayon == noyau::dessin::Crayon::Surligneur {
        14.0
    } else {
        2.6
    };
    painter.add(egui::Shape::line(
        points,
        Stroke::new(largeur, theme::couleur_du_crayon(trait_.crayon, p)),
    ));
}

/// Les quatre finis d'une pièce en clair, « — » pour ceux qui manquent
/// (évolution 27) : plancher, murs, plafond, plinthes.
pub(crate) fn finis_en_clair(finis: &noyau::finis::Finis) -> [String; 4] {
    use noyau::finis::{FiniDePlafond, Plancher, Plinthe};
    let ou_tiret = |aucun: bool, nom: &str| {
        if aucun {
            "—".to_string()
        } else {
            nom.to_string()
        }
    };
    [
        ou_tiret(
            finis.plancher == Plancher::Aucun,
            textes::nom_du_plancher(finis.plancher).traduit(),
        ),
        textes::peinture_en_clair(finis.peinture, &finis.couleur),
        ou_tiret(
            finis.plafond == FiniDePlafond::Aucun,
            textes::nom_du_plafond(finis.plafond).traduit(),
        ),
        ou_tiret(
            finis.plinthes == Plinthe::Aucune,
            textes::nom_de_la_plinthe(finis.plinthes).traduit(),
        ),
    ]
}

/// La quantité d'un fini en clair : une aire, ou une longueur de plinthes.
pub(crate) fn quantite_d_un_fini(
    fini: &noyau::finis::Fini,
    quantite: f64,
    modele: Option<Format>,
    prefs: &Preferences,
) -> String {
    let quantite = if fini.en_longueur() {
        Quantite::Longueur(quantite)
    } else {
        Quantite::Aire(quantite)
    };
    sans_insecables(&quantite.en_clair(modele, prefs))
}

/// Une ligne du bordereau en clair (évolution 24) : son repère, son genre,
/// sa largeur × sa hauteur, le sens d'une porte battante (« — » sinon) et sa
/// pièce.
pub(crate) fn ligne_du_bordereau(
    ligne: &noyau::dessin::LigneDuBordereau,
    plan: &Plan,
    modele: Option<Format>,
    prefs: &Preferences,
) -> [String; 5] {
    let en_clair = |pouces: Option<f64>| {
        pouces.map_or("?".to_string(), |pouces| {
            sans_insecables(&Quantite::Longueur(pouces).en_clair(modele, prefs))
        })
    };
    let sens = match ligne.poussant_gauche {
        Some(true) => textes::DESSIN_POUSSANT_GAUCHE.traduit().to_string(),
        Some(false) => textes::DESSIN_POUSSANT_DROITE.traduit().to_string(),
        None => "—".to_string(),
    };
    [
        ligne.repere.clone(),
        textes::genre_d_ouverture(ligne.genre).traduit().to_string(),
        format!("{} × {}", en_clair(ligne.largeur), en_clair(ligne.hauteur)),
        sens,
        nom_de_la_piece(plan, ligne.rang_de_la_piece),
    ]
}

/// Les finis de la pièce (évolution 27) : son plancher, que montre une
/// trame, la peinture de ses murs, son plafond et ses plinthes. Retourne vrai
/// si le dessin a changé.
fn finis(ui: &mut Ui, p: &Palette, dessin: &mut Dessin, etat: &mut Etat) -> bool {
    use noyau::finis::{FiniDePeinture, FiniDePlafond, Plancher, Plinthe};
    ui.add_space(8.0);
    if !titre_de_section(ui, p, etat, Section::Finis, textes::DESSIN_FINIS.traduit()) {
        return false;
    }
    let avant = dessin.finis.clone();
    let mut finis = avant.clone();
    let mut couleur_prise = false;
    // Une liste par fini; le plancher a sa légende : un carré de sa trame.
    ui.horizontal_wrapped(|ui| {
        ui.label(
            RichText::new(textes::DESSIN_FINI_PLANCHER.traduit()).font(theme::police_grasse(13.5)),
        );
        egui::ComboBox::from_id_salt("fini-plancher")
            .selected_text(textes::nom_du_plancher(finis.plancher).traduit())
            .show_ui(ui, |ui| {
                for plancher in Plancher::TOUS {
                    ui.selectable_value(
                        &mut finis.plancher,
                        plancher,
                        textes::nom_du_plancher(plancher).traduit(),
                    );
                }
            });
        let (zone, _) = ui.allocate_exact_size(Vec2::splat(26.0), Sense::hover());
        echantillon_de_trame(ui.painter(), p, zone, finis.plancher);
    });
    ui.horizontal_wrapped(|ui| {
        ui.label(
            RichText::new(textes::DESSIN_FINI_PEINTURE.traduit()).font(theme::police_grasse(13.5)),
        );
        egui::ComboBox::from_id_salt("fini-peinture")
            .selected_text(textes::nom_de_la_peinture(finis.peinture).traduit())
            .show_ui(ui, |ui| {
                for peinture in FiniDePeinture::TOUS {
                    ui.selectable_value(
                        &mut finis.peinture,
                        peinture,
                        textes::nom_de_la_peinture(peinture).traduit(),
                    );
                }
            });
        let champ = egui::TextEdit::singleline(&mut finis.couleur)
            .desired_width(150.0)
            .char_limit(40)
            .hint_text(textes::DESSIN_FINI_COULEUR.traduit());
        let reponse = ui.add(champ);
        cadre_de_saisie(ui, p, &reponse);
        couleur_prise = reponse.gained_focus();
    });
    ui.horizontal_wrapped(|ui| {
        ui.label(
            RichText::new(textes::DESSIN_FINI_PLAFOND.traduit()).font(theme::police_grasse(13.5)),
        );
        egui::ComboBox::from_id_salt("fini-plafond")
            .selected_text(textes::nom_du_plafond(finis.plafond).traduit())
            .show_ui(ui, |ui| {
                for plafond in FiniDePlafond::TOUS {
                    ui.selectable_value(
                        &mut finis.plafond,
                        plafond,
                        textes::nom_du_plafond(plafond).traduit(),
                    );
                }
            });
        ui.label(
            RichText::new(textes::DESSIN_FINI_PLINTHES.traduit()).font(theme::police_grasse(13.5)),
        );
        egui::ComboBox::from_id_salt("fini-plinthes")
            .selected_text(textes::nom_de_la_plinthe(finis.plinthes).traduit())
            .show_ui(ui, |ui| {
                for plinthe in Plinthe::TOUS {
                    ui.selectable_value(
                        &mut finis.plinthes,
                        plinthe,
                        textes::nom_de_la_plinthe(plinthe).traduit(),
                    );
                }
            });
    });
    // Une étape d'annulation par liste changée, et une pour la couleur, prise
    // quand on entre dans son champ : tout ce qu'on y tape s'annule d'un coup.
    let liste_changee = finis.plancher != avant.plancher
        || finis.peinture != avant.peinture
        || finis.plafond != avant.plafond
        || finis.plinthes != avant.plinthes;
    if liste_changee || couleur_prise {
        etat.memoriser(dessin);
    }
    if finis != avant {
        dessin.finis = finis;
        return true;
    }
    false
}

/// La légende d'un plancher : un carré de sa trame.
fn echantillon_de_trame(
    painter: &egui::Painter,
    p: &Palette,
    zone: Rect,
    plancher: noyau::finis::Plancher,
) {
    painter.rect_filled(zone, 4.0, p.carte);
    painter.rect_stroke(
        zone,
        4.0,
        Stroke::new(1.0, p.bouton_bord),
        egui::StrokeKind::Inside,
    );
    // Un carré de 3 pi, ramené au carré de la légende.
    let cote = 36.0;
    let carre = [(0.0, 0.0), (cote, 0.0), (cote, cote), (0.0, cote)];
    let vers = |(x, y): (f64, f64)| {
        Pos2::new(
            zone.left() + 2.0 + (x / cote) as f32 * (zone.width() - 4.0),
            zone.top() + 2.0 + (y / cote) as f32 * (zone.height() - 4.0),
        )
    };
    trame_sur_la_planche(painter, p, &noyau::finis::trame(plancher, &carre), &vers);
}

/// Les traits d'une trame de plancher, à l'écran, d'un trait pâle.
fn trame_sur_la_planche(
    painter: &egui::Painter,
    p: &Palette,
    traits: &[noyau::finis::TraitDeTrame],
    vers: &dyn Fn((f64, f64)) -> Pos2,
) {
    let couleur = theme::melanger(p.carte, p.texte, 0.28);
    for trait_ in traits {
        match *trait_ {
            noyau::finis::TraitDeTrame::Ligne(a, b) => {
                painter.line_segment([vers(a), vers(b)], Stroke::new(0.8, couleur));
            }
            noyau::finis::TraitDeTrame::Point(point) => {
                painter.circle_filled(vers(point), 1.1, couleur);
            }
        }
    }
}

/// Les espaces que ferment les cloisons (évolution 5) : le nom de chacun,
/// qu'on peut changer, et son aire. Retourne vrai si le dessin a changé.
fn espaces_de_la_piece(
    ui: &mut Ui,
    p: &Palette,
    dessin: &mut Dessin,
    etat: &mut Etat,
    solution: &Solution,
    prefs: &Preferences,
) -> bool {
    let espaces = dessin.espaces(solution);
    if espaces.is_empty() {
        return false;
    }
    ui.add_space(8.0);
    if !titre_de_section(
        ui,
        p,
        etat,
        Section::Espaces,
        textes::DESSIN_ESPACES.traduit(),
    ) {
        return false;
    }
    let mut modifie = false;
    for (rang, espace) in espaces.iter().enumerate() {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(format!("{}.", rang + 1)).font(theme::police_grasse(13.5)));
            let mut nom = dessin.nom_de_l_espace(espace).unwrap_or("").to_string();
            let champ = egui::TextEdit::singleline(&mut nom)
                .desired_width(150.0)
                .char_limit(40)
                .hint_text(textes::dessin_espace(rang + 1));
            let reponse = ui.add(champ);
            cadre_de_saisie(ui, p, &reponse);
            // Tout ce qu'on y tape s'annule d'un coup.
            if reponse.gained_focus() {
                etat.memoriser(dessin);
            }
            if reponse.changed() {
                dessin.nommer_l_espace(espace, &nom);
                modifie = true;
            }
            ui.label(
                RichText::new(sans_insecables(
                    &Quantite::Aire(espace.aire).en_clair(solution.modele, prefs),
                ))
                .color(p.texte_attenue),
            );
        });
    }
    modifie
}

/// Le nom d'un espace : celui qu'on lui a donné, ou « Espace 2 ».
pub(crate) fn nom_d_un_espace(
    dessin: &Dessin,
    espace: &noyau::espaces::Espace,
    rang: usize,
) -> String {
    dessin
        .nom_de_l_espace(espace)
        .map_or_else(|| textes::dessin_espace(rang + 1), str::to_string)
}

/// Les élévations des murs (évolution 18) : chaque mur vu de face, tous à la
/// même échelle, avec ses portes et ses fenêtres à leur hauteur, leurs
/// repères et ses cotes.
fn elevations_des_murs(
    ui: &mut Ui,
    p: &Palette,
    dessin: &Dessin,
    etat: &mut Etat,
    solution: &Solution,
    prefs: &Preferences,
) {
    let elevations = dessin.elevations(solution);
    if elevations.is_empty() {
        return;
    }
    ui.add_space(8.0);
    if !titre_de_section(
        ui,
        p,
        etat,
        Section::Elevations,
        textes::DESSIN_ELEVATIONS.traduit(),
    ) {
        return;
    }
    let plus_long = elevations.iter().map(|e| e.longueur).fold(1.0, f64::max);
    let plus_haut = elevations.iter().map(|e| e.plus_haut()).fold(1.0, f64::max);
    let largeur = ui.available_width();
    let echelle = ((largeur - 56.0) / plus_long as f32).min(150.0 / plus_haut as f32);
    for elevation in &elevations {
        ui.label(
            RichText::new(textes::dessin_mur_vu_de_face(
                elevation.cote + 1,
                &longueur_en_clair(elevation.longueur, solution, prefs),
            ))
            .font(theme::police_grasse(13.0))
            .color(p.titre),
        );
        let hauteur = elevation.plus_haut() as f32 * echelle + 46.0;
        let (zone, _) = ui.allocate_exact_size(Vec2::new(largeur, hauteur), Sense::hover());
        let ecrire = |pouces: f64| longueur_en_clair(pouces, solution, prefs);
        mur_vu_de_face(
            ui.painter(),
            p,
            zone,
            elevation,
            echelle,
            &etat.reperes_des_ouvertures,
            &ecrire,
        );
        ui.add_space(6.0);
    }
}

/// Un mur vu de face dans `zone`, à `echelle` points par pouce : son contour,
/// ses portes et ses fenêtres, leurs repères, la chaîne des cotes au bas et sa
/// hauteur à gauche.
fn mur_vu_de_face(
    painter: &egui::Painter,
    p: &Palette,
    zone: Rect,
    elevation: &noyau::dessin::Elevation,
    echelle: f32,
    reperes: &[String],
    ecrire: &dyn Fn(f64) -> String,
) {
    use noyau::dessin::GenreDOuverture;
    let (gauche, base) = (zone.left() + 34.0, zone.bottom() - 28.0);
    let point = |t: f64, h: f64| Pos2::new(gauche + t as f32 * echelle, base - h as f32 * echelle);
    // Le mur : du plancher, par son haut, et retour.
    let mut contour = vec![point(0.0, 0.0)];
    contour.extend(elevation.haut.iter().map(|&(t, h)| point(t, h)));
    contour.push(point(elevation.longueur, 0.0));
    painter.add(egui::Shape::convex_polygon(
        contour.clone(),
        theme::melanger(p.carte, p.texte, 0.05),
        Stroke::NONE,
    ));
    painter.add(egui::Shape::closed_line(contour, Stroke::new(1.5, p.texte)));
    // Ses ouvertures, et leurs repères.
    for ouverture in &elevation.ouvertures {
        let cadre = Rect::from_two_pos(
            point(ouverture.debut, ouverture.bas),
            point(ouverture.fin, ouverture.haut),
        );
        painter.rect_filled(cadre, 0.0, p.carte);
        if ouverture.genre == GenreDOuverture::BaieLibre {
            let coins = [
                cadre.left_bottom(),
                cadre.left_top(),
                cadre.right_top(),
                cadre.right_bottom(),
            ];
            painter.extend(egui::Shape::dashed_line(
                &coins,
                Stroke::new(1.2, p.texte),
                4.0,
                3.0,
            ));
        } else {
            painter.rect_stroke(
                cadre,
                0.0,
                Stroke::new(1.2, p.texte),
                egui::StrokeKind::Middle,
            );
        }
        let fin = Stroke::new(0.8, p.texte_attenue);
        match ouverture.genre {
            // Le vitrage : deux châssis.
            GenreDOuverture::Fenetre
            | GenreDOuverture::FenetreEnBaie
            | GenreDOuverture::PorteFenetre => {
                painter.line_segment([cadre.center_top(), cadre.center_bottom()], fin);
            }
            // Les panneaux d'une porte de garage.
            GenreDOuverture::PorteDeGarage => {
                for k in 1..4 {
                    let y = cadre.top() + cadre.height() * k as f32 / 4.0;
                    painter.line_segment(
                        [Pos2::new(cadre.left(), y), Pos2::new(cadre.right(), y)],
                        fin,
                    );
                }
            }
            GenreDOuverture::PorteDouble => {
                painter.line_segment([cadre.center_top(), cadre.center_bottom()], fin);
            }
            GenreDOuverture::BaieLibre => {}
            // La poignée d'une porte, du côté opposé aux charnières.
            _ => {
                let x = cadre.right() - (cadre.width() * 0.12).max(3.0);
                painter.circle_filled(
                    Pos2::new(x, cadre.center().y + cadre.height() * 0.05),
                    1.8,
                    p.texte,
                );
            }
        }
        if let Some(repere) = reperes.get(ouverture.rang) {
            painter.text(
                cadre.center_top() + Vec2::new(0.0, 8.0),
                Align2::CENTER_CENTER,
                repere,
                theme::police_grasse(10.0),
                p.texte_attenue,
            );
        }
    }
    // La chaîne des cotes, au bas.
    let chaine = elevation.chaine();
    let y = base + 12.0;
    let trait_ = Stroke::new(1.0, p.accent_du_plan());
    painter.line_segment(
        [
            Pos2::new(point(0.0, 0.0).x, y),
            Pos2::new(point(elevation.longueur, 0.0).x, y),
        ],
        trait_,
    );
    for &t in &chaine {
        let x = point(t, 0.0).x;
        painter.line_segment([Pos2::new(x, y - 4.0), Pos2::new(x, y + 4.0)], trait_);
    }
    for paire in chaine.windows(2) {
        let (x0, x1) = (point(paire[0], 0.0).x, point(paire[1], 0.0).x);
        let ecrit = painter.layout_no_wrap(
            ecrire(paire[1] - paire[0]),
            theme::police(9.5),
            p.accent_du_plan(),
        );
        if ecrit.size().x + 4.0 < x1 - x0 {
            let centre = Pos2::new((x0 + x1) / 2.0, y + 9.0);
            painter.galley(centre - ecrit.size() / 2.0, ecrit, p.accent_du_plan());
        }
    }
    // La hauteur du mur à son début, à gauche.
    if let Some(&(_, h)) = elevation.haut.first() {
        let x = gauche - 14.0;
        let (bas, haut) = (base, point(0.0, h).y);
        painter.line_segment([Pos2::new(x, bas), Pos2::new(x, haut)], trait_);
        for y in [bas, haut] {
            painter.line_segment([Pos2::new(x - 4.0, y), Pos2::new(x + 4.0, y)], trait_);
        }
        let ecrit = painter.layout_no_wrap(ecrire(h), theme::police(9.5), p.accent_du_plan());
        let centre = Pos2::new(x - 9.0, (bas + haut) / 2.0);
        let texte = egui::epaint::TextShape::new(
            centre + Vec2::new(-ecrit.size().y / 2.0, ecrit.size().x / 2.0),
            ecrit,
            p.accent_du_plan(),
        )
        .with_angle(-std::f32::consts::FRAC_PI_2);
        painter.add(texte);
    }
}

/// Le calepinage (évolution 19) : ce qui se pose, le format d'une pièce, le
/// joint, le mur de départ, au coin ou au centre, le sens et le décalage des
/// rangs, et le compte des pièces. Retourne vrai si le dessin a changé.
fn calepinage_de_la_piece(
    ui: &mut Ui,
    p: &Palette,
    dessin: &mut Dessin,
    etat: &mut Etat,
    solution: &Solution,
    prefs: &Preferences,
) -> bool {
    use noyau::calepinage::{Decalage, Depart, JOINTS, MateriauAPoser, SensDeLaPose};
    ui.add_space(8.0);
    if !titre_de_section(
        ui,
        p,
        etat,
        Section::Calepinage,
        textes::DESSIN_CALEPINAGE.traduit(),
    ) {
        return false;
    }
    let avant = dessin.calepinage.clone();
    let mut calepinage = avant.clone();
    let gras = |texte: &str| RichText::new(texte).font(theme::police_grasse(13.5));
    ui.horizontal_wrapped(|ui| {
        if ui
            .add(
                egui::Button::new(textes::DESSIN_MONTRER_L_OSSATURE.traduit())
                    .selected(calepinage.visible),
            )
            .clicked()
        {
            calepinage.visible = !calepinage.visible;
        }
        for materiau in MateriauAPoser::TOUS {
            if ui
                .add(
                    egui::Button::new(textes::materiau_a_poser(materiau).traduit())
                        .selected(calepinage.materiau == materiau),
                )
                .clicked()
                && calepinage.materiau != materiau
            {
                calepinage.changer_de_materiau(materiau);
            }
        }
    });
    // Le format : la largeur et la longueur d'une pièce, au pavé, ou un format
    // courant du matériau; puis le joint.
    ui.horizontal_wrapped(|ui| {
        ui.label(gras(textes::DESSIN_FORMAT.traduit()));
        let (largeur, longueur) = calepinage.materiau.formats()[0];
        for (en_longueur, saisie, defaut) in [
            (false, &calepinage.largeur, largeur),
            (true, &calepinage.longueur, longueur),
        ] {
            let cible = AutreSaisie::Calepinage(en_longueur);
            let bouton = egui::Button::new(dimension_a_poser(saisie, defaut, prefs))
                .selected(etat.autre_choisi == Some(cible));
            if ui.add(bouton).clicked() {
                etat.choisir_autre(Some(cible));
            }
            if !en_longueur {
                ui.label("×");
            }
        }
        let mut format_choisi = None;
        egui::ComboBox::from_id_salt("format-du-calepinage")
            .selected_text(textes::DESSIN_FORMAT.traduit())
            .show_ui(ui, |ui| {
                for &(largeur, longueur) in calepinage.materiau.formats() {
                    let texte = format!(
                        "{} × {}",
                        format_d_une_piece(largeur, prefs),
                        format_d_une_piece(longueur, prefs)
                    );
                    if ui.selectable_label(false, texte).clicked() {
                        format_choisi = Some((largeur, longueur));
                    }
                }
            });
        if let Some((largeur, longueur)) = format_choisi {
            calepinage.largeur = sans_insecables(&format_d_une_piece(largeur, prefs));
            calepinage.longueur = sans_insecables(&format_d_une_piece(longueur, prefs));
        }
        ui.label(gras(textes::DESSIN_CALEPINAGE_JOINT.traduit()));
        let joint_en_clair = |joint: f64| {
            if joint <= 0.0 {
                textes::DESSIN_CALEPINAGE_SANS_JOINT.traduit().to_string()
            } else {
                petite_mesure(joint, solution, prefs)
            }
        };
        egui::ComboBox::from_id_salt("joint-du-calepinage")
            .selected_text(joint_en_clair(calepinage.joint))
            .show_ui(ui, |ui| {
                for joint in JOINTS {
                    ui.selectable_value(&mut calepinage.joint, joint, joint_en_clair(joint));
                }
            });
    });
    // Le départ : le mur que la pose suit, depuis son premier coin ou depuis
    // le centre de la pièce.
    ui.horizontal_wrapped(|ui| {
        ui.label(gras(textes::DESSIN_CALEPINAGE_LE_LONG_DU_MUR.traduit()));
        let murs = dessin.cotes.len().max(1);
        egui::ComboBox::from_id_salt("mur-du-calepinage")
            .width(44.0)
            .selected_text((calepinage.mur % murs + 1).to_string())
            .show_ui(ui, |ui| {
                for mur in 0..murs {
                    ui.selectable_value(&mut calepinage.mur, mur, (mur + 1).to_string());
                }
            });
        for (depart, texte) in [
            (Depart::Coin, textes::DESSIN_CALEPINAGE_AU_COIN),
            (Depart::Centre, textes::DESSIN_CALEPINAGE_AU_CENTRE),
        ] {
            if ui
                .add(egui::Button::new(texte.traduit()).selected(calepinage.depart == depart))
                .clicked()
            {
                calepinage.depart = depart;
            }
        }
    });
    ui.horizontal_wrapped(|ui| {
        egui::ComboBox::from_id_salt("sens-du-calepinage")
            .selected_text(textes::sens_de_la_pose(calepinage.sens).traduit())
            .show_ui(ui, |ui| {
                for sens in SensDeLaPose::TOUS {
                    ui.selectable_value(
                        &mut calepinage.sens,
                        sens,
                        textes::sens_de_la_pose(sens).traduit(),
                    );
                }
            });
        egui::ComboBox::from_id_salt("decalage-du-calepinage")
            .selected_text(textes::decalage_des_rangs(calepinage.decalage).traduit())
            .show_ui(ui, |ui| {
                for decalage in Decalage::TOUS {
                    ui.selectable_value(
                        &mut calepinage.decalage,
                        decalage,
                        textes::decalage_des_rangs(decalage).traduit(),
                    );
                }
            });
    });
    // Le compte des pièces, la perte et la plus petite coupe.
    for (nom, valeur) in lignes_du_calepinage(dessin, &calepinage, solution, prefs) {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(nom).size(12.5).color(p.texte_attenue));
            ui.label(RichText::new(valeur).size(13.5).strong());
        });
    }
    if calepinage != avant {
        etat.memoriser(dessin);
        dessin.calepinage = calepinage;
        return true;
    }
    false
}

/// Le format d'une pièce à poser, en pouces et fractions (« 12 po », pas
/// « 1 pi 0 po ») : les formats courants sont en pouces, même dans une pièce
/// mesurée en métrique.
fn format_d_une_piece(pouces: f64, prefs: &Preferences) -> String {
    noyau::format::en_texte(
        &noyau::valeur::Valeur::en_base(pouces, Format::PoFrac),
        &noyau::format::OptionsFormat::nouvelles(prefs),
    )
}

/// Une dimension inscrite d'une pièce à poser : en pouces et fractions, ou
/// dans l'unité métrique où elle a été inscrite; le format du matériau si rien
/// n'est inscrit.
fn dimension_a_poser(saisie: &str, defaut: f64, prefs: &Preferences) -> String {
    let lue = noyau::analyse::analyser_longueur(saisie.trim(), FORMAT_PAR_DEFAUT)
        .ok()
        .filter(|v| v.est_longueur() && v.base > 0.0 && v.base.is_finite());
    match lue {
        Some(v) if v.format.systeme() == Some(noyau::unites::Systeme::Metrique) => {
            Quantite::Longueur(v.base).en_clair(Some(v.format), prefs)
        }
        Some(v) => format_d_une_piece(v.base, prefs),
        None if saisie.trim().is_empty() => format_d_une_piece(defaut, prefs),
        None => saisie.trim().to_string(),
    }
}

/// Une petite mesure (un joint, une coupe) : en pouces et fractions, ou en
/// millimètres dans une pièce mesurée en métrique.
fn petite_mesure(pouces: f64, solution: &Solution, prefs: &Preferences) -> String {
    let metrique =
        solution.modele.and_then(Format::systeme) == Some(noyau::unites::Systeme::Metrique);
    if metrique {
        Quantite::Longueur(pouces).en_clair(Some(Format::Mm), prefs)
    } else {
        format_d_une_piece(pouces, prefs)
    }
}

/// Le bilan d'un calepinage : ce qui se pose et son format, les pièces
/// entières et coupées, la perte et la plus petite coupe (évolution 19).
fn lignes_du_calepinage(
    dessin: &Dessin,
    calepinage: &noyau::calepinage::Calepinage,
    solution: &Solution,
    prefs: &Preferences,
) -> Vec<(String, String)> {
    let Some(contour) = dessin.contour_interieur(solution) else {
        return Vec::new();
    };
    let pose = noyau::calepinage::poser(calepinage, &contour);
    let (largeur, longueur) = calepinage.dimensions();
    let titre = format!(
        "{} {} × {}",
        textes::materiau_a_poser(calepinage.materiau).traduit(),
        format_d_une_piece(largeur, prefs),
        format_d_une_piece(longueur, prefs)
    );
    if pose.trop_de_pieces {
        return vec![(titre, textes::DESSIN_CALEPINAGE_TROP.traduit().to_string())];
    }
    let mut lignes = vec![
        (
            titre,
            textes::dessin_pieces_posees(pose.entieres, pose.coupees),
        ),
        (
            textes::DESSIN_CALEPINAGE_PERTE.traduit().to_string(),
            noyau::format::nombre_et_unite(
                &noyau::format::nombre_fixe(pose.perte * 100.0, 1, prefs),
                "%",
            ),
        ),
    ];
    if let Some(coupe) = pose.plus_petite_coupe {
        lignes.push((
            textes::DESSIN_CALEPINAGE_PLUS_PETITE_COUPE
                .traduit()
                .to_string(),
            petite_mesure(coupe, solution, prefs),
        ));
    }
    lignes
}

/// Le calepinage sur la planche : les pièces coupées teintées, les bords des
/// pièces d'un trait pâle et le point de départ (évolution 19).
fn calepinage_sur_la_planche(
    painter: &egui::Painter,
    p: &Palette,
    pose: &noyau::calepinage::Pose,
    vers: &dyn Fn((f64, f64)) -> Pos2,
) {
    // Les coupes en un seul maillage : sans lisière entre leurs triangles.
    let teinte = theme::melanger(p.carte, p.accent_du_plan(), 0.16);
    let mut maillage = egui::epaint::Mesh::default();
    for piece in pose.pieces.iter().filter(|piece| !piece.entiere) {
        for triangle in noyau::maquette::trianguler(&piece.contour) {
            let base = maillage.vertices.len() as u32;
            for coin in triangle {
                maillage.colored_vertex(vers(coin), teinte);
            }
            maillage.add_triangle(base, base + 1, base + 2);
        }
    }
    painter.add(egui::Shape::mesh(maillage));
    let joint = Stroke::new(1.0, theme::melanger(p.carte, p.texte, 0.35));
    for &(a, b) in &pose.traits {
        painter.line_segment([vers(a), vers(b)], joint);
    }
    painter.circle_filled(vers(pose.depart), 4.0, p.accent_du_plan());
}

/// L'ossature dessinée (évolution 28) : la montrer sur le plan, son
/// entraxe, et le compte des montants de chaque mur et de chaque segment de
/// cloison. Retourne vrai si le dessin a changé.
fn ossature_de_la_piece(
    ui: &mut Ui,
    p: &Palette,
    dessin: &mut Dessin,
    etat: &mut Etat,
    solution: &Solution,
    prefs: &Preferences,
) -> bool {
    use noyau::ossature::Entraxe;
    ui.add_space(8.0);
    if !titre_de_section(
        ui,
        p,
        etat,
        Section::Ossature,
        textes::DESSIN_OSSATURE.traduit(),
    ) {
        return false;
    }
    let mut modifie = false;
    ui.horizontal_wrapped(|ui| {
        if ui
            .add(
                egui::Button::new(textes::DESSIN_MONTRER_L_OSSATURE.traduit())
                    .selected(dessin.ossature_visible),
            )
            .clicked()
        {
            etat.memoriser(dessin);
            dessin.ossature_visible = !dessin.ossature_visible;
            modifie = true;
        }
        for entraxe in [Entraxe::Seize, Entraxe::VingtQuatre] {
            if ui
                .add(
                    egui::Button::new(textes::entraxe(entraxe).traduit())
                        .selected(dessin.entraxe == entraxe),
                )
                .clicked()
                && dessin.entraxe != entraxe
            {
                etat.memoriser(dessin);
                dessin.entraxe = entraxe;
                modifie = true;
            }
        }
    });
    for mur in noyau::ossature::ossature(dessin, solution) {
        let linteaux: Vec<String> = mur
            .linteaux
            .iter()
            .map(|&(de, a)| longueur_en_clair(a - de, solution, prefs))
            .collect();
        let mut texte = textes::dessin_montants_du_mur(mur.mur, mur.montants.len());
        if !linteaux.is_empty() {
            texte = format!(
                "{texte} · {} {}",
                textes::DESSIN_LINTEAUX.traduit().to_lowercase(),
                linteaux.join(", ")
            );
        }
        ui.label(RichText::new(texte).size(13.0));
    }
    modifie
}

/// Le compte de l'ossature d'une pièce, pour le bilan : les montants, les
/// lisses et les sablières (trois longueurs de mur), et les linteaux.
fn lignes_de_l_ossature(
    dessin: &Dessin,
    solution: &Solution,
    prefs: &Preferences,
) -> Vec<(String, String)> {
    let murs = noyau::ossature::ossature(dessin, solution);
    if murs.is_empty() {
        return Vec::new();
    }
    let montants: usize = murs.iter().map(|mur| mur.montants.len()).sum();
    let longueur: f64 = murs.iter().map(|mur| mur.longueur()).sum();
    let mut lignes = vec![
        (
            format!(
                "{} ({})",
                textes::DESSIN_MONTANTS.traduit(),
                textes::entraxe(dessin.entraxe).traduit()
            ),
            montants.to_string(),
        ),
        (
            textes::DESSIN_LISSES.traduit().to_string(),
            longueur_en_clair(3.0 * longueur, solution, prefs),
        ),
    ];
    let linteaux: Vec<String> = murs
        .iter()
        .flat_map(|mur| mur.linteaux.iter())
        .map(|&(de, a)| longueur_en_clair(a - de, solution, prefs))
        .collect();
    if !linteaux.is_empty() {
        lignes.push((
            textes::DESSIN_LINTEAUX.traduit().to_string(),
            linteaux.join(", "),
        ));
    }
    lignes
}

/// Pose l'objet de rang `rang` contre le mur ou la cloison le plus proche de
/// son centre, s'il en est un assez près (point 40). Retourne vrai s'il s'y
/// est posé.
fn appuyer_au_mur(
    dessin: &mut Dessin,
    solution: &Solution,
    rang: usize,
    dimensions: (f64, f64),
) -> bool {
    let contour = dessin.contour_interieur(solution);
    let cloisons: Vec<[(f64, f64); 4]> = solution
        .cloisons
        .iter()
        .flatten()
        .flat_map(|cloison| cloison.morceaux())
        .map(|morceau| morceau.coins)
        .collect();
    let faces = mobilier::faces_d_appui(contour.as_deref(), &cloisons);
    // Assez près : à sa profondeur, et 1 pi de plus, du mur.
    let portee = dimensions.1.max(12.0) + 12.0;
    let pas = pas_des_objets(solution);
    mobilier::contre_le_mur(&mut dessin.objets[rang], dimensions, &faces, portee, pas)
}

/// Met l'objet de rang `rang` à la place libre la plus proche (à 10 pi au plus);
/// s'il n'y en a pas, au moins dans la pièce, même contre un autre objet (R-98).
fn placer_au_mieux(dessin: &mut Dessin, solution: &Solution, rang: usize, dimensions: (f64, f64)) {
    let contour = dessin.contour_interieur(solution);
    let pas = pas_des_objets(solution);
    let encombre = encombrement(dessin, solution, rang, contour.as_deref());
    let dans_la_piece = Encombrement {
        obstacles: Vec::new(),
        piece: contour.as_deref(),
    };
    let objet = &dessin.objets[rang];
    let place = mobilier::position_libre_proche(objet, dimensions, &encombre, pas, 120.0)
        .or_else(|| mobilier::position_libre_proche(objet, dimensions, &dans_la_piece, pas, 240.0));
    if let Some(place) = place {
        dessin.objets[rang].centre = place;
    }
}

/// Fait pivoter un objet sans qu'il chevauche quoi que ce soit : s'il le faut,
/// il se dégage de 24 po au plus; sans place, il garde son angle (R-98).
fn pivoter_sans_chevaucher(dessin: &mut Dessin, solution: &Solution, rang: usize, degres: f64) {
    let avant = dessin.objets[rang].clone();
    dessin.objets[rang].pivoter(degres);
    let Some(&dimensions) = solution.objets.get(rang) else {
        return;
    };
    let contour = dessin.contour_interieur(solution);
    let encombre = encombrement(dessin, solution, rang, contour.as_deref());
    let pas = pas_des_objets(solution);
    match mobilier::position_libre_proche(&dessin.objets[rang], dimensions, &encombre, pas, 24.0) {
        Some(place) => dessin.objets[rang].centre = place,
        // Un objet qui chevauchait déjà (un plan d'avant) pivote quand même.
        None if encombre.libre(&avant.coins(dimensions)) => dessin.objets[rang] = avant,
        None => {}
    }
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
    nom_du_dessin: &str,
) -> Option<Action> {
    let mut action = None;
    // Le nom du dessin ouvert, et la liste de tous les dessins.
    if en_tete(ui, p, nom_du_dessin, textes::MES_DESSINS.traduit()) {
        action = Some(Action::MesDessins);
    }
    plan.normaliser();
    if barre_des_pieces(ui, p, plan, etat) {
        action = Some(Action::Modifie);
    }
    if etat.vue_etage {
        return afficher_l_etage(ui, p, plan, etat, prefs, action);
    }
    let courante = plan.courante;
    etat.reperes_des_ouvertures = plan
        .reperes_des_ouvertures()
        .into_iter()
        .nth(courante)
        .unwrap_or_default();
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
        // Agrandie (point 37), elle prend toute la hauteur offerte, moins la
        // rangée d'« Annuler » ou le pavé.
        let mesure_en_cours = etat.choisi.is_some() || etat.autre_choisi.is_some();
        let reserve = match (mesure_en_cours, etat.planche_agrandie) {
            (true, _) => 410.0,
            (false, true) => 100.0,
            (false, false) => 250.0,
        };
        let maximum = if etat.planche_agrandie {
            f32::INFINITY
        } else {
            largeur.max(150.0)
        };
        let hauteur = (ui.available_height() - reserve).clamp(150.0, maximum);
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
        // Le bouton « Agrandir », en haut à gauche des boutons du zoom.
        let zone = Rect::from_min_size(
            Pos2::new(planche.right() - 138.0, planche.top() + 8.0),
            Vec2::new(88.0, 34.0),
        );
        let texte = if etat.planche_agrandie {
            textes::DESSIN_REDUIRE
        } else {
            textes::DESSIN_AGRANDIR
        };
        if bouton_de_planche(
            ui,
            &ui.painter_at(planche),
            zone,
            texte.traduit(),
            textes::DESSIN_AGRANDIR_LA_PLANCHE.traduit(),
            etat.planche_agrandie,
            p,
        ) {
            etat.planche_agrandie = !etat.planche_agrandie;
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
            if etat.planche_agrandie {
                return;
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
            if dessin.fermee && dessin.cotes.len() == 4 {
                diagonales(ui, p, dessin, etat, &solution, prefs);
            }
            if dessin.fermee && retraits(ui, p, dessin, etat, &solution, prefs) {
                action = Some(Action::Modifie);
            }
            if dessin.fermee && murs(ui, p, dessin, etat, &solution, prefs) {
                action = Some(Action::Modifie);
            }
            if matches!(solution.etat, EtatDeLaForme::Complete { .. }) && finis(ui, p, dessin, etat)
            {
                action = Some(Action::Modifie);
            }
            if matches!(solution.etat, EtatDeLaForme::Complete { .. })
                && calepinage_de_la_piece(ui, p, dessin, etat, &solution, prefs)
            {
                action = Some(Action::Modifie);
            }
            if dessin.fermee {
                elevations_des_murs(ui, p, dessin, etat, &solution, prefs);
            }
            if matches!(solution.etat, EtatDeLaForme::Complete { .. })
                && ossature_de_la_piece(ui, p, dessin, etat, &solution, prefs)
            {
                action = Some(Action::Modifie);
            }
            if matches!(solution.etat, EtatDeLaForme::Complete { .. })
                && cloisons(ui, p, dessin, etat, &solution, prefs)
            {
                action = Some(Action::Modifie);
            }
            if matches!(solution.etat, EtatDeLaForme::Complete { .. })
                && espaces_de_la_piece(ui, p, dessin, etat, &solution, prefs)
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

/// Quitte la vue de l'étage : si une autre pièce y a été choisie, l'écran de
/// la pièce repart à neuf (évolution 4).
fn quitter_l_etage(plan: &Plan, etat: &mut Etat) {
    etat.vue_etage = false;
    etat.piece_tiree = None;
    etat.contact = None;
    if plan.courante != etat.courante_avant_l_etage {
        etat.changer_de_piece();
    }
}

/// La vue de l'étage (évolution 4) : toutes les pièces complètes à leur place,
/// qu'on glisse et qu'on tourne, puis la surface de l'étage.
fn afficher_l_etage(
    ui: &mut Ui,
    p: &Palette,
    plan: &mut Plan,
    etat: &mut Etat,
    prefs: &Preferences,
    mut action: Option<Action>,
) -> Option<Action> {
    if plan.placer_les_pieces(FORMAT_PAR_DEFAUT) {
        action = Some(Action::Modifie);
    }
    let largeur = ui.available_width() - 20.0;
    let reserve = if etat.planche_agrandie { 100.0 } else { 250.0 };
    let maximum = if etat.planche_agrandie {
        f32::INFINITY
    } else {
        largeur.max(150.0)
    };
    let hauteur = (ui.available_height() - reserve).clamp(150.0, maximum);
    ui.add_space(6.0);
    let (planche, reponse) = ui
        .horizontal(|ui| {
            ui.add_space(10.0);
            ui.allocate_exact_size(Vec2::new(largeur, hauteur), Sense::click_and_drag())
        })
        .inner;
    reponse.widget_info(|| {
        egui::WidgetInfo::labeled(egui::Role::Canvas, true, textes::DESSIN_ETAGE.traduit())
    });
    if planche_de_l_etage(ui, p, planche, &reponse, plan, etat, prefs) {
        action = Some(Action::Modifie);
    }
    let zone = Rect::from_min_size(
        Pos2::new(planche.right() - 138.0, planche.top() + 8.0),
        Vec2::new(88.0, 34.0),
    );
    let texte = if etat.planche_agrandie {
        textes::DESSIN_REDUIRE
    } else {
        textes::DESSIN_AGRANDIR
    };
    if bouton_de_planche(
        ui,
        &ui.painter_at(planche),
        zone,
        texte.traduit(),
        textes::DESSIN_AGRANDIR_LA_PLANCHE.traduit(),
        etat.planche_agrandie,
        p,
    ) {
        etat.planche_agrandie = !etat.planche_agrandie;
    }
    corps(ui, "dessin", |ui| {
        if section_de_l_etage(ui, p, plan, etat, prefs) {
            action = Some(Action::Modifie);
        }
        if etat.planche_agrandie {
            return;
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

/// La planche de l'étage : les pièces à leur place, leurs murs, leurs portes
/// et leurs fenêtres, leur nom et leur surface; glisser une pièce la déplace
/// (ses murs s'aimantent), glisser ailleurs déplace la vue. Retourne vrai si
/// le plan a changé.
fn planche_de_l_etage(
    ui: &mut Ui,
    p: &Palette,
    planche: Rect,
    reponse: &egui::Response,
    plan: &mut Plan,
    etat: &mut Etat,
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
    let pieces = plan.pieces_dans_l_etage(FORMAT_PAR_DEFAUT);
    if pieces.is_empty() {
        painter.text(
            planche.center(),
            Align2::CENTER_CENTER,
            textes::ETAGE_VIDE.traduit(),
            theme::police(13.0),
            p.texte_attenue,
        );
        return false;
    }
    let centre = planche.center();
    // L'étage entier tient dans la planche; pendant qu'une pièce glisse, le
    // cadrage du début du geste reste.
    let echelle = match &etat.piece_tiree {
        Some(tiree) => tiree.echelle,
        None => {
            let tous: Vec<(f64, f64)> = pieces
                .iter()
                .flat_map(|piece| piece.exterieur.iter().copied())
                .collect();
            etat.cadrage_etage.appliquer(cadrer(planche, &tous), centre)
        }
    };
    // Deux doigts, la molette ou le pincement zooment et déplacent la vue.
    let (deux_doigts, pincement, molette, survol) = ui.input(|i| {
        (
            i.multi_touch(),
            i.zoom_delta(),
            i.smooth_scroll_delta,
            i.pointer.hover_pos(),
        )
    });
    if let Some(toucher) = deux_doigts.filter(|m| planche.contains(m.center_pos)) {
        etat.cadrage_etage
            .zoomer(centre, toucher.center_pos, toucher.zoom_delta);
        etat.cadrage_etage.decalage += toucher.translation_delta;
        etat.piece_tiree = None;
        etat.contact = None;
    } else if let Some(pointeur) = survol.filter(|q| planche.contains(*q)) {
        let facteur = if (pincement - 1.0).abs() > 1e-4 {
            pincement
        } else {
            (molette.y * 0.002).exp()
        };
        if (facteur - 1.0).abs() > 1e-4 {
            etat.cadrage_etage.zoomer(centre, pointeur, facteur);
        }
    }
    let sous = |point: Pos2| {
        let dans_l_etage = echelle.vers_le_plan(point);
        pieces
            .iter()
            .rev()
            .find(|piece| noyau::espaces::dedans(dans_l_etage, &piece.exterieur))
            .map(|piece| piece.rang)
    };
    // Un toucher choisit la pièce dessous.
    if reponse.clicked()
        && let Some(rang) = reponse.interact_pointer_pos().and_then(sous)
    {
        plan.courante = rang;
    }
    // Glisser une pièce la déplace; ses murs s'aimantent à ceux des autres.
    if reponse.drag_started_by(egui::PointerButton::Primary)
        && let Some(rang) = reponse.interact_pointer_pos().and_then(sous)
        && let Some(depart) = plan.pieces[rang].place
    {
        memoriser_les_places(plan, etat);
        plan.courante = rang;
        etat.piece_tiree = Some(PieceTiree {
            rang,
            depart,
            chemin: (0.0, 0.0),
            echelle,
        });
    }
    if reponse.dragged_by(egui::PointerButton::Primary) && etat.piece_tiree.is_some() {
        if let Some(tiree) = &mut etat.piece_tiree {
            let glisse = reponse.drag_delta();
            tiree.chemin.0 += f64::from(glisse.x / echelle.facteur);
            tiree.chemin.1 += f64::from(glisse.y / echelle.facteur);
            let proposee = tiree.depart.deplacee(tiree.chemin);
            let seuil = f64::from(14.0 / echelle.facteur);
            let (place, contact) =
                noyau::etage::aimanter(plan, tiree.rang, proposee, seuil, FORMAT_PAR_DEFAUT);
            if let Some(piece) = plan.pieces.get_mut(tiree.rang) {
                piece.place = Some(place);
            }
            etat.contact = contact;
            modifie = true;
        }
    } else if reponse.dragged() {
        etat.cadrage_etage.decalage += reponse.drag_delta();
    }
    if reponse.drag_stopped() {
        etat.piece_tiree = None;
        etat.contact = None;
    }
    // Les pièces, à leur place d'après le geste, en passes : les intérieurs,
    // les murs (interrompus aussi par les ouvertures des voisines, dans les
    // murs communs), les faces, les portes et les fenêtres, puis les noms.
    let pieces = plan.pieces_dans_l_etage(FORMAT_PAR_DEFAUT);
    for piece in &pieces {
        let choisie = piece.rang == plan.courante;
        let teinte = theme::melanger(
            p.carte,
            if choisie {
                p.accent_du_plan()
            } else {
                p.primaire
            },
            if choisie { 0.12 } else { 0.05 },
        );
        let mut maillage = egui::epaint::Mesh::default();
        for triangle in noyau::maquette::trianguler(&piece.interieur) {
            let base = maillage.vertices.len() as u32;
            for coin in triangle {
                maillage.colored_vertex(echelle.vers_l_ecran(coin), teinte);
            }
            maillage.add_triangle(base, base + 1, base + 2);
        }
        painter.add(egui::Shape::mesh(maillage));
    }
    for piece in &pieces {
        for morceau in plan.murs_dans_l_etage(piece.rang, FORMAT_PAR_DEFAUT) {
            dessiner_un_morceau_de_mur(&painter, &echelle, &morceau, None, p);
        }
    }
    let mut symboles = Vec::new();
    for piece in &pieces {
        let dessin = &plan.pieces[piece.rang].dessin;
        let solution = dessin.resoudre(FORMAT_PAR_DEFAUT);
        let choisie = piece.rang == plan.courante;
        let contour: Vec<Pos2> = piece
            .interieur
            .iter()
            .map(|&c| echelle.vers_l_ecran(c))
            .collect();
        let trait_ = if choisie {
            Stroke::new(2.5, p.accent_du_plan())
        } else {
            Stroke::new(1.5, p.primaire)
        };
        painter.add(egui::Shape::closed_line(contour, trait_));
        let Some(sommets) = dessin.sommets_mesures(&solution) else {
            continue;
        };
        let place = piece.place;
        let ecran: Vec<Pos2> = sommets
            .iter()
            .map(|&s| echelle.vers_l_ecran(place.vers_l_etage(s)))
            .collect();
        let aire_signee: f32 = ecran
            .windows(2)
            .map(|s| s[0].x * s[1].y - s[1].x * s[0].y)
            .sum();
        // Un morceau d'un mur, à l'écran : ses deux bouts et le dedans.
        let sur_le_mur = |cote: usize, debut: f64, fin: f64| {
            let (&a, &b) = (ecran.get(cote)?, ecran.get(cote + 1)?);
            let longueur = f64::from(a.distance(b) / echelle.facteur);
            if longueur < 1e-6 {
                return None;
            }
            let le_long = b - a;
            let unite = le_long.normalized();
            let dedans = if aire_signee > 0.0 {
                Vec2::new(-unite.y, unite.x)
            } else {
                Vec2::new(unite.y, -unite.x)
            };
            Some((
                a + le_long * (debut / longueur) as f32,
                a + le_long * (fin / longueur) as f32,
                dedans,
            ))
        };
        // La face s'interrompt aux passages des voisines et aux ouvertures.
        for (cote, debut, fin) in plan.passages_des_voisins(piece.rang, FORMAT_PAR_DEFAUT) {
            if let Some((q0, q1, _)) = sur_le_mur(cote, debut, fin) {
                painter.line_segment([q0, q1], Stroke::new(7.0, p.carte));
            }
        }
        let epaisseur = solution.epaisseur_des_murs as f32 * echelle.facteur;
        for placement in solution.placements.iter().flatten() {
            if let Some((q0, q1, dedans)) =
                sur_le_mur(placement.cote, placement.debut, placement.fin)
            {
                painter.line_segment([q0, q1], Stroke::new(7.0, p.carte));
                let epaisseur = if dessin.mesures_exterieures {
                    -epaisseur
                } else {
                    epaisseur
                };
                symboles.push((q0, q1, dedans, *placement, epaisseur));
            }
        }
    }
    for (q0, q1, dedans, placement, epaisseur) in symboles {
        dessiner_ouverture(
            &painter,
            (q0, q1),
            dedans,
            (
                placement.genre,
                placement.inversee,
                (placement.haut - placement.bas) as f32 * echelle.facteur,
                echelle.facteur,
            ),
            p.texte,
            p.primaire,
            epaisseur,
        );
    }
    for piece in &pieces {
        let modele = plan.pieces[piece.rang]
            .dessin
            .resoudre(FORMAT_PAR_DEFAUT)
            .modele;
        let choisie = piece.rang == plan.courante;
        let (x0, x1, y0, y1) = piece.interieur.iter().fold(
            (f64::MAX, f64::MIN, f64::MAX, f64::MIN),
            |(x0, x1, y0, y1), &(x, y)| (x0.min(x), x1.max(x), y0.min(y), y1.max(y)),
        );
        let milieu = echelle.vers_l_ecran(((x0 + x1) / 2.0, (y0 + y1) / 2.0));
        let largeur = (x1 - x0) as f32 * echelle.facteur - 8.0;
        texte_qui_tient(
            &painter,
            milieu - Vec2::new(0.0, 8.0),
            nom_de_la_piece(plan, piece.rang),
            (theme::police_grasse(12.5), 8.5),
            largeur,
            if choisie { p.accent_du_plan() } else { p.texte },
        );
        texte_qui_tient(
            &painter,
            milieu + Vec2::new(0.0, 8.0),
            Quantite::Aire(piece.aire).en_clair(modele, prefs),
            (theme::police(11.5), 8.0),
            largeur,
            p.texte_attenue,
        );
    }
    // Les deux murs qui se touchent, pendant que la pièce glisse.
    if let (Some(contact), Some(tiree)) = (etat.contact, &etat.piece_tiree) {
        let cote = |rang: usize, k: usize| {
            pieces.iter().find(|piece| piece.rang == rang).map(|piece| {
                let n = piece.interieur.len();
                (piece.interieur[k % n], piece.interieur[(k + 1) % n])
            })
        };
        for (a, b) in [
            cote(tiree.rang, contact.cote),
            cote(contact.autre, contact.cote_de_l_autre),
        ]
        .into_iter()
        .flatten()
        {
            painter.line_segment(
                [echelle.vers_l_ecran(a), echelle.vers_l_ecran(b)],
                Stroke::new(4.0, p.accent_du_plan()),
            );
        }
    }
    painter.text(
        Pos2::new(planche.left() + 12.0, planche.bottom() - 12.0),
        Align2::LEFT_BOTTOM,
        textes::ETAGE_CONSIGNE.traduit(),
        theme::police(11.5),
        p.texte_attenue,
    );
    boutons_du_zoom(ui, p, &painter, planche, centre, &mut etat.cadrage_etage);
    modifie
}

/// Écrit un texte centré s'il tient dans `largeur` points, sa police
/// rapetissée au besoin, mais pas sous `plus_petite`; sinon, ne l'écrit pas.
fn texte_qui_tient(
    painter: &egui::Painter,
    centre: Pos2,
    texte: String,
    (police, plus_petite): (egui::FontId, f32),
    largeur: f32,
    couleur: Color32,
) {
    let mesure = painter
        .layout_no_wrap(texte.clone(), police.clone(), couleur)
        .size()
        .x;
    let taille = if mesure <= largeur {
        police.size
    } else {
        police.size * largeur / mesure
    };
    if taille >= plus_petite {
        painter.text(
            centre,
            Align2::CENTER_CENTER,
            texte,
            egui::FontId::new(taille, police.family),
            couleur,
        );
    }
}

/// Garde les places des pièces, pour « Annuler » dans l'étage.
fn memoriser_les_places(plan: &Plan, etat: &mut Etat) {
    etat.places_d_avant
        .push(plan.pieces.iter().map(|piece| piece.place).collect());
    if etat.places_d_avant.len() > PROFONDEUR_DE_L_HISTORIQUE {
        etat.places_d_avant.remove(0);
    }
}

/// La section de l'étage : sa surface, la pièce choisie (la tourner,
/// l'ouvrir), « Annuler », et la surface de chaque pièce. Retourne vrai si le
/// plan a changé.
fn section_de_l_etage(
    ui: &mut Ui,
    p: &Palette,
    plan: &mut Plan,
    etat: &mut Etat,
    prefs: &Preferences,
) -> bool {
    let mut modifie = false;
    let pieces = plan.pieces_dans_l_etage(FORMAT_PAR_DEFAUT);
    let modele = plan
        .pieces
        .iter()
        .find_map(|piece| piece.dessin.resoudre(FORMAT_PAR_DEFAUT).modele);
    let aire = |po2: f64| Quantite::Aire(po2).en_clair(modele, prefs);
    ui.add_space(8.0);
    ui.horizontal_wrapped(|ui| {
        ui.add_space(10.0);
        ui.label(RichText::new(textes::ETAGE_SURFACE.traduit()).font(theme::police_grasse(14.0)));
        ui.label(
            RichText::new(aire(pieces.iter().map(|piece| piece.aire).sum()))
                .size(15.0)
                .strong()
                .color(p.titre),
        );
    });
    let absentes = plan.pieces.len() - pieces.len();
    if absentes > 0 {
        ui.horizontal(|ui| {
            ui.add_space(10.0);
            ui.label(
                RichText::new(textes::etage_pieces_incompletes(absentes))
                    .size(12.5)
                    .color(p.texte_attenue),
            );
        });
    }
    ui.add_space(6.0);
    let courante = plan.courante;
    let placee = pieces.iter().any(|piece| piece.rang == courante);
    ui.horizontal_wrapped(|ui| {
        ui.add_space(10.0);
        ui.label(RichText::new(nom_de_la_piece(plan, courante)).font(theme::police_grasse(13.5)));
        for (texte, degres) in [
            (textes::ETAGE_TOURNER_A_GAUCHE, -90.0),
            (textes::ETAGE_TOURNER_A_DROITE, 90.0),
        ] {
            if ui
                .add_enabled(placee, egui::Button::new(texte.traduit()))
                .clicked()
            {
                memoriser_les_places(plan, etat);
                if plan.tourner_dans_l_etage(courante, degres, FORMAT_PAR_DEFAUT) {
                    modifie = true;
                }
            }
        }
        if ui
            .add(egui::Button::new(textes::ETAGE_OUVRIR.traduit()))
            .clicked()
        {
            quitter_l_etage(plan, etat);
        }
        if !etat.places_d_avant.is_empty()
            && ui
                .add(egui::Button::new(textes::DESSIN_ANNULER.traduit()))
                .clicked()
            && let Some(places) = etat.places_d_avant.pop()
        {
            for (piece, place) in plan.pieces.iter_mut().zip(places) {
                piece.place = place;
            }
            modifie = true;
        }
    });
    ui.add_space(4.0);
    for piece in &pieces {
        ui.horizontal(|ui| {
            ui.add_space(10.0);
            ui.label(
                RichText::new(nom_de_la_piece(plan, piece.rang))
                    .size(12.5)
                    .color(p.texte_attenue),
            );
            ui.label(RichText::new(aire(piece.aire)).size(13.5).strong());
        });
    }
    // Les murs communs : deux pièces voisines de part et d'autre d'un seul mur.
    let communs = plan.murs_communs(FORMAT_PAR_DEFAUT);
    if !communs.is_empty() {
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.add_space(10.0);
            ui.label(
                RichText::new(textes::ETAGE_MURS_COMMUNS.traduit())
                    .font(theme::police_grasse(13.5)),
            );
        });
        for mur in &communs {
            ui.horizontal(|ui| {
                ui.add_space(10.0);
                ui.label(
                    RichText::new(format!(
                        "{} \u{2013} {}",
                        nom_de_la_piece(plan, mur.pieces.0),
                        nom_de_la_piece(plan, mur.pieces.1)
                    ))
                    .size(12.5)
                    .color(p.texte_attenue),
                );
                ui.label(
                    RichText::new(Quantite::Longueur(mur.longueur()).en_clair(modele, prefs))
                        .size(13.5)
                        .strong(),
                );
            });
        }
    }
    modifie
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
                // L'étage, en tête : toutes les pièces à leur place (évolution 4).
                if plan.pieces.len() > 1 || etat.vue_etage {
                    let etage =
                        egui::Button::new(textes::DESSIN_ETAGE.traduit()).selected(etat.vue_etage);
                    if ui.add(etage).clicked() {
                        if etat.vue_etage {
                            quitter_l_etage(plan, etat);
                        } else {
                            etat.vue_etage = true;
                            etat.courante_avant_l_etage = plan.courante;
                            etat.renommer = false;
                            etat.choisir(None);
                        }
                    }
                }
                for rang in 0..plan.pieces.len() {
                    let bouton = egui::Button::new(nom_de_la_piece(plan, rang))
                        .selected(rang == plan.courante);
                    if ui.add(bouton).clicked() {
                        if etat.vue_etage {
                            // Dans l'étage, toucher une pièce l'ouvre.
                            plan.courante = rang;
                            quitter_l_etage(plan, etat);
                            modifie = true;
                        } else if rang == plan.courante {
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
                    // Une pièce neuve se dessine dans sa vue à elle.
                    etat.vue_etage = false;
                    etat.changer_de_piece();
                    modifie = true;
                }
                let renommer =
                    egui::Button::new(textes::DESSIN_RENOMMER.traduit()).selected(etat.renommer);
                if ui.add(renommer).clicked() {
                    etat.renommer = !etat.renommer;
                    etat.viser_le_nom = etat.renommer;
                }
                // Une copie de la pièce, juste après elle.
                if plan.pieces.len() < noyau::dessin::MAX_PIECES
                    && ui
                        .add(egui::Button::new(textes::DESSIN_DUPLIQUER.traduit()))
                        .clicked()
                    && plan.dupliquer(textes::nom_de_copie(&nom_de_la_piece(plan, plan.courante)))
                {
                    etat.changer_de_piece();
                    modifie = true;
                }
                // La pièce retournée, la gauche et la droite échangées.
                if ui
                    .add(egui::Button::new(textes::DESSIN_MIROIR.traduit()))
                    .clicked()
                {
                    let dessin = &mut plan.pieces[plan.courante].dessin;
                    etat.memoriser(dessin);
                    dessin.miroir();
                    etat.cadrage = Cadrage::default();
                    etat.choisir(None);
                    modifie = true;
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
    // Le bordereau des portes et des fenêtres de tout le plan (évolution 24).
    let bordereau = plan.bordereau(FORMAT_PAR_DEFAUT);
    if !bordereau.is_empty() {
        ui.add_space(10.0);
        if titre_de_section(
            ui,
            p,
            etat,
            Section::Bordereau,
            textes::DESSIN_BORDEREAU.traduit(),
        ) {
            let modele = plan
                .pieces
                .iter()
                .find_map(|piece| piece.dessin.resoudre(FORMAT_PAR_DEFAUT).modele);
            carte(ui, p, |ui| {
                for ligne in &bordereau {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(
                            RichText::new(&ligne.repere)
                                .font(theme::police_grasse(14.0))
                                .color(p.titre),
                        );
                        for texte in ligne_du_bordereau(ligne, plan, modele, prefs)
                            .into_iter()
                            .skip(1)
                        {
                            ui.label(RichText::new(texte).size(13.5));
                        }
                    });
                }
            });
        }
    }
    // Le tableau des finis de tout le plan (évolution 27).
    let tableau = plan.tableau_des_finis(FORMAT_PAR_DEFAUT);
    if !tableau.pieces.is_empty() {
        ui.add_space(10.0);
        if titre_de_section(
            ui,
            p,
            etat,
            Section::TableauDesFinis,
            textes::DESSIN_TABLEAU_DES_FINIS.traduit(),
        ) {
            let modele = plan
                .pieces
                .iter()
                .find_map(|piece| piece.dessin.resoudre(FORMAT_PAR_DEFAUT).modele);
            carte(ui, p, |ui| {
                for (rang, finis, _) in &tableau.pieces {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(
                            RichText::new(nom_de_la_piece(plan, *rang))
                                .font(theme::police_grasse(13.5))
                                .color(p.titre),
                        );
                        ui.label(RichText::new(finis_en_clair(finis).join(" · ")).size(13.5));
                    });
                }
                ui.add_space(4.0);
                ui.label(
                    RichText::new(textes::DESSIN_QUANTITES_PAR_FINI.traduit())
                        .font(theme::police_grasse(13.5))
                        .color(p.titre),
                );
                for (fini, quantite) in &tableau.totaux {
                    ui.label(
                        RichText::new(format!(
                            "{} : {}",
                            textes::fini_en_clair(fini),
                            quantite_d_un_fini(fini, *quantite, modele, prefs)
                        ))
                        .size(13.5),
                    );
                }
            });
        }
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
    if !titre_de_section(
        ui,
        p,
        etat,
        Section::Exporter,
        textes::DESSIN_EXPORTER.traduit(),
    ) {
        return (modifie, export);
    }
    // La mise en page du PDF (point 36) : l'échelle, et le cartouche.
    ui.label(RichText::new(textes::DESSIN_MISE_EN_PAGE.traduit()).size(13.5));
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(textes::DESSIN_ECHELLE.traduit()).size(13.5));
        for echelle in EchelleDuPlan::TOUTES {
            let choisie = plan.mise_en_page.echelle == echelle;
            let bouton =
                egui::Button::new(textes::nom_de_l_echelle(echelle).traduit()).selected(choisie);
            if ui.add(bouton).clicked() && !choisie {
                plan.mise_en_page.echelle = echelle;
                modifie = true;
            }
        }
    });
    // Où tiendrait la pièce à l'écran, à cette échelle.
    if let Some(avis) = plan.pieces.get(plan.courante).and_then(|piece| {
        crate::exportation::avis_de_l_echelle(&piece.dessin, plan.mise_en_page.echelle, prefs)
    }) {
        ui.label(RichText::new(avis).size(12.5).color(p.texte_attenue));
    }
    let mise_en_page = &mut plan.mise_en_page;
    for (libelle, champ) in [
        (textes::DESSIN_PROJET, &mut mise_en_page.projet),
        (textes::DESSIN_ADRESSE, &mut mise_en_page.adresse),
        (textes::DESSIN_DESSINE_PAR, &mut mise_en_page.dessine_par),
    ] {
        ui.horizontal(|ui| {
            let etiquette = ui.label(RichText::new(libelle.traduit()).size(13.5));
            let reponse = ui
                .add(
                    egui::TextEdit::singleline(champ)
                        .desired_width((ui.available_width() - 8.0).clamp(120.0, 300.0))
                        .char_limit(80),
                )
                .labelled_by(etiquette.id);
            cadre_de_saisie(ui, p, &reponse);
            if reponse.changed() {
                modifie = true;
            }
        });
    }
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
    // Le crayon sorti (évolution 30) : un glissement trace un trait, et ne
    // saisit ni mur, ni cloison, ni ouverture, ni objet, ni note.
    let crayon_actif = etat.crayon_sorti && matches!(solution.etat, EtatDeLaForme::Complete { .. });
    // Un mur se saisit par la poignée de son milieu : il glisse parallèlement
    // à lui-même, et ses voisins s'allongent ou raccourcissent.
    if matches!(solution.etat, EtatDeLaForme::Complete { .. })
        && solution.forme_reelle.is_none()
        && etat.mur_tire.is_none()
        && etat.cloison_etiree.is_none()
        && !etat.cloison_a_tracer
        && !crayon_actif
        && reponse.drag_started_by(egui::PointerButton::Primary)
        && let Some(origine) = ui.input(|i| i.pointer.press_origin())
        && let Some(echelle) = etat.derniere_echelle
        && let Some(sommets) = dessin.sommets_mesures(solution)
        && let Some(cote) = (0..sommets.len().saturating_sub(1)).find(|&i| {
            let milieu = (
                (sommets[i].0 + sommets[i + 1].0) / 2.0,
                (sommets[i].1 + sommets[i + 1].1) / 2.0,
            );
            echelle.vers_l_ecran(milieu).distance(origine) <= PORTEE_D_UNE_POIGNEE
        })
    {
        let dehors = noyau::dessin::dehors_des_cotes(&sommets)[cote];
        etat.memoriser(dessin);
        etat.mur_tire = Some(MurTire {
            cote,
            avant: dessin.clone(),
            origine,
            dehors: Vec2::new(dehors.0 as f32, dehors.1 as f32),
            pouces_par_point: 1.0 / f64::from(echelle.facteur),
            echelle,
        });
        etat.choisir(None);
    }
    // Une cloison se saisit aussi par la poignée du milieu de chacun de ses
    // segments (point 38) : le segment glisse parallèlement à lui-même.
    if matches!(solution.etat, EtatDeLaForme::Complete { .. })
        && etat.mur_tire.is_none()
        && etat.cloison_etiree.is_none()
        && !etat.cloison_a_tracer
        && !crayon_actif
        && reponse.drag_started_by(egui::PointerButton::Primary)
        && let Some(origine) = ui.input(|i| i.pointer.press_origin())
        && let Some(echelle) = etat.derniere_echelle
        && let Some((rang, segment, normale)) =
            solution
                .cloisons
                .iter()
                .enumerate()
                .find_map(|(rang, cloison)| {
                    let cloison = cloison.as_ref()?;
                    cloison.sommets.windows(2).enumerate().find_map(|(k, s)| {
                        let milieu = ((s[0].0 + s[1].0) / 2.0, (s[0].1 + s[1].1) / 2.0);
                        let longueur = (s[1].0 - s[0].0).hypot(s[1].1 - s[0].1);
                        (longueur > 1e-9
                            && echelle.vers_l_ecran(milieu).distance(origine)
                                <= PORTEE_D_UNE_POIGNEE)
                            .then(|| {
                                let u =
                                    ((s[1].0 - s[0].0) / longueur, (s[1].1 - s[0].1) / longueur);
                                (rang, k, Vec2::new(-u.1 as f32, u.0 as f32))
                            })
                    })
                })
    {
        etat.memoriser(dessin);
        etat.cloison_etiree = Some(CloisonEtiree {
            rang,
            segment,
            avant: dessin.clone(),
            origine,
            normale,
            pouces_par_point: 1.0 / f64::from(echelle.facteur),
            echelle,
        });
        etat.cloison_choisie = Some(rang);
        etat.choisir(None);
    }
    // Pendant le tracé d'une cloison, rien d'autre ne se saisit.
    let saisie = if dessin.fermee
        && etat.mur_tire.is_none()
        && etat.cloison_etiree.is_none()
        && !etat.cloison_a_tracer
        && !crayon_actif
        && reponse.drag_started_by(egui::PointerButton::Primary)
    {
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
        && !crayon_actif
        && reponse.drag_started_by(egui::PointerButton::Primary)
        && let Some(origine) = ui.input(|i| i.pointer.press_origin())
        && let Some(echelle) = etat.derniere_echelle
    {
        let depart = echelle.vers_le_plan(origine);
        etat.cloison_en_trace = Some(vec![depart]);
    }
    // Une note saisie au doigt se déplace aussi; elle passe avant les objets,
    // qu'elle recouvre.
    let note_saisie = if complete
        && etat.mur_tire.is_none()
        && etat.cloison_etiree.is_none()
        && etat.cloison_en_trace.is_none()
        && etat.ouverture_tiree.is_none()
        && !crayon_actif
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
        && etat.mur_tire.is_none()
        && etat.cloison_etiree.is_none()
        && etat.cloison_en_trace.is_none()
        && etat.ouverture_tiree.is_none()
        && etat.note_tiree.is_none()
        && !crayon_actif
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
        && etat.mur_tire.is_none()
        && etat.cloison_etiree.is_none()
        && etat.cloison_en_trace.is_none()
        && etat.cloison_tiree.is_none()
        && etat.ouverture_tiree.is_none()
        && etat.note_tiree.is_none()
        && !crayon_actif
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
            && !crayon_actif
            && reponse.dragged_by(egui::PointerButton::Primary)
            && etat.ouverture_tiree.is_none()
            && etat.objet_tire.is_none()
            && etat.note_tiree.is_none()
            && etat.cloison_tiree.is_none()
            && etat.cloison_en_trace.is_none()
            && etat.mur_tire.is_none()
            && etat.cloison_etiree.is_none());
    if deplacer && !etat.geste_annule {
        etat.cadrage.decalage += reponse.drag_delta();
    }
    // Le croquis à main levée (évolution 30) : le crayon trace au doigt,
    // par-dessus le plan.
    if crayon_actif
        && reponse.drag_started_by(egui::PointerButton::Primary)
        && let Some(origine) = ui.input(|i| i.pointer.press_origin())
        && let Some(echelle) = etat.derniere_echelle
    {
        etat.trait_en_cours = Some(vec![echelle.vers_le_plan(origine)]);
    }
    if crayon_actif
        && let Some(echelle) = etat.derniere_echelle
        && (reponse.dragged_by(egui::PointerButton::Primary) || reponse.drag_stopped())
        && let Some(points) = etat.trait_en_cours.as_mut()
    {
        // Toutes les positions de l'image : un geste rapide en apporte
        // plusieurs à la fois.
        let positions: Vec<Pos2> = ui.input(|i| {
            i.events
                .iter()
                .filter_map(|evenement| match evenement {
                    egui::Event::PointerMoved(position) => Some(*position),
                    _ => None,
                })
                .collect()
        });
        for position in positions.into_iter().chain(reponse.interact_pointer_pos()) {
            let assez_loin = points
                .last()
                .is_none_or(|dernier| echelle.vers_l_ecran(*dernier).distance(position) > 1.5);
            if assez_loin {
                points.push(echelle.vers_le_plan(position));
            }
        }
    }
    if reponse.drag_stopped()
        && let Some(points) = etat.trait_en_cours.take()
        && let Some(echelle) = etat.derniere_echelle
        && points.len() > 1
        && !etat.geste_annule
    {
        // Allégé au point de l'écran près.
        let tolerance = f64::from(1.0 / echelle.facteur.max(1e-3));
        etat.memoriser(dessin);
        dessin.croquis.push(noyau::dessin::TraitAMainLevee {
            // Au dixième de pouce près : le fichier reste léger.
            points: noyau::dessin::simplifier_le_trait(&points, tolerance)
                .into_iter()
                .map(|(x, y)| ((x * 10.0).round() / 10.0, (y * 10.0).round() / 10.0))
                .collect(),
            crayon: etat.crayon,
        });
        modifie = true;
    }

    // Une pièce complète se cadre à l'extérieur de ses murs.
    let contour_exterieur = if complete {
        dessin.contour_exterieur(solution)
    } else {
        None
    };
    let a_cadrer = contour_exterieur.as_deref().unwrap_or(sommets);
    // Les cloisons s'appuient sur la face intérieure des murs.
    let contour_interieur = if complete {
        dessin.contour_interieur(solution)
    } else {
        None
    };
    let faces_interieures = contour_interieur.as_deref().unwrap_or(sommets);
    // Avec ses lignes de cote, la forme leur laisse de la place, et à leurs
    // textes (point 35).
    let zone_cadree = if complete && dessin.lignes_de_cote {
        zone_de_la_forme.shrink2(Vec2::new(60.0, 26.0))
    } else {
        zone_de_la_forme
    };
    let echelle = etat
        .cadrage
        .appliquer(cadrer(zone_cadree, a_cadrer), centre_de_la_planche);
    // Pendant qu'un mur s'étire, le plan garde son cadrage : le mur suit le doigt.
    let echelle = etat
        .mur_tire
        .as_ref()
        .map(|tire| tire.echelle)
        .or_else(|| etat.cloison_etiree.as_ref().map(|tire| tire.echelle))
        .unwrap_or(echelle);
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
            let pas = pas_des_objets(solution);
            let voulu = (
                ((point.0 - ecart.0) / pas).round() * pas,
                ((point.1 - ecart.1) / pas).round() * pas,
            );
            let genre = dessin.objets[rang].genre;
            if !genre.encombre()
                && genre.contre_un_mur()
                && let Some(&dimensions) = solution.objets.get(rang)
            {
                // Un symbole du mur glisse le long des murs et des cloisons,
                // et s'en détache plus loin (point 40).
                let avant = dessin.objets[rang].clone();
                dessin.objets[rang].centre = voulu;
                appuyer_au_mur(dessin, solution, rang, dimensions);
                if dessin.objets[rang] != avant {
                    modifie = true;
                }
            } else {
                // Il s'arrête contre ce qui l'encombre et glisse le long (R-98).
                let centre = match solution.objets.get(rang) {
                    Some(&dimensions) if dessin.objets[rang].centre != voulu => {
                        let contour = dessin.contour_interieur(solution);
                        let encombre = encombrement(dessin, solution, rang, contour.as_deref());
                        mobilier::deplacer_sans_chevaucher(
                            &dessin.objets[rang],
                            dimensions,
                            voulu,
                            &encombre,
                            pas,
                        )
                    }
                    _ => voulu,
                };
                if dessin.objets[rang].centre != centre {
                    dessin.objets[rang].centre = centre;
                    modifie = true;
                }
            }
        }
        if !reponse.dragged() {
            etat.objet_tire = None;
        }
    }
    // Le mur qui glisse, au pouce près (au centimètre en métrique) : chaque
    // image repart du dessin du début du geste.
    if let Some(tire) = etat.mur_tire.take() {
        if reponse.dragged()
            && let Some(pointeur) = reponse.interact_pointer_pos()
        {
            let pas = pas_du_plan(solution);
            let brut =
                f64::from((pointeur - tire.origine).dot(tire.dehors)) * tire.pouces_par_point;
            let delta = (brut / pas).round() * pas;
            let base = tire.avant.resoudre(FORMAT_PAR_DEFAUT);
            let ecrire = |pouces: f64| {
                sans_insecables(&Quantite::Longueur(pouces).en_clair(base.modele, prefs))
            };
            let etire = if delta.abs() < 1e-9 {
                Some(tire.avant.clone())
            } else {
                tire.avant.etirer(tire.cote, delta, &base, &ecrire)
            };
            if let Some(etire) = etire
                && *dessin != etire
            {
                *dessin = etire;
                modifie = true;
            }
        }
        if reponse.dragged() {
            etat.mur_tire = Some(tire);
        }
    }
    // Le segment de cloison qui glisse, au pouce près (point 38).
    if let Some(tire) = etat.cloison_etiree.take() {
        if reponse.dragged()
            && let Some(pointeur) = reponse.interact_pointer_pos()
        {
            let pas = pas_du_plan(solution);
            let brut =
                f64::from((pointeur - tire.origine).dot(tire.normale)) * tire.pouces_par_point;
            let delta = (brut / pas).round() * pas;
            let base = tire.avant.resoudre(FORMAT_PAR_DEFAUT);
            let ecrire = |pouces: f64| {
                sans_insecables(&Quantite::Longueur(pouces).en_clair(base.modele, prefs))
            };
            let etire = if delta.abs() < 1e-9 {
                Some(tire.avant.clone())
            } else {
                tire.avant
                    .etirer_une_cloison(tire.rang, tire.segment, delta, &base, &ecrire)
            };
            if let Some(etire) = etire
                && *dessin != etire
            {
                *dessin = etire;
                modifie = true;
            }
        }
        if reponse.dragged() {
            etat.cloison_etiree = Some(tire);
        }
    }
    // La cloison qu'on trace suit le doigt, coins compris; elle naît quand il
    // se lève.
    if let Some(mut trace) = etat.cloison_en_trace.take() {
        // Toutes les positions de cette image : un geste rapide en apporte
        // plusieurs à la fois.
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
            let point = echelle.vers_le_plan(position);
            if trace
                .last()
                .is_none_or(|d| (d.0 - point.0).hypot(d.1 - point.1) > 0.5)
            {
                trace.push(point);
            }
        }
        if reponse.dragged() {
            etat.cloison_en_trace = Some(trace);
        } else {
            let appuis = appuis_des_cloisons(faces_interieures, solution);
            if let Some((debut, segments)) = redresser_le_trace(
                &trace,
                &appuis,
                aimant_des_cloisons(&echelle),
                pas_du_plan(solution),
            ) && let Some((premier, suite)) = segments.split_first()
            {
                let ecrire = |pouces: f64| {
                    sans_insecables(&Quantite::Longueur(pouces).en_clair(solution.modele, prefs))
                };
                etat.memoriser(dessin);
                dessin.cloisons.push(Cloison {
                    debut,
                    sens: premier.0,
                    longueur: ecrire(premier.1),
                    epaisseur: String::new(),
                    porte: None,
                    ouvertures: Vec::new(),
                    suite: suite
                        .iter()
                        .map(|&(sens, longueur)| SegmentDeCloison {
                            sens,
                            longueur: ecrire(longueur),
                        })
                        .collect(),
                });
                etat.cloison_choisie = Some(dessin.cloisons.len() - 1);
                etat.cloison_a_tracer = false;
                etat.crayon_sorti = false;
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
        && complete
        && etat.mesure.is_some()
        && let Some(position) = reponse.interact_pointer_pos()
    {
        // L'outil « Mesurer » : chaque toucher pose un point, accroché à un
        // coin ou à une face tout près; un troisième recommence (évolution 23).
        let point = point_a_mesurer(position, &echelle, dessin, solution);
        let points = etat.mesure.get_or_insert_with(Vec::new);
        if points.len() >= 2 {
            points.clear();
        }
        points.push(point);
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
        // L'objet se pose là où la pièce a été touchée, ou à la place libre
        // la plus proche (R-98).
        let (x, y) = echelle.vers_le_plan(position);
        etat.memoriser(dessin);
        dessin
            .objets
            .push(Objet::nouveau(genre, (x.round(), y.round())));
        let rang = dessin.objets.len() - 1;
        let dimensions = dessin.objets[rang].dimensions_de_depart();
        // Un objet du mur s'appuie au mur le plus proche du doigt (point 40).
        if genre.contre_un_mur() {
            appuyer_au_mur(dessin, solution, rang, dimensions);
        }
        placer_au_mieux(dessin, solution, rang, dimensions);
        etat.objet_a_placer = None;
        etat.objet_choisi = Some(rang);
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
        // Toucher une cloison la choisit, et ouvre le pavé sur la longueur du
        // segment touché, comme un côté de mur (point 29).
        etat.cloison_choisie = Some(rang);
        etat.objet_choisi = None;
        etat.note_choisie = None;
        if let Some(Some(cloison)) = solution.cloisons.get(rang) {
            let segment = segment_le_plus_proche(cloison, echelle.vers_le_plan(position));
            etat.choisir_autre(Some(AutreSaisie::Cloison(
                rang,
                ChampDeCloison::Longueur(segment),
            )));
        }
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
    // Les conflits du plan (évolution 23) : un battant qui frappe un meuble,
    // un meuble qui traverse un mur ou une cloison, deux meubles l'un sur
    // l'autre; en rouge.
    let conflits = if complete {
        dessin.conflits(solution)
    } else {
        Vec::new()
    };
    // La trame du plancher, sous tout le reste (évolution 27); le calepinage
    // prend sa place quand il paraît (évolution 19).
    if complete
        && dessin.calepinage.visible
        && let Some(contour) = dessin.contour_interieur(solution)
    {
        let pose = noyau::calepinage::poser(&dessin.calepinage, &contour);
        calepinage_sur_la_planche(&painter, p, &pose, &|point| echelle.vers_l_ecran(point));
    } else if complete
        && dessin.finis.plancher != noyau::finis::Plancher::Aucun
        && let Some(contour) = dessin.contour_interieur(solution)
    {
        let traits = noyau::finis::trame(dessin.finis.plancher, &contour);
        trame_sur_la_planche(&painter, p, &traits, &|point| echelle.vers_l_ecran(point));
    }
    // Le surligneur du croquis passe sous le plan (évolution 30).
    if complete {
        for trait_ in dessin
            .croquis
            .iter()
            .filter(|trait_| trait_.crayon == noyau::dessin::Crayon::Surligneur)
        {
            trait_a_main_levee(&painter, p, trait_, &echelle);
        }
    }
    // Les objets : leur rectangle, plein, leur symbole et leur nom.
    if complete {
        for (rang, (objet, &dimensions)) in dessin.objets.iter().zip(&solution.objets).enumerate() {
            let choisi = etat.objet_choisi == Some(rang)
                || etat.objet_tire.is_some_and(|(r, _)| r == rang)
                || matches!(etat.autre_choisi, Some(AutreSaisie::Objet(r, _)) if r == rang);
            let en_conflit = conflits.iter().any(|conflit| {
                conflit.objet == rang
                    || conflit.raison == noyau::dessin::RaisonDuConflit::Objet(rang)
            });
            let encre = if en_conflit {
                p.erreur
            } else if choisi {
                p.accent_du_plan()
            } else {
                p.texte_attenue
            };
            let coins: Vec<Pos2> = objet
                .coins(dimensions)
                .iter()
                .map(|coin| echelle.vers_l_ecran(*coin))
                .collect();
            // Un symbole sans contour, ou au-dessus de la coupe, n'a pas de
            // fond, sauf choisi (point 40).
            if objet.genre.contour() == Contour::Plein || choisi {
                painter.add(egui::Shape::convex_polygon(
                    coins.clone(),
                    theme::melanger(p.carte, p.texte, 0.06),
                    Stroke::NONE,
                ));
            }
            let contremarches = solution
                .escaliers
                .get(rang)
                .copied()
                .flatten()
                .map(|calcul| calcul.contremarches);
            for trait_ in objet.traits_sur_le_plan(dimensions, contremarches) {
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
            // Son sigle (DDFT, 240 V…), à la taille du symbole; ou son nom,
            // s'il y a la place (point 40).
            let plus_petit = coins[0].distance(coins[1]).min(coins[1].distance(coins[2]));
            match textes::texte_d_objet_sur_le_plan(objet.genre) {
                Some((sigle, true)) if plus_petit >= 9.0 => {
                    let ecrit = painter.layout_no_wrap(
                        sigle.traduit().to_string(),
                        theme::police((plus_petit * 0.42).clamp(7.0, 10.5)),
                        encre,
                    );
                    let (attache, devant) = attache_du_sigle(objet, dimensions);
                    let attache = echelle.vers_l_ecran(attache);
                    let place = match devant {
                        Some(devant) => {
                            let taille = (f64::from(ecrit.size().x), f64::from(ecrit.size().y));
                            let (x, y) = texte_a_cote(
                                (f64::from(attache.x), f64::from(attache.y)),
                                devant,
                                taille,
                                2.0,
                            );
                            Pos2::new(x as f32, y as f32)
                        }
                        None => attache,
                    };
                    painter.galley(place - ecrit.size() / 2.0, ecrit, encre);
                }
                Some((nom, false)) if plus_petit > 34.0 => {
                    painter.text(
                        echelle.vers_l_ecran(objet.centre),
                        Align2::CENTER_CENTER,
                        nom.traduit(),
                        theme::police(10.5),
                        encre,
                    );
                }
                _ => {}
            }
        }
    }
    // Les cloisons, et celle qu'on trace.
    if complete {
        let ecrire = |pouces: f64| longueur_en_clair(pouces, solution, prefs);
        for (rang, cloison) in solution.cloisons.iter().enumerate() {
            let Some(cloison) = cloison else {
                continue;
            };
            let choisie = etat.cloison_choisie == Some(rang)
                || etat.cloison_tiree.is_some_and(|(r, _)| r == rang)
                || matches!(etat.autre_choisi, Some(AutreSaisie::Cloison(r, _)) if r == rang);
            dessiner_une_cloison(&painter, p, &echelle, cloison, choisie, &ecrire);
        }
        if let Some(trace) = &etat.cloison_en_trace
            && let Some((debut, segments)) = redresser_le_trace(
                trace,
                &appuis_des_cloisons(faces_interieures, solution),
                aimant_des_cloisons(&echelle),
                pas_du_plan(solution),
            )
        {
            let apercu =
                CloisonMesuree::depuis(debut, &segments, solution.epaisseur_des_cloisons, None);
            dessiner_une_cloison(&painter, p, &echelle, &apercu, true, &ecrire);
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
            (5.0, p.accent_du_plan())
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
        painter.circle_stroke(*depart, 8.0, Stroke::new(2.0, p.accent_du_plan()));
        painter.circle_filled(*fin, 5.5, p.accent_du_plan());
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
            let encre = if choisie { p.accent_du_plan() } else { p.texte };
            if choisie {
                // Les deux distances jusqu'aux coins, à l'intérieur de la pièce,
                // et le coin A, d'où se mesure la distance au coin.
                let trait_ = Stroke::new(1.0, p.accent_du_plan());
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
                        p.accent_du_plan(),
                    );
                }
                painter.circle_stroke(*a, 6.5, Stroke::new(1.6, p.accent_du_plan()));
                painter.text(
                    *a + dedans * 14.0 + unite * 12.0,
                    Align2::CENTER_CENTER,
                    "A",
                    theme::police_grasse(12.0),
                    p.accent_du_plan(),
                );
            }
            // Le mur s'interrompt.
            painter.line_segment([q0, q1], Stroke::new(7.0, p.carte));
            dessiner_ouverture(
                &painter,
                (q0, q1),
                dedans,
                (
                    placement.genre,
                    placement.inversee,
                    (placement.haut - placement.bas) as f32 * echelle.facteur,
                    echelle.facteur,
                ),
                encre,
                p.primaire,
                // Un mur vers l'intérieur : une épaisseur négative.
                if dessin.mesures_exterieures {
                    -epaisseur_des_murs_a_l_ecran
                } else {
                    epaisseur_des_murs_a_l_ecran
                },
            );
            // Son repère au bordereau, en dedans, contre la face intérieure :
            // au dehors, il gênerait la longueur du mur (évolution 24).
            if let Some(repere) = etat.reperes_des_ouvertures.get(rang) {
                let au_dedans = if dessin.mesures_exterieures {
                    epaisseur_des_murs_a_l_ecran + 10.0
                } else {
                    10.0
                };
                painter.text(
                    q0 + (q1 - q0) * 0.5 + dedans * au_dedans,
                    Align2::CENTER_CENTER,
                    repere,
                    theme::police_grasse(10.5),
                    p.texte_attenue,
                );
            }
        }
    }

    // Les poignées des murs, à leur milieu : on les glisse pour étirer la pièce
    // (pas une pièce tirée de ses diagonales, qui ne s'étire pas).
    if complete && dessin.fermee && solution.forme_reelle.is_none() {
        for (i, s) in points.windows(2).enumerate() {
            let milieu = s[0] + (s[1] - s[0]) * 0.5;
            let tire = etat.mur_tire.as_ref().is_some_and(|tire| tire.cote == i);
            let encre = if tire { p.accent_du_plan() } else { p.primaire };
            painter.circle_filled(milieu, 5.5, p.carte);
            painter.circle_stroke(milieu, 5.5, Stroke::new(1.8, encre));
        }
    }
    // Les poignées des cloisons, au milieu de chaque segment (point 38).
    if complete && dessin.fermee {
        for (rang, cloison) in solution.cloisons.iter().enumerate() {
            let Some(cloison) = cloison else {
                continue;
            };
            for (k, s) in cloison.sommets.windows(2).enumerate() {
                let milieu =
                    echelle.vers_l_ecran(((s[0].0 + s[1].0) / 2.0, (s[0].1 + s[1].1) / 2.0));
                let tire = etat
                    .cloison_etiree
                    .as_ref()
                    .is_some_and(|tire| tire.rang == rang && tire.segment == k);
                let encre = if tire { p.accent_du_plan() } else { p.primaire };
                painter.circle_filled(milieu, 4.5, p.carte);
                painter.circle_stroke(milieu, 4.5, Stroke::new(1.6, encre));
            }
        }
    }
    // Les mesures, à côté de chaque côté, vers l'extérieur de la forme.
    let centre = if points.is_empty() {
        planche.center()
    } else {
        let somme = points.iter().fold(Vec2::ZERO, |s, q| s + q.to_vec2());
        (somme / points.len() as f32).to_pos2()
    };
    let mut cadres_des_mesures: Vec<Option<Rect>> = vec![None; points.len()];
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
            Mesure::Manquante => ("?".to_string(), p.accent_du_plan()),
            Mesure::Invalide => (cote.saisie.trim().to_string(), p.erreur),
        };
        // Hors du mur, à son épaisseur.
        let mur_dehors = if dessin.mesures_exterieures {
            0.0
        } else {
            epaisseur_des_murs_a_l_ecran
        };
        let position = milieu + dehors * (7.0 + mur_dehors);
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
        cadres_des_mesures[i] = Some(cadre);
        painter.rect_filled(cadre, 4, p.carte.gamma_multiply(0.92));
        painter.galley(cadre.min + Vec2::new(3.0, 1.0), galere, couleur);
    }
    // Les lignes de cote (point 35) : hors des murs, au-delà de leurs mesures;
    // l'écart entre deux cloisons, entre elles.
    if complete && dessin.lignes_de_cote {
        let mur_dehors = if dessin.mesures_exterieures {
            0.0
        } else {
            epaisseur_des_murs_a_l_ecran
        };
        for chaine in dessin.chaines_de_cotes(solution) {
            let longueur = (chaine.fin.0 - chaine.debut.0).hypot(chaine.fin.1 - chaine.debut.1);
            if longueur <= 1e-9 {
                continue;
            }
            let dehors = Vec2::new(chaine.dehors.0 as f32, chaine.dehors.1 as f32);
            let (a0, b0) = (
                echelle.vers_l_ecran(chaine.debut),
                echelle.vers_l_ecran(chaine.fin),
            );
            // Au-delà de la mesure du mur, comme la chaîne de détail d'un plan
            // d'architecte.
            let au_dela_de_la_mesure = chaine
                .mur
                .and_then(|mur| cadres_des_mesures.get(mur).copied().flatten())
                .map_or(0.0, |cadre| {
                    [
                        cadre.left_top(),
                        cadre.right_top(),
                        cadre.left_bottom(),
                        cadre.right_bottom(),
                    ]
                    .iter()
                    .map(|coin| (*coin - a0).dot(dehors))
                    .fold(0.0, f32::max)
                        + 6.0
                });
            let decalage = if dehors == Vec2::ZERO {
                0.0
            } else {
                (mur_dehors + 30.0).max(au_dela_de_la_mesure)
            };
            let (a, b) = (a0 + dehors * decalage, b0 + dehors * decalage);
            let le_long = (b - a).normalized();
            // Le trait de cote à 45°, comme sur un plan d'architecte.
            let oblique =
                Vec2::new(le_long.x - le_long.y, le_long.y + le_long.x).normalized() * 4.5;
            painter.line_segment([a, b], Stroke::new(1.0, p.texte_attenue));
            let sur_la_ligne = |t: f64| a + (b - a) * (t / longueur) as f32;
            for &t in &chaine.points {
                let q = sur_la_ligne(t);
                if decalage > 0.0 {
                    // La ligne d'attache, du mur jusqu'à la ligne de cote.
                    let depuis =
                        a0 + (b0 - a0) * (t / longueur) as f32 + dehors * (mur_dehors + 3.0);
                    painter.line_segment(
                        [depuis, q + dehors * 3.0],
                        Stroke::new(0.7, p.texte_attenue.gamma_multiply(0.7)),
                    );
                }
                painter.line_segment([q - oblique, q + oblique], Stroke::new(1.4, p.texte));
            }
            // Une cote verticale s'écrit le long de sa ligne, de bas en haut.
            let verticale = le_long.y.abs() > 0.9;
            for paire in chaine.points.windows(2) {
                let (q0, q1) = (sur_la_ligne(paire[0]), sur_la_ligne(paire[1]));
                let galere = painter.layout_no_wrap(
                    sans_insecables(&longueur_en_clair(paire[1] - paire[0], solution, prefs)),
                    theme::police(10.0),
                    p.texte,
                );
                let place = q0.distance(q1);
                let taille = galere.size();
                let (vers, ancre) = place_de_la_mesure(q0, q1, centre);
                if verticale && taille.x + 4.0 <= place {
                    let milieu = q0 + (q1 - q0) * 0.5 + vers * (taille.y / 2.0 + 2.0);
                    painter.rect_filled(
                        Rect::from_center_size(milieu, Vec2::new(taille.y, taille.x)).expand(1.0),
                        2,
                        p.carte.gamma_multiply(0.85),
                    );
                    // Tourné d'un quart de tour vers la gauche, autour de son coin.
                    let coin = milieu + Vec2::new(-taille.y / 2.0, taille.x / 2.0);
                    painter.add(egui::Shape::Text(
                        egui::epaint::TextShape::new(coin, galere, p.texte)
                            .with_angle(-std::f32::consts::FRAC_PI_2),
                    ));
                    continue;
                }
                // Sinon à l'horizontale : le long d'une ligne horizontale, il lui
                // faut sa largeur; à côté d'une autre, sa hauteur. Un morceau trop
                // court à l'écran garde ses traits, sans texte.
                let le_long_du_texte = if le_long.x.abs() > le_long.y.abs() {
                    taille.x
                } else {
                    taille.y
                };
                if le_long_du_texte + 4.0 > place {
                    continue;
                }
                let cadre = ancre.anchor_size(q0 + (q1 - q0) * 0.5 + vers * 2.0, taille);
                // Le texte reste dans la planche, même près de son bord.
                let cadre = cadre.translate(Vec2::new(
                    (planche.left() + 2.0 - cadre.left()).max(0.0)
                        - (cadre.right() - planche.right() + 2.0).max(0.0),
                    (planche.top() + 2.0 - cadre.top()).max(0.0)
                        - (cadre.bottom() - planche.bottom() + 2.0).max(0.0),
                ));
                painter.rect_filled(cadre.expand(1.0), 2, p.carte.gamma_multiply(0.85));
                painter.galley(cadre.min, galere, p.texte);
            }
        }
    }
    // Le faîte d'un plafond cathédrale, ou le côté haut d'un plafond en pente
    // (point 34) : en tirets, d'un bout à l'autre de la pièce.
    if complete
        && let Some(au_plafond) = solution
            .plafond
            .filter(|plafond| plafond.forme != FormeDePlafond::Plat)
        && let Some(interieur) = dessin.contour_interieur(solution)
    {
        let n = au_plafond.normale;
        let u = (n.1, -n.0);
        let (debut, fin) = interieur
            .iter()
            .map(|q| u.0 * q.0 + u.1 * q.1)
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), s| {
                (lo.min(s), hi.max(s))
            });
        let sur_la_ligne = |s: f64| {
            (
                n.0 * au_plafond.ligne + u.0 * s,
                n.1 * au_plafond.ligne + u.1 * s,
            )
        };
        let (a, b) = (
            echelle.vers_l_ecran(sur_la_ligne(debut)),
            echelle.vers_l_ecran(sur_la_ligne(fin)),
        );
        // En pente, la ligne se tient un peu en dedans de son mur.
        let dedans = if au_plafond.forme == FormeDePlafond::EnPente {
            (centre - (a + (b - a) * 0.5)).normalized() * 6.0
        } else {
            Vec2::ZERO
        };
        painter.extend(egui::Shape::dashed_line(
            &[a + dedans, b + dedans],
            Stroke::new(2.0, p.accent_du_plan()),
            10.0,
            6.0,
        ));
    }
    // La diagonale qu'on mesure (point 33) : en pointillé, avec le numéro de
    // chaque coin, pour savoir d'où à où la prendre.
    if let Some(AutreSaisie::Diagonale(k)) = etat.autre_choisi
        && dessin.fermee
        && points.len() == 5
        && k < 2
    {
        painter.extend(egui::Shape::dashed_line(
            &[points[k], points[k + 2]],
            Stroke::new(2.0, p.accent_du_plan()),
            8.0,
            5.0,
        ));
        for (rang, coin) in points[..4].iter().enumerate() {
            let position = *coin + (centre - *coin).normalized() * 17.0;
            painter.circle_filled(position, 9.5, p.carte);
            painter.circle_stroke(position, 9.5, Stroke::new(1.8, p.accent_du_plan()));
            painter.text(
                position,
                Align2::CENTER_CENTER,
                (rang + 1).to_string(),
                theme::police_grasse(11.5),
                p.texte,
            );
        }
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
                Stroke::new(1.8, p.accent_du_plan())
            } else {
                Stroke::new(1.0, p.carte_bord)
            };
            painter.rect_stroke(cadre, 4, bord, egui::StrokeKind::Inside);
            painter.galley(cadre.min + Vec2::new(5.0, 3.0), galere, couleur);
            etat.notes_a_l_ecran.push((rang, cadre));
        }
    }
    // L'ossature : les montants dans les murs et les cloisons, et les
    // linteaux au-dessus des ouvertures (évolution 28).
    if complete && dessin.ossature_visible {
        let couleur = theme::melanger(p.carte, p.texte, 0.55);
        for mur in noyau::ossature::ossature(dessin, solution) {
            for rectangle in mur.rectangles_des_montants() {
                let coins: Vec<Pos2> = rectangle.iter().map(|c| echelle.vers_l_ecran(*c)).collect();
                painter.add(egui::Shape::convex_polygon(coins, couleur, Stroke::NONE));
            }
            for (a, b) in mur.traits_des_linteaux() {
                painter.line_segment(
                    [echelle.vers_l_ecran(a), echelle.vers_l_ecran(b)],
                    Stroke::new(2.5, couleur),
                );
            }
        }
    }
    // Le nom et l'aire de chaque espace que ferment les cloisons (évolution 5).
    if complete {
        for (rang, espace) in dessin.espaces(solution).iter().enumerate() {
            let centre = echelle.vers_l_ecran(espace.repere);
            painter.text(
                centre - Vec2::new(0.0, 7.0),
                Align2::CENTER_CENTER,
                nom_d_un_espace(dessin, espace, rang),
                theme::police_grasse(11.0),
                p.texte_attenue,
            );
            painter.text(
                centre + Vec2::new(0.0, 7.0),
                Align2::CENTER_CENTER,
                sans_insecables(&Quantite::Aire(espace.aire).en_clair(solution.modele, prefs)),
                theme::police(10.0),
                p.texte_attenue,
            );
        }
    }
    // Le battant d'une porte qui frappe un meuble : son quart de disque en
    // rouge (évolution 23).
    if !conflits.is_empty() {
        for (raison, polygone) in dessin.battants_des_portes(solution) {
            if conflits.iter().any(|conflit| conflit.raison == raison) {
                let points: Vec<Pos2> = polygone
                    .iter()
                    .map(|point| echelle.vers_l_ecran(*point))
                    .collect();
                painter.add(egui::Shape::convex_polygon(
                    points.clone(),
                    p.erreur.gamma_multiply(0.18),
                    Stroke::NONE,
                ));
                painter.add(egui::Shape::closed_line(points, Stroke::new(1.6, p.erreur)));
            }
        }
    }
    // Les dégagements du meuble qu'on déplace ou qu'on a choisi : ses
    // distances libres jusqu'aux murs, aux cloisons et aux autres meubles
    // (évolution 23).
    if complete
        && let Some(rang) = etat.objet_tire.map(|(rang, _)| rang).or(etat.objet_choisi)
        && let (Some(objet), Some(&dimensions)) =
            (dessin.objets.get(rang), solution.objets.get(rang))
        && objet.genre.encombre()
    {
        let obstacles = obstacles_des_degagements(dessin, solution, rang);
        for degagement in mobilier::degagements(objet, dimensions, &obstacles, 600.0) {
            distance_sur_le_plan(
                &painter,
                p,
                (
                    echelle.vers_l_ecran(degagement.de),
                    echelle.vers_l_ecran(degagement.a),
                ),
                longueur_en_clair(degagement.distance, solution, prefs),
                true,
            );
        }
    }
    // L'outil « Mesurer » : ses points, et leur distance.
    if let Some(points) = &etat.mesure {
        for point in points {
            painter.circle_filled(echelle.vers_l_ecran(*point), 4.5, p.accent_du_plan());
        }
        if let [a, b] = points[..] {
            distance_sur_le_plan(
                &painter,
                p,
                (echelle.vers_l_ecran(a), echelle.vers_l_ecran(b)),
                longueur_en_clair((b.0 - a.0).hypot(b.1 - a.1), solution, prefs),
                false,
            );
        }
    }
    // Les traits du crayon, par-dessus tout, et celui que le doigt trace.
    if complete {
        for trait_ in dessin
            .croquis
            .iter()
            .filter(|trait_| trait_.crayon != noyau::dessin::Crayon::Surligneur)
        {
            trait_a_main_levee(&painter, p, trait_, &echelle);
        }
        if let Some(points) = &etat.trait_en_cours {
            let en_cours = noyau::dessin::TraitAMainLevee {
                points: points.clone(),
                crayon: etat.crayon,
            };
            trait_a_main_levee(&painter, p, &en_cours, &echelle);
        }
    }
    // Le tracé en cours.
    if etat.trace.len() > 1 {
        painter.add(egui::Shape::line(
            etat.trace.clone(),
            Stroke::new(3.0, p.accent_du_plan()),
        ));
    }
    // Une cloison attend son glissement.
    if etat.cloison_a_tracer {
        painter.text(
            planche.left_top() + Vec2::new(12.0, 12.0),
            Align2::LEFT_TOP,
            textes::DESSIN_TRACER_LA_CLOISON.traduit(),
            theme::police_grasse(13.0),
            p.accent_du_plan(),
        );
    }
    // Une note attend qu'on touche le plan.
    if etat.note_a_placer {
        painter.text(
            planche.left_top() + Vec2::new(12.0, 12.0),
            Align2::LEFT_TOP,
            textes::DESSIN_TOUCHER_POUR_LA_NOTE.traduit(),
            theme::police_grasse(13.0),
            p.accent_du_plan(),
        );
    }
    // Un objet attend qu'on touche la pièce.
    if let Some(genre) = etat.objet_a_placer {
        painter.text(
            planche.left_top() + Vec2::new(12.0, 12.0),
            Align2::LEFT_TOP,
            textes::dessin_toucher_pour_placer(textes::objet(genre).traduit()),
            theme::police_grasse(13.0),
            p.accent_du_plan(),
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
            p.accent_du_plan(),
        );
    }
    // L'outil « Mesurer » attend ses deux points.
    if etat.mesure.as_ref().is_some_and(|points| points.len() < 2) {
        painter.text(
            planche.left_top() + Vec2::new(12.0, 12.0),
            Align2::LEFT_TOP,
            textes::DESSIN_TOUCHER_DEUX_POINTS.traduit(),
            theme::police_grasse(13.0),
            p.accent_du_plan(),
        );
    }
    boutons_du_zoom(
        ui,
        p,
        &painter,
        planche,
        centre_de_la_planche,
        &mut etat.cadrage,
    );
    // Le meuble choisi pivote aussi d'ici : au 15°, dans un sens ou dans
    // l'autre, ou d'un quart de tour.
    if complete && let Some(rang) = etat.objet_choisi.filter(|rang| *rang < dessin.objets.len()) {
        let mut droite = planche.right() - 8.0;
        // Sur une planche étroite, ils montent au-dessus de « 3D », « Cotes »
        // et « Mesurer ».
        let fin_des_boutons_de_gauche =
            planche.left() + 8.0 + 52.0 + 6.0 + 64.0 + 6.0 + 84.0 + 6.0 + 68.0 + 6.0;
        let bas = if droite - 3.0 * 52.0 - 2.0 * 6.0 < fin_des_boutons_de_gauche {
            planche.bottom() - 82.0
        } else {
            planche.bottom() - 42.0
        };
        for (degres, texte) in [
            (90.0, textes::DESSIN_PLUS_90),
            (15.0, textes::DESSIN_PLUS_15),
            (-15.0, textes::DESSIN_MOINS_15),
        ] {
            let zone = Rect::from_min_size(Pos2::new(droite - 52.0, bas), Vec2::new(52.0, 34.0));
            droite -= 58.0;
            let nom = format!("{} {}", textes::DESSIN_PIVOTER.traduit(), texte.traduit());
            if bouton_de_planche(ui, &painter, zone, texte.traduit(), &nom, false, p) {
                etat.memoriser(dessin);
                pivoter_sans_chevaucher(dessin, solution, rang, degres);
                modifie = true;
            }
        }
    }
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
    // Les lignes de cote (point 35) : ce bouton les montre ou les cache.
    let bouton_des_cotes = Rect::from_min_size(
        Pos2::new(planche.left() + 66.0, planche.bottom() - 42.0),
        Vec2::new(64.0, 34.0),
    );
    if dessin.fermee
        && matches!(solution.etat, EtatDeLaForme::Complete { .. })
        && bouton_de_planche(
            ui,
            &painter,
            bouton_des_cotes,
            textes::DESSIN_COTES.traduit(),
            textes::DESSIN_MONTRER_LES_COTES.traduit(),
            dessin.lignes_de_cote,
            p,
        )
    {
        etat.memoriser(dessin);
        dessin.lignes_de_cote = !dessin.lignes_de_cote;
        modifie = true;
    }
    // L'outil « Mesurer » (évolution 23) : ouvert, chaque toucher du plan pose
    // un point.
    let bouton_mesurer = Rect::from_min_size(
        Pos2::new(planche.left() + 136.0, planche.bottom() - 42.0),
        Vec2::new(84.0, 34.0),
    );
    if dessin.fermee
        && matches!(solution.etat, EtatDeLaForme::Complete { .. })
        && bouton_de_planche(
            ui,
            &painter,
            bouton_mesurer,
            textes::DESSIN_MESURER.traduit(),
            textes::DESSIN_MESURER_UNE_DISTANCE.traduit(),
            etat.mesure.is_some(),
            p,
        )
    {
        etat.mesure = if etat.mesure.is_some() {
            None
        } else {
            Some(Vec::new())
        };
        etat.crayon_sorti = false;
        etat.trait_en_cours = None;
        etat.objet_a_placer = None;
        etat.note_a_placer = false;
        etat.cloison_a_tracer = false;
        etat.crayon_sorti = false;
        etat.ouverture_a_placer = None;
    }
    // Le crayon du croquis à main levée (évolution 30).
    let bouton_crayon = Rect::from_min_size(
        Pos2::new(planche.left() + 226.0, planche.bottom() - 42.0),
        Vec2::new(68.0, 34.0),
    );
    if dessin.fermee
        && matches!(solution.etat, EtatDeLaForme::Complete { .. })
        && bouton_de_planche(
            ui,
            &painter,
            bouton_crayon,
            textes::DESSIN_CRAYON.traduit(),
            textes::DESSIN_DESSINER_A_MAIN_LEVEE.traduit(),
            etat.crayon_sorti,
            p,
        )
    {
        let sortir = !etat.crayon_sorti;
        etat.mesure = None;
        etat.objet_a_placer = None;
        etat.note_a_placer = false;
        etat.cloison_a_tracer = false;
        etat.ouverture_a_placer = None;
        etat.trait_en_cours = None;
        etat.crayon_sorti = sortir;
    }
    // Sa palette, en haut de la planche : ses couleurs, le surligneur et
    // « Effacer ».
    if etat.crayon_sorti && matches!(solution.etat, EtatDeLaForme::Complete { .. }) {
        for (rang, crayon) in noyau::dessin::Crayon::TOUS.into_iter().enumerate() {
            let zone = Rect::from_min_size(
                Pos2::new(
                    planche.left() + 10.0 + 36.0 * rang as f32,
                    planche.top() + 8.0,
                ),
                Vec2::splat(32.0),
            );
            let nom = textes::nom_du_crayon(crayon).traduit();
            let reponse = ui.interact(zone, egui::Id::unique(("crayon", nom)), Sense::click());
            reponse.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, nom));
            painter.circle_filled(zone.center(), 15.0, p.carte);
            painter.circle_filled(
                zone.center(),
                11.0,
                theme::couleur_du_crayon(crayon, p).to_opaque(),
            );
            if crayon == etat.crayon {
                painter.circle_stroke(zone.center(), 14.5, Stroke::new(2.5, p.accent_du_plan()));
            } else {
                painter.circle_stroke(zone.center(), 11.5, Stroke::new(1.0, p.bouton_bord));
            }
            if reponse.clicked() {
                etat.crayon = crayon;
            }
        }
        // « Effacer » sous les couleurs : il ne gêne pas « Agrandir ».
        let zone = Rect::from_min_size(
            Pos2::new(planche.left() + 10.0, planche.top() + 46.0),
            Vec2::new(72.0, 32.0),
        );
        if !dessin.croquis.is_empty()
            && bouton_de_planche(
                ui,
                &painter,
                zone,
                textes::DESSIN_EFFACER_LE_CROQUIS.traduit(),
                textes::DESSIN_EFFACER_TOUS_LES_TRAITS.traduit(),
                false,
                p,
            )
        {
            etat.memoriser(dessin);
            dessin.croquis.clear();
            modifie = true;
        }
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
    cadrage: &mut Cadrage,
) {
    let cote = 34.0;
    let mut haut = planche.top() + 8.0;
    let mut boutons = vec![
        ("+", textes::DESSIN_ZOOMER, 1.25_f32, cote),
        ("\u{2212}", textes::DESSIN_DEZOOMER, 0.8, cote),
    ];
    if *cadrage != Cadrage::default() {
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
                cadrage.zoomer(centre, centre, facteur);
            } else {
                *cadrage = Cadrage::default();
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
            etat.futur.push(std::mem::replace(dessin, precedent));
            etat.choisir(None);
            modifie = true;
        }
        if ui
            .add_enabled(
                !etat.futur.is_empty(),
                egui::Button::new(textes::DESSIN_RETABLIR.traduit()),
            )
            .clicked()
            && let Some(suivant) = etat.futur.pop()
        {
            etat.historique.push(std::mem::replace(dessin, suivant));
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
    let virage = dessin.virage(rang);
    let Some(cote) = dessin.cotes.get_mut(rang) else {
        return false;
    };
    let mut modifie = false;
    let mut suivant = false;
    let mut angle_choisi = None;
    // Une mesure qu'on vient de choisir — au plan ou dans une liste plus bas —
    // amène sa carte à l'écran, même si la liste a défilé.
    let amener_a_l_ecran = etat.viser_le_champ;
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
        if let Some(virage) = virage {
            ui.add_space(6.0);
            angle_choisi =
                rangee_de_l_angle(ui, p, textes::DESSIN_ANGLE_DU_COTE, virage, etat, prefs);
        }
        if amener_a_l_ecran {
            ui.scroll_to_rect(ui.min_rect(), None);
        }
    });
    if let Some(angle) = angle_choisi {
        let avant = dessin.clone();
        if dessin.orienter(rang, angle) {
            etat.memoriser(&avant);
            etat.avis_d_angle = None;
            modifie = true;
        } else {
            etat.avis_d_angle = Some(textes::DESSIN_ANGLE_REFUSE);
        }
    }
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
    // Un segment de cloison après le premier : son angle par rapport au
    // précédent (section 25.1, point 32).
    let virage_du_segment = match autre {
        AutreSaisie::Cloison(rang, ChampDeCloison::Longueur(segment)) => dessin
            .cloisons
            .get(rang)
            .and_then(|cloison| cloison.virage(segment))
            .map(|virage| (rang, segment, virage)),
        _ => None,
    };
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
        AutreSaisie::HautDuPlafond => {
            let titre = if dessin.plafond.forme == FormeDePlafond::Cathedrale {
                textes::DESSIN_HAUTEUR_AU_FAITE
            } else {
                textes::DESSIN_HAUTEUR_DU_COTE_HAUT
            };
            (
                &mut dessin.plafond.haut,
                titre.traduit().to_string(),
                textes::DESSIN_HAUT_EXEMPLE.traduit().to_string(),
                None,
            )
        }
        AutreSaisie::Retombee(rang, champ) => {
            let Some(retombee) = dessin.retombees.get_mut(rang) else {
                return false;
            };
            let (saisie, nom, ensuite) = match champ {
                ChampDeRetombee::Longueur => (
                    &mut retombee.longueur,
                    textes::DESSIN_LONGUEUR,
                    Some(ChampDeRetombee::Largeur),
                ),
                ChampDeRetombee::Largeur => (
                    &mut retombee.largeur,
                    textes::DESSIN_LARGEUR,
                    Some(ChampDeRetombee::Descente),
                ),
                ChampDeRetombee::Descente => (&mut retombee.hauteur, textes::DESSIN_DESCENTE, None),
            };
            (
                saisie,
                format!(
                    "{} {} : {}",
                    textes::DESSIN_RETOMBEE.traduit(),
                    rang + 1,
                    nom.traduit()
                ),
                textes::DESSIN_EXEMPLE.traduit().to_string(),
                ensuite.map(|champ| AutreSaisie::Retombee(rang, champ)),
            )
        }
        AutreSaisie::Calepinage(en_longueur) => {
            let saisie = if en_longueur {
                &mut dessin.calepinage.longueur
            } else {
                &mut dessin.calepinage.largeur
            };
            (
                saisie,
                textes::dessin_dimension_a_poser(en_longueur)
                    .traduit()
                    .to_string(),
                textes::DESSIN_CALEPINAGE_EXEMPLE.traduit().to_string(),
                (!en_longueur).then_some(AutreSaisie::Calepinage(true)),
            )
        }
        AutreSaisie::Diagonale(k) => {
            let Some(saisie) = dessin.diagonales.get_mut(k) else {
                return false;
            };
            (
                saisie,
                textes::dessin_diagonale(k),
                textes::DESSIN_EXEMPLE.traduit().to_string(),
                (k == 0).then_some(AutreSaisie::Diagonale(1)),
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
        AutreSaisie::Objet(rang, champ) => {
            // Les indices : la dimension de départ, la hauteur des murs, le
            // nombre de contremarches calculé.
            let calcul = dessin.resoudre(FORMAT_PAR_DEFAUT);
            let escalier = calcul.escaliers.get(rang).copied().flatten();
            let des_murs = calcul
                .hauteur
                .unwrap_or(noyau::maquette::HAUTEUR_PAR_DEFAUT);
            let Some(objet) = dessin.objets.get_mut(rang) else {
                return false;
            };
            let nom = textes::objet(objet.genre).traduit();
            let est_un_escalier = objet.genre == GenreDObjet::Escalier;
            let (largeur, profondeur) = objet.dimensions_de_depart();
            let en_clair =
                |pouces: f64| sans_insecables(&Quantite::Longueur(pouces).en_clair(None, prefs));
            let (saisie, dimension, indice, ensuite) = match champ {
                ChampDObjet::Largeur => (
                    &mut objet.largeur,
                    textes::DESSIN_LARGEUR,
                    en_clair(largeur),
                    Some(AutreSaisie::Objet(rang, ChampDObjet::Profondeur)),
                ),
                ChampDObjet::Profondeur => (
                    &mut objet.profondeur,
                    textes::DESSIN_PROFONDEUR,
                    en_clair(profondeur),
                    est_un_escalier.then_some(AutreSaisie::Objet(rang, ChampDObjet::Hauteur)),
                ),
                ChampDObjet::Hauteur => (
                    &mut objet.hauteur,
                    textes::DESSIN_HAUTEUR_A_MONTER,
                    en_clair(des_murs),
                    Some(AutreSaisie::Objet(rang, ChampDObjet::Contremarches)),
                ),
                ChampDObjet::Contremarches => (
                    &mut objet.contremarches,
                    textes::DESSIN_CONTREMARCHES,
                    escalier.map_or(String::new(), |e| e.contremarches.to_string()),
                    None,
                ),
            };
            (
                saisie,
                format!("{nom} : {}", dimension.traduit()),
                indice,
                ensuite,
            )
        }
        AutreSaisie::Cloison(rang, champ) => {
            // L'indice de l'épaisseur : celle que la pièce donne à ses cloisons.
            let defaut = dessin.resoudre(FORMAT_PAR_DEFAUT).epaisseur_des_cloisons;
            let Some(cloison) = dessin.cloisons.get_mut(rang) else {
                return false;
            };
            let plusieurs = cloison.nombre_de_segments() > 1;
            // « Suivant » passe au segment suivant, puis referme le pavé.
            let ensuite = match champ {
                ChampDeCloison::Longueur(segment) if segment + 1 < cloison.nombre_de_segments() => {
                    Some(AutreSaisie::Cloison(
                        rang,
                        ChampDeCloison::Longueur(segment + 1),
                    ))
                }
                _ => None,
            };
            let (saisie, dimension, indice) = match champ {
                ChampDeCloison::Longueur(segment) => {
                    let Some(saisie) = cloison.longueur_du_segment_mut(segment) else {
                        return false;
                    };
                    let dimension = if plusieurs {
                        format!(
                            "{}, {}",
                            textes::dessin_segment(segment + 1),
                            textes::DESSIN_LONGUEUR.traduit()
                        )
                    } else {
                        textes::DESSIN_LONGUEUR.traduit().to_string()
                    };
                    (
                        saisie,
                        dimension,
                        textes::DESSIN_EXEMPLE.traduit().to_string(),
                    )
                }
                ChampDeCloison::Epaisseur => (
                    &mut cloison.epaisseur,
                    textes::DESSIN_EPAISSEUR.traduit().to_string(),
                    sans_insecables(&Quantite::Longueur(defaut).en_clair(None, prefs)),
                ),
                ChampDeCloison::LargeurDOuverture(k) => {
                    let Some(porte) = cloison.ouvertures.get_mut(k) else {
                        return false;
                    };
                    let titre = if porte.passage {
                        textes::DESSIN_LARGEUR_DU_PASSAGE
                    } else {
                        textes::DESSIN_LARGEUR_DE_LA_PORTE
                    };
                    (
                        &mut porte.largeur,
                        titre.traduit().to_string(),
                        sans_insecables(
                            &Quantite::Longueur(LARGEUR_DE_PORTE_PAR_DEFAUT).en_clair(None, prefs),
                        ),
                    )
                }
                ChampDeCloison::PositionDOuverture(k) => {
                    let Some(porte) = cloison.ouvertures.get_mut(k) else {
                        return false;
                    };
                    (
                        &mut porte.position,
                        textes::DESSIN_DISTANCE_DU_BOUT_A.traduit().to_string(),
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
                    dimension
                ),
                indice,
                ensuite,
            )
        }
    };
    let mut modifie = false;
    let mut suivant = false;
    let mut angle_choisi = None;
    // Une mesure qu'on vient de choisir — au plan ou dans une liste plus bas —
    // amène sa carte à l'écran, même si la liste a défilé.
    let amener_a_l_ecran = etat.viser_le_champ;
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
        if let Some((_, _, virage)) = virage_du_segment {
            ui.add_space(6.0);
            angle_choisi =
                rangee_de_l_angle(ui, p, textes::DESSIN_ANGLE_DU_SEGMENT, virage, etat, prefs);
        }
        if amener_a_l_ecran {
            ui.scroll_to_rect(ui.min_rect(), None);
        }
    });
    if let (Some((rang, segment, _)), Some(angle)) = (virage_du_segment, angle_choisi) {
        let avant = dessin.clone();
        if dessin.cloisons[rang].orienter(segment, angle) {
            etat.memoriser(&avant);
            etat.avis_d_angle = None;
            modifie = true;
        } else {
            etat.avis_d_angle = Some(textes::DESSIN_ANGLE_REFUSE);
        }
    }
    if suivant {
        // De la première dimension à la seconde, puis l'éditeur se referme.
        etat.choisir_autre(ensuite);
    }
    modifie
}

/// Les angles proposés pour un côté ou un segment de cloison, par rapport au
/// précédent (section 25.1, point 32).
const ANGLES_PROPOSES: [f64; 5] = [22.5, 30.0, 45.0, 60.0, 90.0];

/// Un angle en degrés, au millième près, sans zéros inutiles : « 22,5° ».
fn degres_en_clair(angle: f64, prefs: &Preferences) -> String {
    let mut texte = noyau::format::nombre_fixe(angle, 3, prefs);
    if texte.contains([',', '.']) {
        texte = texte
            .trim_end_matches('0')
            .trim_end_matches([',', '.'])
            .to_string();
    }
    noyau::format::nombre_et_unite(&texte, "°")
}

/// Lit un angle tapé (« 22,5 », « 22.5° »); `None` s'il n'est pas de plus de
/// 0° et de moins de 180°.
fn lire_un_angle(texte: &str) -> Option<f64> {
    let nettoye = texte.trim().trim_end_matches('°').trim().replace(',', ".");
    let angle: f64 = nettoye.parse().ok()?;
    (angle > 0.0 && angle < 180.0).then_some(angle)
}

/// La rangée de l'angle d'un côté ou d'un segment de cloison par rapport au
/// précédent : les angles courants, et un champ pour en taper un autre.
/// Retourne l'angle choisi, s'il diffère de l'angle actuel.
fn rangee_de_l_angle(
    ui: &mut Ui,
    p: &Palette,
    libelle: noyau::langue::Texte,
    actuel: f64,
    etat: &mut Etat,
    prefs: &Preferences,
) -> Option<f64> {
    let mut choisi = None;
    let titre = ui
        .label(RichText::new(libelle.traduit()).size(13.5))
        .on_hover_text(textes::INFOBULLE_DESSIN_ANGLE.traduit());
    ui.horizontal_wrapped(|ui| {
        for angle in ANGLES_PROPOSES {
            let selectionne = (actuel - angle).abs() < 5e-4;
            let bouton = egui::Button::new(degres_en_clair(angle, prefs)).selected(selectionne);
            if ui
                .add(bouton)
                .on_hover_text(textes::INFOBULLE_DESSIN_ANGLE.traduit())
                .clicked()
                && !selectionne
            {
                choisi = Some(angle);
            }
        }
        let champ = egui::TextEdit::singleline(&mut etat.angle_saisi)
            .desired_width(72.0)
            .hint_text(degres_en_clair(actuel, prefs));
        let reponse = ui.add(champ).labelled_by(titre.id);
        cadre_de_saisie(ui, p, &reponse);
        let entree = reponse.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        if ui
            .button(textes::DESSIN_ANGLE_APPLIQUER.traduit())
            .clicked()
            || entree
        {
            match lire_un_angle(&etat.angle_saisi) {
                Some(angle) => {
                    etat.angle_saisi.clear();
                    etat.avis_d_angle = None;
                    if (angle - actuel).abs() >= 5e-4 {
                        choisi = Some(angle);
                    }
                }
                None => etat.avis_d_angle = Some(textes::DESSIN_ANGLE_INVALIDE),
            }
        }
    });
    if let Some(avis) = etat.avis_d_angle {
        ui.label(RichText::new(avis.traduit()).size(12.5).color(p.erreur));
    }
    choisi
}

/// Au-delà de cet écart (pouces) entre ses deux diagonales et ce que donnent
/// ses côtés, une pièce est à mesurer de nouveau.
const ECART_TOLERE_DES_DIAGONALES: f64 = 0.25;

/// Les diagonales d'une pièce à quatre côtés (section 25.1, point 33) : un
/// bouton pour chacune, qui ouvre le pavé; sur le plan, la diagonale choisie
/// paraît en pointillé, avec le numéro des coins.
fn diagonales(
    ui: &mut Ui,
    p: &Palette,
    dessin: &Dessin,
    etat: &mut Etat,
    solution: &Solution,
    prefs: &Preferences,
) {
    ui.add_space(8.0);
    if !titre_de_section(
        ui,
        p,
        etat,
        Section::Diagonales,
        textes::DESSIN_DIAGONALES.traduit(),
    ) {
        return;
    }
    ui.horizontal_wrapped(|ui| {
        for k in 0..2 {
            ui.label(RichText::new(textes::dessin_diagonale_courte(k)).size(13.5));
            let cible = AutreSaisie::Diagonale(k);
            let choisie = etat.autre_choisi == Some(cible);
            let texte = mesure_du_bouton(&dessin.diagonales[k], "?", prefs);
            if ui
                .add(egui::Button::new(texte).selected(choisie))
                .on_hover_text(textes::dessin_diagonale(k))
                .clicked()
            {
                etat.choisir_autre((!choisie).then_some(cible));
            }
        }
    });
    let quatre_mesures = solution
        .mesures
        .iter()
        .all(|mesure| matches!(mesure, Mesure::Lue(_)));
    let aide = if quatre_mesures {
        textes::DESSIN_DIAGONALES_AIDE
    } else {
        textes::DESSIN_DIAGONALES_ATTENTE
    };
    ui.label(
        RichText::new(aide.traduit())
            .size(12.5)
            .color(p.texte_attenue),
    );
    if let Some(ecart) = solution
        .ecart_des_diagonales
        .filter(|ecart| *ecart > ECART_TOLERE_DES_DIAGONALES)
    {
        ui.label(
            RichText::new(textes::dessin_diagonales_en_desaccord(&longueur_en_clair(
                ecart, solution, prefs,
            )))
            .size(12.5)
            .color(p.erreur),
        );
    }
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
    if !titre_de_section(
        ui,
        p,
        etat,
        Section::Retraits,
        textes::DESSIN_RETRAITS.traduit(),
    ) {
        return false;
    }
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
/// mur à son `epaisseur` (points; négative pour un mur vers l'intérieur), les
/// jambages sont déjà tracés, la fenêtre montre ses deux faces et son vitrage,
/// et la porte s'ouvre de la face intérieure.
fn dessiner_ouverture(
    painter: &egui::Painter,
    (q0, q1): (Pos2, Pos2),
    dedans: Vec2,
    (genre, inversee, course, pouce): (GenreDOuverture, bool, f32, f32),
    encre: Color32,
    mur: Color32,
    epaisseur: f32,
) {
    let largeur = q0.distance(q1);
    if largeur < 1.0 {
        return;
    }
    // Les autres genres (évolution 24) : leur symbole, le même qu'au PDF et
    // au DXF.
    if !matches!(genre, GenreDOuverture::Porte | GenreDOuverture::Fenetre) {
        let vers = |q: Pos2| (f64::from(q.x), f64::from(q.y));
        for trait_ in noyau::dessin::symbole_d_ouverture(
            genre,
            inversee,
            (vers(q0), vers(q1)),
            (f64::from(dedans.x), f64::from(dedans.y)),
            f64::from(epaisseur),
            (f64::from(course), f64::from(pouce)),
        ) {
            let points: Vec<Pos2> = trait_
                .points
                .iter()
                .map(|&(x, y)| Pos2::new(x as f32, y as f32))
                .collect();
            let largeur_du_trait = match trait_.style {
                noyau::dessin::StyleDOuverture::Panneau => 2.2,
                noyau::dessin::StyleDOuverture::Vitrage => 1.6,
                _ => 1.2,
            };
            if trait_.style == noyau::dessin::StyleDOuverture::Tirets {
                painter.extend(egui::Shape::dashed_line(
                    &points,
                    Stroke::new(largeur_du_trait, encre),
                    5.0,
                    3.0,
                ));
            } else {
                painter.add(egui::Shape::line(
                    points,
                    Stroke::new(largeur_du_trait, encre),
                ));
            }
        }
        return;
    }
    // Une porte dont la charnière est au bout opposé : vue depuis lui.
    let (q0, q1) = if inversee && genre == GenreDOuverture::Porte {
        (q1, q0)
    } else {
        (q0, q1)
    };
    let porte = genre == GenreDOuverture::Porte;
    let unite = (q1 - q0) / largeur;
    let a_l_epaisseur = epaisseur.abs() >= 2.0;
    if porte {
        if !a_l_epaisseur {
            for q in [q0, q1] {
                painter.line_segment([q - dedans * 3.5, q + dedans * 3.5], Stroke::new(2.0, mur));
            }
        }
        let q0 = if epaisseur < 0.0 {
            q0 - dedans * epaisseur
        } else {
            q0
        };
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
    if !titre_de_section(ui, p, etat, Section::Murs, textes::DESSIN_MURS.traduit()) {
        return false;
    }
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
    // De quel côté des murs sont les mesures : l'intérieur (les murs vont vers
    // l'extérieur) ou l'extérieur (ils vont vers l'intérieur).
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(textes::DESSIN_MESURES.traduit()).size(13.5));
        for (exterieures, texte) in [
            (false, textes::DESSIN_INTERIEUR),
            (true, textes::DESSIN_EXTERIEUR),
        ] {
            let bouton = egui::Button::new(texte.traduit())
                .selected(dessin.mesures_exterieures == exterieures);
            if ui.add(bouton).clicked() && dessin.mesures_exterieures != exterieures {
                etat.memoriser(dessin);
                dessin.mesures_exterieures = exterieures;
                modifie = true;
            }
        }
    });
    // Le plafond (point 34) : plat, cathédrale ou en pente.
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(textes::DESSIN_PLAFOND.traduit()).size(13.5));
        for (forme, texte) in [
            (FormeDePlafond::Plat, textes::DESSIN_PLAFOND_PLAT),
            (
                FormeDePlafond::Cathedrale,
                textes::DESSIN_PLAFOND_CATHEDRALE,
            ),
            (FormeDePlafond::EnPente, textes::DESSIN_PLAFOND_EN_PENTE),
        ] {
            let bouton = egui::Button::new(texte.traduit()).selected(dessin.plafond.forme == forme);
            if ui.add(bouton).clicked() && dessin.plafond.forme != forme {
                etat.memoriser(dessin);
                dessin.plafond.forme = forme;
                modifie = true;
                // Sa hauteur haute s'inscrit ensuite au pavé.
                if forme != FormeDePlafond::Plat && dessin.plafond.haut.trim().is_empty() {
                    etat.choisir_autre(Some(AutreSaisie::HautDuPlafond));
                }
            }
        }
    });
    if dessin.plafond.forme != FormeDePlafond::Plat {
        let nombre = dessin.cotes.len().max(1);
        let cote = dessin.plafond.cote % nombre;
        ui.horizontal_wrapped(|ui| {
            let (libelle, du_cote) = if dessin.plafond.forme == FormeDePlafond::Cathedrale {
                (
                    textes::DESSIN_HAUTEUR_AU_FAITE,
                    textes::dessin_faite_parallele(cote + 1),
                )
            } else {
                (
                    textes::DESSIN_HAUTEUR_DU_COTE_HAUT,
                    textes::dessin_cote_haut(cote + 1),
                )
            };
            ui.label(RichText::new(libelle.traduit()).size(13.5));
            let cible = AutreSaisie::HautDuPlafond;
            let texte = mesure_du_bouton(&dessin.plafond.haut, "?", prefs);
            if ui
                .add(egui::Button::new(texte).selected(etat.autre_choisi == Some(cible)))
                .clicked()
            {
                etat.choisir_autre(Some(cible));
            }
            if ui
                .add(egui::Button::new(du_cote))
                .on_hover_text(textes::INFOBULLE_COTE_DU_PLAFOND.traduit())
                .clicked()
            {
                etat.memoriser(dessin);
                dessin.plafond.cote = (cote + 1) % nombre;
                modifie = true;
            }
        });
        if solution.plafond_invalide {
            ui.label(
                RichText::new(textes::DESSIN_PLAFOND_INVALIDE.traduit())
                    .size(12.5)
                    .color(p.erreur),
            );
        } else if solution.plafond_incomplet {
            ui.label(
                RichText::new(textes::DESSIN_PLAFOND_INCOMPLET.traduit())
                    .size(12.5)
                    .color(p.texte_attenue),
            );
        }
    }
    // Les retombées de plafond : longueur × largeur × descente.
    let mut retombee_a_enlever = None;
    for rang in 0..dessin.retombees.len() {
        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new(format!(
                    "{} {}.",
                    textes::DESSIN_RETOMBEE.traduit(),
                    rang + 1
                ))
                .font(theme::police_grasse(13.5)),
            );
            for (champ, vide) in [
                (ChampDeRetombee::Longueur, textes::DESSIN_LONGUEUR),
                (ChampDeRetombee::Largeur, textes::DESSIN_LARGEUR),
                (ChampDeRetombee::Descente, textes::DESSIN_DESCENTE),
            ] {
                let retombee = &dessin.retombees[rang];
                let saisie = match champ {
                    ChampDeRetombee::Longueur => &retombee.longueur,
                    ChampDeRetombee::Largeur => &retombee.largeur,
                    ChampDeRetombee::Descente => &retombee.hauteur,
                };
                let cible = AutreSaisie::Retombee(rang, champ);
                let bouton = egui::Button::new(mesure_du_bouton(saisie, vide.traduit(), prefs))
                    .selected(etat.autre_choisi == Some(cible));
                if ui.add(bouton).clicked() {
                    etat.choisir_autre(Some(cible));
                }
                if champ != ChampDeRetombee::Descente {
                    ui.label("×");
                }
            }
            if ui
                .add(egui::Button::new(textes::DESSIN_ENLEVER.traduit()))
                .clicked()
            {
                retombee_a_enlever = Some(rang);
            }
        });
    }
    if let Some(rang) = retombee_a_enlever {
        etat.memoriser(dessin);
        dessin.retombees.remove(rang);
        etat.choisir_autre(None);
        modifie = true;
    }
    if solution.retombees.iter().any(Option::is_none) {
        ui.label(
            RichText::new(textes::DESSIN_RETOMBEE_INCOMPLETE.traduit())
                .size(12.5)
                .color(p.erreur),
        );
    }
    if ui
        .add(egui::Button::new(textes::DESSIN_AJOUTER_RETOMBEE.traduit()))
        .clicked()
    {
        etat.memoriser(dessin);
        dessin.retombees.push(Retombee::default());
        etat.choisir_autre(Some(AutreSaisie::Retombee(
            dessin.retombees.len() - 1,
            ChampDeRetombee::Longueur,
        )));
        modifie = true;
    }
    if solution.hauteur_invalide {
        ui.label(
            RichText::new(textes::DESSIN_HAUTEUR_INVALIDE.traduit())
                .size(12.5)
                .color(p.erreur),
        );
    }
    let mut a_enlever = None;
    let mut nouveau_genre = None;
    let mut nouveau_format = None;
    let mut a_inverser = None;
    for rang in 0..dessin.ouvertures.len() {
        ui.horizontal_wrapped(|ui| {
            // Son numéro, son repère au bordereau et son genre (évolution 24).
            let ouverture = &dessin.ouvertures[rang];
            let genre = ouverture.genre_ou_deduit(pouces_de(&ouverture.hauteur));
            let repere = etat
                .reperes_des_ouvertures
                .get(rang)
                .cloned()
                .unwrap_or_default();
            ui.label(
                RichText::new(format!("{}. {repere}", rang + 1)).font(theme::police_grasse(13.5)),
            );
            let mut choisi = genre;
            egui::ComboBox::from_id_salt(("genre-d-ouverture", rang))
                .selected_text(textes::genre_d_ouverture(genre).traduit())
                .show_ui(ui, |ui| {
                    for autre in GenreDOuverture::TOUS {
                        ui.selectable_value(
                            &mut choisi,
                            autre,
                            textes::genre_d_ouverture(autre).traduit(),
                        );
                    }
                });
            if choisi != genre {
                nouveau_genre = Some((rang, choisi));
            }
            // Son mur, et le bouton pour la placer sur un mur en le touchant.
            let (ou, couleur) = match ouverture.cote {
                Some(cote) => (textes::dessin_sur_le_cote(cote + 1), p.texte_attenue),
                None => (
                    textes::DESSIN_PAS_PLACEE.traduit().to_string(),
                    p.accent_du_plan(),
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
                etat.crayon_sorti = false;
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
            // Ses formats courants, et son sens.
            egui::ComboBox::from_id_salt(("format-d-ouverture", rang))
                .selected_text(textes::DESSIN_FORMAT.traduit())
                .show_ui(ui, |ui| {
                    for &(largeur, hauteur) in genre.formats() {
                        let texte = format!(
                            "{} × {}",
                            longueur_en_clair(largeur, solution, prefs),
                            longueur_en_clair(hauteur, solution, prefs)
                        );
                        if ui.selectable_label(false, texte).clicked() {
                            nouveau_format = Some((rang, (largeur, hauteur)));
                        }
                    }
                });
            if genre.a_un_sens()
                && ui
                    .add(
                        egui::Button::new(textes::DESSIN_INVERSER.traduit())
                            .selected(dessin.ouvertures[rang].inversee),
                    )
                    .clicked()
            {
                a_inverser = Some(rang);
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
    // Un autre genre prend son format de départ; un format, sa largeur et sa
    // hauteur (évolution 24).
    let format_de = |genre: GenreDOuverture| genre.formats()[0];
    let changement = nouveau_genre
        .map(|(rang, genre)| (rang, Some(genre), format_de(genre)))
        .or(nouveau_format.map(|(rang, format)| (rang, None, format)));
    if let Some((rang, genre, (largeur, hauteur))) = changement
        && rang < dessin.ouvertures.len()
    {
        etat.memoriser(dessin);
        let ouverture = &mut dessin.ouvertures[rang];
        if let Some(genre) = genre {
            ouverture.genre = Some(genre);
        }
        let en_clair = |pouces: f64| {
            sans_insecables(&Quantite::Longueur(pouces).en_clair(solution.modele, prefs))
        };
        ouverture.largeur = en_clair(largeur);
        ouverture.hauteur = en_clair(hauteur);
        modifie = true;
    }
    if let Some(rang) = a_inverser
        && rang < dessin.ouvertures.len()
    {
        etat.memoriser(dessin);
        dessin.ouvertures[rang].inversee = !dessin.ouvertures[rang].inversee;
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
                    inversee: false,
                });
                let rang = dessin.ouvertures.len() - 1;
                etat.choisir_autre(Some(AutreSaisie::Ouverture(rang, ChampDOuverture::Largeur)));
                etat.ouverture_a_placer = Some(rang);
                etat.objet_a_placer = None;
                etat.note_a_placer = false;
                etat.cloison_a_tracer = false;
                etat.crayon_sorti = false;
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
    if !titre_de_section(
        ui,
        p,
        etat,
        Section::Objets,
        textes::DESSIN_OBJETS.traduit(),
    ) {
        return false;
    }
    let (mut a_pivoter, mut a_enlever) = (None, None);
    let mut a_changer_de_forme = None;
    let mut a_copier = None;
    let mut a_retourner = None;
    for rang in 0..dessin.objets.len() {
        ui.horizontal_wrapped(|ui| {
            let objet = &dessin.objets[rang];
            let choisi = etat.objet_choisi == Some(rang);
            // Son nom, et son angle s'il pivote.
            let angle = objet.angle_total();
            let nom = if angle.abs() < 1e-9 {
                format!("{}. {}", rang + 1, textes::objet(objet.genre).traduit())
            } else {
                format!(
                    "{}. {} · {}°",
                    rang + 1,
                    textes::objet(objet.genre).traduit(),
                    angle.round()
                )
            };
            let nom = RichText::new(nom).font(theme::police_grasse(13.5));
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
                let champ = if en_profondeur {
                    ChampDObjet::Profondeur
                } else {
                    ChampDObjet::Largeur
                };
                let cible = AutreSaisie::Objet(rang, champ);
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
            if objet.genre == GenreDObjet::Escalier {
                let mut forme = objet.forme;
                egui::ComboBox::from_id_salt(("forme-d-escalier", rang))
                    .selected_text(textes::forme_d_escalier(forme).traduit())
                    .show_ui(ui, |ui| {
                        for autre in FormeDEscalier::TOUTES {
                            ui.selectable_value(
                                &mut forme,
                                autre,
                                textes::forme_d_escalier(autre).traduit(),
                            );
                        }
                    });
                if forme != objet.forme {
                    a_changer_de_forme = Some((rang, forme));
                }
                if let Some(calcul) = solution.escaliers.get(rang).copied().flatten() {
                    let hauteur = if objet.hauteur.trim().is_empty() {
                        longueur_en_clair(calcul.hauteur, solution, prefs)
                    } else {
                        mesure_du_bouton(&objet.hauteur, "", prefs)
                    };
                    let contremarche = longueur_en_clair(calcul.contremarche, solution, prefs);
                    for (champ, texte) in [
                        (
                            ChampDObjet::Hauteur,
                            format!("{} {hauteur}", textes::DESSIN_HAUTEUR_COURT.traduit()),
                        ),
                        (
                            ChampDObjet::Contremarches,
                            textes::contremarches_en_clair(calcul.contremarches, &contremarche),
                        ),
                    ] {
                        let cible = AutreSaisie::Objet(rang, champ);
                        if ui
                            .add(
                                egui::Button::new(texte).selected(etat.autre_choisi == Some(cible)),
                            )
                            .clicked()
                        {
                            etat.choisir_autre(Some(cible));
                            etat.objet_choisi = Some(rang);
                        }
                    }
                    if let Some(giron) = calcul.giron {
                        ui.label(textes::giron_en_clair(&longueur_en_clair(
                            giron, solution, prefs,
                        )));
                    }
                }
            }
            for (degres, texte) in [
                (15.0, textes::DESSIN_PLUS_15),
                (90.0, textes::DESSIN_PLUS_90),
            ] {
                if ui
                    .add(egui::Button::new(texte.traduit()))
                    .on_hover_text(textes::DESSIN_PIVOTER.traduit())
                    .clicked()
                {
                    a_pivoter = Some((rang, degres));
                }
            }
            // Un escalier tournant se retourne : il tourne de l'autre côté.
            if objet.genre == GenreDObjet::Escalier
                && objet.forme != FormeDEscalier::Droit
                && ui
                    .add(egui::Button::new(textes::DESSIN_MIROIR.traduit()))
                    .clicked()
            {
                a_retourner = Some(rang);
            }
            if ui
                .add(egui::Button::new(textes::COPIER.traduit()))
                .clicked()
            {
                a_copier = Some(rang);
            }
            if ui
                .add(egui::Button::new(textes::DESSIN_ENLEVER.traduit()))
                .clicked()
            {
                a_enlever = Some(rang);
            }
        });
    }
    if let Some(rang) = a_copier {
        // La copie se pose à côté, à la place libre la plus proche.
        etat.memoriser(dessin);
        let mut copie = dessin.objets[rang].clone();
        let dimensions = solution
            .objets
            .get(rang)
            .copied()
            .unwrap_or_else(|| copie.dimensions_de_depart());
        // Un objet du mur se copie tout contre lui, le long du mur (une
        // rangée d'armoires); un autre, en biais (point 40).
        copie.centre = if copie.genre.contre_un_mur() {
            copie.vers_le_plan((1.5, 0.5), dimensions)
        } else {
            (copie.centre.0 + 12.0, copie.centre.1 + 12.0)
        };
        dessin.objets.push(copie);
        let nouveau = dessin.objets.len() - 1;
        placer_au_mieux(dessin, solution, nouveau, dimensions);
        etat.objet_choisi = Some(nouveau);
        modifie = true;
    }
    if let Some(rang) = a_retourner {
        // Son rectangle ne change pas : il ne peut rien chevaucher de plus.
        etat.memoriser(dessin);
        let objet = &mut dessin.objets[rang];
        objet.retourne = !objet.retourne;
        etat.objet_choisi = Some(rang);
        modifie = true;
    }
    if let Some((rang, degres)) = a_pivoter {
        etat.memoriser(dessin);
        pivoter_sans_chevaucher(dessin, solution, rang, degres);
        etat.objet_choisi = Some(rang);
        modifie = true;
    }
    if let Some((rang, forme)) = a_changer_de_forme {
        // La nouvelle forme reprend ses propres dimensions de départ, et va à
        // la place libre la plus proche où elle tient (R-98).
        etat.memoriser(dessin);
        let objet = &mut dessin.objets[rang];
        objet.forme = forme;
        objet.largeur.clear();
        objet.profondeur.clear();
        let dimensions = dessin.objets[rang].dimensions_de_depart();
        placer_au_mieux(dessin, solution, rang, dimensions);
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
    // La rangée des familles, comme la barre des métiers de la calculatrice,
    // puis un bouton par objet de la famille choisie : on touche ensuite la
    // pièce pour l'y poser (point 40).
    ui.horizontal_wrapped(|ui| {
        for famille in Famille::TOUTES {
            let genre = if etat.famille == famille {
                super::Pastille::Active
            } else {
                super::Pastille::Ordinaire
            };
            let nom = textes::famille_d_objets(famille).traduit();
            if super::pastille(ui, p, nom, genre).clicked() {
                etat.famille = famille;
            }
        }
    });
    ui.add_space(4.0);
    ui.horizontal_wrapped(|ui| {
        for &genre in etat.famille.genres() {
            let en_attente = etat.objet_a_placer == Some(genre);
            let bouton = egui::Button::new(format!("+ {}", textes::objet(genre).traduit()))
                .selected(en_attente);
            if ui.add(bouton).clicked() {
                etat.objet_a_placer = if en_attente { None } else { Some(genre) };
                etat.ouverture_a_placer = None;
                etat.note_a_placer = false;
                etat.cloison_a_tracer = false;
                etat.crayon_sorti = false;
            }
        }
    });
    modifie
}

/// Les cloisons de la pièce : la longueur de chacun de leurs segments, leur
/// épaisseur et leur porte, et le bouton pour en tracer une. Retourne vrai si
/// le dessin a changé.
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
    if !titre_de_section(
        ui,
        p,
        etat,
        Section::Cloisons,
        textes::DESSIN_CLOISONS.traduit(),
    ) {
        return false;
    }
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
        let mesuree = solution.cloisons.get(rang).and_then(Option::as_ref);
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
            // Un bouton par segment : sa longueur; puis l'épaisseur.
            let mut boutons: Vec<(ChampDeCloison, String)> = (0..cloison.nombre_de_segments())
                .map(|segment| {
                    let texte = cloison.longueur_du_segment(segment).unwrap_or("");
                    (
                        ChampDeCloison::Longueur(segment),
                        mesure_du_bouton(texte, "?", prefs),
                    )
                })
                .collect();
            boutons.push((
                ChampDeCloison::Epaisseur,
                longueur_en_clair(epaisseur, solution, prefs),
            ));
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
            for (passage, texte) in [
                (false, textes::DESSIN_AJOUTER_PORTE),
                (true, textes::DESSIN_AJOUTER_PASSAGE),
            ] {
                if ui.add(egui::Button::new(texte.traduit())).clicked() {
                    geste = Some((rang, GesteSurUneCloison::AjouterUneOuverture(passage)));
                }
            }
            if ui
                .add(egui::Button::new(textes::COPIER.traduit()))
                .clicked()
            {
                geste = Some((rang, GesteSurUneCloison::Copier));
            }
            if ui
                .add(egui::Button::new(textes::DESSIN_ENLEVER.traduit()))
                .clicked()
            {
                geste = Some((rang, GesteSurUneCloison::Enlever));
            }
        });
        // Ses ouvertures : leur segment, leur largeur, leur place depuis le
        // bout A, et le côté d'une porte.
        for (k, porte) in cloison.ouvertures.iter().enumerate() {
            let porte_mesuree = mesuree.and_then(|c| c.ouvertures.get(k)).copied();
            ui.horizontal_wrapped(|ui| {
                ui.add_space(16.0);
                let nom = if porte.passage {
                    textes::DESSIN_PASSAGE
                } else {
                    textes::DESSIN_PORTE
                };
                ui.label(RichText::new(format!("{} {}", nom.traduit(), k + 1)).size(13.5));
                if cloison.nombre_de_segments() > 1 {
                    // Un toucher la fait passer au segment suivant.
                    let segment = porte_mesuree.map_or(porte.segment, |q| q.segment);
                    if ui
                        .add(egui::Button::new(textes::dessin_sur_le_segment(
                            segment + 1,
                        )))
                        .clicked()
                    {
                        geste = Some((rang, GesteSurUneCloison::SegmentSuivant(k)));
                    }
                }
                let largeur =
                    porte_mesuree.map_or(LARGEUR_DE_PORTE_PAR_DEFAUT, |q| q.fin - q.debut);
                let place = match porte_mesuree {
                    Some(q) if !porte.position.trim().is_empty() => {
                        textes::dessin_du_bout_a(&longueur_en_clair(q.debut, solution, prefs))
                    }
                    _ => textes::DESSIN_CENTREE.traduit().to_string(),
                };
                for (champ, texte) in [
                    (
                        ChampDeCloison::LargeurDOuverture(k),
                        longueur_en_clair(largeur, solution, prefs),
                    ),
                    (ChampDeCloison::PositionDOuverture(k), place),
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
                if !porte.passage
                    && ui
                        .add(egui::Button::new(textes::DESSIN_INVERSER.traduit()))
                        .clicked()
                {
                    geste = Some((rang, GesteSurUneCloison::Inverser(k)));
                }
                if ui
                    .add(egui::Button::new(textes::DESSIN_ENLEVER.traduit()))
                    .clicked()
                {
                    geste = Some((rang, GesteSurUneCloison::EnleverLOuverture(k)));
                }
            });
        }
    }
    if let Some((rang, geste)) = geste {
        etat.memoriser(dessin);
        match geste {
            GesteSurUneCloison::Copier => {
                // La copie, décalée d'un pied, à faire glisser à sa place.
                let mut copie = dessin.cloisons[rang].clone();
                copie.debut = (copie.debut.0 + 12.0, copie.debut.1 + 12.0);
                dessin.cloisons.push(copie);
                etat.cloison_choisie = Some(dessin.cloisons.len() - 1);
            }
            GesteSurUneCloison::Enlever => {
                dessin.cloisons.remove(rang);
                etat.cloison_choisie = None;
                etat.cloison_tiree = None;
                etat.choisir_autre(None);
            }
            GesteSurUneCloison::AjouterUneOuverture(passage) => {
                // Centrée sur le premier segment; sa largeur et sa place se
                // règlent ensuite au pavé, qui s'ouvre sur sa largeur.
                // Au milieu du premier segment, ou à la première place libre à
                // côté des autres ouvertures.
                let position = solution
                    .cloisons
                    .get(rang)
                    .and_then(Option::as_ref)
                    .filter(|mesuree| mesuree.ouvertures.iter().any(|o| o.segment == 0))
                    .and_then(|mesuree| mesuree.place_libre(0, LARGEUR_DE_PORTE_PAR_DEFAUT))
                    .map(|debut| {
                        sans_insecables(&Quantite::Longueur(debut).en_clair(solution.modele, prefs))
                    })
                    .unwrap_or_default();
                let cloison = &mut dessin.cloisons[rang];
                cloison.ouvertures.push(PorteDeCloison {
                    passage,
                    position,
                    ..PorteDeCloison::default()
                });
                let k = cloison.ouvertures.len() - 1;
                etat.cloison_choisie = Some(rang);
                etat.choisir_autre(Some(AutreSaisie::Cloison(
                    rang,
                    ChampDeCloison::LargeurDOuverture(k),
                )));
            }
            GesteSurUneCloison::Inverser(k) => {
                if let Some(porte) = dessin.cloisons[rang].ouvertures.get_mut(k) {
                    porte.inversee = !porte.inversee;
                }
                etat.cloison_choisie = Some(rang);
            }
            GesteSurUneCloison::EnleverLOuverture(k) => {
                if k < dessin.cloisons[rang].ouvertures.len() {
                    dessin.cloisons[rang].ouvertures.remove(k);
                }
                etat.choisir_autre(None);
            }
            GesteSurUneCloison::SegmentSuivant(k) => {
                let nombre = dessin.cloisons[rang].nombre_de_segments();
                if let Some(porte) = dessin.cloisons[rang].ouvertures.get_mut(k) {
                    // Sur son nouveau segment, l'ouverture se recentre.
                    porte.segment = (porte.segment + 1) % nombre;
                    porte.position.clear();
                }
                etat.cloison_choisie = Some(rang);
            }
        }
        modifie = true;
    }
    let bouton = egui::Button::new(textes::DESSIN_AJOUTER_UNE_CLOISON.traduit())
        .selected(etat.cloison_a_tracer);
    if ui.add(bouton).clicked() {
        etat.cloison_a_tracer = !etat.cloison_a_tracer;
        etat.crayon_sorti = false;
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
    if !titre_de_section(ui, p, etat, Section::Notes, textes::DESSIN_NOTES.traduit()) {
        return false;
    }
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
        etat.crayon_sorti = false;
    }
    modifie
}

/// Les résultats d'une pièce complète : surface, périmètre, diagonale d'un
/// rectangle, surface nette, puis les murs. `detail` ajoute une ligne par
/// retrait et la hauteur des murs (pour l'export); sans lui, le compte des
/// symboles des métiers suit (à l'écran et au ruban).
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
    // Des mesures prises à l'extérieur : la surface brute, à part.
    if let Some(aire_exterieure) = solution.aire_exterieure {
        lignes.push((
            textes::DESSIN_SURFACE_EXTERIEURE.traduit().to_string(),
            aire_en_clair(aire_exterieure),
        ));
    }
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
    // Une pièce redessinée d'après ses diagonales : l'angle de chaque coin et
    // son écart à l'équerre, à 4 pi du coin (à 1 m en métrique).
    if let Some(forme) = &solution.forme_reelle {
        let metrique = solution.modele.and_then(noyau::unites::Format::systeme)
            == Some(noyau::unites::Systeme::Metrique);
        let (sur, format, sous_le_seuil) = if metrique {
            (1000.0 / 25.4, noyau::unites::Format::Mm, 0.5 / 25.4)
        } else {
            (
                48.0,
                noyau::unites::Format::PoFrac,
                0.5 / f64::from(prefs.resolution.max(1)),
            )
        };
        for (rang, angle) in noyau::dessin::angles_des_coins(forme)
            .into_iter()
            .enumerate()
        {
            let ecart = noyau::dessin::ecart_a_l_equerre(angle, sur);
            let degres =
                noyau::format::nombre_et_unite(&noyau::format::nombre_fixe(angle, 1, prefs), "°");
            let ecart_en_clair = if ecart < sous_le_seuil {
                textes::DESSIN_D_EQUERRE.traduit().to_string()
            } else {
                textes::dessin_ecart_sur(
                    &sans_insecables(&Quantite::Longueur(ecart).en_clair(Some(format), prefs)),
                    metrique,
                )
            };
            lignes.push((
                textes::dessin_coin(rang + 1),
                format!("{degres} · {ecart_en_clair}"),
            ));
        }
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
    // Le plafond et le volume de la pièce (point 34).
    if detail
        && let Some(au_plafond) = solution
            .plafond
            .filter(|plafond| plafond.forme != FormeDePlafond::Plat)
    {
        let forme = if au_plafond.forme == FormeDePlafond::Cathedrale {
            textes::DESSIN_PLAFOND_CATHEDRALE
        } else {
            textes::DESSIN_PLAFOND_EN_PENTE
        };
        lignes.push((
            textes::DESSIN_PLAFOND.traduit().to_string(),
            format!("{} · {}", forme.traduit(), longueur(au_plafond.haut)),
        ));
    }
    if let Some(surface) = solution.surface_du_plafond {
        lignes.push((
            textes::DESSIN_SURFACE_DU_PLAFOND.traduit().to_string(),
            aire_en_clair(surface),
        ));
    }
    if let Some(volume) = solution.volume {
        let metrique =
            solution.modele.and_then(Format::systeme) == Some(noyau::unites::Systeme::Metrique);
        let (pi3, m3) = (
            volume / noyau::unites::PO3_PAR_PI3,
            volume / noyau::metiers::PO3_PAR_M3,
        );
        let (pieds_cubes, metres_cubes) =
            (noyau::langue::t("pi³", "ft³"), noyau::langue::tel("m³"));
        let quantite = if metrique {
            Quantite::Double(m3, metres_cubes, pi3, pieds_cubes)
        } else {
            Quantite::Double(pi3, pieds_cubes, m3, metres_cubes)
        };
        lignes.push((
            textes::DESSIN_VOLUME.traduit().to_string(),
            quantite.en_clair(solution.modele, prefs),
        ));
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
    // Le compte de l'ossature, si elle paraît (évolution 28).
    if dessin.ossature_visible {
        lignes.extend(lignes_de_l_ossature(dessin, solution, prefs));
    }
    // Le calepinage, s'il paraît (évolution 19).
    if dessin.calepinage.visible {
        lignes.extend(lignes_du_calepinage(
            dessin,
            &dessin.calepinage,
            solution,
            prefs,
        ));
    }
    // L'aire de chaque espace que ferment les cloisons (évolution 5).
    for (rang, espace) in dessin.espaces(solution).iter().enumerate() {
        lignes.push((
            nom_d_un_espace(dessin, espace, rang),
            aire_en_clair(espace.aire),
        ));
    }
    // Les finis de la pièce et leurs quantités, à l'écran et au ruban (le PDF
    // a son tableau des finis) (évolution 27).
    if !detail {
        let finis = &dessin.finis;
        let quantites = noyau::finis::quantites(solution);
        let avec = |nom: String, quantite: Option<String>| match quantite {
            Some(quantite) => format!("{nom} · {quantite}"),
            None => nom,
        };
        if finis.plancher != noyau::finis::Plancher::Aucun {
            lignes.push((
                textes::DESSIN_FINI_PLANCHER.traduit().to_string(),
                avec(
                    textes::nom_du_plancher(finis.plancher)
                        .traduit()
                        .to_string(),
                    quantites.plancher.map(aire_en_clair),
                ),
            ));
        }
        if finis.peinture_choisie() {
            lignes.push((
                textes::DESSIN_FINI_PEINTURE.traduit().to_string(),
                avec(
                    textes::peinture_en_clair(finis.peinture, &finis.couleur),
                    quantites.murs.map(aire_en_clair),
                ),
            ));
        }
        if finis.plafond != noyau::finis::FiniDePlafond::Aucun {
            lignes.push((
                textes::DESSIN_FINI_PLAFOND.traduit().to_string(),
                avec(
                    textes::nom_du_plafond(finis.plafond).traduit().to_string(),
                    quantites.plafond.map(aire_en_clair),
                ),
            ));
        }
        if finis.plinthes != noyau::finis::Plinthe::Aucune {
            lignes.push((
                textes::DESSIN_FINI_PLINTHES.traduit().to_string(),
                avec(
                    textes::nom_de_la_plinthe(finis.plinthes)
                        .traduit()
                        .to_string(),
                    quantites.plinthes.map(longueur),
                ),
            ));
        }
    }
    // À l'écran (et au ruban), les conflits, puis le compte des appareils et
    // des symboles des métiers : combien de prises, de coudes, d'armoires. Le
    // PDF met ce compte dans sa légende (point 40).
    if !detail {
        let nom = |rang: usize| {
            dessin
                .objets
                .get(rang)
                .map_or("", |objet| textes::objet(objet.genre).traduit())
        };
        for conflit in dessin.conflits(solution) {
            let autre = match conflit.raison {
                noyau::dessin::RaisonDuConflit::Objet(autre) => autre,
                _ => 0,
            };
            lignes.push((
                textes::DESSIN_A_VERIFIER.traduit().to_string(),
                textes::dessin_conflit(
                    conflit.raison,
                    (conflit.objet, nom(conflit.objet)),
                    (autre, nom(autre)),
                ),
            ));
        }
        for (genre, nombre) in comptes_des_symboles(dessin) {
            lignes.push((
                textes::objet(genre).traduit().to_string(),
                nombre.to_string(),
            ));
        }
    }
    lignes
}

/// Combien la pièce compte de chaque appareil et de chaque symbole des
/// métiers (tout sauf le mobilier), dans l'ordre des genres (point 40).
pub(crate) fn comptes_des_symboles(dessin: &Dessin) -> Vec<(GenreDObjet, usize)> {
    GenreDObjet::TOUS
        .into_iter()
        .filter(|genre| genre.famille() != Famille::Mobilier)
        .map(|genre| {
            let nombre = dessin
                .objets
                .iter()
                .filter(|objet| objet.genre == genre)
                .count();
            (genre, nombre)
        })
        .filter(|&(_, nombre)| nombre > 0)
        .collect()
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
        EtatDeLaForme::DiagonaleImpossible(diagonale) => consigne(
            ui,
            &textes::dessin_diagonale_impossible(diagonale),
            p.erreur,
        ),
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
                    // Une ligne trop longue passe à la ligne; un conflit
                    // s'écrit plus petit, en rouge (évolution 23).
                    let conflit = libelle == textes::DESSIN_A_VERIFIER.traduit();
                    ui.horizontal_wrapped(|ui| {
                        ui.label(
                            RichText::new(libelle)
                                .font(theme::police_grasse(13.5))
                                .color(p.titre),
                        );
                        let (taille, couleur) = if conflit {
                            (13.5, p.erreur)
                        } else {
                            (17.0, p.texte)
                        };
                        ui.label(
                            RichText::new(valeur)
                                .font(theme::police_grasse(taille))
                                .color(couleur),
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
                            // La hauteur, si l'outil en a une (point 34) : pour
                            // la ventilation, celle qui redonne le volume de la
                            // pièce; pour les autres, la hauteur moyenne des murs.
                            // Sous un plafond plat, c'est la hauteur des murs.
                            if let Some(rang) = rang_hauteur {
                                let hauteur = if c.metier == noyau::metiers::Metier::Ventilation {
                                    solution.volume.map(|volume| volume / aire_utile)
                                } else {
                                    solution.murs().map(|murs| murs.brute / perimetre)
                                }
                                .or(solution.hauteur);
                                if let Some(hauteur) = hauteur {
                                    champs.push((rang, a_taper(hauteur)));
                                }
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
    fn toucher_une_cloison_choisit_le_segment_le_plus_proche() {
        // Une cloison en L : 72 po vers l'est, puis 48 po vers le sud.
        let cloison = CloisonMesuree::depuis(
            (0.0, 72.0),
            &[
                (noyau::dessin::Sens::Est, 72.0),
                (noyau::dessin::Sens::Sud, 48.0),
            ],
            5.0,
            None,
        );
        assert_eq!(segment_le_plus_proche(&cloison, (37.0, 71.7)), 0);
        assert_eq!(segment_le_plus_proche(&cloison, (72.5, 100.0)), 1);
        // Près du coin, le segment le plus proche l'emporte.
        assert_eq!(segment_le_plus_proche(&cloison, (60.0, 74.0)), 0);
        assert_eq!(segment_le_plus_proche(&cloison, (74.0, 90.0)), 1);
    }

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

    #[test]
    fn un_angle_tape_et_un_angle_ecrit() {
        assert_eq!(lire_un_angle("22,5"), Some(22.5));
        assert_eq!(lire_un_angle(" 37.5° "), Some(37.5));
        assert_eq!(lire_un_angle("90"), Some(90.0));
        assert_eq!(lire_un_angle("0"), None);
        assert_eq!(lire_un_angle("180"), None);
        assert_eq!(lire_un_angle("-30"), None);
        assert_eq!(lire_un_angle("abc"), None);
        let prefs = Preferences::default();
        assert_eq!(degres_en_clair(22.5, &prefs), "22,5°");
        assert_eq!(degres_en_clair(90.0, &prefs), "90°");
        assert_eq!(degres_en_clair(100.0, &prefs), "100°");
        assert_eq!(degres_en_clair(89.625, &prefs), "89,625°");
        let _anglais = noyau::langue::pour_la_portee(noyau::langue::Langue::Anglais);
        let anglaises = Preferences {
            langue: noyau::langue::Langue::Anglais,
            ..Preferences::default()
        };
        assert_eq!(degres_en_clair(22.5, &anglaises), "22.5°");
    }

    #[test]
    fn une_porte_d_avant_devient_une_ouverture() {
        // Un dessin d'avant la version 0.13.22 : sa porte unique se lit, et
        // passe en tête des ouvertures.
        let ancienne: noyau::cloison::Cloison = serde_json::from_str(
            r#"{"debut":[0.0,0.0],"sens":"Est","longueur":"10","porte":{"largeur":"30","position":"","inversee":true,"segment":0}}"#,
        )
        .unwrap();
        assert_eq!(ancienne.toutes_les_ouvertures().count(), 1);
        let mut cloison = ancienne.clone();
        cloison.ouvertures.push(noyau::cloison::PorteDeCloison {
            passage: true,
            ..noyau::cloison::PorteDeCloison::default()
        });
        cloison.normaliser();
        assert!(cloison.porte.is_none());
        assert_eq!(cloison.ouvertures.len(), 2);
        assert!(cloison.ouvertures[0].inversee && !cloison.ouvertures[0].passage);
        assert!(cloison.ouvertures[1].passage);
        // Normalisée, elle ne s'écrit plus avec « porte ».
        let texte = serde_json::to_string(&cloison).unwrap();
        assert!(!texte.contains("\"porte\""), "{texte}");
    }

    #[test]
    fn un_dessin_aux_angles_libres_s_enregistre_et_se_relit() {
        // Un dessin d'avant les angles libres se lit tel quel; un côté libre
        // s'écrit avec son angle et se relit.
        let ancien = r#"{"cotes":[{"sens":"Est","croquis":100.0,"saisie":"12'"},{"sens":"SudEst","croquis":50.0}],"fermee":false}"#;
        let dessin: Dessin = serde_json::from_str(ancien).expect("ancien dessin");
        assert_eq!(dessin.cotes[1].sens, noyau::dessin::Sens::SudEst);
        let mut libre = dessin.clone();
        libre.cotes[1].sens = noyau::dessin::Sens::Libre(30_000);
        let texte = serde_json::to_string(&libre).expect("écrit");
        assert!(texte.contains(r#""sens":{"Libre":30000}"#), "{texte}");
        let relu: Dessin = serde_json::from_str(&texte).expect("relu");
        assert_eq!(relu, libre);
    }
}
