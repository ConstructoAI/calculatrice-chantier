//! Écran « Dessin » (section 25) : on trace le contour d'une pièce avec le
//! doigt ou la souris, puis on touche chaque côté pour y inscrire sa mesure.

use eframe::egui::{self, Align2, Color32, Pos2, Rect, RichText, Sense, Stroke, Ui, Vec2};
use noyau::dessin::{Dessin, Etat as EtatDeLaForme, Mesure, Solution};
use noyau::metiers::Quantite;
use noyau::prefs::Preferences;
use noyau::unites::Format;

use super::{bouton_principal, cadre_de_saisie, carte, corps, en_tete};
use crate::textes;
use crate::theme::{self, Palette};

/// Marge entre le bord de la planche et la forme : la place des mesures.
const MARGE_DE_LA_PLANCHE: f32 = 46.0;
/// Distance, en points, sous laquelle un toucher choisit un côté.
const PORTEE_DU_TOUCHER: f32 = 30.0;
/// Format d'un nombre inscrit sans unité : des pieds.
pub const FORMAT_PAR_DEFAUT: Format = Format::PiPo;

/// Ce que l'écran retient d'une image à l'autre.
#[derive(Default)]
pub struct Etat {
    /// Le côté dont on inscrit la mesure.
    pub choisi: Option<usize>,
    /// Le tracé en cours, en points de l'écran.
    trace: Vec<Pos2>,
    /// Vrai quand le champ de la mesure doit prendre le clavier.
    viser_le_champ: bool,
    /// Les états précédents du dessin, pour le bouton « Annuler ».
    historique: Vec<Dessin>,
}

/// Nombre d'étapes que le bouton « Annuler » peut remonter.
const PROFONDEUR_DE_L_HISTORIQUE: usize = 40;

impl Etat {
    /// Choisit un côté et place le curseur dans le champ de sa mesure.
    fn choisir(&mut self, rang: Option<usize>) {
        self.choisi = rang;
        self.viser_le_champ = rang.is_some();
    }

    /// Retient le dessin tel qu'il est, avant un changement.
    fn memoriser(&mut self, dessin: &Dessin) {
        if self.historique.last() != Some(dessin) {
            self.historique.push(dessin.clone());
            if self.historique.len() > PROFONDEUR_DE_L_HISTORIQUE {
                self.historique.remove(0);
            }
        }
    }
}

pub enum Action {
    /// Le dessin ou une mesure ont changé : à conserver.
    Modifie,
    Fermer,
    /// Inscrire la pièce au ruban : ses mesures, puis ses résultats.
    Consigner {
        entrees: Vec<(String, String)>,
        resultats: Vec<(String, String)>,
    },
}

/// Passage du croquis (ou de la forme à ses mesures) à l'écran.
#[derive(Clone, Copy)]
struct Echelle {
    origine: Pos2,
    facteur: f32,
}

impl Echelle {
    fn vers_l_ecran(&self, (x, y): (f64, f64)) -> Pos2 {
        Pos2::new(
            self.origine.x + x as f32 * self.facteur,
            self.origine.y + y as f32 * self.facteur,
        )
    }
}

