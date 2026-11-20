//! Analyse d'un texte tapé au clavier d'ordinateur (section 7.9).
//!
//! Les unités se lisent dans les deux langues, quelle que soit la langue en
//! cours : « 6 pi 8 po » comme « 6 ft 8 in ». Seule la virgule change de sens
//! (section 24) : séparateur décimal en français (« 2,032 m »), elle sépare les
//! milliers en anglais (« 1,234.5 »).

use crate::erreurs::Erreur;
use crate::etiquettes::symbole;
use crate::langue::{self, Langue};
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

/// Vrai si une virgule, à la position `i`, est suivie d'un groupe de milliers :
/// exactement trois chiffres.
fn groupe_de_milliers(texte: &[char], i: usize) -> bool {
    texte.get(i) == Some(&',')
        && texte
            .get(i + 1..i + 4)
            .is_some_and(|groupe| groupe.iter().all(char::is_ascii_digit))
        && !texte.get(i + 4).is_some_and(char::is_ascii_digit)
}

fn decouper(texte: &[char], langue: Langue) -> Result<Vec<Jeton>, Erreur> {
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
            // En anglais, la virgule sépare les milliers : « 1,234.5 ».
            let mut groupes = false;
            if langue == Langue::Anglais && nombre.len() <= 3 && !nombre.starts_with('0') {
                while groupe_de_milliers(texte, i) {
                    nombre.extend(&texte[i + 1..i + 4]);
                    i += 4;
                    groupes = true;
                }
            }
            if groupes && texte.get(i) == Some(&',') {
                // « 1,234,5 » : une virgule mal placée.
                return Err(Erreur::Saisie);
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
    matches!(mot, "pi" | "'" | "pied" | "pieds" | "ft" | "foot" | "feet")
}

fn est_marque_pouces(mot: &str) -> bool {
    matches!(
        mot,
        "po" | "\"" | "pouce" | "pouces" | "in" | "inch" | "inches"
    )
}

/// Format d'une longueur qui ne s'écrit ni en pieds ni en pouces.
fn format_de_longueur(mot: &str) -> Option<Format> {
    Some(match mot {
        "m" => Format::M,
        "cm" => Format::Cm,
        "mm" => Format::Mm,
        "vg" | "verge" | "verges" | "yd" | "yds" | "yard" | "yards" => Format::Vg,
        _ => return None,
    })
}

/// Format d'une aire, d'après son unité (l'exposant collé : « pi2 », « ft2 »).
fn format_d_aire(mot: &str) -> Option<Format> {
    Some(match mot {
        "po2" | "in2" => Format::Po2,
        "pi2" | "ft2" => Format::Pi2,
        "vg2" | "yd2" => Format::Vg2,
        "m2" => Format::M2,
        "cm2" => Format::Cm2,
        "mm2" => Format::Mm2,
        "acre" | "acres" => Format::Acre,
        _ => return None,
    })
}

/// Sépare le signe du reste.
fn signe_et_jetons(texte: &str) -> Result<(f64, Vec<Jeton>), Erreur> {
    signe_et_jetons_en(texte, langue::en_cours())
}

/// Sépare le signe du reste, la virgule étant lue comme dans la langue donnée.
fn signe_et_jetons_en(texte: &str, langue: Langue) -> Result<(f64, Vec<Jeton>), Erreur> {
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
    let jetons = decouper(&normal[debut..], langue)?;
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
        [Nombre(x), Mot(u)] if format_de_longueur(u).is_some() => {
            Valeur::avec_format(lire(x)?, format_de_longueur(u).unwrap_or(defaut))
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
            Valeur::avec_format(lire(x)?, format_d_aire(u).ok_or(Erreur::Saisie)?)
        }
        // « 128 sq in », « 1.83 sq ft » : l'écriture anglaise en deux mots.
        [Nombre(x), Mot(sq), Mot(u)] if sq == "sq" => {
            let format = match u.as_str() {
                "in" => Format::Po2,
                "ft" => Format::Pi2,
                "yd" => Format::Vg2,
                "m" => Format::M2,
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

/// Symbole, dans la langue en cours, de l'unité que désigne un mot tapé.
fn symbole_du_mot(mot: &str) -> Option<&'static str> {
    let format = if est_marque_pieds(mot) {
        Format::PiPo
    } else if est_marque_pouces(mot) {
        Format::PoFrac
    } else {
        format_de_longueur(mot).or_else(|| format_d_aire(mot))?
    };
    Some(symbole(format))
}

/// Récrit une saisie comme elle se taperait dans une autre langue (section 24) :
/// le séparateur décimal, les milliers et les symboles d'unités changent, la
/// valeur reste la même. « 2,032 m » devient « 2.032 m », « 12 pi 6 1/2 po »
/// devient « 12 ft 6 1/2 in », et « 1,500 » redevient « 1500 ».
///
/// `decimal` : le séparateur décimal de la langue d'arrivée. Une saisie qui ne
/// se lit pas est rendue telle quelle.
pub fn transposer(texte: &str, de: Langue, vers: Langue, decimal: char) -> String {
    let Ok((signe, jetons)) = signe_et_jetons_en(texte, de) else {
        return texte.to_string();
    };
    let _langue = langue::pour_la_portee(vers);
    let mut sortie = String::new();
    if signe < 0.0 {
        sortie.push('-');
    }
    let mut precedent: Option<&Jeton> = None;
    for jeton in &jetons {
        match jeton {
            Jeton::Nombre(nombre) => {
                if matches!(precedent, Some(Jeton::Nombre(_) | Jeton::Mot(_))) {
                    sortie.push(' ');
                }
                sortie.extend(nombre.chars().map(|c| if c == '.' { decimal } else { c }));
            }
            Jeton::Barre => sortie.push('/'),
            Jeton::Tiret => sortie.push('-'),
            Jeton::Mot(mot) => match mot.as_str() {
                // Les signes se collent au nombre et ne changent pas.
                "'" | "\"" | "°" | "%" | "$" => sortie.push_str(mot),
                mot => match symbole_du_mot(mot) {
                    Some(unite) => {
                        sortie.push(' ');
                        sortie.push_str(unite);
                    }
                    None => return texte.to_string(),
                },
            },
        }
        precedent = Some(jeton);
    }
    sortie
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
    fn unites_anglaises_dans_les_deux_langues() {
        // Les mots anglais se lisent aussi en français, et inversement.
        for langue in Langue::TOUTES {
            let _langue = langue::pour_la_portee(langue);
            assert!(proche(po("6 ft 8 in"), 80.0));
            assert!(proche(po("6 FEET 8 Inches"), 80.0));
            assert!(proche(po("7 1/2 in"), 7.5));
            assert!(proche(po("3 yd"), 108.0));
            assert!(proche(po("2 yards"), 72.0));
            assert!(proche(po("6 pi 8 po"), 80.0));
            assert!(proche(po("3 vg"), 108.0));
            let aire = |t: &str| analyser_aire(t, Format::Po2).unwrap();
            assert!(proche(aire("128 in²").base, 128.0));
            assert!(proche(aire("2 ft2").base, 288.0));
            assert!(proche(aire("1 yd²").base, 1296.0));
            assert!(proche(aire("2 sq ft").base, 288.0));
            assert!(proche(aire("128 sq in").base, 128.0));
            assert!(proche(aire("2 pi²").base, 288.0));
            assert!(analyser_aire("2 sq lb", Format::Po2).is_err());
        }
        assert_eq!(
            analyser_longueur("17.32 ft", Format::PoFrac)
                .unwrap()
                .format,
            Format::PiDec
        );
    }

    #[test]
    fn la_virgule_separe_les_milliers_en_anglais_seulement() {
        // En français, la virgule est toujours le séparateur décimal.
        assert!(proche(analyser_nombre("1,500").unwrap(), 1.5));
        assert!(proche(analyser_nombre("2,032").unwrap(), 2.032));
        assert!(analyser_nombre("1,234.5").is_err());
        let _anglais = langue::pour_la_portee(Langue::Anglais);
        assert!(proche(analyser_nombre("1,500").unwrap(), 1500.0));
        assert!(proche(analyser_nombre("1,234.5").unwrap(), 1234.5));
        assert!(proche(
            analyser_nombre("12,345,678.9").unwrap(),
            12_345_678.9
        ));
        assert!(proche(analyser_nombre("25,000").unwrap(), 25_000.0));
        assert!(proche(analyser_nombre("1500.25").unwrap(), 1500.25));
        // Une virgule qui ne sépare pas trois chiffres reste un séparateur décimal.
        assert!(proche(analyser_nombre("1,5").unwrap(), 1.5));
        assert!(proche(analyser_nombre("0,500").unwrap(), 0.5));
        assert!(proche(analyser_nombre("1,2345").unwrap(), 1.2345));
        assert!(proche(analyser_nombre("1234,567").unwrap(), 1234.567));
        assert!(analyser_nombre("1,234,5").is_err());
        assert!(analyser_nombre("1,234,").is_err());
        assert!(proche(po("1,250 ft 6 in"), 15006.0));
        assert!(proche(po("2.032 m"), 80.0));
        let v = analyser_collage("1,234.5").unwrap();
        assert!(v.est_sans_unite() && proche(v.base, 1234.5));
    }

    #[test]
    fn saisies_transposees_d_une_langue_a_l_autre() {
        let fr_en = |t: &str| transposer(t, Langue::Francais, Langue::Anglais, '.');
        let en_fr = |t: &str| transposer(t, Langue::Anglais, Langue::Francais, ',');
        assert_eq!(fr_en("2,032 m"), "2.032 m");
        assert_eq!(fr_en("12 pi 6 1/2 po"), "12 ft 6 1/2 in");
        assert_eq!(fr_en("12\u{a0}pi\u{a0}6\u{a0}po"), "12 ft 6 in");
        assert_eq!(fr_en("1 250 pi 6 po"), "1250 ft 6 in");
        assert_eq!(fr_en("9,975"), "9.975");
        assert_eq!(fr_en("5 1/2"), "5 1/2");
        assert_eq!(fr_en("7-1/2\""), "7-1/2\"");
        assert_eq!(fr_en("6'8\""), "6' 8\"");
        assert_eq!(fr_en("−0,25"), "-0.25");
        assert_eq!(fr_en("3 vg"), "3 yd");
        assert_eq!(fr_en("128 po²"), "128 in²");
        assert_eq!(fr_en("45°"), "45°");
        assert_eq!(en_fr("2.032 m"), "2,032 m");
        assert_eq!(en_fr("1,500"), "1500");
        assert_eq!(en_fr("1,234.5 ft"), "1234,5 pi");
        assert_eq!(en_fr("12 ft 6 1/2 in"), "12 pi 6 1/2 po");
        assert_eq!(en_fr("1.83 ft²"), "1,83 pi²");
        // Avec le point comme séparateur décimal du français.
        assert_eq!(
            transposer("2.5 ft", Langue::Anglais, Langue::Francais, '.'),
            "2.5 pi"
        );
        // Ce qui ne se lit pas reste tel quel, comme une saisie vide.
        assert_eq!(fr_en("abc"), "abc");
        assert_eq!(fr_en("12 lb"), "12 lb");
        assert_eq!(fr_en(""), "");
        // La valeur ne change jamais.
        for texte in ["2,032 m", "12 pi 6 1/2 po", "0,5", "3 1/2", "17,32 pi"] {
            let avant = analyser_longueur(texte, Format::PoFrac).unwrap();
            let _anglais = langue::pour_la_portee(Langue::Anglais);
            let apres = analyser_longueur(&fr_en(texte), Format::PoFrac).unwrap();
            assert!(proche(avant.base, apres.base), "{texte}");
            assert_eq!(avant.format, apres.format, "{texte}");
        }
        // La langue en cours n'est pas dérangée.
        assert_eq!(langue::en_cours(), Langue::Francais);
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
