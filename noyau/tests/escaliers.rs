//! Vecteurs 19.11 : escaliers.

mod commun;
use commun::{attendre, moteur_neuf, suite};

/// Les sorties 12 à 15 (valeurs mémorisées) suivent toujours.
const MEMORISEES: [&str; 4] = [
    "CONTREMARCHE VISÉE │ 7 1/2 po MÉM.",
    "GIRON VISÉ │ 10 po MÉM.",
    "ÉCHAPPÉE │ 6 pi 8 po MÉM.",
    "ÉPAISSEUR PLANCHER │ 10 po MÉM.",
];

#[test]
fn a_montee_seule() {
    let mut m = moteur_neuf();
    attendre(&mut m, "9 Pi 11 Po Montée", "MONTÉE │ 9 pi 11 po");
    suite(
        &mut m,
        "Escalier",
        &[
            "CONTREMARCHE │ 7 7/16 po",
            "NB CONTREMARCHES │ 16",
            "ÉCART CONTREMARCHES │ 0 po",
            "GIRON │ 10 po MÉM.",
            "NB MARCHES │ 15",
            "ÉCART GIRONS │ 0 po",
            "TRÉMIE │ 10 pi 1 po",
            "LIMON │ 15 pi 6 15/16 po",
            "INCLINAISON │ 36,64°",
            "COURSE │ 12 pi 6 po",
            "MONTÉE │ 9 pi 11 po MÉM.",
        ],
    );
    suite(&mut m, "Escalier", &MEMORISEES);
    // Le cycle reboucle.
    suite(&mut m, "Escalier", &["CONTREMARCHE │ 7 7/16 po"]);
    assert!(m.ecran().avertissement.is_none());
}

#[test]
fn b_course_seule() {
    let mut m = moteur_neuf();
    attendre(&mut m, "20 Pi Course", "COURSE │ 20 pi 0 po");
    suite(
        &mut m,
        "Escalier",
        &[
            "CONTREMARCHE │ 7 1/2 po MÉM.",
            "NB CONTREMARCHES │ 25",
            "ÉCART CONTREMARCHES │ 0 po",
            "GIRON │ 10 po",
            "NB MARCHES │ 24",
            "ÉCART GIRONS │ 0 po",
            "TRÉMIE │ 10 pi 0 po",
            "LIMON │ 25 pi 0 po",
            "INCLINAISON │ 36,87°",
            "COURSE │ 20 pi 0 po MÉM.",
            "MONTÉE │ 15 pi 7 1/2 po",
        ],
    );
    suite(&mut m, "Escalier", &MEMORISEES);
}

#[test]
fn c_montee_et_course() {
    let mut m = moteur_neuf();
    attendre(
        &mut m,
        "10 Pi 1 Po Montée 15 Pi 5 Po Course",
        "COURSE │ 15 pi 5 po",
    );
    suite(
        &mut m,
        "Escalier",
        &[
            "CONTREMARCHE │ 7 9/16 po ⚠",
            "NB CONTREMARCHES │ 16",
            "ÉCART CONTREMARCHES │ 0 po",
            "GIRON │ 12 5/16 po",
            "NB MARCHES │ 15",
            "ÉCART GIRONS │ −5/16 po",
            "TRÉMIE │ 12 pi 2 1/2 po",
            "LIMON │ 18 pi 0 3/4 po",
            "INCLINAISON │ 31,56°",
            "COURSE │ 15 pi 5 po MÉM.",
            "MONTÉE │ 10 pi 1 po MÉM.",
        ],
    );
    suite(&mut m, "Escalier", &MEMORISEES);
}

#[test]
fn d_contremarche_maximale() {
    let mut m = moteur_neuf();
    attendre(
        &mut m,
        "10 Pi 1 Po Montée 15 Pi 5 Po Course Conv Escalier",
        "CONTREMARCHE │ 7 1/8 po",
    );
    assert!(m.ecran().avertissement.is_none());
    // Une fonction lancée avec Conv se poursuit sans Conv (R-27).
    suite(
        &mut m,
        "Escalier",
        &[
            "NB CONTREMARCHES │ 17",
            "ÉCART CONTREMARCHES │ 1/8 po",
            "GIRON │ 11 9/16 po",
            "NB MARCHES │ 16",
            "ÉCART GIRONS │ 0 po",
            "TRÉMIE │ 12 pi 2 1/16 po",
            "LIMON │ 18 pi 1 5/16 po",
            "INCLINAISON │ 31,64°",
            "COURSE │ 15 pi 5 po MÉM.",
            "MONTÉE │ 10 pi 1 po MÉM.",
        ],
    );
    suite(&mut m, "Escalier", &MEMORISEES);
}

#[test]
fn escalier_metrique() {
    let mut m = moteur_neuf();
    attendre(&mut m, "2,75 m Montée", "MONTÉE │ 2,750 m");
    suite(
        &mut m,
        "Escalier",
        &[
            "CONTREMARCHE │ 196 mm ⚠",
            "NB CONTREMARCHES │ 14",
            "ÉCART CONTREMARCHES │ −6 mm",
            "GIRON │ 254 mm MÉM.",
            "NB MARCHES │ 13",
            "ÉCART GIRONS │ 0 mm",
            "TRÉMIE │ 2,962 m",
            "LIMON │ 4,171 m",
            "INCLINAISON │ 37,66°",
            "COURSE │ 3,302 m",
            "MONTÉE │ 2,750 m MÉM.",
        ],
    );
}

#[test]
fn escalier_metrique_a_contremarche_maximale() {
    let mut m = moteur_neuf();
    attendre(
        &mut m,
        "2,75 m Montée Conv Escalier",
        "CONTREMARCHE │ 183 mm",
    );
    suite(
        &mut m,
        "Escalier",
        &[
            "NB CONTREMARCHES │ 15",
            "ÉCART CONTREMARCHES │ −5 mm",
            "GIRON │ 254 mm",
            "NB MARCHES │ 14",
            "ÉCART GIRONS │ 0 mm",
            "TRÉMIE │ 3,173 m",
            "LIMON │ 4,383 m",
            "INCLINAISON │ 35,77°",
            "COURSE │ 3,556 m",
            "MONTÉE │ 2,750 m",
        ],
    );
}

#[test]
fn valeurs_visees_modifiees() {
    let mut m = moteur_neuf();
    // Contremarche visée de 7 po : 119 ÷ 7 = 17 contremarches.
    attendre(
        &mut m,
        "7 Po Sto 7 9 Pi 11 Po Montée Escalier",
        "CONTREMARCHE │ 7 po",
    );
    suite(&mut m, "Escalier", &["NB CONTREMARCHES │ 17"]);
}

#[test]
fn la_resolution_regle_l_arrondi_de_chantier() {
    let mut m = commun::moteur_avec(|p| p.resolution = 8);
    // 121 ÷ 16 = 7,5625 : au huitième, la demie s'arrondit vers le haut, soit 7 5/8.
    attendre(
        &mut m,
        "10 Pi 1 Po Montée 15 Pi 5 Po Course Escalier",
        "CONTREMARCHE │ 7 5/8 po ⚠",
    );
    suite(
        &mut m,
        "Escalier",
        &["NB CONTREMARCHES │ 16", "ÉCART CONTREMARCHES │ 1 po"],
    );
}
