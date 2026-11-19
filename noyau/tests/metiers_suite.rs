//! Calculateurs par métier ajoutés à la version 0.4 : valeurs d'essai de la
//! section 22 du cahier des charges. Les résultats attendus ont été calculés à
//! part, hors du programme.

use noyau::Preferences;
use noyau::metiers::calculateur;

/// Remplit un calculateur et compare ses résultats, libellés compris.
/// Les espaces insécables des résultats sont écrites ici comme des espaces.
fn verifier(id: &str, saisies: &[&str], attendus: &[(&str, &str)]) {
    let c = calculateur(id).unwrap_or_else(|| panic!("calculateur inconnu : {id}"));
    let textes: Vec<String> = saisies.iter().map(|s| (*s).to_string()).collect();
    let e = c.evaluer(&textes, &Preferences::default());
    assert!(
        e.erreurs.iter().all(Option::is_none),
        "{id} {saisies:?} : {:?}",
        e.erreurs
    );
    assert_eq!(e.avis, None, "{id} {saisies:?}");
    let obtenus: Vec<(String, String)> = e
        .resultats
        .iter()
        .map(|(libelle, valeur)| ((*libelle).to_string(), valeur.replace('\u{a0}', " ")))
        .collect();
    let attendus: Vec<(String, String)> = attendus
        .iter()
        .map(|(libelle, valeur)| ((*libelle).to_string(), (*valeur).to_string()))
        .collect();
    assert_eq!(obtenus, attendus, "{id} {saisies:?}");
}

/// Le calculateur doit refuser ces valeurs avec ce message.
fn refuser(id: &str, saisies: &[&str], avis: &str) {
    let c = calculateur(id).unwrap_or_else(|| panic!("calculateur inconnu : {id}"));
    let textes: Vec<String> = saisies.iter().map(|s| (*s).to_string()).collect();
    let e = c.evaluer(&textes, &Preferences::default());
    assert_eq!(e.avis, Some(avis), "{id} {saisies:?}");
    assert!(e.resultats.is_empty());
}

// ----------------------------------------------------------------------
// Électricité (22.6)
// ----------------------------------------------------------------------

#[test]
fn calibre_selon_la_chute_de_tension() {
    // Champs : tension, courant, longueur, chute visée, métal, circuit.
    verifier(
        "calibre-pour-chute",
        &["120", "20", "100", "3", "0", "0"],
        &[
            ("Calibre suffisant", "8 AWG"),
            ("Section minimale", "14,3 kcmil"),
            ("Chute avec ce calibre", "3,13 V"),
            ("Chute, en pourcentage", "2,60 %"),
        ],
    );
    // Aluminium, en triphasé.
    verifier(
        "calibre-pour-chute",
        &["600", "100", "300", "2", "1", "1"],
        &[
            ("Calibre suffisant", "1/0 AWG"),
            ("Section minimale", "91,8 kcmil"),
            ("Chute avec ce calibre", "10,4 V"),
            ("Chute, en pourcentage", "1,74 %"),
        ],
    );
    refuser(
        "calibre-pour-chute",
        &["120", "200", "1000", "1", "0", "0"],
        "Aucun calibre de la liste ne suffit : il faut plus gros que 500 kcmil, ou des conducteurs en parallèle.",
    );
    refuser(
        "calibre-pour-chute",
        &["120", "20", "100", "100", "0", "0"],
        "La chute visée doit être inférieure à 100 %.",
    );
}

