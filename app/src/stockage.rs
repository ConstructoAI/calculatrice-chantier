//! Lecture et écriture des données conservées (section 17).
//!
//! Ce module ne sait pas où vont les données : un dossier de fichiers sur le
//! bureau, le stockage local dans un navigateur. C'est l'affaire du `Depot` de
//! la plateforme.

use std::collections::BTreeMap;
use std::io;

use chrono::{Datelike, Timelike};
use noyau::dessin::{Dessin, Plan};
use noyau::langue::Langue;
use noyau::{EtatPersistant, LigneRuban};
use serde::{Deserialize, Serialize};

use crate::plateforme::{Depot, JSON_LISIBLE, Lecture};
use crate::textes;

const FICHIER_ETAT: &str = "etat.json";
const FICHIER_ETAT_CORROMPU: &str = "etat.corrompu.json";
const FICHIER_RUBANS: &str = "rubans.json";
const FICHIER_RUBANS_CORROMPU: &str = "rubans.corrompu.json";
/// Version du format de `rubans.json`.
const VERSION_RUBANS: u32 = 1;
const FICHIER_SAISIES: &str = "metiers.json";
/// Version du format de `metiers.json`.
const VERSION_SAISIES: u32 = 1;
/// Nombre maximal de rubans enregistrés (annexe A.2).
pub const MAX_RUBANS: usize = 200;

/// Met une valeur en JSON : lisible sur le bureau, compact dans un navigateur.
fn en_json<T: Serialize>(valeur: &T) -> io::Result<String> {
    let json = if JSON_LISIBLE {
        serde_json::to_string_pretty(valeur)
    } else {
        serde_json::to_string(valeur)
    };
    json.map_err(io::Error::other)
}

/// Résultat de la lecture de `etat.json`.
pub struct Chargement {
    pub etat: EtatPersistant,
    /// La donnée existait mais n'a pas pu être lue : elle a été mise de côté (R-46).
    pub illisible: bool,
    /// Rien n'avait encore été enregistré : c'est le premier lancement.
    pub premiere_fois: bool,
}

/// Lit l'état conservé. Donnée absente : valeurs par défaut. Donnée illisible
/// ou d'une version inconnue : mise de côté, valeurs par défaut (R-46).
pub fn charger_etat(depot: &Depot) -> Chargement {
    let par_defaut = |illisible: bool| {
        if illisible {
            depot.mettre_de_cote(FICHIER_ETAT, FICHIER_ETAT_CORROMPU);
        }
        Chargement {
            etat: EtatPersistant::default(),
            illisible,
            premiere_fois: !illisible,
        }
    };
    let texte = match depot.lire(FICHIER_ETAT) {
        Lecture::Texte(texte) => texte,
        Lecture::Absent => return par_defaut(false),
        Lecture::Illisible => return par_defaut(true),
    };
    match serde_json::from_str::<EtatPersistant>(&texte) {
        Ok(etat) if etat.version_connue() => Chargement {
            etat,
            illisible: false,
            premiere_fois: false,
        },
        _ => par_defaut(true),
    }
}

/// Enregistre l'état.
pub fn enregistrer_etat(depot: &Depot, etat: &EtatPersistant) -> io::Result<()> {
    depot.ecrire(FICHIER_ETAT, &en_json(etat)?)
}

/// Un ruban enregistré (section 14.3).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct RubanEnregistre {
    pub id: u64,
    pub nom: String,
    /// Date d'enregistrement, ISO 8601.
    pub date: String,
    pub lignes: Vec<LigneRuban>,
}

impl RubanEnregistre {
    /// Date en clair : « 19 novembre 2026, 8 h 04 »; la date brute si elle est illisible.
    pub fn date_en_clair(&self) -> String {
        match chrono::DateTime::parse_from_rfc3339(&self.date) {
            Ok(d) => textes::date_en_clair(d.year(), d.month(), d.day(), d.hour(), d.minute()),
            Err(_) => self.date.clone(),
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct FichierRubans {
    version: u32,
    rubans: Vec<RubanEnregistre>,
}

/// Lit les rubans enregistrés; liste vide s'il n'y en a pas. Une liste
/// illisible ou d'une version inconnue est mise de côté.
pub fn charger_rubans(depot: &Depot) -> Vec<RubanEnregistre> {
    let lus = match depot.lire(FICHIER_RUBANS) {
        Lecture::Absent => return Vec::new(),
        Lecture::Illisible => None,
        Lecture::Texte(texte) => serde_json::from_str::<FichierRubans>(&texte)
            .ok()
            .filter(|fichier| fichier.version == VERSION_RUBANS),
    };
    match lus {
        Some(fichier) => fichier.rubans,
        None => {
            depot.mettre_de_cote(FICHIER_RUBANS, FICHIER_RUBANS_CORROMPU);
            Vec::new()
        }
    }
}

/// Enregistre les rubans, du plus récent au plus ancien.
pub fn enregistrer_rubans(depot: &Depot, rubans: &[RubanEnregistre]) -> io::Result<()> {
    let fichier = FichierRubans {
        version: VERSION_RUBANS,
        rubans: rubans.to_vec(),
    };
    depot.ecrire(FICHIER_RUBANS, &en_json(&fichier)?)
}

const FICHIER_NOTES: &str = "notes.json";
const FICHIER_NOTES_CORROMPU: &str = "notes.corrompu.json";
/// Version du format de `notes.json`.
const VERSION_NOTES: u32 = 1;
/// Nombre maximal de notes du bloc-notes (section 26).
pub const MAX_NOTES: usize = 500;

/// Une note du bloc-notes (section 26) : son titre, son texte, ses dates de
/// création et de dernière modification (ISO 8601), et ses pièces jointes.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct NoteDuBloc {
    pub id: u64,
    pub titre: String,
    pub texte: String,
    pub creee: String,
    pub modifiee: String,
    /// Photos et fichiers joints, dans l'ordre d'ajout.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub pieces: Vec<PieceJointe>,
    /// Vrai pour la liste d'achat (section 26, point 10) : les calculateurs y
    /// ajoutent leurs articles, même si on la renomme.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub liste_d_achat: bool,
}

/// Une pièce jointe d'une note : sa description. Ses octets sont gardés à
/// part, sous la clé que donne `pieces::cle_des_octets` (R-95).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PieceJointe {
    pub id: u64,
    /// Nom du fichier, extension comprise.
    pub nom: String,
    pub type_mime: String,
    /// Taille gardée, en octets.
    pub taille: u64,
    /// Une photo, affichée en vignette.
    pub photo: bool,
    /// Dimensions d'une photo gardée, en pixels.
    pub largeur: u32,
    pub hauteur: u32,
    /// Date d'ajout (ISO 8601).
    pub ajoutee: String,
    /// Les cotes tracées sur une photo (section 26).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cotes: Vec<CoteSurPhoto>,
}

/// Une cote tracée sur une photo jointe : ses deux bouts, en part de la
/// largeur et de la hauteur de la photo (de 0 à 1), et sa mesure.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CoteSurPhoto {
    pub a: (f64, f64),
    pub b: (f64, f64),
    /// La mesure prise au ruban (« 35 1/2 po »); vide : estimée d'après la référence.
    pub mesure: String,
    /// La cote de référence : sa mesure sert à estimer les cotes laissées vides.
    pub reference: bool,
}

impl NoteDuBloc {
    /// Vrai si la note n'a ni titre, ni texte, ni pièce jointe.
    pub fn est_vide(&self) -> bool {
        self.titre.trim().is_empty() && self.texte.trim().is_empty() && self.pieces.is_empty()
    }

    /// La date de sa dernière modification, en clair; la date brute si elle
    /// est illisible.
    pub fn date_en_clair(&self) -> String {
        match chrono::DateTime::parse_from_rfc3339(&self.modifiee) {
            Ok(d) => textes::date_en_clair(d.year(), d.month(), d.day(), d.hour(), d.minute()),
            Err(_) => self.modifiee.clone(),
        }
    }

