//! Étiquettes de l'afficheur (section 4.4) et symboles d'unités (section 4.2),
//! dans les deux langues (section 24).

use crate::langue::selon;
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
    /// Texte exact de l'étiquette (section 4.4), dans la langue en cours.
    pub fn texte(self) -> String {
        use Etiquette::*;
        let fixe = match self {
            Pente => selon("PENTE", "PITCH"),
            RatioPente => selon("RATIO DE PENTE", "PITCH RATIO"),
            PenteIrreguliere => selon("PENTE IRRÉGULIÈRE", "IRREGULAR PITCH"),
            Montee => selon("MONTÉE", "RISE"),
            Course => selon("COURSE", "RUN"),
            Diagonale => selon("DIAGONALE", "DIAGONAL"),
            CoupeAplomb => selon("COUPE D'APLOMB", "PLUMB CUT"),
            CoupeNiveau => selon("COUPE DE NIVEAU", "LEVEL CUT"),
            CoupeLaterale => selon("COUPE LATÉRALE", "CHEEK CUT"),
            CoupeLaterale1 => selon("COUPE LATÉRALE 1", "CHEEK CUT 1"),
            CoupeLaterale2 => selon("COUPE LATÉRALE 2", "CHEEK CUT 2"),
            AretierNoue => selon("ARÊTIER/NOUE", "HIP/VALLEY"),
            AretierNoueIrr => selon("ARÊTIER/NOUE IRRÉG.", "IRREG. HIP/VALLEY"),
            Entraxe => selon("ENTRAXE", "ON CENTRE"),
            Increment => selon("INCRÉMENT", "INCREMENT"),
            Empannon(n) => return format!("{} {n}", selon("EMPANNON", "JACK")),
            EmpannonIrr(n) => return format!("{} {n}", selon("EMPANNON IRRÉG.", "IRREG. JACK")),
            Montant(n) => return format!("{} {n}", selon("MONTANT", "STUD")),
            Base => "BASE",
            AngleMur => selon("ANGLE DU MUR", "WALL ANGLE"),
            AireToit => selon("AIRE DU TOIT", "ROOF AREA"),
            Carres => selon("CARRÉS", "SQUARES"),
            Paquets => selon("PAQUETS", "BUNDLES"),
            Paquet => selon("PAQUET", "BUNDLE"),
            Feuilles4x8 => selon("FEUILLES 4×8", "4×8 SHEETS"),
            Feuilles4x9 => selon("FEUILLES 4×9", "4×9 SHEETS"),
            Feuilles4x10 => selon("FEUILLES 4×10", "4×10 SHEETS"),
            Feuilles4x12 => selon("FEUILLES 4×12", "4×12 SHEETS"),
            AirePlan => selon("AIRE EN PLAN", "PLAN AREA"),
            Contremarche => selon("CONTREMARCHE", "RISER HEIGHT"),
            NbContremarches => selon("NB CONTREMARCHES", "RISERS"),
            EcartContremarches => selon("ÉCART CONTREMARCHES", "RISER DIFFERENCE"),
            Giron => selon("GIRON", "TREAD DEPTH"),
            NbMarches => selon("NB MARCHES", "TREADS"),
            EcartGirons => selon("ÉCART GIRONS", "TREAD DIFFERENCE"),
            Tremie => selon("TRÉMIE", "STAIRWELL OPENING"),
            Limon => selon("LIMON", "STRINGER"),
            Inclinaison => selon("INCLINAISON", "INCLINE"),
            ContremarcheVisee => selon("CONTREMARCHE VISÉE", "TARGET RISER"),
            GironVise => selon("GIRON VISÉ", "TARGET TREAD"),
            Echappee => selon("ÉCHAPPÉE", "HEADROOM"),
            EpaisseurPlancher => selon("ÉPAISSEUR PLANCHER", "FLOOR THICKNESS"),
            Onglet => selon("ONGLET", "MITRE"),
            Biseau => selon("BISEAU", "BEVEL"),
            AngleMoulure => selon("ANGLE MOULURE", "CROWN ANGLE"),
            Coin => selon("COIN", "CORNER"),
            Rayon => selon("RAYON", "RADIUS"),
            Diametre => selon("DIAMÈTRE", "DIAMETER"),
            Aire => selon("AIRE", "AREA"),
            Circonference => selon("CIRCONFÉRENCE", "CIRCUMFERENCE"),
            Arc => "ARC",
            Corde => selon("CORDE", "CHORD"),
            Segment => "SEGMENT",
            Secteur => selon("SECTEUR", "SECTOR"),
            Fleche => selon("FLÈCHE", "ARC RISE"),
            MurCintre(n) => return format!("{} {n}", selon("MUR CINTRÉ", "ARCHED WALL")),
            ColonneVolume => selon("COLONNE – VOLUME", "COLUMN – VOLUME"),
            ColonneSurface => selon("COLONNE – SURFACE", "COLUMN – SURFACE"),
            ConeVolume => selon("CÔNE – VOLUME", "CONE – VOLUME"),
            ConeSurface => selon("CÔNE – SURFACE", "CONE – SURFACE"),
            AngleInterieur => selon("ANGLE INTÉRIEUR", "INTERIOR ANGLE"),
            DemiAngle => selon("DEMI-ANGLE", "HALF ANGLE"),
            Cote => selon("CÔTÉ", "SIDE"),
            Perimetre => selon("PÉRIMÈTRE", "PERIMETER"),
            NbCotes => selon("NB CÔTÉS", "SIDES"),
            Longueur => selon("LONGUEUR", "LENGTH"),
            Largeur => selon("LARGEUR", "WIDTH"),
            Hauteur => selon("HAUTEUR", "HEIGHT"),
            Equerrage => selon("ÉQUERRAGE", "SQUARE-UP"),
            Volume => "VOLUME",
            Murs => selon("MURS", "WALLS"),
            Piece => selon("PIÈCE", "ROOM"),
            Blocs => selon("BLOCS", "BLOCKS"),
            AireBloc => selon("AIRE D'UN BLOC", "BLOCK AREA"),
            LongueurBloc => selon("LONGUEUR D'UN BLOC", "BLOCK LENGTH"),
            Semelle => selon("SEMELLE", "FOOTING"),
            SectionSemelle => selon("SECTION DE SEMELLE", "FOOTING SECTION"),
            Montants => selon("MONTANTS", "STUDS"),
            Cout => selon("COÛT", "COST"),
            MasseVolumique => selon("MASSE VOLUMIQUE", "DENSITY"),
            MPlus => "M+",
            MMoins => "M−",
            TotalM => selon("TOTAL M", "M TOTAL"),
            Moyenne => selon("MOYENNE", "AVERAGE"),
            Nombre => selon("NOMBRE", "COUNT"),
            Registre(n) => return format!("M{n}"),
            Resolution => selon("RÉSOLUTION", "RESOLUTION"),
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
            Note::Sur12 => selon("sur 12", "in 12"),
            Note::FractionsStandard => "Standard",
            Note::FractionsConstantes => "Constant",
        }
    }
}

