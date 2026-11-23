//! Noyau de calcul de la calculatrice de chantier.
//!
//! Ce crate ne connaît ni l'interface graphique ni le système de fichiers (R-48) :
//! il ne fait que du calcul et de la mise en forme.
#![forbid(unsafe_code)]

pub mod analyse;
pub mod cloison;
pub mod conversions;
pub mod coupe;
pub mod dessin;
pub mod erreurs;
pub mod etiquettes;
pub mod fonctions;
pub mod format;
pub mod heures;
pub mod langue;
pub mod maquette;
pub mod memoire;
pub mod metiers;
pub mod mobilier;
pub mod moteur;
pub mod niveau;
pub mod nivellement;
pub mod persistance;
pub mod plafond;
pub mod prefs;
pub mod ruban;
pub mod saisie;
pub mod touches;
pub mod unites;
pub mod valeur;

pub use erreurs::Erreur;
pub use format::{Segment, TexteValeur};
pub use langue::{Langue, Texte};
pub use moteur::{Ecran, Effet, Moteur, Prefixe, ToucheOutil};
pub use persistance::EtatPersistant;
pub use prefs::{Preferences, ValeursMemorisees};
pub use ruban::{LigneRuban, Ruban};
pub use touches::Touche;
pub use valeur::Valeur;
