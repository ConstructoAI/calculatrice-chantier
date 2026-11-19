//! Outils de métier au clavier (section 23) : un calculateur par métier est
//! branché sur le clavier, et se remplit avec les touches de la calculatrice.

use noyau::metiers::Metier;
use noyau::moteur::{CHAMPS_D_OUTIL, TOUCHE_RESULTAT};
use noyau::prefs::SeparateurDecimal;
use noyau::{Moteur, Touche};

const RESULTAT: Touche = Touche::Outil(TOUCHE_RESULTAT);

/// Tape une suite de touches : chiffres, `,`, `/`, `'` (Pieds), `"` (Pouces),
/// `m` (Mètres), `x` (multiplier), `=`.
fn taper(moteur: &mut Moteur, frappes: &str) {
    for c in frappes.chars() {
        let touche = match c {
            '0'..='9' => Touche::Chiffre(c as u8 - b'0'),
            ',' => Touche::Virgule,
            '/' => Touche::Fraction,
            '\'' => Touche::Pieds,
            '"' => Touche::Pouces,
            'm' => Touche::Metres,
            'x' => Touche::Multiplier,
            '=' => Touche::Egal,
            autre => panic!("frappe inconnue : {autre}"),
        };
        moteur.appuyer(touche);
    }
}

/// Tape une valeur, puis appuie sur la touche d'un champ.
fn donner(moteur: &mut Moteur, frappes: &str, rang: u8) {
    taper(moteur, frappes);
    moteur.appuyer(Touche::Outil(rang));
}

