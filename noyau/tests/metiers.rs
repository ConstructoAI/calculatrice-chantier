//! Calculateurs par métier : valeurs d'essai de la section 22 du cahier des
//! charges. Les résultats attendus ont été calculés à part, hors du programme.

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
    let c = calculateur(id).unwrap();
    let textes: Vec<String> = saisies.iter().map(|s| (*s).to_string()).collect();
    let e = c.evaluer(&textes, &Preferences::default());
    assert_eq!(e.avis, Some(avis), "{id} {saisies:?}");
    assert!(e.resultats.is_empty());
}

// ----------------------------------------------------------------------
// Estimation et soumission (22.3)
// ----------------------------------------------------------------------

#[test]
fn prix_d_une_soumission() {
    // Champs : coût de base, administration, imprévus, profit, TPS, TVQ.
    verifier(
        "soumission",
        &["100000", "3", "12", "15", "5", "9,975"],
        &[
            ("Administration", "3 000,00 $"),
            ("Imprévus", "12 000,00 $"),
            ("Profit", "15 000,00 $"),
            ("Prix avant taxes", "130 000,00 $"),
            ("TPS", "6 500,00 $"),
            ("TVQ", "12 967,50 $"),
            ("Prix, taxes comprises", "149 467,50 $"),
        ],
    );
    // Un coût qui ne tombe pas rond : chaque ligne est arrondie au cent.
    verifier(
        "soumission",
        &["251 126,40", "3", "12", "15", "5", "9,975"],
        &[
            ("Administration", "7 533,79 $"),
            ("Imprévus", "30 135,17 $"),
            ("Profit", "37 668,96 $"),
            ("Prix avant taxes", "326 464,32 $"),
            ("TPS", "16 323,22 $"),
            ("TVQ", "32 564,82 $"),
            ("Prix, taxes comprises", "375 352,36 $"),
        ],
    );
    // Rénovation : 15 % d'imprévus.
    verifier(
        "soumission",
        &["48500", "3", "15", "15", "5", "9,975"],
        &[
            ("Administration", "1 455,00 $"),
            ("Imprévus", "7 275,00 $"),
            ("Profit", "7 275,00 $"),
            ("Prix avant taxes", "64 505,00 $"),
            ("TPS", "3 225,25 $"),
            ("TVQ", "6 434,37 $"),
            ("Prix, taxes comprises", "74 164,62 $"),
        ],
    );
}

#[test]
fn budget_au_pied_carre() {
    // 1 400 pi² à 200 $/pi², plus un garage de 400 pi² à 100 $/pi².
    verifier(
        "budget-au-pied-carre",
        &["1400", "200", "400", "100", "3", "12", "15", "5", "9,975"],
        &[
            ("Coût de base", "320 000,00 $"),
            ("Administration, imprévus et profit", "96 000,00 $"),
            ("Prix avant taxes", "416 000,00 $"),
            ("TPS et TVQ", "62 296,00 $"),
            ("Prix, taxes comprises", "478 296,00 $"),
            ("Prix au pied carré, taxes comprises", "265,72 $/pi²"),
        ],
    );
    // Sans garage, les deux champs du garage restent vides.
    verifier(
        "budget-au-pied-carre",
        &["1000", "100", "", "", "0", "0", "0", "5", "9,975"],
        &[
            ("Coût de base", "100 000,00 $"),
            ("Administration, imprévus et profit", "0,00 $"),
            ("Prix avant taxes", "100 000,00 $"),
            ("TPS et TVQ", "14 975,00 $"),
            ("Prix, taxes comprises", "114 975,00 $"),
            ("Prix au pied carré, taxes comprises", "114,98 $/pi²"),
        ],
    );
    refuser(
        "budget-au-pied-carre",
        &["1400", "200", "400", "", "3", "12", "15", "5", "9,975"],
        "Pour le garage, donnez la superficie et le coût au pied carré.",
    );
}

