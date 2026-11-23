//! Valeur et arithmétique dimensionnelle (sections 6.1 et 9.5).

use serde::{Deserialize, Serialize};

use crate::erreurs::Erreur;
use crate::prefs::Preferences;
use crate::unites::{EPS, Famille, Format, Genre, Systeme};

/// Quantité : un nombre en unité de base, un genre et un format d'affichage.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Valeur {
    pub base: f64,
    pub genre: Genre,
    pub format: Format,
}

impl Valeur {
    /// Nombre sans unité.
    pub fn nombre(x: f64) -> Valeur {
        Valeur {
            base: x,
            genre: Genre::SansUnite,
            format: Format::Nombre,
        }
    }

    /// Angle en degrés décimaux.
    pub fn degres(x: f64) -> Valeur {
        Valeur {
            base: x,
            genre: Genre::SansUnite,
            format: Format::Degres,
        }
    }

    /// Valeur dont `base` est déjà en unité de base; le genre découle du format.
    pub fn en_base(base: f64, format: Format) -> Valeur {
        Valeur {
            base,
            genre: format.genre(),
            format,
        }
    }

    /// Valeur dont le nombre est exprimé dans l'unité du format (5 → 5 vg).
    pub fn avec_format(numerique: f64, format: Format) -> Valeur {
        Valeur::en_base(numerique * format.facteur(), format)
    }

    /// Nombre exprimé dans l'unité d'affichage.
    pub fn numerique(&self) -> f64 {
        self.base / self.format.facteur()
    }

    /// La même valeur dans un autre format du même genre (R-11 : `base` ne change pas).
    pub fn convertie(&self, format: Format) -> Valeur {
        if format.genre() == self.genre {
            Valeur { format, ..*self }
        } else {
            *self
        }
    }

    pub fn est_sans_unite(&self) -> bool {
        self.genre == Genre::SansUnite
    }

    pub fn est_longueur(&self) -> bool {
        self.genre == Genre::Longueur
    }

    pub fn est_aire(&self) -> bool {
        self.genre == Genre::Aire
    }

    /// Remet d'accord le genre et le format d'une valeur relue d'un fichier.
    pub fn assainie(self) -> Option<Valeur> {
        if self.base.is_finite() {
            Some(Valeur::en_base(self.base, self.format))
        } else {
            None
        }
    }
}

/// Comparaisons avec tolérance (R-12).
pub fn presque_egal(a: f64, b: f64) -> bool {
    (a - b).abs() <= EPS * a.abs().max(b.abs()).max(1.0)
}

pub fn est_positif(x: f64) -> bool {
    x > EPS
}

/// Opérateur arithmétique.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operateur {
    Additionner,
    Soustraire,
    Multiplier,
    Diviser,
}

impl Operateur {
    /// Symbole typographique (R-08).
    pub fn symbole(self) -> char {
        match self {
            Operateur::Additionner => '+',
            Operateur::Soustraire => '\u{2212}',
            Operateur::Multiplier => '×',
            Operateur::Diviser => '÷',
        }
    }
}

/// Format « Standard » d'une aire calculée à partir d'opérandes de formats donnés
/// (R-31), ou l'unité imposée par les préférences (R-32).
pub fn format_aire(formats: &[Format], prefs: &Preferences) -> Format {
    if let Some(impose) = prefs.format_aire_impose() {
        return impose;
    }
    match regle_familles(formats) {
        Regle::Famille(f) => f.carre(),
        Regle::Systeme(Systeme::Imperial) => Format::Pi2,
        Regle::Systeme(Systeme::Metrique) => Format::M2,
    }
}

/// Format « Standard » d'un volume calculé (R-31), ou l'unité imposée (R-32).
pub fn format_volume(formats: &[Format], prefs: &Preferences) -> Format {
    if let Some(impose) = prefs.format_volume_impose() {
        return impose;
    }
    match regle_familles(formats) {
        Regle::Famille(f) => f.cube(),
        Regle::Systeme(Systeme::Imperial) => Format::Vg3,
        Regle::Systeme(Systeme::Metrique) => Format::M3,
    }
}

