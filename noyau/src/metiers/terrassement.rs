//! Terrassement et aménagement (section 22.9).

use std::f64::consts::PI;

use super::{
    Calcul, Calculateur, Champ, Lues, Metier, PO3_PAR_L, Quantite, Resultat, arrondi_au_dessus,
};
use crate::langue::{Texte, t, tel};
use crate::unites::{Format, LB_PAR_TC, PO2_PAR_PI2, PO3_PAR_VG3};

const FORMES_DE_POTEAU: &[Texte] = &[t("Carré", "Square"), t("Rond", "Round")];

pub const CALCULATEURS: &[Calculateur] = &[
    Calculateur {
        id: "excavation",
        metier: Metier::Terrassement,
        nom: t("Excavation et remblai", "Excavation and backfill"),
        resume: t(
            "Le volume d'une excavation rectangulaire, en place et une fois la terre remuée.",
            "The volume of a rectangular excavation, in place and once the soil is loosened.",
        ),
        champs: &[
            Champ::longueur(t("Longueur", "Length"), Format::PiPo),
            Champ::longueur(t("Largeur", "Width"), Format::PiPo),
            Champ::longueur(t("Profondeur", "Depth"), Format::PiPo),
            Champ::nombre(t("Foisonnement", "Swell"), tel("%"))
                .defaut("25")
                .nul_permis(),
        ],
        note: t(
            "La terre remuée prend plus de place : environ 10 à 15 % de plus pour le sable, 25 % pour la terre, 30 % et plus pour l'argile.",
            "Loosened soil takes up more room: about 10 to 15% more for sand, 25% for common earth, 30% or more for clay.",
        ),
        calcul: excavation,
    },
    Calculateur {
        id: "granulats",
        metier: Metier::Terrassement,
        nom: t("Pierre, gravier et sable", "Stone, gravel and sand"),
        resume: t(
            "Le volume et la masse d'une couche de granulats.",
            "The volume and weight of a layer of aggregate.",
        ),
        champs: &[
            Champ::longueur(t("Longueur", "Length"), Format::PiPo),
            Champ::longueur(t("Largeur", "Width"), Format::PiPo),
            Champ::longueur(t("Épaisseur", "Thickness"), Format::PoFrac),
            Champ::nombre(t("Masse volumique", "Density"), t("tc/vg³", "ton/yd³"))
                .court(t("Masse\nvolumique", "Density"))
                .defaut("1,5"),
        ],
        note: t(
            "Comptez environ 1,5 tonne courte par verge cube de pierre concassée; le fournisseur donne la valeur exacte de son matériau.",
            "Allow about 1.5 short tons per cubic yard of crushed stone; your supplier has the exact figure for their material.",
        ),
        calcul: granulats,
    },
    Calculateur {
        id: "cloture",
        metier: Metier::Terrassement,
        nom: t("Clôture", "Fence"),
        resume: t(
            "Le nombre de poteaux et de sections d'une clôture droite, également espacés.",
            "The number of posts and sections of a straight fence, evenly spaced.",
        ),
        champs: &[
            Champ::longueur(t("Longueur totale", "Total length"), Format::PiPo)
                .court(t("Longueur", "Length")),
            Champ::longueur(
                t("Espacement visé des poteaux", "Target post spacing"),
                Format::PiPo,
            )
            .court(t("Espacement", "Spacing"))
            .defaut("8"),
        ],
        note: Texte::VIDE,
        calcul: cloture,
    },
    Calculateur {
        id: "tranchee",
        metier: Metier::Terrassement,
        nom: t("Tranchée", "Trench"),
        resume: t(
            "Le volume d'une tranchée, à parois droites ou inclinées.",
            "The volume of a trench with vertical or sloped sides.",
        ),
        champs: &[
            Champ::longueur(t("Longueur", "Length"), Format::PiPo),
            Champ::longueur(t("Largeur au fond", "Bottom width"), Format::PiPo)
                .court(t("Largeur\nau fond", "Bottom\nwidth")),
            Champ::longueur(t("Largeur en haut", "Top width"), Format::PiPo)
                .facultatif()
                .court(t("Largeur\nen haut", "Top\nwidth")),
            Champ::longueur(t("Profondeur", "Depth"), Format::PiPo),
            Champ::nombre(t("Foisonnement", "Swell"), tel("%"))
                .defaut("25")
                .nul_permis(),
        ],
        note: t(
            "Laissez la largeur en haut vide pour des parois verticales. L'inclinaison des parois est une question de sécurité : suivez le Code de sécurité pour les travaux de construction.",
            "Leave the top width blank for vertical sides. The slope of the sides is a safety matter: follow the Safety Code for the construction industry.",
        ),
        calcul: tranchee,
    },
    Calculateur {
        id: "voyages-de-camion",
        metier: Metier::Terrassement,
        nom: t("Voyages de camion", "Truckloads"),
        resume: t(
            "Le nombre de voyages pour transporter un volume de terre ou de pierre.",
            "The number of truckloads needed to haul a volume of soil or stone.",
        ),
        champs: &[
            Champ::nombre(t("Volume à transporter", "Volume to haul"), t("vg³", "yd³"))
                .court(tel("Volume")),
            Champ::nombre(t("Capacité du camion", "Truck capacity"), t("vg³", "yd³"))
                .court(t("Camion", "Truck")),
        ],
        note: t(
            "Comptez le volume foisonné : la terre remuée prend plus de place. La capacité dépend du camion, du matériau et des limites de charge.",
            "Use the loose volume: loosened soil takes up more room. Capacity depends on the truck, the material and the load limits.",
        ),
        calcul: voyages_de_camion,
    },
    Calculateur {
        id: "pente-de-terrain",
        metier: Metier::Terrassement,
        nom: t("Pente et dénivelé", "Slope and drop"),
        resume: t(
            "Remplissez deux des trois grandeurs : la troisième est calculée.",
            "Fill in two of the three values: the third is calculated.",
        ),
        champs: &[
            Champ::longueur(
                t("Distance horizontale", "Horizontal distance"),
                Format::PiPo,
            )
            .facultatif()
            .court(tel("Distance")),
            Champ::longueur(t("Dénivelé", "Drop"), Format::PoFrac).facultatif(),
            Champ::nombre(t("Pente", "Slope"), tel("%")).facultatif(),
        ],
        note: t(
            "Une pente de 2 % équivaut à environ 1/4 po au pied.",
            "A 2% slope is about 1/4 in per foot.",
        ),
        calcul: pente_de_terrain,
    },
    Calculateur {
        id: "terrasse",
        metier: Metier::Terrassement,
        nom: t("Terrasse : planches et vis", "Deck: boards and screws"),
        resume: t(
            "Les planches de surface d'une terrasse rectangulaire et les vis pour les fixer.",
            "The decking boards for a rectangular deck and the screws to fasten them.",
        ),
        champs: &[
            Champ::longueur(
                t(
                    "Longueur, dans le sens des planches",
                    "Length, along the boards",
                ),
                Format::PiPo,
            )
            .court(t("Longueur", "Length")),
            Champ::longueur(t("Largeur de la terrasse", "Deck width"), Format::PiPo)
                .court(t("Largeur", "Width")),
            Champ::longueur(t("Largeur d'une planche", "Board width"), Format::PoFrac)
                .defaut("5 1/2")
                .court(t("Planche", "Board")),
            Champ::longueur(
                t("Jeu entre les planches", "Gap between boards"),
                Format::PoFrac,
            )
            .defaut("1/8")
            .nul_permis()
            .court(t("Jeu", "Gap")),
            Champ::longueur(t("Espacement des solives", "Joist spacing"), Format::PoFrac)
                .defaut("16")
                .court(t("Solives", "Joists")),
            Champ::nombre(t("Pertes", "Waste"), tel("%"))
                .defaut("10")
                .nul_permis(),
        ],
        note: t(
            "Deux vis à chaque croisement d'une planche et d'une solive. La structure (solives, poutres, poteaux) se calcule à part.",
            "Two screws wherever a board crosses a joist. The framing (joists, beams, posts) is calculated separately.",
        ),
        calcul: terrasse,
    },
    Calculateur {
        id: "tourbe",
        metier: Metier::Terrassement,
        nom: t("Tourbe en rouleaux", "Sod rolls"),
        resume: t(
            "Le nombre de rouleaux de tourbe pour gazonner une surface.",
            "The number of sod rolls to cover an area.",
        ),
        champs: &[
            Champ::longueur(t("Longueur", "Length"), Format::PiPo),
            Champ::longueur(t("Largeur", "Width"), Format::PiPo),
            Champ::nombre(
                t("Surface d'un rouleau", "Area of one roll"),
                t("pi²", "ft²"),
            )
            .defaut("10")
            .court(t("Rouleau", "Roll")),
            Champ::nombre(t("Pertes", "Waste"), tel("%"))
                .defaut("5")
                .nul_permis(),
        ],
        note: t(
            "Un rouleau mesure souvent 2 pi sur 5 pi, soit 10 pi² : vérifiez auprès du fournisseur.",
            "A roll is often 2 ft by 5 ft, or 10 ft²: check with your supplier.",
        ),
        calcul: tourbe,
    },
    Calculateur {
        id: "poteaux",
        metier: Metier::Terrassement,
        nom: t("Béton des poteaux", "Concrete for posts"),
        resume: t(
            "Le béton pour sceller des poteaux de clôture ou de terrasse dans leurs trous.",
            "The concrete to set fence or deck posts in their holes.",
        ),
        champs: &[
            Champ::longueur(t("Diamètre du trou", "Hole diameter"), Format::PoFrac)
                .court(t("Trou", "Hole")),
            Champ::longueur(t("Profondeur du trou", "Hole depth"), Format::PiPo)
                .court(t("Profondeur", "Depth")),
            Champ::choix(t("Poteau", "Post"), FORMES_DE_POTEAU),
            Champ::longueur(
                t("Côté ou diamètre du poteau", "Post side or diameter"),
                Format::PoFrac,
            )
            .nul_permis()
            .court(t("Côté", "Side")),
            Champ::entier(t("Nombre de poteaux", "Number of posts"), Texte::VIDE)
                .defaut("1")
                .court(t("Nombre", "Count")),
            Champ::nombre(t("Rendement d'un sac", "Yield per bag"), tel("L"))
                .defaut("14")
                .court(t("Sac", "Bag")),
        ],
        note: t(
            "Le volume du poteau est déduit de celui du trou; mettez 0 pour un trou rempli de béton. Rendement : environ 14 L pour un sac de 30 kg de béton prémélangé.",
            "The volume of the post is subtracted from that of the hole; enter 0 for a hole filled with concrete. Yield: about 14 L for a 30 kg bag of premixed concrete.",
        ),
        calcul: poteaux,
    },
];

