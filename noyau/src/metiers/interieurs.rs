//! Gypse et isolation : panneaux, cloisons, isolant, plafonds (section 22.12).

use super::{Calcul, Calculateur, Champ, Lues, Metier, Quantite, Resultat, arrondi_au_dessus};
use crate::langue::{Texte, t, tel};
use crate::unites::{Format, PO2_PAR_PI2};

/// Surface déduite pour une porte et pour une fenêtre, en pieds carrés.
const PI2_PAR_PORTE: f64 = 21.0;
const PI2_PAR_FENETRE: f64 = 15.0;
const SURFACES: &[Texte] = &[
    t("Murs seulement", "Walls only"),
    t("Murs et plafond", "Walls and ceiling"),
];
/// Panneaux de gypse courants, et leur surface en pieds carrés.
const PANNEAUX: &[Texte] = &[
    t("4 pi × 8 pi", "4 ft × 8 ft"),
    t("4 pi × 10 pi", "4 ft × 10 ft"),
    t("4 pi × 12 pi", "4 ft × 12 ft"),
];
const PI2_PAR_PANNEAU: [f64; 3] = [32.0, 40.0, 48.0];
const LISSES: &[Texte] = &[
    t("Une en bas, une en haut", "One bottom, one top"),
    t("Une en bas, deux en haut", "One bottom, two top"),
];
/// Tuiles de plafond suspendu, et leur surface en pieds carrés.
const TUILES: &[Texte] = &[
    t("2 pi × 4 pi", "2 ft × 4 ft"),
    t("2 pi × 2 pi", "2 ft × 2 ft"),
];
const PI2_PAR_TUILE: [f64; 2] = [8.0, 4.0];
const UNITES_D_ISOLATION: &[Texte] = &[
    t("R (impérial)", "R (imperial)"),
    t("RSI (métrique)", "RSI (metric)"),
];
/// Valeur R d'une résistance thermique de 1 RSI (1 m²·K/W en h·pi²·°F/BTU).
pub(crate) const R_PAR_RSI: f64 = 5.678_263;

const PERTES: Champ = Champ::nombre(t("Pertes", "Waste"), tel("%"))
    .defaut("10")
    .nul_permis();

pub const CALCULATEURS: &[Calculateur] = &[
    Calculateur {
        id: "gypse",
        metier: Metier::Interieurs,
        nom: t("Panneaux de gypse d'une pièce", "Drywall for a room"),
        resume: t(
            "Le nombre de panneaux de gypse pour les murs d'une pièce, avec ou sans le plafond.",
            "The number of drywall sheets for the walls of a room, with or without the ceiling.",
        ),
        champs: &[
            Champ::longueur(t("Longueur de la pièce", "Room length"), Format::PiPo)
                .court(t("Longueur", "Length")),
            Champ::longueur(t("Largeur de la pièce", "Room width"), Format::PiPo)
                .court(t("Largeur", "Width")),
            Champ::longueur(t("Hauteur des murs", "Wall height"), Format::PiPo)
                .defaut("8")
                .court(t("Hauteur", "Height")),
            Champ::entier(
                t("Portes (21 pi² chacune)", "Doors (21 ft² each)"),
                Texte::VIDE,
            )
            .defaut("1")
            .nul_permis()
            .court(t("Portes", "Doors")),
            Champ::entier(
                t("Fenêtres (15 pi² chacune)", "Windows (15 ft² each)"),
                Texte::VIDE,
            )
            .defaut("1")
            .nul_permis()
            .court(t("Fenêtres", "Windows")),
            Champ::choix(t("Surfaces à couvrir", "Surfaces to cover"), SURFACES)
                .defaut("1")
                .court(t("Surfaces", "Surfaces")),
            Champ::choix(t("Panneau", "Sheet"), PANNEAUX),
            PERTES,
        ],
        note: t(
            "Les ouvertures sont déduites. Prévoyez plus de pertes pour une pièce très découpée.",
            "Openings are deducted. Allow more waste for a room with many jogs and corners.",
        ),
        calcul: gypse,
    },
    Calculateur {
        id: "montants-de-cloison",
        metier: Metier::Interieurs,
        nom: t("Montants d'une cloison", "Partition studs"),
        resume: t(
            "Le nombre de montants d'une cloison droite et la longueur de ses lisses.",
            "The number of studs in a straight partition and the length of its plates.",
        ),
        champs: &[
            Champ::longueur(
                t("Longueur de la cloison", "Partition length"),
                Format::PiPo,
            )
            .court(t("Longueur", "Length")),
            Champ::longueur(t("Espacement des montants", "Stud spacing"), Format::PoFrac)
                .defaut("16")
                .court(t("Espacement", "Spacing")),
            Champ::entier(t("Montants en plus", "Extra studs"), Texte::VIDE)
                .defaut("0")
                .nul_permis()
                .court(t("En plus", "Extra")),
            Champ::choix(t("Lisses", "Plates"), LISSES),
        ],
        note: t(
            "Un montant à chaque bout, puis un à chaque espacement. Ajoutez les montants des coins, des intersections et des ouvertures.",
            "One stud at each end, then one at every spacing. Add the studs for corners, intersections and openings.",
        ),
        calcul: montants_de_cloison,
    },
    Calculateur {
        id: "isolant",
        metier: Metier::Interieurs,
        nom: t(
            "Isolant en nattes ou en panneaux",
            "Batt or board insulation",
        ),
        resume: t(
            "Le nombre de paquets d'isolant pour couvrir une surface.",
            "The number of packages of insulation to cover an area.",
        ),
        champs: &[
            Champ::longueur(t("Longueur", "Length"), Format::PiPo),
            Champ::longueur(t("Hauteur ou largeur", "Height or width"), Format::PiPo)
                .court(t("Hauteur", "Height")),
            Champ::nombre(t("Pertes", "Waste"), tel("%"))
                .defaut("5")
                .nul_permis(),
            Champ::nombre(
                t("Couverture d'un paquet", "Coverage per package"),
                t("pi²", "ft²"),
            )
            .court(t("Par paquet", "Per package")),
        ],
        note: t(
            "La couverture est inscrite sur l'emballage : elle dépend de l'épaisseur de l'isolant et de l'espacement des montants.",
            "The coverage is printed on the package: it depends on the insulation thickness and the stud spacing.",
        ),
        calcul: isolant,
    },
    Calculateur {
        id: "valeur-r",
        metier: Metier::Interieurs,
        nom: t("Valeur R et valeur RSI", "R-value and RSI value"),
        resume: t(
            "La même résistance thermique en valeur R et en valeur RSI, et le coefficient U.",
            "The same thermal resistance as an R-value and an RSI value, and the U-factor.",
        ),
        champs: &[
            Champ::nombre(t("Résistance thermique", "Thermal resistance"), Texte::VIDE)
                .court(t("Résistance", "Resistance")),
            Champ::choix(t("Unité", "Unit"), UNITES_D_ISOLATION),
        ],
        note: t(
            "La valeur R vaut 5,678 fois la valeur RSI. Le coefficient U est l'inverse de la valeur RSI.",
            "The R-value is 5.678 times the RSI value. The U-factor is the inverse of the RSI value.",
        ),
        calcul: valeur_r,
    },
    Calculateur {
        id: "plafond-suspendu",
        metier: Metier::Interieurs,
        nom: t("Plafond suspendu", "Suspended ceiling"),
        resume: t(
            "Les tuiles d'un plafond suspendu et la cornière qui fait le tour de la pièce.",
            "The tiles for a suspended ceiling and the wall angle that runs around the room.",
        ),
        champs: &[
            Champ::longueur(t("Longueur de la pièce", "Room length"), Format::PiPo)
                .court(t("Longueur", "Length")),
            Champ::longueur(t("Largeur de la pièce", "Room width"), Format::PiPo)
                .court(t("Largeur", "Width")),
            Champ::choix(t("Tuile", "Tile"), TUILES),
            PERTES,
        ],
        note: t(
            "Les tés porteurs et les entretoises dépendent du plan de pose : comptez-les d'après le calepinage.",
            "Main tees and cross tees depend on the layout: count them from the ceiling plan.",
        ),
        calcul: plafond_suspendu,
    },
];

