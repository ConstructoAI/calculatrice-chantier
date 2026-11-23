//! Les touches du clavier (section 5.2).

/// Une touche de la calculatrice.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Touche {
    // Rangée 1
    Pente,
    Montee,
    Course,
    Diag,
    AretierNoue,
    // Rangée 2
    OngletCompose,
    Escalier,
    Arc,
    Cercle,
    Empannon,
    // Rangée 3
    Metres,
    Longueur,
    Largeur,
    Hauteur,
    Pourcent,
    // Rangée 4
    Verges,
    Pieds,
    Pouces,
    Fraction,
    Effacer,
    // Rangées 5 à 8
    Conv,
    Chiffre(u8),
    Diviser,
    Stocker,
    Multiplier,
    Rappel,
    Soustraire,
    MPlus,
    Virgule,
    Egal,
    Additionner,
    // Hors grille
    RetourArriere,
    FractionRapide {
        num: u32,
        den: u32,
    },
    /// Touche d'un outil de métier branché sur le clavier (section 23) : le
    /// rang d'un champ de l'outil, ou celui de la touche `Résultat`.
    Outil(u8),
    /// Le clavier des heures (section 22.20) : les heures et les minutes
    /// d'une durée, comme les pieds et les pouces.
    Heures,
    Minutes,
}

impl Touche {
    /// Les 40 touches de la grille, rangée par rangée (section 5.1).
    pub const GRILLE: [[Touche; 5]; 8] = [
        [
            Touche::Pente,
            Touche::Montee,
            Touche::Course,
            Touche::Diag,
            Touche::AretierNoue,
        ],
        [
            Touche::OngletCompose,
            Touche::Escalier,
            Touche::Arc,
            Touche::Cercle,
            Touche::Empannon,
        ],
        [
            Touche::Metres,
            Touche::Longueur,
            Touche::Largeur,
            Touche::Hauteur,
            Touche::Pourcent,
        ],
        [
            Touche::Verges,
            Touche::Pieds,
            Touche::Pouces,
            Touche::Fraction,
            Touche::Effacer,
        ],
        [
            Touche::Conv,
            Touche::Chiffre(7),
            Touche::Chiffre(8),
            Touche::Chiffre(9),
            Touche::Diviser,
        ],
        [
            Touche::Stocker,
            Touche::Chiffre(4),
            Touche::Chiffre(5),
            Touche::Chiffre(6),
            Touche::Multiplier,
        ],
        [
            Touche::Rappel,
            Touche::Chiffre(1),
            Touche::Chiffre(2),
            Touche::Chiffre(3),
            Touche::Soustraire,
        ],
        [
            Touche::MPlus,
            Touche::Chiffre(0),
            Touche::Virgule,
            Touche::Egal,
            Touche::Additionner,
        ],
    ];

    /// Vrai si la touche possède une fonction secondaire (étiquette au-dessus).
    pub fn a_une_fonction_secondaire(self) -> bool {
        !matches!(
            self,
            Touche::Metres
                | Touche::Verges
                | Touche::Pieds
                | Touche::Pouces
                | Touche::Fraction
                | Touche::Conv
                | Touche::RetourArriere
                | Touche::FractionRapide { .. }
                | Touche::Outil(_)
                | Touche::Heures
                | Touche::Minutes
        )
    }
}

/// Fractions de la bande de fractions rapides, selon la résolution (section 7.6).
pub fn fractions_rapides(resolution: u32) -> Vec<(u32, u32)> {
    let den = match resolution {
        2 => 2,
        4 => 4,
        8 => 8,
        _ => 16,
    };
    (1..den)
        .map(|num| {
            let mut n = num;
            let mut d = den;
            while n % 2 == 0 && d % 2 == 0 {
                n /= 2;
                d /= 2;
            }
            (n, d)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bandes_de_fractions_rapides() {
        assert_eq!(fractions_rapides(2), vec![(1, 2)]);
        assert_eq!(fractions_rapides(4), vec![(1, 4), (1, 2), (3, 4)]);
        assert_eq!(fractions_rapides(8).len(), 7);
        let seiziemes = fractions_rapides(16);
        assert_eq!(seiziemes.len(), 15);
        assert_eq!(seiziemes[0], (1, 16));
        assert_eq!(seiziemes[7], (1, 2));
        assert_eq!(seiziemes[14], (15, 16));
        assert_eq!(fractions_rapides(64), seiziemes);
    }
}
