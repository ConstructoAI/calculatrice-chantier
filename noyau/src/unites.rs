//! Unités, genres, formats d'affichage, familles et systèmes (section 6, annexe A).

use serde::{Deserialize, Serialize};

/// Tolérance de comparaison (R-12), relative à l'ordre de grandeur du pouce.
pub const EPS: f64 = 1e-9;

// Facteurs exacts vers l'unité de base (annexe A.1).
pub const PO_PAR_PI: f64 = 12.0;
pub const PO_PAR_VG: f64 = 36.0;
pub const PO_PAR_M: f64 = 1.0 / 0.0254;
pub const PO_PAR_CM: f64 = 1.0 / 2.54;
pub const PO_PAR_MM: f64 = 1.0 / 25.4;
pub const PO2_PAR_PI2: f64 = 144.0;
pub const PO2_PAR_VG2: f64 = 1296.0;
pub const PO2_PAR_ACRE: f64 = 6_272_640.0;
pub const PO3_PAR_PI3: f64 = 1728.0;
pub const PO3_PAR_VG3: f64 = 46_656.0;
pub const PO3_PAR_PMP: f64 = 144.0;
pub const KG_PAR_LB: f64 = 0.453_592_37;
pub const LB_PAR_KG: f64 = 1.0 / KG_PAR_LB;
pub const LB_PAR_TC: f64 = 2000.0;
pub const LB_PAR_T: f64 = 1000.0 / KG_PAR_LB;
/// Les durées se comptent en minutes (évolution 16).
pub const MIN_PAR_H: f64 = 60.0;

/// Dimension physique d'une valeur.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Genre {
    #[default]
    SansUnite,
    Longueur,
    Aire,
    Volume,
    Masse,
    /// Une durée, en minutes (heures et minutes, section 22.20).
    Duree,
}

/// Format d'affichage d'une valeur.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Format {
    // Sans unité
    #[default]
    Nombre,
    Degres,
    Dms,
    Pourcent,
    Monnaie,
    // Longueur
    PiPo,
    PiDec,
    PoFrac,
    PoDec,
    Vg,
    M,
    Cm,
    Mm,
    // Aire
    Pi2,
    Po2,
    Vg2,
    M2,
    Cm2,
    Mm2,
    Acre,
    // Volume
    Pi3,
    Po3,
    Vg3,
    M3,
    Cm3,
    Mm3,
    Pmp,
    // Masse
    Lb,
    Kg,
    Tc,
    T,
    // Durée
    /// Heures et minutes : « 16 h 15 ».
    HMin,
    /// Heures décimales : « 16,25 h ».
    HDec,
    /// Minutes : « 75 min ».
    Min,
}

/// Famille d'unités d'un format (section 6.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Famille {
    Pieds,
    Pouces,
    Verges,
    Metres,
    Centimetres,
    Millimetres,
}

/// Système de mesure d'une famille.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Systeme {
    Imperial,
    Metrique,
}

impl Famille {
    pub fn systeme(self) -> Systeme {
        match self {
            Famille::Pieds | Famille::Pouces | Famille::Verges => Systeme::Imperial,
            Famille::Metres | Famille::Centimetres | Famille::Millimetres => Systeme::Metrique,
        }
    }

    /// Format linéaire par défaut de la famille.
    pub fn lineaire(self) -> Format {
        match self {
            Famille::Pieds => Format::PiPo,
            Famille::Pouces => Format::PoFrac,
            Famille::Verges => Format::Vg,
            Famille::Metres => Format::M,
            Famille::Centimetres => Format::Cm,
            Famille::Millimetres => Format::Mm,
        }
    }

    pub fn carre(self) -> Format {
        match self {
            Famille::Pieds => Format::Pi2,
            Famille::Pouces => Format::Po2,
            Famille::Verges => Format::Vg2,
            Famille::Metres => Format::M2,
            Famille::Centimetres => Format::Cm2,
            Famille::Millimetres => Format::Mm2,
        }
    }

