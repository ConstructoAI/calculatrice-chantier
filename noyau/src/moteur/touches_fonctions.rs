//! Touches de fonction : « saisir ou calculer » (section 9.8), et raccord entre
//! le moteur et les fonctions pures de la section 12.

use super::Moteur;
use crate::erreurs::Erreur;
use crate::etiquettes::Etiquette;
use crate::fonctions::Sortie;
use crate::fonctions::arc::{self, ArcSaisi, DonneesArc};
use crate::fonctions::aretier::calculer_aretier;
use crate::fonctions::colonne_cone::calculer_colonne_cone;
use crate::fonctions::empannons::{DonneesEmpannons, calculer_empannons};
use crate::fonctions::escalier::{DonneesEscalier, calculer_escalier};
use crate::fonctions::materiaux::{
    DonneeMateriau, calculer_blocs, calculer_cout, calculer_gypse, calculer_montants,
    calculer_semelle,
};
use crate::fonctions::mur_pente::{DonneesMurPente, calculer_mur_pente};
use crate::fonctions::onglet::{calculer_onglet, controler_angle_moulure, interpreter_coin};
use crate::fonctions::polygone::{calculer_polygone, interpreter_cotes};
use crate::fonctions::toiture::{DonneesToiture, calculer_toiture};
use crate::fonctions::triangle::{
    Cote, PenteMemorisee, controler_longueur, cycle_pente, interpreter_pente, valeur_pente,
};
use crate::fonctions::trigo::{FonctionTrigo, calculer_trigo};
use crate::touches::Touche;
use crate::unites::Genre;
use crate::valeur::{Operateur, Valeur, multiplier};

