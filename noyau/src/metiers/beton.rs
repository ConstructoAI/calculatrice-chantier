//! Béton et fondations (section 22.4).

use std::f64::consts::PI;

use super::{
    Calcul, Calculateur, Champ, Lues, Metier, PO3_PAR_L, Quantite, Resultat, arrondi_au_dessus,
};
use crate::langue::{Texte, t, tel};
use crate::unites::{Format, KG_PAR_LB, PO_PAR_M};

/// Béton perdu ou en trop, en pourcentage du volume théorique.
const PERTES: Champ = Champ::nombre(t("Pertes", "Waste"), tel("%"))
    .defaut("10")
    .nul_permis();

/// Barres d'armature métriques : désignation et masse linéique en kg/m
/// (aire nominale × 7 850 kg/m³).
const BARRES: &[Texte] = &[tel("10M"), tel("15M"), tel("20M"), tel("25M")];
const MASSE_DES_BARRES: [f64; 4] = [0.785, 1.570, 2.355, 3.925];

pub const CALCULATEURS: &[Calculateur] = &[
    Calculateur {
        id: "dalle",
        metier: Metier::Beton,
        nom: t("Dalle de béton", "Concrete slab"),
        resume: t(
            "Le volume de béton d'une dalle rectangulaire.",
            "The volume of concrete for a rectangular slab.",
        ),
        champs: &[
            Champ::longueur(t("Longueur", "Length"), Format::PiPo),
            Champ::longueur(t("Largeur", "Width"), Format::PiPo),
            Champ::longueur(t("Épaisseur", "Thickness"), Format::PoFrac),
            PERTES,
        ],
        note: Texte::VIDE,
        calcul: dalle,
    },
    Calculateur {
        id: "semelle",
        metier: Metier::Beton,
        nom: t("Semelle filante", "Strip footing"),
        resume: t(
            "Le volume de béton d'une semelle continue, sous un mur.",
            "The volume of concrete for a continuous footing under a wall.",
        ),
        champs: &[
            Champ::longueur(t("Longueur totale", "Total length"), Format::PiPo)
                .court(t("Longueur", "Length")),
            Champ::longueur(t("Largeur", "Width"), Format::PoFrac),
            Champ::longueur(t("Épaisseur", "Thickness"), Format::PoFrac),
            PERTES,
        ],
        note: Texte::VIDE,
        calcul: semelle,
    },
    Calculateur {
        id: "mur-de-fondation",
        metier: Metier::Beton,
        nom: t("Mur de fondation", "Foundation wall"),
        resume: t(
            "Le volume de béton d'un mur et la surface de son coffrage.",
            "The volume of concrete for a wall and the area of its formwork.",
        ),
        champs: &[
            Champ::longueur(t("Longueur totale", "Total length"), Format::PiPo)
                .court(t("Longueur", "Length")),
            Champ::longueur(t("Hauteur", "Height"), Format::PiPo),
            Champ::longueur(t("Épaisseur", "Thickness"), Format::PoFrac),
            PERTES,
        ],
        note: t(
            "Les ouvertures ne sont pas déduites.",
            "Openings are not deducted.",
        ),
        calcul: mur,
    },
    Calculateur {
        id: "colonnes",
        metier: Metier::Beton,
        nom: t("Colonnes et pieux ronds", "Round columns and piers"),
        resume: t(
            "Le béton de colonnes rondes coulées dans des tubes, en volume et en sacs.",
            "The concrete for round columns poured in form tubes, by volume and in bags.",
        ),
        champs: &[
            Champ::longueur(t("Diamètre", "Diameter"), Format::PoFrac),
            Champ::longueur(t("Hauteur", "Height"), Format::PiPo),
            Champ::entier(t("Nombre de colonnes", "Number of columns"), Texte::VIDE)
                .court(t("Nombre", "Count"))
                .defaut("1"),
            PERTES,
            Champ::nombre(t("Rendement d'un sac", "Yield per bag"), tel("L"))
                .court(t("Sac", "Bag"))
                .defaut("14"),
        ],
        note: t(
            "Le rendement est inscrit sur le sac : environ 14 L pour un sac de 30 kg de béton prémélangé.",
            "The yield is printed on the bag: about 14 L for a 30 kg bag of premixed concrete.",
        ),
        calcul: colonnes,
    },
    Calculateur {
        id: "armature",
        metier: Metier::Beton,
        nom: t("Armature d'une dalle", "Slab rebar"),
        resume: t(
            "Les barres d'armature d'une dalle, posées dans les deux sens à espacement régulier.",
            "The rebar for a slab, laid both ways at regular spacing.",
        ),
        champs: &[
            Champ::longueur(t("Longueur", "Length"), Format::PiPo),
            Champ::longueur(t("Largeur", "Width"), Format::PiPo),
            Champ::longueur(t("Espacement des barres", "Bar spacing"), Format::PoFrac)
                .court(t("Espacement", "Spacing"))
                .defaut("16"),
            Champ::choix(t("Barre", "Bar"), BARRES),
        ],
        note: t(
            "Sans les chevauchements ni l'enrobage des rives : ajoutez-les selon les plans.",
            "Laps and edge cover are not included: add them as the drawings require.",
        ),
        calcul: armature,
    },
    Calculateur {
        id: "sacs-de-beton",
        metier: Metier::Beton,
        nom: t("Béton en sacs", "Bagged concrete"),
        resume: t(
            "Le nombre de sacs de béton prémélangé pour un petit ouvrage rectangulaire.",
            "The number of bags of premixed concrete for a small rectangular pour.",
        ),
        champs: &[
            Champ::longueur(t("Longueur", "Length"), Format::PiPo),
            Champ::longueur(t("Largeur", "Width"), Format::PiPo),
            Champ::longueur(t("Épaisseur", "Thickness"), Format::PoFrac),
            PERTES,
            Champ::nombre(t("Rendement d'un sac", "Yield per bag"), tel("L"))
                .defaut("14")
                .court(t("Sac", "Bag")),
        ],
        note: t(
            "Le rendement est inscrit sur le sac : environ 14 L pour un sac de 30 kg de béton prémélangé.",
            "The yield is printed on the bag: about 14 L for a 30 kg bag of premixed concrete.",
        ),
        calcul: sacs_de_beton,
    },
    Calculateur {
        id: "escalier-de-beton",
        metier: Metier::Beton,
        nom: t("Escalier de béton", "Concrete stairs"),
        resume: t(
            "Le volume de béton d'un escalier droit coulé sur une paillasse inclinée.",
            "The volume of concrete for a straight stair poured on a sloped waist slab.",
        ),
        champs: &[
            Champ::entier(t("Nombre de marches", "Number of steps"), Texte::VIDE)
                .court(t("Marches", "Steps")),
            Champ::longueur(
                t("Hauteur d'une contremarche", "Riser height"),
                Format::PoFrac,
            )
            .court(t("Contremarche", "Riser")),
            Champ::longueur(t("Giron", "Tread depth"), Format::PoFrac),
            Champ::longueur(t("Largeur de l'escalier", "Stair width"), Format::PiPo)
                .court(t("Largeur", "Width")),
            Champ::longueur(
                t("Épaisseur de la paillasse", "Waist slab thickness"),
                Format::PoFrac,
            )
            .nul_permis()
            .court(t("Paillasse", "Waist slab")),
            PERTES,
        ],
        note: t(
            "La paillasse est la dalle inclinée sous les marches; son épaisseur se mesure perpendiculairement à la pente. Mettez 0 pour des marches coulées sur un sol en pente.",
            "The waist slab is the sloped slab under the steps; its thickness is measured perpendicular to the slope. Enter 0 for steps poured on sloped ground.",
        ),
        calcul: escalier_de_beton,
    },
    Calculateur {
        id: "treillis",
        metier: Metier::Beton,
        nom: t("Treillis métallique", "Welded wire mesh"),
        resume: t(
            "Le nombre de feuilles de treillis pour armer une dalle, chevauchements compris.",
            "The number of mesh sheets to reinforce a slab, overlaps included.",
        ),
        champs: &[
            Champ::longueur(t("Longueur", "Length"), Format::PiPo),
            Champ::longueur(t("Largeur", "Width"), Format::PiPo),
            Champ::longueur(t("Longueur d'une feuille", "Sheet length"), Format::PiPo)
                .defaut("8")
                .court(t("Longueur\nfeuille", "Sheet\nlength")),
            Champ::longueur(t("Largeur d'une feuille", "Sheet width"), Format::PiPo)
                .defaut("4")
                .court(t("Largeur\nfeuille", "Sheet\nwidth")),
            Champ::longueur(t("Chevauchement", "Overlap"), Format::PoFrac)
                .defaut("6")
                .nul_permis()
                .court(t("Chevauch.", "Overlap")),
        ],
        note: t(
            "Chaque feuille perd la largeur d'un chevauchement sur deux de ses côtés.",
            "Each sheet loses one overlap width along two of its sides.",
        ),
        calcul: treillis,
    },
];

