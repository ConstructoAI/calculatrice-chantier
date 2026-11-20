//! Maçonnerie et pavage : briques, blocs, murets, pavés (section 22.10).

use super::{Calcul, Calculateur, Champ, Lues, Metier, Quantite, Resultat, arrondi_au_dessus};
use crate::langue::{Texte, t, tel};
use crate::unites::{Format, PO2_PAR_PI2};

const JOINT: Champ = Champ::longueur(t("Joint de mortier", "Mortar joint"), Format::PoFrac)
    .defaut("3/8")
    .nul_permis()
    .court(tel("Joint"));
const PERTES: Champ = Champ::nombre(t("Pertes", "Waste"), tel("%"))
    .defaut("5")
    .nul_permis();

pub const CALCULATEURS: &[Calculateur] = &[
    Calculateur {
        id: "briques-et-blocs",
        metier: Metier::Maconnerie,
        nom: t("Briques et blocs", "Bricks and blocks"),
        resume: t(
            "Le nombre de briques ou de blocs d'un mur, joints de mortier compris.",
            "The number of bricks or blocks in a wall, mortar joints included.",
        ),
        champs: &[
            Champ::longueur(t("Longueur du mur", "Wall length"), Format::PiPo)
                .court(t("Longueur", "Length")),
            Champ::longueur(t("Hauteur du mur", "Wall height"), Format::PiPo)
                .court(t("Hauteur", "Height")),
            Champ::nombre(
                t("Ouvertures à déduire", "Openings to deduct"),
                t("pi²", "ft²"),
            )
            .defaut("0")
            .nul_permis()
            .court(t("Ouvertures", "Openings")),
            Champ::longueur(t("Longueur d'une unité", "Unit length"), Format::PoFrac)
                .court(t("Longueur\nunité", "Unit\nlength")),
            Champ::longueur(t("Hauteur d'une unité", "Unit height"), Format::PoFrac)
                .court(t("Hauteur\nunité", "Unit\nheight")),
            JOINT,
            PERTES,
        ],
        note: t(
            "Mesurez la brique ou le bloc sans le joint. Un bloc de béton courant mesure 15 5/8 po sur 7 5/8 po (390 mm sur 190 mm).",
            "Measure the brick or block without the joint. A standard concrete block is 15 5/8 in by 7 5/8 in (390 mm by 190 mm).",
        ),
        calcul: briques_et_blocs,
    },
    Calculateur {
        id: "rangs-de-maconnerie",
        metier: Metier::Maconnerie,
        nom: t("Rangs de maçonnerie", "Masonry courses"),
        resume: t(
            "Le nombre de rangs pour atteindre une hauteur, joints de mortier compris.",
            "The number of courses to reach a height, mortar joints included.",
        ),
        champs: &[
            Champ::longueur(t("Hauteur à atteindre", "Height to reach"), Format::PiPo)
                .court(t("Hauteur", "Height")),
            Champ::longueur(t("Hauteur d'une unité", "Unit height"), Format::PoFrac)
                .court(t("Hauteur\nunité", "Unit\nheight")),
            JOINT,
        ],
        note: Texte::VIDE,
        calcul: rangs_de_maconnerie,
    },
    Calculateur {
        id: "muret",
        metier: Metier::Maconnerie,
        nom: t("Muret de blocs", "Dry-stacked block wall"),
        resume: t(
            "Les blocs d'un muret posé à sec : par rang, et en tout.",
            "The blocks for a dry-stacked low wall: per course and in total.",
        ),
        champs: &[
            Champ::longueur(t("Longueur du muret", "Wall length"), Format::PiPo)
                .court(t("Longueur", "Length")),
            Champ::longueur(t("Hauteur du muret", "Wall height"), Format::PiPo)
                .court(t("Hauteur", "Height")),
            Champ::longueur(t("Longueur d'un bloc", "Block length"), Format::PoFrac)
                .court(t("Longueur\nbloc", "Block\nlength")),
            Champ::longueur(t("Hauteur d'un bloc", "Block height"), Format::PoFrac)
                .court(t("Hauteur\nbloc", "Block\nheight")),
        ],
        note: t(
            "Sans les blocs de couronnement ni le rang enfoui. Un muret de soutènement se construit selon les directives du fabricant.",
            "Cap blocks and the buried course are not included. A retaining wall must be built to the manufacturer's instructions.",
        ),
        calcul: muret,
    },
    Calculateur {
        id: "paves",
        metier: Metier::Maconnerie,
        nom: t("Pavés et dalles", "Pavers and slabs"),
        resume: t(
            "Le nombre de pavés d'une surface rectangulaire et le sable de son lit de pose.",
            "The number of pavers for a rectangular area and the sand for its bedding layer.",
        ),
        champs: &[
            Champ::longueur(t("Longueur", "Length"), Format::PiPo),
            Champ::longueur(t("Largeur", "Width"), Format::PiPo),
            Champ::longueur(t("Longueur d'un pavé", "Paver length"), Format::PoFrac)
                .court(t("Longueur\npavé", "Paver\nlength")),
            Champ::longueur(t("Largeur d'un pavé", "Paver width"), Format::PoFrac)
                .court(t("Largeur\npavé", "Paver\nwidth")),
            PERTES,
            Champ::longueur(
                t("Épaisseur du lit de sable", "Sand bed thickness"),
                Format::PoFrac,
            )
            .defaut("1")
            .nul_permis()
            .court(t("Lit\nde sable", "Sand\nbed")),
        ],
        note: t(
            "Les joints ne sont pas comptés. Prévoyez plus de pertes pour une pose en diagonale ou des bordures courbes.",
            "Joints are not counted. Allow more waste for a diagonal pattern or curved edges.",
        ),
        calcul: paves,
    },
];

