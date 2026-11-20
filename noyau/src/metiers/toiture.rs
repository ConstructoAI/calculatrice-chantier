//! Toiture et revêtement extérieur (section 22.11).

use super::{Calcul, Calculateur, Champ, Lues, Metier, Quantite, Resultat, arrondi_au_dessus};
use crate::langue::{Texte, t, tel};
use crate::unites::{Format, PO2_PAR_PI2};

const UNITES_DE_PENTE: &[Texte] = &[
    t("sur 12", "in 12"),
    t("degrés", "degrees"),
    t("pour cent", "percent"),
];
/// Panneaux de revêtement courants, et leur surface en pieds carrés.
const PANNEAUX: &[Texte] = &[
    t("4 pi × 8 pi", "4 ft × 8 ft"),
    t("4 pi × 9 pi", "4 ft × 9 ft"),
    t("4 pi × 10 pi", "4 ft × 10 ft"),
];
const PI2_PAR_PANNEAU: [f64; 3] = [32.0, 36.0, 40.0];
/// Surface d'un carré de couvreur, en pieds carrés.
const PI2_PAR_CARRE: f64 = 100.0;

const PERTES: Champ = Champ::nombre(t("Pertes", "Waste"), tel("%"))
    .defaut("10")
    .nul_permis();

pub const CALCULATEURS: &[Calculateur] = &[
    Calculateur {
        id: "bardeaux",
        metier: Metier::Toiture,
        nom: t("Bardeaux d'asphalte", "Asphalt shingles"),
        resume: t(
            "Les paquets de bardeaux d'un toit à deux versants, d'après ses mesures au sol et sa pente.",
            "The bundles of shingles for a gable roof, from its plan dimensions and its pitch.",
        ),
        champs: &[
            Champ::longueur(t("Longueur du toit", "Roof length"), Format::PiPo)
                .court(t("Longueur", "Length")),
            Champ::longueur(
                t(
                    "Largeur au sol, débords compris",
                    "Plan width, overhangs included",
                ),
                Format::PiPo,
            )
            .court(t("Largeur\nau sol", "Plan\nwidth")),
            Champ::nombre(t("Pente du toit", "Roof pitch"), t("sur 12", "in 12"))
                .nul_permis()
                .court(t("Pente", "Pitch")),
            PERTES,
            Champ::nombre(
                t("Couverture d'un paquet", "Coverage per bundle"),
                t("pi²", "ft²"),
            )
            .defaut("33,3")
            .court(t("Par paquet", "Per bundle")),
        ],
        note: t(
            "Toit à deux versants de même pente; les mesures comprennent les débords. Un carré vaut 100 pi² : le plus souvent trois paquets. Ajoutez les bardeaux de départ et de faîte.",
            "Gable roof with both sides at the same pitch; the dimensions include the overhangs. One square is 100 ft², most often three bundles. Add the starter and ridge shingles.",
        ),
        calcul: bardeaux,
    },
    Calculateur {
        id: "pente-de-toit",
        metier: Metier::Toiture,
        nom: t("Pente d'un toit", "Roof pitch"),
        resume: t(
            "La même pente en pouces sur 12, en degrés et en pourcentage, avec son facteur de pente.",
            "The same pitch in inches per 12, in degrees and as a percentage, with its pitch factor.",
        ),
        champs: &[
            Champ::nombre(t("Pente", "Pitch"), Texte::VIDE),
            Champ::choix(t("Unité", "Unit"), UNITES_DE_PENTE),
        ],
        note: t(
            "Le facteur de pente multiplie une longueur ou une surface mesurée au sol pour donner celle du toit.",
            "The pitch factor multiplies a length or an area measured in plan to give the one on the roof.",
        ),
        calcul: pente_de_toit,
    },
    Calculateur {
        id: "panneaux",
        metier: Metier::Toiture,
        nom: t("Panneaux de contreplaqué ou d'OSB", "Plywood or OSB sheets"),
        resume: t(
            "Le nombre de panneaux pour couvrir un toit, un plancher ou un mur.",
            "The number of sheets to cover a roof, a floor or a wall.",
        ),
        champs: &[
            Champ::longueur(t("Longueur", "Length"), Format::PiPo),
            Champ::longueur(t("Largeur ou hauteur", "Width or height"), Format::PiPo)
                .court(t("Largeur", "Width")),
            Champ::choix(t("Panneau", "Sheet"), PANNEAUX),
            PERTES,
        ],
        note: t(
            "Pour un toit, la largeur se mesure le long de la pente : de l'avant-toit au faîte.",
            "For a roof, the width is measured along the slope: from the eave to the ridge.",
        ),
        calcul: panneaux,
    },
    Calculateur {
        id: "revetement-exterieur",
        metier: Metier::Toiture,
        nom: t("Revêtement extérieur", "Siding"),
        resume: t(
            "Le revêtement des murs extérieurs, ouvertures déduites, en boîtes ou en paquets.",
            "The siding for exterior walls, openings deducted, in boxes or bundles.",
        ),
        champs: &[
            Champ::longueur(t("Longueur des murs", "Wall length"), Format::PiPo)
                .court(t("Longueur", "Length")),
            Champ::longueur(t("Hauteur des murs", "Wall height"), Format::PiPo)
                .court(t("Hauteur", "Height")),
            Champ::nombre(
                t("Ouvertures à déduire", "Openings to deduct"),
                t("pi²", "ft²"),
            )
            .defaut("0")
            .nul_permis()
            .court(t("Ouvertures", "Openings")),
            PERTES,
            Champ::nombre(
                t("Couverture d'une boîte", "Coverage per box"),
                t("pi²", "ft²"),
            )
            .court(t("Par boîte", "Per box")),
        ],
        note: t(
            "La longueur des murs est le tour du bâtiment. Les pignons se calculent à part.",
            "The wall length is the perimeter of the building. Gable ends are calculated separately.",
        ),
        calcul: revetement_exterieur,
    },
    Calculateur {
        id: "membrane",
        metier: Metier::Toiture,
        nom: t("Membrane et papier en rouleaux", "Membrane and paper rolls"),
        resume: t(
            "Le nombre de rouleaux de membrane, de papier ou de pare-air, chevauchements compris.",
            "The number of rolls of membrane, building paper or air barrier, overlaps included.",
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
            Champ::longueur(t("Largeur d'un rouleau", "Roll width"), Format::PoFrac)
                .defaut("36")
                .court(t("Largeur\nrouleau", "Roll\nwidth")),
            Champ::longueur(t("Longueur d'un rouleau", "Roll length"), Format::PiPo)
                .court(t("Longueur\nrouleau", "Roll\nlength")),
            Champ::longueur(t("Chevauchement", "Overlap"), Format::PoFrac)
                .nul_permis()
                .court(t("Chevauch.", "Overlap")),
            PERTES,
        ],
        note: t(
            "Chaque rang recouvre le précédent de la largeur du chevauchement.",
            "Each row laps over the previous one by the overlap width.",
        ),
        calcul: membrane,
    },
];

