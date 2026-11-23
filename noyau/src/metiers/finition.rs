//! Finition : peinture, céramique, revêtements de sol (section 22.7).

use super::{Calcul, Calculateur, Champ, Lues, Metier, Quantite, Resultat, arrondi_au_dessus};
use crate::langue::{Texte, t, tel};
use crate::unites::{Format, PO2_PAR_PI2};

/// Surface déduite pour une porte et pour une fenêtre, en pieds carrés.
const PI2_PAR_PORTE: f64 = 21.0;
const PI2_PAR_FENETRE: f64 = 15.0;
/// Contenance du contenant de peinture courant, en litres.
const LITRES_PAR_CONTENANT: f64 = 3.78;
const SURFACES: &[Texte] = &[
    t("Murs seulement", "Walls only"),
    t("Murs et plafond", "Walls and ceiling"),
];

const PERTES: Champ = Champ::nombre(t("Pertes", "Waste"), tel("%"))
    .defaut("10")
    .nul_permis();

pub const CALCULATEURS: &[Calculateur] = &[
    Calculateur {
        id: "peinture",
        metier: Metier::Finition,
        nom: t("Peinture d'une pièce", "Paint for a room"),
        resume: t(
            "La surface à peindre d'une pièce rectangulaire et la peinture qu'il faut.",
            "The area to paint in a rectangular room and the paint required.",
        ),
        champs: &[
            Champ::longueur(t("Longueur de la pièce", "Room length"), Format::PiPo)
                .court(t("Longueur", "Length")),
            Champ::longueur(t("Largeur de la pièce", "Room width"), Format::PiPo)
                .court(t("Largeur", "Width")),
            Champ::longueur(t("Hauteur des murs", "Wall height"), Format::PiPo)
                .court(t("Hauteur", "Height"))
                .defaut("8"),
            Champ::entier(
                t("Portes (21 pi² chacune)", "Doors (21 ft² each)"),
                Texte::VIDE,
            )
            .court(t("Portes", "Doors"))
            .defaut("1")
            .nul_permis(),
            Champ::entier(
                t("Fenêtres (15 pi² chacune)", "Windows (15 ft² each)"),
                Texte::VIDE,
            )
            .court(t("Fenêtres", "Windows"))
            .defaut("1")
            .nul_permis(),
            Champ::choix(t("Surfaces à peindre", "Surfaces to paint"), SURFACES)
                .court(t("Surfaces", "Surfaces")),
            Champ::entier(t("Couches", "Coats"), Texte::VIDE).defaut("2"),
            Champ::nombre(
                t(
                    "Rendement d'un contenant de 3,78 L",
                    "Coverage of a 3.78 L can",
                ),
                t("pi²", "ft²"),
            )
            .court(t("Rendement", "Coverage"))
            .defaut("350"),
        ],
        note: t(
            "Le rendement est inscrit sur le contenant : de 300 à 400 pi² selon la peinture et la surface.",
            "The coverage is printed on the can: 300 to 400 ft² depending on the paint and the surface.",
        ),
        calcul: peinture,
    },
    Calculateur {
        id: "ceramique",
        metier: Metier::Finition,
        nom: t("Céramique et tuiles", "Ceramic and tile"),
        resume: t(
            "Le nombre de tuiles pour couvrir une surface rectangulaire.",
            "The number of tiles to cover a rectangular area.",
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
            Champ::longueur(t("Longueur d'une tuile", "Tile length"), Format::PoFrac)
                .court(t("Longueur\ntuile", "Tile\nlength")),
            Champ::longueur(t("Largeur d'une tuile", "Tile width"), Format::PoFrac)
                .court(t("Largeur\ntuile", "Tile\nwidth")),
            PERTES,
            Champ::entier(t("Tuiles par boîte", "Tiles per box"), Texte::VIDE)
                .court(t("Par boîte", "Per box"))
                .facultatif(),
        ],
        note: t(
            "Les joints ne sont pas comptés. Prévoyez plus de pertes pour une pose en diagonale.",
            "Joints are not counted. Allow more waste for a diagonal layout.",
        ),
        calcul: ceramique,
    },
    Calculateur {
        id: "revetement-de-sol",
        metier: Metier::Finition,
        nom: t("Revêtement de sol en boîtes", "Boxed flooring"),
        resume: t(
            "Le nombre de boîtes de bois franc, de stratifié ou de vinyle pour un plancher.",
            "The number of boxes of hardwood, laminate or vinyl for a floor.",
        ),
        champs: &[
            Champ::longueur(t("Longueur du plancher", "Floor length"), Format::PiPo)
                .court(t("Longueur", "Length")),
            Champ::longueur(t("Largeur du plancher", "Floor width"), Format::PiPo)
                .court(t("Largeur", "Width")),
            PERTES,
            Champ::nombre(
                t("Couverture d'une boîte", "Coverage per box"),
                t("pi²", "ft²"),
            )
            .court(t("Par boîte", "Per box")),
        ],
        note: Texte::VIDE,
        calcul: revetement_de_sol,
    },
    Calculateur {
        id: "peinture-d-une-surface",
        metier: Metier::Finition,
        nom: t("Peinture d'une surface", "Paint for a surface"),
        resume: t(
            "La peinture pour un mur, un plancher ou une clôture.",
            "The paint for a wall, a floor or a fence.",
        ),
        champs: &[
            Champ::longueur(t("Longueur", "Length"), Format::PiPo),
            Champ::longueur(t("Hauteur ou largeur", "Height or width"), Format::PiPo)
                .court(t("Hauteur", "Height")),
            Champ::entier(t("Couches", "Coats"), Texte::VIDE).defaut("2"),
            Champ::nombre(
                t(
                    "Rendement d'un contenant de 3,78 L",
                    "Coverage of a 3.78 L can",
                ),
                t("pi²", "ft²"),
            )
            .defaut("350")
            .court(t("Rendement", "Coverage")),
        ],
        note: t(
            "Le rendement est inscrit sur le contenant : de 300 à 400 pi² selon la peinture et la surface.",
            "The coverage is printed on the can: 300 to 400 ft² depending on the paint and the surface.",
        ),
        calcul: peinture_d_une_surface,
    },
    Calculateur {
        id: "moulures",
        metier: Metier::Finition,
        nom: t("Plinthes et moulures", "Baseboards and mouldings"),
        resume: t(
            "La longueur de plinthes ou de moulures pour faire le tour d'une pièce.",
            "The length of baseboard or moulding to go around a room.",
        ),
        champs: &[
            Champ::longueur(t("Longueur de la pièce", "Room length"), Format::PiPo)
                .court(t("Longueur", "Length")),
            Champ::longueur(t("Largeur de la pièce", "Room width"), Format::PiPo)
                .court(t("Largeur", "Width")),
            Champ::entier(t("Portes", "Doors"), Texte::VIDE)
                .defaut("1")
                .nul_permis(),
            Champ::longueur(t("Largeur d'une porte", "Door width"), Format::PoFrac)
                .defaut("32")
                .court(t("Largeur\nporte", "Door\nwidth")),
            PERTES,
            Champ::longueur(t("Longueur d'une moulure", "Moulding length"), Format::PiPo)
                .defaut("8")
                .court(t("Longueur\nmoulure", "Moulding\nlength")),
        ],
        note: t(
            "Les portes sont déduites pour des plinthes; mettez 0 porte pour une moulure de plafond.",
            "Doors are deducted for baseboards; enter 0 doors for a ceiling moulding.",
        ),
        calcul: moulures,
    },
];

