//! Plomberie (section 22.5).

use std::f64::consts::PI;

use super::{
    Calcul, Calculateur, Champ, Lues, Metier, PO3_PAR_GAL_US, PO3_PAR_L, Quantite, Resultat,
};
use crate::langue::{Texte, t, tel};
use crate::unites::{Format, KG_PAR_LB, PO_PAR_M, PO_PAR_PI};

/// Angles des raccords courants, et leur valeur en degrés.
const ANGLES: &[Texte] = &[
    t("11,25°", "11.25°"),
    t("22,5°", "22.5°"),
    tel("30°"),
    tel("45°"),
    tel("60°"),
];
const DEGRES: [f64; 5] = [11.25, 22.5, 30.0, 45.0, 60.0];
/// Rang de 45° dans la liste : l'angle proposé au départ.
const ANGLE_PAR_DEFAUT: &str = "3";

/// Pression d'un mètre d'eau, en kilopascals (masse volumique de 1 000 kg/m³).
const KPA_PAR_METRE_D_EAU: f64 = 9.806_65;
/// Kilopascals par livre au pouce carré.
const KPA_PAR_PSI: f64 = 6.894_757_293_168_361;
/// Chaleur massique de l'eau, en kilojoules par kilogramme et par degré Celsius.
const KJ_PAR_KG_ET_DEGRE: f64 = 4.186;
const UNITES_DE_DEBIT: &[Texte] = &[t("gal US/min", "US gal/min"), tel("L/min"), tel("L/s")];

const NOTE_DES_DEPORTS: Texte = t(
    "Mesures de centre à centre : déduisez l'emboîtement des raccords.",
    "Centre-to-centre measurements: deduct the fitting allowance.",
);

