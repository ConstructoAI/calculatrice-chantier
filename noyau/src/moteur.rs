//! Moteur de calcul : états, préfixes, opérateurs, cycles de sortie (section 9).

mod memoires;
mod touches_fonctions;

use crate::conversions::{Conversion, UniteTouche, convertir};
use crate::erreurs::{Avertissement, Erreur, Information};
use crate::etiquettes::{Etiquette, Note, symbole_masse_volumique};
use crate::fonctions::arc::ArcSaisi;
use crate::fonctions::dimensions::RegistresDimensions;
use crate::fonctions::onglet::Coin;
use crate::fonctions::triangle::{FormePente, RegistresTriangle};
use crate::fonctions::{Cycle, Sortie};
use crate::format::{OptionsFormat, Rendu, Segment, TexteValeur, est_affichable, formater};
use crate::memoire::MemoireCumulative;
use crate::persistance::{EtatPersistant, VERSION_FORMAT};
use crate::prefs::{ModeFractions, Preferences, ValeursMemorisees};
use crate::ruban::Ruban;
use crate::saisie::{Frappe, Issue, Saisie, UniteSaisie};
use crate::touches::Touche;
use crate::unites::{Format, Genre, UniteMasseVolumique};
use crate::valeur::{Operateur, Valeur, calculer, calculer_pourcent, carre, inverse, racine};

/// Préfixe armé, tel que l'afficheur le montre.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Prefixe {
    #[default]
    Aucun,
    Conv,
    Stocker,
    Rappel,
}

/// D'où vient la valeur montrée (section 9.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Provenance {
    /// Afficheur vide après effacement.
    Zero,
    /// Une saisie est en cours.
    Saisie,
    /// Valeur libre : saisie validée, résultat, valeur rappelée.
    Resultat,
    /// Sortie d'une touche de fonction, avec étiquette.
    Sortie,
    /// Erreur ou message.
    Message,
}

/// Action qui relève de l'interface.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effet {
    OuvrirPreferences,
    BasculerRuban,
    ConfirmerEffacerTout,
    /// Quelque chose est à sauvegarder.
    EtatModifie,
}

/// Ce que l'afficheur doit montrer (section 8.1).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Ecran {
    pub etiquette: Option<String>,
    pub valeur: TexteValeur,
    /// Indicateur MÉM.
    pub memorise: bool,
    pub avertissement: Option<String>,
    pub prefixe: Prefixe,
    /// Opération en attente : + − × ÷.
    pub operateur: Option<char>,
    /// Indicateur M : la mémoire M+ n'est pas vide.
    pub memoire_active: bool,
    /// Erreur ou message plein écran.
    pub message: Option<String>,
    /// Le message est une erreur bloquante (R-38).
    pub erreur: bool,
    /// Mention complémentaire : « sur 12 », mode des fractions.
    pub note: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PrefixeInterne {
    Aucun,
    Conv,
    Stocker,
    Rappel,
    /// `Rappel` puis `Conv` : en attente de `Arêtier Noue` (pente irrégulière).
    RappelConv,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Message {
    Erreur(Erreur),
    Information(Information),
}

/// Habillage de la valeur montrée quand aucune saisie n'est en cours.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Presentation {
    etiquette: Option<Etiquette>,
    memorise: bool,
    avertissement: Option<Avertissement>,
    rendu: Rendu,
    /// La valeur est une masse volumique : son unité suit le nombre.
    masse_volumique: Option<UniteMasseVolumique>,
    note: Option<Note>,
}

/// Registres du toit : pente irrégulière et base de mur (sections 12.2 et 12.4).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct RegistresToit {
    pente_irreguliere: Option<(f64, FormePente)>,
    base: Option<Valeur>,
}

/// Registres du cercle : diamètre et arc saisis (section 12.8).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct RegistresCercle {
    diametre: Option<Valeur>,
    arc: Option<ArcSaisi>,
}

/// Nombre de côtés du polygone et coin de l'onglet composé.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct RegistresDivers {
    nb_cotes: Option<u32>,
    coin: Option<Coin>,
}

/// Fonction mathématique à un opérande (section 9.7).
#[derive(Clone, Copy, Debug)]
enum FonctionMath {
    Carre,
    Racine,
    Inverse,
}

/// Le moteur de la calculatrice.
#[derive(Clone, Debug)]
pub struct Moteur {
    prefs: Preferences,
    memo: ValeursMemorisees,
    saisie: Saisie,
    courant: Valeur,
    provenance: Provenance,
    presentation: Presentation,
    message: Option<Message>,
    operation: Option<(Valeur, Operateur)>,
    /// Un opérande droit a été fourni depuis l'appui sur l'opérateur.
    operande_pret: bool,
    prefixe: PrefixeInterne,
    triangle: RegistresTriangle,
    toit: RegistresToit,
    cercle: RegistresCercle,
    dimensions: RegistresDimensions,
    divers: RegistresDivers,
    memoire: MemoireCumulative,
    registres: [Option<Valeur>; 3],
    cycle: Option<Cycle>,
    resolution_temp: Option<u32>,
    ruban: Ruban,
    /// Ordre chronologique des saisies du triangle.
    compteur: u64,
    /// La touche précédente était une première pression d'`Effacer`.
    dernier_effacer: bool,
    /// Touche dont la valeur montrée vient d'être rappelée (R-37).
    rappel_de: Option<Touche>,
    bienvenue_vue: bool,
    /// Compteur de modifications de l'état à sauvegarder.
    revision: u64,
}

