//! Carnet de nivellement (section 22.21) : le repère, les lectures au niveau
//! laser point par point, les élévations, et ce qu'il faut creuser ou
//! remblayer à chaque point par rapport à l'élévation visée. Toutes les
//! longueurs sont en pouces.

/// Ce qu'un point du carnet donne : son élévation, et son écart à
/// l'élévation visée s'il y en a une — positif, il faut creuser; négatif,
/// remblayer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointNivele {
    pub elevation: f64,
    pub ecart: Option<f64>,
}

/// La hauteur de l'instrument : l'élévation du repère plus la lecture faite
/// sur lui (la lecture arrière).
pub fn hauteur_de_l_instrument(repere: f64, lecture_sur_le_repere: f64) -> f64 {
    repere + lecture_sur_le_repere
}

/// Un point : son élévation est la hauteur de l'instrument moins la lecture
/// faite sur lui; son écart, cette élévation moins l'élévation visée.
pub fn niveler(hauteur: f64, lecture: f64, visee: Option<f64>) -> PointNivele {
    let elevation = hauteur - lecture;
    PointNivele {
        elevation,
        ecart: visee.map(|visee| elevation - visee),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn un_carnet_de_nivellement() {
        // Repère à 100 pi, lu à 5 pi : l'instrument est à 105 pi.
        let hauteur = hauteur_de_l_instrument(1200.0, 60.0);
        assert_eq!(hauteur, 1260.0);
        // Lu à 6 pi 3 po : 98 pi 9 po, 3 po sous les 99 pi visés, à remblayer.
        let bas = niveler(hauteur, 75.0, Some(1188.0));
        assert_eq!(bas.elevation, 1185.0);
        assert_eq!(bas.ecart, Some(-3.0));
        // Lu à 4 pi : 101 pi, 2 pi au-dessus, à creuser.
        let haut = niveler(hauteur, 48.0, Some(1188.0));
        assert_eq!(haut.ecart, Some(24.0));
        // Sans élévation visée : l'élévation seule.
        assert_eq!(niveler(hauteur, 60.0, None).ecart, None);
    }
}
