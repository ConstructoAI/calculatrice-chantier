//! Calculateurs par métier (section 22 du cahier des charges).
//!
//! Chaque calculateur est un petit formulaire : des champs à remplir, puis des
//! résultats. Il est décrit par des données (libellés, nature des champs) et
//! par une fonction pure de calcul; l'interface le dessine sans rien savoir de
//! lui. Les libellés des calculateurs sont dans les fichiers de ce dossier
//! (R-52).
//!
//! Règles communes :
//! - une longueur se tape comme partout ailleurs (« 12 pi 6 po », « 30 po »,
//!   « 3,8 m »); un nombre seul prend l'unité indiquée à côté du champ;
//! - les résultats suivent le système de la première longueur saisie : les
//!   longueurs s'affichent en pieds-pouces ou en métrique, les aires, les
//!   volumes et les masses dans les deux systèmes, celui de la saisie en tête;
//! - aucun calculateur ne remplace les plans ni le Code : ils font de la
//!   géométrie et de la physique, pas de la conformité.

pub mod beton;
pub mod electricite;
pub mod finition;
pub mod plomberie;
pub mod terrassement;
pub mod ventilation;

use crate::analyse::{analyser_longueur, analyser_nombre};
use crate::etiquettes::{INSECABLE, symbole};
use crate::format::{OptionsFormat, en_texte, nombre_fixe, nombre_flottant};
use crate::prefs::Preferences;
use crate::unites::{
    EPS, Format, KG_PAR_LB, LB_PAR_T, LB_PAR_TC, PO_PAR_M, PO2_PAR_PI2, PO3_PAR_VG3, Systeme,
};
use crate::valeur::Valeur;

/// Pouces carrés par mètre carré.
pub const PO2_PAR_M2: f64 = PO_PAR_M * PO_PAR_M;
/// Pouces cubes par mètre cube.
pub const PO3_PAR_M3: f64 = PO_PAR_M * PO_PAR_M * PO_PAR_M;
/// Pouces cubes par litre.
pub const PO3_PAR_L: f64 = PO3_PAR_M3 / 1000.0;
/// Pouces cubes par gallon américain (définition exacte).
pub const PO3_PAR_GAL_US: f64 = 231.0;

const MSG_LONGUEUR: &str = "Entrez une longueur, par exemple 12 pi 6 po, 30 po ou 3,8 m.";
const MSG_NOMBRE: &str = "Entrez un nombre.";
const MSG_ENTIER: &str = "Entrez un nombre entier.";
const MSG_POSITIF: &str = "La valeur doit être plus grande que zéro.";
const MSG_NON_NEGATIF: &str = "La valeur ne peut pas être négative.";

/// Corps de métier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Metier {
    Beton,
    Plomberie,
    Electricite,
    Finition,
    Ventilation,
    Terrassement,
}

impl Metier {
    /// Les métiers, dans l'ordre où l'interface les présente.
    pub const TOUS: [Metier; 6] = [
        Metier::Beton,
        Metier::Plomberie,
        Metier::Electricite,
        Metier::Finition,
        Metier::Ventilation,
        Metier::Terrassement,
    ];

    pub fn nom(self) -> &'static str {
        match self {
            Metier::Beton => "Béton et fondations",
            Metier::Plomberie => "Plomberie",
            Metier::Electricite => "Électricité",
            Metier::Finition => "Finition",
            Metier::Ventilation => "Ventilation et chauffage",
            Metier::Terrassement => "Terrassement et aménagement",
        }
    }

    /// Ce qu'on y trouve, en quelques mots.
    pub fn resume(self) -> &'static str {
        match self {
            Metier::Beton => "Dalles, semelles, murs, colonnes, armature",
            Metier::Plomberie => "Pentes, déports, contenances, pression",
            Metier::Electricite => "Loi d'Ohm, chute de tension, puissance, cintrage",
            Metier::Finition => "Peinture, céramique, revêtements de sol",
            Metier::Ventilation => "Débit des conduits, puissance thermique",
            Metier::Terrassement => "Excavation, granulats, clôtures",
        }
    }

    pub fn calculateurs(self) -> &'static [Calculateur] {
        match self {
            Metier::Beton => beton::CALCULATEURS,
            Metier::Plomberie => plomberie::CALCULATEURS,
            Metier::Electricite => electricite::CALCULATEURS,
            Metier::Finition => finition::CALCULATEURS,
            Metier::Ventilation => ventilation::CALCULATEURS,
            Metier::Terrassement => terrassement::CALCULATEURS,
        }
    }
}