    pub fn cube(self) -> Format {
        match self {
            Famille::Pieds => Format::Pi3,
            Famille::Pouces => Format::Po3,
            Famille::Verges => Format::Vg3,
            Famille::Metres => Format::M3,
            Famille::Centimetres => Format::Cm3,
            Famille::Millimetres => Format::Mm3,
        }
    }
}

impl Format {
    /// Genre des valeurs qui s'affichent dans ce format.
    pub fn genre(self) -> Genre {
        use Format::*;
        match self {
            Nombre | Degres | Dms | Pourcent | Monnaie => Genre::SansUnite,
            PiPo | PiDec | PoFrac | PoDec | Vg | M | Cm | Mm => Genre::Longueur,
            Pi2 | Po2 | Vg2 | M2 | Cm2 | Mm2 | Acre => Genre::Aire,
            Pi3 | Po3 | Vg3 | M3 | Cm3 | Mm3 | Pmp => Genre::Volume,
            Lb | Kg | Tc | T => Genre::Masse,
            HMin | HDec | Min => Genre::Duree,
        }
    }

    /// Nombre d'unités de base contenues dans une unité affichée.
    pub fn facteur(self) -> f64 {
        use Format::*;
        match self {
            Nombre | Degres | Dms | Monnaie => 1.0,
            Pourcent => 0.01,
            PiPo | PiDec => PO_PAR_PI,
            PoFrac | PoDec => 1.0,
            Vg => PO_PAR_VG,
            M => PO_PAR_M,
            Cm => PO_PAR_CM,
            Mm => PO_PAR_MM,
            Pi2 => PO2_PAR_PI2,
            Po2 => 1.0,
            Vg2 => PO2_PAR_VG2,
            M2 => PO_PAR_M * PO_PAR_M,
            Cm2 => PO_PAR_CM * PO_PAR_CM,
            Mm2 => PO_PAR_MM * PO_PAR_MM,
            Acre => PO2_PAR_ACRE,
            Pi3 => PO3_PAR_PI3,
            Po3 => 1.0,
            Vg3 => PO3_PAR_VG3,
            M3 => PO_PAR_M * PO_PAR_M * PO_PAR_M,
            Cm3 => PO_PAR_CM * PO_PAR_CM * PO_PAR_CM,
            Mm3 => PO_PAR_MM * PO_PAR_MM * PO_PAR_MM,
            Pmp => PO3_PAR_PMP,
            Lb => 1.0,
            Kg => LB_PAR_KG,
            Tc => LB_PAR_TC,
            T => LB_PAR_T,
            HMin | HDec => MIN_PAR_H,
            Min => 1.0,
        }
    }

    /// Famille d'unités (section 6.2); `None` pour les formats sans unité et les masses.
    pub fn famille(self) -> Option<Famille> {
        use Format::*;
        match self {
            PiPo | PiDec | Pi2 | Pi3 | Pmp | Acre => Some(Famille::Pieds),
            PoFrac | PoDec | Po2 | Po3 => Some(Famille::Pouces),
            Vg | Vg2 | Vg3 => Some(Famille::Verges),
            M | M2 | M3 => Some(Famille::Metres),
            Cm | Cm2 | Cm3 => Some(Famille::Centimetres),
            Mm | Mm2 | Mm3 => Some(Famille::Millimetres),
            _ => None,
        }
    }

    pub fn systeme(self) -> Option<Systeme> {
        self.famille().map(Famille::systeme)
    }

    /// Unité supérieure du même système, pour les très grandes valeurs (section 8.7).
    pub fn superieur(self) -> Option<Format> {
        use Format::*;
        match self {
            Mm | Cm => Some(M),
            PoFrac => Some(PiPo),
            PoDec => Some(PiDec),
            PiPo | PiDec => Some(Vg),
            Mm2 | Cm2 => Some(M2),
            Po2 => Some(Pi2),
            Pi2 => Some(Vg2),
            Mm3 | Cm3 => Some(M3),
            Po3 => Some(Pi3),
            Pi3 => Some(Vg3),
            _ => None,
        }
    }

