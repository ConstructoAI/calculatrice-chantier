//! Le plan d'exemple : une maison de plain-pied, son garage attaché et son
//! terrain. Il montre le plus possible de ce que fait l'onglet Dessin — des
//! pièces placées dans l'étage et leurs murs communs, une pièce en L, un mur
//! en biais, un terrain au côté libre, des plafonds cathédrale et en pente,
//! une retombée, un retrait, les dix genres d'ouverture, des cloisons et des
//! espaces nommés, du mobilier et les symboles de chaque corps de métier, un
//! escalier et ses règles, les finis, le calepinage, l'ossature, les lignes de
//! cote, une pièce mesurée à l'extérieur, des notes et des traits à main
//! levée — et, par ses notes, quelques-unes de ses limites.
//!
//! Ses noms, ses notes et ses mesures s'écrivent dans la langue en cours.
//!
//! Le mode Débutant a son exemple à lui (section 25.1, point 61) : une
//! chambre simple, sa porte, sa fenêtre et ses cotes.

use std::f64::consts::{FRAC_1_SQRT_2, TAU};

use noyau::calepinage::{Calepinage, Decalage, Depart, MateriauAPoser, SensDeLaPose};
use noyau::cloison::{Cloison, PorteDeCloison, SegmentDeCloison};
use noyau::dessin::{
    Cote, Crayon, Dessin, GenreDOuverture, MiseEnPage, Note, Ouverture, Piece, Plan, Retrait, Sens,
    TraitAMainLevee,
};
use noyau::espaces::NomDEspace;
use noyau::etage::Place;
use noyau::finis::{self, FiniDePeinture, FiniDePlafond, Finis, Plancher};
use noyau::langue::selon;
use noyau::mobilier::{GenreDObjet, Objet};
use noyau::ossature::Entraxe;
use noyau::plafond::{FormeDePlafond, Plafond, Retombee};
use noyau::prefs::Preferences;

// ----------------------------------------------------------------------
// La maison dans l'étage, en pouces : l'extérieur du mur avant (au nord, du
// côté de la rue) en y = 0, celui du mur ouest en x = 0. Toutes les pièces
// ont les murs de 9 po de départ; deux voisines se rangent de part et d'autre
// d'un seul mur, leurs faces intérieures à 9 po l'une de l'autre.
// ----------------------------------------------------------------------

/// L'épaisseur des murs, celle de départ.
const MUR: f64 = 9.0;
/// La face intérieure ouest de chaque colonne de pièces : le salon et la
/// cuisine (16 pi de large), l'entrée et la salle de bain (8 pi), les
/// chambres (13 pi).
const COLONNE_A: f64 = MUR;
const COLONNE_B: f64 = COLONNE_A + 192.0 + MUR;
const COLONNE_C: f64 = COLONNE_B + 96.0 + MUR;
/// La face intérieure est des chambres : le garage, mesuré à l'extérieur,
/// commence là, de l'autre côté de leur mur commun.
const FACE_EST: f64 = COLONNE_C + 156.0;
/// La face intérieure nord de chaque rangée : à l'avant (13 pi de profond),
/// le corridor (4 pi), à l'arrière (12 pi).
const RANGEE_AVANT: f64 = MUR;
const RANGEE_CORRIDOR: f64 = RANGEE_AVANT + 156.0 + MUR;
const RANGEE_ARRIERE: f64 = RANGEE_CORRIDOR + 48.0 + MUR;
/// Le coin nord-ouest de la maison sur le terrain, à 10 pi de la limite ouest
/// et à 26 pi de la rue : le terrain se place pour que la maison y tombe.
const MAISON_SUR_LE_TERRAIN: (f64, f64) = (120.0, 312.0);

/// Les quarts de tour d'un objet, d'après le côté où il tourne le dos (v = 0) :
/// un objet contre un mur lui tourne le dos. Le haut d'un escalier est de ce
/// côté-là.
const DOS_AU_NORD: u8 = 0;
const DOS_A_L_EST: u8 = 1;
const DOS_AU_SUD: u8 = 2;
const DOS_A_L_OUEST: u8 = 3;

/// Le plan d'exemple : sept pièces placées dans l'étage, puis le terrain; le
/// salon est à l'écran.
pub fn plan_d_exemple() -> Plan {
    Plan {
        pieces: vec![
            salon(),
            cuisine(),
            entree(),
            salle_de_bain(),
            chambre_principale(),
            chambre_2(),
            garage(),
            terrain(),
        ],
        courante: 0,
        mise_en_page: MiseEnPage {
            projet: selon("Maison d'exemple", "Sample house").to_string(),
            adresse: selon("123, rue de l'Exemple", "123 Example Street").to_string(),
            dessine_par: "Constructo AI".to_string(),
            ..MiseEnPage::default()
        },
    }
}

/// L'exemple du mode Débutant (section 25.1, point 61) : une chambre de
/// 12 pi × 10 pi, ses murs de 8 pi, sa fenêtre centrée sur le mur du haut et
/// sa porte sur le mur de droite; ses cotes se montrent d'elles-mêmes.
pub fn plan_simple() -> Plan {
    let dessin = Dessin {
        ouvertures: vec![
            ouverture(GenreDOuverture::Fenetre, 0, "4", &pi_po(3, "6"), ""),
            ouverture(GenreDOuverture::Porte, 1, &po("32"), &pi_po(6, "8"), "1"),
        ],
        ..rectangle(144.0, 120.0, "12", "10")
    };
    Plan {
        pieces: vec![Piece {
            nom: selon("Chambre", "Bedroom").to_string(),
            dessin,
            place: None,
        }],
        ..Plan::default()
    }
}

/// L'exemple de ce mode du dessin, et le nom de son dessin : la maison
/// complète en Avancé, la chambre simple en Débutant.
pub fn exemple_du_mode(avance: bool) -> (Plan, noyau::langue::Texte) {
    if avance {
        (plan_d_exemple(), crate::textes::NOM_DU_PLAN_D_EXEMPLE)
    } else {
        (plan_simple(), crate::textes::NOM_DE_L_EXEMPLE_SIMPLE)
    }
}

