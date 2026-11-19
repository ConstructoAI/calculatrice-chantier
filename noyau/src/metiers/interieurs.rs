//! Gypse et isolation : panneaux, cloisons, isolant, plafonds (section 22.12).

use super::{Calcul, Calculateur, Champ, Lues, Metier, Quantite, Resultat, arrondi_au_dessus};
use crate::unites::{Format, PO2_PAR_PI2};

/// Surface déduite pour une porte et pour une fenêtre, en pieds carrés.
const PI2_PAR_PORTE: f64 = 21.0;
const PI2_PAR_FENETRE: f64 = 15.0;
const SURFACES: &[&str] = &["Murs seulement", "Murs et plafond"];
/// Panneaux de gypse courants, et leur surface en pieds carrés.
const PANNEAUX: &[&str] = &["4 pi × 8 pi", "4 pi × 10 pi", "4 pi × 12 pi"];
const PI2_PAR_PANNEAU: [f64; 3] = [32.0, 40.0, 48.0];
const LISSES: &[&str] = &["Une en bas, une en haut", "Une en bas, deux en haut"];
/// Tuiles de plafond suspendu, et leur surface en pieds carrés.
const TUILES: &[&str] = &["2 pi × 4 pi", "2 pi × 2 pi"];
const PI2_PAR_TUILE: [f64; 2] = [8.0, 4.0];
const UNITES_D_ISOLATION: &[&str] = &["R (impérial)", "RSI (métrique)"];
/// Valeur R d'une résistance thermique de 1 RSI (1 m²·K/W en h·pi²·°F/BTU).
const R_PAR_RSI: f64 = 5.678_263;

const PERTES: Champ = Champ::nombre("Pertes", "%").defaut("10").nul_permis();

pub const CALCULATEURS: &[Calculateur] = &[
    Calculateur {
        id: "gypse",
        metier: Metier::Interieurs,
        nom: "Panneaux de gypse d'une pièce",
        resume: "Le nombre de panneaux de gypse pour les murs d'une pièce, avec ou sans le plafond.",
        champs: &[
            Champ::longueur("Longueur de la pièce", Format::PiPo).court("Longueur"),
            Champ::longueur("Largeur de la pièce", Format::PiPo).court("Largeur"),
            Champ::longueur("Hauteur des murs", Format::PiPo)
                .defaut("8")
                .court("Hauteur"),
            Champ::entier("Portes (21 pi² chacune)", "")
                .defaut("1")
                .nul_permis()
                .court("Portes"),
            Champ::entier("Fenêtres (15 pi² chacune)", "")
                .defaut("1")
                .nul_permis()
                .court("Fenêtres"),
            Champ::choix("Surfaces à couvrir", SURFACES)
                .defaut("1")
                .court("Surfaces"),
            Champ::choix("Panneau", PANNEAUX),
            PERTES,
        ],
        note: "Les ouvertures sont déduites. Prévoyez plus de pertes pour une pièce très découpée.",
        calcul: gypse,
    },
    Calculateur {
        id: "montants-de-cloison",
        metier: Metier::Interieurs,
        nom: "Montants d'une cloison",
        resume: "Le nombre de montants d'une cloison droite et la longueur de ses lisses.",
        champs: &[
            Champ::longueur("Longueur de la cloison", Format::PiPo).court("Longueur"),
            Champ::longueur("Espacement des montants", Format::PoFrac)
                .defaut("16")
                .court("Espacement"),
            Champ::entier("Montants en plus", "")
                .defaut("0")
                .nul_permis()
                .court("En plus"),
            Champ::choix("Lisses", LISSES),
        ],
        note: "Un montant à chaque bout, puis un à chaque espacement. Ajoutez les montants des coins, des intersections et des ouvertures.",
        calcul: montants_de_cloison,
    },
    Calculateur {
        id: "isolant",
        metier: Metier::Interieurs,
        nom: "Isolant en nattes ou en panneaux",
        resume: "Le nombre de paquets d'isolant pour couvrir une surface.",
        champs: &[
            Champ::longueur("Longueur", Format::PiPo),
            Champ::longueur("Hauteur ou largeur", Format::PiPo).court("Hauteur"),
            Champ::nombre("Pertes", "%").defaut("5").nul_permis(),
            Champ::nombre("Couverture d'un paquet", "pi²").court("Par paquet"),
        ],
        note: "La couverture est inscrite sur l'emballage : elle dépend de l'épaisseur de l'isolant et de l'espacement des montants.",
        calcul: isolant,
    },
    Calculateur {
        id: "valeur-r",
        metier: Metier::Interieurs,
        nom: "Valeur R et valeur RSI",
        resume: "La même résistance thermique en valeur R et en valeur RSI, et le coefficient U.",
        champs: &[
            Champ::nombre("Résistance thermique", "").court("Résistance"),
            Champ::choix("Unité", UNITES_D_ISOLATION),
        ],
        note: "La valeur R vaut 5,678 fois la valeur RSI. Le coefficient U est l'inverse de la valeur RSI.",
        calcul: valeur_r,
    },
    Calculateur {
        id: "plafond-suspendu",
        metier: Metier::Interieurs,
        nom: "Plafond suspendu",
        resume: "Les tuiles d'un plafond suspendu et la cornière qui fait le tour de la pièce.",
        champs: &[
            Champ::longueur("Longueur de la pièce", Format::PiPo).court("Longueur"),
            Champ::longueur("Largeur de la pièce", Format::PiPo).court("Largeur"),
            Champ::choix("Tuile", TUILES),
            PERTES,
        ],
        note: "Les tés porteurs et les entretoises dépendent du plan de pose : comptez-les d'après le calepinage.",
        calcul: plafond_suspendu,
    },
];

