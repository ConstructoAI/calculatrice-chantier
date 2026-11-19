//! Vecteurs 19.4 : arithmétique, pourcentages, fonctions mathématiques.

mod commun;
use commun::verifier;

#[test]
fn operations_et_unites() {
    verifier(&[
        ("2 + 3 × 4 =", "20"),
        ("11 Po + 2 Pi 1 Po =", "3 pi 0 po"),
        ("5 Pi 7 Po 1 / 2 + 18 Pi 8 Po =", "24 pi 3 1/2 po"),
        ("11 Pi 7 Po 1 / 2 − 3 Pi =", "8 pi 7 1/2 po"),
        ("81 Po − 32 Po =", "49 po"),
        ("5 Pi 4 Po 1 / 2 + 8 Po + 3,5 Vg =", "16 pi 6 1/2 po"),
        ("3,5 Vg + 2 Pi =", "4,166667 vg"),
        ("5 Pi 3 Po × 11 Pi 6 Po 1 / 2 =", "60,59375 pi²"),
        ("2 Pi 7 Po × 10 =", "25 pi 10 po"),
        ("30 Pi 4 Po ÷ 7 Po =", "52"),
        ("20 Pi 3 Po ÷ 9 =", "2 pi 3 po"),
        ("10 Pi ÷ 2 Pi 2 Po =", "4,615385"),
        ("2 Pi 5 Po × 3 =", "7 pi 3 po"),
        ("↳ Pi", "7,25 pi"),
        ("13 Pi 8 Po ÷ 2 =", "6 pi 10 po"),
        ("12 Pi 6 Po × 15 Pi 8 Po =", "195,8333 pi²"),
        ("8 Po × 16 Po =", "128 po²"),
        ("10 Pi × 6 Po =", "5 pi²"),
        ("2 m × 50 Conv 7 =", "1 m²"),
        ("10 Pi × 2 m =", "65,6168 pi²"),
        ("3 Pi × 1 Pi 9 Po 5 / 8 × 2 Pi 4 Po =", "12,61458 pi³"),
        ("↳ Conv Vg", "0,467207 vg³"),
        ("36 Pi 3 Po × 11 Pi 6 Po =", "416,875 pi²"),
        ("↳ × 4 Po =", "5,146605 vg³"),
        ("↳ × 55 Conv 0", "COÛT │ 283,06 $"),
        ("12 Pi 6 Po × 3,50 Conv 0", "COÛT │ 43,75 $"),
        ("156 Po ÷ 5 Po 1 / 2 =", "28,36364"),
        ("114 Po ÷ 29 =", "3 15/16 po"),
        ("5 Pi + 3 Pi Conv m", "2,438 m"),
    ]);
}

#[test]
fn pourcentages() {
    verifier(&[
        ("500 Pi × 18 %", "90 pi 0 po"),
        ("137 Pi Pi + 10 %", "150,7 pi²"),
        ("552 Pi 6 Po − 20 %", "442 pi 0 po"),
        ("350 Vg Vg Vg ÷ 80 %", "437,5 vg³"),
        ("75 %", "75 %"),
        ("75 % × 200 =", "150"),
        ("200 + 10 %", "220"),
    ]);
}

#[test]
fn fonctions_mathematiques() {
    verifier(&[
        ("7 Pi 4 Po Conv %", "53,77778 pi²"),
        ("100 Conv Eff", "10"),
        ("25 Pi Pi Conv Eff", "5 pi 0 po"),
        ("8 Conv ÷", "0,125"),
        ("Conv +", "3,141593"),
        ("5 Conv − =", "−5"),
        ("2 × Conv + =", "6,283185"),
        ("2 + 3 Conv % =", "11"),
    ]);
}

#[test]
fn resolution_temporaire() {
    verifier(&[
        ("44 / 64 + 1 / 64 =", "45/64 po"),
        ("↳ Conv 1", "11/16 po"),
        ("↳ Conv 2", "1/2 po"),
        ("↳ Conv 3", "23/32 po"),
        ("↳ Conv 4", "3/4 po"),
        ("↳ Conv 6", "45/64 po"),
        ("↳ Conv 8", "3/4 po"),
        // La résolution temporaire tombe à la seconde pression d'Effacer.
        ("44 / 64 = Eff Eff 45 / 64 =", "45/64 po"),
        (
            "44 / 64 = Conv 1 Eff Eff 1 / 2 + 1 / 32 Conv 1 =",
            "9/16 po",
        ),
    ]);
}

#[test]
fn chainage_des_operateurs() {
    verifier(&[
        // Sans opérande droit, l'opérateur est remplacé.
        ("5 + × 3 =", "15"),
        // « = » sans opérande droit annule l'opération et garde l'opérande gauche.
        ("5 + =", "5"),
        // Après « = », un opérateur poursuit avec le résultat.
        ("2 + 3 = × 4 =", "20"),
        // Après « = », un chiffre commence un nouveau calcul.
        ("2 + 3 = 4 × 2 =", "8"),
    ]);
}
