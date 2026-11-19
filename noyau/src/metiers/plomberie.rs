//! Plomberie (section 22.5).

use std::f64::consts::PI;

use super::{
    Calcul, Calculateur, Champ, Lues, Metier, PO3_PAR_GAL_US, PO3_PAR_L, Quantite, Resultat,
};
use crate::unites::{Format, KG_PAR_LB, PO_PAR_M, PO_PAR_PI};

/// Angles des raccords courants, et leur valeur en degrés.
const ANGLES: &[&str] = &["11,25°", "22,5°", "30°", "45°", "60°"];
const DEGRES: [f64; 5] = [11.25, 22.5, 30.0, 45.0, 60.0];
/// Rang de 45° dans la liste : l'angle proposé au départ.
const ANGLE_PAR_DEFAUT: &str = "3";

/// Pression d'un mètre d'eau, en kilopascals (masse volumique de 1 000 kg/m³).
const KPA_PAR_METRE_D_EAU: f64 = 9.806_65;
/// Kilopascals par livre au pouce carré.
const KPA_PAR_PSI: f64 = 6.894_757_293_168_361;
/// Chaleur massique de l'eau, en kilojoules par kilogramme et par degré Celsius.
const KJ_PAR_KG_ET_DEGRE: f64 = 4.186;
const UNITES_DE_DEBIT: &[&str] = &["gal US/min", "L/min", "L/s"];

const NOTE_DES_DEPORTS: &str = "Mesures de centre à centre : déduisez l'emboîtement des raccords.";

