//! Conversions : toutes les unités utiles sur un chantier (section 22.23,
//! demande 41). Chaque convertisseur prend une valeur et son unité, et la
//! donne dans toutes les unités de sa grandeur, à quatre chiffres
//! significatifs; la pente, l'isolation et le bois d'œuvre ont leur calcul.

use super::{Calcul, Calculateur, Champ, Lues, Metier, Quantite, Resultat};
use crate::langue::{Texte, t, tel};
use crate::unites::{EPS, Format, PO_PAR_PI};

/// Une unité d'une grandeur : son symbole, écrit après le nombre, et ce
/// qu'elle vaut dans l'unité de base de la grandeur (base = valeur ×
/// `facteur` + `decalage`; le décalage ne sert qu'aux températures).
struct Unite {
    symbole: Texte,
    facteur: f64,
    decalage: f64,
}

/// Les unités d'une grandeur : leurs noms (la liste du choix et les libellés
/// des résultats) et leurs valeurs, dans le même ordre.
macro_rules! unites {
    ($noms:ident, $unites:ident, [$(($nom_fr:literal, $nom_en:literal, $symbole_fr:literal, $symbole_en:literal, $facteur:expr $(, $decalage:expr)?)),+ $(,)?]) => {
        const $noms: &[Texte] = &[$(t($nom_fr, $nom_en)),+];
        const $unites: &[Unite] = &[$(Unite {
            symbole: t($symbole_fr, $symbole_en),
            facteur: $facteur,
            decalage: 0.0 $(+ $decalage)?,
        }),+];
    };
}

// Les longueurs, en mètres.
unites!(
    NOMS_LONGUEURS,
    LONGUEURS,
    [
        ("Pouces (po)", "Inches (in)", "po", "in", 0.0254),
        ("Pieds (pi)", "Feet (ft)", "pi", "ft", 0.3048),
        ("Verges (vg)", "Yards (yd)", "vg", "yd", 0.9144),
        ("Milles (mi)", "Miles (mi)", "mi", "mi", 1_609.344),
        ("Millimètres (mm)", "Millimetres (mm)", "mm", "mm", 0.001),
        ("Centimètres (cm)", "Centimetres (cm)", "cm", "cm", 0.01),
        ("Mètres (m)", "Metres (m)", "m", "m", 1.0),
        ("Kilomètres (km)", "Kilometres (km)", "km", "km", 1_000.0),
    ]
);

// Les aires, en mètres carrés.
unites!(
    NOMS_AIRES,
    AIRES,
    [
        (
            "Pouces carrés (po²)",
            "Square inches (in²)",
            "po²",
            "in²",
            0.000_645_16
        ),
        (
            "Pieds carrés (pi²)",
            "Square feet (ft²)",
            "pi²",
            "ft²",
            0.092_903_04
        ),
        (
            "Verges carrées (vg²)",
            "Square yards (yd²)",
            "vg²",
            "yd²",
            0.836_127_36
        ),
        (
            "Carrés de toiture (100 pi²)",
            "Roofing squares (100 ft²)",
            "carrés",
            "squares",
            9.290_304
        ),
        ("Acres", "Acres", "acres", "acres", 4_046.856_422_4),
        (
            "Centimètres carrés (cm²)",
            "Square centimetres (cm²)",
            "cm²",
            "cm²",
            0.000_1
        ),
        ("Mètres carrés (m²)", "Square metres (m²)", "m²", "m²", 1.0),
        ("Hectares (ha)", "Hectares (ha)", "ha", "ha", 10_000.0),
    ]
);

// Les volumes, en mètres cubes.
unites!(
    NOMS_VOLUMES,
    VOLUMES,
    [
        (
            "Pouces cubes (po³)",
            "Cubic inches (in³)",
            "po³",
            "in³",
            0.000_016_387_064
        ),
        (
            "Pieds cubes (pi³)",
            "Cubic feet (ft³)",
            "pi³",
            "ft³",
            0.028_316_846_592
        ),
        (
            "Verges cubes (vg³)",
            "Cubic yards (yd³)",
            "vg³",
            "yd³",
            0.764_554_857_984
        ),
        (
            "Pieds-mesure de planche (pmp)",
            "Board feet (fbm)",
            "pmp",
            "fbm",
            0.002_359_737_216
        ),
        ("Litres (L)", "Litres (L)", "L", "L", 0.001),
        ("Mètres cubes (m³)", "Cubic metres (m³)", "m³", "m³", 1.0),
    ]
);

// Les liquides, en litres.
unites!(
    NOMS_LIQUIDES,
    LIQUIDES,
    [
        ("Millilitres (mL)", "Millilitres (mL)", "mL", "mL", 0.001),
        ("Litres (L)", "Litres (L)", "L", "L", 1.0),
        (
            "Mètres cubes (m³)",
            "Cubic metres (m³)",
            "m³",
            "m³",
            1_000.0
        ),
        (
            "Onces liquides US",
            "US fluid ounces",
            "oz liq. US",
            "US fl oz",
            0.029_573_529_562_5
        ),
        (
            "Gallons US",
            "US gallons",
            "gal US",
            "US gal",
            3.785_411_784
        ),
        (
            "Gallons impériaux",
            "Imperial gallons",
            "gal imp.",
            "imp. gal",
            4.546_09
        ),
        (
            "Pieds cubes (pi³)",
            "Cubic feet (ft³)",
            "pi³",
            "ft³",
            28.316_846_592
        ),
        (
            "Pouces cubes (po³)",
            "Cubic inches (in³)",
            "po³",
            "in³",
            0.016_387_064
        ),
    ]
);

// Les masses, en kilogrammes.
unites!(
    NOMS_MASSES,
    MASSES,
    [
        ("Onces (oz)", "Ounces (oz)", "oz", "oz", 0.028_349_523_125),
        ("Livres (lb)", "Pounds (lb)", "lb", "lb", 0.453_592_37),
        (
            "Tonnes courtes (2 000 lb)",
            "Short tons (2,000 lb)",
            "tc",
            "tn",
            907.184_74
        ),
        ("Grammes (g)", "Grams (g)", "g", "g", 0.001),
        ("Kilogrammes (kg)", "Kilograms (kg)", "kg", "kg", 1.0),
        (
            "Tonnes métriques (t)",
            "Metric tonnes (t)",
            "t",
            "t",
            1_000.0
        ),
    ]
);

