//! Électricité (section 22.6).

use std::f64::consts::PI;

use super::{
    Calcul, Calculateur, Champ, Lues, Metier, PO2_PAR_M2, Quantite, Resultat, arrondi_au_dessus,
};
use crate::unites::{Format, PO_PAR_PI};

/// Calibres des conducteurs et leur section en mils circulaires.
const CALIBRES: &[&str] = &[
    "14 AWG",
    "12 AWG",
    "10 AWG",
    "8 AWG",
    "6 AWG",
    "4 AWG",
    "3 AWG",
    "2 AWG",
    "1 AWG",
    "1/0 AWG",
    "2/0 AWG",
    "3/0 AWG",
    "4/0 AWG",
    "250 kcmil",
    "350 kcmil",
    "500 kcmil",
];
const MILS_CIRCULAIRES: [f64; 16] = [
    4_110.0, 6_530.0, 10_380.0, 16_510.0, 26_240.0, 41_740.0, 52_620.0, 66_360.0, 83_690.0,
    105_600.0, 133_100.0, 167_800.0, 211_600.0, 250_000.0, 350_000.0, 500_000.0,
];
const METAUX: &[&str] = &["Cuivre", "Aluminium"];
/// Résistivité à 75 °C, en ohms-mils circulaires par pied.
const RESISTIVITE: [f64; 2] = [12.9, 21.2];
const CIRCUITS: &[&str] = &["Monophasé", "Triphasé"];
const CIRCUITS_OU_CONTINU: &[&str] = &["Monophasé ou continu", "Triphasé"];
/// Angles de cintrage courants, et leur valeur en degrés.
const ANGLES: &[&str] = &["10°", "22,5°", "30°", "45°", "60°"];
const DEGRES: [f64; 5] = [10.0, 22.5, 30.0, 45.0, 60.0];
/// Watts par HP : définition du cheval-vapeur électrique.
const WATTS_PAR_HP: f64 = 746.0;
/// Millimètres carrés par pouce carré (25,4 mm au pouce).
const MM2_PAR_PO2: f64 = 645.16;