    /// Vrai pour les formats en pieds-pouces-fraction ou en pouces-fraction.
    pub fn est_fractionnaire(self) -> bool {
        matches!(self, Format::PiPo | Format::PoFrac)
    }
}

/// Unité de la masse volumique mémorisée (annexe A.3).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum UniteMasseVolumique {
    #[default]
    TcParVg3,
    LbParVg3,
    LbParPi3,
    TParM3,
    KgParM3,
}

impl UniteMasseVolumique {
    pub const TOUTES: [UniteMasseVolumique; 5] = [
        UniteMasseVolumique::TcParVg3,
        UniteMasseVolumique::LbParVg3,
        UniteMasseVolumique::LbParPi3,
        UniteMasseVolumique::TParM3,
        UniteMasseVolumique::KgParM3,
    ];
}

/// Masse volumique : un nombre et son unité.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MasseVolumique {
    pub valeur: f64,
    pub unite: UniteMasseVolumique,
}

impl Default for MasseVolumique {
    fn default() -> Self {
        MasseVolumique {
            valeur: 1.5,
            unite: UniteMasseVolumique::TcParVg3,
        }
    }
}

impl MasseVolumique {
    /// Masse volumique exprimée en livres par pouce cube.
    pub fn lb_par_po3(&self) -> f64 {
        let m3_en_po3 = PO_PAR_M * PO_PAR_M * PO_PAR_M;
        match self.unite {
            UniteMasseVolumique::TcParVg3 => self.valeur * LB_PAR_TC / PO3_PAR_VG3,
            UniteMasseVolumique::LbParVg3 => self.valeur / PO3_PAR_VG3,
            UniteMasseVolumique::LbParPi3 => self.valeur / PO3_PAR_PI3,
            UniteMasseVolumique::TParM3 => self.valeur * LB_PAR_T / m3_en_po3,
            UniteMasseVolumique::KgParM3 => self.valeur * LB_PAR_KG / m3_en_po3,
        }
    }

    /// La même masse volumique exprimée dans une autre unité.
    pub fn convertie(&self, unite: UniteMasseVolumique) -> MasseVolumique {
        let unitaire = MasseVolumique { valeur: 1.0, unite }.lb_par_po3();
        MasseVolumique {
            valeur: self.lb_par_po3() / unitaire,
            unite,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reperes_de_masse_volumique() {
        let mv = MasseVolumique::default();
        let proche = |a: f64, b: f64| (a - b).abs() < 1e-3 * b.abs().max(1.0);
        assert!(proche(
            mv.convertie(UniteMasseVolumique::LbParVg3).valeur,
            3000.0
        ));
        assert!(proche(
            mv.convertie(UniteMasseVolumique::LbParPi3).valeur,
            111.1111
        ));
        assert!(proche(
            mv.convertie(UniteMasseVolumique::TParM3).valeur,
            1.779829
        ));
        assert!(proche(
            mv.convertie(UniteMasseVolumique::KgParM3).valeur,
            1779.829
        ));
    }

    #[test]
    fn genres_et_facteurs_coherents() {
        assert_eq!(Format::PiPo.genre(), Genre::Longueur);
        assert_eq!(Format::Acre.genre(), Genre::Aire);
        assert_eq!(Format::Pmp.genre(), Genre::Volume);
        assert_eq!(Format::Tc.genre(), Genre::Masse);
        assert_eq!(Format::HMin.genre(), Genre::Duree);
        assert_eq!(Format::HDec.facteur(), 60.0);
        assert!((Format::M.facteur() - 39.370_078_74).abs() < 1e-8);
        assert!((Format::Acre.facteur() - 43_560.0 * 144.0).abs() < 1e-6);
    }
}
