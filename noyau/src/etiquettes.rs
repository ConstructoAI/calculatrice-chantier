//! Étiquettes de l'afficheur (section 4.4) et symboles d'unités (section 4.2).

use crate::unites::{Format, UniteMasseVolumique};

/// Espace insécable (R-04, R-05).
pub const INSECABLE: char = '\u{00A0}';
/// Signe moins typographique (R-08).
pub const MOINS: char = '\u{2212}';

/// Étiquette affichée au-dessus de la valeur.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Etiquette {
    Pente,
    RatioPente,
    PenteIrreguliere,
    Montee,
    Course,
    Diagonale,
    CoupeAplomb,
    CoupeNiveau,
    CoupeLaterale,
    CoupeLaterale1,
    CoupeLaterale2,
    AretierNoue,
    AretierNoueIrr,
    Entraxe,
    Increment,
    Empannon(u32),
    EmpannonIrr(u32),
    Montant(u32),
    Base,
    AngleMur,
    AireToit,
    Carres,
    Paquets,
    Paquet,
    Feuilles4x8,
    Feuilles4x9,
    Feuilles4x10,
    Feuilles4x12,
    AirePlan,
    Contremarche,
    NbContremarches,
    EcartContremarches,
    Giron,
    NbMarches,
    EcartGirons,
    Tremie,
    Limon,
    Inclinaison,
    ContremarcheVisee,
    GironVise,
    Echappee,
    EpaisseurPlancher,
    Onglet,
    Biseau,
    AngleMoulure,
    Coin,
    Rayon,
    Diametre,
    Aire,
    Circonference,
    Arc,
    Corde,
    Segment,
    Secteur,
    Fleche,
    MurCintre(u32),
    ColonneVolume,
    ColonneSurface,
    ConeVolume,
    ConeSurface,
    AngleInterieur,
    DemiAngle,
    Cote,
    Perimetre,
    NbCotes,
    Longueur,
    Largeur,
    Hauteur,
    Equerrage,
    Volume,
    Murs,
    Piece,
    Blocs,
    AireBloc,
    LongueurBloc,
    Semelle,
    SectionSemelle,
    Montants,
    Cout,
    MasseVolumique,
    MPlus,
    MMoins,
    TotalM,
    Moyenne,
    Nombre,
    Registre(u8),
    Resolution,
}