fn excavation(e: &Lues) -> Calcul {
    let volume = e.longueur(0) * e.longueur(1) * e.longueur(2);
    Ok(vec![
        Resultat::nouveau(
            t("Volume en place", "Volume in place"),
            Quantite::Vrac(volume),
        ),
        Resultat::nouveau(
            t("Volume à transporter, foisonné", "Volume to haul, loose"),
            Quantite::Vrac(volume * (1.0 + e.nombre(3) / 100.0)),
        ),
    ])
}

fn granulats(e: &Lues) -> Calcul {
    let volume = e.longueur(0) * e.longueur(1) * e.longueur(2);
    let tonnes_courtes = volume / PO3_PAR_VG3 * e.nombre(3);
    Ok(vec![
        Resultat::nouveau(tel("Volume"), Quantite::Vrac(volume)),
        Resultat::nouveau(
            t("Masse", "Weight"),
            Quantite::Tonnes(tonnes_courtes * LB_PAR_TC),
        ),
    ])
}

fn cloture(e: &Lues) -> Calcul {
    let longueur = e.longueur(0);
    let sections = arrondi_au_dessus(longueur / e.longueur(1)).max(1.0);
    Ok(vec![
        Resultat::nouveau(tel("Sections"), Quantite::Compte(sections, tel("sections"))),
        Resultat::nouveau(
            t("Poteaux", "Posts"),
            Quantite::Compte(sections + 1.0, t("poteaux", "posts")),
        )
        .a_acheter(t("Poteaux de clôture", "Fence posts"), &[]),
        Resultat::nouveau(
            t("Espacement réel", "Actual spacing"),
            Quantite::Longueur(longueur / sections),
        ),
    ])
}

