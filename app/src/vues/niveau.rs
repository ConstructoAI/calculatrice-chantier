//! Écran « Niveau » (section 22.22) : le téléphone posé sur un chevron, un
//! tuyau ou une rampe donne la pente en degrés, sur 12 et en pour cent, qu'un
//! toucher envoie à la calculatrice. Le calcul est dans le noyau
//! (`noyau::niveau`); le capteur, dans `plateforme`.

use std::time::Duration;

use eframe::egui::{self, Pos2, RichText, Sense, Stroke, Ui, Vec2};
use noyau::Preferences;
use noyau::format::{OptionsFormat, en_texte, nombre_et_unite, nombre_fixe};
use noyau::niveau::{PRESQUE_DEBOUT, en_pourcentage, inclinaison, sur_douze};
use noyau::unites::Format;
use noyau::valeur::Valeur;

use super::metiers::{Action, Position};
use super::{bouton_principal, carte, corps, en_tete};
use crate::plateforme;
use crate::textes;
use crate::theme::{self, Palette};

/// Les angles figés, gardés d'une image à l'autre.
#[derive(Clone, Copy, Default)]
struct Fige(Option<[f64; 2]>);

/// Affiche le niveau.
pub fn afficher(
    ui: &mut Ui,
    p: &Palette,
    position: &mut Position,
    prefs: &Preferences,
) -> Option<Action> {
    let mut action = None;
    if en_tete(ui, p, textes::NIVEAU.traduit(), textes::RETOUR.traduit()) {
        position.niveau = false;
    }
    let id = egui::Id::unique("niveau-fige");
    let mut fige = ui
        .ctx()
        .memory_mut(|m| *m.data.get_temp_mut_or_default::<Fige>(id));
    let lue = plateforme::inclinaison_lue();
    let angles = fige.0.or(lue);
    if fige.0.is_none() {
        // La lecture suit le capteur, une dizaine de fois par seconde.
        ui.ctx().request_repaint_after(Duration::from_millis(100));
    }
    corps(ui, "niveau", |ui| {
        ui.add_space(6.0);
        ui.label(RichText::new(textes::NIVEAU_RESUME.traduit()).color(p.texte_attenue));
        ui.add_space(8.0);
        let Some([beta, gamma]) = angles else {
            if plateforme::inclinaison_a_permettre() {
                if ui
                    .add(bouton_principal(
                        p,
                        textes::NIVEAU_AUTORISER.traduit(),
                        15.0,
                    ))
                    .clicked()
                {
                    plateforme::demander_l_inclinaison();
                }
            } else {
                ui.label(RichText::new(textes::NIVEAU_EN_ATTENTE.traduit()).color(p.texte_attenue));
            }
            return;
        };
        let etalon = prefs.etalon_niveau;
        let degres = inclinaison(beta, gamma, etalon);
        // La bulle : au centre quand l'écran est de niveau.
        let cote = ui.available_width().min(220.0);
        let (zone, _) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), cote), Sense::hover());
        let centre = zone.center();
        let rayon = cote / 2.0 - 4.0;
        let painter = ui.painter();
        painter.circle_filled(centre, rayon, p.carte);
        painter.circle_stroke(centre, rayon, Stroke::new(2.0, p.carte_bord));
        painter.circle_stroke(centre, rayon * 0.18, Stroke::new(1.5, p.texte_attenue));
        painter.line_segment(
            [
                centre - Vec2::new(rayon, 0.0),
                centre + Vec2::new(rayon, 0.0),
            ],
            Stroke::new(1.0, p.carte_bord),
        );
        painter.line_segment(
            [
                centre - Vec2::new(0.0, rayon),
                centre + Vec2::new(0.0, rayon),
            ],
            Stroke::new(1.0, p.carte_bord),
        );
        // La bulle monte du côté haut : à l'opposé de la pente, comme un vrai
        // niveau; 10° la mènent au bord.
        let echelle = rayon / 10.0;
        let mut ecart = Vec2::new(
            -((gamma - etalon[1]) as f32) * echelle,
            -((beta - etalon[0]) as f32) * echelle,
        );
        if ecart.length() > rayon * 0.85 {
            ecart = ecart.normalized() * rayon * 0.85;
        }
        let de_niveau = degres < 0.1;
        let couleur = if de_niveau {
            p.touche_fonction.fond
        } else {
            p.accent_du_plan()
        };
        painter.circle_filled(
            Pos2::new(centre.x + ecart.x, centre.y + ecart.y),
            rayon * 0.14,
            couleur,
        );

        // Les trois lectures; un toucher en envoie une au clavier.
        let o = OptionsFormat::nouvelles(prefs);
        let mut lectures = vec![(
            en_texte(&Valeur::avec_format(degres, Format::Degres), &o),
            Valeur::avec_format(degres, Format::Degres),
        )];
        if degres < PRESQUE_DEBOUT {
            // À l'écran, deux décimales sur 12 et une en pour cent; la valeur
            // envoyée garde toute sa précision.
            let douze = sur_douze(degres);
            lectures.push((
                textes::niveau_sur_douze(&nombre_fixe(douze, 2, prefs)),
                Valeur::nombre(douze),
            ));
            let pourcent = en_pourcentage(degres);
            lectures.push((
                nombre_et_unite(&nombre_fixe(pourcent, 1, prefs), "%"),
                Valeur::avec_format(pourcent, Format::Pourcent),
            ));
        }
        ui.add_space(8.0);
        carte(ui, p, |ui| {
            if de_niveau {
                ui.label(
                    RichText::new(textes::NIVEAU_DE_NIVEAU.traduit())
                        .font(theme::police_grasse(14.0))
                        .color(p.titre),
                );
            }
            for (texte, valeur) in lectures {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(&texte)
                            .font(theme::police_grasse(22.0))
                            .color(p.entete),
                    );
                    if ui
                        .button(textes::NIVEAU_ENVOYER.traduit())
                        .on_hover_text(textes::INFOBULLE_NIVEAU_ENVOYER.traduit())
                        .clicked()
                    {
                        action = Some(Action::ValeurAuClavier(valeur));
                    }
                });
            }
        });
        ui.add_space(10.0);
        ui.horizontal_wrapped(|ui| {
            let libelle = if fige.0.is_some() {
                textes::NIVEAU_REPRENDRE
            } else {
                textes::NIVEAU_FIGER
            };
            if ui.button(libelle.traduit()).clicked() {
                fige.0 = if fige.0.is_some() { None } else { lue };
            }
            if ui
                .button(textes::NIVEAU_ETALONNER.traduit())
                .on_hover_text(textes::INFOBULLE_NIVEAU_ETALONNER.traduit())
                .clicked()
                && let Some(angles) = lue
            {
                action = Some(Action::Etalonner(angles));
            }
            if etalon != [0.0; 2]
                && ui
                    .button(textes::NIVEAU_SANS_ETALONNAGE.traduit())
                    .clicked()
            {
                action = Some(Action::Etalonner([0.0; 2]));
            }
        });
        ui.add_space(6.0);
        ui.label(
            RichText::new(textes::NIVEAU_AIDE.traduit())
                .size(12.5)
                .color(p.texte_attenue),
        );
        ui.add_space(14.0);
    });
    ui.ctx()
        .memory_mut(|m| *m.data.get_temp_mut_or_default::<Fige>(id) = fige);
    action
}