enum Regle {
    Famille(Famille),
    Systeme(Systeme),
}

fn regle_familles(formats: &[Format]) -> Regle {
    let familles: Vec<Famille> = formats.iter().filter_map(|f| f.famille()).collect();
    let Some(premiere) = familles.first().copied() else {
        return Regle::Systeme(Systeme::Imperial);
    };
    if familles.iter().all(|f| *f == premiere) {
        return Regle::Famille(premiere);
    }
    let systeme = premiere.systeme();
    // Familles différentes : le système commun, sinon celui du premier opérande.
    Regle::Systeme(systeme)
}

/// Format d'une longueur issue d'une division (R-33).
fn format_quotient_lineaire(dividende: Format, diviseur: Format) -> Format {
    match (dividende.famille(), diviseur.famille()) {
        (Some(a), Some(b)) if a == b => a.lineaire(),
        (Some(a), _) if a.systeme() == Systeme::Metrique => Format::M,
        _ => Format::PiPo,
    }
}

/// Format d'une aire issue d'une division (R-33, R-32).
fn format_quotient_carre(dividende: Format, diviseur: Format, prefs: &Preferences) -> Format {
    if let Some(impose) = prefs.format_aire_impose() {
        return impose;
    }
    match (dividende.famille(), diviseur.famille()) {
        (Some(a), Some(b)) if a == b => a.carre(),
        (Some(a), _) if a.systeme() == Systeme::Metrique => Format::M2,
        _ => Format::Pi2,
    }
}

fn est_pouces(f: Format) -> bool {
    matches!(f, Format::PoFrac | Format::PoDec)
}

fn est_pieds(f: Format) -> bool {
    matches!(f, Format::PiPo | Format::PiDec)
}

/// Un nombre nu ou un pourcentage : simple facteur multiplicatif.
fn est_facteur_simple(f: Format) -> bool {
    matches!(f, Format::Nombre | Format::Pourcent)
}

fn fini(v: Valeur) -> Result<Valeur, Erreur> {
    if v.base.is_finite() {
        Ok(v)
    } else {
        Err(Erreur::Depassement)
    }
}

/// Addition ou soustraction (R-30).
pub fn additionner(a: &Valeur, b: &Valeur, soustraire: bool) -> Result<Valeur, Erreur> {
    if a.genre != b.genre {
        return Err(Erreur::Unites);
    }
    let base = if soustraire {
        a.base - b.base
    } else {
        a.base + b.base
    };
    let format = if a.genre == Genre::SansUnite {
        if a.format == Format::Dms || b.format == Format::Dms {
            Format::Dms
        } else if a.format == Format::Monnaie && b.format == Format::Monnaie {
            Format::Monnaie
        } else {
            Format::Nombre
        }
    } else if a.genre == Genre::Duree {
        // Heures et minutes l'emportent; sinon, les heures décimales.
        if a.format == b.format {
            a.format
        } else if a.format == Format::HMin || b.format == Format::HMin {
            Format::HMin
        } else {
            Format::HDec
        }
    } else if (est_pouces(a.format) && est_pieds(b.format))
        || (est_pieds(a.format) && est_pouces(b.format))
    {
        if est_pieds(a.format) {
            a.format
        } else {
            Format::PiPo
        }
    } else {
        a.format
    };
    fini(Valeur {
        base,
        genre: a.genre,
        format,
    })
}

/// Multiplication (section 9.5).
pub fn multiplier(a: &Valeur, b: &Valeur, prefs: &Preferences) -> Result<Valeur, Erreur> {
    use Genre::*;
    let resultat = match (a.genre, b.genre) {
        (SansUnite, SansUnite) => {
            let format = if est_facteur_simple(a.format) {
                if est_facteur_simple(b.format) {
                    Format::Nombre
                } else {
                    b.format
                }
            } else if est_facteur_simple(b.format) {
                a.format
            } else {
                Format::Nombre
            };
            Valeur {
                base: a.base * b.base,
                genre: SansUnite,
                format,
            }
        }
        (SansUnite, _) => Valeur {
            base: a.base * b.base,
            ..*b
        },
        (_, SansUnite) => Valeur {
            base: a.base * b.base,
            ..*a
        },
        (Longueur, Longueur) => {
            Valeur::en_base(a.base * b.base, format_aire(&[a.format, b.format], prefs))
        }
        (Longueur, Aire) | (Aire, Longueur) => {
            Valeur::en_base(a.base * b.base, format_volume(&[a.format, b.format], prefs))
        }
        _ => return Err(Erreur::Unites),
    };
    fini(resultat)
}