impl Default for Moteur {
    fn default() -> Self {
        Moteur::nouveau(EtatPersistant::default())
    }
}

impl Moteur {
    /// Crée un moteur à partir de l'état conservé.
    pub fn nouveau(etat: EtatPersistant) -> Moteur {
        let etat = etat.assaini();
        // La pente d'une séance précédente est la plus ancienne des saisies (R-41).
        let triangle = RegistresTriangle {
            pente: etat.pente.map(|mut p| {
                p.ordre = 0;
                p
            }),
            ..RegistresTriangle::default()
        };
        Moteur {
            prefs: etat.preferences,
            memo: etat.memorises,
            saisie: Saisie::nouvelle(),
            courant: Valeur::nombre(0.0),
            provenance: Provenance::Zero,
            presentation: Presentation::default(),
            message: None,
            operation: None,
            operande_pret: false,
            prefixe: PrefixeInterne::Aucun,
            triangle,
            toit: RegistresToit::default(),
            cercle: RegistresCercle::default(),
            dimensions: RegistresDimensions::default(),
            divers: RegistresDivers::default(),
            memoire: etat.memoire,
            registres: etat.registres,
            cycle: None,
            resolution_temp: None,
            ruban: etat.ruban,
            compteur: 0,
            dernier_effacer: false,
            rappel_de: None,
            bienvenue_vue: etat.bienvenue_vue,
            revision: 0,
        }
    }

    // ------------------------------------------------------------------
    // Interface publique
    // ------------------------------------------------------------------

    /// Traite une touche. Ne panique jamais (R-26) : toute anomalie devient une
    /// erreur affichée.
    pub fn appuyer(&mut self, touche: Touche) -> Vec<Effet> {
        let revision = self.revision;
        let mut effets = Vec::new();
        let effacer_precedent = std::mem::replace(&mut self.dernier_effacer, false);
        if self.erreur_bloquante() {
            // En état d'erreur, seule la touche Effacer est acceptée (R-38).
            if touche == Touche::Effacer {
                self.effacer(false);
                self.dernier_effacer = true;
            }
            return effets;
        }
        let prefixe = std::mem::replace(&mut self.prefixe, PrefixeInterne::Aucun);
        let resultat = match prefixe {
            PrefixeInterne::Aucun => self.touche_simple(touche, effacer_precedent),
            PrefixeInterne::Conv => self.touche_conv(touche, &mut effets),
            PrefixeInterne::Stocker => self.touche_stocker(touche),
            PrefixeInterne::Rappel => self.touche_rappel(touche, &mut effets),
            PrefixeInterne::RappelConv => self.touche_rappel_conv(touche),
        };
        if let Err(erreur) = resultat {
            self.signaler(erreur);
        }
        self.controler_capacite();
        if self.revision != revision {
            effets.push(Effet::EtatModifie);
        }
        effets
    }

    /// Ce que l'afficheur doit montrer.
    pub fn ecran(&self) -> Ecran {
        let prefixe = match self.prefixe {
            PrefixeInterne::Aucun => Prefixe::Aucun,
            PrefixeInterne::Conv => Prefixe::Conv,
            PrefixeInterne::Stocker => Prefixe::Stocker,
            PrefixeInterne::Rappel | PrefixeInterne::RappelConv => Prefixe::Rappel,
        };
        let commun = Ecran {
            prefixe,
            operateur: self.operation.map(|(_, op)| op.symbole()),
            memoire_active: !self.memoire.est_vide(),
            ..Ecran::default()
        };
        if let Some(message) = self.message {
            let (texte, erreur) = match message {
                Message::Erreur(e) => (e.texte(), e.est_bloquante()),
                Message::Information(i) => (i.texte(), false),
            };
            return Ecran {
                message: Some(texte.to_string()),
                erreur,
                ..commun
            };
        }
        if self.provenance == Provenance::Saisie {
            return Ecran {
                valeur: self.saisie.echo(&self.prefs),
                ..commun
            };
        }
        let p = &self.presentation;
        Ecran {
            etiquette: p.etiquette.map(Etiquette::texte),
            valeur: self.texte_valeur(&self.courant, p),
            memorise: p.memorise,
            avertissement: p.avertissement.map(|a| a.texte().to_string()),
            note: p.note.map(|n| n.texte().to_string()),
            ..commun
        }
    }

