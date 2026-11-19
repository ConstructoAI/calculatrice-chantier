//! Acier et soudure : la masse des pièces de métal (section 22.13).

use std::f64::consts::PI;

use super::{Calcul, Calculateur, Champ, Lues, Metier, PO3_PAR_M3, Quantite, Resultat};
use crate::unites::{Format, KG_PAR_LB, PO_PAR_M, PO_PAR_PI};

const METAUX: &[&str] = &["Acier", "Acier inoxydable", "Aluminium"];
/// Masse volumique de chaque métal, en kilogrammes par mètre cube.
const KG_PAR_M3: [f64; 3] = [7_850.0, 8_000.0, 2_700.0];

const METAL: Champ = Champ::choix("Métal", METAUX);
const PIECES: Champ = Champ::entier("Nombre de pièces", "")
    .defaut("1")
    .court("Pièces");
const PAROI: Champ = Champ::longueur("Épaisseur de la paroi", Format::PoFrac)
    .facultatif()
    .court("Paroi");

const NOTE_DES_METAUX: &str = "Masses volumiques : acier, 7 850 kg/m³; acier inoxydable, 8 000 kg/m³; aluminium, 2 700 kg/m³.";
const NOTE_DES_BARRES: &str = "Laissez la paroi vide pour une barre pleine. Masses volumiques : acier, 7 850 kg/m³; acier inoxydable, 8 000 kg/m³; aluminium, 2 700 kg/m³.";
const NOTE_DES_TUBES: &str = "Laissez la paroi vide pour une barre pleine. Coins vifs : un tube aux coins arrondis pèse un peu moins.";

pub const CALCULATEURS: &[Calculateur] = &[
    Calculateur {
        id: "plaque",
        metier: Metier::Acier,
        nom: "Masse d'une plaque",
        resume: "La masse d'une plaque ou d'une tôle rectangulaire.",
        champs: &[
            Champ::longueur("Longueur", Format::PiPo),
            Champ::longueur("Largeur", Format::PiPo),
            Champ::longueur("Épaisseur", Format::PoFrac),
            METAL,
            PIECES,
        ],
        note: NOTE_DES_METAUX,
        calcul: plaque,
    },
    Calculateur {
        id: "barre-ronde",
        metier: Metier::Acier,
        nom: "Masse d'une barre ou d'un tube rond",
        resume: "La masse d'une barre ronde pleine ou d'un tube rond, au pied et en tout.",
        champs: &[
            Champ::longueur("Diamètre extérieur", Format::PoFrac).court("Diamètre"),
            PAROI,
            Champ::longueur("Longueur", Format::PiPo),
            METAL,
            PIECES,
        ],
        note: NOTE_DES_BARRES,
        calcul: barre_ronde,
    },
    Calculateur {
        id: "profile-rectangulaire",
        metier: Metier::Acier,
        nom: "Masse d'une barre plate ou d'un tube rectangulaire",
        resume: "La masse d'une barre plate, d'un carré plein ou d'un tube carré ou rectangulaire.",
        champs: &[
            Champ::longueur("Largeur", Format::PoFrac),
            Champ::longueur("Hauteur", Format::PoFrac),
            PAROI,
            Champ::longueur("Longueur", Format::PiPo),
            METAL,
            PIECES,
        ],
        note: NOTE_DES_TUBES,
        calcul: profile_rectangulaire,
    },
    Calculateur {
        id: "cordon-de-soudure",
        metier: Metier::Acier,
        nom: "Cordon de soudure d'angle",
        resume: "Le métal que demande un cordon de soudure d'angle en acier, et les électrodes à prévoir.",
        champs: &[
            Champ::longueur("Côté du cordon", Format::PoFrac).court("Côté"),
            Champ::longueur("Longueur du cordon", Format::PiPo).court("Longueur"),
            Champ::nombre("Rendement de dépôt", "%")
                .defaut("65")
                .court("Rendement"),
        ],
        note: "Cordon à face plane et à côtés égaux, en acier. Le rendement de dépôt dépend du procédé : environ 60 à 65 % à l'électrode enrobée, plus de 90 % au fil plein. Prévoyez un surplus pour le bombé du cordon.",
        calcul: cordon_de_soudure,
    },
];

