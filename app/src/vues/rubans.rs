//! Écran « Rubans » : ruban courant et rubans enregistrés (section 14).

use eframe::egui::{self, Align, Layout, RichText, Ui};
use noyau::ruban::GenreLigne;
use noyau::{LigneRuban, Ruban};

use super::{carte, corps, en_tete, section};
use crate::stockage::RubanEnregistre;
use crate::textes;
use crate::theme::{self, Palette};

/// Ce que l'utilisateur a demandé dans l'écran des rubans.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Enregistrer,
    Copier,
    Exporter,
    Vider,
    Ouvrir(u64),
    Renommer(u64),
    CopierEnregistre(u64),
    ExporterEnregistre(u64),
    Supprimer(u64),
    /// Ferme le ruban ouvert en lecture seule.
    FermerLecture,
}

/// Dessine les lignes d'un ruban : valeurs alignées à droite, totaux en gras,
/// séparateurs en filet.
pub fn dessiner_lignes(ui: &mut Ui, p: &Palette, lignes: &[LigneRuban]) {
    for ligne in lignes {
        if ligne.genre == GenreLigne::Separateur {
            ui.separator();
            continue;
        }
        if ligne.genre == GenreLigne::Titre {
            // Nom d'un calculateur par métier : en tête de son bloc de lignes.
            ui.label(
                RichText::new(&ligne.texte)
                    .font(theme::police_grasse(14.0))
                    .color(p.titre),
            );
            continue;
        }
        let mut gauche = String::new();
        if let Some(signe) = ligne.signe() {
            gauche.push(signe);
        }
        if let Some(etiquette) = &ligne.etiquette {
            if !gauche.is_empty() {
                gauche.push(' ');
            }
            gauche.push_str(etiquette);
        }
        let total = matches!(ligne.genre, GenreLigne::Total | GenreLigne::SousTotal);
        ui.horizontal(|ui| {
            ui.label(RichText::new(gauche).size(12.5).color(p.texte_attenue));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let police = if total {
                    theme::police_grasse(15.0)
                } else {
                    theme::police(15.0)
                };
                ui.label(RichText::new(&ligne.texte).font(police));
            });
        });
    }
}

/// Boutons d'action du ruban courant.
fn actions_du_ruban_courant(ui: &mut Ui, vide: bool) -> Option<Action> {
    let mut action = None;
    ui.horizontal_wrapped(|ui| {
        for (libelle, a) in [
            (textes::ENREGISTRER.traduit(), Action::Enregistrer),
            (textes::COPIER.traduit(), Action::Copier),
            (textes::EXPORTER.traduit(), Action::Exporter),
            (textes::VIDER.traduit(), Action::Vider),
        ] {
            if ui.add_enabled(!vide, egui::Button::new(libelle)).clicked() {
                action = Some(a);
            }
        }
    });
    action
}

/// Ruban courant : actions, puis lignes dans une zone défilante qui suit la
/// dernière ligne. Sert à l'écran des rubans et au panneau de la fenêtre large.
pub fn ruban_courant(
    ui: &mut Ui,
    p: &Palette,
    ruban: &Ruban,
    hauteur_maximale: f32,
) -> Option<Action> {
    let action = actions_du_ruban_courant(ui, ruban.est_vide());
    ui.add_space(2.0);
    carte(ui, p, |ui| {
        if ruban.est_vide() {
            ui.label(RichText::new(textes::RUBAN_VIDE.traduit()).color(p.texte_attenue));
            return;
        }
        egui::ScrollArea::vertical()
            .id_salt("ruban-courant")
            .max_height(hauteur_maximale)
            .auto_shrink([false, true])
            .stick_to_bottom(true)
            .show(ui, |ui| dessiner_lignes(ui, p, &ruban.lignes));
    });
    action
}

/// Affiche l'écran des rubans.
pub fn afficher(
    ui: &mut Ui,
    p: &Palette,
    ruban: &Ruban,
    enregistres: &[RubanEnregistre],
    ouvert: Option<&RubanEnregistre>,
) -> Option<Action> {
    let mut action = None;

    // Un ruban enregistré, ouvert en lecture seule.
    if let Some(r) = ouvert {
        if en_tete(ui, p, &r.nom, textes::RETOUR.traduit()) {
            action = Some(Action::FermerLecture);
        }
        corps(ui, "ruban-lecture", |ui| {
            ui.label(
                RichText::new(textes::avec_tiret(
                    &r.date_en_clair(),
                    textes::LECTURE_SEULE.traduit(),
                ))
                .color(p.texte_attenue),
            );
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if ui.button(textes::COPIER.traduit()).clicked() {
                    action = Some(Action::CopierEnregistre(r.id));
                }
                if ui.button(textes::EXPORTER.traduit()).clicked() {
                    action = Some(Action::ExporterEnregistre(r.id));
                }
            });
            ui.add_space(4.0);
            carte(ui, p, |ui| dessiner_lignes(ui, p, &r.lignes));
            ui.add_space(12.0);
        });
        return action;
    }

    ui.add_space(10.0);
    ui.horizontal(|ui| {
        ui.add_space(14.0);
        ui.label(
            RichText::new(textes::ONGLET_RUBANS.traduit())
                .font(theme::police_grasse(20.0))
                .color(p.entete),
        );
    });
    ui.add_space(4.0);
    ui.separator();
    corps(ui, "rubans", |ui| {
        section(ui, p, textes::RUBAN_COURANT.traduit());
        if let Some(a) = ruban_courant(ui, p, ruban, 300.0) {
            action = Some(a);
        }

        section(ui, p, textes::RUBANS_ENREGISTRES.traduit());
        if enregistres.is_empty() {
            ui.label(
                RichText::new(textes::AUCUN_RUBAN_ENREGISTRE.traduit()).color(p.texte_attenue),
            );
        }
        for r in enregistres {
            carte(ui, p, |ui| {
                ui.label(RichText::new(&r.nom).font(theme::police_grasse(15.0)));
                ui.label(
                    RichText::new(textes::avec_tiret(
                        &r.date_en_clair(),
                        &textes::nombre_de_lignes(r.lignes.len()),
                    ))
                    .size(12.5)
                    .color(p.texte_attenue),
                );
                ui.horizontal_wrapped(|ui| {
                    for (libelle, a) in [
                        (textes::OUVRIR.traduit(), Action::Ouvrir(r.id)),
                        (textes::RENOMMER.traduit(), Action::Renommer(r.id)),
                        (textes::COPIER.traduit(), Action::CopierEnregistre(r.id)),
                        (textes::EXPORTER.traduit(), Action::ExporterEnregistre(r.id)),
                        (textes::SUPPRIMER.traduit(), Action::Supprimer(r.id)),
                    ] {
                        if ui.small_button(libelle).clicked() {
                            action = Some(a);
                        }
                    }
                });
            });
            ui.add_space(2.0);
        }
        ui.add_space(12.0);
    });
    action
}
