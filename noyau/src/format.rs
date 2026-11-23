//! Formatage des valeurs pour l'afficheur (section 8).

use crate::etiquettes::{DEGRE, INSECABLE, MINUTE, MOINS, SECONDE, symbole};
use crate::langue::{self, Langue};
use crate::prefs::{ModeFractions, Preferences, SeparateurDecimal};
use crate::unites::{EPS, Format, Genre};
use crate::valeur::Valeur;

/// Morceau typé du texte d'une valeur (section 8.9).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Segment {
    /// Chiffres, signes, « ° ».
    Texte(String),
    /// Fraction, dessinée empilée par l'interface.
    Fraction { num: u32, den: u32 },
    /// Symbole d'unité, dessiné en plus petit.
    Unite(String),
}

/// Texte structuré d'une valeur.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TexteValeur(pub Vec<Segment>);

impl TexteValeur {
    pub fn simple(texte: impl Into<String>) -> TexteValeur {
        TexteValeur(vec![Segment::Texte(texte.into())])
    }

    /// Pour chaque morceau : vrai s'il est séparé du précédent par une espace.
    ///
    /// Un signe seul se colle à ce qui suit (« −5/16 po »). En anglais, le
    /// signe de dollar précède le nombre et s'y colle (« $283.06 »), et le
    /// signe de pourcentage suit le nombre sans espace (« 47.25% »).
    pub fn espaces(&self) -> Vec<bool> {
        let anglais = langue::en_cours() == Langue::Anglais;
        let mut colle = true;
        self.0
            .iter()
            .map(|segment| {
                let (espace, colle_ensuite) = match segment {
                    Segment::Texte(t) => (!colle, t.chars().count() == 1 && t.starts_with(MOINS)),
                    Segment::Unite(u) if anglais && u == "$" => (!colle, true),
                    Segment::Unite(u) if anglais && u == "%" => (false, false),
                    Segment::Unite(_) | Segment::Fraction { .. } => (!colle, false),
                };
                colle = colle_ensuite;
                espace
            })
            .collect()
    }

    /// Texte à plat, avec espaces insécables : « 5 pi 7 1/2 po ».
    pub fn en_texte(&self) -> String {
        let mut sortie = String::new();
        for (segment, espace) in self.0.iter().zip(self.espaces()) {
            if espace {
                sortie.push(INSECABLE);
            }
            match segment {
                Segment::Texte(t) => sortie.push_str(t),
                Segment::Fraction { num, den } => sortie.push_str(&format!("{num}/{den}")),
                Segment::Unite(u) => sortie.push_str(u),
            }
        }
        sortie
    }
}

/// Manière de rendre le nombre d'une sortie (section 8.6).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Rendu {
    /// Selon le format de la valeur et les préférences.
    #[default]
    Normal,
    /// Dénombrement entier.
    Entier,
    /// Deux décimales fixes (carrés, paquets, feuilles de toiture).
    DeuxDecimales,
    /// Flottant, sans égard à la préférence des valeurs sans unité (blocs, gypse).
    Flottant,
}

/// Contexte de formatage.
#[derive(Clone, Copy, Debug)]
pub struct OptionsFormat<'a> {
    pub prefs: &'a Preferences,
    /// Résolution temporaire des fractions (R-24).
    pub resolution_temp: Option<u32>,
}

impl<'a> OptionsFormat<'a> {
    pub fn nouvelles(prefs: &'a Preferences) -> Self {
        OptionsFormat {
            prefs,
            resolution_temp: None,
        }
    }

    /// Résolution des fractions en vigueur.
    pub fn resolution(&self) -> u32 {
        match self.resolution_temp {
            Some(r) if crate::prefs::RESOLUTIONS.contains(&r) => r,
            _ => self.prefs.resolution_valide(),
        }
    }
}

/// Capacité d'affichage en clair (R-25).
const CAPACITE: f64 = 1e12;
/// Seuil de passage à l'unité supérieure ou à la notation exponentielle (section 8.7).
const SEUIL_GRAND: f64 = 1e9;
const SEUIL_PETIT: f64 = 1e-6;
/// Coup de pouce relatif : une demie exacte s'arrondit vers le haut.
const POUSSEE: f64 = 1e-12;