/// Division (section 9.5).
pub fn diviser(a: &Valeur, b: &Valeur, prefs: &Preferences) -> Result<Valeur, Erreur> {
    use Genre::*;
    let compatible = matches!(
        (a.genre, b.genre),
        (_, SansUnite) | (Aire, Longueur) | (Volume, Longueur) | (Volume, Aire)
    ) || a.genre == b.genre;
    if !compatible {
        return Err(Erreur::Unites);
    }
    if b.base == 0.0 {
        return Err(Erreur::Calcul);
    }
    let base = a.base / b.base;
    let resultat = match (a.genre, b.genre) {
        (SansUnite, SansUnite) => {
            let format = if est_facteur_simple(b.format) && !est_facteur_simple(a.format) {
                a.format
            } else {
                Format::Nombre
            };
            Valeur {
                base,
                genre: SansUnite,
                format,
            }
        }
        (_, SansUnite) => Valeur { base, ..*a },
        (Aire, Longueur) | (Volume, Aire) => {
            Valeur::en_base(base, format_quotient_lineaire(a.format, b.format))
        }
        (Volume, Longueur) => {
            Valeur::en_base(base, format_quotient_carre(a.format, b.format, prefs))
        }
        _ => Valeur::nombre(base),
    };
    fini(resultat)
}

/// Calcule `a ∘ b`.
pub fn calculer(
    a: &Valeur,
    op: Operateur,
    b: &Valeur,
    prefs: &Preferences,
) -> Result<Valeur, Erreur> {
    match op {
        Operateur::Additionner => additionner(a, b, false),
        Operateur::Soustraire => additionner(a, b, true),
        Operateur::Multiplier => multiplier(a, b, prefs),
        Operateur::Diviser => diviser(a, b, prefs),
    }
}

/// Calcul avec pourcentage (R-34) : `b` est le pourcentage, sans unité.
pub fn calculer_pourcent(a: &Valeur, op: Operateur, b: &Valeur) -> Result<Valeur, Erreur> {
    if !b.est_sans_unite() {
        return Err(Erreur::Unites);
    }
    // Un opérande déjà marqué « % » porte la fraction; sinon le nombre est le pourcentage.
    let fraction = if b.format == Format::Pourcent {
        b.base
    } else {
        b.base / 100.0
    };
    let base = match op {
        Operateur::Additionner => a.base * (1.0 + fraction),
        Operateur::Soustraire => a.base * (1.0 - fraction),
        Operateur::Multiplier => a.base * fraction,
        Operateur::Diviser => {
            if fraction == 0.0 {
                return Err(Erreur::Calcul);
            }
            a.base / fraction
        }
    };
    let format = if a.format == Format::Pourcent {
        Format::Nombre
    } else {
        a.format
    };
    fini(Valeur {
        base,
        genre: a.genre,
        format,
    })
}

/// Carré (section 9.7).
pub fn carre(v: &Valeur, prefs: &Preferences) -> Result<Valeur, Erreur> {
    match v.genre {
        Genre::SansUnite => fini(Valeur::nombre(v.base * v.base)),
        Genre::Longueur => fini(Valeur::en_base(
            v.base * v.base,
            format_aire(&[v.format], prefs),
        )),
        _ => Err(Erreur::Unites),
    }
}