pub const CALCULATEURS: &[Calculateur] = &[
    Calculateur {
        id: "pente-de-drain",
        metier: Metier::Plomberie,
        nom: t("Pente d'un drain", "Drain slope"),
        resume: t(
            "Le dénivelé d'un tuyau posé en pente régulière.",
            "The total drop of a pipe laid at a constant slope.",
        ),
        champs: &[
            Champ::longueur(t("Longueur du tuyau", "Pipe length"), Format::PiPo)
                .court(t("Longueur", "Length")),
            Champ::longueur(
                t("Pente, par pied de longueur", "Slope per foot of run"),
                Format::PoFrac,
            )
            .court(t("Pente\npar pied", "Slope\nper foot"))
            .defaut("1/4"),
        ],
        note: t(
            "Le Code fixe la pente minimale selon le diamètre du tuyau.",
            "The Code sets the minimum slope according to the pipe diameter.",
        ),
        calcul: pente,
    },
    Calculateur {
        id: "deport",
        metier: Metier::Plomberie,
        nom: t("Déport de tuyauterie", "Pipe offset"),
        resume: t(
            "Le tronçon incliné qui relie deux tuyaux parallèles.",
            "The angled run that connects two parallel pipes.",
        ),
        champs: &[
            Champ::longueur(t("Déport", "Offset"), Format::PoFrac),
            Champ::choix(t("Angle des raccords", "Fitting angle"), ANGLES)
                .court(tel("Angle"))
                .defaut(ANGLE_PAR_DEFAUT),
        ],
        note: NOTE_DES_DEPORTS,
        calcul: deport,
    },
    Calculateur {
        id: "deport-roule",
        metier: Metier::Plomberie,
        nom: t("Déport roulé", "Rolling offset"),
        resume: t(
            "Un déport à la fois horizontal et vertical.",
            "An offset that is both horizontal and vertical.",
        ),
        champs: &[
            Champ::longueur(t("Déport horizontal", "Horizontal offset"), Format::PoFrac)
                .court(t("Déport\nhorizontal", "Horizontal\noffset")),
            Champ::longueur(t("Déport vertical", "Vertical offset"), Format::PoFrac)
                .court(t("Déport\nvertical", "Vertical\noffset")),
            Champ::choix(t("Angle des raccords", "Fitting angle"), ANGLES)
                .court(tel("Angle"))
                .defaut(ANGLE_PAR_DEFAUT),
        ],
        note: NOTE_DES_DEPORTS,
        calcul: deport_roule,
    },
    Calculateur {
        id: "contenance-de-tuyau",
        metier: Metier::Plomberie,
        nom: t("Contenance d'un tuyau", "Pipe capacity"),
        resume: t(
            "Le volume d'eau que contient un tuyau plein, et sa masse.",
            "The volume of water in a full pipe, and its weight.",
        ),
        champs: &[
            Champ::longueur(t("Diamètre intérieur", "Inside diameter"), Format::PoFrac)
                .court(t("Diamètre", "Diameter")),
            Champ::longueur(t("Longueur", "Length"), Format::PiPo),
        ],
        note: Texte::VIDE,
        calcul: contenance_de_tuyau,
    },
    Calculateur {
        id: "pression-d-eau",
        metier: Metier::Plomberie,
        nom: t("Pression d'une colonne d'eau", "Pressure of a water column"),
        resume: t(
            "La pression statique au bas d'une hauteur d'eau.",
            "The static pressure at the bottom of a height of water.",
        ),
        champs: &[
            Champ::longueur(t("Hauteur d'eau", "Water height"), Format::PiPo)
                .court(t("Hauteur", "Height")),
        ],
        note: t(
            "Eau froide au repos : la pression baisse dès que l'eau circule.",
            "Cold water at rest: the pressure drops as soon as the water flows.",
        ),
        calcul: pression,
    },
    Calculateur {
        id: "reservoir",
        metier: Metier::Plomberie,
        nom: t("Réservoir cylindrique", "Cylindrical tank"),
        resume: t(
            "La contenance d'un réservoir rond.",
            "The capacity of a round tank.",
        ),
        champs: &[
            Champ::longueur(t("Diamètre intérieur", "Inside diameter"), Format::PoFrac)
                .court(t("Diamètre", "Diameter")),
            Champ::longueur(t("Hauteur ou longueur", "Height or length"), Format::PoFrac)
                .court(t("Hauteur", "Height")),
        ],
        note: Texte::VIDE,
        calcul: reservoir,
    },
    Calculateur {
        id: "debit-et-vitesse",
        metier: Metier::Plomberie,
        nom: t("Débit et vitesse de l'eau", "Water flow and velocity"),
        resume: t(
            "La vitesse de l'eau dans un tuyau, d'après son diamètre intérieur et le débit.",
            "The velocity of water in a pipe, from its inside diameter and the flow rate.",
        ),
        champs: &[
            Champ::longueur(t("Diamètre intérieur", "Inside diameter"), Format::PoFrac)
                .court(t("Diamètre", "Diameter")),
            Champ::nombre(t("Débit", "Flow rate"), Texte::VIDE),
            Champ::choix(t("Unité du débit", "Flow unit"), UNITES_DE_DEBIT)
                .court(t("Unité", "Unit")),
        ],
        note: t(
            "Une vitesse élevée cause du bruit et de l'usure : comparez-la à la limite recommandée pour le matériau du tuyau.",
            "High velocity causes noise and wear: compare it with the limit recommended for the pipe material.",
        ),
        calcul: debit_et_vitesse,
    },
    Calculateur {
        id: "longueur-de-coupe",
        metier: Metier::Plomberie,
        nom: t("Longueur de coupe d'un tuyau", "Pipe cut length"),
        resume: t(
            "La longueur à couper entre deux raccords, d'après la mesure de centre à centre.",
            "The length to cut between two fittings, from the centre-to-centre measurement.",
        ),
        champs: &[
            Champ::longueur(
                t("Mesure de centre à centre", "Centre-to-centre measurement"),
                Format::PoFrac,
            )
            .court(t("Centre\nà centre", "Centre\nto centre")),
            Champ::longueur(
                t("Retrait du premier raccord", "First fitting take-off"),
                Format::PoFrac,
            )
            .nul_permis()
            .court(t("Retrait 1", "Take-off 1")),
            Champ::longueur(
                t("Retrait du second raccord", "Second fitting take-off"),
                Format::PoFrac,
            )
            .nul_permis()
            .court(t("Retrait 2", "Take-off 2")),
            Champ::longueur(
                t("Emboîtement à chaque bout", "Engagement at each end"),
                Format::PoFrac,
            )
            .defaut("0")
            .nul_permis()
            .court(t("Emboîtement", "Engagement")),
        ],
        note: t(
            "Le retrait va du centre du raccord à sa face; l'emboîtement est la profondeur où le tuyau entre dans le raccord. Les deux figurent dans les tables du fabricant.",
            "The take-off runs from the centre of the fitting to its face; the engagement is how far the pipe goes into the fitting. Both are listed in the manufacturer's tables.",
        ),
        calcul: longueur_de_coupe,
    },
    Calculateur {
        id: "deports-paralleles",
        metier: Metier::Plomberie,
        nom: t("Déports parallèles", "Parallel offsets"),
        resume: t(
            "Des tuyaux côte à côte qui dévient ensemble, en gardant le même écart.",
            "Side-by-side pipes that offset together and keep the same spacing.",
        ),
        champs: &[
            Champ::longueur(t("Déport", "Offset"), Format::PoFrac),
            Champ::longueur(
                t(
                    "Écart entre les tuyaux, de centre à centre",
                    "Pipe spacing, centre to centre",
                ),
                Format::PoFrac,
            )
            .court(t("Écart", "Spacing")),
            Champ::choix(t("Angle des raccords", "Fitting angle"), ANGLES)
                .defaut(ANGLE_PAR_DEFAUT)
                .court(tel("Angle")),
        ],
        note: t(
            "Chaque tuyau commence son déport plus loin que le précédent, de la valeur du décalage.",
            "Each pipe starts its offset farther along than the previous one, by the stagger value.",
        ),
        calcul: deports_paralleles,
    },
    Calculateur {
        id: "dilatation",
        metier: Metier::Plomberie,
        nom: t("Dilatation d'un tuyau", "Pipe expansion"),
        resume: t(
            "L'allongement d'un tuyau qui se réchauffe.",
            "How much a pipe grows as it heats up.",
        ),
        champs: &[
            Champ::longueur(t("Longueur du tuyau", "Pipe length"), Format::PiPo)
                .court(t("Longueur", "Length")),
            Champ::nombre(t("Écart de température", "Temperature change"), tel("°C"))
                .court(t("Écart\nde temp.", "Temp.\nchange")),
            Champ::nombre(
                t("Coefficient de dilatation", "Expansion coefficient"),
                tel("mm/m/°C"),
            )
            .defaut("0,017")
            .court(tel("Coefficient")),
        ],
        note: t(
            "Coefficients courants, en millimètres par mètre et par degré Celsius : cuivre, 0,017; acier, 0,012; PVC, 0,054; CPVC, 0,065; ABS, 0,10; PEX, de 0,15 à 0,20. Celui du fabricant fait foi.",
            "Common coefficients, in millimetres per metre per degree Celsius: copper, 0.017; steel, 0.012; PVC, 0.054; CPVC, 0.065; ABS, 0.10; PEX, 0.15 to 0.20. The manufacturer's value governs.",
        ),
        calcul: dilatation,
    },
    Calculateur {
        id: "chauffe-eau",
        metier: Metier::Plomberie,
        nom: t("Chauffage de l'eau", "Water heating"),
        resume: t(
            "L'énergie et le temps qu'il faut pour réchauffer un volume d'eau.",
            "The energy and time needed to heat a volume of water.",
        ),
        champs: &[
            Champ::nombre(t("Volume d'eau", "Water volume"), tel("L")).court(tel("Volume")),
            Champ::nombre(t("Élévation de température", "Temperature rise"), tel("°C"))
                .court(t("Élévation", "Rise")),
            Champ::nombre(t("Puissance de chauffe", "Heating power"), tel("kW"))
                .court(t("Puissance", "Power")),
            Champ::nombre(t("Rendement", "Efficiency"), tel("%")).defaut("100"),
        ],
        note: t(
            "Un litre d'eau demande 4,186 kJ pour gagner 1 °C. La contenance en litres est inscrite sur la plaque du chauffe-eau.",
            "One litre of water takes 4.186 kJ to gain 1 °C. The capacity in litres is on the water heater's nameplate.",
        ),
        calcul: chauffe_eau,
    },
];