#[test]
fn resistances_transformateur_et_moteur() {
    verifier(
        "resistances",
        &["100", "220", "470", ""],
        &[("En série", "790 Ω"), ("En parallèle", "60,0 Ω")],
    );
    verifier(
        "resistances",
        &["8", "8", "", ""],
        &[("En série", "16,0 Ω"), ("En parallèle", "4,00 Ω")],
    );
    // 75 kVA, 600 V à 208 V, en triphasé.
    verifier(
        "transformateur",
        &["75", "600", "208", "1"],
        &[
            ("Courant au primaire", "72,2 A"),
            ("Courant au secondaire", "208 A"),
            ("Rapport de transformation", "2,88"),
        ],
    );
    verifier(
        "transformateur",
        &["25", "240", "120", "0"],
        &[
            ("Courant au primaire", "104 A"),
            ("Courant au secondaire", "208 A"),
            ("Rapport de transformation", "2,00"),
        ],
    );
    // 10 HP à 575 V en triphasé, rendement de 90 %, facteur de puissance de 0,85.
    verifier(
        "moteur",
        &["10", "575", "90", "0,85", "1"],
        &[
            ("Puissance à l'arbre", "7,46 kW"),
            ("Puissance absorbée", "8,29 kW"),
            ("Courant à pleine charge", "9,79 A"),
        ],
    );
    verifier(
        "moteur",
        &["1", "120", "75", "0,8", "0"],
        &[
            ("Puissance à l'arbre", "0,746 kW"),
            ("Puissance absorbée", "0,995 kW"),
            ("Courant à pleine charge", "10,4 A"),
        ],
    );
    refuser(
        "moteur",
        &["10", "575", "110", "0,85", "1"],
        "Le rendement ne dépasse pas 100 %.",
    );
    refuser(
        "moteur",
        &["10", "575", "90", "1,2", "1"],
        "Le facteur de puissance se situe entre 0 et 1.",
    );
}

#[test]
fn remplissage_de_conduit_et_eclairage() {
    // Conduit de 0,824 po de diamètre intérieur, neuf conducteurs de 0,111 po.
    verifier(
        "remplissage-de-conduit",
        &["0,824", "0,111", "9"],
        &[
            ("Section du conduit", "0,533 po² (344 mm²)"),
            ("Section des conducteurs", "0,087 po² (56,2 mm²)"),
            ("Remplissage", "16,3 %"),
        ],
    );
    refuser(
        "remplissage-de-conduit",
        &["1/2", "1/4", "5"],
        "Les conducteurs n'entrent pas dans ce conduit.",
    );
    // Pièce de 30 pi sur 20 pi, 300 lux, luminaires de 4 000 lm.
    verifier(
        "eclairage",
        &["30", "20", "300", "4000", "0,6", "0,8"],
        &[
            ("Surface", "600 pi² (55,7 m²)"),
            ("Flux lumineux nécessaire", "34 839 lm"),
            ("Luminaires", "9 luminaires"),
        ],
    );
    refuser(
        "eclairage",
        &["30", "20", "300", "4000", "1,2", "0,8"],
        "Les deux facteurs se situent entre 0 et 1.",
    );
}

// ----------------------------------------------------------------------
// Plomberie (22.5)
// ----------------------------------------------------------------------

#[test]
fn debit_coupe_et_deports_paralleles() {
    // 8 gal US/min dans un tuyau de 0,785 po de diamètre intérieur.
    verifier(
        "debit-et-vitesse",
        &["0,785", "8", "0"],
        &[
            ("Vitesse de l'eau", "5,30 pi/s (1,62 m/s)"),
            ("Débit", "8,00 gal US/min (30,3 L/min)"),
        ],
    );
    // 20 L/min dans un tuyau de 1 po.
    verifier(
        "debit-et-vitesse",
        &["1", "20", "1"],
        &[
            ("Vitesse de l'eau", "2,16 pi/s (0,658 m/s)"),
            ("Débit", "5,28 gal US/min (20,0 L/min)"),
        ],
    );
    // 0,5 L/s dans un tuyau de 3/4 po.
    verifier(
        "debit-et-vitesse",
        &["3/4", "0,5", "2"],
        &[
            ("Vitesse de l'eau", "5,76 pi/s (1,75 m/s)"),
            ("Débit", "7,93 gal US/min (30,0 L/min)"),
        ],
    );
    // Champs : centre à centre, retrait des deux raccords, emboîtement.
    verifier(
        "longueur-de-coupe",
        &["48", "1 1/8", "1 1/8", "3/4"],
        &[("Longueur de coupe", "3 pi 11 1/4 po")],
    );
    verifier(
        "longueur-de-coupe",
        &["30 1/2", "1 3/4", "2 1/4", "0"],
        &[("Longueur de coupe", "2 pi 2 1/2 po")],
    );
    refuser(
        "longueur-de-coupe",
        &["2", "1 1/8", "1 1/8", "0"],
        "Les retraits dépassent la mesure de centre à centre.",
    );
    // Déport de 12 po, tuyaux espacés de 6 po, raccords à 45°.
    verifier(
        "deports-paralleles",
        &["12", "6", "3"],
        &[
            ("Longueur de parcours", "1 pi 5 po"),
            ("Avance", "1 pi 0 po"),
            ("Décalage d'un tuyau au suivant", "2 1/2 po"),
        ],
    );
    verifier(
        "deports-paralleles",
        &["10", "8", "1"],
        &[
            ("Longueur de parcours", "2 pi 2 1/8 po"),
            ("Avance", "2 pi 0 1/8 po"),
            ("Décalage d'un tuyau au suivant", "1 9/16 po"),
        ],
    );
}