fn peinture(e: &Lues) -> Calcul {
    let (longueur, largeur, hauteur) = (e.longueur(0), e.longueur(1), e.longueur(2));
    let ouvertures = (e.nombre(3) * PI2_PAR_PORTE + e.nombre(4) * PI2_PAR_FENETRE) * PO2_PAR_PI2;
    let murs = 2.0 * (longueur + largeur) * hauteur - ouvertures;
    if murs <= 0.0 {
        return Err(t(
            "Les ouvertures couvrent toute la surface des murs.",
            "The openings cover the whole wall area.",
        ));
    }
    let plafond = if e.choix(5) == 1 {
        longueur * largeur
    } else {
        0.0
    };
    let surface = murs + plafond;
    let contenants = surface / PO2_PAR_PI2 * e.nombre(6) / e.nombre(7);
    Ok(vec![
        Resultat::nouveau(
            t("Surface à peindre", "Area to paint"),
            Quantite::Aire(surface),
        ),
        Resultat::nouveau(
            t("Peinture nécessaire", "Paint required"),
            Quantite::Mesure(contenants * LITRES_PAR_CONTENANT, tel("L")),
        ),
        Resultat::nouveau(
            t("Contenants de 3,78 L", "3.78 L cans"),
            Quantite::Compte(arrondi_au_dessus(contenants), t("contenants", "cans")),
        )
        .a_acheter(
            t("Contenants de peinture de 3,78 L", "3.78 L cans of paint"),
            &[],
        ),
    ])
}

