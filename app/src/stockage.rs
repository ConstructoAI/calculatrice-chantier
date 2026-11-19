//! Lecture et écriture des fichiers de données (section 17).

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use chrono::{Datelike, Timelike};
use noyau::{EtatPersistant, LigneRuban};
use serde::{Deserialize, Serialize};

use crate::textes;

const FICHIER_ETAT: &str = "etat.json";
const FICHIER_ETAT_CORROMPU: &str = "etat.corrompu.json";
const FICHIER_RUBANS: &str = "rubans.json";
const FICHIER_RUBANS_CORROMPU: &str = "rubans.corrompu.json";
/// Version du format de `rubans.json`.
const VERSION_RUBANS: u32 = 1;
/// Nombre maximal de rubans enregistrés (annexe A.2).
pub const MAX_RUBANS: usize = 200;

/// Dossier de données de l'utilisateur (section 17.1).
///
/// `eframe::storage_dir` n'est offert qu'avec la fonctionnalité `persistence`,
/// que l'application n'active pas : on applique donc le repli prévu, soit
/// `%APPDATA%\CalculatriceChantier\` sous Windows.
pub fn dossier_de_donnees() -> PathBuf {
    let variable = |nom: &str| {
        std::env::var_os(nom)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
    };
    let base = if cfg!(windows) {
        variable("APPDATA")
    } else if cfg!(target_os = "macos") {
        variable("HOME").map(|h| h.join("Library").join("Application Support"))
    } else {
        variable("XDG_DATA_HOME")
            .or_else(|| variable("HOME").map(|h| h.join(".local").join("share")))
    };
    match base {
        Some(base) => base.join(textes::APP_ID),
        None => PathBuf::from("."),
    }
}

/// Écriture atomique : dans un fichier temporaire, puis renommage (R-44).
fn ecrire_atomique(chemin: &Path, contenu: &str) -> io::Result<()> {
    if let Some(parent) = chemin.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut nom_temporaire = chemin.as_os_str().to_owned();
    nom_temporaire.push(".tmp");
    let temporaire = PathBuf::from(nom_temporaire);
    {
        let mut fichier = fs::File::create(&temporaire)?;
        fichier.write_all(contenu.as_bytes())?;
        fichier.sync_all()?;
    }
    fs::rename(&temporaire, chemin)
}

/// Résultat de la lecture de `etat.json`.
pub struct Chargement {
    pub etat: EtatPersistant,
    /// Le fichier existait mais n'a pas pu être lu : il a été mis de côté (R-46).
    pub illisible: bool,
}

/// Lit l'état conservé. Fichier absent : valeurs par défaut. Fichier illisible
/// ou d'une version inconnue : mis de côté, valeurs par défaut (R-46).
pub fn charger_etat(dossier: &Path) -> Chargement {
    let chemin = dossier.join(FICHIER_ETAT);
    let texte = match fs::read_to_string(&chemin) {
        Ok(texte) => texte,
        Err(erreur) => {
            let illisible = erreur.kind() != io::ErrorKind::NotFound;
            if illisible {
                let _ = fs::rename(&chemin, dossier.join(FICHIER_ETAT_CORROMPU));
            }
            return Chargement {
                etat: EtatPersistant::default(),
                illisible,
            };
        }
    };
    match serde_json::from_str::<EtatPersistant>(&texte) {
        Ok(etat) if etat.version_connue() => Chargement {
            etat,
            illisible: false,
        },
        _ => {
            let _ = fs::rename(&chemin, dossier.join(FICHIER_ETAT_CORROMPU));
            Chargement {
                etat: EtatPersistant::default(),
                illisible: true,
            }
        }
    }
}