/// Retrouve un calculateur par son identifiant.
pub fn calculateur(id: &str) -> Option<&'static Calculateur> {
    Metier::TOUS
        .iter()
        .flat_map(|metier| metier.calculateurs())
        .find(|calculateur| calculateur.id == id)
}

/// Ce qu'un champ attend.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Nature {
    /// Une longueur; le format est celui d'un nombre tapé sans unité.
    Longueur(Format),
    /// Un nombre, avec l'unité à afficher à côté du champ (vide s'il n'y en a pas).
    Nombre(&'static str),
    /// Un nombre entier, avec son unité.
    Entier(&'static str),
    /// Un choix dans une liste; la saisie est le rang de l'option.
    Choix(&'static [&'static str]),
}

/// Un champ de formulaire.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Champ {
    pub libelle: &'static str,
    pub nature: Nature,
    /// Texte proposé au départ (pour un choix : le rang de l'option).
    pub defaut: &'static str,
    /// Un champ facultatif peut rester vide.
    pub facultatif: bool,
    /// Faux : la valeur doit être plus grande que zéro; vrai : zéro est accepté.
    pub nul_permis: bool,
}

impl Champ {
    const fn nouveau(libelle: &'static str, nature: Nature) -> Champ {
        Champ {
            libelle,
            nature,
            defaut: "",
            facultatif: false,
            nul_permis: false,
        }
    }

    pub const fn longueur(libelle: &'static str, unite: Format) -> Champ {
        Champ::nouveau(libelle, Nature::Longueur(unite))
    }

    pub const fn nombre(libelle: &'static str, unite: &'static str) -> Champ {
        Champ::nouveau(libelle, Nature::Nombre(unite))
    }

    pub const fn entier(libelle: &'static str, unite: &'static str) -> Champ {
        Champ::nouveau(libelle, Nature::Entier(unite))
    }

    pub const fn choix(libelle: &'static str, options: &'static [&'static str]) -> Champ {
        let mut champ = Champ::nouveau(libelle, Nature::Choix(options));
        champ.defaut = "0";
        champ
    }

    pub const fn defaut(mut self, texte: &'static str) -> Champ {
        self.defaut = texte;
        self
    }

    pub const fn facultatif(mut self) -> Champ {
        self.facultatif = true;
        self
    }

    pub const fn nul_permis(mut self) -> Champ {
        self.nul_permis = true;
        self
    }

    /// Unité d'un nombre tapé sans unité, à montrer à côté du champ.
    pub fn unite(&self) -> &'static str {
        match self.nature {
            Nature::Longueur(format) => symbole(format),
            Nature::Nombre(unite) | Nature::Entier(unite) => unite,
            Nature::Choix(_) => "",
        }
    }

    fn verifier_le_signe(&self, x: f64) -> Result<(), &'static str> {
        if x < -EPS {
            Err(MSG_NON_NEGATIF)
        } else if x <= EPS && !self.nul_permis {
            Err(MSG_POSITIF)
        } else {
            Ok(())
        }
    }

    /// Lit la saisie d'un champ rempli.
    fn lire(&self, texte: &str, prefs: &Preferences) -> Result<Lecture, &'static str> {
        match self.nature {
            Nature::Longueur(defaut) => {
                let valeur = analyser_longueur(texte, defaut).map_err(|_| MSG_LONGUEUR)?;
                if !valeur.est_longueur() {
                    return Err(MSG_LONGUEUR);
                }
                self.verifier_le_signe(valeur.base)?;
                Ok(Lecture {
                    lue: Lue::Longueur(valeur.base),
                    format: Some(valeur.format),
                    texte: en_texte(&valeur, &OptionsFormat::nouvelles(prefs)),
                })
            }
            Nature::Nombre(unite) => {
                let x = analyser_nombre(texte).map_err(|_| MSG_NOMBRE)?;
                self.verifier_le_signe(x)?;
                Ok(Lecture {
                    lue: Lue::Nombre(x),
                    format: None,
                    texte: avec_unite(nombre_flottant(x, prefs), unite),
                })
            }
            Nature::Entier(unite) => {
                let x = analyser_nombre(texte).map_err(|_| MSG_ENTIER)?;
                if (x - x.round()).abs() > EPS {
                    return Err(MSG_ENTIER);
                }
                self.verifier_le_signe(x)?;
                Ok(Lecture {
                    lue: Lue::Nombre(x.round()),
                    format: None,
                    texte: avec_unite(nombre_fixe(x.round(), 0, prefs), unite),
                })
            }
            Nature::Choix(options) => {
                let rang = texte
                    .parse::<usize>()
                    .ok()
                    .filter(|rang| *rang < options.len())
                    .unwrap_or(0);
                Ok(Lecture {
                    lue: Lue::Choix(rang),
                    format: None,
                    texte: options.get(rang).copied().unwrap_or("").to_string(),
                })
            }
        }
    }
}

