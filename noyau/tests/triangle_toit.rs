//! Vecteurs 19.8 à 19.10 : pente, triangle, arêtier, empannons, mur en pente, toiture.

mod commun;
use commun::{attendre, moteur_avec, moteur_neuf, suite, verifier};
use noyau::prefs::{EmpannonsIrreguliers, Ordre};

#[test]
fn formes_de_la_pente() {
    verifier(&[
        ("30,45 Pente", "PENTE │ 30,45°"),
        ("↳ Pente", "PENTE │ 58,78702 %"),
        ("↳ Pente", "RATIO DE PENTE │ 0,58787"),
        ("↳ Pente", "PENTE │ 7 1/16 po"),
        ("47,25 % Pente", "PENTE │ 47,25 %"),
        ("↳ Pente", "RATIO DE PENTE │ 0,4725"),
        ("↳ Pente", "PENTE │ 5 11/16 po"),
        ("↳ Pente", "PENTE │ 25,29°"),
        ("0,65 Conv Pente", "RATIO DE PENTE │ 0,65"),
        ("↳ Pente", "PENTE │ 7 13/16 po"),
        ("↳ Pente", "PENTE │ 33,02°"),
        ("↳ Pente", "PENTE │ 65 %"),
        ("250 Conv 9 Pente", "PENTE │ 25 %"),
        ("↳ Pente Pente", "PENTE │ 3 po"),
        ("7 Po Pente", "PENTE │ 7 po"),
        ("↳ Pente", "PENTE │ 30,26°"),
        ("↳ Pente", "PENTE │ 58,33333 %"),
        ("↳ Pente", "RATIO DE PENTE │ 0,583333"),
        ("↳ Pente", "PENTE │ 7 po"),
    ]);
}

#[test]
fn triangle_rectangle() {
    verifier(&[
        ("7 Po Pente 14 Pi 4 Po ÷ 2 = Course", "COURSE │ 7 pi 2 po"),
        ("↳ Diag", "DIAGONALE │ 8 pi 3 9/16 po"),
        ("↳ Diag", "COUPE D'APLOMB │ 30,26°"),
        ("↳ Diag", "COUPE DE NIVEAU │ 59,74°"),
        ("↳ Diag", "DIAGONALE │ 8 pi 3 9/16 po"),
        (
            "6 Pi 11 Po 1 / 2 Montée 14 Pi 6 Po Course Diag",
            "DIAGONALE │ 16 pi 1 po",
        ),
        ("↳ Pente", "PENTE │ 5 3/4 po"),
        ("↳ Pente", "PENTE │ 25,64°"),
        ("9 Pi Montée 12 Pi Course Diag", "DIAGONALE │ 15 pi 0 po"),
        ("↳ Pente", "PENTE │ 9 po"),
        ("↳ Pente", "PENTE │ 36,87°"),
        (
            "7 Po Pente 11 Pi 6 Po Course Montée",
            "MONTÉE │ 6 pi 8 1/2 po",
        ),
        (
            "30 Pente 20 Pi 4 Po Course Montée",
            "MONTÉE │ 11 pi 8 7/8 po",
        ),
        ("↳ Diag", "DIAGONALE │ 23 pi 5 3/4 po"),
        ("6 Po Pente 4 Pi Diag Course", "COURSE │ 3 pi 6 15/16 po"),
        ("↳ Montée", "MONTÉE │ 1 pi 9 7/16 po"),
        ("65 % Pente 2 Pi Montée Course", "COURSE │ 3 pi 0 15/16 po"),
        ("12 Pi Course 13 Pi Diag Montée", "MONTÉE │ 5 pi 0 po"),
        ("↳ Pente", "PENTE │ 5 po"),
        (
            "23 Pi 8 Po Montée 45 Pi 6 Po Course Diag",
            "DIAGONALE │ 51 pi 3 7/16 po",
        ),
        ("3 Montée 4 Course Diag", "DIAGONALE │ 5"),
        // La pente survit à Effacer ×2.
        (
            "7 Po Pente Eff Eff 8 Pi Course Montée",
            "MONTÉE │ 4 pi 8 po",
        ),
        (
            "65 % Pente 2 Pi Montée Course 55 Pi × Rap Course × 2 Pi =",
            "338,4615 pi³",
        ),
    ]);
}

