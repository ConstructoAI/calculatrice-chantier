//! Écran « Trucs de pro » (annexe B.4).

use eframe::egui::Ui;

use super::{corps, en_tete, point_numerote};
use crate::textes;
use crate::theme::Palette;

/// Retourne vrai quand l'utilisateur ferme l'écran.
pub fn afficher(ui: &mut Ui, p: &Palette) -> bool {
    let fermer = en_tete(ui, p, textes::TRUCS_TITRE, textes::FERMER);
    corps(ui, "trucs", |ui| {
        ui.add_space(6.0);
        for (i, (titre, texte)) in textes::TRUCS.iter().enumerate() {
            point_numerote(ui, p, i + 1, titre, texte);
        }
    });
    fermer
}