fn bardeaux(e: &Lues) -> Calcul {
    let pente = e.nombre(2) / 12.0;
    // Le versant est plus long que sa projection au sol.
    let surface = e.longueur(0) * e.longueur(1) * (1.0 + pente * pente).sqrt();
    let a_couvrir = surface * (1.0 + e.nombre(3) / 100.0) / PO2_PAR_PI2;
    Ok(vec![
        Resultat::nouveau(t("Surface du toit", "Roof area"), Quantite::Aire(surface)),
        Resultat::nouveau(
            t(
                "Carrés de 100 pi², pertes comprises",
                "Squares of 100 ft², waste included",
            ),
            Quantite::Fixe(a_couvrir / PI2_PAR_CARRE, 1, t("carrés", "squares")),
        ),
        Resultat::nouveau(
            t("Paquets de bardeaux", "Bundles of shingles"),
            Quantite::Compte(
                arrondi_au_dessus(a_couvrir / e.nombre(4)),
                t("paquets", "bundles"),
            ),
        ),
    ])
}

fn pente_de_toit(e: &Lues) -> Calcul {
    // Le rapport de la montée à la course.
    let rapport = match e.choix(1) {
        1 => {
            if e.nombre(0) >= 90.0 {
                return Err(t(
                    "L'angle doit être inférieur à 90°.",
                    "The angle must be less than 90°.",
                ));
            }
            e.nombre(0).to_radians().tan()
        }
        2 => e.nombre(0) / 100.0,
        _ => e.nombre(0) / 12.0,
    };
    Ok(vec![
        Resultat::nouveau(
            t("Pente, sur 12", "Pitch, in 12"),
            Quantite::Fixe(rapport * 12.0, 2, t("sur 12", "in 12")),
        ),
        Resultat::nouveau(
            tel("Angle"),
            Quantite::Fixe(rapport.atan().to_degrees(), 2, tel("°")),
        ),
        Resultat::nouveau(
            t("Pente, en pourcentage", "Pitch, as a percentage"),
            Quantite::Fixe(rapport * 100.0, 1, tel("%")),
        ),
        Resultat::nouveau(
            t("Facteur de pente", "Pitch factor"),
            Quantite::Fixe((1.0 + rapport * rapport).sqrt(), 4, Texte::VIDE),
        ),
    ])
}