#[test]
fn donnee_puis_valeur_saisie_reaffichee() {
    verifier(&[
        ("8 Pi Course", "COURSE │ 8 pi 0 po !MÉM."),
        ("↳ Course", "COURSE │ 8 pi 0 po MÉM."),
        ("13 Pi Diag", "DIAGONALE │ 13 pi 0 po"),
        ("↳ Diag", "DIAGONALE │ 13 pi 0 po MÉM."),
        // Une sortie de fonction n'est pas réaffectée à une autre touche (R-36)…
        ("8 Pi Course Longueur", "AUCUNE VALEUR"),
        // … sauf après « = » (promotion).
        ("8 Pi Course = Longueur", "LONGUEUR │ 8 pi 0 po"),
        // La donnée d'une fonction peut servir d'opérande.
        ("5 Pi + 3 Pi Course =", "8 pi 0 po"),
        // Une sortie convertie garde son étiquette et son cycle.
        ("7 Po Pente 7 Pi 2 Po Course Diag m", "DIAGONALE │ 2,529 m"),
        ("↳ Diag", "COUPE D'APLOMB │ 30,26°"),
    ]);
}

#[test]
fn aretier_et_empannons_d_un_toit_regulier() {
    let mut m = moteur_neuf();
    attendre(
        &mut m,
        "8 Pi 5 Po Course 7 Po Pente Diag",
        "DIAGONALE │ 9 pi 8 15/16 po",
    );
    suite(
        &mut m,
        "Arêtier",
        &[
            "ARÊTIER/NOUE │ 12 pi 10 1/2 po",
            "COUPE D'APLOMB │ 22,42°",
            "COUPE DE NIVEAU │ 67,58°",
            "COUPE LATÉRALE │ 45,00°",
            "ARÊTIER/NOUE │ 12 pi 10 1/2 po",
        ],
    );
    suite(
        &mut m,
        "Empannon",
        &[
            "ENTRAXE │ 16 po MÉM.",
            "INCRÉMENT │ 1 pi 6 1/2 po",
            "EMPANNON 1 │ 8 pi 2 3/8 po",
            "EMPANNON 2 │ 6 pi 7 7/8 po",
            "EMPANNON 3 │ 5 pi 1 3/8 po",
            "EMPANNON 4 │ 3 pi 6 13/16 po",
            "EMPANNON 5 │ 2 pi 0 5/16 po",
            "EMPANNON 6 │ 0 pi 5 13/16 po",
            "COUPE D'APLOMB │ 30,26°",
            "COUPE DE NIVEAU │ 59,74°",
            "COUPE LATÉRALE │ 45,00°",
            "ENTRAXE │ 16 po",
        ],
    );
}

#[test]
fn empannons_9_sur_12_a_18_pouces() {
    let mut m = moteur_neuf();
    attendre(
        &mut m,
        "18 Po Sto 5 9 Po Pente 6 Pi 9 Po Course",
        "COURSE │ 6 pi 9 po",
    );
    suite(
        &mut m,
        "Empannon",
        &[
            "ENTRAXE │ 18 po",
            "INCRÉMENT │ 1 pi 10 1/2 po",
            "EMPANNON 1 │ 6 pi 6 3/4 po",
            "EMPANNON 2 │ 4 pi 8 1/4 po",
            "EMPANNON 3 │ 2 pi 9 3/4 po",
            "EMPANNON 4 │ 0 pi 11 1/4 po",
            "COUPE D'APLOMB │ 36,87°",
            "COUPE DE NIVEAU │ 53,13°",
            "COUPE LATÉRALE │ 45,00°",
        ],
    );
}

#[test]
fn toit_6_sur_12_huit_empannons() {
    let mut m = moteur_neuf();
    attendre(
        &mut m,
        "6 Po Pente 12 Pi Course Diag",
        "DIAGONALE │ 13 pi 5 po",
    );
    suite(
        &mut m,
        "Arêtier",
        &[
            "ARÊTIER/NOUE │ 18 pi 0 po",
            "COUPE D'APLOMB │ 19,47°",
            "COUPE DE NIVEAU │ 70,53°",
        ],
    );
    suite(
        &mut m,
        "Empannon",
        &[
            "ENTRAXE │ 16 po",
            "INCRÉMENT │ 1 pi 5 7/8 po",
            "EMPANNON 1 │ 11 pi 11 1/8 po",
            "EMPANNON 2 │ 10 pi 5 1/4 po",
            "EMPANNON 3 │ 8 pi 11 5/16 po",
            "EMPANNON 4 │ 7 pi 5 7/16 po",
            "EMPANNON 5 │ 5 pi 11 9/16 po",
            "EMPANNON 6 │ 4 pi 5 11/16 po",
            "EMPANNON 7 │ 2 pi 11 3/4 po",
            "EMPANNON 8 │ 1 pi 5 7/8 po",
            // Le neuvième, de longueur nulle, n'est pas listé.
            "COUPE D'APLOMB │ 26,57°",
        ],
    );
}

