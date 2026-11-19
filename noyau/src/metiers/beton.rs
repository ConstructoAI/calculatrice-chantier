//! Béton et fondations (section 22.3).

use std::f64::consts::PI;

use super::{
    Calcul, Calculateur, Champ, Lues, Metier, PO3_PAR_L, Quantite, Resultat, arrondi_au_dessus,
};
use crate::unites::{Format, KG_PAR_LB, PO_PAR_M};

/// Béton perdu ou en trop, en pourcentage du volume théorique.
const PERTES: Champ = Champ::nombre("Pertes", "%").defaut("10").nul_permis();

/// Barres d'armature métriques : désignation et masse linéique en kg/m
/// (aire nominale × 7 850 kg/m³).
const BARRES: &[&str] = &["10M", "15M", "20M", "25M"];
const MASSE_DES_BARRES: [f64; 4] = [0.785, 1.570, 2.355, 3.925];

pub const CALCULATEURS: &[Calculateur] = &[
    Calculateur {
        id: "dalle",
        metier: Metier::Beton,
        nom: "Dalle de béton",
        resume: "Le volume de béton d'une dalle rectangulaire.",
        champs: &[
            Champ::longueur("Longueur", Format::PiPo),
            Champ::longueur("Largeur", Format::PiPo),
            Champ::longueur("Épaisseur", Format::PoFrac),
            PERTES,
        ],
        note: "",
        calcul: dalle,
    },
    Calculateur {
        id: "semelle",
        metier: Metier::Beton,
        nom: "Semelle filante",
        resume: "Le volume de béton d'une semelle continue, sous un mur.",
        champs: &[
            Champ::longueur("Longueur totale", Format::PiPo),
            Champ::longueur("Largeur", Format::PoFrac),
            Champ::longueur("Épaisseur", Format::PoFrac),
            PERTES,
        ],
        note: "",
        calcul: semelle,
    },
    Calculateur {
        id: "mur-de-fondation",
        metier: Metier::Beton,
        nom: "Mur de fondation",
        resume: "Le volume de béton d'un mur et la surface de son coffrage.",
        champs: &[
            Champ::longueur("Longueur totale", Format::PiPo),
            Champ::longueur("Hauteur", Format::PiPo),
            Champ::longueur("Épaisseur", Format::PoFrac),
            PERTES,
        ],
        note: "Les ouvertures ne sont pas déduites.",
        calcul: mur,
    },
    Calculateur {
        id: "colonnes",
        metier: Metier::Beton,
        nom: "Colonnes et pieux ronds",
        resume: "Le béton de colonnes rondes coulées dans des tubes, en volume et en sacs.",
        champs: &[
            Champ::longueur("Diamètre", Format::PoFrac),
            Champ::longueur("Hauteur", Format::PiPo),
            Champ::entier("Nombre de colonnes", "").defaut("1"),
            PERTES,
            Champ::nombre("Rendement d'un sac", "L").defaut("14"),
        ],
        note: "Le rendement est inscrit sur le sac : environ 14 L pour un sac de 30 kg de béton prémélangé.",
        calcul: colonnes,
    },
    Calculateur {
        id: "armature",
        metier: Metier::Beton,
        nom: "Armature d'une dalle",
        resume: "Les barres d'armature d'une dalle, posées dans les deux sens à espacement régulier.",
        champs: &[
            Champ::longueur("Longueur", Format::PiPo),
            Champ::longueur("Largeur", Format::PiPo),
            Champ::longueur("Espacement des barres", Format::PoFrac).defaut("16"),
            Champ::choix("Barre", BARRES),
        ],
        note: "Sans les chevauchements ni l'enrobage des rives : ajoutez-les selon les plans.",
        calcul: armature,
    },
];

/// Les deux lignes communes : le volume, puis le volume à commander.
fn volumes(volume: f64, pertes: f64) -> [Resultat; 2] {
    [
        Resultat::nouveau("Volume de béton", Quantite::Vrac(volume)),
        Resultat::nouveau(
            "À commander, pertes comprises",
            Quantite::Vrac(volume * (1.0 + pertes / 100.0)),
        ),
    ]
}

fn dalle(e: &Lues) -> Calcul {
    let aire = e.longueur(0) * e.longueur(1);
    let mut resultats = vec![Resultat::nouveau("Surface", Quantite::Aire(aire))];
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
        "Coffrage, deux faces",
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
        "Sacs de béton prémélangé",
        Quantite::Compte(
            arrondi_au_dessus(a_commander / PO3_PAR_L / e.nombre(4)),
            "sacs",
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
            "Barres dans le sens de la longueur",
            Quantite::Compte(dans_la_longueur, "barres"),
        ),
        Resultat::nouveau(
            "Barres dans le sens de la largeur",
            Quantite::Compte(dans_la_largeur, "barres"),
        ),
        Resultat::nouveau("Longueur totale de barres", Quantite::Longueur(total)),
        Resultat::nouveau("Masse d'acier", Quantite::Masse(masse_kg / KG_PAR_LB)),
    ])
}
