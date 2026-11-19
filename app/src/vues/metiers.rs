//! Écran « Métiers » : les calculateurs par corps de métier (section 22).
//!
//! Trois niveaux : la liste des métiers, les calculateurs d'un métier, puis
//! le formulaire d'un calculateur. Tout le calcul est dans le noyau (R-49) :
//! cet écran ne fait que dessiner ce que le noyau décrit.

use std::collections::BTreeMap;

use eframe::egui::{self, Id, RichText, Sense, Ui};
use noyau::Preferences;
use noyau::metiers::{Calculateur, Metier, Nature, calculateur};
use noyau::unites::Format;

use super::{bouton_principal, cadre_de_saisie, carte, corps, en_tete, section};
use crate::textes;
use crate::theme::{self, Palette};

/// Ce que l'utilisateur a tapé dans chaque calculateur, par identifiant.
pub type Saisies = BTreeMap<String, Vec<String>>;

/// Où l'on se trouve dans l'écran.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Position {
    pub metier: Option<Metier>,
    /// Identifiant du calculateur ouvert.
    pub calculateur: Option<&'static str>,
}

impl Position {
    /// Remonte d'un niveau. Retourne faux si l'on était déjà à la liste des métiers.
    pub fn remonter(&mut self) -> bool {
        if self.calculateur.take().is_some() {
            true
        } else {
            self.metier.take().is_some()
        }
    }
}

/// Ce que l'utilisateur a demandé.
pub enum Action {
    /// Les saisies ont changé : elles sont à conserver.
    Modifie,
    /// Inscrire au ruban le calcul affiché.
    Consigner {
        titre: &'static str,
        entrees: Vec<(&'static str, String)>,
        resultats: Vec<(&'static str, String)>,
    },
    /// Ouvrir le clavier principal.
    Clavier,
    /// Brancher ce calculateur sur le clavier (section 23).
    AuClavier(&'static str),
}

/// Une carte cliquable : un titre, une ligne d'explication et, à droite, une mention.
pub fn carte_cliquable(
    ui: &mut Ui,
    p: &Palette,
    identifiant: impl std::hash::Hash + std::fmt::Debug,
    titre: &str,
    explication: &str,
    mention: &str,
) -> bool {
    let cadre = egui::Frame::new()
        .fill(p.carte)
        .stroke(egui::Stroke::new(1.0, p.carte_bord))
        .corner_radius(8)
        .inner_margin(egui::Margin::symmetric(12, 9))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().interact_size.y = 0.0;
            ui.spacing_mut().item_spacing.y = 3.0;
            ui.horizontal(|ui| {
                ui.label(RichText::new(titre).font(theme::police_grasse(15.5)));
                if !mention.is_empty() {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(mention).size(12.0).color(p.titre));
                    });
                }
            });
            ui.label(RichText::new(explication).size(13.0).color(p.texte_attenue));
        });
    let reponse = ui.interact(cadre.response.rect, Id::unique(identifiant), Sense::click());
    reponse.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, titre));
    if reponse.hovered() {
        ui.painter().rect_stroke(
            cadre.response.rect,
            8,
            egui::Stroke::new(1.5, p.selection),
            egui::StrokeKind::Inside,
        );
    }
    ui.add_space(2.0);
    reponse.clicked()
}

/// Titre d'un écran sans bouton.
fn titre(ui: &mut Ui, p: &Palette, texte: &str) {
    ui.add_space(10.0);
    ui.horizontal(|ui| {
        ui.add_space(14.0);
        ui.label(
            RichText::new(texte)
                .font(theme::police_grasse(20.0))
                .color(p.entete),
        );
    });
    ui.add_space(4.0);
    ui.separator();
}

