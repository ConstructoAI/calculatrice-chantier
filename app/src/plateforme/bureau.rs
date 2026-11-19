//! Version de bureau : fichiers dans le dossier de données de l'utilisateur,
//! boîte « Enregistrer sous » du système, plein écran appliqué à la fenêtre.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use super::{Export, Fichier, Lecture};
use crate::textes;

/// L'application tourne-t-elle dans un navigateur?
pub const NAVIGATEUR: bool = false;
/// Les fichiers JSON restent lisibles dans un éditeur de texte (section 3).
pub const JSON_LISIBLE: bool = true;
/// Intervalle minimal entre deux écritures de l'état, en secondes (R-45).
pub const INTERVALLE_SAUVEGARDE: f64 = 1.0;

/// Dossier de données de l'utilisateur (section 17.1).
///
/// `eframe::storage_dir` n'est offert qu'avec la fonctionnalité `persistence`,
/// que l'application n'active pas : on applique donc le repli prévu, soit
/// `%APPDATA%\CalculatriceChantier\` sous Windows.
fn dossier_de_donnees() -> PathBuf {
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

/// Là où sont conservées les données : un dossier de fichiers.
pub struct Depot {
    dossier: PathBuf,
}

impl Depot {
    /// Le dossier de données de l'utilisateur.
    pub fn ouvrir() -> Self {
        Depot {
            dossier: dossier_de_donnees(),
        }
    }

    /// Un dossier précis, pour les tests.
    #[cfg(test)]
    pub fn dans(dossier: &Path) -> Self {
        Depot {
            dossier: dossier.to_path_buf(),
        }
    }

    /// Dossier des données, affiché dans « À propos ».
    pub fn emplacement(&self) -> Option<String> {
        Some(self.dossier.display().to_string())
    }

    pub fn lire(&self, nom: &str) -> Lecture {
        match fs::read_to_string(self.dossier.join(nom)) {
            Ok(texte) => Lecture::Texte(texte),
            Err(erreur) if erreur.kind() == io::ErrorKind::NotFound => Lecture::Absent,
            Err(_) => Lecture::Illisible,
        }
    }

    pub fn ecrire(&self, nom: &str, contenu: &str) -> io::Result<()> {
        ecrire_atomique(&self.dossier.join(nom), contenu)
    }

    /// Renomme une donnée illisible pour ne pas l'écraser (R-46).
    pub fn mettre_de_cote(&self, nom: &str, nouveau_nom: &str) {
        let _ = fs::rename(self.dossier.join(nom), self.dossier.join(nouveau_nom));
    }
}

/// Demande où enregistrer le fichier, puis l'écrit.
pub fn exporter(fichier: &Fichier<'_>) -> Export {
    let choix = rfd::FileDialog::new()
        .set_title(fichier.titre)
        .set_file_name(fichier.nom)
        .add_filter(fichier.filtre, &["txt"])
        .save_file();
    match choix {
        None => Export::Annule,
        Some(chemin) => match fs::write(chemin, fichier.contenu) {
            Ok(()) => Export::Reussi,
            Err(_) => Export::Echec,
        },
    }
}

/// Le bouton de plein écran a-t-il sa place dans la barre supérieure?
pub fn plein_ecran_possible() -> bool {
    true
}

/// Sans objet sur le bureau : le plein écran y est une préférence, que
/// l'application applique elle-même à sa fenêtre.
pub fn basculer_plein_ecran() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ecriture_atomique_sans_fichier_temporaire() {
        let dossier = std::env::temp_dir().join(format!(
            "calculatrice-chantier-test-{}-depot",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dossier);
        let depot = Depot::dans(&dossier);
        assert!(matches!(depot.lire("essai.json"), Lecture::Absent));
        // Le dossier est créé à la première écriture.
        depot.ecrire("essai.json", "{ \"a\": 1 }").unwrap();
        assert!(!dossier.join("essai.json.tmp").exists());
        assert!(matches!(depot.lire("essai.json"), Lecture::Texte(t) if t == "{ \"a\": 1 }"));
        depot.mettre_de_cote("essai.json", "essai.corrompu.json");
        assert!(matches!(depot.lire("essai.json"), Lecture::Absent));
        assert!(dossier.join("essai.corrompu.json").exists());
        // Un contenu qui n'est pas de l'UTF-8 est illisible, pas absent.
        fs::write(dossier.join("essai.json"), [0xFF, 0xFE, 0x00]).unwrap();
        assert!(matches!(depot.lire("essai.json"), Lecture::Illisible));
        let _ = fs::remove_dir_all(&dossier);
    }
}