    pub fn ruban(&self) -> &Ruban {
        &self.ruban
    }

    /// Vide le ruban courant.
    pub fn vider_ruban(&mut self) {
        self.ruban.vider();
        self.revision += 1;
    }

    /// Inscrit au ruban un calcul fait dans un calculateur par métier
    /// (section 22) : un titre, les valeurs saisies, puis les résultats.
    pub fn consigner(
        &mut self,
        titre: &str,
        entrees: &[(&str, String)],
        resultats: &[(&str, String)],
    ) {
        self.ruban.separateur();
        self.ruban.titre(titre.to_string());
        for (libelle, valeur) in entrees {
            self.ruban.sortie((*libelle).to_string(), valeur.clone());
        }
        for (libelle, valeur) in resultats {
            self.ruban.resultat((*libelle).to_string(), valeur.clone());
        }
        self.ruban.separateur();
        self.revision += 1;
    }

    /// `Effacer tout`, après confirmation par l'interface (section 9.9).
    pub fn effacer_tout(&mut self) {
        self.effacer(true);
        self.memoire.vider();
        self.registres = [None; 3];
        self.triangle.pente = None;
        self.ruban.vider();
        self.memo = ValeursMemorisees::default();
        self.dernier_effacer = false;
        self.montrer_information(Information::ToutEfface);
        self.revision += 1;
    }

    /// `Effacer tout`, plus la remise des préférences à leurs valeurs par défaut.
    pub fn reinitialiser(&mut self) {
        self.effacer_tout();
        self.prefs = Preferences::default();
    }

    /// État à conserver d'une séance à l'autre.
    pub fn etat_persistant(&self) -> EtatPersistant {
        EtatPersistant {
            version: VERSION_FORMAT,
            preferences: self.prefs.clone(),
            memorises: self.memo.clone(),
            pente: self.triangle.pente,
            memoire: self.memoire,
            registres: self.registres,
            ruban: self.ruban.clone(),
            bienvenue_vue: self.bienvenue_vue,
        }
    }

    pub fn preferences(&self) -> &Preferences {
        &self.prefs
    }

    /// Accès en écriture aux préférences. Met fin au cycle en cours et à la
    /// résolution temporaire, qui pourraient ne plus correspondre.
    pub fn preferences_mut(&mut self) -> &mut Preferences {
        self.cycle = None;
        self.resolution_temp = None;
        self.revision += 1;
        &mut self.prefs
    }

    pub fn memorises(&self) -> &ValeursMemorisees {
        &self.memo
    }

    /// Accès en écriture aux valeurs mémorisées. Met fin au cycle en cours.
    pub fn memorises_mut(&mut self) -> &mut ValeursMemorisees {
        self.cycle = None;
        self.revision += 1;
        &mut self.memo
    }

    pub fn bienvenue_vue(&self) -> bool {
        self.bienvenue_vue
    }

    pub fn marquer_bienvenue_vue(&mut self) {
        if !self.bienvenue_vue {
            self.bienvenue_vue = true;
            self.revision += 1;
        }
    }

    /// Résolution des fractions en vigueur (temporaire, sinon celle des préférences).
    pub fn resolution(&self) -> u32 {
        self.options().resolution()
    }

    /// Texte à plat de ce que montre l'afficheur (presse-papiers).
    pub fn texte_affiche(&self) -> String {
        let ecran = self.ecran();
        match ecran.message {
            Some(message) => message,
            None => ecran.valeur.en_texte(),
        }
    }

    /// Place une valeur libre à l'écran, comme si elle venait d'être saisie (collage).
    pub fn injecter(&mut self, valeur: Valeur) {
        if self.erreur_bloquante() || !valeur.base.is_finite() {
            return;
        }
        self.prefixe = PrefixeInterne::Aucun;
        self.dernier_effacer = false;
        self.saisie.vider();
        self.montrer_resultat(valeur);
        self.controler_capacite();
    }

    // ------------------------------------------------------------------
    // Répartition des touches
    // ------------------------------------------------------------------

