//! Écran « Notes » : le bloc-notes (section 26). Une liste de notes, chacune
//! avec son titre, son texte, sa date et ses pièces jointes; une note s'ouvre
//! pour s'écrire, et une photo jointe s'y voit en grand.

use std::collections::HashMap;

use eframe::egui::{self, Color32, Pos2, Rect, RichText, Sense, Stroke, Ui, Vec2};
use noyau::metiers::Quantite;
use noyau::prefs::Preferences;

use super::{bouton_principal, cadre_de_saisie, carte, corps, en_tete};
use crate::pieces::{self, LectureDeCote};
use crate::plateforme;
use crate::stockage::{CoteSurPhoto, NoteDuBloc, PieceJointe};
use crate::textes;
use crate::theme::{self, Palette};

/// Ce que l'utilisateur a demandé dans l'écran des notes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Nouvelle,
    Ouvrir(u64),
    /// Revenir à la liste.
    Fermer,
    /// Le titre ou le texte de la note ouverte a changé.
    Modifiee,
    Copier(u64),
    Exporter(u64),
    Supprimer(u64),
    /// Joindre des photos (vrai) ou des fichiers (faux) à la note ouverte.
    Joindre(bool),
    /// Voir une photo jointe en grand.
    VoirPhoto(u64),
    /// Revenir de la photo à la note.
    FermerPhoto,
    /// Ouvrir un fichier joint dans un programme du poste (bureau).
    OuvrirPiece(u64),
    EnregistrerPiece(u64),
    RetirerPiece(u64),
    /// Exporter une photo et ses cotes en PDF.
    PdfDeLaPhoto(u64),
}

/// Ce que la vue en grand d'une photo retient pendant qu'on la cote (section
/// 26, mesures sur une photo).
#[derive(Clone, Debug, Default)]
pub struct EtatDeLaPhoto {
    /// « + Cote » est enfoncé : le prochain glissé trace une cote.
    pub tracer: bool,
    /// Le départ de la cote en train d'être tracée (en part de la photo).
    pub depart: Option<(f64, f64)>,
    /// La cote choisie, dont la mesure s'inscrit sous la photo.
    pub choisie: Option<usize>,
    /// Le bout de la cote choisie qu'on fait glisser : (rang, bout A).
    pub poignee: Option<(usize, bool)>,
}

/// Le texte d'une cote, sur la photo comme dans le PDF : « Réf. 32 po » pour la
/// référence, « ~ 2 pi 0 po » pour une mesure estimée, « ? » sans mesure.
pub fn texte_d_une_cote(
    cote: &CoteSurPhoto,
    lecture: LectureDeCote,
    prefs: &Preferences,
) -> String {
    let en_clair = |pouces: f64, format| Quantite::Longueur(pouces).en_clair(Some(format), prefs);
    let texte = match lecture {
        LectureDeCote::Inscrite(pouces, format) => en_clair(pouces, format),
        LectureDeCote::Estimee(pouces, format) => format!("~ {}", en_clair(pouces, format)),
        LectureDeCote::Illisible => format!("{} ?", cote.mesure.trim()),
        LectureDeCote::Inconnue => "?".to_string(),
    };
    if cote.reference {
        format!("{} {texte}", textes::PREFIXE_REFERENCE.traduit())
    } else {
        texte
    }
}

/// Ce que l'écran montre des pièces jointes : les vignettes des photos de la
/// note ouverte (rien si la vignette est introuvable), et la photo vue en
/// grand, avec sa texture une fois lue; et les préférences, pour écrire les
/// mesures de ses cotes.
pub struct Images<'a> {
    pub vignettes: &'a HashMap<u64, Option<egui::TextureHandle>>,
    pub photo: Option<(u64, Option<&'a egui::TextureHandle>)>,
    pub prefs: &'a Preferences,
}

/// Côté d'une vignette à l'écran, en points.
const COTE_VIGNETTE: f32 = 104.0;

/// Le titre d'une note, ou « Note sans titre ».
pub fn titre_de(note: &NoteDuBloc) -> String {
    let titre = note.titre.trim();
    if titre.is_empty() {
        textes::NOTE_SANS_TITRE.traduit().to_string()
    } else {
        titre.to_string()
    }
}

