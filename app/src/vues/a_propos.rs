//! Écran « À propos » (sections 1.5 et 15.2).

use std::path::Path;

use eframe::egui::{RichText, Ui};

use super::{carte, corps, en_tete, section};
use crate::textes;
use crate::theme::{self, Palette};

/// Retourne vrai quand l'utilisateur ferme l'écran.
pub fn afficher(ui: &mut Ui, p: &Palette, dossier: &Path) -> bool {
    let fermer = en_tete(ui, p, textes::A_PROPOS, textes::FERMER);
    corps(ui, "a-propos", |ui| {
        ui.add_space(8.0);
        ui.label(
            RichText::new(textes::NOM_APPLICATION)
                .font(theme::police_grasse(22.0))
                .color(p.entete),
        );
        ui.label(
            RichText::new(format!("{} {}", textes::A_PROPOS_VERSION, textes::VERSION))
                .color(p.texte_attenue),
        );
        ui.add_space(6.0);
        ui.label(textes::A_PROPOS_DESCRIPTION);

        section(ui, p, textes::AVERTISSEMENT_TITRE);
        carte(ui, p, |ui| {
            ui.label(textes::AVERTISSEMENT);
            ui.add_space(6.0);
            ui.label(RichText::new(textes::AVERTISSEMENT_DEFAUTS).color(p.texte_attenue));
        });

        section(ui, p, textes::DOSSIER_DE_DONNEES);
        ui.label(RichText::new(dossier.display().to_string()).monospace());
        ui.label(RichText::new(textes::CONFIDENTIALITE).color(p.texte_attenue));

        section(ui, p, textes::BIBLIOTHEQUES_TITRE);
        ui.label(textes::BIBLIOTHEQUES);
        ui.add_space(12.0);
    });
    fermer
}
