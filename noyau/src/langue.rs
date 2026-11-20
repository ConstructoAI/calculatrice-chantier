//! Langue de l'interface : français ou anglais (section 24).
//!
//! Chaque texte montré à l'utilisateur existe dans les deux langues (`Texte`),
//! et se lit dans la langue en cours (`Texte::traduit`). La langue en cours
//! règle aussi l'écriture des nombres et les symboles d'unités : « 1 234,5 pi² »
//! en français, « 1,234.5 ft² » en anglais.
//!
//! La langue en cours est propre au fil d'exécution : l'interface la règle
//! d'après les préférences, et les essais peuvent en changer sans se nuire.

use std::cell::Cell;
use std::fmt;

use serde::{Deserialize, Serialize};

/// Langue de l'interface.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Langue {
    #[default]
    Francais,
    Anglais,
}

impl Langue {
    /// Les langues offertes, dans l'ordre du bouton de langue.
    pub const TOUTES: [Langue; 2] = [Langue::Francais, Langue::Anglais];

    /// Code de la langue, tel que le bouton de langue l'écrit.
    pub fn code(self) -> &'static str {
        match self {
            Langue::Francais => "FR",
            Langue::Anglais => "EN",
        }
    }

    /// Nom de la langue, écrit dans cette langue.
    pub fn nom(self) -> &'static str {
        match self {
            Langue::Francais => "Français",
            Langue::Anglais => "English",
        }
    }

    /// L'autre langue : celle que donne une pression du bouton de langue.
    pub fn autre(self) -> Langue {
        match self {
            Langue::Francais => Langue::Anglais,
            Langue::Anglais => Langue::Francais,
        }
    }

    /// Langue désignée par une étiquette du système ou du navigateur :
    /// « fr-CA », « en_US.UTF-8 ». `None` pour toute autre langue.
    pub fn d_apres(etiquette: &str) -> Option<Langue> {
        let debut: String = etiquette
            .trim()
            .chars()
            .take_while(char::is_ascii_alphabetic)
            .flat_map(char::to_lowercase)
            .collect();
        match debut.as_str() {
            "fr" | "fra" | "fre" | "french" => Some(Langue::Francais),
            "en" | "eng" | "english" => Some(Langue::Anglais),
            _ => None,
        }
    }
}

thread_local! {
    static EN_COURS: Cell<Langue> = const { Cell::new(Langue::Francais) };
}

/// La langue en cours sur ce fil d'exécution (le français au départ).
pub fn en_cours() -> Langue {
    EN_COURS.with(Cell::get)
}

/// Change la langue en cours sur ce fil d'exécution.
pub fn choisir(langue: Langue) {
    EN_COURS.with(|en_cours| en_cours.set(langue));
}

/// Remet la langue précédente quand il sort de la portée.
#[must_use = "la langue précédente revient dès que cette valeur est abandonnée"]
pub struct Retour(Langue);

impl Drop for Retour {
    fn drop(&mut self) {
        choisir(self.0);
    }
}

/// Choisit une langue jusqu'à la fin de la portée; la précédente revient ensuite.
pub fn pour_la_portee(langue: Langue) -> Retour {
    let precedente = en_cours();
    choisir(langue);
    Retour(precedente)
}

/// Celui des deux textes qui est dans la langue en cours.
pub fn selon(fr: &'static str, en: &'static str) -> &'static str {
    match en_cours() {
        Langue::Francais => fr,
        Langue::Anglais => en,
    }
}

/// Un texte dans les deux langues.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Texte {
    pub fr: &'static str,
    pub en: &'static str,
}

/// Un texte en français et en anglais.
pub const fn t(fr: &'static str, en: &'static str) -> Texte {
    Texte { fr, en }
}

/// Un texte qui s'écrit de la même façon dans les deux langues : un symbole,
/// un sigle, un nom propre.
pub const fn tel(texte: &'static str) -> Texte {
    Texte {
        fr: texte,
        en: texte,
    }
}

impl Texte {
    /// Aucun texte.
    pub const VIDE: Texte = tel("");

    /// Le texte dans la langue en cours.
    pub fn traduit(self) -> &'static str {
        self.dans(en_cours())
    }

    /// Le texte dans une langue donnée.
    pub fn dans(self, langue: Langue) -> &'static str {
        match langue {
            Langue::Francais => self.fr,
            Langue::Anglais => self.en,
        }
    }

    /// Vrai s'il n'y a de texte dans aucune des deux langues.
    pub const fn est_vide(self) -> bool {
        self.fr.is_empty() && self.en.is_empty()
    }
}

impl fmt::Display for Texte {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.traduit())
    }
}

impl From<Texte> for String {
    fn from(texte: Texte) -> String {
        texte.traduit().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_francais_au_depart_puis_la_langue_choisie() {
        let texte = t("Longueur", "Length");
        assert_eq!(en_cours(), Langue::Francais);
        assert_eq!(texte.traduit(), "Longueur");
        {
            let _anglais = pour_la_portee(Langue::Anglais);
            assert_eq!(texte.traduit(), "Length");
            assert_eq!(texte.to_string(), "Length");
            assert_eq!(selon("oui", "yes"), "yes");
        }
        // La langue précédente est revenue.
        assert_eq!(en_cours(), Langue::Francais);
        assert_eq!(selon("oui", "yes"), "oui");
        assert_eq!(tel("kW").dans(Langue::Anglais), "kW");
        assert!(Texte::VIDE.est_vide() && !texte.est_vide());
    }

    #[test]
    fn langue_d_apres_une_etiquette_du_systeme() {
        assert_eq!(Langue::d_apres("fr-CA"), Some(Langue::Francais));
        assert_eq!(Langue::d_apres("fr"), Some(Langue::Francais));
        assert_eq!(Langue::d_apres("FR_ca.UTF-8"), Some(Langue::Francais));
        assert_eq!(Langue::d_apres("en-US"), Some(Langue::Anglais));
        assert_eq!(Langue::d_apres(" en_GB"), Some(Langue::Anglais));
        assert_eq!(Langue::d_apres("es-MX"), None);
        assert_eq!(Langue::d_apres(""), None);
        assert_eq!(Langue::Francais.autre(), Langue::Anglais);
        assert_eq!(Langue::Anglais.autre(), Langue::Francais);
    }
}