/// Affiche l'écran des notes : la photo vue en grand, la note ouverte, ou la
/// liste. `viser` donne le clavier au texte de la note ouverte (une note neuve).
pub fn afficher(
    ui: &mut Ui,
    p: &Palette,
    notes: &mut [NoteDuBloc],
    ouverte: Option<u64>,
    viser: &mut bool,
    images: &Images<'_>,
    etat_de_la_photo: &mut EtatDeLaPhoto,
) -> Option<Action> {
    let mut action = None;

    // Une note ouverte : son titre et son texte s'écrivent sur place.
    if let Some(note) = ouverte.and_then(|id| notes.iter_mut().find(|n| n.id == id)) {
        if let Some((id_photo, texture)) = images.photo
            && let Some(piece) = note.pieces.iter_mut().find(|piece| piece.id == id_photo)
        {
            return afficher_la_photo(ui, p, piece, texture, etat_de_la_photo, images.prefs);
        }
        let id = note.id;
        if en_tete(ui, p, &titre_de(note), textes::RETOUR.traduit()) {
            action = Some(Action::Fermer);
        }
        corps(ui, "note-ouverte", |ui| {
            ui.label(
                RichText::new(note.date_en_clair())
                    .size(12.5)
                    .color(p.texte_attenue),
            );
            ui.add_space(4.0);
            let titre = egui::TextEdit::singleline(&mut note.titre)
                .desired_width(f32::INFINITY)
                .char_limit(80)
                .font(theme::police_grasse(16.0))
                .hint_text(textes::NOTE_TITRE_INDICE.traduit());
            let reponse = ui.add(titre);
            cadre_de_saisie(ui, p, &reponse);
            if reponse.changed() {
                action = Some(Action::Modifiee);
            }
            ui.add_space(6.0);
            let texte = egui::TextEdit::multiline(&mut note.texte)
                .desired_width(f32::INFINITY)
                .desired_rows(14)
                .char_limit(20_000)
                .font(theme::police(15.0))
                .hint_text(textes::NOTE_TEXTE_INDICE.traduit());
            let reponse = ui.add(texte);
            cadre_de_saisie(ui, p, &reponse);
            if std::mem::take(viser) {
                reponse.request_focus();
            }
            if reponse.changed() {
                action = Some(Action::Modifiee);
            }
            ui.add_space(10.0);
            if let Some(a) = afficher_les_pieces(ui, p, &note.pieces, images.vignettes) {
                action = Some(a);
            }
            ui.add_space(10.0);
            ui.horizontal_wrapped(|ui| {
                for (libelle, a) in [
                    (textes::COPIER.traduit(), Action::Copier(id)),
                    (textes::EXPORTER.traduit(), Action::Exporter(id)),
                    (textes::SUPPRIMER.traduit(), Action::Supprimer(id)),
                ] {
                    if ui.button(libelle).clicked() {
                        action = Some(a);
                    }
                }
            });
            ui.add_space(12.0);
        });
        return action;
    }

    // La liste des notes, de la plus récente à la plus ancienne.
    ui.add_space(10.0);
    ui.horizontal(|ui| {
        ui.add_space(14.0);
        ui.label(
            RichText::new(textes::ONGLET_NOTES.traduit())
                .font(theme::police_grasse(20.0))
                .color(p.entete),
        );
    });
    ui.add_space(4.0);
    ui.separator();
    corps(ui, "notes", |ui| {
        ui.add_space(4.0);
        if ui
            .add(bouton_principal(p, textes::NOUVELLE_NOTE.traduit(), 14.0))
            .clicked()
        {
            action = Some(Action::Nouvelle);
        }
        ui.add_space(8.0);
        if notes.is_empty() {
            ui.label(RichText::new(textes::AUCUNE_NOTE.traduit()).color(p.texte_attenue));
        }
        for note in notes.iter() {
            carte(ui, p, |ui| {
                ui.label(RichText::new(titre_de(note)).font(theme::police_grasse(15.0)));
                let mut date = note.date_en_clair();
                if !note.pieces.is_empty() {
                    date.push_str(" · ");
                    date.push_str(&textes::nombre_de_pieces(note.pieces.len()));
                }
                ui.label(RichText::new(date).size(12.5).color(p.texte_attenue));
                // Le début du texte.
                let apercu: String = note
                    .texte
                    .lines()
                    .find(|ligne| !ligne.trim().is_empty())
                    .unwrap_or("")
                    .chars()
                    .take(90)
                    .collect();
                if !apercu.is_empty() {
                    ui.label(RichText::new(apercu).size(13.5));
                }
                ui.horizontal_wrapped(|ui| {
                    for (libelle, a) in [
                        (textes::OUVRIR.traduit(), Action::Ouvrir(note.id)),
                        (textes::COPIER.traduit(), Action::Copier(note.id)),
                        (textes::EXPORTER.traduit(), Action::Exporter(note.id)),
                        (textes::SUPPRIMER.traduit(), Action::Supprimer(note.id)),
                    ] {
                        if ui.small_button(libelle).clicked() {
                            action = Some(a);
                        }
                    }
                });
            });
            ui.add_space(2.0);
        }
        ui.add_space(12.0);
    });
    action
}