    /// La note à copier ou à exporter : son titre, sa date, son texte, puis
    /// le nom de ses pièces jointes.
    pub fn en_texte(&self, titre: &str) -> String {
        let mut texte = format!("{titre}\n{}\n\n{}", self.date_en_clair(), self.texte);
        if !self.pieces.is_empty() {
            let noms: Vec<&str> = self.pieces.iter().map(|p| p.nom.as_str()).collect();
            texte.push_str("\n\n");
            texte.push_str(textes::PIECES_JOINTES_DEUX_POINTS.traduit());
            texte.push(' ');
            texte.push_str(&noms.join(", "));
        }
        texte
    }
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct FichierNotes {
    version: u32,
    notes: Vec<NoteDuBloc>,
}

/// Lit les notes du bloc-notes; liste vide s'il n'y en a pas. Une liste
/// illisible ou d'une version inconnue est mise de côté.
pub fn charger_notes(depot: &Depot) -> Vec<NoteDuBloc> {
    let lues = match depot.lire(FICHIER_NOTES) {
        Lecture::Absent => return Vec::new(),
        Lecture::Illisible => None,
        Lecture::Texte(texte) => serde_json::from_str::<FichierNotes>(&texte)
            .ok()
            .filter(|fichier| fichier.version == VERSION_NOTES),
    };
    match lues {
        Some(fichier) => fichier.notes,
        None => {
            depot.mettre_de_cote(FICHIER_NOTES, FICHIER_NOTES_CORROMPU);
            Vec::new()
        }
    }
}

/// Enregistre les notes, de la plus récente à la plus ancienne.
pub fn enregistrer_notes(depot: &Depot, notes: &[NoteDuBloc]) -> io::Result<()> {
    let fichier = FichierNotes {
        version: VERSION_NOTES,
        notes: notes.to_vec(),
    };
    depot.ecrire(FICHIER_NOTES, &en_json(&fichier)?)
}

const FICHIER_NIVELLEMENT: &str = "nivellement.json";
const FICHIER_NIVELLEMENT_CORROMPU: &str = "nivellement.corrompu.json";
/// Version du format de `nivellement.json`.
const VERSION_NIVELLEMENT: u32 = 1;
/// Nombre maximal de points du carnet de nivellement.
pub const MAX_POINTS: usize = 200;

/// Le carnet de nivellement (section 22.21), chaque valeur telle qu'elle a
/// été tapée.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Carnet {
    pub titre: String,
    /// L'élévation du repère, et la lecture faite sur lui.
    pub repere: String,
    pub lecture_repere: String,
    /// L'élévation visée des points qui n'ont pas la leur.
    pub visee: String,
    pub points: Vec<PointDuCarnet>,
}

/// Un point du carnet : son nom, sa lecture au niveau, son élévation visée
/// (vide : celle du carnet).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PointDuCarnet {
    pub nom: String,
    pub lecture: String,
    pub visee: String,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct FichierCarnet {
    version: u32,
    carnet: Carnet,
}

/// Lit le carnet de nivellement; vide s'il n'y en a pas. Un carnet illisible
/// ou d'une version inconnue est mis de côté.
pub fn charger_carnet(depot: &Depot) -> Carnet {
    let lu = match depot.lire(FICHIER_NIVELLEMENT) {
        Lecture::Absent => return Carnet::default(),
        Lecture::Illisible => None,
        Lecture::Texte(texte) => serde_json::from_str::<FichierCarnet>(&texte)
            .ok()
            .filter(|fichier| fichier.version == VERSION_NIVELLEMENT),
    };
    match lu {
        Some(fichier) => fichier.carnet,
        None => {
            depot.mettre_de_cote(FICHIER_NIVELLEMENT, FICHIER_NIVELLEMENT_CORROMPU);
            Carnet::default()
        }
    }
}

/// Enregistre le carnet de nivellement.
pub fn enregistrer_carnet(depot: &Depot, carnet: &Carnet) -> io::Result<()> {
    let fichier = FichierCarnet {
        version: VERSION_NIVELLEMENT,
        carnet: carnet.clone(),
    };
    depot.ecrire(FICHIER_NIVELLEMENT, &en_json(&fichier)?)
}

/// Saisies des calculateurs par métier (section 22), par identifiant de
/// calculateur : ce que l'utilisateur a tapé la dernière fois.
pub type Saisies = BTreeMap<String, Vec<String>>;

/// Nombre de calculateurs qu'on peut épingler (section 22.16).
pub const MAX_EPINGLES: usize = 12;

/// Nombre de calculateurs récents retenus.
pub const MAX_RECENTS: usize = 6;

/// Les calculateurs épinglés et les derniers ouverts (section 22.16), par
/// identifiant; gardés avec les saisies, dans `metiers.json`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Favoris {
    /// Les calculateurs épinglés, dans l'ordre où ils l'ont été.
    pub epingles: Vec<String>,
    /// Les derniers calculateurs ouverts, du plus récent au plus ancien.
    pub recents: Vec<String>,
}

impl Favoris {
    pub fn est_epingle(&self, id: &str) -> bool {
        self.epingles.iter().any(|e| e == id)
    }

    /// Épingle le calculateur, ou le désépingle s'il l'était. Faux si la
    /// liste est pleine : il n'est alors pas épinglé.
    pub fn basculer(&mut self, id: &str) -> bool {
        if self.est_epingle(id) {
            self.epingles.retain(|e| e != id);
        } else if self.epingles.len() >= MAX_EPINGLES {
            return false;
        } else {
            self.epingles.push(id.to_string());
        }
        true
    }

    /// Le calculateur vient d'être ouvert : il passe en tête des récents.
    /// Vrai si la liste a changé.
    pub fn utiliser(&mut self, id: &str) -> bool {
        if self.recents.first().is_some_and(|r| r == id) {
            return false;
        }
        self.recents.retain(|r| r != id);
        self.recents.insert(0, id.to_string());
        self.recents.truncate(MAX_RECENTS);
        true
    }

