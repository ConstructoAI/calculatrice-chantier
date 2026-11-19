//! Clavier : grille de 40 touches, étiquettes secondaires, bande de fractions
//! rapides, clics et appui long (section 16.4).

use eframe::egui::{
    self, Align2, Color32, Id, Painter, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2,
};
use noyau::Touche;

use crate::raccourcis;
use crate::textes;
use crate::theme::{self, Famille, Palette};

/// Ce que l'utilisateur a demandé en agissant sur le clavier.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ActionClavier {
    /// Appui sur une touche.
    Appui(Touche),
    /// Clic sur une étiquette secondaire : `Conv` puis la touche.
    Secondaire(Touche),
    /// Clic droit ou appui long : la fiche d'aide de la touche.
    Aide(Touche),
}

/// Durée de l'appui long, en secondes (annexe A.2).
const APPUI_LONG: f64 = 0.6;
/// Espacement entre les cases.
const ESPACE: f32 = 6.0;
/// Marge autour de la grille.
const MARGE: f32 = 8.0;
/// Part de la hauteur d'une case réservée à l'étiquette secondaire.
const PART_ETIQUETTE: f32 = 0.24;
/// Marge du libellé dans le bouton.
const MARGE_LIBELLE: f32 = 6.0;
/// Hauteur d'une rangée de fractions rapides.
pub const HAUTEUR_FRACTIONS: f32 = 34.0;
/// Rayon des coins d'une touche.
const RAYON_TOUCHE: u8 = 7;

/// Suivi de l'appui en cours, pour l'appui long.
#[derive(Clone, Copy, Debug)]
struct Appui {
    touche: Touche,
    debut: f64,
    aide_ouverte: bool,
}

/// État du clavier conservé d'une image à l'autre.
#[derive(Default)]
pub struct EtatClavier {
    appui: Option<Appui>,
}

/// Ce dont le dessin du clavier a besoin de savoir.
#[derive(Clone, Copy)]
pub struct Contexte {
    pub trigo: bool,
    pub point_decimal: bool,
    pub conv_arme: bool,
}

/// Largeur d'un texte sur une ligne.
fn largeur(painter: &Painter, texte: &str, police: egui::FontId) -> f32 {
    painter
        .layout_no_wrap(texte.to_owned(), police, Color32::WHITE)
        .size()
        .x
}

/// Plus grande taille de police (jusqu'à `plafond`) pour laquelle tous les
/// libellés tiennent dans `disponible`.
fn taille_ajustee<'a>(
    painter: &Painter,
    libelles: impl Iterator<Item = &'a str>,
    plafond: f32,
    disponible: f32,
    gras: bool,
) -> f32 {
    const REFERENCE: f32 = 20.0;
    let police = if gras {
        theme::police_grasse(REFERENCE)
    } else {
        theme::police_italique(REFERENCE)
    };
    let plus_large = libelles
        .flat_map(|l| l.split('\n'))
        .map(|ligne| largeur(painter, ligne, police.clone()))
        .fold(0.0f32, f32::max);
    if plus_large <= 0.0 {
        return plafond;
    }
    (REFERENCE * disponible / plus_large).min(plafond).max(7.0)
}

/// Un libellé court (chiffre, opérateur, symbole) s'écrit en grand.
fn est_court(libelle: &str) -> bool {
    libelle.chars().count() <= 2
}

/// Dessine le corps d'une touche et son libellé (texte et taille), selon son
/// état (enfoncée, survolée).
fn dessiner_bouton(
    painter: &Painter,
    rect: Rect,
    (libelle, taille): (&str, f32),
    p: &Palette,
    famille: Famille,
    (enfoncee, survolee): (bool, bool),
) {
    let couleurs = p.couleurs(famille);
    // Touche enfoncée : assombrie de 12 %; survolée : éclaircie de 6 %.
    let nuance = |c: Color32| {
        if enfoncee {
            theme::assombrir(c, 0.12)
        } else if survolee {
            theme::eclaircir(c, 0.06)
        } else {
            c
        }
    };
    let rayon = RAYON_TOUCHE;
    // La touche enfoncée descend de 1 point.
    let decalage = if enfoncee { 1.0 } else { 0.0 };
    // Ombre portée : neutre sous les touches claires, teintée sous les touches
    // de couleur, plus courte quand la touche est enfoncée.
    let teinte = if famille == Famille::Chiffre {
        p.ombre
    } else {
        couleurs
            .fond_bas
            .gamma_multiply(if p.sombre { 0.55 } else { 0.42 })
    };
    let portee = if enfoncee { (3, 1) } else { (8, 3) };
    theme::ombre_portee(painter, rect, rayon, teinte, portee);
    // Liseré plus foncé de 2 points au bas, pour suggérer le relief.
    painter.rect_filled(rect, rayon, theme::assombrir(couleurs.fond_bas, 0.32));
    let face = Rect::from_min_max(
        Pos2::new(rect.left(), rect.top() + decalage),
        Pos2::new(rect.right(), rect.bottom() - 2.0 + decalage),
    );
    theme::rect_degrade(
        painter,
        face,
        rayon,
        nuance(couleurs.fond),
        nuance(couleurs.fond_bas),
    );
    painter.rect_stroke(
        face,
        rayon,
        Stroke::new(1.0, couleurs.bord),
        StrokeKind::Inside,
    );
    // Reflet : un filet clair sous l'arête supérieure.
    let reflet = Rect::from_min_max(
        Pos2::new(face.left() + f32::from(rayon), face.top() + 1.5),
        Pos2::new(face.right() - f32::from(rayon), face.top() + 2.5),
    );
    painter.rect_filled(
        reflet,
        0,
        Color32::from_white_alpha(if p.sombre { 26 } else { 58 }),
    );
    let police = theme::police_grasse(taille);
    let lignes: Vec<&str> = libelle.split('\n').collect();
    let interligne = taille * 1.12;
    let hauteur = interligne * lignes.len() as f32;
    for (i, ligne) in lignes.iter().enumerate() {
        let y = face.center().y - hauteur / 2.0 + interligne * (i as f32 + 0.5);
        painter.text(
            Pos2::new(face.center().x, y),
            Align2::CENTER_CENTER,
            ligne,
            police.clone(),
            couleurs.texte,
        );
    }
}

