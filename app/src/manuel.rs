//! Manuel d'utilisation (`app/manuel.html`).
//!
//! La page est produite à partir d'un gabarit (`app/outils/manuel-gabarit.html`)
//! et du contenu même du programme : les fiches d'aide, les trucs de pro, les
//! calculateurs par métier et, pour chacun, un exemple calculé par le noyau.
//! Elle ne peut donc pas s'écarter de ce que fait la calculatrice.
//!
//! Ce module n'existe que pour les tests : l'un vérifie que la page publiée est
//! à jour, l'autre la refait.
//!
//! ```text
//! cargo test -p calculatrice-chantier regenerer_le_manuel -- --ignored
//! ```

use std::path::{Path, PathBuf};

use noyau::Preferences;
use noyau::metiers::{Calculateur, Metier, Nature, calculateur};

use crate::aide::{FICHES, Fiche};
use crate::textes;

const GABARIT: &str = include_str!("../outils/manuel-gabarit.html");
const MANUEL: &str = include_str!("../manuel.html");

/// Un exemple par calculateur : les saisies d'une valeur d'essai du cahier des
/// charges (section 22.14), vérifiée par les tests du noyau.
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

fn ancre_du_metier(metier: Metier) -> String {
    format!("metier-{}", ancre(metier.nom_court()))
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
                echapper(titre),
                echapper(texte)
            )
        })
        .collect::<String>()
        .trim_end()
        .to_string()
}