/// Symbole affiché après le nombre (section 4.2), dans la langue en cours;
/// vide pour un nombre nu.
pub fn symbole(format: Format) -> &'static str {
    use Format::*;
    match format {
        Nombre | Dms => "",
        Degres => "°",
        Pourcent => "%",
        Monnaie => "$",
        PiPo | PiDec => selon("pi", "ft"),
        PoFrac | PoDec => selon("po", "in"),
        Vg => selon("vg", "yd"),
        M => "m",
        Cm => "cm",
        Mm => "mm",
        Pi2 => selon("pi²", "ft²"),
        Po2 => selon("po²", "in²"),
        Vg2 => selon("vg²", "yd²"),
        M2 => "m²",
        Cm2 => "cm²",
        Mm2 => "mm²",
        Acre => "acre",
        Pi3 => selon("pi³", "ft³"),
        Po3 => selon("po³", "in³"),
        Vg3 => selon("vg³", "yd³"),
        M3 => "m³",
        Cm3 => "cm³",
        Mm3 => "mm³",
        Pmp => selon("pmp", "bd ft"),
        Lb => "lb",
        Kg => "kg",
        Tc => selon("tc", "ton"),
        T => "t",
    }
}

/// Rang d'un résultat ou d'une option parmi plusieurs : « 2 de 3 » (section 23).
pub fn rang_parmi(rang: usize, nombre: usize) -> String {
    format!("{rang} {} {nombre}", selon("de", "of"))
}

/// Symbole d'une unité de masse volumique.
pub fn symbole_masse_volumique(unite: UniteMasseVolumique) -> &'static str {
    match unite {
        UniteMasseVolumique::TcParVg3 => selon("tc/vg³", "ton/yd³"),
        UniteMasseVolumique::LbParVg3 => selon("lb/vg³", "lb/yd³"),
        UniteMasseVolumique::LbParPi3 => selon("lb/pi³", "lb/ft³"),
        UniteMasseVolumique::TParM3 => "t/m³",
        UniteMasseVolumique::KgParM3 => "kg/m³",
    }
}

/// Signes d'angle : degré, minute, seconde.
pub const DEGRE: char = '°';
pub const MINUTE: char = '′';
pub const SECONDE: char = '″';
