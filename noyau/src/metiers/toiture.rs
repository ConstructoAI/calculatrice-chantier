//! Toiture et revêtement extérieur (section 22.11).

use super::{Calcul, Calculateur, Champ, Lues, Metier, Quantite, Resultat, arrondi_au_dessus};
use crate::unites::{Format, PO2_PAR_PI2};

const UNITES_DE_PENTE: &[&str] = &["sur 12", "degrés", "pour cent"];
/// Panneaux de revêtement courants, et leur surface en pieds carrés.
const PANNEAUX: &[&str] = &["4 pi × 8 pi", "4 pi × 9 pi", "4 pi × 10 pi"];
const PI2_PAR_PANNEAU: [f64; 3] = [32.0, 36.0, 40.0];
/// Surface d'un carré de couvreur, en pieds carrés.
const PI2_PAR_CARRE: f64 = 100.0;

const PERTES: Champ = Champ::nombre("Pertes", "%").defaut("10").nul_permis();

pub const CALCULATEURS: &[Calculateur] = &[
    Calculateur {
        id: "bardeaux",
        metier: Metier::Toiture,
        nom: "Bardeaux d'asphalte",
        resume: "Les paquets de bardeaux d'un toit à deux versants, d'après ses mesures au sol et sa pente.",
        champs: &[
            Champ::longueur("Longueur du toit", Format::PiPo).court("Longueur"),
            Champ::longueur("Largeur au sol, débords compris", Format::PiPo)
                .court("Largeur\nau sol"),
            Champ::nombre("Pente du toit", "sur 12")
                .nul_permis()
                .court("Pente"),
            PERTES,
            Champ::nombre("Couverture d'un paquet", "pi²")
                .defaut("33,3")
                .court("Par paquet"),
        ],
        note: "Toit à deux versants de même pente; les mesures comprennent les débords. Un carré vaut 100 pi² : le plus souvent trois paquets. Ajoutez les bardeaux de départ et de faîte.",
        calcul: bardeaux,
    },
    Calculateur {
        id: "pente-de-toit",
        metier: Metier::Toiture,
        nom: "Pente d'un toit",
        resume: "La même pente en pouces sur 12, en degrés et en pourcentage, avec son facteur de pente.",
        champs: &[
            Champ::nombre("Pente", ""),
            Champ::choix("Unité", UNITES_DE_PENTE),
        ],
        note: "Le facteur de pente multiplie une longueur ou une surface mesurée au sol pour donner celle du toit.",
        calcul: pente_de_toit,
    },
    Calculateur {
        id: "panneaux",
        metier: Metier::Toiture,
        nom: "Panneaux de contreplaqué ou d'OSB",
        resume: "Le nombre de panneaux pour couvrir un toit, un plancher ou un mur.",
        champs: &[
            Champ::longueur("Longueur", Format::PiPo),
            Champ::longueur("Largeur ou hauteur", Format::PiPo).court("Largeur"),
            Champ::choix("Panneau", PANNEAUX),
            PERTES,
        ],
        note: "Pour un toit, la largeur se mesure le long de la pente : de l'avant-toit au faîte.",
        calcul: panneaux,
    },
    Calculateur {
        id: "revetement-exterieur",
        metier: Metier::Toiture,
        nom: "Revêtement extérieur",
        resume: "Le revêtement des murs extérieurs, ouvertures déduites, en boîtes ou en paquets.",
        champs: &[
            Champ::longueur("Longueur des murs", Format::PiPo).court("Longueur"),
            Champ::longueur("Hauteur des murs", Format::PiPo).court("Hauteur"),
            Champ::nombre("Ouvertures à déduire", "pi²")
                .defaut("0")
                .nul_permis()
                .court("Ouvertures"),
            PERTES,
            Champ::nombre("Couverture d'une boîte", "pi²").court("Par boîte"),
        ],
        note: "La longueur des murs est le tour du bâtiment. Les pignons se calculent à part.",
        calcul: revetement_exterieur,
    },
    Calculateur {
        id: "membrane",
        metier: Metier::Toiture,
        nom: "Membrane et papier en rouleaux",
        resume: "Le nombre de rouleaux de membrane, de papier ou de pare-air, chevauchements compris.",
        champs: &[
            Champ::longueur("Longueur de la surface", Format::PiPo).court("Longueur"),
            Champ::longueur("Largeur de la surface", Format::PiPo).court("Largeur"),
            Champ::longueur("Largeur d'un rouleau", Format::PoFrac)
                .defaut("36")
                .court("Largeur\nrouleau"),
            Champ::longueur("Longueur d'un rouleau", Format::PiPo).court("Longueur\nrouleau"),
            Champ::longueur("Chevauchement", Format::PoFrac)
                .nul_permis()
                .court("Chevauch."),
            PERTES,
        ],
        note: "Chaque rang recouvre le précédent de la largeur du chevauchement.",
        calcul: membrane,
    },
];