fn angle(e: &Lues, champ: usize) -> f64 {
    DEGRES
        .get(e.choix(champ))
        .copied()
        .unwrap_or(45.0)
        .to_radians()
}

fn pente(e: &Lues) -> Calcul {
    let pente_par_pied = e.longueur(1);
    Ok(vec![
        Resultat::nouveau(
            t("Dénivelé", "Drop"),
            Quantite::Longueur(e.longueur(0) / PO_PAR_PI * pente_par_pied),
        ),
        Resultat::nouveau(
            t("Pente", "Slope"),
            Quantite::Fixe(pente_par_pied / PO_PAR_PI * 100.0, 2, tel("%")),
        ),
    ])
}

/// Les deux lignes d'un déport : le tronçon incliné et l'avance.
fn troncon(deport: f64, angle: f64) -> [Resultat; 2] {
    [
        Resultat::nouveau(
            t("Longueur de parcours", "Travel"),
            Quantite::Longueur(deport / angle.sin()),
        ),
        Resultat::nouveau(t("Avance", "Run"), Quantite::Longueur(deport / angle.tan())),
    ]
}

fn deport(e: &Lues) -> Calcul {
    Ok(troncon(e.longueur(0), angle(e, 1)).to_vec())
}

fn deport_roule(e: &Lues) -> Calcul {
    let reel = e.longueur(0).hypot(e.longueur(1));
    let mut resultats = vec![Resultat::nouveau(
        t("Déport réel", "True offset"),
        Quantite::Longueur(reel),
    )];
    resultats.extend(troncon(reel, angle(e, 2)));
    Ok(resultats)
}

fn cylindre(diametre: f64, longueur: f64) -> f64 {
    PI / 4.0 * diametre * diametre * longueur
}