#[test]
fn dilatation_et_chauffage_de_l_eau() {
    // 50 pi de cuivre qui gagnent 60 °C.
    verifier(
        "dilatation",
        &["50", "60", "0,017"],
        &[
            ("Allongement", "5/8 po"),
            ("Allongement, en millimètres", "15,5 mm"),
        ],
    );
    // 30 m de PVC : le résultat suit l'unité de la saisie.
    verifier(
        "dilatation",
        &["30 m", "50", "0,054"],
        &[
            ("Allongement", "0,081 m"),
            ("Allongement, en millimètres", "81,0 mm"),
        ],
    );
    // 180 L à réchauffer de 45 °C avec 4,5 kW.
    verifier(
        "chauffe-eau",
        &["180", "45", "4,5", "100"],
        &[
            ("Énergie nécessaire", "9,42 kWh"),
            ("Temps de chauffe", "126 min"),
            ("Temps de chauffe, en heures", "2,09 h"),
        ],
    );
    verifier(
        "chauffe-eau",
        &["270", "50", "11", "80"],
        &[
            ("Énergie nécessaire", "15,7 kWh"),
            ("Temps de chauffe", "107 min"),
            ("Temps de chauffe, en heures", "1,78 h"),
        ],
    );
    refuser(
        "chauffe-eau",
        &["180", "45", "4,5", "120"],
        "Le rendement ne dépasse pas 100 %.",
    );
}

// ----------------------------------------------------------------------
// Béton et fondations (22.4)
// ----------------------------------------------------------------------

#[test]
fn sacs_escalier_et_treillis() {
    // Dalle de 4 pi sur 4 pi, épaisse de 4 po.
    verifier(
        "sacs-de-beton",
        &["4", "4", "4", "10", "14"],
        &[
            ("Volume de béton", "0,198 vg³ (0,151 m³)"),
            ("Volume à préparer, pertes comprises", "166 L"),
            ("Sacs de béton prémélangé", "12 sacs"),
        ],
    );
    // Cinq marches de 7 po sur 11 po, larges de 4 pi, sur une paillasse de 5 po.
    verifier(
        "escalier-de-beton",
        &["5", "7", "11", "4", "5", "10"],
        &[
            ("Volume de béton", "0,533 vg³ (0,408 m³)"),
            ("À commander, pertes comprises", "0,587 vg³ (0,449 m³)"),
        ],
    );
    // Sans paillasse : les marches seules.
    verifier(
        "escalier-de-beton",
        &["3", "7 1/2", "10", "3", "0", "0"],
        &[
            ("Volume de béton", "0,087 vg³ (0,066 m³)"),
            ("À commander, pertes comprises", "0,087 vg³ (0,066 m³)"),
        ],
    );
    verifier(
        "treillis",
        &["24", "20", "8", "4", "6"],
        &[
            ("Surface", "480 pi² (44,6 m²)"),
            ("Feuilles de treillis", "19 feuilles"),
        ],
    );
    refuser(
        "treillis",
        &["24", "20", "8", "4", "48"],
        "Le chevauchement dépasse la feuille.",
    );
}

// ----------------------------------------------------------------------
// Finition (22.7)
// ----------------------------------------------------------------------

#[test]
fn peinture_d_une_surface_et_moulures() {
    verifier(
        "peinture-d-une-surface",
        &["40", "8", "2", "350"],
        &[
            ("Surface à peindre", "320 pi² (29,7 m²)"),
            ("Peinture nécessaire", "6,91 L"),
            ("Contenants de 3,78 L", "2 contenants"),
        ],
    );
    // Pièce de 12 pi sur 10 pi, une porte de 32 po, moulures de 8 pi.
    verifier(
        "moulures",
        &["12", "10", "1", "32", "10", "8"],
        &[
            ("Tour de la pièce", "44 pi 0 po"),
            ("Longueur à poser", "41 pi 4 po"),
            ("Longueur à acheter, pertes comprises", "45 pi 5 5/8 po"),
            ("Moulures", "6 moulures"),
        ],
    );
    refuser(
        "moulures",
        &["3", "2", "4", "32", "10", "8"],
        "Les portes couvrent tout le tour de la pièce.",
    );
}