impl Moteur {
    /// Traite une touche de fonction, principale ou secondaire (R-35 à R-37).
    pub(super) fn touche_fonction(
        &mut self,
        touche: Touche,
        secondaire: bool,
    ) -> Result<(), Erreur> {
        // Mode trigonométrique : Longueur, Largeur, Hauteur deviennent sin, cos, tan.
        if self.prefs.mode_trigo {
            let fonction = match (touche, secondaire) {
                (Touche::Longueur, false) => Some(FonctionTrigo::Sin),
                (Touche::Largeur, false) => Some(FonctionTrigo::Cos),
                (Touche::Hauteur, false) => Some(FonctionTrigo::Tan),
                (Touche::Longueur, true) => Some(FonctionTrigo::ArcSin),
                (Touche::Largeur, true) => Some(FonctionTrigo::ArcCos),
                (Touche::Hauteur, true) => Some(FonctionTrigo::ArcTan),
                _ => None,
            };
            if let Some(f) = fonction {
                let v = self.valeur_montree()?;
                let resultat = calculer_trigo(f, &v)?;
                self.montrer_resultat(resultat);
                return Ok(());
            }
        }

        let donnee = self.donnee_libre(touche)?;
        // Sans donnée, la même touche fait défiler le cycle en cours (R-27). Une
        // fonction lancée avec `Conv` se poursuit sans `Conv` si elle a plusieurs sorties.
        if donnee.is_none() {
            let poursuivre = self.cycle.as_ref().is_some_and(|c| {
                c.touche == touche
                    && (c.secondaire == secondaire || (c.secondaire && c.sorties.len() > 1))
            });
            if poursuivre {
                return self.avancer_cycle();
            }
        }

        match (touche, secondaire) {
            (Touche::Pente, _) => self.f_pente(donnee, secondaire),
            (Touche::Montee, false) => self.f_cote(Cote::Montee, donnee),
            (Touche::Course, false) => self.f_cote(Cote::Course, donnee),
            (Touche::Diag, false) => self.f_diag(donnee),
            (Touche::Montee, true) => self.f_mur_pente(donnee),
            (Touche::Course, true) => self.f_polygone(donnee),
            (Touche::Diag, true) => self.f_toiture(donnee),
            (Touche::AretierNoue, false) => {
                let sorties = calculer_aretier(
                    &self.triangle.resoudre()?,
                    self.toit.pente_irreguliere.map(|p| p.0),
                )?;
                self.lancer_cycle(touche, false, sorties)
            }
            (Touche::AretierNoue, true) => self.f_pente_irreguliere(donnee),
            (Touche::OngletCompose, false) => self.f_onglet(donnee),
            (Touche::OngletCompose, true) => self.f_angle_moulure(donnee),
            (Touche::Escalier, _) => {
                let donnees = DonneesEscalier {
                    montee: self.triangle.montee.map(|x| x.0),
                    course: self.triangle.course.map(|x| x.0),
                    contremarche_visee: self.memo.contremarche_visee,
                    giron_vise: self.memo.giron_vise,
                    echappee: self.memo.echappee,
                    epaisseur_plancher: self.memo.epaisseur_plancher,
                    contremarche_max: secondaire,
                    resolution: self.resolution(),
                };
                let sorties = calculer_escalier(&donnees)?;
                self.lancer_cycle(touche, secondaire, sorties)
            }
            (Touche::Arc, false) => self.f_arc(donnee),
            (Touche::Arc, true) => self.f_rayon(donnee),
            (Touche::Cercle, false) => self.f_cercle(donnee),
            (Touche::Cercle, true) => {
                let diametre = self.cercle.diametre.ok_or(Erreur::AucuneValeur)?;
                // En mode trigonométrique, la hauteur se lit dans Montée (12.18).
                let hauteur = if self.prefs.mode_trigo {
                    self.triangle.montee.map(|x| x.0)
                } else {
                    self.dimensions.hauteur
                };
                let hauteur = hauteur.ok_or(Erreur::AucuneValeur)?;
                let sorties = calculer_colonne_cone(&diametre, &hauteur, &self.prefs)?;
                self.lancer_cycle(touche, true, sorties)
            }
            (Touche::Empannon, _) => {
                let donnees = DonneesEmpannons {
                    triangle: self.triangle.resoudre()?,
                    pente_irreguliere: self.toit.pente_irreguliere.map(|p| p.0),
                    entraxe: self.memo.entraxe,
                    ordre: self.prefs.empannons,
                    mode_irregulier: self.prefs.empannons_irreguliers,
                    irregulier_d_abord: secondaire,
                };
                let sorties = calculer_empannons(&donnees)?;
                self.lancer_cycle(touche, secondaire, sorties)
            }
            (Touche::Longueur | Touche::Largeur | Touche::Hauteur, false) => {
                self.f_dimension(touche, donnee)
            }
            (Touche::Longueur, true) => {
                let materiau = self.donnee_materiau(donnee)?;
                let sorties =
                    calculer_blocs(&materiau, &self.memo.longueur_bloc, &self.memo.aire_bloc)?;
                self.lancer_cycle(touche, true, sorties)
            }
            (Touche::Largeur, true) => {
                let longueur = self.longueur_libre_ou_registre(donnee)?;
                let sorties = calculer_semelle(&longueur, &self.memo.section_semelle, &self.prefs)?;
                self.lancer_cycle(touche, true, sorties)
            }
            (Touche::Hauteur, true) => {
                let materiau = self.donnee_materiau(donnee)?;
                let sorties = calculer_gypse(&materiau)?;
                self.lancer_cycle(touche, true, sorties)
            }
            (Touche::Chiffre(5), true) => {
                // Montants : sortie unique, jamais de cycle sur un chiffre (12.15).
                let longueur = self.longueur_libre_ou_registre(donnee)?;
                let sortie = calculer_montants(&longueur, &self.memo.entraxe)?;
                self.montrer_sortie_unique(&sortie);
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Coût (section 12.17) : exige une multiplication en attente, quantité × prix.
    pub(super) fn cout(&mut self) -> Result<(), Erreur> {
        let prix = self.valeur_montree()?;
        match self.operation {
            Some((quantite, Operateur::Multiplier)) if self.operande_pret => {
                let sortie = calculer_cout(&quantite, &prix)?;
                let texte = self.texte_montre();
                self.ruban_operande(Some(Operateur::Multiplier.symbole()), texte);
                self.operation = None;
                self.montrer_sortie_unique(&sortie);
                self.operande_pret = false;
                Ok(())
            }
            _ => Err(Erreur::Saisie),
        }
    }

    // ------------------------------------------------------------------
    // Triangle (section 12.1)
    // ------------------------------------------------------------------

    fn f_pente(&mut self, donnee: Option<Valeur>, comme_ratio: bool) -> Result<(), Erreur> {
        if let Some(d) = donnee {
            let (ratio, forme) = interpreter_pente(&d, comme_ratio)?;
            self.compteur += 1;
            self.triangle.pente = Some(PenteMemorisee {
                ratio,
                origine: forme,
                ordre: self.compteur,
            });
            self.revision += 1;
            let (etiquette, valeur) = valeur_pente(ratio, forme);
            let cycle = cycle_pente(ratio, forme, false);
            self.montrer_donnee(
                Touche::Pente,
                false,
                Sortie::nouvelle(etiquette, valeur),
                Ok(cycle),
            );
            return Ok(());
        }
        let (ratio, saisie, forme) = self.triangle.pente_en_vigueur()?;
        let depart = if comme_ratio {
            crate::fonctions::triangle::FormePente::Ratio
        } else {
            forme
        };
        self.lancer_cycle(Touche::Pente, false, cycle_pente(ratio, depart, saisie))
    }

    /// Sortie de la touche Montée ou Course, sans donnée.
    fn cycle_cote(&self, cote: Cote) -> Result<Vec<Sortie>, Erreur> {
        // En présence d'un diamètre : flèche ou corde de l'arc (R-43).
        if let Some(diametre) = &self.cercle.diametre {
            match (cote, self.triangle.montee, self.triangle.course) {
                (Cote::Montee, None, Some((corde, _))) => {
                    let fleche = arc::fleche_depuis_corde(diametre, &corde)?;
                    return Ok(vec![Sortie::nouvelle(Etiquette::Fleche, fleche)]);
                }
                (Cote::Course, Some((fleche, _)), None) => {
                    let corde = arc::corde_depuis_fleche(diametre, &fleche)?;
                    return Ok(vec![Sortie::nouvelle(Etiquette::Corde, corde)]);
                }
                _ => {}
            }
        }
        Ok(vec![self.triangle.sortie_cote(cote)?])
    }

    fn f_cote(&mut self, cote: Cote, donnee: Option<Valeur>) -> Result<(), Erreur> {
        let (touche, etiquette) = match cote {
            Cote::Montee => (Touche::Montee, Etiquette::Montee),
            _ => (Touche::Course, Etiquette::Course),
        };
        if let Some(d) = donnee {
            let v = controler_longueur(&d)?;
            self.compteur += 1;
            match cote {
                Cote::Montee => self.triangle.montee = Some((v, self.compteur)),
                _ => self.triangle.course = Some((v, self.compteur)),
            }
            let cycle = self.cycle_cote(cote);
            self.montrer_donnee(touche, false, Sortie::nouvelle(etiquette, v), cycle);
            return Ok(());
        }
        let sorties = self.cycle_cote(cote)?;
        self.lancer_cycle(touche, false, sorties)
    }

    fn f_diag(&mut self, donnee: Option<Valeur>) -> Result<(), Erreur> {
        if let Some(d) = donnee {
            let v = controler_longueur(&d)?;
            self.compteur += 1;
            self.triangle.diag = Some((v, self.compteur));
            let cycle = self.triangle.cycle_diag();
            self.montrer_donnee(
                Touche::Diag,
                false,
                Sortie::nouvelle(Etiquette::Diagonale, v),
                cycle,
            );
            return Ok(());
        }
        let sorties = self.triangle.cycle_diag()?;
        self.lancer_cycle(Touche::Diag, false, sorties)
    }

    // ------------------------------------------------------------------
    // Toit (sections 12.2, 12.4 et 12.5)
    // ------------------------------------------------------------------

    fn f_pente_irreguliere(&mut self, donnee: Option<Valeur>) -> Result<(), Erreur> {
        if let Some(d) = donnee {
            self.toit.pente_irreguliere = Some(interpreter_pente(&d, false)?);
        }
        let (ratio, forme) = self.toit.pente_irreguliere.ok_or(Erreur::AucuneValeur)?;
        let (_, valeur) = valeur_pente(ratio, forme);
        let sortie = Sortie::nouvelle(Etiquette::PenteIrreguliere, valeur);
        if donnee.is_some() {
            self.montrer_donnee(
                Touche::AretierNoue,
                true,
                sortie,
                Ok(vec![sortie.memorisee()]),
            );
            Ok(())
        } else {
            self.lancer_cycle(Touche::AretierNoue, true, vec![sortie.memorisee()])
        }
    }

    fn calculer_mur(&self) -> Result<Vec<Sortie>, Erreur> {
        calculer_mur_pente(&DonneesMurPente {
            triangle: self.triangle.resoudre()?,
            entraxe: self.memo.entraxe,
            base: self.toit.base,
            ordre: self.prefs.mur_pente,
        })
    }

    fn f_mur_pente(&mut self, donnee: Option<Valeur>) -> Result<(), Erreur> {
        if let Some(d) = donnee {
            // Une longueur à l'écran devient la base du mur (12.4).
            if d.genre != Genre::Longueur {
                return Err(Erreur::Unites);
            }
            if !(d.base.is_finite() && d.base >= 0.0) {
                return Err(Erreur::Saisie);
            }
            self.toit.base = Some(d);
            let cycle = self.calculer_mur();
            self.montrer_donnee(
                Touche::Montee,
                true,
                Sortie::nouvelle(Etiquette::Base, d),
                cycle,
            );
            return Ok(());
        }
        let sorties = self.calculer_mur()?;
        self.lancer_cycle(Touche::Montee, true, sorties)
    }

    fn f_toiture(&mut self, donnee: Option<Valeur>) -> Result<(), Erreur> {
        // Pente : dérivée des deux longueurs les plus récentes, sinon la pente saisie.
        let (ratio, pente_saisie) = if self.triangle.paire_de_longueurs() {
            (self.triangle.resoudre()?.ratio, false)
        } else {
            match &self.triangle.pente {
                Some(p) => (p.ratio, true),
                None => return Err(Erreur::AucuneValeur),
            }
        };
        // Aire en plan : une aire en donnée, sinon longueur × largeur.
        let aire_plan = match donnee {
            Some(d) if d.est_aire() => d,
            _ => match (self.dimensions.longueur, self.dimensions.largeur) {
                (Some(l), Some(w)) => multiplier(&l, &w, &self.prefs)?,
                _ => return Err(Erreur::AucuneValeur),
            },
        };
        let sorties = calculer_toiture(&DonneesToiture {
            ratio,
            pente_saisie,
            aire_plan,
            paquets_par_carre: self.memo.paquets_par_carre,
        })?;
        self.lancer_cycle(Touche::Diag, true, sorties)
    }

    // ------------------------------------------------------------------
    // Onglet composé (section 12.7)
    // ------------------------------------------------------------------

    fn f_onglet(&mut self, donnee: Option<Valeur>) -> Result<(), Erreur> {
        if let Some(d) = donnee {
            self.divers.coin = Some(interpreter_coin(&d)?);
        }
        let coin = self.divers.coin.ok_or(Erreur::AucuneValeur)?;
        let sorties = calculer_onglet(coin, self.memo.angle_moulure)?;
        self.lancer_cycle(Touche::OngletCompose, false, sorties)
    }

    fn f_angle_moulure(&mut self, donnee: Option<Valeur>) -> Result<(), Erreur> {
        if let Some(d) = donnee {
            self.memo.angle_moulure = controler_angle_moulure(&d)?;
            self.revision += 1;
        }
        let sortie = Sortie::angle(Etiquette::AngleMoulure, self.memo.angle_moulure).memorisee();
        if donnee.is_some() {
            self.montrer_donnee(Touche::OngletCompose, true, sortie, Ok(vec![sortie]));
            Ok(())
        } else {
            self.lancer_cycle(Touche::OngletCompose, true, vec![sortie])
        }
    }

    // ------------------------------------------------------------------
    // Cercle, arc, polygone (sections 12.8 et 12.10)
    // ------------------------------------------------------------------

    fn donnees_arc(&self) -> DonneesArc {
        DonneesArc {
            diametre: self.cercle.diametre,
            arc: self.cercle.arc,
            corde: self.triangle.course.map(|x| x.0),
            fleche: self.triangle.montee.map(|x| x.0),
        }
    }

    fn calculer_arc(&self) -> Result<Vec<Sortie>, Erreur> {
        self.donnees_arc().cycle_arc(
            &self.memo.entraxe,
            self.toit.base,
            self.prefs.mur_cintre,
            &self.prefs,
        )
    }

    fn sortie_arc_saisi(saisi: ArcSaisi) -> Sortie {
        match saisi {
            ArcSaisi::Angle(a) => Sortie::angle(Etiquette::Arc, a),
            ArcSaisi::Longueur(l) => Sortie::nouvelle(Etiquette::Arc, l),
        }
    }

    fn f_arc(&mut self, donnee: Option<Valeur>) -> Result<(), Erreur> {
        if let Some(d) = donnee {
            let saisi = arc::interpreter_arc(&d)?;
            self.cercle.arc = Some(saisi);
            let cycle = self.calculer_arc();
            self.montrer_donnee(Touche::Arc, false, Self::sortie_arc_saisi(saisi), cycle);
            // Une longueur d'arc en donnée vient d'être montrée : elle est sautée
            // au premier tour, après l'angle.
            if let (ArcSaisi::Longueur(_), Some(c)) = (saisi, self.cycle.as_mut())
                && c.sorties.len() > 1
            {
                c.reprendre_apres(1);
                c.index = 0;
                c.a_sauter = Some(1);
            }
            return Ok(());
        }
        match self.calculer_arc() {
            Ok(sorties) => self.lancer_cycle(Touche::Arc, false, sorties),
            // Arc non résolu : la touche réaffiche seulement la donnée.
            Err(Erreur::AucuneValeur) => {
                let saisi = self.cercle.arc.ok_or(Erreur::AucuneValeur)?;
                self.lancer_cycle(
                    Touche::Arc,
                    false,
                    vec![Self::sortie_arc_saisi(saisi).memorisee()],
                )
            }
            Err(e) => Err(e),
        }
    }

    fn f_rayon(&mut self, donnee: Option<Valeur>) -> Result<(), Erreur> {
        if let Some(d) = donnee {
            let rayon = arc::controler_longueur(&d)?;
            self.cercle.diametre = Some(Valeur {
                base: rayon.base * 2.0,
                ..rayon
            });
            let sortie = Sortie::nouvelle(Etiquette::Rayon, rayon);
            self.montrer_donnee(Touche::Arc, true, sortie, Ok(vec![sortie.memorisee()]));
            return Ok(());
        }
        let sortie = self.donnees_arc().sortie_rayon()?;
        self.lancer_cycle(Touche::Arc, true, vec![sortie])
    }

    fn f_cercle(&mut self, donnee: Option<Valeur>) -> Result<(), Erreur> {
        if let Some(d) = donnee {
            let diametre = arc::controler_longueur(&d)?;
            self.cercle.diametre = Some(diametre);
            let cycle = self.donnees_arc().cycle_cercle(&self.prefs);
            self.montrer_donnee(
                Touche::Cercle,
                false,
                Sortie::nouvelle(Etiquette::Diametre, diametre),
                cycle,
            );
            return Ok(());
        }
        let sorties = self.donnees_arc().cycle_cercle(&self.prefs)?;
        self.lancer_cycle(Touche::Cercle, false, sorties)
    }

    fn f_polygone(&mut self, donnee: Option<Valeur>) -> Result<(), Erreur> {
        if let Some(d) = donnee {
            self.divers.nb_cotes = Some(interpreter_cotes(&d)?);
        }
        let n = self.divers.nb_cotes.ok_or(Erreur::AucuneValeur)?;
        let diametre = self.cercle.diametre.ok_or(Erreur::AucuneValeur)?;
        let rayon = Valeur {
            base: diametre.base / 2.0,
            ..diametre
        };
        let sorties = calculer_polygone(n, &rayon, &self.prefs)?;
        self.lancer_cycle(Touche::Course, true, sorties)
    }

    // ------------------------------------------------------------------
    // Dimensions et matériaux (sections 12.11 à 12.15)
    // ------------------------------------------------------------------

    fn cycle_dimension(&self, touche: Touche) -> Result<Vec<Sortie>, Erreur> {
        match touche {
            Touche::Longueur => self.dimensions.cycle_longueur(),
            Touche::Largeur => self.dimensions.cycle_largeur(&self.prefs),
            _ => self.dimensions.cycle_hauteur(&self.prefs),
        }
    }

    fn f_dimension(&mut self, touche: Touche, donnee: Option<Valeur>) -> Result<(), Erreur> {
        if let Some(d) = donnee {
            let v = controler_longueur(&d)?;
            let etiquette = match touche {
                Touche::Longueur => {
                    self.dimensions.longueur = Some(v);
                    Etiquette::Longueur
                }
                Touche::Largeur => {
                    self.dimensions.largeur = Some(v);
                    Etiquette::Largeur
                }
                _ => {
                    self.dimensions.hauteur = Some(v);
                    Etiquette::Hauteur
                }
            };
            let cycle = self.cycle_dimension(touche);
            self.montrer_donnee(touche, false, Sortie::nouvelle(etiquette, v), cycle);
            return Ok(());
        }
        let sorties = self.cycle_dimension(touche)?;
        self.lancer_cycle(touche, false, sorties)
    }

    /// Donnée d'un calcul de blocs ou de gypse, par ordre de priorité (12.12, 12.14) :
    /// une longueur ou une aire libre à l'écran, sinon les registres.
    fn donnee_materiau(&self, donnee: Option<Valeur>) -> Result<DonneeMateriau, Erreur> {
        match donnee {
            Some(d) if d.est_longueur() => return Ok(DonneeMateriau::Lineaire(d)),
            Some(d) if d.est_aire() => return Ok(DonneeMateriau::Surface(d)),
            _ => {}
        }
        match (self.dimensions.longueur, self.dimensions.hauteur) {
            (Some(l), Some(h)) => Ok(DonneeMateriau::Surface(multiplier(&l, &h, &self.prefs)?)),
            (Some(l), None) => Ok(DonneeMateriau::Lineaire(l)),
            _ => Err(Erreur::AucuneValeur),
        }
    }

    /// Longueur libre à l'écran, sinon le registre longueur (12.13, 12.15).
    fn longueur_libre_ou_registre(&self, donnee: Option<Valeur>) -> Result<Valeur, Erreur> {
        match donnee {
            Some(d) if d.est_longueur() => Ok(d),
            _ => self.dimensions.longueur.ok_or(Erreur::AucuneValeur),
        }
    }
}