/// Gère l'appui long et retourne l'action déclenchée par une touche.
fn interpreter(
    ui: &Ui,
    reponse: &Response,
    touche: Touche,
    etat: &mut EtatClavier,
    une_touche_enfoncee: &mut bool,
) -> Option<ActionClavier> {
    let maintenant = ui.input(|i| i.time);
    let mut action = None;
    if reponse.is_pointer_button_down_on() {
        *une_touche_enfoncee = true;
        match &mut etat.appui {
            Some(a) if a.touche == touche => {
                if !a.aide_ouverte && maintenant - a.debut >= APPUI_LONG {
                    a.aide_ouverte = true;
                    action = Some(ActionClavier::Aide(touche));
                }
            }
            _ => {
                etat.appui = Some(Appui {
                    touche,
                    debut: maintenant,
                    aide_ouverte: false,
                });
            }
        }
        // Pour détecter l'appui long sans autre événement.
        ui.request_repaint_after(std::time::Duration::from_millis(100));
    }
    if reponse.secondary_clicked() {
        action = Some(ActionClavier::Aide(touche));
    } else if reponse.clicked() {
        // Le relâchement qui suit un appui long ne déclenche pas la touche.
        let apres_appui_long = etat
            .appui
            .is_some_and(|a| a.touche == touche && a.aide_ouverte);
        if !apres_appui_long {
            action = Some(ActionClavier::Appui(touche));
        }
    }
    action
}

