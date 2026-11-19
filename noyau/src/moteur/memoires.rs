//! Mémoire cumulative, `Stocker` et `Rappel` (section 11).

use super::{Effet, Moteur, PrefixeInterne, Presentation, note_fractions};
use crate::erreurs::Erreur;
use crate::etiquettes::{Etiquette, Note};
use crate::fonctions::arc::ArcSaisi;
use crate::fonctions::onglet::controler_angle_moulure;
use crate::fonctions::triangle::{Cote, valeur_pente};
use crate::fonctions::{Cycle, Sortie};
use crate::format::Rendu;
use crate::touches::Touche;
use crate::unites::{Format, Genre};
use crate::valeur::Valeur;

fn longueur_positive(v: &Valeur) -> Result<Valeur, Erreur> {
    if v.genre != Genre::Longueur {
        Err(Erreur::Unites)
    } else if v.base > 0.0 {
        Ok(*v)
    } else {
        Err(Erreur::Saisie)
    }
}

fn aire_positive(v: &Valeur) -> Result<Valeur, Erreur> {
    if v.genre != Genre::Aire {
        Err(Erreur::Unites)
    } else if v.base > 0.0 {
        Ok(*v)
    } else {
        Err(Erreur::Saisie)
    }
}

impl Moteur {
    /// `M+` et `M−` (section 11.1).
    pub(super) fn m_plus(&mut self, soustraire: bool) -> Result<(), Erreur> {
        // Juste après `Rappel` `M+`, la touche fait défiler moyenne et nombre (R-40).
        if !soustraire
            && self
                .cycle
                .as_ref()
                .is_some_and(|c| c.touche == Touche::MPlus)
        {
            return self.avancer_cycle();
        }
        if matches!(
            self.provenance,
            super::Provenance::Zero | super::Provenance::Message
        ) {
            return Ok(());
        }
        let v = self.valeur_montree()?;
        let texte = self.texte_montre();
        self.memoire.ajouter(&v, soustraire)?;
        self.revision += 1;
        let etiquette = if soustraire {
            Etiquette::MMoins
        } else {
            Etiquette::MPlus
        };
        self.ruban.memoire(etiquette.texte(), texte);
        let presentation = self.presentation;
        self.cycle = None;
        self.montrer_sortie(&Sortie::nouvelle(etiquette, v));
        // La valeur ajoutée garde son rendu (entier, deux décimales…).
        self.presentation.rendu = presentation.rendu;
        self.presentation.masse_volumique = presentation.masse_volumique;
        Ok(())
    }

    /// `Stocker` + touche (section 11.2).
    pub(super) fn touche_stocker(&mut self, touche: Touche) -> Result<(), Erreur> {
        match touche {
            Touche::Stocker => return Ok(()),
            Touche::Effacer => {
                self.effacer(false);
                self.dernier_effacer = true;
                return Ok(());
            }
            _ => {}
        }
        let v = self.valeur_montree()?;
        let mut masse_volumique = None;
        let (etiquette, valeur) = match touche {
            Touche::Chiffre(n @ 1..=3) => {
                // `0` `Stocker` `1` vide le registre.
                if let Some(registre) = self.registres.get_mut(usize::from(n - 1)) {
                    *registre = (v.base != 0.0).then_some(v);
                }
                (Etiquette::Registre(n), v)
            }
            Touche::Chiffre(4) => match v.genre {
                Genre::Aire => {
                    self.memo.aire_bloc = aire_positive(&v)?;
                    (Etiquette::AireBloc, v)
                }
                Genre::Longueur => {
                    self.memo.longueur_bloc = longueur_positive(&v)?;
                    (Etiquette::LongueurBloc, v)
                }
                _ => return Err(Erreur::Unites),
            },
            Touche::Chiffre(5) => {
                self.memo.entraxe = longueur_positive(&v)?;
                (Etiquette::Entraxe, v)
            }
            Touche::Chiffre(6) => {
                self.memo.section_semelle = aire_positive(&v)?;
                (Etiquette::SectionSemelle, v)
            }
            Touche::Chiffre(7) => {
                self.memo.contremarche_visee = longueur_positive(&v)?;
                (Etiquette::ContremarcheVisee, v)
            }
            Touche::Chiffre(8) => {
                if v.genre != Genre::Longueur {
                    return Err(Erreur::Unites);
                }
                if v.base < 0.0 {
                    return Err(Erreur::Saisie);
                }
                self.memo.epaisseur_plancher = v;
                (Etiquette::EpaisseurPlancher, v)
            }
            Touche::Chiffre(9) => {
                self.memo.giron_vise = longueur_positive(&v)?;
                (Etiquette::GironVise, v)
            }
            Touche::Chiffre(0) => {
                if !v.est_sans_unite() {
                    return Err(Erreur::Unites);
                }
                if !(v.base.is_finite() && v.base > 0.0) {
                    return Err(Erreur::Saisie);
                }
                self.memo.masse_volumique.valeur = v.base;
                masse_volumique = Some(self.memo.masse_volumique.unite);
                (Etiquette::MasseVolumique, Valeur::nombre(v.base))
            }
            Touche::OngletCompose => {
                let angle = controler_angle_moulure(&v)?;
                self.memo.angle_moulure = angle;
                (Etiquette::AngleMoulure, Valeur::degres(angle))
            }
            _ => return Err(Erreur::Saisie),
        };
        self.revision += 1;
        self.cycle = None;
        self.montrer_sortie(&Sortie::nouvelle(etiquette, valeur).memorisee());
        self.presentation.masse_volumique = masse_volumique;
        Ok(())
    }