// ----------------------------------------------------------------------
// Gypse et isolation (22.12)
// ----------------------------------------------------------------------

#[test]
fn gypse_et_cloisons() {
    // Champs : longueur, largeur, hauteur, portes, fenêtres, surfaces, panneau, pertes.
    verifier(
        "gypse",
        &["12", "10", "8", "1", "1", "1", "0", "10"],
        &[
            ("Surface à couvrir", "436 pi² (40,5 m²)"),
            ("Panneaux", "15 panneaux"),
        ],
    );
    // Les murs seulement, en panneaux de 12 pi.
    verifier(
        "gypse",
        &["12", "10", "8", "1", "1", "0", "2", "10"],
        &[
            ("Surface à couvrir", "316 pi² (29,4 m²)"),
            ("Panneaux", "8 panneaux"),
        ],
    );
    refuser(
        "gypse",
        &["4", "3", "8", "6", "0", "0", "0", "10"],
        "Les ouvertures couvrent toute la surface des murs.",
    );
    // 12 pi aux 16 po : neuf espacements, donc dix montants; trois lisses.
    verifier(
        "montants-de-cloison",
        &["12", "16", "0", "1"],
        &[
            ("Montants", "10 montants"),
            ("Longueur de lisses", "36 pi 0 po"),
        ],
    );
    verifier(
        "montants-de-cloison",
        &["12 pi 6 po", "16", "2", "0"],
        &[
            ("Montants", "13 montants"),
            ("Longueur de lisses", "25 pi 0 po"),
        ],
    );
}

#[test]
fn isolant_valeur_r_et_plafond() {
    verifier(
        "isolant",
        &["40", "8", "5", "78,3"],
        &[
            ("Surface", "320 pi² (29,7 m²)"),
            ("Surface à couvrir, pertes comprises", "336 pi² (31,2 m²)"),
            ("Paquets", "5 paquets"),
        ],
    );
    verifier(
        "valeur-r",
        &["24,5", "0"],
        &[
            ("Valeur R", "24,5"),
            ("Valeur RSI", "4,31"),
            ("Coefficient U", "0,232 W/m²·K"),
        ],
    );
    verifier(
        "valeur-r",
        &["3,52", "1"],
        &[
            ("Valeur R", "20,0"),
            ("Valeur RSI", "3,52"),
            ("Coefficient U", "0,284 W/m²·K"),
        ],
    );
    verifier(
        "plafond-suspendu",
        &["20", "15", "0", "10"],
        &[
            ("Surface", "300 pi² (27,9 m²)"),
            ("Tuiles", "42 tuiles"),
            ("Cornière murale", "70 pi 0 po"),
        ],
    );
    verifier(
        "plafond-suspendu",
        &["20", "15", "1", "5"],
        &[
            ("Surface", "300 pi² (27,9 m²)"),
            ("Tuiles", "79 tuiles"),
            ("Cornière murale", "70 pi 0 po"),
        ],
    );
}

// ----------------------------------------------------------------------
// Toiture et revêtement (22.11)
// ----------------------------------------------------------------------

