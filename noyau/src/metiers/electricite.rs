//! Électricité (section 22.6).

use std::f64::consts::PI;

use super::{
    Calcul, Calculateur, Champ, Lues, Metier, PO2_PAR_M2, Quantite, Resultat, arrondi_au_dessus,
};
use crate::langue::{Texte, t, tel};
use crate::unites::{Format, PO_PAR_PI};

/// Calibres des conducteurs et leur section en mils circulaires.
const CALIBRES: &[Texte] = &[
    tel("14 AWG"),
    tel("12 AWG"),
    tel("10 AWG"),
    tel("8 AWG"),
    tel("6 AWG"),
    tel("4 AWG"),
    tel("3 AWG"),
    tel("2 AWG"),
    tel("1 AWG"),
    tel("1/0 AWG"),
    tel("2/0 AWG"),
    tel("3/0 AWG"),
    tel("4/0 AWG"),
    tel("250 kcmil"),
    tel("350 kcmil"),
    tel("500 kcmil"),
];
const MILS_CIRCULAIRES: [f64; 16] = [
    4_110.0, 6_530.0, 10_380.0, 16_510.0, 26_240.0, 41_740.0, 52_620.0, 66_360.0, 83_690.0,
    105_600.0, 133_100.0, 167_800.0, 211_600.0, 250_000.0, 350_000.0, 500_000.0,
];
const METAUX: &[Texte] = &[t("Cuivre", "Copper"), t("Aluminium", "Aluminum")];
/// Résistivité à 75 °C, en ohms-mils circulaires par pied.
const RESISTIVITE: [f64; 2] = [12.9, 21.2];
const CIRCUITS: &[Texte] = &[t("Monophasé", "Single-phase"), t("Triphasé", "Three-phase")];
const CIRCUITS_OU_CONTINU: &[Texte] = &[
    t("Monophasé ou continu", "Single-phase or DC"),
    t("Triphasé", "Three-phase"),
];
/// Angles de cintrage courants, et leur valeur en degrés.
const ANGLES: &[Texte] = &[
    tel("10°"),
    t("22,5°", "22.5°"),
    tel("30°"),
    tel("45°"),
    tel("60°"),
];
const DEGRES: [f64; 5] = [10.0, 22.5, 30.0, 45.0, 60.0];
/// Watts par HP : définition du cheval-vapeur électrique.
const WATTS_PAR_HP: f64 = 746.0;
/// Millimètres carrés par pouce carré (25,4 mm au pouce).
const MM2_PAR_PO2: f64 = 645.16;

