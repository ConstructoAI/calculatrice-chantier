//! Écran « Notes » : le bloc-notes (section 26). Une liste de notes, chacune
//! avec son titre, son texte, sa date et ses pièces jointes; une note s'ouvre
//! pour s'écrire, et une photo jointe s'y voit en grand.

use std::collections::HashMap;

use eframe::egui::{self, RichText, Ui, Vec2};

use super::{bouton_principal, cadre_de_saisie, carte, corps, en_tete};
use crate::plateforme;
use crate::stockage::{NoteDuBloc, PieceJointe};
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
}

/// Ce que l'écran montre des pièces jointes : les vignettes des photos de la
/// note ouverte (rien si la vignette est introuvable), et la photo vue en
/// grand, avec sa texture une fois lue.
pub struct Images<'a> {
    pub vignettes: &'a HashMap<u64, Option<egui::TextureHandle>>,
    pub photo: Option<(u64, Option<&'a egui::TextureHandle>)>,
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
) -> Option<Action> {
    let mut action = None;

    // Une note ouverte : son titre et son texte s'écrivent sur place.
    if let Some(note) = ouverte.and_then(|id| notes.iter_mut().find(|n| n.id == id)) {
        if let Some((id_photo, texture)) = images.photo
            && let Some(piece) = note.pieces.iter().find(|piece| piece.id == id_photo)
        {
            return afficher_la_photo(ui, p, piece, texture);
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

/// Une photo jointe, vue en grand dans la place qui reste, avec ses boutons.
fn afficher_la_photo(
    ui: &mut Ui,
    p: &Palette,
    piece: &PieceJointe,
    texture: Option<&egui::TextureHandle>,
) -> Option<Action> {
    let mut action = None;
    if en_tete(ui, p, &piece.nom, textes::RETOUR.traduit()) {
        action = Some(Action::FermerPhoto);
    }
    corps(ui, "photo-jointe", |ui| {
        ui.horizontal_wrapped(|ui| {
            if ui.button(textes::ENREGISTRER.traduit()).clicked() {
                action = Some(Action::EnregistrerPiece(piece.id));
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
        ui.add_space(8.0);
        let place = Vec2::new(
            ui.available_width(),
            (ui.available_height() - 12.0).max(120.0),
        );
        match texture {
            Some(texture) => {
                let taille = texture.size_vec2();
                let echelle = (place.x / taille.x).min(place.y / taille.y).max(0.01);
                ui.add(
                    egui::Image::from_texture(egui::load::SizedTexture::from_handle(texture))
                        .fit_to_exact_size(taille * echelle),
                );
            }
            None => {
                ui.add_space(place.y / 3.0);
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new(textes::CHARGEMENT.traduit()).color(p.texte_attenue));
                });
            }
        }
        ui.add_space(12.0);
    });
    action
}