/// Masse, en livres, d'un volume de métal donné en pouces cubes.
fn masse(volume: f64, metal: usize) -> f64 {
    let masse_volumique = KG_PAR_M3.get(metal).copied().unwrap_or(KG_PAR_M3[0]);
    volume / PO3_PAR_M3 * masse_volumique / KG_PAR_LB
}

/// Les deux lignes communes : la masse d'une pièce, puis celle de toutes.
fn masses(d_une_piece: f64, pieces: f64) -> [Resultat; 2] {
    [
        Resultat::nouveau("Masse d'une pièce", Quantite::Masse(d_une_piece)),
        Resultat::nouveau("Masse totale", Quantite::Masse(d_une_piece * pieces)),
    ]
}

/// La masse linéique d'un profilé de section donnée (en pouces carrés) :
/// en livres au pied et en kilogrammes au mètre.
fn masse_lineique(section: f64, metal: usize) -> Resultat {
    let lb_au_pied = masse(section * PO_PAR_PI, metal);
    Resultat::nouveau(
        "Masse linéique",
        Quantite::Double(
            lb_au_pied,
            "lb/pi",
            lb_au_pied * KG_PAR_LB / PO_PAR_PI * PO_PAR_M,
            "kg/m",
        ),
    )
}

fn plaque(e: &Lues) -> Calcul {
    let face = e.longueur(0) * e.longueur(1);
    let mut resultats = vec![Resultat::nouveau(
        "Surface d'une face",
        Quantite::Aire(face),
    )];
    resultats.extend(masses(masse(face * e.longueur(2), e.choix(3)), e.nombre(4)));
    Ok(resultats)
}

fn barre_ronde(e: &Lues) -> Calcul {
    let diametre = e.longueur(0);
    let creux = if e.presente(1) {
        let interieur = diametre - 2.0 * e.longueur(1);
        if interieur < 0.0 {
            return Err("La paroi dépasse la moitié du diamètre.");
        }
        interieur
    } else {
        0.0
    };
    let section = PI / 4.0 * (diametre * diametre - creux * creux);
    let metal = e.choix(3);
    let mut resultats = vec![masse_lineique(section, metal)];
    resultats.extend(masses(masse(section * e.longueur(2), metal), e.nombre(4)));
    Ok(resultats)
}

fn profile_rectangulaire(e: &Lues) -> Calcul {
    let (largeur, hauteur) = (e.longueur(0), e.longueur(1));
    let creux = if e.presente(2) {
        let paroi = e.longueur(2);
        if 2.0 * paroi > largeur.min(hauteur) {
            return Err("La paroi dépasse la moitié du plus petit côté.");
        }
        (largeur - 2.0 * paroi) * (hauteur - 2.0 * paroi)
    } else {
        0.0
    };
    let section = largeur * hauteur - creux;
    let metal = e.choix(4);
    let mut resultats = vec![masse_lineique(section, metal)];
    resultats.extend(masses(masse(section * e.longueur(3), metal), e.nombre(5)));
    Ok(resultats)
}

fn cordon_de_soudure(e: &Lues) -> Calcul {
    let rendement = e.nombre(2);
    if rendement > 100.0 {
        return Err("Le rendement ne dépasse pas 100 %.");
    }
    let cote = e.longueur(0);
    // Section triangulaire : la moitié du carré construit sur le côté.
    let depose = masse(cote * cote / 2.0 * e.longueur(1), 0);
    Ok(vec![
        Resultat::nouveau("Métal déposé", Quantite::Masse(depose)),
        Resultat::nouveau(
            "Électrodes ou fil à prévoir",
            Quantite::Masse(depose / (rendement / 100.0)),
        ),
    ])
}
