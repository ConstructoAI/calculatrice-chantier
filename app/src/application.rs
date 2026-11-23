//! L'application : fenêtre, navigation, dialogues, clavier d'ordinateur et
//! sauvegarde. Aucune formule ici (R-49) : tout le calcul est dans le noyau.

use std::collections::HashMap;
use std::time::Duration;

use chrono::{Datelike as _, Timelike as _};

use eframe::egui::{self, Align2, Color32, Painter, Pos2, Rect, RichText, Sense, Stroke, Ui, Vec2};
use noyau::analyse::analyser_collage;
use noyau::langue::{Langue, Texte};
use noyau::metiers::Metier;
use noyau::prefs::{Preferences, Theme, ValeursMemorisees};
use noyau::unites::UniteMasseVolumique;
use noyau::{Effet, Moteur, Prefixe, Touche};

use crate::afficheur::{self, ActionAfficheur};
use crate::aide;
use crate::clavier::{self, ActionClavier, Contexte, EtatClavier};
use crate::pieces;
use crate::plateforme::{
    self, Arrivee, ChoixDeFichiers, Depot, Export, Fichier, FichierChoisi, Installation,
};
use crate::raccourcis::{self, Action};
use crate::stockage::{
    self, DessinEnregistre, MAX_DESSINS, MAX_NOTES, MAX_RUBANS, NoteDuBloc, PieceJointe,
    RubanEnregistre,
};
use crate::textes;
use crate::theme::{self, Palette};
use crate::vues::preferences::Memorisee;
use crate::vues::{self, notes, rubans};

/// Taille nominale de la calculatrice, en points logiques (section 16.1).
const LARGEUR_NOMINALE: f32 = 420.0;
const HAUTEUR_NOMINALE: f32 = 860.0;
/// Largeur à partir de laquelle le ruban s'affiche à droite.
const LARGEUR_LARGE: f32 = 880.0;
/// Largeur maximale de la calculatrice dans une fenêtre large.
const LARGEUR_PLAFOND: f32 = 520.0;
const HAUTEUR_BARRE_SUPERIEURE: f32 = 48.0;
const HAUTEUR_BARRE_INFERIEURE: f32 = 56.0;
/// Hauteur de la barre des métiers, au-dessus du clavier (section 23).
const HAUTEUR_BARRE_DES_METIERS: f32 = 34.0;
/// Largeur du bouton de langue « FR | EN », dans la barre supérieure (section 24).
const LARGEUR_BOUTON_DE_LANGUE: f32 = 60.0;
/// Durée d'affichage des confirmations discrètes, en secondes.
const DUREE_FUGACE: f64 = 1.8;

/// Fonction qui dessine un pictogramme dans un rectangle, d'une couleur donnée.
type Dessin = fn(&Painter, Rect, Color32);

/// Écran affiché.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Vue {
    Accueil,
    /// Calculateurs par métier (section 22).
    Metiers,
    /// Dessin d'une pièce (section 25).
    Dessin,
    Rubans,
    /// Le bloc-notes (section 26).
    Notes,
    Aide,
    Preferences,
    Bienvenue,
    Trucs,
    APropos,
}

/// Entrée du menu ☰.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ChoixMenu {
    Aller(Vue),
    /// Ouvrir le manuel d'utilisation dans le navigateur.
    Manuel,
    /// Installer l'application sur l'appareil (version Web).
    Installer,
    /// Réunir toutes les données en un fichier (section 17.4).
    ExporterDonnees,
    /// Remplacer les données par celles d'une sauvegarde.
    ImporterDonnees,
    /// Passer au mode chantier, ou le quitter (section 16.7).
    ModeChantier,
    Quitter,
}

/// Question posée à l'utilisateur avant une action irréversible.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Confirmation {
    EffacerTout,
    Reinitialiser,
    ViderRuban,
    SupprimerRuban(u64),
    SupprimerNote(u64),
    /// Retirer une pièce jointe de la note ouverte : son numéro.
    RetirerPiece(u64),
    /// Supprimer un dessin de la bibliothèque.
    SupprimerDessin(u64),
    /// Remplacer les données par celles de la sauvegarde choisie.
    Restaurer,
    /// Limite de rubans atteinte : supprimer les plus anciens pour enregistrer celui-ci.
    TropDeRubans(String),
}

/// Pourquoi une pièce jointe a été demandée à la plateforme.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Demande {
    /// La vignette d'une photo, pour la note ouverte.
    Vignette(u64),
    /// La photo, pour la voir en grand.
    Grande(u64),
    /// Les octets, pour les enregistrer ailleurs.
    Enregistrer(u64),
    /// La photo, pour l'exporter en PDF avec ses cotes.
    Pdf(u64),
    /// Une pièce jointe, pour la sauvegarde complète.
    Sauvegarde,
}

/// À quoi servent les fichiers que l'utilisateur est en train de choisir.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Destination {
    /// Les joindre à cette note.
    Note(u64),
    /// Les rouvrir comme des dessins.
    Dessins,
    /// Importer une sauvegarde complète.
    Sauvegarde,
}

/// Une sauvegarde complète qui attend ses pièces jointes, lues une à une.
struct SauvegardeEnCours {
    fichiers: std::collections::BTreeMap<String, serde_json::Value>,
    pieces: std::collections::BTreeMap<String, Vec<u8>>,
    attendues: usize,
}

/// Dialogue ouvert par-dessus l'écran.
enum Dialogue {
    Aucun,
    Avis(Texte),
    Confirmer(Confirmation),
    NommerRuban {
        texte: String,
        renommer: Option<u64>,
        /// Premier affichage : le champ prend le focus.
        nouveau: bool,
    },
    /// Renommer un dessin de la bibliothèque.
    NommerDessin {
        texte: String,
        id: u64,
        nouveau: bool,
    },
    ModifierMemorisee {
        quoi: Memorisee,
        texte: String,
        unite: UniteMasseVolumique,
        refusee: bool,
        nouveau: bool,
    },
    AideTouche {
        touche: Touche,
        secondaire: bool,
    },
    /// Liste des métiers : le clavier s'adapte à celui qu'on choisit (section 23).
    ChoisirMetier,
    /// Mode d'emploi de l'outil branché sur le clavier : son identifiant.
    AideOutil(&'static str),
}

pub struct Application {
    moteur: Moteur,
    /// Là où sont conservées les données : un dossier ou le navigateur.
    depot: Depot,
    rubans: Vec<RubanEnregistre>,
    vue: Vue,
    tiroir_ouvert: bool,
    dialogue: Dialogue,
    clavier: EtatClavier,
    recherche: String,
    fiche_ouverte: Option<usize>,
    ruban_ouvert: Option<u64>,
    /// Écran des métiers : où l'on se trouve, et ce qui a été tapé dans chaque calculateur.
    position_metiers: vues::metiers::Position,
    saisies_metiers: vues::metiers::Saisies,
    saisies_a_sauvegarder: bool,
    /// Dernier outil branché de chaque métier, pour y revenir (section 23).
    derniers_outils: Vec<(Metier, &'static str)>,
    /// Les calculateurs épinglés et récents (section 22.16).
    favoris: stockage::Favoris,
    /// La pastille de l'outil branché doit être amenée en vue dans la barre.
    outil_a_montrer: bool,
    /// Confirmation discrète (« Copié », « Ruban enregistré ») et son échéance.
    fugace: Option<(Texte, f64)>,
    a_sauvegarder: bool,
    /// Moment de la dernière écriture de l'état, en secondes de l'horloge d'egui.
    derniere_sauvegarde: f64,
    echec_d_ecriture_signale: bool,
    plein_ecran_applique: bool,
    /// La palette en vigueur : (thème sombre, mode chantier).
    palette_appliquee: Option<(bool, bool)>,
    /// Langue pour laquelle le titre de la fenêtre (ou de la page) est écrit.
    langue_appliquee: Option<Langue>,
    /// Texture des dégradés; elle doit vivre aussi longtemps que l'application.
    _rampe: egui::TextureHandle,
    /// Le logo, affiché à gauche du titre.
    marque: egui::TextureHandle,
    /// La pièce dessinée (section 25), et ce que son écran retient.
    plan: noyau::dessin::Plan,
    etat_du_dessin: vues::dessin::Etat,
    dessin_a_sauvegarder: bool,
    /// Le bloc-notes (section 26) : les notes, celle qui est ouverte, son
    /// texte qui doit prendre le clavier, et ce qui reste à enregistrer.
    notes: Vec<NoteDuBloc>,
    note_ouverte: Option<u64>,
    viser_la_note: bool,
    notes_a_sauvegarder: bool,
    /// Les vignettes des photos de la note ouverte, par numéro de pièce
    /// (rien : introuvable); la photo vue en grand et sa texture une fois lue.
    vignettes: HashMap<u64, Option<egui::TextureHandle>>,
    photo_vue: Option<(u64, Option<egui::TextureHandle>)>,
    /// La photo vue en grand, pendant qu'on la cote.
    etat_de_la_photo: notes::EtatDeLaPhoto,
    /// Pièces jointes demandées à la plateforme, par clé, et pourquoi.
    demandes: HashMap<String, Vec<Demande>>,
    /// Ce que deviendront les fichiers que l'utilisateur est en train de choisir.
    fichiers_pour: Option<Destination>,
    /// La bibliothèque des dessins (section 25) : le plan à l'écran est la
    /// copie de travail de celui de numéro `dessin_courant`, recopiée dans la
    /// bibliothèque quand on change de dessin ou qu'on l'enregistre.
    dessins: Vec<DessinEnregistre>,
    dessin_courant: u64,
    dessins_a_sauvegarder: bool,
    /// La liste des dessins est à l'écran, au lieu du dessin.
    liste_des_dessins: bool,
    /// La sauvegarde complète en train de réunir ses pièces jointes, et celle
    /// qui attend d'être confirmée pour remplacer les données.
    sauvegarde_en_cours: Option<SauvegardeEnCours>,
    sauvegarde_a_restaurer: Option<stockage::SauvegardeLue>,
}

impl Application {
    pub fn nouvelle(cc: &eframe::CreationContext<'_>) -> Self {
        theme::installer_les_polices(&cc.egui_ctx);
        // Ce qui arrive plus tard (pièces jointes, fichiers choisis) fait redessiner.
        plateforme::retenir_le_contexte(&cc.egui_ctx);
        // L'échelle suit la taille de la fenêtre : pas de zoom au clavier.
        cc.egui_ctx.options_mut(|o| o.zoom_with_keyboard = false);
        let mut application = Self::charger(&cc.egui_ctx);
        // Un raccourci de l'application installée ouvre son écran (section
        // 22.16); au premier lancement, la bienvenue passe d'abord.
        let demande = plateforme::ecran_demande();
        let vue = match demande.as_deref() {
            Some("metiers") => Some(Vue::Metiers),
            Some("dessin") => Some(Vue::Dessin),
            Some("notes") => Some(Vue::Notes),
            Some("rubans") => Some(Vue::Rubans),
            _ => None,
        };
        if let Some(vue) = vue
            && application.vue == Vue::Accueil
        {
            application.aller(vue);
        }
        application
    }

    /// Lit toutes les données conservées : au démarrage, et après avoir
    /// importé une sauvegarde complète.
    fn charger(ctx: &egui::Context) -> Self {
        let depot = Depot::ouvrir();
        let mut chargement = stockage::charger_etat(&depot);
        // Au premier lancement, la langue est celle de l'appareil, si c'est le
        // français ou l'anglais; ensuite, celle que l'utilisateur a choisie
        // (section 24).
        if (chargement.premiere_fois || chargement.illisible)
            && let Some(langue) = plateforme::langue_de_l_appareil()
        {
            chargement.etat.preferences.langue = langue;
        }
        let langue = chargement.etat.preferences.langue;
        noyau::langue::choisir(langue);
        let rubans = stockage::charger_rubans(&depot);
        let notes = stockage::charger_notes(&depot);
        let (mut saisies_metiers, outil, langue_des_saisies) = stockage::charger_saisies(&depot);
        let mut favoris = stockage::charger_favoris(&depot);
        favoris.nettoyer(|id| noyau::metiers::calculateur(id).is_some());
        let stockage::DessinCharge {
            mut plan,
            langue: langue_du_dessin,
            courant,
        } = stockage::charger_dessin(&depot);
        // La bibliothèque des dessins; le plan de travail lui appartient. Un
        // plan d'avant la bibliothèque y entre comme un premier dessin.
        let mut dessins = stockage::charger_dessins(&depot);
        let dessins_a_sauvegarder = !courant.is_some_and(|id| dessins.iter().any(|d| d.id == id));
        let dessin_courant = match courant.filter(|id| dessins.iter().any(|d| d.id == *id)) {
            Some(id) => id,
            None => {
                let (date, _) = stockage::maintenant();
                let id = nouveau_numero(dessins.iter().map(|d| d.id));
                dessins.push(DessinEnregistre {
                    id,
                    nom: textes::dessin_par_defaut(dessins.len() + 1),
                    cree: date.clone(),
                    modifie: date,
                    langue: langue_du_dessin,
                    plan: plan.clone(),
                });
                id
            }
        };
        for piece in &mut plan.pieces {
            transposer_le_dessin(
                &mut piece.dessin,
                langue_du_dessin,
                langue,
                &chargement.etat.preferences,
            );
        }
        // Des saisies écrites dans l'autre langue sont récrites dans celle-ci.
        let saisies_a_sauvegarder = langue_des_saisies != langue;
        transposer_les_saisies(
            &mut saisies_metiers,
            langue_des_saisies,
            langue,
            &chargement.etat.preferences,
        );
        let mut moteur = Moteur::nouveau(chargement.etat);
        // L'outil de métier branché sur le clavier à la dernière séance (section 23).
        if let Some(id) = outil.as_deref() {
            moteur.choisir_outil(Some(id), saisies_metiers.get(id).map(Vec::as_slice));
        }
        let vue = if moteur.bienvenue_vue() {
            Vue::Accueil
        } else {
            Vue::Bienvenue
        };
        let dialogue = if chargement.illisible {
            Dialogue::Avis(textes::AVIS_REGLAGES_ILLISIBLES)
        } else {
            Dialogue::Aucun
        };
        Application {
            moteur,
            depot,
            rubans,
            vue,
            tiroir_ouvert: false,
            dialogue,
            clavier: EtatClavier::default(),
            recherche: String::new(),
            fiche_ouverte: None,
            ruban_ouvert: None,
            position_metiers: vues::metiers::Position::default(),
            saisies_metiers,
            saisies_a_sauvegarder,
            derniers_outils: Vec::new(),
            favoris,
            outil_a_montrer: true,
            fugace: None,
            a_sauvegarder: chargement.illisible,
            // Le premier changement est enregistré sans attendre.
            derniere_sauvegarde: f64::NEG_INFINITY,
            echec_d_ecriture_signale: false,
            plein_ecran_applique: false,
            palette_appliquee: None,
            langue_appliquee: None,
            _rampe: theme::creer_la_rampe(ctx),
            marque: theme::creer_la_marque(ctx),
            plan,
            etat_du_dessin: vues::dessin::Etat::default(),
            // Le plan récrit dans la langue en cours, ou rattaché à son dessin.
            dessin_a_sauvegarder: langue_du_dessin != langue || dessins_a_sauvegarder,
            notes,
            note_ouverte: None,
            viser_la_note: false,
            notes_a_sauvegarder: false,
            vignettes: HashMap::new(),
            photo_vue: None,
            etat_de_la_photo: notes::EtatDeLaPhoto::default(),
            demandes: HashMap::new(),
            fichiers_pour: None,
            dessins,
            dessin_courant,
            dessins_a_sauvegarder,
            liste_des_dessins: false,
            sauvegarde_en_cours: None,
            sauvegarde_a_restaurer: None,
        }
    }

    // ------------------------------------------------------------------
    // Moteur
    // ------------------------------------------------------------------

    /// Transmet une touche au moteur et exécute les effets qu'il retourne.
    fn appuyer(&mut self, touche: Touche) {
        for effet in self.moteur.appuyer(touche) {
            match effet {
                Effet::OuvrirPreferences => self.aller(Vue::Preferences),
                Effet::BasculerRuban => {
                    let vue = if self.vue == Vue::Rubans {
                        Vue::Accueil
                    } else {
                        Vue::Rubans
                    };
                    self.aller(vue);
                }
                Effet::ConfirmerEffacerTout => {
                    self.dialogue = Dialogue::Confirmer(Confirmation::EffacerTout)
                }
                Effet::EtatModifie => self.a_sauvegarder = true,
            }
        }
    }

    fn aller(&mut self, vue: Vue) {
        self.vue = vue;
        self.tiroir_ouvert = false;
        self.fiche_ouverte = None;
        self.ruban_ouvert = None;
        if vue == Vue::Accueil {
            self.brancher_l_outil(self.moteur.outil().map(|c| c.id));
        }
    }

    /// Branche un calculateur de métier sur le clavier (section 23), avec ce
    /// que son formulaire contient; `None` revient au clavier de charpente.
    fn brancher_l_outil(&mut self, id: Option<&'static str>) {
        let avant = self.moteur.outil().map(|c| c.id);
        let saisies = id.and_then(|id| self.saisies_metiers.get(id));
        self.moteur.choisir_outil(id, saisies.map(Vec::as_slice));
        let apres = self.moteur.outil();
        if avant != apres.map(|c| c.id) {
            self.saisies_a_sauvegarder = true;
            self.outil_a_montrer = true;
        }
        // On reviendra à cet outil en revenant à ce métier; il passe en tête
        // des récents.
        if let Some(c) = apres {
            self.derniers_outils
                .retain(|(metier, _)| *metier != c.metier);
            self.derniers_outils.push((c.metier, c.id));
            if self.favoris.utiliser(c.id) {
                self.saisies_a_sauvegarder = true;
            }
        }
    }