/// Chiffres d'un nombre positif à `decimales` décimales fixes (séparateur « . »).
fn chiffres_fixes(a: f64, decimales: usize) -> String {
    format!("{:.*}", decimales, a * (1.0 + POUSSEE))
}

/// Chiffres « flottants » d'un nombre positif : 7 chiffres au total (R-22).
fn chiffres_flottants(a: f64) -> String {
    let a = a * (1.0 + POUSSEE);
    let mut entiers = nb_chiffres_entiers(a);
    let mut texte;
    loop {
        let decimales = 7usize.saturating_sub(entiers);
        texte = format!("{:.*}", decimales, a);
        let obtenus = texte.split('.').next().map(str::len).unwrap_or(1);
        if obtenus > entiers && decimales > 0 {
            entiers = obtenus;
        } else {
            break;
        }
    }
    if texte.contains('.') {
        let rogne = texte.trim_end_matches('0').trim_end_matches('.');
        texte = rogne.to_string();
    }
    texte
}

fn nb_chiffres_entiers(a: f64) -> usize {
    if a < 1.0 {
        1
    } else {
        format!("{:.0}", a.trunc()).len()
    }
}

/// Habille des chiffres bruts (« 1234.5 ») : signe, milliers, séparateur décimal.
fn habiller(chiffres: &str, negatif: bool, prefs: &Preferences) -> String {
    let (entier, decimales) = match chiffres.split_once('.') {
        Some((e, d)) => (e, Some(d)),
        None => (chiffres, None),
    };
    let nul = chiffres.chars().all(|c| c == '0' || c == '.');
    let mut sortie = String::new();
    if negatif && !nul {
        sortie.push(MOINS);
    }
    sortie.push_str(&grouper(entier, prefs));
    if let Some(d) = decimales {
        sortie.push(separateur_decimal(prefs));
        sortie.push_str(d);
    }
    sortie
}

/// Caractère du séparateur décimal : en français, celui des préférences; en
/// anglais, toujours le point (section 24).
pub fn separateur_decimal(prefs: &Preferences) -> char {
    separateur_decimal_en(prefs, langue::en_cours())
}

/// Caractère du séparateur décimal dans une langue donnée.
pub fn separateur_decimal_en(prefs: &Preferences, langue: Langue) -> char {
    match (langue, prefs.separateur_decimal) {
        (Langue::Anglais, _) | (Langue::Francais, SeparateurDecimal::Point) => '.',
        (Langue::Francais, SeparateurDecimal::Virgule) => ',',
    }
}

/// Caractère qui sépare les groupes de trois chiffres : l'espace insécable en
/// français (R-04), la virgule en anglais.
pub fn separateur_de_milliers() -> char {
    match langue::en_cours() {
        Langue::Francais => INSECABLE,
        Langue::Anglais => ',',
    }
}

/// Groupe des chiffres par trois (R-04) : « 1 234 567 », « 1,234,567 » en anglais.
pub fn grouper(entier: &str, prefs: &Preferences) -> String {
    if !prefs.separateur_milliers {
        return entier.to_string();
    }
    let separateur = separateur_de_milliers();
    let n = entier.chars().count();
    let mut sortie = String::with_capacity(n + n / 3);
    for (i, c) in entier.chars().enumerate() {
        if i > 0 && (n - i).is_multiple_of(3) {
            sortie.push(separateur);
        }
        sortie.push(c);
    }
    sortie
}

/// Un nombre déjà écrit et son unité, à plat : « 12,5 pi² », « 45° »; en
/// anglais, « $1,234.56 », « $25.00/ft² », « 10% » et « 8.5¢/kWh ».
pub fn nombre_et_unite(nombre: &str, unite: &str) -> String {
    if unite.is_empty() {
        return nombre.to_string();
    }
    if unite == "°" {
        return format!("{nombre}°");
    }
    if langue::en_cours() == Langue::Anglais {
        if let Some(suite) = unite.strip_prefix('$') {
            // Le signe du nombre passe devant le signe de dollar.
            return match nombre.strip_prefix(MOINS) {
                Some(absolu) => format!("{MOINS}${absolu}{suite}"),
                None => format!("${nombre}{suite}"),
            };
        }
        if unite == "%" || unite.starts_with('¢') {
            return format!("{nombre}{unite}");
        }
    }
    format!("{nombre}{INSECABLE}{unite}")
}

