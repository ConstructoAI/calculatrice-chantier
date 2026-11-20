//! Calculateurs par métier (section 22 du cahier des charges).
//!
//! Chaque calculateur est un petit formulaire : des champs à remplir, puis des
//! résultats. Il est décrit par des données (libellés, nature des champs) et
//! par une fonction pure de calcul; l'interface le dessine sans rien savoir de
//! lui. Les libellés des calculateurs sont dans les fichiers de ce dossier
//! (R-52), chacun en français et en anglais (section 24).
//!
//! Règles communes :
//! - une longueur se tape comme partout ailleurs (« 12 pi 6 po », « 30 po »,
//!   « 3,8 m »); un nombre seul prend l'unité indiquée à côté du champ;
//! - les résultats suivent le système de la première longueur saisie : les
//!   longueurs s'affichent en pieds-pouces ou en métrique, les aires, les
//!   volumes et les masses dans les deux systèmes, celui de la saisie en tête;
//! - aucun calculateur ne remplace les plans ni le Code : ils font de la
//!   géométrie et de la physique, pas de la conformité.

pub mod acier;
pub mod beton;
pub mod electricite;
pub mod estimation;
pub mod finition;
pub mod interieurs;
pub mod maconnerie;
pub mod plomberie;
pub mod terrassement;
pub mod toiture;
pub mod ventilation;

use crate::analyse::{analyser_longueur, analyser_nombre, transposer};
use crate::etiquettes::symbole;
use crate::format::{
    OptionsFormat, en_texte, nombre_et_unite, nombre_fixe, nombre_flottant, separateur_decimal_en,
};
use crate::langue::{self, Langue, Texte, selon, t};
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

const MSG_LONGUEUR: Texte = t(
    "Entrez une longueur, par exemple 12 pi 6 po, 30 po ou 3,8 m.",
    "Enter a length, for example 12 ft 6 in, 30 in or 3.8 m.",
);
const MSG_NOMBRE: Texte = t("Entrez un nombre.", "Enter a number.");
const MSG_ENTIER: Texte = t("Entrez un nombre entier.", "Enter a whole number.");
const MSG_POSITIF: Texte = t(
    "La valeur doit être plus grande que zéro.",
    "The value must be greater than zero.",
);
const MSG_NON_NEGATIF: Texte = t(
    "La valeur ne peut pas être négative.",
    "The value cannot be negative.",
);

/// Corps de métier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Metier {
    Estimation,
    Terrassement,
    Beton,
    Maconnerie,
    Toiture,
    /// Systèmes intérieurs : gypse, cloisons, isolant, plafonds.
    Interieurs,
    Finition,
    Plomberie,
    Electricite,
    Ventilation,
    Acier,
}

impl Metier {
    /// Les métiers, dans l'ordre où l'interface les présente.
    pub const TOUS: [Metier; 11] = [
        Metier::Estimation,
        Metier::Terrassement,
        Metier::Beton,
        Metier::Maconnerie,
        Metier::Toiture,
        Metier::Interieurs,
        Metier::Finition,
        Metier::Plomberie,
        Metier::Electricite,
        Metier::Ventilation,
        Metier::Acier,
    ];