pub const CALCULATEURS: &[Calculateur] = &[
    Calculateur {
        id: "pente-de-drain",
        metier: Metier::Plomberie,
        nom: "Pente d'un drain",
        resume: "Le dénivelé d'un tuyau posé en pente régulière.",
        champs: &[
            Champ::longueur("Longueur du tuyau", Format::PiPo).court("Longueur"),
            Champ::longueur("Pente, par pied de longueur", Format::PoFrac)
                .court("Pente\npar pied")
                .defaut("1/4"),
        ],
        note: "Le Code fixe la pente minimale selon le diamètre du tuyau.",
        calcul: pente,
    },
    Calculateur {
        id: "deport",
        metier: Metier::Plomberie,
        nom: "Déport de tuyauterie",
        resume: "Le tronçon incliné qui relie deux tuyaux parallèles.",
        champs: &[
            Champ::longueur("Déport", Format::PoFrac),
            Champ::choix("Angle des raccords", ANGLES)
                .court("Angle")
                .defaut(ANGLE_PAR_DEFAUT),
        ],
        note: NOTE_DES_DEPORTS,
        calcul: deport,
    },
    Calculateur {
        id: "deport-roule",
        metier: Metier::Plomberie,
        nom: "Déport roulé",
        resume: "Un déport à la fois horizontal et vertical.",
        champs: &[
            Champ::longueur("Déport horizontal", Format::PoFrac).court("Déport\nhorizontal"),
            Champ::longueur("Déport vertical", Format::PoFrac).court("Déport\nvertical"),
            Champ::choix("Angle des raccords", ANGLES)
                .court("Angle")
                .defaut(ANGLE_PAR_DEFAUT),
        ],
        note: NOTE_DES_DEPORTS,
        calcul: deport_roule,
    },
    Calculateur {
        id: "contenance-de-tuyau",
        metier: Metier::Plomberie,
        nom: "Contenance d'un tuyau",
        resume: "Le volume d'eau que contient un tuyau plein, et sa masse.",
        champs: &[
            Champ::longueur("Diamètre intérieur", Format::PoFrac).court("Diamètre"),
            Champ::longueur("Longueur", Format::PiPo),
        ],
        note: "",
        calcul: contenance_de_tuyau,
    },
    Calculateur {
        id: "pression-d-eau",
        metier: Metier::Plomberie,
        nom: "Pression d'une colonne d'eau",
        resume: "La pression statique au bas d'une hauteur d'eau.",
        champs: &[Champ::longueur("Hauteur d'eau", Format::PiPo).court("Hauteur")],
        note: "Eau froide au repos : la pression baisse dès que l'eau circule.",
        calcul: pression,
    },
    Calculateur {
        id: "reservoir",
        metier: Metier::Plomberie,
        nom: "Réservoir cylindrique",
        resume: "La contenance d'un réservoir rond.",
        champs: &[
            Champ::longueur("Diamètre intérieur", Format::PoFrac).court("Diamètre"),
            Champ::longueur("Hauteur ou longueur", Format::PoFrac).court("Hauteur"),
        ],
        note: "",
        calcul: reservoir,
    },
    Calculateur {
        id: "debit-et-vitesse",
        metier: Metier::Plomberie,
        nom: "Débit et vitesse de l'eau",
        resume: "La vitesse de l'eau dans un tuyau, d'après son diamètre intérieur et le débit.",
        champs: &[
            Champ::longueur("Diamètre intérieur", Format::PoFrac).court("Diamètre"),
            Champ::nombre("Débit", ""),
            Champ::choix("Unité du débit", UNITES_DE_DEBIT).court("Unité"),
        ],
        note: "Une vitesse élevée cause du bruit et de l'usure : comparez-la à la limite recommandée pour le matériau du tuyau.",
        calcul: debit_et_vitesse,
    },
    Calculateur {
        id: "longueur-de-coupe",
        metier: Metier::Plomberie,
        nom: "Longueur de coupe d'un tuyau",
        resume: "La longueur à couper entre deux raccords, d'après la mesure de centre à centre.",
        champs: &[
            Champ::longueur("Mesure de centre à centre", Format::PoFrac).court("Centre\nà centre"),
            Champ::longueur("Retrait du premier raccord", Format::PoFrac)
                .nul_permis()
                .court("Retrait 1"),
            Champ::longueur("Retrait du second raccord", Format::PoFrac)
                .nul_permis()
                .court("Retrait 2"),
            Champ::longueur("Emboîtement à chaque bout", Format::PoFrac)
                .defaut("0")
                .nul_permis()
                .court("Emboîtement"),
        ],
        note: "Le retrait va du centre du raccord à sa face; l'emboîtement est la profondeur où le tuyau entre dans le raccord. Les deux figurent dans les tables du fabricant.",
        calcul: longueur_de_coupe,
    },
    Calculateur {
        id: "deports-paralleles",
        metier: Metier::Plomberie,
        nom: "Déports parallèles",
        resume: "Des tuyaux côte à côte qui dévient ensemble, en gardant le même écart.",
        champs: &[
            Champ::longueur("Déport", Format::PoFrac),
            Champ::longueur("Écart entre les tuyaux, de centre à centre", Format::PoFrac)
                .court("Écart"),
            Champ::choix("Angle des raccords", ANGLES)
                .defaut(ANGLE_PAR_DEFAUT)
                .court("Angle"),
        ],
        note: "Chaque tuyau commence son déport plus loin que le précédent, de la valeur du décalage.",
        calcul: deports_paralleles,
    },
    Calculateur {
        id: "dilatation",
        metier: Metier::Plomberie,
        nom: "Dilatation d'un tuyau",
        resume: "L'allongement d'un tuyau qui se réchauffe.",
        champs: &[
            Champ::longueur("Longueur du tuyau", Format::PiPo).court("Longueur"),
            Champ::nombre("Écart de température", "°C").court("Écart\nde temp."),
            Champ::nombre("Coefficient de dilatation", "mm/m/°C")
                .defaut("0,017")
                .court("Coefficient"),
        ],
        note: "Coefficients courants, en millimètres par mètre et par degré Celsius : cuivre, 0,017; acier, 0,012; PVC, 0,054; CPVC, 0,065; ABS, 0,10; PEX, de 0,15 à 0,20. Celui du fabricant fait foi.",
        calcul: dilatation,
    },
    Calculateur {
        id: "chauffe-eau",
        metier: Metier::Plomberie,
        nom: "Chauffage de l'eau",
        resume: "L'énergie et le temps qu'il faut pour réchauffer un volume d'eau.",
        champs: &[
            Champ::nombre("Volume d'eau", "L").court("Volume"),
            Champ::nombre("Élévation de température", "°C").court("Élévation"),
            Champ::nombre("Puissance de chauffe", "kW").court("Puissance"),
            Champ::nombre("Rendement", "%").defaut("100"),
        ],
        note: "Un litre d'eau demande 4,186 kJ pour gagner 1 °C. La contenance en litres est inscrite sur la plaque du chauffe-eau.",
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
            "Dénivelé",
            Quantite::Longueur(e.longueur(0) / PO_PAR_PI * pente_par_pied),
        ),
        Resultat::nouveau(
            "Pente",
            Quantite::Fixe(pente_par_pied / PO_PAR_PI * 100.0, 2, "%"),
        ),
    ])
}