    fn touche_simple(&mut self, touche: Touche, effacer_precedent: bool) -> Result<(), Erreur> {
        match touche {
            Touche::Chiffre(c) => self.frappe_chiffre(c),
            Touche::Virgule => self.frapper(Frappe::Virgule, true),
            Touche::Fraction => self.frapper(Frappe::Barre, false),
            Touche::FractionRapide { num, den } => {
                self.frapper(Frappe::FractionRapide { num, den }, true)
            }
            Touche::RetourArriere => {
                self.retour_arriere();
                Ok(())
            }
            Touche::Pieds => self.touche_unite(UniteTouche::Pieds, false),
            Touche::Pouces => self.touche_unite(UniteTouche::Pouces, false),
            Touche::Verges => self.touche_unite(UniteTouche::Verges, false),
            Touche::Metres => self.touche_unite(UniteTouche::Metres, false),
            Touche::Additionner => self.operateur(Operateur::Additionner),
            Touche::Soustraire => self.operateur(Operateur::Soustraire),
            Touche::Multiplier => self.operateur(Operateur::Multiplier),
            Touche::Diviser => self.operateur(Operateur::Diviser),
            Touche::Egal => self.egal(),
            Touche::Pourcent => self.pourcent(),
            Touche::Effacer => {
                self.effacer(effacer_precedent);
                self.dernier_effacer = !effacer_precedent;
                Ok(())
            }
            Touche::Conv => {
                self.prefixe = PrefixeInterne::Conv;
                Ok(())
            }
            Touche::Stocker => {
                self.prefixe = PrefixeInterne::Stocker;
                Ok(())
            }
            Touche::Rappel => {
                self.prefixe = PrefixeInterne::Rappel;
                Ok(())
            }
            Touche::MPlus => self.m_plus(false),
            Touche::Pente
            | Touche::Montee
            | Touche::Course
            | Touche::Diag
            | Touche::AretierNoue
            | Touche::OngletCompose
            | Touche::Escalier
            | Touche::Arc
            | Touche::Cercle
            | Touche::Empannon
            | Touche::Longueur
            | Touche::Largeur
            | Touche::Hauteur => self.touche_fonction(touche, false),
        }
    }

    fn touche_conv(&mut self, touche: Touche, effets: &mut Vec<Effet>) -> Result<(), Erreur> {
        match touche {
            // `Conv` une seconde fois désarme le préfixe.
            Touche::Conv | Touche::Fraction => Ok(()),
            Touche::Pieds => self.touche_unite(UniteTouche::Pieds, true),
            Touche::Pouces => self.touche_unite(UniteTouche::Pouces, true),
            Touche::Verges => self.touche_unite(UniteTouche::Verges, true),
            Touche::Metres => self.touche_unite(UniteTouche::Metres, true),
            Touche::Chiffre(7) => self.touche_unite(UniteTouche::Cm, true),
            Touche::Chiffre(8) => self.touche_unite(UniteTouche::Pmp, true),
            Touche::Chiffre(9) => self.touche_unite(UniteTouche::Mm, true),
            Touche::Chiffre(4) => self.touche_unite(UniteTouche::Lb, true),
            Touche::Chiffre(6) => self.touche_unite(UniteTouche::Tc, true),
            Touche::Chiffre(1) => self.touche_unite(UniteTouche::Kg, true),
            Touche::Chiffre(2) => self.touche_unite(UniteTouche::Acre, true),
            Touche::Chiffre(3) => self.touche_unite(UniteTouche::T, true),
            Touche::Chiffre(5) => self.touche_fonction(touche, true),
            Touche::Chiffre(0) => self.cout(),
            Touche::Chiffre(_) => Ok(()),
            Touche::Pourcent => self.fonction_math(FonctionMath::Carre),
            Touche::Effacer => self.fonction_math(FonctionMath::Racine),
            Touche::Diviser => self.fonction_math(FonctionMath::Inverse),
            Touche::Soustraire => self.changer_signe(),
            Touche::Additionner => {
                self.saisie.vider();
                self.montrer_resultat(Valeur::nombre(std::f64::consts::PI));
                Ok(())
            }
            Touche::Multiplier => {
                effets.push(Effet::ConfirmerEffacerTout);
                Ok(())
            }
            Touche::Egal => {
                effets.push(Effet::BasculerRuban);
                Ok(())
            }
            Touche::Stocker => {
                effets.push(Effet::OuvrirPreferences);
                Ok(())
            }
            Touche::Rappel => {
                self.memoire.vider();
                self.revision += 1;
                self.montrer_information(Information::MemoireEffacee);
                Ok(())
            }
            Touche::MPlus => self.m_plus(true),
            Touche::Virgule => self.basculer_dms(),
            Touche::RetourArriere | Touche::FractionRapide { .. } => {
                self.touche_simple(touche, false)
            }
            Touche::Pente
            | Touche::Montee
            | Touche::Course
            | Touche::Diag
            | Touche::AretierNoue
            | Touche::OngletCompose
            | Touche::Escalier
            | Touche::Arc
            | Touche::Cercle
            | Touche::Empannon
            | Touche::Longueur
            | Touche::Largeur
            | Touche::Hauteur => self.touche_fonction(touche, true),
        }
    }

    // ------------------------------------------------------------------
    // État de l'afficheur
    // ------------------------------------------------------------------