fn liste_des_metiers(ui: &mut Ui, p: &Palette, position: &mut Position) -> Option<Action> {
    let mut action = None;
    titre(ui, p, textes::METIERS);
    corps(ui, "metiers", |ui| {
        ui.add_space(4.0);
        ui.label(RichText::new(textes::METIERS_INTRODUCTION).color(p.texte_attenue));
        ui.add_space(6.0);
        for metier in Metier::TOUS {
            let nombre = textes::nombre_de_calculateurs(metier.calculateurs().len());
            if carte_cliquable(
                ui,
                p,
                ("metier", metier),
                metier.nom(),
                metier.resume(),
                &nombre,
            ) {
                position.metier = Some(metier);
            }
        }
        // La charpente, la toiture et les escaliers se calculent au clavier principal.
        if carte_cliquable(
            ui,
            p,
            "metier-charpente",
            textes::METIERS_CHARPENTE,
            textes::METIERS_CHARPENTE_RESUME,
            textes::METIERS_CHARPENTE_MENTION,
        ) {
            action = Some(Action::Clavier);
        }
        ui.add_space(8.0);
        ui.label(
            RichText::new(textes::METIERS_AVERTISSEMENT)
                .size(12.5)
                .color(p.texte_attenue),
        );
        ui.add_space(12.0);
    });
    action
}

fn liste_des_calculateurs(ui: &mut Ui, p: &Palette, position: &mut Position, metier: Metier) {
    if en_tete(ui, p, metier.nom(), textes::RETOUR) {
        position.metier = None;
    }
    corps(ui, "metiers-calculateurs", |ui| {
        ui.add_space(6.0);
        for c in metier.calculateurs() {
            if carte_cliquable(ui, p, ("calculateur", c.id), c.nom, c.resume, "") {
                position.calculateur = Some(c.id);
            }
        }
        ui.add_space(12.0);
    });
}

/// Exemple de saisie montré dans un champ de longueur encore vide.
fn indice(nature: Nature) -> &'static str {
    match nature {
        Nature::Longueur(Format::PoFrac) => textes::INDICE_POUCES,
        Nature::Longueur(_) => textes::INDICE_PIEDS,
        _ => "",
    }
}