/// Vrai quand l'exemple de ce mode s'est déjà ouvert de lui-même (points 58
/// et 61) : il ne s'ajoute plus aux dessins.
pub fn exemple_deja_ouvert(prefs: &Preferences, avance: bool) -> bool {
    if avance {
        prefs.exemple_ouvert
    } else {
        prefs.exemple_simple_ouvert
    }
}

/// Retient que l'exemple de ce mode s'est ouvert.
pub fn marquer_l_exemple_ouvert(prefs: &mut Preferences, avance: bool) {
    if avance {
        prefs.exemple_ouvert = true;
    } else {
        prefs.exemple_simple_ouvert = true;
    }
}

// ----------------------------------------------------------------------
// Les pièces
// ----------------------------------------------------------------------

/// Le salon (16 pi × 13 pi) : son coin nord-ouest coupé à 45° par une fenêtre
/// en baie (le côté en biais se déduit des autres), son plafond cathédrale,
/// une porte double vers l'entrée et des portes coulissantes vers la salle à
/// manger, dans leurs murs communs; le groupe extérieur de la thermopompe,
/// dehors, contre le mur ouest. Deux notes y disent deux limites du dessin.
fn salon() -> Piece {
    // Le tracé part du bout du coin coupé, sur le mur avant : un point compté
    // depuis le coin nord-ouest du rectangle de la pièce passe dans le repère
    // du tracé.
    let p = |x: f64, y: f64| (x - 60.0, y);
    // Le pied de la plinthe sous la baie : au milieu du mur en biais, vers
    // l'intérieur.
    let sous_la_baie = au_dixieme(30.0 + 2.5 * FRAC_1_SQRT_2);
    let dessin = Dessin {
        cotes: vec![
            cote(Sens::Est, 132.0, "11"),
            cote(Sens::Sud, 156.0, "13"),
            cote(Sens::Ouest, 192.0, "16"),
            cote(Sens::Nord, 96.0, "8"),
            cote(Sens::NordEst, au_dixieme(60.0_f64.hypot(60.0)), ""),
        ],
        fermee: true,
        hauteur_des_murs: "8".to_string(),
        plafond: Plafond {
            forme: FormeDePlafond::Cathedrale,
            haut: "12".to_string(),
            cote: 0,
        },
        ouvertures: vec![
            ouverture(GenreDOuverture::Fenetre, 0, "4", "4", ""),
            ouverture(GenreDOuverture::PorteDouble, 1, "5", &pi_po(6, "8"), "4"),
            ouverture(
                GenreDOuverture::PorteCoulissante,
                2,
                "5",
                &pi_po(6, "8"),
                &pi_po(1, "6"),
            ),
            Ouverture {
                allege: "2".to_string(),
                ..ouverture(GenreDOuverture::FenetreEnBaie, 4, "6", &pi_po(4, "6"), "")
            },
        ],
        objets: vec![
            // Le sofa contre le mur ouest, face à la porte double.
            objet(GenreDObjet::Sofa, p(18.0, 108.0), DOS_A_L_OUEST),
            // La thermopompe murale en haut du mur est; son groupe extérieur.
            objet(GenreDObjet::ThermopompeMurale, p(187.5, 134.0), DOS_A_L_EST),
            objet(
                GenreDObjet::ThermopompeExterieure,
                p(-MUR - 7.0, 140.0),
                DOS_A_L_EST,
            ),
            // Les plinthes électriques : sous la fenêtre avant, et sous la
            // baie, de biais.
            objet(GenreDObjet::Plinthe, p(126.0, 2.5), DOS_AU_NORD),
            Objet {
                angle: 315.0,
                ..Objet::nouveau(GenreDObjet::Plinthe, p(sous_la_baie, sous_la_baie))
            },
            objet(GenreDObjet::Thermostat, p(92.0, 152.0), DOS_AU_SUD),
            objet(GenreDObjet::Prise, p(80.0, 6.0), DOS_AU_NORD),
            objet(GenreDObjet::Prise, p(50.0, 150.0), DOS_AU_SUD),
            objet(GenreDObjet::Interrupteur, p(186.0, 36.0), DOS_A_L_EST),
            objet(GenreDObjet::Encastre, p(80.0, 45.0), DOS_AU_NORD),
            objet(GenreDObjet::Encastre, p(150.0, 45.0), DOS_AU_NORD),
            objet(GenreDObjet::Encastre, p(80.0, 95.0), DOS_AU_NORD),
            objet(GenreDObjet::Encastre, p(150.0, 95.0), DOS_AU_NORD),
        ],
        // Deux limites du dessin, en notes courtes : le PDF de l'étage écrit
        // chaque note sur une seule ligne.
        notes: vec![
            note(
                selon("Limite : un étage à la fois", "Limit: one storey at a time"),
                p(100.0, 112.0),
            ),
            note(
                selon("Limite : aucun mur courbe", "Limit: no curved walls"),
                p(100.0, 136.0),
            ),
        ],
        finis: Finis {
            plancher: Plancher::BoisFranc,
            peinture: FiniDePeinture::Velours,
            couleur: selon("Blanc cassé", "Off-white").to_string(),
            plafond: FiniDePlafond::GypsePeint,
            plinthes: finis::Plinthe::Mdf,
        },
        ..Dessin::default()
    };
    piece(
        selon("Salon", "Living room"),
        dessin,
        (COLONNE_A + 60.0, RANGEE_AVANT),
    )
}

