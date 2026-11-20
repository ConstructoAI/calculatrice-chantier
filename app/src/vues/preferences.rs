//! Écran « Préférences » (section 13).

use eframe::egui::{self, Align, Layout, RichText, Ui};
use noyau::analyse::{analyser_aire, analyser_longueur, analyser_nombre};
use noyau::format::{OptionsFormat, en_texte, nombre_flottant};
use noyau::langue::Langue;
use noyau::prefs::{
    AffichageAires, AffichageVolumes, EmpannonsIrreguliers, ModeFractions, MurCintre, Ordre,
    Preferences, RESOLUTIONS, SeparateurDecimal, Theme, ValeursMemorisees,
};
use noyau::unites::{Format, UniteMasseVolumique};
use noyau::{Moteur, Valeur};

use super::{carte, corps, en_tete, section};
use crate::plateforme;
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
            Memorisee::Echappee => textes::MEMO_ECHAPPEE.traduit(),
            Memorisee::Contremarche => textes::MEMO_CONTREMARCHE.traduit(),
            Memorisee::Plancher => textes::MEMO_PLANCHER.traduit(),
            Memorisee::Giron => textes::MEMO_GIRON.traduit(),
            Memorisee::AireBloc => textes::MEMO_AIRE_BLOC.traduit(),
            Memorisee::LongueurBloc => textes::MEMO_LONGUEUR_BLOC.traduit(),
            Memorisee::AngleMoulure => textes::MEMO_ANGLE_MOULURE.traduit(),
            Memorisee::Entraxe => textes::MEMO_ENTRAXE.traduit(),
            Memorisee::SectionSemelle => textes::MEMO_SECTION_SEMELLE.traduit(),
            Memorisee::MasseVolumique => textes::MEMO_MASSE_VOLUMIQUE.traduit(),
            Memorisee::Paquets => textes::MEMO_PAQUETS.traduit(),
        }
    }

    fn usage(self) -> &'static str {
        match self {
            Memorisee::Echappee => textes::MEMO_USAGE_ECHAPPEE.traduit(),
            Memorisee::Contremarche => textes::MEMO_USAGE_CONTREMARCHE.traduit(),
            Memorisee::Plancher => textes::MEMO_USAGE_PLANCHER.traduit(),
            Memorisee::Giron => textes::MEMO_USAGE_GIRON.traduit(),
            Memorisee::AireBloc => textes::MEMO_USAGE_AIRE_BLOC.traduit(),
            Memorisee::LongueurBloc => textes::MEMO_USAGE_LONGUEUR_BLOC.traduit(),
            Memorisee::AngleMoulure => textes::MEMO_USAGE_ANGLE_MOULURE.traduit(),
            Memorisee::Entraxe => textes::MEMO_USAGE_ENTRAXE.traduit(),
            Memorisee::SectionSemelle => textes::MEMO_USAGE_SECTION_SEMELLE.traduit(),
            Memorisee::MasseVolumique => textes::MEMO_USAGE_MASSE_VOLUMIQUE.traduit(),
            Memorisee::Paquets => textes::MEMO_USAGE_PAQUETS.traduit(),
        }
    }

    /// Indication montrée sous le champ de saisie.
    pub fn exemples(self) -> &'static str {
        match self {
            Memorisee::AireBloc | Memorisee::SectionSemelle => textes::EXEMPLES_AIRE.traduit(),
            Memorisee::AngleMoulure => textes::EXEMPLES_ANGLE.traduit(),
            Memorisee::MasseVolumique | Memorisee::Paquets => textes::EXEMPLES_NOMBRE.traduit(),
            _ => textes::EXEMPLES_LONGUEUR.traduit(),
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
    /// Changer la langue de l'interface (section 24).
    Langue(Langue),
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
    if en_tete(
        ui,
        p,
        textes::PREFERENCES.traduit(),
        textes::TERMINE.traduit(),
    ) {
        action = Some(Action::Terminer);
    }
    let mut prefs = moteur.preferences().clone();
    corps(ui, "preferences", |ui| {
        section(ui, p, textes::SECTION_GENERAL.traduit());
        carte(ui, p, |ui| {
            // La langue ne se change pas ici même : l'application récrit
            // d'abord les saisies comme elles se tapent dans la nouvelle langue.
            let mut langue = prefs.langue;
            choix(
                ui,
                textes::LANGUE.traduit(),
                &mut langue,
                &Langue::TOUTES,
                |l| l.nom().to_string(),
            );
            if langue != prefs.langue {
                action = Some(Action::Langue(langue));
            }
            bascule(
                ui,
                p,
                textes::PREF_TRIGO.traduit(),
                textes::PREF_TRIGO_AIDE.traduit(),
                &mut prefs.mode_trigo,
            );
            bascule(
                ui,
                p,
                textes::PREF_FRACTIONS_RAPIDES.traduit(),
                textes::PREF_FRACTIONS_RAPIDES_AIDE.traduit(),
                &mut prefs.fractions_rapides,
            );
            // Dans un navigateur, le plein écran se demande par le bouton de la
            // barre supérieure : il ne peut pas être rétabli au démarrage.
            if !plateforme::NAVIGATEUR {
                bascule(
                    ui,
                    p,
                    textes::PREF_PLEIN_ECRAN.traduit(),
                    textes::PREF_PLEIN_ECRAN_AIDE.traduit(),
                    &mut prefs.plein_ecran,
                );
            }
            choix(
                ui,
                textes::PREF_THEME.traduit(),
                &mut prefs.theme,
                &[Theme::Automatique, Theme::Clair, Theme::Sombre],
                |t| textes::theme(t).to_string(),
            );
        });

        section(ui, p, textes::SECTION_AFFICHAGE.traduit());
        carte(ui, p, |ui| {
            choix(
                ui,
                textes::PREF_RESOLUTION.traduit(),
                &mut prefs.resolution,
                &RESOLUTIONS,
                textes::resolution,
            );
            choix(
                ui,
                textes::PREF_MODE_FRACTIONS.traduit(),
                &mut prefs.mode_fractions,
                &[ModeFractions::Standard, ModeFractions::Constant],
                |m| textes::mode_fractions(m).to_string(),
            );
            bascule(
                ui,
                p,
                textes::PREF_EXPONENTIELLE.traduit(),
                textes::PREF_EXPONENTIELLE_AIDE.traduit(),
                &mut prefs.exponentielle,
            );
            choix(
                ui,
                textes::PREF_AIRES.traduit(),
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
                textes::PREF_VOLUMES.traduit(),
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
                        textes::FLOTTANT.traduit().to_string()
                    }
                }
            };
            choix(
                ui,
                textes::PREF_METRES.traduit(),
                &mut prefs.metres_fixes,
                &[true, false],
                fixe_ou_flottant(textes::METRES_FIXES.traduit()),
            );
            choix(
                ui,
                textes::PREF_DEGRES.traduit(),
                &mut prefs.degres_fixes,
                &[true, false],
                fixe_ou_flottant(textes::DEGRES_FIXES.traduit()),
            );
            bascule(
                ui,
                p,
                textes::PREF_MILLIERS.traduit(),
                textes::PREF_MILLIERS_AIDE.traduit(),
                &mut prefs.separateur_milliers,
            );
            // En anglais, le séparateur décimal est toujours le point.
            if prefs.langue == Langue::Francais {
                choix(
                    ui,
                    textes::PREF_DECIMAL.traduit(),
                    &mut prefs.separateur_decimal,
                    &[SeparateurDecimal::Virgule, SeparateurDecimal::Point],
                    |s| textes::separateur_decimal(s).to_string(),
                );
            }
            choix(
                ui,
                textes::PREF_DECIMALES.traduit(),
                &mut prefs.decimales_sans_unite,
                &[None, Some(0), Some(1), Some(2), Some(3), Some(4)],
                textes::decimales,
            );
        });

        section(ui, p, textes::SECTION_FONCTIONS.traduit());
        carte(ui, p, |ui| {
            let ordres = [Ordre::Decroissant, Ordre::Croissant];
            choix(
                ui,
                textes::PREF_MUR_PENTE.traduit(),
                &mut prefs.mur_pente,
                &ordres,
                |o| textes::ordre(o).to_string(),
            );
            choix(
                ui,
                textes::PREF_MUR_CINTRE.traduit(),
                &mut prefs.mur_cintre,
                &[MurCintre::Exterieur, MurCintre::Interieur],
                |m| textes::mur_cintre(m).to_string(),
            );
            choix(
                ui,
                textes::PREF_EMPANNONS.traduit(),
                &mut prefs.empannons,
                &ordres,
                |o| textes::ordre(o).to_string(),
            );
            choix(
                ui,
                textes::PREF_EMPANNONS_IRREGULIERS.traduit(),
                &mut prefs.empannons_irreguliers,
                &[
                    EmpannonsIrreguliers::EntraxeConstant,
                    EmpannonsIrreguliers::Apparies,
                ],
                |e| textes::empannons_irreguliers(e).to_string(),
            );
        });

        section(ui, p, textes::SECTION_MEMORISEES.traduit());
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
                        if ui.button(textes::MODIFIER.traduit()).clicked() {
                            action = Some(Action::Modifier(quoi));
                        }
                        ui.label(RichText::new(texte).font(theme::police_grasse(14.5)));
                    });
                });
            }
        });

        section(ui, p, textes::SECTION_REINITIALISATION.traduit());
        let bouton = egui::Button::new(
            RichText::new(textes::BOUTON_REINITIALISER_TOUT.traduit()).color(p.erreur),
        );
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