pub const CALCULATEURS: &[Calculateur] = &[
    Calculateur {
        id: "loi-d-ohm",
        metier: Metier::Electricite,
        nom: t("Loi d'Ohm et puissance", "Ohm's law and power"),
        resume: t(
            "Remplissez deux des quatre grandeurs : les deux autres sont calculées.",
            "Fill in two of the four values: the other two are calculated.",
        ),
        champs: &[
            Champ::nombre(t("Tension", "Voltage"), tel("V")).facultatif(),
            Champ::nombre(t("Courant", "Current"), tel("A")).facultatif(),
            Champ::nombre(t("Résistance", "Resistance"), tel("Ω")).facultatif(),
            Champ::nombre(t("Puissance", "Power"), tel("W")).facultatif(),
        ],
        note: t(
            "Courant continu, ou charge résistive en courant alternatif.",
            "Direct current, or a resistive load on alternating current.",
        ),
        calcul: loi_d_ohm,
    },
    Calculateur {
        id: "chute-de-tension",
        metier: Metier::Electricite,
        nom: t("Chute de tension", "Voltage drop"),
        resume: t(
            "La tension perdue le long d'un conducteur.",
            "The voltage lost along a conductor.",
        ),
        champs: &[
            Champ::nombre(t("Tension du circuit", "Circuit voltage"), tel("V"))
                .court(t("Tension", "Voltage"))
                .defaut("120"),
            Champ::nombre(t("Courant", "Current"), tel("A")),
            Champ::longueur(
                t("Longueur du parcours, aller simple", "One-way run length"),
                Format::PiPo,
            )
            .court(t("Longueur", "Length")),
            Champ::choix(t("Calibre du conducteur", "Conductor size"), CALIBRES)
                .court(t("Calibre", "Size")),
            Champ::choix(t("Métal", "Metal"), METAUX),
            Champ::choix(tel("Circuit"), CIRCUITS_OU_CONTINU),
        ],
        note: t(
            "Calcul selon la résistivité du conducteur à 75 °C. Le Code fixe la chute de tension permise et le calibre minimal.",
            "Calculated from the resistivity of the conductor at 75 °C. The Code sets the permitted voltage drop and the minimum conductor size.",
        ),
        calcul: chute_de_tension,
    },
    Calculateur {
        id: "puissance",
        metier: Metier::Electricite,
        nom: t("Puissance d'un circuit", "Circuit power"),
        resume: t(
            "La puissance d'une charge, d'après sa tension et son courant.",
            "The power of a load, from its voltage and current.",
        ),
        champs: &[
            Champ::nombre(t("Tension", "Voltage"), tel("V")),
            Champ::nombre(t("Courant", "Current"), tel("A")),
            Champ::nombre(t("Facteur de puissance", "Power factor"), Texte::VIDE)
                .court(t("Facteur\npuissance", "Power\nfactor"))
                .defaut("1"),
            Champ::choix(tel("Circuit"), CIRCUITS),
        ],
        note: t(
            "En triphasé, la tension est celle entre deux phases. 1 HP vaut 746 W.",
            "In three-phase, the voltage is the line-to-line voltage. 1 HP equals 746 W.",
        ),
        calcul: puissance,
    },
    Calculateur {
        id: "cintrage",
        metier: Metier::Electricite,
        nom: t("Cintrage d'un conduit : déport", "Conduit bending: offset"),
        resume: t(
            "L'écart entre les deux coudes d'un déport, et ce que le déport fait perdre en longueur.",
            "The distance between the two bends of an offset, and the length the offset takes up.",
        ),
        champs: &[
            Champ::longueur(t("Hauteur du déport", "Offset height"), Format::PoFrac)
                .court(t("Déport", "Offset")),
            Champ::choix(t("Angle des coudes", "Bend angle"), ANGLES)
                .court(tel("Angle"))
                .defaut("2"),
        ],
        note: t(
            "Le raccourcissement est la longueur que le déport enlève au parcours du conduit.",
            "The shrink is the length the offset takes away from the conduit run.",
        ),
        calcul: cintrage,
    },
    Calculateur {
        id: "cout-d-energie",
        metier: Metier::Electricite,
        nom: t("Coût d'utilisation", "Operating cost"),
        resume: t(
            "L'énergie que consomme un appareil, et ce qu'elle coûte.",
            "The energy an appliance uses, and what it costs.",
        ),
        champs: &[
            Champ::nombre(t("Puissance de l'appareil", "Appliance power"), tel("W"))
                .court(t("Puissance", "Power")),
            Champ::nombre(t("Utilisation par jour", "Use per day"), tel("h"))
                .court(t("Heures\npar jour", "Hours\nper day")),
            Champ::entier(t("Nombre de jours", "Number of days"), Texte::VIDE)
                .court(t("Jours", "Days"))
                .defaut("30"),
            Champ::nombre(t("Tarif", "Rate"), tel("¢/kWh")),
        ],
        note: t(
            "Le tarif figure sur votre facture d'électricité.",
            "The rate is shown on your electricity bill.",
        ),
        calcul: cout_d_energie,
    },
    Calculateur {
        id: "calibre-pour-chute",
        metier: Metier::Electricite,
        nom: t(
            "Calibre selon la chute de tension",
            "Wire size for voltage drop",
        ),
        resume: t(
            "Le plus petit conducteur qui garde la chute de tension sous la limite visée.",
            "The smallest conductor that keeps the voltage drop under the target limit.",
        ),
        champs: &[
            Champ::nombre(t("Tension du circuit", "Circuit voltage"), tel("V"))
                .defaut("120")
                .court(t("Tension", "Voltage")),
            Champ::nombre(t("Courant", "Current"), tel("A")),
            Champ::longueur(
                t("Longueur du parcours, aller simple", "One-way run length"),
                Format::PiPo,
            )
            .court(t("Longueur", "Length")),
            Champ::nombre(t("Chute de tension visée", "Target voltage drop"), tel("%"))
                .defaut("3")
                .court(t("Chute\nvisée", "Target\ndrop")),
            Champ::choix(t("Métal", "Metal"), METAUX),
            Champ::choix(tel("Circuit"), CIRCUITS_OU_CONTINU),
        ],
        note: t(
            "Le calibre répond à la chute de tension seulement : le Code fixe aussi le calibre minimal d'après le courant admissible.",
            "The size addresses voltage drop only: the Code also sets the minimum size based on ampacity.",
        ),
        calcul: calibre_pour_chute,
    },
    Calculateur {
        id: "resistances",
        metier: Metier::Electricite,
        nom: t(
            "Résistances en série et en parallèle",
            "Resistors in series and parallel",
        ),
        resume: t(
            "La résistance équivalente de deux à quatre résistances.",
            "The equivalent resistance of two to four resistors.",
        ),
        champs: &[
            Champ::nombre(t("Résistance 1", "Resistor 1"), tel("Ω")).court(tel("R1")),
            Champ::nombre(t("Résistance 2", "Resistor 2"), tel("Ω")).court(tel("R2")),
            Champ::nombre(t("Résistance 3", "Resistor 3"), tel("Ω"))
                .facultatif()
                .court(tel("R3")),
            Champ::nombre(t("Résistance 4", "Resistor 4"), tel("Ω"))
                .facultatif()
                .court(tel("R4")),
        ],
        note: Texte::VIDE,
        calcul: resistances,
    },
    Calculateur {
        id: "transformateur",
        metier: Metier::Electricite,
        nom: t("Courants d'un transformateur", "Transformer currents"),
        resume: t(
            "Les courants au primaire et au secondaire d'un transformateur, d'après sa puissance.",
            "The primary and secondary currents of a transformer, from its rating.",
        ),
        champs: &[
            Champ::nombre(t("Puissance", "Power"), tel("kVA")),
            Champ::nombre(t("Tension au primaire", "Primary voltage"), tel("V"))
                .court(t("Primaire", "Primary")),
            Champ::nombre(t("Tension au secondaire", "Secondary voltage"), tel("V"))
                .court(t("Secondaire", "Secondary")),
            Champ::choix(tel("Circuit"), CIRCUITS),
        ],
        note: t(
            "Courants à pleine charge, sans les pertes du transformateur. En triphasé, les tensions sont celles entre deux phases.",
            "Full-load currents, not counting transformer losses. In three-phase, the voltages are line-to-line.",
        ),
        calcul: transformateur,
    },
    Calculateur {
        id: "moteur",
        metier: Metier::Electricite,
        nom: t("Courant d'un moteur", "Motor current"),
        resume: t(
            "Le courant à pleine charge d'un moteur, d'après sa puissance, son rendement et son facteur de puissance.",
            "The full-load current of a motor, from its power, efficiency and power factor.",
        ),
        champs: &[
            Champ::nombre(t("Puissance du moteur", "Motor power"), tel("HP"))
                .court(t("Puissance", "Power")),
            Champ::nombre(t("Tension", "Voltage"), tel("V")),
            Champ::nombre(t("Rendement", "Efficiency"), tel("%")).defaut("90"),
            Champ::nombre(t("Facteur de puissance", "Power factor"), Texte::VIDE)
                .defaut("0,85")
                .court(t("Facteur\npuissance", "Power\nfactor")),
            Champ::choix(tel("Circuit"), CIRCUITS).defaut("1"),
        ],
        note: t(
            "Calcul d'après le rendement et le facteur de puissance indiqués. La plaque signalétique du moteur et les tables du Code font foi.",
            "Calculated from the efficiency and power factor entered. The motor nameplate and the Code tables govern.",
        ),
        calcul: moteur,
    },
    Calculateur {
        id: "remplissage-de-conduit",
        metier: Metier::Electricite,
        nom: t("Remplissage d'un conduit", "Conduit fill"),
        resume: t(
            "La part de la section d'un conduit qu'occupent des conducteurs de même grosseur.",
            "The share of a conduit's cross-section taken up by conductors of the same size.",
        ),
        champs: &[
            Champ::longueur(
                t("Diamètre intérieur du conduit", "Conduit inside diameter"),
                Format::PoFrac,
            )
            .court(tel("Conduit")),
            Champ::longueur(
                t(
                    "Diamètre d'un conducteur, isolant compris",
                    "Conductor diameter, insulation included",
                ),
                Format::PoFrac,
            )
            .court(t("Conducteur", "Conductor")),
            Champ::entier(
                t("Nombre de conducteurs", "Number of conductors"),
                Texte::VIDE,
            )
            .court(t("Nombre", "Count")),
        ],
        note: t(
            "Les diamètres figurent dans les tables des fabricants. Le Code fixe le remplissage permis selon le nombre de conducteurs.",
            "Diameters are listed in the manufacturers' tables. The Code sets the permitted fill according to the number of conductors.",
        ),
        calcul: remplissage_de_conduit,
    },
    Calculateur {
        id: "eclairage",
        metier: Metier::Electricite,
        nom: t(
            "Éclairage : nombre de luminaires",
            "Lighting: number of fixtures",
        ),
        resume: t(
            "Le nombre de luminaires pour atteindre un éclairement, par la méthode des lumens.",
            "The number of fixtures to reach a target illuminance, by the lumen method.",
        ),
        champs: &[
            Champ::longueur(t("Longueur de la pièce", "Room length"), Format::PiPo)
                .court(t("Longueur", "Length")),
            Champ::longueur(t("Largeur de la pièce", "Room width"), Format::PiPo)
                .court(t("Largeur", "Width")),
            Champ::nombre(t("Éclairement visé", "Target illuminance"), tel("lux"))
                .court(t("Éclairement", "Illuminance")),
            Champ::nombre(t("Flux d'un luminaire", "Lumens per fixture"), tel("lm"))
                .court(t("Flux", "Lumens")),
            Champ::nombre(
                t("Facteur d'utilisation", "Coefficient of utilization"),
                Texte::VIDE,
            )
            .defaut("0,6")
            .court(t("Utilisation", "Utilization")),
            Champ::nombre(
                t("Facteur de maintenance", "Maintenance factor"),
                Texte::VIDE,
            )
            .defaut("0,8")
            .court(tel("Maintenance")),
        ],
        note: t(
            "Estimation : les deux facteurs dépendent du luminaire, de la pièce et de l'entretien. Un pied-bougie vaut 10,76 lux.",
            "Estimate: both factors depend on the fixture, the room and the upkeep. One foot-candle equals 10.76 lux.",
        ),
        calcul: eclairage,
    },
];

