//! Terrassement et aménagement (section 22.9).

use std::f64::consts::PI;

use super::{
    Calcul, Calculateur, Champ, Lues, Metier, PO3_PAR_L, Quantite, Resultat, arrondi_au_dessus,
};
use crate::unites::{Format, LB_PAR_TC, PO2_PAR_PI2, PO3_PAR_VG3};

const FORMES_DE_POTEAU: &[&str] = &["Carré", "Rond"];

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
            Champ::nombre("Masse volumique", "tc/vg³")
                .court("Masse\nvolumique")
                .defaut("1,5"),
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
            Champ::longueur("Longueur totale", Format::PiPo).court("Longueur"),
            Champ::longueur("Espacement visé des poteaux", Format::PiPo)
                .court("Espacement")
                .defaut("8"),
        ],
        note: "",
        calcul: cloture,
    },
    Calculateur {
        id: "tranchee",
        metier: Metier::Terrassement,
        nom: "Tranchée",
        resume: "Le volume d'une tranchée, à parois droites ou inclinées.",
        champs: &[
            Champ::longueur("Longueur", Format::PiPo),
            Champ::longueur("Largeur au fond", Format::PiPo).court("Largeur\nau fond"),
            Champ::longueur("Largeur en haut", Format::PiPo)
                .facultatif()
                .court("Largeur\nen haut"),
            Champ::longueur("Profondeur", Format::PiPo),
            Champ::nombre("Foisonnement", "%").defaut("25").nul_permis(),
        ],
        note: "Laissez la largeur en haut vide pour des parois verticales. L'inclinaison des parois est une question de sécurité : suivez le Code de sécurité pour les travaux de construction.",
        calcul: tranchee,
    },
    Calculateur {
        id: "voyages-de-camion",
        metier: Metier::Terrassement,
        nom: "Voyages de camion",
        resume: "Le nombre de voyages pour transporter un volume de terre ou de pierre.",
        champs: &[
            Champ::nombre("Volume à transporter", "vg³").court("Volume"),
            Champ::nombre("Capacité du camion", "vg³").court("Camion"),
        ],
        note: "Comptez le volume foisonné : la terre remuée prend plus de place. La capacité dépend du camion, du matériau et des limites de charge.",
        calcul: voyages_de_camion,
    },
    Calculateur {
        id: "pente-de-terrain",
        metier: Metier::Terrassement,
        nom: "Pente et dénivelé",
        resume: "Remplissez deux des trois grandeurs : la troisième est calculée.",
        champs: &[
            Champ::longueur("Distance horizontale", Format::PiPo)
                .facultatif()
                .court("Distance"),
            Champ::longueur("Dénivelé", Format::PoFrac).facultatif(),
            Champ::nombre("Pente", "%").facultatif(),
        ],
        note: "Une pente de 2 % équivaut à environ 1/4 po au pied.",
        calcul: pente_de_terrain,
    },
    Calculateur {
        id: "terrasse",
        metier: Metier::Terrassement,
        nom: "Terrasse : planches et vis",
        resume: "Les planches de surface d'une terrasse rectangulaire et les vis pour les fixer.",
        champs: &[
            Champ::longueur("Longueur, dans le sens des planches", Format::PiPo).court("Longueur"),
            Champ::longueur("Largeur de la terrasse", Format::PiPo).court("Largeur"),
            Champ::longueur("Largeur d'une planche", Format::PoFrac)
                .defaut("5 1/2")
                .court("Planche"),
            Champ::longueur("Jeu entre les planches", Format::PoFrac)
                .defaut("1/8")
                .nul_permis()
                .court("Jeu"),
            Champ::longueur("Espacement des solives", Format::PoFrac)
                .defaut("16")
                .court("Solives"),
            Champ::nombre("Pertes", "%").defaut("10").nul_permis(),
        ],
        note: "Deux vis à chaque croisement d'une planche et d'une solive. La structure (solives, poutres, poteaux) se calcule à part.",
        calcul: terrasse,
    },
    Calculateur {
        id: "tourbe",
        metier: Metier::Terrassement,
        nom: "Tourbe en rouleaux",
        resume: "Le nombre de rouleaux de tourbe pour gazonner une surface.",
        champs: &[
            Champ::longueur("Longueur", Format::PiPo),
            Champ::longueur("Largeur", Format::PiPo),
            Champ::nombre("Surface d'un rouleau", "pi²")
                .defaut("10")
                .court("Rouleau"),
            Champ::nombre("Pertes", "%").defaut("5").nul_permis(),
        ],
        note: "Un rouleau mesure souvent 2 pi sur 5 pi, soit 10 pi² : vérifiez auprès du fournisseur.",
        calcul: tourbe,
    },
    Calculateur {
        id: "poteaux",
        metier: Metier::Terrassement,
        nom: "Béton des poteaux",
        resume: "Le béton pour sceller des poteaux de clôture ou de terrasse dans leurs trous.",
        champs: &[
            Champ::longueur("Diamètre du trou", Format::PoFrac).court("Trou"),
            Champ::longueur("Profondeur du trou", Format::PiPo).court("Profondeur"),
            Champ::choix("Poteau", FORMES_DE_POTEAU),
            Champ::longueur("Côté ou diamètre du poteau", Format::PoFrac)
                .nul_permis()
                .court("Côté"),
            Champ::entier("Nombre de poteaux", "")
                .defaut("1")
                .court("Nombre"),
            Champ::nombre("Rendement d'un sac", "L")
                .defaut("14")
                .court("Sac"),
        ],
        note: "Le volume du poteau est déduit de celui du trou; mettez 0 pour un trou rempli de béton. Rendement : environ 14 L pour un sac de 30 kg de béton prémélangé.",
        calcul: poteaux,
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

fn tranchee(e: &Lues) -> Calcul {
    let fond = e.longueur(1);
    let haut = if e.presente(2) { e.longueur(2) } else { fond };
    if haut < fond {
        return Err("La largeur en haut ne peut pas être plus petite que la largeur au fond.");
    }
    let volume = e.longueur(0) * (fond + haut) / 2.0 * e.longueur(3);
    Ok(vec![
        Resultat::nouveau("Volume en place", Quantite::Vrac(volume)),
        Resultat::nouveau(
            "Volume à transporter, foisonné",
            Quantite::Vrac(volume * (1.0 + e.nombre(4) / 100.0)),
        ),
    ])
}

fn voyages_de_camion(e: &Lues) -> Calcul {
    let voyages = arrondi_au_dessus(e.nombre(0) / e.nombre(1)).max(1.0);
    Ok(vec![
        Resultat::nouveau("Voyages", Quantite::Compte(voyages, "voyages")),
        Resultat::nouveau(
            "Charge moyenne d'un voyage",
            Quantite::Mesure(e.nombre(0) / voyages, "vg³"),
        ),
    ])
}

fn pente_de_terrain(e: &Lues) -> Calcul {
    let remplis = (0..3).filter(|i| e.presente(*i)).count();
    if remplis != 2 {
        return Err("Remplissez exactement deux des trois champs.");
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
        Resultat::nouveau("Distance horizontale", Quantite::Longueur(distance)),
        Resultat::nouveau("Dénivelé", Quantite::Longueur(denivele)),
        Resultat::nouveau("Pente", Quantite::Fixe(pente, 2, "%")),
        Resultat::nouveau(
            "Angle",
            Quantite::Fixe((pente / 100.0).atan().to_degrees(), 2, "°"),
        ),
        Resultat::nouveau(
            "Longueur en pente",
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
        Resultat::nouveau("Surface", Quantite::Aire(longueur * largeur)),
        Resultat::nouveau("Rangs de planches", Quantite::Compte(rangs, "rangs")),
        Resultat::nouveau("Longueur de planches", Quantite::Longueur(planches)),
        Resultat::nouveau(
            "Longueur à acheter, pertes comprises",
            Quantite::Longueur(planches * (1.0 + e.nombre(5) / 100.0)),
        ),
        Resultat::nouveau("Vis", Quantite::Compte(rangs * solives * 2.0, "vis")),
    ])
}

fn tourbe(e: &Lues) -> Calcul {
    let surface = e.longueur(0) * e.longueur(1);
    let a_couvrir = surface * (1.0 + e.nombre(3) / 100.0);
    Ok(vec![
        Resultat::nouveau("Surface", Quantite::Aire(surface)),
        Resultat::nouveau(
            "Rouleaux",
            Quantite::Compte(
                arrondi_au_dessus(a_couvrir / PO2_PAR_PI2 / e.nombre(2)),
                "rouleaux",
            ),
        ),
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
        return Err("Le poteau n'entre pas dans le trou.");
    }
    let volume = (trou - poteau) * e.longueur(1) * e.nombre(4);
    let litres = volume / PO3_PAR_L;
    Ok(vec![
        Resultat::nouveau("Volume de béton", Quantite::Vrac(volume)),
        Resultat::nouveau("Volume, en litres", Quantite::Mesure(litres, "L")),
        Resultat::nouveau(
            "Sacs de béton prémélangé",
            Quantite::Compte(arrondi_au_dessus(litres / e.nombre(5)), "sacs"),
        ),
    ])
}
