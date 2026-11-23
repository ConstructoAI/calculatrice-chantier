//! Version de bureau : fichiers dans le dossier de données de l'utilisateur,
//! boîte « Enregistrer sous » du système, plein écran appliqué à la fenêtre.

use std::cell::RefCell;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use noyau::langue::Langue;

use super::{Arrivee, Export, Fichier, FichierChoisi, Installation, Lecture};
use crate::{pieces, textes};

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
fn ecrire_atomique(chemin: &Path, contenu: &[u8]) -> io::Result<()> {
    if let Some(parent) = chemin.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut nom_temporaire = chemin.as_os_str().to_owned();
    nom_temporaire.push(".tmp");
    let temporaire = PathBuf::from(nom_temporaire);
    {
        let mut fichier = fs::File::create(&temporaire)?;
        fichier.write_all(contenu)?;
        fichier.sync_all()?;
    }
    fs::rename(&temporaire, chemin)
}

/// Sous-dossier des pièces jointes du bloc-notes, dans le dossier de données (R-95).
const DOSSIER_DES_PIECES: &str = "pieces-jointes";

thread_local! {
    /// Ce qui attend d'être relevé par l'application (`arrivees`).
    static ARRIVEES: RefCell<Vec<Arrivee>> = const { RefCell::new(Vec::new()) };
}

fn deposer(arrivee: Arrivee) {
    ARRIVEES.with(|a| a.borrow_mut().push(arrivee));
}

/// Relève ce qui est arrivé depuis la dernière fois : pièces jointes lues,
/// fichiers choisis, échecs d'écriture.
pub fn arrivees() -> Vec<Arrivee> {
    ARRIVEES.with(|a| std::mem::take(&mut *a.borrow_mut()))
}

/// Sans objet sur le bureau : tout y répond tout de suite.
pub fn retenir_le_contexte(_ctx: &eframe::egui::Context) {}

/// Ouvre la boîte « Ouvrir » du système pour choisir un ou plusieurs fichiers
/// (des photos seulement, si `photos`). Le résultat arrive par `arrivees`.
pub fn choisir_des_fichiers(photos: bool) {
    let titre = if photos {
        textes::TITRE_CHOISIR_PHOTOS.traduit()
    } else {
        textes::TITRE_CHOISIR_FICHIERS.traduit()
    };
    let mut boite = rfd::FileDialog::new().set_title(titre);
    if photos {
        boite = boite.add_filter(textes::FILTRE_PHOTOS.traduit(), &["jpg", "jpeg", "png"]);
    }
    let Some(chemins) = boite.pick_files() else {
        return;
    };
    let fichiers = chemins
        .into_iter()
        .map(|chemin| {
            let nom = chemin
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            let taille = fs::metadata(&chemin).map(|m| m.len()).unwrap_or(u64::MAX);
            let octets = if taille > pieces::TAILLE_MAX_LECTURE {
                None
            } else {
                fs::read(&chemin).ok()
            };
            FichierChoisi {
                nom,
                type_mime: String::new(),
                octets,
            }
        })
        .collect();
    deposer(Arrivee::Choisis(fichiers));
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
        ecrire_atomique(&self.dossier.join(nom), contenu.as_bytes())
    }

    /// Renomme une donnée illisible pour ne pas l'écraser (R-46).
    pub fn mettre_de_cote(&self, nom: &str, nouveau_nom: &str) {
        let _ = fs::rename(self.dossier.join(nom), self.dossier.join(nouveau_nom));
    }

    fn chemin_de_piece(&self, cle: &str) -> PathBuf {
        self.dossier.join(DOSSIER_DES_PIECES).join(cle)
    }

    /// Garde une pièce jointe sous sa clé; un échec arrive par `arrivees`.
    pub fn ecrire_piece(&self, cle: &str, octets: Vec<u8>) {
        if ecrire_atomique(&self.chemin_de_piece(cle), &octets).is_err() {
            deposer(Arrivee::EchecEcriture);
        }
    }

    /// Lit une pièce jointe; elle arrive par `arrivees`.
    pub fn lire_piece(&self, cle: &str) {
        let octets = fs::read(self.chemin_de_piece(cle)).ok();
        deposer(Arrivee::Lue {
            cle: cle.to_string(),
            octets,
        });
    }

    /// Efface une pièce jointe (rien si elle n'existe pas).
    pub fn supprimer_piece(&self, cle: &str) {
        let _ = fs::remove_file(self.chemin_de_piece(cle));
    }

    /// Ouvre une pièce jointe dans le programme que le poste associe à son
    /// type (le lecteur de PDF, par exemple). Faux si c'est impossible.
    pub fn ouvrir_piece(&self, cle: &str) -> bool {
        let chemin = self.chemin_de_piece(cle);
        if !chemin.is_file() {
            return false;
        }
        // L'explorateur reçoit le chemin tel quel : aucun interpréteur de
        // commandes ne le relit.
        let programme = if cfg!(windows) {
            "explorer"
        } else if cfg!(target_os = "macos") {
            "open"
        } else {
            "xdg-open"
        };
        std::process::Command::new(programme)
            .arg(&chemin)
            .spawn()
            .is_ok()
    }
}

/// Le manuel d'utilisation, embarqué dans le programme, dans chaque langue.
const MANUEL: &str = include_str!("../../manuel.html");
const MANUEL_ANGLAIS: &str = include_str!("../../manual.html");

