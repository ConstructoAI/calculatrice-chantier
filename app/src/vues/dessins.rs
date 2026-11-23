//! Écran « Mes dessins » : la bibliothèque des dessins (section 25). Chaque
//! dessin garde ses pièces; il s'ouvre, se renomme, s'enregistre en fichier
//! pour être rouvert sur un autre appareil, ou se supprime.

use eframe::egui::{RichText, Ui};

use super::{bouton_principal, carte, corps, en_tete};
use crate::stockage::DessinEnregistre;
use crate::textes;
use crate::theme::{self, Palette};

/// Ce que l'utilisateur a demandé dans la liste des dessins.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// Revenir au dessin ouvert.
    Retour,
    Nouveau,
    /// Rouvrir un dessin enregistré en fichier.
    OuvrirUnFichier,
    Ouvrir(u64),
    Renommer(u64),
    /// L'enregistrer en fichier.
    Enregistrer(u64),
    Supprimer(u64),
    /// En faire une copie, pour comparer deux variantes.
    Dupliquer(u64),
}

/// Affiche la liste des dessins, du plus récemment modifié au plus ancien;
/// le dessin ouvert (`courant`) est en tête.
pub fn afficher(
    ui: &mut Ui,
    p: &Palette,
    dessins: &[DessinEnregistre],
    courant: u64,
) -> Option<Action> {
    let mut action = None;
    if en_tete(
        ui,
        p,
        textes::MES_DESSINS.traduit(),
        textes::RETOUR.traduit(),
    ) {
        action = Some(Action::Retour);
    }
    corps(ui, "mes-dessins", |ui| {
        ui.horizontal_wrapped(|ui| {
            if ui
                .add(bouton_principal(p, textes::NOUVEAU_DESSIN.traduit(), 14.0))
                .clicked()
            {
                action = Some(Action::Nouveau);
            }
            if ui.button(textes::OUVRIR_UN_FICHIER.traduit()).clicked() {
                action = Some(Action::OuvrirUnFichier);
            }
        });
        ui.add_space(4.0);
        ui.label(
            RichText::new(textes::DESSINS_EXPLICATION.traduit())
                .size(12.5)
                .color(p.texte_attenue),
        );
        ui.add_space(8.0);
        let mut ordre: Vec<&DessinEnregistre> = dessins.iter().collect();
        ordre.sort_by(|a, b| {
            (b.id == courant)
                .cmp(&(a.id == courant))
                .then_with(|| b.modifie.cmp(&a.modifie))
        });
        for dessin in ordre {
            let ouvert = dessin.id == courant;
            carte(ui, p, |ui| {
                let mut nom = dessin.nom.clone();
                if ouvert {
                    nom.push_str(" · ");
                    nom.push_str(textes::DESSIN_OUVERT.traduit());
                }
                ui.label(RichText::new(nom).font(theme::police_grasse(15.0)));
                let mut details = textes::nombre_de_pieces_du_dessin(dessin.plan.pieces.len());
                if !dessin.modifie.is_empty() {
                    details = format!("{} · {details}", dessin.date_en_clair());
                }
                ui.label(RichText::new(details).size(12.5).color(p.texte_attenue));
                ui.horizontal_wrapped(|ui| {
                    if !ouvert && ui.small_button(textes::OUVRIR.traduit()).clicked() {
                        action = Some(Action::Ouvrir(dessin.id));
                    }
                    for (libelle, a) in [
                        (textes::RENOMMER.traduit(), Action::Renommer(dessin.id)),
                        (
                            textes::DESSIN_DUPLIQUER.traduit(),
                            Action::Dupliquer(dessin.id),
                        ),
                        (
                            textes::ENREGISTRER_LE_FICHIER.traduit(),
                            Action::Enregistrer(dessin.id),
                        ),
                        (textes::SUPPRIMER.traduit(), Action::Supprimer(dessin.id)),
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
