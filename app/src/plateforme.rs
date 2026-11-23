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
//! | manuel | copie dans le dossier de données | page du site |
//! | langue de l'appareil | langue du système | langue du navigateur |
//! | pièces jointes | sous-dossier `pieces-jointes` | base IndexedDB du site |
//! | choix de fichiers | boîte « Ouvrir » du système | champ de fichier du navigateur |
//!
//! Les pièces jointes et le choix de fichiers ne répondent pas tout de suite
//! dans un navigateur : leur résultat arrive plus tard, et l'application le
//! relève à chaque image par `arrivees()`. Sur le bureau, il arrive aussitôt,
//! par le même chemin.

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

/// Un fichier à exporter : un ruban, une note, le plan dessiné (PDF et DXF),
/// ou une pièce jointe du bloc-notes.
// Le titre, le filtre et l'extension ne servent que sur le bureau.
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
pub struct Fichier<'a> {
    /// Titre de la boîte « Enregistrer sous » (bureau).
    pub titre: &'a str,
    /// Nom du type de fichier dans cette boîte (bureau).
    pub filtre: &'a str,
    /// Extension du type de fichier, sans le point (bureau).
    pub extension: &'a str,
    /// Type du fichier pour le navigateur (« application/pdf »).
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    pub type_mime: &'a str,
    /// Nom de fichier proposé, extension comprise.
    pub nom: &'a str,
    pub contenu: &'a [u8],
}

/// Ce que l'utilisateur va choisir.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChoixDeFichiers {
    /// Des photos à joindre (sur un téléphone : l'appareil photo ou la galerie).
    Photos,
    /// Des fichiers de tout type à joindre.
    Tous,
    /// Des dessins enregistrés en fichier (JSON), à rouvrir.
    Dessins,
}

/// Un fichier que l'utilisateur a choisi : à joindre à une note, ou un dessin à rouvrir.
pub struct FichierChoisi {
    /// Son nom, extension comprise.
    pub nom: String,
    /// Son type, quand le navigateur le donne (« image/jpeg »); vide sur le bureau.
    pub type_mime: String,
    /// Son contenu; rien s'il est trop gros pour être lu, ou illisible.
    pub octets: Option<Vec<u8>>,
}

/// Résultat d'une opération qui ne répond pas tout de suite dans un navigateur.
pub enum Arrivee {
    /// Une pièce jointe lue; rien si elle est absente ou illisible.
    Lue {
        cle: String,
        octets: Option<Vec<u8>>,
    },
    /// Les fichiers choisis par l'utilisateur.
    Choisis(Vec<FichierChoisi>),
    /// Une pièce jointe n'a pas pu être enregistrée (espace plein, refus).
    EchecEcriture,
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
