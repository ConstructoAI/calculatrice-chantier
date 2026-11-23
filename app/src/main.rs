//! Constructo AI, calculatrice de chantier — point d'entrée de l'interface graphique.
//!
//! Le même programme donne deux versions : une application de bureau et une
//! page Web (cible `wasm32-unknown-unknown`, page hôte `app/index.html`).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![forbid(unsafe_code)]

mod afficheur;
mod aide;
mod application;
mod clavier;
mod exportation;
#[cfg(test)]
mod manuel;
mod pieces;
mod plateforme;
mod raccourcis;
mod stockage;
mod textes;
mod theme;
mod vues;

use application::Application;

/// Version de bureau : une fenêtre native.
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    use eframe::egui;

    let icone = egui::IconData {
        rgba: theme::ICONE_RVBA.to_vec(),
        width: theme::COTE_ICONE as u32,
        height: theme::COTE_ICONE as u32,
    };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(textes::NOM_APPLICATION.traduit())
            .with_inner_size([420.0, 860.0])
            .with_min_inner_size([340.0, 620.0])
            .with_app_id(textes::APP_ID)
            .with_icon(icone),
        ..Default::default()
    };
    eframe::run_native(
        textes::NOM_APPLICATION.traduit(),
        options,
        Box::new(|cc| Ok(Box::new(Application::nouvelle(cc)))),
    )
}

/// Version Web : l'application se dessine dans le canevas de la page.
#[cfg(target_arch = "wasm32")]
fn main() {
    plateforme::demarrer(Box::new(|cc| Ok(Box::new(Application::nouvelle(cc)))));
}
