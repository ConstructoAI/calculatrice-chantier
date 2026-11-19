//! Analyse d'un texte tapé au clavier d'ordinateur (section 7.9).

use crate::erreurs::Erreur;
use crate::unites::Format;
use crate::valeur::Valeur;

#[derive(Clone, Debug, PartialEq)]
enum Jeton {
    Nombre(String),
    Barre,
    Tiret,
    Mot(String),
}

/// Met le texte en minuscules et unifie espaces, guillemets et signes.
fn normaliser(texte: &str) -> Vec<char> {
    let mut sortie: Vec<char> = Vec::with_capacity(texte.len());
    for c in texte.chars() {
        match c {
            '\u{00A0}' | '\u{202F}' | '\u{2009}' | '\t' => sortie.push(' '),
            '’' | '′' | '‘' => sortie.push('\''),
            '”' | '″' | '“' | '«' | '»' => sortie.push('"'),
            '\u{2212}' | '–' => sortie.push('-'),
            autre => sortie.extend(autre.to_lowercase()),
        }
    }
    retirer_espaces_de_milliers(&sortie)
}

/// « 1 234,5 » → « 1234,5 » : un chiffre, une espace, puis exactement trois chiffres.
fn retirer_espaces_de_milliers(c: &[char]) -> Vec<char> {
    let mut sortie = Vec::with_capacity(c.len());
    let mut i = 0;
    while i < c.len() {
        let groupe = c[i] == ' '
            && i > 0
            && c[i - 1].is_ascii_digit()
            && i + 3 < c.len() + 1
            && c.get(i + 1..i + 4)
                .is_some_and(|g| g.iter().all(char::is_ascii_digit))
            && !c
                .get(i + 4)
                .is_some_and(|s| s.is_ascii_digit() || *s == '/');
        if !groupe {
            sortie.push(c[i]);
        }
        i += 1;
    }
    sortie
}

fn decouper(texte: &[char]) -> Result<Vec<Jeton>, Erreur> {
    let mut jetons = Vec::new();
    let mut i = 0;
    while i < texte.len() {
        let c = texte[i];
        if c == ' ' {
            i += 1;
        } else if c.is_ascii_digit() {
            let mut nombre = String::new();
            while i < texte.len() && texte[i].is_ascii_digit() {
                nombre.push(texte[i]);
                i += 1;
            }
            if i < texte.len() && (texte[i] == ',' || texte[i] == '.') {
                // Le séparateur doit être suivi d'au moins un chiffre.
                if !texte.get(i + 1).is_some_and(char::is_ascii_digit) {
                    return Err(Erreur::Saisie);
                }
                nombre.push('.');
                i += 1;
                while i < texte.len() && texte[i].is_ascii_digit() {
                    nombre.push(texte[i]);
                    i += 1;
                }
            }
            jetons.push(Jeton::Nombre(nombre));
        } else if c == '/' {
            jetons.push(Jeton::Barre);
            i += 1;
        } else if c == '-' {
            jetons.push(Jeton::Tiret);
            i += 1;
        } else if c == '\'' {
            // Deux apostrophes de suite valent un guillemet (pouces).
            if texte.get(i + 1) == Some(&'\'') {
                jetons.push(Jeton::Mot("\"".to_string()));
                i += 2;
            } else {
                jetons.push(Jeton::Mot("'".to_string()));
                i += 1;
            }
        } else if c == '"' || c == '°' || c == '%' || c == '$' {
            jetons.push(Jeton::Mot(c.to_string()));
            i += 1;
        } else if c.is_alphabetic() {
            let mut mot = String::new();
            while i < texte.len() && texte[i].is_alphabetic() {
                mot.push(texte[i]);
                i += 1;
            }
            // Exposant collé à l'unité : « po² », « po2 », « m3 ».
            if let Some(&e) = texte.get(i)
                && matches!(e, '²' | '³' | '2' | '3')
            {
                mot.push(match e {
                    '²' | '2' => '2',
                    _ => '3',
                });
                i += 1;
            }
            jetons.push(Jeton::Mot(mot));
        } else {
            return Err(Erreur::Saisie);
        }
    }
    Ok(jetons)
}

fn lire(nombre: &str) -> Result<f64, Erreur> {
    nombre
        .parse::<f64>()
        .ok()
        .filter(|x| x.is_finite())
        .ok_or(Erreur::Saisie)
}

fn lire_entier(nombre: &str) -> Result<f64, Erreur> {
    if nombre.contains('.') {
        return Err(Erreur::Saisie);
    }
    lire(nombre)
}

fn est_marque_pieds(mot: &str) -> bool {
    matches!(mot, "pi" | "'" | "pied" | "pieds")
}

fn est_marque_pouces(mot: &str) -> bool {
    matches!(mot, "po" | "\"" | "pouce" | "pouces")
}

