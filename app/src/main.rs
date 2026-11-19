//! Calculatrice de chantier — point d'entrée de l'interface graphique.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![forbid(unsafe_code)]

mod afficheur;
mod aide;
mod application;
mod clavier;
mod raccourcis;
mod stockage;
mod textes;
mod theme;
mod vues;

use eframe::egui;

use application::Application;

/// Côté de l'icône de la fenêtre, en pixels.
const TAILLE_ICONE: u32 = 64;

fn main() -> eframe::Result {
    let icone = egui::IconData {
        rgba: theme::icone_rvba(TAILLE_ICONE),
        width: TAILLE_ICONE,
        height: TAILLE_ICONE,
    };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(textes::NOM_APPLICATION)
            .with_inner_size([420.0, 860.0])
            .with_min_inner_size([340.0, 620.0])
            .with_app_id(textes::APP_ID)
            .with_icon(icone),
        ..Default::default()
    };
    eframe::run_native(
        textes::NOM_APPLICATION,
        options,
        Box::new(|cc| Ok(Box::new(Application::nouvelle(cc)))),
    )
}