// Les masses volumiques, en kilogrammes par mètre cube.
unites!(
    NOMS_MASSES_VOLUMIQUES,
    MASSES_VOLUMIQUES,
    [
        (
            "Kilogrammes par mètre cube (kg/m³)",
            "Kilograms per cubic metre (kg/m³)",
            "kg/m³",
            "kg/m³",
            1.0
        ),
        (
            "Tonnes par mètre cube (t/m³)",
            "Tonnes per cubic metre (t/m³)",
            "t/m³",
            "t/m³",
            1_000.0
        ),
        (
            "Livres par pied cube (lb/pi³)",
            "Pounds per cubic foot (lb/ft³)",
            "lb/pi³",
            "lb/ft³",
            16.018_463_373_960_14
        ),
        (
            "Livres par verge cube (lb/vg³)",
            "Pounds per cubic yard (lb/yd³)",
            "lb/vg³",
            "lb/yd³",
            0.593_276_421_257_783
        ),
        (
            "Livres par gallon US",
            "Pounds per US gallon",
            "lb/gal US",
            "lb/US gal",
            119.826_427_316_896_6
        ),
    ]
);

// Les pressions et les charges réparties, en pascals.
unites!(
    NOMS_PRESSIONS,
    PRESSIONS,
    [
        (
            "Kilopascals (kPa)",
            "Kilopascals (kPa)",
            "kPa",
            "kPa",
            1_000.0
        ),
        (
            "Mégapascals (MPa)",
            "Megapascals (MPa)",
            "MPa",
            "MPa",
            1.0e6
        ),
        ("Pascals (Pa)", "Pascals (Pa)", "Pa", "Pa", 1.0),
        ("Bars", "Bars", "bar", "bar", 1.0e5),
        (
            "Livres par pouce carré (lb/po²)",
            "Pounds per square inch (psi)",
            "lb/po²",
            "psi",
            6_894.757_293_168_36
        ),
        (
            "Livres par pied carré (lb/pi²)",
            "Pounds per square foot (psf)",
            "lb/pi²",
            "psf",
            47.880_258_980_335_84
        ),
        (
            "Atmosphères (atm)",
            "Atmospheres (atm)",
            "atm",
            "atm",
            101_325.0
        ),
        (
            "Pouces de mercure (po Hg)",
            "Inches of mercury (inHg)",
            "po Hg",
            "inHg",
            3_386.389
        ),
        (
            "Pouces de colonne d'eau (po CE)",
            "Inches of water column (in. w.c.)",
            "po CE",
            "in. w.c.",
            249.088_9
        ),
        (
            "Pieds de colonne d'eau (pi CE)",
            "Feet of water column (ft w.c.)",
            "pi CE",
            "ft w.c.",
            2_989.066_8
        ),
    ]
);

// Les forces, en newtons.
unites!(
    NOMS_FORCES,
    FORCES,
    [
        ("Newtons (N)", "Newtons (N)", "N", "N", 1.0),
        ("Kilonewtons (kN)", "Kilonewtons (kN)", "kN", "kN", 1_000.0),
        (
            "Livres-force (lbf)",
            "Pounds-force (lbf)",
            "lbf",
            "lbf",
            4.448_221_615_260_5
        ),
        (
            "Kips (1 000 lbf)",
            "Kips (1,000 lbf)",
            "kip",
            "kip",
            4_448.221_615_260_5
        ),
        (
            "Kilogrammes-force (kgf)",
            "Kilograms-force (kgf)",
            "kgf",
            "kgf",
            9.806_65
        ),
    ]
);

// Les charges linéaires, en newtons par mètre.
unites!(
    NOMS_CHARGES_LINEAIRES,
    CHARGES_LINEAIRES,
    [
        (
            "Kilonewtons par mètre (kN/m)",
            "Kilonewtons per metre (kN/m)",
            "kN/m",
            "kN/m",
            1_000.0
        ),
        (
            "Newtons par mètre (N/m)",
            "Newtons per metre (N/m)",
            "N/m",
            "N/m",
            1.0
        ),
        (
            "Livres par pied (lb/pi)",
            "Pounds per foot (plf)",
            "lb/pi",
            "plf",
            14.593_902_937_206_36
        ),
        (
            "Kips par pied (kip/pi)",
            "Kips per foot (klf)",
            "kip/pi",
            "klf",
            14_593.902_937_206_36
        ),
        (
            "Kilogrammes-force par mètre (kgf/m)",
            "Kilograms-force per metre (kgf/m)",
            "kgf/m",
            "kgf/m",
            9.806_65
        ),
    ]
);

// Les couples et les moments, en newtons-mètres.
unites!(
    NOMS_COUPLES,
    COUPLES,
    [
        (
            "Newtons-mètres (N·m)",
            "Newton-metres (N·m)",
            "N·m",
            "N·m",
            1.0
        ),
        (
            "Kilonewtons-mètres (kN·m)",
            "Kilonewton-metres (kN·m)",
            "kN·m",
            "kN·m",
            1_000.0
        ),
        (
            "Livres-pieds (lb·pi)",
            "Pound-feet (lb·ft)",
            "lb·pi",
            "lb·ft",
            1.355_817_948_331_4
        ),
        (
            "Livres-pouces (lb·po)",
            "Pound-inches (lb·in)",
            "lb·po",
            "lb·in",
            0.112_984_829_027_617
        ),
        (
            "Kips-pieds (kip·pi)",
            "Kip-feet (kip·ft)",
            "kip·pi",
            "kip·ft",
            1_355.817_948_331_4
        ),
        (
            "Kilogrammes-force-mètres (kgf·m)",
            "Kilogram-force metres (kgf·m)",
            "kgf·m",
            "kgf·m",
            9.806_65
        ),
    ]
);

