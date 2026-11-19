//! Mémoire cumulative M+ (section 11.1).

use serde::{Deserialize, Serialize};

use crate::erreurs::Erreur;
use crate::unites::{Format, Genre};
use crate::valeur::Valeur;

/// Mémoire cumulative : somme, nombre de valeurs, genre et format.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MemoireCumulative {
    pub somme: f64,
    pub nombre: u32,
    pub genre: Genre,
    pub format: Format,
}

impl MemoireCumulative {
    pub fn est_vide(&self) -> bool {
        self.nombre == 0
    }

    pub fn vider(&mut self) {
        *self = MemoireCumulative::default();
    }

    /// Ajoute (ou soustrait) une valeur. La première fixe le genre et le format.
    pub fn ajouter(&mut self, v: &Valeur, soustraire: bool) -> Result<(), Erreur> {
        if self.est_vide() {
            self.genre = v.genre;
            self.format = v.format;
            self.somme = 0.0;
        } else if self.genre != v.genre {
            return Err(Erreur::Unites);
        }
        let somme = if soustraire {
            self.somme - v.base
        } else {
            self.somme + v.base
        };
        if !somme.is_finite() {
            return Err(Erreur::Depassement);
        }
        self.somme = somme;
        self.nombre = self.nombre.saturating_add(1);
        Ok(())
    }

    pub fn total(&self) -> Option<Valeur> {
        (!self.est_vide()).then_some(Valeur {
            base: self.somme,
            genre: self.genre,
            format: self.format,
        })
    }

    pub fn moyenne(&self) -> Option<Valeur> {
        self.total().map(|t| Valeur {
            base: t.base / f64::from(self.nombre),
            ..t
        })
    }

    /// Remet d'accord le genre et le format après relecture d'un fichier.
    pub fn assainir(&mut self) {
        if !self.somme.is_finite() || self.format.genre() != self.genre {
            self.vider();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn somme_moyenne_et_nombre() {
        let mut m = MemoireCumulative::default();
        m.ajouter(&Valeur::nombre(100.0), false).unwrap();
        m.ajouter(&Valeur::nombre(200.0), false).unwrap();
        m.ajouter(&Valeur::nombre(50.0), true).unwrap();
        assert_eq!(m.total().unwrap().base, 250.0);
        assert_eq!(m.nombre, 3);
        assert!((m.moyenne().unwrap().base - 250.0 / 3.0).abs() < 1e-12);
    }

    #[test]
    fn genres_differents_refuses() {
        let mut m = MemoireCumulative::default();
        m.ajouter(&Valeur::en_base(12.0, Format::PiPo), false)
            .unwrap();
        assert_eq!(m.ajouter(&Valeur::nombre(3.0), false), Err(Erreur::Unites));
        assert_eq!(m.nombre, 1);
    }
}
