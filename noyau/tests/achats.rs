//! Articles à acheter (section 22.17) : ce que les calculateurs mettent sur
//! la liste d'achat, et ce qu'ils n'y mettent pas.

use noyau::Preferences;
use noyau::metiers::{Article, calculateur};

fn achats(id: &str, saisies: &[&str]) -> Vec<Article> {
    let c = calculateur(id).unwrap_or_else(|| panic!("calculateur inconnu : {id}"));
    let textes: Vec<String> = saisies.iter().map(|s| (*s).to_string()).collect();
    let e = c.evaluer(&textes, &Preferences::default());
    assert!(
        !e.resultats.is_empty(),
        "{id} {saisies:?} : {:?}",
        e.erreurs
    );
    e.achats
}

#[test]
fn les_sacs_de_beton_s_additionnent_d_un_calculateur_a_l_autre() {
    let sacs = achats("sacs-de-beton", &["4", "4", "4", "10", "14"]);
    assert_eq!(sacs.len(), 1);
    assert_eq!(sacs[0].nombre, 12);
    assert!(
        sacs[0]
            .description
            .starts_with("Sacs de béton prémélangé · Rendement d'un sac : 14")
    );
    // Les colonnes donnent le même article, de même rendement.
    let colonnes = achats("colonnes", &["10", "4", "6", "10", "14"]);
    assert_eq!(colonnes.len(), 1);
    assert_eq!(colonnes[0].nombre, 30);
    assert_eq!(colonnes[0].description, sacs[0].description);
    // Un autre rendement : un autre article.
    let autres = achats("colonnes", &["10", "4", "6", "10", "20"]);
    assert_ne!(autres[0].description, sacs[0].description);
}

#[test]
fn ce_qui_ne_s_achete_pas_n_y_va_pas() {
    // Le volume d'une dalle n'est pas un nombre d'articles.
    assert!(achats("dalle", &["24", "20", "4", "10"]).is_empty());
    // Les rangs d'un muret ne s'achètent pas; ses blocs, oui.
    let muret = achats("muret", &["20", "3", "16", "8"]);
    assert_eq!(muret.len(), 1);
    assert!(muret[0].description.starts_with("Blocs de muret · "));
}

#[test]
fn la_ceramique_en_boites_ou_en_tuiles() {
    let en_boites = achats("ceramique", &["10", "8", "12", "12", "10", "11"]);
    assert_eq!(en_boites.len(), 1);
    assert!(en_boites[0].description.starts_with("Boîtes de tuiles · "));
    let en_tuiles = achats("ceramique", &["10", "8", "12", "12", "10", ""]);
    assert_eq!(en_tuiles.len(), 1);
    assert!(en_tuiles[0].description.starts_with("Tuiles · "));
}
