//! Préférences et valeurs mémorisées (section 13, annexe A.4).

use serde::{Deserialize, Serialize};

use crate::langue::Langue;
use crate::unites::{Format, MasseVolumique};
use crate::valeur::Valeur;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    #[default]
    Automatique,
    Clair,
    Sombre,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModeFractions {
    /// Fraction réduite (8/16 → 1/2).
    #[default]
    Standard,
    /// Dénominateur fixe, celui de la résolution.
    Constant,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AffichageAires {
    #[default]
    Standard,
    PiedsCarres,
    VergesCarrees,
    MetresCarres,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AffichageVolumes {
    #[default]
    Standard,
    VergesCubes,
    PiedsCubes,
    MetresCubes,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeparateurDecimal {
    #[default]
    Virgule,
    Point,
}

/// Ordre d'une liste de montants ou d'empannons.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Ordre {
    /// Du plus long au plus court.
    #[default]
    Decroissant,
    Croissant,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MurCintre {
    /// Montants au-dessus de l'arc.
    #[default]
    Exterieur,
    /// Montants sous l'arc.
    Interieur,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmpannonsIrreguliers {
    #[default]
    EntraxeConstant,
    Apparies,
}

/// Résolutions de fraction permises (dénominateur maximal).
pub const RESOLUTIONS: [u32; 6] = [2, 4, 8, 16, 32, 64];

/// Préférences de l'utilisateur (sections 13.1 à 13.3).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Preferences {
    // Général
    /// Langue de l'interface (section 24).
    pub langue: Langue,
    pub mode_trigo: bool,
    pub fractions_rapides: bool,
    pub plein_ecran: bool,
    pub theme: Theme,
    /// Mode chantier (section 16.7) : contraste élevé, et les touches du bas
    /// plus grosses, jusqu'au bas de l'écran. Il passe avant le thème.
    pub mode_chantier: bool,
    /// Vrai quand le plan d'exemple s'est déjà ouvert de lui-même (section
    /// 25.1, point 58) : il ne s'ajoute plus aux dessins.
    pub exemple_ouvert: bool,
    /// Vrai quand l'exemple simple du mode Débutant s'est déjà ouvert de
    /// lui-même (section 25.1, point 61).
    pub exemple_simple_ouvert: bool,
    /// Vrai quand le module Dessin montre tous ses outils : le mode Avancé
    /// (section 25.1, point 59). Faux, le mode Débutant : l'essentiel d'un
    /// croquis précis.
    pub dessin_avance: bool,
    /// Le projet écrit sur les rapports de calcul (section 22.18); vide : aucun.
    pub projet: String,
    /// Les angles du niveau sur sa surface de référence (section 22.22) :
    /// l'avant-arrière et la gauche-droite, en degrés, retranchés des lectures.
    pub etalon_niveau: [f64; 2],
    // Affichage
    pub resolution: u32,
    pub mode_fractions: ModeFractions,
    pub exponentielle: bool,
    pub affichage_aires: AffichageAires,
    pub affichage_volumes: AffichageVolumes,
    /// Vrai : mètres à trois décimales fixes; faux : flottant.
    pub metres_fixes: bool,
    /// Vrai : degrés à deux décimales fixes; faux : flottant.
    pub degres_fixes: bool,
    pub separateur_milliers: bool,
    pub separateur_decimal: SeparateurDecimal,
    /// `None` : flottant; `Some(n)` : n décimales fixes (0 à 4).
    pub decimales_sans_unite: Option<u8>,
    // Fonctions
    pub mur_pente: Ordre,
    pub mur_cintre: MurCintre,
    pub empannons: Ordre,
    pub empannons_irreguliers: EmpannonsIrreguliers,
}

impl Default for Preferences {
    fn default() -> Self {
        Preferences {
            langue: Langue::Francais,
            mode_trigo: false,
            fractions_rapides: false,
            plein_ecran: false,
            theme: Theme::Automatique,
            mode_chantier: false,
            exemple_ouvert: false,
            exemple_simple_ouvert: false,
            dessin_avance: false,
            projet: String::new(),
            etalon_niveau: [0.0; 2],
            resolution: 16,
            mode_fractions: ModeFractions::Standard,
            exponentielle: false,
            affichage_aires: AffichageAires::Standard,
            affichage_volumes: AffichageVolumes::Standard,
            metres_fixes: true,
            degres_fixes: true,
            separateur_milliers: true,
            separateur_decimal: SeparateurDecimal::Virgule,
            decimales_sans_unite: None,
            mur_pente: Ordre::Decroissant,
            mur_cintre: MurCintre::Exterieur,
            empannons: Ordre::Decroissant,
            empannons_irreguliers: EmpannonsIrreguliers::EntraxeConstant,
        }
    }
}

impl Preferences {
    /// Résolution des fractions, ramenée à une valeur permise.
    pub fn resolution_valide(&self) -> u32 {
        if RESOLUTIONS.contains(&self.resolution) {
            self.resolution
        } else {
            16
        }
    }

    /// Unité imposée aux aires calculées (R-32), ou `None` en mode Standard.
    pub fn format_aire_impose(&self) -> Option<Format> {
        match self.affichage_aires {
            AffichageAires::Standard => None,
            AffichageAires::PiedsCarres => Some(Format::Pi2),
            AffichageAires::VergesCarrees => Some(Format::Vg2),
            AffichageAires::MetresCarres => Some(Format::M2),
        }
    }

    /// Unité imposée aux volumes calculés (R-32), ou `None` en mode Standard.
    pub fn format_volume_impose(&self) -> Option<Format> {
        match self.affichage_volumes {
            AffichageVolumes::Standard => None,
            AffichageVolumes::VergesCubes => Some(Format::Vg3),
            AffichageVolumes::PiedsCubes => Some(Format::Pi3),
            AffichageVolumes::MetresCubes => Some(Format::M3),
        }
    }
}

/// Valeurs mémorisées (section 13.4).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ValeursMemorisees {
    pub echappee: Valeur,
    pub contremarche_visee: Valeur,
    pub epaisseur_plancher: Valeur,
    pub giron_vise: Valeur,
    pub aire_bloc: Valeur,
    pub longueur_bloc: Valeur,
    /// Angle de la moulure, en degrés.
    pub angle_moulure: f64,
    pub entraxe: Valeur,
    pub section_semelle: Valeur,
    pub masse_volumique: MasseVolumique,
    pub paquets_par_carre: f64,
}

impl Default for ValeursMemorisees {
    fn default() -> Self {
        ValeursMemorisees {
            echappee: Valeur::en_base(80.0, Format::PiPo),
            contremarche_visee: Valeur::en_base(7.5, Format::PoFrac),
            epaisseur_plancher: Valeur::en_base(10.0, Format::PoFrac),
            giron_vise: Valeur::en_base(10.0, Format::PoFrac),
            aire_bloc: Valeur::en_base(128.0, Format::Po2),
            longueur_bloc: Valeur::en_base(16.0, Format::PoFrac),
            angle_moulure: 45.0,
            entraxe: Valeur::en_base(16.0, Format::PoFrac),
            section_semelle: Valeur::en_base(264.0, Format::Po2),
            masse_volumique: MasseVolumique::default(),
            paquets_par_carre: 3.0,
        }
    }
}

impl ValeursMemorisees {
    /// Remet en état les valeurs incohérentes d'un fichier modifié à la main.
    pub fn assainir(&mut self) {
        let defaut = ValeursMemorisees::default();
        let longueur_positive = |v: &Valeur| {
            v.genre == crate::unites::Genre::Longueur && v.base.is_finite() && v.base > 0.0
        };
        let aire_positive = |v: &Valeur| {
            v.genre == crate::unites::Genre::Aire && v.base.is_finite() && v.base > 0.0
        };
        if !longueur_positive(&self.echappee) {
            self.echappee = defaut.echappee;
        }
        if !longueur_positive(&self.contremarche_visee) {
            self.contremarche_visee = defaut.contremarche_visee;
        }
        if !(self.epaisseur_plancher.genre == crate::unites::Genre::Longueur
            && self.epaisseur_plancher.base.is_finite()
            && self.epaisseur_plancher.base >= 0.0)
        {
            self.epaisseur_plancher = defaut.epaisseur_plancher;
        }
        if !longueur_positive(&self.giron_vise) {
            self.giron_vise = defaut.giron_vise;
        }
        if !aire_positive(&self.aire_bloc) {
            self.aire_bloc = defaut.aire_bloc;
        }
        if !longueur_positive(&self.longueur_bloc) {
            self.longueur_bloc = defaut.longueur_bloc;
        }
        if !(self.angle_moulure.is_finite()
            && self.angle_moulure > 0.0
            && self.angle_moulure < 90.0)
        {
            self.angle_moulure = defaut.angle_moulure;
        }
        if !longueur_positive(&self.entraxe) {
            self.entraxe = defaut.entraxe;
        }
        if !aire_positive(&self.section_semelle) {
            self.section_semelle = defaut.section_semelle;
        }
        if !(self.masse_volumique.valeur.is_finite() && self.masse_volumique.valeur > 0.0) {
            self.masse_volumique = defaut.masse_volumique;
        }
        if !(self.paquets_par_carre.is_finite() && self.paquets_par_carre > 0.0) {
            self.paquets_par_carre = defaut.paquets_par_carre;
        }
    }
}