/// Sépare le signe du reste.
fn signe_et_jetons(texte: &str) -> Result<(f64, Vec<Jeton>), Erreur> {
    let normal = normaliser(texte);
    let mut debut = 0;
    while debut < normal.len() && normal[debut] == ' ' {
        debut += 1;
    }
    let mut signe = 1.0;
    if let Some(&c) = normal.get(debut)
        && (c == '-' || c == '+')
    {
        if c == '-' {
            signe = -1.0;
        }
        debut += 1;
    }
    let jetons = decouper(&normal[debut..])?;
    if jetons.is_empty() {
        return Err(Erreur::Saisie);
    }
    Ok((signe, jetons))
}

/// Partie « pouces » d'une longueur impériale : nombre mixte, fraction ou nombre.
/// Retourne (pouces, contient une fraction, nombre décimal).
fn lire_pouces(jetons: &[Jeton], marque_requise: bool) -> Result<(f64, bool, bool), Erreur> {
    use Jeton::*;
    let (corps, marque) = match jetons.split_last() {
        Some((Mot(m), debut)) if est_marque_pouces(m) => (debut, true),
        Some((Mot(_), _)) => return Err(Erreur::Saisie),
        _ => (jetons, false),
    };
    let fraction = |num: &str, den: &str| -> Result<f64, Erreur> {
        let n = lire_entier(num)?;
        let d = lire_entier(den)?;
        if d == 0.0 {
            Err(Erreur::Saisie)
        } else {
            Ok(n / d)
        }
    };
    match corps {
        [Nombre(e), Nombre(n), Barre, Nombre(d)]
        | [Nombre(e), Tiret, Nombre(n), Barre, Nombre(d)] => {
            Ok((lire_entier(e)? + fraction(n, d)?, true, false))
        }
        [Nombre(n), Barre, Nombre(d)] => Ok((fraction(n, d)?, true, false)),
        [Nombre(x)] if marque || !marque_requise => Ok((lire(x)?, false, x.contains('.'))),
        _ => Err(Erreur::Saisie),
    }
}

/// Lit une longueur : « 6'8" », « 6 pi 8 po », « 7 1/2 po », « 2,032 m », « 16 ».
/// Un nombre seul prend le format `defaut`.
pub fn analyser_longueur(texte: &str, defaut: Format) -> Result<Valeur, Erreur> {
    use Jeton::*;
    let (signe, jetons) = signe_et_jetons(texte)?;
    let valeur = match jetons.as_slice() {
        [Nombre(x)] => Valeur::avec_format(lire(x)?, defaut),
        [Nombre(x), Mot(u)]
            if matches!(u.as_str(), "m" | "cm" | "mm" | "vg" | "verge" | "verges") =>
        {
            let format = match u.as_str() {
                "m" => Format::M,
                "cm" => Format::Cm,
                "mm" => Format::Mm,
                _ => Format::Vg,
            };
            Valeur::avec_format(lire(x)?, format)
        }
        [Nombre(x), Mot(u), reste @ ..] if est_marque_pieds(u) => {
            let pieds = lire(x)?;
            if reste.is_empty() {
                let format = if x.contains('.') {
                    Format::PiDec
                } else {
                    Format::PiPo
                };
                Valeur::avec_format(pieds, format)
            } else {
                let (pouces, _, _) = lire_pouces(reste, false)?;
                Valeur::en_base(pieds * 12.0 + pouces, Format::PiPo)
            }
        }
        autres => {
            let (pouces, fraction, decimal) = lire_pouces(autres, true)?;
            let format = if decimal && !fraction {
                Format::PoDec
            } else {
                Format::PoFrac
            };
            Valeur::en_base(pouces, format)
        }
    };
    if !valeur.base.is_finite() {
        return Err(Erreur::Saisie);
    }
    Ok(Valeur {
        base: signe * valeur.base,
        ..valeur
    })
}

/// Lit une aire : « 128 po² », « 128 po2 », « 1,83 pi² », « 0,17 m² ».
/// Un nombre seul prend le format `defaut`.
pub fn analyser_aire(texte: &str, defaut: Format) -> Result<Valeur, Erreur> {
    use Jeton::*;
    let (signe, jetons) = signe_et_jetons(texte)?;
    let valeur = match jetons.as_slice() {
        [Nombre(x)] => Valeur::avec_format(lire(x)?, defaut),
        [Nombre(x), Mot(u)] => {
            let format = match u.as_str() {
                "po2" => Format::Po2,
                "pi2" => Format::Pi2,
                "vg2" => Format::Vg2,
                "m2" => Format::M2,
                "cm2" => Format::Cm2,
                "mm2" => Format::Mm2,
                "acre" | "acres" => Format::Acre,
                _ => return Err(Erreur::Saisie),
            };
            Valeur::avec_format(lire(x)?, format)
        }
        _ => return Err(Erreur::Saisie),
    };
    if !valeur.base.is_finite() {
        return Err(Erreur::Saisie);
    }
    Ok(Valeur {
        base: signe * valeur.base,
        ..valeur
    })
}