    /// `Rappel` + touche (section 11.3).
    pub(super) fn touche_rappel(
        &mut self,
        touche: Touche,
        effets: &mut Vec<Effet>,
    ) -> Result<(), Erreur> {
        match touche {
            Touche::Conv => {
                self.prefixe = PrefixeInterne::RappelConv;
                return Ok(());
            }
            Touche::Effacer => {
                self.effacer(false);
                self.dernier_effacer = true;
                return Ok(());
            }
            Touche::Egal => {
                effets.push(Effet::BasculerRuban);
                return Ok(());
            }
            _ => {}
        }
        let memo = &self.memo;
        let mut masse_volumique = None;
        let mut note = None;
        let (etiquette, valeur, memorise) = match touche {
            // `Rappel` `Rappel` : montre le total et vide la mémoire.
            Touche::Rappel => {
                let total = self.memoire.total().ok_or(Erreur::AucuneValeur)?;
                (Etiquette::MPlus, total, false)
            }
            Touche::MPlus => {
                let total = self.memoire.total().ok_or(Erreur::AucuneValeur)?;
                (Etiquette::TotalM, total, false)
            }
            Touche::Chiffre(n @ 1..=3) => {
                let registre = self.registres.get(usize::from(n - 1)).copied().flatten();
                (
                    Etiquette::Registre(n),
                    registre.ok_or(Erreur::AucuneValeur)?,
                    false,
                )
            }
            Touche::Chiffre(4) => {
                // Une seconde fois de suite : la longueur d'un bloc, en alternance.
                let alterner = self.rappel_de == Some(Touche::Chiffre(4))
                    && self.presentation.etiquette == Some(Etiquette::AireBloc);
                if alterner {
                    (Etiquette::LongueurBloc, memo.longueur_bloc, true)
                } else {
                    (Etiquette::AireBloc, memo.aire_bloc, true)
                }
            }
            Touche::Chiffre(5) => (Etiquette::Entraxe, memo.entraxe, true),
            Touche::Chiffre(6) => (Etiquette::SectionSemelle, memo.section_semelle, true),
            Touche::Chiffre(7) => (Etiquette::ContremarcheVisee, memo.contremarche_visee, true),
            Touche::Chiffre(8) => (Etiquette::EpaisseurPlancher, memo.epaisseur_plancher, true),
            Touche::Chiffre(9) => (Etiquette::GironVise, memo.giron_vise, true),
            Touche::Chiffre(0) => {
                masse_volumique = Some(memo.masse_volumique.unite);
                (
                    Etiquette::MasseVolumique,
                    Valeur::nombre(memo.masse_volumique.valeur),
                    true,
                )
            }
            Touche::OngletCompose => (
                Etiquette::AngleMoulure,
                Valeur::degres(memo.angle_moulure),
                true,
            ),
            Touche::Pente => {
                let (ratio, saisie, forme) = self.triangle.pente_en_vigueur()?;
                let (etiquette, valeur) = valeur_pente(ratio, forme);
                (etiquette, valeur, saisie)
            }
            Touche::Montee | Touche::Course | Touche::Diag => {
                let cote = match touche {
                    Touche::Montee => Cote::Montee,
                    Touche::Course => Cote::Course,
                    _ => Cote::Diag,
                };
                let s = self.triangle.sortie_cote(cote)?;
                (s.etiquette, s.valeur, s.memorise)
            }
            Touche::Longueur | Touche::Largeur | Touche::Hauteur if !self.prefs.mode_trigo => {
                let (etiquette, registre) = match touche {
                    Touche::Longueur => (Etiquette::Longueur, self.dimensions.longueur),
                    Touche::Largeur => (Etiquette::Largeur, self.dimensions.largeur),
                    _ => (Etiquette::Hauteur, self.dimensions.hauteur),
                };
                (etiquette, registre.ok_or(Erreur::AucuneValeur)?, true)
            }
            Touche::Cercle => (
                Etiquette::Diametre,
                self.cercle.diametre.ok_or(Erreur::AucuneValeur)?,
                true,
            ),
            Touche::Arc => match self.cercle.arc.ok_or(Erreur::AucuneValeur)? {
                ArcSaisi::Angle(a) => (Etiquette::Arc, Valeur::degres(a), true),
                ArcSaisi::Longueur(l) => (Etiquette::Arc, l, true),
            },
            Touche::Fraction => {
                note = Some(note_fractions(self.prefs.mode_fractions));
                let resolution = f64::from(self.resolution());
                (
                    Etiquette::Resolution,
                    Valeur::en_base(1.0 / resolution, Format::PoFrac),
                    false,
                )
            }
            _ => return Err(Erreur::Saisie),
        };
        if matches!(etiquette, Etiquette::Pente) && valeur.genre == Genre::Longueur {
            note = Some(Note::Sur12);
        }
        // Une valeur rappelée est une valeur libre (elle peut servir d'opérande).
        self.saisie.vider();
        self.montrer_resultat(valeur);
        self.presentation = Presentation {
            libre: None,
            etiquette: Some(etiquette),
            memorise,
            avertissement: None,
            rendu: Rendu::Normal,
            masse_volumique,
            note,
        };
        self.rappel_de = Some(touche);
        match touche {
            Touche::Rappel => {
                let texte = self.texte_montre();
                self.ruban.memoire(Etiquette::TotalM.texte(), texte);
                self.memoire.vider();
                self.revision += 1;
            }
            Touche::MPlus => {
                let texte = self.texte_montre();
                self.ruban.memoire(Etiquette::TotalM.texte(), texte);
                self.revision += 1;
                // Total → moyenne → nombre de valeurs (R-40).
                let mut sorties = vec![Sortie::nouvelle(Etiquette::TotalM, valeur)];
                if let Some(moyenne) = self.memoire.moyenne() {
                    sorties.push(Sortie::nouvelle(Etiquette::Moyenne, moyenne));
                }
                sorties.push(Sortie::entier(
                    Etiquette::Nombre,
                    f64::from(self.memoire.nombre),
                ));
                let mut cycle = Cycle::nouveau(Touche::MPlus, false, sorties);
                cycle.reprendre_apres(0);
                self.cycle = Some(cycle);
            }
            _ => {}
        }
        Ok(())
    }

    /// `Rappel` `Conv` + touche : seule la pente irrégulière se rappelle ainsi.
    pub(super) fn touche_rappel_conv(&mut self, touche: Touche) -> Result<(), Erreur> {
        if touche != Touche::AretierNoue {
            return Err(Erreur::Saisie);
        }
        let (ratio, forme) = self.toit.pente_irreguliere.ok_or(Erreur::AucuneValeur)?;
        let (_, valeur) = valeur_pente(ratio, forme);
        self.saisie.vider();
        self.montrer_resultat(valeur);
        self.presentation = Presentation {
            etiquette: Some(Etiquette::PenteIrreguliere),
            memorise: true,
            note: (valeur.genre == Genre::Longueur).then_some(Note::Sur12),
            ..Presentation::default()
        };
        self.rappel_de = Some(touche);
        Ok(())
    }
}
