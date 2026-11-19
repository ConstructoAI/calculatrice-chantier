//! Écran « Préférences » (section 13).

use eframe::egui::{self, Align, Layout, RichText, Ui};
use noyau::analyse::{analyser_aire, analyser_longueur, analyser_nombre};
use noyau::format::{OptionsFormat, en_texte, nombre_flottant};
use noyau::prefs::{
    AffichageAires, AffichageVolumes, EmpannonsIrreguliers, ModeFractions, MurCintre, Ordre,
    Preferences, RESOLUTIONS, SeparateurDecimal, Theme, ValeursMemorisees,
};
use noyau::unites::{Format, UniteMasseVolumique};
use noyau::{Moteur, Valeur};

use super::{carte, corps, en_tete, section};
use crate::textes;
use crate::theme::{self, Palette};

/// Valeur mémorisée modifiable (section 13.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Memorisee {
    Echappee,
    Contremarche,
    Plancher,
    Giron,
    AireBloc,
    LongueurBloc,
    AngleMoulure,
    Entraxe,
    SectionSemelle,
    MasseVolumique,
    Paquets,
}

impl Memorisee {
    pub const TOUTES: [Memorisee; 11] = [
        Memorisee::Echappee,
        Memorisee::Contremarche,
        Memorisee::Plancher,
        Memorisee::Giron,
        Memorisee::AireBloc,
        Memorisee::LongueurBloc,
        Memorisee::AngleMoulure,
        Memorisee::Entraxe,
        Memorisee::SectionSemelle,
        Memorisee::MasseVolumique,
        Memorisee::Paquets,
    ];