#[test]
fn main_d_oeuvre_taxes_et_marge() {
    verifier(
        "main-d-oeuvre",
        &["3", "40", "45,50", "30"],
        &[
            ("Heures totales", "120,0 h"),
            ("Coût horaire, charges comprises", "59,15 $/h"),
            ("Coût de la main-d'œuvre", "7 098,00 $"),
        ],
    );
    verifier(
        "main-d-oeuvre",
        &["2", "37,5", "95", "0"],
        &[
            ("Heures totales", "75,0 h"),
            ("Coût horaire, charges comprises", "95,00 $/h"),
            ("Coût de la main-d'œuvre", "7 125,00 $"),
        ],
    );
    // Sens du montant : rang dans la liste « avant taxes » · « taxes comprises ».
    verifier(
        "taxes",
        &["1000", "0", "5", "9,975"],
        &[
            ("Montant avant taxes", "1 000,00 $"),
            ("TPS", "50,00 $"),
            ("TVQ", "99,75 $"),
            ("Total, taxes comprises", "1 149,75 $"),
        ],
    );
    verifier(
        "taxes",
        &["1149,75", "1", "5", "9,975"],
        &[
            ("Montant avant taxes", "1 000,00 $"),
            ("TPS", "50,00 $"),
            ("TVQ", "99,75 $"),
            ("Total, taxes comprises", "1 149,75 $"),
        ],
    );
    // Les trois lignes redonnent exactement le total saisi.
    verifier(
        "taxes",
        &["500", "1", "5", "9,975"],
        &[
            ("Montant avant taxes", "434,88 $"),
            ("TPS", "21,74 $"),
            ("TVQ", "43,38 $"),
            ("Total, taxes comprises", "500,00 $"),
        ],
    );
    verifier(
        "majoration-et-marge",
        &["800", "25"],
        &[
            ("Prix de vente", "1 000,00 $"),
            ("Profit", "200,00 $"),
            ("Marge sur le prix de vente", "20,00 %"),
        ],
    );
    verifier(
        "majoration-et-marge",
        &["1250", "40"],
        &[
            ("Prix de vente", "1 750,00 $"),
            ("Profit", "500,00 $"),
            ("Marge sur le prix de vente", "28,57 %"),
        ],
    );
}

// ----------------------------------------------------------------------
// Béton et fondations (22.4)
// ----------------------------------------------------------------------

#[test]
fn dalle() {
    verifier(
        "dalle",
        &["24", "20", "4", "10"],
        &[
            ("Surface", "480 pi² (44,6 m²)"),
            ("Volume de béton", "5,93 vg³ (4,53 m³)"),
            ("À commander, pertes comprises", "6,52 vg³ (4,98 m³)"),
        ],
    );
    // Les mêmes dimensions tapées autrement donnent la même chose.
    verifier(
        "dalle",
        &["24 pi", "20'", "4 po", "10"],
        &[
            ("Surface", "480 pi² (44,6 m²)"),
            ("Volume de béton", "5,93 vg³ (4,53 m³)"),
            ("À commander, pertes comprises", "6,52 vg³ (4,98 m³)"),
        ],
    );
}

#[test]
fn dalle_en_metrique() {
    // 7,3 m × 6 m × 100 mm : 43,8 m² et 4,38 m³. Le métrique passe en tête.
    verifier(
        "dalle",
        &["7,3 m", "6 m", "100 mm", "10"],
        &[
            ("Surface", "43,8 m² (471 pi²)"),
            ("Volume de béton", "4,38 m³ (5,73 vg³)"),
            ("À commander, pertes comprises", "4,82 m³ (6,30 vg³)"),
        ],
    );
}

#[test]
fn semelle_et_mur() {
    verifier(
        "semelle",
        &["120", "20", "8", "10"],
        &[
            ("Volume de béton", "4,94 vg³ (3,78 m³)"),
            ("À commander, pertes comprises", "5,43 vg³ (4,15 m³)"),
        ],
    );
    verifier(
        "mur-de-fondation",
        &["120", "8", "8", "10"],
        &[
            ("Volume de béton", "23,7 vg³ (18,1 m³)"),
            ("À commander, pertes comprises", "26,1 vg³ (19,9 m³)"),
            ("Coffrage, deux faces", "1 920 pi² (178 m²)"),
        ],
    );
}

