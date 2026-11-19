//! Outils de métier au clavier (section 23).
//!
//! Un calculateur par métier (section 22) peut être « branché » sur le
//! clavier : ses champs deviennent les touches du haut, et la touche
//! `Résultat` donne ses résultats. Les valeurs se tapent avec les touches de
//! la calculatrice (`Pieds`, `Pouces`, fractions), sans clavier d'ordinateur.
//!
//! Les saisies sont les mêmes textes que ceux du formulaire : la valeur montrée
//! à l'afficheur est rangée telle qu'elle s'écrit, puis relue par le champ.
//! Le formulaire et le clavier donnent donc toujours les mêmes résultats.

use super::{Message, Moteur, Presentation, Provenance};
use crate::erreurs::{Erreur, Information};
use crate::etiquettes::{rang_parmi, symbole};
use crate::format::Rendu;
use crate::metiers::{self, Calculateur, Champ, Lecture, Lue, Nature};
use crate::prefs::SeparateurDecimal;
use crate::touches::Touche;
use crate::unites::Format;
use crate::valeur::Valeur;

/// Nombre maximal de champs d'un outil : deux rangées de cinq touches.
pub const CHAMPS_D_OUTIL: usize = 10;
/// Rang de la touche `Résultat` : il suit ceux des champs.
pub const TOUCHE_RESULTAT: u8 = CHAMPS_D_OUTIL as u8;

/// L'outil branché sur le clavier.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct OutilActif {
    /// Identifiant du calculateur.
    pub(super) id: &'static str,
    /// Ce que contient chaque champ, comme dans le formulaire.
    pub(super) saisies: Vec<String>,
    /// Rang du résultat montré, tant que la touche `Résultat` les fait défiler.
    pub(super) rang: Option<usize>,
}

/// Ce que montre la touche d'un champ, pour l'interface.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToucheOutil {
    /// Libellé court, pour une touche étroite.
    pub libelle: &'static str,
    /// Libellé complet du champ.
    pub nom: &'static str,
    /// Contenu du champ, remis au propre (vide si le champ est vide).
    pub valeur: String,
    /// Unité d'un nombre tapé sans unité.
    pub unite: &'static str,
    /// Vrai : chaque pression passe à l'option suivante d'une liste.
    pub choix: bool,
    /// Vrai : le champ peut rester vide.
    pub facultatif: bool,
}

impl Moteur {
    /// Branche un outil sur le clavier, ou revient au clavier de charpente
    /// (`None`). `saisies` : le contenu des champs, s'il est connu.
    pub fn choisir_outil(&mut self, id: Option<&str>, saisies: Option<&[String]>) {
        let nouveau = id.and_then(metiers::calculateur).map(|c| OutilActif {
            id: c.id,
            saisies: saisies
                .filter(|s| s.len() == c.champs.len())
                .map_or_else(|| c.saisies_par_defaut(), <[String]>::to_vec),
            rang: None,
        });
        // En changeant d'outil, l'afficheur ne garde ni le libellé ni les
        // messages de l'outil qu'on quitte. La valeur montrée, elle, reste
        // comme une valeur ordinaire : un résultat passe ainsi d'un outil au
        // suivant (le volume d'une tranchée vers les voyages de camion).
        let change = self.outil.as_ref().map(|o| o.id) != nouveau.as_ref().map(|o| o.id);
        if change {
            let message_d_outil = matches!(
                self.message,
                Some(
                    Message::Avis(_)
                        | Message::Manque(_)
                        | Message::ACorriger(_)
                        | Message::Information(
                            Information::ChampVide | Information::OutilRecommence
                        )
                )
            );
            if message_d_outil {
                self.mettre_a_zero();
            } else if self.presentation.libre.is_some() {
                // Un choix dans une liste ou un mot n'ont pas de valeur à garder.
                if self.courant.base == 0.0 {
                    self.mettre_a_zero();
                } else {
                    let valeur = self.courant;
                    self.montrer_resultat(valeur);
                }
            }
        }
        self.outil = nouveau;
    }

