//! Manuel d'utilisation : `app/manuel.html` en français, `app/manual.html` en
//! anglais (section 24).
//!
//! Chaque page est produite à partir d'un gabarit (`app/outils/manuel-gabarit.html`,
//! `app/outils/manuel-gabarit-en.html`) et du contenu même du programme : les
//! fiches d'aide, les trucs de pro, les calculateurs par métier et, pour chacun,
//! un exemple calculé par le noyau. Elle ne peut donc pas s'écarter de ce que
//! fait la calculatrice.
//!
//! Ce module n'existe que pour les tests : l'un vérifie que les pages publiées
//! sont à jour, l'autre les refait.
//!
//! ```text
//! cargo test -p calculatrice-chantier regenerer_le_manuel -- --ignored
//! ```

use std::path::{Path, PathBuf};

use noyau::Preferences;
use noyau::langue::{self, Langue, selon};
use noyau::metiers::{Calculateur, Metier, Nature, calculateur};

use crate::aide::{FICHES, Fiche};
use crate::textes;

/// Le gabarit et la page publiée, dans une langue.
fn gabarit(langue: Langue) -> &'static str {
    match langue {
        Langue::Francais => include_str!("../outils/manuel-gabarit.html"),
        Langue::Anglais => include_str!("../outils/manuel-gabarit-en.html"),
    }
}

fn page_publiee(langue: Langue) -> &'static str {
    match langue {
        Langue::Francais => include_str!("../manuel.html"),
        Langue::Anglais => include_str!("../manual.html"),
    }
}