/// Lit un nombre nu : « 38 », « 1,75 », « 45° ».
pub fn analyser_nombre(texte: &str) -> Result<f64, Erreur> {
    use Jeton::*;
    let (signe, jetons) = signe_et_jetons(texte)?;
    match jetons.as_slice() {
        [Nombre(x)] => Ok(signe * lire(x)?),
        [Nombre(x), Mot(u)] if u == "°" => Ok(signe * lire(x)?),
        _ => Err(Erreur::Saisie),
    }
}

/// Lit un texte collé : un nombre nu ou une longueur.
pub fn analyser_collage(texte: &str) -> Result<Valeur, Erreur> {
    analyser_longueur(texte, Format::Nombre)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unites::Genre;

    fn po(texte: &str) -> f64 {
        analyser_longueur(texte, Format::PoFrac).unwrap().base
    }

    fn proche(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn longueurs_de_la_section_7_9() {
        assert!(proche(po("6'8\""), 80.0));
        assert!(proche(po("6 pi 8 po"), 80.0));
        assert!(proche(po("7 1/2 po"), 7.5));
        assert!(proche(po("7-1/2\""), 7.5));
        assert!(proche(po("5' 7 1/2\""), 67.5));
        assert!(proche(po("2,032 m"), 80.0));
        assert!(proche(po("1/2"), 0.5));
        assert!(proche(po("16"), 16.0));
    }

    #[test]
    fn variantes_acceptees() {
        assert!(proche(po("  6 PIEDS 8 Pouces "), 80.0));
        assert!(proche(po("6'8''"), 80.0));
        assert!(proche(po("6\u{00A0}pi\u{00A0}8\u{00A0}po"), 80.0));
        assert!(proche(po("−5 pi 3 1/2 po"), -63.5));
        assert!(proche(po("3 vg"), 108.0));
        assert!(proche(po("2.5 cm"), 2.5 / 2.54));
        assert!(proche(po("1 250 pi 6 po"), 15006.0));
        assert_eq!(
            analyser_longueur("17,32 pi", Format::PoFrac)
                .unwrap()
                .format,
            Format::PiDec
        );
        assert_eq!(
            analyser_longueur("6'8\"", Format::PoFrac).unwrap().format,
            Format::PiPo
        );
        assert_eq!(
            analyser_longueur("9,0625 po", Format::PoFrac)
                .unwrap()
                .format,
            Format::PoDec
        );
        assert_eq!(
            analyser_longueur("2 m", Format::PoFrac).unwrap().format,
            Format::M
        );
    }

    #[test]
    fn aires_de_la_section_7_9() {
        let aire = |t: &str| analyser_aire(t, Format::Po2).unwrap();
        assert!(proche(aire("128 po²").base, 128.0));
        assert!(proche(aire("128 po2").base, 128.0));
        assert!(proche(aire("1,83 pi²").base, 1.83 * 144.0));
        assert!(proche(aire("0,17 m²").numerique(), 0.17));
        assert_eq!(aire("0,17 m²").genre, Genre::Aire);
        assert!(proche(aire("264").base, 264.0));
    }

    #[test]
    fn entrees_invalides() {
        for texte in [
            "abc",
            "5 pi 6 pi",
            "1/0",
            "--3",
            "",
            "   ",
            "5 po 6 pi",
            "1/2/3",
            "5,",
            "5 m 3",
            "pi",
            "7 1/2 m",
            "6' 8 9",
            "1,5/2",
        ] {
            assert!(
                analyser_longueur(texte, Format::PoFrac).is_err(),
                "« {texte} » devrait être refusé"
            );
        }
        for texte in ["", "128 po", "abc", "12 m3", "1/2 po²"] {
            assert!(
                analyser_aire(texte, Format::Po2).is_err(),
                "« {texte} » devrait être refusé"
            );
        }
    }

    #[test]
    fn nombres_et_collage() {
        assert!(proche(analyser_nombre("38").unwrap(), 38.0));
        assert!(proche(analyser_nombre("1,75").unwrap(), 1.75));
        assert!(proche(analyser_nombre("45°").unwrap(), 45.0));
        assert!(analyser_nombre("45 po").is_err());
        let v = analyser_collage("1 234,5").unwrap();
        assert!(v.est_sans_unite() && proche(v.base, 1234.5));
        let l = analyser_collage("5 pi 7 1/2 po").unwrap();
        assert!(l.est_longueur() && proche(l.base, 67.5));
    }
}
