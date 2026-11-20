//! Acier et soudure : la masse des pièces de métal (section 22.13).

use std::f64::consts::PI;

use super::{Calcul, Calculateur, Champ, Lues, Metier, PO3_PAR_M3, Quantite, Resultat};
use crate::langue::{Texte, t, tel};
use crate::unites::{Format, KG_PAR_LB, PO_PAR_M, PO_PAR_PI};

const METAUX: &[Texte] = &[
    t("Acier", "Steel"),
    t("Acier inoxydable", "Stainless steel"),
    t("Aluminium", "Aluminum"),
];
/// Masse volumique de chaque métal, en kilogrammes par mètre cube.
const KG_PAR_M3: [f64; 3] = [7_850.0, 8_000.0, 2_700.0];

const METAL: Champ = Champ::choix(t("Métal", "Metal"), METAUX);
const PIECES: Champ = Champ::entier(t("Nombre de pièces", "Number of pieces"), Texte::VIDE)
    .defaut("1")
    .court(t("Pièces", "Pieces"));
const PAROI: Champ = Champ::longueur(t("Épaisseur de la paroi", "Wall thickness"), Format::PoFrac)
    .facultatif()
    .court(t("Paroi", "Wall"));

const NOTE_DES_METAUX: Texte = t(
    "Masses volumiques : acier, 7 850 kg/m³; acier inoxydable, 8 000 kg/m³; aluminium, 2 700 kg/m³.",
    "Densities: steel, 7,850 kg/m³; stainless steel, 8,000 kg/m³; aluminum, 2,700 kg/m³.",
);
const NOTE_DES_BARRES: Texte = t(
    "Laissez la paroi vide pour une barre pleine. Masses volumiques : acier, 7 850 kg/m³; acier inoxydable, 8 000 kg/m³; aluminium, 2 700 kg/m³.",
    "Leave the wall blank for a solid bar. Densities: steel, 7,850 kg/m³; stainless steel, 8,000 kg/m³; aluminum, 2,700 kg/m³.",
);
const NOTE_DES_TUBES: Texte = t(
    "Laissez la paroi vide pour une barre pleine. Coins vifs : un tube aux coins arrondis pèse un peu moins.",
    "Leave the wall blank for a solid bar. Square corners: a tube with rounded corners weighs slightly less.",
);

