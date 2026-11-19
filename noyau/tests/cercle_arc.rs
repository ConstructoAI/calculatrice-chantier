//! Vecteurs 19.12 et 19.13 : cercle, arc, murs cintrés, colonne, cône, polygone,
//! onglet composé, angles et trigonométrie.

mod commun;
use commun::{attendre, moteur_avec, moteur_neuf, suite, verifier, verifier_avec};
use noyau::prefs::MurCintre;

#[test]
fn cercle_et_arc() {
    verifier(&[
        ("11 Po Cercle", "DIAMÈTRE │ 11 po"),
        ("↳ Cercle", "AIRE │ 95,03318 po²"),
        ("↳ Cercle", "CIRCONFÉRENCE │ 34 9/16 po"),
        ("↳ Cercle", "DIAMÈTRE │ 11 po MÉM."),
        ("5 Pi Cercle 85 Arc", "ARC │ 85,00°"),
        ("↳ Arc", "ARC │ 3 pi 8 1/2 po"),
        (
            "3 Pi 6 Po Course 1 Pi 3 Po Montée Conv Arc",
            "RAYON │ 1 pi 10 3/16 po",
        ),
        ("↳ Arc", "ARC │ 142,15°"),
        ("↳ Arc", "ARC │ 4 pi 7 1/16 po"),
        ("5 Pi Cercle 3 Pi 3 Po Arc", "ARC │ 3 pi 3 po"),
        ("↳ Arc", "ARC │ 74,48°"),
        ("↳ Arc", "CORDE │ 3 pi 0 5/16 po"),
        ("↳ Arc", "SEGMENT │ 1,051381 pi²"),
        ("↳ Arc", "SECTEUR │ 4,0625 pi²"),
        ("↳ Arc", "FLÈCHE │ 0 pi 6 1/8 po"),
        ("↳ Arc", "ENTRAXE │ 16 po MÉM."),
        ("↳ Arc", "MUR CINTRÉ 1 │ 0 pi 4 5/8 po"),
        // Au second tour, la longueur d'arc n'est plus sautée.
        ("↳ Arc", "ARC │ 74,48°"),
        ("↳ Arc", "ARC │ 3 pi 3 po"),
        ("10 Pi Cercle 6 Pi Course Montée", "FLÈCHE │ 1 pi 0 po"),
        ("10 Pi Cercle 1 Pi Montée Course", "CORDE │ 6 pi 0 po"),
        // Arc non résolu : la touche réaffiche seulement la donnée.
        ("85 Arc Arc", "ARC │ 85,00°"),
        ("10 Cercle Cercle", "AIRE │ 78,53982"),
    ]);
}

#[test]
fn mur_cintre_exterieur() {
    let mut m = moteur_neuf();
    attendre(
        &mut m,
        "8 Pi 1 Po Course 2 Pi 10 Po 1 / 2 Montée",
        "MONTÉE │ 2 pi 10 1/2 po",
    );
    suite(
        &mut m,
        "Arc",
        &[
            "ARC │ 141,70°",
            "ARC │ 10 pi 7 po",
            "CORDE │ 8 pi 1 po",
            "SEGMENT │ 16,96327 pi²",
            "SECTEUR │ 22,63527 pi²",
            "FLÈCHE │ 2 pi 10 1/2 po",
            "ENTRAXE │ 16 po",
            "MUR CINTRÉ 1 │ 0 pi 2 9/16 po",
            "MUR CINTRÉ 2 │ 0 pi 11 3/16 po",
            "MUR CINTRÉ 3 │ 2 pi 9 1/8 po",
            "ARC │ 141,70°",
        ],
    );
}

#[test]
fn mur_cintre_exterieur_avec_base() {
    let mut m = moteur_neuf();
    attendre(
        &mut m,
        "8 Pi 1 Po Course 2 Pi 10 Po 1 / 2 Montée 2 Pi Conv Montée",
        "BASE │ 2 pi 0 po",
    );
    suite(
        &mut m,
        "Arc",
        &[
            "ARC │ 141,70°",
            "ARC │ 10 pi 7 po",
            "CORDE │ 8 pi 1 po",
            "SEGMENT │ 16,96327 pi²",
            "SECTEUR │ 22,63527 pi²",
            "FLÈCHE │ 2 pi 10 1/2 po",
            "ENTRAXE │ 16 po",
            "MUR CINTRÉ 1 │ 2 pi 2 9/16 po",
            "MUR CINTRÉ 2 │ 2 pi 11 3/16 po",
            "MUR CINTRÉ 3 │ 4 pi 9 1/8 po",
        ],
    );
}