/// Racine carrée (section 9.7).
pub fn racine(v: &Valeur) -> Result<Valeur, Erreur> {
    match v.genre {
        Genre::SansUnite | Genre::Aire => {
            if v.base < 0.0 {
                return Err(Erreur::Calcul);
            }
            let r = v.base.sqrt();
            if v.genre == Genre::SansUnite {
                Ok(Valeur::nombre(r))
            } else {
                let format = v
                    .format
                    .famille()
                    .map(Famille::lineaire)
                    .unwrap_or(Format::PiPo);
                Ok(Valeur::en_base(r, format))
            }
        }
        _ => Err(Erreur::Unites),
    }
}

/// Inverse (section 9.7).
pub fn inverse(v: &Valeur) -> Result<Valeur, Erreur> {
    if !v.est_sans_unite() {
        return Err(Erreur::Unites);
    }
    if v.base == 0.0 {
        return Err(Erreur::Calcul);
    }
    fini(Valeur::nombre(1.0 / v.base))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pi_po(po: f64) -> Valeur {
        Valeur::en_base(po, Format::PiPo)
    }

    #[test]
    fn somme_pouces_et_pieds_donne_pieds_pouces() {
        let a = Valeur::en_base(11.0, Format::PoFrac);
        let b = pi_po(25.0);
        let s = additionner(&a, &b, false).unwrap();
        assert_eq!(s.format, Format::PiPo);
        assert_eq!(s.base, 36.0);
    }

    #[test]
    fn somme_de_genres_differents_refusee() {
        let a = pi_po(60.0);
        let b = Valeur::nombre(3.0);
        assert_eq!(additionner(&a, &b, false), Err(Erreur::Unites));
    }

    #[test]
    fn produits_selon_les_familles() {
        let p = Preferences::default();
        let aire = multiplier(&pi_po(63.0), &pi_po(138.5), &p).unwrap();
        assert_eq!(aire.format, Format::Pi2);
        let po = Valeur::en_base(8.0, Format::PoFrac);
        assert_eq!(
            multiplier(&po, &Valeur::en_base(16.0, Format::PoFrac), &p)
                .unwrap()
                .format,
            Format::Po2
        );
        assert_eq!(
            multiplier(&pi_po(120.0), &po, &p).unwrap().format,
            Format::Pi2
        );
        let m = Valeur::avec_format(2.0, Format::M);
        let cm = Valeur::avec_format(50.0, Format::Cm);
        assert_eq!(multiplier(&m, &cm, &p).unwrap().format, Format::M2);
        assert_eq!(
            multiplier(&pi_po(120.0), &m, &p).unwrap().format,
            Format::Pi2
        );
        let pi2 = Valeur::avec_format(416.875, Format::Pi2);
        assert_eq!(
            multiplier(&pi2, &Valeur::en_base(4.0, Format::PoFrac), &p)
                .unwrap()
                .format,
            Format::Vg3
        );
        assert_eq!(
            multiplier(&pi2, &pi_po(28.0), &p).unwrap().format,
            Format::Pi3
        );
        assert_eq!(multiplier(&pi2, &pi2, &p), Err(Erreur::Unites));
    }

    #[test]
    fn divisions() {
        let p = Preferences::default();
        let q = diviser(&pi_po(364.0), &Valeur::en_base(7.0, Format::PoFrac), &p).unwrap();
        assert!(q.est_sans_unite());
        assert_eq!(q.base, 52.0);
        let r = diviser(&pi_po(243.0), &Valeur::nombre(9.0), &p).unwrap();
        assert_eq!(r.format, Format::PiPo);
        assert_eq!(
            diviser(&Valeur::nombre(5.0), &Valeur::nombre(0.0), &p),
            Err(Erreur::Calcul)
        );
        assert_eq!(
            diviser(&Valeur::nombre(5.0), &pi_po(12.0), &p),
            Err(Erreur::Unites)
        );
    }

    #[test]
    fn unite_imposee_pour_les_aires() {
        let p = Preferences {
            affichage_aires: crate::prefs::AffichageAires::MetresCarres,
            ..Preferences::default()
        };
        assert_eq!(
            multiplier(&pi_po(12.0), &pi_po(12.0), &p).unwrap().format,
            Format::M2
        );
    }
}