/// La cuisine et la salle à manger (16 pi × 16 pi 9 po) : les armoires en L,
/// les électroménagers, l'îlot et la table; un passe-plat vers le salon dans
/// leur mur commun, la porte-patio vers la terrasse, la retombée au-dessus
/// des armoires. Un cercle rouge et une note y marquent une correction.
fn cuisine() -> Piece {
    use GenreDObjet::*;
    let dessin = Dessin {
        ouvertures: vec![
            // Le passe-plat, dans le mur commun avec le salon.
            Ouverture {
                allege: pi_po(3, "6"),
                ..ouverture(GenreDOuverture::Fenetre, 0, "4", "2", "3")
            },
            ouverture(
                GenreDOuverture::PorteFenetre,
                2,
                "5",
                &pi_po(6, "8"),
                &po("6"),
            ),
            // La fenêtre au-dessus de l'évier.
            Ouverture {
                allege: pi_po(3, "6"),
                ..ouverture(GenreDOuverture::Fenetre, 2, "3", "3", &pi_po(10, "6"))
            },
        ],
        retombees: vec![Retombee {
            longueur: pi_po(14, "9"),
            largeur: po("12"),
            hauteur: po("12"),
        }],
        objets: vec![
            // Le long du mur ouest, du nord au sud : le réfrigérateur, les
            // armoires, la cuisinière sous sa hotte et sa prise de 240 V.
            objet(Refrigerateur, (16.0, 15.0), DOS_A_L_OUEST),
            objet(ArmoireDuBas, (12.0, 45.0), DOS_A_L_OUEST),
            objet(ArmoireDuHaut, (6.0, 45.0), DOS_A_L_OUEST),
            objet(Cuisiniere, (13.0, 75.0), DOS_A_L_OUEST),
            objet(Hotte, (10.0, 75.0), DOS_A_L_OUEST),
            objet(Prise240, (6.0, 75.0), DOS_A_L_OUEST),
            objet(ArmoireDuBas, (12.0, 105.0), DOS_A_L_OUEST),
            objet(PriseDdft, (6.0, 105.0), DOS_A_L_OUEST),
            objet(ArmoireDuBas, (12.0, 135.0), DOS_A_L_OUEST),
            objet(ArmoireDuHaut, (6.0, 135.0), DOS_A_L_OUEST),
            objet_mesure(
                ArmoireDuBas,
                (12.0, 163.5),
                DOS_A_L_OUEST,
                &po("27"),
                &po("24"),
            ),
            objet(PriseDdft, (6.0, 163.0), DOS_A_L_OUEST),
            // Le long du mur sud : l'armoire du coin, l'évier sous la
            // fenêtre, le lave-vaisselle, une dernière armoire.
            objet(ArmoireDuBas, (15.0, 189.0), DOS_AU_SUD),
            objet(ArmoireDuHaut, (15.0, 195.0), DOS_AU_SUD),
            objet_mesure(
                ArmoireDuBas,
                (48.0, 189.0),
                DOS_AU_SUD,
                &po("36"),
                &po("24"),
            ),
            objet(Evier, (48.0, 190.0), DOS_AU_SUD),
            objet(LaveVaisselle, (78.0, 189.0), DOS_AU_SUD),
            objet(ArmoireDuBas, (105.0, 189.0), DOS_AU_SUD),
            objet(PriseDdft, (100.0, 195.0), DOS_AU_SUD),
            // L'îlot, son côté repas vers la table; la table et ses chaises.
            objet(Ilot, (104.0, 118.0), DOS_AU_SUD),
            objet(Encastre, (86.0, 118.0), DOS_AU_NORD),
            objet(Encastre, (122.0, 118.0), DOS_AU_NORD),
            objet(Table, (132.0, 46.0), DOS_A_L_EST),
            objet(Luminaire, (132.0, 46.0), DOS_AU_NORD),
            objet(Interrupteur, (186.0, 52.0), DOS_A_L_EST),
            objet(Plinthe, (189.5, 124.0), DOS_A_L_EST),
        ],
        notes: vec![note(
            selon("Îlot : ajouter une prise", "Island: add an outlet"),
            (100.0, 160.0),
        )],
        croquis: vec![TraitAMainLevee {
            points: cercle_a_main_levee((104.0, 118.0), (50.0, 30.0)),
            crayon: Crayon::Rouge,
        }],
        finis: Finis {
            plancher: Plancher::Ceramique,
            peinture: FiniDePeinture::Satine,
            couleur: selon("Blanc", "White").to_string(),
            plafond: FiniDePlafond::GypsePeint,
            plinthes: finis::Plinthe::Mdf,
        },
        ..rectangle(192.0, 201.0, "16", &pi_po(16, "9"))
    };
    piece(
        selon("Cuisine et salle à manger", "Kitchen and dining room"),
        dessin,
        (COLONNE_A, RANGEE_CORRIDOR),
    )
}

/// L'entrée et le corridor, en L : la porte d'entrée, l'escalier droit vers
/// le sous-sol et ses règles, sa trémie retirée du plancher, une baie libre
/// vers la cuisine et la porte du garage; le surligneur y suit le passage.
fn entree() -> Piece {
    use GenreDObjet::*;
    let dessin = Dessin {
        cotes: vec![
            cote(Sens::Est, 96.0, "8"),
            cote(Sens::Sud, 165.0, &pi_po(13, "9")),
            cote(Sens::Est, 165.0, &pi_po(13, "9")),
            cote(Sens::Sud, 48.0, "4"),
            cote(Sens::Ouest, 261.0, ""),
            cote(Sens::Nord, 213.0, ""),
        ],
        fermee: true,
        hauteur_des_murs: "8".to_string(),
        // La trémie de l'escalier, retirée de la surface du plancher.
        retraits: vec![Retrait {
            longueur: pi_po(10, "6"),
            largeur: pi_po(3, "4"),
        }],
        ouvertures: vec![
            ouverture(GenreDOuverture::Porte, 0, "3", &pi_po(6, "8"), "1"),
            ouverture(GenreDOuverture::BaieLibre, 5, "3", "7", &po("6")),
            ouverture(
                GenreDOuverture::Porte,
                3,
                &pi_po(2, "8"),
                &pi_po(6, "8"),
                &po("8"),
            ),
        ],
        objets: vec![
            // L'escalier contre le mur est, son haut au sud, près du corridor;
            // sa longueur suit ses règles : 12 girons de 10 po.
            Objet {
                hauteur: "8".to_string(),
                contremarche_max: po("7 7/8"),
                giron_min: po("10"),
                ..objet(Escalier, (78.0, 93.0), DOS_AU_SUD)
            },
            objet(Plinthe, (2.5, 138.0), DOS_A_L_OUEST),
            objet(Prise, (6.0, 30.0), DOS_A_L_OUEST),
            objet(Prise, (80.0, 207.0), DOS_AU_SUD),
            objet(Interrupteur, (56.0, 6.0), DOS_AU_NORD),
            objet(Interrupteur, (102.0, 171.0), DOS_AU_NORD),
            objet(Luminaire, (30.0, 70.0), DOS_AU_NORD),
            objet(Luminaire, (40.0, 189.0), DOS_AU_NORD),
            objet(Detecteur, (30.0, 140.0), DOS_AU_NORD),
            objet(RetourDAir, (180.0, 167.0), DOS_AU_NORD),
        ],
        // Le dessin ne fait que monter les escaliers : en 3D, celui du
        // sous-sol monte vers le plafond.
        notes: vec![note(
            selon(
                "Escalier du sous-sol : il monte en 3D",
                "Basement stair: it goes up in 3D",
            ),
            (140.0, 200.0),
        )],
        croquis: vec![TraitAMainLevee {
            points: vec![
                (30.0, 14.0),
                (31.0, 100.0),
                (30.0, 189.0),
                (130.0, 190.0),
                (240.0, 188.0),
            ],
            crayon: Crayon::Surligneur,
        }],
        finis: Finis {
            plancher: Plancher::Vinyle,
            peinture: FiniDePeinture::CoquilleDOeuf,
            couleur: selon("Gris pâle", "Light grey").to_string(),
            plafond: FiniDePlafond::GypsePeint,
            plinthes: finis::Plinthe::Mdf,
        },
        ..Dessin::default()
    };
    piece(
        selon("Entrée et corridor", "Entry and hallway"),
        dessin,
        (COLONNE_B, RANGEE_AVANT),
    )
}