    /// Nom du métier, dans la langue en cours.
    pub fn nom(self) -> &'static str {
        match self {
            Metier::Estimation => selon("Estimation et soumission", "Estimating and bidding"),
            Metier::Terrassement => {
                selon("Terrassement et aménagement", "Earthwork and landscaping")
            }
            Metier::Beton => selon("Béton et fondations", "Concrete and foundations"),
            Metier::Maconnerie => selon("Maçonnerie et pavage", "Masonry and paving"),
            Metier::Toiture => selon("Toiture et revêtement", "Roofing and siding"),
            Metier::Interieurs => selon("Gypse et isolation", "Drywall and insulation"),
            Metier::Finition => selon("Finition", "Finishing"),
            Metier::Plomberie => selon("Plomberie", "Plumbing"),
            Metier::Electricite => selon("Électricité", "Electrical"),
            Metier::Ventilation => selon("Ventilation et chauffage", "Ventilation and heating"),
            Metier::Acier => selon("Acier et soudure", "Steel and welding"),
        }
    }

    /// Nom en un mot, pour la barre des métiers (section 23).
    pub fn nom_court(self) -> &'static str {
        match self {
            Metier::Estimation => selon("Estimation", "Estimating"),
            Metier::Terrassement => selon("Terrassement", "Earthwork"),
            Metier::Beton => selon("Béton", "Concrete"),
            Metier::Maconnerie => selon("Maçonnerie", "Masonry"),
            Metier::Toiture => selon("Toiture", "Roofing"),
            Metier::Interieurs => selon("Gypse", "Drywall"),
            Metier::Finition => selon("Finition", "Finishing"),
            Metier::Plomberie => selon("Plomberie", "Plumbing"),
            Metier::Electricite => selon("Électricité", "Electrical"),
            Metier::Ventilation => selon("Ventilation", "HVAC"),
            Metier::Acier => selon("Acier", "Steel"),
        }
    }

    /// Ce qu'on y trouve, en quelques mots.
    pub fn resume(self) -> &'static str {
        match self {
            Metier::Estimation => selon(
                "Soumission, budget au pied carré, main-d'œuvre, taxes, facture d'étape",
                "Bid price, budget per square foot, labour, taxes, progress invoice",
            ),
            Metier::Terrassement => selon(
                "Excavation, tranchées, granulats, pentes, clôtures, terrasses",
                "Excavation, trenches, aggregate, slopes, fences, decks",
            ),
            Metier::Beton => selon(
                "Dalles, semelles, murs, colonnes, escaliers, armature, sacs",
                "Slabs, footings, walls, columns, stairs, rebar, bags",
            ),
            Metier::Maconnerie => selon(
                "Briques, blocs, rangs, murets, pavés",
                "Bricks, blocks, courses, low walls, pavers",
            ),
            Metier::Toiture => selon(
                "Bardeaux, pentes, panneaux, revêtement extérieur, membranes",
                "Shingles, pitch, sheathing, siding, membranes",
            ),
            Metier::Interieurs => selon(
                "Panneaux de gypse, cloisons, isolant, valeur R, plafonds",
                "Drywall sheets, partitions, insulation, R-value, ceilings",
            ),
            Metier::Finition => selon(
                "Peinture, céramique, planchers, plinthes et moulures",
                "Paint, tile, flooring, baseboards and mouldings",
            ),
            Metier::Plomberie => selon(
                "Pentes, déports, coupes, débit, dilatation, contenances",
                "Slopes, offsets, cut lengths, flow, expansion, capacities",
            ),
            Metier::Electricite => selon(
                "Loi d'Ohm, chute de tension, calibre, moteurs, transformateurs, conduits",
                "Ohm's law, voltage drop, wire size, motors, transformers, conduit",
            ),
            Metier::Ventilation => selon(
                "Conduits, changements d'air, puissance, plancher chauffant",
                "Ducts, air changes, capacity, radiant floor",
            ),
            Metier::Acier => selon(
                "Masse des plaques, des barres et des tubes, cordons de soudure",
                "Weight of plates, bars and tubes, fillet welds",
            ),
        }
    }

    pub fn calculateurs(self) -> &'static [Calculateur] {
        match self {
            Metier::Estimation => estimation::CALCULATEURS,
            Metier::Terrassement => terrassement::CALCULATEURS,
            Metier::Beton => beton::CALCULATEURS,
            Metier::Maconnerie => maconnerie::CALCULATEURS,
            Metier::Toiture => toiture::CALCULATEURS,
            Metier::Interieurs => interieurs::CALCULATEURS,
            Metier::Finition => finition::CALCULATEURS,
            Metier::Plomberie => plomberie::CALCULATEURS,
            Metier::Electricite => electricite::CALCULATEURS,
            Metier::Ventilation => ventilation::CALCULATEURS,
            Metier::Acier => acier::CALCULATEURS,
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
    Nombre(Texte),
    /// Un nombre entier, avec son unité.
    Entier(Texte),
    /// Un choix dans une liste; la saisie est le rang de l'option.
    Choix(&'static [Texte]),
}

/// Un champ de formulaire.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Champ {
    pub libelle: Texte,
    /// Libellé abrégé pour une touche du clavier; vide : le libellé convient.
    pub court: Texte,
    pub nature: Nature,
    /// Texte proposé au départ, écrit à la française (pour un choix : le rang
    /// de l'option). `depart` le donne tel qu'il se tape dans la langue en cours.
    pub defaut: &'static str,
    /// Un champ facultatif peut rester vide.
    pub facultatif: bool,
    /// Faux : la valeur doit être plus grande que zéro; vrai : zéro est accepté.
    pub nul_permis: bool,
}