    /// Change la langue de l'interface (section 24). Les saisies des
    /// calculateurs sont récrites comme elles se tapent dans la nouvelle
    /// langue : « 2,032 m » devient « 2.032 m », et la valeur reste la même.
    fn changer_de_langue(&mut self, langue: Langue) {
        let ancienne = self.moteur.preferences().langue;
        if ancienne == langue {
            return;
        }
        transposer_les_saisies(
            &mut self.saisies_metiers,
            ancienne,
            langue,
            self.moteur.preferences(),
        );
        for piece in &mut self.plan.pieces {
            transposer_le_dessin(
                &mut piece.dessin,
                ancienne,
                langue,
                self.moteur.preferences(),
            );
        }
        self.dessin_a_sauvegarder = true;
        self.moteur.changer_de_langue(langue);
        self.saisies_a_sauvegarder = true;
        self.a_sauvegarder = true;
    }

    /// Règle la langue en cours d'après les préférences, et récrit le titre de
    /// la fenêtre (ou de la page) quand la langue a changé.
    fn appliquer_la_langue(&mut self, ctx: &egui::Context) {
        let langue = self.moteur.preferences().langue;
        noyau::langue::choisir(langue);
        if self.langue_appliquee != Some(langue) {
            self.langue_appliquee = Some(langue);
            let titre = textes::NOM_APPLICATION.traduit();
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(titre.to_owned()));
            plateforme::regler_la_page(langue, titre);
        }
    }

    /// Adapte le clavier à un métier : son dernier outil utilisé, ou le
    /// premier de sa liste; `None` revient au clavier de charpente.
    fn choisir_metier(&mut self, metier: Option<Metier>) {
        let id = metier.and_then(|metier| {
            self.derniers_outils
                .iter()
                .find(|(m, _)| *m == metier)
                .map(|(_, id)| *id)
                .or_else(|| metier.calculateurs().first().map(|c| c.id))
        });
        self.brancher_l_outil(id);
    }

    /// Reporte dans les formulaires ce que le clavier a rangé dans l'outil.
    fn relever_l_outil(&mut self) {
        let (Some(c), Some(saisies)) = (self.moteur.outil(), self.moteur.saisies_d_outil()) else {
            return;
        };
        if self.saisies_metiers.get(c.id).map(Vec::as_slice) != Some(saisies) {
            self.saisies_metiers
                .insert(c.id.to_string(), saisies.to_vec());
            self.saisies_a_sauvegarder = true;
        }
    }

