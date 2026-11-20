//! Ventilation et chauffage (section 22.8).

use std::f64::consts::PI;

use super::{
    Calcul, Calculateur, Champ, Lues, Metier, PO3_PAR_L, PO3_PAR_M3, Quantite, Resultat,
    arrondi_au_dessus,
};
use crate::langue::{Texte, t, tel};
use crate::unites::{Format, PO2_PAR_PI2, PO3_PAR_PI3};

/// Litres par seconde dans un pied cube par minute.
const LS_PAR_PCM: f64 = PO3_PAR_PI3 / PO3_PAR_L / 60.0;
/// BTU par heure dans un kilowatt (BTU de la table internationale).
const BTU_H_PAR_KW: f64 = 3_412.141_633;
/// BTU par heure dans une tonne de réfrigération.
const BTU_H_PAR_TONNE: f64 = 12_000.0;
const UNITES_THERMIQUES: &[Texte] = &[
    tel("BTU/h"),
    tel("kW"),
    t("tonnes de réfrigération", "tons of refrigeration"),
];
/// BTU par heure que transporte un pied cube d'air à la minute, par degré
/// Fahrenheit d'écart : air normal, 0,075 lb/pi³ × 0,24 BTU/lb·°F × 60 min.
const BTU_H_PAR_PCM_ET_DEGRE: f64 = 1.08;
const ECARTS: &[Texte] = &[tel("°C"), tel("°F")];

