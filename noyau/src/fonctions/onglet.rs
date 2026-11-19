//! Onglet composé pour moulures (section 12.7).

use super::Sortie;
use crate::erreurs::Erreur;
use crate::etiquettes::Etiquette;
use crate::unites::EPS;
use crate::valeur::Valeur;

/// En deçà de ce seuil, la donnée est un nombre de côtés (annexe A.2).
const SEUIL_COTES: f64 = 25.0;

/// Coin d'un ouvrage : un angle, ou un nombre de côtés d'un polygone régulier.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Coin {
    Angle(f64),
    Cotes(u32),
}

impl Coin {
    /// Angle du coin, en degrés.
    pub fn angle(self) -> f64 {
        match self {
            Coin::Angle(a) => a,
            Coin::Cotes(n) => (f64::from(n) - 2.0) * 180.0 / f64::from(n),
        }
    }
}

/// Interprète la donnée tapée avant la touche.
pub fn interpreter_coin(donnee: &Valeur) -> Result<Coin, Erreur> {
    if !donnee.est_sans_unite() {
        return Err(Erreur::Unites);
    }
    let x = donnee.base;
    if !x.is_finite() || x > 180.0 + EPS {
        return Err(Erreur::Saisie);
    }
    if x >= SEUIL_COTES {
        return Ok(Coin::Angle(x.min(180.0)));
    }
    let n = x.round();
    if (x - n).abs() > EPS || n < 3.0 {
        return Err(Erreur::Saisie);
    }
    Ok(Coin::Cotes(n as u32))
}

/// Contrôle un angle de moulure : strictement entre 0 et 90 degrés.
pub fn controler_angle_moulure(donnee: &Valeur) -> Result<f64, Erreur> {
    if !donnee.est_sans_unite() {
        return Err(Erreur::Unites);
    }
    if donnee.base.is_finite() && donnee.base > 0.0 && donnee.base < 90.0 {
        Ok(donnee.base)
    } else {
        Err(Erreur::Saisie)
    }
}

/// Calcule l'onglet et le biseau d'une moulure coupée à plat.
pub fn calculer_onglet(coin: Coin, angle_moulure: f64) -> Result<Vec<Sortie>, Erreur> {
    let c = coin.angle();
    if !(c > 0.0 && c <= 180.0 + EPS && angle_moulure > 0.0 && angle_moulure < 90.0) {
        return Err(Erreur::Saisie);
    }
    let s = angle_moulure.to_radians();
    let demi = (c / 2.0).to_radians();
    let onglet = (s.sin() / demi.tan()).atan().to_degrees();
    let biseau = (s.cos() * demi.cos()).clamp(-1.0, 1.0).asin().to_degrees();
    if !(onglet.is_finite() && biseau.is_finite()) {
        return Err(Erreur::Calcul);
    }
    let s_onglet = Sortie::angle(Etiquette::Onglet, onglet);
    let s_biseau = Sortie::angle(Etiquette::Biseau, biseau);
    let s_moulure = Sortie::angle(Etiquette::AngleMoulure, angle_moulure).memorisee();
    let s_coin = Sortie::angle(Etiquette::Coin, c);
    Ok(match coin {
        Coin::Angle(_) => vec![s_onglet, s_biseau, s_moulure, s_coin],
        Coin::Cotes(_) => vec![s_coin, s_onglet, s_biseau, s_moulure],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn proche(a: f64, b: f64) -> bool {
        (a - b).abs() < 5e-3
    }

    #[test]
    fn coin_de_60_degres_moulure_de_38() {
        let s = calculer_onglet(Coin::Angle(60.0), 38.0).unwrap();
        assert!(proche(s[0].valeur.base, 46.84));
        assert!(proche(s[1].valeur.base, 43.03));
        assert!(s[2].memorise);
        assert_eq!(s[3].valeur.base, 60.0);
    }

    #[test]
    fn coin_d_equerre() {
        let s = calculer_onglet(Coin::Angle(90.0), 45.0).unwrap();
        assert!(proche(s[0].valeur.base, 35.26));
        assert!(proche(s[1].valeur.base, 30.00));
        let s = calculer_onglet(Coin::Angle(90.0), 38.0).unwrap();
        assert!(proche(s[0].valeur.base, 31.62));
        assert!(proche(s[1].valeur.base, 33.86));
    }

    #[test]
    fn nombre_de_cotes() {
        assert_eq!(interpreter_coin(&Valeur::nombre(8.0)), Ok(Coin::Cotes(8)));
        let s = calculer_onglet(Coin::Cotes(8), 45.0).unwrap();
        assert_eq!(s[0].etiquette, Etiquette::Coin);
        assert_eq!(s[0].valeur.base, 135.0);
        assert!(proche(s[1].valeur.base, 16.32));
        assert!(proche(s[2].valeur.base, 15.70));
    }

    #[test]
    fn donnees_refusees() {
        assert_eq!(interpreter_coin(&Valeur::nombre(2.0)), Err(Erreur::Saisie));
        assert_eq!(interpreter_coin(&Valeur::nombre(7.5)), Err(Erreur::Saisie));
        assert_eq!(
            interpreter_coin(&Valeur::nombre(200.0)),
            Err(Erreur::Saisie)
        );
        assert_eq!(
            controler_angle_moulure(&Valeur::nombre(90.0)),
            Err(Erreur::Saisie)
        );
        assert_eq!(controler_angle_moulure(&Valeur::nombre(38.0)), Ok(38.0));
    }
}