    /// Barre des métiers, au-dessus du clavier (section 23) : le métier
    /// choisi, puis ses outils — ou, au clavier de charpente, les métiers.
    fn barre_des_metiers(&mut self, ui: &mut Ui, zone: Rect, p: &Palette) {
        /// Ce que l'utilisateur a touché dans la barre.
        enum Choix {
            ListeDesMetiers,
            Metier(Metier),
            Outil(&'static str),
        }
        let outil = self.moteur.outil();
        let montrer = std::mem::replace(&mut self.outil_a_montrer, false);
        let epingles: Vec<&'static noyau::metiers::Calculateur> = self
            .favoris
            .epingles
            .iter()
            .filter_map(|id| noyau::metiers::calculateur(id))
            .collect();
        let mut choix = None;
        ui.scope_builder(
            egui::UiBuilder::new()
                .max_rect(zone.shrink2(Vec2::new(8.0, 3.0)))
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
            |ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                let nom =
                    outil.map_or(textes::METIER_CHARPENTE.traduit(), |c| c.metier.nom_court());
                if vues::pastille(ui, p, nom, vues::Pastille::Menu)
                    .on_hover_text(textes::INFOBULLE_METIER.traduit())
                    .clicked()
                {
                    choix = Some(Choix::ListeDesMetiers);
                }
                // Chaque métier garde sa propre position de défilement.
                egui::ScrollArea::horizontal()
                    .id_salt(("barre-des-metiers", nom))
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                    .scroll_source(egui::scroll_area::ScrollSource {
                        scroll_bar: false,
                        drag: egui::scroll_area::DragScroll::Always,
                        mouse_wheel: true,
                    })
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.horizontal_centered(|ui| {
                            // Les calculateurs épinglés, en tête, marqués d'une
                            // étoile (section 22.16) : un toucher les branche.
                            for c in &epingles {
                                let actif = outil.is_some_and(|a| a.id == c.id);
                                if vues::pastille_epinglee(ui, p, c.nom.traduit(), actif)
                                    .on_hover_text(c.resume.traduit())
                                    .clicked()
                                    && !actif
                                {
                                    choix = Some(Choix::Outil(c.id));
                                }
                            }
                            if !epingles.is_empty() {
                                ui.separator();
                            }
                            match outil {
                                // Un métier est choisi : ses outils, celui du clavier en évidence.
                                Some(actif) => {
                                    for c in actif.metier.calculateurs() {
                                        let genre = if c.id == actif.id {
                                            vues::Pastille::Active
                                        } else {
                                            vues::Pastille::Ordinaire
                                        };
                                        let reponse = vues::pastille(ui, p, c.nom.traduit(), genre)
                                            .on_hover_text(c.resume.traduit());
                                        if c.id == actif.id && montrer {
                                            reponse.scroll_to_me(Some(egui::Align::Center));
                                        }
                                        if reponse.clicked() && c.id != actif.id {
                                            choix = Some(Choix::Outil(c.id));
                                        }
                                    }
                                }
                                // Clavier de charpente : les autres métiers, à un toucher.
                                None => {
                                    for metier in Metier::TOUS {
                                        if vues::pastille(
                                            ui,
                                            p,
                                            metier.nom_court(),
                                            vues::Pastille::Ordinaire,
                                        )
                                        .on_hover_text(metier.resume())
                                        .clicked()
                                        {
                                            choix = Some(Choix::Metier(metier));
                                        }
                                    }
                                }
                            }
                        });
                    });
            },
        );
        match choix {
            Some(Choix::ListeDesMetiers) => self.dialogue = Dialogue::ChoisirMetier,
            Some(Choix::Metier(metier)) => self.choisir_metier(Some(metier)),
            Some(Choix::Outil(id)) => self.brancher_l_outil(Some(id)),
            None => {}
        }
    }

    fn dialogue_ouvert(&self) -> bool {
        !matches!(self.dialogue, Dialogue::Aucun)
    }

    fn annoncer(&mut self, ctx: &egui::Context, texte: Texte) {
        let maintenant = ctx.input(|i| i.time);
        self.fugace = Some((texte, maintenant + DUREE_FUGACE));
        ctx.request_repaint_after(Duration::from_secs_f64(DUREE_FUGACE + 0.1));
    }

    fn copier_valeur(&mut self, ctx: &egui::Context) {
        ctx.copy_text(self.moteur.texte_affiche());
        self.annoncer(ctx, textes::COPIE);
    }

    fn coller(&mut self, texte: &str) {
        match analyser_collage(texte.trim()) {
            Ok(valeur) => self.moteur.injecter(valeur),
            Err(_) => self.dialogue = Dialogue::Avis(textes::AVIS_COLLAGE_REFUSE),
        }
    }

    fn basculer_plein_ecran(&mut self) {
        if plateforme::NAVIGATEUR {
            // Dans un navigateur, le plein écran se demande sur-le-champ : il
            // n'est pas mémorisé dans les préférences.
            plateforme::basculer_plein_ecran();
            return;
        }
        let voulu = !self.moteur.preferences().plein_ecran;
        self.moteur.preferences_mut().plein_ecran = voulu;
        self.a_sauvegarder = true;
    }

    // ------------------------------------------------------------------
    // Sauvegarde (section 17.3)
    // ------------------------------------------------------------------

    /// Signale, une seule fois, que les données ne peuvent pas être conservées.
    fn signaler_l_echec_d_ecriture(&mut self) {
        if self.echec_d_ecriture_signale {
            return;
        }
        self.echec_d_ecriture_signale = true;
        self.dialogue = Dialogue::Avis(if plateforme::NAVIGATEUR {
            textes::AVIS_ECRITURE_IMPOSSIBLE_WEB
        } else {
            textes::AVIS_ECRITURE_IMPOSSIBLE
        });
    }

    fn sauvegarder(&mut self, ctx: &egui::Context) {
        self.a_sauvegarder = false;
        self.derniere_sauvegarde = ctx.input(|i| i.time);
        if stockage::enregistrer_etat(&self.depot, &self.moteur.etat_persistant()).is_err() {
            self.signaler_l_echec_d_ecriture();
        }
        if self.saisies_a_sauvegarder {
            self.saisies_a_sauvegarder = false;
            let outil = self.moteur.outil().map(|c| c.id);
            let langue = self.moteur.preferences().langue;
            if stockage::enregistrer_saisies(
                &self.depot,
                &self.saisies_metiers,
                outil,
                langue,
                &self.favoris,
            )
            .is_err()
            {
                self.signaler_l_echec_d_ecriture();
            }
        }
        if self.dessin_a_sauvegarder {
            self.dessin_a_sauvegarder = false;
            let langue = self.moteur.preferences().langue;
            let courant = Some(self.dessin_courant);
            if stockage::enregistrer_dessin(&self.depot, &self.plan, langue, courant).is_err() {
                self.signaler_l_echec_d_ecriture();
            }
        }
        if self.dessins_a_sauvegarder {
            self.dessins_a_sauvegarder = false;
            self.synchroniser_le_dessin_courant();
            if stockage::enregistrer_dessins(&self.depot, &self.dessins).is_err() {
                self.signaler_l_echec_d_ecriture();
            }
        }
        if self.notes_a_sauvegarder {
            self.notes_a_sauvegarder = false;
            if stockage::enregistrer_notes(&self.depot, &self.notes).is_err() {
                self.signaler_l_echec_d_ecriture();
            }
        }
    }

    /// Sur le bureau, au plus une écriture par seconde (R-45); dans un
    /// navigateur, sans délai.
    fn sauvegarder_si_du(&mut self, ctx: &egui::Context) {
        if !self.a_sauvegarder
            && !self.saisies_a_sauvegarder
            && !self.dessin_a_sauvegarder
            && !self.dessins_a_sauvegarder
            && !self.notes_a_sauvegarder
        {
            return;
        }
        let ecoule = ctx.input(|i| i.time) - self.derniere_sauvegarde;
        if ecoule >= plateforme::INTERVALLE_SAUVEGARDE {
            self.sauvegarder(ctx);
        } else {
            let attente = (plateforme::INTERVALLE_SAUVEGARDE - ecoule).max(0.0);
            ctx.request_repaint_after(Duration::from_secs_f64(attente));
        }
    }

    fn ecrire_rubans(&mut self) {
        if stockage::enregistrer_rubans(&self.depot, &self.rubans).is_err() {
            self.signaler_l_echec_d_ecriture();
        }
    }

    // ------------------------------------------------------------------
    // Rubans (section 14)
    // ------------------------------------------------------------------

    fn enregistrer_ruban(&mut self, ctx: &egui::Context, nom: String) {
        if self.rubans.len() >= MAX_RUBANS {
            self.dialogue = Dialogue::Confirmer(Confirmation::TropDeRubans(nom));
            return;
        }
        let (date, _) = stockage::maintenant();
        let id = self.rubans.iter().map(|r| r.id).max().unwrap_or(0) + 1;
        // Du plus récent au plus ancien.
        self.rubans.insert(
            0,
            RubanEnregistre {
                id,
                nom,
                date,
                lignes: self.moteur.ruban().lignes.clone(),
            },
        );
        self.ecrire_rubans();
        self.annoncer(ctx, textes::AVIS_RUBAN_ENREGISTRE);
    }

    fn nom_par_defaut() -> String {
        let (_, date) = stockage::maintenant();
        textes::nom_de_ruban(&date)
    }

    /// Texte d'un ruban à copier ou à exporter : (nom, texte).
    fn texte_de_ruban(&self, id: Option<u64>) -> Option<(String, String)> {
        match id {
            None => {
                let nom = Self::nom_par_defaut();
                let texte = self.moteur.ruban().en_texte(&nom);
                Some((nom, texte))
            }
            Some(id) => self.rubans.iter().find(|r| r.id == id).map(|r| {
                let titre = textes::avec_tiret(&r.nom, &r.date_en_clair());
                (r.nom.clone(), noyau::ruban::exporter(&titre, &r.lignes))
            }),
        }
    }

    fn copier_ruban(&mut self, ctx: &egui::Context, id: Option<u64>) {
        if let Some((_, texte)) = self.texte_de_ruban(id) {
            ctx.copy_text(texte);
            self.annoncer(ctx, textes::AVIS_RUBAN_COPIE);
        }
    }

    fn exporter_ruban(&mut self, ctx: &egui::Context, id: Option<u64>) {
        let Some((nom, texte)) = self.texte_de_ruban(id) else {
            return;
        };
        // Bureau : boîte « Enregistrer sous »; navigateur : téléchargement.
        let contenu = stockage::contenu_exporte(&texte);
        let fichier = Fichier {
            titre: textes::TITRE_EXPORTER.traduit(),
            filtre: textes::FILTRE_TEXTE.traduit(),
            extension: "txt",
            type_mime: "text/plain;charset=utf-8",
            nom: &stockage::nom_de_fichier(&nom),
            contenu: contenu.as_bytes(),
        };
        match plateforme::exporter(&fichier) {
            Export::Reussi => self.annoncer(ctx, textes::AVIS_EXPORT_REUSSI),
            Export::Annule => {}
            Export::Echec => self.dialogue = Dialogue::Avis(textes::AVIS_EXPORT_ECHEC),
        }
    }

    /// Exporte le plan dessiné (section 25), en PDF ou en DXF. Bureau : boîte
    /// « Enregistrer sous »; navigateur : téléchargement.
    fn exporter_le_dessin(&mut self, ctx: &egui::Context, format: vues::dessin::FormatDExport) {
        let prefs = self.moteur.preferences();
        let (iso, date) = stockage::maintenant();
        let (contenu, extension, type_mime, filtre) = match format {
            vues::dessin::FormatDExport::Pdf => (
                crate::exportation::pdf_du_plan(&self.plan, prefs, &date),
                "pdf",
                "application/pdf",
                textes::FILTRE_PDF.traduit(),
            ),
            vues::dessin::FormatDExport::Dxf => {
                let Some(contenu) = crate::exportation::dxf_du_plan(&self.plan, prefs) else {
                    return;
                };
                (
                    contenu,
                    "dxf",
                    "image/vnd.dxf",
                    textes::FILTRE_DXF.traduit(),
                )
            }
        };
        let jour = iso.get(..10).unwrap_or_default();
        let nom = stockage::nom_de_fichier_avec(&textes::dessin_nom_de_fichier(jour), extension);
        let fichier = Fichier {
            titre: textes::TITRE_EXPORTER_DESSIN.traduit(),
            filtre,
            extension,
            type_mime,
            nom: &nom,
            contenu: contenu.as_bytes(),
        };
        match plateforme::exporter(&fichier) {
            Export::Reussi => self.annoncer(ctx, textes::AVIS_DESSIN_EXPORTE),
            Export::Annule => {}
            Export::Echec => self.dialogue = Dialogue::Avis(textes::AVIS_DESSIN_EXPORT_ECHEC),
        }
    }

    fn traiter_action_ruban(&mut self, ctx: &egui::Context, action: rubans::Action) {
        let vide = self.moteur.ruban().est_vide();
        match action {
            rubans::Action::Enregistrer if vide => {
                self.dialogue = Dialogue::Avis(textes::AVIS_RUBAN_VIDE)
            }
            rubans::Action::Enregistrer => {
                self.dialogue = Dialogue::NommerRuban {
                    texte: Self::nom_par_defaut(),
                    renommer: None,
                    nouveau: true,
                };
            }
            rubans::Action::Copier => self.copier_ruban(ctx, None),
            rubans::Action::Exporter => self.exporter_ruban(ctx, None),
            rubans::Action::Vider => self.dialogue = Dialogue::Confirmer(Confirmation::ViderRuban),
            rubans::Action::Ouvrir(id) => self.ruban_ouvert = Some(id),
            rubans::Action::Renommer(id) => {
                if let Some(r) = self.rubans.iter().find(|r| r.id == id) {
                    self.dialogue = Dialogue::NommerRuban {
                        texte: r.nom.clone(),
                        renommer: Some(id),
                        nouveau: true,
                    };
                }
            }
            rubans::Action::CopierEnregistre(id) => self.copier_ruban(ctx, Some(id)),
            rubans::Action::ExporterEnregistre(id) => self.exporter_ruban(ctx, Some(id)),
            rubans::Action::Supprimer(id) => {
                self.dialogue = Dialogue::Confirmer(Confirmation::SupprimerRuban(id))
            }
            rubans::Action::FermerLecture => self.ruban_ouvert = None,
        }
    }

    // ------------------------------------------------------------------
    // Bloc-notes (section 26)
    // ------------------------------------------------------------------

    fn traiter_action_note(&mut self, ctx: &egui::Context, action: notes::Action) {
        match action {
            notes::Action::Nouvelle => {
                if self.notes.len() >= MAX_NOTES {
                    self.dialogue = Dialogue::Avis(textes::AVIS_TROP_DE_NOTES);
                    return;
                }
                let (date, _) = stockage::maintenant();
                let id = self.notes.iter().map(|n| n.id).max().unwrap_or(0) + 1;
                self.notes.insert(
                    0,
                    NoteDuBloc {
                        id,
                        creee: date.clone(),
                        modifiee: date,
                        ..NoteDuBloc::default()
                    },
                );
                self.oublier_les_images();
                self.note_ouverte = Some(id);
                self.viser_la_note = true;
                self.notes_a_sauvegarder = true;
            }
            notes::Action::Ouvrir(id) => {
                self.oublier_les_images();
                self.note_ouverte = Some(id);
                self.demander_les_vignettes();
            }
            notes::Action::Fermer => self.fermer_la_note(),
            notes::Action::Joindre(photos) => {
                let pleine = self
                    .note_ouverte
                    .and_then(|id| self.notes.iter().find(|n| n.id == id))
                    .is_some_and(|note| note.pieces.len() >= pieces::MAX_PIECES_PAR_NOTE);
                if pleine {
                    self.dialogue = Dialogue::Avis(textes::AVIS_TROP_DE_PIECES);
                } else if let Some(id) = self.note_ouverte {
                    self.fichiers_pour = Some(Destination::Note(id));
                    plateforme::choisir_des_fichiers(if photos {
                        ChoixDeFichiers::Photos
                    } else {
                        ChoixDeFichiers::Tous
                    });
                }
            }
            notes::Action::VoirPhoto(id) => {
                if let Some(piece) = self.piece_de_la_note_ouverte(id).cloned() {
                    self.photo_vue = Some((id, None));
                    self.etat_de_la_photo = notes::EtatDeLaPhoto::default();
                    self.demander(pieces::cle_des_octets(&piece), Demande::Grande(id));
                }
            }
            notes::Action::FermerPhoto => {
                self.photo_vue = None;
                self.etat_de_la_photo = notes::EtatDeLaPhoto::default();
            }
            notes::Action::PdfDeLaPhoto(id) => {
                if let Some(piece) = self.piece_de_la_note_ouverte(id).cloned() {
                    self.demander(pieces::cle_des_octets(&piece), Demande::Pdf(id));
                }
            }
            notes::Action::OuvrirPiece(id) => {
                if let Some(piece) = self.piece_de_la_note_ouverte(id).cloned()
                    && !self.depot.ouvrir_piece(&pieces::cle_des_octets(&piece))
                {
                    self.dialogue = Dialogue::Avis(textes::AVIS_OUVERTURE_ECHEC);
                }
            }
            notes::Action::EnregistrerPiece(id) => {
                if let Some(piece) = self.piece_de_la_note_ouverte(id).cloned() {
                    self.demander(pieces::cle_des_octets(&piece), Demande::Enregistrer(id));
                }
            }
            notes::Action::RetirerPiece(id) => {
                self.dialogue = Dialogue::Confirmer(Confirmation::RetirerPiece(id));
            }
            notes::Action::Modifiee => {
                if let Some(note) = self
                    .note_ouverte
                    .and_then(|id| self.notes.iter_mut().find(|n| n.id == id))
                {
                    note.modifiee = stockage::maintenant().0;
                }
                self.notes_a_sauvegarder = true;
            }
            notes::Action::Copier(id) => {
                let texte = self
                    .notes
                    .iter()
                    .find(|n| n.id == id)
                    .map(|note| note.en_texte(&notes::titre_de(note)));
                if let Some(texte) = texte {
                    ctx.copy_text(texte);
                    self.annoncer(ctx, textes::AVIS_NOTE_COPIEE);
                }
            }
            notes::Action::Exporter(id) => self.exporter_note(ctx, id),
            notes::Action::Supprimer(id) => {
                self.dialogue = Dialogue::Confirmer(Confirmation::SupprimerNote(id));
            }
        }
    }

    /// Revient à la liste des notes : une note laissée vide disparaît, et la
    /// plus récemment modifiée passe en tête.
    fn fermer_la_note(&mut self) {
        self.note_ouverte = None;
        self.oublier_les_images();
        self.notes.retain(|note| !note.est_vide());
        self.notes.sort_by(|a, b| b.modifiee.cmp(&a.modifiee));
        self.notes_a_sauvegarder = true;
    }

    /// Exporte une note en fichier texte. Bureau : boîte « Enregistrer sous »;
    /// navigateur : téléchargement.
    fn exporter_note(&mut self, ctx: &egui::Context, id: u64) {
        let Some((titre, texte)) = self.notes.iter().find(|n| n.id == id).map(|note| {
            let titre = notes::titre_de(note);
            let texte = note.en_texte(&titre);
            (titre, texte)
        }) else {
            return;
        };
        let nom = stockage::nom_de_fichier(&titre);
        let contenu = stockage::contenu_exporte(&texte);
        let fichier = Fichier {
            titre: textes::TITRE_EXPORTER.traduit(),
            filtre: textes::FILTRE_TEXTE.traduit(),
            extension: "txt",
            type_mime: "text/plain;charset=utf-8",
            nom: &nom,
            contenu: contenu.as_bytes(),
        };
        match plateforme::exporter(&fichier) {
            Export::Reussi => self.annoncer(ctx, textes::AVIS_EXPORT_REUSSI),
            Export::Annule => {}
            Export::Echec => self.dialogue = Dialogue::Avis(textes::AVIS_EXPORT_ECHEC),
        }
    }

    // ------------------------------------------------------------------
    // Pièces jointes du bloc-notes (section 26)
    // ------------------------------------------------------------------

    /// Demande à la plateforme une pièce jointe : elle n'est lue qu'une fois,
    /// même si on la demande pour deux raisons.
    fn demander(&mut self, cle: String, demande: Demande) {
        let attente = self.demandes.entry(cle.clone()).or_default();
        if attente.is_empty() {
            self.depot.lire_piece(&cle);
        }
        if !attente.contains(&demande) {
            attente.push(demande);
        }
    }

    /// Demande les vignettes des photos de la note ouverte qui ne sont pas encore là.
    fn demander_les_vignettes(&mut self) {
        let Some(note) = self
            .note_ouverte
            .and_then(|id| self.notes.iter().find(|n| n.id == id))
        else {
            return;
        };
        let manquantes: Vec<u64> = note
            .pieces
            .iter()
            .filter(|piece| piece.photo && !self.vignettes.contains_key(&piece.id))
            .map(|piece| piece.id)
            .collect();
        for id in manquantes {
            self.demander(pieces::cle_de_la_vignette(id), Demande::Vignette(id));
        }
    }

    /// Oublie les images de la note qu'on quitte (leur mémoire est rendue).
    fn oublier_les_images(&mut self) {
        self.vignettes.clear();
        self.photo_vue = None;
        self.etat_de_la_photo = notes::EtatDeLaPhoto::default();
        self.demandes.clear();
    }

    /// La pièce jointe de ce numéro, dans la note ouverte.
    fn piece_de_la_note_ouverte(&self, id: u64) -> Option<&PieceJointe> {
        self.note_ouverte
            .and_then(|note| self.notes.iter().find(|n| n.id == note))
            .and_then(|note| note.pieces.iter().find(|piece| piece.id == id))
    }

    /// Un numéro de pièce jointe neuf.
    fn nouveau_numero_de_piece(&self) -> u64 {
        nouveau_numero(
            self.notes
                .iter()
                .flat_map(|note| &note.pieces)
                .map(|piece| piece.id),
        )
    }

    // ------------------------------------------------------------------
    // Mes dessins (section 25)
    // ------------------------------------------------------------------

    /// Le nom du dessin ouvert, raccourci pour tenir dans l'en-tête.
    fn nom_du_dessin_courant(&self) -> String {
        let nom = self
            .dessins
            .iter()
            .find(|d| d.id == self.dessin_courant)
            .map_or("", |d| d.nom.trim());
        let nom = if nom.is_empty() {
            textes::DESSIN_TITRE.traduit()
        } else {
            nom
        };
        if nom.chars().count() > 22 {
            nom.chars().take(21).chain(['…']).collect()
        } else {
            nom.to_string()
        }
    }

    /// Un nom pour un dessin neuf : « Dessin 3 », le premier numéro libre.
    fn nom_libre(&self) -> String {
        (1..)
            .map(textes::dessin_par_defaut)
            .find(|nom| !self.dessins.iter().any(|d| &d.nom == nom))
            .unwrap_or_default()
    }

    /// Recopie le plan de travail à sa place dans la bibliothèque; sa date de
    /// modification change s'il a changé.
    fn synchroniser_le_dessin_courant(&mut self) {
        let langue = self.moteur.preferences().langue;
        if let Some(dessin) = self
            .dessins
            .iter_mut()
            .find(|d| d.id == self.dessin_courant)
            && (dessin.plan != self.plan || dessin.langue != langue)
        {
            dessin.plan = self.plan.clone();
            dessin.langue = langue;
            dessin.modifie = stockage::maintenant().0;
        }
    }

    /// Vrai si le dessin ouvert est vide et porte encore un nom par défaut
    /// (« Dessin 2 ») : rien n'y a été fait.
    fn dessin_courant_inutilise(&self) -> bool {
        self.dessins
            .iter()
            .find(|d| d.id == self.dessin_courant)
            .is_some_and(|d| {
                stockage::plan_est_vide(&self.plan)
                    && (1..=self.dessins.len() + 1).any(|n| textes::dessin_par_defaut(n) == d.nom)
            })
    }

    /// Met ce dessin à l'écran : son plan devient le plan de travail, récrit
    /// dans la langue en cours. Le dessin quitté est d'abord recopié; resté
    /// vide sous son nom par défaut, il disparaît, comme une note vide.
    fn ouvrir_le_dessin(&mut self, id: u64) {
        if id != self.dessin_courant && self.dessin_courant_inutilise() {
            let quitte = self.dessin_courant;
            self.dessins.retain(|d| d.id != quitte);
        } else {
            self.synchroniser_le_dessin_courant();
        }
        let Some(dessin) = self.dessins.iter().find(|d| d.id == id) else {
            return;
        };
        let mut plan = dessin.plan.clone();
        let prefs = self.moteur.preferences();
        for piece in &mut plan.pieces {
            transposer_le_dessin(&mut piece.dessin, dessin.langue, prefs.langue, prefs);
        }
        plan.normaliser();
        self.plan = plan;
        self.dessin_courant = id;
        self.etat_du_dessin = vues::dessin::Etat::default();
        self.liste_des_dessins = false;
        self.dessin_a_sauvegarder = true;
        self.dessins_a_sauvegarder = true;
    }

    /// Ajoute un dessin vide à la bibliothèque; son numéro.
    fn ajouter_un_dessin_vide(&mut self) -> u64 {
        let (date, _) = stockage::maintenant();
        let id = nouveau_numero(self.dessins.iter().map(|d| d.id));
        let nom = self.nom_libre();
        self.dessins.push(DessinEnregistre {
            id,
            nom,
            cree: date.clone(),
            modifie: date,
            langue: self.moteur.preferences().langue,
            plan: noyau::dessin::Plan::default(),
        });
        id
    }

    fn traiter_action_dessins(&mut self, ctx: &egui::Context, action: vues::dessins::Action) {
        use vues::dessins::Action as A;
        match action {
            A::Retour => self.liste_des_dessins = false,
            A::Nouveau | A::OuvrirUnFichier if self.dessins.len() >= MAX_DESSINS => {
                self.dialogue = Dialogue::Avis(textes::AVIS_TROP_DE_DESSINS);
            }
            // Le dessin ouvert est déjà vide et sans nom : c'est lui, le nouveau.
            A::Nouveau if self.dessin_courant_inutilise() => self.liste_des_dessins = false,
            A::Nouveau => {
                let id = self.ajouter_un_dessin_vide();
                self.ouvrir_le_dessin(id);
            }
            A::OuvrirUnFichier => {
                self.fichiers_pour = Some(Destination::Dessins);
                plateforme::choisir_des_fichiers(ChoixDeFichiers::Dessins);
            }
            A::Ouvrir(id) => self.ouvrir_le_dessin(id),
            A::Renommer(id) => {
                if let Some(dessin) = self.dessins.iter().find(|d| d.id == id) {
                    self.dialogue = Dialogue::NommerDessin {
                        texte: dessin.nom.clone(),
                        id,
                        nouveau: true,
                    };
                }
            }
            A::Enregistrer(id) => self.enregistrer_le_dessin_en_fichier(ctx, id),
            A::Supprimer(id) => {
                self.dialogue = Dialogue::Confirmer(Confirmation::SupprimerDessin(id));
            }
            A::Dupliquer(id) => {
                // La copie entre dans la liste, sous le nom de l'original.
                self.synchroniser_le_dessin_courant();
                if self.dessins.len() >= MAX_DESSINS {
                    self.dialogue = Dialogue::Avis(textes::AVIS_TROP_DE_DESSINS);
                } else if let Some(original) = self.dessins.iter().find(|d| d.id == id).cloned() {
                    let (date, _) = stockage::maintenant();
                    self.dessins.push(DessinEnregistre {
                        id: nouveau_numero(self.dessins.iter().map(|d| d.id)),
                        nom: textes::nom_de_copie(&original.nom),
                        cree: date.clone(),
                        modifie: date,
                        ..original
                    });
                    self.dessins_a_sauvegarder = true;
                    self.annoncer(ctx, textes::AVIS_DESSIN_DUPLIQUE);
                }
            }
        }
    }

    /// Enregistre un dessin en fichier, pour le rouvrir sur un autre
    /// appareil. Bureau : boîte « Enregistrer sous »; navigateur : téléchargement.
    fn enregistrer_le_dessin_en_fichier(&mut self, ctx: &egui::Context, id: u64) {
        self.synchroniser_le_dessin_courant();
        let Some((nom, contenu)) = self.dessins.iter().find(|d| d.id == id).map(|dessin| {
            (
                stockage::nom_de_fichier_avec(&dessin.nom, "json"),
                stockage::fichier_d_un_dessin(dessin),
            )
        }) else {
            return;
        };
        let fichier = Fichier {
            titre: textes::TITRE_ENREGISTRER_DESSIN.traduit(),
            filtre: textes::FILTRE_DESSIN.traduit(),
            extension: "json",
            type_mime: "application/json",
            nom: &nom,
            contenu: contenu.as_bytes(),
        };
        match plateforme::exporter(&fichier) {
            Export::Reussi => self.annoncer(ctx, textes::AVIS_DESSIN_ENREGISTRE),
            Export::Annule => {}
            Export::Echec => self.dialogue = Dialogue::Avis(textes::AVIS_DESSIN_ENREGISTRE_ECHEC),
        }
    }

    /// Rouvre les dessins choisis : chacun entre dans la bibliothèque, et le
    /// dernier s'ouvre. Un fichier qui n'est pas un dessin est signalé.
    fn importer_des_dessins(&mut self, ctx: &egui::Context, fichiers: Vec<FichierChoisi>) {
        let mut dernier = None;
        let mut illisible = false;
        for fichier in fichiers {
            if self.dessins.len() >= MAX_DESSINS {
                self.dialogue = Dialogue::Avis(textes::AVIS_TROP_DE_DESSINS);
                break;
            }
            let Some((nom, langue, plan)) = fichier
                .octets
                .as_deref()
                .and_then(stockage::lire_un_fichier_de_dessin)
            else {
                illisible = true;
                continue;
            };
            let nom = if nom.trim().is_empty() {
                fichier.nom.trim_end_matches(".json").to_string()
            } else {
                nom
            };
            let (date, _) = stockage::maintenant();
            let id = nouveau_numero(self.dessins.iter().map(|d| d.id));
            self.dessins.push(DessinEnregistre {
                id,
                nom,
                cree: date.clone(),
                modifie: date,
                langue,
                plan,
            });
            dernier = Some(id);
        }
        if let Some(id) = dernier {
            self.ouvrir_le_dessin(id);
            self.annoncer(ctx, textes::AVIS_DESSIN_IMPORTE);
        }
        if illisible {
            self.dialogue = Dialogue::Avis(textes::AVIS_DESSIN_ILLISIBLE);
        }
    }

    /// Traite ce qui est arrivé de la plateforme depuis la dernière image.
    fn relever_les_arrivees(&mut self, ctx: &egui::Context) {
        for arrivee in plateforme::arrivees() {
            match arrivee {
                Arrivee::EchecEcriture => self.dialogue = Dialogue::Avis(textes::AVIS_ECHEC_PIECE),
                Arrivee::Choisis(fichiers) => match self.fichiers_pour.take() {
                    Some(Destination::Note(id)) => self.joindre(ctx, id, fichiers),
                    Some(Destination::Dessins) => self.importer_des_dessins(ctx, fichiers),
                    Some(Destination::Sauvegarde) => self.lire_la_sauvegarde(fichiers),
                    None => {}
                },
                Arrivee::Lue { cle, octets } => {
                    let Some(attente) = self.demandes.remove(&cle) else {
                        continue;
                    };
                    for demande in attente {
                        self.recevoir_une_piece(ctx, &cle, demande, octets.clone());
                    }
                }
            }
        }
    }

    fn recevoir_une_piece(
        &mut self,
        ctx: &egui::Context,
        cle: &str,
        demande: Demande,
        octets: Option<Vec<u8>>,
    ) {
        match (demande, octets) {
            (Demande::Vignette(id), octets) => {
                let texture = octets.and_then(|o| texture_de(ctx, &format!("vignette-{id}"), &o));
                self.vignettes.insert(id, texture);
            }
            (Demande::Grande(id), Some(octets)) => {
                if self.photo_vue.as_ref().is_some_and(|(vue, _)| *vue == id) {
                    match texture_de(ctx, &format!("photo-{id}"), &octets) {
                        Some(texture) => self.photo_vue = Some((id, Some(texture))),
                        None => {
                            self.photo_vue = None;
                            self.dialogue = Dialogue::Avis(textes::AVIS_PIECE_ABSENTE);
                        }
                    }
                }
            }
            (Demande::Grande(id), None) => {
                if self.photo_vue.as_ref().is_some_and(|(vue, _)| *vue == id) {
                    self.photo_vue = None;
                }
                self.dialogue = Dialogue::Avis(textes::AVIS_PIECE_ABSENTE);
            }
            (Demande::Enregistrer(id), Some(octets)) => self.enregistrer_la_piece(ctx, id, &octets),
            (Demande::Pdf(id), Some(octets)) => self.exporter_la_photo_cotee(ctx, id, &octets),
            (Demande::Sauvegarde, octets) => self.recevoir_pour_la_sauvegarde(ctx, cle, octets),
            (Demande::Enregistrer(_) | Demande::Pdf(_), None) => {
                self.dialogue = Dialogue::Avis(textes::AVIS_PIECE_ABSENTE);
            }
        }
    }

    /// Joint les fichiers choisis à la note qui les attendait : une photo est
    /// préparée (réduite, avec sa vignette), un autre fichier est gardé tel quel.
    fn joindre(&mut self, ctx: &egui::Context, id_note: u64, fichiers: Vec<FichierChoisi>) {
        if !self.notes.iter().any(|n| n.id == id_note) {
            return;
        }
        let mut avis = None;
        let mut ajoutees = Vec::new();
        let deja = self
            .notes
            .iter()
            .find(|n| n.id == id_note)
            .map_or(0, |n| n.pieces.len());
        for fichier in fichiers {
            if deja + ajoutees.len() >= pieces::MAX_PIECES_PAR_NOTE {
                avis = Some(textes::AVIS_TROP_DE_PIECES);
                break;
            }
            let Some(octets) = fichier.octets else {
                avis = Some(textes::AVIS_FICHIER_TROP_GROS);
                continue;
            };
            let id = self.nouveau_numero_de_piece().max(
                ajoutees
                    .iter()
                    .map(|piece: &PieceJointe| piece.id + 1)
                    .max()
                    .unwrap_or(0),
            );
            let ajoutee = stockage::maintenant().0;
            let nom = if fichier.nom.trim().is_empty() {
                textes::FICHIER_PAR_DEFAUT.traduit().to_string()
            } else {
                fichier.nom
            };
            if let Some(photo) = pieces::preparer_photo(&octets) {
                // Le nom porte l'extension du format gardé.
                let nom = if matches!(pieces::extension_de(&nom).as_str(), "jpg" | "jpeg" | "png") {
                    nom
                } else {
                    format!("{nom}.{}", photo.extension)
                };
                let piece = PieceJointe {
                    id,
                    nom,
                    type_mime: photo.type_mime.to_string(),
                    taille: photo.octets.len() as u64,
                    photo: true,
                    largeur: photo.largeur,
                    hauteur: photo.hauteur,
                    ajoutee,
                    cotes: Vec::new(),
                };
                let texture = texture_de(ctx, &format!("vignette-{id}"), &photo.vignette);
                self.vignettes.insert(id, texture);
                self.depot
                    .ecrire_piece(&pieces::cle_de_la_vignette(id), photo.vignette);
                self.depot
                    .ecrire_piece(&pieces::cle_des_octets(&piece), photo.octets);
                ajoutees.push(piece);
                continue;
            }
            if octets.len() > pieces::TAILLE_MAX_FICHIER {
                avis = Some(textes::AVIS_FICHIER_TROP_GROS);
                continue;
            }
            if pieces::est_une_photo(&octets) || fichier.type_mime.starts_with("image/") {
                avis = Some(textes::AVIS_IMAGE_ILLISIBLE);
            }
            let type_mime = if fichier.type_mime.is_empty() {
                pieces::type_mime_de(&pieces::extension_de(&nom)).to_string()
            } else {
                fichier.type_mime
            };
            let piece = PieceJointe {
                id,
                nom,
                type_mime,
                taille: octets.len() as u64,
                ajoutee,
                ..PieceJointe::default()
            };
            self.depot
                .ecrire_piece(&pieces::cle_des_octets(&piece), octets);
            ajoutees.push(piece);
        }
        if !ajoutees.is_empty()
            && let Some(note) = self.notes.iter_mut().find(|n| n.id == id_note)
        {
            note.pieces.extend(ajoutees);
            note.modifiee = stockage::maintenant().0;
            self.notes_a_sauvegarder = true;
        }
        if let Some(avis) = avis {
            self.dialogue = Dialogue::Avis(avis);
        }
    }

    /// Enregistre une pièce jointe ailleurs. Bureau : boîte « Enregistrer
    /// sous »; navigateur : téléchargement.
    fn enregistrer_la_piece(&mut self, ctx: &egui::Context, id: u64, octets: &[u8]) {
        let Some(piece) = self.piece_de_la_note_ouverte(id).cloned() else {
            return;
        };
        let extension = pieces::extension_de(&piece.nom);
        let fichier = Fichier {
            titre: textes::TITRE_ENREGISTRER_PIECE.traduit(),
            filtre: textes::FILTRE_PIECE.traduit(),
            extension: &extension,
            type_mime: &piece.type_mime,
            nom: &piece.nom,
            contenu: octets,
        };
        match plateforme::exporter(&fichier) {
            Export::Reussi => self.annoncer(ctx, textes::AVIS_PIECE_ENREGISTREE),
            Export::Annule => {}
            Export::Echec => self.dialogue = Dialogue::Avis(textes::AVIS_PIECE_EXPORT_ECHEC),
        }
    }

    /// Exporte une photo et ses cotes en PDF (R-100). Bureau : boîte
    /// « Enregistrer sous »; navigateur : téléchargement.
    fn exporter_la_photo_cotee(&mut self, ctx: &egui::Context, id: u64, octets: &[u8]) {
        let Some(note) = self
            .note_ouverte
            .and_then(|ouverte| self.notes.iter().find(|n| n.id == ouverte))
        else {
            return;
        };
        let Some(piece) = note.pieces.iter().find(|piece| piece.id == id).cloned() else {
            return;
        };
        let entete = format!("{} · {}", notes::titre_de(note), note.date_en_clair());
        let Some((jpeg, largeur, hauteur)) = pieces::jpeg_pour_pdf(octets) else {
            self.dialogue = Dialogue::Avis(textes::AVIS_PHOTO_EXPORT_ECHEC);
            return;
        };
        let prefs = self.moteur.preferences();
        let lectures = pieces::lire_les_cotes(&piece.cotes, piece.largeur, piece.hauteur);
        let cotes: Vec<crate::exportation::CoteExportee> = piece
            .cotes
            .iter()
            .zip(lectures)
            .map(|(cote, lecture)| crate::exportation::CoteExportee {
                a: cote.a,
                b: cote.b,
                texte: notes::texte_d_une_cote(cote, lecture, prefs),
            })
            .collect();
        let pdf = crate::exportation::pdf_d_une_photo(
            &piece.nom,
            &entete,
            &jpeg,
            (largeur, hauteur),
            &cotes,
        );
        let base = piece
            .nom
            .rsplit_once('.')
            .map_or(piece.nom.as_str(), |(base, _)| base);
        let nom = stockage::nom_de_fichier_avec(base, "pdf");
        let fichier = Fichier {
            titre: textes::TITRE_EXPORTER_PHOTO.traduit(),
            filtre: textes::FILTRE_PDF.traduit(),
            extension: "pdf",
            type_mime: "application/pdf",
            nom: &nom,
            contenu: &pdf,
        };
        match plateforme::exporter(&fichier) {
            Export::Reussi => self.annoncer(ctx, textes::AVIS_PHOTO_EXPORTEE),
            Export::Annule => {}
            Export::Echec => self.dialogue = Dialogue::Avis(textes::AVIS_PHOTO_EXPORT_ECHEC),
        }
    }

    // ------------------------------------------------------------------
    // Sauvegarde complète (section 17.4)
    // ------------------------------------------------------------------

    /// Réunit toutes les données en un fichier : d'abord tout ce qui reste à
    /// enregistrer, puis les données, puis les pièces jointes, lues une à une.
    fn exporter_les_donnees(&mut self, ctx: &egui::Context) {
        if self.sauvegarde_en_cours.is_some() {
            return;
        }
        self.synchroniser_le_dessin_courant();
        self.dessins_a_sauvegarder = true;
        self.sauvegarder(ctx);
        let fichiers = stockage::donnees_a_sauvegarder(&self.depot);
        let mut cles = Vec::new();
        for piece in self.notes.iter().flat_map(|note| &note.pieces) {
            cles.push(pieces::cle_des_octets(piece));
            if piece.photo {
                cles.push(pieces::cle_de_la_vignette(piece.id));
            }
        }
        self.sauvegarde_en_cours = Some(SauvegardeEnCours {
            fichiers,
            pieces: std::collections::BTreeMap::new(),
            attendues: cles.len(),
        });
        if cles.is_empty() {
            self.terminer_la_sauvegarde(ctx);
            return;
        }
        for cle in cles {
            self.demander(cle, Demande::Sauvegarde);
        }
    }

    /// Une pièce jointe lue pour la sauvegarde, sous sa clé; la dernière la
    /// termine. Une pièce introuvable est laissée de côté.
    fn recevoir_pour_la_sauvegarde(
        &mut self,
        ctx: &egui::Context,
        cle: &str,
        octets: Option<Vec<u8>>,
    ) {
        let Some(en_cours) = self.sauvegarde_en_cours.as_mut() else {
            return;
        };
        en_cours.attendues = en_cours.attendues.saturating_sub(1);
        if let Some(octets) = octets {
            en_cours.pieces.insert(cle.to_string(), octets);
        }
        if en_cours.attendues == 0 {
            self.terminer_la_sauvegarde(ctx);
        }
    }

    /// Écrit le fichier de la sauvegarde complète. Bureau : boîte
    /// « Enregistrer sous »; navigateur : téléchargement.
    fn terminer_la_sauvegarde(&mut self, ctx: &egui::Context) {
        let Some(en_cours) = self.sauvegarde_en_cours.take() else {
            return;
        };
        let (date, _) = stockage::maintenant();
        let contenu = stockage::fichier_de_sauvegarde(&date, en_cours.fichiers, &en_cours.pieces);
        let jour = date.get(..10).unwrap_or_default();
        let nom = stockage::nom_de_fichier_avec(&textes::nom_de_sauvegarde(jour), "json");
        let fichier = Fichier {
            titre: textes::TITRE_EXPORTER_DONNEES.traduit(),
            filtre: textes::FILTRE_SAUVEGARDE.traduit(),
            extension: "json",
            type_mime: "application/json",
            nom: &nom,
            contenu: contenu.as_bytes(),
        };
        match plateforme::exporter(&fichier) {
            Export::Reussi => self.annoncer(ctx, textes::AVIS_DONNEES_EXPORTEES),
            Export::Annule => {}
            Export::Echec => self.dialogue = Dialogue::Avis(textes::AVIS_DONNEES_EXPORT_ECHEC),
        }
    }

    /// Lit la sauvegarde choisie, puis demande confirmation avant de remplacer
    /// les données.
    fn lire_la_sauvegarde(&mut self, fichiers: Vec<FichierChoisi>) {
        let lue = fichiers
            .into_iter()
            .next()
            .and_then(|fichier| fichier.octets)
            .and_then(|octets| stockage::lire_une_sauvegarde(&octets));
        match lue {
            Some(sauvegarde) => {
                self.sauvegarde_a_restaurer = Some(sauvegarde);
                self.dialogue = Dialogue::Confirmer(Confirmation::Restaurer);
            }
            None => self.dialogue = Dialogue::Avis(textes::AVIS_SAUVEGARDE_ILLISIBLE),
        }
    }

    /// Remplace les données par celles de la sauvegarde confirmée, puis
    /// recharge tout, comme au démarrage.
    fn restaurer_la_sauvegarde(&mut self, ctx: &egui::Context) {
        let Some(sauvegarde) = self.sauvegarde_a_restaurer.take() else {
            return;
        };
        // Les pièces jointes d'aujourd'hui laissent la place à celles de la sauvegarde.
        let anciennes: Vec<PieceJointe> = self
            .notes
            .iter()
            .flat_map(|note| note.pieces.clone())
            .collect();
        for piece in &anciennes {
            self.effacer_la_piece(piece);
        }
        if stockage::restaurer(&self.depot, &sauvegarde).is_err() {
            self.dialogue = Dialogue::Avis(textes::AVIS_DONNEES_IMPORT_ECHEC);
            return;
        }
        *self = Self::charger(ctx);
        self.annoncer(ctx, textes::AVIS_DONNEES_IMPORTEES);
    }

    /// Efface les octets d'une pièce jointe (et sa vignette).
    fn effacer_la_piece(&mut self, piece: &PieceJointe) {
        self.depot.supprimer_piece(&pieces::cle_des_octets(piece));
        if piece.photo {
            self.depot
                .supprimer_piece(&pieces::cle_de_la_vignette(piece.id));
        }
        self.vignettes.remove(&piece.id);
        if self
            .photo_vue
            .as_ref()
            .is_some_and(|(vue, _)| *vue == piece.id)
        {
            self.photo_vue = None;
        }
    }

    // ------------------------------------------------------------------
    // Fenêtre : échelle, thème, plein écran
    // ------------------------------------------------------------------

    /// Met tout le contenu à l'échelle de la fenêtre (section 16.1). Retourne
    /// vrai si la fenêtre est large : le ruban s'affiche alors à droite.
    fn regler_echelle(&self, ctx: &egui::Context) -> bool {
        let zoom = ctx.zoom_factor();
        let natif = ctx.content_rect().size() * zoom;
        if natif.x < 1.0 || natif.y < 1.0 {
            return false;
        }
        let large = natif.x >= LARGEUR_LARGE;
        let voulu = if large {
            (natif.y / HAUTEUR_NOMINALE).min(LARGEUR_PLAFOND / LARGEUR_NOMINALE)
        } else {
            (natif.x / LARGEUR_NOMINALE).min(natif.y / HAUTEUR_NOMINALE)
        }
        .clamp(0.4, 3.0);
        if (voulu - zoom).abs() > 0.003 {
            ctx.set_zoom_factor(voulu);
            ctx.request_repaint();
        }
        large
    }

    fn appliquer_theme(&mut self, ctx: &egui::Context) -> Palette {
        let prefs = self.moteur.preferences();
        let sombre = match prefs.theme {
            Theme::Clair => false,
            Theme::Sombre => true,
            Theme::Automatique => ctx.system_theme() == Some(egui::Theme::Dark),
        };
        let voulue = (sombre, prefs.mode_chantier);
        if self.palette_appliquee != Some(voulue) {
            theme::installer_le_style(ctx, theme::palette(voulue.0, voulue.1));
            self.palette_appliquee = Some(voulue);
        }
        *theme::palette(voulue.0, voulue.1)
    }

    fn appliquer_plein_ecran(&mut self, ctx: &egui::Context) {
        if plateforme::NAVIGATEUR {
            // Le navigateur gère son plein écran (voir `basculer_plein_ecran`).
            return;
        }
        let voulu = self.moteur.preferences().plein_ecran;
        if voulu != self.plein_ecran_applique {
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(voulu));
            self.plein_ecran_applique = voulu;
        }
    }

    // ------------------------------------------------------------------
    // Clavier d'ordinateur (section 16.5)
    // ------------------------------------------------------------------

    fn traiter_clavier(&mut self, ctx: &egui::Context) {
        let evenements = ctx.input(|i| i.events.clone());
        if evenements.is_empty() {
            return;
        }
        let champ_actif = ctx.text_edit_focused();
        for evenement in evenements {
            // Un dialogue ouvert reçoit seul le clavier (sauf F11).
            let libre = !self.dialogue_ouvert() && !self.tiroir_ouvert;
            let calculatrice = libre && self.vue == Vue::Accueil && !champ_actif;
            match evenement {
                egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } => {
                    if key == egui::Key::Escape && !self.dialogue_ouvert() {
                        // Échap ferme le tiroir ou l'écran secondaire, sinon efface.
                        if self.tiroir_ouvert {
                            self.tiroir_ouvert = false;
                            continue;
                        }
                        // Dans l'écran des métiers, Échap remonte d'un niveau
                        // (et commence par quitter le champ en cours de saisie).
                        if self.vue == Vue::Metiers
                            && (champ_actif || self.position_metiers.remonter())
                        {
                            continue;
                        }
                        if self.vue != Vue::Accueil {
                            self.aller(Vue::Accueil);
                            continue;
                        }
                    }
                    let ctrl = modifiers.ctrl || modifiers.command;
                    match raccourcis::action_de_la_touche(key, ctrl) {
                        // Dans un navigateur, F11 appartient au navigateur.
                        Some(Action::PleinEcran) if plateforme::NAVIGATEUR => {}
                        Some(Action::PleinEcran) => self.basculer_plein_ecran(),
                        Some(Action::Aide) if libre => self.aller(Vue::Aide),
                        Some(Action::Preferences) if libre => self.aller(Vue::Preferences),
                        Some(Action::Ruban) if libre => self.aller(Vue::Rubans),
                        Some(Action::Touche(touche))
                            if calculatrice
                                && clavier::est_au_clavier(
                                    touche,
                                    self.moteur.outil().is_some(),
                                ) =>
                        {
                            self.appuyer(touche)
                        }
                        _ => {}
                    }
                }
                egui::Event::Text(texte) if calculatrice => {
                    // Ctrl seul n'écrit rien; Ctrl+Alt (AltGr) écrit des caractères.
                    let m = ctx.input(|i| i.modifiers);
                    if (m.ctrl || m.command) && !m.alt {
                        continue;
                    }
                    for c in texte.chars() {
                        // Avec un outil de métier, les touches masquées ne répondent pas.
                        if let Some(touche) = raccourcis::touche_du_caractere(c)
                            && clavier::est_au_clavier(touche, self.moteur.outil().is_some())
                        {
                            self.appuyer(touche);
                        }
                    }
                }
                egui::Event::Copy if calculatrice => self.copier_valeur(ctx),
                egui::Event::Paste(texte) if calculatrice => self.coller(&texte),
                _ => {}
            }
        }
    }

    // ------------------------------------------------------------------
    // Barres
    // ------------------------------------------------------------------

    fn barre_superieure(&mut self, ui: &mut Ui, p: &Palette) {
        let rect = ui.max_rect();
        ui.allocate_rect(rect, Sense::hover());
        // Dégradé bleu marine et filet d'accent.
        theme::rect_degrade(ui.painter(), rect, 0, p.barre_fond, p.barre_fond_bas);
        let filet = Rect::from_min_max(
            Pos2::new(rect.left(), rect.bottom() - 2.0),
            rect.right_bottom(),
        );
        ui.painter().rect_filled(filet, 0, p.primaire);
        let cote = rect.height();
        let case = |i: f32| {
            Rect::from_min_size(
                Pos2::new(rect.left() + cote * i, rect.top()),
                Vec2::splat(cote),
            )
        };
        if bouton_icone(
            ui,
            case(0.0),
            textes::NOM_MENU.traduit(),
            textes::INFOBULLE_MENU.traduit(),
            p,
            dessiner_menu,
        ) {
            self.tiroir_ouvert = !self.tiroir_ouvert;
        }
        // Certains navigateurs (sur téléphone surtout) n'offrent pas le plein écran.
        let plein_ecran = plateforme::plein_ecran_possible();
        if plein_ecran
            && bouton_icone(
                ui,
                case(1.0),
                textes::NOM_PLEIN_ECRAN.traduit(),
                textes::INFOBULLE_PLEIN_ECRAN.traduit(),
                p,
                dessiner_plein_ecran,
            )
        {
            self.basculer_plein_ecran();
        }
        let boutons_a_gauche = if plein_ecran { 2.0 } else { 1.0 };
        let droite = Rect::from_min_size(
            Pos2::new(rect.right() - cote, rect.top()),
            Vec2::splat(cote),
        );
        if bouton_icone(
            ui,
            droite,
            textes::PREFERENCES.traduit(),
            textes::INFOBULLE_PREFERENCES.traduit(),
            p,
            dessiner_roue,
        ) {
            let vue = if self.vue == Vue::Preferences {
                Vue::Accueil
            } else {
                Vue::Preferences
            };
            self.aller(vue);
        }
        // Bouton de langue, à gauche des préférences (section 24).
        let zone_langue = Rect::from_min_max(
            Pos2::new(droite.left() - LARGEUR_BOUTON_DE_LANGUE, rect.top()),
            droite.left_bottom(),
        );
        let en_cours = self.moteur.preferences().langue;
        if let Some(langue) = bouton_de_langue(ui, zone_langue, p, en_cours) {
            self.changer_de_langue(langue);
            ui.ctx().request_repaint();
        }
        // Logo : celui de l'icône de l'application.
        let logo = Rect::from_center_size(
            Pos2::new(
                rect.left() + boutons_a_gauche * cote + 14.0,
                rect.center().y,
            ),
            Vec2::splat(24.0),
        );
        ui.painter().image(
            self.marque.id(),
            logo,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );
        let zone_titre = Rect::from_min_max(
            Pos2::new(logo.right() + 9.0, rect.top()),
            Pos2::new(zone_langue.left() - 4.0, rect.bottom()),
        );
        let police = theme::police_grasse(16.5);
        let g = ui.painter().layout_no_wrap(
            textes::NOM_APPLICATION.traduit().to_owned(),
            police.clone(),
            p.barre_texte,
        );
        let police = if g.size().x > zone_titre.width() && g.size().x > 0.0 {
            egui::FontId::new(
                (police.size * zone_titre.width() / g.size().x).max(9.0),
                police.family,
            )
        } else {
            police
        };
        ui.painter().text(
            zone_titre.left_center(),
            Align2::LEFT_CENTER,
            textes::NOM_APPLICATION.traduit(),
            police,
            p.barre_texte,
        );
    }

    fn barre_inferieure(&mut self, ui: &mut Ui, p: &Palette) {
        let rect = ui.max_rect();
        ui.allocate_rect(rect, Sense::hover());
        theme::rect_degrade(ui.painter(), rect, 0, p.pied_fond, p.pied_fond_bas);
        let filet = Rect::from_min_size(rect.min, Vec2::new(rect.width(), 1.0));
        ui.painter()
            .rect_filled(filet, 0, Color32::from_white_alpha(22));
        let onglets: [(Vue, &str, Dessin); 6] = [
            (
                Vue::Accueil,
                textes::ONGLET_ACCUEIL.traduit(),
                dessiner_onglet_accueil,
            ),
            (
                Vue::Metiers,
                textes::METIERS.traduit(),
                dessiner_onglet_metiers,
            ),
            (
                Vue::Dessin,
                textes::ONGLET_DESSIN.traduit(),
                dessiner_onglet_dessin,
            ),
            (
                Vue::Rubans,
                textes::ONGLET_RUBANS.traduit(),
                dessiner_onglet_rubans,
            ),
            (
                Vue::Notes,
                textes::ONGLET_NOTES.traduit(),
                dessiner_onglet_notes,
            ),
            (
                Vue::Aide,
                textes::ONGLET_AIDE.traduit(),
                dessiner_onglet_aide,
            ),
        ];
        let largeur = rect.width() / onglets.len() as f32;
        for (i, (vue, libelle, dessin)) in onglets.into_iter().enumerate() {
            let zone = Rect::from_min_size(
                Pos2::new(rect.left() + largeur * i as f32, rect.top()),
                Vec2::new(largeur, rect.height()),
            );
            let reponse = ui.interact(zone, egui::Id::unique(("onglet", i)), Sense::CLICK);
            reponse.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Tab, true, libelle));
            let actif = self.vue == vue;
            let couleur = if actif || reponse.hovered() {
                p.barre_texte
            } else {
                theme::melanger(p.barre_texte, p.pied_fond, 0.38)
            };
            let painter = ui.painter();
            if actif {
                // Onglet actif : pastille claire et repère orange.
                painter.rect_filled(
                    zone.shrink2(Vec2::new(10.0, 6.0)),
                    8,
                    Color32::from_white_alpha(20),
                );
                let trait_ = Rect::from_min_size(
                    Pos2::new(zone.left() + 22.0, zone.top()),
                    Vec2::new(zone.width() - 44.0, 3.0),
                );
                painter.rect_filled(trait_, 2, p.barre_accent);
            } else if reponse.hovered() {
                painter.rect_filled(
                    zone.shrink2(Vec2::new(10.0, 6.0)),
                    8,
                    Color32::from_white_alpha(10),
                );
            }
            let icone = Rect::from_center_size(
                Pos2::new(zone.center().x, zone.top() + 20.0),
                Vec2::splat(22.0),
            );
            dessin(painter, icone, couleur);
            painter.text(
                Pos2::new(zone.center().x, zone.bottom() - 13.0),
                Align2::CENTER_CENTER,
                libelle,
                theme::police_grasse(12.0),
                couleur,
            );
            if reponse.clicked() {
                self.aller(vue);
            }
        }
    }

    // ------------------------------------------------------------------
    // Écrans
    // ------------------------------------------------------------------

    fn accueil(&mut self, ui: &mut Ui, p: &Palette) {
        let rect = ui.max_rect();
        ui.allocate_rect(rect, Sense::hover());
        let prefs = self.moteur.preferences();
        let resolution = prefs.resolution_valide();
        // La touche décimale porte le séparateur de la langue en cours.
        let contexte_base = (
            prefs.mode_trigo,
            noyau::format::separateur_decimal(prefs) == '.',
        );
        // Mode chantier : l'afficheur cède un peu de place aux grosses touches.
        let hauteur_afficheur = if p.chantier {
            (rect.height() * 0.18).clamp(118.0, 220.0)
        } else {
            (rect.height() * 0.20).clamp(118.0, 240.0)
        };
        let hauteur_fractions = if prefs.fractions_rapides {
            clavier::rangees_de_fractions(resolution) as f32 * clavier::HAUTEUR_FRACTIONS + 4.0
        } else {
            0.0
        };
        let zone_afficheur =
            Rect::from_min_size(rect.min, Vec2::new(rect.width(), hauteur_afficheur));
        let zone_fractions = Rect::from_min_size(
            zone_afficheur.left_bottom(),
            Vec2::new(rect.width(), hauteur_fractions),
        );
        let zone_metiers = Rect::from_min_size(
            zone_fractions.left_bottom(),
            Vec2::new(rect.width(), HAUTEUR_BARRE_DES_METIERS),
        );
        let zone_clavier = Rect::from_min_max(zone_metiers.left_bottom(), rect.max);

        let ecran = self.moteur.ecran();
        let maintenant = ui.input(|i| i.time);
        let copie_visible = self
            .fugace
            .is_some_and(|(texte, fin)| texte == textes::COPIE && maintenant < fin);
        let mut agi = false;
        match afficheur::dessiner(ui, zone_afficheur, p, &ecran, copie_visible) {
            Some(ActionAfficheur::RetourArriere) => {
                self.appuyer(Touche::RetourArriere);
                agi = true;
            }
            Some(ActionAfficheur::Copier) => self.copier_valeur(ui.ctx()),
            None => {}
        }
        if hauteur_fractions > 0.0
            && let Some(touche) = clavier::dessiner_fractions(ui, zone_fractions, p, resolution)
        {
            self.appuyer(touche);
            agi = true;
        }
        let contexte = Contexte {
            trigo: contexte_base.0,
            point_decimal: contexte_base.1,
            conv_arme: ecran.prefixe == Prefixe::Conv,
        };
        self.barre_des_metiers(ui, zone_metiers, p);
        let outil = self.moteur.touches_d_outil();
        let actions = clavier::dessiner(
            ui,
            zone_clavier,
            p,
            contexte,
            outil.as_deref(),
            &mut self.clavier,
        );
        for action in actions {
            agi = true;
            match action {
                ActionClavier::Appui(touche) => self.appuyer(touche),
                ActionClavier::Secondaire(touche) => {
                    // Clic sur l'étiquette : Conv puis la touche.
                    if self.moteur.ecran().prefixe != Prefixe::Conv {
                        self.appuyer(Touche::Conv);
                    }
                    self.appuyer(touche);
                }
                // L'aide d'une touche d'outil : le mode d'emploi de l'outil.
                ActionClavier::Aide(Touche::Outil(_)) => {
                    if let Some(c) = self.moteur.outil() {
                        self.dialogue = Dialogue::AideOutil(c.id);
                    }
                }
                ActionClavier::Aide(touche) => {
                    self.dialogue = Dialogue::AideTouche {
                        touche,
                        secondaire: false,
                    };
                }
            }
        }
        // Ce que le clavier a rangé dans l'outil rejoint les formulaires.
        self.relever_l_outil();
        if agi {
            // L'afficheur a été dessiné avant l'action : on redessine.
            ui.ctx().request_repaint();
        }
    }

    fn centre(&mut self, ui: &mut Ui, p: &Palette) {
        theme::peindre_le_fond(ui, ui.max_rect(), p);
        match self.vue {
            Vue::Accueil => self.accueil(ui, p),
            Vue::Notes => {
                let images = notes::Images {
                    vignettes: &self.vignettes,
                    photo: self
                        .photo_vue
                        .as_ref()
                        .map(|(id, texture)| (*id, texture.as_ref())),
                    prefs: self.moteur.preferences(),
                };
                let action = notes::afficher(
                    ui,
                    p,
                    &mut self.notes,
                    self.note_ouverte,
                    &mut self.viser_la_note,
                    &images,
                    &mut self.etat_de_la_photo,
                );
                if let Some(action) = action {
                    self.traiter_action_note(ui.ctx(), action);
                }
            }
            Vue::Rubans => {
                let ouvert = self
                    .ruban_ouvert
                    .and_then(|id| self.rubans.iter().find(|r| r.id == id));
                let action = rubans::afficher(ui, p, self.moteur.ruban(), &self.rubans, ouvert);
                if let Some(action) = action {
                    self.traiter_action_ruban(ui.ctx(), action);
                }
            }
            Vue::Metiers => {
                let avant = self.position_metiers.calculateur;
                let action = vues::metiers::afficher(
                    ui,
                    p,
                    &mut self.position_metiers,
                    &mut self.saisies_metiers,
                    self.moteur.preferences(),
                    &self.favoris,
                );
                // Un formulaire qui s'ouvre passe en tête des récents.
                if let Some(id) = self.position_metiers.calculateur
                    && avant != Some(id)
                    && self.favoris.utiliser(id)
                {
                    self.saisies_a_sauvegarder = true;
                }
                match action {
                    Some(vues::metiers::Action::Epingler(id)) => {
                        if self.favoris.basculer(id) {
                            self.saisies_a_sauvegarder = true;
                        } else {
                            self.dialogue = Dialogue::Avis(textes::AVIS_TROP_D_EPINGLES);
                        }
                    }
                    Some(vues::metiers::Action::Modifie) => self.saisies_a_sauvegarder = true,
                    Some(vues::metiers::Action::Consigner {
                        titre,
                        entrees,
                        resultats,
                    }) => {
                        self.moteur.consigner(titre, &entrees, &resultats);
                        self.a_sauvegarder = true;
                        self.annoncer(ui.ctx(), textes::AVIS_AJOUTE_AU_RUBAN);
                    }
                    // La carte « Charpente » : le clavier de base.
                    Some(vues::metiers::Action::Clavier) => {
                        self.brancher_l_outil(None);
                        self.aller(Vue::Accueil);
                    }
                    Some(vues::metiers::Action::AuClavier(id)) => {
                        self.brancher_l_outil(Some(id));
                        self.aller(Vue::Accueil);
                    }
                    None => {}
                }
            }
            Vue::Dessin if self.liste_des_dessins => {
                let action = vues::dessins::afficher(ui, p, &self.dessins, self.dessin_courant);
                if let Some(action) = action {
                    self.traiter_action_dessins(ui.ctx(), action);
                }
            }
            Vue::Dessin => {
                let nom = self.nom_du_dessin_courant();
                let action = vues::dessin::afficher(
                    ui,
                    p,
                    &mut self.plan,
                    &mut self.etat_du_dessin,
                    self.moteur.preferences(),
                    &nom,
                );
                match action {
                    Some(vues::dessin::Action::Modifie) => self.dessin_a_sauvegarder = true,
                    Some(vues::dessin::Action::MesDessins) => {
                        // La liste montre le dessin ouvert tel qu'il est.
                        self.synchroniser_le_dessin_courant();
                        self.liste_des_dessins = true;
                    }
                    Some(vues::dessin::Action::Exporter(format)) => {
                        self.exporter_le_dessin(ui.ctx(), format);
                    }
                    Some(vues::dessin::Action::VersOutil { id, champs }) => {
                        if let Some(c) = noyau::metiers::calculateur(id) {
                            let saisies = self
                                .saisies_metiers
                                .entry(id.to_string())
                                .or_insert_with(|| c.saisies_par_defaut());
                            saisies.resize(c.champs.len(), String::new());
                            for (rang, texte) in champs {
                                if let Some(saisie) = saisies.get_mut(rang) {
                                    *saisie = texte;
                                }
                            }
                            self.saisies_a_sauvegarder = true;
                            // L'outil s'ouvre au clavier, avec ces mesures.
                            self.brancher_l_outil(None);
                            self.brancher_l_outil(Some(id));
                            self.aller(Vue::Accueil);
                        }
                    }
                    Some(vues::dessin::Action::Consigner {
                        titre,
                        entrees,
                        resultats,
                    }) => {
                        let entrees: Vec<(&str, String)> = entrees
                            .iter()
                            .map(|(l, v)| (l.as_str(), v.clone()))
                            .collect();
                        let resultats: Vec<(&str, String)> = resultats
                            .iter()
                            .map(|(l, v)| (l.as_str(), v.clone()))
                            .collect();
                        self.moteur.consigner(&titre, &entrees, &resultats);
                        self.a_sauvegarder = true;
                        self.annoncer(ui.ctx(), textes::AVIS_AJOUTE_AU_RUBAN);
                    }
                    None => {}
                }
            }
            Vue::Aide => match vues::aide::afficher(ui, p, &mut self.recherche, self.fiche_ouverte)
            {
                Some(vues::aide::Action::Ouvrir(i)) => self.fiche_ouverte = Some(i),
                Some(vues::aide::Action::Fermer) => self.fiche_ouverte = None,
                None => {}
            },
            Vue::Preferences => {
                let avant = self.moteur.preferences().clone();
                let action = vues::preferences::afficher(ui, p, &mut self.moteur);
                if *self.moteur.preferences() != avant {
                    self.a_sauvegarder = true;
                }
                match action {
                    Some(vues::preferences::Action::Terminer) => self.aller(Vue::Accueil),
                    Some(vues::preferences::Action::Modifier(quoi)) => {
                        self.dialogue = Dialogue::ModifierMemorisee {
                            quoi,
                            texte: quoi.texte(self.moteur.memorises(), self.moteur.preferences()),
                            unite: self.moteur.memorises().masse_volumique.unite,
                            refusee: false,
                            nouveau: true,
                        };
                    }
                    Some(vues::preferences::Action::Reinitialiser) => {
                        self.dialogue = Dialogue::Confirmer(Confirmation::Reinitialiser);
                    }
                    Some(vues::preferences::Action::Langue(langue)) => {
                        self.changer_de_langue(langue);
                        ui.ctx().request_repaint();
                    }
                    None => {}
                }
            }
            Vue::Bienvenue => {
                if vues::bienvenue::afficher(ui, p) {
                    self.moteur.marquer_bienvenue_vue();
                    self.a_sauvegarder = true;
                    self.aller(Vue::Accueil);
                }
            }
            Vue::Trucs => {
                if vues::trucs::afficher(ui, p) {
                    self.aller(Vue::Accueil);
                }
            }
            Vue::APropos => {
                if vues::a_propos::afficher(ui, p, self.depot.emplacement().as_deref()) {
                    self.aller(Vue::Accueil);
                }
            }
        }
    }

    /// Panneau du ruban courant, à droite dans une fenêtre large (section 14.2).
    fn panneau_ruban(&mut self, ui: &mut Ui, p: &Palette) {
        theme::peindre_le_fond(ui, ui.max_rect(), p);
        let action = egui::Frame::new()
            .inner_margin(egui::Margin::symmetric(14, 0))
            .show(ui, |ui| {
                ui.add_space(14.0);
                ui.label(
                    RichText::new(textes::RUBAN_COURANT.traduit())
                        .font(theme::police_grasse(18.0))
                        .color(p.titre),
                );
                ui.add_space(4.0);
                let hauteur = (ui.available_height() - 76.0).max(80.0);
                rubans::ruban_courant(ui, p, self.moteur.ruban(), hauteur)
            })
            .inner;
        if let Some(action) = action {
            self.traiter_action_ruban(ui.ctx(), action);
        }
    }

    // ------------------------------------------------------------------
    // Tiroir du menu (section 15.1)
    // ------------------------------------------------------------------

    fn tiroir(&mut self, ctx: &egui::Context, p: &Palette) {
        if !self.tiroir_ouvert {
            return;
        }
        let ecran = ctx.content_rect();
        let mut choix: Option<ChoixMenu> = None;
        let mut fermer = false;
        // Lien touché au pied du menu : (adresse, dans un nouvel onglet).
        let mut lien: Option<(&str, bool)> = None;
        // Les entrées du menu dépendent de la plateforme : on ne « quitte » pas
        // une page Web (on ferme son onglet), et seul un navigateur peut
        // installer l'application sur l'appareil.
        let mut elements: Vec<(&str, ChoixMenu)> = vec![
            (
                textes::MENU_METIERS.traduit(),
                ChoixMenu::Aller(Vue::Metiers),
            ),
            (textes::MENU_DESSIN.traduit(), ChoixMenu::Aller(Vue::Dessin)),
            (textes::MENU_RUBANS.traduit(), ChoixMenu::Aller(Vue::Rubans)),
            (textes::MENU_NOTES.traduit(), ChoixMenu::Aller(Vue::Notes)),
            (textes::MENU_AIDE.traduit(), ChoixMenu::Aller(Vue::Aide)),
            (textes::MENU_MANUEL.traduit(), ChoixMenu::Manuel),
            (
                textes::MENU_BIENVENUE.traduit(),
                ChoixMenu::Aller(Vue::Bienvenue),
            ),
            (textes::MENU_TRUCS.traduit(), ChoixMenu::Aller(Vue::Trucs)),
            (
                textes::MENU_PREFERENCES.traduit(),
                ChoixMenu::Aller(Vue::Preferences),
            ),
            (
                if self.moteur.preferences().mode_chantier {
                    textes::MENU_QUITTER_MODE_CHANTIER.traduit()
                } else {
                    textes::MENU_MODE_CHANTIER.traduit()
                },
                ChoixMenu::ModeChantier,
            ),
            (
                textes::MENU_EXPORTER_DONNEES.traduit(),
                ChoixMenu::ExporterDonnees,
            ),
            (
                textes::MENU_IMPORTER_DONNEES.traduit(),
                ChoixMenu::ImporterDonnees,
            ),
            (
                textes::MENU_A_PROPOS.traduit(),
                ChoixMenu::Aller(Vue::APropos),
            ),
        ];
        if !matches!(plateforme::installation(), Installation::Aucune) {
            elements.push((textes::MENU_INSTALLER.traduit(), ChoixMenu::Installer));
        }
        if !plateforme::NAVIGATEUR {
            elements.push((textes::MENU_QUITTER.traduit(), ChoixMenu::Quitter));
        }
        egui::Area::new(egui::Id::unique("tiroir"))
            .order(egui::Order::Foreground)
            .fixed_pos(ecran.min)
            .show(ctx, |ui| {
                ui.allocate_rect(ecran, Sense::hover());
                let voile = ui.interact(ecran, egui::Id::unique("tiroir-voile"), Sense::CLICK);
                ui.painter().rect_filled(ecran, 0, p.voile);
                let largeur = 280.0f32.min(ecran.width() * 0.82);
                let panneau = Rect::from_min_size(ecran.min, Vec2::new(largeur, ecran.height()));
                let mut fin_des_entrees = panneau.top();
                // Le fond du panneau absorbe les clics.
                ui.interact(panneau, egui::Id::unique("tiroir-fond"), Sense::CLICK);
                theme::ombre_portee(ui.painter(), panneau, 0, p.ombre, (24, 0));
                theme::rect_degrade(ui.painter(), panneau, 0, p.pied_fond, p.pied_fond_bas);
                let bord = Rect::from_min_max(
                    Pos2::new(panneau.right() - 1.0, panneau.top()),
                    panneau.right_bottom(),
                );
                ui.painter()
                    .rect_filled(bord, 0, Color32::from_white_alpha(24));
                ui.scope_builder(
                    egui::UiBuilder::new().max_rect(panneau.shrink2(Vec2::new(12.0, 14.0))),
                    |ui| {
                        ui.label(
                            RichText::new(textes::NOM_APPLICATION.traduit())
                                .font(theme::police_grasse(17.0))
                                .color(p.barre_texte),
                        );
                        ui.add_space(8.0);
                        let (filet, _) = ui.allocate_exact_size(
                            Vec2::new(ui.available_width(), 2.0),
                            Sense::hover(),
                        );
                        ui.painter().rect_filled(filet, 1, p.barre_accent);
                        ui.add_space(6.0);
                        for (libelle, element) in elements {
                            // Texte blanc; le fond s'éclaire au survol.
                            let (zone, reponse) = ui.allocate_exact_size(
                                Vec2::new(ui.available_width(), 42.0),
                                Sense::click(),
                            );
                            reponse.widget_info(|| {
                                egui::WidgetInfo::labeled(egui::Role::Button, true, libelle)
                            });
                            let actif = element == ChoixMenu::Aller(self.vue);
                            if reponse.hovered() || actif {
                                let voile = if actif { 30 } else { 18 };
                                ui.painter()
                                    .rect_filled(zone, 6, Color32::from_white_alpha(voile));
                            }
                            ui.painter().text(
                                Pos2::new(zone.left() + 12.0, zone.center().y),
                                Align2::LEFT_CENTER,
                                libelle,
                                theme::police(15.5),
                                p.barre_texte,
                            );
                            if reponse.clicked() {
                                choix = Some(element);
                            }
                        }
                        fin_des_entrees = ui.min_rect().bottom();
                    },
                );
                // Au pied du menu, de bas en haut : la version du programme, le
                // site et le courriel de Constructo AI, la mention de l'auteur.
                let attenue = theme::melanger(p.barre_texte, p.pied_fond, 0.30);
                let gauche = panneau.left() + 24.0;
                let version = ui.painter().layout_no_wrap(
                    textes::version_et_revision(),
                    theme::police(11.5),
                    attenue,
                );
                let mention = ui.painter().layout(
                    textes::GRACIEUSETE.traduit().to_owned(),
                    theme::police(12.0),
                    attenue,
                    panneau.width() - 48.0,
                );
                const HAUTEUR_DE_LIEN: f32 = 26.0;
                let liens = [
                    (
                        textes::SITE_CONSTRUCTO,
                        textes::LIEN_SITE_CONSTRUCTO,
                        textes::INFOBULLE_SITE.traduit(),
                        true,
                    ),
                    (
                        textes::COURRIEL_CONSTRUCTO,
                        textes::LIEN_COURRIEL_CONSTRUCTO,
                        textes::INFOBULLE_COURRIEL.traduit(),
                        false,
                    ),
                ];
                let hauteur_du_pied = 14.0
                    + version.size().y
                    + 4.0
                    + HAUTEUR_DE_LIEN * liens.len() as f32
                    + 4.0
                    + mention.size().y;
                // Dans une fenêtre trop basse, les entrées du menu passent d'abord.
                if panneau.bottom() - hauteur_du_pied >= fin_des_entrees + 6.0 {
                    let mut y = panneau.bottom() - 14.0 - version.size().y;
                    ui.painter().galley(Pos2::new(gauche, y), version, attenue);
                    y -= 4.0;
                    for (texte, adresse, infobulle, nouvel_onglet) in liens {
                        let ecrit = ui.painter().layout_no_wrap(
                            texte.to_owned(),
                            theme::police(13.0),
                            p.barre_texte,
                        );
                        let zone = Rect::from_min_size(
                            Pos2::new(gauche - 6.0, y - HAUTEUR_DE_LIEN),
                            Vec2::new(ecrit.size().x + 12.0, HAUTEUR_DE_LIEN),
                        );
                        let reponse = ui
                            .interact(
                                zone,
                                egui::Id::unique(("lien-du-menu", adresse)),
                                Sense::CLICK,
                            )
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .on_hover_text(infobulle);
                        reponse.widget_info(|| {
                            egui::WidgetInfo::labeled(egui::Role::Link, true, texte)
                        });
                        let position = Pos2::new(gauche, zone.center().y - ecrit.size().y / 2.0);
                        // Un filet sous le texte : c'est un lien; il s'affirme au survol.
                        let filet = if reponse.hovered() { 200 } else { 90 };
                        let sous = position.y + ecrit.size().y + 0.5;
                        ui.painter().line_segment(
                            [
                                Pos2::new(position.x, sous),
                                Pos2::new(position.x + ecrit.size().x, sous),
                            ],
                            Stroke::new(1.0, Color32::from_white_alpha(filet)),
                        );
                        ui.painter().galley(position, ecrit, p.barre_texte);
                        if reponse.clicked() {
                            lien = Some((adresse, nouvel_onglet));
                        }
                        y -= HAUTEUR_DE_LIEN;
                    }
                    y -= 4.0;
                    let position = Pos2::new(gauche, y - mention.size().y);
                    ui.painter().galley(position, mention, attenue);
                }
                if voile.clicked() {
                    fermer = true;
                }
            });
        match choix {
            Some(ChoixMenu::Aller(vue)) => self.aller(vue),
            Some(ChoixMenu::ExporterDonnees) => {
                self.tiroir_ouvert = false;
                self.exporter_les_donnees(ctx);
            }
            Some(ChoixMenu::ModeChantier) => {
                self.tiroir_ouvert = false;
                let prefs = self.moteur.preferences_mut();
                prefs.mode_chantier = !prefs.mode_chantier;
                self.a_sauvegarder = true;
            }
            Some(ChoixMenu::ImporterDonnees) => {
                self.tiroir_ouvert = false;
                self.fichiers_pour = Some(Destination::Sauvegarde);
                plateforme::choisir_des_fichiers(ChoixDeFichiers::Sauvegarde);
            }
            Some(ChoixMenu::Manuel) => {
                self.tiroir_ouvert = false;
                ctx.open_url(egui::OpenUrl::new_tab(plateforme::adresse_du_manuel(
                    &self.depot,
                )));
            }
            Some(ChoixMenu::Installer) => {
                self.tiroir_ouvert = false;
                match plateforme::installation() {
                    // Le navigateur sait installer l'application : on le lui demande.
                    Installation::Proposee => plateforme::installer(),
                    // Sinon, on explique où se trouve la commande du navigateur.
                    Installation::Manuelle => {
                        self.dialogue = Dialogue::Avis(textes::AVIS_INSTALLER);
                    }
                    Installation::Aucune => {}
                }
            }
            Some(ChoixMenu::Quitter) => {
                self.sauvegarder(ctx);
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            None if fermer => self.tiroir_ouvert = false,
            None => {}
        }
        if let Some((adresse, nouvel_onglet)) = lien {
            ctx.open_url(egui::OpenUrl {
                url: adresse.to_owned(),
                new_tab: nouvel_onglet,
            });
        }
    }

    // ------------------------------------------------------------------
    // Dialogues
    // ------------------------------------------------------------------

    fn dialogues(&mut self, ctx: &egui::Context, p: &Palette) {
        let dialogue = std::mem::replace(&mut self.dialogue, Dialogue::Aucun);
        let suivant = match dialogue {
            Dialogue::Aucun => Dialogue::Aucun,
            Dialogue::Avis(texte) => {
                let (ok, fermee) = modale(ctx, 300.0, |ui| {
                    ui.label(texte.traduit());
                    ui.add_space(10.0);
                    ui.vertical_centered(|ui| ui.button(textes::OK.traduit()).clicked())
                        .inner
                });
                if ok || fermee {
                    Dialogue::Aucun
                } else {
                    Dialogue::Avis(texte)
                }
            }
            Dialogue::Confirmer(question) => self.dialogue_confirmation(ctx, p, question),
            Dialogue::NommerRuban {
                mut texte,
                renommer,
                nouveau,
            } => {
                let titre = if renommer.is_some() {
                    textes::TITRE_RENOMMER_RUBAN.traduit()
                } else {
                    textes::TITRE_ENREGISTRER_RUBAN.traduit()
                };
                let (choix, fermee) = modale(ctx, 320.0, |ui| {
                    ui.label(RichText::new(titre).font(theme::police_grasse(17.0)));
                    ui.add_space(6.0);
                    ui.label(textes::NOM_DU_RUBAN.traduit());
                    let champ =
                        ui.add(egui::TextEdit::singleline(&mut texte).desired_width(f32::INFINITY));
                    vues::cadre_de_saisie(ui, p, &champ);
                    if nouveau {
                        champ.request_focus();
                    }
                    let entree = ui.input(|i| i.key_pressed(egui::Key::Enter));
                    ui.add_space(10.0);
                    boutons(ui, p, textes::ENREGISTRER.traduit(), false).or(entree.then_some(true))
                });
                let nom = texte.trim().to_string();
                match choix {
                    Some(true) if !nom.is_empty() => {
                        match renommer {
                            Some(id) => {
                                if let Some(r) = self.rubans.iter_mut().find(|r| r.id == id) {
                                    r.nom = nom;
                                }
                                self.ecrire_rubans();
                            }
                            None => self.enregistrer_ruban(ctx, nom),
                        }
                        std::mem::replace(&mut self.dialogue, Dialogue::Aucun)
                    }
                    Some(false) => Dialogue::Aucun,
                    _ if fermee => Dialogue::Aucun,
                    _ => Dialogue::NommerRuban {
                        texte,
                        renommer,
                        nouveau: false,
                    },
                }
            }
            Dialogue::NommerDessin {
                mut texte,
                id,
                nouveau,
            } => {
                let (choix, fermee) = modale(ctx, 320.0, |ui| {
                    ui.label(
                        RichText::new(textes::TITRE_NOMMER_DESSIN.traduit())
                            .font(theme::police_grasse(17.0)),
                    );
                    ui.add_space(6.0);
                    ui.label(textes::NOM_DU_DESSIN.traduit());
                    let champ = ui.add(
                        egui::TextEdit::singleline(&mut texte)
                            .char_limit(60)
                            .desired_width(f32::INFINITY),
                    );
                    vues::cadre_de_saisie(ui, p, &champ);
                    if nouveau {
                        champ.request_focus();
                    }
                    let entree = ui.input(|i| i.key_pressed(egui::Key::Enter));
                    ui.add_space(10.0);
                    boutons(ui, p, textes::ENREGISTRER.traduit(), false).or(entree.then_some(true))
                });
                let nom = texte.trim().to_string();
                match choix {
                    Some(true) if !nom.is_empty() => {
                        if let Some(dessin) = self.dessins.iter_mut().find(|d| d.id == id) {
                            dessin.nom = nom;
                        }
                        self.dessins_a_sauvegarder = true;
                        Dialogue::Aucun
                    }
                    Some(false) => Dialogue::Aucun,
                    _ if fermee => Dialogue::Aucun,
                    _ => Dialogue::NommerDessin {
                        texte,
                        id,
                        nouveau: false,
                    },
                }
            }
            Dialogue::ModifierMemorisee {
                quoi,
                mut texte,
                mut unite,
                mut refusee,
                nouveau,
            } => {
                let (choix, fermee) = modale(ctx, 330.0, |ui| {
                    ui.label(RichText::new(quoi.nom()).font(theme::police_grasse(17.0)));
                    ui.add_space(6.0);
                    let champ =
                        ui.add(egui::TextEdit::singleline(&mut texte).desired_width(f32::INFINITY));
                    vues::cadre_de_saisie(ui, p, &champ);
                    if nouveau {
                        champ.request_focus();
                    }
                    if champ.changed() {
                        refusee = false;
                    }
                    if quoi == Memorisee::MasseVolumique {
                        ui.horizontal(|ui| {
                            ui.label(textes::UNITE.traduit());
                            egui::ComboBox::from_id_salt("unite-masse-volumique")
                                .selected_text(textes::unite_masse_volumique(unite))
                                .show_ui(ui, |ui| {
                                    for u in UniteMasseVolumique::TOUTES {
                                        ui.selectable_value(
                                            &mut unite,
                                            u,
                                            textes::unite_masse_volumique(u),
                                        );
                                    }
                                });
                        });
                    }
                    ui.label(
                        RichText::new(quoi.exemples())
                            .size(12.5)
                            .color(p.texte_attenue),
                    );
                    if refusee {
                        ui.label(
                            RichText::new(textes::VALEUR_NON_RECONNUE.traduit()).color(p.erreur),
                        );
                    }
                    if ui.link(textes::VALEUR_PAR_DEFAUT.traduit()).clicked() {
                        let defaut = ValeursMemorisees::default();
                        texte = quoi.texte(&defaut, self.moteur.preferences());
                        unite = defaut.masse_volumique.unite;
                        refusee = false;
                    }
                    let entree = ui.input(|i| i.key_pressed(egui::Key::Enter));
                    ui.add_space(10.0);
                    boutons(ui, p, textes::ENREGISTRER.traduit(), false).or(entree.then_some(true))
                });
                match choix {
                    Some(true) => {
                        if quoi.enregistrer(&mut self.moteur, &texte, unite) {
                            self.a_sauvegarder = true;
                            Dialogue::Aucun
                        } else {
                            // Saisie invalide : rien n'est enregistré.
                            Dialogue::ModifierMemorisee {
                                quoi,
                                texte,
                                unite,
                                refusee: true,
                                nouveau: false,
                            }
                        }
                    }
                    Some(false) => Dialogue::Aucun,
                    None if fermee => Dialogue::Aucun,
                    None => Dialogue::ModifierMemorisee {
                        quoi,
                        texte,
                        unite,
                        refusee,
                        nouveau: false,
                    },
                }
            }
            Dialogue::AideTouche {
                touche,
                mut secondaire,
            } => {
                let trigo = self.moteur.preferences().mode_trigo;
                let principale = aide::fiche_principale(touche, trigo);
                let seconde = aide::fiche_secondaire(touche, trigo);
                let hauteur = (ctx.content_rect().height() * 0.62).max(200.0);
                let (fermer, fermee) = modale(ctx, 350.0, |ui| {
                    // Deux onglets si la touche a une fonction secondaire.
                    if principale.is_some() && seconde.is_some() {
                        ui.horizontal(|ui| {
                            ui.selectable_value(
                                &mut secondaire,
                                false,
                                textes::FONCTION_PRINCIPALE.traduit(),
                            );
                            ui.selectable_value(
                                &mut secondaire,
                                true,
                                textes::FONCTION_SECONDAIRE.traduit(),
                            );
                        });
                        ui.separator();
                    }
                    let fiche = match (principale, seconde) {
                        (Some(a), Some(b)) => Some(if secondaire { b } else { a }),
                        (a, b) => a.or(b),
                    };
                    egui::ScrollArea::vertical()
                        .max_height(hauteur)
                        .auto_shrink([false, true])
                        .show(ui, |ui| match fiche {
                            Some(f) => vues::aide::dessiner_fiche(ui, p, f),
                            None => {
                                ui.label(textes::AUCUNE_FICHE.traduit());
                            }
                        });
                    ui.add_space(8.0);
                    ui.vertical_centered(|ui| ui.button(textes::FERMER.traduit()).clicked())
                        .inner
                });
                if fermer || fermee {
                    Dialogue::Aucun
                } else {
                    Dialogue::AideTouche { touche, secondaire }
                }
            }
            Dialogue::ChoisirMetier => {
                let actuel = self.moteur.outil().map(|c| c.metier);
                let hauteur = (ctx.content_rect().height() * 0.62).max(200.0);
                let mention = |en_cours: bool| {
                    if en_cours {
                        textes::METIER_ACTUEL.traduit()
                    } else {
                        ""
                    }
                };
                let ((choix, fermer), fermee) = modale(ctx, 350.0, |ui| {
                    // `Some(None)` : le clavier de charpente.
                    let mut choix: Option<Option<Metier>> = None;
                    ui.label(
                        RichText::new(textes::TITRE_CHOISIR_METIER.traduit())
                            .font(theme::police_grasse(17.0)),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(textes::CHOISIR_METIER_EXPLICATION.traduit())
                            .size(13.0)
                            .color(p.texte_attenue),
                    );
                    ui.add_space(8.0);
                    egui::ScrollArea::vertical()
                        .max_height(hauteur)
                        .auto_shrink([false, true])
                        .show(ui, |ui| {
                            if vues::metiers::carte_cliquable(
                                ui,
                                p,
                                "metier-charpente",
                                textes::CHARPENTE_NOM.traduit(),
                                textes::CHARPENTE_RESUME.traduit(),
                                mention(actuel.is_none()),
                            ) {
                                choix = Some(None);
                            }
                            for metier in Metier::TOUS {
                                if vues::metiers::carte_cliquable(
                                    ui,
                                    p,
                                    ("metier-au-clavier", metier),
                                    metier.nom(),
                                    metier.resume(),
                                    mention(actuel == Some(metier)),
                                ) {
                                    choix = Some(Some(metier));
                                }
                            }
                        });
                    ui.add_space(8.0);
                    let fermer = ui
                        .vertical_centered(|ui| ui.button(textes::FERMER.traduit()).clicked())
                        .inner;
                    (choix, fermer)
                });
                match choix {
                    Some(metier) => {
                        self.choisir_metier(metier);
                        Dialogue::Aucun
                    }
                    None if fermer || fermee => Dialogue::Aucun,
                    None => Dialogue::ChoisirMetier,
                }
            }
            Dialogue::AideOutil(id) => match noyau::metiers::calculateur(id) {
                None => Dialogue::Aucun,
                Some(c) => {
                    let hauteur = (ctx.content_rect().height() * 0.62).max(200.0);
                    let ((fermer, formulaire), fermee) = modale(ctx, 350.0, |ui| {
                        ui.label(RichText::new(c.nom).font(theme::police_grasse(17.0)));
                        ui.add_space(4.0);
                        egui::ScrollArea::vertical()
                            .max_height(hauteur)
                            .auto_shrink([false, true])
                            .show(ui, |ui| {
                                ui.label(c.resume.traduit());
                                ui.add_space(8.0);
                                ui.label(
                                    RichText::new(textes::AIDE_OUTIL_TITRE.traduit())
                                        .font(theme::police_grasse(14.5))
                                        .color(p.titre),
                                );
                                ui.add_space(4.0);
                                for (i, (titre, texte)) in textes::AIDE_OUTIL.iter().enumerate() {
                                    vues::point_numerote(
                                        ui,
                                        p,
                                        i + 1,
                                        titre.traduit(),
                                        texte.traduit(),
                                    );
                                }
                                if !c.note.est_vide() {
                                    ui.add_space(6.0);
                                    ui.label(
                                        RichText::new(c.note).size(13.0).color(p.texte_attenue),
                                    );
                                }
                            });
                        ui.add_space(10.0);
                        let mut fermer = false;
                        let mut formulaire = false;
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            fermer = ui.button(textes::FERMER.traduit()).clicked();
                            formulaire = ui
                                .button(textes::OUVRIR_LE_FORMULAIRE.traduit())
                                .on_hover_text(textes::INFOBULLE_OUVRIR_LE_FORMULAIRE.traduit())
                                .clicked();
                        });
                        (fermer, formulaire)
                    });
                    if formulaire {
                        self.position_metiers = vues::metiers::Position {
                            metier: Some(c.metier),
                            calculateur: Some(c.id),
                        };
                        self.aller(Vue::Metiers);
                        Dialogue::Aucun
                    } else if fermer || fermee {
                        Dialogue::Aucun
                    } else {
                        Dialogue::AideOutil(id)
                    }
                }
            },
        };
        // Une action confirmée peut avoir ouvert un autre dialogue.
        if !self.dialogue_ouvert() {
            self.dialogue = suivant;
        }
    }

    fn dialogue_confirmation(
        &mut self,
        ctx: &egui::Context,
        p: &Palette,
        question: Confirmation,
    ) -> Dialogue {
        let (titre, message, bouton) = match &question {
            Confirmation::EffacerTout => (
                textes::TITRE_EFFACER_TOUT.traduit(),
                textes::CONFIRMER_EFFACER_TOUT.traduit(),
                textes::BOUTON_EFFACER_TOUT.traduit(),
            ),
            Confirmation::Reinitialiser => (
                textes::TITRE_REINITIALISER.traduit(),
                textes::CONFIRMER_REINITIALISER.traduit(),
                textes::BOUTON_REINITIALISER.traduit(),
            ),
            Confirmation::ViderRuban => (
                textes::TITRE_VIDER_RUBAN.traduit(),
                textes::CONFIRMER_VIDER_RUBAN.traduit(),
                textes::VIDER.traduit(),
            ),
            Confirmation::SupprimerRuban(_) => (
                textes::TITRE_SUPPRIMER_RUBAN.traduit(),
                textes::CONFIRMER_SUPPRIMER_RUBAN.traduit(),
                textes::SUPPRIMER.traduit(),
            ),
            Confirmation::SupprimerNote(_) => (
                textes::TITRE_SUPPRIMER_NOTE.traduit(),
                textes::CONFIRMER_SUPPRIMER_NOTE.traduit(),
                textes::SUPPRIMER.traduit(),
            ),
            Confirmation::RetirerPiece(_) => (
                textes::TITRE_RETIRER_PIECE.traduit(),
                textes::CONFIRMER_RETIRER_PIECE.traduit(),
                textes::RETIRER.traduit(),
            ),
            Confirmation::SupprimerDessin(_) => (
                textes::TITRE_SUPPRIMER_DESSIN.traduit(),
                textes::CONFIRMER_SUPPRIMER_DESSIN.traduit(),
                textes::SUPPRIMER.traduit(),
            ),
            Confirmation::Restaurer => (
                textes::TITRE_RESTAURER.traduit(),
                textes::CONFIRMER_RESTAURER.traduit(),
                textes::REMPLACER.traduit(),
            ),
            Confirmation::TropDeRubans(_) => (
                textes::TITRE_ENREGISTRER_RUBAN.traduit(),
                textes::TROP_DE_RUBANS.traduit(),
                textes::SUPPRIMER_LES_PLUS_ANCIENS.traduit(),
            ),
        };
        let (choix, fermee) = modale(ctx, 320.0, |ui| {
            ui.label(RichText::new(titre).font(theme::police_grasse(17.0)));
            ui.add_space(6.0);
            ui.label(message);
            // La date de la sauvegarde à importer.
            if matches!(question, Confirmation::Restaurer)
                && let Some(date) = self
                    .sauvegarde_a_restaurer
                    .as_ref()
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s.date).ok())
            {
                ui.add_space(4.0);
                ui.label(
                    RichText::new(textes::sauvegarde_du(&textes::date_en_clair(
                        date.year(),
                        date.month(),
                        date.day(),
                        date.hour(),
                        date.minute(),
                    )))
                    .font(theme::police_grasse(14.0)),
                );
            }
            ui.add_space(12.0);
            boutons(ui, p, bouton, true)
        });
        match choix {
            Some(true) => {
                match question {
                    Confirmation::EffacerTout => self.moteur.effacer_tout(),
                    Confirmation::Reinitialiser => {
                        self.moteur.reinitialiser();
                        // Le thème et le plein écran suivent les préférences remises à neuf.
                        self.palette_appliquee = None;
                    }
                    Confirmation::ViderRuban => self.moteur.vider_ruban(),
                    Confirmation::SupprimerRuban(id) => {
                        self.rubans.retain(|r| r.id != id);
                        if self.ruban_ouvert == Some(id) {
                            self.ruban_ouvert = None;
                        }
                        self.ecrire_rubans();
                    }
                    Confirmation::SupprimerNote(id) => {
                        // Ses pièces jointes s'effacent avec elle.
                        let a_effacer: Vec<PieceJointe> = self
                            .notes
                            .iter()
                            .find(|n| n.id == id)
                            .map(|n| n.pieces.clone())
                            .unwrap_or_default();
                        for piece in &a_effacer {
                            self.effacer_la_piece(piece);
                        }
                        self.notes.retain(|n| n.id != id);
                        if self.note_ouverte == Some(id) {
                            self.note_ouverte = None;
                            self.oublier_les_images();
                        }
                        self.notes_a_sauvegarder = true;
                    }
                    Confirmation::RetirerPiece(id) => {
                        let retiree = self.notes.iter_mut().find_map(|note| {
                            let position = note.pieces.iter().position(|p| p.id == id)?;
                            note.modifiee = stockage::maintenant().0;
                            Some(note.pieces.remove(position))
                        });
                        if let Some(piece) = retiree {
                            self.effacer_la_piece(&piece);
                        }
                        self.notes_a_sauvegarder = true;
                    }
                    Confirmation::Restaurer => {
                        self.restaurer_la_sauvegarde(ctx);
                        // L'application est rechargée : rien d'autre à faire.
                        return Dialogue::Aucun;
                    }
                    Confirmation::SupprimerDessin(id) => {
                        self.dessins.retain(|d| d.id != id);
                        if self.dessin_courant == id {
                            // Le plus récent des autres s'ouvre, ou un dessin neuf.
                            let suivant = self
                                .dessins
                                .iter()
                                .max_by(|a, b| a.modifie.cmp(&b.modifie))
                                .map(|d| d.id);
                            let suivant = match suivant {
                                Some(id) => id,
                                None => self.ajouter_un_dessin_vide(),
                            };
                            self.ouvrir_le_dessin(suivant);
                            self.liste_des_dessins = true;
                        }
                        self.dessins_a_sauvegarder = true;
                    }
                    Confirmation::TropDeRubans(nom) => {
                        // Les plus anciens sont à la fin de la liste.
                        self.rubans.truncate(MAX_RUBANS - 1);
                        self.enregistrer_ruban(ctx, nom);
                    }
                }
                self.a_sauvegarder = true;
                Dialogue::Aucun
            }
            Some(false) => Dialogue::Aucun,
            None if fermee => Dialogue::Aucun,
            None => Dialogue::Confirmer(question),
        }
    }

    /// Confirmation discrète, en bas de la fenêtre.
    fn confirmation_fugace(&mut self, ctx: &egui::Context, p: &Palette) {
        let Some((texte, fin)) = self.fugace else {
            return;
        };
        if ctx.input(|i| i.time) >= fin {
            self.fugace = None;
            return;
        }
        // « Copié » s'affiche dans l'afficheur quand la calculatrice est à l'écran.
        if texte == textes::COPIE && self.vue == Vue::Accueil {
            return;
        }
        egui::Area::new(egui::Id::unique("confirmation-fugace"))
            .order(egui::Order::Tooltip)
            .anchor(
                Align2::CENTER_BOTTOM,
                Vec2::new(0.0, -HAUTEUR_BARRE_INFERIEURE - 14.0),
            )
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(p.barre_fond)
                    .corner_radius(16)
                    .inner_margin(egui::Margin::symmetric(14, 8))
                    .show(ui, |ui| {
                        ui.label(RichText::new(texte.traduit()).color(p.barre_texte));
                    });
            });
    }
}