// Les températures, en kelvins.
unites!(
    NOMS_TEMPERATURES,
    TEMPERATURES,
    [
        (
            "Degrés Celsius (°C)",
            "Degrees Celsius (°C)",
            "°C",
            "°C",
            1.0,
            273.15
        ),
        (
            "Degrés Fahrenheit (°F)",
            "Degrees Fahrenheit (°F)",
            "°F",
            "°F",
            5.0 / 9.0,
            273.15 - 160.0 / 9.0
        ),
        ("Kelvins (K)", "Kelvins (K)", "K", "K", 1.0),
    ]
);

// Les puissances, en watts.
unites!(
    NOMS_PUISSANCES,
    PUISSANCES,
    [
        ("Watts (W)", "Watts (W)", "W", "W", 1.0),
        ("Kilowatts (kW)", "Kilowatts (kW)", "kW", "kW", 1_000.0),
        (
            "BTU par heure (BTU/h)",
            "BTU per hour (BTU/h)",
            "BTU/h",
            "BTU/h",
            0.293_071_070_172_22
        ),
        (
            "Milliers de BTU par heure (MBH)",
            "Thousands of BTU per hour (MBH)",
            "MBH",
            "MBH",
            293.071_070_172_22
        ),
        (
            "Tonnes de réfrigération (12 000 BTU/h)",
            "Tons of refrigeration (12,000 BTU/h)",
            "tonnes",
            "tons",
            3_516.852_842_066_7
        ),
        (
            "Chevaux-vapeur (HP)",
            "Horsepower (hp)",
            "HP",
            "hp",
            745.699_871_582_27
        ),
    ]
);

// Les énergies, en joules.
unites!(
    NOMS_ENERGIES,
    ENERGIES,
    [
        ("Joules (J)", "Joules (J)", "J", "J", 1.0),
        ("Kilojoules (kJ)", "Kilojoules (kJ)", "kJ", "kJ", 1_000.0),
        ("Mégajoules (MJ)", "Megajoules (MJ)", "MJ", "MJ", 1.0e6),
        ("Wattheures (Wh)", "Watt-hours (Wh)", "Wh", "Wh", 3_600.0),
        (
            "Kilowattheures (kWh)",
            "Kilowatt-hours (kWh)",
            "kWh",
            "kWh",
            3.6e6
        ),
        ("BTU", "BTU", "BTU", "BTU", 1_055.055_852_62),
        (
            "Therms (100 000 BTU)",
            "Therms (100,000 BTU)",
            "therms",
            "therms",
            105_505_585.262
        ),
        (
            "Kilocalories (kcal)",
            "Kilocalories (kcal)",
            "kcal",
            "kcal",
            4_184.0
        ),
    ]
);

// Les débits, en litres par seconde.
unites!(
    NOMS_DEBITS,
    DEBITS,
    [
        (
            "Litres par seconde (L/s)",
            "Litres per second (L/s)",
            "L/s",
            "L/s",
            1.0
        ),
        (
            "Litres par minute (L/min)",
            "Litres per minute (L/min)",
            "L/min",
            "L/min",
            1.0 / 60.0
        ),
        (
            "Mètres cubes par heure (m³/h)",
            "Cubic metres per hour (m³/h)",
            "m³/h",
            "m³/h",
            1_000.0 / 3_600.0
        ),
        (
            "Gallons US par minute",
            "US gallons per minute (US gpm)",
            "gal US/min",
            "US gpm",
            3.785_411_784 / 60.0
        ),
        (
            "Gallons impériaux par minute",
            "Imperial gallons per minute",
            "gal imp./min",
            "imp. gpm",
            4.546_09 / 60.0
        ),
        (
            "Pieds cubes par minute (pi³/min)",
            "Cubic feet per minute (CFM)",
            "pi³/min",
            "CFM",
            28.316_846_592 / 60.0
        ),
    ]
);

// Les vitesses, en mètres par seconde.
unites!(
    NOMS_VITESSES,
    VITESSES,
    [
        (
            "Mètres par seconde (m/s)",
            "Metres per second (m/s)",
            "m/s",
            "m/s",
            1.0
        ),
        (
            "Kilomètres à l'heure (km/h)",
            "Kilometres per hour (km/h)",
            "km/h",
            "km/h",
            1.0 / 3.6
        ),
        (
            "Pieds par minute (pi/min)",
            "Feet per minute (fpm)",
            "pi/min",
            "fpm",
            0.005_08
        ),
        (
            "Pieds par seconde (pi/s)",
            "Feet per second (ft/s)",
            "pi/s",
            "ft/s",
            0.304_8
        ),
        (
            "Milles à l'heure (mi/h)",
            "Miles per hour (mph)",
            "mi/h",
            "mph",
            0.447_04
        ),
    ]
);

// Les éclairements, en lux.
unites!(
    NOMS_ECLAIREMENTS,
    ECLAIREMENTS,
    [
        ("Lux (lx)", "Lux (lx)", "lx", "lx", 1.0),
        (
            "Pieds-bougies",
            "Foot-candles (fc)",
            "pieds-bougies",
            "fc",
            10.763_910_416_709_72
        ),
    ]
);

const PENTES: &[Texte] = &[
    t("Pourcentage (%)", "Percent (%)"),
    t("Degrés (°)", "Degrees (°)"),
    t("Pente au pied (x/12)", "Rise per foot (x/12)"),
    t("Rapport (1:x)", "Ratio (1:x)"),
    t(
        "Millimètres par mètre (mm/m)",
        "Millimetres per metre (mm/m)",
    ),
];

const RESISTANCES: &[Texte] = &[
    t("Valeur R (impériale)", "R-value (imperial)"),
    t("Valeur RSI (métrique)", "RSI value (metric)"),
    t("Coefficient U impérial", "Imperial U-factor"),
    t("Coefficient U métrique", "Metric U-factor"),
];

const DIMENSIONS_NOMINALES: &[Texte] = &[
    tel("1 × 2"),
    tel("1 × 3"),
    tel("1 × 4"),
    tel("1 × 6"),
    tel("1 × 8"),
    tel("1 × 10"),
    tel("1 × 12"),
    tel("2 × 2"),
    tel("2 × 3"),
    tel("2 × 4"),
    tel("2 × 6"),
    tel("2 × 8"),
    tel("2 × 10"),
    tel("2 × 12"),
    tel("4 × 4"),
    tel("4 × 6"),
    tel("6 × 6"),
];