/// Saisie d'un champ, une fois lue.
struct Lecture {
    lue: Lue,
    /// Format d'une longueur, tel que l'utilisateur l'a tapée.
    format: Option<Format>,
    /// La saisie remise au propre, pour le ruban.
    texte: String,
}

/// Valeur d'un champ : longueur en pouces, nombre, ou rang d'un choix.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Lue {
    Vide,
    Longueur(f64),
    Nombre(f64),
    Choix(usize),
}

/// Les valeurs des champs d'un calculateur, dans l'ordre des champs.
pub struct Lues<'a>(pub &'a [Lue]);

impl Lues<'_> {
    /// Longueur du champ `i`, en pouces (0 si le champ est vide).
    pub fn longueur(&self, i: usize) -> f64 {
        match self.0.get(i) {
            Some(Lue::Longueur(pouces)) => *pouces,
            _ => 0.0,
        }
    }

    /// Nombre du champ `i` (0 si le champ est vide).
    pub fn nombre(&self, i: usize) -> f64 {
        match self.0.get(i) {
            Some(Lue::Nombre(x)) => *x,
            _ => 0.0,
        }
    }

    /// Rang de l'option choisie au champ `i`.
    pub fn choix(&self, i: usize) -> usize {
        match self.0.get(i) {
            Some(Lue::Choix(rang)) => *rang,
            _ => 0,
        }
    }

    /// Le champ `i` est-il rempli?
    pub fn presente(&self, i: usize) -> bool {
        !matches!(self.0.get(i), None | Some(Lue::Vide))
    }
}

