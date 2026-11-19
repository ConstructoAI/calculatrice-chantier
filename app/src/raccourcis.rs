//! Table des raccourcis clavier (section 16.5).

use eframe::egui::Key;
use noyau::Touche;

use crate::textes;

/// Action déclenchée par une touche spéciale du clavier d'ordinateur.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Touche(Touche),
    Aide,
    PleinEcran,
    Preferences,
    Ruban,
}

/// Caractères imprimables et leur touche. Les lettres sont insensibles à la casse.
const CARACTERES: &[(char, Touche)] = &[
    (',', Touche::Virgule),
    ('.', Touche::Virgule),
    ('+', Touche::Additionner),
    ('-', Touche::Soustraire),
    ('*', Touche::Multiplier),
    (':', Touche::Diviser),
    ('/', Touche::Fraction),
    ('=', Touche::Egal),
    ('\'', Touche::Pieds),
    ('"', Touche::Pouces),
    ('%', Touche::Pourcent),
    (' ', Touche::Conv),
    ('v', Touche::Verges),
    ('m', Touche::Metres),
    ('s', Touche::Stocker),
    ('r', Touche::Rappel),
    ('p', Touche::Pente),
    ('t', Touche::Montee),
    ('c', Touche::Course),
    ('d', Touche::Diag),
    ('a', Touche::AretierNoue),
    ('o', Touche::OngletCompose),
    ('e', Touche::Escalier),
    ('b', Touche::Arc),
    ('i', Touche::Cercle),
    ('j', Touche::Empannon),
    ('l', Touche::Longueur),
    ('g', Touche::Largeur),
    ('h', Touche::Hauteur),
];

/// Touche de la calculatrice correspondant à un caractère tapé.
pub fn touche_du_caractere(c: char) -> Option<Touche> {
    if let Some(chiffre) = c.to_digit(10) {
        return Some(Touche::Chiffre(chiffre as u8));
    }
    let minuscule = c.to_lowercase().next().unwrap_or(c);
    CARACTERES
        .iter()
        .find(|(car, _)| *car == minuscule)
        .map(|(_, t)| *t)
}

/// Action d'une touche spéciale (non imprimable), avec ou sans `Ctrl`.
pub fn action_de_la_touche(touche: Key, ctrl: bool) -> Option<Action> {
    if ctrl {
        return match touche {
            Key::Comma => Some(Action::Preferences),
            Key::R => Some(Action::Ruban),
            _ => None,
        };
    }
    match touche {
        Key::Enter => Some(Action::Touche(Touche::Egal)),
        Key::Backspace => Some(Action::Touche(Touche::RetourArriere)),
        Key::Escape | Key::Delete => Some(Action::Touche(Touche::Effacer)),
        Key::Insert => Some(Action::Touche(Touche::MPlus)),
        Key::F1 => Some(Action::Aide),
        Key::F11 => Some(Action::PleinEcran),
        _ => None,
    }
}

/// Raccourci d'une touche, tel qu'on le rappelle dans son infobulle.
pub fn raccourci(touche: Touche) -> Option<String> {
    match touche {
        Touche::Chiffre(c) => Some(c.to_string()),
        Touche::Virgule => Some(textes::l_un_ou_l_autre(",", ".")),
        Touche::Egal => Some(textes::l_un_ou_l_autre(textes::TOUCHE_ENTREE, "=")),
        Touche::Effacer => Some(textes::l_un_ou_l_autre(
            textes::TOUCHE_ECHAP,
            textes::TOUCHE_SUPPR,
        )),
        Touche::MPlus => Some(textes::TOUCHE_INSER.to_string()),
        Touche::RetourArriere => Some(textes::TOUCHE_RETOUR_ARRIERE.to_string()),
        Touche::Conv => Some(textes::TOUCHE_ESPACE.to_string()),
        autre => CARACTERES
            .iter()
            .find(|(_, t)| *t == autre)
            .map(|(c, _)| c.to_uppercase().to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caracteres() {
        assert_eq!(touche_du_caractere('7'), Some(Touche::Chiffre(7)));
        assert_eq!(touche_du_caractere('.'), Some(Touche::Virgule));
        assert_eq!(touche_du_caractere('P'), Some(Touche::Pente));
        assert_eq!(touche_du_caractere('p'), Some(Touche::Pente));
        assert_eq!(touche_du_caractere('"'), Some(Touche::Pouces));
        assert_eq!(touche_du_caractere(' '), Some(Touche::Conv));
        assert_eq!(touche_du_caractere('z'), None);
    }

    #[test]
    fn chaque_caractere_mene_a_une_seule_touche() {
        for (i, (c, _)) in CARACTERES.iter().enumerate() {
            assert!(
                CARACTERES[i + 1..].iter().all(|(autre, _)| autre != c),
                "« {c} » en double"
            );
        }
    }

    #[test]
    fn touches_speciales() {
        assert_eq!(
            action_de_la_touche(Key::Enter, false),
            Some(Action::Touche(Touche::Egal))
        );
        assert_eq!(
            action_de_la_touche(Key::Delete, false),
            Some(Action::Touche(Touche::Effacer))
        );
        assert_eq!(
            action_de_la_touche(Key::F11, false),
            Some(Action::PleinEcran)
        );
        assert_eq!(
            action_de_la_touche(Key::Comma, true),
            Some(Action::Preferences)
        );
        assert_eq!(action_de_la_touche(Key::R, true), Some(Action::Ruban));
        assert_eq!(action_de_la_touche(Key::R, false), None);
    }

    #[test]
    fn rappel_dans_les_infobulles() {
        assert_eq!(raccourci(Touche::Montee).as_deref(), Some("T"));
        assert_eq!(raccourci(Touche::Egal).as_deref(), Some("Entrée ou ="));
        assert_eq!(raccourci(Touche::Pieds).as_deref(), Some("'"));
        assert_eq!(raccourci(Touche::FractionRapide { num: 1, den: 2 }), None);
    }
}