/// Une fiche d'aide.
fn fiche(f: &Fiche) -> String {
    let mut html = String::from("<article>\n");
    html.push_str(&format!("    <h4>{}", echapper(f.titre)));
    if !f.acces.is_empty() {
        html.push_str(&format!(
            " <span class=\"acces\">{}</span>",
            avec_touches(f.acces)
        ));
    }
    html.push_str("</h4>\n");
    html.push_str(&format!(
        "    <p><strong>{}</strong> {}</p>\n",
        avec_touches(f.resume),
        avec_touches(f.mode_emploi)
    ));
    if let Some((frappes, resultat)) = f.exemple {
        html.push_str(&format!(
            "    <p class=\"exemple\"><span class=\"mot\">{}</span> {} → <strong>{}</strong></p>\n",
            textes::EXEMPLE,
            avec_touches(frappes),
            echapper(resultat)
        ));
    }
    if let Some(note) = f.note {
        html.push_str(&format!(
            "    <p class=\"note\">{}</p>\n",
            avec_touches(note)
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
        ("Les touches principales", |f| {
            !f.secondaire && !f.trigo && !f.touches.is_empty()
        }),
        ("Les secondes fonctions, avec Conv", |f| f.secondaire),
        ("Les autres fonctions", |f| {
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

/// Les saisies de l'exemple d'un calculateur.
fn exemple_de(id: &str) -> Option<Vec<String>> {
    EXEMPLES
        .iter()
        .find(|(identifiant, _)| *identifiant == id)
        .map(|(_, saisies)| saisies.iter().map(|s| (*s).to_string()).collect())
}

/// Le tableau des champs d'un calculateur : ce sont aussi ses touches.
fn champs(c: &Calculateur, prefs: &Preferences) -> String {
    // Les valeurs de départ, remises au propre par le calculateur lui-même.
    let depart = c.evaluer(&c.saisies_par_defaut(), prefs);
    let mut au_propre = depart.entrees.iter();
    let mut html = String::from(
        "    <div class=\"table\"><table>\n        <thead><tr><th>Champ</th><th>Unité</th><th>Au départ</th></tr></thead>\n        <tbody>\n",
    );
    for champ in c.champs {
        let mut nom = echapper(champ.libelle);
        let court = champ.libelle_court().replace('\n', " ");
        if court != champ.libelle {
            nom.push_str(&format!(
                " <span class=\"acces\">(touche : {})</span>",
                echapper(&court)
            ));
        }
        if champ.facultatif {
            nom.push_str(" <span class=\"acces\">— facultatif</span>");
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
                            format!("<strong>{}</strong>", echapper(option))
                        } else {
                            echapper(option)
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" · ");
                ("liste".to_string(), liste)
            }
            nature => {
                let unite = match nature {
                    Nature::Entier("") => "nombre entier".to_string(),
                    Nature::Entier(unite) => format!("{unite}, nombre entier"),
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
    html.push_str(&format!("    <h4>{}</h4>\n", echapper(c.nom)));
    html.push_str(&format!("    <p>{}</p>\n", echapper(c.resume)));
    html.push_str(&champs(c, prefs));
    if let Some(saisies) = exemple_de(c.id) {
        let e = c.evaluer(&saisies, prefs);
        let liste = |lignes: &[(&str, String)], gras: bool| {
            lignes
                .iter()
                .map(|(libelle, valeur)| {
                    if gras {
                        format!(
                            "{} : <strong>{}</strong>",
                            echapper(libelle),
                            echapper(valeur)
                        )
                    } else {
                        format!("{} : {}", echapper(libelle), echapper(valeur))
                    }
                })
                .collect::<Vec<_>>()
                .join(" · ")
        };
        html.push_str(&format!(
            "    <p class=\"exemple\"><span class=\"mot\">{}</span> {} → {}</p>\n",
            textes::EXEMPLE,
            liste(&e.entrees, false),
            liste(&e.resultats, true)
        ));
    }
    if !c.note.is_empty() {
        html.push_str(&format!("    <p class=\"note\">{}</p>\n", echapper(c.note)));
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

/// La page complète.
fn generer() -> String {
    let nombre_d_outils: usize = Metier::TOUS
        .iter()
        .map(|metier| metier.calculateurs().len())
        .sum();
    let page = GABARIT
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

fn chemin_du_manuel() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("manuel.html")
}

#[test]
fn chaque_calculateur_a_son_exemple() {
    let prefs = Preferences::default();
    for metier in Metier::TOUS {
        for c in metier.calculateurs() {
            let saisies = exemple_de(c.id).unwrap_or_else(|| panic!("pas d'exemple pour {}", c.id));
            assert_eq!(saisies.len(), c.champs.len(), "{}", c.id);
            let e = c.evaluer(&saisies, &prefs);
            assert!(e.erreurs.iter().all(Option::is_none), "{}", c.id);
            assert_eq!(e.avis, None, "{}", c.id);
            assert!(!e.resultats.is_empty(), "{}", c.id);
        }
    }
    // Aucun exemple orphelin.
    for (id, _) in EXEMPLES {
        assert!(calculateur(id).is_some(), "calculateur inconnu : {id}");
    }
}

#[test]
fn le_manuel_est_complet() {
    let page = generer();
    assert!(
        !page.contains("[["),
        "un repère du gabarit n'a pas été remplacé"
    );
    assert!(!page.contains("<!--"));
    for metier in Metier::TOUS {
        assert!(page.contains(&echapper(metier.nom())));
        for c in metier.calculateurs() {
            assert!(page.contains(&format!("id=\"outil-{}\"", c.id)), "{}", c.id);
        }
    }
    for f in FICHES {
        assert!(page.contains(&echapper(f.titre)), "{}", f.titre);
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

#[test]
fn le_manuel_est_a_jour() {
    assert!(
        MANUEL.replace("\r\n", "\n") == generer(),
        "app/manuel.html n'est plus à jour : relancez \
         `cargo test -p calculatrice-chantier regenerer_le_manuel -- --ignored`"
    );
}

/// Refait `app/manuel.html`. À lancer après avoir modifié le gabarit, une fiche
/// d'aide ou un calculateur.
#[test]
#[ignore = "refait app/manuel.html"]
fn regenerer_le_manuel() {
    std::fs::write(chemin_du_manuel(), generer()).expect("écriture de app/manuel.html");
}