/// Les deux lignes d'un déport : le tronçon incliné et l'avance.
fn troncon(deport: f64, angle: f64) -> [Resultat; 2] {
    [
        Resultat::nouveau(
            "Longueur de parcours",
            Quantite::Longueur(deport / angle.sin()),
        ),
        Resultat::nouveau("Avance", Quantite::Longueur(deport / angle.tan())),
    ]
}

fn deport(e: &Lues) -> Calcul {
    Ok(troncon(e.longueur(0), angle(e, 1)).to_vec())
}

fn deport_roule(e: &Lues) -> Calcul {
    let reel = e.longueur(0).hypot(e.longueur(1));
    let mut resultats = vec![Resultat::nouveau("Déport réel", Quantite::Longueur(reel))];
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
        Resultat::nouveau("Contenance", Quantite::Liquide(volume)),
        Resultat::nouveau("Masse de l'eau", Quantite::Masse(masse_kg / KG_PAR_LB)),
    ])
}

fn pression(e: &Lues) -> Calcul {
    let kpa = e.longueur(0) / PO_PAR_M * KPA_PAR_METRE_D_EAU;
    Ok(vec![Resultat::nouveau(
        "Pression",
        Quantite::Double(kpa, "kPa", kpa / KPA_PAR_PSI, "lb/po²"),
    )])
}

fn reservoir(e: &Lues) -> Calcul {
    Ok(vec![Resultat::nouveau(
        "Contenance",
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
            "Vitesse de l'eau",
            Quantite::Double(vitesse / PO_PAR_PI, "pi/s", vitesse / PO_PAR_M, "m/s"),
        ),
        Resultat::nouveau(
            "Débit",
            Quantite::Double(
                debit * 60.0 / PO3_PAR_GAL_US,
                "gal US/min",
                debit * 60.0 / PO3_PAR_L,
                "L/min",
            ),
        ),
    ])
}

fn longueur_de_coupe(e: &Lues) -> Calcul {
    let coupe = e.longueur(0) - e.longueur(1) - e.longueur(2) + 2.0 * e.longueur(3);
    if coupe <= 0.0 {
        return Err("Les retraits dépassent la mesure de centre à centre.");
    }
    Ok(vec![Resultat::nouveau(
        "Longueur de coupe",
        Quantite::Longueur(coupe),
    )])
}

fn deports_paralleles(e: &Lues) -> Calcul {
    let angle = angle(e, 2);
    let mut resultats = troncon(e.longueur(0), angle).to_vec();
    resultats.push(Resultat::nouveau(
        "Décalage d'un tuyau au suivant",
        Quantite::Longueur(e.longueur(1) * (angle / 2.0).tan()),
    ));
    Ok(resultats)
}

fn dilatation(e: &Lues) -> Calcul {
    // Coefficient en millimètres par mètre et par degré.
    let millimetres = e.nombre(2) * e.longueur(0) / PO_PAR_M * e.nombre(1);
    Ok(vec![
        Resultat::nouveau(
            "Allongement",
            Quantite::Longueur(millimetres / 1000.0 * PO_PAR_M),
        ),
        Resultat::nouveau(
            "Allongement, en millimètres",
            Quantite::Fixe(millimetres, 1, "mm"),
        ),
    ])
}

fn chauffe_eau(e: &Lues) -> Calcul {
    let rendement = e.nombre(3);
    if rendement > 100.0 {
        return Err("Le rendement ne dépasse pas 100 %.");
    }
    // Un litre d'eau pèse un kilogramme; 3 600 kJ font un kilowattheure.
    let energie = e.nombre(0) * KJ_PAR_KG_ET_DEGRE * e.nombre(1) / 3600.0;
    let heures = energie / (e.nombre(2) * rendement / 100.0);
    Ok(vec![
        Resultat::nouveau("Énergie nécessaire", Quantite::Mesure(energie, "kWh")),
        Resultat::nouveau("Temps de chauffe", Quantite::Fixe(heures * 60.0, 0, "min")),
        Resultat::nouveau(
            "Temps de chauffe, en heures",
            Quantite::Fixe(heures, 2, "h"),
        ),
    ])
}
