//! Écran « Bienvenue » (annexe B.3), affiché au premier lancement.

use eframe::egui::{RichText, Ui};

use super::{bouton_principal, corps, point_numerote};
use crate::textes;
use crate::theme::{self, Palette};

/// Retourne vrai quand l'utilisateur choisit de commencer.
pub fn afficher(ui: &mut Ui, p: &Palette) -> bool {
    corps(ui, "bienvenue", |ui| {
        ui.add_space(18.0);
        ui.label(
            RichText::new(textes::BIENVENUE_TITRE.traduit())
                .font(theme::police_grasse(26.0))
                .color(p.entete),
        );
        ui.add_space(8.0);
        ui.label(RichText::new(textes::BIENVENUE_INTRO.traduit()).size(15.0));
        ui.add_space(14.0);
        for (i, (titre, suite)) in textes::BIENVENUE_POINTS.iter().enumerate() {
            point_numerote(ui, p, i + 1, titre.traduit(), suite.traduit());
            ui.add_space(4.0);
        }
        ui.add_space(16.0);
        ui.vertical_centered(|ui| {
            let bouton = bouton_principal(p, textes::COMMENCER.traduit(), 16.0)
                .min_size(eframe::egui::vec2(180.0, 42.0));
            ui.add(bouton).clicked()
        })
        .inner
    })
}