#[test]
fn autre_arc_exterieur() {
    let mut m = moteur_neuf();
    attendre(
        &mut m,
        "9 Pi 2 Po Course 3 Pi 11 Po 1 / 2 Montée",
        "MONTÉE │ 3 pi 11 1/2 po",
    );
    suite(
        &mut m,
        "Arc",
        &[
            "ARC │ 163,26°",
            "ARC │ 13 pi 2 3/8 po",
            "CORDE │ 9 pi 2 po",
            "SEGMENT │ 27,48605 pi²",
            "SECTEUR │ 30,57678 pi²",
            "FLÈCHE │ 3 pi 11 1/2 po",
            "ENTRAXE │ 16 po",
            "MUR CINTRÉ 1 │ 0 pi 2 3/8 po",
            "MUR CINTRÉ 2 │ 0 pi 10 1/8 po",
            "MUR CINTRÉ 3 │ 2 pi 3 9/16 po",
        ],
    );
}

#[test]
fn mur_cintre_interieur() {
    let mut m = moteur_avec(|p| p.mur_cintre = MurCintre::Interieur);
    attendre(&mut m, "15 Pi Course 5 Pi Montée", "MONTÉE │ 5 pi 0 po");
    suite(
        &mut m,
        "Arc",
        &[
            "ARC │ 134,76°",
            "ARC │ 19 pi 1 5/16 po",
            "CORDE │ 15 pi 0 po",
            "SEGMENT │ 54,19722 pi²",
            "SECTEUR │ 77,63472 pi²",
            "FLÈCHE │ 5 pi 0 po",
            "ENTRAXE │ 16 po",
            "MUR CINTRÉ 1 │ 4 pi 10 11/16 po",
            "MUR CINTRÉ 2 │ 4 pi 6 5/8 po",
            "MUR CINTRÉ 3 │ 3 pi 11 3/8 po",
            "MUR CINTRÉ 4 │ 3 pi 0 1/16 po",
            "MUR CINTRÉ 5 │ 1 pi 6 1/4 po",
        ],
    );
}

#[test]
fn colonne_et_cone() {
    verifier(&[
        ("2 Pi 4 Po Cercle Cercle", "AIRE │ 4,276057 pi²"),
        (
            "↳ 4 Pi 6 Po Hauteur Conv Cercle",
            "COLONNE – VOLUME │ 19,24226 pi³",
        ),
        ("↳ Cercle", "COLONNE – SURFACE │ 41,53884 pi²"),
        (
            "3 Pi 6 Po Cercle 5 Pi Hauteur Conv Cercle Cercle Cercle",
            "CÔNE – VOLUME │ 16,03521 pi³",
        ),
        ("↳ Cercle", "CÔNE – SURFACE │ 38,74513 pi²"),
        ("1,75 Sto 0", "MASSE VOLUMIQUE │ 1,75 tc/vg³ MÉM."),
        ("↳ 3 Pi 4 Po 1 / 2 Cercle Cercle", "AIRE │ 8,946176 pi²"),
        (
            "↳ 11 Pi 6 Po Hauteur Conv Cercle",
            "COLONNE – VOLUME │ 102,881 pi³",
        ),
        // Conversion d'une sortie : l'étiquette reste.
        ("↳ Conv Vg", "COLONNE – VOLUME │ 3,810408 vg³"),
        ("↳ × 5 =", "19,05204 vg³"),
        ("↳ Conv 6", "33,34107 tc"),
        ("↳ Conv 4", "66 682,14 lb"),
        ("↳ Conv 1", "30 246,51 kg"),
    ]);
}