/// La salle de bain et la lessive (8 pi × 12 pi) : sa porte de grange sur le
/// corridor, le bain, la toilette, la vanité, la laveuse et la sécheuse; la
/// céramique calepinée au tiers.
fn salle_de_bain() -> Piece {
    use GenreDObjet::*;
    let dessin = Dessin {
        ouvertures: vec![
            ouverture(
                GenreDOuverture::PorteDeGrange,
                0,
                &pi_po(2, "8"),
                "7",
                &po("8"),
            ),
            Ouverture {
                allege: "4".to_string(),
                ..ouverture(
                    GenreDOuverture::Fenetre,
                    2,
                    &pi_po(2, "6"),
                    "3",
                    &pi_po(4, "3"),
                )
            },
        ],
        objets: vec![
            objet(Bain, (30.0, 129.0), DOS_A_L_EST),
            objet(Toilette, (76.0, 130.0), DOS_AU_SUD),
            objet(ColonneDeChute, (91.0, 139.0), DOS_AU_NORD),
            objet_mesure(Vanite, (10.5, 70.0), DOS_A_L_OUEST, &po("36"), &po("21")),
            objet(Laveuse, (81.0, 57.5), DOS_A_L_EST),
            objet(Secheuse, (81.0, 84.5), DOS_A_L_EST),
            objet(RobinetDArret, (92.0, 40.0), DOS_AU_NORD),
            objet(Prise240, (90.0, 106.0), DOS_A_L_EST),
            objet(PriseDdft, (6.0, 44.0), DOS_A_L_OUEST),
            objet(Interrupteur, (6.0, 14.0), DOS_A_L_OUEST),
            objet(Ventilateur, (60.0, 108.0), DOS_AU_NORD),
            objet(Encastre, (36.0, 70.0), DOS_AU_NORD),
            objet(DrainDePlancher, (36.0, 100.0), DOS_AU_NORD),
        ],
        // La céramique de 12 × 24 po, posée au tiers depuis le centre.
        calepinage: Calepinage {
            visible: true,
            materiau: MateriauAPoser::Ceramique,
            largeur: po("12"),
            longueur: po("24"),
            joint: 0.125,
            mur: 0,
            depart: Depart::Centre,
            sens: SensDeLaPose::Droit,
            decalage: Decalage::Tiers,
        },
        finis: Finis {
            plancher: Plancher::Ceramique,
            peinture: FiniDePeinture::SemiLustre,
            couleur: selon("Blanc", "White").to_string(),
            plafond: FiniDePlafond::GypsePeint,
            plinthes: finis::Plinthe::Ceramique,
        },
        ..rectangle(96.0, 144.0, "8", "12")
    };
    piece(
        selon("Salle de bain", "Bathroom"),
        dessin,
        (COLONNE_B, RANGEE_ARRIERE),
    )
}

/// La chambre principale (13 pi × 12 pi) : une garde-robe fermée par une
/// cloison en L et sa porte, deux espaces nommés, ses lignes de cote.
fn chambre_principale() -> Piece {
    use GenreDObjet::*;
    let dessin = Dessin {
        ouvertures: vec![
            ouverture(
                GenreDOuverture::Porte,
                0,
                &pi_po(2, "8"),
                &pi_po(6, "8"),
                &po("8"),
            ),
            ouverture(GenreDOuverture::Fenetre, 2, "5", "4", "4"),
            ouverture(GenreDOuverture::Fenetre, 1, "3", "4", "7"),
        ],
        // La garde-robe dans le coin nord-est : 5 pi × 5 pi, sa porte s'ouvre
        // vers l'intérieur.
        cloisons: vec![Cloison {
            debut: (93.5, 0.0),
            sens: Sens::Sud,
            longueur: po("62 1/2"),
            suite: vec![SegmentDeCloison {
                sens: Sens::Est,
                longueur: po("62 1/2"),
            }],
            ouvertures: vec![PorteDeCloison {
                largeur: po("28"),
                position: po("16"),
                inversee: true,
                segment: 0,
                passage: false,
            }],
            ..Cloison::default()
        }],
        noms_des_espaces: vec![
            nom_d_espace(selon("Garde-robe", "Walk-in closet"), (126.0, 30.0)),
            nom_d_espace(selon("Chambre", "Bedroom"), (60.0, 128.0)),
        ],
        objets: vec![
            objet(Lit, (40.0, 80.0), DOS_A_L_OUEST),
            objet(Prise, (6.0, 42.0), DOS_A_L_OUEST),
            objet(Prise, (6.0, 118.0), DOS_A_L_OUEST),
            objet(Prise, (150.0, 136.0), DOS_A_L_EST),
            objet(Plinthe, (78.0, 141.5), DOS_AU_SUD),
            objet(Plinthe, (153.5, 102.0), DOS_A_L_EST),
            objet(Thermostat, (70.0, 4.0), DOS_AU_NORD),
            objet(Interrupteur, (48.0, 6.0), DOS_AU_NORD),
            objet(Detecteur, (70.0, 30.0), DOS_AU_NORD),
            objet(Luminaire, (110.0, 100.0), DOS_AU_NORD),
            objet(BoucheDeSoufflage, (128.0, 141.5), DOS_AU_SUD),
        ],
        finis: Finis {
            plancher: Plancher::Tapis,
            peinture: FiniDePeinture::Velours,
            couleur: selon("Bleu ardoise", "Slate blue").to_string(),
            plafond: FiniDePlafond::GypsePeint,
            plinthes: finis::Plinthe::Mdf,
        },
        ..rectangle(156.0, 144.0, "13", "12")
    };
    piece(
        selon("Chambre principale", "Primary bedroom"),
        dessin,
        (COLONNE_C, RANGEE_ARRIERE),
    )
}

