//! Les formes de départ du dessin (section 25.1, point 60) : un rectangle,
//! une pièce en L, une pièce en T, un rectangle au coin coupé à 45°. Une
//! forme pose ses côtés, fermés et sans mesure; on inscrit ensuite leurs
//! mesures au pavé, d'un côté au suivant, et les deux derniers se déduisent.

use crate::dessin::{Cote, Dessin, Sens};

/// Une forme de départ.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormeDeDepart {
    /// Quatre côtés : deux mesures.
    Rectangle,
    /// Six côtés : quatre mesures.
    EnL,
    /// Huit côtés : six mesures.
    EnT,
    /// Un rectangle au coin du haut à gauche coupé à 45° : trois mesures.
    CoinA45,
}

impl FormeDeDepart {
    /// Les formes, dans l'ordre de leurs boutons.
    pub const TOUTES: [FormeDeDepart; 4] = [
        FormeDeDepart::Rectangle,
        FormeDeDepart::EnL,
        FormeDeDepart::EnT,
        FormeDeDepart::CoinA45,
    ];

    /// Ses côtés : leur direction et leur longueur sur le croquis (en pouces :
    /// un rectangle de 12 pi × 10 pi, une pièce en L de 12 pi × 11 pi…), dans
    /// le sens des aiguilles d'une montre, du coin du haut à gauche — pour le
    /// coin à 45°, du bout de son biais sur le mur du haut. Les côtés à
    /// mesurer viennent d'abord : les deux derniers se déduisent des autres.
    pub fn trace(self) -> &'static [(Sens, f64)] {
        match self {
            FormeDeDepart::Rectangle => &[
                (Sens::Est, 144.0),
                (Sens::Sud, 120.0),
                (Sens::Ouest, 144.0),
                (Sens::Nord, 120.0),
            ],
            FormeDeDepart::EnL => &[
                (Sens::Est, 144.0),
                (Sens::Sud, 72.0),
                (Sens::Ouest, 60.0),
                (Sens::Sud, 60.0),
                (Sens::Ouest, 84.0),
                (Sens::Nord, 132.0),
            ],
            FormeDeDepart::EnT => &[
                (Sens::Est, 192.0),
                (Sens::Sud, 60.0),
                (Sens::Ouest, 60.0),
                (Sens::Sud, 84.0),
                (Sens::Ouest, 72.0),
                (Sens::Nord, 84.0),
                (Sens::Ouest, 60.0),
                (Sens::Nord, 60.0),
            ],
            // Le biais retranche 36 po de chaque mur du coin : 36 √2 po.
            FormeDeDepart::CoinA45 => &[
                (Sens::Est, 108.0),
                (Sens::Sud, 132.0),
                (Sens::Ouest, 144.0),
                (Sens::Nord, 96.0),
                (Sens::NordEst, 50.9),
            ],
        }
    }

    /// Combien de mesures il faut inscrire : les deux derniers côtés se
    /// déduisent.
    pub fn mesures_a_inscrire(self) -> usize {
        self.trace().len() - 2
    }

    /// Son contour, pour la dessiner sur son bouton : un sommet par côté
    /// (pouces du croquis, l'axe des y vers le bas), du départ au dernier coin.
    pub fn sommets(self) -> Vec<(f64, f64)> {
        let mut point = (0.0, 0.0);
        let mut sommets = vec![point];
        for &(sens, longueur) in self.trace() {
            let angle = sens.angle().to_radians();
            point = (
                point.0 + longueur * angle.cos(),
                point.1 + longueur * angle.sin(),
            );
            sommets.push(point);
        }
        // Le dernier côté revient au départ.
        sommets.pop();
        sommets
    }
}

impl Dessin {
    /// Pose une forme de départ (point 60) : ses côtés, fermés, sans mesure.
    /// Le reste du dessin ne change pas : un terrain reste un terrain.
    pub fn poser_une_forme(&mut self, forme: FormeDeDepart) {
        self.cotes = forme
            .trace()
            .iter()
            .map(|&(sens, croquis)| Cote {
                sens,
                croquis,
                saisie: String::new(),
            })
            .collect();
        self.fermee = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dessin::{Etat, Mesure};
    use crate::unites::Format;

    /// Inscrit les mesures comme au pavé : du premier côté, chaque mesure
    /// passe au prochain côté sans mesure, comme « Suivant ». Retourne le
    /// nombre de mesures qu'il a fallu et l'aire (pi²) de la forme complète.
    fn mesurer(forme: FormeDeDepart, mesures: &[&str]) -> (usize, f64) {
        let mut dessin = Dessin::default();
        dessin.poser_une_forme(forme);
        let mut rang = 0;
        for (k, mesure) in mesures.iter().enumerate() {
            dessin.cotes[rang].saisie = mesure.to_string();
            let solution = dessin.resoudre(Format::PiPo);
            if let Etat::Complete { aire, .. } = solution.etat {
                return (k + 1, aire / 144.0);
            }
            let nombre = dessin.cotes.len();
            rang = (1..=nombre)
                .map(|d| (rang + d) % nombre)
                .find(|&i| matches!(solution.mesures[i], Mesure::Manquante))
                .unwrap_or_else(|| panic!("{forme:?} : plus de côté sans mesure"));
        }
        panic!("{forme:?} : incomplète après {} mesures", mesures.len());
    }

    #[test]
    fn chaque_forme_se_ferme() {
        for forme in FormeDeDepart::TOUTES {
            let (x, y) =
                forme
                    .trace()
                    .iter()
                    .fold((0.0_f64, 0.0_f64), |(x, y), &(sens, longueur)| {
                        let angle = sens.angle().to_radians();
                        (x + longueur * angle.cos(), y + longueur * angle.sin())
                    });
            assert!(x.hypot(y) < 0.1, "{forme:?} ne se ferme pas : ({x}, {y})");
            assert_eq!(forme.sommets().len(), forme.trace().len());
        }
    }

    #[test]
    fn une_forme_se_complete_au_pave() {
        // (forme, mesures dans l'ordre des côtés, aire attendue en pi²)
        let cas: [(FormeDeDepart, &[&str], f64); 4] = [
            // 12 × 10 pi.
            (FormeDeDepart::Rectangle, &["12", "10"], 120.0),
            // 12 × 11 pi, moins l'encoche de 5 × 5 pi.
            (FormeDeDepart::EnL, &["12", "6", "5", "5"], 107.0),
            // La barre de 16 × 5 pi et le pied de 6 × 7 pi.
            (FormeDeDepart::EnT, &["16", "5", "5", "7", "6", "7"], 122.0),
            // 12 × 11 pi, moins le coin de 3 pi coupé à 45°.
            (FormeDeDepart::CoinA45, &["9", "11", "12"], 127.5),
        ];
        for (forme, mesures, attendue) in cas {
            let (nombre, aire) = mesurer(forme, mesures);
            assert_eq!(nombre, forme.mesures_a_inscrire(), "{forme:?}");
            assert!((aire - attendue).abs() < 1e-6, "{forme:?} : {aire} pi²");
        }
    }

    #[test]
    fn une_forme_garde_le_genre_de_la_piece() {
        let mut terrain = Dessin {
            terrain: true,
            hauteur_des_murs: "8".to_string(),
            ..Dessin::default()
        };
        terrain.poser_une_forme(FormeDeDepart::Rectangle);
        assert!(terrain.terrain && terrain.fermee);
        assert_eq!(terrain.hauteur_des_murs, "8");
        assert_eq!(terrain.cotes.len(), 4);
        assert!(terrain.cotes.iter().all(|cote| cote.saisie.is_empty()));
    }
}
