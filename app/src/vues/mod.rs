//! Écrans secondaires (section 15.2) et éléments de mise en page communs.

pub mod a_propos;
pub mod aide;
pub mod bienvenue;
pub mod dessin;
pub mod metiers;
pub mod preferences;
pub mod rubans;
pub mod trucs;

use eframe::egui::text::{LayoutJob, TextFormat};
use eframe::egui::{
    self, Align, Layout, Margin, Pos2, Response, RichText, Sense, Stroke, StrokeKind, Ui, Vec2,
};
use noyau::langue::{self, Langue};

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
        // En français, une espace précède le deux-points qui ouvre la suite;
        // en anglais, le deux-points se colle au mot.
        let colle = texte.starts_with(':') && langue::en_cours() == Langue::Anglais;
        if !colle {
            job.append(" ", 0.0, courant.clone());
        }
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

/// Genre d'une pastille de la barre des métiers (section 23).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pastille {
    /// Le métier choisi : pleine, avec un chevron; elle ouvre la liste des métiers.
    Menu,
    /// L'outil branché sur le clavier.
    Active,
    Ordinaire,
}

/// Une pastille cliquable : un mot ou deux dans un cadre arrondi.
pub fn pastille(ui: &mut Ui, p: &Palette, texte: &str, genre: Pastille) -> Response {
    const HAUTEUR: f32 = 26.0;
    let (fond, bord, encre) = match genre {
        Pastille::Menu => (p.selection, p.selection, p.selection_texte),
        Pastille::Active => (
            theme::melanger(p.carte, p.selection, 0.20),
            p.selection,
            p.titre,
        ),
        Pastille::Ordinaire => (p.bouton, p.bouton_bord, p.texte),
    };
    let police = if genre == Pastille::Ordinaire {
        theme::police(12.5)
    } else {
        theme::police_grasse(12.5)
    };
    let ecrit = ui.painter().layout_no_wrap(texte.to_owned(), police, encre);
    // Place du chevron de la pastille de menu.
    let chevron = if genre == Pastille::Menu { 13.0 } else { 0.0 };
    let (rect, reponse) = ui.allocate_exact_size(
        Vec2::new(ecrit.size().x + 22.0 + chevron, HAUTEUR),
        Sense::click(),
    );
    reponse.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, texte));
    if ui.is_rect_visible(rect) {
        let part = if reponse.is_pointer_button_down_on() {
            0.22
        } else if reponse.hovered() {
            0.10
        } else {
            0.0
        };
        let painter = ui.painter();
        painter.rect_filled(rect, 13, theme::melanger(fond, encre, part));
        painter.rect_stroke(
            rect,
            13,
            Stroke::new(if genre == Pastille::Active { 1.5 } else { 1.0 }, bord),
            StrokeKind::Inside,
        );
        painter.galley(
            Pos2::new(rect.left() + 11.0, rect.center().y - ecrit.size().y / 2.0),
            ecrit,
            encre,
        );
        if genre == Pastille::Menu {
            // Chevron vers le bas : la pastille ouvre une liste.
            let centre = Pos2::new(rect.right() - 14.0, rect.center().y + 0.5);
            painter.add(egui::Shape::convex_polygon(
                vec![
                    Pos2::new(centre.x - 4.0, centre.y - 2.5),
                    Pos2::new(centre.x + 4.0, centre.y - 2.5),
                    Pos2::new(centre.x, centre.y + 3.0),
                ],
                encre,
                Stroke::NONE,
            ));
        }
    }
    reponse
}
