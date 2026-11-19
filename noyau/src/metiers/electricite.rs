//! Électricité (section 22.6).

use super::{Calcul, Calculateur, Champ, Lues, Metier, Quantite, Resultat};
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
const CIRCUITS_OU_CONTINU: &[&str] = &["Monophasé ou courant continu", "Triphasé"];
/// Angles de cintrage courants, et leur valeur en degrés.
const ANGLES: &[&str] = &["10°", "22,5°", "30°", "45°", "60°"];
const DEGRES: [f64; 5] = [10.0, 22.5, 30.0, 45.0, 60.0];
/// Watts par HP : définition du cheval-vapeur électrique.
const WATTS_PAR_HP: f64 = 746.0;

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
            Champ::nombre("Tension du circuit", "V").defaut("120"),
            Champ::nombre("Courant", "A"),
            Champ::longueur("Longueur du parcours, aller simple", Format::PiPo),
            Champ::choix("Calibre du conducteur", CALIBRES),
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
            Champ::nombre("Facteur de puissance", "").defaut("1"),
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
            Champ::longueur("Hauteur du déport", Format::PoFrac),
            Champ::choix("Angle des coudes", ANGLES).defaut("2"),
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
            Champ::nombre("Puissance de l'appareil", "W"),
            Champ::nombre("Utilisation par jour", "h"),
            Champ::entier("Nombre de jours", "").defaut("30"),
            Champ::nombre("Tarif", "¢/kWh"),
        ],
        note: "Le tarif figure sur votre facture d'électricité.",
        calcul: cout_d_energie,
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