pub const CALCULATEURS: &[Calculateur] = &[
    Calculateur {
        id: "loi-d-ohm",
        metier: Metier::Electricite,
        nom: "Loi d'Ohm et puissance",
        resume: "Remplissez deux des quatre grandeurs : les deux autres sont calculées.",
        champs: &[
            Champ::nombre("Tension", "V").facultatif(),
            Champ::nombre("Courant", "A").facultatif(),
            Champ::nombre("Résistance", "Ω").facultatif(),
            Champ::nombre("Puissance", "W").facultatif(),
        ],
        note: "Courant continu, ou charge résistive en courant alternatif.",
        calcul: loi_d_ohm,
    },
    Calculateur {
        id: "chute-de-tension",
        metier: Metier::Electricite,
        nom: "Chute de tension",
        resume: "La tension perdue le long d'un conducteur.",
        champs: &[
            Champ::nombre("Tension du circuit", "V")
                .court("Tension")
                .defaut("120"),
            Champ::nombre("Courant", "A"),
            Champ::longueur("Longueur du parcours, aller simple", Format::PiPo).court("Longueur"),
            Champ::choix("Calibre du conducteur", CALIBRES).court("Calibre"),
            Champ::choix("Métal", METAUX),
            Champ::choix("Circuit", CIRCUITS_OU_CONTINU),
        ],
        note: "Calcul selon la résistivité du conducteur à 75 °C. Le Code fixe la chute de tension permise et le calibre minimal.",
        calcul: chute_de_tension,
    },
    Calculateur {
        id: "puissance",
        metier: Metier::Electricite,
        nom: "Puissance d'un circuit",
        resume: "La puissance d'une charge, d'après sa tension et son courant.",
        champs: &[
            Champ::nombre("Tension", "V"),
            Champ::nombre("Courant", "A"),
            Champ::nombre("Facteur de puissance", "")
                .court("Facteur\npuissance")
                .defaut("1"),
            Champ::choix("Circuit", CIRCUITS),
        ],
        note: "En triphasé, la tension est celle entre deux phases. 1 HP vaut 746 W.",
        calcul: puissance,
    },
    Calculateur {
        id: "cintrage",
        metier: Metier::Electricite,
        nom: "Cintrage d'un conduit : déport",
        resume: "L'écart entre les deux coudes d'un déport, et ce que le déport fait perdre en longueur.",
        champs: &[
            Champ::longueur("Hauteur du déport", Format::PoFrac).court("Déport"),
            Champ::choix("Angle des coudes", ANGLES)
                .court("Angle")
                .defaut("2"),
        ],
        note: "Le raccourcissement est la longueur que le déport enlève au parcours du conduit.",
        calcul: cintrage,
    },
    Calculateur {
        id: "cout-d-energie",
        metier: Metier::Electricite,
        nom: "Coût d'utilisation",
        resume: "L'énergie que consomme un appareil, et ce qu'elle coûte.",
        champs: &[
            Champ::nombre("Puissance de l'appareil", "W").court("Puissance"),
            Champ::nombre("Utilisation par jour", "h").court("Heures\npar jour"),
            Champ::entier("Nombre de jours", "")
                .court("Jours")
                .defaut("30"),
            Champ::nombre("Tarif", "¢/kWh"),
        ],
        note: "Le tarif figure sur votre facture d'électricité.",
        calcul: cout_d_energie,
    },
    Calculateur {
        id: "calibre-pour-chute",
        metier: Metier::Electricite,
        nom: "Calibre selon la chute de tension",
        resume: "Le plus petit conducteur qui garde la chute de tension sous la limite visée.",
        champs: &[
            Champ::nombre("Tension du circuit", "V")
                .defaut("120")
                .court("Tension"),
            Champ::nombre("Courant", "A"),
            Champ::longueur("Longueur du parcours, aller simple", Format::PiPo).court("Longueur"),
            Champ::nombre("Chute de tension visée", "%")
                .defaut("3")
                .court("Chute\nvisée"),
            Champ::choix("Métal", METAUX),
            Champ::choix("Circuit", CIRCUITS_OU_CONTINU),
        ],
        note: "Le calibre répond à la chute de tension seulement : le Code fixe aussi le calibre minimal d'après le courant admissible.",
        calcul: calibre_pour_chute,
    },
    Calculateur {
        id: "resistances",
        metier: Metier::Electricite,
        nom: "Résistances en série et en parallèle",
        resume: "La résistance équivalente de deux à quatre résistances.",
        champs: &[
            Champ::nombre("Résistance 1", "Ω").court("R1"),
            Champ::nombre("Résistance 2", "Ω").court("R2"),
            Champ::nombre("Résistance 3", "Ω").facultatif().court("R3"),
            Champ::nombre("Résistance 4", "Ω").facultatif().court("R4"),
        ],
        note: "",
        calcul: resistances,
    },
    Calculateur {
        id: "transformateur",
        metier: Metier::Electricite,
        nom: "Courants d'un transformateur",
        resume: "Les courants au primaire et au secondaire d'un transformateur, d'après sa puissance.",
        champs: &[
            Champ::nombre("Puissance", "kVA"),
            Champ::nombre("Tension au primaire", "V").court("Primaire"),
            Champ::nombre("Tension au secondaire", "V").court("Secondaire"),
            Champ::choix("Circuit", CIRCUITS),
        ],
        note: "Courants à pleine charge, sans les pertes du transformateur. En triphasé, les tensions sont celles entre deux phases.",
        calcul: transformateur,
    },
    Calculateur {
        id: "moteur",
        metier: Metier::Electricite,
        nom: "Courant d'un moteur",
        resume: "Le courant à pleine charge d'un moteur, d'après sa puissance, son rendement et son facteur de puissance.",
        champs: &[
            Champ::nombre("Puissance du moteur", "HP").court("Puissance"),
            Champ::nombre("Tension", "V"),
            Champ::nombre("Rendement", "%").defaut("90"),
            Champ::nombre("Facteur de puissance", "")
                .defaut("0,85")
                .court("Facteur\npuissance"),
            Champ::choix("Circuit", CIRCUITS).defaut("1"),
        ],
        note: "Calcul d'après le rendement et le facteur de puissance indiqués. La plaque signalétique du moteur et les tables du Code font foi.",
        calcul: moteur,
    },
    Calculateur {
        id: "remplissage-de-conduit",
        metier: Metier::Electricite,
        nom: "Remplissage d'un conduit",
        resume: "La part de la section d'un conduit qu'occupent des conducteurs de même grosseur.",
        champs: &[
            Champ::longueur("Diamètre intérieur du conduit", Format::PoFrac).court("Conduit"),
            Champ::longueur("Diamètre d'un conducteur, isolant compris", Format::PoFrac)
                .court("Conducteur"),
            Champ::entier("Nombre de conducteurs", "").court("Nombre"),
        ],
        note: "Les diamètres figurent dans les tables des fabricants. Le Code fixe le remplissage permis selon le nombre de conducteurs.",
        calcul: remplissage_de_conduit,
    },
    Calculateur {
        id: "eclairage",
        metier: Metier::Electricite,
        nom: "Éclairage : nombre de luminaires",
        resume: "Le nombre de luminaires pour atteindre un éclairement, par la méthode des lumens.",
        champs: &[
            Champ::longueur("Longueur de la pièce", Format::PiPo).court("Longueur"),
            Champ::longueur("Largeur de la pièce", Format::PiPo).court("Largeur"),
            Champ::nombre("Éclairement visé", "lux").court("Éclairement"),
            Champ::nombre("Flux d'un luminaire", "lm").court("Flux"),
            Champ::nombre("Facteur d'utilisation", "")
                .defaut("0,6")
                .court("Utilisation"),
            Champ::nombre("Facteur de maintenance", "")
                .defaut("0,8")
                .court("Maintenance"),
        ],
        note: "Estimation : les deux facteurs dépendent du luminaire, de la pièce et de l'entretien. Un pied-bougie vaut 10,76 lux.",
        calcul: eclairage,
    },
];

