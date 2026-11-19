//! Ce qui dépend de l'endroit où tourne l'application : le bureau (Windows,
//! Linux, macOS) ou un navigateur Web.
//!
//! Les deux sous-modules offrent exactement les mêmes noms; un seul est
//! compilé. Le reste de l'application ne contient aucun `cfg` de plateforme,
//! à trois exceptions près : le point d'entrée (`main.rs`), le chargement des
//! polices et l'icône de fenêtre (`theme.rs`), et la sauvegarde à la fermeture
//! (`on_exit` dans `application.rs`).
//!
//! | Élément | Bureau | Navigateur |
//! |---|---|---|
//! | `Depot` | fichiers du dossier de données | stockage local du site |
//! | `exporter` | boîte « Enregistrer sous » | téléchargement |
//! | plein écran | préférence appliquée à la fenêtre | bouton, à la demande |
//! | installation | sans objet | application Web installable |

#[cfg(not(target_arch = "wasm32"))]
mod bureau;
#[cfg(not(target_arch = "wasm32"))]
pub use bureau::*;

#[cfg(target_arch = "wasm32")]
mod web;
#[cfg(target_arch = "wasm32")]
pub use web::*;

/// Résultat de la lecture d'une donnée conservée.
pub enum Lecture {
    /// Rien n'a encore été enregistré sous ce nom.
    Absent,
    Texte(String),
    /// La donnée existe, mais elle n'a pas pu être lue.
    Illisible,
}

/// Un fichier texte à exporter.
// Le titre et le filtre ne servent que sur le bureau.
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
pub struct Fichier<'a> {
    /// Titre de la boîte « Enregistrer sous » (bureau).
    pub titre: &'a str,
    /// Nom du type de fichier dans cette boîte (bureau).
    pub filtre: &'a str,
    /// Nom de fichier proposé, extension comprise.
    pub nom: &'a str,
    pub contenu: &'a str,
}

/// Ce que le menu peut offrir pour installer l'application sur l'appareil.
// Sur le bureau, il n'y a rien à installer : seule la première variante sert.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub enum Installation {
    /// Sans objet : version de bureau, ou application déjà installée.
    Aucune,
    /// Le navigateur sait l'installer : il suffit de le lui demander.
    Proposee,
    /// Il faut passer par le menu du navigateur : l'application explique comment.
    Manuelle,
}

/// Résultat de l'export d'un fichier.
// Dans un navigateur, il n'y a pas de boîte de dialogue à annuler.
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
pub enum Export {
    Reussi,
    /// L'utilisateur a fermé la boîte de dialogue sans choisir de fichier.
    Annule,
    Echec,
}