impl eframe::App for Application {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        // Pièces jointes lues, fichiers choisis : ce qui est arrivé depuis l'image d'avant.
        self.relever_les_arrivees(&ctx);
        self.appliquer_la_langue(&ctx);
        let large = self.regler_echelle(&ctx);
        let p = self.appliquer_theme(&ctx);
        self.appliquer_plein_ecran(&ctx);
        self.traiter_clavier(&ctx);

        let sans_marge = |fond: Color32| egui::Frame::new().fill(fond);
        if large {
            // Le ruban occupe ce qui reste à droite de la calculatrice (section 16.1).
            let largeur = (ctx.content_rect().width() - LARGEUR_NOMINALE).max(120.0);
            egui::Panel::right("panneau-ruban")
                .exact_size(largeur)
                .resizable(false)
                .show_separator_line(false)
                .frame(sans_marge(p.fond_app))
                .show(ui, |ui| self.panneau_ruban(ui, &p));
        }
        egui::Panel::top("barre-superieure")
            .exact_size(HAUTEUR_BARRE_SUPERIEURE)
            .show_separator_line(false)
            .frame(sans_marge(p.barre_fond))
            .show(ui, |ui| self.barre_superieure(ui, &p));
        // Mode chantier : sur l'écran de la calculatrice, le clavier descend
        // jusqu'au bas de l'écran; le menu mène aux autres écrans (section 16.7).
        if !(p.chantier && self.vue == Vue::Accueil) {
            egui::Panel::bottom("barre-inferieure")
                .exact_size(HAUTEUR_BARRE_INFERIEURE)
                .show_separator_line(false)
                .frame(sans_marge(p.barre_fond))
                .show(ui, |ui| self.barre_inferieure(ui, &p));
        }
        egui::CentralPanel::default()
            .frame(sans_marge(p.fond_app))
            .show(ui, |ui| self.centre(ui, &p));

