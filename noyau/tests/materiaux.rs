//! Vecteurs 19.6 et 19.7 : dimensions, masses, blocs, bois, semelle, gypse, montants.

mod commun;
use commun::{verifier, verifier_avec};
use noyau::prefs::{AffichageAires, AffichageVolumes};

#[test]
fn longueur_largeur_hauteur() {
    verifier(&[
        ("20 Pi 6 Po Longueur", "LONGUEUR │ 20 pi 6 po"),
        ("↳ 25 Pi 6 Po Largeur", "LARGEUR │ 25 pi 6 po"),
        ("↳ Largeur", "AIRE │ 522,75 pi²"),
        ("↳ Largeur", "ÉQUERRAGE │ 32 pi 8 5/8 po"),
        ("↳ Largeur", "PÉRIMÈTRE │ 92 pi 0 po"),
        ("↳ Largeur", "LONGUEUR │ 20 pi 6 po"),
        ("↳ Largeur", "LARGEUR │ 25 pi 6 po"),
        ("↳ Largeur", "AIRE │ 522,75 pi²"),
        (
            "15 Pi Longueur 20 Pi Largeur 12 Pi Hauteur",
            "HAUTEUR │ 12 pi 0 po",
        ),
        ("↳ Hauteur", "VOLUME │ 3 600 pi³"),
        ("↳ Hauteur", "MURS │ 840 pi²"),
        ("↳ Hauteur", "PIÈCE │ 1 140 pi²"),
        ("↳ Hauteur", "LONGUEUR │ 15 pi 0 po"),
        ("↳ Hauteur", "LARGEUR │ 20 pi 0 po"),
        ("↳ Hauteur", "HAUTEUR │ 12 pi 0 po"),
        (
            "23 Pi 8 Po Longueur 45 Pi 6 Po Largeur Largeur Largeur",
            "ÉQUERRAGE │ 51 pi 3 7/16 po",
        ),
        // Sans donnée, Longueur réaffiche sa valeur.
        ("20 Pi Longueur Longueur", "LONGUEUR │ 20 pi 0 po MÉM."),
        // Sans la longueur, Largeur réaffiche seulement sa propre valeur.
        ("12 Pi Largeur Largeur", "LARGEUR │ 12 pi 0 po MÉM."),
        ("12 Pi Hauteur Hauteur", "HAUTEUR │ 12 pi 0 po MÉM."),
    ]);
}

#[test]
fn masses() {
    verifier(&[
        ("2500 Conv 4", "2 500 lb"),
        ("↳ Conv 1", "1 133,981 kg"),
        ("↳ Conv 6", "1,25 tc"),
        ("↳ Conv 3", "1,133981 t"),
        ("5 Vg Vg Vg Conv 4", "15 000 lb"),
        ("↳ Conv 6", "7,5 tc"),
        ("↳ Conv 1", "6 803,886 kg"),
        ("3 Conv 6 Conv Vg", "2 vg³"),
        ("5 Pi Conv 2", "5 pi 0 po"),
        ("5 Pi Conv Pi Conv 1", "ERREUR – unités incompatibles"),
        ("5 Pi Pi Conv 8", "ERREUR – unités incompatibles"),
    ]);
}

#[test]
fn blocs_et_briques() {
    verifier(&[
        ("22 Pi + 15 Pi 8 Po = Longueur", "LONGUEUR │ 37 pi 8 po"),
        ("↳ 4 Pi Hauteur Conv Longueur", "BLOCS │ 169,5"),
        ("↳ + 5 %", "177,975"),
        ("300 Pi Pi Conv Longueur", "BLOCS │ 337,5"),
        ("↳ + 3 %", "347,625"),
        (
            "30 Pi Longueur 45 Pi Largeur Largeur Largeur Largeur",
            "PÉRIMÈTRE │ 150 pi 0 po",
        ),
        ("↳ = Conv Longueur", "BLOCS │ 112,5"),
        ("↳ + 3 %", "115,875"),
        ("20 Pi Conv Longueur", "BLOCS │ 15"),
        ("↳ Longueur", "LONGUEUR D'UN BLOC │ 16 po MÉM."),
        ("↳ Longueur", "BLOCS │ 15"),
        ("21 Po Po Sto 4", "AIRE D'UN BLOC │ 21 po² MÉM."),
        (
            "↳ 40 Pi Longueur 8 Pi Hauteur Conv Longueur",
            "BLOCS │ 2 194,286",
        ),
        ("↳ Longueur", "AIRE D'UN BLOC │ 21 po² MÉM."),
        ("↳ + 3 %", "21,63 po²"),
        (
            "21 Po Po Sto 4 40 Pi Longueur 8 Pi Hauteur Conv Longueur + 3 %",
            "2 260,114",
        ),
    ]);
}

