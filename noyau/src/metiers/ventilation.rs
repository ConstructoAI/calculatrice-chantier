//! Ventilation et chauffage (section 22.8).

use std::f64::consts::PI;

use super::{Calcul, Calculateur, Champ, Lues, Metier, PO3_PAR_L, Quantite, Resultat};
use crate::unites::{Format, PO2_PAR_PI2, PO3_PAR_PI3};

/// Litres par seconde dans un pied cube par minute.
const LS_PAR_PCM: f64 = PO3_PAR_PI3 / PO3_PAR_L / 60.0;
/// BTU par heure dans un kilowatt (BTU de la table internationale).
const BTU_H_PAR_KW: f64 = 3_412.141_633;
/// BTU par heure dans une tonne de réfrigération.
const BTU_H_PAR_TONNE: f64 = 12_000.0;
const UNITES_THERMIQUES: &[&str] = &["BTU/h", "kW", "tonnes de réfrigération"];

pub const CALCULATEURS: &[Calculateur] = &[
    Calculateur {
        id: "conduit-rond",
        metier: Metier::Ventilation,
        nom: "Débit d'un conduit rond",
        resume: "Le débit d'air d'un conduit rond, d'après son diamètre et la vitesse de l'air.",
        champs: &[
            Champ::longueur("Diamètre", Format::PoFrac),
            Champ::nombre("Vitesse de l'air", "pi/min"),
        ],
        note: "",
        calcul: conduit_rond,
    },
    Calculateur {
        id: "conduit-rectangulaire",
        metier: Metier::Ventilation,
        nom: "Débit d'un conduit rectangulaire",
        resume: "Le débit d'air d'un conduit rectangulaire, et le conduit rond qui le remplace.",
        champs: &[
            Champ::longueur("Largeur", Format::PoFrac),
            Champ::longueur("Hauteur", Format::PoFrac),
            Champ::nombre("Vitesse de l'air", "pi/min"),
        ],
        note: "Le conduit rond équivalent offre la même perte de charge pour le même débit (formule de Huebscher).",
        calcul: conduit_rectangulaire,
    },
    Calculateur {
        id: "puissance-thermique",
        metier: Metier::Ventilation,
        nom: "Puissance de chauffage et de climatisation",
        resume: "La même puissance en BTU/h, en kilowatts et en tonnes de réfrigération.",
        champs: &[
            Champ::nombre("Puissance", ""),
            Champ::choix("Unité", UNITES_THERMIQUES),
        ],
        note: "",
        calcul: puissance_thermique,
    },
];

/// Les deux lignes communes : la section, puis le débit.
fn debit(section: f64, vitesse: f64) -> [Resultat; 2] {
    let pcm = section / PO2_PAR_PI2 * vitesse;
    [
        Resultat::nouveau("Section", Quantite::Aire(section)),
        Resultat::nouveau(
            "Débit",
            Quantite::Double(pcm, "pi³/min", pcm * LS_PAR_PCM, "L/s"),
        ),
    ]
}

fn conduit_rond(e: &Lues) -> Calcul {
    let diametre = e.longueur(0);
    Ok(debit(PI / 4.0 * diametre * diametre, e.nombre(1)).to_vec())
}

fn conduit_rectangulaire(e: &Lues) -> Calcul {
    let (largeur, hauteur) = (e.longueur(0), e.longueur(1));
    let mut resultats = debit(largeur * hauteur, e.nombre(2)).to_vec();
    let equivalent = 1.30 * (largeur * hauteur).powf(0.625) / (largeur + hauteur).powf(0.25);
    resultats.push(Resultat::nouveau(
        "Conduit rond équivalent",
        Quantite::Longueur(equivalent),
    ));
    Ok(resultats)
}

fn puissance_thermique(e: &Lues) -> Calcul {
    let btu_h = match e.choix(1) {
        1 => e.nombre(0) * BTU_H_PAR_KW,
        2 => e.nombre(0) * BTU_H_PAR_TONNE,
        _ => e.nombre(0),
    };
    Ok(vec![
        Resultat::nouveau("En BTU par heure", Quantite::Mesure(btu_h, "BTU/h")),
        Resultat::nouveau("En kilowatts", Quantite::Mesure(btu_h / BTU_H_PAR_KW, "kW")),
        Resultat::nouveau(
            "En tonnes de réfrigération",
            Quantite::Mesure(btu_h / BTU_H_PAR_TONNE, "tonnes"),
        ),
    ])
}
