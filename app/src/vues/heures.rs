//! Écran « Feuille de temps » (section 22.20) : l'arrivée, le départ et le
//! dîner de chaque jour de la semaine; les heures de chaque jour et le total.
//! Les heures se lisent et se comptent dans le noyau (`noyau::heures`).

use eframe::egui::{self, RichText, Ui};
use noyau::Preferences;
use noyau::format::{OptionsFormat, en_texte};
use noyau::heures::{lire_une_duree, lire_une_heure, minutes_du_jour};
use noyau::unites::Format;
use noyau::valeur::Valeur;

use super::metiers::{Action, Position};
use super::{bouton_principal, cadre_de_saisie, carte, corps, en_tete, section};
use crate::textes;
use crate::theme::{self, Palette};

/// La clé de la feuille de temps parmi les saisies des métiers (`metiers.json`).
pub const IDENTIFIANT: &str = "feuille-de-temps";

/// Lundi à dimanche, trois saisies par jour : l'arrivée, le départ, le dîner.
const JOURS: usize = 7;

/// Une semaine vide.
pub fn saisies_de_depart() -> Vec<String> {
    vec![String::new(); JOURS * 3]
}

/// Ce qu'une journée donne : rien (elle est vide ou incomplète), ses minutes
/// travaillées, ou ce qui ne va pas.
enum Journee {
    Vide,
    Minutes(f64),
    Erreur(String),
}

fn compter(jour: &str, arrivee: &str, depart: &str, diner: &str) -> Journee {
    let (arrivee, depart, diner) = (arrivee.trim(), depart.trim(), diner.trim());
    if arrivee.is_empty() || depart.is_empty() {
        return Journee::Vide;
    }
    let Some(debut) = lire_une_heure(arrivee) else {
        return Journee::Erreur(textes::heure_illisible(jour, arrivee));
    };
    let Some(fin) = lire_une_heure(depart) else {
        return Journee::Erreur(textes::heure_illisible(jour, depart));
    };
    let pause = if diner.is_empty() {
        0.0
    } else {
        match lire_une_duree(diner) {
            Some(minutes) => minutes,
            None => return Journee::Erreur(textes::heure_illisible(jour, diner)),
        }
    };
    match minutes_du_jour(debut, fin, pause) {
        Some(minutes) => Journee::Minutes(minutes),
        None => Journee::Erreur(textes::diner_trop_long(jour)),
    }
}

/// Une durée en heures et minutes (« 8 h 30 »), ou en heures décimales.
fn en_clair(minutes: f64, format: Format, prefs: &Preferences) -> String {
    en_texte(
        &Valeur::en_base(minutes, format),
        &OptionsFormat::nouvelles(prefs),
    )
}

