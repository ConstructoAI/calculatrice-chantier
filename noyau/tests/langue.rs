//! Les deux langues de l'interface (section 24) : tout ce que le noyau écrit
//! existe en français et en anglais, et les valeurs ne changent jamais avec
//! la langue.

use noyau::langue::{self, Langue, Texte};
use noyau::metiers::{self, Calculateur, Metier, Nature};
use noyau::moteur::TOUCHE_RESULTAT;
use noyau::{Moteur, Preferences, Touche};

const RESULTAT: Touche = Touche::Outil(TOUCHE_RESULTAT);

/// Tape une suite de touches : chiffres, `,`, `/`, `'` (Pieds), `"` (Pouces),
/// `m` (Mètres).
fn taper(moteur: &mut Moteur, frappes: &str) {
    for c in frappes.chars() {
        let touche = match c {
            '0'..='9' => Touche::Chiffre(c as u8 - b'0'),
            ',' => Touche::Virgule,
            '/' => Touche::Fraction,
            '\'' => Touche::Pieds,
            '"' => Touche::Pouces,
            'm' => Touche::Metres,
            autre => panic!("frappe inconnue : {autre}"),
        };
        moteur.appuyer(touche);
    }
}

/// Tape une valeur, puis appuie sur la touche d'un champ d'outil.
fn donner(moteur: &mut Moteur, frappes: &str, rang: u8) {
    taper(moteur, frappes);
    moteur.appuyer(Touche::Outil(rang));
}

/// L'afficheur : (étiquette, valeur), espaces insécables écrites comme des espaces.
fn afficheur(moteur: &Moteur) -> (String, String) {
    let ecran = moteur.ecran();
    let valeur = match ecran.message {
        Some(message) => message,
        None => ecran.valeur.en_texte(),
    };
    (
        ecran.etiquette.unwrap_or_default(),
        valeur.replace('\u{a0}', " "),
    )
}

fn attendre(moteur: &Moteur, etiquette: &str, valeur: &str) {
    assert_eq!(
        afficheur(moteur),
        (etiquette.to_string(), valeur.to_string())
    );
}

fn saisies(textes: &[&str]) -> Vec<String> {
    textes.iter().map(|texte| (*texte).to_string()).collect()
}