#[test]
fn polygone() {
    verifier(&[
        ("15 Pi Conv Arc", "RAYON │ 15 pi 0 po"),
        ("↳ 6 Conv Course", "ANGLE INTÉRIEUR │ 120,00°"),
        ("↳ Course", "DEMI-ANGLE │ 60,00°"),
        ("↳ Course", "CÔTÉ │ 15 pi 0 po"),
        ("↳ Course", "PÉRIMÈTRE │ 90 pi 0 po"),
        ("↳ Course", "AIRE │ 584,5671 pi²"),
        ("↳ Course", "RAYON │ 15 pi 0 po"),
        ("↳ Course", "NB CÔTÉS │ 6"),
        ("↳ Course", "ANGLE INTÉRIEUR │ 120,00°"),
        (
            "10 Pi Conv Arc 8 Conv Course Course Course",
            "CÔTÉ │ 7 pi 7 7/8 po",
        ),
        ("↳ Course", "PÉRIMÈTRE │ 61 pi 2 3/4 po"),
        ("↳ Course", "AIRE │ 282,8427 pi²"),
        ("10 Pi Conv Arc 2 Conv Course", "ERREUR – saisie invalide"),
        // Le nombre de côtés est repris du dernier calcul.
        (
            "10 Pi Conv Arc 8 Conv Course Eff Conv Course",
            "ANGLE INTÉRIEUR │ 135,00°",
        ),
    ]);
}

#[test]
fn onglet_compose() {
    verifier(&[
        ("38 Conv Onglet", "ANGLE MOULURE │ 38,00° MÉM."),
        ("↳ 60 Onglet", "ONGLET │ 46,84°"),
        ("↳ Onglet", "BISEAU │ 43,03°"),
        ("↳ Onglet", "ANGLE MOULURE │ 38,00° MÉM."),
        ("↳ Onglet", "COIN │ 60,00°"),
        ("90 Onglet", "ONGLET │ 35,26°"),
        ("↳ Onglet", "BISEAU │ 30,00°"),
        ("8 Onglet", "COIN │ 135,00°"),
        ("↳ Onglet", "ONGLET │ 16,32°"),
        ("↳ Onglet", "BISEAU │ 15,70°"),
        ("38 Sto Onglet 90 Onglet", "ONGLET │ 31,62°"),
        ("↳ Onglet", "BISEAU │ 33,86°"),
        ("40 Sto Onglet 70 Onglet", "ONGLET │ 42,55°"),
        ("↳ Onglet", "BISEAU │ 38,87°"),
        // Sans donnée : la touche secondaire réaffiche l'angle de la moulure.
        ("Conv Onglet", "ANGLE MOULURE │ 45,00° MÉM."),
        // Le dernier coin saisi est repris.
        ("90 Onglet Eff Onglet", "ONGLET │ 35,26°"),
        ("200 Onglet", "ERREUR – saisie invalide"),
    ]);
}

#[test]
fn mode_trigonometrique() {
    verifier_avec(
        |p| p.mode_trigo = true,
        &[
            ("30 Longueur", "0,5"),
            ("60 Largeur", "0,5"),
            ("45 Hauteur", "1"),
            ("0,5 Conv Longueur", "30,00°"),
            ("8 Po Pente Pente", "PENTE │ 33,69°"),
            ("↳ Hauteur", "0,666667"),
            ("30,25 Hauteur", "0,583183"),
            ("↳ Conv Pente", "RATIO DE PENTE │ 0,583183"),
            ("↳ Pente", "PENTE │ 7 po"),
            ("6 Pi ÷ 10 Pi = Conv Largeur", "53,13°"),
            ("↳ Conv ,", "53°07′48″"),
            ("35 % Conv Hauteur", "19,29°"),
            ("↳ Conv ,", "19°17′24″"),
            ("1 Pi Longueur", "ERREUR – trigonométrie"),
            ("2 Conv Longueur", "ERREUR – trigonométrie"),
            ("90 Hauteur", "ERREUR – calcul impossible"),
            // Colonne et cône lisent leur hauteur dans Montée (12.18).
            (
                "2 Pi 4 Po Cercle 4 Pi 6 Po Montée Conv Cercle",
                "COLONNE – VOLUME │ 19,24226 pi³",
            ),
        ],
    );
}