#[test]
fn bardeaux_et_pente() {
    // Toit de 40 pi sur 28 pi au sol, pente de 6 sur 12.
    verifier(
        "bardeaux",
        &["40", "28", "6", "10", "33,3"],
        &[
            ("Surface du toit", "1 252 pi² (116 m²)"),
            ("Carrés de 100 pi², pertes comprises", "13,8 carrés"),
            ("Paquets de bardeaux", "42 paquets"),
        ],
    );
    // Pente nulle et aucune perte : la surface au sol.
    verifier(
        "bardeaux",
        &["40", "28", "0", "0", "33,3"],
        &[
            ("Surface du toit", "1 120 pi² (104 m²)"),
            ("Carrés de 100 pi², pertes comprises", "11,2 carrés"),
            ("Paquets de bardeaux", "34 paquets"),
        ],
    );
    // Unité : rang dans la liste « sur 12 » · « degrés » · « pour cent ».
    verifier(
        "pente-de-toit",
        &["6", "0"],
        &[
            ("Pente, sur 12", "6,00 sur 12"),
            ("Angle", "26,57°"),
            ("Pente, en pourcentage", "50,0 %"),
            ("Facteur de pente", "1,1180"),
        ],
    );
    verifier(
        "pente-de-toit",
        &["30", "1"],
        &[
            ("Pente, sur 12", "6,93 sur 12"),
            ("Angle", "30,00°"),
            ("Pente, en pourcentage", "57,7 %"),
            ("Facteur de pente", "1,1547"),
        ],
    );
    verifier(
        "pente-de-toit",
        &["75", "2"],
        &[
            ("Pente, sur 12", "9,00 sur 12"),
            ("Angle", "36,87°"),
            ("Pente, en pourcentage", "75,0 %"),
            ("Facteur de pente", "1,2500"),
        ],
    );
    refuser(
        "pente-de-toit",
        &["90", "1"],
        "L'angle doit être inférieur à 90°.",
    );
}

#[test]
fn panneaux_revetement_et_membrane() {
    verifier(
        "panneaux",
        &["24", "16", "0", "10"],
        &[
            ("Surface", "384 pi² (35,7 m²)"),
            ("Panneaux", "14 panneaux"),
        ],
    );
    // 120 pi de murs hauts de 9 pi, moins 150 pi² d'ouvertures.
    verifier(
        "revetement-exterieur",
        &["120", "9", "150", "10", "200"],
        &[
            ("Surface à couvrir", "930 pi² (86,4 m²)"),
            ("Surface à acheter, pertes comprises", "1 023 pi² (95,0 m²)"),
            ("Boîtes", "6 boîtes"),
        ],
    );
    refuser(
        "revetement-exterieur",
        &["10", "8", "100", "10", "200"],
        "Les ouvertures couvrent toute la surface des murs.",
    );
    // Rouleaux de 36 po sur 65 pi, posés avec 4 po de chevauchement.
    verifier(
        "membrane",
        &["40", "14", "36", "65", "4", "10"],
        &[("Surface", "560 pi² (52,0 m²)"), ("Rouleaux", "4 rouleaux")],
    );
    refuser(
        "membrane",
        &["40", "14", "36", "65", "36", "10"],
        "Le chevauchement dépasse la largeur du rouleau.",
    );
}

// ----------------------------------------------------------------------
// Maçonnerie et pavage (22.10)
// ----------------------------------------------------------------------

#[test]
fn briques_blocs_et_rangs() {
    // Blocs de 15 5/8 po sur 7 5/8 po, joints de 3/8 po.
    verifier(
        "briques-et-blocs",
        &["22", "8", "0", "15 5/8", "7 5/8", "3/8", "5"],
        &[
            ("Surface du mur", "176 pi² (16,4 m²)"),
            ("Unités au pied carré", "1,125"),
            ("Unités nécessaires, pertes comprises", "208 unités"),
        ],
    );
    // Briques, avec 40 pi² d'ouvertures.
    verifier(
        "briques-et-blocs",
        &["24", "9", "40", "7 5/8", "2 1/4", "3/8", "5"],
        &[
            ("Surface du mur", "176 pi² (16,4 m²)"),
            ("Unités au pied carré", "6,857"),
            ("Unités nécessaires, pertes comprises", "1 268 unités"),
        ],
    );
    refuser(
        "briques-et-blocs",
        &["5", "4", "20", "15 5/8", "7 5/8", "3/8", "5"],
        "Les ouvertures couvrent toute la surface du mur.",
    );
    verifier(
        "rangs-de-maconnerie",
        &["8 pi 1 po", "7 5/8", "3/8"],
        &[
            ("Hauteur d'un rang, joint compris", "8 po"),
            ("Rangs", "13 rangs"),
            ("Hauteur obtenue", "8 pi 8 po"),
        ],
    );
    verifier(
        "rangs-de-maconnerie",
        &["8", "2 1/4", "3/8"],
        &[
            ("Hauteur d'un rang, joint compris", "2 5/8 po"),
            ("Rangs", "37 rangs"),
            ("Hauteur obtenue", "8 pi 1 1/8 po"),
        ],
    );
}

