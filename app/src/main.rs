//! Calculatrice de chantier — point d'entrée de l'interface graphique.
//!
//! Le même programme donne deux versions : une application de bureau et une
//! page Web (cible `wasm32-unknown-unknown`, page hôte `app/index.html`).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![forbid(unsafe_code)]

mod afficheur;
mod aide;
mod application;
mod clavier;
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

    /// Côté de l'icône de la fenêtre, en pixels.
    const TAILLE_ICONE: u32 = 64;

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

/// Version Web : l'application se dessine dans le canevas de la page.
#[cfg(target_arch = "wasm32")]
fn main() {
    plateforme::demarrer(Box::new(|cc| Ok(Box::new(Application::nouvelle(cc)))));
}