/// Les deux lignes communes : le volume, puis le volume à commander.
fn volumes(volume: f64, pertes: f64) -> [Resultat; 2] {
    [
        Resultat::nouveau(
            t("Volume de béton", "Concrete volume"),
            Quantite::Vrac(volume),
        ),
        Resultat::nouveau(
            t("À commander, pertes comprises", "To order, waste included"),
            Quantite::Vrac(volume * (1.0 + pertes / 100.0)),
        ),
    ]
}

fn dalle(e: &Lues) -> Calcul {
    let aire = e.longueur(0) * e.longueur(1);
    let mut resultats = vec![Resultat::nouveau(
        t("Surface", "Area"),
        Quantite::Aire(aire),
    )];
    resultats.extend(volumes(aire * e.longueur(2), e.nombre(3)));
    Ok(resultats)
}

fn semelle(e: &Lues) -> Calcul {
    let volume = e.longueur(0) * e.longueur(1) * e.longueur(2);
    Ok(volumes(volume, e.nombre(3)).to_vec())
}

fn mur(e: &Lues) -> Calcul {
    let face = e.longueur(0) * e.longueur(1);
    let mut resultats = volumes(face * e.longueur(2), e.nombre(3)).to_vec();
    resultats.push(Resultat::nouveau(
        t("Coffrage, deux faces", "Formwork, both faces"),
        Quantite::Aire(2.0 * face),
    ));
    Ok(resultats)
}