#[test]
fn muret_et_paves() {
    verifier(
        "muret",
        &["21 pi 6 po", "40 po", "12", "6"],
        &[
            ("Blocs par rang", "22 blocs"),
            ("Rangs", "7 rangs"),
            ("Blocs", "154 blocs"),
            ("Hauteur obtenue", "3 pi 6 po"),
        ],
    );
    // Pavés de 7 7/8 po sur 3 15/16 po, sur 1 po de sable.
    verifier(
        "paves",
        &["20", "12", "7 7/8", "3 15/16", "5", "1"],
        &[
            ("Surface", "240 pi² (22,3 m²)"),
            ("Pavés", "1 171 pavés"),
            ("Sable du lit de pose", "0,741 vg³ (0,566 m³)"),
        ],
    );
}

// ----------------------------------------------------------------------
// Acier et soudure (22.13)
// ----------------------------------------------------------------------

#[test]
fn masse_des_plaques_et_des_barres() {
    // Plaque d'acier de 4 pi sur 8 pi, épaisse de 1/4 po.
    verifier(
        "plaque",
        &["8", "4", "1/4", "0", "1"],
        &[
            ("Surface d'une face", "32,0 pi² (2,97 m²)"),
            ("Masse d'une pièce", "327 lb (148 kg)"),
            ("Masse totale", "327 lb (148 kg)"),
        ],
    );
    // Trois plaques d'aluminium.
    verifier(
        "plaque",
        &["8", "4", "1/4", "2", "3"],
        &[
            ("Surface d'une face", "32,0 pi² (2,97 m²)"),
            ("Masse d'une pièce", "112 lb (51,0 kg)"),
            ("Masse totale", "337 lb (153 kg)"),
        ],
    );
    // Barre pleine de 1 po, longue de 12 pi.
    verifier(
        "barre-ronde",
        &["1", "", "12", "0", "1"],
        &[
            ("Masse linéique", "2,67 lb/pi (3,98 kg/m)"),
            ("Masse d'une pièce", "32,1 lb (14,5 kg)"),
            ("Masse totale", "32,1 lb (14,5 kg)"),
        ],
    );
    // Deux tubes de 2 po, paroi de 1/8 po, longs de 20 pi.
    verifier(
        "barre-ronde",
        &["2", "1/8", "20", "0", "2"],
        &[
            ("Masse linéique", "2,51 lb/pi (3,73 kg/m)"),
            ("Masse d'une pièce", "50,1 lb (22,7 kg)"),
            ("Masse totale", "100 lb (45,5 kg)"),
        ],
    );
    refuser(
        "barre-ronde",
        &["1", "3/4", "12", "0", "1"],
        "La paroi dépasse la moitié du diamètre.",
    );
    // Tube carré de 2 po, paroi de 1/8 po.
    verifier(
        "profile-rectangulaire",
        &["2", "2", "1/8", "20", "0", "1"],
        &[
            ("Masse linéique", "3,19 lb/pi (4,75 kg/m)"),
            ("Masse d'une pièce", "63,8 lb (28,9 kg)"),
            ("Masse totale", "63,8 lb (28,9 kg)"),
        ],
    );
    // Quatre barres plates de 2 po sur 1/4 po.
    verifier(
        "profile-rectangulaire",
        &["2", "1/4", "", "10", "0", "4"],
        &[
            ("Masse linéique", "1,70 lb/pi (2,53 kg/m)"),
            ("Masse d'une pièce", "17,0 lb (7,72 kg)"),
            ("Masse totale", "68,1 lb (30,9 kg)"),
        ],
    );
    refuser(
        "profile-rectangulaire",
        &["2", "1", "3/4", "10", "0", "1"],
        "La paroi dépasse la moitié du plus petit côté.",
    );
    // 100 pi de cordon de 1/4 po.
    verifier(
        "cordon-de-soudure",
        &["1/4", "100", "65"],
        &[
            ("Métal déposé", "10,6 lb (4,82 kg)"),
            ("Électrodes ou fil à prévoir", "16,4 lb (7,42 kg)"),
        ],
    );
    refuser(
        "cordon-de-soudure",
        &["1/4", "100", "120"],
        "Le rendement ne dépasse pas 100 %.",
    );
}

