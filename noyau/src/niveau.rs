//! Niveau et inclinomètre (section 22.22) : la pente d'une surface, d'après
//! l'inclinaison du téléphone posé dessus, l'écran vers le haut. Le capteur
//! donne deux angles : `beta`, l'avant et l'arrière (autour de l'axe des x),
//! et `gamma`, la gauche et la droite (autour de l'axe des y), en degrés.

/// Au-delà de cette inclinaison, une pente sur 12 ou en pour cent n'a plus
/// de sens : le téléphone est debout.
pub const PRESQUE_DEBOUT: f64 = 89.5;

/// L'inclinaison de l'écran par rapport à l'horizontale, de 0 à 90 degrés,
/// moins les angles de l'étalonnage : quelle que soit l'orientation du
/// téléphone sur la surface, c'est la pente de celle-ci.
pub fn inclinaison(beta: f64, gamma: f64, etalon: [f64; 2]) -> f64 {
    let (b, g) = (
        (beta - etalon[0]).to_radians(),
        (gamma - etalon[1]).to_radians(),
    );
    (b.cos() * g.cos())
        .abs()
        .clamp(0.0, 1.0)
        .acos()
        .to_degrees()
}

/// La pente sur 12 — la montée pour 12 de course — d'un angle en degrés.
pub fn sur_douze(degres: f64) -> f64 {
    12.0 * degres.to_radians().tan()
}

/// La pente en pour cent d'un angle en degrés.
pub fn en_pourcentage(degres: f64) -> f64 {
    100.0 * degres.to_radians().tan()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn proche(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    #[test]
    fn la_pente_d_une_surface() {
        // À plat : 0.
        assert!(proche(inclinaison(0.0, 0.0, [0.0; 2]), 0.0));
        // Une pente de 6/12 : 26,565°, dans un sens ou dans l'autre.
        let six_sur_douze = 0.5f64.atan().to_degrees();
        assert!(proche(
            sur_douze(inclinaison(six_sur_douze, 0.0, [0.0; 2])),
            6.0
        ));
        assert!(proche(
            sur_douze(inclinaison(0.0, -six_sur_douze, [0.0; 2])),
            6.0
        ));
        assert!(proche(en_pourcentage(45.0), 100.0));
        // Le téléphone de travers sur la surface : la même pente.
        let travers = inclinaison(30.0, 40.0, [0.0; 2]);
        assert!(proche(
            travers,
            (30f64.to_radians().cos() * 40f64.to_radians().cos())
                .acos()
                .to_degrees()
        ));
        // L'étalonnage retranche les angles de la surface de référence.
        assert!(proche(inclinaison(1.5, -2.0, [1.5, -2.0]), 0.0));
        // Écran vers le bas : la pente reste entre 0 et 90.
        assert!(inclinaison(170.0, 0.0, [0.0; 2]) <= 90.0);
    }
}