    pub fn nom(self) -> &'static str {
        match self {
            Memorisee::Echappee => textes::MEMO_ECHAPPEE,
            Memorisee::Contremarche => textes::MEMO_CONTREMARCHE,
            Memorisee::Plancher => textes::MEMO_PLANCHER,
            Memorisee::Giron => textes::MEMO_GIRON,
            Memorisee::AireBloc => textes::MEMO_AIRE_BLOC,
            Memorisee::LongueurBloc => textes::MEMO_LONGUEUR_BLOC,
            Memorisee::AngleMoulure => textes::MEMO_ANGLE_MOULURE,
            Memorisee::Entraxe => textes::MEMO_ENTRAXE,
            Memorisee::SectionSemelle => textes::MEMO_SECTION_SEMELLE,
            Memorisee::MasseVolumique => textes::MEMO_MASSE_VOLUMIQUE,
            Memorisee::Paquets => textes::MEMO_PAQUETS,
        }
    }

    fn usage(self) -> &'static str {
        match self {
            Memorisee::Echappee => textes::MEMO_USAGE_ECHAPPEE,
            Memorisee::Contremarche => textes::MEMO_USAGE_CONTREMARCHE,
            Memorisee::Plancher => textes::MEMO_USAGE_PLANCHER,
            Memorisee::Giron => textes::MEMO_USAGE_GIRON,
            Memorisee::AireBloc => textes::MEMO_USAGE_AIRE_BLOC,
            Memorisee::LongueurBloc => textes::MEMO_USAGE_LONGUEUR_BLOC,
            Memorisee::AngleMoulure => textes::MEMO_USAGE_ANGLE_MOULURE,
            Memorisee::Entraxe => textes::MEMO_USAGE_ENTRAXE,
            Memorisee::SectionSemelle => textes::MEMO_USAGE_SECTION_SEMELLE,
            Memorisee::MasseVolumique => textes::MEMO_USAGE_MASSE_VOLUMIQUE,
            Memorisee::Paquets => textes::MEMO_USAGE_PAQUETS,
        }
    }

    /// Indication montrée sous le champ de saisie.
    pub fn exemples(self) -> &'static str {
        match self {
            Memorisee::AireBloc | Memorisee::SectionSemelle => textes::EXEMPLES_AIRE,
            Memorisee::AngleMoulure => textes::EXEMPLES_ANGLE,
            Memorisee::MasseVolumique | Memorisee::Paquets => textes::EXEMPLES_NOMBRE,
            _ => textes::EXEMPLES_LONGUEUR,
        }
    }

    /// Texte de la valeur, tel qu'on le montre et qu'on le propose à la modification.
    pub fn texte(self, memo: &ValeursMemorisees, prefs: &Preferences) -> String {
        let o = OptionsFormat::nouvelles(prefs);
        let brut = match self {
            Memorisee::Echappee => en_texte(&memo.echappee, &o),
            Memorisee::Contremarche => en_texte(&memo.contremarche_visee, &o),
            Memorisee::Plancher => en_texte(&memo.epaisseur_plancher, &o),
            Memorisee::Giron => en_texte(&memo.giron_vise, &o),
            Memorisee::AireBloc => en_texte(&memo.aire_bloc, &o),
            Memorisee::LongueurBloc => en_texte(&memo.longueur_bloc, &o),
            Memorisee::AngleMoulure => en_texte(&Valeur::degres(memo.angle_moulure), &o),
            Memorisee::Entraxe => en_texte(&memo.entraxe, &o),
            Memorisee::SectionSemelle => en_texte(&memo.section_semelle, &o),
            Memorisee::MasseVolumique => nombre_flottant(memo.masse_volumique.valeur, prefs),
            Memorisee::Paquets => nombre_flottant(memo.paquets_par_carre, prefs),
        };
        brut.replace('\u{00A0}', " ")
    }

    /// Analyse le texte saisi et enregistre la valeur. Retourne faux si le texte
    /// n'est pas reconnu : rien n'est alors modifié.
    pub fn enregistrer(self, moteur: &mut Moteur, texte: &str, unite: UniteMasseVolumique) -> bool {
        let longueur = |strictement_positive: bool| -> Option<Valeur> {
            analyser_longueur(texte, Format::PoFrac).ok().filter(|v| {
                if strictement_positive {
                    v.base > 0.0
                } else {
                    v.base >= 0.0
                }
            })
        };
        let aire = || {
            analyser_aire(texte, Format::Po2)
                .ok()
                .filter(|v| v.base > 0.0)
        };
        let nombre = || analyser_nombre(texte).ok().filter(|x| *x > 0.0);
        // On valide d'abord : l'accès en écriture met fin au cycle en cours.
        match self {
            Memorisee::Echappee => longueur(true).map(|v| moteur.memorises_mut().echappee = v),
            Memorisee::Contremarche => {
                longueur(true).map(|v| moteur.memorises_mut().contremarche_visee = v)
            }
            Memorisee::Plancher => {
                longueur(false).map(|v| moteur.memorises_mut().epaisseur_plancher = v)
            }
            Memorisee::Giron => longueur(true).map(|v| moteur.memorises_mut().giron_vise = v),
            Memorisee::LongueurBloc => {
                longueur(true).map(|v| moteur.memorises_mut().longueur_bloc = v)
            }
            Memorisee::Entraxe => longueur(true).map(|v| moteur.memorises_mut().entraxe = v),
            Memorisee::AireBloc => aire().map(|v| moteur.memorises_mut().aire_bloc = v),
            Memorisee::SectionSemelle => aire().map(|v| moteur.memorises_mut().section_semelle = v),
            Memorisee::AngleMoulure => nombre()
                .filter(|a| *a < 90.0)
                .map(|a| moteur.memorises_mut().angle_moulure = a),
            Memorisee::MasseVolumique => nombre().map(|x| {
                let memo = moteur.memorises_mut();
                memo.masse_volumique.valeur = x;
                memo.masse_volumique.unite = unite;
            }),
            Memorisee::Paquets => nombre().map(|x| moteur.memorises_mut().paquets_par_carre = x),
        }
        .is_some()
    }
}

