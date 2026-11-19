//! Plomberie (section 22.4).

use std::f64::consts::PI;

use super::{Calcul, Calculateur, Champ, Lues, Metier, PO3_PAR_L, Quantite, Resultat};
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

const NOTE_DES_DEPORTS: &str = "Mesures de centre à centre : déduisez l'emboîtement des raccords.";

pub const CALCULATEURS: &[Calculateur] = &[
    Calculateur {
        id: "pente-de-drain",
        metier: Metier::Plomberie,
        nom: "Pente d'un drain",
        resume: "Le dénivelé d'un tuyau posé en pente régulière.",
        champs: &[
            Champ::longueur("Longueur du tuyau", Format::PiPo),
            Champ::longueur("Pente, par pied de longueur", Format::PoFrac).defaut("1/4"),
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
            Champ::choix("Angle des raccords", ANGLES).defaut(ANGLE_PAR_DEFAUT),
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
            Champ::longueur("Déport horizontal", Format::PoFrac),
            Champ::longueur("Déport vertical", Format::PoFrac),
            Champ::choix("Angle des raccords", ANGLES).defaut(ANGLE_PAR_DEFAUT),
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
            Champ::longueur("Diamètre intérieur", Format::PoFrac),
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
        champs: &[Champ::longueur("Hauteur d'eau", Format::PiPo)],
        note: "Eau froide au repos : la pression baisse dès que l'eau circule.",
        calcul: pression,
    },
    Calculateur {
        id: "reservoir",
        metier: Metier::Plomberie,
        nom: "Réservoir cylindrique",
        resume: "La contenance d'un réservoir rond.",
        champs: &[
            Champ::longueur("Diamètre intérieur", Format::PoFrac),
            Champ::longueur("Hauteur ou longueur", Format::PoFrac),
        ],
        note: "",
        calcul: reservoir,
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
