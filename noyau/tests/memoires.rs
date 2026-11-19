//! Vecteurs 19.5 : mémoires.

mod commun;
use commun::{attendre, moteur_neuf, verifier};

#[test]
fn memoire_cumulative() {
    let mut m = moteur_neuf();
    attendre(&mut m, "100 M+", "M+ │ 100");
    attendre(&mut m, "200 M+", "M+ │ 200");
    attendre(&mut m, "50 Conv M+", "M− │ 50");
    assert!(m.ecran().memoire_active);
    attendre(&mut m, "Rap M+", "TOTAL M │ 250");
    attendre(&mut m, "M+", "MOYENNE │ 83,33333");
    attendre(&mut m, "M+", "NOMBRE │ 3");
    attendre(&mut m, "Rap Rap", "M+ │ 250");
    // La mémoire est ensuite vide; l'indicateur M est éteint.
    assert!(!m.ecran().memoire_active);
    attendre(&mut m, "Rap M+", "AUCUNE VALEUR");
}

#[test]
fn le_defilement_ne_modifie_pas_la_memoire() {
    let mut m = moteur_neuf();
    attendre(&mut m, "100 M+ 200 M+ Rap M+ M+ M+ M+", "TOTAL M │ 300");
    // Une autre touche met fin au défilement : M+ ajoute de nouveau.
    attendre(&mut m, "50 M+ Rap M+", "TOTAL M │ 350");
    attendre(&mut m, "Conv Rap", "MÉMOIRE EFFACÉE");
    assert!(!m.ecran().memoire_active);
}

#[test]
fn genre_fixe_par_la_premiere_valeur() {
    verifier(&[
        ("5 Pi M+ 3 M+", "ERREUR – unités incompatibles"),
        ("5 Pi M+ 18 Po M+ Rap M+", "TOTAL M │ 6 pi 6 po"),
    ]);
}

#[test]
fn registres_et_valeurs_memorisees() {
    verifier(&[
        ("175 Sto 1 Eff Eff Rap 1", "M1 │ 175"),
        ("1575 Vg Vg Sto 2 Eff Eff Rap 2", "M2 │ 1 575 vg²"),
        ("175 Sto 3 0 Sto 3 Rap 3", "AUCUNE VALEUR"),
        ("18 Po Sto 5", "ENTRAXE │ 18 po MÉM."),
        ("↳ Rap 5", "ENTRAXE │ 18 po MÉM."),
        ("Rap 5", "ENTRAXE │ 16 po MÉM."),
        ("Rap 6", "SECTION DE SEMELLE │ 264 po² MÉM."),
        ("Rap 7", "CONTREMARCHE VISÉE │ 7 1/2 po MÉM."),
        ("Rap 8", "ÉPAISSEUR PLANCHER │ 10 po MÉM."),
        ("Rap 9", "GIRON VISÉ │ 10 po MÉM."),
        ("Rap 4", "AIRE D'UN BLOC │ 128 po² MÉM."),
        ("↳ Rap 4", "LONGUEUR D'UN BLOC │ 16 po MÉM."),
        ("↳ Rap 4", "AIRE D'UN BLOC │ 128 po² MÉM."),
        ("Rap 0", "MASSE VOLUMIQUE │ 1,5 tc/vg³ MÉM."),
        ("Rap Onglet", "ANGLE MOULURE │ 45,00° MÉM."),
        ("Rap /", "RÉSOLUTION │ 1/16 po"),
        ("Rap 1", "AUCUNE VALEUR"),
        ("7 Po Sto 7 Rap 7", "CONTREMARCHE VISÉE │ 7 po MÉM."),
        ("9 Po Sto 8 Rap 8", "ÉPAISSEUR PLANCHER │ 9 po MÉM."),
        ("11 Po Sto 9 Rap 9", "GIRON VISÉ │ 11 po MÉM."),
        ("18 Po Sto 4 Rap 4 Rap 4", "LONGUEUR D'UN BLOC │ 18 po MÉM."),
    ]);
}

#[test]
fn donnees_refusees_par_stocker() {
    verifier(&[
        ("5 Sto 5", "ERREUR – unités incompatibles"),
        ("5 Pi Sto 6", "ERREUR – unités incompatibles"),
        ("5 Pi Sto 0", "ERREUR – unités incompatibles"),
        ("95 Sto Onglet", "ERREUR – saisie invalide"),
        ("5 Sto Pente", "ERREUR – saisie invalide"),
    ]);
}

#[test]
fn rappel_des_valeurs_saisies() {
    verifier(&[
        (
            "20 Pi Longueur Eff Rap Longueur",
            "LONGUEUR │ 20 pi 0 po MÉM.",
        ),
        ("12 Pi Largeur Eff Rap Largeur", "LARGEUR │ 12 pi 0 po"),
        ("8 Pi Hauteur Eff Rap Hauteur", "HAUTEUR │ 8 pi 0 po"),
        ("5 Pi Cercle Eff Rap Cercle", "DIAMÈTRE │ 5 pi 0 po"),
        ("5 Pi Cercle 85 Arc Eff Rap Arc", "ARC │ 85,00°"),
        ("7 Po Pente Eff Rap Pente", "PENTE │ 7 po MÉM."),
        (
            "8 Po Conv Arêtier Eff Rap Conv Arêtier",
            "PENTE IRRÉGULIÈRE │ 8 po",
        ),
        ("Rap Longueur", "AUCUNE VALEUR"),
        ("Rap Conv Arêtier", "AUCUNE VALEUR"),
    ]);
}

#[test]
fn une_valeur_rappelee_sert_d_operande() {
    verifier(&[
        ("12 Pi Course 2 Pi × Rap Course =", "24 pi²"),
        // Appuyer aussitôt sur la touche d'origine ne réaffecte pas la valeur (R-37).
        (
            "12 Pi Course Eff Rap Course Course",
            "COURSE │ 12 pi 0 po MÉM.",
        ),
        // Une valeur rappelée est une valeur libre : une autre touche la reçoit.
        (
            "12 Pi Course Eff Rap Course Longueur",
            "LONGUEUR │ 12 pi 0 po",
        ),
    ]);
}