fn loi_d_ohm(e: &Lues) -> Calcul {
    let remplis = (0..4).filter(|i| e.presente(*i)).count();
    if remplis != 2 {
        return Err(t(
            "Remplissez exactement deux des quatre champs.",
            "Fill in exactly two of the four fields.",
        ));
    }
    let (u, i, r, p) = (e.nombre(0), e.nombre(1), e.nombre(2), e.nombre(3));
    let (u, i, r, p) = match (e.presente(0), e.presente(1), e.presente(2), e.presente(3)) {
        (true, true, _, _) => (u, i, u / i, u * i),
        (true, _, true, _) => (u, u / r, r, u * u / r),
        (true, _, _, true) => (u, p / u, u * u / p, p),
        (_, true, true, _) => (i * r, i, r, i * i * r),
        (_, true, _, true) => (p / i, i, p / (i * i), p),
        _ => ((p * r).sqrt(), (p / r).sqrt(), r, p),
    };
    Ok(vec![
        Resultat::nouveau(t("Tension", "Voltage"), Quantite::Mesure(u, tel("V"))),
        Resultat::nouveau(t("Courant", "Current"), Quantite::Mesure(i, tel("A"))),
        Resultat::nouveau(t("Résistance", "Resistance"), Quantite::Mesure(r, tel("Ω"))),
        Resultat::nouveau(t("Puissance", "Power"), Quantite::Mesure(p, tel("W"))),
    ])
}