fn ceramique(e: &Lues) -> Calcul {
    let surface = e.longueur(0) * e.longueur(1);
    let tuile = e.longueur(2) * e.longueur(3);
    let tuiles = arrondi_au_dessus(surface * (1.0 + e.nombre(4) / 100.0) / tuile);
    let mut resultats = vec![
        Resultat::nouveau(t("Surface", "Area"), Quantite::Aire(surface)),
        Resultat::nouveau(
            t("Tuiles nécessaires", "Tiles needed"),
            Quantite::Compte(tuiles, t("tuiles", "tiles")),
        ),
    ];
    if e.presente(5) {
        resultats.push(
            Resultat::nouveau(
                t("Boîtes", "Boxes"),
                Quantite::Compte(
                    arrondi_au_dessus(tuiles / e.nombre(5)),
                    t("boîtes", "boxes"),
                ),
            )
            .a_acheter(t("Boîtes de tuiles", "Boxes of tiles"), &[2, 3, 5]),
        );
    } else {
        // Sans le nombre de tuiles par boîte, on achète les tuiles (section 22.17).
        resultats[1] = resultats[1].a_acheter(t("Tuiles", "Tiles"), &[2, 3]);
    }
    Ok(resultats)
}

fn revetement_de_sol(e: &Lues) -> Calcul {
    let surface = e.longueur(0) * e.longueur(1);
    let a_couvrir = surface * (1.0 + e.nombre(2) / 100.0);
    Ok(vec![
        Resultat::nouveau(t("Surface", "Area"), Quantite::Aire(surface)),
        Resultat::nouveau(
            t(
                "Surface à couvrir, pertes comprises",
                "Area to cover, waste included",
            ),
            Quantite::Aire(a_couvrir),
        ),
        Resultat::nouveau(
            t("Boîtes", "Boxes"),
            Quantite::Compte(
                arrondi_au_dessus(a_couvrir / PO2_PAR_PI2 / e.nombre(3)),
                t("boîtes", "boxes"),
            ),
        )
        .a_acheter(t("Boîtes de revêtement de sol", "Boxes of flooring"), &[3]),
    ])
}

fn peinture_d_une_surface(e: &Lues) -> Calcul {
    let surface = e.longueur(0) * e.longueur(1);
    let contenants = surface / PO2_PAR_PI2 * e.nombre(2) / e.nombre(3);
    Ok(vec![
        Resultat::nouveau(
            t("Surface à peindre", "Area to paint"),
            Quantite::Aire(surface),
        ),
        Resultat::nouveau(
            t("Peinture nécessaire", "Paint required"),
            Quantite::Mesure(contenants * LITRES_PAR_CONTENANT, tel("L")),
        ),
        Resultat::nouveau(
            t("Contenants de 3,78 L", "3.78 L cans"),
            Quantite::Compte(arrondi_au_dessus(contenants), t("contenants", "cans")),
        )
        .a_acheter(
            t("Contenants de peinture de 3,78 L", "3.78 L cans of paint"),
            &[],
        ),
    ])
}

fn moulures(e: &Lues) -> Calcul {
    let tour = 2.0 * (e.longueur(0) + e.longueur(1));
    let a_poser = tour - e.nombre(2) * e.longueur(3);
    if a_poser <= 0.0 {
        return Err(t(
            "Les portes couvrent tout le tour de la pièce.",
            "The doors take up the whole perimeter of the room.",
        ));
    }
    let a_acheter = a_poser * (1.0 + e.nombre(4) / 100.0);
    Ok(vec![
        Resultat::nouveau(
            t("Tour de la pièce", "Room perimeter"),
            Quantite::Longueur(tour),
        ),
        Resultat::nouveau(
            t("Longueur à poser", "Length to install"),
            Quantite::Longueur(a_poser),
        ),
        Resultat::nouveau(
            t(
                "Longueur à acheter, pertes comprises",
                "Length to buy, waste included",
            ),
            Quantite::Longueur(a_acheter),
        ),
        Resultat::nouveau(
            t("Moulures", "Mouldings"),
            Quantite::Compte(
                arrondi_au_dessus(a_acheter / e.longueur(5)),
                t("moulures", "mouldings"),
            ),
        )
        .a_acheter(t("Moulures", "Mouldings"), &[5]),
    ])
}