#[test]
fn colonnes() {
    verifier(
        "colonnes",
        &["10", "4", "6", "10", "14"],
        &[
            ("Volume de béton", "0,485 vg³ (0,371 m³)"),
            ("À commander, pertes comprises", "0,533 vg³ (0,408 m³)"),
            ("Sacs de béton prémélangé", "30 sacs"),
        ],
    );
}

#[test]
fn armature() {
    verifier(
        "armature",
        &["24", "20", "16", "0"],
        &[
            ("Barres dans le sens de la longueur", "16 barres"),
            ("Barres dans le sens de la largeur", "19 barres"),
            ("Longueur totale de barres", "764 pi 0 po"),
            ("Masse d'acier", "403 lb (183 kg)"),
        ],
    );
    // La barre 15M pèse deux fois la 10M.
    verifier(
        "armature",
        &["24", "20", "16", "1"],
        &[
            ("Barres dans le sens de la longueur", "16 barres"),
            ("Barres dans le sens de la largeur", "19 barres"),
            ("Longueur totale de barres", "764 pi 0 po"),
            ("Masse d'acier", "806 lb (366 kg)"),
        ],
    );
}

// ----------------------------------------------------------------------
// Plomberie (22.5)
// ----------------------------------------------------------------------

#[test]
fn pente_de_drain() {
    verifier(
        "pente-de-drain",
        &["40", "1/4"],
        &[("Dénivelé", "10 po"), ("Pente", "2,08 %")],
    );
    // Longueur en mètres : le dénivelé sort en mètres.
    verifier(
        "pente-de-drain",
        &["12 m", "1/4"],
        &[("Dénivelé", "0,250 m"), ("Pente", "2,08 %")],
    );
}

#[test]
fn deports() {
    // Angle : rang dans la liste 11,25° · 22,5° · 30° · 45° · 60°.
    verifier(
        "deport",
        &["12", "3"],
        &[
            ("Longueur de parcours", "1 pi 5 po"),
            ("Avance", "1 pi 0 po"),
        ],
    );
    verifier(
        "deport",
        &["12", "1"],
        &[
            ("Longueur de parcours", "2 pi 7 3/8 po"),
            ("Avance", "2 pi 5 po"),
        ],
    );
    verifier(
        "deport",
        &["10", "4"],
        &[
            ("Longueur de parcours", "11 9/16 po"),
            ("Avance", "5 3/4 po"),
        ],
    );
    verifier(
        "deport",
        &["12", "0"],
        &[
            ("Longueur de parcours", "5 pi 1 1/2 po"),
            ("Avance", "5 pi 0 5/16 po"),
        ],
    );
    verifier(
        "deport",
        &["12", "2"],
        &[
            ("Longueur de parcours", "2 pi 0 po"),
            ("Avance", "1 pi 8 13/16 po"),
        ],
    );
}

#[test]
fn deport_roule() {
    verifier(
        "deport-roule",
        &["8", "6", "3"],
        &[
            ("Déport réel", "10 po"),
            ("Longueur de parcours", "1 pi 2 1/8 po"),
            ("Avance", "10 po"),
        ],
    );
    verifier(
        "deport-roule",
        &["15", "20", "1"],
        &[
            ("Déport réel", "2 pi 1 po"),
            ("Longueur de parcours", "5 pi 5 5/16 po"),
            ("Avance", "5 pi 0 3/8 po"),
        ],
    );
}

#[test]
fn contenances_et_pression() {
    verifier(
        "contenance-de-tuyau",
        &["4", "10"],
        &[
            ("Contenance", "24,7 L (6,53 gal US)"),
            ("Masse de l'eau", "54,5 lb (24,7 kg)"),
        ],
    );
    verifier(
        "pression-d-eau",
        &["30"],
        &[("Pression", "89,7 kPa (13,0 lb/po²)")],
    );
    verifier(
        "pression-d-eau",
        &["10 m"],
        &[("Pression", "98,1 kPa (14,2 lb/po²)")],
    );
    verifier(
        "reservoir",
        &["20", "48"],
        &[("Contenance", "247 L (65,3 gal US)")],
    );
}

// ----------------------------------------------------------------------
// Électricité (22.6)
// ----------------------------------------------------------------------

