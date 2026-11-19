//! Écran « Aide » : recherche et fiches (section 15.3).

use eframe::egui::{self, Id, RichText, Sense, Ui};

use super::{cadre_de_saisie, carte, corps, en_tete, section, texte_a_touches};
use crate::aide::{FICHES, Fiche};
use crate::textes;
use crate::theme::{self, Palette};

/// Ce que l'utilisateur a demandé dans l'écran d'aide.
pub enum Action {
    Ouvrir(usize),
    /// Retour à la liste des fiches.
    Fermer,
}

/// Dessine une fiche complète : titre, accès, texte, exemple, note.
pub fn dessiner_fiche(ui: &mut Ui, p: &Palette, fiche: &Fiche) {
    ui.label(
        RichText::new(fiche.titre)
            .font(theme::police_grasse(21.0))
            .color(p.entete),
    );
    if !fiche.acces.is_empty() {
        ui.label(texte_a_touches(p, "", fiche.acces, 13.5));
    }
    ui.add_space(6.0);
    // Le résumé, en gras, ouvre le mode d'emploi.
    ui.label(texte_a_touches(p, fiche.resume, fiche.mode_emploi, 14.5));
    if let Some((frappes, resultat)) = fiche.exemple {
        section(ui, p, textes::EXEMPLE);
        carte(ui, p, |ui| {
            ui.label(texte_a_touches(p, "", frappes, 14.0));
            ui.add_space(2.0);
            ui.label(RichText::new(textes::resultat(resultat)).font(theme::police_grasse(14.5)));
        });
    }
    if let Some(note) = fiche.note {
        section(ui, p, textes::NOTE);
        ui.label(texte_a_touches(p, "", note, 14.0));
    }
}

/// Une ligne de la liste : titre et résumé; cliquable.
fn ligne_de_fiche(ui: &mut Ui, p: &Palette, index: usize, fiche: &Fiche) -> bool {
    let cadre = egui::Frame::new()
        .fill(p.carte)
        .stroke(egui::Stroke::new(1.0, p.carte_bord))
        .corner_radius(8)
        .inner_margin(egui::Margin::symmetric(10, 7))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().interact_size.y = 0.0;
            ui.spacing_mut().item_spacing.y = 3.0;
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new(fiche.titre).font(theme::police_grasse(15.0)));
                if !fiche.acces.is_empty() {
                    ui.label(texte_a_touches(p, "", fiche.acces, 12.0));
                }
            });
            ui.label(
                RichText::new(fiche.resume)
                    .size(13.0)
                    .color(p.texte_attenue),
            );
        });
    let reponse = ui.interact(
        cadre.response.rect,
        Id::unique(("fiche-aide", index)),
        Sense::click(),
    );
    reponse.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, fiche.titre));
    if reponse.hovered() {
        ui.painter().rect_stroke(
            cadre.response.rect,
            8,
            egui::Stroke::new(1.5, p.selection),
            egui::StrokeKind::Inside,
        );
    }
    reponse.clicked()
}

/// Affiche la liste des fiches, ou la fiche ouverte.
pub fn afficher(
    ui: &mut Ui,
    p: &Palette,
    recherche: &mut String,
    ouverte: Option<usize>,
) -> Option<Action> {
    let mut action = None;
    if let Some(fiche) = ouverte.and_then(|i| FICHES.get(i)) {
        if en_tete(ui, p, textes::AIDE, textes::RETOUR) {
            action = Some(Action::Fermer);
        }
        corps(ui, "aide-fiche", |ui| {
            ui.add_space(6.0);
            dessiner_fiche(ui, p, fiche);
            ui.add_space(12.0);
        });
        return action;
    }

    ui.add_space(10.0);
    ui.horizontal(|ui| {
        ui.add_space(14.0);
        ui.label(
            RichText::new(textes::AIDE)
                .font(theme::police_grasse(20.0))
                .color(p.entete),
        );
    });
    ui.horizontal(|ui| {
        ui.add_space(14.0);
        let champ = ui.add(
            egui::TextEdit::singleline(recherche)
                .hint_text(textes::RECHERCHER)
                .desired_width(ui.available_width() - 28.0),
        );
        cadre_de_saisie(ui, p, &champ);
    });
    ui.add_space(2.0);
    ui.separator();
    corps(ui, "aide-liste", |ui| {
        let trouvees: Vec<(usize, &Fiche)> = FICHES
            .iter()
            .enumerate()
            .filter(|(_, f)| f.correspond(recherche))
            .collect();
        if trouvees.is_empty() {
            ui.add_space(12.0);
            ui.label(RichText::new(textes::AUCUNE_FICHE).color(p.texte_attenue));
            return;
        }
        let mut titre_secondaire_ecrit = false;
        let mut titre_principal_ecrit = false;
        for (index, fiche) in trouvees {
            if fiche.secondaire && !titre_secondaire_ecrit {
                section(ui, p, textes::FONCTIONS_SECONDAIRES);
                titre_secondaire_ecrit = true;
            } else if !fiche.secondaire && !titre_principal_ecrit && !titre_secondaire_ecrit {
                section(ui, p, textes::TOUCHES_PRINCIPALES);
                titre_principal_ecrit = true;
            }
            if ligne_de_fiche(ui, p, index, fiche) {
                action = Some(Action::Ouvrir(index));
            }
        }
        ui.add_space(12.0);
    });
    action
}