/// Notation scientifique : « 2,5 E9 », « 1,2 E−7 ».
fn exponentielle(x: f64, prefs: &Preferences) -> String {
    let a = x.abs();
    if a == 0.0 || !a.is_finite() {
        return "0".to_string();
    }
    let mut exposant = a.log10().floor() as i32;
    let mut mantisse = chiffres_flottants(a / 10f64.powi(exposant));
    if mantisse.starts_with("10") {
        exposant += 1;
        mantisse = chiffres_flottants(a / 10f64.powi(exposant));
    }
    let mut sortie = habiller(&mantisse, x < 0.0, prefs);
    sortie.push(INSECABLE);
    sortie.push('E');
    if exposant < 0 {
        sortie.push(MOINS);
    }
    sortie.push_str(&exposant.unsigned_abs().to_string());
    sortie
}

/// Nombre flottant (R-22), avec la notation exponentielle au besoin (section 8.7).
pub fn nombre_flottant(x: f64, prefs: &Preferences) -> String {
    if !x.is_finite() {
        return "0".to_string();
    }
    let a = x.abs();
    let exp = (prefs.exponentielle && (a >= SEUIL_GRAND || (a > 0.0 && a < SEUIL_PETIT)))
        || a >= CAPACITE;
    if exp {
        return exponentielle(x, prefs);
    }
    habiller(&chiffres_flottants(a), x < 0.0, prefs)
}

/// Nombre à décimales fixes.
pub fn nombre_fixe(x: f64, decimales: usize, prefs: &Preferences) -> String {
    if !x.is_finite() {
        return "0".to_string();
    }
    let a = x.abs();
    if (prefs.exponentielle && a >= SEUIL_GRAND) || a >= CAPACITE {
        return exponentielle(x, prefs);
    }
    habiller(&chiffres_fixes(a, decimales), x < 0.0, prefs)
}

/// Format réellement utilisé : passe à l'unité supérieure pour les très grandes
/// valeurs quand la notation exponentielle est désactivée (section 8.7).
pub fn format_effectif(v: &Valeur, prefs: &Preferences) -> Format {
    let mut format = v.format;
    if prefs.exponentielle {
        // Les formats à fraction ne se prêtent pas à la notation scientifique.
        if (v.base / format.facteur()).abs() >= SEUIL_GRAND {
            format = match format {
                Format::PiPo => Format::PiDec,
                Format::PoFrac => Format::PoDec,
                autre => autre,
            };
        }
        return format;
    }
    while (v.base / format.facteur()).abs() >= SEUIL_GRAND {
        match format.superieur() {
            Some(plus_grand) => format = plus_grand,
            None => break,
        }
    }
    format
}

/// Vrai si la valeur peut s'afficher (R-25).
pub fn est_affichable(v: &Valeur, prefs: &Preferences) -> bool {
    if !v.base.is_finite() {
        return false;
    }
    let format = format_effectif(v, prefs);
    let numerique = (v.base / format.facteur()).abs();
    if prefs.exponentielle {
        numerique < 1e100
    } else {
        numerique < CAPACITE
    }
}

fn pgcd(a: u32, b: u32) -> u32 {
    let (mut a, mut b) = (a, b);
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a.max(1)
}

/// Décompose des pouces en unités de fraction arrondies (R-23).
/// Retourne (pouces entiers, numérateur, dénominateur); le numérateur peut être nul.
fn decomposer(pouces_abs: f64, resolution: u32, mode: ModeFractions) -> (u64, u32, u32) {
    let res = resolution.max(1) as f64;
    let unites = (pouces_abs * res + 0.5 + EPS).floor().max(0.0) as u64;
    let entiers = unites / resolution.max(1) as u64;
    let reste = (unites % resolution.max(1) as u64) as u32;
    if reste == 0 {
        return (entiers, 0, resolution);
    }
    match mode {
        ModeFractions::Standard => {
            let d = pgcd(reste, resolution);
            (entiers, reste / d, resolution / d)
        }
        ModeFractions::Constant => (entiers, reste, resolution),
    }
}

/// Arrondit une longueur (en pouces) à la résolution des fractions (R-23, R-42).
pub fn arrondir_a_la_fraction(pouces: f64, resolution: u32) -> f64 {
    let res = resolution.max(1) as f64;
    let unites = (pouces.abs() * res + 0.5 + EPS).floor();
    (unites / res).copysign(pouces)
}