fn gypse(e: &Lues) -> Calcul {
    let (longueur, largeur, hauteur) = (e.longueur(0), e.longueur(1), e.longueur(2));
    let ouvertures = (e.nombre(3) * PI2_PAR_PORTE + e.nombre(4) * PI2_PAR_FENETRE) * PO2_PAR_PI2;
    let murs = 2.0 * (longueur + largeur) * hauteur - ouvertures;
    if murs <= 0.0 {
        return Err("Les ouvertures couvrent toute la surface des murs.");
    }
    let plafond = if e.choix(5) == 1 {
        longueur * largeur
    } else {
        0.0
    };
    let surface = murs + plafond;
    let panneau = PI2_PAR_PANNEAU
        .get(e.choix(6))
        .copied()
        .unwrap_or(PI2_PAR_PANNEAU[0])
        * PO2_PAR_PI2;
    let panneaux = arrondi_au_dessus(surface * (1.0 + e.nombre(7) / 100.0) / panneau);
    Ok(vec![
        Resultat::nouveau("Surface à couvrir", Quantite::Aire(surface)),
        Resultat::nouveau("Panneaux", Quantite::Compte(panneaux, "panneaux")),
    ])
}

fn montants_de_cloison(e: &Lues) -> Calcul {
    let longueur = e.longueur(0);
    // Un montant à chaque bout, puis un à chaque espacement.
    let montants = arrondi_au_dessus(longueur / e.longueur(1)) + 1.0 + e.nombre(2);
    let lisses = if e.choix(3) == 1 { 3.0 } else { 2.0 };
    Ok(vec![
        Resultat::nouveau("Montants", Quantite::Compte(montants, "montants")),
        Resultat::nouveau("Longueur de lisses", Quantite::Longueur(longueur * lisses)),
    ])
}

fn isolant(e: &Lues) -> Calcul {
    let surface = e.longueur(0) * e.longueur(1);
    let a_couvrir = surface * (1.0 + e.nombre(2) / 100.0);
    Ok(vec![
        Resultat::nouveau("Surface", Quantite::Aire(surface)),
        Resultat::nouveau(
            "Surface à couvrir, pertes comprises",
            Quantite::Aire(a_couvrir),
        ),
        Resultat::nouveau(
            "Paquets",
            Quantite::Compte(
                arrondi_au_dessus(a_couvrir / PO2_PAR_PI2 / e.nombre(3)),
                "paquets",
            ),
        ),
    ])
}

fn valeur_r(e: &Lues) -> Calcul {
    let rsi = if e.choix(1) == 1 {
        e.nombre(0)
    } else {
        e.nombre(0) / R_PAR_RSI
    };
    Ok(vec![
        Resultat::nouveau("Valeur R", Quantite::Fixe(rsi * R_PAR_RSI, 1, "")),
        Resultat::nouveau("Valeur RSI", Quantite::Fixe(rsi, 2, "")),
        Resultat::nouveau("Coefficient U", Quantite::Fixe(1.0 / rsi, 3, "W/m²·K")),
    ])
}

fn plafond_suspendu(e: &Lues) -> Calcul {
    let (longueur, largeur) = (e.longueur(0), e.longueur(1));
    let surface = longueur * largeur;
    let tuile = PI2_PAR_TUILE
        .get(e.choix(2))
        .copied()
        .unwrap_or(PI2_PAR_TUILE[0])
        * PO2_PAR_PI2;
    let tuiles = arrondi_au_dessus(surface * (1.0 + e.nombre(3) / 100.0) / tuile);
    Ok(vec![
        Resultat::nouveau("Surface", Quantite::Aire(surface)),
        Resultat::nouveau("Tuiles", Quantite::Compte(tuiles, "tuiles")),
        Resultat::nouveau(
            "Cornière murale",
            Quantite::Longueur(2.0 * (longueur + largeur)),
        ),
    ])
}