/// Un exemple par calculateur : les saisies d'une valeur d'essai du cahier des
/// charges (section 22.14), vérifiée par les tests du noyau. Elles sont
/// écrites à la française; le manuel anglais les récrit à l'anglaise.
const EXEMPLES: &[(&str, &[&str])] = &[
    ("soumission", &["100000", "3", "12", "15", "5", "9,975"]),
    (
        "budget-au-pied-carre",
        &["1400", "200", "400", "100", "3", "12", "15", "5", "9,975"],
    ),
    ("main-d-oeuvre", &["3", "40", "45,50", "30"]),
    ("taxes", &["1000", "0", "5", "9,975"]),
    ("majoration-et-marge", &["800", "25"]),
    ("dalle", &["24", "20", "4", "10"]),
    ("semelle", &["120", "20", "8", "10"]),
    ("mur-de-fondation", &["120", "8", "8", "10"]),
    ("colonnes", &["10", "4", "6", "10", "14"]),
    ("armature", &["24", "20", "16", "0"]),
    ("pente-de-drain", &["40", "1/4"]),
    ("deport", &["12", "3"]),
    ("deport-roule", &["8", "6", "3"]),
    ("contenance-de-tuyau", &["4", "10"]),
    ("pression-d-eau", &["30"]),
    ("reservoir", &["20", "48"]),
    ("loi-d-ohm", &["120", "15", "", ""]),
    ("chute-de-tension", &["120", "15", "100", "0", "0", "0"]),
    ("puissance", &["240", "30", "1", "0"]),
    ("cintrage", &["6", "2"]),
    ("cout-d-energie", &["1500", "8", "30", "10"]),
    ("peinture", &["12", "10", "8", "1", "1", "0", "2", "350"]),
    ("ceramique", &["10", "8", "24", "12", "10", "8"]),
    ("revetement-de-sol", &["15", "12", "10", "22,5"]),
    ("conduit-rond", &["8", "700"]),
    ("conduit-rectangulaire", &["14", "8", "900"]),
    ("puissance-thermique", &["36000", "0"]),
    ("excavation", &["30", "20", "4", "25"]),
    ("granulats", &["40", "12", "6", "1,5"]),
    ("cloture", &["100", "8"]),
    ("calibre-pour-chute", &["120", "20", "100", "3", "0", "0"]),
    ("resistances", &["100", "220", "470", ""]),
    ("transformateur", &["75", "600", "208", "1"]),
    ("moteur", &["10", "575", "90", "0,85", "1"]),
    ("remplissage-de-conduit", &["0,824", "0,111", "9"]),
    ("eclairage", &["30", "20", "300", "4000", "0,6", "0,8"]),
    ("debit-et-vitesse", &["0,785", "8", "0"]),
    ("longueur-de-coupe", &["48", "1 1/8", "1 1/8", "3/4"]),
    ("deports-paralleles", &["12", "6", "3"]),
    ("dilatation", &["50", "60", "0,017"]),
    ("chauffe-eau", &["180", "45", "4,5", "100"]),
    ("sacs-de-beton", &["4", "4", "4", "10", "14"]),
    ("escalier-de-beton", &["5", "7", "11", "4", "5", "10"]),
    ("treillis", &["24", "20", "8", "4", "6"]),
    ("peinture-d-une-surface", &["40", "8", "2", "350"]),
    ("moulures", &["12", "10", "1", "32", "10", "8"]),
    ("gypse", &["12", "10", "8", "1", "1", "1", "0", "10"]),
    ("montants-de-cloison", &["12", "16", "0", "1"]),
    ("isolant", &["40", "8", "5", "78,3"]),
    ("valeur-r", &["24,5", "0"]),
    ("plafond-suspendu", &["20", "15", "0", "10"]),
    ("bardeaux", &["40", "28", "6", "10", "33,3"]),
    ("pente-de-toit", &["6", "0"]),
    ("panneaux", &["24", "16", "0", "10"]),
    ("revetement-exterieur", &["120", "9", "150", "10", "200"]),
    ("membrane", &["40", "14", "36", "65", "4", "10"]),
    (
        "briques-et-blocs",
        &["22", "8", "0", "15 5/8", "7 5/8", "3/8", "5"],
    ),
    ("rangs-de-maconnerie", &["8 pi 1 po", "7 5/8", "3/8"]),
    ("muret", &["21 pi 6 po", "40 po", "12", "6"]),
    ("paves", &["20", "12", "7 7/8", "3 15/16", "5", "1"]),
    ("plaque", &["8", "4", "1/4", "0", "1"]),
    ("barre-ronde", &["1", "", "12", "0", "1"]),
    ("profile-rectangulaire", &["2", "2", "1/8", "20", "0", "1"]),
    ("cordon-de-soudure", &["1/4", "100", "65"]),
    ("conduit-pour-un-debit", &["400", "800"]),
    ("changements-d-air", &["12", "10", "8", "6"]),
    ("chaleur-de-l-air", &["1000", "20", "1"]),
    ("plancher-chauffant", &["30", "20", "9", "300"]),
    ("tranchee", &["100", "2", "6", "4", "25"]),
    ("voyages-de-camion", &["74", "12"]),
    ("pente-de-terrain", &["50", "", "2"]),
    ("terrasse", &["16", "12", "5 1/2", "1/8", "16", "10"]),
    ("tourbe", &["50", "30", "10", "5"]),
    ("poteaux", &["10", "4", "0", "3 1/2", "12", "14"]),
    (
        "facturation-progressive",
        &["100000", "40", "10", "20000", "5", "9,975"],
    ),
    ("duree-des-travaux", &["1200", "50", "2", "8"]),
];