/// La chambre 2 (13 pi × 13 pi) : la lingerie du corridor prise dans son coin
/// par une cloison en L, ouverte par des portes pliantes dans le mur commun;
/// ses lignes de cote.
fn chambre_2() -> Piece {
    use GenreDObjet::*;
    let dessin = Dessin {
        ouvertures: vec![
            ouverture(GenreDOuverture::Fenetre, 0, "4", "4", ""),
            ouverture(
                GenreDOuverture::Porte,
                2,
                &pi_po(2, "8"),
                &pi_po(6, "8"),
                "2",
            ),
            ouverture(
                GenreDOuverture::PortePliante,
                2,
                "4",
                &pi_po(6, "8"),
                &pi_po(8, "6"),
            ),
        ],
        cloisons: vec![Cloison {
            debut: (0.0, 127.5),
            sens: Sens::Est,
            longueur: po("62 1/2"),
            suite: vec![SegmentDeCloison {
                sens: Sens::Sud,
                longueur: po("28 1/2"),
            }],
            ..Cloison::default()
        }],
        noms_des_espaces: vec![
            nom_d_espace(selon("Lingerie", "Linen closet"), (30.0, 143.0)),
            nom_d_espace(selon("Chambre", "Bedroom"), (45.0, 45.0)),
        ],
        objets: vec![
            objet_mesure(Lit, (118.5, 70.0), DOS_A_L_EST, &po("54"), &po("75")),
            objet(Plinthe, (78.0, 2.5), DOS_AU_NORD),
            objet(Prise, (150.0, 34.0), DOS_A_L_EST),
            objet(Prise, (150.0, 106.0), DOS_A_L_EST),
            objet(Prise, (6.0, 60.0), DOS_A_L_OUEST),
            objet(Interrupteur, (92.0, 150.0), DOS_AU_SUD),
            objet(Detecteur, (30.0, 100.0), DOS_AU_NORD),
            objet(Luminaire, (60.0, 85.0), DOS_AU_NORD),
            objet(BoucheDeSoufflage, (2.5, 100.0), DOS_A_L_OUEST),
        ],
        finis: Finis {
            plancher: Plancher::Flottant,
            peinture: FiniDePeinture::Velours,
            couleur: selon("Vert sauge", "Sage green").to_string(),
            plafond: FiniDePlafond::GypsePeint,
            plinthes: finis::Plinthe::Mdf,
        },
        ..rectangle(156.0, 156.0, "13", "13")
    };
    piece(
        selon("Chambre 2", "Bedroom 2"),
        dessin,
        (COLONNE_C, RANGEE_AVANT),
    )
}

/// Le garage, mesuré à l'extérieur (24 pi 9 po × 24 pi, son mur commun avec la
/// maison compris) : sa porte de garage, son plafond en pente, son ossature
/// aux 24 po; le panneau, le chauffe-eau, la prise de la borne de recharge, et
/// le compteur électrique dehors, contre le mur est. Sa porte de côté s'ouvre
/// poussant gauche.
fn garage() -> Piece {
    use GenreDObjet::*;
    let dessin = Dessin {
        cotes: vec![
            cote(Sens::Est, 297.0, &pi_po(24, "9")),
            cote(Sens::Sud, 288.0, "24"),
            cote(Sens::Ouest, 297.0, ""),
            cote(Sens::Nord, 288.0, ""),
        ],
        fermee: true,
        mesures_exterieures: true,
        hauteur_des_murs: "8".to_string(),
        // La pente monte vers le mur commun avec la maison.
        plafond: Plafond {
            forme: FormeDePlafond::EnPente,
            haut: "10".to_string(),
            cote: 3,
        },
        ossature_visible: true,
        entraxe: Entraxe::VingtQuatre,
        ouvertures: vec![
            ouverture(GenreDOuverture::PorteDeGarage, 0, "16", "7", ""),
            Ouverture {
                inversee: true,
                ..ouverture(
                    GenreDOuverture::Porte,
                    2,
                    &pi_po(2, "8"),
                    &pi_po(6, "8"),
                    &pi_po(3, "9"),
                )
            },
            Ouverture {
                allege: "4".to_string(),
                ..ouverture(GenreDOuverture::Fenetre, 1, "3", "2", "10")
            },
        ],
        objets: vec![
            objet(ChauffeEau, (276.0, 21.0), DOS_AU_NORD),
            objet(CompteurDEau, (258.0, 14.0), DOS_AU_NORD),
            objet(RobinetDArret, (258.0, 26.0), DOS_AU_NORD),
            objet(DrainDePlancher, (148.5, 150.0), DOS_AU_NORD),
            objet(Panneau, (12.0, 100.0), DOS_A_L_OUEST),
            objet(Prise240, (15.0, 60.0), DOS_A_L_OUEST),
            objet(Interrupteur, (15.0, 222.0), DOS_A_L_OUEST),
            objet(Prise, (282.0, 200.0), DOS_A_L_EST),
            objet(Luminaire, (148.5, 100.0), DOS_AU_NORD),
            objet(Luminaire, (148.5, 200.0), DOS_AU_NORD),
            objet(CompteurElectrique, (303.0, 230.0), DOS_A_L_OUEST),
        ],
        notes: vec![
            note(
                selon(
                    "Prise 240 V : borne de recharge",
                    "240 V outlet: EV charger",
                ),
                (120.0, 60.0),
            ),
            note(
                selon(
                    "Porte coupe-feu à fermeture automatique",
                    "Self-closing fire-rated door",
                ),
                (150.0, 198.0),
            ),
        ],
        finis: Finis {
            plancher: Plancher::Beton,
            plafond: FiniDePlafond::GypsePeint,
            ..Finis::default()
        },
        ..Dessin::default()
    };
    piece(selon("Garage", "Garage"), dessin, (FACE_EST, 0.0))
}