    /// Le calculateur branché sur le clavier, s'il y en a un.
    pub fn outil(&self) -> Option<&'static Calculateur> {
        self.outil.as_ref().and_then(|o| metiers::calculateur(o.id))
    }

    /// Le contenu des champs de l'outil.
    pub fn saisies_d_outil(&self) -> Option<&[String]> {
        self.outil.as_ref().map(|o| o.saisies.as_slice())
    }

    /// Les touches des champs de l'outil, dans l'ordre des champs.
    pub fn touches_d_outil(&self) -> Option<Vec<ToucheOutil>> {
        let outil = self.outil.as_ref()?;
        let c = metiers::calculateur(outil.id)?;
        let touches = c
            .champs
            .iter()
            .take(CHAMPS_D_OUTIL)
            .enumerate()
            .map(|(rang, champ)| {
                let saisie = outil.saisies.get(rang).map_or("", |s| s.trim());
                let valeur = if saisie.is_empty() {
                    String::new()
                } else {
                    champ
                        .lire(saisie, &self.prefs)
                        .map_or_else(|_| saisie.to_string(), |lecture| lecture.texte)
                };
                ToucheOutil {
                    libelle: champ.libelle_court(),
                    nom: champ.libelle,
                    valeur,
                    unite: champ.unite(),
                    choix: matches!(champ.nature, Nature::Choix(_)),
                    facultatif: champ.facultatif,
                }
            })
            .collect();
        Some(touches)
    }

    /// Remet les champs de l'outil à leurs valeurs de départ.
    pub(super) fn vider_l_outil(&mut self) {
        if let Some(c) = self.outil()
            && let Some(outil) = self.outil.as_mut()
        {
            let depart = c.saisies_par_defaut();
            if outil.saisies != depart {
                outil.saisies = depart;
                self.revision += 1;
            }
            outil.rang = None;
        }
    }

    /// Toute autre touche que `Résultat` met fin au défilement des résultats.
    pub(super) fn interrompre_le_defilement(&mut self, touche: Touche) {
        if touche != Touche::Outil(TOUCHE_RESULTAT)
            && let Some(outil) = self.outil.as_mut()
        {
            outil.rang = None;
        }
    }

    fn saisie_d_outil(&self, rang: usize) -> String {
        self.outil
            .as_ref()
            .and_then(|o| o.saisies.get(rang))
            .cloned()
            .unwrap_or_default()
    }

    fn ecrire_saisie_d_outil(&mut self, rang: usize, texte: String) {
        if let Some(saisie) = self.outil.as_mut().and_then(|o| o.saisies.get_mut(rang))
            && *saisie != texte
        {
            *saisie = texte;
            self.revision += 1;
        }
    }

    /// Montre un message d'outil : il n'est pas bloquant, la touche suivante est acceptée.
    fn montrer_message_d_outil(&mut self, message: Message) {
        self.saisie.vider();
        self.message = Some(message);
        self.provenance = Provenance::Message;
        self.courant = Valeur::nombre(0.0);
        self.presentation = Presentation::default();
        self.cycle = None;
        self.rappel_de = None;
    }

    /// Montre une valeur d'outil : un libellé libre et, au besoin, un texte
    /// libre et une mention à côté du libellé.
    fn montrer_outil(
        &mut self,
        libelle: &'static str,
        valeur: Valeur,
        (texte, note): (Option<String>, Option<String>),
        memorise: bool,
    ) {
        self.saisie.vider();
        self.courant = valeur;
        self.provenance = Provenance::Sortie;
        self.presentation = Presentation {
            libre: Some(libelle),
            memorise,
            ..Presentation::default()
        };
        self.texte_libre = texte;
        self.note_libre = note;
        self.message = None;
        self.cycle = None;
        self.rappel_de = None;
        self.operande_pret = true;
    }

    /// Montre le contenu d'un champ. Une longueur s'écrit comme toute longueur
    /// de l'afficheur (fractions empilées); le reste s'écrit avec son unité.
    fn montrer_champ(&mut self, champ: &Champ, lecture: Lecture, touche: Touche) {
        let (valeur, texte) = match lecture.lue {
            Lue::Longueur(pouces) => (
                Valeur::en_base(pouces, lecture.format.unwrap_or(Format::PoFrac)),
                None,
            ),
            Lue::Nombre(x) => (Valeur::nombre(x), Some(lecture.texte)),
            Lue::Choix(_) | Lue::Vide => (Valeur::nombre(0.0), Some(lecture.texte)),
        };
        self.montrer_outil(champ.libelle, valeur, (texte, None), true);
        // Une seconde pression rappelle le champ au lieu de le réécrire (R-37).
        self.rappel_de = Some(touche);
    }

    /// Le texte à ranger dans un champ pour la valeur de l'afficheur.
    ///
    /// Une longueur est rangée telle que l'afficheur l'écrit. Un nombre est
    /// rangé avec toutes ses décimales, sans séparateur de milliers : le champ
    /// le relit donc sans rien perdre, quelles que soient les préférences.
    fn texte_pour_le_champ(&self, champ: &Champ, valeur: &Valeur) -> Result<String, Erreur> {
        let longueur_attendue = matches!(champ.nature, Nature::Longueur(_));
        if longueur_attendue && valeur.est_longueur() {
            return Ok(self.texte_de(valeur, Rendu::Normal));
        }
        // Un nombre seul prend l'unité du champ; une aire ou une masse va dans
        // un champ de la même unité (des pieds carrés, par exemple).
        let meme_unite = !longueur_attendue
            && !valeur.est_longueur()
            && !champ.unite().is_empty()
            && symbole(valeur.format) == champ.unite();
        if !valeur.est_sans_unite() && !meme_unite {
            return Err(Erreur::Unites);
        }
        let x = valeur.numerique();
        if !x.is_finite() {
            return Err(Erreur::Calcul);
        }
        let mut texte = format!("{x:.9}");
        if texte.contains('.') {
            texte.truncate(texte.trim_end_matches('0').trim_end_matches('.').len());
        }
        if self.prefs.separateur_decimal == SeparateurDecimal::Virgule {
            texte = texte.replace('.', ",");
        }
        Ok(texte)
    }

    /// Traite une touche d'outil. Sans outil branché, la touche ne fait rien.
    pub(super) fn touche_outil(&mut self, rang: u8) -> Result<(), Erreur> {
        let touche = Touche::Outil(rang);
        let Some(c) = self.outil() else {
            return Ok(());
        };
        if rang == TOUCHE_RESULTAT {
            return self.resultats_d_outil(c);
        }
        let rang = usize::from(rang);
        let Some(champ) = c.champs.get(rang) else {
            return Ok(());
        };
        if let Nature::Choix(options) = champ.nature {
            // Chaque pression passe à l'option suivante.
            if options.is_empty() {
                return Ok(());
            }
            let courant = self
                .saisie_d_outil(rang)
                .trim()
                .parse::<usize>()
                .unwrap_or(0);
            let suivant = (courant + 1) % options.len();
            self.ecrire_saisie_d_outil(rang, suivant.to_string());
            self.montrer_choix(champ, options, suivant);
            return Ok(());
        }
        match self.donnee_libre(touche)? {
            // Une valeur vient d'être tapée ou calculée : elle va dans le champ.
            Some(valeur) => {
                let texte = self.texte_pour_le_champ(champ, &valeur)?;
                let lecture = champ
                    .lire(&texte, &self.prefs)
                    .map_err(|_| Erreur::Saisie)?;
                self.ecrire_saisie_d_outil(rang, texte);
                self.montrer_champ(champ, lecture, touche);
            }
            // Sinon, la touche rappelle le contenu du champ.
            None => {
                let texte = self.saisie_d_outil(rang);
                if texte.trim().is_empty() {
                    return Err(Erreur::AucuneValeur);
                }
                match champ.lire(texte.trim(), &self.prefs) {
                    Ok(lecture) => self.montrer_champ(champ, lecture, touche),
                    // Le formulaire peut avoir laissé une saisie à corriger.
                    Err(_) => self
                        .montrer_message_d_outil(Message::ACorriger(champ.libelle_pour_message())),
                }
            }
        }
        Ok(())
    }

    /// Montre l'option choisie dans une liste, avec son rang.
    fn montrer_choix(&mut self, champ: &Champ, options: &[&'static str], rang: usize) {
        let texte = options.get(rang).copied().unwrap_or("").to_string();
        let note = rang_parmi(rang + 1, options.len());
        // Le rang d'une option n'est pas une valeur : rien à reprendre dans un calcul.
        self.montrer_outil(
            champ.libelle,
            Valeur::nombre(0.0),
            (Some(texte), Some(note)),
            true,
        );
    }

    /// `Conv` puis une touche d'outil : le champ reprend sa valeur de départ;
    /// `Conv` puis `Résultat` : tous les champs la reprennent.
    pub(super) fn touche_outil_conv(&mut self, rang: u8) -> Result<(), Erreur> {
        let Some(c) = self.outil() else {
            return Ok(());
        };
        if rang == TOUCHE_RESULTAT {
            self.vider_l_outil();
            self.montrer_message_d_outil(Message::Information(Information::OutilRecommence));
            return Ok(());
        }
        let touche = Touche::Outil(rang);
        let rang = usize::from(rang);
        let Some(champ) = c.champs.get(rang) else {
            return Ok(());
        };
        self.ecrire_saisie_d_outil(rang, champ.defaut.to_string());
        if champ.defaut.is_empty() {
            self.montrer_message_d_outil(Message::Information(Information::ChampVide));
        } else if let Nature::Choix(options) = champ.nature {
            let rang_du_choix = champ.defaut.parse::<usize>().unwrap_or(0);
            self.montrer_choix(champ, options, rang_du_choix);
        } else if let Ok(lecture) = champ.lire(champ.defaut, &self.prefs) {
            self.montrer_champ(champ, lecture, touche);
        }
        Ok(())
    }

    /// La touche `Résultat` : calcule, inscrit le calcul au ruban, puis fait
    /// défiler les résultats à chaque pression.
    fn resultats_d_outil(&mut self, c: &'static Calculateur) -> Result<(), Erreur> {
        let Some(outil) = self.outil.as_ref() else {
            return Ok(());
        };
        let deja_montre = outil.rang;
        let evaluation = c.evaluer(&outil.saisies, &self.prefs);
        // Une saisie refusée, laissée par le formulaire.
        if let Some(rang) = evaluation.erreurs.iter().position(Option::is_some) {
            let libelle = c.champs.get(rang).map_or("", Champ::libelle_pour_message);
            self.montrer_message_d_outil(Message::ACorriger(libelle));
            return Ok(());
        }
        if let Some(avis) = evaluation.avis {
            self.montrer_message_d_outil(Message::Avis(avis));
            return Ok(());
        }
        if evaluation.resultats.is_empty() {
            // Il manque une valeur : on dit laquelle.
            let manquant = c.champs.iter().zip(&outil.saisies).find(|(champ, saisie)| {
                !champ.facultatif
                    && !matches!(champ.nature, Nature::Choix(_))
                    && saisie.trim().is_empty()
            });
            return match manquant {
                Some((champ, _)) => {
                    self.montrer_message_d_outil(Message::Manque(champ.libelle_pour_message()));
                    Ok(())
                }
                // Tous les champs sont remplis, mais aucun résultat n'est affichable.
                None => Err(Erreur::Depassement),
            };
        }
        let nombre = evaluation.resultats.len();
        let rang = match deja_montre {
            Some(precedent) => (precedent + 1) % nombre,
            None => {
                self.consigner(c.nom, &evaluation.entrees, &evaluation.resultats);
                0
            }
        };
        if let Some(outil) = self.outil.as_mut() {
            outil.rang = Some(rang);
        }
        let (Some((libelle, _)), Some(quantite)) = (
            evaluation.resultats.get(rang),
            evaluation.quantites.get(rang),
        ) else {
            return Err(Erreur::Calcul);
        };
        // « 2 de 3 » : il reste des résultats à voir.
        let note = (nombre > 1).then(|| rang_parmi(rang + 1, nombre));
        self.montrer_outil(
            libelle,
            quantite.valeur(evaluation.modele),
            (quantite.texte_libre(evaluation.modele, &self.prefs), note),
            false,
        );
        Ok(())
    }
}