/// Remplace les caractères qui ont un sens en HTML.
fn echapper(texte: &str) -> String {
    texte
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Texte d'aide : les passages entre accents graves deviennent des touches.
fn avec_touches(texte: &str) -> String {
    let mut html = String::new();
    for (i, morceau) in texte.split('`').enumerate() {
        if i % 2 == 1 {
            html.push_str("<kbd>");
            html.push_str(&echapper(morceau));
            html.push_str("</kbd>");
        } else {
            html.push_str(&echapper(morceau));
        }
    }
    html
}

/// Identifiant d'ancre : des minuscules sans accents, séparées par des tirets.
fn ancre(texte: &str) -> String {
    let mut ancre = String::new();
    for c in texte.to_lowercase().chars() {
        let sans_accent = match c {
            'à' | 'â' => 'a',
            'ç' => 'c',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'î' | 'ï' => 'i',
            'ô' => 'o',
            'ù' | 'û' => 'u',
            autre => autre,
        };
        if sans_accent.is_ascii_alphanumeric() {
            ancre.push(sans_accent);
        } else if !ancre.ends_with('-') {
            ancre.push('-');
        }
    }
    ancre.trim_matches('-').to_string()
}

/// L'ancre d'un métier : la même dans les deux langues, d'après son nom français.
fn ancre_du_metier(metier: Metier) -> String {
    let _francais = langue::pour_la_portee(Langue::Francais);
    format!("metier-{}", ancre(metier.nom_court()))
}

/// Un libellé et sa valeur : le deux-points est précédé d'une espace en
/// français seulement.
fn deux_points(libelle: &str, valeur: &str) -> String {
    format!("{libelle}{} {valeur}", selon(" :", ":"))
}

/// Les entrées du sommaire : un lien par métier.
fn sommaire_des_metiers() -> String {
    Metier::TOUS
        .iter()
        .map(|metier| {
            format!(
                "                <li><a href=\"#{}\">{}</a></li>\n",
                ancre_du_metier(*metier),
                echapper(metier.nom())
            )
        })
        .collect::<String>()
        .trim_end()
        .to_string()
}

/// Les trucs de pro de l'application.
fn trucs() -> String {
    textes::TRUCS
        .iter()
        .map(|(titre, texte)| {
            format!(
                "    <li><strong>{}</strong> {}</li>\n",
                echapper(titre.traduit()),
                echapper(texte.traduit())
            )
        })
        .collect::<String>()
        .trim_end()
        .to_string()
}

/// Une fiche d'aide.
fn fiche(f: &Fiche) -> String {
    let mut html = String::from("<article>\n");
    html.push_str(&format!("    <h4>{}", echapper(f.titre.traduit())));
    if !f.acces.est_vide() {
        html.push_str(&format!(
            " <span class=\"acces\">{}</span>",
            avec_touches(f.acces.traduit())
        ));
    }
    html.push_str("</h4>\n");
    html.push_str(&format!(
        "    <p><strong>{}</strong> {}</p>\n",
        avec_touches(f.resume.traduit()),
        avec_touches(f.mode_emploi.traduit())
    ));
    if let Some((frappes, resultat)) = f.exemple {
        html.push_str(&format!(
            "    <p class=\"exemple\"><span class=\"mot\">{}</span> {} → <strong>{}</strong></p>\n",
            textes::EXEMPLE.traduit(),
            avec_touches(frappes.traduit()),
            echapper(resultat.traduit())
        ));
    }
    if let Some(note) = f.note {
        html.push_str(&format!(
            "    <p class=\"note\">{}</p>\n",
            avec_touches(note.traduit())
        ));
    }
    html.push_str("</article>\n");
    html
}

/// Dit si une fiche d'aide fait partie d'un groupe du répertoire.
type DuGroupe = fn(&Fiche) -> bool;

/// Les fiches d'aide, en trois groupes.
fn fonctions() -> String {
    let groupes: [(&str, DuGroupe); 3] = [
        (selon("Les touches principales", "The main keys"), |f| {
            !f.secondaire && !f.trigo && !f.touches.is_empty()
        }),
        (
            selon(
                "Les secondes fonctions, avec Conv",
                "The second functions, with Conv",
            ),
            |f| f.secondaire,
        ),
        (selon("Les autres fonctions", "Other functions"), |f| {
            !f.secondaire && (f.trigo || f.touches.is_empty())
        }),
    ];
    let mut html = String::new();
    for (titre, du_groupe) in groupes {
        html.push_str(&format!("<h3>{titre}</h3>\n"));
        for f in FICHES.iter().filter(|f| du_groupe(f)) {
            html.push_str(&fiche(f));
        }
    }
    html.trim_end().to_string()
}

/// Les saisies de l'exemple d'un calculateur, telles qu'elles se tapent dans
/// la langue en cours.
fn exemple_de(c: &Calculateur, prefs: &Preferences) -> Option<Vec<String>> {
    let (_, saisies) = EXEMPLES.iter().find(|(id, _)| *id == c.id)?;
    let mut saisies: Vec<String> = saisies.iter().map(|s| (*s).to_string()).collect();
    c.transposer_les_saisies(&mut saisies, Langue::Francais, langue::en_cours(), prefs);
    Some(saisies)
}

/// Le tableau des champs d'un calculateur : ce sont aussi ses touches.
fn champs(c: &Calculateur, prefs: &Preferences) -> String {
    // Les valeurs de départ, remises au propre par le calculateur lui-même.
    let depart = c.evaluer(&c.saisies_par_defaut(), prefs);
    let mut au_propre = depart.entrees.iter();
    let mut html = format!(
        "    <div class=\"table\"><table>\n        <thead><tr><th>{}</th><th>{}</th><th>{}</th></tr></thead>\n        <tbody>\n",
        selon("Champ", "Field"),
        selon("Unité", "Unit"),
        selon("Au départ", "Default"),
    );
    for champ in c.champs {
        let libelle = champ.libelle.traduit();
        let mut nom = echapper(libelle);
        let court = champ.libelle_court().replace('\n', " ");
        if court != libelle {
            nom.push_str(&format!(
                " <span class=\"acces\">({})</span>",
                deux_points(selon("touche", "key"), &echapper(&court))
            ));
        }
        if champ.facultatif {
            nom.push_str(&format!(
                " <span class=\"acces\">— {}</span>",
                selon("facultatif", "optional")
            ));
        }
        let (unite, valeur) = match champ.nature {
            Nature::Choix(options) => {
                au_propre.next();
                let choisi = champ.defaut.parse::<usize>().unwrap_or(0);
                let liste = options
                    .iter()
                    .enumerate()
                    .map(|(rang, option)| {
                        if rang == choisi {
                            format!("<strong>{}</strong>", echapper(option.traduit()))
                        } else {
                            echapper(option.traduit())
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" · ");
                (selon("liste", "list").to_string(), liste)
            }
            nature => {
                let entier = selon("nombre entier", "whole number");
                let unite = match nature {
                    Nature::Entier(unite) if unite.est_vide() => entier.to_string(),
                    Nature::Entier(unite) => format!("{}, {entier}", unite.traduit()),
                    _ => champ.unite().to_string(),
                };
                let valeur = if champ.defaut.is_empty() {
                    String::new()
                } else {
                    au_propre
                        .next()
                        .map(|(_, texte)| echapper(texte))
                        .unwrap_or_default()
                };
                (echapper(&unite), valeur)
            }
        };
        html.push_str(&format!(
            "            <tr><td>{nom}</td><td>{unite}</td><td>{valeur}</td></tr>\n"
        ));
    }
    html.push_str("        </tbody>\n    </table></div>\n");
    html
}

/// Un outil : son résumé, ses champs, un exemple calculé et sa note.
fn outil(c: &Calculateur, prefs: &Preferences) -> String {
    let mut html = format!("<article id=\"outil-{}\">\n", c.id);
    html.push_str(&format!("    <h4>{}</h4>\n", echapper(c.nom.traduit())));
    html.push_str(&format!("    <p>{}</p>\n", echapper(c.resume.traduit())));
    html.push_str(&champs(c, prefs));
    if let Some(saisies) = exemple_de(c, prefs) {
        let e = c.evaluer(&saisies, prefs);
        let liste = |lignes: &[(&str, String)], gras: bool| {
            lignes
                .iter()
                .map(|(libelle, valeur)| {
                    if gras {
                        deux_points(
                            &echapper(libelle),
                            &format!("<strong>{}</strong>", echapper(valeur)),
                        )
                    } else {
                        deux_points(&echapper(libelle), &echapper(valeur))
                    }
                })
                .collect::<Vec<_>>()
                .join(" · ")
        };
        html.push_str(&format!(
            "    <p class=\"exemple\"><span class=\"mot\">{}</span> {} → {}</p>\n",
            textes::EXEMPLE.traduit(),
            liste(&e.entrees, false),
            liste(&e.resultats, true)
        ));
    }
    if !c.note.est_vide() {
        html.push_str(&format!(
            "    <p class=\"note\">{}</p>\n",
            echapper(c.note.traduit())
        ));
    }
    html.push_str("</article>\n");
    html
}

/// Tous les outils, métier par métier.
fn outils() -> String {
    let prefs = Preferences::default();
    let mut html = String::new();
    for metier in Metier::TOUS {
        html.push_str(&format!(
            "<h3 id=\"{}\">{}</h3>\n<p class=\"note\">{}.</p>\n",
            ancre_du_metier(metier),
            echapper(metier.nom()),
            echapper(metier.resume())
        ));
        for c in metier.calculateurs() {
            html.push_str(&outil(c, &prefs));
        }
    }
    html.trim_end().to_string()
}

/// La page complète, dans une langue.
fn generer(langue: Langue) -> String {
    let _langue = langue::pour_la_portee(langue);
    let nombre_d_outils: usize = Metier::TOUS
        .iter()
        .map(|metier| metier.calculateurs().len())
        .sum();
    let page = gabarit(langue)
        .replace("\r\n", "\n")
        .replace("[[NOMBRE_D_OUTILS]]", &nombre_d_outils.to_string())
        .replace("[[NOMBRE_DE_METIERS]]", &Metier::TOUS.len().to_string())
        .replace("[[SOMMAIRE_DES_METIERS]]", &sommaire_des_metiers())
        .replace("[[TRUCS]]", &trucs())
        .replace("[[FONCTIONS]]", &fonctions())
        .replace("[[OUTILS]]", &outils());
    // Le commentaire du gabarit ne concerne que le gabarit.
    match (page.find("<!--"), page.find("-->")) {
        (Some(debut), Some(fin)) if debut < fin => {
            format!(
                "{}{}",
                &page[..debut],
                page[fin + 3..].trim_start_matches('\n')
            )
        }
        _ => page,
    }
}

fn chemin_du_manuel(langue: Langue) -> PathBuf {
    let _langue = langue::pour_la_portee(langue);
    Path::new(env!("CARGO_MANIFEST_DIR")).join(textes::FICHIER_DU_MANUEL.traduit())
}

#[test]
fn chaque_calculateur_a_son_exemple() {
    let prefs = Preferences::default();
    for langue in Langue::TOUTES {
        let _langue = langue::pour_la_portee(langue);
        for metier in Metier::TOUS {
            for c in metier.calculateurs() {
                let saisies =
                    exemple_de(c, &prefs).unwrap_or_else(|| panic!("pas d'exemple pour {}", c.id));
                assert_eq!(saisies.len(), c.champs.len(), "{}", c.id);
                let e = c.evaluer(&saisies, &prefs);
                assert!(
                    e.erreurs.iter().all(Option::is_none),
                    "{langue:?}, {}",
                    c.id
                );
                assert_eq!(e.avis, None, "{langue:?}, {}", c.id);
                assert!(!e.resultats.is_empty(), "{langue:?}, {}", c.id);
            }
        }
    }
    // Aucun exemple orphelin.
    for (id, _) in EXEMPLES {
        assert!(calculateur(id).is_some(), "calculateur inconnu : {id}");
    }
}

#[test]
fn les_exemples_donnent_les_memes_grandeurs_dans_les_deux_langues() {
    let prefs = Preferences::default();
    for metier in Metier::TOUS {
        for c in metier.calculateurs() {
            let evaluer = |langue: Langue| {
                let _langue = langue::pour_la_portee(langue);
                let saisies = exemple_de(c, &prefs).unwrap_or_default();
                let e = c.evaluer(&saisies, &prefs);
                e.quantites
                    .iter()
                    .map(|quantite| quantite.valeur(e.modele))
                    .collect::<Vec<_>>()
            };
            assert_eq!(
                evaluer(Langue::Francais),
                evaluer(Langue::Anglais),
                "{}",
                c.id
            );
        }
    }
}

#[test]
fn le_manuel_est_complet() {
    for langue in Langue::TOUTES {
        let page = generer(langue);
        let _langue = langue::pour_la_portee(langue);
        assert!(
            !page.contains("[["),
            "un repère du gabarit n'a pas été remplacé"
        );
        assert!(!page.contains("<!--"));
        for metier in Metier::TOUS {
            assert!(page.contains(&echapper(metier.nom())));
            for c in metier.calculateurs() {
                assert!(page.contains(&format!("id=\"outil-{}\"", c.id)), "{}", c.id);
                assert!(page.contains(&echapper(c.nom.traduit())), "{}", c.id);
            }
        }
        for f in FICHES {
            assert!(
                page.contains(&echapper(f.titre.traduit())),
                "{}",
                f.titre.traduit()
            );
        }
        // Les ancres du sommaire mènent quelque part.
        for cible in page.split("href=\"#").skip(1) {
            let cible = cible.split('"').next().unwrap_or_default();
            assert!(
                page.contains(&format!("id=\"{cible}\"")),
                "ancre sans cible : {cible}"
            );
        }
    }
    // Chaque manuel mène à l'autre.
    assert!(generer(Langue::Francais).contains("href=\"manual.html\""));
    assert!(generer(Langue::Anglais).contains("href=\"manuel.html\""));
}

/// Ce qui suit chaque occurrence d'un repère, jusqu'à un guillemet : la liste
/// des identifiants d'une page, par exemple.
fn valeurs_apres<'a>(page: &'a str, repere: &str) -> Vec<&'a str> {
    page.split(repere)
        .skip(1)
        .map(|suite| suite.split('"').next().unwrap_or_default())
        .collect()
}

#[test]
fn les_deux_gabarits_se_suivent() {
    let francais = gabarit(Langue::Francais).replace("\r\n", "\n");
    let anglais = gabarit(Langue::Anglais).replace("\r\n", "\n");
    // La même feuille de style.
    let style = |page: &str| {
        let debut = page.find("<style>").expect("feuille de style");
        let fin = page.find("</style>").expect("feuille de style");
        page[debut..fin].to_string()
    };
    assert!(
        style(&francais) == style(&anglais),
        "feuilles de style différentes"
    );
    // Les mêmes chapitres, les mêmes repères et autant de touches.
    assert_eq!(
        valeurs_apres(&francais, "id=\""),
        valeurs_apres(&anglais, "id=\"")
    );
    let reperes = |page: &str| -> Vec<String> {
        page.split("[[")
            .skip(1)
            .map(|suite| suite.split("]]").next().unwrap_or_default().to_string())
            .collect()
    };
    assert_eq!(reperes(&francais), reperes(&anglais));
    for balise in ["<kbd>", "<li>", "<tr>", "<h3>", "<p", "class=\"ecran\""] {
        assert_eq!(
            francais.matches(balise).count(),
            anglais.matches(balise).count(),
            "{balise}"
        );
    }
    // Les mêmes nombres : seule leur écriture change d'une langue à l'autre.
    let chiffres = |page: &str| -> String { page.chars().filter(char::is_ascii_digit).collect() };
    let (a, b) = (chiffres(&francais), chiffres(&anglais));
    if a != b {
        let commun = a.bytes().zip(b.bytes()).take_while(|(x, y)| x == y).count();
        let suite = |texte: &str| texte[commun..(commun + 24).min(texte.len())].to_string();
        panic!(
            "les nombres diffèrent après {commun} chiffres : « {} » en français, « {} » en anglais",
            suite(&a),
            suite(&b)
        );
    }
}

#[test]
fn chaque_gabarit_cite_les_messages_de_son_afficheur() {
    use noyau::Erreur;
    for langue in Langue::TOUTES {
        let _langue = langue::pour_la_portee(langue);
        let page = gabarit(langue);
        for erreur in [
            Erreur::Depassement,
            Erreur::Calcul,
            Erreur::Unites,
            Erreur::Saisie,
            Erreur::Trigonometrie,
            Erreur::AucuneValeur,
        ] {
            assert!(
                page.contains(erreur.texte()),
                "{langue:?} : {}",
                erreur.texte()
            );
        }
        // « IL MANQUE : LARGEUR », « MISSING: WIDTH ».
        let manque = noyau::erreurs::valeur_manquante(selon("Largeur", "Width"));
        assert!(page.contains(&manque), "{langue:?} : {manque}");
        let a_corriger = noyau::erreurs::valeur_a_corriger("…");
        assert!(page.contains(&a_corriger), "{langue:?} : {a_corriger}");
        // Les indicateurs de l'afficheur et le nom de la calculatrice.
        for texte in [
            textes::INDICATEUR_STOCKER,
            textes::INDICATEUR_RAPPEL,
            textes::INDICATEUR_MEMORISE,
            textes::NOM_APPLICATION,
            textes::GRACIEUSETE,
        ] {
            assert!(page.contains(texte.traduit()), "{langue:?} : {texte}");
        }
    }
}

#[test]
fn le_manuel_est_a_jour() {
    for langue in Langue::TOUTES {
        assert!(
            page_publiee(langue).replace("\r\n", "\n") == generer(langue),
            "{} n'est plus à jour : relancez \
             `cargo test -p calculatrice-chantier regenerer_le_manuel -- --ignored`",
            chemin_du_manuel(langue).display()
        );
    }
}

/// Refait `app/manuel.html` et `app/manual.html`. À lancer après avoir modifié
/// un gabarit, une fiche d'aide ou un calculateur.
#[test]
#[ignore = "refait app/manuel.html et app/manual.html"]
fn regenerer_le_manuel() {
    for langue in Langue::TOUTES {
        let chemin = chemin_du_manuel(langue);
        std::fs::write(&chemin, generer(langue))
            .unwrap_or_else(|erreur| panic!("écriture de {} : {erreur}", chemin.display()));
    }
}