pub const CALCULATEURS: &[Calculateur] = &[
    Calculateur {
        id: "conduit-rond",
        metier: Metier::Ventilation,
        nom: t("Débit d'un conduit rond", "Round duct airflow"),
        resume: t(
            "Le débit d'air d'un conduit rond, d'après son diamètre et la vitesse de l'air.",
            "The airflow of a round duct, from its diameter and the air velocity.",
        ),
        champs: &[
            Champ::longueur(t("Diamètre", "Diameter"), Format::PoFrac),
            Champ::nombre(t("Vitesse de l'air", "Air velocity"), t("pi/min", "ft/min"))
                .court(t("Vitesse", "Velocity")),
        ],
        note: Texte::VIDE,
        calcul: conduit_rond,
    },
    Calculateur {
        id: "conduit-rectangulaire",
        metier: Metier::Ventilation,
        nom: t(
            "Débit d'un conduit rectangulaire",
            "Rectangular duct airflow",
        ),
        resume: t(
            "Le débit d'air d'un conduit rectangulaire, et le conduit rond qui le remplace.",
            "The airflow of a rectangular duct, and the round duct that replaces it.",
        ),
        champs: &[
            Champ::longueur(t("Largeur", "Width"), Format::PoFrac),
            Champ::longueur(t("Hauteur", "Height"), Format::PoFrac),
            Champ::nombre(t("Vitesse de l'air", "Air velocity"), t("pi/min", "ft/min"))
                .court(t("Vitesse", "Velocity")),
        ],
        note: t(
            "Le conduit rond équivalent offre la même perte de charge pour le même débit (formule de Huebscher).",
            "The equivalent round duct has the same friction loss at the same airflow (Huebscher's formula).",
        ),
        calcul: conduit_rectangulaire,
    },
    Calculateur {
        id: "puissance-thermique",
        metier: Metier::Ventilation,
        nom: t(
            "Puissance de chauffage et de climatisation",
            "Heating and cooling capacity",
        ),
        resume: t(
            "La même puissance en BTU/h, en kilowatts et en tonnes de réfrigération.",
            "The same capacity in BTU/h, kilowatts and tons of refrigeration.",
        ),
        champs: &[
            Champ::nombre(t("Puissance", "Capacity"), Texte::VIDE),
            Champ::choix(t("Unité", "Unit"), UNITES_THERMIQUES),
        ],
        note: Texte::VIDE,
        calcul: puissance_thermique,
    },
    Calculateur {
        id: "conduit-pour-un-debit",
        metier: Metier::Ventilation,
        nom: t("Conduit rond pour un débit", "Round duct for an airflow"),
        resume: t(
            "Le diamètre du conduit rond qui transporte un débit d'air à la vitesse visée.",
            "The diameter of the round duct that carries an airflow at the target velocity.",
        ),
        champs: &[
            Champ::nombre(t("Débit d'air", "Airflow"), t("pi³/min", "CFM"))
                .court(t("Débit", "Airflow")),
            Champ::nombre(t("Vitesse de l'air", "Air velocity"), t("pi/min", "ft/min"))
                .court(t("Vitesse", "Velocity")),
        ],
        note: Texte::VIDE,
        calcul: conduit_pour_un_debit,
    },
    Calculateur {
        id: "changements-d-air",
        metier: Metier::Ventilation,
        nom: t("Changements d'air à l'heure", "Air changes per hour"),
        resume: t(
            "Le débit qu'il faut pour renouveler l'air d'une pièce un certain nombre de fois à l'heure.",
            "The airflow needed to change the air in a room a given number of times per hour.",
        ),
        champs: &[
            Champ::longueur(t("Longueur de la pièce", "Room length"), Format::PiPo)
                .court(t("Longueur", "Length")),
            Champ::longueur(t("Largeur de la pièce", "Room width"), Format::PiPo)
                .court(t("Largeur", "Width")),
            Champ::longueur(t("Hauteur du plafond", "Ceiling height"), Format::PiPo)
                .defaut("8")
                .court(t("Hauteur", "Height")),
            Champ::nombre(
                t("Changements d'air à l'heure", "Air changes per hour"),
                Texte::VIDE,
            )
            .court(t("Changements", "Air changes")),
        ],
        note: Texte::VIDE,
        calcul: changements_d_air,
    },
    Calculateur {
        id: "chaleur-de-l-air",
        metier: Metier::Ventilation,
        nom: t("Chaleur transportée par l'air", "Heat carried by air"),
        resume: t(
            "La puissance de chauffage ou de refroidissement d'un débit d'air, d'après l'écart de température.",
            "The heating or cooling capacity of an airflow, from the temperature difference.",
        ),
        champs: &[
            Champ::nombre(t("Débit d'air", "Airflow"), t("pi³/min", "CFM"))
                .court(t("Débit", "Airflow")),
            Champ::nombre(
                t("Écart de température", "Temperature difference"),
                Texte::VIDE,
            )
            .court(t("Écart\nde temp.", "Temp.\ndifference")),
            Champ::choix(t("Unité de l'écart", "Unit of the difference"), ECARTS)
                .court(t("Unité", "Unit")),
        ],
        note: t(
            "Chaleur sensible seulement, pour de l'air aux conditions normales : 1,08 BTU/h par pi³/min et par degré Fahrenheit.",
            "Sensible heat only, for air at standard conditions: 1.08 BTU/h per CFM per degree Fahrenheit.",
        ),
        calcul: chaleur_de_l_air,
    },
    Calculateur {
        id: "plancher-chauffant",
        metier: Metier::Ventilation,
        nom: t("Tuyau de plancher chauffant", "Radiant floor tubing"),
        resume: t(
            "La longueur de tuyau d'un plancher chauffant et le nombre de boucles.",
            "The length of tubing for a radiant floor and the number of loops.",
        ),
        champs: &[
            Champ::longueur(
                t("Longueur de la surface", "Length of the area"),
                Format::PiPo,
            )
            .court(t("Longueur", "Length")),
            Champ::longueur(
                t("Largeur de la surface", "Width of the area"),
                Format::PiPo,
            )
            .court(t("Largeur", "Width")),
            Champ::longueur(t("Espacement des tuyaux", "Tube spacing"), Format::PoFrac)
                .defaut("9")
                .court(t("Espacement", "Spacing")),
            Champ::longueur(
                t("Longueur maximale d'une boucle", "Maximum loop length"),
                Format::PiPo,
            )
            .defaut("300")
            .court(t("Boucle\nmaximale", "Maximum\nloop")),
        ],
        note: t(
            "Sans les longueurs d'amenée jusqu'au collecteur. L'espacement et la longueur maximale d'une boucle viennent du concepteur ou du fabricant.",
            "Leader lengths to the manifold are not included. The spacing and the maximum loop length come from the designer or the manufacturer.",
        ),
        calcul: plancher_chauffant,
    },
];