/// Les pièces jointes de la note ouverte : les boutons pour en ajouter, les
/// photos en vignettes (toucher une vignette la montre en grand), puis les
/// autres fichiers avec leur taille et leurs boutons.
fn afficher_les_pieces(
    ui: &mut Ui,
    p: &Palette,
    pieces: &[PieceJointe],
    vignettes: &HashMap<u64, Option<egui::TextureHandle>>,
) -> Option<Action> {
    let mut action = None;
    ui.label(RichText::new(textes::PIECES_JOINTES.traduit()).font(theme::police_grasse(14.0)));
    ui.add_space(2.0);
    ui.horizontal_wrapped(|ui| {
        if ui.button(textes::AJOUTER_PHOTO.traduit()).clicked() {
            action = Some(Action::Joindre(true));
        }
        if ui.button(textes::AJOUTER_FICHIER.traduit()).clicked() {
            action = Some(Action::Joindre(false));
        }
    });
    if pieces.iter().any(|piece| piece.photo) {
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            for piece in pieces.iter().filter(|piece| piece.photo) {
                let reponse = match vignettes.get(&piece.id) {
                    Some(Some(texture)) => ui.add(
                        egui::Image::from_texture(egui::load::SizedTexture::from_handle(texture))
                            .max_size(Vec2::splat(COTE_VIGNETTE))
                            .corner_radius(4.0)
                            .sense(egui::Sense::click()),
                    ),
                    // Vignette introuvable : le nom de la photo en tient lieu.
                    Some(None) => case_de_vignette(ui, p, &piece.nom),
                    None => case_de_vignette(ui, p, textes::CHARGEMENT.traduit()),
                };
                if reponse.on_hover_text(&piece.nom).clicked() {
                    action = Some(Action::VoirPhoto(piece.id));
                }
            }
        });
    }
    for piece in pieces.iter().filter(|piece| !piece.photo) {
        ui.add_space(4.0);
        carte(ui, p, |ui| {
            ui.label(RichText::new(&piece.nom).font(theme::police_grasse(14.0)));
            ui.label(
                RichText::new(textes::taille_en_clair(piece.taille))
                    .size(12.5)
                    .color(p.texte_attenue),
            );
            ui.horizontal_wrapped(|ui| {
                // Dans un navigateur, un fichier s'enregistre au lieu de s'ouvrir.
                if !plateforme::NAVIGATEUR && ui.small_button(textes::OUVRIR.traduit()).clicked() {
                    action = Some(Action::OuvrirPiece(piece.id));
                }
                if ui.small_button(textes::ENREGISTRER.traduit()).clicked() {
                    action = Some(Action::EnregistrerPiece(piece.id));
                }
                if ui.small_button(textes::RETIRER.traduit()).clicked() {
                    action = Some(Action::RetirerPiece(piece.id));
                }
            });
        });
    }
    action
}