// ----------------------------------------------------------------------
// Ventilation et chauffage (22.8)
// ----------------------------------------------------------------------

#[test]
fn conduit_changements_d_air_et_chaleur() {
    verifier(
        "conduit-pour-un-debit",
        &["400", "800"],
        &[
            ("Section nécessaire", "0,500 pi² (0,046 m²)"),
            ("Diamètre du conduit rond", "9 9/16 po"),
        ],
    );
    // Pièce de 12 pi sur 10 pi sur 8 pi, six changements d'air à l'heure.
    verifier(
        "changements-d-air",
        &["12", "10", "8", "6"],
        &[
            ("Volume de la pièce", "960 pi³ (27,2 m³)"),
            ("Débit", "96,0 pi³/min (45,3 L/s)"),
        ],
    );
    // 1 000 pi³/min, écart de 20 °F.
    verifier(
        "chaleur-de-l-air",
        &["1000", "20", "1"],
        &[("Puissance", "21 600 BTU/h (6,33 kW)")],
    );
    // Le même débit, écart de 11 °C.
    verifier(
        "chaleur-de-l-air",
        &["1000", "11", "0"],
        &[("Puissance", "21 384 BTU/h (6,27 kW)")],
    );
    verifier(
        "plancher-chauffant",
        &["30", "20", "9", "300"],
        &[
            ("Surface", "600 pi² (55,7 m²)"),
            ("Longueur de tuyau", "800 pi 0 po"),
            ("Boucles", "3 boucles"),
            ("Longueur par boucle", "266 pi 8 po"),
        ],
    );
}

// ----------------------------------------------------------------------
// Terrassement et aménagement (22.9)
// ----------------------------------------------------------------------

#[test]
fn tranchee_voyages_et_pente() {
    // Champs : longueur, largeur au fond, largeur en haut, profondeur, foisonnement.
    verifier(
        "tranchee",
        &["100", "2", "6", "4", "25"],
        &[
            ("Volume en place", "59,3 vg³ (45,3 m³)"),
            ("Volume à transporter, foisonné", "74,1 vg³ (56,6 m³)"),
        ],
    );
    // Parois verticales : la largeur en haut reste vide.
    verifier(
        "tranchee",
        &["100", "2", "", "4", "25"],
        &[
            ("Volume en place", "29,6 vg³ (22,7 m³)"),
            ("Volume à transporter, foisonné", "37,0 vg³ (28,3 m³)"),
        ],
    );
    refuser(
        "tranchee",
        &["100", "4", "2", "4", "25"],
        "La largeur en haut ne peut pas être plus petite que la largeur au fond.",
    );
    verifier(
        "voyages-de-camion",
        &["74", "12"],
        &[
            ("Voyages", "7 voyages"),
            ("Charge moyenne d'un voyage", "10,6 vg³"),
        ],
    );
    // 50 pi à 2 % : le dénivelé est calculé.
    verifier(
        "pente-de-terrain",
        &["50", "", "2"],
        &[
            ("Distance horizontale", "50 pi 0 po"),
            ("Dénivelé", "1 pi 0 po"),
            ("Pente", "2,00 %"),
            ("Angle", "1,15°"),
            ("Longueur en pente", "50 pi 0 1/8 po"),
        ],
    );
    // 50 pi et 9 po : la pente est calculée.
    verifier(
        "pente-de-terrain",
        &["50", "9", ""],
        &[
            ("Distance horizontale", "50 pi 0 po"),
            ("Dénivelé", "9 po"),
            ("Pente", "1,50 %"),
            ("Angle", "0,86°"),
            ("Longueur en pente", "50 pi 0 1/16 po"),
        ],
    );
    // 6 po à 1 % : la distance est calculée.
    verifier(
        "pente-de-terrain",
        &["", "6", "1"],
        &[
            ("Distance horizontale", "50 pi 0 po"),
            ("Dénivelé", "6 po"),
            ("Pente", "1,00 %"),
            ("Angle", "0,57°"),
            ("Longueur en pente", "50 pi 0 po"),
        ],
    );
    refuser(
        "pente-de-terrain",
        &["50", "", ""],
        "Remplissez exactement deux des trois champs.",
    );
    refuser(
        "pente-de-terrain",
        &["50", "9", "2"],
        "Remplissez exactement deux des trois champs.",
    );
}