fn contenance_de_tuyau(e: &Lues) -> Calcul {
    let volume = cylindre(e.longueur(0), e.longueur(1));
    // Un litre d'eau pèse un kilogramme.
    let masse_kg = volume / PO3_PAR_L;
    Ok(vec![
        Resultat::nouveau(t("Contenance", "Capacity"), Quantite::Liquide(volume)),
        Resultat::nouveau(
            t("Masse de l'eau", "Weight of water"),
            Quantite::Masse(masse_kg / KG_PAR_LB),
        ),
    ])
}

fn pression(e: &Lues) -> Calcul {
    let kpa = e.longueur(0) / PO_PAR_M * KPA_PAR_METRE_D_EAU;
    Ok(vec![Resultat::nouveau(
        t("Pression", "Pressure"),
        Quantite::Double(kpa, tel("kPa"), kpa / KPA_PAR_PSI, t("lb/po²", "psi")),
    )])
}

fn reservoir(e: &Lues) -> Calcul {
    Ok(vec![Resultat::nouveau(
        t("Contenance", "Capacity"),
        Quantite::Liquide(cylindre(e.longueur(0), e.longueur(1))),
    )])
}

fn debit_et_vitesse(e: &Lues) -> Calcul {
    // Le débit, en pouces cubes par seconde.
    let debit = match e.choix(2) {
        1 => e.nombre(1) * PO3_PAR_L / 60.0,
        2 => e.nombre(1) * PO3_PAR_L,
        _ => e.nombre(1) * PO3_PAR_GAL_US / 60.0,
    };
    let diametre = e.longueur(0);
    let vitesse = debit / (PI / 4.0 * diametre * diametre);
    Ok(vec![
        Resultat::nouveau(
            t("Vitesse de l'eau", "Water velocity"),
            Quantite::Double(
                vitesse / PO_PAR_PI,
                t("pi/s", "ft/s"),
                vitesse / PO_PAR_M,
                tel("m/s"),
            ),
        ),
        Resultat::nouveau(
            t("Débit", "Flow rate"),
            Quantite::Double(
                debit * 60.0 / PO3_PAR_GAL_US,
                t("gal US/min", "US gal/min"),
                debit * 60.0 / PO3_PAR_L,
                tel("L/min"),
            ),
        ),
    ])
}

fn longueur_de_coupe(e: &Lues) -> Calcul {
    let coupe = e.longueur(0) - e.longueur(1) - e.longueur(2) + 2.0 * e.longueur(3);
    if coupe <= 0.0 {
        return Err(t(
            "Les retraits dépassent la mesure de centre à centre.",
            "The take-offs exceed the centre-to-centre measurement.",
        ));
    }
    Ok(vec![Resultat::nouveau(
        t("Longueur de coupe", "Cut length"),
        Quantite::Longueur(coupe),
    )])
}

fn deports_paralleles(e: &Lues) -> Calcul {
    let angle = angle(e, 2);
    let mut resultats = troncon(e.longueur(0), angle).to_vec();
    resultats.push(Resultat::nouveau(
        t(
            "Décalage d'un tuyau au suivant",
            "Stagger from one pipe to the next",
        ),
        Quantite::Longueur(e.longueur(1) * (angle / 2.0).tan()),
    ));
    Ok(resultats)
}

fn dilatation(e: &Lues) -> Calcul {
    // Coefficient en millimètres par mètre et par degré.
    let millimetres = e.nombre(2) * e.longueur(0) / PO_PAR_M * e.nombre(1);
    Ok(vec![
        Resultat::nouveau(
            t("Allongement", "Expansion"),
            Quantite::Longueur(millimetres / 1000.0 * PO_PAR_M),
        ),
        Resultat::nouveau(
            t("Allongement, en millimètres", "Expansion, in millimetres"),
            Quantite::Fixe(millimetres, 1, tel("mm")),
        ),
    ])
}

fn chauffe_eau(e: &Lues) -> Calcul {
    let rendement = e.nombre(3);
    if rendement > 100.0 {
        return Err(t(
            "Le rendement ne dépasse pas 100 %.",
            "Efficiency cannot exceed 100%.",
        ));
    }
    // Un litre d'eau pèse un kilogramme; 3 600 kJ font un kilowattheure.
    let energie = e.nombre(0) * KJ_PAR_KG_ET_DEGRE * e.nombre(1) / 3600.0;
    let heures = energie / (e.nombre(2) * rendement / 100.0);
    Ok(vec![
        Resultat::nouveau(
            t("Énergie nécessaire", "Energy required"),
            Quantite::Mesure(energie, tel("kWh")),
        ),
        Resultat::nouveau(
            t("Temps de chauffe", "Heating time"),
            Quantite::Fixe(heures * 60.0, 0, tel("min")),
        ),
        Resultat::nouveau(
            t("Temps de chauffe, en heures", "Heating time, in hours"),
            Quantite::Fixe(heures, 2, tel("h")),
        ),
    ])
}