#[test]
fn loi_d_ohm() {
    let complet = [
        ("Tension", "120 V"),
        ("Courant", "15,0 A"),
        ("Résistance", "8,00 Ω"),
        ("Puissance", "1 800 W"),
    ];
    // Les six paires possibles mènent aux mêmes quatre grandeurs.
    verifier("loi-d-ohm", &["120", "15", "", ""], &complet);
    verifier("loi-d-ohm", &["120", "", "8", ""], &complet);
    verifier("loi-d-ohm", &["120", "", "", "1800"], &complet);
    verifier("loi-d-ohm", &["", "15", "8", ""], &complet);
    verifier("loi-d-ohm", &["", "15", "", "1800"], &complet);
    verifier("loi-d-ohm", &["", "", "8", "1800"], &complet);
    verifier(
        "loi-d-ohm",
        &["", "10", "12", ""],
        &[
            ("Tension", "120 V"),
            ("Courant", "10,0 A"),
            ("Résistance", "12,0 Ω"),
            ("Puissance", "1 200 W"),
        ],
    );
    let avis = "Remplissez exactement deux des quatre champs.";
    refuser("loi-d-ohm", &["", "", "", ""], avis);
    refuser("loi-d-ohm", &["120", "", "", ""], avis);
    refuser("loi-d-ohm", &["120", "15", "8", ""], avis);
}

#[test]
fn chute_de_tension() {
    // Champs : tension, courant, longueur, calibre, métal, circuit.
    verifier(
        "chute-de-tension",
        &["120", "15", "100", "0", "0", "0"],
        &[
            ("Chute de tension", "9,42 V"),
            ("Chute, en pourcentage", "7,85 %"),
            ("Tension à la charge", "110,6 V"),
        ],
    );
    // Un calibre plus gros réduit la chute.
    verifier(
        "chute-de-tension",
        &["120", "15", "100", "1", "0", "0"],
        &[
            ("Chute de tension", "5,93 V"),
            ("Chute, en pourcentage", "4,94 %"),
            ("Tension à la charge", "114,1 V"),
        ],
    );
    verifier(
        "chute-de-tension",
        &["240", "40", "150", "3", "0", "0"],
        &[
            ("Chute de tension", "9,38 V"),
            ("Chute, en pourcentage", "3,91 %"),
            ("Tension à la charge", "230,6 V"),
        ],
    );
    // Aluminium 1/0 en triphasé.
    verifier(
        "chute-de-tension",
        &["600", "100", "300", "9", "1", "1"],
        &[
            ("Chute de tension", "10,4 V"),
            ("Chute, en pourcentage", "1,74 %"),
            ("Tension à la charge", "589,6 V"),
        ],
    );
    // Longueur en mètres.
    verifier(
        "chute-de-tension",
        &["240", "30", "30 m", "2", "0", "0"],
        &[
            ("Chute de tension", "7,34 V"),
            ("Chute, en pourcentage", "3,06 %"),
            ("Tension à la charge", "232,7 V"),
        ],
    );
    refuser(
        "chute-de-tension",
        &["12", "200", "1000", "0", "0", "0"],
        "La chute dépasse la tension du circuit : vérifiez les valeurs.",
    );
}

#[test]
fn puissance() {
    verifier(
        "puissance",
        &["240", "30", "1", "0"],
        &[
            ("Puissance apparente", "7,20 kVA"),
            ("Puissance active", "7,20 kW"),
            ("Équivalent en HP", "9,65 HP"),
        ],
    );
    verifier(
        "puissance",
        &["600", "20", "0,85", "1"],
        &[
            ("Puissance apparente", "20,8 kVA"),
            ("Puissance active", "17,7 kW"),
            ("Équivalent en HP", "23,7 HP"),
        ],
    );
    refuser(
        "puissance",
        &["240", "30", "1,2", "0"],
        "Le facteur de puissance se situe entre 0 et 1.",
    );
}