/// Une case de la taille d'une vignette, avec un court texte au centre.
fn case_de_vignette(ui: &mut Ui, p: &Palette, texte: &str) -> egui::Response {
    let (rect, reponse) = ui.allocate_exact_size(Vec2::splat(COTE_VIGNETTE), egui::Sense::click());
    ui.painter().rect_filled(rect, 4.0, p.champ);
    let court: String = if texte.chars().count() > 14 {
        texte.chars().take(13).chain(['…']).collect()
    } else {
        texte.to_string()
    };
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        court,
        theme::police(11.5),
        p.texte_attenue,
    );
    reponse
}

/// Distance, en points, à laquelle on saisit une cote ou l'un de ses bouts.
const PORTEE_D_UNE_COTE: f32 = 18.0;
/// L'orange des cotes, bien visible sur une photo, et son liseré sombre.
const ORANGE_DES_COTES: Color32 = Color32::from_rgb(255, 140, 0);
const LISERE_DES_COTES: Color32 = Color32::from_rgb(30, 30, 30);

/// Distance d'un point à un segment, à l'écran.
fn distance_au_segment(point: Pos2, a: Pos2, b: Pos2) -> f32 {
    let ab = b - a;
    let longueur = ab.length_sq();
    if longueur <= f32::EPSILON {
        return point.distance(a);
    }
    let t = ((point - a).dot(ab) / longueur).clamp(0.0, 1.0);
    point.distance(a + ab * t)
}

