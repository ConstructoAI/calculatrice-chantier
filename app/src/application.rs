//! L'application : fenêtre, navigation, dialogues, clavier d'ordinateur et
//! sauvegarde. Aucune formule ici (R-49) : tout le calcul est dans le noyau.

use std::time::Duration;

use eframe::egui::{self, Align2, Color32, Painter, Pos2, Rect, RichText, Sense, Stroke, Ui, Vec2};
use noyau::analyse::analyser_collage;
use noyau::metiers::Metier;
use noyau::prefs::{SeparateurDecimal, Theme, ValeursMemorisees};
use noyau::unites::UniteMasseVolumique;
use noyau::{Effet, Moteur, Prefixe, Touche};

use crate::afficheur::{self, ActionAfficheur};
use crate::aide;
use crate::clavier::{self, ActionClavier, Contexte, EtatClavier};
use crate::plateforme::{self, Depot, Export, Fichier, Installation};
use crate::raccourcis::{self, Action};
use crate::stockage::{self, MAX_RUBANS, RubanEnregistre};
use crate::textes;
use crate::theme::{self, Palette};
use crate::vues::preferences::Memorisee;
use crate::vues::{self, rubans};

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
    Rubans,
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
    Quitter,
}

/// Question posée à l'utilisateur avant une action irréversible.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Confirmation {
    EffacerTout,
    Reinitialiser,
    ViderRuban,
    SupprimerRuban(u64),
    /// Limite de rubans atteinte : supprimer les plus anciens pour enregistrer celui-ci.
    TropDeRubans(String),
}