#[test]
fn cintrage() {
    // Angle : rang dans la liste 10° · 22,5° · 30° · 45° · 60°.
    verifier(
        "cintrage",
        &["6", "2"],
        &[
            ("Distance entre les marques", "1 pi 0 po"),
            ("Raccourcissement", "1 5/8 po"),
        ],
    );
    verifier(
        "cintrage",
        &["10", "1"],
        &[
            ("Distance entre les marques", "2 pi 2 1/8 po"),
            ("Raccourcissement", "2 po"),
        ],
    );
    verifier(
        "cintrage",
        &["3", "0"],
        &[
            ("Distance entre les marques", "1 pi 5 1/4 po"),
            ("Raccourcissement", "1/4 po"),
        ],
    );
    verifier(
        "cintrage",
        &["8", "4"],
        &[
            ("Distance entre les marques", "9 1/4 po"),
            ("Raccourcissement", "4 5/8 po"),
        ],
    );
    // En millimètres : 150 mm à 30° donnent 300 mm entre les marques.
    verifier(
        "cintrage",
        &["150 mm", "2"],
        &[
            ("Distance entre les marques", "300 mm"),
            ("Raccourcissement", "40 mm"),
        ],
    );
}

#[test]
fn cout_d_energie() {
    verifier(
        "cout-d-energie",
        &["1500", "8", "30", "10"],
        &[("Énergie", "360 kWh"), ("Coût", "36,00 $")],
    );
    verifier(
        "cout-d-energie",
        &["60", "24", "365", "8,5"],
        &[("Énergie", "526 kWh"), ("Coût", "44,68 $")],
    );
    refuser(
        "cout-d-energie",
        &["60", "25", "30", "10"],
        "Une journée compte 24 heures.",
    );
}

// ----------------------------------------------------------------------
// Finition (22.7)
// ----------------------------------------------------------------------

#[test]
fn peinture() {
    // Champs : longueur, largeur, hauteur, portes, fenêtres, surfaces, couches, rendement.
    verifier(
        "peinture",
        &["12", "10", "8", "1", "1", "0", "2", "350"],
        &[
            ("Surface à peindre", "316 pi² (29,4 m²)"),
            ("Peinture nécessaire", "6,83 L"),
            ("Contenants de 3,78 L", "2 contenants"),
        ],
    );
    verifier(
        "peinture",
        &["12", "10", "8", "1", "1", "1", "2", "350"],
        &[
            ("Surface à peindre", "436 pi² (40,5 m²)"),
            ("Peinture nécessaire", "9,42 L"),
            ("Contenants de 3,78 L", "3 contenants"),
        ],
    );
    refuser(
        "peinture",
        &["4", "4", "2", "2", "2", "0", "1", "350"],
        "Les ouvertures couvrent toute la surface des murs.",
    );
}

#[test]
fn ceramique_et_revetement() {
    verifier(
        "ceramique",
        &["10", "8", "24", "12", "10", "8"],
        &[
            ("Surface", "80,0 pi² (7,43 m²)"),
            ("Tuiles nécessaires", "44 tuiles"),
            ("Boîtes", "6 boîtes"),
        ],
    );
    // Sans le nombre de tuiles par boîte, la ligne des boîtes disparaît.
    verifier(
        "ceramique",
        &["12,5", "9,25", "13", "13", "10", ""],
        &[
            ("Surface", "116 pi² (10,7 m²)"),
            ("Tuiles nécessaires", "109 tuiles"),
        ],
    );
    verifier(
        "revetement-de-sol",
        &["15", "12", "10", "22,5"],
        &[
            ("Surface", "180 pi² (16,7 m²)"),
            ("Surface à couvrir, pertes comprises", "198 pi² (18,4 m²)"),
            ("Boîtes", "9 boîtes"),
        ],
    );
}

// ----------------------------------------------------------------------
// Ventilation et chauffage (22.8)
// ----------------------------------------------------------------------

#[test]
fn conduits() {
    verifier(
        "conduit-rond",
        &["8", "700"],
        &[
            ("Section", "0,349 pi² (0,032 m²)"),
            ("Débit", "244 pi³/min (115 L/s)"),
        ],
    );
    verifier(
        "conduit-rectangulaire",
        &["14", "8", "900"],
        &[
            ("Section", "0,778 pi² (0,072 m²)"),
            ("Débit", "700 pi³/min (330 L/s)"),
            ("Conduit rond équivalent", "11 7/16 po"),
        ],
    );
    verifier(
        "conduit-rectangulaire",
        &["20", "10", "1000"],
        &[
            ("Section", "1,39 pi² (0,129 m²)"),
            ("Débit", "1 389 pi³/min (655 L/s)"),
            ("Conduit rond équivalent", "1 pi 3 1/4 po"),
        ],
    );
}