/// Affiche la feuille de temps. `saisies` : celles de `IDENTIFIANT`.
pub fn afficher(
    ui: &mut Ui,
    p: &Palette,
    position: &mut Position,
    saisies: &mut Vec<String>,
    prefs: &Preferences,
) -> Option<Action> {
    let mut action = None;
    if en_tete(
        ui,
        p,
        textes::FEUILLE_DE_TEMPS.traduit(),
        textes::RETOUR.traduit(),
    ) {
        position.feuille_de_temps = false;
    }
    if saisies.len() != JOURS * 3 {
        *saisies = saisies_de_depart();
    }
    let mut modifie = false;
    corps(ui, "feuille-de-temps", |ui| {
        ui.add_space(6.0);
        ui.label(RichText::new(textes::FEUILLE_DE_TEMPS_RESUME.traduit()).color(p.texte_attenue));
        ui.add_space(6.0);
        let mut journees = Vec::with_capacity(JOURS);
        carte(ui, p, |ui| {
            egui::Grid::new("grille-feuille-de-temps")
                .num_columns(5)
                .spacing(egui::vec2(6.0, 6.0))
                .show(ui, |ui| {
                    ui.label("");
                    for colonne in [
                        textes::COLONNE_ARRIVEE,
                        textes::COLONNE_DEPART,
                        textes::COLONNE_DINER,
                        textes::COLONNE_HEURES,
                    ] {
                        ui.label(
                            RichText::new(colonne.traduit())
                                .font(theme::police_grasse(12.0))
                                .color(p.texte_attenue),
                        );
                    }
                    ui.end_row();
                    for jour in 0..JOURS {
                        ui.label(
                            RichText::new(textes::JOURS_COURTS[jour].traduit())
                                .font(theme::police_grasse(13.0)),
                        );
                        for (k, indice) in ["7:30", "16:00", "30"].into_iter().enumerate() {
                            // Une taille fixe : dans une grille, un champ prendrait la
                            // largeur de sa colonne à l'image d'avant, et resterait étroit.
                            let reponse = ui.add_sized(
                                egui::vec2(60.0, 26.0),
                                egui::TextEdit::singleline(&mut saisies[3 * jour + k])
                                    .char_limit(8)
                                    .hint_text(indice),
                            );
                            cadre_de_saisie(ui, p, &reponse);
                            modifie |= reponse.changed();
                        }
                        let journee = compter(
                            textes::JOURS[jour].traduit(),
                            &saisies[3 * jour],
                            &saisies[3 * jour + 1],
                            &saisies[3 * jour + 2],
                        );
                        let texte = match &journee {
                            Journee::Vide => "—".to_string(),
                            Journee::Minutes(minutes) => en_clair(*minutes, Format::HMin, prefs),
                            Journee::Erreur(_) => "?".to_string(),
                        };
                        ui.label(RichText::new(texte).font(theme::police_grasse(13.5)));
                        ui.end_row();
                        journees.push(journee);
                    }
                });
        });
        // Ce qui ne se lit pas.
        for journee in &journees {
            if let Journee::Erreur(message) = journee {
                ui.label(RichText::new(message).color(p.avertissement));
            }
        }
        let total: f64 = journees
            .iter()
            .map(|journee| match journee {
                Journee::Minutes(minutes) => *minutes,
                _ => 0.0,
            })
            .sum();
        let total_en_clair = format!(
            "{} ({})",
            en_clair(total, Format::HMin, prefs),
            en_clair(total, Format::HDec, prefs)
        );
        section(ui, p, textes::TOTAL_DE_LA_SEMAINE.traduit());
        carte(ui, p, |ui| {
            ui.label(
                RichText::new(&total_en_clair)
                    .font(theme::police_grasse(18.0))
                    .color(p.entete),
            );
        });
        ui.add_space(10.0);
        ui.horizontal_wrapped(|ui| {
            let comptees: Vec<(usize, f64)> = journees
                .iter()
                .enumerate()
                .filter_map(|(jour, journee)| match journee {
                    Journee::Minutes(minutes) => Some((jour, *minutes)),
                    _ => None,
                })
                .collect();
            let ajouter = bouton_principal(p, textes::AJOUTER_AU_RUBAN.traduit(), 14.5);
            if ui.add_enabled(!comptees.is_empty(), ajouter).clicked() {
                let mut resultats: Vec<(&'static str, String)> = comptees
                    .iter()
                    .map(|&(jour, minutes)| {
                        (
                            textes::JOURS[jour].traduit(),
                            en_clair(minutes, Format::HMin, prefs),
                        )
                    })
                    .collect();
                resultats.push((
                    textes::TOTAL_DE_LA_SEMAINE.traduit(),
                    total_en_clair.clone(),
                ));
                let entrees = comptees
                    .iter()
                    .map(|&(jour, _)| {
                        (
                            textes::JOURS[jour].traduit(),
                            textes::journee_en_clair(
                                saisies[3 * jour].trim(),
                                saisies[3 * jour + 1].trim(),
                                saisies[3 * jour + 2].trim(),
                            ),
                        )
                    })
                    .collect();
                action = Some(Action::Consigner {
                    titre: textes::FEUILLE_DE_TEMPS.traduit(),
                    entrees,
                    resultats,
                });
            }
            if ui.button(textes::VIDER_LA_SEMAINE.traduit()).clicked() {
                *saisies = saisies_de_depart();
                modifie = true;
            }
        });
        ui.add_space(14.0);
    });
    if modifie && action.is_none() {
        action = Some(Action::Modifie);
    }
    action
}