fn gypse(e: &Lues) -> Calcul {
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
    let panneau = PI2_PAR_PANNEAU
        .get(e.choix(6))
        .copied()
        .unwrap_or(PI2_PAR_PANNEAU[0])
        * PO2_PAR_PI2;
    let panneaux = arrondi_au_dessus(surface * (1.0 + e.nombre(7) / 100.0) / panneau);
    Ok(vec![
        Resultat::nouveau(
            t("Surface à couvrir", "Area to cover"),
            Quantite::Aire(surface),
        ),
        Resultat::nouveau(
            t("Panneaux", "Sheets"),
            Quantite::Compte(panneaux, t("panneaux", "sheets")),
        )
        .a_acheter(t("Panneaux de gypse", "Gypsum boards"), &[6]),
    ])
}

fn montants_de_cloison(e: &Lues) -> Calcul {
    let longueur = e.longueur(0);
    // Un montant à chaque bout, puis un à chaque espacement.
    let montants = arrondi_au_dessus(longueur / e.longueur(1)) + 1.0 + e.nombre(2);
    let lisses = if e.choix(3) == 1 { 3.0 } else { 2.0 };
    Ok(vec![
        Resultat::nouveau(
            t("Montants", "Studs"),
            Quantite::Compte(montants, t("montants", "studs")),
        )
        .a_acheter(t("Montants", "Studs"), &[]),
        Resultat::nouveau(
            t("Longueur de lisses", "Total plate length"),
            Quantite::Longueur(longueur * lisses),
        ),
    ])
}

fn isolant(e: &Lues) -> Calcul {
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
            t("Paquets", "Packages"),
            Quantite::Compte(
                arrondi_au_dessus(a_couvrir / PO2_PAR_PI2 / e.nombre(3)),
                t("paquets", "packages"),
            ),
        )
        .a_acheter(t("Paquets d'isolant", "Packages of insulation"), &[3]),
    ])
}

fn valeur_r(e: &Lues) -> Calcul {
    let rsi = if e.choix(1) == 1 {
        e.nombre(0)
    } else {
        e.nombre(0) / R_PAR_RSI
    };
    Ok(vec![
        Resultat::nouveau(
            t("Valeur R", "R-value"),
            Quantite::Fixe(rsi * R_PAR_RSI, 1, Texte::VIDE),
        ),
        Resultat::nouveau(
            t("Valeur RSI", "RSI value"),
            Quantite::Fixe(rsi, 2, Texte::VIDE),
        ),
        Resultat::nouveau(
            t("Coefficient U", "U-factor"),
            Quantite::Fixe(1.0 / rsi, 3, tel("W/m²·K")),
        ),
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
        Resultat::nouveau(t("Surface", "Area"), Quantite::Aire(surface)),
        Resultat::nouveau(
            t("Tuiles", "Tiles"),
            Quantite::Compte(tuiles, t("tuiles", "tiles")),
        )
        .a_acheter(t("Tuiles de plafond", "Ceiling tiles"), &[2]),
        Resultat::nouveau(
            t("Cornière murale", "Wall angle"),
            Quantite::Longueur(2.0 * (longueur + largeur)),
        ),
    ])
}