/// Des dimensions de bois d'œuvre : nominales et réelles (pouces), métriques
/// (millimètres).
type DimensionsDuBois = ((f64, f64), (f64, f64), (f64, f64));

/// Le bois d'œuvre sec et raboté (norme CSA O141) : ses dimensions nominales
/// (pouces), réelles (pouces) et métriques (millimètres), dans l'ordre de
/// `DIMENSIONS_NOMINALES`.
const BOIS: [DimensionsDuBois; 17] = [
    ((1.0, 2.0), (0.75, 1.5), (19.0, 38.0)),
    ((1.0, 3.0), (0.75, 2.5), (19.0, 64.0)),
    ((1.0, 4.0), (0.75, 3.5), (19.0, 89.0)),
    ((1.0, 6.0), (0.75, 5.5), (19.0, 140.0)),
    ((1.0, 8.0), (0.75, 7.25), (19.0, 184.0)),
    ((1.0, 10.0), (0.75, 9.25), (19.0, 235.0)),
    ((1.0, 12.0), (0.75, 11.25), (19.0, 286.0)),
    ((2.0, 2.0), (1.5, 1.5), (38.0, 38.0)),
    ((2.0, 3.0), (1.5, 2.5), (38.0, 64.0)),
    ((2.0, 4.0), (1.5, 3.5), (38.0, 89.0)),
    ((2.0, 6.0), (1.5, 5.5), (38.0, 140.0)),
    ((2.0, 8.0), (1.5, 7.25), (38.0, 184.0)),
    ((2.0, 10.0), (1.5, 9.25), (38.0, 235.0)),
    ((2.0, 12.0), (1.5, 11.25), (38.0, 286.0)),
    ((4.0, 4.0), (3.5, 3.5), (89.0, 89.0)),
    ((4.0, 6.0), (3.5, 5.5), (89.0, 140.0)),
    ((6.0, 6.0), (5.5, 5.5), (140.0, 140.0)),
];

const VALEUR: Champ = Champ::nombre(t("Valeur", "Value"), Texte::VIDE).nul_permis();

/// Le champ de l'unité de départ d'une grandeur.
const fn unite_de_depart(noms: &'static [Texte]) -> Champ {
    Champ::choix(t("Unité de départ", "From unit"), noms).court(t("Unité", "Unit"))
}

/// Un convertisseur d'une grandeur, sans note.
const fn convertisseur(
    id: &'static str,
    nom: Texte,
    resume: Texte,
    champs: &'static [Champ],
    calcul: fn(&Lues) -> Calcul,
) -> Calculateur {
    Calculateur {
        id,
        metier: Metier::Conversions,
        nom,
        resume,
        champs,
        note: Texte::VIDE,
        calcul,
    }
}

