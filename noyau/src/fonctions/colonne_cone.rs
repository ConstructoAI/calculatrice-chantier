//! Colonne et cône (section 12.9).

use std::f64::consts::PI;

use super::Sortie;
use crate::erreurs::Erreur;
use crate::etiquettes::Etiquette;
use crate::prefs::Preferences;
use crate::unites::Genre;
use crate::valeur::{Valeur, format_aire, format_volume};

/// Volume et surface totale d'une colonne, puis d'un cône.
pub fn calculer_colonne_cone(
    diametre: &Valeur,
    hauteur: &Valeur,
    prefs: &Preferences,
) -> Result<Vec<Sortie>, Erreur> {
    if diametre.genre != hauteur.genre {
        return Err(Erreur::Unites);
    }
    let r = diametre.base / 2.0;
    let h = hauteur.base;
    if !(r.is_finite() && r > 0.0 && h.is_finite() && h > 0.0) {
        return Err(Erreur::Saisie);
    }
    let formats = [diametre.format, hauteur.format];
    // Formats du volume et de l'aire; `None` pour des nombres nus.
    let unites = match diametre.genre {
        Genre::Longueur => Some((format_volume(&formats, prefs), format_aire(&formats, prefs))),
        Genre::SansUnite => None,
        _ => return Err(Erreur::Unites),
    };
    let volume = |base: f64| match unites {
        Some((format, _)) => Valeur::en_base(base, format),
        None => Valeur::nombre(base),
    };
    let aire = |base: f64| match unites {
        Some((_, format)) => Valeur::en_base(base, format),
        None => Valeur::nombre(base),
    };
    let sorties = vec![
        Sortie::nouvelle(Etiquette::ColonneVolume, volume(PI * r * r * h)),
        Sortie::nouvelle(Etiquette::ColonneSurface, aire(2.0 * PI * r * (r + h))),
        Sortie::nouvelle(Etiquette::ConeVolume, volume(PI * r * r * h / 3.0)),
        Sortie::nouvelle(Etiquette::ConeSurface, aire(PI * r * (r + r.hypot(h)))),
    ];
    if sorties.iter().any(|s| !s.valeur.base.is_finite()) {
        return Err(Erreur::Depassement);
    }
    Ok(sorties)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unites::Format;

    #[test]
    fn colonne_et_cone() {
        let p = Preferences::default();
        let d = Valeur::en_base(28.0, Format::PiPo);
        let h = Valeur::en_base(54.0, Format::PiPo);
        let s = calculer_colonne_cone(&d, &h, &p).unwrap();
        assert!((s[0].valeur.numerique() - 19.242_26).abs() < 1e-5);
        assert_eq!(s[0].valeur.format, Format::Pi3);
        assert!((s[1].valeur.numerique() - 41.538_84).abs() < 1e-5);
        let d = Valeur::en_base(42.0, Format::PiPo);
        let h = Valeur::en_base(60.0, Format::PiPo);
        let s = calculer_colonne_cone(&d, &h, &p).unwrap();
        assert!((s[2].valeur.numerique() - 16.035_21).abs() < 1e-5);
        assert!((s[3].valeur.numerique() - 38.745_13).abs() < 1e-5);
    }

    #[test]
    fn genres_incompatibles() {
        let p = Preferences::default();
        let d = Valeur::en_base(28.0, Format::PiPo);
        assert_eq!(
            calculer_colonne_cone(&d, &Valeur::nombre(5.0), &p).map(|_| ()),
            Err(Erreur::Unites)
        );
    }
}