fn panneaux(e: &Lues) -> Calcul {
    let surface = e.longueur(0) * e.longueur(1);
    let panneau = PI2_PAR_PANNEAU
        .get(e.choix(2))
        .copied()
        .unwrap_or(PI2_PAR_PANNEAU[0])
        * PO2_PAR_PI2;
    Ok(vec![
        Resultat::nouveau(t("Surface", "Area"), Quantite::Aire(surface)),
        Resultat::nouveau(
            t("Panneaux", "Sheets"),
            Quantite::Compte(
                arrondi_au_dessus(surface * (1.0 + e.nombre(3) / 100.0) / panneau),
                t("panneaux", "sheets"),
            ),
        ),
    ])
}

fn revetement_exterieur(e: &Lues) -> Calcul {
    let surface = e.longueur(0) * e.longueur(1) - e.nombre(2) * PO2_PAR_PI2;
    if surface <= 0.0 {
        return Err(t(
            "Les ouvertures couvrent toute la surface des murs.",
            "The openings cover the whole wall area.",
        ));
    }
    let a_acheter = surface * (1.0 + e.nombre(3) / 100.0);
    Ok(vec![
        Resultat::nouveau(
            t("Surface à couvrir", "Area to cover"),
            Quantite::Aire(surface),
        ),
        Resultat::nouveau(
            t(
                "Surface à acheter, pertes comprises",
                "Area to buy, waste included",
            ),
            Quantite::Aire(a_acheter),
        ),
        Resultat::nouveau(
            t("Boîtes", "Boxes"),
            Quantite::Compte(
                arrondi_au_dessus(a_acheter / PO2_PAR_PI2 / e.nombre(4)),
                t("boîtes", "boxes"),
            ),
        ),
    ])
}

fn membrane(e: &Lues) -> Calcul {
    let (largeur_du_rouleau, chevauchement) = (e.longueur(2), e.longueur(4));
    if chevauchement >= largeur_du_rouleau {
        return Err(t(
            "Le chevauchement dépasse la largeur du rouleau.",
            "The overlap is wider than the roll.",
        ));
    }
    let surface = e.longueur(0) * e.longueur(1);
    // Ce qu'un rouleau couvre vraiment, une fois recouvert par le rang suivant.
    let utile = (largeur_du_rouleau - chevauchement) * e.longueur(3);
    Ok(vec![
        Resultat::nouveau(t("Surface", "Area"), Quantite::Aire(surface)),
        Resultat::nouveau(
            t("Rouleaux", "Rolls"),
            Quantite::Compte(
                arrondi_au_dessus(surface * (1.0 + e.nombre(5) / 100.0) / utile),
                t("rouleaux", "rolls"),
            ),
        ),
    ])
}