fn tranchee(e: &Lues) -> Calcul {
    let fond = e.longueur(1);
    let haut = if e.presente(2) { e.longueur(2) } else { fond };
    if haut < fond {
        return Err(t(
            "La largeur en haut ne peut pas être plus petite que la largeur au fond.",
            "The top width cannot be smaller than the bottom width.",
        ));
    }
    let volume = e.longueur(0) * (fond + haut) / 2.0 * e.longueur(3);
    Ok(vec![
        Resultat::nouveau(
            t("Volume en place", "Volume in place"),
            Quantite::Vrac(volume),
        ),
        Resultat::nouveau(
            t("Volume à transporter, foisonné", "Volume to haul, loose"),
            Quantite::Vrac(volume * (1.0 + e.nombre(4) / 100.0)),
        ),
    ])
}

fn voyages_de_camion(e: &Lues) -> Calcul {
    let voyages = arrondi_au_dessus(e.nombre(0) / e.nombre(1)).max(1.0);
    Ok(vec![
        Resultat::nouveau(
            t("Voyages", "Truckloads"),
            Quantite::Compte(voyages, t("voyages", "loads")),
        ),
        Resultat::nouveau(
            t("Charge moyenne d'un voyage", "Average load per trip"),
            Quantite::Mesure(e.nombre(0) / voyages, t("vg³", "yd³")),
        ),
    ])
}