#[test]
fn toit_irregulier_entraxe_constant() {
    let mut m = moteur_neuf();
    attendre(
        &mut m,
        "7 Po Pente 4 Pi Course Diag",
        "DIAGONALE │ 4 pi 7 9/16 po",
    );
    attendre(&mut m, "8 Po Conv Arêtier", "PENTE IRRÉGULIÈRE │ 8 po");
    suite(
        &mut m,
        "Arêtier",
        &[
            "ARÊTIER/NOUE IRRÉG. │ 5 pi 9 11/16 po",
            "COUPE D'APLOMB │ 23,70°",
            "COUPE DE NIVEAU │ 66,30°",
            "COUPE LATÉRALE 1 │ 41,19°",
            "COUPE LATÉRALE 2 │ 48,81°",
        ],
    );
    attendre(&mut m, "Conv Empannon", "ENTRAXE │ 16 po MÉM.");
    suite(
        &mut m,
        "Empannon",
        &[
            "INCRÉMENT │ 1 pi 4 13/16 po",
            "EMPANNON IRRÉG. 1 │ 2 pi 9 5/8 po",
            "EMPANNON IRRÉG. 2 │ 1 pi 4 13/16 po",
            "COUPE D'APLOMB │ 33,69°",
            "COUPE DE NIVEAU │ 56,31°",
            "COUPE LATÉRALE │ 41,19°",
            "ENTRAXE │ 16 po MÉM.",
            "INCRÉMENT │ 1 pi 9 3/16 po",
            "EMPANNON 1 │ 2 pi 10 3/8 po",
            "EMPANNON 2 │ 1 pi 1 1/4 po",
            "COUPE D'APLOMB │ 30,26°",
            "COUPE DE NIVEAU │ 59,74°",
            "COUPE LATÉRALE │ 48,81°",
        ],
    );
}

#[test]
fn toit_irregulier_apparie_et_croissant() {
    let mut m = moteur_avec(|p| {
        p.empannons = Ordre::Croissant;
        p.empannons_irreguliers = EmpannonsIrreguliers::Apparies;
    });
    attendre(
        &mut m,
        "7 Po Pente 4 Pi Course 8 Po Conv Arêtier",
        "PENTE IRRÉGULIÈRE │ 8 po",
    );
    attendre(&mut m, "Conv Empannon", "ENTRAXE │ 16 po MÉM.");
    suite(
        &mut m,
        "Empannon",
        &[
            "INCRÉMENT │ 1 pi 4 13/16 po",
            "EMPANNON IRRÉG. 1 │ 1 pi 4 13/16 po",
            "EMPANNON IRRÉG. 2 │ 2 pi 9 5/8 po",
            "EMPANNON IRRÉG. 3 │ 4 pi 2 1/2 po",
            "COUPE D'APLOMB │ 33,69°",
            "COUPE DE NIVEAU │ 56,31°",
            "COUPE LATÉRALE │ 41,19°",
            // Entraxe recalculé : sans MÉM.
            "ENTRAXE │ 14 po !MÉM.",
            "INCRÉMENT │ 1 pi 6 1/2 po",
            "EMPANNON 1 │ 1 pi 6 1/2 po",
            "EMPANNON 2 │ 3 pi 1 1/16 po",
            "EMPANNON 3 │ 4 pi 7 9/16 po",
            "COUPE D'APLOMB │ 30,26°",
            "COUPE DE NIVEAU │ 59,74°",
            "COUPE LATÉRALE │ 48,81°",
        ],
    );
}

#[test]
fn empannon_irregulier_sans_pente_irreguliere() {
    verifier(&[("7 Po Pente 4 Pi Course Conv Empannon", "AUCUNE VALEUR")]);
}

#[test]
fn mur_en_pente() {
    let mut m = moteur_neuf();
    attendre(
        &mut m,
        "4 Pi Montée 8 Pi Course Conv Montée",
        "ENTRAXE │ 16 po MÉM.",
    );
    suite(
        &mut m,
        "Montée",
        &[
            "MONTANT 1 │ 3 pi 4 po",
            "MONTANT 2 │ 2 pi 8 po",
            "MONTANT 3 │ 2 pi 0 po",
            "MONTANT 4 │ 1 pi 4 po",
            "MONTANT 5 │ 0 pi 8 po",
            "BASE │ 0 pi 0 po",
            "ANGLE DU MUR │ 26,57°",
            "ENTRAXE │ 16 po MÉM.",
        ],
    );
}