impl Etiquette {
    /// Texte exact de l'étiquette (section 4.4).
    pub fn texte(self) -> String {
        use Etiquette::*;
        let fixe = match self {
            Pente => "PENTE",
            RatioPente => "RATIO DE PENTE",
            PenteIrreguliere => "PENTE IRRÉGULIÈRE",
            Montee => "MONTÉE",
            Course => "COURSE",
            Diagonale => "DIAGONALE",
            CoupeAplomb => "COUPE D'APLOMB",
            CoupeNiveau => "COUPE DE NIVEAU",
            CoupeLaterale => "COUPE LATÉRALE",
            CoupeLaterale1 => "COUPE LATÉRALE 1",
            CoupeLaterale2 => "COUPE LATÉRALE 2",
            AretierNoue => "ARÊTIER/NOUE",
            AretierNoueIrr => "ARÊTIER/NOUE IRRÉG.",
            Entraxe => "ENTRAXE",
            Increment => "INCRÉMENT",
            Empannon(n) => return format!("EMPANNON {n}"),
            EmpannonIrr(n) => return format!("EMPANNON IRRÉG. {n}"),
            Montant(n) => return format!("MONTANT {n}"),
            Base => "BASE",
            AngleMur => "ANGLE DU MUR",
            AireToit => "AIRE DU TOIT",
            Carres => "CARRÉS",
            Paquets => "PAQUETS",
            Paquet => "PAQUET",
            Feuilles4x8 => "FEUILLES 4×8",
            Feuilles4x9 => "FEUILLES 4×9",
            Feuilles4x10 => "FEUILLES 4×10",
            Feuilles4x12 => "FEUILLES 4×12",
            AirePlan => "AIRE EN PLAN",
            Contremarche => "CONTREMARCHE",
            NbContremarches => "NB CONTREMARCHES",
            EcartContremarches => "ÉCART CONTREMARCHES",
            Giron => "GIRON",
            NbMarches => "NB MARCHES",
            EcartGirons => "ÉCART GIRONS",
            Tremie => "TRÉMIE",
            Limon => "LIMON",
            Inclinaison => "INCLINAISON",
            ContremarcheVisee => "CONTREMARCHE VISÉE",
            GironVise => "GIRON VISÉ",
            Echappee => "ÉCHAPPÉE",
            EpaisseurPlancher => "ÉPAISSEUR PLANCHER",
            Onglet => "ONGLET",
            Biseau => "BISEAU",
            AngleMoulure => "ANGLE MOULURE",
            Coin => "COIN",
            Rayon => "RAYON",
            Diametre => "DIAMÈTRE",
            Aire => "AIRE",
            Circonference => "CIRCONFÉRENCE",
            Arc => "ARC",
            Corde => "CORDE",
            Segment => "SEGMENT",
            Secteur => "SECTEUR",
            Fleche => "FLÈCHE",
            MurCintre(n) => return format!("MUR CINTRÉ {n}"),
            ColonneVolume => "COLONNE – VOLUME",
            ColonneSurface => "COLONNE – SURFACE",
            ConeVolume => "CÔNE – VOLUME",
            ConeSurface => "CÔNE – SURFACE",
            AngleInterieur => "ANGLE INTÉRIEUR",
            DemiAngle => "DEMI-ANGLE",
            Cote => "CÔTÉ",
            Perimetre => "PÉRIMÈTRE",
            NbCotes => "NB CÔTÉS",
            Longueur => "LONGUEUR",
            Largeur => "LARGEUR",
            Hauteur => "HAUTEUR",
            Equerrage => "ÉQUERRAGE",
            Volume => "VOLUME",
            Murs => "MURS",
            Piece => "PIÈCE",
            Blocs => "BLOCS",
            AireBloc => "AIRE D'UN BLOC",
            LongueurBloc => "LONGUEUR D'UN BLOC",
            Semelle => "SEMELLE",
            SectionSemelle => "SECTION DE SEMELLE",
            Montants => "MONTANTS",
            Cout => "COÛT",
            MasseVolumique => "MASSE VOLUMIQUE",
            MPlus => "M+",
            MMoins => "M−",
            TotalM => "TOTAL M",
            Moyenne => "MOYENNE",
            Nombre => "NOMBRE",
            Registre(n) => return format!("M{n}"),
            Resolution => "RÉSOLUTION",
        };
        fixe.to_string()
    }
}

/// Mention complémentaire montrée près de l'étiquette.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Note {
    /// Pente exprimée en pouces de montée pour 12 pouces de course.
    Sur12,
    FractionsStandard,
    FractionsConstantes,
}

impl Note {
    pub fn texte(self) -> &'static str {
        match self {
            Note::Sur12 => "sur 12",
            Note::FractionsStandard => "Standard",
            Note::FractionsConstantes => "Constant",
        }
    }
}

/// Symbole affiché après le nombre (section 4.2); vide pour un nombre nu.
pub fn symbole(format: Format) -> &'static str {
    use Format::*;
    match format {
        Nombre | Dms => "",
        Degres => "°",
        Pourcent => "%",
        Monnaie => "$",
        PiPo | PiDec => "pi",
        PoFrac | PoDec => "po",
        Vg => "vg",
        M => "m",
        Cm => "cm",
        Mm => "mm",
        Pi2 => "pi²",
        Po2 => "po²",
        Vg2 => "vg²",
        M2 => "m²",
        Cm2 => "cm²",
        Mm2 => "mm²",
        Acre => "acre",
        Pi3 => "pi³",
        Po3 => "po³",
        Vg3 => "vg³",
        M3 => "m³",
        Cm3 => "cm³",
        Mm3 => "mm³",
        Pmp => "pmp",
        Lb => "lb",
        Kg => "kg",
        Tc => "tc",
        T => "t",
    }
}

/// Symbole d'une unité de masse volumique.
pub fn symbole_masse_volumique(unite: UniteMasseVolumique) -> &'static str {
    match unite {
        UniteMasseVolumique::TcParVg3 => "tc/vg³",
        UniteMasseVolumique::LbParVg3 => "lb/vg³",
        UniteMasseVolumique::LbParPi3 => "lb/pi³",
        UniteMasseVolumique::TParM3 => "t/m³",
        UniteMasseVolumique::KgParM3 => "kg/m³",
    }
}

/// Signes d'angle : degré, minute, seconde.
pub const DEGRE: char = '°';
pub const MINUTE: char = '′';
pub const SECONDE: char = '″';