fn chute_de_tension(e: &Lues) -> Calcul {
    let tension = e.nombre(0);
    let pieds = e.longueur(2) / PO_PAR_PI;
    let section = MILS_CIRCULAIRES
        .get(e.choix(3))
        .copied()
        .unwrap_or(MILS_CIRCULAIRES[0]);
    let resistivite = RESISTIVITE
        .get(e.choix(4))
        .copied()
        .unwrap_or(RESISTIVITE[0]);
    // Deux conducteurs parcourus en monophasé; racine de trois en triphasé.
    let facteur = if e.choix(5) == 1 { 3f64.sqrt() } else { 2.0 };
    let chute = facteur * resistivite * e.nombre(1) * pieds / section;
    if chute >= tension {
        return Err(t(
            "La chute dépasse la tension du circuit : vérifiez les valeurs.",
            "The drop exceeds the circuit voltage: check the values.",
        ));
    }
    Ok(vec![
        Resultat::nouveau(
            t("Chute de tension", "Voltage drop"),
            Quantite::Mesure(chute, tel("V")),
        ),
        Resultat::nouveau(
            t("Chute, en pourcentage", "Drop, as a percentage"),
            Quantite::Fixe(chute / tension * 100.0, 2, tel("%")),
        ),
        Resultat::nouveau(
            t("Tension à la charge", "Voltage at the load"),
            Quantite::Fixe(tension - chute, 1, tel("V")),
        ),
    ])
}