#[test]
fn mur_en_pente_avec_base() {
    let mut m = moteur_neuf();
    attendre(
        &mut m,
        "4 Pi Montée 8 Pi Course 5 Pi Conv Montée",
        "BASE │ 5 pi 0 po",
    );
    suite(
        &mut m,
        "Montée",
        &[
            "ENTRAXE │ 16 po",
            "MONTANT 1 │ 8 pi 4 po",
            "MONTANT 2 │ 7 pi 8 po",
            "MONTANT 3 │ 7 pi 0 po",
            "MONTANT 4 │ 6 pi 4 po",
            "MONTANT 5 │ 5 pi 8 po",
            "BASE │ 5 pi 0 po",
            "ANGLE DU MUR │ 26,57°",
        ],
    );
}

#[test]
fn mur_en_pente_croissant() {
    let mut m = moteur_avec(|p| p.mur_pente = Ordre::Croissant);
    attendre(
        &mut m,
        "4 Pi Montée 8 Pi Course Conv Montée",
        "ENTRAXE │ 16 po MÉM.",
    );
    suite(
        &mut m,
        "Montée",
        &[
            "MONTANT 1 │ 0 pi 8 po",
            "MONTANT 2 │ 1 pi 4 po",
            "MONTANT 3 │ 2 pi 0 po",
            "MONTANT 4 │ 2 pi 8 po",
            "MONTANT 5 │ 3 pi 4 po",
            "MONTANT 6 │ 4 pi 0 po",
            "BASE │ 0 pi 0 po",
        ],
    );
}

#[test]
fn mur_en_pente_4_sur_6() {
    let mut m = moteur_neuf();
    attendre(
        &mut m,
        "4 Pi Montée 6 Pi Course Conv Montée",
        "ENTRAXE │ 16 po MÉM.",
    );
    suite(
        &mut m,
        "Montée",
        &[
            "MONTANT 1 │ 3 pi 1 5/16 po",
            "MONTANT 2 │ 2 pi 2 11/16 po",
            "MONTANT 3 │ 1 pi 4 po",
            "MONTANT 4 │ 0 pi 5 5/16 po",
            "BASE │ 0 pi 0 po",
            "ANGLE DU MUR │ 33,69°",
        ],
    );
}

#[test]
fn toiture_10_sur_12() {
    let mut m = moteur_neuf();
    attendre(
        &mut m,
        "10 Po Pente 14 Pi Longueur 11 Pi Largeur Conv Diag",
        "AIRE DU TOIT │ 200,4631 pi²",
    );
    suite(
        &mut m,
        "Diag",
        &[
            "CARRÉS │ 2,00",
            "PAQUETS │ 6,01",
            "PAQUET │ 33,33 pi² MÉM.",
            "FEUILLES 4×8 │ 6,26",
            "PENTE │ 10 po MÉM.",
            "AIRE EN PLAN │ 154 pi²",
            "AIRE DU TOIT │ 200,4631 pi²",
        ],
    );
}

#[test]
fn toiture_a_partir_de_la_montee_et_de_la_course() {
    let mut m = moteur_neuf();
    attendre(
        &mut m,
        "10 Pi Montée 15 Pi Course 50 Pi Longueur 30 Pi Largeur Conv Diag",
        "AIRE DU TOIT │ 1 802,776 pi²",
    );
    suite(
        &mut m,
        "Diag",
        &[
            "CARRÉS │ 18,03",
            "PAQUETS │ 54,08",
            "PAQUET │ 33,33 pi²",
            "FEUILLES 4×8 │ 56,34",
            "PENTE │ 8 po !MÉM.",
            "AIRE EN PLAN │ 1 500 pi²",
        ],
    );
}

#[test]
fn toiture_avec_une_aire_en_donnee() {
    verifier(&[
        (
            "6 Po Pente 100 Pi Pi Conv Diag",
            "AIRE DU TOIT │ 111,8034 pi²",
        ),
        ("100 Pi Pi Conv Diag", "AUCUNE VALEUR"),
        ("6 Po Pente Conv Diag", "AUCUNE VALEUR"),
    ]);
}
