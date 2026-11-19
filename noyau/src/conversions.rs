//! Conversions d'unités (section 10).

use crate::erreurs::Erreur;
use crate::unites::{Format, Genre, MasseVolumique, PO3_PAR_PMP};
use crate::valeur::Valeur;

/// Unité demandée par une touche (principale ou secondaire).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UniteTouche {
    Verges,
    Pieds,
    Pouces,
    Metres,
    Cm,
    Mm,
    Lb,
    Kg,
    Tc,
    T,
    Acre,
    Pmp,
}

/// Effet d'une touche d'unité sur une valeur montrée.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Conversion {
    /// La valeur dans son nouveau format (ou son nouveau genre).
    Valeur(Valeur),
    /// Nouvelle résolution temporaire des fractions (R-24).
    Resolution(u32),
}

/// Applique une touche d'unité à une valeur validée (sections 10.1 à 10.3).
pub fn convertir(
    v: &Valeur,
    unite: UniteTouche,
    masse_volumique: &MasseVolumique,
) -> Result<Conversion, Erreur> {
    use UniteTouche::*;
    let valeur = match v.genre {
        Genre::SansUnite => {
            // Un nombre nu reçoit l'unité (R-39); un angle ou un montant, non.
            if v.format != Format::Nombre {
                return Err(Erreur::Unites);
            }
            match unite {
                Verges => Valeur::avec_format(v.base, Format::Vg),
                Pieds => Valeur::avec_format(v.base, Format::PiPo),
                Pouces => Valeur::avec_format(v.base, Format::PoFrac),
                Metres => Valeur::avec_format(v.base, Format::M),
                Cm => Valeur::avec_format(v.base, Format::Cm),
                Mm => Valeur::avec_format(v.base, Format::Mm),
                Lb => Valeur::avec_format(v.base, Format::Lb),
                Kg => Valeur::avec_format(v.base, Format::Kg),
                Tc => Valeur::avec_format(v.base, Format::Tc),
                T => Valeur::avec_format(v.base, Format::T),
                Acre => Valeur::avec_format(v.base, Format::Acre),
                // Produit « pouces × pouces × pieds » d'une pièce de bois (R-20).
                Pmp => Valeur::en_base(v.base / 12.0 * PO3_PAR_PMP, Format::Pmp),
            }
        }
        Genre::Longueur => {
            let format = match unite {
                Verges => Format::Vg,
                Pieds => match v.format {
                    Format::PiPo | Format::PoDec => Format::PiDec,
                    _ => Format::PiPo,
                },
                Pouces => match v.format {
                    Format::PoFrac | Format::PiDec => Format::PoDec,
                    _ => Format::PoFrac,
                },
                Metres => Format::M,
                Cm => Format::Cm,
                Mm => Format::Mm,
                Lb | Kg | Tc | T | Acre | Pmp => {
                    // Sur une longueur à fraction : résolution temporaire (R-24).
                    if !v.format.est_fractionnaire() {
                        return Err(Erreur::Unites);
                    }
                    let resolution = match unite {
                        Kg => 16,
                        Acre => 2,
                        T => 32,
                        Lb => 4,
                        Tc => 64,
                        _ => 8,
                    };
                    return Ok(Conversion::Resolution(resolution));
                }
            };
            v.convertie(format)
        }
        Genre::Aire => {
            let format = match unite {
                Verges => Format::Vg2,
                Pieds => Format::Pi2,
                Pouces => Format::Po2,
                Metres => Format::M2,
                Cm => Format::Cm2,
                Mm => Format::Mm2,
                Acre => Format::Acre,
                _ => return Err(Erreur::Unites),
            };
            v.convertie(format)
        }
        Genre::Volume => match unite {
            Verges => v.convertie(Format::Vg3),
            Pieds => v.convertie(Format::Pi3),
            Pouces => v.convertie(Format::Po3),
            Metres => v.convertie(Format::M3),
            Cm => v.convertie(Format::Cm3),
            Mm => v.convertie(Format::Mm3),
            Pmp => v.convertie(Format::Pmp),
            Lb | Kg | Tc | T => {
                let format = format_masse(unite);
                Valeur::en_base(v.base * masse_volumique.lb_par_po3(), format)
            }
            Acre => return Err(Erreur::Unites),
        },
        Genre::Masse => match unite {
            Lb | Kg | Tc | T => v.convertie(format_masse(unite)),
            Verges | Pieds | Pouces | Metres | Cm | Mm => {
                let densite = masse_volumique.lb_par_po3();
                if densite <= 0.0 {
                    return Err(Erreur::Calcul);
                }
                let format = match unite {
                    Verges => Format::Vg3,
                    Pieds => Format::Pi3,
                    Pouces => Format::Po3,
                    Metres => Format::M3,
                    Cm => Format::Cm3,
                    _ => Format::Mm3,
                };
                Valeur::en_base(v.base / densite, format)
            }
            Acre | Pmp => return Err(Erreur::Unites),
        },
    };
    if valeur.base.is_finite() {
        Ok(Conversion::Valeur(valeur))
    } else {
        Err(Erreur::Depassement)
    }
}

