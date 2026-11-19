//! Lecture et écriture des données conservées (section 17).
//!
//! Ce module ne sait pas où vont les données : un dossier de fichiers sur le
//! bureau, le stockage local dans un navigateur. C'est l'affaire du `Depot` de
//! la plateforme.

use std::collections::BTreeMap;
use std::io;

use chrono::{Datelike, Timelike};
use noyau::{EtatPersistant, LigneRuban};
use serde::{Deserialize, Serialize};

use crate::plateforme::{Depot, JSON_LISIBLE, Lecture};
use crate::textes;

const FICHIER_ETAT: &str = "etat.json";
const FICHIER_ETAT_CORROMPU: &str = "etat.corrompu.json";
const FICHIER_RUBANS: &str = "rubans.json";
const FICHIER_RUBANS_CORROMPU: &str = "rubans.corrompu.json";
/// Version du format de `rubans.json`.
const VERSION_RUBANS: u32 = 1;
const FICHIER_SAISIES: &str = "metiers.json";
/// Version du format de `metiers.json`.
const VERSION_SAISIES: u32 = 1;
/// Nombre maximal de rubans enregistrés (annexe A.2).
pub const MAX_RUBANS: usize = 200;

/// Met une valeur en JSON : lisible sur le bureau, compact dans un navigateur.
fn en_json<T: Serialize>(valeur: &T) -> io::Result<String> {
    let json = if JSON_LISIBLE {
        serde_json::to_string_pretty(valeur)
    } else {
        serde_json::to_string(valeur)
    };
    json.map_err(io::Error::other)
}

/// Résultat de la lecture de `etat.json`.
pub struct Chargement {
    pub etat: EtatPersistant,
    /// La donnée existait mais n'a pas pu être lue : elle a été mise de côté (R-46).
    pub illisible: bool,
}

/// Lit l'état conservé. Donnée absente : valeurs par défaut. Donnée illisible
/// ou d'une version inconnue : mise de côté, valeurs par défaut (R-46).
pub fn charger_etat(depot: &Depot) -> Chargement {
    let par_defaut = |illisible: bool| {
        if illisible {
            depot.mettre_de_cote(FICHIER_ETAT, FICHIER_ETAT_CORROMPU);
        }
        Chargement {
            etat: EtatPersistant::default(),
            illisible,
        }
    };
    let texte = match depot.lire(FICHIER_ETAT) {
        Lecture::Texte(texte) => texte,
        Lecture::Absent => return par_defaut(false),
        Lecture::Illisible => return par_defaut(true),
    };
    match serde_json::from_str::<EtatPersistant>(&texte) {
        Ok(etat) if etat.version_connue() => Chargement {
            etat,
            illisible: false,
        },
        _ => par_defaut(true),
    }
}

/// Enregistre l'état.
pub fn enregistrer_etat(depot: &Depot, etat: &EtatPersistant) -> io::Result<()> {
    depot.ecrire(FICHIER_ETAT, &en_json(etat)?)
}

/// Un ruban enregistré (section 14.3).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct RubanEnregistre {
    pub id: u64,
    pub nom: String,
    /// Date d'enregistrement, ISO 8601.
    pub date: String,
    pub lignes: Vec<LigneRuban>,
}

