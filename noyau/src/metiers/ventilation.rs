//! Ventilation et chauffage (section 22.8).

use std::f64::consts::PI;

use super::{
    Calcul, Calculateur, Champ, Lues, Metier, PO3_PAR_L, PO3_PAR_M3, Quantite, Resultat,
    arrondi_au_dessus,
};
use crate::unites::{Format, PO2_PAR_PI2, PO3_PAR_PI3};

/// Litres par seconde dans un pied cube par minute.
const LS_PAR_PCM: f64 = PO3_PAR_PI3 / PO3_PAR_L / 60.0;
/// BTU par heure dans un kilowatt (BTU de la table internationale).
const BTU_H_PAR_KW: f64 = 3_412.141_633;
/// BTU par heure dans une tonne de réfrigération.
const BTU_H_PAR_TONNE: f64 = 12_000.0;
const UNITES_THERMIQUES: &[&str] = &["BTU/h", "kW", "tonnes de réfrigération"];
/// BTU par heure que transporte un pied cube d'air à la minute, par degré
/// Fahrenheit d'écart : air normal, 0,075 lb/pi³ × 0,24 BTU/lb·°F × 60 min.
const BTU_H_PAR_PCM_ET_DEGRE: f64 = 1.08;
const ECARTS: &[&str] = &["°C", "°F"];

pub const CALCULATEURS: &[Calculateur] = &[
    Calculateur {
        id: "conduit-rond",
        metier: Metier::Ventilation,
        nom: "Débit d'un conduit rond",
        resume: "Le débit d'air d'un conduit rond, d'après son diamètre et la vitesse de l'air.",
        champs: &[
            Champ::longueur("Diamètre", Format::PoFrac),
            Champ::nombre("Vitesse de l'air", "pi/min").court("Vitesse"),
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
            Champ::nombre("Vitesse de l'air", "pi/min").court("Vitesse"),
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
    Calculateur {
        id: "conduit-pour-un-debit",
        metier: Metier::Ventilation,
        nom: "Conduit rond pour un débit",
        resume: "Le diamètre du conduit rond qui transporte un débit d'air à la vitesse visée.",
        champs: &[
            Champ::nombre("Débit d'air", "pi³/min").court("Débit"),
            Champ::nombre("Vitesse de l'air", "pi/min").court("Vitesse"),
        ],
        note: "",
        calcul: conduit_pour_un_debit,
    },
    Calculateur {
        id: "changements-d-air",
        metier: Metier::Ventilation,
        nom: "Changements d'air à l'heure",
        resume: "Le débit qu'il faut pour renouveler l'air d'une pièce un certain nombre de fois à l'heure.",
        champs: &[
            Champ::longueur("Longueur de la pièce", Format::PiPo).court("Longueur"),
            Champ::longueur("Largeur de la pièce", Format::PiPo).court("Largeur"),
            Champ::longueur("Hauteur du plafond", Format::PiPo)
                .defaut("8")
                .court("Hauteur"),
            Champ::nombre("Changements d'air à l'heure", "").court("Changements"),
        ],
        note: "",
        calcul: changements_d_air,
    },
    Calculateur {
        id: "chaleur-de-l-air",
        metier: Metier::Ventilation,
        nom: "Chaleur transportée par l'air",
        resume: "La puissance de chauffage ou de refroidissement d'un débit d'air, d'après l'écart de température.",
        champs: &[
            Champ::nombre("Débit d'air", "pi³/min").court("Débit"),
            Champ::nombre("Écart de température", "").court("Écart\nde temp."),
            Champ::choix("Unité de l'écart", ECARTS).court("Unité"),
        ],
        note: "Chaleur sensible seulement, pour de l'air aux conditions normales : 1,08 BTU/h par pi³/min et par degré Fahrenheit.",
        calcul: chaleur_de_l_air,
    },
    Calculateur {
        id: "plancher-chauffant",
        metier: Metier::Ventilation,
        nom: "Tuyau de plancher chauffant",
        resume: "La longueur de tuyau d'un plancher chauffant et le nombre de boucles.",
        champs: &[
            Champ::longueur("Longueur de la surface", Format::PiPo).court("Longueur"),
            Champ::longueur("Largeur de la surface", Format::PiPo).court("Largeur"),
            Champ::longueur("Espacement des tuyaux", Format::PoFrac)
                .defaut("9")
                .court("Espacement"),
            Champ::longueur("Longueur maximale d'une boucle", Format::PiPo)
                .defaut("300")
                .court("Boucle\nmaximale"),
        ],
        note: "Sans les longueurs d'amenée jusqu'au collecteur. L'espacement et la longueur maximale d'une boucle viennent du concepteur ou du fabricant.",
        calcul: plancher_chauffant,
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

fn conduit_pour_un_debit(e: &Lues) -> Calcul {
    let section = e.nombre(0) / e.nombre(1) * PO2_PAR_PI2;
    Ok(vec![
        Resultat::nouveau("Section nécessaire", Quantite::Aire(section)),
        Resultat::nouveau(
            "Diamètre du conduit rond",
            Quantite::Longueur((4.0 * section / PI).sqrt()),
        ),
    ])
}

fn changements_d_air(e: &Lues) -> Calcul {
    let volume = e.longueur(0) * e.longueur(1) * e.longueur(2);
    let pcm = volume / PO3_PAR_PI3 * e.nombre(3) / 60.0;
    Ok(vec![
        Resultat::nouveau(
            "Volume de la pièce",
            Quantite::Double(volume / PO3_PAR_PI3, "pi³", volume / PO3_PAR_M3, "m³"),
        ),
        Resultat::nouveau(
            "Débit",
            Quantite::Double(pcm, "pi³/min", pcm * LS_PAR_PCM, "L/s"),
        ),
    ])
}

fn chaleur_de_l_air(e: &Lues) -> Calcul {
    // Un écart de 1 °C vaut un écart de 1,8 °F.
    let ecart = if e.choix(2) == 1 {
        e.nombre(1)
    } else {
        e.nombre(1) * 1.8
    };
    let btu_h = BTU_H_PAR_PCM_ET_DEGRE * e.nombre(0) * ecart;
    Ok(vec![Resultat::nouveau(
        "Puissance",
        Quantite::Double(btu_h, "BTU/h", btu_h / BTU_H_PAR_KW, "kW"),
    )])
}

fn plancher_chauffant(e: &Lues) -> Calcul {
    let surface = e.longueur(0) * e.longueur(1);
    let tuyau = surface / e.longueur(2);
    let boucles = arrondi_au_dessus(tuyau / e.longueur(3)).max(1.0);
    Ok(vec![
        Resultat::nouveau("Surface", Quantite::Aire(surface)),
        Resultat::nouveau("Longueur de tuyau", Quantite::Longueur(tuyau)),
        Resultat::nouveau("Boucles", Quantite::Compte(boucles, "boucles")),
        Resultat::nouveau("Longueur par boucle", Quantite::Longueur(tuyau / boucles)),
    ])
}
