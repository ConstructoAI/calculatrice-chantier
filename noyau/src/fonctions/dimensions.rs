//! Longueur, largeur, hauteur (section 12.11).

use super::Sortie;
use crate::erreurs::Erreur;
use crate::etiquettes::Etiquette;
use crate::prefs::Preferences;
use crate::unites::Genre;
use crate::valeur::{Valeur, format_aire, format_volume};

/// Registres des dimensions saisies.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RegistresDimensions {
    pub longueur: Option<Valeur>,
    pub largeur: Option<Valeur>,
    pub hauteur: Option<Valeur>,
}

fn memorisee(etiquette: Etiquette, v: Valeur) -> Sortie {
    Sortie::nouvelle(etiquette, v).memorisee()
}

fn meme_genre(valeurs: &[&Valeur]) -> Result<Genre, Erreur> {
    let genre = valeurs.first().map(|v| v.genre).unwrap_or(Genre::Longueur);
    if valeurs.iter().any(|v| v.genre != genre) {
        return Err(Erreur::Unites);
    }
    Ok(genre)
}

fn fini(sorties: Vec<Sortie>) -> Result<Vec<Sortie>, Erreur> {
    if sorties.iter().any(|s| !s.valeur.base.is_finite()) {
        Err(Erreur::Depassement)
    } else {
        Ok(sorties)
    }
}

impl RegistresDimensions {
    /// Touche Longueur sans donnée : réaffiche la longueur.
    pub fn cycle_longueur(&self) -> Result<Vec<Sortie>, Erreur> {
        let l = self.longueur.ok_or(Erreur::AucuneValeur)?;
        Ok(vec![memorisee(Etiquette::Longueur, l)])
    }

    /// Touche Largeur : aire, équerrage, périmètre, longueur, largeur.
    pub fn cycle_largeur(&self, prefs: &Preferences) -> Result<Vec<Sortie>, Erreur> {
        let largeur = self.largeur.ok_or(Erreur::AucuneValeur)?;
        let Some(longueur) = self.longueur else {
            return Ok(vec![memorisee(Etiquette::Largeur, largeur)]);
        };
        let genre = meme_genre(&[&longueur, &largeur])?;
        let (l, w) = (longueur.base, largeur.base);
        let aire = match genre {
            Genre::Longueur => Valeur::en_base(
                l * w,
                format_aire(&[longueur.format, largeur.format], prefs),
            ),
            _ => Valeur::nombre(l * w),
        };
        let lineaire = |base: f64| Valeur { base, ..longueur };
        fini(vec![
            Sortie::nouvelle(Etiquette::Aire, aire),
            Sortie::nouvelle(Etiquette::Equerrage, lineaire(l.hypot(w))),
            Sortie::nouvelle(Etiquette::Perimetre, lineaire(2.0 * (l + w))),
            memorisee(Etiquette::Longueur, longueur),
            memorisee(Etiquette::Largeur, largeur),
        ])
    }

    /// Touche Hauteur : volume, murs, pièce, longueur, largeur, hauteur.
    pub fn cycle_hauteur(&self, prefs: &Preferences) -> Result<Vec<Sortie>, Erreur> {
        let hauteur = self.hauteur.ok_or(Erreur::AucuneValeur)?;
        let (Some(longueur), Some(largeur)) = (self.longueur, self.largeur) else {
            return Ok(vec![memorisee(Etiquette::Hauteur, hauteur)]);
        };
        let genre = meme_genre(&[&longueur, &largeur, &hauteur])?;
        let (l, w, h) = (longueur.base, largeur.base, hauteur.base);
        let formats = [longueur.format, largeur.format, hauteur.format];
        let (volume, murs, piece) = (l * w * h, 2.0 * (l + w) * h, 2.0 * (l + w) * h + l * w);
        let (v_volume, v_murs, v_piece) = match genre {
            Genre::Longueur => {
                let fa = format_aire(&formats, prefs);
                (
                    Valeur::en_base(volume, format_volume(&formats, prefs)),
                    Valeur::en_base(murs, fa),
                    Valeur::en_base(piece, fa),
                )
            }
            _ => (
                Valeur::nombre(volume),
                Valeur::nombre(murs),
                Valeur::nombre(piece),
            ),
        };
        fini(vec![
            Sortie::nouvelle(Etiquette::Volume, v_volume),
            Sortie::nouvelle(Etiquette::Murs, v_murs),
            Sortie::nouvelle(Etiquette::Piece, v_piece),
            memorisee(Etiquette::Longueur, longueur),
            memorisee(Etiquette::Largeur, largeur),
            memorisee(Etiquette::Hauteur, hauteur),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unites::Format;

    fn pi_po(po: f64) -> Option<Valeur> {
        Some(Valeur::en_base(po, Format::PiPo))
    }

    #[test]
    fn rectangle() {
        let r = RegistresDimensions {
            longueur: pi_po(246.0),
            largeur: pi_po(306.0),
            hauteur: None,
        };
        let s = r.cycle_largeur(&Preferences::default()).unwrap();
        assert!((s[0].valeur.numerique() - 522.75).abs() < 1e-9);
        assert!((s[1].valeur.base - 392.622).abs() < 1e-3);
        assert_eq!(s[2].valeur.base, 1104.0);
        assert!(s[3].memorise && s[4].memorise);
    }

    #[test]
    fn piece() {
        let r = RegistresDimensions {
            longueur: pi_po(180.0),
            largeur: pi_po(240.0),
            hauteur: pi_po(144.0),
        };
        let s = r.cycle_hauteur(&Preferences::default()).unwrap();
        assert!((s[0].valeur.numerique() - 3600.0).abs() < 1e-9);
        assert_eq!(s[0].valeur.format, Format::Pi3);
        assert!((s[1].valeur.numerique() - 840.0).abs() < 1e-9);
        assert!((s[2].valeur.numerique() - 1140.0).abs() < 1e-9);
    }

    #[test]
    fn dimensions_manquantes() {
        let r = RegistresDimensions {
            largeur: pi_po(120.0),
            ..RegistresDimensions::default()
        };
        let s = r.cycle_largeur(&Preferences::default()).unwrap();
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].etiquette, Etiquette::Largeur);
        assert_eq!(r.cycle_longueur().map(|_| ()), Err(Erreur::AucuneValeur));
        assert_eq!(
            r.cycle_hauteur(&Preferences::default()).map(|_| ()),
            Err(Erreur::AucuneValeur)
        );
    }
}