fn segments_pieds_pouces(pouces: f64, o: &OptionsFormat) -> Vec<Segment> {
    let resolution = o.resolution();
    let (pouces_entiers, num, den) = decomposer(pouces.abs(), resolution, o.prefs.mode_fractions);
    let pieds = pouces_entiers / 12;
    let reste_pouces = pouces_entiers % 12;
    let nul = pouces_entiers == 0 && num == 0;
    let mut texte_pieds = String::new();
    if pouces < 0.0 && !nul {
        texte_pieds.push(MOINS);
    }
    texte_pieds.push_str(&grouper(&pieds.to_string(), o.prefs));
    let mut segments = vec![
        Segment::Texte(texte_pieds),
        Segment::Unite(symbole(Format::PiPo).to_string()),
        Segment::Texte(reste_pouces.to_string()),
    ];
    if num != 0 {
        segments.push(Segment::Fraction { num, den });
    }
    segments.push(Segment::Unite(symbole(Format::PoFrac).to_string()));
    segments
}

/// « 16 h 15 » : les heures, puis les minutes sur deux chiffres, à la minute
/// près (en anglais, « 16 h 15 min »).
fn segments_heures_minutes(minutes: f64, o: &OptionsFormat) -> Vec<Segment> {
    let total = minutes.abs().round() as u64;
    let (heures, reste) = (total / 60, total % 60);
    let mut texte_heures = String::new();
    if minutes < 0.0 && total != 0 {
        texte_heures.push(MOINS);
    }
    texte_heures.push_str(&grouper(&heures.to_string(), o.prefs));
    let mut segments = vec![
        Segment::Texte(texte_heures),
        Segment::Unite(symbole(Format::HMin).to_string()),
        Segment::Texte(format!("{reste:02}")),
    ];
    if langue::en_cours() == Langue::Anglais {
        segments.push(Segment::Unite(symbole(Format::Min).to_string()));
    }
    segments
}

fn segments_pouces_fraction(pouces: f64, o: &OptionsFormat) -> Vec<Segment> {
    let resolution = o.resolution();
    let (entiers, num, den) = decomposer(pouces.abs(), resolution, o.prefs.mode_fractions);
    let nul = entiers == 0 && num == 0;
    let negatif = pouces < 0.0 && !nul;
    let mut segments = Vec::new();
    if entiers == 0 && num != 0 {
        if negatif {
            segments.push(Segment::Texte(MOINS.to_string()));
        }
        segments.push(Segment::Fraction { num, den });
    } else {
        let mut texte = String::new();
        if negatif {
            texte.push(MOINS);
        }
        texte.push_str(&grouper(&entiers.to_string(), o.prefs));
        segments.push(Segment::Texte(texte));
        if num != 0 {
            segments.push(Segment::Fraction { num, den });
        }
    }
    segments.push(Segment::Unite(symbole(Format::PoFrac).to_string()));
    segments
}

/// Angle en degrés-minutes-secondes : « 44°17′24″ ».
pub fn texte_dms(degres: f64) -> String {
    if !degres.is_finite() {
        return "0".to_string();
    }
    let total = (degres.abs() * 3600.0 + 0.5 + EPS).floor().max(0.0) as u64;
    let (d, m, s) = (total / 3600, (total % 3600) / 60, total % 60);
    let signe = if degres < 0.0 && total != 0 {
        MOINS.to_string()
    } else {
        String::new()
    };
    format!("{signe}{d}{DEGRE}{m:02}{MINUTE}{s:02}{SECONDE}")
}