/// Dialogue ouvert par-dessus l'écran.
enum Dialogue {
    Aucun,
    Avis(&'static str),
    Confirmer(Confirmation),
    NommerRuban {
        texte: String,
        renommer: Option<u64>,
        /// Premier affichage : le champ prend le focus.
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
    /// La pastille de l'outil branché doit être amenée en vue dans la barre.
    outil_a_montrer: bool,
    /// Confirmation discrète (« Copié », « Ruban enregistré ») et son échéance.
    fugace: Option<(&'static str, f64)>,
    a_sauvegarder: bool,
    /// Moment de la dernière écriture de l'état, en secondes de l'horloge d'egui.
    derniere_sauvegarde: f64,
    echec_d_ecriture_signale: bool,
    plein_ecran_applique: bool,
    sombre_applique: Option<bool>,
    /// Texture des dégradés; elle doit vivre aussi longtemps que l'application.
    _rampe: egui::TextureHandle,
}

impl Application {
    pub fn nouvelle(cc: &eframe::CreationContext<'_>) -> Self {
        theme::installer_les_polices(&cc.egui_ctx);
        // L'échelle suit la taille de la fenêtre : pas de zoom au clavier.
        cc.egui_ctx.options_mut(|o| o.zoom_with_keyboard = false);
        let depot = Depot::ouvrir();
        let chargement = stockage::charger_etat(&depot);
        let rubans = stockage::charger_rubans(&depot);
        let (saisies_metiers, outil) = stockage::charger_saisies(&depot);
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
            saisies_a_sauvegarder: false,
            derniers_outils: Vec::new(),
            outil_a_montrer: true,
            fugace: None,
            a_sauvegarder: chargement.illisible,
            // Le premier changement est enregistré sans attendre.
            derniere_sauvegarde: f64::NEG_INFINITY,
            echec_d_ecriture_signale: false,
            plein_ecran_applique: false,
            sombre_applique: None,
            _rampe: theme::creer_la_rampe(&cc.egui_ctx),
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
        // On reviendra à cet outil en revenant à ce métier.
        if let Some(c) = apres {
            self.derniers_outils
                .retain(|(metier, _)| *metier != c.metier);
            self.derniers_outils.push((c.metier, c.id));
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
        let mut choix = None;
        ui.scope_builder(
            egui::UiBuilder::new()
                .max_rect(zone.shrink2(Vec2::new(8.0, 3.0)))
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
            |ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                let nom = outil.map_or(textes::METIER_CHARPENTE, |c| c.metier.nom_court());
                if vues::pastille(ui, p, nom, vues::Pastille::Menu)
                    .on_hover_text(textes::INFOBULLE_METIER)
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
                        ui.horizontal_centered(|ui| match outil {
                            // Un métier est choisi : ses outils, celui du clavier en évidence.
                            Some(actif) => {
                                for c in actif.metier.calculateurs() {
                                    let genre = if c.id == actif.id {
                                        vues::Pastille::Active
                                    } else {
                                        vues::Pastille::Ordinaire
                                    };
                                    let reponse =
                                        vues::pastille(ui, p, c.nom, genre).on_hover_text(c.resume);
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

    fn annoncer(&mut self, ctx: &egui::Context, texte: &'static str) {
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
            if stockage::enregistrer_saisies(&self.depot, &self.saisies_metiers, outil).is_err() {
                self.signaler_l_echec_d_ecriture();
            }
        }
    }

    /// Sur le bureau, au plus une écriture par seconde (R-45); dans un
    /// navigateur, sans délai.
    fn sauvegarder_si_du(&mut self, ctx: &egui::Context) {
        if !self.a_sauvegarder && !self.saisies_a_sauvegarder {
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
        let fichier = Fichier {
            titre: textes::TITRE_EXPORTER,
            filtre: textes::FILTRE_TEXTE,
            nom: &stockage::nom_de_fichier(&nom),
            contenu: &stockage::contenu_exporte(&texte),
        };
        match plateforme::exporter(&fichier) {
            Export::Reussi => self.annoncer(ctx, textes::AVIS_EXPORT_REUSSI),
            Export::Annule => {}
            Export::Echec => self.dialogue = Dialogue::Avis(textes::AVIS_EXPORT_ECHEC),
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
        let sombre = match self.moteur.preferences().theme {
            Theme::Clair => false,
            Theme::Sombre => true,
            Theme::Automatique => ctx.system_theme() == Some(egui::Theme::Dark),
        };
        if self.sombre_applique != Some(sombre) {
            theme::installer_le_style(ctx, theme::palette(sombre));
            self.sombre_applique = Some(sombre);
        }
        *theme::palette(sombre)
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
            textes::NOM_MENU,
            textes::INFOBULLE_MENU,
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
                textes::NOM_PLEIN_ECRAN,
                textes::INFOBULLE_PLEIN_ECRAN,
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
            textes::PREFERENCES,
            textes::INFOBULLE_PREFERENCES,
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
        // Logo : la petite équerre orange de l'icône.
        let logo = Rect::from_center_size(
            Pos2::new(
                rect.left() + boutons_a_gauche * cote + 12.0,
                rect.center().y,
            ),
            Vec2::splat(17.0),
        );
        dessiner_equerre(ui.painter(), logo, p.barre_accent);
        let zone_titre = Rect::from_min_max(
            Pos2::new(logo.right() + 9.0, rect.top()),
            droite.left_bottom(),
        );
        let police = theme::police_grasse(16.5);
        let g = ui.painter().layout_no_wrap(
            textes::NOM_APPLICATION.to_owned(),
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
            textes::NOM_APPLICATION,
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
        let onglets: [(Vue, &str, Dessin); 4] = [
            (
                Vue::Accueil,
                textes::ONGLET_ACCUEIL,
                dessiner_onglet_accueil,
            ),
            (Vue::Metiers, textes::METIERS, dessiner_onglet_metiers),
            (Vue::Rubans, textes::ONGLET_RUBANS, dessiner_onglet_rubans),
            (Vue::Aide, textes::ONGLET_AIDE, dessiner_onglet_aide),
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
        let contexte_base = (
            prefs.mode_trigo,
            prefs.separateur_decimal == SeparateurDecimal::Point,
        );
        let hauteur_afficheur = (rect.height() * 0.20).clamp(118.0, 240.0);
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
                let action = vues::metiers::afficher(
                    ui,
                    p,
                    &mut self.position_metiers,
                    &mut self.saisies_metiers,
                    self.moteur.preferences(),
                );
                match action {
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
                    RichText::new(textes::RUBAN_COURANT)
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
            (textes::MENU_METIERS, ChoixMenu::Aller(Vue::Metiers)),
            (textes::MENU_RUBANS, ChoixMenu::Aller(Vue::Rubans)),
            (textes::MENU_AIDE, ChoixMenu::Aller(Vue::Aide)),
            (textes::MENU_MANUEL, ChoixMenu::Manuel),
            (textes::MENU_BIENVENUE, ChoixMenu::Aller(Vue::Bienvenue)),
            (textes::MENU_TRUCS, ChoixMenu::Aller(Vue::Trucs)),
            (textes::MENU_PREFERENCES, ChoixMenu::Aller(Vue::Preferences)),
            (textes::MENU_A_PROPOS, ChoixMenu::Aller(Vue::APropos)),
        ];
        if !matches!(plateforme::installation(), Installation::Aucune) {
            elements.push((textes::MENU_INSTALLER, ChoixMenu::Installer));
        }
        if !plateforme::NAVIGATEUR {
            elements.push((textes::MENU_QUITTER, ChoixMenu::Quitter));
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
                            RichText::new(textes::NOM_APPLICATION)
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
                    textes::GRACIEUSETE.to_owned(),
                    theme::police(12.0),
                    attenue,
                    panneau.width() - 48.0,
                );
                const HAUTEUR_DE_LIEN: f32 = 26.0;
                let liens = [
                    (
                        textes::SITE_CONSTRUCTO,
                        textes::LIEN_SITE_CONSTRUCTO,
                        textes::INFOBULLE_SITE,
                        true,
                    ),
                    (
                        textes::COURRIEL_CONSTRUCTO,
                        textes::LIEN_COURRIEL_CONSTRUCTO,
                        textes::INFOBULLE_COURRIEL,
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
                    ui.label(texte);
                    ui.add_space(10.0);
                    ui.vertical_centered(|ui| ui.button(textes::OK).clicked())
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
                    textes::TITRE_RENOMMER_RUBAN
                } else {
                    textes::TITRE_ENREGISTRER_RUBAN
                };
                let (choix, fermee) = modale(ctx, 320.0, |ui| {
                    ui.label(RichText::new(titre).font(theme::police_grasse(17.0)));
                    ui.add_space(6.0);
                    ui.label(textes::NOM_DU_RUBAN);
                    let champ =
                        ui.add(egui::TextEdit::singleline(&mut texte).desired_width(f32::INFINITY));
                    vues::cadre_de_saisie(ui, p, &champ);
                    if nouveau {
                        champ.request_focus();
                    }
                    let entree = ui.input(|i| i.key_pressed(egui::Key::Enter));
                    ui.add_space(10.0);
                    boutons(ui, p, textes::ENREGISTRER, false).or(entree.then_some(true))
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
                            ui.label(textes::UNITE);
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
                        ui.label(RichText::new(textes::VALEUR_NON_RECONNUE).color(p.erreur));
                    }
                    if ui.link(textes::VALEUR_PAR_DEFAUT).clicked() {
                        let defaut = ValeursMemorisees::default();
                        texte = quoi.texte(&defaut, self.moteur.preferences());
                        unite = defaut.masse_volumique.unite;
                        refusee = false;
                    }
                    let entree = ui.input(|i| i.key_pressed(egui::Key::Enter));
                    ui.add_space(10.0);
                    boutons(ui, p, textes::ENREGISTRER, false).or(entree.then_some(true))
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
                                textes::FONCTION_PRINCIPALE,
                            );
                            ui.selectable_value(&mut secondaire, true, textes::FONCTION_SECONDAIRE);
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
                                ui.label(textes::AUCUNE_FICHE);
                            }
                        });
                    ui.add_space(8.0);
                    ui.vertical_centered(|ui| ui.button(textes::FERMER).clicked())
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
                    if en_cours { textes::METIER_ACTUEL } else { "" }
                };
                let ((choix, fermer), fermee) = modale(ctx, 350.0, |ui| {
                    // `Some(None)` : le clavier de charpente.
                    let mut choix: Option<Option<Metier>> = None;
                    ui.label(
                        RichText::new(textes::TITRE_CHOISIR_METIER)
                            .font(theme::police_grasse(17.0)),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(textes::CHOISIR_METIER_EXPLICATION)
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
                                textes::CHARPENTE_NOM,
                                textes::CHARPENTE_RESUME,
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
                        .vertical_centered(|ui| ui.button(textes::FERMER).clicked())
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
                                ui.label(c.resume);
                                ui.add_space(8.0);
                                ui.label(
                                    RichText::new(textes::AIDE_OUTIL_TITRE)
                                        .font(theme::police_grasse(14.5))
                                        .color(p.titre),
                                );
                                ui.add_space(4.0);
                                for (i, (titre, texte)) in textes::AIDE_OUTIL.iter().enumerate() {
                                    vues::point_numerote(ui, p, i + 1, titre, texte);
                                }
                                if !c.note.is_empty() {
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
                            fermer = ui.button(textes::FERMER).clicked();
                            formulaire = ui
                                .button(textes::OUVRIR_LE_FORMULAIRE)
                                .on_hover_text(textes::INFOBULLE_OUVRIR_LE_FORMULAIRE)
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
                textes::TITRE_EFFACER_TOUT,
                textes::CONFIRMER_EFFACER_TOUT,
                textes::BOUTON_EFFACER_TOUT,
            ),
            Confirmation::Reinitialiser => (
                textes::TITRE_REINITIALISER,
                textes::CONFIRMER_REINITIALISER,
                textes::BOUTON_REINITIALISER,
            ),
            Confirmation::ViderRuban => (
                textes::TITRE_VIDER_RUBAN,
                textes::CONFIRMER_VIDER_RUBAN,
                textes::VIDER,
            ),
            Confirmation::SupprimerRuban(_) => (
                textes::TITRE_SUPPRIMER_RUBAN,
                textes::CONFIRMER_SUPPRIMER_RUBAN,
                textes::SUPPRIMER,
            ),
            Confirmation::TropDeRubans(_) => (
                textes::TITRE_ENREGISTRER_RUBAN,
                textes::TROP_DE_RUBANS,
                textes::SUPPRIMER_LES_PLUS_ANCIENS,
            ),
        };
        let (choix, fermee) = modale(ctx, 320.0, |ui| {
            ui.label(RichText::new(titre).font(theme::police_grasse(17.0)));
            ui.add_space(6.0);
            ui.label(message);
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
                        self.sombre_applique = None;
                    }
                    Confirmation::ViderRuban => self.moteur.vider_ruban(),
                    Confirmation::SupprimerRuban(id) => {
                        self.rubans.retain(|r| r.id != id);
                        if self.ruban_ouvert == Some(id) {
                            self.ruban_ouvert = None;
                        }
                        self.ecrire_rubans();
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
                        ui.label(RichText::new(texte).color(p.barre_texte));
                    });
            });
    }
}

impl eframe::App for Application {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
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
        egui::Panel::bottom("barre-inferieure")
            .exact_size(HAUTEUR_BARRE_INFERIEURE)
            .show_separator_line(false)
            .frame(sans_marge(p.barre_fond))
            .show(ui, |ui| self.barre_inferieure(ui, &p));
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
            let _ = stockage::enregistrer_saisies(&self.depot, &self.saisies_metiers, outil);
        }
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        let sombre = self.sombre_applique.unwrap_or(false);
        let c = theme::palette(sombre).fond_app;
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
        if ui.button(textes::ANNULER).clicked() {
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

/// Équerre de charpentier : deux branches à angle droit.
fn dessiner_equerre(painter: &Painter, r: Rect, couleur: Color32) {
    let epaisseur = r.width() * 0.30;
    let verticale = Rect::from_min_max(r.left_top(), Pos2::new(r.left() + epaisseur, r.bottom()));
    let horizontale = Rect::from_min_max(
        Pos2::new(r.left(), r.bottom() - epaisseur),
        r.right_bottom(),
    );
    painter.rect_filled(verticale, 1, couleur);
    painter.rect_filled(horizontale, 1, couleur);
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