pub const CALCULATEURS: &[Calculateur] = &[
    Calculateur {
        id: "plaque",
        metier: Metier::Acier,
        nom: t("Masse d'une plaque", "Plate weight"),
        resume: t(
            "La masse d'une plaque ou d'une tôle rectangulaire.",
            "The weight of a rectangular plate or sheet.",
        ),
        champs: &[
            Champ::longueur(t("Longueur", "Length"), Format::PiPo),
            Champ::longueur(t("Largeur", "Width"), Format::PiPo),
            Champ::longueur(t("Épaisseur", "Thickness"), Format::PoFrac),
            METAL,
            PIECES,
        ],
        note: NOTE_DES_METAUX,
        calcul: plaque,
    },
    Calculateur {
        id: "barre-ronde",
        metier: Metier::Acier,
        nom: t(
            "Masse d'une barre ou d'un tube rond",
            "Round bar or tube weight",
        ),
        resume: t(
            "La masse d'une barre ronde pleine ou d'un tube rond, au pied et en tout.",
            "The weight of a solid round bar or a round tube, per foot and in total.",
        ),
        champs: &[
            Champ::longueur(t("Diamètre extérieur", "Outside diameter"), Format::PoFrac)
                .court(t("Diamètre", "Diameter")),
            PAROI,
            Champ::longueur(t("Longueur", "Length"), Format::PiPo),
            METAL,
            PIECES,
        ],
        note: NOTE_DES_BARRES,
        calcul: barre_ronde,
    },
    Calculateur {
        id: "profile-rectangulaire",
        metier: Metier::Acier,
        nom: t(
            "Masse d'une barre plate ou d'un tube rectangulaire",
            "Flat bar or rectangular tube weight",
        ),
        resume: t(
            "La masse d'une barre plate, d'un carré plein ou d'un tube carré ou rectangulaire.",
            "The weight of a flat bar, a solid square bar, or a square or rectangular tube.",
        ),
        champs: &[
            Champ::longueur(t("Largeur", "Width"), Format::PoFrac),
            Champ::longueur(t("Hauteur", "Height"), Format::PoFrac),
            PAROI,
            Champ::longueur(t("Longueur", "Length"), Format::PiPo),
            METAL,
            PIECES,
        ],
        note: NOTE_DES_TUBES,
        calcul: profile_rectangulaire,
    },
    Calculateur {
        id: "cordon-de-soudure",
        metier: Metier::Acier,
        nom: t("Cordon de soudure d'angle", "Fillet weld"),
        resume: t(
            "Le métal que demande un cordon de soudure d'angle en acier, et les électrodes à prévoir.",
            "The metal needed for a steel fillet weld, and the electrodes to allow for.",
        ),
        champs: &[
            Champ::longueur(t("Côté du cordon", "Weld leg size"), Format::PoFrac)
                .court(t("Côté", "Leg")),
            Champ::longueur(t("Longueur du cordon", "Weld length"), Format::PiPo)
                .court(t("Longueur", "Length")),
            Champ::nombre(t("Rendement de dépôt", "Deposition efficiency"), tel("%"))
                .defaut("65")
                .court(t("Rendement", "Efficiency")),
        ],
        note: t(
            "Cordon à face plane et à côtés égaux, en acier. Le rendement de dépôt dépend du procédé : environ 60 à 65 % à l'électrode enrobée, plus de 90 % au fil plein. Prévoyez un surplus pour le bombé du cordon.",
            "Flat-faced, equal-leg fillet weld in steel. Deposition efficiency depends on the process: about 60 to 65% for stick electrodes, over 90% for solid wire. Allow extra for the weld reinforcement.",
        ),
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
        Resultat::nouveau(
            t("Masse d'une pièce", "Weight per piece"),
            Quantite::Masse(d_une_piece),
        ),
        Resultat::nouveau(
            t("Masse totale", "Total weight"),
            Quantite::Masse(d_une_piece * pieces),
        ),
    ]
}

/// La masse linéique d'un profilé de section donnée (en pouces carrés) :
/// en livres au pied et en kilogrammes au mètre.
fn masse_lineique(section: f64, metal: usize) -> Resultat {
    let lb_au_pied = masse(section * PO_PAR_PI, metal);
    Resultat::nouveau(
        t("Masse linéique", "Weight per length"),
        Quantite::Double(
            lb_au_pied,
            t("lb/pi", "lb/ft"),
            lb_au_pied * KG_PAR_LB / PO_PAR_PI * PO_PAR_M,
            tel("kg/m"),
        ),
    )
}

fn plaque(e: &Lues) -> Calcul {
    let face = e.longueur(0) * e.longueur(1);
    let mut resultats = vec![Resultat::nouveau(
        t("Surface d'une face", "Area of one face"),
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
            return Err(t(
                "La paroi dépasse la moitié du diamètre.",
                "The wall is more than half the diameter.",
            ));
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
            return Err(t(
                "La paroi dépasse la moitié du plus petit côté.",
                "The wall is more than half the smaller side.",
            ));
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
        return Err(t(
            "Le rendement ne dépasse pas 100 %.",
            "Efficiency cannot exceed 100%.",
        ));
    }
    let cote = e.longueur(0);
    // Section triangulaire : la moitié du carré construit sur le côté.
    let depose = masse(cote * cote / 2.0 * e.longueur(1), 0);
    Ok(vec![
        Resultat::nouveau(
            t("Métal déposé", "Deposited metal"),
            Quantite::Masse(depose),
        ),
        Resultat::nouveau(
            t("Électrodes ou fil à prévoir", "Electrodes or wire needed"),
            Quantite::Masse(depose / (rendement / 100.0)),
        ),
    ])
}