impl Champ {
    const fn nouveau(libelle: Texte, nature: Nature) -> Champ {
        Champ {
            libelle,
            court: Texte::VIDE,
            nature,
            defaut: "",
            facultatif: false,
            nul_permis: false,
        }
    }

    pub const fn longueur(libelle: Texte, unite: Format) -> Champ {
        Champ::nouveau(libelle, Nature::Longueur(unite))
    }

    pub const fn nombre(libelle: Texte, unite: Texte) -> Champ {
        Champ::nouveau(libelle, Nature::Nombre(unite))
    }

    pub const fn entier(libelle: Texte, unite: Texte) -> Champ {
        Champ::nouveau(libelle, Nature::Entier(unite))
    }

    pub const fn choix(libelle: Texte, options: &'static [Texte]) -> Champ {
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

    /// Donne au champ un libellé abrégé, pour une touche étroite du clavier
    /// (section 23) : deux lignes au plus, de douze caractères au plus. Il peut
    /// n'exister que dans une langue : `t("", "Tread")`.
    pub const fn court(mut self, libelle: Texte) -> Champ {
        self.court = libelle;
        self
    }

    /// Libellé abrégé, pour tenir sur une touche du clavier (section 23), dans
    /// la langue en cours.
    pub fn libelle_court(&self) -> &'static str {
        let court = self.court.traduit();
        if court.is_empty() {
            self.libelle.traduit()
        } else {
            court
        }
    }

    /// Le libellé à écrire dans un message de l'afficheur : le libellé complet
    /// s'il est bref, sinon le libellé court (un saut de ligne y sépare deux mots).
    pub fn libelle_pour_message(&self) -> &'static str {
        let libelle = self.libelle.traduit();
        if libelle.chars().count() <= 24 {
            libelle
        } else {
            self.libelle_court()
        }
    }

    /// Unité d'un nombre tapé sans unité, à montrer à côté du champ.
    pub fn unite(&self) -> &'static str {
        match self.nature {
            Nature::Longueur(format) => symbole(format),
            Nature::Nombre(unite) | Nature::Entier(unite) => unite.traduit(),
            Nature::Choix(_) => "",
        }
    }

    /// Le texte proposé au départ, tel qu'il se tape dans la langue en cours :
    /// « 9,975 » en français, « 9.975 » en anglais.
    pub fn depart(&self) -> String {
        match (langue::en_cours(), self.nature) {
            (Langue::Francais, _) | (_, Nature::Choix(_)) => self.defaut.to_string(),
            (Langue::Anglais, _) => transposer(self.defaut, Langue::Francais, Langue::Anglais, '.'),
        }
    }

    /// Récrit une saisie de ce champ comme elle se tape dans une autre langue
    /// (section 24) : la valeur reste la même.
    pub fn transposer(&self, texte: &str, de: Langue, vers: Langue, prefs: &Preferences) -> String {
        if de == vers || matches!(self.nature, Nature::Choix(_)) {
            return texte.to_string();
        }
        transposer(texte, de, vers, separateur_decimal_en(prefs, vers))
    }

    fn verifier_le_signe(&self, x: f64) -> Result<(), Texte> {
        if x < -EPS {
            Err(MSG_NON_NEGATIF)
        } else if x <= EPS && !self.nul_permis {
            Err(MSG_POSITIF)
        } else {
            Ok(())
        }
    }

    /// Lit la saisie d'un champ rempli.
    pub(crate) fn lire(&self, texte: &str, prefs: &Preferences) -> Result<Lecture, Texte> {
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
                    texte: nombre_et_unite(&nombre_flottant(x, prefs), unite.traduit()),
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
                    texte: nombre_et_unite(&nombre_fixe(x.round(), 0, prefs), unite.traduit()),
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
                    texte: options.get(rang).map_or("", |o| o.traduit()).to_string(),
                })
            }
        }
    }
}