#[test]
fn terrasse_tourbe_et_poteaux() {
    // Terrasse de 16 pi sur 12 pi, planches de 5 1/2 po, solives aux 16 po.
    verifier(
        "terrasse",
        &["16", "12", "5 1/2", "1/8", "16", "10"],
        &[
            ("Surface", "192 pi² (17,8 m²)"),
            ("Rangs de planches", "26 rangs"),
            ("Longueur de planches", "416 pi 0 po"),
            ("Longueur à acheter, pertes comprises", "457 pi 7 3/16 po"),
            ("Vis", "676 vis"),
        ],
    );
    verifier(
        "tourbe",
        &["50", "30", "10", "5"],
        &[
            ("Surface", "1 500 pi² (139 m²)"),
            ("Rouleaux", "158 rouleaux"),
        ],
    );
    // Douze poteaux carrés de 3 1/2 po dans des trous de 10 po, profonds de 4 pi.
    verifier(
        "poteaux",
        &["10", "4", "0", "3 1/2", "12", "14"],
        &[
            ("Volume de béton", "0,818 vg³ (0,626 m³)"),
            ("Volume, en litres", "626 L"),
            ("Sacs de béton prémélangé", "45 sacs"),
        ],
    );
    // Quatre trous de 12 po remplis de béton.
    verifier(
        "poteaux",
        &["12", "4", "1", "0", "4", "14"],
        &[
            ("Volume de béton", "0,465 vg³ (0,356 m³)"),
            ("Volume, en litres", "356 L"),
            ("Sacs de béton prémélangé", "26 sacs"),
        ],
    );
    refuser(
        "poteaux",
        &["4", "4", "0", "6", "1", "14"],
        "Le poteau n'entre pas dans le trou.",
    );
}

// ----------------------------------------------------------------------
// Estimation et soumission (22.3)
// ----------------------------------------------------------------------

#[test]
fn facture_d_etape_et_duree_des_travaux() {
    // Contrat de 100 000 $ avancé à 40 %, retenue de 10 %, 20 000 $ déjà facturés.
    verifier(
        "facturation-progressive",
        &["100000", "40", "10", "20000", "5", "9,975"],
        &[
            ("Travaux exécutés à ce jour", "40 000,00 $"),
            ("Retenue", "4 000,00 $"),
            ("Cette facture, avant taxes", "16 000,00 $"),
            ("TPS", "800,00 $"),
            ("TVQ", "1 596,00 $"),
            ("Cette facture, taxes comprises", "18 396,00 $"),
        ],
    );
    // Des montants qui ne tombent pas rond : chaque ligne est arrondie au cent.
    verifier(
        "facturation-progressive",
        &["87450,75", "65", "10", "30000", "5", "9,975"],
        &[
            ("Travaux exécutés à ce jour", "56 842,99 $"),
            ("Retenue", "5 684,30 $"),
            ("Cette facture, avant taxes", "21 158,69 $"),
            ("TPS", "1 057,93 $"),
            ("TVQ", "2 110,58 $"),
            ("Cette facture, taxes comprises", "24 327,20 $"),
        ],
    );
    refuser(
        "facturation-progressive",
        &["100000", "40", "10", "50000", "5", "9,975"],
        "Le montant déjà facturé dépasse les travaux exécutés, retenue déduite.",
    );
    refuser(
        "facturation-progressive",
        &["100000", "140", "10", "0", "5", "9,975"],
        "L'avancement ne dépasse pas 100 %.",
    );
    // 1 200 pi² à 50 pi² à l'heure par ouvrier, deux ouvriers, journées de 8 h.
    verifier(
        "duree-des-travaux",
        &["1200", "50", "2", "8"],
        &[
            ("Heures de travail, en tout", "24,0 h"),
            ("Durée, à cette équipe", "12,0 h"),
            ("Jours de travail", "1,5 jours"),
        ],
    );
    verifier(
        "duree-des-travaux",
        &["350", "12", "3", "7,5"],
        &[
            ("Heures de travail, en tout", "29,2 h"),
            ("Durée, à cette équipe", "9,7 h"),
            ("Jours de travail", "1,3 jours"),
        ],
    );
    refuser(
        "duree-des-travaux",
        &["1200", "50", "2", "30"],
        "Une journée compte 24 heures.",
    );
}