fn colonnes(e: &Lues) -> Calcul {
    let diametre = e.longueur(0);
    let volume = PI / 4.0 * diametre * diametre * e.longueur(1) * e.nombre(2);
    let a_commander = volume * (1.0 + e.nombre(3) / 100.0);
    let mut resultats = volumes(volume, e.nombre(3)).to_vec();
    resultats.push(Resultat::nouveau(
        t("Sacs de béton prémélangé", "Bags of premixed concrete"),
        Quantite::Compte(
            arrondi_au_dessus(a_commander / PO3_PAR_L / e.nombre(4)),
            t("sacs", "bags"),
        ),
    ));
    Ok(resultats)
}

fn armature(e: &Lues) -> Calcul {
    let (longueur, largeur, espacement) = (e.longueur(0), e.longueur(1), e.longueur(2));
    // Une barre à chaque rive, puis une à chaque espacement.
    let rangs = |portee: f64| (portee / espacement + 1e-9).floor() + 1.0;
    let dans_la_longueur = rangs(largeur);
    let dans_la_largeur = rangs(longueur);
    let total = dans_la_longueur * longueur + dans_la_largeur * largeur;
    let masse_lineique = MASSE_DES_BARRES
        .get(e.choix(3))
        .copied()
        .unwrap_or(MASSE_DES_BARRES[0]);
    let masse_kg = total / PO_PAR_M * masse_lineique;
    Ok(vec![
        Resultat::nouveau(
            t(
                "Barres dans le sens de la longueur",
                "Bars running lengthwise",
            ),
            Quantite::Compte(dans_la_longueur, t("barres", "bars")),
        ),
        Resultat::nouveau(
            t(
                "Barres dans le sens de la largeur",
                "Bars running crosswise",
            ),
            Quantite::Compte(dans_la_largeur, t("barres", "bars")),
        ),
        Resultat::nouveau(
            t("Longueur totale de barres", "Total bar length"),
            Quantite::Longueur(total),
        ),
        Resultat::nouveau(
            t("Masse d'acier", "Steel weight"),
            Quantite::Masse(masse_kg / KG_PAR_LB),
        ),
    ])
}

fn sacs_de_beton(e: &Lues) -> Calcul {
    let volume = e.longueur(0) * e.longueur(1) * e.longueur(2);
    let litres = volume * (1.0 + e.nombre(3) / 100.0) / PO3_PAR_L;
    Ok(vec![
        Resultat::nouveau(
            t("Volume de béton", "Concrete volume"),
            Quantite::Vrac(volume),
        ),
        Resultat::nouveau(
            t(
                "Volume à préparer, pertes comprises",
                "Volume to mix, waste included",
            ),
            Quantite::Mesure(litres, tel("L")),
        ),
        Resultat::nouveau(
            t("Sacs de béton prémélangé", "Bags of premixed concrete"),
            Quantite::Compte(arrondi_au_dessus(litres / e.nombre(4)), t("sacs", "bags")),
        ),
    ])
}

fn escalier_de_beton(e: &Lues) -> Calcul {
    let (marches, hauteur, giron, largeur) =
        (e.nombre(0), e.longueur(1), e.longueur(2), e.longueur(3));
    // Chaque marche est un prisme triangulaire; la paillasse suit la pente.
    let des_marches = marches * hauteur * giron / 2.0 * largeur;
    let de_la_paillasse = marches * hauteur.hypot(giron) * e.longueur(4) * largeur;
    Ok(volumes(des_marches + de_la_paillasse, e.nombre(5)).to_vec())
}

fn treillis(e: &Lues) -> Calcul {
    let chevauchement = e.longueur(4);
    if chevauchement >= e.longueur(2).min(e.longueur(3)) {
        return Err(t(
            "Le chevauchement dépasse la feuille.",
            "The overlap is larger than the sheet.",
        ));
    }
    let surface = e.longueur(0) * e.longueur(1);
    let utile = (e.longueur(2) - chevauchement) * (e.longueur(3) - chevauchement);
    Ok(vec![
        Resultat::nouveau(t("Surface", "Area"), Quantite::Aire(surface)),
        Resultat::nouveau(
            t("Feuilles de treillis", "Mesh sheets"),
            Quantite::Compte(arrondi_au_dessus(surface / utile), t("feuilles", "sheets")),
        ),
    ])
}
