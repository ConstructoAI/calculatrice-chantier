//! Mode trigonométrique (section 12.18).

use crate::erreurs::Erreur;
use crate::valeur::Valeur;

/// Fonction trigonométrique, en degrés.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FonctionTrigo {
    Sin,
    Cos,
    Tan,
    ArcSin,
    ArcCos,
    ArcTan,
}

/// Applique une fonction trigonométrique à une valeur sans unité.
pub fn calculer_trigo(fonction: FonctionTrigo, v: &Valeur) -> Result<Valeur, Erreur> {
    if !v.est_sans_unite() {
        return Err(Erreur::Trigonometrie);
    }
    let x = v.base;
    let resultat = match fonction {
        FonctionTrigo::Sin => Valeur::nombre(x.to_radians().sin()),
        FonctionTrigo::Cos => Valeur::nombre(x.to_radians().cos()),
        FonctionTrigo::Tan => {
            let radians = x.to_radians();
            if radians.cos().abs() < 1e-12 {
                return Err(Erreur::Calcul);
            }
            Valeur::nombre(radians.tan())
        }
        FonctionTrigo::ArcSin | FonctionTrigo::ArcCos => {
            if !(-1.0..=1.0).contains(&x) {
                return Err(Erreur::Trigonometrie);
            }
            let radians = if fonction == FonctionTrigo::ArcSin {
                x.asin()
            } else {
                x.acos()
            };
            Valeur::degres(radians.to_degrees())
        }
        FonctionTrigo::ArcTan => Valeur::degres(x.atan().to_degrees()),
    };
    if !resultat.base.is_finite() {
        return Err(Erreur::Trigonometrie);
    }
    // Les valeurs remarquables tombent juste (sin 30° = 0,5 et non 0,49999…).
    let arrondi = (resultat.base * 1e12).round() / 1e12;
    Ok(Valeur {
        base: arrondi,
        ..resultat
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unites::Format;

    #[test]
    fn valeurs_remarquables() {
        let f = |fonction, x| calculer_trigo(fonction, &Valeur::nombre(x)).unwrap().base;
        assert_eq!(f(FonctionTrigo::Sin, 30.0), 0.5);
        assert_eq!(f(FonctionTrigo::Cos, 60.0), 0.5);
        assert_eq!(f(FonctionTrigo::Tan, 45.0), 1.0);
        assert_eq!(f(FonctionTrigo::ArcSin, 0.5), 30.0);
        assert!((f(FonctionTrigo::ArcCos, 0.6) - 53.130_102).abs() < 1e-6);
    }

    #[test]
    fn erreurs() {
        let longueur = Valeur::en_base(12.0, Format::PiPo);
        assert_eq!(
            calculer_trigo(FonctionTrigo::Sin, &longueur),
            Err(Erreur::Trigonometrie)
        );
        assert_eq!(
            calculer_trigo(FonctionTrigo::ArcSin, &Valeur::nombre(1.5)),
            Err(Erreur::Trigonometrie)
        );
        assert_eq!(
            calculer_trigo(FonctionTrigo::Tan, &Valeur::nombre(90.0)),
            Err(Erreur::Calcul)
        );
    }
}