/// Formate une valeur pour l'afficheur.
pub fn formater(v: &Valeur, rendu: Rendu, o: &OptionsFormat) -> TexteValeur {
    let prefs = o.prefs;
    let format = format_effectif(v, prefs);
    let numerique = v.base / format.facteur();
    let unite = symbole(format);
    let dollar_devant = format == Format::Monnaie && langue::en_cours() == Langue::Anglais;
    let avec_unite = |nombre: String| -> TexteValeur {
        if dollar_devant {
            // « $283.06 », « −$283.06 ».
            let mut segments = Vec::with_capacity(3);
            let absolu = match nombre.strip_prefix(MOINS) {
                Some(absolu) => {
                    segments.push(Segment::Texte(MOINS.to_string()));
                    absolu.to_string()
                }
                None => nombre,
            };
            segments.push(Segment::Unite(unite.to_string()));
            segments.push(Segment::Texte(absolu));
            return TexteValeur(segments);
        }
        let mut segments = vec![Segment::Texte(nombre)];
        if !unite.is_empty() {
            segments.push(Segment::Unite(unite.to_string()));
        }
        TexteValeur(segments)
    };
    match rendu {
        Rendu::Entier => return avec_unite(nombre_fixe(numerique.round(), 0, prefs)),
        Rendu::DeuxDecimales => return avec_unite(nombre_fixe(numerique, 2, prefs)),
        Rendu::Flottant => return avec_unite(nombre_flottant(numerique, prefs)),
        Rendu::Normal => {}
    }
    match format {
        Format::Nombre => match prefs.decimales_sans_unite {
            Some(n) => TexteValeur::simple(nombre_fixe(numerique, usize::from(n.min(4)), prefs)),
            None => TexteValeur::simple(nombre_flottant(numerique, prefs)),
        },
        Format::Degres => {
            let mut texte = if prefs.degres_fixes {
                nombre_fixe(numerique, 2, prefs)
            } else {
                nombre_flottant(numerique, prefs)
            };
            texte.push(DEGRE);
            TexteValeur::simple(texte)
        }
        Format::Dms => TexteValeur::simple(texte_dms(numerique)),
        Format::Monnaie => avec_unite(nombre_fixe(numerique, 2, prefs)),
        Format::PiPo => TexteValeur(segments_pieds_pouces(v.base, o)),
        Format::PoFrac => TexteValeur(segments_pouces_fraction(v.base, o)),
        Format::HMin => TexteValeur(segments_heures_minutes(v.base, o)),
        Format::M if prefs.metres_fixes => avec_unite(nombre_fixe(numerique, 3, prefs)),
        _ => avec_unite(nombre_flottant(numerique, prefs)),
    }
}

/// Raccourci : texte à plat d'une valeur, rendu normal.
pub fn en_texte(v: &Valeur, o: &OptionsFormat) -> String {
    formater(v, Rendu::Normal, o).en_texte()
}