    fn options(&self) -> OptionsFormat<'_> {
        OptionsFormat {
            prefs: &self.prefs,
            resolution_temp: self.resolution_temp,
        }
    }

    fn texte_valeur(&self, valeur: &Valeur, p: &Presentation) -> TexteValeur {
        let mut texte = formater(valeur, p.rendu, &self.options());
        if let Some(unite) = p.masse_volumique {
            texte
                .0
                .push(Segment::Unite(symbole_masse_volumique(unite).to_string()));
        }
        texte
    }

    /// Texte à plat de la valeur montrée, pour le ruban.
    fn texte_montre(&self) -> String {
        if self.provenance == Provenance::Saisie {
            return self.saisie.echo(&self.prefs).en_texte();
        }
        self.texte_valeur(&self.courant, &self.presentation)
            .en_texte()
    }

    fn texte_de(&self, valeur: &Valeur, rendu: Rendu) -> String {
        formater(valeur, rendu, &self.options()).en_texte()
    }

    fn erreur_bloquante(&self) -> bool {
        matches!(self.message, Some(Message::Erreur(e)) if e.est_bloquante())
    }

    /// Affiche une erreur. Une erreur bloquante annule l'opération en attente (R-38).
    fn signaler(&mut self, erreur: Erreur) {
        self.saisie.vider();
        self.message = Some(Message::Erreur(erreur));
        self.provenance = Provenance::Message;
        self.courant = Valeur::nombre(0.0);
        self.presentation = Presentation::default();
        self.cycle = None;
        self.rappel_de = None;
        self.prefixe = PrefixeInterne::Aucun;
        if erreur.est_bloquante() {
            self.operation = None;
            self.operande_pret = false;
        }
    }

    fn montrer_information(&mut self, information: Information) {
        self.saisie.vider();
        self.message = Some(Message::Information(information));
        self.provenance = Provenance::Message;
        self.courant = Valeur::nombre(0.0);
        self.presentation = Presentation::default();
        self.cycle = None;
        self.rappel_de = None;
    }

    /// Affiche `0`, sans étiquette (section 8.8).
    fn mettre_a_zero(&mut self) {
        self.saisie.vider();
        self.message = None;
        self.provenance = Provenance::Zero;
        self.courant = Valeur::nombre(0.0);
        self.presentation = Presentation::default();
        self.cycle = None;
        self.rappel_de = None;
        self.operande_pret = false;
    }

    /// Montre une valeur libre, sans étiquette. Met fin au cycle en cours (R-28).
    fn montrer_resultat(&mut self, valeur: Valeur) {
        self.courant = valeur;
        self.provenance = Provenance::Resultat;
        self.presentation = Presentation::default();
        self.message = None;
        self.cycle = None;
        self.rappel_de = None;
        self.operande_pret = true;
    }

    /// Montre une sortie de fonction, avec son étiquette. Ne touche pas au cycle.
    fn montrer_sortie(&mut self, sortie: &Sortie) {
        let sur_12 = matches!(
            sortie.etiquette,
            Etiquette::Pente | Etiquette::PenteIrreguliere
        ) && sortie.valeur.genre == Genre::Longueur;
        self.courant = sortie.valeur;
        self.provenance = Provenance::Sortie;
        self.presentation = Presentation {
            etiquette: Some(sortie.etiquette),
            memorise: sortie.memorise,
            avertissement: sortie.avertissement,
            rendu: sortie.rendu,
            masse_volumique: None,
            note: sur_12.then_some(Note::Sur12),
        };
        self.message = None;
        self.rappel_de = None;
        self.operande_pret = true;
    }

    /// Contrôle la capacité d'affichage de la valeur montrée (R-25).
    fn controler_capacite(&mut self) {
        if matches!(self.provenance, Provenance::Resultat | Provenance::Sortie)
            && !est_affichable(&self.courant, &self.prefs)
        {
            let erreur = if self.courant.base.is_nan() {
                Erreur::Calcul
            } else {
                Erreur::Depassement
            };
            self.signaler(erreur);
        }
    }

    // ------------------------------------------------------------------
    // Ruban
    // ------------------------------------------------------------------

    fn ruban_operande(&mut self, operateur: Option<char>, texte: String) {
        self.ruban.operande(operateur, texte);
        self.revision += 1;
    }

    fn ruban_total(&mut self, valeur: &Valeur) {
        let texte = self.texte_de(valeur, Rendu::Normal);
        self.ruban.total(texte);
        self.revision += 1;
    }

    fn ruban_sortie(&mut self, sortie: &Sortie) {
        let texte = self.texte_de(&sortie.valeur, sortie.rendu);
        self.ruban.sortie(sortie.etiquette.texte(), texte);
        self.revision += 1;
    }

    // ------------------------------------------------------------------
    // Saisie
    // ------------------------------------------------------------------

    fn commencer_saisie(&mut self) {
        self.saisie.vider();
        self.provenance = Provenance::Saisie;
        self.message = None;
        self.presentation = Presentation::default();
        self.courant = Valeur::nombre(0.0);
        self.cycle = None;
        self.rappel_de = None;
        self.operande_pret = true;
    }

    /// Soumet une frappe à la saisie. `peut_commencer` : la frappe peut ouvrir
    /// une nouvelle saisie quand aucune n'est en cours.
    fn frapper(&mut self, frappe: Frappe, peut_commencer: bool) -> Result<(), Erreur> {
        if self.provenance != Provenance::Saisie {
            if !peut_commencer {
                return Ok(());
            }
            self.commencer_saisie();
        }
        let mut issue = self.saisie.frapper(frappe);
        if issue == Issue::Nouvelle {
            // La saisie précédente était complète : celle-ci la remplace (7.1).
            self.commencer_saisie();
            issue = self.saisie.frapper(frappe);
        }
        if issue == Issue::Erreur {
            return Err(Erreur::Saisie);
        }
        if !self.saisie.est_active() {
            self.mettre_a_zero();
        }
        Ok(())
    }

    fn frappe_chiffre(&mut self, chiffre: u8) -> Result<(), Erreur> {
        if self.provenance == Provenance::Saisie {
            // Après `Conv` `7` (cm) ou `Conv` `9` (mm), le même chiffre fait défiler
            // l'exposant (R-19).
            let unite = match (self.saisie.unite_seule(), chiffre) {
                (Some(UniteSaisie::Cm), 7) => Some(UniteSaisie::Cm),
                (Some(UniteSaisie::Mm), 9) => Some(UniteSaisie::Mm),
                _ => None,
            };
            if let Some(u) = unite {
                return self.frapper(Frappe::Unite(u), false);
            }
        }
        self.frapper(Frappe::Chiffre(chiffre), true)
    }

    fn retour_arriere(&mut self) {
        if self.provenance == Provenance::Saisie {
            self.saisie.retour_arriere();
            if !self.saisie.est_active() {
                self.mettre_a_zero();
            }
        }
    }

    /// Valide la saisie en cours : elle devient une valeur libre (7.8).
    fn valider_saisie(&mut self) -> Result<Valeur, Erreur> {
        let validee = self.saisie.valider(self.resolution())?;
        self.saisie.vider();
        if let Some(r) = validee.resolution_temp {
            self.resolution_temp = Some(r);
        }
        self.montrer_resultat(validee.valeur);
        Ok(validee.valeur)
    }

    /// Valeur montrée, après validation de la saisie éventuelle. Zéro si
    /// l'afficheur est vide ou montre un message.
    fn valeur_montree(&mut self) -> Result<Valeur, Erreur> {
        match self.provenance {
            Provenance::Saisie => self.valider_saisie(),
            Provenance::Resultat | Provenance::Sortie => Ok(self.courant),
            Provenance::Zero | Provenance::Message => Ok(Valeur::nombre(0.0)),
        }
    }

    /// Donnée destinée à une touche de fonction (R-35, R-37) : la saisie en cours
    /// ou une valeur libre, sauf si elle vient d'être rappelée de cette touche.
    fn donnee_libre(&mut self, touche: Touche) -> Result<Option<Valeur>, Erreur> {
        match self.provenance {
            Provenance::Saisie => self.valider_saisie().map(Some),
            Provenance::Resultat if self.rappel_de != Some(touche) => Ok(Some(self.courant)),
            _ => Ok(None),
        }
    }

    // ------------------------------------------------------------------
    // Opérateurs (sections 9.4 à 9.7)
    // ------------------------------------------------------------------

    fn operateur(&mut self, op: Operateur) -> Result<(), Erreur> {
        let droite = self.valeur_montree()?;
        if self.provenance == Provenance::Message {
            self.message = None;
            self.provenance = Provenance::Zero;
        }
        match self.operation.take() {
            Some((gauche, precedent)) => {
                if self.operande_pret {
                    let texte = self.texte_montre();
                    self.ruban_operande(Some(precedent.symbole()), texte);
                    // Exécution immédiate, de gauche à droite (R-29).
                    let resultat = calculer(&gauche, precedent, &droite, &self.prefs)?;
                    self.montrer_resultat(resultat);
                    self.operation = Some((resultat, op));
                } else {
                    // Pas d'opérande droit : l'opérateur est simplement remplacé.
                    self.operation = Some((gauche, op));
                }
            }
            None => {
                let texte = self.texte_montre();
                self.ruban_operande(None, texte);
                self.cycle = None;
                self.operation = Some((droite, op));
            }
        }
        self.operande_pret = false;
        Ok(())
    }

    /// Calcule l'opération en attente si son opérande droit est prêt (comme `=`).
    fn terminer_operation(&mut self) -> Result<(), Erreur> {
        if !self.operande_pret {
            return Ok(());
        }
        if let Some((gauche, op)) = self.operation.take() {
            let droite = self.valeur_montree()?;
            let texte = self.texte_montre();
            self.ruban_operande(Some(op.symbole()), texte);
            let resultat = calculer(&gauche, op, &droite, &self.prefs)?;
            self.ruban_total(&resultat);
            self.montrer_resultat(resultat);
            self.operande_pret = false;
        }
        Ok(())
    }

    fn egal(&mut self) -> Result<(), Erreur> {
        self.valeur_montree()?;
        match self.operation {
            Some((gauche, _)) => {
                if self.operande_pret {
                    self.terminer_operation()?;
                } else {
                    // Pas d'opérande droit : l'opération est annulée.
                    self.operation = None;
                    self.montrer_resultat(gauche);
                }
            }
            None => {
                // Promotion d'une sortie de fonction en valeur libre (R-36).
                if self.provenance == Provenance::Sortie {
                    self.montrer_resultat(self.courant);
                }
            }
        }
        self.operande_pret = false;
        Ok(())
    }

    fn pourcent(&mut self) -> Result<(), Erreur> {
        let b = self.valeur_montree()?;
        match self.operation.take() {
            Some((a, op)) => {
                if self.operande_pret {
                    // `%` termine le calcul (R-34).
                    let resultat = calculer_pourcent(&a, op, &b)?;
                    let fraction = if b.format == Format::Pourcent {
                        b.base
                    } else {
                        b.base / 100.0
                    };
                    let texte = self.texte_de(
                        &Valeur {
                            base: fraction,
                            genre: Genre::SansUnite,
                            format: Format::Pourcent,
                        },
                        Rendu::Normal,
                    );
                    self.ruban_operande(Some(op.symbole()), texte);
                    self.ruban_total(&resultat);
                    self.montrer_resultat(resultat);
                } else {
                    self.montrer_resultat(a);
                }
                self.operande_pret = false;
            }
            None => {
                if !matches!(self.provenance, Provenance::Resultat | Provenance::Sortie) {
                    return Ok(());
                }
                if !b.est_sans_unite() {
                    return Err(Erreur::Unites);
                }
                // Marque le nombre comme pourcentage : 75 → 75 % (base 0,75).
                if b.format != Format::Pourcent {
                    self.montrer_resultat(Valeur {
                        base: b.base / 100.0,
                        genre: Genre::SansUnite,
                        format: Format::Pourcent,
                    });
                }
            }
        }
        Ok(())
    }

    fn fonction_math(&mut self, fonction: FonctionMath) -> Result<(), Erreur> {
        let v = self.valeur_montree()?;
        let resultat = match fonction {
            FonctionMath::Carre => carre(&v, &self.prefs)?,
            FonctionMath::Racine => racine(&v)?,
            FonctionMath::Inverse => inverse(&v)?,
        };
        self.montrer_resultat(resultat);
        Ok(())
    }

    fn changer_signe(&mut self) -> Result<(), Erreur> {
        match self.provenance {
            Provenance::Saisie => self.saisie.changer_signe(),
            Provenance::Resultat | Provenance::Sortie => {
                let v = self.courant;
                self.montrer_resultat(Valeur { base: -v.base, ..v });
            }
            Provenance::Zero | Provenance::Message => {}
        }
        Ok(())
    }

    /// `Conv` `,` : bascule entre degrés décimaux et degrés-minutes-secondes (7.5).
    fn basculer_dms(&mut self) -> Result<(), Erreur> {
        let v = self.valeur_montree()?;
        if !matches!(self.provenance, Provenance::Resultat | Provenance::Sortie) {
            return Ok(());
        }
        let format = match v.format {
            Format::Dms => Format::Degres,
            Format::Nombre | Format::Degres => Format::Dms,
            _ => return Err(Erreur::Unites),
        };
        let convertie = Valeur { format, ..v };
        if self.provenance == Provenance::Sortie {
            // Comme une conversion : l'étiquette et le cycle sont conservés.
            self.courant = convertie;
        } else {
            self.montrer_resultat(convertie);
        }
        Ok(())
    }

    // ------------------------------------------------------------------
    // Effacement (section 9.9)
    // ------------------------------------------------------------------

    /// `deuxieme` : seconde pression consécutive, qui efface aussi les valeurs
    /// temporaires.
    fn effacer(&mut self, deuxieme: bool) {
        self.prefixe = PrefixeInterne::Aucun;
        self.mettre_a_zero();
        if deuxieme {
            self.operation = None;
            self.triangle.montee = None;
            self.triangle.course = None;
            self.triangle.diag = None;
            // La pente saisie est conservée, mais devient la plus ancienne (R-41).
            if let Some(p) = &mut self.triangle.pente {
                p.ordre = 0;
            }
            self.toit = RegistresToit::default();
            self.cercle = RegistresCercle::default();
            self.dimensions = RegistresDimensions::default();
            self.divers = RegistresDivers::default();
            self.resolution_temp = None;
            self.ruban.separateur();
            self.revision += 1;
        }
    }

    // ------------------------------------------------------------------
    // Unités et conversions (section 10)
    // ------------------------------------------------------------------

    fn touche_unite(&mut self, unite: UniteTouche, conv: bool) -> Result<(), Erreur> {
        if self.provenance == Provenance::Saisie {
            // Pendant une saisie, l'unité étiquette le nombre ou fait défiler
            // l'exposant; `Conv` + unité sur une saisie déjà étiquetée convertit (R-39).
            if let Some(u) = unite_de_saisie(unite)
                && (!conv || self.saisie.est_nombre_simple())
            {
                return self.frapper(Frappe::Unite(u), false);
            }
            self.valider_saisie()?;
        }
        if conv {
            // L'opération en attente est d'abord calculée (R-39 bis).
            self.terminer_operation()?;
        }
        self.convertir_montre(unite)
    }

    fn convertir_montre(&mut self, unite: UniteTouche) -> Result<(), Erreur> {
        if !matches!(self.provenance, Provenance::Resultat | Provenance::Sortie) {
            return Ok(());
        }
        let avant = self.courant;
        match convertir(&avant, unite, &self.memo.masse_volumique)? {
            Conversion::Resolution(r) => self.resolution_temp = Some(r),
            Conversion::Valeur(v) => {
                if self.provenance == Provenance::Sortie && v.genre == avant.genre {
                    // Seul le format change : étiquette, provenance et cycle restent (R-28).
                    self.courant = v;
                    self.presentation.masse_volumique = None;
                } else {
                    self.montrer_resultat(v);
                }
                let texte = self.texte_montre();
                self.ruban.conversion(texte);
                self.revision += 1;
            }
        }
        Ok(())
    }

    // ------------------------------------------------------------------
    // Cycles de sortie (section 9.2)
    // ------------------------------------------------------------------

    /// Lance un nouveau cycle et montre sa première sortie.
    fn lancer_cycle(
        &mut self,
        touche: Touche,
        secondaire: bool,
        sorties: Vec<Sortie>,
    ) -> Result<(), Erreur> {
        self.cycle = Some(Cycle::nouveau(touche, secondaire, sorties));
        self.avancer_cycle()
    }

    /// Montre la sortie suivante du cycle en cours (R-27).
    fn avancer_cycle(&mut self) -> Result<(), Erreur> {
        let suivante = self.cycle.as_mut().and_then(Cycle::suivante);
        let Some((sortie, premiere_fois)) = suivante else {
            self.cycle = None;
            return Err(Erreur::AucuneValeur);
        };
        if premiere_fois {
            self.ruban_sortie(&sortie);
        }
        self.montrer_sortie(&sortie);
        Ok(())
    }

    /// Montre une sortie unique, sans cycle (fonctions placées sur des chiffres).
    fn montrer_sortie_unique(&mut self, sortie: &Sortie) {
        self.cycle = None;
        self.ruban_sortie(sortie);
        self.montrer_sortie(sortie);
    }

    /// Affecte une donnée à une fonction : la montre avec son étiquette et prépare
    /// le cycle, dont la première sortie apparaîtra à la pression suivante (R-35).
    fn montrer_donnee(
        &mut self,
        touche: Touche,
        secondaire: bool,
        donnee: Sortie,
        cycle: Result<Vec<Sortie>, Erreur>,
    ) {
        self.ruban_sortie(&donnee);
        self.montrer_sortie(&donnee);
        self.cycle = cycle.ok().filter(|s| !s.is_empty()).map(|sorties| {
            let mut c = Cycle::nouveau(touche, secondaire, sorties);
            // La donnée tient lieu de première sortie quand celle-ci la répète.
            let repete = c.sorties.first().is_some_and(|s| {
                s.etiquette == donnee.etiquette && s.valeur.genre == donnee.valeur.genre
            });
            if repete {
                c.reprendre_apres(0);
            }
            c
        });
    }
}

/// Unité de saisie correspondant à une touche d'unité; `None` pour le pmp, qui
/// n'étiquette jamais une saisie (R-20).
fn unite_de_saisie(unite: UniteTouche) -> Option<UniteSaisie> {
    Some(match unite {
        UniteTouche::Verges => UniteSaisie::Verges,
        UniteTouche::Pieds => UniteSaisie::Pieds,
        UniteTouche::Pouces => UniteSaisie::Pouces,
        UniteTouche::Metres => UniteSaisie::Metres,
        UniteTouche::Cm => UniteSaisie::Cm,
        UniteTouche::Mm => UniteSaisie::Mm,
        UniteTouche::Lb => UniteSaisie::Lb,
        UniteTouche::Kg => UniteSaisie::Kg,
        UniteTouche::Tc => UniteSaisie::Tc,
        UniteTouche::T => UniteSaisie::T,
        UniteTouche::Acre => UniteSaisie::Acre,
        UniteTouche::Pmp => return None,
    })
}

/// Note à montrer avec la résolution des fractions.
fn note_fractions(mode: ModeFractions) -> Note {
    match mode {
        ModeFractions::Standard => Note::FractionsStandard,
        ModeFractions::Constant => Note::FractionsConstantes,
    }
}