        self.tiroir(&ctx, &p);
        self.dialogues(&ctx, &p);
        self.confirmation_fugace(&ctx, &p);
        self.sauvegarder_si_du(&ctx);
    }

    // Bureau seulement : une page Web se ferme sans préavis, et chaque
    // changement y est déjà enregistré sur-le-champ. (La signature de cette
    // méthode dépend aussi du moteur de rendu, qui n'est pas le même sur le Web.)
    #[cfg(not(target_arch = "wasm32"))]
    fn on_exit(&mut self) {
        // L'état est sauvegardé à la fermeture (R-45).
        let _ = stockage::enregistrer_etat(&self.depot, &self.moteur.etat_persistant());
        if self.saisies_a_sauvegarder {
            let outil = self.moteur.outil().map(|c| c.id);
            let langue = self.moteur.preferences().langue;
            let _ = stockage::enregistrer_saisies(
                &self.depot,
                &self.saisies_metiers,
                outil,
                langue,
                &self.favoris,
            );
        }
        if self.dessin_a_sauvegarder {
            let langue = self.moteur.preferences().langue;
            let courant = Some(self.dessin_courant);
            let _ = stockage::enregistrer_dessin(&self.depot, &self.plan, langue, courant);
        }
        // La bibliothèque reçoit le dessin ouvert, tel qu'on le quitte.
        self.synchroniser_le_dessin_courant();
        let _ = stockage::enregistrer_dessins(&self.depot, &self.dessins);
        if self.notes_a_sauvegarder {
            let _ = stockage::enregistrer_notes(&self.depot, &self.notes);
        }
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        let (sombre, chantier) = self.palette_appliquee.unwrap_or((false, false));
        let c = theme::palette(sombre, chantier).fond_app;
        [
            f32::from(c.r()) / 255.0,
            f32::from(c.g()) / 255.0,
            f32::from(c.b()) / 255.0,
            1.0,
        ]
    }
}