/// Les deux lignes communes : la section, puis le débit.
fn debit(section: f64, vitesse: f64) -> [Resultat; 2] {
    let pcm = section / PO2_PAR_PI2 * vitesse;
    [
        Resultat::nouveau(t("Section", "Cross-section"), Quantite::Aire(section)),
        Resultat::nouveau(
            t("Débit", "Airflow"),
            Quantite::Double(pcm, t("pi³/min", "CFM"), pcm * LS_PAR_PCM, tel("L/s")),
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
        t("Conduit rond équivalent", "Equivalent round duct"),
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
        Resultat::nouveau(
            t("En BTU par heure", "In BTU per hour"),
            Quantite::Mesure(btu_h, tel("BTU/h")),
        ),
        Resultat::nouveau(
            t("En kilowatts", "In kilowatts"),
            Quantite::Mesure(btu_h / BTU_H_PAR_KW, tel("kW")),
        ),
        Resultat::nouveau(
            t("En tonnes de réfrigération", "In tons of refrigeration"),
            Quantite::Mesure(btu_h / BTU_H_PAR_TONNE, t("tonnes", "tons")),
        ),
    ])
}

fn conduit_pour_un_debit(e: &Lues) -> Calcul {
    let section = e.nombre(0) / e.nombre(1) * PO2_PAR_PI2;
    Ok(vec![
        Resultat::nouveau(
            t("Section nécessaire", "Cross-section required"),
            Quantite::Aire(section),
        ),
        Resultat::nouveau(
            t("Diamètre du conduit rond", "Round duct diameter"),
            Quantite::Longueur((4.0 * section / PI).sqrt()),
        ),
    ])
}

fn changements_d_air(e: &Lues) -> Calcul {
    let volume = e.longueur(0) * e.longueur(1) * e.longueur(2);
    let pcm = volume / PO3_PAR_PI3 * e.nombre(3) / 60.0;
    Ok(vec![
        Resultat::nouveau(
            t("Volume de la pièce", "Room volume"),
            Quantite::Double(
                volume / PO3_PAR_PI3,
                t("pi³", "ft³"),
                volume / PO3_PAR_M3,
                tel("m³"),
            ),
        ),
        Resultat::nouveau(
            t("Débit", "Airflow"),
            Quantite::Double(pcm, t("pi³/min", "CFM"), pcm * LS_PAR_PCM, tel("L/s")),
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
        t("Puissance", "Capacity"),
        Quantite::Double(btu_h, tel("BTU/h"), btu_h / BTU_H_PAR_KW, tel("kW")),
    )])
}

fn plancher_chauffant(e: &Lues) -> Calcul {
    let surface = e.longueur(0) * e.longueur(1);
    let tuyau = surface / e.longueur(2);
    let boucles = arrondi_au_dessus(tuyau / e.longueur(3)).max(1.0);
    Ok(vec![
        Resultat::nouveau(t("Surface", "Area"), Quantite::Aire(surface)),
        Resultat::nouveau(
            t("Longueur de tuyau", "Tubing length"),
            Quantite::Longueur(tuyau),
        ),
        Resultat::nouveau(
            t("Boucles", "Loops"),
            Quantite::Compte(boucles, t("boucles", "loops")),
        ),
        Resultat::nouveau(
            t("Longueur par boucle", "Length per loop"),
            Quantite::Longueur(tuyau / boucles),
        ),
    ])
}