/// Cadre la forme dans la planche. Sans forme : un point par unité, depuis le coin.
fn cadrer(planche: Rect, sommets: &[(f64, f64)]) -> Echelle {
    let (mut x0, mut x1, mut y0, mut y1) = (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
    for &(x, y) in sommets {
        x0 = x0.min(x);
        x1 = x1.max(x);
        y0 = y0.min(y);
        y1 = y1.max(y);
    }
    if sommets.len() < 2 {
        return Echelle {
            origine: planche.min,
            facteur: 1.0,
        };
    }
    let (largeur, hauteur) = ((x1 - x0) as f32, (y1 - y0) as f32);
    let utile = planche.shrink(MARGE_DE_LA_PLANCHE);
    let facteur = (utile.width() / largeur.max(1e-6))
        .min(utile.height() / hauteur.max(1e-6))
        .max(1e-6);
    let centre = planche.center();
    Echelle {
        origine: Pos2::new(
            centre.x - ((x0 + x1) / 2.0) as f32 * facteur,
            centre.y - ((y0 + y1) / 2.0) as f32 * facteur,
        ),
        facteur,
    }
}

/// Distance d'un point à un segment.
fn distance_au_segment(p: Pos2, a: Pos2, b: Pos2) -> f32 {
    let ab = b - a;
    let longueur2 = ab.length_sq();
    if longueur2 <= f32::EPSILON {
        return p.distance(a);
    }
    let t = ((p - a).dot(ab) / longueur2).clamp(0.0, 1.0);
    p.distance(a + ab * t)
}

/// Texte d'une longueur en pouces, dans le système des mesures inscrites.
fn longueur_en_clair(pouces: f64, solution: &Solution, prefs: &Preferences) -> String {
    Quantite::Longueur(pouces).en_clair(solution.modele, prefs)
}

/// Premier côté sans mesure (ni inscrite, ni déduite), s'il y en a un.
fn premier_cote_a_mesurer(solution: &Solution) -> Option<usize> {
    solution
        .mesures
        .iter()
        .position(|m| matches!(m, Mesure::Manquante | Mesure::Invalide))
}

pub fn afficher(
    ui: &mut Ui,
    p: &Palette,
    dessin: &mut Dessin,
    etat: &mut Etat,
    prefs: &Preferences,
) -> Option<Action> {
    let mut action = None;
    if en_tete(
        ui,
        p,
        textes::DESSIN_TITRE.traduit(),
        textes::FERMER.traduit(),
    ) {
        action = Some(Action::Fermer);
    }
    if etat.choisi.is_some_and(|i| i >= dessin.cotes.len()) {
        etat.choisi = None;
    }
    let solution = dessin.resoudre(FORMAT_PAR_DEFAUT);

    // La planche à dessin : hors de la zone qui défile, pour que le doigt trace
    // au lieu de faire défiler l'écran.
    let largeur = ui.available_width() - 20.0;
    let hauteur = (ui.available_height() - 250.0).clamp(170.0, largeur.max(170.0));
    ui.add_space(8.0);
    let (planche, reponse) = ui
        .horizontal(|ui| {
            ui.add_space(10.0);
            ui.allocate_exact_size(Vec2::new(largeur, hauteur), Sense::click_and_drag())
        })
        .inner;
    reponse.widget_info(|| {
        egui::WidgetInfo::labeled(egui::Role::Canvas, true, textes::DESSIN_PLANCHE.traduit())
    });
    if planche_et_gestes(ui, p, planche, &reponse, dessin, etat, &solution, prefs) {
        action = Some(Action::Modifie);
    }

    corps(ui, "dessin", |ui| {
        // Les mesures peuvent avoir changé pendant cette image.
        let solution = dessin.resoudre(FORMAT_PAR_DEFAUT);
        if boutons(ui, p, dessin, etat) {
            action = Some(Action::Modifie);
        }
        ui.add_space(6.0);
        if mesure_du_cote(ui, p, dessin, etat, &solution, prefs) {
            action = Some(Action::Modifie);
        }
        let solution = dessin.resoudre(FORMAT_PAR_DEFAUT);
        if let Some(consigner) = bilan(ui, p, dessin, &solution, prefs) {
            action = Some(consigner);
        }
        ui.add_space(10.0);
    });
    action
}

/// Dessine la planche et traite les gestes. Retourne vrai si le dessin a changé.
#[allow(clippy::too_many_arguments)]
fn planche_et_gestes(
    ui: &mut Ui,
    p: &Palette,
    planche: Rect,
    reponse: &egui::Response,
    dessin: &mut Dessin,
    etat: &mut Etat,
    solution: &Solution,
    prefs: &Preferences,
) -> bool {
    let mut modifie = false;
    let painter = ui.painter_at(planche);
    painter.rect_filled(planche, 10, p.carte);
    painter.rect_stroke(
        planche,
        10,
        Stroke::new(1.0, p.carte_bord),
        egui::StrokeKind::Inside,
    );
    // Un quadrillage léger, comme une feuille de chantier.
    let pas = 26.0;
    let mut x = planche.left() + pas;
    while x < planche.right() {
        painter.line_segment(
            [Pos2::new(x, planche.top()), Pos2::new(x, planche.bottom())],
            Stroke::new(1.0, p.grille),
        );
        x += pas;
    }
    let mut y = planche.top() + pas;
    while y < planche.bottom() {
        painter.line_segment(
            [Pos2::new(planche.left(), y), Pos2::new(planche.right(), y)],
            Stroke::new(1.0, p.grille),
        );
        y += pas;
    }

    // La forme à ses mesures quand elles sont toutes connues, sinon le croquis.
    let a_l_echelle = dessin.sommets_mesures(solution);
    let sommets_croquis = dessin.sommets_du_croquis();
    let sommets = a_l_echelle.as_deref().unwrap_or(&sommets_croquis);
    let echelle = cadrer(planche, sommets);
    // Le tracé, lui, se mesure toujours à l'échelle du croquis.
    let echelle_du_croquis = cadrer(planche, &sommets_croquis);
    let points: Vec<Pos2> = sommets.iter().map(|s| echelle.vers_l_ecran(*s)).collect();

    // --- Gestes
    if reponse.drag_started() {
        etat.trace.clear();
    }
    if reponse.dragged()
        && let Some(position) = reponse.interact_pointer_pos()
        && etat.trace.last().is_none_or(|d| d.distance(position) > 1.5)
    {
        etat.trace.push(position);
    }
    if reponse.drag_stopped() {
        let trace: Vec<(f64, f64)> = etat
            .trace
            .iter()
            .map(|point| {
                (
                    f64::from(
                        (point.x - echelle_du_croquis.origine.x) / echelle_du_croquis.facteur,
                    ),
                    f64::from(
                        (point.y - echelle_du_croquis.origine.y) / echelle_du_croquis.facteur,
                    ),
                )
            })
            .collect();
        etat.trace.clear();
        let avant = dessin.clone();
        if dessin.ajouter_trace(&trace) {
            etat.memoriser(&avant);
            modifie = true;
            if dessin.fermee {
                let solution = dessin.resoudre(FORMAT_PAR_DEFAUT);
                etat.choisir(premier_cote_a_mesurer(&solution).or(Some(0)));
            }
        }
    } else if reponse.clicked()
        && let Some(position) = reponse.interact_pointer_pos()
    {
        let plus_proche = points
            .windows(2)
            .enumerate()
            .map(|(i, s)| (i, distance_au_segment(position, s[0], s[1])))
            .min_by(|a, b| a.1.total_cmp(&b.1));
        match plus_proche {
            Some((i, distance)) if distance <= PORTEE_DU_TOUCHER => etat.choisir(Some(i)),
            _ => etat.choisir(None),
        }
    }

    // --- Dessin
    if dessin.est_vide() && etat.trace.is_empty() {
        painter.text(
            planche.center(),
            Align2::CENTER_CENTER,
            textes::DESSIN_INVITE.traduit(),
            theme::police(14.5),
            p.texte_attenue,
        );
    }
    for (i, s) in points.windows(2).enumerate() {
        let choisi = etat.choisi == Some(i);
        let (epaisseur, couleur) = if choisi {
            (5.0, p.barre_accent)
        } else {
            (3.0, p.primaire)
        };
        painter.line_segment([s[0], s[1]], Stroke::new(epaisseur, couleur));
    }
    for point in &points {
        painter.circle_filled(*point, 3.5, p.primaire);
    }
    if !dessin.fermee
        && let (Some(depart), Some(fin)) = (points.first(), points.last())
        && points.len() > 1
    {
        // Le départ à rejoindre, et le bout d'où repart le tracé.
        painter.circle_stroke(*depart, 8.0, Stroke::new(2.0, p.barre_accent));
        painter.circle_filled(*fin, 5.5, p.barre_accent);
    }
    // Les mesures, à côté de chaque côté, vers l'extérieur de la forme.
    let centre = if points.is_empty() {
        planche.center()
    } else {
        let somme = points.iter().fold(Vec2::ZERO, |s, q| s + q.to_vec2());
        (somme / points.len() as f32).to_pos2()
    };
    for (i, s) in points.windows(2).enumerate() {
        let (Some(cote), Some(mesure)) = (dessin.cotes.get(i), solution.mesures.get(i)) else {
            continue;
        };
        let milieu = s[0] + (s[1] - s[0]) * 0.5;
        let dehors = if cote.sens.horizontal() {
            Vec2::new(0.0, if milieu.y >= centre.y { 1.0 } else { -1.0 })
        } else {
            Vec2::new(if milieu.x >= centre.x { 1.0 } else { -1.0 }, 0.0)
        };
        let (texte, couleur) = match mesure {
            Mesure::Lue(_) => (cote.saisie.trim().to_string(), p.texte),
            Mesure::Deduite(pouces) => {
                (longueur_en_clair(*pouces, solution, prefs), p.texte_attenue)
            }
            Mesure::Manquante => ("?".to_string(), p.barre_accent),
            Mesure::Invalide => (cote.saisie.trim().to_string(), p.erreur),
        };
        let ancre = if cote.sens.horizontal() {
            if dehors.y > 0.0 {
                Align2::CENTER_TOP
            } else {
                Align2::CENTER_BOTTOM
            }
        } else if dehors.x > 0.0 {
            Align2::LEFT_CENTER
        } else {
            Align2::RIGHT_CENTER
        };
        let position = milieu + dehors * 7.0;
        let galere = painter.layout_no_wrap(texte, theme::police_grasse(12.5), couleur);
        let cadre = ancre
            .anchor_size(position, galere.size())
            .expand2(Vec2::new(3.0, 1.0));
        // La mesure reste dans la planche, même pour un côté collé au bord.
        let decalage = Vec2::new(
            (planche.left() + 3.0 - cadre.left()).max(0.0)
                - (cadre.right() - planche.right() + 3.0).max(0.0),
            (planche.top() + 3.0 - cadre.top()).max(0.0)
                - (cadre.bottom() - planche.bottom() + 3.0).max(0.0),
        );
        let cadre = cadre.translate(decalage);
        painter.rect_filled(cadre, 4, p.carte.gamma_multiply(0.92));
        painter.galley(cadre.min + Vec2::new(3.0, 1.0), galere, couleur);
    }
    // Le tracé en cours.
    if etat.trace.len() > 1 {
        painter.add(egui::Shape::line(
            etat.trace.clone(),
            Stroke::new(3.0, p.barre_accent),
        ));
    }
    modifie
}

/// Les boutons sous la planche. Retourne vrai si le dessin a changé.
fn boutons(ui: &mut Ui, p: &Palette, dessin: &mut Dessin, etat: &mut Etat) -> bool {
    let mut modifie = false;
    ui.horizontal_wrapped(|ui| {
        if !dessin.fermee
            && dessin.cotes.len() >= 2
            && ui
                .add(bouton_principal(
                    p,
                    textes::DESSIN_FERMER_LA_FORME.traduit(),
                    13.5,
                ))
                .clicked()
        {
            let avant = dessin.clone();
            if dessin.fermer() {
                etat.memoriser(&avant);
                modifie = true;
            }
        }
        if modifie {
            let solution = dessin.resoudre(FORMAT_PAR_DEFAUT);
            etat.choisir(premier_cote_a_mesurer(&solution).or(Some(0)));
        }
        let plein = !dessin.est_vide();
        if ui
            .add_enabled(
                !etat.historique.is_empty(),
                egui::Button::new(textes::DESSIN_ANNULER.traduit()),
            )
            .clicked()
            && let Some(precedent) = etat.historique.pop()
        {
            *dessin = precedent;
            etat.choisir(None);
            modifie = true;
        }
        if ui
            .add_enabled(plein, egui::Button::new(textes::DESSIN_RETIRER.traduit()))
            .clicked()
        {
            etat.memoriser(dessin);
            dessin.retirer_le_dernier();
            etat.choisir(None);
            modifie = true;
        }
        if ui
            .add_enabled(plein, egui::Button::new(textes::DESSIN_EFFACER.traduit()))
            .clicked()
        {
            etat.memoriser(dessin);
            dessin.effacer();
            etat.choisir(None);
            modifie = true;
        }
    });
    modifie
}

/// Le champ de la mesure du côté choisi. Retourne vrai si une mesure a changé.
fn mesure_du_cote(
    ui: &mut Ui,
    p: &Palette,
    dessin: &mut Dessin,
    etat: &mut Etat,
    solution: &Solution,
    prefs: &Preferences,
) -> bool {
    let Some(rang) = etat.choisi else {
        return false;
    };
    let nombre = dessin.cotes.len();
    let Some(cote) = dessin.cotes.get_mut(rang) else {
        return false;
    };
    let mut modifie = false;
    let mut suivant = false;
    carte(ui, p, |ui| {
        let libelle = ui.label(
            RichText::new(textes::dessin_cote(rang + 1, nombre)).font(theme::police_grasse(13.5)),
        );
        ui.horizontal(|ui| {
            // Une mesure déduite se lit dans le champ vide, en gris.
            let indice = match solution.mesures.get(rang) {
                Some(Mesure::Deduite(pouces)) => longueur_en_clair(*pouces, solution, prefs),
                _ => textes::DESSIN_EXEMPLE.traduit().to_string(),
            };
            let largeur = (ui.available_width() - 110.0).clamp(90.0, 260.0);
            let champ = egui::TextEdit::singleline(&mut cote.saisie)
                .desired_width(largeur)
                .hint_text(indice);
            let reponse = ui.add(champ).labelled_by(libelle.id);
            cadre_de_saisie(ui, p, &reponse);
            if etat.viser_le_champ {
                etat.viser_le_champ = false;
                reponse.request_focus();
            }
            if reponse.changed() {
                modifie = true;
            }
            let entree = reponse.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            if ui
                .add(bouton_principal(
                    p,
                    textes::DESSIN_COTE_SUIVANT.traduit(),
                    13.5,
                ))
                .clicked()
                || entree
            {
                suivant = true;
            }
        });
        if matches!(solution.mesures.get(rang), Some(Mesure::Invalide)) {
            ui.label(
                RichText::new(textes::DESSIN_MESURE_INVALIDE.traduit())
                    .size(12.5)
                    .color(p.erreur),
            );
        } else if matches!(solution.mesures.get(rang), Some(Mesure::Deduite(_))) {
            ui.label(
                RichText::new(textes::DESSIN_MESURE_DEDUITE.traduit())
                    .size(12.5)
                    .color(p.texte_attenue),
            );
        }
    });
    if suivant && nombre > 0 {
        // Le prochain côté sans mesure, sinon simplement le suivant.
        let solution = dessin.resoudre(FORMAT_PAR_DEFAUT);
        let prochain = (1..=nombre)
            .map(|k| (rang + k) % nombre)
            .find(|&i| matches!(solution.mesures.get(i), Some(Mesure::Manquante)));
        etat.choisir(Some(prochain.unwrap_or((rang + 1) % nombre)));
    }
    modifie
}

/// Où en est la forme : ce qu'il reste à faire, ou la surface et le périmètre.
fn bilan(
    ui: &mut Ui,
    p: &Palette,
    dessin: &Dessin,
    solution: &Solution,
    prefs: &Preferences,
) -> Option<Action> {
    let mut action = None;
    ui.add_space(8.0);
    let consigne = |ui: &mut Ui, texte: &str, couleur: Color32| {
        ui.label(RichText::new(texte).size(13.5).color(couleur));
    };
    match solution.etat {
        EtatDeLaForme::Vide => consigne(ui, textes::DESSIN_ETAPE_TRACER.traduit(), p.texte),
        EtatDeLaForme::Ouverte => consigne(ui, textes::DESSIN_ETAPE_FERMER.traduit(), p.texte),
        EtatDeLaForme::MesureInvalide => {
            consigne(ui, textes::DESSIN_MESURE_INVALIDE.traduit(), p.erreur);
        }
        EtatDeLaForme::Incomplete(manquantes) => {
            consigne(ui, &textes::dessin_etape_mesurer(manquantes), p.texte);
        }
        EtatDeLaForme::NeFermePas(ecart) => consigne(
            ui,
            &textes::dessin_ne_ferme_pas(&longueur_en_clair(ecart, solution, prefs)),
            p.erreur,
        ),
        EtatDeLaForme::SeCroise => consigne(ui, textes::DESSIN_SE_CROISE.traduit(), p.erreur),
        EtatDeLaForme::Complete { aire, perimetre } => {
            let mut resultats = vec![
                (
                    textes::DESSIN_SURFACE.traduit().to_string(),
                    Quantite::Aire(aire).en_clair(solution.modele, prefs),
                ),
                (
                    textes::DESSIN_PERIMETRE.traduit().to_string(),
                    longueur_en_clair(perimetre, solution, prefs),
                ),
            ];
            // Un rectangle est d'équerre quand ses deux diagonales ont cette longueur.
            if let [Some(a), Some(b), _, _] = solution
                .mesures
                .iter()
                .map(|m| m.pouces())
                .collect::<Vec<_>>()[..]
            {
                resultats.push((
                    textes::DESSIN_DIAGONALE.traduit().to_string(),
                    longueur_en_clair(a.hypot(b), solution, prefs),
                ));
            }
            carte(ui, p, |ui| {
                for (libelle, valeur) in &resultats {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(libelle)
                                .font(theme::police_grasse(13.5))
                                .color(p.titre),
                        );
                        ui.label(
                            RichText::new(valeur)
                                .font(theme::police_grasse(17.0))
                                .color(p.texte),
                        );
                    });
                }
                ui.add_space(4.0);
                if ui
                    .add(bouton_principal(p, textes::DESSIN_AU_RUBAN.traduit(), 13.5))
                    .clicked()
                {
                    let entrees = solution
                        .mesures
                        .iter()
                        .enumerate()
                        .filter_map(|(i, m)| {
                            m.pouces().map(|pouces| {
                                (
                                    textes::dessin_cote(i + 1, dessin.cotes.len()),
                                    longueur_en_clair(pouces, solution, prefs),
                                )
                            })
                        })
                        .collect();
                    action = Some(Action::Consigner {
                        entrees,
                        resultats: resultats.clone(),
                    });
                }
            });
        }
    }
    action
}