fn puissance(e: &Lues) -> Calcul {
    let facteur_de_puissance = e.nombre(2);
    if facteur_de_puissance > 1.0 {
        return Err(t(
            "Le facteur de puissance se situe entre 0 et 1.",
            "The power factor must be between 0 and 1.",
        ));
    }
    let phases = if e.choix(3) == 1 { 3f64.sqrt() } else { 1.0 };
    let apparente = e.nombre(0) * e.nombre(1) * phases / 1000.0;
    let active = apparente * facteur_de_puissance;
    Ok(vec![
        Resultat::nouveau(
            t("Puissance apparente", "Apparent power"),
            Quantite::Mesure(apparente, tel("kVA")),
        ),
        Resultat::nouveau(
            t("Puissance active", "Real power"),
            Quantite::Mesure(active, tel("kW")),
        ),
        Resultat::nouveau(
            t("Équivalent en HP", "Equivalent in HP"),
            Quantite::Mesure(active * 1000.0 / WATTS_PAR_HP, tel("HP")),
        ),
    ])
}

fn cintrage(e: &Lues) -> Calcul {
    let hauteur = e.longueur(0);
    let angle = DEGRES.get(e.choix(1)).copied().unwrap_or(30.0).to_radians();
    Ok(vec![
        Resultat::nouveau(
            t("Distance entre les marques", "Distance between marks"),
            Quantite::Longueur(hauteur / angle.sin()),
        ),
        Resultat::nouveau(
            t("Raccourcissement", "Shrink"),
            Quantite::Longueur(hauteur * (angle / 2.0).tan()),
        ),
    ])
}