/// Le terrain (85 pi sur la rue), sa limite arrière de biais (un côté libre,
/// à 5° de l'équerre), ses marges de recul; la maison, le garage, l'entrée
/// d'auto, la terrasse devant la porte-patio, la piscine et le cabanon (dans
/// la marge arrière). Deux notes y disent deux autres limites.
fn terrain() -> Piece {
    use GenreDObjet::*;
    // La longueur, sur le croquis, des deux côtés qui se déduisent.
    let biais = 5.0_f64.to_radians();
    let arriere = au_dixieme(1020.0 / biais.cos());
    let ouest = au_dixieme(1680.0 + arriere * biais.sin());
    let (x, y) = MAISON_SUR_LE_TERRAIN;
    let dessin = Dessin {
        cotes: vec![
            cote(Sens::Est, 1020.0, "85"),
            cote(Sens::Sud, 1680.0, "140"),
            cote(Sens::Libre(175_000), arriere, ""),
            cote(Sens::Nord, ouest, ""),
        ],
        fermee: true,
        terrain: true,
        marges_de_recul: ["25", "6", "30", "6"].map(String::from).to_vec(),
        objets: vec![
            objet_mesure(Maison, (x + 240.0, y + 192.0), DOS_AU_NORD, "40", "32"),
            objet_mesure(Garage, (x + 624.0, y + 144.0), DOS_AU_NORD, "24", "24"),
            objet_mesure(Entree, (x + 624.0, y / 2.0), DOS_AU_NORD, "18", "26"),
            objet(Terrasse, (x + 166.0, y + 456.0), DOS_AU_NORD),
            objet(Piscine, (x + 400.0, y + 748.0), DOS_A_L_EST),
            objet(Cabanon, (x + 768.0, y + 1140.0), DOS_AU_NORD),
        ],
        // Dans la cour arrière, sous la zone où bâtir et loin du cabanon.
        notes: vec![
            note(
                selon(
                    "Limite : aucune règle d'urbanisme",
                    "Limit: zoning rules not checked",
                ),
                (380.0, 1480.0),
            ),
            note(
                selon(
                    "Code QR : partagez une pièce seule",
                    "QR code: share a single room",
                ),
                (380.0, 1540.0),
            ),
        ],
        ..Dessin::default()
    };
    piece(selon("Terrain", "Lot"), dessin, (-x, -y))
}

// ----------------------------------------------------------------------
// Les morceaux d'un dessin
// ----------------------------------------------------------------------

/// Des pouces, comme au clavier : « 32 po » (« 32 in » en anglais).
fn po(pouces: &str) -> String {
    format!("{pouces} {}", selon("po", "in"))
}

/// Des pieds et des pouces : « 6 pi 8 po » (« 6 ft 8 in »).
fn pi_po(pieds: u32, pouces: &str) -> String {
    format!("{pieds} {} {}", selon("pi", "ft"), po(pouces))
}

/// Une pièce du plan, son coin de départ placé à ce point de l'étage.
fn piece(nom: &str, dessin: Dessin, (x, y): (f64, f64)) -> Piece {
    Piece {
        nom: nom.to_string(),
        dessin,
        place: Some(Place {
            x,
            y,
            rotation: 0.0,
        }),
    }
}

/// Un côté du tracé : sa direction, sa longueur sur le croquis (ici, ses
/// pouces) et sa mesure; une mesure vide se déduit des autres.
fn cote(sens: Sens, croquis: f64, saisie: &str) -> Cote {
    Cote {
        sens,
        croquis,
        saisie: saisie.to_string(),
    }
}

/// Une pièce rectangulaire mesurée à l'intérieur, tracée de son coin
/// nord-ouest : sa largeur et sa profondeur (pouces, pour le croquis), et
/// leurs mesures; les deux autres côtés se déduisent. Ses murs ont 8 pi.
fn rectangle(largeur: f64, profondeur: f64, vers_l_est: &str, vers_le_sud: &str) -> Dessin {
    Dessin {
        cotes: vec![
            cote(Sens::Est, largeur, vers_l_est),
            cote(Sens::Sud, profondeur, vers_le_sud),
            cote(Sens::Ouest, largeur, ""),
            cote(Sens::Nord, profondeur, ""),
        ],
        fermee: true,
        hauteur_des_murs: "8".to_string(),
        ..Dessin::default()
    }
}

/// Une porte ou une fenêtre sur le côté `cote`, à `position` de son premier
/// coin (vide : centrée).
fn ouverture(
    genre: GenreDOuverture,
    cote: usize,
    largeur: &str,
    hauteur: &str,
    position: &str,
) -> Ouverture {
    Ouverture {
        largeur: largeur.to_string(),
        hauteur: hauteur.to_string(),
        genre: Some(genre),
        cote: Some(cote),
        position: position.to_string(),
        ..Ouverture::default()
    }
}

/// Un objet à ses dimensions de départ, son centre dans le repère du tracé
/// de sa pièce.
fn objet(genre: GenreDObjet, centre: (f64, f64), quarts: u8) -> Objet {
    Objet {
        rotation: quarts,
        ..Objet::nouveau(genre, centre)
    }
}

/// Un objet à ses mesures : sa largeur et sa profondeur.
fn objet_mesure(
    genre: GenreDObjet,
    centre: (f64, f64),
    quarts: u8,
    largeur: &str,
    profondeur: &str,
) -> Objet {
    Objet {
        largeur: largeur.to_string(),
        profondeur: profondeur.to_string(),
        ..objet(genre, centre, quarts)
    }
}

fn note(texte: &str, position: (f64, f64)) -> Note {
    Note {
        texte: texte.to_string(),
        position,
    }
}

fn nom_d_espace(nom: &str, point: (f64, f64)) -> NomDEspace {
    NomDEspace {
        point,
        nom: nom.to_string(),
    }
}

/// Une longueur calculée, au dixième de pouce près, comme l'application garde
/// les traits à main levée : le fichier reste léger, et se relit tel quel.
fn au_dixieme(pouces: f64) -> f64 {
    (pouces * 10.0).round() / 10.0
}

