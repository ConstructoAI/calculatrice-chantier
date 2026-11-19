//! Écran « À propos » (sections 1.5 et 15.2).

use eframe::egui::{RichText, Ui};

use super::{carte, corps, en_tete, section};
use crate::plateforme;
use crate::textes;
use crate::theme::{self, Palette};

/// `dossier` : le dossier des données sur le bureau; `None` dans un
/// navigateur, qui les conserve lui-même. Retourne vrai quand l'utilisateur
/// ferme l'écran.
pub fn afficher(ui: &mut Ui, p: &Palette, dossier: Option<&str>) -> bool {
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

        match dossier {
            Some(dossier) => {
                section(ui, p, textes::DOSSIER_DE_DONNEES);
                ui.label(RichText::new(dossier).monospace());
                ui.label(RichText::new(textes::CONFIDENTIALITE).color(p.texte_attenue));
            }
            None => {
                section(ui, p, textes::DONNEES_WEB_TITRE);
                ui.label(textes::DONNEES_WEB);
                ui.label(RichText::new(textes::CONFIDENTIALITE_WEB).color(p.texte_attenue));
            }
        }

        section(ui, p, textes::BIBLIOTHEQUES_TITRE);
        if plateforme::NAVIGATEUR {
            ui.label(textes::BIBLIOTHEQUES_WEB);
            ui.add_space(4.0);
            ui.label(textes::POLICES_WEB);
        } else {
            ui.label(textes::BIBLIOTHEQUES);
        }
        ui.add_space(12.0);
    });
    fermer
}