fn pente_de_terrain(e: &Lues) -> Calcul {
    let remplis = (0..3).filter(|i| e.presente(*i)).count();
    if remplis != 2 {
        return Err(t(
            "Remplissez exactement deux des trois champs.",
            "Fill in exactly two of the three fields.",
        ));
    }
    let (distance, denivele, pente) = match (e.presente(0), e.presente(1)) {
        (true, true) => (
            e.longueur(0),
            e.longueur(1),
            e.longueur(1) / e.longueur(0) * 100.0,
        ),
        (true, false) => (
            e.longueur(0),
            e.longueur(0) * e.nombre(2) / 100.0,
            e.nombre(2),
        ),
        _ => (
            e.longueur(1) / (e.nombre(2) / 100.0),
            e.longueur(1),
            e.nombre(2),
        ),
    };
    Ok(vec![
        Resultat::nouveau(
            t("Distance horizontale", "Horizontal distance"),
            Quantite::Longueur(distance),
        ),
        Resultat::nouveau(t("Dénivelé", "Drop"), Quantite::Longueur(denivele)),
        Resultat::nouveau(t("Pente", "Slope"), Quantite::Fixe(pente, 2, tel("%"))),
        Resultat::nouveau(
            tel("Angle"),
            Quantite::Fixe((pente / 100.0).atan().to_degrees(), 2, tel("°")),
        ),
        Resultat::nouveau(
            t("Longueur en pente", "Slope length"),
            Quantite::Longueur(distance.hypot(denivele)),
        ),
    ])
}

fn terrasse(e: &Lues) -> Calcul {
    let (longueur, largeur) = (e.longueur(0), e.longueur(1));
    let rangs = arrondi_au_dessus(largeur / (e.longueur(2) + e.longueur(3))).max(1.0);
    let planches = rangs * longueur;
    // Une solive à chaque bout, puis une à chaque espacement.
    let solives = arrondi_au_dessus(longueur / e.longueur(4)) + 1.0;
    Ok(vec![
        Resultat::nouveau(t("Surface", "Area"), Quantite::Aire(longueur * largeur)),
        Resultat::nouveau(
            t("Rangs de planches", "Rows of boards"),
            Quantite::Compte(rangs, t("rangs", "rows")),
        ),
        Resultat::nouveau(
            t("Longueur de planches", "Total board length"),
            Quantite::Longueur(planches),
        ),
        Resultat::nouveau(
            t(
                "Longueur à acheter, pertes comprises",
                "Length to buy, waste included",
            ),
            Quantite::Longueur(planches * (1.0 + e.nombre(5) / 100.0)),
        ),
        Resultat::nouveau(
            t("Vis", "Screws"),
            Quantite::Compte(rangs * solives * 2.0, t("vis", "screws")),
        )
        .a_acheter(t("Vis de terrasse", "Deck screws"), &[]),
    ])
}

fn tourbe(e: &Lues) -> Calcul {
    let surface = e.longueur(0) * e.longueur(1);
    let a_couvrir = surface * (1.0 + e.nombre(3) / 100.0);
    Ok(vec![
        Resultat::nouveau(t("Surface", "Area"), Quantite::Aire(surface)),
        Resultat::nouveau(
            t("Rouleaux", "Rolls"),
            Quantite::Compte(
                arrondi_au_dessus(a_couvrir / PO2_PAR_PI2 / e.nombre(2)),
                t("rouleaux", "rolls"),
            ),
        )
        .a_acheter(t("Rouleaux de tourbe", "Rolls of sod"), &[2]),
    ])
}

fn poteaux(e: &Lues) -> Calcul {
    let rond = |diametre: f64| PI / 4.0 * diametre * diametre;
    let trou = rond(e.longueur(0));
    let poteau = if e.choix(2) == 1 {
        rond(e.longueur(3))
    } else {
        e.longueur(3) * e.longueur(3)
    };
    if poteau >= trou {
        return Err(t(
            "Le poteau n'entre pas dans le trou.",
            "The post does not fit in the hole.",
        ));
    }
    let volume = (trou - poteau) * e.longueur(1) * e.nombre(4);
    let litres = volume / PO3_PAR_L;
    Ok(vec![
        Resultat::nouveau(
            t("Volume de béton", "Concrete volume"),
            Quantite::Vrac(volume),
        ),
        Resultat::nouveau(
            t("Volume, en litres", "Volume, in litres"),
            Quantite::Mesure(litres, tel("L")),
        ),
        Resultat::nouveau(
            t("Sacs de béton prémélangé", "Bags of premixed concrete"),
            Quantite::Compte(arrondi_au_dessus(litres / e.nombre(5)), t("sacs", "bags")),
        )
        .a_acheter(
            t("Sacs de béton prémélangé", "Bags of premixed concrete"),
            &[4],
        ),
    ])
}