fn cout_d_energie(e: &Lues) -> Calcul {
    let heures = e.nombre(1);
    if heures > 24.0 {
        return Err(t("Une journée compte 24 heures.", "A day has 24 hours."));
    }
    let energie = e.nombre(0) * heures * e.nombre(2) / 1000.0;
    Ok(vec![
        Resultat::nouveau(
            t("Énergie", "Energy"),
            Quantite::Mesure(energie, tel("kWh")),
        ),
        Resultat::nouveau(
            t("Coût", "Cost"),
            Quantite::Fixe(energie * e.nombre(3) / 100.0, 2, tel("$")),
        ),
    ])
}

/// Le facteur du circuit (deux conducteurs parcourus en monophasé, racine de
/// trois en triphasé) multiplié par la résistivité du métal.
fn facteur_de_chute(e: &Lues, metal: usize, circuit: usize) -> f64 {
    let resistivite = RESISTIVITE
        .get(e.choix(metal))
        .copied()
        .unwrap_or(RESISTIVITE[0]);
    let facteur = if e.choix(circuit) == 1 {
        3f64.sqrt()
    } else {
        2.0
    };
    facteur * resistivite
}

fn calibre_pour_chute(e: &Lues) -> Calcul {
    let tension = e.nombre(0);
    let visee = e.nombre(3);
    if visee >= 100.0 {
        return Err(t(
            "La chute visée doit être inférieure à 100 %.",
            "The target drop must be less than 100%.",
        ));
    }
    // Chute = facteur × résistivité × courant × longueur ÷ section.
    let produit = facteur_de_chute(e, 4, 5) * e.nombre(1) * e.longueur(2) / PO_PAR_PI;
    let section_minimale = produit / (tension * visee / 100.0);
    let Some(rang) = MILS_CIRCULAIRES
        .iter()
        .position(|section| *section >= section_minimale - 1e-9)
    else {
        return Err(t(
            "Aucun calibre de la liste ne suffit : il faut plus gros que 500 kcmil, ou des conducteurs en parallèle.",
            "No size in the list is large enough: you need larger than 500 kcmil, or parallel conductors.",
        ));
    };
    let chute = produit / MILS_CIRCULAIRES[rang];
    Ok(vec![
        Resultat::nouveau(
            t("Calibre suffisant", "Adequate size"),
            Quantite::Mot(CALIBRES[rang]),
        ),
        Resultat::nouveau(
            t("Section minimale", "Minimum cross-section"),
            Quantite::Mesure(section_minimale / 1000.0, tel("kcmil")),
        ),
        Resultat::nouveau(
            t("Chute avec ce calibre", "Drop with this size"),
            Quantite::Mesure(chute, tel("V")),
        ),
        Resultat::nouveau(
            t("Chute, en pourcentage", "Drop, as a percentage"),
            Quantite::Fixe(chute / tension * 100.0, 2, tel("%")),
        ),
    ])
}

fn resistances(e: &Lues) -> Calcul {
    let valeurs: Vec<f64> = (0..4)
        .filter(|i| e.presente(*i))
        .map(|i| e.nombre(i))
        .collect();
    let serie: f64 = valeurs.iter().sum();
    let parallele = 1.0 / valeurs.iter().map(|r| 1.0 / r).sum::<f64>();
    Ok(vec![
        Resultat::nouveau(
            t("En série", "In series"),
            Quantite::Mesure(serie, tel("Ω")),
        ),
        Resultat::nouveau(
            t("En parallèle", "In parallel"),
            Quantite::Mesure(parallele, tel("Ω")),
        ),
    ])
}

/// Racine de trois en triphasé, un en monophasé.
fn phases(e: &Lues, circuit: usize) -> f64 {
    if e.choix(circuit) == 1 {
        3f64.sqrt()
    } else {
        1.0
    }
}