pub const CALCULATEURS: &[Calculateur] = &[
    convertisseur(
        "conversion-longueurs",
        t("Longueurs", "Lengths"),
        t(
            "Pouces, pieds, verges, milles, millimètres, centimètres, mètres, kilomètres.",
            "Inches, feet, yards, miles, millimetres, centimetres, metres, kilometres.",
        ),
        &[VALEUR, unite_de_depart(NOMS_LONGUEURS)],
        longueurs,
    ),
    convertisseur(
        "conversion-aires",
        t("Aires", "Areas"),
        t(
            "Pieds et verges carrés, carrés de toiture, acres, mètres carrés, hectares.",
            "Square feet and yards, roofing squares, acres, square metres, hectares.",
        ),
        &[VALEUR, unite_de_depart(NOMS_AIRES)],
        aires,
    ),
    convertisseur(
        "conversion-volumes",
        t("Volumes", "Volumes"),
        t(
            "Pieds et verges cubes, pieds-mesure de planche, litres, mètres cubes.",
            "Cubic feet and yards, board feet, litres, cubic metres.",
        ),
        &[VALEUR, unite_de_depart(NOMS_VOLUMES)],
        volumes,
    ),
    Calculateur {
        note: t(
            "Le gallon US (3,785 L) est celui des fiches techniques; le gallon impérial (4,546 L), celui du Canada d'autrefois.",
            "The US gallon (3.785 L) is the one on spec sheets; the imperial gallon (4.546 L), Canada's former one.",
        ),
        ..convertisseur(
            "conversion-liquides",
            t("Liquides", "Liquids"),
            t(
                "Litres, gallons US et impériaux, onces liquides, pieds cubes.",
                "Litres, US and imperial gallons, fluid ounces, cubic feet.",
            ),
            &[VALEUR, unite_de_depart(NOMS_LIQUIDES)],
            liquides,
        )
    },
    convertisseur(
        "conversion-masses",
        t("Masses", "Weights"),
        t(
            "Onces, livres, tonnes courtes, grammes, kilogrammes, tonnes métriques.",
            "Ounces, pounds, short tons, grams, kilograms, metric tonnes.",
        ),
        &[VALEUR, unite_de_depart(NOMS_MASSES)],
        masses,
    ),
    convertisseur(
        "conversion-masse-volumique",
        t("Masse volumique", "Density"),
        t(
            "Kilogrammes par mètre cube, livres par pied cube ou par verge cube.",
            "Kilograms per cubic metre, pounds per cubic foot or per cubic yard.",
        ),
        &[VALEUR, unite_de_depart(NOMS_MASSES_VOLUMIQUES)],
        masses_volumiques,
    ),
    Calculateur {
        note: t(
            "Colonne d'eau à 4 °C, mercure à 0 °C. Les charges de neige et de plancher se donnent en kPa ou en lb/pi².",
            "Water column at 4 °C, mercury at 0 °C. Snow and floor loads are given in kPa or in psf.",
        ),
        ..convertisseur(
            "conversion-pressions",
            t("Pressions et charges", "Pressure and loads"),
            t(
                "Kilopascals, lb/po², bars, lb/pi² des charges, colonne d'eau, mercure.",
                "Kilopascals, psi, bars, psf for loads, water column, mercury.",
            ),
            &[VALEUR, unite_de_depart(NOMS_PRESSIONS)],
            pressions,
        )
    },
    convertisseur(
        "conversion-forces",
        t("Forces", "Forces"),
        t(
            "Newtons, kilonewtons, livres-force, kips, kilogrammes-force.",
            "Newtons, kilonewtons, pounds-force, kips, kilograms-force.",
        ),
        &[VALEUR, unite_de_depart(NOMS_FORCES)],
        forces,
    ),
    convertisseur(
        "conversion-charges-lineaires",
        t("Charges linéaires", "Line loads"),
        t(
            "La charge sur une poutre ou un mur : kN/m, lb/pi, kip/pi.",
            "The load on a beam or a wall: kN/m, plf, klf.",
        ),
        &[VALEUR, unite_de_depart(NOMS_CHARGES_LINEAIRES)],
        charges_lineaires,
    ),
    convertisseur(
        "conversion-couples",
        t("Couples de serrage", "Torque"),
        t(
            "Le serrage d'un boulon ou le moment d'une poutre : N·m, lb·pi, lb·po.",
            "Bolt torque or beam moment: N·m, lb·ft, lb·in.",
        ),
        &[VALEUR, unite_de_depart(NOMS_COUPLES)],
        couples,
    ),
    Calculateur {
        note: t(
            "Pour un écart de température, 1 °C d'écart vaut 1,8 °F d'écart.",
            "For a temperature difference, a 1 °C difference equals a 1.8 °F difference.",
        ),
        ..convertisseur(
            "conversion-temperatures",
            t("Températures", "Temperatures"),
            t(
                "Degrés Celsius, degrés Fahrenheit, kelvins.",
                "Degrees Celsius, degrees Fahrenheit, kelvins.",
            ),
            &[VALEUR.negatif_permis(), unite_de_depart(NOMS_TEMPERATURES)],
            temperatures,
        )
    },
    Calculateur {
        note: t(
            "BTU international; cheval-vapeur mécanique (745,7 W).",
            "International BTU; mechanical horsepower (745.7 W).",
        ),
        ..convertisseur(
            "conversion-puissances",
            t("Puissances", "Power"),
            t(
                "Watts, kilowatts, BTU/h, MBH, tonnes de réfrigération, chevaux-vapeur.",
                "Watts, kilowatts, BTU/h, MBH, tons of refrigeration, horsepower.",
            ),
            &[VALEUR, unite_de_depart(NOMS_PUISSANCES)],
            puissances,
        )
    },
    convertisseur(
        "conversion-energies",
        t("Énergies", "Energy"),
        t(
            "Joules, kilowattheures, BTU, therms, kilocalories.",
            "Joules, kilowatt-hours, BTU, therms, kilocalories.",
        ),
        &[VALEUR, unite_de_depart(NOMS_ENERGIES)],
        energies,
    ),
    convertisseur(
        "conversion-debits",
        t("Débits", "Flow rates"),
        t(
            "L'eau et l'air : L/s, L/min, m³/h, gallons par minute, pi³/min.",
            "Water and air: L/s, L/min, m³/h, gallons per minute, CFM.",
        ),
        &[VALEUR, unite_de_depart(NOMS_DEBITS)],
        debits,
    ),
    convertisseur(
        "conversion-vitesses",
        t("Vitesses", "Speeds"),
        t(
            "Mètres par seconde, km/h, pieds par minute (l'air d'un conduit), mi/h.",
            "Metres per second, km/h, feet per minute (duct air), mph.",
        ),
        &[VALEUR, unite_de_depart(NOMS_VITESSES)],
        vitesses,
    ),
    Calculateur {
        note: t(
            "x/12 : les pouces de montée par pied de course, comme une pente de toit (4/12). 1:x : x de course pour 1 de montée, comme une rampe (1:12).",
            "x/12: inches of rise per foot of run, like a roof pitch (4/12). 1:x: x of run for 1 of rise, like a ramp (1:12).",
        ),
        ..convertisseur(
            "conversion-pentes",
            t("Pentes", "Slopes"),
            t(
                "Pourcentage, degrés, pente au pied (x/12), rapport 1:x, mm/m.",
                "Percent, degrees, rise per foot (x/12), 1:x ratio, mm/m.",
            ),
            &[
                VALEUR,
                Champ::choix(t("Unité de départ", "From unit"), PENTES).court(t("Unité", "Unit")),
            ],
            pente,
        )
    },
    Calculateur {
        note: t(
            "La valeur R vaut 5,678 fois la valeur RSI; chaque coefficient U est l'inverse de sa valeur R ou RSI.",
            "The R-value is 5.678 times the RSI value; each U-factor is the inverse of its R or RSI value.",
        ),
        ..convertisseur(
            "conversion-isolation",
            t("Isolation thermique", "Thermal insulation"),
            t(
                "Valeur R, valeur RSI, coefficients U impérial et métrique.",
                "R-value, RSI value, imperial and metric U-factors.",
            ),
            &[
                Champ::nombre(t("Valeur", "Value"), Texte::VIDE),
                Champ::choix(t("Unité de départ", "From unit"), RESISTANCES)
                    .court(t("Unité", "Unit")),
            ],
            isolation,
        )
    },
    convertisseur(
        "conversion-eclairement",
        t("Éclairement", "Illuminance"),
        t("Lux et pieds-bougies.", "Lux and foot-candles."),
        &[VALEUR, unite_de_depart(NOMS_ECLAIREMENTS)],
        eclairements,
    ),
    Calculateur {
        note: t(
            "Bois sec et raboté (norme CSA O141). Les pieds-mesure de planche se comptent aux dimensions nominales.",
            "Dry, dressed lumber (CSA O141). Board feet are counted on the nominal size.",
        ),
        ..convertisseur(
            "bois-d-oeuvre",
            t("Bois d'œuvre", "Dimension lumber"),
            t(
                "Les dimensions réelles d'un 2 × 4, d'un 2 × 6…, en pouces et en millimètres, et ses pieds-mesure de planche.",
                "The actual size of a 2 × 4, a 2 × 6…, in inches and millimetres, and its board feet.",
            ),
            &[
                Champ::choix(
                    t("Dimension nominale", "Nominal size"),
                    DIMENSIONS_NOMINALES,
                )
                .defaut("9")
                .court(t("Dimension", "Size")),
                Champ::longueur(t("Longueur", "Length"), Format::PiPo).facultatif(),
                Champ::entier(t("Nombre de pièces", "Number of pieces"), Texte::VIDE)
                    .defaut("1")
                    .facultatif()
                    .court(t("Pièces", "Pieces")),
            ],
            bois,
        )
    },
];