fn loi_d_ohm(e: &Lues) -> Calcul {
    let remplis = (0..4).filter(|i| e.presente(*i)).count();
    if remplis != 2 {
        return Err("Remplissez exactement deux des quatre champs.");
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
        Resultat::nouveau("Tension", Quantite::Mesure(u, "V")),
        Resultat::nouveau("Courant", Quantite::Mesure(i, "A")),
        Resultat::nouveau("Résistance", Quantite::Mesure(r, "Ω")),
        Resultat::nouveau("Puissance", Quantite::Mesure(p, "W")),
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
        return Err("La chute dépasse la tension du circuit : vérifiez les valeurs.");
    }
    Ok(vec![
        Resultat::nouveau("Chute de tension", Quantite::Mesure(chute, "V")),
        Resultat::nouveau(
            "Chute, en pourcentage",
            Quantite::Fixe(chute / tension * 100.0, 2, "%"),
        ),
        Resultat::nouveau(
            "Tension à la charge",
            Quantite::Fixe(tension - chute, 1, "V"),
        ),
    ])
}

fn puissance(e: &Lues) -> Calcul {
    let facteur_de_puissance = e.nombre(2);
    if facteur_de_puissance > 1.0 {
        return Err("Le facteur de puissance se situe entre 0 et 1.");
    }
    let phases = if e.choix(3) == 1 { 3f64.sqrt() } else { 1.0 };
    let apparente = e.nombre(0) * e.nombre(1) * phases / 1000.0;
    let active = apparente * facteur_de_puissance;
    Ok(vec![
        Resultat::nouveau("Puissance apparente", Quantite::Mesure(apparente, "kVA")),
        Resultat::nouveau("Puissance active", Quantite::Mesure(active, "kW")),
        Resultat::nouveau(
            "Équivalent en HP",
            Quantite::Mesure(active * 1000.0 / WATTS_PAR_HP, "HP"),
        ),
    ])
}

fn cintrage(e: &Lues) -> Calcul {
    let hauteur = e.longueur(0);
    let angle = DEGRES.get(e.choix(1)).copied().unwrap_or(30.0).to_radians();
    Ok(vec![
        Resultat::nouveau(
            "Distance entre les marques",
            Quantite::Longueur(hauteur / angle.sin()),
        ),
        Resultat::nouveau(
            "Raccourcissement",
            Quantite::Longueur(hauteur * (angle / 2.0).tan()),
        ),
    ])
}