/// Saisie d'un champ, une fois lue.
pub(crate) struct Lecture {
    pub(crate) lue: Lue,
    /// Format d'une longueur, tel que l'utilisateur l'a tapée.
    pub(crate) format: Option<Format>,
    /// La saisie remise au propre, pour le ruban.
    pub(crate) texte: String,
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
    Mesure(f64, Texte),
    /// Nombre à décimales fixes, avec son unité.
    Fixe(f64, usize, Texte),
    /// Deux écritures de la même grandeur : « 89,7 kPa (13,0 lb/po²) ».
    Double(f64, Texte, f64, Texte),
    /// Quantité entière, arrondie vers le haut par le calculateur.
    Compte(f64, Texte),
    /// Un mot plutôt qu'un nombre : le calibre d'un conducteur, par exemple.
    Mot(Texte),
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
        let utile = |x: f64, unite: &str| nombre_et_unite(&nombre_utile(x, prefs), unite);
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
                utile(po3 / PO3_PAR_GAL_US, selon("gal US", "US gal"))
            ),
            Quantite::Masse(lb) => deux(
                utile(lb, symbole(Format::Lb)),
                utile(lb * KG_PAR_LB, symbole(Format::Kg)),
            ),
            Quantite::Tonnes(lb) => deux(
                utile(lb / LB_PAR_TC, symbole(Format::Tc)),
                utile(lb / LB_PAR_T, symbole(Format::T)),
            ),
            Quantite::Mesure(x, unite) => utile(x, unite.traduit()),
            Quantite::Fixe(x, decimales, unite) => {
                nombre_et_unite(&nombre_fixe(x, decimales, prefs), unite.traduit())
            }
            Quantite::Double(x, unite, y, autre) => {
                format!(
                    "{} ({})",
                    utile(x, unite.traduit()),
                    utile(y, autre.traduit())
                )
            }
            Quantite::Compte(n, unite) => {
                nombre_et_unite(&nombre_fixe(n, 0, prefs), unite.traduit())
            }
            Quantite::Mot(mot) => mot.traduit().to_string(),
        }
    }
}

/// Un résultat : son libellé et sa grandeur.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Resultat {
    pub libelle: Texte,
    pub quantite: Quantite,
}

impl Resultat {
    pub const fn nouveau(libelle: Texte, quantite: Quantite) -> Resultat {
        Resultat { libelle, quantite }
    }
}

/// Ce que rend la fonction de calcul : des résultats, ou la raison pour
/// laquelle les valeurs saisies ne se prêtent pas au calcul.
pub type Calcul = Result<Vec<Resultat>, Texte>;

/// Un calculateur : sa description et sa fonction de calcul.
pub struct Calculateur {
    /// Identifiant stable, pour retrouver les dernières saisies.
    pub id: &'static str,
    pub metier: Metier,
    pub nom: Texte,
    /// À quoi il sert, en une phrase.
    pub resume: Texte,
    pub champs: &'static [Champ],
    /// Précision ou mise en garde montrée sous les résultats (vide s'il n'y en a pas).
    pub note: Texte,
    calcul: fn(&Lues) -> Calcul,
}

/// Ce que donne un formulaire, tel qu'il est rempli. Les textes sont dans la
/// langue en cours au moment du calcul.
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
    /// La grandeur de chaque résultat, dans le même ordre.
    pub quantites: Vec<Quantite>,
    /// Format de la première longueur saisie : il donne le ton des résultats.
    pub modele: Option<Format>,
}

impl Calculateur {
    /// Les textes proposés à l'ouverture du formulaire, tels qu'ils se tapent
    /// dans la langue en cours.
    pub fn saisies_par_defaut(&self) -> Vec<String> {
        self.champs.iter().map(Champ::depart).collect()
    }