fn briques_et_blocs(e: &Lues) -> Calcul {
    let surface = e.longueur(0) * e.longueur(1) - e.nombre(2) * PO2_PAR_PI2;
    if surface <= 0.0 {
        return Err(t(
            "Les ouvertures couvrent toute la surface du mur.",
            "The openings cover the whole wall.",
        ));
    }
    let joint = e.longueur(5);
    // Chaque unité occupe sa face, plus un joint sur deux de ses côtés.
    let unite = (e.longueur(3) + joint) * (e.longueur(4) + joint);
    Ok(vec![
        Resultat::nouveau(t("Surface du mur", "Wall area"), Quantite::Aire(surface)),
        Resultat::nouveau(
            t("Unités au pied carré", "Units per square foot"),
            Quantite::Fixe(PO2_PAR_PI2 / unite, 3, Texte::VIDE),
        ),
        Resultat::nouveau(
            t(
                "Unités nécessaires, pertes comprises",
                "Units needed, waste included",
            ),
            Quantite::Compte(
                arrondi_au_dessus(surface / unite * (1.0 + e.nombre(6) / 100.0)),
                t("unités", "units"),
            ),
        ),
    ])
}

fn rangs_de_maconnerie(e: &Lues) -> Calcul {
    let rang = e.longueur(1) + e.longueur(2);
    let rangs = arrondi_au_dessus(e.longueur(0) / rang).max(1.0);
    Ok(vec![
        Resultat::nouveau(
            t(
                "Hauteur d'un rang, joint compris",
                "Course height, joint included",
            ),
            Quantite::Longueur(rang),
        ),
        Resultat::nouveau(
            t("Rangs", "Courses"),
            Quantite::Compte(rangs, t("rangs", "courses")),
        ),
        Resultat::nouveau(
            t("Hauteur obtenue", "Height reached"),
            Quantite::Longueur(rangs * rang),
        ),
    ])
}

fn muret(e: &Lues) -> Calcul {
    let par_rang = arrondi_au_dessus(e.longueur(0) / e.longueur(2)).max(1.0);
    let rangs = arrondi_au_dessus(e.longueur(1) / e.longueur(3)).max(1.0);
    Ok(vec![
        Resultat::nouveau(
            t("Blocs par rang", "Blocks per course"),
            Quantite::Compte(par_rang, t("blocs", "blocks")),
        ),
        Resultat::nouveau(
            t("Rangs", "Courses"),
            Quantite::Compte(rangs, t("rangs", "courses")),
        ),
        Resultat::nouveau(
            t("Blocs", "Blocks"),
            Quantite::Compte(par_rang * rangs, t("blocs", "blocks")),
        ),
        Resultat::nouveau(
            t("Hauteur obtenue", "Height reached"),
            Quantite::Longueur(rangs * e.longueur(3)),
        ),
    ])
}

fn paves(e: &Lues) -> Calcul {
    let surface = e.longueur(0) * e.longueur(1);
    let pave = e.longueur(2) * e.longueur(3);
    Ok(vec![
        Resultat::nouveau(t("Surface", "Area"), Quantite::Aire(surface)),
        Resultat::nouveau(
            t("Pavés", "Pavers"),
            Quantite::Compte(
                arrondi_au_dessus(surface * (1.0 + e.nombre(4) / 100.0) / pave),
                t("pavés", "pavers"),
            ),
        ),
        Resultat::nouveau(
            t("Sable du lit de pose", "Bedding sand"),
            Quantite::Vrac(surface * e.longueur(5)),
        ),
    ])
}
