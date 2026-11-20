//! Écran « À propos » (sections 1.5 et 15.2).

use eframe::egui::{self, RichText, Ui};

use super::{carte, corps, en_tete, section};
use crate::plateforme;
use crate::textes;
use crate::theme::{self, Palette};

/// `dossier` : le dossier des données sur le bureau; `None` dans un
/// navigateur, qui les conserve lui-même. Retourne vrai quand l'utilisateur
/// ferme l'écran.
pub fn afficher(ui: &mut Ui, p: &Palette, dossier: Option<&str>) -> bool {
    let fermer = en_tete(ui, p, textes::A_PROPOS.traduit(), textes::FERMER.traduit());
    corps(ui, "a-propos", |ui| {
        ui.add_space(8.0);
        ui.label(
            RichText::new(textes::NOM_APPLICATION.traduit())
                .font(theme::police_grasse(22.0))
                .color(p.entete),
        );
        ui.label(RichText::new(textes::version_et_revision()).color(p.texte_attenue));
        ui.add_space(6.0);
        ui.label(textes::A_PROPOS_DESCRIPTION.traduit());
        ui.add_space(4.0);
        ui.label(
            RichText::new(textes::GRACIEUSETE.traduit())
                .font(theme::police_grasse(14.5))
                .color(p.titre),
        );
        // Les coordonnées de Constructo AI : le courriel s'ouvre dans le
        // logiciel de courriel, le site dans un nouvel onglet.
        ui.hyperlink_to(
            textes::COURRIEL_CONSTRUCTO,
            textes::LIEN_COURRIEL_CONSTRUCTO,
        );
        ui.add(
            egui::Hyperlink::from_label_and_url(
                textes::SITE_CONSTRUCTO,
                textes::LIEN_SITE_CONSTRUCTO,
            )
            .open_in_new_tab(true),
        );

        section(ui, p, textes::AVERTISSEMENT_TITRE.traduit());
        carte(ui, p, |ui| {
            ui.label(textes::AVERTISSEMENT.traduit());
            ui.add_space(6.0);
            ui.label(RichText::new(textes::AVERTISSEMENT_DEFAUTS.traduit()).color(p.texte_attenue));
        });

        match dossier {
            Some(dossier) => {
                section(ui, p, textes::DOSSIER_DE_DONNEES.traduit());
                ui.label(RichText::new(dossier).monospace());
                ui.label(RichText::new(textes::CONFIDENTIALITE.traduit()).color(p.texte_attenue));
            }
            None => {
                section(ui, p, textes::DONNEES_WEB_TITRE.traduit());
                ui.label(textes::DONNEES_WEB.traduit());
                ui.label(
                    RichText::new(textes::CONFIDENTIALITE_WEB.traduit()).color(p.texte_attenue),
                );
            }
        }

        section(ui, p, textes::LICENCE_TITRE.traduit());
        ui.label(textes::LICENCE.traduit());
        ui.add(
            egui::Hyperlink::from_label_and_url(
                textes::CODE_SOURCE.traduit(),
                textes::LIEN_CODE_SOURCE,
            )
            .open_in_new_tab(true),
        );

        section(ui, p, textes::BIBLIOTHEQUES_TITRE.traduit());
        if plateforme::NAVIGATEUR {
            ui.label(textes::BIBLIOTHEQUES_WEB.traduit());
            ui.add_space(4.0);
            ui.label(textes::POLICES_WEB.traduit());
        } else {
            ui.label(textes::BIBLIOTHEQUES.traduit());
        }
        ui.add_space(12.0);
    });
    fermer
}