    /// Récrit des saisies comme elles se tapent dans une autre langue
    /// (section 24) : les valeurs restent les mêmes.
    pub fn transposer_les_saisies(
        &self,
        saisies: &mut [String],
        de: Langue,
        vers: Langue,
        prefs: &Preferences,
    ) {
        for (champ, saisie) in self.champs.iter().zip(saisies) {
            *saisie = champ.transposer(saisie, de, vers, prefs);
        }
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
                    evaluation
                        .entrees
                        .push((champ.libelle.traduit(), lecture.texte));
                    lues.push(lecture.lue);
                }
                Err(message) => {
                    evaluation.erreurs[i] = Some(message.traduit());
                    complet = false;
                    lues.push(Lue::Vide);
                }
            }
        }
        if !complet {
            return evaluation;
        }
        evaluation.modele = modele;
        match (self.calcul)(&Lues(&lues)) {
            Ok(resultats) => {
                for resultat in resultats
                    .iter()
                    .filter(|resultat| resultat.quantite.est_finie())
                {
                    evaluation.resultats.push((
                        resultat.libelle.traduit(),
                        resultat.quantite.texte(modele, prefs),
                    ));
                    evaluation.quantites.push(resultat.quantite);
                }
            }
            Err(avis) => evaluation.avis = Some(avis.traduit()),
        }
        evaluation
    }
}

impl Quantite {
    /// La grandeur comme valeur de la calculatrice, pour poursuivre un calcul
    /// au clavier (section 23).
    pub fn valeur(&self, modele: Option<Format>) -> Valeur {
        let metrique = modele.and_then(Format::systeme) == Some(Systeme::Metrique);
        let selon = |imperial: Format, autre: Format| if metrique { autre } else { imperial };
        match *self {
            Quantite::Longueur(pouces) => {
                let format = match modele {
                    Some(format) if metrique => format,
                    _ if pouces.abs() < 12.0 - EPS => Format::PoFrac,
                    _ => Format::PiPo,
                };
                Valeur::en_base(pouces, format)
            }
            Quantite::Aire(po2) => Valeur::en_base(po2, selon(Format::Pi2, Format::M2)),
            Quantite::Vrac(po3) => Valeur::en_base(po3, selon(Format::Vg3, Format::M3)),
            Quantite::Masse(lb) => Valeur::en_base(lb, selon(Format::Lb, Format::Kg)),
            Quantite::Tonnes(lb) => Valeur::en_base(lb, selon(Format::Tc, Format::T)),
            Quantite::Liquide(po3) => Valeur::nombre(po3 / PO3_PAR_L),
            Quantite::Mesure(x, _)
            | Quantite::Fixe(x, _, _)
            | Quantite::Compte(x, _)
            | Quantite::Double(x, _, _, _) => Valeur::nombre(x),
            // Un mot n'a pas de valeur : rien à reprendre dans un calcul.
            Quantite::Mot(_) => Valeur::nombre(0.0),
        }
    }

    /// Texte à montrer tel quel à l'afficheur; `None` pour une longueur en
    /// pieds-pouces, que l'afficheur écrit lui-même avec ses fractions.
    pub fn texte_libre(&self, modele: Option<Format>, prefs: &Preferences) -> Option<String> {
        let metrique = modele.and_then(Format::systeme) == Some(Systeme::Metrique);
        match self {
            Quantite::Longueur(_) if !metrique => None,
            _ => Some(self.texte(modele, prefs)),
        }
    }

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
            Quantite::Mot(_) => true,
        }
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
        nombre_et_unite(
            &nombre_fixe(pouces / format.facteur(), decimales, prefs),
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
                assert!(!c.champs.is_empty(), "{}", c.id);
                for texte in [c.nom, c.resume] {
                    assert!(!texte.fr.is_empty() && !texte.en.is_empty(), "{}", c.id);
                }
                // Une note existe dans les deux langues, ou dans aucune.
                assert_eq!(c.note.fr.is_empty(), c.note.en.is_empty(), "{}", c.id);
            }
        }
        assert_eq!(total, 76);
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
                        assert!(rang < options.len(), "{} : {}", c.id, champ.libelle.fr);
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
        assert_eq!(e.erreurs[0], Some(MSG_LONGUEUR.fr));
        assert_eq!(e.erreurs[1], None);
        assert_eq!(e.erreurs[2], Some(MSG_POSITIF.fr));
        assert_eq!(e.erreurs[3], Some(MSG_NON_NEGATIF.fr));
        assert!(e.resultats.is_empty());
        // Un nombre entier est exigé là où il le faut.
        let c = calculateur("colonnes").unwrap();
        let e = c.evaluer(&saisies(&["10", "4", "2,5", "10", "14"]), &prefs);
        assert_eq!(e.erreurs[2], Some(MSG_ENTIER.fr));
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