/// Ce que l'utilisateur a demandé dans les préférences.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Terminer,
    Modifier(Memorisee),
    Reinitialiser,
}

/// Case à cocher suivie de son explication.
fn bascule(ui: &mut Ui, p: &Palette, libelle: &str, aide: &str, valeur: &mut bool) {
    ui.checkbox(valeur, libelle);
    if !aide.is_empty() {
        ui.label(RichText::new(aide).size(12.5).color(p.texte_attenue));
    }
    ui.add_space(3.0);
}

/// Liste déroulante sous son libellé.
fn choix<T: Copy + PartialEq>(
    ui: &mut Ui,
    libelle: &str,
    valeur: &mut T,
    options: &[T],
    texte: impl Fn(T) -> String,
) {
    ui.label(libelle);
    egui::ComboBox::from_id_salt(libelle)
        .selected_text(texte(*valeur))
        .width(ui.available_width().min(320.0))
        .show_ui(ui, |ui| {
            for option in options {
                ui.selectable_value(valeur, *option, texte(*option));
            }
        });
    ui.add_space(3.0);
}

/// Affiche les préférences. Les changements s'appliquent immédiatement.
pub fn afficher(ui: &mut Ui, p: &Palette, moteur: &mut Moteur) -> Option<Action> {
    let mut action = None;
    if en_tete(ui, p, textes::PREFERENCES, textes::TERMINE) {
        action = Some(Action::Terminer);
    }
    let mut prefs = moteur.preferences().clone();
    corps(ui, "preferences", |ui| {
        section(ui, p, textes::SECTION_GENERAL);
        carte(ui, p, |ui| {
            bascule(
                ui,
                p,
                textes::PREF_TRIGO,
                textes::PREF_TRIGO_AIDE,
                &mut prefs.mode_trigo,
            );
            bascule(
                ui,
                p,
                textes::PREF_FRACTIONS_RAPIDES,
                textes::PREF_FRACTIONS_RAPIDES_AIDE,
                &mut prefs.fractions_rapides,
            );
            bascule(
                ui,
                p,
                textes::PREF_PLEIN_ECRAN,
                textes::PREF_PLEIN_ECRAN_AIDE,
                &mut prefs.plein_ecran,
            );
            choix(
                ui,
                textes::PREF_THEME,
                &mut prefs.theme,
                &[Theme::Automatique, Theme::Clair, Theme::Sombre],
                |t| textes::theme(t).to_string(),
            );
        });

        section(ui, p, textes::SECTION_AFFICHAGE);
        carte(ui, p, |ui| {
            choix(
                ui,
                textes::PREF_RESOLUTION,
                &mut prefs.resolution,
                &RESOLUTIONS,
                textes::resolution,
            );
            choix(
                ui,
                textes::PREF_MODE_FRACTIONS,
                &mut prefs.mode_fractions,
                &[ModeFractions::Standard, ModeFractions::Constant],
                |m| textes::mode_fractions(m).to_string(),
            );
            bascule(
                ui,
                p,
                textes::PREF_EXPONENTIELLE,
                textes::PREF_EXPONENTIELLE_AIDE,
                &mut prefs.exponentielle,
            );
            choix(
                ui,
                textes::PREF_AIRES,
                &mut prefs.affichage_aires,
                &[
                    AffichageAires::Standard,
                    AffichageAires::PiedsCarres,
                    AffichageAires::VergesCarrees,
                    AffichageAires::MetresCarres,
                ],
                |a| textes::affichage_aires(a).to_string(),
            );
            choix(
                ui,
                textes::PREF_VOLUMES,
                &mut prefs.affichage_volumes,
                &[
                    AffichageVolumes::Standard,
                    AffichageVolumes::VergesCubes,
                    AffichageVolumes::PiedsCubes,
                    AffichageVolumes::MetresCubes,
                ],
                |v| textes::affichage_volumes(v).to_string(),
            );
            let fixe_ou_flottant = |fixe: &'static str| {
                move |b: bool| {
                    if b {
                        fixe.to_string()
                    } else {
                        textes::FLOTTANT.to_string()
                    }
                }
            };
            choix(
                ui,
                textes::PREF_METRES,
                &mut prefs.metres_fixes,
                &[true, false],
                fixe_ou_flottant(textes::METRES_FIXES),
            );
            choix(
                ui,
                textes::PREF_DEGRES,
                &mut prefs.degres_fixes,
                &[true, false],
                fixe_ou_flottant(textes::DEGRES_FIXES),
            );
            bascule(
                ui,
                p,
                textes::PREF_MILLIERS,
                textes::PREF_MILLIERS_AIDE,
                &mut prefs.separateur_milliers,
            );
            choix(
                ui,
                textes::PREF_DECIMAL,
                &mut prefs.separateur_decimal,
                &[SeparateurDecimal::Virgule, SeparateurDecimal::Point],
                |s| textes::separateur_decimal(s).to_string(),
            );
            choix(
                ui,
                textes::PREF_DECIMALES,
                &mut prefs.decimales_sans_unite,
                &[None, Some(0), Some(1), Some(2), Some(3), Some(4)],
                textes::decimales,
            );
        });

        section(ui, p, textes::SECTION_FONCTIONS);
        carte(ui, p, |ui| {
            let ordres = [Ordre::Decroissant, Ordre::Croissant];
            choix(
                ui,
                textes::PREF_MUR_PENTE,
                &mut prefs.mur_pente,
                &ordres,
                |o| textes::ordre(o).to_string(),
            );
            choix(
                ui,
                textes::PREF_MUR_CINTRE,
                &mut prefs.mur_cintre,
                &[MurCintre::Exterieur, MurCintre::Interieur],
                |m| textes::mur_cintre(m).to_string(),
            );
            choix(
                ui,
                textes::PREF_EMPANNONS,
                &mut prefs.empannons,
                &ordres,
                |o| textes::ordre(o).to_string(),
            );
            choix(
                ui,
                textes::PREF_EMPANNONS_IRREGULIERS,
                &mut prefs.empannons_irreguliers,
                &[
                    EmpannonsIrreguliers::EntraxeConstant,
                    EmpannonsIrreguliers::Apparies,
                ],
                |e| textes::empannons_irreguliers(e).to_string(),
            );
        });

        section(ui, p, textes::SECTION_MEMORISEES);
        carte(ui, p, |ui| {
            for (i, quoi) in Memorisee::TOUTES.iter().copied().enumerate() {
                if i > 0 {
                    ui.separator();
                }
                let mut texte = quoi.texte(moteur.memorises(), moteur.preferences());
                if quoi == Memorisee::MasseVolumique {
                    texte.push(' ');
                    texte.push_str(textes::unite_masse_volumique(
                        moteur.memorises().masse_volumique.unite,
                    ));
                }
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.set_max_width((ui.available_width() - 150.0).max(120.0));
                        ui.label(RichText::new(quoi.nom()).font(theme::police_grasse(14.5)));
                        ui.label(
                            RichText::new(quoi.usage())
                                .size(12.0)
                                .color(p.texte_attenue),
                        );
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button(textes::MODIFIER).clicked() {
                            action = Some(Action::Modifier(quoi));
                        }
                        ui.label(RichText::new(texte).font(theme::police_grasse(14.5)));
                    });
                });
            }
        });

        section(ui, p, textes::SECTION_REINITIALISATION);
        let bouton =
            egui::Button::new(RichText::new(textes::BOUTON_REINITIALISER_TOUT).color(p.erreur));
        if ui.add(bouton).clicked() {
            action = Some(Action::Reinitialiser);
        }
        ui.add_space(16.0);
    });
    if prefs != *moteur.preferences() {
        *moteur.preferences_mut() = prefs;
    }
    action
}
