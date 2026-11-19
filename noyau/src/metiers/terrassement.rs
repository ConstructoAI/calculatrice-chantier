//! Terrassement et aménagement (section 22.9).

use super::{Calcul, Calculateur, Champ, Lues, Metier, Quantite, Resultat, arrondi_au_dessus};
use crate::unites::{Format, LB_PAR_TC, PO3_PAR_VG3};

pub const CALCULATEURS: &[Calculateur] = &[
    Calculateur {
        id: "excavation",
        metier: Metier::Terrassement,
        nom: "Excavation et remblai",
        resume: "Le volume d'une excavation rectangulaire, en place et une fois la terre remuée.",
        champs: &[
            Champ::longueur("Longueur", Format::PiPo),
            Champ::longueur("Largeur", Format::PiPo),
            Champ::longueur("Profondeur", Format::PiPo),
            Champ::nombre("Foisonnement", "%").defaut("25").nul_permis(),
        ],
        note: "La terre remuée prend plus de place : environ 10 à 15 % de plus pour le sable, 25 % pour la terre, 30 % et plus pour l'argile.",
        calcul: excavation,
    },
    Calculateur {
        id: "granulats",
        metier: Metier::Terrassement,
        nom: "Pierre, gravier et sable",
        resume: "Le volume et la masse d'une couche de granulats.",
        champs: &[
            Champ::longueur("Longueur", Format::PiPo),
            Champ::longueur("Largeur", Format::PiPo),
            Champ::longueur("Épaisseur", Format::PoFrac),
            Champ::nombre("Masse volumique", "tc/vg³").defaut("1,5"),
        ],
        note: "Comptez environ 1,5 tonne courte par verge cube de pierre concassée; le fournisseur donne la valeur exacte de son matériau.",
        calcul: granulats,
    },
    Calculateur {
        id: "cloture",
        metier: Metier::Terrassement,
        nom: "Clôture",
        resume: "Le nombre de poteaux et de sections d'une clôture droite, également espacés.",
        champs: &[
            Champ::longueur("Longueur totale", Format::PiPo),
            Champ::longueur("Espacement visé des poteaux", Format::PiPo).defaut("8"),
        ],
        note: "",
        calcul: cloture,
    },
];

fn excavation(e: &Lues) -> Calcul {
    let volume = e.longueur(0) * e.longueur(1) * e.longueur(2);
    Ok(vec![
        Resultat::nouveau("Volume en place", Quantite::Vrac(volume)),
        Resultat::nouveau(
            "Volume à transporter, foisonné",
            Quantite::Vrac(volume * (1.0 + e.nombre(3) / 100.0)),
        ),
    ])
}

fn granulats(e: &Lues) -> Calcul {
    let volume = e.longueur(0) * e.longueur(1) * e.longueur(2);
    let tonnes_courtes = volume / PO3_PAR_VG3 * e.nombre(3);
    Ok(vec![
        Resultat::nouveau("Volume", Quantite::Vrac(volume)),
        Resultat::nouveau("Masse", Quantite::Tonnes(tonnes_courtes * LB_PAR_TC)),
    ])
}

fn cloture(e: &Lues) -> Calcul {
    let longueur = e.longueur(0);
    let sections = arrondi_au_dessus(longueur / e.longueur(1)).max(1.0);
    Ok(vec![
        Resultat::nouveau("Sections", Quantite::Compte(sections, "sections")),
        Resultat::nouveau("Poteaux", Quantite::Compte(sections + 1.0, "poteaux")),
        Resultat::nouveau("Espacement réel", Quantite::Longueur(longueur / sections)),
    ])
}