/// Enregistre l'état, en JSON lisible.
pub fn enregistrer_etat(dossier: &Path, etat: &EtatPersistant) -> io::Result<()> {
    let json = serde_json::to_string_pretty(etat).map_err(io::Error::other)?;
    ecrire_atomique(&dossier.join(FICHIER_ETAT), &json)
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

/// Lit les rubans enregistrés; liste vide si le fichier est absent ou illisible.
pub fn charger_rubans(dossier: &Path) -> Vec<RubanEnregistre> {
    let chemin = dossier.join(FICHIER_RUBANS);
    let Ok(texte) = fs::read_to_string(&chemin) else {
        return Vec::new();
    };
    match serde_json::from_str::<FichierRubans>(&texte) {
        Ok(fichier) if fichier.version == VERSION_RUBANS => fichier.rubans,
        _ => {
            let _ = fs::rename(&chemin, dossier.join(FICHIER_RUBANS_CORROMPU));
            Vec::new()
        }
    }
}

/// Enregistre les rubans, du plus récent au plus ancien.
pub fn enregistrer_rubans(dossier: &Path, rubans: &[RubanEnregistre]) -> io::Result<()> {
    let fichier = FichierRubans {
        version: VERSION_RUBANS,
        rubans: rubans.to_vec(),
    };
    let json = serde_json::to_string_pretty(&fichier).map_err(io::Error::other)?;
    ecrire_atomique(&dossier.join(FICHIER_RUBANS), &json)
}

/// Écrit un texte exporté : UTF-8 avec indicateur d'ordre des octets et fins de
/// ligne Windows, pour que le Bloc-notes garde les accents intacts.
pub fn exporter_texte(chemin: &Path, texte: &str) -> io::Result<()> {
    let mut contenu = String::with_capacity(texte.len() + 16);
    contenu.push('\u{FEFF}');
    for ligne in texte.lines() {
        contenu.push_str(ligne);
        contenu.push_str("\r\n");
    }
    fs::write(chemin, contenu)
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
    }

    impl Drop for DossierTemporaire {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn fichier_absent_valeurs_par_defaut() {
        let d = DossierTemporaire::nouveau("absent");
        let c = charger_etat(&d.0);
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
        enregistrer_etat(&d.0, &etat).unwrap();
        // L'écriture est atomique : il ne reste pas de fichier temporaire.
        assert!(!d.0.join("etat.json.tmp").exists());
        let relu = charger_etat(&d.0);
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
        // Le JSON est lisible et en UTF-8.
        let texte = fs::read_to_string(d.0.join("etat.json")).unwrap();
        assert!(texte.contains("\"resolution\": 8"));
    }

    #[test]
    fn fichier_corrompu_mis_de_cote() {
        let d = DossierTemporaire::nouveau("corrompu");
        fs::write(d.0.join("etat.json"), "{ ceci n'est pas du JSON").unwrap();
        let c = charger_etat(&d.0);
        assert!(c.illisible);
        assert_eq!(c.etat, EtatPersistant::default());
        assert!(d.0.join("etat.corrompu.json").exists());
        assert!(!d.0.join("etat.json").exists());
    }

    #[test]
    fn version_inconnue_mise_de_cote() {
        let d = DossierTemporaire::nouveau("version");
        fs::write(d.0.join("etat.json"), "{ \"version\": 99 }").unwrap();
        assert!(charger_etat(&d.0).illisible);
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
        let c = charger_etat(&d.0);
        assert!(!c.illisible);
        assert_eq!(c.etat.preferences.resolution, 32);
        assert!(c.etat.preferences.separateur_milliers);
        assert_eq!(c.etat.memorises, noyau::ValeursMemorisees::default());
    }

    #[test]
    fn aller_retour_des_rubans() {
        let d = DossierTemporaire::nouveau("rubans");
        assert!(charger_rubans(&d.0).is_empty());
        let ruban = RubanEnregistre {
            id: 1,
            nom: "Escalier du sous-sol".to_string(),
            date: "2026-11-19T08:04:00-05:00".to_string(),
            lignes: vec![LigneRuban {
                texte: "5 pi 7 1/2 po".to_string(),
                ..LigneRuban::default()
            }],
        };
        enregistrer_rubans(&d.0, std::slice::from_ref(&ruban)).unwrap();
        assert_eq!(charger_rubans(&d.0), vec![ruban.clone()]);
        assert_eq!(ruban.date_en_clair(), "19 novembre 2026, 8 h 04");
    }

    #[test]
    fn export_lisible_dans_le_bloc_notes() {
        let d = DossierTemporaire::nouveau("export");
        let chemin = d.0.join("ruban.txt");
        exporter_texte(&chemin, "Montée\n= 5 pi 7 1/2 po\n").unwrap();
        let octets = fs::read(&chemin).unwrap();
        assert_eq!(&octets[..3], &[0xEF, 0xBB, 0xBF]);
        let texte = String::from_utf8(octets).unwrap();
        assert!(texte.contains("Montée\r\n= 5 pi 7 1/2 po\r\n"));
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