/// Dessine la grille de 8 × 5 touches dans `rect`.
pub fn dessiner(
    ui: &mut Ui,
    rect: Rect,
    p: &Palette,
    contexte: Contexte,
    etat: &mut EtatClavier,
) -> Vec<ActionClavier> {
    let mut actions = Vec::new();
    let painter = ui.painter_at(rect);
    let grille = rect.shrink(MARGE);
    let rangees = Touche::GRILLE.len() as f32;
    let colonnes = 5.0;
    let case = Vec2::new(
        (grille.width() - ESPACE * (colonnes - 1.0)) / colonnes,
        (grille.height() - ESPACE * (rangees - 1.0)) / rangees,
    );
    if case.x < 8.0 || case.y < 8.0 {
        return actions;
    }
    let hauteur_etiquette = (case.y * PART_ETIQUETTE).max(11.0);
    let bouton = Vec2::new(case.x, case.y - hauteur_etiquette);

    // Taille de police uniforme par groupe : les mots, puis les chiffres et symboles.
    let touches = || Touche::GRILLE.iter().flatten().copied();
    let libelles: Vec<String> = touches()
        .map(|t| textes::libelle(t, contexte.trigo, contexte.point_decimal))
        .collect();
    let disponible = bouton.x - 2.0 * MARGE_LIBELLE;
    let taille_mots = taille_ajustee(
        &painter,
        libelles
            .iter()
            .map(String::as_str)
            .filter(|l| !est_court(l)),
        (bouton.y * 0.36).clamp(9.0, 17.0),
        disponible,
        true,
    );
    let taille_courts = (bouton.y * 0.50).clamp(12.0, 26.0);
    let taille_secondaire = taille_ajustee(
        &painter,
        touches().map(|t| textes::libelle_secondaire(t, contexte.trigo)),
        (hauteur_etiquette * 0.72).clamp(8.0, 11.5),
        case.x + ESPACE - 2.0,
        true,
    );

    let mut une_touche_enfoncee = false;
    for (r, rangee) in Touche::GRILLE.iter().enumerate() {
        for (c, touche) in rangee.iter().copied().enumerate() {
            let origine = Pos2::new(
                grille.left() + (case.x + ESPACE) * c as f32,
                grille.top() + (case.y + ESPACE) * r as f32,
            );
            let zone_etiquette = Rect::from_min_size(origine, Vec2::new(case.x, hauteur_etiquette));
            let zone_bouton =
                Rect::from_min_size(Pos2::new(origine.x, origine.y + hauteur_etiquette), bouton);

            // Étiquette secondaire : un clic équivaut à Conv puis la touche.
            let secondaire = textes::libelle_secondaire(touche, contexte.trigo);
            if !secondaire.is_empty() {
                let reponse = ui.interact(
                    zone_etiquette,
                    Id::unique(("etiquette-secondaire", r, c)),
                    Sense::CLICK,
                );
                let nom = textes::nom_secondaire(touche, contexte.trigo);
                reponse.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, nom));
                let police = if contexte.conv_arme {
                    theme::police_grasse(taille_secondaire)
                } else {
                    theme::police_italique(taille_secondaire)
                };
                let couleur = if reponse.hovered() {
                    theme::melanger(p.etiquette_secondaire, p.texte, 0.35)
                } else {
                    p.etiquette_secondaire
                };
                painter.text(
                    Pos2::new(
                        zone_etiquette.center().x,
                        zone_etiquette.bottom() - hauteur_etiquette * 0.42,
                    ),
                    Align2::CENTER_CENTER,
                    secondaire,
                    police,
                    couleur,
                );
                if reponse.secondary_clicked() {
                    actions.push(ActionClavier::Aide(touche));
                } else if reponse.clicked() {
                    actions.push(ActionClavier::Secondaire(touche));
                }
            }

            // Bouton.
            let reponse = ui.interact(zone_bouton, Id::unique(("touche", r, c)), Sense::CLICK);
            let nom = textes::nom(touche, contexte.trigo);
            reponse.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, &nom));
            let libelle = &libelles[r * 5 + c];
            let taille = if est_court(libelle) {
                taille_courts
            } else {
                taille_mots
            };
            // Conv armé : la touche reste visuellement enfoncée.
            let enfoncee = reponse.is_pointer_button_down_on()
                || (touche == Touche::Conv && contexte.conv_arme);
            dessiner_bouton(
                &painter,
                zone_bouton,
                (libelle, taille),
                p,
                theme::famille(touche),
                (enfoncee, reponse.hovered()),
            );
            if let Some(action) = interpreter(ui, &reponse, touche, etat, &mut une_touche_enfoncee)
            {
                actions.push(action);
            }
            let raccourci = raccourcis::raccourci(touche);
            reponse.on_hover_text(textes::infobulle(
                touche,
                contexte.trigo,
                raccourci.as_deref(),
            ));
        }
    }
    if !une_touche_enfoncee {
        etat.appui = None;
    }
    actions
}

/// Nombre de rangées de la bande de fractions rapides pour une résolution.
pub fn rangees_de_fractions(resolution: u32) -> usize {
    if noyau::touches::fractions_rapides(resolution).len() > 8 {
        2
    } else {
        1
    }
}

/// Dessine la bande de fractions rapides (section 7.6) et retourne la touche pressée.
pub fn dessiner_fractions(ui: &mut Ui, rect: Rect, p: &Palette, resolution: u32) -> Option<Touche> {
    let fractions = noyau::touches::fractions_rapides(resolution);
    let painter = ui.painter_at(rect);
    let zone = rect.shrink2(Vec2::new(MARGE, 0.0));
    let par_rangee = if fractions.len() > 8 {
        8
    } else {
        fractions.len().max(1)
    };
    let nb_rangees = fractions.len().div_ceil(par_rangee).max(1);
    let hauteur = (zone.height() - 3.0 * (nb_rangees as f32 - 1.0)) / nb_rangees as f32;
    let largeur_case = (zone.width() - 4.0 * (par_rangee as f32 - 1.0)) / par_rangee as f32;
    let mut pressee = None;
    for (i, (num, den)) in fractions.iter().copied().enumerate() {
        let (r, c) = (i / par_rangee, i % par_rangee);
        let case = Rect::from_min_size(
            Pos2::new(
                zone.left() + (largeur_case + 4.0) * c as f32,
                zone.top() + (hauteur + 3.0) * r as f32,
            ),
            Vec2::new(largeur_case, hauteur),
        );
        let touche = Touche::FractionRapide { num, den };
        let reponse = ui.interact(
            case,
            Id::unique(("fraction-rapide", num, den)),
            Sense::CLICK,
        );
        let nom = textes::nom(touche, false);
        reponse.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, &nom));
        let taille = (hauteur * 0.46).clamp(9.0, 15.0).min(largeur_case * 0.36);
        dessiner_bouton(
            &painter,
            case,
            (&textes::libelle(touche, false, false), taille),
            p,
            Famille::Chiffre,
            (reponse.is_pointer_button_down_on(), reponse.hovered()),
        );
        if reponse.clicked() {
            pressee = Some(touche);
        }
    }
    pressee
}