// ----------------------------------------------------------------------
// Éléments communs
// ----------------------------------------------------------------------

/// Affiche un dialogue modal. Retourne la valeur du contenu et vrai si
/// l'utilisateur l'a fermé (Échap ou clic à côté).
fn modale<T>(ctx: &egui::Context, largeur: f32, contenu: impl FnOnce(&mut Ui) -> T) -> (T, bool) {
    let largeur = largeur.min(ctx.content_rect().width() - 40.0).max(160.0);
    let reponse = egui::Modal::new(egui::Id::unique("dialogue")).show(ctx, |ui| {
        ui.set_width(largeur);
        contenu(ui)
    });
    let fermee = reponse.should_close();
    (reponse.inner, fermee)
}

/// Récrit toutes les saisies des calculateurs comme elles se tapent dans une
/// autre langue (section 24). Sans effet si les deux langues sont les mêmes.
fn transposer_les_saisies(
    saisies: &mut stockage::Saisies,
    de: Langue,
    vers: Langue,
    prefs: &Preferences,
) {
    if de == vers {
        return;
    }
    for (id, textes) in saisies.iter_mut() {
        if let Some(c) = noyau::metiers::calculateur(id) {
            c.transposer_les_saisies(textes, de, vers, prefs);
        }
    }
}

