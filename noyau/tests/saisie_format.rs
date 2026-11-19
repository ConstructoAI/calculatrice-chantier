//! Vecteurs 19.3 : saisie et conversions.

mod commun;
use commun::{attendre, moteur_neuf, verifier};

#[test]
fn saisie_des_longueurs() {
    verifier(&[
        ("5 Pi 1 Po 1 / 2 =", "5 pi 1 1/2 po"),
        ("5 Pi 6 =", "5 pi 6 po"),
        ("5 Pi 1 / 2 =", "5 pi 0 1/2 po"),
        ("15 / =", "15/16 po"),
        ("3 / 2 =", "1 1/2 po"),
        ("5 Vg =", "5 vg"),
        ("8 Po 1 / 8 =", "8 1/8 po"),
        ("1 / 2 =", "1/2 po"),
        ("17,32 Pi =", "17,32 pi"),
        ("9,0625 Po =", "9,0625 po"),
        ("14 Pi =", "14 pi 0 po"),
        ("5 Pi 6 [3/8] =", "5 pi 6 3/8 po"),
        ("6 [3/8] =", "6 3/8 po"),
        ("[3/8] =", "3/8 po"),
        ("5 Pi 1 / 2 [3/8] =", "5 pi 0 3/8 po"),
        ("12 3 ⌫ ⌫ 7 Po =", "17 po"),
    ]);
}

#[test]
fn conversions_de_longueurs() {
    verifier(&[
        ("14 Pi Conv Vg", "4,666667 vg"),
        ("↳ Conv Pi", "14 pi 0 po"),
        ("↳ Conv Po", "168 po"),
        ("↳ Conv m", "4,267 m"),
        ("↳ Conv 7", "426,72 cm"),
        ("↳ Conv 9", "4 267,2 mm"),
        ("15 Pi 9 Po 1 / 2 Conv Pi", "15,79167 pi"),
        ("↳ Pi", "15 pi 9 1/2 po"),
        ("17,32 Pi Conv Pi", "17 pi 3 13/16 po"),
        ("↳ Pi", "17,32 pi"),
        ("8 Po 1 / 8 Conv Po", "8,125 po"),
        ("↳ Pi", "0,677083 pi"),
        ("↳ Po", "8,125 po"),
        ("9,0625 Po Conv Po", "9 1/16 po"),
        ("↳ Pi", "0 pi 9 1/16 po"),
        ("↳ Pi", "0,755208 pi"),
        ("10 m Conv Pi", "32 pi 9 11/16 po"),
        ("2,4 m Conv Pi", "7 pi 10 1/2 po"),
        ("8 Pi Conv m", "2,438 m"),
        ("3 / 4 Conv 9", "19,05 mm"),
    ]);
}

#[test]
fn conversions_d_aires_et_de_volumes() {
    verifier(&[
        ("14 Pi Pi Conv Po", "2 016 po²"),
        ("↳ Conv Vg", "1,555556 vg²"),
        ("↳ Conv m", "1,300643 m²"),
        ("↳ Conv 7", "13 006,43 cm²"),
        ("14 Pi Pi Pi Conv Po", "24 192 po³"),
        ("↳ Conv Vg", "0,518519 vg³"),
        ("↳ Conv m", "0,396436 m³"),
        ("5 Conv 7 7 =", "5 cm²"),
        ("2 Conv 2 Conv Pi", "87 120 pi²"),
        ("↳ Conv m", "8 093,713 m²"),
        ("10000 Pi Pi Conv 2", "0,229568 acre"),
        ("5 × 3 = Pi", "15 pi 0 po"),
        ("100 Pi Pi =", "100 pi²"),
        ("100 Pi Pi Pi =", "100 pi³"),
        ("5 m m =", "5 m²"),
        ("5 Conv 7 =", "5 cm"),
        ("17,5 m =", "17,500 m"),
    ]);
}

#[test]
fn echo_de_saisie_dans_l_ordre_de_la_frappe() {
    let mut m = moteur_neuf();
    attendre(&mut m, "5 Pi 6 Po 1 /", "5 pi 6 po 1/");
    attendre(&mut m, "2", "5 pi 6 po 1/2");
    attendre(&mut m, "=", "5 pi 6 1/2 po");
    let mut m = moteur_neuf();
    attendre(&mut m, "23,42,", "23°42′");
    attendre(&mut m, "39", "23°42′39″");
    let mut m = moteur_neuf();
    attendre(&mut m, "1234567", "1 234 567");
    attendre(&mut m, ",50", "1 234 567,50");
}

#[test]
fn retour_arriere() {
    let mut m = moteur_neuf();
    attendre(&mut m, "5 Pi 6 Po ⌫", "5 pi 6");
    attendre(&mut m, "⌫ ⌫", "5");
    attendre(&mut m, "⌫", "0");
    // Sans saisie en cours, la touche est sans effet.
    attendre(&mut m, "7 = ⌫", "7");
}

#[test]
fn une_saisie_remplace_le_resultat_affiche() {
    let mut m = moteur_neuf();
    attendre(&mut m, "2 + 3 =", "5");
    attendre(&mut m, "7", "7");
    attendre(&mut m, "=", "7");
}

#[test]
fn angles_en_degres_minutes_secondes() {
    verifier(&[
        ("23,42,39 Conv ,", "23,71°"),
        ("44,29 Conv ,", "44°17′24″"),
        ("7,45,33 + 11,16,20 =", "19°01′53″"),
        ("30,89, =", "31°29′00″"),
    ]);
}