fn format_masse(unite: UniteTouche) -> Format {
    match unite {
        UniteTouche::Kg => Format::Kg,
        UniteTouche::Tc => Format::Tc,
        UniteTouche::T => Format::T,
        _ => Format::Lb,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn conv(v: Valeur, u: UniteTouche) -> Valeur {
        match convertir(&v, u, &MasseVolumique::default()).unwrap() {
            Conversion::Valeur(v) => v,
            Conversion::Resolution(_) => panic!("résolution inattendue"),
        }
    }

    #[test]
    fn alternances_pieds_et_pouces() {
        let v = Valeur::en_base(189.5, Format::PiPo);
        let dec = conv(v, UniteTouche::Pieds);
        assert_eq!(dec.format, Format::PiDec);
        assert_eq!(conv(dec, UniteTouche::Pieds).format, Format::PiPo);
        assert_eq!(conv(dec, UniteTouche::Pouces).format, Format::PoDec);
        let po = Valeur::en_base(8.125, Format::PoFrac);
        assert_eq!(conv(po, UniteTouche::Pouces).format, Format::PoDec);
        assert_eq!(
            conv(Valeur::en_base(8.125, Format::PoDec), UniteTouche::Pieds).format,
            Format::PiDec
        );
        assert_eq!(conv(po, UniteTouche::Pieds).format, Format::PiPo);
    }

    #[test]
    fn la_base_ne_change_pas() {
        let v = Valeur::en_base(168.0, Format::PiPo);
        for u in [
            UniteTouche::Verges,
            UniteTouche::Metres,
            UniteTouche::Cm,
            UniteTouche::Mm,
            UniteTouche::Pouces,
        ] {
            assert_eq!(conv(v, u).base, 168.0);
        }
    }

    #[test]
    fn volume_et_masse() {
        let volume = Valeur::avec_format(5.0, Format::Vg3);
        let masse = conv(volume, UniteTouche::Lb);
        assert!((masse.base - 15_000.0).abs() < 1e-6);
        let retour = conv(Valeur::avec_format(3.0, Format::Tc), UniteTouche::Verges);
        assert!((retour.numerique() - 2.0).abs() < 1e-9);
        assert_eq!(retour.format, Format::Vg3);
    }

    #[test]
    fn resolution_temporaire_et_refus() {
        let l = Valeur::en_base(45.0 / 64.0, Format::PoFrac);
        let mv = MasseVolumique::default();
        assert_eq!(
            convertir(&l, UniteTouche::T, &mv),
            Ok(Conversion::Resolution(32))
        );
        assert_eq!(
            convertir(&l.convertie(Format::M), UniteTouche::Kg, &mv),
            Err(Erreur::Unites)
        );
        let aire = Valeur::avec_format(10.0, Format::Pi2);
        assert_eq!(convertir(&aire, UniteTouche::Pmp, &mv), Err(Erreur::Unites));
        let nombre = conv(Valeur::nombre(112.0), UniteTouche::Pmp);
        assert!((nombre.numerique() - 112.0 / 12.0).abs() < 1e-12);
    }
}