#[test]
fn puissance_thermique() {
    // Unité : rang dans la liste BTU/h · kW · tonnes de réfrigération.
    verifier(
        "puissance-thermique",
        &["36000", "0"],
        &[
            ("En BTU par heure", "36 000 BTU/h"),
            ("En kilowatts", "10,6 kW"),
            ("En tonnes de réfrigération", "3,00 tonnes"),
        ],
    );
    verifier(
        "puissance-thermique",
        &["5", "1"],
        &[
            ("En BTU par heure", "17 061 BTU/h"),
            ("En kilowatts", "5,00 kW"),
            ("En tonnes de réfrigération", "1,42 tonnes"),
        ],
    );
    verifier(
        "puissance-thermique",
        &["2", "2"],
        &[
            ("En BTU par heure", "24 000 BTU/h"),
            ("En kilowatts", "7,03 kW"),
            ("En tonnes de réfrigération", "2,00 tonnes"),
        ],
    );
}

// ----------------------------------------------------------------------
// Terrassement et aménagement (22.9)
// ----------------------------------------------------------------------

#[test]
fn excavation_et_granulats() {
    verifier(
        "excavation",
        &["30", "20", "4", "25"],
        &[
            ("Volume en place", "88,9 vg³ (68,0 m³)"),
            ("Volume à transporter, foisonné", "111 vg³ (85,0 m³)"),
        ],
    );
    verifier(
        "granulats",
        &["40", "12", "6", "1,5"],
        &[
            ("Volume", "8,89 vg³ (6,80 m³)"),
            ("Masse", "13,3 tc (12,1 t)"),
        ],
    );
}

#[test]
fn cloture() {
    verifier(
        "cloture",
        &["100", "8"],
        &[
            ("Sections", "13 sections"),
            ("Poteaux", "14 poteaux"),
            ("Espacement réel", "7 pi 8 5/16 po"),
        ],
    );
    // Une longueur qui tombe juste ne crée pas de section en trop.
    verifier(
        "cloture",
        &["96", "8"],
        &[
            ("Sections", "12 sections"),
            ("Poteaux", "13 poteaux"),
            ("Espacement réel", "8 pi 0 po"),
        ],
    );
}

// ----------------------------------------------------------------------
// Règles communes (22.2)
// ----------------------------------------------------------------------

#[test]
fn saisies_remises_au_propre_pour_le_ruban() {
    let c = calculateur("chute-de-tension").unwrap();
    let saisies: Vec<String> = ["120", "15", "100", "1", "0", "1"]
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    let e = c.evaluer(&saisies, &Preferences::default());
    let entrees: Vec<(String, String)> = e
        .entrees
        .iter()
        .map(|(libelle, valeur)| ((*libelle).to_string(), valeur.replace('\u{a0}', " ")))
        .collect();
    let attendues = [
        ("Tension du circuit", "120 V"),
        ("Courant", "15 A"),
        ("Longueur du parcours, aller simple", "100 pi 0 po"),
        ("Calibre du conducteur", "12 AWG"),
        ("Métal", "Cuivre"),
        ("Circuit", "Triphasé"),
    ];
    assert_eq!(entrees.len(), attendues.len());
    for (obtenue, attendue) in entrees.iter().zip(attendues) {
        assert_eq!((obtenue.0.as_str(), obtenue.1.as_str()), attendue);
    }
}

#[test]
fn point_decimal_selon_les_preferences() {
    let mut prefs = Preferences::default();
    prefs.separateur_decimal = noyau::prefs::SeparateurDecimal::Point;
    let c = calculateur("dalle").unwrap();
    let saisies: Vec<String> = ["24", "20", "4", "10"]
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    let e = c.evaluer(&saisies, &prefs);
    assert_eq!(
        e.resultats[1].1.replace('\u{a0}', " "),
        "5.93 vg³ (4.53 m³)"
    );
}