fn transformateur(e: &Lues) -> Calcul {
    let voltamperes = e.nombre(0) * 1000.0;
    let phases = phases(e, 3);
    Ok(vec![
        Resultat::nouveau(
            t("Courant au primaire", "Primary current"),
            Quantite::Mesure(voltamperes / (e.nombre(1) * phases), tel("A")),
        ),
        Resultat::nouveau(
            t("Courant au secondaire", "Secondary current"),
            Quantite::Mesure(voltamperes / (e.nombre(2) * phases), tel("A")),
        ),
        Resultat::nouveau(
            t("Rapport de transformation", "Turns ratio"),
            Quantite::Fixe(e.nombre(1) / e.nombre(2), 2, Texte::VIDE),
        ),
    ])
}

fn moteur(e: &Lues) -> Calcul {
    let (rendement, facteur_de_puissance) = (e.nombre(2), e.nombre(3));
    if rendement > 100.0 {
        return Err(t(
            "Le rendement ne dépasse pas 100 %.",
            "Efficiency cannot exceed 100%.",
        ));
    }
    if facteur_de_puissance > 1.0 {
        return Err(t(
            "Le facteur de puissance se situe entre 0 et 1.",
            "The power factor must be between 0 and 1.",
        ));
    }
    let a_l_arbre = e.nombre(0) * WATTS_PAR_HP / 1000.0;
    let absorbee = a_l_arbre / (rendement / 100.0);
    let courant = absorbee * 1000.0 / (e.nombre(1) * facteur_de_puissance * phases(e, 4));
    Ok(vec![
        Resultat::nouveau(
            t("Puissance à l'arbre", "Shaft power"),
            Quantite::Mesure(a_l_arbre, tel("kW")),
        ),
        Resultat::nouveau(
            t("Puissance absorbée", "Input power"),
            Quantite::Mesure(absorbee, tel("kW")),
        ),
        Resultat::nouveau(
            t("Courant à pleine charge", "Full-load current"),
            Quantite::Mesure(courant, tel("A")),
        ),
    ])
}

fn remplissage_de_conduit(e: &Lues) -> Calcul {
    let section = |diametre: f64| PI / 4.0 * diametre * diametre;
    let conduit = section(e.longueur(0));
    let conducteurs = section(e.longueur(1)) * e.nombre(2);
    if conducteurs >= conduit {
        return Err(t(
            "Les conducteurs n'entrent pas dans ce conduit.",
            "The conductors do not fit in this conduit.",
        ));
    }
    Ok(vec![
        Resultat::nouveau(
            t("Section du conduit", "Conduit cross-section"),
            Quantite::Double(conduit, t("po²", "in²"), conduit * MM2_PAR_PO2, tel("mm²")),
        ),
        Resultat::nouveau(
            t("Section des conducteurs", "Conductor cross-section"),
            Quantite::Double(
                conducteurs,
                t("po²", "in²"),
                conducteurs * MM2_PAR_PO2,
                tel("mm²"),
            ),
        ),
        Resultat::nouveau(
            t("Remplissage", "Fill"),
            Quantite::Fixe(conducteurs / conduit * 100.0, 1, tel("%")),
        ),
    ])
}

fn eclairage(e: &Lues) -> Calcul {
    if e.nombre(4) > 1.0 || e.nombre(5) > 1.0 {
        return Err(t(
            "Les deux facteurs se situent entre 0 et 1.",
            "Both factors must be between 0 and 1.",
        ));
    }
    let surface = e.longueur(0) * e.longueur(1);
    // Des lux sont des lumens au mètre carré.
    let flux = e.nombre(2) * surface / PO2_PAR_M2 / (e.nombre(4) * e.nombre(5));
    Ok(vec![
        Resultat::nouveau(t("Surface", "Area"), Quantite::Aire(surface)),
        Resultat::nouveau(
            t("Flux lumineux nécessaire", "Lumens required"),
            Quantite::Mesure(flux, tel("lm")),
        ),
        Resultat::nouveau(
            t("Luminaires", "Fixtures"),
            Quantite::Compte(
                arrondi_au_dessus(flux / e.nombre(3)),
                t("luminaires", "fixtures"),
            ),
        ),
    ])
}