#[test]
fn bois_en_pmp_et_cout() {
    verifier(&[
        ("2 × 4 × 14 Conv 8", "9,333333 pmp"),
        ("↳ M+ 2 × 10 × 16 Conv 8", "26,66667 pmp"),
        ("↳ M+ 2 × 12 × 18 Conv 8", "36 pmp"),
        ("↳ M+ Rap Rap", "M+ │ 72 pmp"),
        ("↳ × 250 Conv 0", "COÛT │ 18,00 $"),
        ("150 Pi Pi Pi Conv 8", "1 800 pmp"),
    ]);
}

#[test]
fn semelle() {
    verifier(&[
        ("16 Po × 8 Po = Sto 6", "SECTION DE SEMELLE │ 128 po² MÉM."),
        ("↳ 232 Pi 6 Po Conv Largeur", "SEMELLE │ 7,654321 vg³"),
        ("↳ × 5 =", "38,2716 vg³"),
        ("100 Pi Conv Largeur", "SEMELLE │ 6,790123 vg³"),
        ("↳ Largeur", "SECTION DE SEMELLE │ 264 po² MÉM."),
        ("30 m Conv Largeur", "SEMELLE │ 5,109667 m³"),
        ("100 Pi Longueur Eff Conv Largeur", "SEMELLE │ 6,790123 vg³"),
    ]);
}

#[test]
fn gypse() {
    verifier(&[
        ("150 Pi Pi Conv Hauteur", "FEUILLES 4×8 │ 4,6875"),
        ("↳ Hauteur", "FEUILLES 4×9 │ 4,166667"),
        ("↳ Hauteur", "FEUILLES 4×10 │ 3,75"),
        ("↳ Hauteur", "FEUILLES 4×12 │ 3,125"),
        ("↳ Hauteur", "AIRE │ 150 pi²"),
        ("40 Pi Conv Hauteur", "FEUILLES 4×8 │ 10"),
        ("↳ Hauteur Hauteur Hauteur Hauteur", "LONGUEUR │ 40 pi 0 po"),
        (
            "20 Pi Longueur 8 Pi Hauteur Conv Hauteur",
            "FEUILLES 4×8 │ 5",
        ),
    ]);
}

#[test]
fn montants() {
    verifier(&[
        ("18 Pi 7 Po 1 / 2 Conv 5", "MONTANTS │ 15"),
        ("16 Pi Conv 5", "MONTANTS │ 13"),
        ("24 Po Sto 5 12 Pi Conv 5", "MONTANTS │ 7"),
        ("16 Pi Longueur Eff Conv 5", "MONTANTS │ 13"),
        // Sur un chiffre, jamais de cycle : le chiffre suivant commence une saisie.
        ("16 Pi Conv 5 5", "5"),
    ]);
}

#[test]
fn unites_imposees_par_les_preferences() {
    verifier_avec(
        |p| {
            p.affichage_aires = AffichageAires::MetresCarres;
            p.affichage_volumes = AffichageVolumes::PiedsCubes;
        },
        &[
            ("10 Pi × 10 Pi =", "9,290304 m²"),
            ("36 Pi 3 Po × 11 Pi 6 Po × 4 Po =", "138,9583 pi³"),
            ("7 Pi 4 Po Conv %", "4,996119 m²"),
            ("100 Pi Conv Largeur", "SEMELLE │ 183,3333 pi³"),
            // Les valeurs saisies et les conversions explicites ne sont pas touchées (R-32).
            ("100 Pi Pi =", "100 pi²"),
            ("100 Pi Pi Conv Vg", "11,11111 vg²"),
        ],
    );
}