fn bardeaux(e: &Lues) -> Calcul {
    let pente = e.nombre(2) / 12.0;
    // Le versant est plus long que sa projection au sol.
    let surface = e.longueur(0) * e.longueur(1) * (1.0 + pente * pente).sqrt();
    let a_couvrir = surface * (1.0 + e.nombre(3) / 100.0) / PO2_PAR_PI2;
    Ok(vec![
        Resultat::nouveau("Surface du toit", Quantite::Aire(surface)),
        Resultat::nouveau(
            "Carrés de 100 pi², pertes comprises",
            Quantite::Fixe(a_couvrir / PI2_PAR_CARRE, 1, "carrés"),
        ),
        Resultat::nouveau(
            "Paquets de bardeaux",
            Quantite::Compte(arrondi_au_dessus(a_couvrir / e.nombre(4)), "paquets"),
        ),
    ])
}

fn pente_de_toit(e: &Lues) -> Calcul {
    // Le rapport de la montée à la course.
    let rapport = match e.choix(1) {
        1 => {
            if e.nombre(0) >= 90.0 {
                return Err("L'angle doit être inférieur à 90°.");
            }
            e.nombre(0).to_radians().tan()
        }
        2 => e.nombre(0) / 100.0,
        _ => e.nombre(0) / 12.0,
    };
    Ok(vec![
        Resultat::nouveau("Pente, sur 12", Quantite::Fixe(rapport * 12.0, 2, "sur 12")),
        Resultat::nouveau("Angle", Quantite::Fixe(rapport.atan().to_degrees(), 2, "°")),
        Resultat::nouveau(
            "Pente, en pourcentage",
            Quantite::Fixe(rapport * 100.0, 1, "%"),
        ),
        Resultat::nouveau(
            "Facteur de pente",
            Quantite::Fixe((1.0 + rapport * rapport).sqrt(), 4, ""),
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
        Resultat::nouveau("Surface", Quantite::Aire(surface)),
        Resultat::nouveau(
            "Panneaux",
            Quantite::Compte(
                arrondi_au_dessus(surface * (1.0 + e.nombre(3) / 100.0) / panneau),
                "panneaux",
            ),
        ),
    ])
}

fn revetement_exterieur(e: &Lues) -> Calcul {
    let surface = e.longueur(0) * e.longueur(1) - e.nombre(2) * PO2_PAR_PI2;
    if surface <= 0.0 {
        return Err("Les ouvertures couvrent toute la surface des murs.");
    }
    let a_acheter = surface * (1.0 + e.nombre(3) / 100.0);
    Ok(vec![
        Resultat::nouveau("Surface à couvrir", Quantite::Aire(surface)),
        Resultat::nouveau(
            "Surface à acheter, pertes comprises",
            Quantite::Aire(a_acheter),
        ),
        Resultat::nouveau(
            "Boîtes",
            Quantite::Compte(
                arrondi_au_dessus(a_acheter / PO2_PAR_PI2 / e.nombre(4)),
                "boîtes",
            ),
        ),
    ])
}

fn membrane(e: &Lues) -> Calcul {
    let (largeur_du_rouleau, chevauchement) = (e.longueur(2), e.longueur(4));
    if chevauchement >= largeur_du_rouleau {
        return Err("Le chevauchement dépasse la largeur du rouleau.");
    }
    let surface = e.longueur(0) * e.longueur(1);
    // Ce qu'un rouleau couvre vraiment, une fois recouvert par le rang suivant.
    let utile = (largeur_du_rouleau - chevauchement) * e.longueur(3);
    Ok(vec![
        Resultat::nouveau("Surface", Quantite::Aire(surface)),
        Resultat::nouveau(
            "Rouleaux",
            Quantite::Compte(
                arrondi_au_dessus(surface * (1.0 + e.nombre(5) / 100.0) / utile),
                "rouleaux",
            ),
        ),
    ])
}