/// Une photo jointe, vue en grand dans la place qui reste, avec ses boutons et
/// ses cotes : « + Cote », puis un glissé d'un bout à l'autre de ce qu'on a
/// mesuré; la mesure s'inscrit sous la photo (R-100).
fn afficher_la_photo(
    ui: &mut Ui,
    p: &Palette,
    piece: &mut PieceJointe,
    texture: Option<&egui::TextureHandle>,
    etat: &mut EtatDeLaPhoto,
    prefs: &Preferences,
) -> Option<Action> {
    let mut action = None;
    if etat.choisie.is_some_and(|rang| rang >= piece.cotes.len()) {
        etat.choisie = None;
    }
    if en_tete(ui, p, &piece.nom, textes::RETOUR.traduit()) {
        action = Some(Action::FermerPhoto);
    }
    corps(ui, "photo-jointe", |ui| {
        ui.horizontal_wrapped(|ui| {
            if ui.button(textes::ENREGISTRER.traduit()).clicked() {
                action = Some(Action::EnregistrerPiece(piece.id));
            }
            if ui.button(textes::DESSIN_EXPORTER_PDF.traduit()).clicked() {
                action = Some(Action::PdfDeLaPhoto(piece.id));
            }
            if ui.button(textes::RETIRER.traduit()).clicked() {
                action = Some(Action::RetirerPiece(piece.id));
            }
            ui.label(
                RichText::new(textes::taille_en_clair(piece.taille))
                    .size(12.5)
                    .color(p.texte_attenue),
            );
        });
        ui.horizontal_wrapped(|ui| {
            if ui
                .selectable_label(etat.tracer, textes::AJOUTER_COTE.traduit())
                .clicked()
            {
                etat.tracer = !etat.tracer;
                etat.depart = None;
            }
        });
        ui.add_space(6.0);
        // La photo garde de la place pour la mesure de la cote choisie.
        let reserve = if etat.choisie.is_some() { 150.0 } else { 90.0 };
        let place = Vec2::new(
            ui.available_width(),
            (ui.available_height() - reserve).max(160.0),
        );
        let Some(texture) = texture else {
            ui.add_space(place.y / 3.0);
            ui.vertical_centered(|ui| {
                ui.label(RichText::new(textes::CHARGEMENT.traduit()).color(p.texte_attenue));
            });
            return;
        };
        let taille = texture.size_vec2();
        let echelle = (place.x / taille.x).min(place.y / taille.y).max(0.01);
        let reponse = ui.add(
            egui::Image::from_texture(egui::load::SizedTexture::from_handle(texture))
                .fit_to_exact_size(taille * echelle)
                .sense(Sense::click_and_drag()),
        );
        let cadre = reponse.rect;
        let vers_l_ecran = |(u, v): (f64, f64)| {
            Pos2::new(
                cadre.left() + u as f32 * cadre.width(),
                cadre.top() + v as f32 * cadre.height(),
            )
        };
        let vers_la_photo = |point: Pos2| {
            (
                f64::from(((point.x - cadre.left()) / cadre.width()).clamp(0.0, 1.0)),
                f64::from(((point.y - cadre.top()) / cadre.height()).clamp(0.0, 1.0)),
            )
        };
        if gestes_sur_la_photo(ui, &reponse, piece, etat, vers_l_ecran, vers_la_photo) {
            action = Some(Action::Modifiee);
        }
        // Les cotes, par-dessus la photo : les traits, puis leurs textes, pour
        // qu'aucun trait ne cache un texte.
        let peintre = ui.painter_at(cadre);
        for (rang, cote) in piece.cotes.iter().enumerate() {
            let (a, b) = (vers_l_ecran(cote.a), vers_l_ecran(cote.b));
            peindre_le_trait_d_une_cote(&peintre, a, b);
            if etat.choisie == Some(rang) {
                for bout in [a, b] {
                    peintre.circle(
                        bout,
                        7.0,
                        Color32::WHITE,
                        Stroke::new(2.5, ORANGE_DES_COTES),
                    );
                }
            }
        }
        let lectures = pieces::lire_les_cotes(&piece.cotes, piece.largeur, piece.hauteur);
        for (cote, lecture) in piece.cotes.iter().zip(lectures) {
            let milieu = vers_l_ecran(cote.a) + (vers_l_ecran(cote.b) - vers_l_ecran(cote.a)) / 2.0;
            peindre_le_texte_d_une_cote(&peintre, milieu, &texte_d_une_cote(cote, lecture, prefs));
        }
        // La cote en train d'être tracée.
        if let (Some(depart), Some(pointeur)) = (etat.depart, reponse.interact_pointer_pos()) {
            peindre_le_trait_d_une_cote(&peintre, vers_l_ecran(depart), pointeur);
        }
        ui.add_space(8.0);
        // La mesure de la cote choisie.
        if let Some(rang) = etat.choisie {
            let mut enlever = false;
            ui.horizontal_wrapped(|ui| {
                ui.label(textes::MESURE_DE_LA_COTE.traduit());
                let champ = ui.add(
                    egui::TextEdit::singleline(&mut piece.cotes[rang].mesure)
                        .desired_width(140.0)
                        .char_limit(40)
                        .hint_text(textes::MESURE_INDICE.traduit()),
                );
                cadre_de_saisie(ui, p, &champ);
                if champ.changed() {
                    action = Some(Action::Modifiee);
                }
                let mut reference = piece.cotes[rang].reference;
                if ui
                    .checkbox(&mut reference, textes::REFERENCE.traduit())
                    .changed()
                {
                    // Une seule cote de référence.
                    for (k, cote) in piece.cotes.iter_mut().enumerate() {
                        cote.reference = reference && k == rang;
                    }
                    action = Some(Action::Modifiee);
                }
                if ui.button(textes::DESSIN_ENLEVER.traduit()).clicked() {
                    enlever = true;
                }
            });
            if enlever {
                piece.cotes.remove(rang);
                etat.choisie = None;
                action = Some(Action::Modifiee);
            }
        }
        // La consigne s'écrit sous la photo, pour que la photo ne bouge pas.
        if etat.tracer {
            ui.label(RichText::new(textes::COTE_CONSIGNE.traduit()).color(p.texte));
        }
        if !piece.cotes.is_empty() || etat.tracer {
            ui.label(
                RichText::new(textes::COTES_EXPLICATION.traduit())
                    .size(12.5)
                    .color(p.texte_attenue),
            );
        }
        ui.add_space(12.0);
    });
    action
}