/// Un cercle tracé du doigt autour d'un point : un peu plus d'un tour, qui
/// tremble, au dixième de pouce près comme les traits à main levée.
fn cercle_a_main_levee(centre: (f64, f64), (rx, ry): (f64, f64)) -> Vec<(f64, f64)> {
    (0..=13)
        .map(|k| {
            let k = f64::from(k);
            let angle = TAU * k / 12.0 - 0.5;
            let tremblement = 1.0 + 0.04 * (k * 2.3).sin();
            (
                au_dixieme(centre.0 + rx * tremblement * angle.cos()),
                au_dixieme(centre.1 + ry * tremblement * angle.sin()),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use noyau::dessin::{Etat, Mesure};
    use noyau::langue::{self, Langue};
    use noyau::unites::Format;

    const FORMAT: Format = Format::PiPo;

    #[test]
    fn chaque_piece_est_complete_et_sans_conflit() {
        let plan = plan_d_exemple();
        assert_eq!(plan.pieces.len(), 8);
        for piece in &plan.pieces {
            let dessin = &piece.dessin;
            let solution = dessin.resoudre(FORMAT);
            assert!(
                matches!(solution.etat, Etat::Complete { .. }),
                "{} : {:?}",
                piece.nom,
                solution.etat
            );
            assert!(solution.aire_nette().is_some(), "{}", piece.nom);
            // Chaque porte et chaque fenêtre est placée sur son mur, chaque
            // cloison est à ses mesures.
            assert!(
                solution.placements.iter().all(Option::is_some),
                "{} : {:?}",
                piece.nom,
                solution.placements
            );
            assert!(solution.cloisons.iter().all(Option::is_some));
            let conflits: Vec<String> = dessin
                .conflits(&solution)
                .iter()
                .map(|conflit| format!("{:?} : {:?}", dessin.objets[conflit.objet].genre, conflit))
                .collect();
            assert!(conflits.is_empty(), "{} : {conflits:?}", piece.nom);
        }
    }

    #[test]
    fn les_pieces_voisines_partagent_leurs_murs() {
        let mut plan = plan_d_exemple();
        // Toutes les pièces ont leur place : l'étage ne les déplace pas.
        assert!(!plan.placer_les_pieces(FORMAT));
        let pieces = plan.pieces_dans_l_etage(FORMAT);
        assert_eq!(pieces.len(), 7);
        // Aucune pièce n'empiète sur une autre.
        for une in &pieces {
            for autre in pieces.iter().filter(|autre| autre.rang != une.rang) {
                assert!(
                    !une.interieur
                        .iter()
                        .any(|&coin| noyau::espaces::dedans(coin, &autre.interieur)),
                    "{} dans {}",
                    plan.pieces[une.rang].nom,
                    plan.pieces[autre.rang].nom
                );
            }
        }
        let murs = plan.murs_communs(FORMAT);
        assert_eq!(murs.len(), 12, "{murs:?}");
        assert!(murs.iter().all(|mur| (mur.epaisseur - MUR).abs() < 1e-9));
        // L'entrée touche toutes les pièces de la maison.
        let entree = 2;
        let voisines: Vec<usize> = murs
            .iter()
            .filter_map(|mur| match mur.pieces {
                (a, b) if a == entree => Some(b),
                (a, b) if b == entree => Some(a),
                _ => None,
            })
            .collect();
        for rang in [0, 1, 3, 4, 5, 6] {
            assert!(voisines.contains(&rang), "{rang} : {voisines:?}");
        }
        // Les portes et les fenêtres des voisines percent les murs communs :
        // la porte double et les portes coulissantes du salon, le passe-plat,
        // la baie libre, la porte de grange, les portes des chambres et les
        // portes pliantes de la lingerie, la porte du garage.
        let passages: usize = (0..plan.pieces.len())
            .map(|rang| plan.passages_des_voisins(rang, FORMAT).len())
            .sum();
        assert_eq!(passages, 9);
        assert!(plan.maquette_de_l_etage(FORMAT).is_some());
    }

    #[test]
    fn le_plan_montre_ce_que_fait_le_dessin() {
        let plan = plan_d_exemple();
        let dessins: Vec<&Dessin> = plan.pieces.iter().map(|piece| &piece.dessin).collect();
        // Les dix genres d'ouverture.
        for genre in GenreDOuverture::TOUS {
            assert!(
                dessins
                    .iter()
                    .any(|dessin| dessin.ouvertures.iter().any(|o| o.genre == Some(genre))),
                "{genre:?}"
            );
        }
        // Chaque famille d'objets, celle du terrain comprise.
        for famille in noyau::mobilier::Famille::TOUTES {
            assert!(
                dessins
                    .iter()
                    .any(|dessin| dessin.objets.iter().any(|o| o.genre.famille() == famille)),
                "{famille:?}"
            );
        }
        // Le salon : un mur en biais déduit, un plafond cathédrale.
        let salon = dessins[0].resoudre(FORMAT);
        assert!(
            matches!(salon.mesures[4], Mesure::Deduite(d) if (d - 60.0 * 2f64.sqrt()).abs() < 1e-6)
        );
        assert_eq!(
            salon.plafond.map(|plafond| plafond.forme),
            Some(FormeDePlafond::Cathedrale)
        );
        // La cuisine : sa retombée se compte au volume.
        assert!(dessins[1].resoudre(FORMAT).volume.is_some());
        // L'escalier de l'entrée suit ses règles : 13 contremarches, 12 girons
        // de 10 po, sur 10 pi.
        let entree = dessins[2].resoudre(FORMAT);
        let escalier = entree.escaliers.iter().flatten().next().unwrap();
        assert_eq!(escalier.contremarches, 13);
        assert!(escalier.contremarche <= 7.875);
        assert!((escalier.giron.unwrap() - 10.0).abs() < 1e-9);
        assert!((entree.objets[0].1 - 120.0).abs() < 1e-9);
        // Les deux chambres : deux espaces chacune, tous nommés.
        for rang in [4, 5] {
            let dessin = dessins[rang];
            let espaces = dessin.espaces(&dessin.resoudre(FORMAT));
            assert_eq!(espaces.len(), 2, "{}", plan.pieces[rang].nom);
            assert!(espaces.iter().all(|e| dessin.nom_de_l_espace(e).is_some()));
        }
        // Le garage : mesuré à l'extérieur, son ossature paraît, son plafond
        // est en pente.
        let garage = dessins[6].resoudre(FORMAT);
        assert!(dessins[6].mesures_exterieures && garage.aire_exterieure.is_some());
        assert!(dessins[6].ossature_visible);
        assert_eq!(
            garage.plafond.map(|plafond| plafond.forme),
            Some(FormeDePlafond::EnPente)
        );
        // Le terrain : un côté libre déduit, une zone où bâtir, le cabanon
        // dans la marge arrière.
        let terrain = dessins[7];
        let solution = terrain.resoudre(FORMAT);
        assert!(terrain.cotes[2].sens.libre());
        assert!(noyau::implantation::zone_constructible(terrain, &solution).is_some());
        let distances = noyau::implantation::distances_aux_limites(terrain, &solution);
        assert_eq!(distances.len(), 6);
        let cabanon = &distances[5].1;
        assert!(cabanon[2].dans_la_marge, "{cabanon:?}");
        let maison = &distances[0].1;
        assert!(maison.iter().all(|distance| !distance.dans_la_marge));
    }

    #[test]
    fn le_plan_s_exporte_en_pdf_et_en_dxf() {
        let plan = plan_d_exemple();
        let pdf = crate::exportation::pdf_du_plan(&plan, &Preferences::default(), "");
        assert!(!pdf.is_empty());
        assert!(pdf.starts_with("%PDF-1.4") && pdf.ends_with("%%EOF\n"));
        assert!(crate::exportation::dxf_du_plan(&plan, &Preferences::default()).is_some());
    }

    #[test]
    fn chaque_piece_a_sa_3d_et_le_plan_s_enregistre() {
        let plan = plan_d_exemple();
        for piece in &plan.pieces {
            let solution = piece.dessin.resoudre(FORMAT);
            assert!(
                noyau::maquette::maquette(&piece.dessin, &solution).is_some(),
                "{}",
                piece.nom
            );
        }
        // Le plan s'enregistre et se relit tel quel : ses longueurs calculées
        // sont au dixième de pouce, que le JSON relit exactement.
        let json = serde_json::to_string(&plan).unwrap();
        assert_eq!(serde_json::from_str::<Plan>(&json).unwrap(), plan);
    }

    #[test]
    fn le_plan_suit_la_langue() {
        let francais = plan_d_exemple();
        let anglais = {
            let _anglais = langue::pour_la_portee(Langue::Anglais);
            plan_d_exemple()
        };
        assert_eq!(francais.pieces[0].nom, "Salon");
        assert_eq!(anglais.pieces[0].nom, "Living room");
        assert_eq!(anglais.pieces[2].dessin.ouvertures[0].hauteur, "6 ft 8 in");
        // Les mesures écrites en anglais se lisent de même.
        for (une, autre) in francais.pieces.iter().zip(&anglais.pieces) {
            assert_eq!(une.dessin.resoudre(FORMAT), autre.dessin.resoudre(FORMAT));
        }
        assert_eq!(anglais.murs_communs(FORMAT).len(), 12);
    }

    #[test]
    fn l_exemple_simple_montre_l_essentiel() {
        let plan = plan_simple();
        assert_eq!(plan.pieces.len(), 1);
        let dessin = &plan.pieces[0].dessin;
        let solution = dessin.resoudre(FORMAT);
        // Une chambre de 12 pi × 10 pi, complète.
        assert!(
            matches!(solution.etat, Etat::Complete { aire, .. } if (aire - 120.0 * 144.0).abs() < 1e-6),
            "{:?}",
            solution.etat
        );
        // Sa porte et sa fenêtre, placées sur leur mur; ses cotes se montrent.
        assert_eq!(dessin.ouvertures.len(), 2);
        assert!(solution.placements.iter().all(Option::is_some));
        assert!(dessin.cotes_visibles());
        // Rien du mode Avancé.
        assert!(dessin.cloisons.is_empty() && dessin.objets.is_empty());
        assert!(dessin.notes.is_empty() && dessin.croquis.is_empty());
        assert!(dessin.retraits.is_empty() && dessin.retombees.is_empty());
        // Il s'exporte, et suit la langue.
        assert!(!crate::exportation::pdf_du_plan(&plan, &Preferences::default(), "").is_empty());
        assert!(crate::exportation::dxf_du_plan(&plan, &Preferences::default()).is_some());
        let anglais = {
            let _anglais = langue::pour_la_portee(Langue::Anglais);
            plan_simple()
        };
        assert_eq!(plan.pieces[0].nom, "Chambre");
        assert_eq!(anglais.pieces[0].nom, "Bedroom");
        assert_eq!(anglais.pieces[0].dessin.resoudre(FORMAT), solution);
    }

    #[test]
    fn chaque_mode_a_son_exemple_qui_s_ouvre_une_fois() {
        assert_eq!(exemple_du_mode(false).0, plan_simple());
        assert_eq!(exemple_du_mode(true).0, plan_d_exemple());
        assert_ne!(
            exemple_du_mode(false).1.traduit(),
            exemple_du_mode(true).1.traduit()
        );
        // Débutant par défaut; chaque exemple ne s'ajoute qu'une fois, dans
        // son mode.
        let mut prefs = Preferences::default();
        assert!(!prefs.dessin_avance);
        assert!(!exemple_deja_ouvert(&prefs, false) && !exemple_deja_ouvert(&prefs, true));
        marquer_l_exemple_ouvert(&mut prefs, false);
        assert!(exemple_deja_ouvert(&prefs, false) && !exemple_deja_ouvert(&prefs, true));
        marquer_l_exemple_ouvert(&mut prefs, true);
        assert!(exemple_deja_ouvert(&prefs, true));
    }

    #[test]
    fn ses_limites_sont_vraies() {
        // Tout le plan ne tient pas dans un code QR; chaque pièce seule, oui.
        let plan = plan_d_exemple();
        let au_plus = noyau::qr::capacite(noyau::qr::Correction::L);
        assert!(crate::partage::adresse_du_plan(&plan).len() > au_plus);
        for rang in 0..plan.pieces.len() {
            let adresse = crate::partage::adresse_de_la_piece(&plan, rang);
            assert!(
                adresse.len() <= au_plus,
                "{} : {}",
                plan.pieces[rang].nom,
                adresse.len()
            );
        }
    }
}