/// `Conv`, puis une touche d'outil.
fn conv(moteur: &mut Moteur, touche: Touche) {
    moteur.appuyer(Touche::Conv);
    moteur.appuyer(touche);
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

/// La mention écrite à côté de l'étiquette : « 2 de 3 ».
fn note(moteur: &Moteur) -> String {
    moteur.ecran().note.unwrap_or_default()
}

/// Les champs de l'outil, espaces insécables écrites comme des espaces.
fn saisies(moteur: &Moteur) -> Vec<String> {
    moteur
        .saisies_d_outil()
        .unwrap()
        .iter()
        .map(|saisie| saisie.replace('\u{a0}', " "))
        .collect()
}

fn avec_outil(id: &str) -> Moteur {
    let mut moteur = Moteur::default();
    moteur.choisir_outil(Some(id), None);
    assert_eq!(moteur.outil().map(|c| c.id), Some(id));
    moteur
}

#[test]
fn dalle_au_clavier() {
    let mut m = avec_outil("dalle");
    donner(&mut m, "24'", 0);
    attendre(&m, "LONGUEUR", "24 pi 0 po");
    assert!(m.ecran().memorise);
    donner(&mut m, "20'", 1);
    attendre(&m, "LARGEUR", "20 pi 0 po");
    donner(&mut m, "4\"", 2);
    attendre(&m, "ÉPAISSEUR", "4 po");
    // Les pertes gardent leur valeur de départ : 10 %.
    m.appuyer(RESULTAT);
    attendre(&m, "SURFACE", "480 pi² (44,6 m²)");
    assert_eq!(note(&m), "1 de 3");
    assert!(!m.ecran().memorise);
    m.appuyer(RESULTAT);
    attendre(&m, "VOLUME DE BÉTON", "5,93 vg³ (4,53 m³)");
    assert_eq!(note(&m), "2 de 3");
    m.appuyer(RESULTAT);
    attendre(&m, "À COMMANDER, PERTES COMPRISES", "6,52 vg³ (4,98 m³)");
    assert_eq!(note(&m), "3 de 3");
    // Après le dernier résultat, le défilement reprend au premier.
    m.appuyer(RESULTAT);
    attendre(&m, "SURFACE", "480 pi² (44,6 m²)");

    // Le calcul est inscrit au ruban une seule fois, sous son titre.
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
    let titres = lignes.iter().filter(|l| *l == "|Dalle de béton").count();
    assert_eq!(titres, 1, "{lignes:?}");
    assert!(lignes.contains(&"Volume de béton|5,93 vg³ (4,53 m³)".to_string()));
    assert!(lignes.contains(&"Pertes|10 %".to_string()));

    // Les champs contiennent ce que l'afficheur montrait.
    assert_eq!(saisies(&m), ["24 pi 0 po", "20 pi 0 po", "4 po", "10"]);

    // Une valeur changée, puis `Résultat` : nouveau calcul, nouvelle inscription.
    donner(&mut m, "6\"", 2);
    m.appuyer(RESULTAT);
    attendre(&m, "SURFACE", "480 pi² (44,6 m²)");
    m.appuyer(RESULTAT);
    attendre(&m, "VOLUME DE BÉTON", "8,89 vg³ (6,80 m³)");
    let titres = m
        .ruban()
        .lignes
        .iter()
        .filter(|l| l.texte == "Dalle de béton")
        .count();
    assert_eq!(titres, 2);
}

#[test]
fn un_nombre_seul_prend_l_unite_du_champ() {
    let mut m = avec_outil("dalle");
    // 24 sans unité dans un champ en pieds : 24 pieds.
    donner(&mut m, "24", 0);
    attendre(&m, "LONGUEUR", "24 pi 0 po");
    // 4 sans unité dans un champ en pouces : 4 pouces.
    donner(&mut m, "4", 2);
    attendre(&m, "ÉPAISSEUR", "4 po");
    // En mètres, l'unité tapée l'emporte.
    donner(&mut m, "7,3m", 0);
    attendre(&m, "LONGUEUR", "7,300 m");
    assert_eq!(saisies(&m), ["7,300 m", "", "4", "10"]);
}

#[test]
fn rappel_et_valeur_manquante() {
    let mut m = avec_outil("dalle");
    donner(&mut m, "24'", 0);
    // Une seconde pression rappelle le champ sans le réécrire.
    m.appuyer(Touche::Outil(0));
    attendre(&m, "LONGUEUR", "24 pi 0 po");
    // Un champ vide n'a rien à rappeler; ce n'est pas une erreur bloquante.
    m.appuyer(Touche::Outil(1));
    attendre(&m, "", "AUCUNE VALEUR");
    assert!(!m.ecran().erreur);
    // Il manque des valeurs : l'afficheur dit laquelle donner en premier.
    m.appuyer(RESULTAT);
    attendre(&m, "", "IL MANQUE : LARGEUR");
    assert!(!m.ecran().erreur);
    // La calculatrice reste utilisable aussitôt.
    donner(&mut m, "20'", 1);
    attendre(&m, "LARGEUR", "20 pi 0 po");
    // La valeur montrée d'un champ n'est pas recopiée dans un autre par mégarde.
    m.appuyer(Touche::Outil(2));
    attendre(&m, "", "AUCUNE VALEUR");
    m.appuyer(RESULTAT);
    attendre(&m, "", "IL MANQUE : ÉPAISSEUR");
    // Rien n'a été inscrit au ruban tant que le calcul est incomplet.
    assert!(m.ruban().lignes.iter().all(|l| l.texte != "Dalle de béton"));
}

#[test]
fn choix_et_resultats_en_longueur() {
    let mut m = avec_outil("deport");
    donner(&mut m, "12\"", 0);
    attendre(&m, "DÉPORT", "12 po");
    // L'angle de départ est 45°.
    m.appuyer(RESULTAT);
    attendre(&m, "LONGUEUR DE PARCOURS", "1 pi 5 po");
    assert_eq!(note(&m), "1 de 2");
    m.appuyer(RESULTAT);
    attendre(&m, "AVANCE", "1 pi 0 po");
    // Chaque pression sur la touche de l'angle passe à l'option suivante.
    m.appuyer(Touche::Outil(1));
    attendre(&m, "ANGLE DES RACCORDS", "60°");
    assert_eq!(note(&m), "5 de 5");
    m.appuyer(Touche::Outil(1));
    attendre(&m, "ANGLE DES RACCORDS", "11,25°");
    assert_eq!(note(&m), "1 de 5");
    m.appuyer(Touche::Outil(1));
    attendre(&m, "ANGLE DES RACCORDS", "22,5°");
    m.appuyer(RESULTAT);
    attendre(&m, "LONGUEUR DE PARCOURS", "2 pi 7 3/8 po");
}

#[test]
fn un_resultat_sert_a_la_suite_du_calcul() {
    let mut m = avec_outil("deport");
    donner(&mut m, "12\"", 0);
    m.appuyer(RESULTAT);
    attendre(&m, "LONGUEUR DE PARCOURS", "1 pi 5 po");
    // Deux tronçons : la longueur exacte (16,97 po) est doublée, pas l'arrondi.
    taper(&mut m, "x2=");
    attendre(&m, "", "2 pi 9 15/16 po");

    // Un volume de béton se multiplie par un prix.
    let mut m = avec_outil("dalle");
    donner(&mut m, "24'", 0);
    donner(&mut m, "20'", 1);
    donner(&mut m, "4\"", 2);
    m.appuyer(RESULTAT);
    m.appuyer(RESULTAT);
    attendre(&m, "VOLUME DE BÉTON", "5,93 vg³ (4,53 m³)");
    taper(&mut m, "x200=");
    // 5,925926 vg³ × 200.
    attendre(&m, "", "1 185,185 vg³");
}

#[test]
fn nombres_et_unites_incompatibles() {
    let mut m = avec_outil("chute-de-tension");
    // La tension de départ est 120 V.
    m.appuyer(Touche::Outil(0));
    attendre(&m, "TENSION DU CIRCUIT", "120 V");
    donner(&mut m, "15", 1);
    attendre(&m, "COURANT", "15 A");
    donner(&mut m, "100'", 2);
    attendre(&m, "LONGUEUR DU PARCOURS, ALLER SIMPLE", "100 pi 0 po");
    m.appuyer(RESULTAT);
    attendre(&m, "CHUTE DE TENSION", "9,42 V");
    m.appuyer(RESULTAT);
    attendre(&m, "CHUTE, EN POURCENTAGE", "7,85 %");
    m.appuyer(RESULTAT);
    attendre(&m, "TENSION À LA CHARGE", "110,6 V");
    // Un calibre plus gros : la touche du calibre passe de 14 à 12 AWG.
    m.appuyer(Touche::Outil(3));
    attendre(&m, "CALIBRE DU CONDUCTEUR", "12 AWG");
    m.appuyer(RESULTAT);
    attendre(&m, "CHUTE DE TENSION", "5,93 V");

    // Une longueur dans un champ qui attend un nombre : unités incompatibles.
    donner(&mut m, "5'", 1);
    attendre(&m, "", "ERREUR – unités incompatibles");
    assert!(m.ecran().erreur);
    m.appuyer(Touche::Effacer);
    // Le champ a gardé sa valeur.
    m.appuyer(Touche::Outil(1));
    attendre(&m, "COURANT", "15 A");
    // Zéro n'est pas un courant : la saisie est refusée, le champ ne change pas.
    donner(&mut m, "0", 1);
    attendre(&m, "", "ERREUR – saisie invalide");
    m.appuyer(Touche::Effacer);
    assert_eq!(saisies(&m)[1], "15");
}

#[test]
fn avis_du_calculateur() {
    let mut m = avec_outil("loi-d-ohm");
    donner(&mut m, "120", 0);
    m.appuyer(RESULTAT);
    attendre(&m, "", "Remplissez exactement deux des quatre champs.");
    assert!(!m.ecran().erreur);
    donner(&mut m, "15", 1);
    m.appuyer(RESULTAT);
    attendre(&m, "TENSION", "120 V");
    assert_eq!(note(&m), "1 de 4");
    m.appuyer(RESULTAT);
    attendre(&m, "COURANT", "15,0 A");
    m.appuyer(RESULTAT);
    attendre(&m, "RÉSISTANCE", "8,00 Ω");
    m.appuyer(RESULTAT);
    attendre(&m, "PUISSANCE", "1 800 W");
}

#[test]
fn conv_remet_un_champ_au_depart() {
    // Un champ sans valeur de départ est vidé.
    let mut m = avec_outil("loi-d-ohm");
    donner(&mut m, "120", 0);
    donner(&mut m, "15", 1);
    conv(&mut m, Touche::Outil(1));
    attendre(&m, "", "CHAMP VIDÉ");
    assert!(!m.ecran().erreur);
    assert_eq!(saisies(&m), ["120", "", "", ""]);
    // Tension et résistance : le courant est maintenant calculé.
    donner(&mut m, "8", 2);
    m.appuyer(RESULTAT);
    m.appuyer(RESULTAT);
    attendre(&m, "COURANT", "15,0 A");

    // Un champ qui a une valeur de départ la reprend.
    let mut m = avec_outil("dalle");
    donner(&mut m, "5", 3);
    attendre(&m, "PERTES", "5 %");
    conv(&mut m, Touche::Outil(3));
    attendre(&m, "PERTES", "10 %");
    assert_eq!(saisies(&m)[3], "10");

    // Un choix revient à son option de départ.
    let mut m = avec_outil("deport");
    m.appuyer(Touche::Outil(1));
    attendre(&m, "ANGLE DES RACCORDS", "60°");
    conv(&mut m, Touche::Outil(1));
    attendre(&m, "ANGLE DES RACCORDS", "45°");
    assert_eq!(note(&m), "4 de 5");

    // `Conv` puis `Résultat` : tous les champs reprennent leur valeur de départ.
    let mut m = avec_outil("dalle");
    donner(&mut m, "24'", 0);
    donner(&mut m, "5", 3);
    conv(&mut m, RESULTAT);
    attendre(&m, "", "CHAMPS REMIS AU DÉPART");
    assert_eq!(saisies(&m), ["", "", "", "10"]);
}

#[test]
fn grands_nombres_et_preferences() {
    // Un montant garde ses cents, même s'il dépasse la capacité de l'afficheur.
    let mut m = avec_outil("soumission");
    donner(&mut m, "1234567,89", 0);
    attendre(&m, "COÛT DE BASE DES TRAVAUX", "1 234 568 $");
    assert_eq!(saisies(&m)[0], "1234567,89");
    m.appuyer(RESULTAT);
    // 3 % de 1 234 567,89 $.
    attendre(&m, "ADMINISTRATION", "37 037,04 $");
    assert_eq!(note(&m), "1 de 7");

    // Avec le point décimal, la valeur rangée s'écrit avec un point; elle reste
    // lisible si la préférence change ensuite.
    let mut m = avec_outil("soumission");
    m.preferences_mut().separateur_decimal = SeparateurDecimal::Point;
    donner(&mut m, "2500,5", 0);
    assert_eq!(saisies(&m)[0], "2500.5");
    m.preferences_mut().separateur_decimal = SeparateurDecimal::Virgule;
    m.appuyer(RESULTAT);
    attendre(&m, "ADMINISTRATION", "75,02 $");

    // Sans séparateur de milliers : mêmes résultats.
    let mut m = avec_outil("dalle");
    m.preferences_mut().separateur_milliers = false;
    donner(&mut m, "1500'", 0);
    attendre(&m, "LONGUEUR", "1500 pi 0 po");
    donner(&mut m, "20'", 1);
    donner(&mut m, "4\"", 2);
    m.appuyer(RESULTAT);
    attendre(&m, "SURFACE", "30000 pi² (2787 m²)");
    m.preferences_mut().separateur_milliers = true;
    m.appuyer(Touche::Outil(0));
    attendre(&m, "LONGUEUR", "1 500 pi 0 po");
    m.appuyer(RESULTAT);
    attendre(&m, "SURFACE", "30 000 pi² (2 787 m²)");
}

#[test]
fn une_aire_va_dans_un_champ_en_pieds_carres() {
    let mut m = avec_outil("budget-au-pied-carre");
    // 40 pi × 35 pi : la superficie calculée au clavier remplit le champ.
    taper(&mut m, "40'x35'=");
    attendre(&m, "", "1 400 pi²");
    m.appuyer(Touche::Outil(0));
    attendre(&m, "SUPERFICIE HABITABLE", "1 400 pi²");
    assert_eq!(saisies(&m)[0], "1400");
    // Des mètres carrés dans un champ en pieds carrés : unités incompatibles.
    taper(&mut m, "10mx10m=");
    m.appuyer(Touche::Outil(0));
    attendre(&m, "", "ERREUR – unités incompatibles");
    m.appuyer(Touche::Effacer);
    // Une longueur non plus.
    donner(&mut m, "12'", 0);
    attendre(&m, "", "ERREUR – unités incompatibles");
    m.appuyer(Touche::Effacer);
    assert_eq!(saisies(&m)[0], "1400");
    // Le reste du calcul : 200 $/pi², sans garage.
    donner(&mut m, "200", 1);
    attendre(&m, "COÛT DE BASE", "200 $/pi²");
    m.appuyer(RESULTAT);
    attendre(&m, "COÛT DE BASE", "280 000,00 $");
}

#[test]
fn touches_de_l_outil_pour_l_interface() {
    let mut m = avec_outil("dalle");
    donner(&mut m, "24'", 0);
    let touches = m.touches_d_outil().unwrap();
    assert_eq!(touches.len(), 4);
    assert_eq!(touches[0].libelle, "Longueur");
    assert_eq!(touches[0].valeur.replace('\u{a0}', " "), "24 pi 0 po");
    assert_eq!(touches[0].unite, "pi");
    assert!(!touches[0].choix && !touches[0].facultatif);
    assert_eq!(touches[1].valeur, "");
    assert_eq!(touches[2].unite, "po");
    assert_eq!(touches[3].libelle, "Pertes");
    assert_eq!(touches[3].valeur.replace('\u{a0}', " "), "10 %");

    // Un libellé long est abrégé sur la touche; le nom complet reste disponible.
    let m = avec_outil("chute-de-tension");
    let touches = m.touches_d_outil().unwrap();
    assert_eq!(touches.len(), 6);
    assert_eq!(touches[2].libelle, "Longueur");
    assert_eq!(touches[2].nom, "Longueur du parcours, aller simple");
    assert!(touches[3].choix);
    assert_eq!(touches[3].valeur, "14 AWG");

    let m = avec_outil("loi-d-ohm");
    assert!(m.touches_d_outil().unwrap().iter().all(|t| t.facultatif));
}

#[test]
fn tous_les_outils_tiennent_sur_le_clavier() {
    for metier in Metier::TOUS {
        for c in metier.calculateurs() {
            assert!(
                c.champs.len() <= CHAMPS_D_OUTIL,
                "{} : {} champs",
                c.id,
                c.champs.len()
            );
            for champ in c.champs {
                // Deux lignes au plus, assez courtes pour une touche étroite.
                let court = champ.libelle_court();
                let lignes: Vec<&str> = court.split('\n').collect();
                assert!(lignes.len() <= 2, "{} : {court:?}", c.id);
                for ligne in lignes {
                    assert!(
                        (1..=12).contains(&ligne.chars().count()),
                        "{} : {court:?}",
                        c.id
                    );
                }
                assert!(!champ.libelle_pour_message().is_empty(), "{}", c.id);
            }
            // Chaque outil se branche et montre ses touches.
            let m = avec_outil(c.id);
            assert_eq!(m.touches_d_outil().unwrap().len(), c.champs.len());
        }
    }
}

#[test]
fn saisies_partagees_avec_le_formulaire() {
    // Les champs remplis dans le formulaire se retrouvent au clavier.
    let mut m = Moteur::default();
    let du_formulaire =
        |textes: &[&str]| -> Vec<String> { textes.iter().map(|s| (*s).to_string()).collect() };
    let remplies = du_formulaire(&["24", "20", "4", "10"]);
    m.choisir_outil(Some("dalle"), Some(&remplies));
    m.appuyer(RESULTAT);
    attendre(&m, "SURFACE", "480 pi² (44,6 m²)");
    // Une saisie que le formulaire refuse : l'afficheur dit laquelle corriger.
    m.choisir_outil(
        Some("dalle"),
        Some(&du_formulaire(&["abc", "20", "4", "10"])),
    );
    m.appuyer(RESULTAT);
    attendre(&m, "", "À CORRIGER : LONGUEUR");
    assert!(!m.ecran().erreur);
    m.appuyer(Touche::Outil(0));
    attendre(&m, "", "À CORRIGER : LONGUEUR");
    donner(&mut m, "24'", 0);
    m.appuyer(RESULTAT);
    attendre(&m, "SURFACE", "480 pi² (44,6 m²)");
    // Un nombre de champs qui ne correspond pas : valeurs de départ.
    m.choisir_outil(Some("dalle"), Some(&remplies[..2]));
    assert_eq!(saisies(&m), ["", "", "", "10"]);
    // Un identifiant inconnu, ou `None` : retour au clavier de charpente.
    m.choisir_outil(Some("inconnu"), None);
    assert!(m.outil().is_none());
    m.choisir_outil(Some("dalle"), None);
    m.choisir_outil(None, None);
    assert!(m.outil().is_none());
    assert!(m.touches_d_outil().is_none());
    assert!(m.saisies_d_outil().is_none());
}

#[test]
fn un_resultat_passe_d_un_outil_a_l_autre() {
    // Le volume foisonné d'une tranchée sert au calcul des voyages de camion.
    let mut m = avec_outil("tranchee");
    donner(&mut m, "100'", 0);
    donner(&mut m, "2'", 1);
    donner(&mut m, "6'", 2);
    donner(&mut m, "4'", 3);
    m.appuyer(RESULTAT);
    m.appuyer(RESULTAT);
    attendre(&m, "VOLUME À TRANSPORTER, FOISONNÉ", "74,1 vg³ (56,6 m³)");
    // En changeant d'outil, la valeur reste, sans le libellé de l'outil quitté.
    m.choisir_outil(Some("voyages-de-camion"), None);
    attendre(&m, "", "74,07407 vg³");
    m.appuyer(Touche::Outil(0));
    attendre(&m, "VOLUME À TRANSPORTER", "74,07407 vg³");
    // Le champ garde toutes les décimales du résultat.
    assert_eq!(saisies(&m)[0], "74,074074074");
    donner(&mut m, "12", 1);
    m.appuyer(RESULTAT);
    attendre(&m, "VOYAGES", "7 voyages");

    // Les mesures d'une dalle servent à son armature.
    let mut m = avec_outil("dalle");
    donner(&mut m, "24'", 0);
    m.choisir_outil(Some("armature"), None);
    attendre(&m, "", "24 pi 0 po");
    m.appuyer(Touche::Outil(0));
    attendre(&m, "LONGUEUR", "24 pi 0 po");
}

#[test]
fn changer_d_outil_remet_l_afficheur_au_propre() {
    // Rebrancher le même outil ne touche pas à l'afficheur.
    let mut m = avec_outil("dalle");
    donner(&mut m, "24'", 0);
    let remplies = m.saisies_d_outil().unwrap().to_vec();
    m.choisir_outil(Some("dalle"), Some(&remplies));
    attendre(&m, "LONGUEUR", "24 pi 0 po");
    // Un message de l'outil disparaît avec lui.
    m.appuyer(RESULTAT);
    attendre(&m, "", "IL MANQUE : LARGEUR");
    m.choisir_outil(None, None);
    attendre(&m, "", "0");
    // Un choix dans une liste n'est pas une valeur : rien ne reste.
    let mut m = avec_outil("deport");
    m.appuyer(Touche::Outil(1));
    attendre(&m, "ANGLE DES RACCORDS", "60°");
    m.choisir_outil(Some("deport-roule"), None);
    attendre(&m, "", "0");
    // Un mot non plus : le calibre d'un conducteur.
    let mut m = avec_outil("calibre-pour-chute");
    donner(&mut m, "20", 1);
    donner(&mut m, "100'", 2);
    m.appuyer(RESULTAT);
    attendre(&m, "CALIBRE SUFFISANT", "8 AWG");
    assert_eq!(note(&m), "1 de 4");
    m.choisir_outil(Some("chute-de-tension"), None);
    attendre(&m, "", "0");
    // Une valeur tapée reste telle quelle.
    taper(&mut m, "15");
    m.choisir_outil(Some("loi-d-ohm"), None);
    attendre(&m, "", "15");
}

#[test]
fn sans_outil_les_touches_ne_font_rien() {
    let mut m = Moteur::default();
    taper(&mut m, "12");
    m.appuyer(Touche::Outil(0));
    m.appuyer(RESULTAT);
    attendre(&m, "", "12");
    // Une touche au-delà des champs de l'outil ne fait rien non plus.
    let mut m = avec_outil("dalle");
    taper(&mut m, "12");
    m.appuyer(Touche::Outil(7));
    attendre(&m, "", "12");
    assert_eq!(saisies(&m), ["", "", "", "10"]);
}

#[test]
fn effacer_tout_vide_l_outil() {
    let mut m = avec_outil("dalle");
    donner(&mut m, "24'", 0);
    m.effacer_tout();
    assert_eq!(saisies(&m), ["", "", "", "10"]);
    // L'outil reste branché.
    assert_eq!(m.outil().map(|c| c.id), Some("dalle"));
}

#[test]
fn aucune_suite_de_touches_ne_fait_paniquer() {
    // Générateur pseudo-aléatoire maison, comme dans les tests de robustesse.
    let mut graine: u64 = 0x9E37_79B9_7F4A_7C15;
    let mut suivant = move || {
        graine ^= graine << 13;
        graine ^= graine >> 7;
        graine ^= graine << 17;
        graine
    };
    let touches = [
        Touche::Chiffre(0),
        Touche::Chiffre(1),
        Touche::Chiffre(5),
        Touche::Chiffre(9),
        Touche::Virgule,
        Touche::Fraction,
        Touche::Pieds,
        Touche::Pouces,
        Touche::Metres,
        Touche::Additionner,
        Touche::Multiplier,
        Touche::Diviser,
        Touche::Egal,
        Touche::Effacer,
        Touche::Conv,
        Touche::Stocker,
        Touche::Rappel,
        Touche::MPlus,
        Touche::Pourcent,
        Touche::Soustraire,
    ];
    for metier in Metier::TOUS {
        for c in metier.calculateurs() {
            let mut m = avec_outil(c.id);
            for _ in 0..2_500 {
                let tirage = suivant() % 32;
                let touche = if tirage < 12 {
                    Touche::Outil(tirage as u8)
                } else {
                    touches[(tirage - 12) as usize]
                };
                m.appuyer(touche);
                // L'afficheur et les touches restent toujours présentables.
                let _ = m.ecran();
                assert_eq!(m.touches_d_outil().unwrap().len(), c.champs.len());
                assert_eq!(m.saisies_d_outil().unwrap().len(), c.champs.len());
            }
        }
    }
}
