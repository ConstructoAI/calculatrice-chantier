//! Ruban de calcul (section 14) : ce que le moteur y inscrit.

mod commun;
use commun::{jouer, moteur_neuf};
use noyau::ruban::GenreLigne;
use noyau::{Effet, Moteur, Touche};

fn lignes(moteur: &Moteur) -> Vec<String> {
    moteur
        .ruban()
        .lignes
        .iter()
        .map(|l| {
            let signe = l.signe().map(|s| format!("{s} ")).unwrap_or_default();
            let etiquette = l
                .etiquette
                .as_ref()
                .map(|e| format!("{e} : "))
                .unwrap_or_default();
            format!("{signe}{etiquette}{}", l.texte).replace('\u{00A0}', " ")
        })
        .collect()
}

#[test]
fn chaine_d_additions() {
    let mut m = moteur_neuf();
    jouer(&mut m, "5 Pi 4 Po 1 / 2 + 8 Po + 3,5 Vg =");
    assert_eq!(
        lignes(&m),
        vec!["5 pi 4 1/2 po", "+ 8 po", "+ 3,5 vg", "= 16 pi 6 1/2 po"]
    );
    assert_eq!(m.ruban().lignes[3].genre, GenreLigne::Total);
    // Le texte exporté reprend le modèle de la section 14.3.
    let texte = m
        .ruban()
        .en_texte("Ruban du 19 novembre 2026, 8 h 04")
        .replace('\u{00A0}', " ");
    let attendu = "Ruban du 19 novembre 2026, 8 h 04\n\
                   ---------------------------------\n\
                   \u{20}                   5 pi 4 1/2 po\n\
                   +                            8 po\n\
                   +                          3,5 vg\n\
                   =                  16 pi 6 1/2 po\n";
    assert_eq!(texte, attendu);
}

#[test]
fn pourcentage_conversion_et_fonctions() {
    let mut m = moteur_neuf();
    jouer(&mut m, "137 Pi Pi + 10 %");
    assert_eq!(lignes(&m), vec!["137 pi²", "+ 10 %", "= 150,7 pi²"]);

    let mut m = moteur_neuf();
    jouer(&mut m, "14 Pi Conv Vg");
    assert_eq!(lignes(&m), vec!["→ 4,666667 vg"]);

    let mut m = moteur_neuf();
    jouer(
        &mut m,
        "7 Po Pente 8 Pi 5 Po Course Diag Diag Diag Diag Diag",
    );
    // Une sortie n'est inscrite que la première fois qu'elle est affichée.
    assert_eq!(
        lignes(&m),
        vec![
            "PENTE : 7 po",
            "COURSE : 8 pi 5 po",
            "DIAGONALE : 9 pi 8 15/16 po",
            "COUPE D'APLOMB : 30,26°",
            "COUPE DE NIVEAU : 59,74°",
        ]
    );
}

#[test]
fn memoire_separateur_et_cout() {
    let mut m = moteur_neuf();
    jouer(&mut m, "100 M+ 50 Conv M+ Rap M+ Eff Eff Eff Eff");
    assert_eq!(lignes(&m), vec!["M+ : 100", "M− : 50", "TOTAL M : 50", ""]);
    assert_eq!(m.ruban().lignes[3].genre, GenreLigne::Separateur);

    let mut m = moteur_neuf();
    jouer(&mut m, "12 Pi 6 Po × 3,50 Conv 0");
    assert_eq!(lignes(&m), vec!["12 pi 6 po", "× 3,5", "COÛT : 43,75 $"]);
}

#[test]
fn le_ruban_survit_a_effacer_deux_fois_mais_pas_a_effacer_tout() {
    let mut m = moteur_neuf();
    jouer(&mut m, "2 + 3 = Eff Eff");
    assert_eq!(m.ruban().lignes.len(), 4);
    m.effacer_tout();
    assert!(m.ruban().est_vide());
}

#[test]
fn etat_modifie_signale_ce_qui_est_a_sauvegarder() {
    let mut m = moteur_neuf();
    // Une simple frappe de chiffre ne change rien de persistant.
    assert!(m.appuyer(Touche::Chiffre(5)).is_empty());
    // Un opérateur inscrit une ligne au ruban.
    assert_eq!(m.appuyer(Touche::Additionner), vec![Effet::EtatModifie]);
    // L'état conservé reprend le ruban, la pente et les mémoires.
    jouer(&mut m, "3 = 7 Po Pente 175 Sto 1 100 M+");
    let copie = Moteur::nouveau(m.etat_persistant());
    assert_eq!(copie.ruban(), m.ruban());
    let mut copie = copie;
    assert_eq!(commun::texte(&jouer(&mut copie, "Rap 1")), "M1 │ 175");
    assert_eq!(commun::texte(&jouer(&mut copie, "Rap M+")), "TOTAL M │ 100");
    assert_eq!(
        commun::texte(&jouer(&mut copie, "8 Pi Course Montée")),
        "MONTÉE │ 4 pi 8 po"
    );
}