/// Les gestes sur la photo : tracer une cote (après « + Cote »), faire glisser
/// un bout de la cote choisie, ou toucher une cote pour la choisir. Vrai si
/// les cotes ont changé.
fn gestes_sur_la_photo(
    ui: &Ui,
    reponse: &egui::Response,
    piece: &mut PieceJointe,
    etat: &mut EtatDeLaPhoto,
    vers_l_ecran: impl Fn((f64, f64)) -> Pos2,
    vers_la_photo: impl Fn(Pos2) -> (f64, f64),
) -> bool {
    let mut modifiee = false;
    if reponse.drag_started()
        && let Some(origine) = ui.input(|i| i.pointer.press_origin())
    {
        if etat.tracer {
            etat.depart = Some(vers_la_photo(origine));
        } else if let Some(rang) = etat.choisie {
            let cote = &piece.cotes[rang];
            let (a, b) = (vers_l_ecran(cote.a), vers_l_ecran(cote.b));
            if origine.distance(a) <= PORTEE_D_UNE_COTE {
                etat.poignee = Some((rang, true));
            } else if origine.distance(b) <= PORTEE_D_UNE_COTE {
                etat.poignee = Some((rang, false));
            }
        }
    }
    if reponse.dragged()
        && let Some(pointeur) = reponse.interact_pointer_pos()
        && let Some((rang, bout_a)) = etat.poignee
        && let Some(cote) = piece.cotes.get_mut(rang)
    {
        let point = vers_la_photo(pointeur);
        if bout_a {
            cote.a = point;
        } else {
            cote.b = point;
        }
        modifiee = true;
    }
    if reponse.drag_stopped() {
        etat.poignee = None;
        if let (Some(depart), Some(arrivee)) = (etat.depart.take(), reponse.interact_pointer_pos())
        {
            // Un geste trop court n'est pas une cote.
            if vers_l_ecran(depart).distance(arrivee) >= 12.0 {
                piece.cotes.push(CoteSurPhoto {
                    a: depart,
                    b: vers_la_photo(arrivee),
                    mesure: String::new(),
                    // La première cote sert de référence.
                    reference: piece.cotes.is_empty(),
                });
                etat.choisie = Some(piece.cotes.len() - 1);
                etat.tracer = false;
                modifiee = true;
            }
        }
    }
    if reponse.clicked()
        && let Some(point) = reponse.interact_pointer_pos()
    {
        etat.choisie = piece
            .cotes
            .iter()
            .enumerate()
            .map(|(rang, cote)| {
                let distance =
                    distance_au_segment(point, vers_l_ecran(cote.a), vers_l_ecran(cote.b));
                (rang, distance)
            })
            .filter(|(_, distance)| *distance <= PORTEE_D_UNE_COTE)
            .min_by(|x, y| x.1.total_cmp(&y.1))
            .map(|(rang, _)| rang);
    }
    modifiee
}

/// Le trait d'une cote sur la photo : orange liseré de sombre, avec un trait
/// en travers à chaque bout.
fn peindre_le_trait_d_une_cote(peintre: &egui::Painter, a: Pos2, b: Pos2) {
    let ab = b - a;
    let normale = if ab.length() > f32::EPSILON {
        Vec2::new(-ab.y, ab.x).normalized() * 7.0
    } else {
        Vec2::ZERO
    };
    for (largeur, couleur) in [(5.0, LISERE_DES_COTES), (2.5, ORANGE_DES_COTES)] {
        peintre.line_segment([a, b], Stroke::new(largeur, couleur));
        for bout in [a, b] {
            peintre.line_segment(
                [bout - normale, bout + normale],
                Stroke::new(largeur, couleur),
            );
        }
    }
}

/// Le texte d'une cote, au milieu de son trait, sur fond blanc.
fn peindre_le_texte_d_une_cote(peintre: &egui::Painter, milieu: Pos2, texte: &str) {
    let galee = peintre.layout_no_wrap(
        texte.to_string(),
        theme::police_grasse(13.0),
        LISERE_DES_COTES,
    );
    let fond = Rect::from_center_size(milieu, galee.size() + Vec2::new(10.0, 4.0));
    peintre.rect_filled(fond, 4.0, Color32::from_white_alpha(235));
    peintre.galley(fond.center() - galee.size() / 2.0, galee, LISERE_DES_COTES);
}
