//! Écrans secondaires (section 15.2) et éléments de mise en page communs.

pub mod a_propos;
pub mod aide;
pub mod bienvenue;
pub mod metiers;
pub mod preferences;
pub mod rubans;
pub mod trucs;

use eframe::egui::text::{LayoutJob, TextFormat};
use eframe::egui::{self, Align, Layout, Margin, RichText, Ui};

use crate::theme::{self, Palette};

/// Marge latérale des écrans secondaires.
const MARGE: i8 = 14;

/// En-tête d'un écran : le titre à gauche, un bouton à droite.
/// Retourne vrai si le bouton a été cliqué.
pub fn en_tete(ui: &mut Ui, p: &Palette, titre: &str, bouton: &str) -> bool {
    let mut clic = false;
    ui.add_space(10.0);
    ui.horizontal(|ui| {
        ui.add_space(f32::from(MARGE));
        ui.label(
            RichText::new(titre)
                .font(theme::police_grasse(20.0))
                .color(p.entete),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add_space(f32::from(MARGE));
            clic = ui.add(bouton_principal(p, bouton, 14.5)).clicked();
        });
    });
    ui.add_space(4.0);
    ui.separator();
    clic
}

/// Bouton d'action principal : plein, de la couleur primaire.
pub fn bouton_principal<'a>(p: &Palette, libelle: &'a str, taille: f32) -> egui::Button<'a> {
    egui::Button::new(
        RichText::new(libelle)
            .font(theme::police_grasse(taille))
            .color(p.selection_texte),
    )
    .fill(p.selection)
    .stroke(egui::Stroke::new(1.0, p.selection))
}

/// Entoure d'un liseré de la couleur primaire le champ de texte en cours de
/// saisie. (Le liseré qu'egui dessine de lui-même prend la couleur du texte
/// sélectionné : blanc sur blanc dans le thème clair.)
pub fn cadre_de_saisie(ui: &Ui, p: &Palette, champ: &egui::Response) {
    if champ.has_focus() {
        ui.painter().rect_stroke(
            champ.rect,
            4,
            egui::Stroke::new(1.5, p.primaire),
            egui::StrokeKind::Outside,
        );
    }
}

/// Corps défilant d'un écran, avec ses marges.
pub fn corps<R>(ui: &mut Ui, identifiant: &str, contenu: impl FnOnce(&mut Ui) -> R) -> R {
    egui::ScrollArea::vertical()
        .id_salt(identifiant)
        .auto_shrink(false)
        .show(ui, |ui| {
            egui::Frame::new()
                .inner_margin(Margin::symmetric(MARGE, 8))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    contenu(ui)
                })
                .inner
        })
        .inner
}

/// Titre de section.
pub fn section(ui: &mut Ui, p: &Palette, titre: &str) {
    ui.add_space(10.0);
    ui.label(
        RichText::new(titre)
            .font(theme::police_grasse(13.0))
            .color(p.titre),
    );
    ui.add_space(2.0);
}

/// Carte : un cadre arrondi sur le fond de l'application.
pub fn carte<R>(ui: &mut Ui, p: &Palette, contenu: impl FnOnce(&mut Ui) -> R) -> R {
    egui::Frame::new()
        .fill(p.carte)
        .stroke(egui::Stroke::new(1.0, p.carte_bord))
        .corner_radius(6)
        .shadow(egui::Shadow {
            offset: [0, 2],
            blur: 8,
            spread: 0,
            color: p.ombre.gamma_multiply(0.45),
        })
        .inner_margin(Margin::symmetric(12, 10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            contenu(ui)
        })
        .inner
}

/// Compose un texte dont les passages entre accents graves sont dessinés comme
/// de petites touches. `accroche`, s'il n'est pas vide, précède le texte, en gras.
pub fn texte_a_touches(p: &Palette, accroche: &str, texte: &str, taille: f32) -> LayoutJob {
    let courant = TextFormat {
        font_id: theme::police(taille),
        color: p.texte,
        ..TextFormat::default()
    };
    let gras = TextFormat {
        font_id: theme::police_grasse(taille),
        color: p.texte,
        ..TextFormat::default()
    };
    let touche = TextFormat {
        font_id: theme::police_grasse(taille * 0.92),
        color: p.puce_texte,
        background: p.puce,
        expand_bg: 1.5,
        ..TextFormat::default()
    };
    let mut job = LayoutJob::default();
    if !accroche.is_empty() {
        job.append(accroche, 0.0, gras);
        job.append(" ", 0.0, courant.clone());
    }
    for (i, morceau) in texte.split('`').enumerate() {
        if morceau.is_empty() {
            continue;
        }
        if i % 2 == 1 {
            // Espaces fines de part et d'autre, pour que le fond déborde un peu.
            job.append(&format!("\u{2009}{morceau}\u{2009}"), 0.0, touche.clone());
        } else {
            job.append(morceau, 0.0, courant.clone());
        }
    }
    job
}

/// Paragraphe numéroté : « 1. **Titre** suite du texte ».
pub fn point_numerote(ui: &mut Ui, p: &Palette, numero: usize, titre: &str, texte: &str) {
    ui.horizontal_top(|ui| {
        ui.label(
            RichText::new(format!("{numero}."))
                .font(theme::police_grasse(14.5))
                .color(p.titre),
        );
        ui.vertical(|ui| {
            ui.label(texte_a_touches(p, titre, texte, 14.5));
        });
    });
    ui.add_space(4.0);
}