/// La valeur du champ 0, dans l'unité choisie au champ 1, écrite dans
/// chacune des unités de sa grandeur.
fn convertir(e: &Lues, noms: &'static [Texte], unites: &[Unite]) -> Calcul {
    let de = &unites[e.choix(1).min(unites.len() - 1)];
    let base = e.nombre(0) * de.facteur + de.decalage;
    Ok(noms
        .iter()
        .zip(unites)
        .map(|(nom, unite)| {
            Resultat::nouveau(
                *nom,
                Quantite::Chiffres((base - unite.decalage) / unite.facteur, unite.symbole),
            )
        })
        .collect())
}

fn longueurs(e: &Lues) -> Calcul {
    convertir(e, NOMS_LONGUEURS, LONGUEURS)
}

fn aires(e: &Lues) -> Calcul {
    convertir(e, NOMS_AIRES, AIRES)
}

fn volumes(e: &Lues) -> Calcul {
    convertir(e, NOMS_VOLUMES, VOLUMES)
}

fn liquides(e: &Lues) -> Calcul {
    convertir(e, NOMS_LIQUIDES, LIQUIDES)
}

fn masses(e: &Lues) -> Calcul {
    convertir(e, NOMS_MASSES, MASSES)
}

fn masses_volumiques(e: &Lues) -> Calcul {
    convertir(e, NOMS_MASSES_VOLUMIQUES, MASSES_VOLUMIQUES)
}

fn pressions(e: &Lues) -> Calcul {
    convertir(e, NOMS_PRESSIONS, PRESSIONS)
}

fn forces(e: &Lues) -> Calcul {
    convertir(e, NOMS_FORCES, FORCES)
}

fn charges_lineaires(e: &Lues) -> Calcul {
    convertir(e, NOMS_CHARGES_LINEAIRES, CHARGES_LINEAIRES)
}

fn couples(e: &Lues) -> Calcul {
    convertir(e, NOMS_COUPLES, COUPLES)
}

fn temperatures(e: &Lues) -> Calcul {
    // Rien sous le zéro absolu.
    let de = &TEMPERATURES[e.choix(1).min(TEMPERATURES.len() - 1)];
    if e.nombre(0) * de.facteur + de.decalage < -EPS {
        return Err(t(
            "Cette température est sous le zéro absolu (−273,15 °C).",
            "This temperature is below absolute zero (−273.15 °C).",
        ));
    }
    convertir(e, NOMS_TEMPERATURES, TEMPERATURES)
}

fn puissances(e: &Lues) -> Calcul {
    convertir(e, NOMS_PUISSANCES, PUISSANCES)
}

fn energies(e: &Lues) -> Calcul {
    convertir(e, NOMS_ENERGIES, ENERGIES)
}

fn debits(e: &Lues) -> Calcul {
    convertir(e, NOMS_DEBITS, DEBITS)
}

fn vitesses(e: &Lues) -> Calcul {
    convertir(e, NOMS_VITESSES, VITESSES)
}

fn eclairements(e: &Lues) -> Calcul {
    convertir(e, NOMS_ECLAIREMENTS, ECLAIREMENTS)
}

/// Une pente, de l'une de ses écritures à toutes les autres; elle passe par
/// le rapport de la montée à la course.
fn pente(e: &Lues) -> Calcul {
    let x = e.nombre(0);
    let rapport = match e.choix(1) {
        0 => x / 100.0,
        1 => {
            if x >= 90.0 - EPS {
                return Err(t(
                    "Un angle de 90° ou plus n'est pas une pente.",
                    "An angle of 90° or more is not a slope.",
                ));
            }
            x.to_radians().tan()
        }
        2 => x / PO_PAR_PI,
        3 => {
            if x <= EPS {
                return Err(t(
                    "Un rapport 1:0 n'existe pas.",
                    "A 1:0 ratio does not exist.",
                ));
            }
            1.0 / x
        }
        _ => x / 1_000.0,
    };
    let mut resultats = vec![
        Resultat::nouveau(
            t("Pourcentage", "Percent"),
            Quantite::Chiffres(rapport * 100.0, tel("%")),
        ),
        Resultat::nouveau(
            t("Angle", "Angle"),
            Quantite::Chiffres(rapport.atan().to_degrees(), tel("°")),
        ),
        Resultat::nouveau(
            t("Pente au pied (x/12)", "Rise per foot (x/12)"),
            Quantite::Chiffres(rapport * PO_PAR_PI, t("po/pi", "in/ft")),
        ),
    ];
    // Une pente nulle n'a pas de rapport 1:x.
    if rapport > EPS {
        resultats.push(Resultat::nouveau(
            t("Rapport (1:x)", "Ratio (1:x)"),
            Quantite::Chiffres(1.0 / rapport, Texte::VIDE),
        ));
    }
    resultats.push(Resultat::nouveau(
        t("Millimètres par mètre", "Millimetres per metre"),
        Quantite::Chiffres(rapport * 1_000.0, tel("mm/m")),
    ));
    Ok(resultats)
}

/// Une résistance thermique, de l'une de ses écritures à toutes les autres;
/// elle passe par la valeur RSI.
fn isolation(e: &Lues) -> Calcul {
    use super::interieurs::R_PAR_RSI;
    let x = e.nombre(0);
    let rsi = match e.choix(1) {
        0 => x / R_PAR_RSI,
        1 => x,
        2 => 1.0 / (x * R_PAR_RSI),
        _ => 1.0 / x,
    };
    Ok(vec![
        Resultat::nouveau(
            RESISTANCES[0],
            Quantite::Chiffres(rsi * R_PAR_RSI, Texte::VIDE),
        ),
        Resultat::nouveau(RESISTANCES[1], Quantite::Chiffres(rsi, Texte::VIDE)),
        Resultat::nouveau(
            RESISTANCES[2],
            Quantite::Chiffres(1.0 / (rsi * R_PAR_RSI), t("BTU/h·pi²·°F", "BTU/h·ft²·°F")),
        ),
        Resultat::nouveau(RESISTANCES[3], Quantite::Chiffres(1.0 / rsi, tel("W/m²·K"))),
    ])
}