fn cout_d_energie(e: &Lues) -> Calcul {
    let heures = e.nombre(1);
    if heures > 24.0 {
        return Err("Une journée compte 24 heures.");
    }
    let energie = e.nombre(0) * heures * e.nombre(2) / 1000.0;
    Ok(vec![
        Resultat::nouveau("Énergie", Quantite::Mesure(energie, "kWh")),
        Resultat::nouveau(
            "Coût",
            Quantite::Fixe(energie * e.nombre(3) / 100.0, 2, "$"),
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
        return Err("La chute visée doit être inférieure à 100 %.");
    }
    // Chute = facteur × résistivité × courant × longueur ÷ section.
    let produit = facteur_de_chute(e, 4, 5) * e.nombre(1) * e.longueur(2) / PO_PAR_PI;
    let section_minimale = produit / (tension * visee / 100.0);
    let Some(rang) = MILS_CIRCULAIRES
        .iter()
        .position(|section| *section >= section_minimale - 1e-9)
    else {
        return Err(
            "Aucun calibre de la liste ne suffit : il faut plus gros que 500 kcmil, ou des conducteurs en parallèle.",
        );
    };
    let chute = produit / MILS_CIRCULAIRES[rang];
    Ok(vec![
        Resultat::nouveau("Calibre suffisant", Quantite::Mot(CALIBRES[rang])),
        Resultat::nouveau(
            "Section minimale",
            Quantite::Mesure(section_minimale / 1000.0, "kcmil"),
        ),
        Resultat::nouveau("Chute avec ce calibre", Quantite::Mesure(chute, "V")),
        Resultat::nouveau(
            "Chute, en pourcentage",
            Quantite::Fixe(chute / tension * 100.0, 2, "%"),
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
        Resultat::nouveau("En série", Quantite::Mesure(serie, "Ω")),
        Resultat::nouveau("En parallèle", Quantite::Mesure(parallele, "Ω")),
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
            "Courant au primaire",
            Quantite::Mesure(voltamperes / (e.nombre(1) * phases), "A"),
        ),
        Resultat::nouveau(
            "Courant au secondaire",
            Quantite::Mesure(voltamperes / (e.nombre(2) * phases), "A"),
        ),
        Resultat::nouveau(
            "Rapport de transformation",
            Quantite::Fixe(e.nombre(1) / e.nombre(2), 2, ""),
        ),
    ])
}

fn moteur(e: &Lues) -> Calcul {
    let (rendement, facteur_de_puissance) = (e.nombre(2), e.nombre(3));
    if rendement > 100.0 {
        return Err("Le rendement ne dépasse pas 100 %.");
    }
    if facteur_de_puissance > 1.0 {
        return Err("Le facteur de puissance se situe entre 0 et 1.");
    }
    let a_l_arbre = e.nombre(0) * WATTS_PAR_HP / 1000.0;
    let absorbee = a_l_arbre / (rendement / 100.0);
    let courant = absorbee * 1000.0 / (e.nombre(1) * facteur_de_puissance * phases(e, 4));
    Ok(vec![
        Resultat::nouveau("Puissance à l'arbre", Quantite::Mesure(a_l_arbre, "kW")),
        Resultat::nouveau("Puissance absorbée", Quantite::Mesure(absorbee, "kW")),
        Resultat::nouveau("Courant à pleine charge", Quantite::Mesure(courant, "A")),
    ])
}

fn remplissage_de_conduit(e: &Lues) -> Calcul {
    let section = |diametre: f64| PI / 4.0 * diametre * diametre;
    let conduit = section(e.longueur(0));
    let conducteurs = section(e.longueur(1)) * e.nombre(2);
    if conducteurs >= conduit {
        return Err("Les conducteurs n'entrent pas dans ce conduit.");
    }
    Ok(vec![
        Resultat::nouveau(
            "Section du conduit",
            Quantite::Double(conduit, "po²", conduit * MM2_PAR_PO2, "mm²"),
        ),
        Resultat::nouveau(
            "Section des conducteurs",
            Quantite::Double(conducteurs, "po²", conducteurs * MM2_PAR_PO2, "mm²"),
        ),
        Resultat::nouveau(
            "Remplissage",
            Quantite::Fixe(conducteurs / conduit * 100.0, 1, "%"),
        ),
    ])
}

fn eclairage(e: &Lues) -> Calcul {
    if e.nombre(4) > 1.0 || e.nombre(5) > 1.0 {
        return Err("Les deux facteurs se situent entre 0 et 1.");
    }
    let surface = e.longueur(0) * e.longueur(1);
    // Des lux sont des lumens au mètre carré.
    let flux = e.nombre(2) * surface / PO2_PAR_M2 / (e.nombre(4) * e.nombre(5));
    Ok(vec![
        Resultat::nouveau("Surface", Quantite::Aire(surface)),
        Resultat::nouveau("Flux lumineux nécessaire", Quantite::Mesure(flux, "lm")),
        Resultat::nouveau(
            "Luminaires",
            Quantite::Compte(arrondi_au_dessus(flux / e.nombre(3)), "luminaires"),
        ),
    ])
}