/// Dessine les champs. Retourne vrai si une saisie a changé.
fn champs(
    ui: &mut Ui,
    p: &Palette,
    c: &Calculateur,
    saisies: &mut [String],
    erreurs: &[Option<&'static str>],
) -> bool {
    let mut modifie = false;
    for (i, champ) in c.champs.iter().enumerate() {
        let Some(saisie) = saisies.get_mut(i) else {
            continue;
        };
        let libelle = ui.label(RichText::new(champ.libelle).font(theme::police_grasse(13.5)));
        match champ.nature {
            Nature::Choix(options) => {
                let avant = saisie
                    .parse::<usize>()
                    .unwrap_or(0)
                    .min(options.len().saturating_sub(1));
                let mut rang = avant;
                egui::ComboBox::from_id_salt(("metier-choix", c.id, i))
                    .selected_text(options.get(rang).copied().unwrap_or(""))
                    .width(ui.available_width().min(320.0))
                    .show_ui(ui, |ui| {
                        for (k, option) in options.iter().enumerate() {
                            ui.selectable_value(&mut rang, k, *option);
                        }
                    });
                if rang != avant {
                    *saisie = rang.to_string();
                    modifie = true;
                }
            }
            nature => {
                ui.horizontal(|ui| {
                    let unite = champ.unite();
                    // La place de l'unité, à droite du champ.
                    let reserve = if unite.is_empty() { 0.0 } else { 62.0 };
                    let largeur = (ui.available_width() - reserve).clamp(80.0, 320.0);
                    let champ_de_texte = egui::TextEdit::singleline(saisie)
                        .desired_width(largeur)
                        .hint_text(indice(nature));
                    // Le libellé nomme le champ pour les lecteurs d'écran.
                    let reponse = ui.add(champ_de_texte).labelled_by(libelle.id);
                    cadre_de_saisie(ui, p, &reponse);
                    if reponse.changed() {
                        modifie = true;
                    }
                    ui.label(RichText::new(unite).color(p.texte_attenue));
                });
            }
        }
        if let Some(Some(message)) = erreurs.get(i) {
            ui.label(RichText::new(*message).size(12.5).color(p.erreur));
        }
        ui.add_space(5.0);
    }
    modifie
}

fn formulaire(
    ui: &mut Ui,
    p: &Palette,
    position: &mut Position,
    c: &'static Calculateur,
    toutes_les_saisies: &mut Saisies,
    prefs: &Preferences,
) -> Option<Action> {
    let mut action = None;
    if en_tete(ui, p, c.nom, textes::RETOUR) {
        position.calculateur = None;
    }
    corps(ui, "metiers-formulaire", |ui| {
        ui.add_space(6.0);
        ui.label(RichText::new(c.resume).color(p.texte_attenue));
        ui.add_space(6.0);
        let saisies = toutes_les_saisies
            .entry(c.id.to_string())
            .or_insert_with(|| c.saisies_par_defaut());
        // Un calculateur dont les champs ont changé repart de ses valeurs par défaut.
        if saisies.len() != c.champs.len() {
            *saisies = c.saisies_par_defaut();
        }
        let evaluation = c.evaluer(saisies, prefs);
        let mut modifie = false;
        carte(ui, p, |ui| {
            modifie = champs(ui, p, c, saisies, &evaluation.erreurs);
        });

        if let Some(avis) = evaluation.avis {
            ui.add_space(8.0);
            ui.label(RichText::new(avis).color(p.avertissement));
        }
        if !evaluation.resultats.is_empty() {
            section(ui, p, textes::RESULTATS);
            carte(ui, p, |ui| {
                let dernier = evaluation.resultats.len() - 1;
                for (i, (libelle, valeur)) in evaluation.resultats.iter().enumerate() {
                    ui.label(RichText::new(*libelle).size(12.5).color(p.texte_attenue));
                    ui.label(
                        RichText::new(valeur)
                            .font(theme::police_grasse(17.0))
                            .color(p.entete),
                    );
                    if i != dernier {
                        ui.add_space(1.0);
                        ui.separator();
                    }
                }
            });
        }
        if !c.note.is_empty() {
            ui.add_space(8.0);
            ui.label(RichText::new(c.note).size(12.5).color(p.texte_attenue));
        }

        ui.add_space(10.0);
        ui.horizontal(|ui| {
            let pret = !evaluation.resultats.is_empty();
            let ajouter = bouton_principal(p, textes::AJOUTER_AU_RUBAN, 14.5);
            if ui.add_enabled(pret, ajouter).clicked() {
                action = Some(Action::Consigner {
                    titre: c.nom,
                    entrees: evaluation.entrees.clone(),
                    resultats: evaluation.resultats.clone(),
                });
            }
            if ui
                .button(textes::AU_CLAVIER)
                .on_hover_text(textes::INFOBULLE_AU_CLAVIER)
                .clicked()
            {
                action = Some(Action::AuClavier(c.id));
            }
            if ui.button(textes::RECOMMENCER).clicked() {
                *saisies = c.saisies_par_defaut();
                modifie = true;
            }
        });
        ui.add_space(14.0);
        if modifie && action.is_none() {
            action = Some(Action::Modifie);
        }
    });
    action
}

/// Affiche l'écran des métiers, au niveau où l'utilisateur se trouve.
pub fn afficher(
    ui: &mut Ui,
    p: &Palette,
    position: &mut Position,
    saisies: &mut Saisies,
    prefs: &Preferences,
) -> Option<Action> {
    if let Some(c) = position.calculateur.and_then(calculateur) {
        return formulaire(ui, p, position, c, saisies, prefs);
    }
    match position.metier {
        Some(metier) => {
            liste_des_calculateurs(ui, p, position, metier);
            None
        }
        None => liste_des_metiers(ui, p, position),
    }
}
