//! Écran « Notes » : le bloc-notes (section 26). Une liste de notes, chacune
//! avec son titre, son texte et sa date; une note s'ouvre pour s'écrire.

use eframe::egui::{self, RichText, Ui};

use super::{bouton_principal, cadre_de_saisie, carte, corps, en_tete};
use crate::stockage::NoteDuBloc;
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
}

/// Le titre d'une note, ou « Note sans titre ».
pub fn titre_de(note: &NoteDuBloc) -> String {
    let titre = note.titre.trim();
    if titre.is_empty() {
        textes::NOTE_SANS_TITRE.traduit().to_string()
    } else {
        titre.to_string()
    }
}

/// Affiche l'écran des notes : la note ouverte, ou la liste. `viser` donne le
/// clavier au texte de la note ouverte (une note neuve).
pub fn afficher(
    ui: &mut Ui,
    p: &Palette,
    notes: &mut [NoteDuBloc],
    ouverte: Option<u64>,
    viser: &mut bool,
) -> Option<Action> {
    let mut action = None;

    // Une note ouverte : son titre et son texte s'écrivent sur place.
    if let Some(note) = ouverte.and_then(|id| notes.iter_mut().find(|n| n.id == id)) {
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
            ui.add_space(8.0);
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
                ui.label(
                    RichText::new(note.date_en_clair())
                        .size(12.5)
                        .color(p.texte_attenue),
                );
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