impl RubanEnregistre {
    /// Date en clair : « 19 novembre 2026, 8 h 04 »; la date brute si elle est illisible.
    pub fn date_en_clair(&self) -> String {
        match chrono::DateTime::parse_from_rfc3339(&self.date) {
            Ok(d) => textes::date_en_clair(d.year(), d.month(), d.day(), d.hour(), d.minute()),
            Err(_) => self.date.clone(),
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct FichierRubans {
    version: u32,
    rubans: Vec<RubanEnregistre>,
}

/// Lit les rubans enregistrés; liste vide s'il n'y en a pas. Une liste
/// illisible ou d'une version inconnue est mise de côté.
pub fn charger_rubans(depot: &Depot) -> Vec<RubanEnregistre> {
    let lus = match depot.lire(FICHIER_RUBANS) {
        Lecture::Absent => return Vec::new(),
        Lecture::Illisible => None,
        Lecture::Texte(texte) => serde_json::from_str::<FichierRubans>(&texte)
            .ok()
            .filter(|fichier| fichier.version == VERSION_RUBANS),
    };
    match lus {
        Some(fichier) => fichier.rubans,
        None => {
            depot.mettre_de_cote(FICHIER_RUBANS, FICHIER_RUBANS_CORROMPU);
            Vec::new()
        }
    }
}

/// Enregistre les rubans, du plus récent au plus ancien.
pub fn enregistrer_rubans(depot: &Depot, rubans: &[RubanEnregistre]) -> io::Result<()> {
    let fichier = FichierRubans {
        version: VERSION_RUBANS,
        rubans: rubans.to_vec(),
    };
    depot.ecrire(FICHIER_RUBANS, &en_json(&fichier)?)
}

/// Saisies des calculateurs par métier (section 22), par identifiant de
/// calculateur : ce que l'utilisateur a tapé la dernière fois.
pub type Saisies = BTreeMap<String, Vec<String>>;

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct FichierSaisies {
    version: u32,
    saisies: Saisies,
}

/// Lit les dernières saisies; aucune si la donnée est absente, illisible ou
/// d'une autre version (ce ne sont que des commodités : rien n'est mis de côté).
pub fn charger_saisies(depot: &Depot) -> Saisies {
    match depot.lire(FICHIER_SAISIES) {
        Lecture::Texte(texte) => serde_json::from_str::<FichierSaisies>(&texte)
            .ok()
            .filter(|fichier| fichier.version == VERSION_SAISIES)
            .map(|fichier| fichier.saisies)
            .unwrap_or_default(),
        Lecture::Absent | Lecture::Illisible => Saisies::new(),
    }
}

/// Enregistre les dernières saisies.
pub fn enregistrer_saisies(depot: &Depot, saisies: &Saisies) -> io::Result<()> {
    let fichier = FichierSaisies {
        version: VERSION_SAISIES,
        saisies: saisies.clone(),
    };
    depot.ecrire(FICHIER_SAISIES, &en_json(&fichier)?)
}

/// Contenu d'un fichier texte exporté : UTF-8 avec indicateur d'ordre des
/// octets et fins de ligne Windows, pour que le Bloc-notes garde les accents
/// intacts.
pub fn contenu_exporte(texte: &str) -> String {
    let mut contenu = String::with_capacity(texte.len() + 16);
    contenu.push('\u{FEFF}');
    for ligne in texte.lines() {
        contenu.push_str(ligne);
        contenu.push_str("\r\n");
    }
    contenu
}

/// Date et heure locales : (ISO 8601, date en clair).
pub fn maintenant() -> (String, String) {
    let m = chrono::Local::now();
    let iso = m.to_rfc3339_opts(chrono::SecondsFormat::Secs, false);
    let clair = textes::date_en_clair(m.year(), m.month(), m.day(), m.hour(), m.minute());
    (iso, clair)
}

/// Nom de fichier sûr à partir du nom d'un ruban.
pub fn nom_de_fichier(nom: &str) -> String {
    let propre: String = nom
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '-',
            c if c.is_control() => ' ',
            c => c,
        })
        .collect();
    let propre = propre.trim().trim_end_matches('.').to_string();
    let base = if propre.is_empty() {
        textes::FICHIER_PAR_DEFAUT
    } else {
        &propre
    };
    format!("{base}.txt")
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::*;
    use noyau::{Moteur, Touche};

    /// Dossier temporaire propre à un test, supprimé à la fin.
    struct DossierTemporaire(PathBuf);

    impl DossierTemporaire {
        fn nouveau(nom: &str) -> Self {
            let chemin = std::env::temp_dir().join(format!(
                "calculatrice-chantier-test-{}-{nom}",
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&chemin);
            fs::create_dir_all(&chemin).unwrap();
            DossierTemporaire(chemin)
        }

        fn depot(&self) -> Depot {
            Depot::dans(&self.0)
        }
    }

    impl Drop for DossierTemporaire {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn fichier_absent_valeurs_par_defaut() {
        let d = DossierTemporaire::nouveau("absent");
        let c = charger_etat(&d.depot());
        assert!(!c.illisible);
        assert_eq!(c.etat, EtatPersistant::default());
    }

    #[test]
    fn aller_retour_de_l_etat() {
        let d = DossierTemporaire::nouveau("aller-retour");
        let mut moteur = Moteur::default();
        for t in [
            Touche::Chiffre(7),
            Touche::Pouces,
            Touche::Pente,
            Touche::Chiffre(5),
            Touche::Pieds,
            Touche::MPlus,
            Touche::Chiffre(9),
            Touche::Stocker,
            Touche::Chiffre(1),
        ] {
            moteur.appuyer(t);
        }
        moteur.preferences_mut().resolution = 8;
        moteur.marquer_bienvenue_vue();
        let etat = moteur.etat_persistant();
        enregistrer_etat(&d.depot(), &etat).unwrap();
        // L'écriture est atomique : il ne reste pas de fichier temporaire.
        assert!(!d.0.join("etat.json.tmp").exists());
        let relu = charger_etat(&d.depot());
        assert!(!relu.illisible);
        assert_eq!(relu.etat.preferences, etat.preferences);
        assert_eq!(relu.etat.memoire, etat.memoire);
        assert_eq!(relu.etat.registres, etat.registres);
        assert_eq!(relu.etat.ruban, etat.ruban);
        assert_eq!(
            relu.etat.pente.map(|p| p.ratio),
            etat.pente.map(|p| p.ratio)
        );
        assert!(relu.etat.bienvenue_vue);
        // Sur le bureau, le JSON est lisible et en UTF-8.
        let texte = fs::read_to_string(d.0.join("etat.json")).unwrap();
        assert!(texte.contains("\"resolution\": 8"));
    }

    #[test]
    fn fichier_corrompu_mis_de_cote() {
        let d = DossierTemporaire::nouveau("corrompu");
        fs::write(d.0.join("etat.json"), "{ ceci n'est pas du JSON").unwrap();
        let c = charger_etat(&d.depot());
        assert!(c.illisible);
        assert_eq!(c.etat, EtatPersistant::default());
        assert!(d.0.join("etat.corrompu.json").exists());
        assert!(!d.0.join("etat.json").exists());
    }

    #[test]
    fn version_inconnue_mise_de_cote() {
        let d = DossierTemporaire::nouveau("version");
        fs::write(d.0.join("etat.json"), "{ \"version\": 99 }").unwrap();
        assert!(charger_etat(&d.depot()).illisible);
        assert!(d.0.join("etat.corrompu.json").exists());
    }

    #[test]
    fn champs_manquants_tolerés() {
        // R-47 : un fichier partiel reste lisible.
        let d = DossierTemporaire::nouveau("partiel");
        fs::write(
            d.0.join("etat.json"),
            "{ \"version\": 1, \"preferences\": { \"resolution\": 32 } }",
        )
        .unwrap();
        let c = charger_etat(&d.depot());
        assert!(!c.illisible);
        assert_eq!(c.etat.preferences.resolution, 32);
        assert!(c.etat.preferences.separateur_milliers);
        assert_eq!(c.etat.memorises, noyau::ValeursMemorisees::default());
    }

    #[test]
    fn json_compact_relu_comme_le_json_lisible() {
        // Le navigateur conserve un JSON compact : il doit se relire pareil.
        let d = DossierTemporaire::nouveau("compact");
        let mut moteur = Moteur::default();
        for t in [Touche::Chiffre(4), Touche::Pieds, Touche::MPlus] {
            moteur.appuyer(t);
        }
        let etat = moteur.etat_persistant();
        fs::write(d.0.join("etat.json"), serde_json::to_string(&etat).unwrap()).unwrap();
        let relu = charger_etat(&d.depot());
        assert!(!relu.illisible);
        assert_eq!(relu.etat.memoire, etat.memoire);
        assert_eq!(relu.etat.ruban, etat.ruban);
    }

    #[test]
    fn aller_retour_des_rubans() {
        let d = DossierTemporaire::nouveau("rubans");
        assert!(charger_rubans(&d.depot()).is_empty());
        let ruban = RubanEnregistre {
            id: 1,
            nom: "Escalier du sous-sol".to_string(),
            date: "2026-11-19T08:04:00-05:00".to_string(),
            lignes: vec![LigneRuban {
                texte: "5 pi 7 1/2 po".to_string(),
                ..LigneRuban::default()
            }],
        };
        enregistrer_rubans(&d.depot(), std::slice::from_ref(&ruban)).unwrap();
        assert_eq!(charger_rubans(&d.depot()), vec![ruban.clone()]);
        assert_eq!(ruban.date_en_clair(), "19 novembre 2026, 8 h 04");
    }

    #[test]
    fn rubans_illisibles_mis_de_cote() {
        let d = DossierTemporaire::nouveau("rubans-illisibles");
        fs::write(
            d.0.join("rubans.json"),
            "{ \"version\": 99, \"rubans\": [] }",
        )
        .unwrap();
        assert!(charger_rubans(&d.depot()).is_empty());
        assert!(d.0.join("rubans.corrompu.json").exists());
        assert!(!d.0.join("rubans.json").exists());
    }

    #[test]
    fn aller_retour_des_saisies() {
        let d = DossierTemporaire::nouveau("saisies");
        assert!(charger_saisies(&d.depot()).is_empty());
        let mut saisies = Saisies::new();
        saisies.insert(
            "dalle".to_string(),
            vec!["24".to_string(), "20".to_string(), "4".to_string()],
        );
        enregistrer_saisies(&d.depot(), &saisies).unwrap();
        assert_eq!(charger_saisies(&d.depot()), saisies);
        // Une donnée illisible est simplement ignorée.
        fs::write(d.0.join("metiers.json"), "pas du JSON").unwrap();
        assert!(charger_saisies(&d.depot()).is_empty());
    }

    #[test]
    fn export_lisible_dans_le_bloc_notes() {
        let contenu = contenu_exporte("Montée\n= 5 pi 7 1/2 po\n");
        // Indicateur d'ordre des octets en tête, puis des fins de ligne Windows.
        assert_eq!(&contenu.as_bytes()[..3], &[0xEF, 0xBB, 0xBF]);
        assert_eq!(contenu, "\u{FEFF}Montée\r\n= 5 pi 7 1/2 po\r\n");
    }

    #[test]
    fn noms_de_fichier_surs() {
        assert_eq!(
            nom_de_fichier("Ruban du 19 novembre 2026, 8 h 04"),
            "Ruban du 19 novembre 2026, 8 h 04.txt"
        );
        assert_eq!(nom_de_fichier("a/b:c?"), "a-b-c-.txt");
        assert_eq!(nom_de_fichier("  "), "ruban.txt");
    }
}