/// Vrai si la valeur s'affiche avec une fraction de pouce.
pub fn utilise_les_fractions(v: &Valeur) -> bool {
    v.genre == Genre::Longueur && v.format.est_fractionnaire()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prefs::{ModeFractions, SeparateurDecimal};

    /// Texte comparable : espaces insécables remplacées par des espaces ordinaires.
    fn t(v: Valeur, prefs: &Preferences) -> String {
        en_texte(&v, &OptionsFormat::nouvelles(prefs)).replace(INSECABLE, " ")
    }

    fn nb(x: f64) -> String {
        t(Valeur::nombre(x), &Preferences::default())
    }

    #[test]
    fn vecteurs_19_15_longueurs() {
        let p = Preferences::default();
        assert_eq!(t(Valeur::en_base(11.99, Format::PiPo), &p), "1 pi 0 po");
        assert_eq!(t(Valeur::en_base(11.97, Format::PiPo), &p), "1 pi 0 po");
        assert_eq!(
            t(Valeur::en_base(-63.5, Format::PiPo), &p),
            "−5 pi 3 1/2 po"
        );
        assert_eq!(t(Valeur::en_base(0.5, Format::PoFrac), &p), "1/2 po");
        assert_eq!(t(Valeur::en_base(0.03, Format::PoFrac), &p), "0 po");
        assert_eq!(t(Valeur::en_base(0.04, Format::PoFrac), &p), "1/16 po");
        assert_eq!(
            t(Valeur::en_base(15006.0, Format::PiPo), &p),
            "1 250 pi 6 po"
        );
        assert_eq!(t(Valeur::en_base(96.0, Format::M), &p), "2,438 m");
        let constant = Preferences {
            mode_fractions: ModeFractions::Constant,
            ..Preferences::default()
        };
        assert_eq!(
            t(Valeur::en_base(67.5, Format::PiPo), &constant),
            "5 pi 7 8/16 po"
        );
        let quart = Preferences {
            resolution: 4,
            ..Preferences::default()
        };
        assert_eq!(
            t(Valeur::en_base(67.5, Format::PiPo), &quart),
            "5 pi 7 1/2 po"
        );
        let flottant = Preferences {
            metres_fixes: false,
            ..Preferences::default()
        };
        assert_eq!(t(Valeur::en_base(96.0, Format::M), &flottant), "2,4384 m");
    }

    #[test]
    fn vecteurs_19_15_nombres() {
        assert_eq!(nb(1_234_567.891), "1 234 568");
        assert_eq!(nb(0.000_123_456), "0,000123");
        assert_eq!(nb(9.999_999_9), "10");
        assert_eq!(nb(-0.000_000_01), "0");
        assert_eq!(nb(2.0 / 3.0), "0,666667");
        assert_eq!(nb(1_000_000.0 / 3.0), "333 333,3");
        assert_eq!(nb(12_345_678.9), "12 345 679");
        let sans_milliers = Preferences {
            separateur_milliers: false,
            ..Preferences::default()
        };
        assert_eq!(t(Valeur::nombre(1234.5), &sans_milliers), "1234,5");
        let point = Preferences {
            separateur_decimal: SeparateurDecimal::Point,
            ..Preferences::default()
        };
        assert_eq!(t(Valeur::nombre(1234.5), &point), "1 234.5");
    }

    #[test]
    fn exemples_de_la_regle_r22() {
        assert_eq!(nb(14.0 / 3.0), "4,666667");
        assert_eq!(nb(14.0 / 27.0), "0,518519");
        assert_eq!(nb(189.5 / 12.0), "15,79167");
        assert_eq!(nb(1_133.980_925), "1 133,981");
        assert_eq!(nb(13006.4256), "13 006,43");
        assert_eq!(nb(0.1 + 0.2), "0,3");
        assert_eq!(nb(0.0), "0");
        assert_eq!(nb(-5.0), "−5");
    }

    #[test]
    fn vecteurs_19_15_angles_pourcent_monnaie() {
        let p = Preferences::default();
        let sans = |base: f64, format: Format| Valeur {
            base,
            genre: Genre::SansUnite,
            format,
        };
        assert_eq!(t(sans(44.29, Format::Dms), &p), "44°17′24″");
        assert_eq!(t(sans(0.4725, Format::Pourcent), &p), "47,25 %");
        assert_eq!(t(sans(283.0633, Format::Monnaie), &p), "283,06 $");
        assert_eq!(t(sans(36.869_897, Format::Degres), &p), "36,87°");
        assert_eq!(t(sans(45.0, Format::Degres), &p), "45,00°");
        assert_eq!(
            t(sans(19.0 + 1.0 / 60.0 + 53.0 / 3600.0, Format::Dms), &p),
            "19°01′53″"
        );
        assert_eq!(t(sans(0.125, Format::Monnaie), &p), "0,13 $");
    }

    #[test]
    fn rendus_particuliers() {
        let p = Preferences::default();
        let o = OptionsFormat::nouvelles(&p);
        let texte = |v: Valeur, r: Rendu| formater(&v, r, &o).en_texte().replace(INSECABLE, " ");
        assert_eq!(texte(Valeur::nombre(16.0), Rendu::Entier), "16");
        assert_eq!(
            texte(Valeur::nombre(2.004_631), Rendu::DeuxDecimales),
            "2,00"
        );
        assert_eq!(
            texte(
                Valeur::avec_format(100.0 / 3.0, Format::Pi2),
                Rendu::DeuxDecimales
            ),
            "33,33 pi²"
        );
        assert_eq!(texte(Valeur::nombre(169.5), Rendu::Flottant), "169,5");
    }

    #[test]
    fn fraction_negative_seule() {
        let p = Preferences::default();
        assert_eq!(t(Valeur::en_base(-0.3125, Format::PoFrac), &p), "−5/16 po");
        assert_eq!(
            t(Valeur::en_base(45.0 / 64.0, Format::PoFrac), &p),
            "11/16 po"
        );
        let o = OptionsFormat {
            prefs: &p,
            resolution_temp: Some(64),
        };
        assert_eq!(
            en_texte(&Valeur::en_base(45.0 / 64.0, Format::PoFrac), &o).replace(INSECABLE, " "),
            "45/64 po"
        );
    }

    #[test]
    fn en_anglais_le_point_la_virgule_et_les_symboles_anglais() {
        let _anglais = langue::pour_la_portee(Langue::Anglais);
        let p = Preferences::default();
        // Le point décimal s'impose, quelle que soit la préférence.
        assert_eq!(separateur_decimal(&p), '.');
        assert_eq!(t(Valeur::nombre(1234.5), &p), "1,234.5");
        assert_eq!(t(Valeur::nombre(1_234_567.891), &p), "1,234,568");
        assert_eq!(t(Valeur::nombre(-0.5), &p), "−0.5");
        let sans_milliers = Preferences {
            separateur_milliers: false,
            ..Preferences::default()
        };
        assert_eq!(t(Valeur::nombre(1234.5), &sans_milliers), "1234.5");
        assert_eq!(
            t(Valeur::en_base(-63.5, Format::PiPo), &p),
            "−5 ft 3 1/2 in"
        );
        assert_eq!(
            t(Valeur::en_base(15006.0, Format::PiPo), &p),
            "1,250 ft 6 in"
        );
        assert_eq!(t(Valeur::en_base(0.5, Format::PoFrac), &p), "1/2 in");
        assert_eq!(t(Valeur::en_base(96.0, Format::M), &p), "2.438 m");
        assert_eq!(
            t(Valeur::avec_format(1234.5, Format::Pi2), &p),
            "1,234.5 ft²"
        );
        assert_eq!(t(Valeur::avec_format(2.5, Format::Vg3), &p), "2.5 yd³");
        assert_eq!(t(Valeur::avec_format(12.0, Format::Pmp), &p), "12 bd ft");
        assert_eq!(t(Valeur::avec_format(1.5, Format::Tc), &p), "1.5 ton");
        let sans = |base: f64, format: Format| Valeur {
            base,
            genre: Genre::SansUnite,
            format,
        };
        // Le dollar devant, le pourcentage collé.
        assert_eq!(t(sans(1283.0633, Format::Monnaie), &p), "$1,283.06");
        assert_eq!(t(sans(-0.125, Format::Monnaie), &p), "−$0.13");
        assert_eq!(t(sans(0.4725, Format::Pourcent), &p), "47.25%");
        assert_eq!(t(sans(36.869_897, Format::Degres), &p), "36.87°");
        let exp = Preferences {
            exponentielle: true,
            ..Preferences::default()
        };
        assert_eq!(t(Valeur::nombre(2.5e9), &exp), "2.5 E9");
    }

    #[test]
    fn nombre_et_unite_selon_la_langue() {
        assert_eq!(nombre_et_unite("12,5", "pi²"), "12,5\u{a0}pi²");
        assert_eq!(nombre_et_unite("45,00", "°"), "45,00°");
        assert_eq!(
            nombre_et_unite("1\u{a0}234,56", "$"),
            "1\u{a0}234,56\u{a0}$"
        );
        assert_eq!(nombre_et_unite("10", "%"), "10\u{a0}%");
        assert_eq!(nombre_et_unite("12", ""), "12");
        let _anglais = langue::pour_la_portee(Langue::Anglais);
        assert_eq!(nombre_et_unite("12.5", "ft²"), "12.5\u{a0}ft²");
        assert_eq!(nombre_et_unite("1,234.56", "$"), "$1,234.56");
        assert_eq!(nombre_et_unite("−5.00", "$"), "−$5.00");
        assert_eq!(nombre_et_unite("25.00", "$/ft²"), "$25.00/ft²");
        assert_eq!(nombre_et_unite("10", "%"), "10%");
        assert_eq!(nombre_et_unite("8.5", "¢/kWh"), "8.5¢/kWh");
        assert_eq!(nombre_et_unite("5", "°C"), "5\u{a0}°C");
    }

    #[test]
    fn grands_nombres() {
        let p = Preferences::default();
        // 2 × 10⁹ mm passe aux mètres.
        assert_eq!(
            t(Valeur::avec_format(2e9, Format::Mm), &p),
            "2 000 000,000 m"
        );
        assert!(est_affichable(&Valeur::nombre(9.99e11), &p));
        assert!(!est_affichable(&Valeur::nombre(1e12), &p));
        assert!(!est_affichable(&Valeur::nombre(f64::INFINITY), &p));
        let exp = Preferences {
            exponentielle: true,
            ..Preferences::default()
        };
        assert_eq!(t(Valeur::nombre(2.5e9), &exp), "2,5 E9");
        assert_eq!(t(Valeur::nombre(1.2e-7), &exp), "1,2 E−7");
    }
}