/// Les couples (libellé, valeur), espaces insécables écrites comme des espaces.
fn au_propre(couples: &[(&'static str, String)]) -> Vec<(String, String)> {
    couples
        .iter()
        .map(|(libelle, valeur)| ((*libelle).to_string(), valeur.replace('\u{a0}', " ")))
        .collect()
}

fn couples(attendus: &[(&str, &str)]) -> Vec<(String, String)> {
    attendus
        .iter()
        .map(|(libelle, valeur)| ((*libelle).to_string(), (*valeur).to_string()))
        .collect()
}

fn tous_les_calculateurs() -> impl Iterator<Item = &'static Calculateur> {
    Metier::TOUS
        .into_iter()
        .flat_map(|metier| metier.calculateurs())
}

/// Les textes écrits d'avance d'un calculateur : (ce que c'est, le texte).
fn textes_du_calculateur(c: &Calculateur) -> Vec<(&'static str, Texte)> {
    let mut textes = vec![("nom", c.nom), ("résumé", c.resume), ("note", c.note)];
    for champ in c.champs {
        textes.push(("libellé", champ.libelle));
        textes.push(("libellé court", champ.court));
        match champ.nature {
            Nature::Nombre(unite) | Nature::Entier(unite) => textes.push(("unité", unite)),
            Nature::Choix(options) => {
                textes.extend(options.iter().map(|option| ("option", *option)));
            }
            Nature::Longueur(_) => {}
        }
    }
    textes
}

/// Vrai si un texte anglais contient une lettre accentuée du français.
/// « Québec » s'écrit avec son accent dans les deux langues.
fn semble_francais(texte: &str) -> bool {
    texte
        .replace("Québec", "")
        .chars()
        .any(|c| "àâäçèéêëîïôöùûüœæÀÂÄÇÈÉÊËÎÏÔÖÙÛÜŒÆ".contains(c))
}

#[test]
fn chaque_texte_du_catalogue_existe_dans_les_deux_langues() {
    for c in tous_les_calculateurs() {
        for (quoi, texte) in textes_du_calculateur(c) {
            // Une note, une unité ou un libellé court peuvent manquer, mais
            // seul un libellé court peut n'exister que dans une langue.
            if quoi != "libellé court" {
                assert_eq!(
                    texte.fr.is_empty(),
                    texte.en.is_empty(),
                    "{} : {quoi} {texte:?}",
                    c.id
                );
            }
            if matches!(quoi, "nom" | "résumé" | "libellé" | "option") {
                assert!(!texte.en.is_empty(), "{} : {quoi} {texte:?}", c.id);
            }
        }
    }
}

#[test]
fn aucun_texte_anglais_n_est_reste_en_francais() {
    for c in tous_les_calculateurs() {
        for (quoi, texte) in textes_du_calculateur(c) {
            assert!(
                !semble_francais(texte.en),
                "{} : {quoi} « {} »",
                c.id,
                texte.en
            );
        }
    }
    // Ce que les calculs écrivent : libellés des résultats, unités, avis.
    let _anglais = langue::pour_la_portee(Langue::Anglais);
    let prefs = Preferences::default();
    for metier in Metier::TOUS {
        for texte in [metier.nom(), metier.nom_court(), metier.resume()] {
            assert!(!semble_francais(texte), "{texte}");
        }
    }
    for c in tous_les_calculateurs() {
        // Deux essais : les valeurs de départ, puis tous les champs remplis.
        let depart = c.saisies_par_defaut();
        let remplis: Vec<String> = c
            .champs
            .iter()
            .zip(&depart)
            .map(|(champ, saisie)| match champ.nature {
                Nature::Choix(_) => saisie.clone(),
                _ if saisie.is_empty() && !champ.facultatif => "10".to_string(),
                _ => saisie.clone(),
            })
            .collect();
        let pleins: Vec<String> = c
            .champs
            .iter()
            .zip(&remplis)
            .map(|(_, saisie)| {
                if saisie.is_empty() {
                    "2".to_string()
                } else {
                    saisie.clone()
                }
            })
            .collect();
        for essai in [&depart, &remplis, &pleins] {
            let e = c.evaluer(essai, &prefs);
            let ecrits = e
                .entrees
                .iter()
                .chain(&e.resultats)
                .flat_map(|(libelle, valeur)| [(*libelle).to_string(), valeur.clone()])
                .chain(e.avis.map(str::to_string))
                .chain(e.erreurs.iter().flatten().map(|m| (*m).to_string()));
            for texte in ecrits {
                assert!(!semble_francais(&texte), "{} : « {texte} »", c.id);
                // Aucun symbole français d'unité.
                let francais = [
                    "pi", "po", "vg", "tc", "pi²", "po²", "vg²", "pi³", "po³", "vg³", "pmp",
                ];
                for mot in texte.split(|c: char| c.is_whitespace() || "()/".contains(c)) {
                    assert!(!francais.contains(&mot), "{} : « {texte} »", c.id);
                }
            }
        }
    }
}

#[test]
fn les_libelles_tiennent_sur_les_touches_dans_les_deux_langues() {
    for langue in Langue::TOUTES {
        let _langue = langue::pour_la_portee(langue);
        for c in tous_les_calculateurs() {
            for champ in c.champs {
                // Deux lignes au plus, assez courtes pour une touche étroite.
                let court = champ.libelle_court();
                let lignes: Vec<&str> = court.split('\n').collect();
                assert!(lignes.len() <= 2, "{langue:?}, {} : {court:?}", c.id);
                for ligne in lignes {
                    assert!(
                        (1..=12).contains(&ligne.chars().count()),
                        "{langue:?}, {} : {court:?}",
                        c.id
                    );
                }
                assert!(!champ.libelle_pour_message().is_empty(), "{}", c.id);
            }
        }
    }
}

#[test]
fn les_valeurs_de_depart_se_lisent_dans_les_deux_langues() {
    let prefs = Preferences::default();
    for c in tous_les_calculateurs() {
        let francais = c.evaluer(&c.saisies_par_defaut(), &prefs);
        let _anglais = langue::pour_la_portee(Langue::Anglais);
        let depart = c.saisies_par_defaut();
        // Aucune valeur de départ n'est écrite avec une virgule décimale.
        for (champ, saisie) in c.champs.iter().zip(&depart) {
            if !matches!(champ.nature, Nature::Choix(_)) {
                assert!(!saisie.contains(','), "{} : {saisie}", c.id);
            }
        }
        let anglais = c.evaluer(&depart, &prefs);
        assert!(anglais.erreurs.iter().all(Option::is_none), "{}", c.id);
        // Les mêmes grandeurs, quelle que soit la langue.
        assert_eq!(
            francais.quantites.len(),
            anglais.quantites.len(),
            "{}",
            c.id
        );
        for (a, b) in francais.quantites.iter().zip(&anglais.quantites) {
            assert_eq!(
                a.valeur(francais.modele),
                b.valeur(anglais.modele),
                "{}",
                c.id
            );
        }
    }
}

#[test]
fn resultats_en_anglais() {
    let _anglais = langue::pour_la_portee(Langue::Anglais);
    let prefs = Preferences::default();
    let evaluer = |id: &str, textes: &[&str]| {
        metiers::calculateur(id)
            .unwrap()
            .evaluer(&saisies(textes), &prefs)
    };

    let e = evaluer("dalle", &["24", "20", "4", "10"]);
    assert_eq!(
        au_propre(&e.entrees),
        couples(&[
            ("Length", "24 ft 0 in"),
            ("Width", "20 ft 0 in"),
            ("Thickness", "4 in"),
            ("Waste", "10%"),
        ])
    );
    assert_eq!(
        au_propre(&e.resultats),
        couples(&[
            ("Area", "480 ft² (44.6 m²)"),
            ("Concrete volume", "5.93 yd³ (4.53 m³)"),
            ("To order, waste included", "6.52 yd³ (4.98 m³)"),
        ])
    );

    // Les montants : le dollar devant, la virgule aux milliers, y compris
    // dans ce qui est tapé.
    let e = evaluer("soumission", &["100,000", "3", "12", "15", "5", "9.975"]);
    assert_eq!(
        au_propre(&e.entrees)[0],
        ("Base cost of the work".to_string(), "$100,000".to_string())
    );
    assert_eq!(
        au_propre(&e.resultats),
        couples(&[
            ("Administration", "$3,000.00"),
            ("Contingency", "$12,000.00"),
            ("Profit", "$15,000.00"),
            ("Price before taxes", "$130,000.00"),
            ("GST", "$6,500.00"),
            ("QST", "$12,967.50"),
            ("Price, taxes included", "$149,467.50"),
        ])
    );

    // Deux unités à la fois, et les symboles anglais.
    let e = evaluer("pression-d-eau", &["30"]);
    assert_eq!(
        au_propre(&e.resultats),
        couples(&[("Pressure", "89.7 kPa (13.0 psi)")])
    );
    let e = evaluer("conduit-rond", &["8", "900"]);
    assert_eq!(
        au_propre(&e.resultats),
        couples(&[
            ("Cross-section", "0.349 ft² (0.032 m²)"),
            ("Airflow", "314 CFM (148 L/s)"),
        ])
    );
    let e = evaluer("rangs-de-maconnerie", &["8", "7 5/8", "3/8"]);
    assert_eq!(
        au_propre(&e.resultats),
        couples(&[
            ("Course height, joint included", "8 in"),
            ("Courses", "12 courses"),
            ("Height reached", "8 ft 0 in"),
        ])
    );

    // Un avis, une saisie refusée et un choix dans une liste.
    let e = evaluer("treillis", &["20", "10", "8", "4", "60"]);
    assert_eq!(e.avis, Some("The overlap is larger than the sheet."));
    let e = evaluer("dalle", &["abc", "20", "0", "-5"]);
    assert_eq!(
        e.erreurs[0],
        Some("Enter a length, for example 12 ft 6 in, 30 in or 3.8 m.")
    );
    assert_eq!(e.erreurs[2], Some("The value must be greater than zero."));
    assert_eq!(e.erreurs[3], Some("The value cannot be negative."));
    let e = evaluer("taxes", &["100", "1", "5", "9.975"]);
    assert_eq!(
        au_propre(&e.entrees)[1],
        ("This amount is".to_string(), "taxes included".to_string())
    );

    // Les valeurs de départ s'écrivent à l'anglaise.
    let taxes = metiers::calculateur("taxes").unwrap();
    assert_eq!(taxes.saisies_par_defaut(), ["", "0", "5", "9.975"]);
    assert_eq!(taxes.champs[3].unite(), "%");
    let budget = metiers::calculateur("budget-au-pied-carre").unwrap();
    assert_eq!(budget.champs[0].unite(), "ft²");
    assert_eq!(budget.champs[1].unite(), "$/ft²");
    assert_eq!(Metier::Beton.nom(), "Concrete and foundations");
    assert_eq!(Metier::Ventilation.nom_court(), "HVAC");
}

#[test]
fn outil_au_clavier_en_anglais() {
    let _anglais = langue::pour_la_portee(Langue::Anglais);
    let mut m = Moteur::default();
    m.choisir_outil(Some("dalle"), None);
    donner(&mut m, "24'", 0);
    attendre(&m, "LENGTH", "24 ft 0 in");
    // Il manque une valeur : on dit laquelle.
    m.appuyer(RESULTAT);
    attendre(&m, "", "MISSING: WIDTH");
    donner(&mut m, "20'", 1);
    donner(&mut m, "4\"", 2);
    m.appuyer(RESULTAT);
    attendre(&m, "AREA", "480 ft² (44.6 m²)");
    assert_eq!(m.ecran().note.as_deref(), Some("1 of 3"));
    m.appuyer(RESULTAT);
    attendre(&m, "CONCRETE VOLUME", "5.93 yd³ (4.53 m³)");

    // Le ruban est écrit en anglais lui aussi.
    let lignes: Vec<String> = m
        .ruban()
        .lignes
        .iter()
        .map(|l| {
            format!(
                "{}|{}",
                l.etiquette.clone().unwrap_or_default(),
                l.texte.replace('\u{a0}', " ")
            )
        })
        .collect();
    assert!(lignes.contains(&"|Concrete slab".to_string()), "{lignes:?}");
    assert!(lignes.contains(&"Waste|10%".to_string()), "{lignes:?}");
    assert!(lignes.contains(&"Concrete volume|5.93 yd³ (4.53 m³)".to_string()));

    // Les touches de l'outil, pour l'interface.
    let touches = m.touches_d_outil().unwrap();
    assert_eq!(touches[0].libelle, "Length");
    assert_eq!(touches[0].unite, "ft");
    assert_eq!(touches[2].unite, "in");
    assert_eq!(touches[3].valeur, "10%");

    // Un nombre décimal se range avec le point, et se relit.
    taper(&mut m, "12,5");
    assert_eq!(afficheur(&m).1, "12.5");
    m.appuyer(Touche::Outil(3));
    attendre(&m, "WASTE", "12.5%");
    assert_eq!(m.saisies_d_outil().unwrap()[3], "12.5");

    // `Conv` puis la touche d'un champ : la valeur de départ, à l'anglaise.
    m.choisir_outil(Some("taxes"), None);
    donner(&mut m, "8", 3);
    m.appuyer(Touche::Conv);
    m.appuyer(Touche::Outil(3));
    attendre(&m, "QST", "9.975%");
    m.appuyer(Touche::Conv);
    m.appuyer(RESULTAT);
    attendre(&m, "", "FIELDS RESET");
}

#[test]
fn charpente_en_anglais() {
    let _anglais = langue::pour_la_portee(Langue::Anglais);
    let mut m = Moteur::default();
    taper(&mut m, "7\"");
    m.appuyer(Touche::Pente);
    attendre(&m, "PITCH", "7 in");
    assert_eq!(m.ecran().note.as_deref(), Some("in 12"));
    taper(&mut m, "7'2\"");
    m.appuyer(Touche::Course);
    attendre(&m, "RUN", "7 ft 2 in");
    m.appuyer(Touche::Diag);
    attendre(&m, "DIAGONAL", "8 ft 3 9/16 in");
    m.appuyer(Touche::Diag);
    attendre(&m, "PLUMB CUT", "30.26°");
    m.appuyer(Touche::Diag);
    attendre(&m, "LEVEL CUT", "59.74°");
    // Les messages et les erreurs.
    m.appuyer(Touche::Effacer);
    m.appuyer(Touche::Effacer);
    m.appuyer(Touche::Empannon);
    assert!(afficheur(&m).1.starts_with("NO VALUE") || afficheur(&m).1.starts_with("ERROR"));
    m.appuyer(Touche::Effacer);
    taper(&mut m, "5'");
    m.appuyer(Touche::Additionner);
    taper(&mut m, "3");
    m.appuyer(Touche::Conv);
    m.appuyer(Touche::Chiffre(1));
    m.appuyer(Touche::Egal);
    attendre(&m, "", "ERROR – unit mismatch");
}

#[test]
fn changer_de_langue_garde_les_valeurs() {
    // La langue du fil revient au français à la fin de l'essai.
    let _depart = langue::pour_la_portee(Langue::Francais);
    let mut m = Moteur::default();
    m.choisir_outil(Some("dalle"), None);
    // « 3,500 m » : en anglais, cette virgule séparerait les milliers.
    donner(&mut m, "3,5m", 0);
    attendre(&m, "LONGUEUR", "3,500 m");
    donner(&mut m, "1250'6\"", 1);
    donner(&mut m, "4,5\"", 2);
    donner(&mut m, "7,5", 3);
    let avant: Vec<String> = m.saisies_d_outil().unwrap().to_vec();
    assert_eq!(
        avant
            .iter()
            .map(|s| s.replace('\u{a0}', " "))
            .collect::<Vec<_>>(),
        ["3,500 m", "1 250 pi 6 po", "4,5 po", "7,5"]
    );
    m.appuyer(RESULTAT);
    let francais = m.ecran().valeur.en_texte().replace('\u{a0}', " ");
    assert_eq!(m.ecran().etiquette.as_deref(), Some("SURFACE"));

    m.changer_de_langue(Langue::Anglais);
    assert_eq!(langue::en_cours(), Langue::Anglais);
    assert_eq!(m.preferences().langue, Langue::Anglais);
    assert_eq!(
        m.saisies_d_outil().unwrap(),
        ["3.500 m", "1250 ft 6 in", "4.5 in", "7.5"]
    );
    // Le résultat montré est récrit en anglais : la même surface, le même rang.
    let lignes_du_ruban = m.ruban().lignes.len();
    let anglais = m.ecran().valeur.en_texte().replace('\u{a0}', " ");
    assert_eq!(m.ecran().etiquette.as_deref(), Some("AREA"));
    assert_eq!(m.ecran().note.as_deref(), Some("1 of 3"));
    // La touche `Résultat` poursuit le défilement, sans réinscrire le calcul au ruban.
    m.appuyer(RESULTAT);
    assert_eq!(m.ecran().etiquette.as_deref(), Some("CONCRETE VOLUME"));
    assert_eq!(m.ecran().note.as_deref(), Some("2 of 3"));
    assert_eq!(m.ruban().lignes.len(), lignes_du_ruban);
    m.appuyer(RESULTAT);
    m.appuyer(RESULTAT);
    assert_eq!(m.ecran().etiquette.as_deref(), Some("AREA"));
    assert_eq!(
        francais
            .replace(',', ".")
            .replace("pi²", "ft²")
            .replace(' ', ""),
        anglais.replace([',', ' '], "")
    );

    // Le retour au français rend les saisies à la française.
    m.changer_de_langue(Langue::Francais);
    assert_eq!(langue::en_cours(), Langue::Francais);
    assert_eq!(
        m.saisies_d_outil().unwrap(),
        ["3,500 m", "1250 pi 6 po", "4,5 po", "7,5"]
    );
    assert_eq!(m.ecran().etiquette.as_deref(), Some("SURFACE"));
    assert_eq!(m.ecran().note.as_deref(), Some("1 de 3"));
    assert_eq!(m.ecran().valeur.en_texte().replace('\u{a0}', " "), francais);
}

#[test]
fn un_champ_ou_un_message_d_outil_ne_restent_pas_dans_l_ancienne_langue() {
    let _depart = langue::pour_la_portee(Langue::Francais);
    let mut m = Moteur::default();
    m.choisir_outil(Some("dalle"), None);
    // Le contenu d'un champ : la valeur seule reste à l'afficheur.
    donner(&mut m, "24'", 0);
    attendre(&m, "LONGUEUR", "24 pi 0 po");
    m.changer_de_langue(Langue::Anglais);
    attendre(&m, "", "24 ft 0 in");
    // Un message d'outil : l'afficheur revient à zéro.
    m.appuyer(RESULTAT);
    attendre(&m, "", "MISSING: WIDTH");
    m.changer_de_langue(Langue::Francais);
    attendre(&m, "", "0");
    m.appuyer(RESULTAT);
    attendre(&m, "", "IL MANQUE : LARGEUR");
    // Un message de la calculatrice, lui, suit la langue sur-le-champ.
    m.appuyer(Touche::Effacer);
    m.appuyer(Touche::Conv);
    m.appuyer(Touche::Rappel);
    attendre(&m, "", "MÉMOIRE EFFACÉE");
    m.changer_de_langue(Langue::Anglais);
    attendre(&m, "", "MEMORY CLEARED");
}

#[test]
fn les_etiquettes_de_charpente_suivent_la_langue() {
    let _depart = langue::pour_la_portee(Langue::Francais);
    let mut m = Moteur::default();
    taper(&mut m, "7\"");
    m.appuyer(Touche::Pente);
    attendre(&m, "PENTE", "7 po");
    // L'étiquette et la valeur montrées changent de langue sur-le-champ.
    m.changer_de_langue(Langue::Anglais);
    attendre(&m, "PITCH", "7 in");
    assert_eq!(m.ecran().note.as_deref(), Some("in 12"));
    m.changer_de_langue(Langue::Francais);
    attendre(&m, "PENTE", "7 po");
    // Changer pour la même langue ne dérange rien.
    m.changer_de_langue(Langue::Francais);
    attendre(&m, "PENTE", "7 po");
}

#[test]
fn la_reinitialisation_garde_la_langue() {
    let _depart = langue::pour_la_portee(Langue::Francais);
    let mut m = Moteur::default();
    m.changer_de_langue(Langue::Anglais);
    m.preferences_mut().mode_trigo = true;
    m.reinitialiser();
    assert_eq!(m.preferences().langue, Langue::Anglais);
    assert!(!m.preferences().mode_trigo);
    attendre(&m, "", "ALL CLEARED");
    // La langue fait partie de l'état conservé.
    let etat = m.etat_persistant();
    assert_eq!(etat.preferences.langue, Langue::Anglais);
    assert_eq!(Preferences::default().langue, Langue::Francais);
}