/// Bouton de langue de la barre supérieure : « FR | EN », la langue en cours
/// en évidence. Retourne la langue touchée, si ce n'est pas celle en cours.
fn bouton_de_langue(ui: &mut Ui, zone: Rect, p: &Palette, en_cours: Langue) -> Option<Langue> {
    let pastille = Rect::from_center_size(zone.center(), Vec2::new(zone.width() - 8.0, 26.0));
    let moitie = pastille.width() / Langue::TOUTES.len() as f32;
    let mut choisie = None;
    // Le contour de la pastille.
    ui.painter().rect_stroke(
        pastille,
        13,
        Stroke::new(1.0, Color32::from_white_alpha(110)),
        egui::StrokeKind::Inside,
    );
    for (i, langue) in Langue::TOUTES.into_iter().enumerate() {
        // Chaque moitié se touche sur toute la hauteur de la barre.
        let gauche = pastille.left() + moitie * i as f32;
        let touchable = Rect::from_min_max(
            Pos2::new(gauche, zone.top()),
            Pos2::new(gauche + moitie, zone.bottom()),
        );
        let reponse = ui
            .interact(
                touchable,
                egui::Id::unique(("bouton-de-langue", langue)),
                Sense::CLICK,
            )
            .on_hover_cursor(egui::CursorIcon::PointingHand);
        reponse.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, langue.nom()));
        let active = langue == en_cours;
        let case = Rect::from_min_size(
            Pos2::new(gauche, pastille.top()),
            Vec2::new(moitie, pastille.height()),
        );
        let (fond, encre) = if active {
            (Some(p.barre_texte), p.barre_fond)
        } else if reponse.hovered() {
            (Some(Color32::from_white_alpha(40)), p.barre_texte)
        } else {
            (None, theme::melanger(p.barre_texte, p.barre_fond, 0.25))
        };
        if let Some(fond) = fond {
            ui.painter().rect_filled(case.shrink(2.0), 11, fond);
        }
        ui.painter().text(
            case.center(),
            Align2::CENTER_CENTER,
            langue.code(),
            theme::police_grasse(12.5),
            encre,
        );
        if reponse.clicked() && !active {
            choisie = Some(langue);
        }
        reponse.on_hover_text(textes::infobulle_de_langue(en_cours));
    }
    choisie
}

