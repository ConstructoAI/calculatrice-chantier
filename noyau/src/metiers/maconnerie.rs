//! Maçonnerie et pavage : briques, blocs, murets, pavés (section 22.10).

use super::{Calcul, Calculateur, Champ, Lues, Metier, Quantite, Resultat, arrondi_au_dessus};
use crate::unites::{Format, PO2_PAR_PI2};

const JOINT: Champ = Champ::longueur("Joint de mortier", Format::PoFrac)
    .defaut("3/8")
    .nul_permis()
    .court("Joint");
const PERTES: Champ = Champ::nombre("Pertes", "%").defaut("5").nul_permis();

pub const CALCULATEURS: &[Calculateur] = &[
    Calculateur {
        id: "briques-et-blocs",
        metier: Metier::Maconnerie,
        nom: "Briques et blocs",
        resume: "Le nombre de briques ou de blocs d'un mur, joints de mortier compris.",
        champs: &[
            Champ::longueur("Longueur du mur", Format::PiPo).court("Longueur"),
            Champ::longueur("Hauteur du mur", Format::PiPo).court("Hauteur"),
            Champ::nombre("Ouvertures à déduire", "pi²")
                .defaut("0")
                .nul_permis()
                .court("Ouvertures"),
            Champ::longueur("Longueur d'une unité", Format::PoFrac).court("Longueur\nunité"),
            Champ::longueur("Hauteur d'une unité", Format::PoFrac).court("Hauteur\nunité"),
            JOINT,
            PERTES,
        ],
        note: "Mesurez la brique ou le bloc sans le joint. Un bloc de béton courant mesure 15 5/8 po sur 7 5/8 po (390 mm sur 190 mm).",
        calcul: briques_et_blocs,
    },
    Calculateur {
        id: "rangs-de-maconnerie",
        metier: Metier::Maconnerie,
        nom: "Rangs de maçonnerie",
        resume: "Le nombre de rangs pour atteindre une hauteur, joints de mortier compris.",
        champs: &[
            Champ::longueur("Hauteur à atteindre", Format::PiPo).court("Hauteur"),
            Champ::longueur("Hauteur d'une unité", Format::PoFrac).court("Hauteur\nunité"),
            JOINT,
        ],
        note: "",
        calcul: rangs_de_maconnerie,
    },
    Calculateur {
        id: "muret",
        metier: Metier::Maconnerie,
        nom: "Muret de blocs",
        resume: "Les blocs d'un muret posé à sec : par rang, et en tout.",
        champs: &[
            Champ::longueur("Longueur du muret", Format::PiPo).court("Longueur"),
            Champ::longueur("Hauteur du muret", Format::PiPo).court("Hauteur"),
            Champ::longueur("Longueur d'un bloc", Format::PoFrac).court("Longueur\nbloc"),
            Champ::longueur("Hauteur d'un bloc", Format::PoFrac).court("Hauteur\nbloc"),
        ],
        note: "Sans les blocs de couronnement ni le rang enfoui. Un muret de soutènement se construit selon les directives du fabricant.",
        calcul: muret,
    },
    Calculateur {
        id: "paves",
        metier: Metier::Maconnerie,
        nom: "Pavés et dalles",
        resume: "Le nombre de pavés d'une surface rectangulaire et le sable de son lit de pose.",
        champs: &[
            Champ::longueur("Longueur", Format::PiPo),
            Champ::longueur("Largeur", Format::PiPo),
            Champ::longueur("Longueur d'un pavé", Format::PoFrac).court("Longueur\npavé"),
            Champ::longueur("Largeur d'un pavé", Format::PoFrac).court("Largeur\npavé"),
            PERTES,
            Champ::longueur("Épaisseur du lit de sable", Format::PoFrac)
                .defaut("1")
                .nul_permis()
                .court("Lit\nde sable"),
        ],
        note: "Les joints ne sont pas comptés. Prévoyez plus de pertes pour une pose en diagonale ou des bordures courbes.",
        calcul: paves,
    },
];

fn briques_et_blocs(e: &Lues) -> Calcul {
    let surface = e.longueur(0) * e.longueur(1) - e.nombre(2) * PO2_PAR_PI2;
    if surface <= 0.0 {
        return Err("Les ouvertures couvrent toute la surface du mur.");
    }
    let joint = e.longueur(5);
    // Chaque unité occupe sa face, plus un joint sur deux de ses côtés.
    let unite = (e.longueur(3) + joint) * (e.longueur(4) + joint);
    Ok(vec![
        Resultat::nouveau("Surface du mur", Quantite::Aire(surface)),
        Resultat::nouveau(
            "Unités au pied carré",
            Quantite::Fixe(PO2_PAR_PI2 / unite, 3, ""),
        ),
        Resultat::nouveau(
            "Unités nécessaires, pertes comprises",
            Quantite::Compte(
                arrondi_au_dessus(surface / unite * (1.0 + e.nombre(6) / 100.0)),
                "unités",
            ),
        ),
    ])
}

fn rangs_de_maconnerie(e: &Lues) -> Calcul {
    let rang = e.longueur(1) + e.longueur(2);
    let rangs = arrondi_au_dessus(e.longueur(0) / rang).max(1.0);
    Ok(vec![
        Resultat::nouveau("Hauteur d'un rang, joint compris", Quantite::Longueur(rang)),
        Resultat::nouveau("Rangs", Quantite::Compte(rangs, "rangs")),
        Resultat::nouveau("Hauteur obtenue", Quantite::Longueur(rangs * rang)),
    ])
}

fn muret(e: &Lues) -> Calcul {
    let par_rang = arrondi_au_dessus(e.longueur(0) / e.longueur(2)).max(1.0);
    let rangs = arrondi_au_dessus(e.longueur(1) / e.longueur(3)).max(1.0);
    Ok(vec![
        Resultat::nouveau("Blocs par rang", Quantite::Compte(par_rang, "blocs")),
        Resultat::nouveau("Rangs", Quantite::Compte(rangs, "rangs")),
        Resultat::nouveau("Blocs", Quantite::Compte(par_rang * rangs, "blocs")),
        Resultat::nouveau("Hauteur obtenue", Quantite::Longueur(rangs * e.longueur(3))),
    ])
}

fn paves(e: &Lues) -> Calcul {
    let surface = e.longueur(0) * e.longueur(1);
    let pave = e.longueur(2) * e.longueur(3);
    Ok(vec![
        Resultat::nouveau("Surface", Quantite::Aire(surface)),
        Resultat::nouveau(
            "Pavés",
            Quantite::Compte(
                arrondi_au_dessus(surface * (1.0 + e.nombre(4) / 100.0) / pave),
                "pavés",
            ),
        ),
        Resultat::nouveau(
            "Sable du lit de pose",
            Quantite::Vrac(surface * e.longueur(5)),
        ),
    ])
}