    /// Oublie les calculateurs inconnus (d'une autre version), les doublons
    /// et ce qui dépasse.
    pub fn nettoyer(&mut self, connu: impl Fn(&str) -> bool) {
        for liste in [&mut self.epingles, &mut self.recents] {
            let mut vus = std::collections::BTreeSet::new();
            liste.retain(|id| connu(id) && vus.insert(id.clone()));
        }
        self.epingles.truncate(MAX_EPINGLES);
        self.recents.truncate(MAX_RECENTS);
    }
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct FichierSaisies {
    version: u32,
    saisies: Saisies,
    /// Les calculateurs épinglés et récents (section 22.16).
    favoris: Favoris,
    /// Identifiant du calculateur branché sur le clavier (section 23).
    outil: Option<String>,
    /// Langue dans laquelle les saisies sont écrites (section 24) : une
    /// virgule n'y a pas le même sens en français et en anglais.
    langue: Langue,
}

/// Lit les dernières saisies; aucune si la donnée est absente, illisible ou
/// d'une autre version (ce ne sont que des commodités : rien n'est mis de côté).
/// Retourne aussi l'identifiant du calculateur branché sur le clavier, et la
/// langue dans laquelle les saisies sont écrites.
pub fn charger_saisies(depot: &Depot) -> (Saisies, Option<String>, Langue) {
    match depot.lire(FICHIER_SAISIES) {
        Lecture::Texte(texte) => serde_json::from_str::<FichierSaisies>(&texte)
            .ok()
            .filter(|fichier| fichier.version == VERSION_SAISIES)
            .map(|fichier| (fichier.saisies, fichier.outil, fichier.langue))
            .unwrap_or_default(),
        Lecture::Absent | Lecture::Illisible => (Saisies::new(), None, Langue::default()),
    }
}

/// Les calculateurs épinglés et récents; aucun si la donnée est absente,
/// illisible ou d'une autre version.
pub fn charger_favoris(depot: &Depot) -> Favoris {
    match depot.lire(FICHIER_SAISIES) {
        Lecture::Texte(texte) => serde_json::from_str::<FichierSaisies>(&texte)
            .ok()
            .filter(|fichier| fichier.version == VERSION_SAISIES)
            .map(|fichier| fichier.favoris)
            .unwrap_or_default(),
        Lecture::Absent | Lecture::Illisible => Favoris::default(),
    }
}

/// Enregistre les dernières saisies, l'outil branché sur le clavier, la
/// langue dans laquelle les saisies sont écrites, et les calculateurs
/// épinglés et récents.
pub fn enregistrer_saisies(
    depot: &Depot,
    saisies: &Saisies,
    outil: Option<&str>,
    langue: Langue,
    favoris: &Favoris,
) -> io::Result<()> {
    let fichier = FichierSaisies {
        version: VERSION_SAISIES,
        saisies: saisies.clone(),
        favoris: favoris.clone(),
        outil: outil.map(str::to_string),
        langue,
    };
    depot.ecrire(FICHIER_SAISIES, &en_json(&fichier)?)
}

const FICHIER_DESSIN: &str = "dessin.json";
/// Version du format de `dessin.json`.
const VERSION_DESSIN: u32 = 1;

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct FichierDessin {
    version: u32,
    /// La pièce unique des versions 0.7 (lue, plus jamais écrite).
    #[serde(skip_serializing)]
    dessin: Dessin,
    /// Les pièces (depuis la version 0.8).
    plan: Option<Plan>,
    /// Langue dans laquelle les mesures sont écrites (section 24).
    langue: Langue,
    /// Le dessin de la bibliothèque (`dessins.json`) dont ce plan est la copie
    /// de travail (depuis la version 0.12.3).
    #[serde(skip_serializing_if = "Option::is_none")]
    courant: Option<u64>,
}

/// Le plan de travail lu dans `dessin.json`.
pub struct DessinCharge {
    pub plan: Plan,
    /// Langue dans laquelle ses mesures sont écrites.
    pub langue: Langue,
    /// Le dessin de la bibliothèque auquel il appartient, s'il est connu.
    pub courant: Option<u64>,
}

/// Vrai si le croquis de chaque côté est utilisable.
fn plan_coherent(plan: &Plan) -> bool {
    plan.pieces.iter().all(|piece| {
        piece
            .dessin
            .cotes
            .iter()
            .all(|cote| cote.croquis.is_finite() && cote.croquis > 0.0)
    })
}

/// Vrai si rien n'est dessiné : aucune pièce n'a de côté.
pub fn plan_est_vide(plan: &Plan) -> bool {
    plan.pieces
        .iter()
        .all(|piece| piece.dessin.cotes.is_empty())
}

/// Lit les pièces dessinées (section 25), la langue de leurs mesures et le
/// dessin de la bibliothèque auquel elles appartiennent; un plan vide si la
/// donnée est absente, illisible ou incohérente.
pub fn charger_dessin(depot: &Depot) -> DessinCharge {
    let lu = match depot.lire(FICHIER_DESSIN) {
        Lecture::Texte(texte) => serde_json::from_str::<FichierDessin>(&texte)
            .ok()
            .filter(|fichier| fichier.version == VERSION_DESSIN),
        Lecture::Absent | Lecture::Illisible => None,
    };
    let vide = DessinCharge {
        plan: Plan::default(),
        langue: Langue::default(),
        courant: None,
    };
    let Some(fichier) = lu else {
        return vide;
    };
    let mut plan = fichier
        .plan
        .unwrap_or_else(|| Plan::d_une_piece(fichier.dessin));
    plan.normaliser();
    if plan_coherent(&plan) {
        DessinCharge {
            plan,
            langue: fichier.langue,
            courant: fichier.courant,
        }
    } else {
        vide
    }
}

/// Enregistre les pièces dessinées, la langue de leurs mesures et le dessin
/// de la bibliothèque auquel elles appartiennent.
pub fn enregistrer_dessin(
    depot: &Depot,
    plan: &Plan,
    langue: Langue,
    courant: Option<u64>,
) -> io::Result<()> {
    let fichier = FichierDessin {
        version: VERSION_DESSIN,
        dessin: Dessin::default(),
        plan: Some(plan.clone()),
        langue,
        courant,
    };
    depot.ecrire(FICHIER_DESSIN, &en_json(&fichier)?)
}

const FICHIER_DESSINS: &str = "dessins.json";
const FICHIER_DESSINS_CORROMPU: &str = "dessins.corrompu.json";
/// Version du format de `dessins.json`.
const VERSION_DESSINS: u32 = 1;
/// Nombre maximal de dessins gardés dans la calculatrice (section 25).
pub const MAX_DESSINS: usize = 100;
/// Marque d'un dessin enregistré en fichier, pour le rouvrir ailleurs.
const FORMAT_D_UN_DESSIN: &str = "constructo-ai-dessin";
const VERSION_D_UN_DESSIN: u32 = 1;

/// Un dessin de la bibliothèque : son nom, ses dates (ISO 8601), la langue de
/// ses mesures et ses pièces.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DessinEnregistre {
    pub id: u64,
    pub nom: String,
    pub cree: String,
    pub modifie: String,
    pub langue: Langue,
    pub plan: Plan,
}