/// Grandeur d'un résultat, avec la façon de l'écrire.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Quantite {
    /// Longueur en pouces : pieds-pouces ou métrique, selon la saisie.
    Longueur(f64),
    /// Aire en pouces carrés : pieds carrés et mètres carrés.
    Aire(f64),
    /// Volume en pouces cubes de béton, de terre ou de pierre : verges cubes et mètres cubes.
    Vrac(f64),
    /// Volume en pouces cubes d'un liquide : litres et gallons américains.
    Liquide(f64),
    /// Masse en livres : livres et kilogrammes.
    Masse(f64),
    /// Masse en livres d'un matériau en vrac : tonnes courtes et tonnes métriques.
    Tonnes(f64),
    /// Nombre avec son unité; le nombre de décimales suit la grandeur.
    Mesure(f64, &'static str),
    /// Nombre à décimales fixes, avec son unité.
    Fixe(f64, usize, &'static str),
    /// Deux écritures de la même grandeur : « 89,7 kPa (13,0 lb/po²) ».
    Double(f64, &'static str, f64, &'static str),
    /// Quantité entière, arrondie vers le haut par le calculateur.
    Compte(f64, &'static str),
}

impl Quantite {
    fn texte(&self, modele: Option<Format>, prefs: &Preferences) -> String {
        let metrique = modele.and_then(Format::systeme) == Some(Systeme::Metrique);
        // Le système de la saisie en tête, l'autre entre parenthèses.
        let deux = |imperial: String, autre: String| {
            if metrique {
                format!("{autre} ({imperial})")
            } else {
                format!("{imperial} ({autre})")
            }
        };
        let utile = |x: f64, unite: &str| avec_unite(nombre_utile(x, prefs), unite);
        match *self {
            Quantite::Longueur(pouces) => texte_longueur(pouces, modele, prefs),
            Quantite::Aire(po2) => deux(
                utile(po2 / PO2_PAR_PI2, symbole(Format::Pi2)),
                utile(po2 / PO2_PAR_M2, symbole(Format::M2)),
            ),
            Quantite::Vrac(po3) => deux(
                utile(po3 / PO3_PAR_VG3, symbole(Format::Vg3)),
                utile(po3 / PO3_PAR_M3, symbole(Format::M3)),
            ),
            Quantite::Liquide(po3) => format!(
                "{} ({})",
                utile(po3 / PO3_PAR_L, "L"),
                utile(po3 / PO3_PAR_GAL_US, "gal US")
            ),
            Quantite::Masse(lb) => deux(
                utile(lb, symbole(Format::Lb)),
                utile(lb * KG_PAR_LB, symbole(Format::Kg)),
            ),
            Quantite::Tonnes(lb) => deux(
                utile(lb / LB_PAR_TC, symbole(Format::Tc)),
                utile(lb / LB_PAR_T, symbole(Format::T)),
            ),
            Quantite::Mesure(x, unite) => utile(x, unite),
            Quantite::Fixe(x, decimales, unite) => {
                avec_unite(nombre_fixe(x, decimales, prefs), unite)
            }
            Quantite::Double(x, unite, y, autre) => {
                format!("{} ({})", utile(x, unite), utile(y, autre))
            }
            Quantite::Compte(n, unite) => avec_unite(nombre_fixe(n, 0, prefs), unite),
        }
    }
}

/// Un résultat : son libellé et sa grandeur.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Resultat {
    pub libelle: &'static str,
    pub quantite: Quantite,
}

impl Resultat {
    pub const fn nouveau(libelle: &'static str, quantite: Quantite) -> Resultat {
        Resultat { libelle, quantite }
    }
}

/// Ce que rend la fonction de calcul : des résultats, ou la raison pour
/// laquelle les valeurs saisies ne se prêtent pas au calcul.
pub type Calcul = Result<Vec<Resultat>, &'static str>;

/// Un calculateur : sa description et sa fonction de calcul.
pub struct Calculateur {
    /// Identifiant stable, pour retrouver les dernières saisies.
    pub id: &'static str,
    pub metier: Metier,
    pub nom: &'static str,
    /// À quoi il sert, en une phrase.
    pub resume: &'static str,
    pub champs: &'static [Champ],
    /// Précision ou mise en garde montrée sous les résultats (vide s'il n'y en a pas).
    pub note: &'static str,
    calcul: fn(&Lues) -> Calcul,
}

/// Ce que donne un formulaire, tel qu'il est rempli.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Evaluation {
    /// Pour chaque champ : le message à montrer si la saisie est refusée.
    pub erreurs: Vec<Option<&'static str>>,
    /// Message général, quand les valeurs ne se prêtent pas au calcul.
    pub avis: Option<&'static str>,
    /// Les champs remplis, remis au propre : (libellé, valeur).
    pub entrees: Vec<(&'static str, String)>,
    /// Les résultats : (libellé, valeur). Vide tant que le formulaire est incomplet.
    pub resultats: Vec<(&'static str, String)>,
}

impl Calculateur {
    /// Les textes proposés à l'ouverture du formulaire.
    pub fn saisies_par_defaut(&self) -> Vec<String> {
        self.champs
            .iter()
            .map(|champ| champ.defaut.to_string())
            .collect()
    }

    /// Lit les saisies et calcule les résultats.
    pub fn evaluer(&self, saisies: &[String], prefs: &Preferences) -> Evaluation {
        let mut evaluation = Evaluation {
            erreurs: vec![None; self.champs.len()],
            ..Evaluation::default()
        };
        let mut lues = Vec::with_capacity(self.champs.len());
        // Format de la première longueur saisie : il donne le ton des résultats.
        let mut modele = None;
        let mut complet = true;
        for (i, champ) in self.champs.iter().enumerate() {
            let texte = saisies.get(i).map_or("", |saisie| saisie.trim());
            let choix = matches!(champ.nature, Nature::Choix(_));
            if texte.is_empty() && !choix {
                complet &= champ.facultatif;
                lues.push(Lue::Vide);
                continue;
            }
            match champ.lire(texte, prefs) {
                Ok(lecture) => {
                    if modele.is_none() {
                        modele = lecture.format;
                    }
                    evaluation.entrees.push((champ.libelle, lecture.texte));
                    lues.push(lecture.lue);
                }
                Err(message) => {
                    evaluation.erreurs[i] = Some(message);
                    complet = false;
                    lues.push(Lue::Vide);
                }
            }
        }
        if !complet {
            return evaluation;
        }
        match (self.calcul)(&Lues(&lues)) {
            Ok(resultats) => {
                evaluation.resultats = resultats
                    .iter()
                    .filter(|resultat| resultat.quantite.est_finie())
                    .map(|resultat| (resultat.libelle, resultat.quantite.texte(modele, prefs)))
                    .collect();
            }
            Err(avis) => evaluation.avis = Some(avis),
        }
        evaluation
    }
}

impl Quantite {
    /// Faux si le calcul a débordé : le résultat n'est alors pas montré.
    fn est_finie(&self) -> bool {
        match *self {
            Quantite::Longueur(x)
            | Quantite::Aire(x)
            | Quantite::Vrac(x)
            | Quantite::Liquide(x)
            | Quantite::Masse(x)
            | Quantite::Tonnes(x)
            | Quantite::Mesure(x, _)
            | Quantite::Fixe(x, _, _)
            | Quantite::Compte(x, _) => x.is_finite(),
            Quantite::Double(x, _, y, _) => x.is_finite() && y.is_finite(),
        }
    }
}

/// Un nombre et son unité, séparés par une espace insécable.
fn avec_unite(nombre: String, unite: &str) -> String {
    if unite.is_empty() {
        nombre
    } else {
        format!("{nombre}{INSECABLE}{unite}")
    }
}

/// Écrit un nombre avec les décimales utiles sur un chantier : aucune à partir
/// de 100, une à partir de 10, deux à partir de 1, trois en dessous.
pub fn nombre_utile(x: f64, prefs: &Preferences) -> String {
    let a = x.abs();
    let decimales = if a >= 99.95 {
        0
    } else if a >= 9.995 {
        1
    } else if a >= 0.9995 {
        2
    } else {
        3
    };
    nombre_fixe(x, decimales, prefs)
}

/// Écrit une longueur : dans l'unité métrique de la saisie, au millimètre
/// près, ou en pieds-pouces (en pouces seulement sous un pied).
fn texte_longueur(pouces: f64, modele: Option<Format>, prefs: &Preferences) -> String {
    let metrique = |format: Format, decimales: usize| {
        avec_unite(
            nombre_fixe(pouces / format.facteur(), decimales, prefs),
            symbole(format),
        )
    };
    match modele {
        Some(Format::M) => metrique(Format::M, 3),
        Some(Format::Cm) => metrique(Format::Cm, 1),
        Some(Format::Mm) => metrique(Format::Mm, 0),
        _ => {
            let format = if pouces.abs() < 12.0 - EPS {
                Format::PoFrac
            } else {
                Format::PiPo
            };
            en_texte(
                &Valeur::en_base(pouces, format),
                &OptionsFormat::nouvelles(prefs),
            )
        }
    }
}

/// Arrondit vers le haut une quantité à acheter (44,0000001 reste 44).
pub fn arrondi_au_dessus(x: f64) -> f64 {
    (x - 1e-9).ceil().max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn saisies(textes: &[&str]) -> Vec<String> {
        textes.iter().map(|t| (*t).to_string()).collect()
    }

    #[test]
    fn identifiants_uniques_et_retrouvables() {
        let mut vus = std::collections::BTreeSet::new();
        let mut total = 0;
        for metier in Metier::TOUS {
            assert!(!metier.calculateurs().is_empty(), "{}", metier.nom());
            for c in metier.calculateurs() {
                total += 1;
                assert_eq!(c.metier, metier, "{}", c.id);
                assert!(vus.insert(c.id), "identifiant en double : {}", c.id);
                assert!(calculateur(c.id).is_some());
                assert!(!c.nom.is_empty() && !c.resume.is_empty() && !c.champs.is_empty());
            }
        }
        assert_eq!(total, 25);
        assert!(calculateur("inconnu").is_none());
    }

    #[test]
    fn valeurs_par_defaut_lisibles() {
        // Les valeurs proposées au départ doivent toutes être acceptées.
        let prefs = Preferences::default();
        for metier in Metier::TOUS {
            for c in metier.calculateurs() {
                let e = c.evaluer(&c.saisies_par_defaut(), &prefs);
                assert!(
                    e.erreurs.iter().all(Option::is_none),
                    "{} : {:?}",
                    c.id,
                    e.erreurs
                );
                for champ in c.champs {
                    if let Nature::Choix(options) = champ.nature {
                        let rang: usize = champ.defaut.parse().unwrap();
                        assert!(rang < options.len(), "{} : {}", c.id, champ.libelle);
                    }
                }
            }
        }
    }

    #[test]
    fn formulaire_incomplet_sans_resultat_ni_reproche() {
        let prefs = Preferences::default();
        let c = calculateur("dalle").unwrap();
        let e = c.evaluer(&saisies(&["24", "", "4", "10"]), &prefs);
        assert!(e.resultats.is_empty());
        assert!(e.erreurs.iter().all(Option::is_none));
        assert_eq!(e.avis, None);
        // Les champs déjà remplis sont remis au propre.
        assert_eq!(
            e.entrees[0],
            ("Longueur", "24\u{a0}pi\u{a0}0\u{a0}po".to_string())
        );
    }

    #[test]
    fn saisies_refusees() {
        let prefs = Preferences::default();
        let c = calculateur("dalle").unwrap();
        let e = c.evaluer(&saisies(&["abc", "20", "0", "-5"]), &prefs);
        assert_eq!(e.erreurs[0], Some(MSG_LONGUEUR));
        assert_eq!(e.erreurs[1], None);
        assert_eq!(e.erreurs[2], Some(MSG_POSITIF));
        assert_eq!(e.erreurs[3], Some(MSG_NON_NEGATIF));
        assert!(e.resultats.is_empty());
        // Un nombre entier est exigé là où il le faut.
        let c = calculateur("colonnes").unwrap();
        let e = c.evaluer(&saisies(&["10", "4", "2,5", "10", "14"]), &prefs);
        assert_eq!(e.erreurs[2], Some(MSG_ENTIER));
    }

    #[test]
    fn decimales_utiles() {
        let prefs = Preferences::default();
        assert_eq!(nombre_utile(1234.56, &prefs), "1\u{a0}235");
        assert_eq!(nombre_utile(99.96, &prefs), "100");
        assert_eq!(nombre_utile(44.5935, &prefs), "44,6");
        assert_eq!(nombre_utile(5.9259, &prefs), "5,93");
        assert_eq!(nombre_utile(0.4848, &prefs), "0,485");
        assert_eq!(nombre_utile(0.0, &prefs), "0,000");
    }

    #[test]
    fn longueurs_selon_la_saisie() {
        let prefs = Preferences::default();
        assert_eq!(texte_longueur(10.0, None, &prefs), "10\u{a0}po");
        assert_eq!(
            texte_longueur(17.0, Some(Format::PiPo), &prefs),
            "1\u{a0}pi\u{a0}5\u{a0}po"
        );
        assert_eq!(
            texte_longueur(PO_PAR_M / 4.0, Some(Format::M), &prefs),
            "0,250\u{a0}m"
        );
        assert_eq!(
            texte_longueur(PO_PAR_M / 4.0, Some(Format::Mm), &prefs),
            "250\u{a0}mm"
        );
    }

    #[test]
    fn arrondi_des_quantites_a_acheter() {
        assert_eq!(arrondi_au_dessus(44.000_000_000_1), 44.0);
        assert_eq!(arrondi_au_dessus(44.01), 45.0);
        assert_eq!(arrondi_au_dessus(0.0), 0.0);
    }
}
