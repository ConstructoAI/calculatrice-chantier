//! Finition : peinture, céramique, revêtements de sol (section 22.7).

use super::{Calcul, Calculateur, Champ, Lues, Metier, Quantite, Resultat, arrondi_au_dessus};
use crate::unites::{Format, PO2_PAR_PI2};

/// Surface déduite pour une porte et pour une fenêtre, en pieds carrés.
const PI2_PAR_PORTE: f64 = 21.0;
const PI2_PAR_FENETRE: f64 = 15.0;
/// Contenance du contenant de peinture courant, en litres.
const LITRES_PAR_CONTENANT: f64 = 3.78;
const SURFACES: &[&str] = &["Murs seulement", "Murs et plafond"];

const PERTES: Champ = Champ::nombre("Pertes", "%").defaut("10").nul_permis();

pub const CALCULATEURS: &[Calculateur] = &[
    Calculateur {
        id: "peinture",
        metier: Metier::Finition,
        nom: "Peinture d'une pièce",
        resume: "La surface à peindre d'une pièce rectangulaire et la peinture qu'il faut.",
        champs: &[
            Champ::longueur("Longueur de la pièce", Format::PiPo),
            Champ::longueur("Largeur de la pièce", Format::PiPo),
            Champ::longueur("Hauteur des murs", Format::PiPo).defaut("8"),
            Champ::entier("Portes (21 pi² chacune)", "")
                .defaut("1")
                .nul_permis(),
            Champ::entier("Fenêtres (15 pi² chacune)", "")
                .defaut("1")
                .nul_permis(),
            Champ::choix("Surfaces à peindre", SURFACES),
            Champ::entier("Couches", "").defaut("2"),
            Champ::nombre("Rendement d'un contenant de 3,78 L", "pi²").defaut("350"),
        ],
        note: "Le rendement est inscrit sur le contenant : de 300 à 400 pi² selon la peinture et la surface.",
        calcul: peinture,
    },
    Calculateur {
        id: "ceramique",
        metier: Metier::Finition,
        nom: "Céramique et tuiles",
        resume: "Le nombre de tuiles pour couvrir une surface rectangulaire.",
        champs: &[
            Champ::longueur("Longueur de la surface", Format::PiPo),
            Champ::longueur("Largeur de la surface", Format::PiPo),
            Champ::longueur("Longueur d'une tuile", Format::PoFrac),
            Champ::longueur("Largeur d'une tuile", Format::PoFrac),
            PERTES,
            Champ::entier("Tuiles par boîte", "").facultatif(),
        ],
        note: "Les joints ne sont pas comptés. Prévoyez plus de pertes pour une pose en diagonale.",
        calcul: ceramique,
    },
    Calculateur {
        id: "revetement-de-sol",
        metier: Metier::Finition,
        nom: "Revêtement de sol en boîtes",
        resume: "Le nombre de boîtes de bois franc, de stratifié ou de vinyle pour un plancher.",
        champs: &[
            Champ::longueur("Longueur du plancher", Format::PiPo),
            Champ::longueur("Largeur du plancher", Format::PiPo),
            PERTES,
            Champ::nombre("Couverture d'une boîte", "pi²"),
        ],
        note: "",
        calcul: revetement_de_sol,
    },
];

fn peinture(e: &Lues) -> Calcul {
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
    let contenants = surface / PO2_PAR_PI2 * e.nombre(6) / e.nombre(7);
    Ok(vec![
        Resultat::nouveau("Surface à peindre", Quantite::Aire(surface)),
        Resultat::nouveau(
            "Peinture nécessaire",
            Quantite::Mesure(contenants * LITRES_PAR_CONTENANT, "L"),
        ),
        Resultat::nouveau(
            "Contenants de 3,78 L",
            Quantite::Compte(arrondi_au_dessus(contenants), "contenants"),
        ),
    ])
}

fn ceramique(e: &Lues) -> Calcul {
    let surface = e.longueur(0) * e.longueur(1);
    let tuile = e.longueur(2) * e.longueur(3);
    let tuiles = arrondi_au_dessus(surface * (1.0 + e.nombre(4) / 100.0) / tuile);
    let mut resultats = vec![
        Resultat::nouveau("Surface", Quantite::Aire(surface)),
        Resultat::nouveau("Tuiles nécessaires", Quantite::Compte(tuiles, "tuiles")),
    ];
    if e.presente(5) {
        resultats.push(Resultat::nouveau(
            "Boîtes",
            Quantite::Compte(arrondi_au_dessus(tuiles / e.nombre(5)), "boîtes"),
        ));
    }
    Ok(resultats)
}

fn revetement_de_sol(e: &Lues) -> Calcul {
    let surface = e.longueur(0) * e.longueur(1);
    let a_couvrir = surface * (1.0 + e.nombre(2) / 100.0);
    Ok(vec![
        Resultat::nouveau("Surface", Quantite::Aire(surface)),
        Resultat::nouveau(
            "Surface à couvrir, pertes comprises",
            Quantite::Aire(a_couvrir),
        ),
        Resultat::nouveau(
            "Boîtes",
            Quantite::Compte(
                arrondi_au_dessus(a_couvrir / PO2_PAR_PI2 / e.nombre(3)),
                "boîtes",
            ),
        ),
    ])
}
