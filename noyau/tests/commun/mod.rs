//! Outil de test : jouer une séquence de touches (section 19.2).
#![allow(dead_code)]

use noyau::prefs::Preferences;
use noyau::{Ecran, EtatPersistant, Moteur, Touche};

/// Moteur dans son état par défaut, préférences par défaut.
pub fn moteur_neuf() -> Moteur {
    Moteur::nouveau(EtatPersistant::default())
}

/// Moteur neuf dont les préférences ont été retouchées.
pub fn moteur_avec(regler: impl FnOnce(&mut Preferences)) -> Moteur {
    let mut moteur = moteur_neuf();
    regler(moteur.preferences_mut());
    moteur
}

fn sans_accents(texte: &str) -> String {
    texte
        .to_lowercase()
        .chars()
        .map(|c| match c {
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'à' | 'â' => 'a',
            'î' | 'ï' => 'i',
            'ô' => 'o',
            'û' | 'ù' => 'u',
            'ç' => 'c',
            autre => autre,
        })
        .collect()
}

/// Traduit un jeton en touches.
fn touches(jeton: &str) -> Vec<Touche> {
    if jeton.chars().all(|c| c.is_ascii_digit() || c == ',') {
        return jeton
            .chars()
            .map(|c| match c.to_digit(10) {
                Some(d) => Touche::Chiffre(d as u8),
                None => Touche::Virgule,
            })
            .collect();
    }
    if let Some(fraction) = jeton.strip_prefix('[').and_then(|j| j.strip_suffix(']')) {
        let (num, den) = fraction
            .split_once('/')
            .expect("fraction rapide : [num/den]");
        return vec![Touche::FractionRapide {
            num: num.parse().expect("numérateur"),
            den: den.parse().expect("dénominateur"),
        }];
    }
    let touche = match sans_accents(jeton).as_str() {
        "pi" => Touche::Pieds,
        "po" => Touche::Pouces,
        "vg" => Touche::Verges,
        "m" => Touche::Metres,
        "/" => Touche::Fraction,
        "+" => Touche::Additionner,
        "−" | "-" => Touche::Soustraire,
        "×" | "x" | "*" => Touche::Multiplier,
        "÷" | ":" => Touche::Diviser,
        "=" => Touche::Egal,
        "%" => Touche::Pourcent,
        "conv" => Touche::Conv,
        "sto" => Touche::Stocker,
        "rap" => Touche::Rappel,
        "m+" => Touche::MPlus,
        "eff" => Touche::Effacer,
        "⌫" => Touche::RetourArriere,
        "pente" => Touche::Pente,
        "montee" => Touche::Montee,
        "course" => Touche::Course,
        "diag" => Touche::Diag,
        "aretier" => Touche::AretierNoue,
        "onglet" => Touche::OngletCompose,
        "escalier" => Touche::Escalier,
        "arc" => Touche::Arc,
        "cercle" => Touche::Cercle,
        "empannon" => Touche::Empannon,
        "longueur" => Touche::Longueur,
        "largeur" => Touche::Largeur,
        "hauteur" => Touche::Hauteur,
        autre => panic!("jeton inconnu : « {autre} »"),
    };
    vec![touche]
}

/// Joue une séquence de jetons séparés par des espaces et retourne l'écran final.
pub fn jouer(moteur: &mut Moteur, sequence: &str) -> Ecran {
    for jeton in sequence.split_whitespace() {
        for touche in touches(jeton) {
            moteur.appuyer(touche);
        }
    }
    moteur.ecran()
}

/// Texte comparable : « ÉTIQUETTE │ valeur », la valeur seule, ou le message.
/// Les espaces insécables sont remplacées par des espaces ordinaires.
pub fn texte(ecran: &Ecran) -> String {
    let brut = match (&ecran.message, &ecran.etiquette) {
        (Some(message), _) => message.clone(),
        (None, Some(etiquette)) => format!("{etiquette} │ {}", ecran.valeur.en_texte()),
        (None, None) => ecran.valeur.en_texte(),
    };
    brut.replace('\u{00A0}', " ")
}

/// Compare un écran à l'attendu. L'attendu peut se terminer par « MÉM. » ou « ⚠ »
/// (indicateur exigé) ou par « !MÉM. » (indicateur interdit).
pub fn comparer(ecran: &Ecran, attendu: &str, contexte: &str) {
    let mut reste = attendu.trim();
    let mut memorise = None;
    let mut avertissement = false;
    loop {
        if let Some(r) = reste.strip_suffix("!MÉM.") {
            memorise = Some(false);
            reste = r.trim_end();
        } else if let Some(r) = reste.strip_suffix("MÉM.") {
            memorise = Some(true);
            reste = r.trim_end();
        } else if let Some(r) = reste.strip_suffix('⚠') {
            avertissement = true;
            reste = r.trim_end();
        } else {
            break;
        }
    }
    assert_eq!(texte(ecran), reste, "{contexte}");
    if let Some(m) = memorise {
        assert_eq!(ecran.memorise, m, "indicateur MÉM. — {contexte}");
    }
    if avertissement {
        assert!(
            ecran.avertissement.is_some(),
            "avertissement attendu — {contexte}"
        );
    }
}

/// Vérifie une table de vecteurs. Chaque ligne part d'un moteur neuf, sauf celles
/// qui commencent par « ↳ », qui poursuivent la ligne précédente.
pub fn verifier(table: &[(&str, &str)]) {
    verifier_avec(|_| {}, table);
}

/// Comme `verifier`, avec des préférences retouchées pour chaque moteur neuf.
pub fn verifier_avec(regler: impl Fn(&mut Preferences), table: &[(&str, &str)]) {
    let mut moteur = moteur_avec(&regler);
    let mut historique = String::new();
    for (frappes, attendu) in table {
        let suite = match frappes.strip_prefix('↳') {
            Some(reste) => reste,
            None => {
                moteur = moteur_avec(&regler);
                historique.clear();
                frappes
            }
        };
        historique.push_str(suite.trim());
        historique.push(' ');
        let ecran = jouer(&mut moteur, suite);
        comparer(&ecran, attendu, &format!("après « {} »", historique.trim()));
    }
}

/// Appuie plusieurs fois de suite sur la même touche et compare chaque écran.
pub fn suite(moteur: &mut Moteur, touche: &str, attendus: &[&str]) {
    for (i, attendu) in attendus.iter().enumerate() {
        let ecran = jouer(moteur, touche);
        comparer(
            &ecran,
            attendu,
            &format!("pression n° {} de « {touche} »", i + 1),
        );
    }
}

/// Joue une séquence et compare l'écran final.
pub fn attendre(moteur: &mut Moteur, sequence: &str, attendu: &str) {
    let ecran = jouer(moteur, sequence);
    comparer(&ecran, attendu, &format!("après « {sequence} »"));
}