/// Adresse du manuel d'utilisation, dans la langue en cours, à ouvrir dans le
/// navigateur. La page est d'abord copiée dans le dossier de données : elle se
/// lit ainsi sans réseau. Si la copie échoue, c'est l'adresse de la page
/// publiée qui est rendue.
pub fn adresse_du_manuel(depot: &Depot) -> String {
    let nom = textes::FICHIER_DU_MANUEL.traduit();
    let page = match noyau::langue::en_cours() {
        Langue::Francais => MANUEL,
        Langue::Anglais => MANUEL_ANGLAIS,
    };
    match depot.ecrire(nom, page) {
        Ok(()) => {
            let chemin = depot.dossier.join(nom);
            adresse_de_fichier(&std::path::absolute(&chemin).unwrap_or(chemin))
        }
        Err(_) => textes::ADRESSE_DU_MANUEL.traduit().to_string(),
    }
}

/// Langue du système, si c'est le français ou l'anglais : elle sert au premier
/// lancement seulement (section 24). Les langues préférées sont examinées dans
/// l'ordre.
pub fn langue_de_l_appareil() -> Option<Langue> {
    sys_locale::get_locales().find_map(|langue| Langue::d_apres(&langue))
}

/// Sur le bureau, le titre de la fenêtre est écrit par l'application : rien
/// d'autre ne dépend de la langue.
pub fn regler_la_page(_langue: Langue, _titre: &str) {}

/// Adresse `file:` d'un fichier du poste : les barres obliques inverses de
/// Windows deviennent des barres obliques, et tout caractère hors des lettres,
/// des chiffres et de quelques signes est codé en pourcentage.
fn adresse_de_fichier(chemin: &Path) -> String {
    let texte = chemin.to_string_lossy().replace('\\', "/");
    let mut adresse = String::from("file://");
    if !texte.starts_with('/') {
        adresse.push('/');
    }
    for octet in texte.bytes() {
        match octet {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b'/' | b':' => {
                adresse.push(char::from(octet))
            }
            autre => adresse.push_str(&format!("%{autre:02X}")),
        }
    }
    adresse
}

/// Demande où enregistrer le fichier, puis l'écrit.
pub fn exporter(fichier: &Fichier<'_>) -> Export {
    let choix = rfd::FileDialog::new()
        .set_title(fichier.titre)
        .set_file_name(fichier.nom)
        .add_filter(fichier.filtre, &[fichier.extension])
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

/// La version de bureau est déjà un programme du poste : rien à installer.
pub fn installation() -> Installation {
    Installation::Aucune
}

/// Sans objet sur le bureau.
pub fn installer() {}

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
        // Une pièce jointe : écrite, relue par `arrivees`, puis effacée.
        let _ = arrivees();
        depot.ecrire_piece("7.pdf", vec![1, 2, 3]);
        assert!(dossier.join("pieces-jointes").join("7.pdf").is_file());
        depot.lire_piece("7.pdf");
        depot.supprimer_piece("7.pdf");
        depot.lire_piece("7.pdf");
        let recues = arrivees();
        assert!(matches!(
            recues.as_slice(),
            [
                Arrivee::Lue { cle: a, octets: Some(o) },
                Arrivee::Lue { cle: b, octets: None },
            ] if a == "7.pdf" && o == &[1, 2, 3] && b == "7.pdf"
        ));
        assert!(matches!(depot.lire("essai.json"), Lecture::Texte(t) if t == "{ \"a\": 1 }"));
        depot.mettre_de_cote("essai.json", "essai.corrompu.json");
        assert!(matches!(depot.lire("essai.json"), Lecture::Absent));
        assert!(dossier.join("essai.corrompu.json").exists());
        // Un contenu qui n'est pas de l'UTF-8 est illisible, pas absent.
        fs::write(dossier.join("essai.json"), [0xFF, 0xFE, 0x00]).unwrap();
        assert!(matches!(depot.lire("essai.json"), Lecture::Illisible));
        let _ = fs::remove_dir_all(&dossier);
    }

    #[test]
    fn adresse_d_un_fichier_du_poste() {
        assert_eq!(
            adresse_de_fichier(Path::new("/tmp/Jean Tremblay/manuel.html")),
            "file:///tmp/Jean%20Tremblay/manuel.html"
        );
        assert_eq!(
            adresse_de_fichier(Path::new(
                "C:\\Users\\Zoé\\AppData\\Roaming\\CalculatriceChantier\\manuel.html"
            )),
            "file:///C:/Users/Zo%C3%A9/AppData/Roaming/CalculatriceChantier/manuel.html"
        );
    }

    #[test]
    fn le_manuel_est_copie_dans_le_dossier_de_donnees() {
        let dossier = std::env::temp_dir().join(format!(
            "calculatrice-chantier-manuel-{}",
            std::process::id()
        ));
        let depot = Depot::dans(&dossier);
        let adresse = adresse_du_manuel(&depot);
        assert!(adresse.starts_with("file:///"), "{adresse}");
        assert!(adresse.ends_with("/manuel.html"), "{adresse}");
        let copie = fs::read_to_string(dossier.join("manuel.html")).unwrap();
        assert!(copie.starts_with("<!doctype html>"));
        assert!(copie.contains("Manuel d'utilisation"));
        let _ = fs::remove_dir_all(&dossier);
    }
}