/// Rangée de boutons d'un dialogue : `Annuler` et l'action. Retourne
/// `Some(true)` pour l'action, `Some(false)` pour l'annulation.
fn boutons(ui: &mut Ui, p: &Palette, action: &str, destructive: bool) -> Option<bool> {
    let mut choix = None;
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        let (fond, texte) = if destructive {
            (p.touche_effacer.fond, p.touche_effacer.texte)
        } else {
            (p.selection, p.selection_texte)
        };
        let bouton = egui::Button::new(
            RichText::new(action)
                .font(theme::police_grasse(14.5))
                .color(texte),
        )
        .fill(fond)
        .stroke(Stroke::new(1.0, fond));
        if ui.add(bouton).clicked() {
            choix = Some(true);
        }
        if ui.button(textes::ANNULER.traduit()).clicked() {
            choix = Some(false);
        }
    });
    choix
}

/// Bouton carré de la barre supérieure, à pictogramme dessiné.
fn bouton_icone(
    ui: &mut Ui,
    rect: Rect,
    nom: &str,
    infobulle: &str,
    p: &Palette,
    dessin: Dessin,
) -> bool {
    let reponse = ui.interact(
        rect,
        egui::Id::unique(("barre-superieure", nom)),
        Sense::CLICK,
    );
    reponse.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, nom));
    let painter = ui.painter();
    if reponse.hovered() {
        painter.rect_filled(rect.shrink(6.0), 6, Color32::from_white_alpha(28));
    }
    dessin(
        painter,
        Rect::from_center_size(rect.center(), Vec2::splat(22.0)),
        p.barre_texte,
    );
    let clic = reponse.clicked();
    reponse.on_hover_text(infobulle);
    clic
}

fn dessiner_menu(painter: &Painter, r: Rect, couleur: Color32) {
    for fraction in [0.22, 0.5, 0.78] {
        let y = r.top() + r.height() * fraction;
        painter.line_segment(
            [Pos2::new(r.left(), y), Pos2::new(r.right(), y)],
            Stroke::new(2.0, couleur),
        );
    }
}

fn dessiner_plein_ecran(painter: &Painter, r: Rect, couleur: Color32) {
    let trait_ = Stroke::new(2.0, couleur);
    let b = r.width() * 0.34;
    let r = r.shrink(1.5);
    for (coin, dx, dy) in [
        (r.left_top(), 1.0, 1.0),
        (r.right_top(), -1.0, 1.0),
        (r.left_bottom(), 1.0, -1.0),
        (r.right_bottom(), -1.0, -1.0),
    ] {
        painter.line_segment([coin, Pos2::new(coin.x + b * dx, coin.y)], trait_);
        painter.line_segment([coin, Pos2::new(coin.x, coin.y + b * dy)], trait_);
    }
}

fn dessiner_roue(painter: &Painter, r: Rect, couleur: Color32) {
    let c = r.center();
    let rayon = r.width() * 0.30;
    let trait_ = Stroke::new(2.0, couleur);
    painter.circle_stroke(c, rayon, trait_);
    painter.circle_filled(c, rayon * 0.32, couleur);
    for i in 0..8 {
        let angle = std::f32::consts::TAU * i as f32 / 8.0;
        let direction = Vec2::new(angle.cos(), angle.sin());
        painter.line_segment(
            [c + direction * rayon, c + direction * (rayon * 1.55)],
            Stroke::new(3.0, couleur),
        );
    }
}

/// Un numéro neuf : l'heure en millisecondes, ou le suivant du plus grand
/// numéro déjà donné. Il ne resservira pas (R-97).
fn nouveau_numero(existants: impl Iterator<Item = u64>) -> u64 {
    let plus_grand = existants.max().unwrap_or(0);
    let maintenant = u64::try_from(chrono::Utc::now().timestamp_millis()).unwrap_or(0);
    maintenant.max(plus_grand + 1)
}

/// Une texture d'egui faite d'une photo ou d'une vignette jointe à une note;
/// rien si l'image est illisible.
fn texture_de(ctx: &egui::Context, nom: &str, octets: &[u8]) -> Option<egui::TextureHandle> {
    let (largeur, hauteur, rvba) = pieces::pixels(octets)?;
    let image =
        egui::ColorImage::from_rgba_unmultiplied([largeur as usize, hauteur as usize], &rvba);
    Some(ctx.load_texture(nom, image, egui::TextureOptions::LINEAR))
}

fn dessiner_onglet_accueil(painter: &Painter, r: Rect, couleur: Color32) {
    // Une petite calculatrice : un afficheur et des touches.
    let corps = r.shrink2(Vec2::new(3.0, 0.5));
    painter.rect_stroke(
        corps,
        3,
        Stroke::new(1.6, couleur),
        egui::StrokeKind::Inside,
    );
    let afficheur = Rect::from_min_max(
        Pos2::new(corps.left() + 3.5, corps.top() + 3.5),
        Pos2::new(corps.right() - 3.5, corps.top() + 8.0),
    );
    painter.rect_filled(afficheur, 1, couleur);
    for ligne in 0..2 {
        for colonne in 0..3 {
            let centre = Pos2::new(
                corps.left() + corps.width() * (0.25 + 0.25 * colonne as f32),
                corps.top() + 12.5 + 5.0 * ligne as f32,
            );
            painter.circle_filled(centre, 1.3, couleur);
        }
    }
}

fn dessiner_onglet_dessin(painter: &Painter, r: Rect, couleur: Color32) {
    // Le plan d'une pièce en L, avec le point d'où part le tracé.
    let trait_ = Stroke::new(1.7, couleur);
    let (g, d, h, b) = (
        r.left() + 3.0,
        r.right() - 3.0,
        r.top() + 3.5,
        r.bottom() - 3.5,
    );
    let (mx, my) = (r.center().x + 1.0, r.center().y);
    let coins = vec![
        Pos2::new(g, h),
        Pos2::new(d, h),
        Pos2::new(d, my),
        Pos2::new(mx, my),
        Pos2::new(mx, b),
        Pos2::new(g, b),
    ];
    painter.add(egui::Shape::closed_line(coins, trait_));
    painter.circle_filled(Pos2::new(g, h), 2.4, couleur);
}

fn dessiner_onglet_metiers(painter: &Painter, r: Rect, couleur: Color32) {
    // Un casque de chantier : la calotte, sa nervure et la visière.
    let trait_ = Stroke::new(1.7, couleur);
    let base = r.bottom() - 5.5;
    let centre = Pos2::new(r.center().x, base);
    let rayon = r.width() * 0.37;
    let calotte: Vec<Pos2> = (0..=18)
        .map(|i| {
            let angle = std::f32::consts::PI * (1.0 + i as f32 / 18.0);
            Pos2::new(
                centre.x + rayon * angle.cos(),
                centre.y + rayon * angle.sin(),
            )
        })
        .collect();
    painter.add(egui::Shape::line(calotte, trait_));
    painter.line_segment(
        [
            Pos2::new(centre.x, base - rayon - 2.6),
            Pos2::new(centre.x, base - rayon * 0.35),
        ],
        trait_,
    );
    painter.line_segment(
        [
            Pos2::new(r.left() + 0.5, base),
            Pos2::new(r.right() - 0.5, base),
        ],
        Stroke::new(2.3, couleur),
    );
}

fn dessiner_onglet_rubans(painter: &Painter, r: Rect, couleur: Color32) {
    // Un ruban : des lignes de longueurs inégales, alignées à droite.
    let trait_ = Stroke::new(1.8, couleur);
    for (i, part) in [0.55, 0.85, 0.40, 1.0].into_iter().enumerate() {
        let y = r.top() + 3.0 + 5.2 * i as f32;
        painter.line_segment(
            [
                Pos2::new(r.right() - r.width() * part, y),
                Pos2::new(r.right(), y),
            ],
            trait_,
        );
    }
}

fn dessiner_onglet_notes(painter: &Painter, r: Rect, couleur: Color32) {
    // Un bloc-notes : la feuille, sa reliure en haut, et trois lignes.
    let feuille =
        Rect::from_center_size(r.center(), Vec2::new(r.width() * 0.72, r.height() * 0.92));
    let trait_ = Stroke::new(1.6, couleur);
    painter.rect_stroke(feuille, 2, trait_, egui::StrokeKind::Middle);
    painter.line_segment(
        [
            Pos2::new(feuille.left(), feuille.top() + 3.5),
            Pos2::new(feuille.right(), feuille.top() + 3.5),
        ],
        trait_,
    );
    for k in 0..3 {
        let y = feuille.top() + 7.5 + 3.6 * k as f32;
        painter.line_segment(
            [
                Pos2::new(feuille.left() + 3.0, y),
                Pos2::new(feuille.right() - 3.0, y),
            ],
            Stroke::new(1.2, couleur),
        );
    }
}

fn dessiner_onglet_aide(painter: &Painter, r: Rect, couleur: Color32) {
    painter.circle_stroke(r.center(), r.width() * 0.46, Stroke::new(1.6, couleur));
    painter.text(
        r.center(),
        Align2::CENTER_CENTER,
        textes::SYMBOLE_AIDE,
        theme::police_grasse(14.0),
        couleur,
    );
}

/// Récrit les mesures du dessin comme elles se tapent dans une autre langue
/// (section 24) : les valeurs restent les mêmes.
fn transposer_le_dessin(
    dessin: &mut noyau::dessin::Dessin,
    de: Langue,
    vers: Langue,
    prefs: &Preferences,
) {
    if de == vers {
        return;
    }
    let decimal = noyau::format::separateur_decimal_en(prefs, vers);
    let mesures = dessin
        .cotes
        .iter_mut()
        .map(|cote| &mut cote.saisie)
        .chain(
            dessin
                .retraits
                .iter_mut()
                .flat_map(|retrait| [&mut retrait.longueur, &mut retrait.largeur]),
        )
        .chain(dessin.ouvertures.iter_mut().flat_map(|ouverture| {
            [
                &mut ouverture.largeur,
                &mut ouverture.hauteur,
                &mut ouverture.allege,
                &mut ouverture.position,
            ]
        }))
        .chain(
            dessin
                .objets
                .iter_mut()
                .flat_map(|objet| [&mut objet.largeur, &mut objet.profondeur]),
        )
        .chain(dessin.cloisons.iter_mut().flat_map(|cloison| {
            let noyau::cloison::Cloison {
                longueur,
                epaisseur,
                porte,
                suite,
                ..
            } = cloison;
            let mut saisies = vec![longueur, epaisseur];
            saisies.extend(suite.iter_mut().map(|segment| &mut segment.longueur));
            if let Some(porte) = porte {
                saisies.push(&mut porte.largeur);
                saisies.push(&mut porte.position);
            }
            saisies
        }))
        .chain([
            &mut dessin.hauteur_des_murs,
            &mut dessin.epaisseur_des_murs,
            &mut dessin.epaisseur_des_cloisons,
        ]);
    for saisie in mesures {
        if !saisie.trim().is_empty() {
            *saisie = noyau::analyse::transposer(saisie, de, vers, decimal);
        }
    }
}