impl DessinEnregistre {
    /// La date de sa dernière modification, en clair; la date brute si elle
    /// est illisible.
    pub fn date_en_clair(&self) -> String {
        match chrono::DateTime::parse_from_rfc3339(&self.modifie) {
            Ok(d) => textes::date_en_clair(d.year(), d.month(), d.day(), d.hour(), d.minute()),
            Err(_) => self.modifie.clone(),
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct FichierDessins {
    version: u32,
    dessins: Vec<DessinEnregistre>,
}

/// Lit la bibliothèque des dessins; vide s'il n'y en a pas. Une bibliothèque
/// illisible ou d'une version inconnue est mise de côté; un dessin incohérent
/// est écarté.
pub fn charger_dessins(depot: &Depot) -> Vec<DessinEnregistre> {
    let lus = match depot.lire(FICHIER_DESSINS) {
        Lecture::Absent => return Vec::new(),
        Lecture::Illisible => None,
        Lecture::Texte(texte) => serde_json::from_str::<FichierDessins>(&texte)
            .ok()
            .filter(|fichier| fichier.version == VERSION_DESSINS),
    };
    match lus {
        Some(fichier) => fichier
            .dessins
            .into_iter()
            .filter_map(|mut dessin| {
                dessin.plan.normaliser();
                plan_coherent(&dessin.plan).then_some(dessin)
            })
            .collect(),
        None => {
            depot.mettre_de_cote(FICHIER_DESSINS, FICHIER_DESSINS_CORROMPU);
            Vec::new()
        }
    }
}

/// Enregistre la bibliothèque des dessins.
pub fn enregistrer_dessins(depot: &Depot, dessins: &[DessinEnregistre]) -> io::Result<()> {
    let fichier = FichierDessins {
        version: VERSION_DESSINS,
        dessins: dessins.to_vec(),
    };
    depot.ecrire(FICHIER_DESSINS, &en_json(&fichier)?)
}

/// Un dessin enregistré en fichier, pour le rouvrir sur un autre appareil.
#[derive(Serialize, Deserialize)]
struct FichierDUnDessin {
    format: String,
    version: u32,
    nom: String,
    date: String,
    langue: Langue,
    plan: Plan,
}

/// Le contenu du fichier d'un dessin : du JSON lisible, marqué comme tel.
pub fn fichier_d_un_dessin(dessin: &DessinEnregistre) -> String {
    let fichier = FichierDUnDessin {
        format: FORMAT_D_UN_DESSIN.to_string(),
        version: VERSION_D_UN_DESSIN,
        nom: dessin.nom.clone(),
        date: dessin.modifie.clone(),
        langue: dessin.langue,
        plan: dessin.plan.clone(),
    };
    serde_json::to_string_pretty(&fichier).unwrap_or_default()
}

/// Lit le fichier d'un dessin : son nom, la langue de ses mesures et ses
/// pièces; rien si ce n'est pas un dessin de la calculatrice, ou s'il est abîmé.
pub fn lire_un_fichier_de_dessin(octets: &[u8]) -> Option<(String, Langue, Plan)> {
    let texte = std::str::from_utf8(octets).ok()?;
    let fichier: FichierDUnDessin =
        serde_json::from_str(texte.trim_start_matches('\u{FEFF}')).ok()?;
    if fichier.format != FORMAT_D_UN_DESSIN || fichier.version != VERSION_D_UN_DESSIN {
        return None;
    }
    let mut plan = fichier.plan;
    plan.normaliser();
    plan_coherent(&plan).then_some((fichier.nom, fichier.langue, plan))
}

/// Contenu d'un fichier texte exporté : UTF-8 avec indicateur d'ordre des
/// octets et fins de ligne Windows, pour que le Bloc-notes garde les accents
/// intacts.
pub fn contenu_exporte(texte: &str) -> String {
    let mut contenu = String::with_capacity(texte.len() + 16);
    contenu.push('\u{FEFF}');
    for ligne in texte.lines() {
        contenu.push_str(ligne);
        contenu.push_str("\r\n");
    }
    contenu
}

/// Date et heure locales : (ISO 8601, date en clair).
pub fn maintenant() -> (String, String) {
    let m = chrono::Local::now();
    let iso = m.to_rfc3339_opts(chrono::SecondsFormat::Secs, false);
    let clair = textes::date_en_clair(m.year(), m.month(), m.day(), m.hour(), m.minute());
    (iso, clair)
}

/// Nom de fichier sûr à partir du nom d'un ruban.
pub fn nom_de_fichier(nom: &str) -> String {
    nom_de_fichier_avec(nom, "txt")
}

/// Nom de fichier sûr, avec l'extension donnée (sans le point).
pub fn nom_de_fichier_avec(nom: &str, extension: &str) -> String {
    let propre: String = nom
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '-',
            c if c.is_control() => ' ',
            c => c,
        })
        .collect();
    let propre = propre.trim().trim_end_matches('.').to_string();
    let base = if propre.is_empty() {
        textes::FICHIER_PAR_DEFAUT.traduit()
    } else {
        &propre
    };
    format!("{base}.{extension}")
}

// ----------------------------------------------------------------------
// Sauvegarde complète (section 17.4, R-102) : un seul fichier qui réunit tout
// ce que la calculatrice garde, pour le mettre à l'abri ou le passer à un
// autre appareil.
// ----------------------------------------------------------------------

/// Les données que réunit une sauvegarde complète.
const FICHIERS_SAUVEGARDES: [&str; 7] = [
    FICHIER_ETAT,
    FICHIER_RUBANS,
    FICHIER_SAISIES,
    FICHIER_DESSIN,
    FICHIER_DESSINS,
    FICHIER_NOTES,
    FICHIER_NIVELLEMENT,
];
const FORMAT_DE_SAUVEGARDE: &str = "constructo-ai-sauvegarde";
const VERSION_DE_SAUVEGARDE: u32 = 1;

#[derive(Serialize, Deserialize)]
struct FichierDeSauvegarde {
    format: String,
    version: u32,
    date: String,
    /// La version de la calculatrice qui l'a faite.
    application: String,
    /// Chaque donnée, par son nom de fichier, telle qu'elle est gardée.
    fichiers: BTreeMap<String, serde_json::Value>,
    /// Les pièces jointes du bloc-notes, par clé, en base64.
    #[serde(default)]
    pieces: BTreeMap<String, String>,
}

/// Les données gardées sur l'appareil, telles quelles, par nom de fichier
/// (celles qui sont absentes ou illisibles n'y sont pas).
pub fn donnees_a_sauvegarder(depot: &Depot) -> BTreeMap<String, serde_json::Value> {
    FICHIERS_SAUVEGARDES
        .iter()
        .filter_map(|nom| match depot.lire(nom) {
            Lecture::Texte(texte) => serde_json::from_str(&texte)
                .ok()
                .map(|valeur| (nom.to_string(), valeur)),
            Lecture::Absent | Lecture::Illisible => None,
        })
        .collect()
}

/// Le contenu d'une sauvegarde complète : les données, et les pièces jointes en base64.
pub fn fichier_de_sauvegarde(
    date: &str,
    fichiers: BTreeMap<String, serde_json::Value>,
    pieces: &BTreeMap<String, Vec<u8>>,
) -> String {
    let fichier = FichierDeSauvegarde {
        format: FORMAT_DE_SAUVEGARDE.to_string(),
        version: VERSION_DE_SAUVEGARDE,
        date: date.to_string(),
        application: textes::VERSION.to_string(),
        fichiers,
        pieces: pieces
            .iter()
            .map(|(cle, octets)| (cle.clone(), en_base64(octets)))
            .collect(),
    };
    serde_json::to_string(&fichier).unwrap_or_default()
}

/// Une sauvegarde lue, prête à remplacer les données de l'appareil.
#[derive(Debug)]
pub struct SauvegardeLue {
    /// Sa date (ISO 8601).
    pub date: String,
    /// Les données : (nom de fichier, JSON).
    pub fichiers: Vec<(String, String)>,
    /// Les pièces jointes : (clé, octets).
    pub pieces: Vec<(String, Vec<u8>)>,
}

/// Vrai pour une clé de pièce jointe sûre : des lettres, des chiffres et des
/// points, sans point au début (aucun chemin n'en sort).
fn cle_de_piece_sure(cle: &str) -> bool {
    !cle.is_empty()
        && cle.len() <= 64
        && !cle.starts_with('.')
        && cle.chars().all(|c| c.is_ascii_alphanumeric() || c == '.')
}

/// Lit une sauvegarde complète; rien si ce n'en est pas une, ou si elle est
/// abîmée. Seules les données connues sont reprises.
pub fn lire_une_sauvegarde(octets: &[u8]) -> Option<SauvegardeLue> {
    let texte = std::str::from_utf8(octets).ok()?;
    let fichier: FichierDeSauvegarde =
        serde_json::from_str(texte.trim_start_matches('\u{FEFF}')).ok()?;
    if fichier.format != FORMAT_DE_SAUVEGARDE || fichier.version != VERSION_DE_SAUVEGARDE {
        return None;
    }
    let mut fichiers = Vec::new();
    for (nom, valeur) in fichier.fichiers {
        if FICHIERS_SAUVEGARDES.contains(&nom.as_str()) {
            fichiers.push((nom, serde_json::to_string(&valeur).ok()?));
        }
    }
    let mut pieces = Vec::new();
    for (cle, base64) in fichier.pieces {
        if !cle_de_piece_sure(&cle) {
            return None;
        }
        pieces.push((cle, de_base64(&base64)?));
    }
    Some(SauvegardeLue {
        date: fichier.date,
        fichiers,
        pieces,
    })
}

/// Écrit les données d'une sauvegarde à la place de celles de l'appareil; les
/// données absentes de la sauvegarde restent telles quelles.
pub fn restaurer(depot: &Depot, sauvegarde: &SauvegardeLue) -> io::Result<()> {
    for (nom, texte) in &sauvegarde.fichiers {
        depot.ecrire(nom, texte)?;
    }
    for (cle, octets) in &sauvegarde.pieces {
        depot.ecrire_piece(cle, octets.clone());
    }
    Ok(())
}

const ALPHABET_BASE64: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Des octets en base64 (alphabet standard, avec remplissage).
pub fn en_base64(octets: &[u8]) -> String {
    let mut texte = String::with_capacity(octets.len().div_ceil(3) * 4);
    for bloc in octets.chunks(3) {
        let n = (u32::from(bloc[0]) << 16)
            | (u32::from(bloc.get(1).copied().unwrap_or(0)) << 8)
            | u32::from(bloc.get(2).copied().unwrap_or(0));
        for k in 0..4 {
            if k <= bloc.len() {
                texte.push(char::from(
                    ALPHABET_BASE64[((n >> (18 - 6 * k)) & 63) as usize],
                ));
            } else {
                texte.push('=');
            }
        }
    }
    texte
}

/// Du base64 en octets; rien s'il est mal formé.
pub fn de_base64(texte: &str) -> Option<Vec<u8>> {
    let valeur = |c: u8| -> Option<u32> {
        match c {
            b'A'..=b'Z' => Some(u32::from(c - b'A')),
            b'a'..=b'z' => Some(u32::from(c - b'a') + 26),
            b'0'..=b'9' => Some(u32::from(c - b'0') + 52),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    };
    let signes: Vec<u8> = texte.bytes().filter(|c| !c.is_ascii_whitespace()).collect();
    if !signes.len().is_multiple_of(4) {
        return None;
    }
    let blocs = signes.len() / 4;
    let mut octets = Vec::with_capacity(blocs * 3);
    for (rang, bloc) in signes.chunks(4).enumerate() {
        let remplissage = bloc.iter().rev().take_while(|c| **c == b'=').count();
        // Le remplissage n'est permis qu'à la toute fin.
        if remplissage > 2 || (remplissage > 0 && rang + 1 < blocs) {
            return None;
        }
        let mut n = 0u32;
        for (k, &c) in bloc.iter().enumerate() {
            let v = if k >= 4 - remplissage { 0 } else { valeur(c)? };
            n = (n << 6) | v;
        }
        let trois = [(n >> 16) as u8, (n >> 8) as u8, n as u8];
        octets.extend_from_slice(&trois[..3 - remplissage]);
    }
    Some(octets)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::*;
    use noyau::{Moteur, Touche};

    /// Dossier temporaire propre à un test, supprimé à la fin.
    struct DossierTemporaire(PathBuf);

    impl DossierTemporaire {
        fn nouveau(nom: &str) -> Self {
            let chemin = std::env::temp_dir().join(format!(
                "calculatrice-chantier-test-{}-{nom}",
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&chemin);
            fs::create_dir_all(&chemin).unwrap();
            DossierTemporaire(chemin)
        }

        fn depot(&self) -> Depot {
            Depot::dans(&self.0)
        }
    }

    impl Drop for DossierTemporaire {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn fichier_absent_valeurs_par_defaut() {
        let d = DossierTemporaire::nouveau("absent");
        let c = charger_etat(&d.depot());
        assert!(!c.illisible);
        assert_eq!(c.etat, EtatPersistant::default());
    }

    #[test]
    fn aller_retour_de_l_etat() {
        let d = DossierTemporaire::nouveau("aller-retour");
        let mut moteur = Moteur::default();
        for t in [
            Touche::Chiffre(7),
            Touche::Pouces,
            Touche::Pente,
            Touche::Chiffre(5),
            Touche::Pieds,
            Touche::MPlus,
            Touche::Chiffre(9),
            Touche::Stocker,
            Touche::Chiffre(1),
        ] {
            moteur.appuyer(t);
        }
        moteur.preferences_mut().resolution = 8;
        moteur.marquer_bienvenue_vue();
        let etat = moteur.etat_persistant();
        enregistrer_etat(&d.depot(), &etat).unwrap();
        // L'écriture est atomique : il ne reste pas de fichier temporaire.
        assert!(!d.0.join("etat.json.tmp").exists());
        let relu = charger_etat(&d.depot());
        assert!(!relu.illisible);
        assert_eq!(relu.etat.preferences, etat.preferences);
        assert_eq!(relu.etat.memoire, etat.memoire);
        assert_eq!(relu.etat.registres, etat.registres);
        assert_eq!(relu.etat.ruban, etat.ruban);
        assert_eq!(
            relu.etat.pente.map(|p| p.ratio),
            etat.pente.map(|p| p.ratio)
        );
        assert!(relu.etat.bienvenue_vue);
        // Sur le bureau, le JSON est lisible et en UTF-8.
        let texte = fs::read_to_string(d.0.join("etat.json")).unwrap();
        assert!(texte.contains("\"resolution\": 8"));
    }

    #[test]
    fn fichier_corrompu_mis_de_cote() {
        let d = DossierTemporaire::nouveau("corrompu");
        fs::write(d.0.join("etat.json"), "{ ceci n'est pas du JSON").unwrap();
        let c = charger_etat(&d.depot());
        assert!(c.illisible);
        assert_eq!(c.etat, EtatPersistant::default());
        assert!(d.0.join("etat.corrompu.json").exists());
        assert!(!d.0.join("etat.json").exists());
    }

    #[test]
    fn version_inconnue_mise_de_cote() {
        let d = DossierTemporaire::nouveau("version");
        fs::write(d.0.join("etat.json"), "{ \"version\": 99 }").unwrap();
        assert!(charger_etat(&d.depot()).illisible);
        assert!(d.0.join("etat.corrompu.json").exists());
    }

    #[test]
    fn champs_manquants_tolerés() {
        // R-47 : un fichier partiel reste lisible.
        let d = DossierTemporaire::nouveau("partiel");
        fs::write(
            d.0.join("etat.json"),
            "{ \"version\": 1, \"preferences\": { \"resolution\": 32 } }",
        )
        .unwrap();
        let c = charger_etat(&d.depot());
        assert!(!c.illisible);
        assert_eq!(c.etat.preferences.resolution, 32);
        assert!(c.etat.preferences.separateur_milliers);
        assert_eq!(c.etat.memorises, noyau::ValeursMemorisees::default());
    }

    #[test]
    fn json_compact_relu_comme_le_json_lisible() {
        // Le navigateur conserve un JSON compact : il doit se relire pareil.
        let d = DossierTemporaire::nouveau("compact");
        let mut moteur = Moteur::default();
        for t in [Touche::Chiffre(4), Touche::Pieds, Touche::MPlus] {
            moteur.appuyer(t);
        }
        let etat = moteur.etat_persistant();
        fs::write(d.0.join("etat.json"), serde_json::to_string(&etat).unwrap()).unwrap();
        let relu = charger_etat(&d.depot());
        assert!(!relu.illisible);
        assert_eq!(relu.etat.memoire, etat.memoire);
        assert_eq!(relu.etat.ruban, etat.ruban);
    }

    #[test]
    fn aller_retour_des_rubans() {
        let d = DossierTemporaire::nouveau("rubans");
        assert!(charger_rubans(&d.depot()).is_empty());
        let ruban = RubanEnregistre {
            id: 1,
            nom: "Escalier du sous-sol".to_string(),
            date: "2026-11-19T08:04:00-05:00".to_string(),
            lignes: vec![LigneRuban {
                texte: "5 pi 7 1/2 po".to_string(),
                ..LigneRuban::default()
            }],
        };
        enregistrer_rubans(&d.depot(), std::slice::from_ref(&ruban)).unwrap();
        assert_eq!(charger_rubans(&d.depot()), vec![ruban.clone()]);
        assert_eq!(ruban.date_en_clair(), "19 novembre 2026, 8 h 04");
    }

    #[test]
    fn rubans_illisibles_mis_de_cote() {
        let d = DossierTemporaire::nouveau("rubans-illisibles");
        fs::write(
            d.0.join("rubans.json"),
            "{ \"version\": 99, \"rubans\": [] }",
        )
        .unwrap();
        assert!(charger_rubans(&d.depot()).is_empty());
        assert!(d.0.join("rubans.corrompu.json").exists());
        assert!(!d.0.join("rubans.json").exists());
    }

    #[test]
    fn le_base64_aller_retour() {
        assert_eq!(en_base64(b"Man"), "TWFu");
        assert_eq!(en_base64(b"Ma"), "TWE=");
        assert_eq!(en_base64(b"M"), "TQ==");
        assert_eq!(en_base64(b""), "");
        for longueur in 0..40usize {
            let octets: Vec<u8> = (0..longueur).map(|k| (k * 37 + 11) as u8).collect();
            assert_eq!(
                de_base64(&en_base64(&octets)).unwrap(),
                octets,
                "{longueur}"
            );
        }
        assert!(de_base64("TQ=").is_none());
        assert!(de_base64("T?==").is_none());
        assert!(de_base64("TQ==TWFu").is_none());
        assert_eq!(de_base64("TW\nFu").unwrap(), b"Man");
    }

    #[test]
    fn une_sauvegarde_complete_aller_retour() {
        let d = DossierTemporaire::nouveau("sauvegarde");
        let note = NoteDuBloc {
            id: 4,
            titre: "Chantier".to_string(),
            texte: "Commander le gypse".to_string(),
            ..NoteDuBloc::default()
        };
        enregistrer_notes(&d.depot(), std::slice::from_ref(&note)).unwrap();
        let maison = DessinEnregistre {
            id: 9,
            nom: "Maison".to_string(),
            ..DessinEnregistre::default()
        };
        enregistrer_dessins(&d.depot(), std::slice::from_ref(&maison)).unwrap();
        let carnet = Carnet {
            titre: "Entrée".to_string(),
            repere: "100".to_string(),
            points: vec![PointDuCarnet {
                nom: "P1".to_string(),
                lecture: "4 pi 6 po".to_string(),
                visee: String::new(),
            }],
            ..Carnet::default()
        };
        enregistrer_carnet(&d.depot(), &carnet).unwrap();
        let fichiers = donnees_a_sauvegarder(&d.depot());
        assert_eq!(
            fichiers.keys().collect::<Vec<_>>(),
            ["dessins.json", "nivellement.json", "notes.json"]
        );
        let mut pieces = BTreeMap::new();
        pieces.insert("12.jpg".to_string(), vec![0xFF, 0xD8, 1, 2, 3]);
        let contenu = fichier_de_sauvegarde("2026-11-23T15:00:00-05:00", fichiers, &pieces);
        assert!(contenu.contains("\"format\":\"constructo-ai-sauvegarde\""));
        // Un autre appareil : vide, il reçoit tout.
        let autre = DossierTemporaire::nouveau("sauvegarde-autre");
        let lue = lire_une_sauvegarde(contenu.as_bytes()).unwrap();
        assert_eq!(lue.date, "2026-11-23T15:00:00-05:00");
        restaurer(&autre.depot(), &lue).unwrap();
        assert_eq!(charger_notes(&autre.depot()), vec![note]);
        assert_eq!(charger_dessins(&autre.depot()), vec![maison]);
        assert_eq!(charger_carnet(&autre.depot()), carnet);
        let _ = crate::plateforme::arrivees();
        autre.depot().lire_piece("12.jpg");
        assert!(matches!(
            crate::plateforme::arrivees().as_slice(),
            [crate::plateforme::Arrivee::Lue { octets: Some(o), .. }] if o == &[0xFF, 0xD8, 1, 2, 3]
        ));
    }

    #[test]
    fn aller_retour_du_carnet_de_nivellement() {
        let d = DossierTemporaire::nouveau("nivellement");
        assert_eq!(charger_carnet(&d.depot()), Carnet::default());
        let carnet = Carnet {
            repere: "100".to_string(),
            lecture_repere: "5".to_string(),
            ..Carnet::default()
        };
        enregistrer_carnet(&d.depot(), &carnet).unwrap();
        assert_eq!(charger_carnet(&d.depot()), carnet);
        // Un carnet illisible est mis de côté.
        fs::write(d.0.join("nivellement.json"), "pas du JSON").unwrap();
        assert_eq!(charger_carnet(&d.depot()), Carnet::default());
        assert!(d.0.join("nivellement.corrompu.json").exists());
    }

    #[test]
    fn une_sauvegarde_douteuse_est_refusee() {
        assert!(lire_une_sauvegarde(b"pas du json").is_none());
        assert!(lire_une_sauvegarde(b"{ \"format\": \"autre\", \"version\": 1, \"date\": \"\", \"application\": \"\", \"fichiers\": {} }").is_none());
        let modele = |pieces: &str, fichiers: &str| {
            format!(
                "{{ \"format\": \"constructo-ai-sauvegarde\", \"version\": 1, \"date\": \"\", \"application\": \"0.13.0\", \"fichiers\": {fichiers}, \"pieces\": {pieces} }}"
            )
        };
        // Une clé qui sortirait du dossier des pièces jointes.
        assert!(
            lire_une_sauvegarde(modele("{ \"../etat.json\": \"TQ==\" }", "{}").as_bytes())
                .is_none()
        );
        assert!(lire_une_sauvegarde(modele("{ \".cache\": \"TQ==\" }", "{}").as_bytes()).is_none());
        // Du base64 abîmé.
        assert!(lire_une_sauvegarde(modele("{ \"3.jpg\": \"T?==\" }", "{}").as_bytes()).is_none());
        // Une donnée inconnue est laissée de côté, pas refusée.
        let lue = lire_une_sauvegarde(
            modele(
                "{}",
                "{ \"autre.json\": {}, \"notes.json\": { \"version\": 1, \"notes\": [] } }",
            )
            .as_bytes(),
        )
        .unwrap();
        assert_eq!(lue.fichiers.len(), 1);
        assert_eq!(lue.fichiers[0].0, "notes.json");
    }

    #[test]
    fn aller_retour_des_notes() {
        let d = DossierTemporaire::nouveau("notes");
        assert!(charger_notes(&d.depot()).is_empty());
        let note = NoteDuBloc {
            id: 1,
            titre: "Chantier du sous-sol".to_string(),
            texte: "Acheter 12 feuilles de gypse de 1/2 po.\nAppeler l'électricien.".to_string(),
            creee: "2026-11-21T08:04:00-05:00".to_string(),
            modifiee: "2026-11-21T09:15:00-05:00".to_string(),
            pieces: Vec::new(),
            liste_d_achat: false,
        };
        enregistrer_notes(&d.depot(), std::slice::from_ref(&note)).unwrap();
        assert_eq!(charger_notes(&d.depot()), vec![note.clone()]);
        // Une note sans pièce jointe s'écrit comme avant la version 0.12.0.
        let texte = fs::read_to_string(d.0.join("notes.json")).unwrap();
        assert!(!texte.contains("pieces"), "{texte}");
        // Ni le drapeau de la liste d'achat, pour une note ordinaire; la liste
        // d'achat garde le sien.
        assert!(!texte.contains("liste_d_achat"), "{texte}");
        let liste = NoteDuBloc {
            id: 2,
            liste_d_achat: true,
            ..note.clone()
        };
        enregistrer_notes(&d.depot(), std::slice::from_ref(&liste)).unwrap();
        assert_eq!(charger_notes(&d.depot()), vec![liste]);
        assert_eq!(note.date_en_clair(), "21 novembre 2026, 9 h 15");
        assert!(
            note.en_texte(&note.titre)
                .starts_with("Chantier du sous-sol\n21 novembre 2026, 9 h 15\n\nAcheter")
        );
        assert!(!note.est_vide());
        assert!(NoteDuBloc::default().est_vide());
    }

    #[test]
    fn pieces_jointes_des_notes() {
        let d = DossierTemporaire::nouveau("notes-pieces");
        let photo = PieceJointe {
            id: 1_763_900_000_000,
            nom: "IMG_2041.jpg".to_string(),
            type_mime: "image/jpeg".to_string(),
            taille: 412_000,
            photo: true,
            largeur: 2048,
            hauteur: 1536,
            ajoutee: "2026-11-23T10:00:00-05:00".to_string(),
            // Une cote de référence, mesurée au ruban.
            cotes: vec![CoteSurPhoto {
                a: (0.1, 0.5),
                b: (0.6, 0.5),
                mesure: "32 po".to_string(),
                reference: true,
            }],
        };
        let devis = PieceJointe {
            id: 1_763_900_000_001,
            nom: "Soumission.pdf".to_string(),
            type_mime: "application/pdf".to_string(),
            taille: 88_000,
            ..PieceJointe::default()
        };
        let note = NoteDuBloc {
            id: 3,
            pieces: vec![photo, devis],
            ..NoteDuBloc::default()
        };
        // Une note faite de pièces jointes seulement n'est pas vide.
        assert!(!note.est_vide());
        enregistrer_notes(&d.depot(), std::slice::from_ref(&note)).unwrap();
        assert_eq!(charger_notes(&d.depot()), vec![note.clone()]);
        assert!(
            note.en_texte("Fenêtres")
                .ends_with("\n\nPièces jointes : IMG_2041.jpg, Soumission.pdf"),
            "{}",
            note.en_texte("Fenêtres")
        );
        // Une liste écrite avant les pièces jointes se lit toujours.
        fs::write(
            d.0.join("notes.json"),
            "{ \"version\": 1, \"notes\": [ { \"id\": 9, \"titre\": \"Ancienne\", \
             \"texte\": \"t\", \"creee\": \"\", \"modifiee\": \"\" } ] }",
        )
        .unwrap();
        let lues = charger_notes(&d.depot());
        assert_eq!(lues.len(), 1);
        assert!(lues[0].pieces.is_empty());
    }

    #[test]
    fn notes_illisibles_mises_de_cote() {
        let d = DossierTemporaire::nouveau("notes-illisibles");
        fs::write(d.0.join("notes.json"), "{ \"version\": 99, \"notes\": [] }").unwrap();
        assert!(charger_notes(&d.depot()).is_empty());
        assert!(d.0.join("notes.corrompu.json").exists());
    }

    #[test]
    fn aller_retour_des_saisies() {
        let d = DossierTemporaire::nouveau("saisies");
        let rien = (Saisies::new(), None, Langue::Francais);
        assert_eq!(charger_saisies(&d.depot()), rien);
        let mut saisies = Saisies::new();
        saisies.insert(
            "dalle".to_string(),
            vec!["24".to_string(), "20".to_string(), "4".to_string()],
        );
        let favoris = Favoris::default();
        enregistrer_saisies(
            &d.depot(),
            &saisies,
            Some("dalle"),
            Langue::Francais,
            &favoris,
        )
        .unwrap();
        assert_eq!(
            charger_saisies(&d.depot()),
            (saisies.clone(), Some("dalle".to_string()), Langue::Francais)
        );
        // La langue dans laquelle les saisies sont écrites est conservée avec elles.
        enregistrer_saisies(&d.depot(), &saisies, None, Langue::Anglais, &favoris).unwrap();
        assert_eq!(
            charger_saisies(&d.depot()),
            (saisies.clone(), None, Langue::Anglais)
        );
        // Un fichier d'une version précédente, sans langue : il est en français.
        fs::write(
            d.0.join("metiers.json"),
            "{ \"version\": 1, \"saisies\": { \"dalle\": [\"24\", \"20\", \"4\"] }, \"outil\": null }",
        )
        .unwrap();
        assert_eq!(
            charger_saisies(&d.depot()),
            (saisies, None, Langue::Francais)
        );
        // Une donnée illisible est simplement ignorée.
        fs::write(d.0.join("metiers.json"), "pas du JSON").unwrap();
        assert_eq!(charger_saisies(&d.depot()), rien);
        assert_eq!(charger_favoris(&d.depot()), Favoris::default());
    }

    #[test]
    fn calculateurs_epingles_et_recents() {
        let mut favoris = Favoris::default();
        // Épingler, puis désépingler.
        assert!(favoris.basculer("dalle") && favoris.est_epingle("dalle"));
        assert!(favoris.basculer("dalle") && !favoris.est_epingle("dalle"));
        // Douze au plus : le treizième est refusé.
        for k in 0..MAX_EPINGLES {
            assert!(favoris.basculer(&format!("outil-{k}")));
        }
        assert!(!favoris.basculer("dalle") && !favoris.est_epingle("dalle"));
        assert_eq!(favoris.epingles.len(), MAX_EPINGLES);
        // Les récents : le dernier ouvert en tête, sans doublon, six au plus.
        let mut favoris = Favoris::default();
        assert!(favoris.utiliser("dalle"));
        assert!(!favoris.utiliser("dalle"));
        assert!(favoris.utiliser("escalier"));
        assert!(favoris.utiliser("dalle"));
        assert_eq!(favoris.recents, ["dalle", "escalier"]);
        for k in 0..10 {
            favoris.utiliser(&format!("outil-{k}"));
        }
        assert_eq!(favoris.recents.len(), MAX_RECENTS);
        assert_eq!(favoris.recents[0], "outil-9");
        // Un calculateur inconnu et un doublon sont oubliés à la lecture.
        let mut lus = Favoris {
            epingles: vec!["dalle".into(), "disparu".into(), "dalle".into()],
            recents: vec!["escalier".into(), "disparu".into()],
        };
        lus.nettoyer(|id| id != "disparu");
        assert_eq!(lus.epingles, ["dalle"]);
        assert_eq!(lus.recents, ["escalier"]);
        // Ils sont gardés avec les saisies, et un fichier d'avant n'en a pas.
        let d = DossierTemporaire::nouveau("favoris");
        assert_eq!(charger_favoris(&d.depot()), Favoris::default());
        enregistrer_saisies(&d.depot(), &Saisies::new(), None, Langue::Francais, &lus).unwrap();
        assert_eq!(charger_favoris(&d.depot()), lus);
        fs::write(
            d.0.join("metiers.json"),
            "{ \"version\": 1, \"saisies\": {}, \"outil\": null }",
        )
        .unwrap();
        assert_eq!(charger_favoris(&d.depot()), Favoris::default());
    }

    /// Ce que `charger_dessin` a lu, pour comparer.
    fn lu(d: &DossierTemporaire) -> (Plan, Langue, Option<u64>) {
        let charge = charger_dessin(&d.depot());
        (charge.plan, charge.langue, charge.courant)
    }

    fn un_rectangle() -> Dessin {
        let mut dessin = Dessin::default();
        dessin.ajouter_trace(&[
            (0.0, 0.0),
            (100.0, 0.0),
            (100.0, 60.0),
            (0.0, 60.0),
            (0.0, 4.0),
        ]);
        dessin.cotes[0].saisie = "12".to_string();
        dessin.cotes[1].saisie = "10".to_string();
        dessin
    }

    #[test]
    fn la_bibliotheque_des_dessins() {
        let d = DossierTemporaire::nouveau("dessins");
        assert!(charger_dessins(&d.depot()).is_empty());
        let maison = DessinEnregistre {
            id: 1_763_900_000_000,
            nom: "Maison Tremblay".to_string(),
            cree: "2026-11-23T09:00:00-05:00".to_string(),
            modifie: "2026-11-23T10:30:00-05:00".to_string(),
            langue: Langue::Francais,
            plan: Plan::d_une_piece(un_rectangle()),
        };
        let vide = DessinEnregistre {
            id: 1_763_900_000_001,
            nom: "Dessin 2".to_string(),
            ..DessinEnregistre::default()
        };
        assert!(!plan_est_vide(&maison.plan));
        assert!(plan_est_vide(&vide.plan));
        enregistrer_dessins(&d.depot(), &[maison.clone(), vide.clone()]).unwrap();
        assert_eq!(charger_dessins(&d.depot()), vec![maison.clone(), vide]);
        assert_eq!(maison.date_en_clair(), "23 novembre 2026, 10 h 30");
        // Le plan de travail retient le dessin auquel il appartient.
        enregistrer_dessin(&d.depot(), &maison.plan, Langue::Francais, Some(maison.id)).unwrap();
        assert_eq!(
            lu(&d),
            (maison.plan.clone(), Langue::Francais, Some(maison.id))
        );
        // Une bibliothèque illisible est mise de côté.
        fs::write(d.0.join("dessins.json"), "{ \"version\": 7 }").unwrap();
        assert!(charger_dessins(&d.depot()).is_empty());
        assert!(d.0.join("dessins.corrompu.json").exists());
    }

    #[test]
    fn un_dessin_se_rouvre_d_un_fichier() {
        let maison = DessinEnregistre {
            id: 5,
            nom: "Chalet".to_string(),
            modifie: "2026-11-23T10:30:00-05:00".to_string(),
            langue: Langue::Anglais,
            plan: Plan::d_une_piece(un_rectangle()),
            ..DessinEnregistre::default()
        };
        let contenu = fichier_d_un_dessin(&maison);
        assert!(
            contenu.contains("\"format\": \"constructo-ai-dessin\""),
            "{contenu}"
        );
        let (nom, langue, plan) = lire_un_fichier_de_dessin(contenu.as_bytes()).unwrap();
        assert_eq!((nom.as_str(), langue), ("Chalet", Langue::Anglais));
        assert_eq!(plan, maison.plan);
        // Avec l'indicateur d'ordre des octets d'un éditeur de texte, aussi.
        let avec_bom = format!("\u{FEFF}{contenu}");
        assert!(lire_un_fichier_de_dessin(avec_bom.as_bytes()).is_some());
        // Ce qui n'est pas un dessin de la calculatrice est refusé.
        assert!(lire_un_fichier_de_dessin(b"pas du json").is_none());
        assert!(lire_un_fichier_de_dessin(b"{ \"version\": 1, \"notes\": [] }").is_none());
        let autre_format = contenu.replace("constructo-ai-dessin", "autre-chose");
        assert!(lire_un_fichier_de_dessin(autre_format.as_bytes()).is_none());
        let abime = contenu.replacen("\"croquis\": ", "\"croquis\": -", 1);
        assert!(lire_un_fichier_de_dessin(abime.as_bytes()).is_none());
    }

    #[test]
    fn le_dessin_est_conserve() {
        let d = DossierTemporaire::nouveau("dessin");
        assert_eq!(lu(&d), (Plan::default(), Langue::Francais, None));
        let mut dessin = Dessin::default();
        dessin.ajouter_trace(&[
            (0.0, 0.0),
            (100.0, 0.0),
            (100.0, 60.0),
            (0.0, 60.0),
            (0.0, 4.0),
        ]);
        assert!(dessin.fermee);
        dessin.cotes[0].saisie = "12 ft 6 in".to_string();
        let mut plan = Plan::d_une_piece(dessin.clone());
        plan.pieces[0].nom = "Kitchen".to_string();
        plan.ajouter();
        enregistrer_dessin(&d.depot(), &plan, Langue::Anglais, None).unwrap();
        assert_eq!(lu(&d), (plan, Langue::Anglais, None));
        // Un plan d'avant la bibliothèque ne porte pas de « courant ».
        let texte = fs::read_to_string(d.0.join("dessin.json")).unwrap();
        assert!(!texte.contains("\"courant\""), "{texte}");
        // Un dessin de la version 0.7 (une seule pièce) devient un plan d'une pièce.
        let ancien = format!(
            "{{ \"version\": 1, \"langue\": \"Anglais\", \"dessin\": {} }}",
            serde_json::to_string(&dessin).unwrap()
        );
        fs::write(d.0.join("dessin.json"), ancien).unwrap();
        assert_eq!(lu(&d), (Plan::d_une_piece(dessin), Langue::Anglais, None));
        // Un fichier abîmé ou incohérent donne un plan vide, sans erreur.
        fs::write(
            d.0.join("dessin.json"),
            "{ \"version\": 1, \"dessin\": { \"cotes\": [{ \"sens\": \"Est\", \"croquis\": -4.0 }] } }",
        )
        .unwrap();
        assert_eq!(lu(&d).0, Plan::default());
        fs::write(
            d.0.join("dessin.json"),
            "{ \"version\": 1, \"plan\": { \"pieces\": [], \"courante\": 9 } }",
        )
        .unwrap();
        assert_eq!(lu(&d).0, Plan::default());
        fs::write(d.0.join("dessin.json"), "pas du json").unwrap();
        assert_eq!(lu(&d).0, Plan::default());
        // Un escalier d'avant les formes (version 0.12.5) se lit comme un
        // escalier droit, à la hauteur des murs.
        let mut avec_escalier = un_rectangle();
        avec_escalier.objets.push(noyau::mobilier::Objet::nouveau(
            noyau::mobilier::GenreDObjet::Escalier,
            (30.0, 40.0),
        ));
        let texte = serde_json::to_string(&avec_escalier).unwrap().replace(
            ",\"forme\":\"Droit\",\"hauteur\":\"\",\"contremarches\":\"\"",
            "",
        );
        assert!(!texte.contains("forme"), "{texte}");
        fs::write(
            d.0.join("dessin.json"),
            format!("{{ \"version\": 1, \"dessin\": {texte} }}"),
        )
        .unwrap();
        let escalier = &lu(&d).0.pieces[0].dessin.objets[0];
        assert_eq!(escalier.forme, noyau::mobilier::FormeDEscalier::Droit);
        assert!(escalier.hauteur.is_empty() && escalier.contremarches.is_empty());
    }

    #[test]
    fn premier_lancement_et_langue_conservee() {
        let d = DossierTemporaire::nouveau("langue");
        // Rien n'a encore été enregistré : c'est le premier lancement.
        let c = charger_etat(&d.depot());
        assert!(c.premiere_fois && !c.illisible);
        assert_eq!(c.etat.preferences.langue, Langue::Francais);
        // La langue choisie est conservée avec les préférences.
        let mut etat = c.etat;
        etat.preferences.langue = Langue::Anglais;
        enregistrer_etat(&d.depot(), &etat).unwrap();
        let relu = charger_etat(&d.depot());
        assert!(!relu.premiere_fois && !relu.illisible);
        assert_eq!(relu.etat.preferences.langue, Langue::Anglais);
        // Un état d'une version précédente, sans langue : il est en français.
        fs::write(
            d.0.join("etat.json"),
            "{ \"version\": 1, \"preferences\": { \"mode_trigo\": true } }",
        )
        .unwrap();
        let ancien = charger_etat(&d.depot());
        assert!(!ancien.premiere_fois && !ancien.illisible);
        assert_eq!(ancien.etat.preferences.langue, Langue::Francais);
        assert!(ancien.etat.preferences.mode_trigo);
        // Un état illisible n'est pas un premier lancement.
        fs::write(d.0.join("etat.json"), "pas du JSON").unwrap();
        let illisible = charger_etat(&d.depot());
        assert!(illisible.illisible && !illisible.premiere_fois);
    }

    #[test]
    fn export_lisible_dans_le_bloc_notes() {
        let contenu = contenu_exporte("Montée\n= 5 pi 7 1/2 po\n");
        // Indicateur d'ordre des octets en tête, puis des fins de ligne Windows.
        assert_eq!(&contenu.as_bytes()[..3], &[0xEF, 0xBB, 0xBF]);
        assert_eq!(contenu, "\u{FEFF}Montée\r\n= 5 pi 7 1/2 po\r\n");
    }

    #[test]
    fn noms_de_fichier_surs() {
        assert_eq!(
            nom_de_fichier("Ruban du 19 novembre 2026, 8 h 04"),
            "Ruban du 19 novembre 2026, 8 h 04.txt"
        );
        assert_eq!(nom_de_fichier("a/b:c?"), "a-b-c-.txt");
        assert_eq!(nom_de_fichier("  "), "ruban.txt");
    }
}
