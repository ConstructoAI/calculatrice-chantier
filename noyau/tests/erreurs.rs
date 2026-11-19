//! Vecteurs 19.14 : erreurs et effacement.

mod commun;
use commun::{attendre, moteur_neuf, verifier};
use noyau::{Effet, Touche};

#[test]
fn erreurs() {
    verifier(&[
        ("5 Pi + 3 =", "ERREUR – unités incompatibles"),
        // En état d'erreur, seule la touche Effacer est acceptée (R-38).
        ("↳ 7", "ERREUR – unités incompatibles"),
        ("↳ Eff", "0"),
        ("5 ÷ 0 =", "ERREUR – calcul impossible"),
        ("5 Pi Pi × 2 Pi Pi =", "ERREUR – unités incompatibles"),
        ("4 Pi Conv Eff", "ERREUR – unités incompatibles"),
        ("9 Conv − Conv Eff", "ERREUR – calcul impossible"),
        ("5 Pi 6 Pi", "ERREUR – saisie invalide"),
        ("5 Po 6 Po", "ERREUR – saisie invalide"),
        ("1 / 0 =", "ERREUR – saisie invalide"),
        ("1,5 / 2", "ERREUR – saisie invalide"),
        ("5 Po 6 =", "ERREUR – saisie invalide"),
        ("0 Conv ÷", "ERREUR – calcul impossible"),
        ("55 Conv 0", "ERREUR – saisie invalide"),
        ("5 + 55 Conv 0", "ERREUR – saisie invalide"),
    ]);
}

#[test]
fn donnees_insuffisantes() {
    verifier(&[
        ("Escalier", "AUCUNE VALEUR"),
        ("Diag", "AUCUNE VALEUR"),
        ("Pente", "AUCUNE VALEUR"),
        ("Arêtier", "AUCUNE VALEUR"),
        ("Empannon", "AUCUNE VALEUR"),
        ("Cercle", "AUCUNE VALEUR"),
        ("Arc", "AUCUNE VALEUR"),
        ("Largeur", "AUCUNE VALEUR"),
        ("Conv Longueur", "AUCUNE VALEUR"),
        ("Conv Cercle", "AUCUNE VALEUR"),
        ("Onglet", "AUCUNE VALEUR"),
        (
            "10 Pi Diag 12 Pi Montée Course",
            "ERREUR – calcul impossible",
        ),
        ("5 Conv Course", "AUCUNE VALEUR"),
        ("8 Pi Course Eff Eff Course", "AUCUNE VALEUR"),
        // « AUCUNE VALEUR » n'est pas bloquant : la touche suivante est acceptée.
        ("Diag 7 =", "7"),
    ]);
}

#[test]
fn effacement() {
    verifier(&[
        // La première pression d'Effacer garde l'opération en attente.
        ("5 + 3 Eff 4 =", "9"),
        // La seconde l'annule.
        ("5 + 3 Eff Eff 4 =", "4"),
        // La pente survit aux deux pressions; la course, non.
        ("7 Po Pente 8 Pi Course Eff Eff Rap Pente", "PENTE │ 7 po"),
        ("7 Po Pente 8 Pi Course Eff Eff Rap Course", "AUCUNE VALEUR"),
        // La première pression garde les registres.
        ("8 Pi Course Eff Course", "COURSE │ 8 pi 0 po MÉM."),
        // Les mémoires survivent.
        ("175 Sto 1 100 M+ Eff Eff Rap 1", "M1 │ 175"),
    ]);
}

#[test]
fn effacer_desarme_les_prefixes() {
    let mut m = moteur_neuf();
    attendre(&mut m, "5 Sto Eff", "0");
    attendre(&mut m, "5 Rap Eff", "0");
    // Conv deux fois : désarmé.
    attendre(&mut m, "5 Conv Conv 6", "56");
}

#[test]
fn effets_destines_a_l_interface() {
    let mut m = moteur_neuf();
    m.appuyer(Touche::Conv);
    assert_eq!(m.appuyer(Touche::Stocker), vec![Effet::OuvrirPreferences]);
    m.appuyer(Touche::Conv);
    assert_eq!(m.appuyer(Touche::Egal), vec![Effet::BasculerRuban]);
    m.appuyer(Touche::Rappel);
    assert_eq!(m.appuyer(Touche::Egal), vec![Effet::BasculerRuban]);
    m.appuyer(Touche::Conv);
    assert_eq!(
        m.appuyer(Touche::Multiplier),
        vec![Effet::ConfirmerEffacerTout]
    );
}

#[test]
fn effacer_tout() {
    let mut m = moteur_neuf();
    attendre(
        &mut m,
        "7 Po Pente 175 Sto 1 100 M+ 18 Po Sto 5 2 + 3 =",
        "5",
    );
    m.effacer_tout();
    attendre(&mut m, "", "TOUT EFFACÉ");
    assert!(!m.ecran().erreur);
    assert!(m.ruban().est_vide());
    attendre(&mut m, "Rap 1", "AUCUNE VALEUR");
    attendre(&mut m, "Rap M+", "AUCUNE VALEUR");
    attendre(&mut m, "Rap Pente", "AUCUNE VALEUR");
    // Les valeurs mémorisées reviennent à leurs valeurs par défaut.
    attendre(&mut m, "Rap 5", "ENTRAXE │ 16 po MÉM.");
}

#[test]
fn depassement_de_capacite() {
    let mut m = moteur_neuf();
    attendre(
        &mut m,
        "999999999999 × 999999999999 =",
        "ERREUR – dépassement",
    );
    attendre(&mut m, "Eff", "0");
    // Une longueur trop grande en pouces passe à l'unité supérieure.
    attendre(&mut m, "999999999 Po × 12 =", "999 999 999 pi 0 po");
}