/// Les dimensions réelles d'une pièce de bois d'œuvre, et ses pieds-mesure de
/// planche si sa longueur est donnée.
fn bois(e: &Lues) -> Calcul {
    let ((epaisseur_nominale, largeur_nominale), (epaisseur, largeur), (epaisseur_mm, largeur_mm)) =
        BOIS[e.choix(0).min(BOIS.len() - 1)];
    let pmp_au_pied = epaisseur_nominale * largeur_nominale / PO_PAR_PI;
    let mut resultats = vec![
        Resultat::nouveau(
            t("Épaisseur réelle", "Actual thickness"),
            Quantite::Longueur(epaisseur),
        ),
        Resultat::nouveau(
            t("Largeur réelle", "Actual width"),
            Quantite::Longueur(largeur),
        ),
        Resultat::nouveau(
            t("Épaisseur en millimètres", "Thickness in millimetres"),
            Quantite::Fixe(epaisseur_mm, 0, tel("mm")),
        ),
        Resultat::nouveau(
            t("Largeur en millimètres", "Width in millimetres"),
            Quantite::Fixe(largeur_mm, 0, tel("mm")),
        ),
        Resultat::nouveau(
            t("Pieds-mesure de planche au pied", "Board feet per foot"),
            Quantite::Chiffres(pmp_au_pied, t("pmp/pi", "fbm/ft")),
        ),
    ];
    if e.presente(1) {
        let pieds = e.longueur(1) / PO_PAR_PI;
        let pieces = if e.presente(2) { e.nombre(2) } else { 1.0 };
        resultats.push(Resultat::nouveau(
            t(
                "Pieds-mesure de planche d'une pièce",
                "Board feet per piece",
            ),
            Quantite::Chiffres(pmp_au_pied * pieds, t("pmp", "fbm")),
        ));
        if pieces > 1.0 + EPS {
            resultats.push(Resultat::nouveau(
                t("Pieds-mesure de planche en tout", "Total board feet"),
                Quantite::Chiffres(pmp_au_pied * pieds * pieces, t("pmp", "fbm")),
            ));
        }
    }
    Ok(resultats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metiers::calculateur;
    use crate::prefs::Preferences;

    /// Le résultat de rang `rang` d'un convertisseur, en nombre.
    fn resultat(id: &str, saisies: &[&str], rang: usize) -> f64 {
        let c = calculateur(id).unwrap();
        let saisies: Vec<String> = saisies.iter().map(|s| s.to_string()).collect();
        let e = c.evaluer(&saisies, &Preferences::default());
        assert!(
            e.erreurs.iter().all(Option::is_none),
            "{id} : {:?}",
            e.erreurs
        );
        assert_eq!(e.avis, None, "{id}");
        e.quantites[rang].valeur(e.modele).base
    }

    fn proche(a: f64, b: f64, tolerance: f64) -> bool {
        (a - b).abs() <= tolerance * b.abs().max(1e-12)
    }

    #[test]
    fn chaque_grandeur_a_autant_de_noms_que_d_unites() {
        for (noms, unites) in [
            (NOMS_LONGUEURS, LONGUEURS),
            (NOMS_AIRES, AIRES),
            (NOMS_VOLUMES, VOLUMES),
            (NOMS_LIQUIDES, LIQUIDES),
            (NOMS_MASSES, MASSES),
            (NOMS_MASSES_VOLUMIQUES, MASSES_VOLUMIQUES),
            (NOMS_PRESSIONS, PRESSIONS),
            (NOMS_FORCES, FORCES),
            (NOMS_CHARGES_LINEAIRES, CHARGES_LINEAIRES),
            (NOMS_COUPLES, COUPLES),
            (NOMS_TEMPERATURES, TEMPERATURES),
            (NOMS_PUISSANCES, PUISSANCES),
            (NOMS_ENERGIES, ENERGIES),
            (NOMS_DEBITS, DEBITS),
            (NOMS_VITESSES, VITESSES),
            (NOMS_ECLAIREMENTS, ECLAIREMENTS),
        ] {
            assert_eq!(noms.len(), unites.len());
        }
        assert_eq!(DIMENSIONS_NOMINALES.len(), BOIS.len());
    }

    #[test]
    fn les_valeurs_d_essai_des_conversions() {
        // 12 pi = 3,658 m; 1 mille = 5 280 pi.
        assert!(proche(
            resultat("conversion-longueurs", &["12", "1"], 6),
            3.6576,
            1e-9
        ));
        assert!(proche(
            resultat("conversion-longueurs", &["1", "3"], 1),
            5_280.0,
            1e-9
        ));
        // 1 000 pi² = 10 carrés de toiture = 92,90 m².
        assert!(proche(
            resultat("conversion-aires", &["1000", "1"], 3),
            10.0,
            1e-9
        ));
        assert!(proche(
            resultat("conversion-aires", &["1000", "1"], 6),
            92.903_04,
            1e-9
        ));
        // 1 vg³ = 27 pi³ = 0,7646 m³; 1 pmp = 144 po³.
        assert!(proche(
            resultat("conversion-volumes", &["1", "2"], 1),
            27.0,
            1e-9
        ));
        assert!(proche(
            resultat("conversion-volumes", &["1", "3"], 0),
            144.0,
            1e-9
        ));
        // Un chauffe-eau de 40 gal US : 151,4 L.
        assert!(proche(
            resultat("conversion-liquides", &["40", "4"], 1),
            151.416,
            1e-4
        ));
        // 50 lb = 22,68 kg.
        assert!(proche(
            resultat("conversion-masses", &["50", "1"], 4),
            22.679_6,
            1e-5
        ));
        // Le béton, 2 400 kg/m³ = 149,8 lb/pi³.
        assert!(proche(
            resultat("conversion-masse-volumique", &["2400", "0"], 2),
            149.83,
            1e-4
        ));
        // 60 lb/po² = 413,7 kPa; 2,4 kPa de neige = 50,1 lb/pi².
        assert!(proche(
            resultat("conversion-pressions", &["60", "4"], 0),
            413.685,
            1e-5
        ));
        assert!(proche(
            resultat("conversion-pressions", &["2,4", "0"], 5),
            50.125,
            1e-4
        ));
        // 1 po CE = 249,1 Pa.
        assert!(proche(
            resultat("conversion-pressions", &["1", "8"], 2),
            249.088_9,
            1e-9
        ));
        // 10 kN = 2 248 lbf; 500 lb/pi = 7,297 kN/m; 100 lb·pi = 135,6 N·m.
        assert!(proche(
            resultat("conversion-forces", &["10", "1"], 2),
            2_248.09,
            1e-5
        ));
        assert!(proche(
            resultat("conversion-charges-lineaires", &["500", "2"], 0),
            7.296_95,
            1e-5
        ));
        assert!(proche(
            resultat("conversion-couples", &["100", "2"], 0),
            135.582,
            1e-5
        ));
        // −20 °C = −4 °F; 212 °F = 100 °C; −40 est le même des deux côtés.
        assert!(proche(
            resultat("conversion-temperatures", &["-20", "0"], 1),
            -4.0,
            1e-9
        ));
        assert!(proche(
            resultat("conversion-temperatures", &["212", "1"], 0),
            100.0,
            1e-9
        ));
        assert!(proche(
            resultat("conversion-temperatures", &["-40", "1"], 0),
            -40.0,
            1e-9
        ));
        // 3 tonnes de réfrigération = 36 000 BTU/h = 10,55 kW.
        assert!(proche(
            resultat("conversion-puissances", &["3", "4"], 2),
            36_000.0,
            1e-9
        ));
        assert!(proche(
            resultat("conversion-puissances", &["3", "4"], 1),
            10.550_6,
            1e-5
        ));
        // 100 kWh = 341 214 BTU.
        assert!(proche(
            resultat("conversion-energies", &["100", "4"], 5),
            341_214.16,
            1e-6
        ));
        // 10 gal US/min = 0,6309 L/s; 1 000 pi³/min = 471,9 L/s.
        assert!(proche(
            resultat("conversion-debits", &["10", "3"], 0),
            0.630_902,
            1e-5
        ));
        assert!(proche(
            resultat("conversion-debits", &["1000", "5"], 0),
            471.947,
            1e-5
        ));
        // 800 pi/min = 4,064 m/s.
        assert!(proche(
            resultat("conversion-vitesses", &["800", "2"], 0),
            4.064,
            1e-9
        ));
        // 50 pieds-bougies = 538,2 lux.
        assert!(proche(
            resultat("conversion-eclairement", &["50", "1"], 0),
            538.195_5,
            1e-6
        ));
    }

    #[test]
    fn une_pente_dans_toutes_ses_ecritures() {
        // 4/12 : 33,33 %, 18,43°, 1:3, 333,3 mm/m.
        assert!(proche(
            resultat("conversion-pentes", &["4", "2"], 0),
            33.333_3,
            1e-5
        ));
        assert!(proche(
            resultat("conversion-pentes", &["4", "2"], 1),
            18.434_9,
            1e-5
        ));
        assert!(proche(
            resultat("conversion-pentes", &["4", "2"], 3),
            3.0,
            1e-9
        ));
        // 45° : 100 %, 12/12.
        assert!(proche(
            resultat("conversion-pentes", &["45", "1"], 2),
            12.0,
            1e-9
        ));
        // Une rampe de 1:12 : 8,33 %.
        assert!(proche(
            resultat("conversion-pentes", &["12", "3"], 0),
            8.333_3,
            1e-5
        ));
        // Un drain de 1/4 po au pied, soit 2,08 %.
        assert!(proche(
            resultat("conversion-pentes", &["0,25", "2"], 0),
            2.083_3,
            1e-4
        ));
        // 90° n'est pas une pente; une pente nulle n'a pas de rapport 1:x.
        let c = calculateur("conversion-pentes").unwrap();
        let prefs = Preferences::default();
        let e = c.evaluer(&["90".to_string(), "1".to_string()], &prefs);
        assert!(e.avis.is_some() && e.resultats.is_empty());
        let e = c.evaluer(&["0".to_string(), "0".to_string()], &prefs);
        assert_eq!(e.resultats.len(), 4);
    }

    #[test]
    fn isolation_et_bois_d_oeuvre() {
        // R-20 = RSI 3,52; U = 0,05 en impérial, 0,284 en métrique.
        assert!(proche(
            resultat("conversion-isolation", &["20", "0"], 1),
            3.522_2,
            1e-4
        ));
        assert!(proche(
            resultat("conversion-isolation", &["20", "0"], 2),
            0.05,
            1e-9
        ));
        assert!(proche(
            resultat("conversion-isolation", &["20", "0"], 3),
            0.283_9,
            1e-3
        ));
        // Un 2 × 4 : 1 1/2 po × 3 1/2 po, 38 mm × 89 mm; 8 pi : 5,333 pmp.
        assert!(proche(
            resultat("bois-d-oeuvre", &["9", "8", "10"], 0),
            1.5,
            1e-9
        ));
        assert!(proche(
            resultat("bois-d-oeuvre", &["9", "8", "10"], 1),
            3.5,
            1e-9
        ));
        assert!(proche(
            resultat("bois-d-oeuvre", &["9", "8", "10"], 3),
            89.0,
            1e-9
        ));
        assert!(proche(
            resultat("bois-d-oeuvre", &["9", "8", "10"], 5),
            16.0 / 3.0,
            1e-9
        ));
        assert!(proche(
            resultat("bois-d-oeuvre", &["9", "8", "10"], 6),
            160.0 / 3.0,
            1e-9
        ));
        // Sous le zéro absolu : refusé.
        let c = calculateur("conversion-temperatures").unwrap();
        let e = c.evaluer(
            &["-300".to_string(), "0".to_string()],
            &Preferences::default(),
        );
        assert!(e.avis.is_some());
    }
}
