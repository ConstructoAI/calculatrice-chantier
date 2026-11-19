//! Polygone régulier (section 12.10).

use std::f64::consts::PI;

use super::Sortie;
use crate::erreurs::Erreur;
use crate::etiquettes::Etiquette;
use crate::prefs::Preferences;
use crate::unites::{EPS, Genre};
use crate::valeur::{Valeur, format_aire};

/// Nombre de côtés le plus élevé accepté.
const MAX_COTES: f64 = 1_000_000.0;

/// Interprète la donnée comme un nombre de côtés : entier ≥ 3.
pub fn interpreter_cotes(donnee: &Valeur) -> Result<u32, Erreur> {
    if !donnee.est_sans_unite() {
        return Err(Erreur::Unites);
    }
    let n = donnee.base.round();
    if !donnee.base.is_finite() || (donnee.base - n).abs() > EPS || !(3.0..=MAX_COTES).contains(&n)
    {
        return Err(Erreur::Saisie);
    }
    Ok(n as u32)
}

/// Calcule un polygone régulier de `n` côtés inscrit dans un cercle de rayon donné.
pub fn calculer_polygone(
    n: u32,
    rayon: &Valeur,
    prefs: &Preferences,
) -> Result<Vec<Sortie>, Erreur> {
    if n < 3 {
        return Err(Erreur::Saisie);
    }
    let r = rayon.base;
    if !(r.is_finite() && r > 0.0) {
        return Err(Erreur::Saisie);
    }
    let cotes = f64::from(n);
    let angle = (cotes - 2.0) * 180.0 / cotes;
    let cote = 2.0 * r * (PI / cotes).sin();
    let surface = cotes * r * r * (2.0 * PI / cotes).sin() / 2.0;
    let longueur = |base: f64| Valeur { base, ..*rayon };
    let aire = match rayon.genre {
        Genre::Longueur => Valeur::en_base(surface, format_aire(&[rayon.format], prefs)),
        Genre::SansUnite => Valeur::nombre(surface),
        _ => return Err(Erreur::Unites),
    };
    Ok(vec![
        Sortie::angle(Etiquette::AngleInterieur, angle),
        Sortie::angle(Etiquette::DemiAngle, angle / 2.0),
        Sortie::nouvelle(Etiquette::Cote, longueur(cote)),
        Sortie::nouvelle(Etiquette::Perimetre, longueur(cotes * cote)),
        Sortie::nouvelle(Etiquette::Aire, aire),
        Sortie::nouvelle(Etiquette::Rayon, *rayon),
        Sortie::entier(Etiquette::NbCotes, cotes),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unites::Format;

    #[test]
    fn hexagone_de_15_pieds_de_rayon() {
        let p = Preferences::default();
        let s = calculer_polygone(6, &Valeur::en_base(180.0, Format::PiPo), &p).unwrap();
        assert_eq!(s[0].valeur.base, 120.0);
        assert_eq!(s[1].valeur.base, 60.0);
        assert!((s[2].valeur.base - 180.0).abs() < 1e-9);
        assert!((s[3].valeur.base - 1080.0).abs() < 1e-9);
        assert!((s[4].valeur.numerique() - 584.5671).abs() < 1e-4);
        assert_eq!(s[6].valeur.base, 6.0);
    }

    #[test]
    fn nombre_de_cotes_invalide() {
        assert_eq!(interpreter_cotes(&Valeur::nombre(2.0)), Err(Erreur::Saisie));
        assert_eq!(interpreter_cotes(&Valeur::nombre(5.5)), Err(Erreur::Saisie));
        assert_eq!(
            interpreter_cotes(&Valeur::en_base(60.0, Format::PiPo)),
            Err(Erreur::Unites)
        );
        assert_eq!(interpreter_cotes(&Valeur::nombre(8.0)), Ok(8));
    }
}
