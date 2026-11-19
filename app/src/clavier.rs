//! Clavier : grille de 40 touches, étiquettes secondaires, bande de fractions
//! rapides, clics et appui long (section 16.4); touches d'un outil de métier
//! (section 23).

use eframe::egui::{
    self, Align2, Color32, Id, Painter, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2,
};
use noyau::moteur::{CHAMPS_D_OUTIL, TOUCHE_RESULTAT};
use noyau::{Touche, ToucheOutil};

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

/// Vrai si la touche figure au clavier : avec un outil de métier, les touches
/// de charpente et `Longueur`, `Largeur`, `Hauteur` n'y sont pas (section 23.3).
/// Leurs raccourcis clavier sont alors sans effet.
pub fn est_au_clavier(touche: Touche, outil_branche: bool) -> bool {
    !outil_branche
        || matches!(touche, Touche::Metres | Touche::Pourcent)
        || !Touche::GRILLE
            .iter()
            .take(3)
            .flatten()
            .any(|masquee| *masquee == touche)
}

/// Une case du clavier : une touche et la place qu'elle occupe, étiquette comprise.
struct Case {
    touche: Touche,
    rect: Rect,
}

/// Dispose les touches dans `grille`.
///
/// Sans outil : huit rangées de cinq touches. Avec un outil de métier
/// (`champs` : le nombre de ses champs), ses touches occupent une ou deux
/// rangées, étirées sur toute la largeur; viennent ensuite la rangée de la
/// touche `Résultat`, puis les cinq rangées du bas, inchangées (section 23).
fn disposer(grille: Rect, champs: Option<usize>) -> Vec<Case> {
    let colonne = (grille.width() - ESPACE * 4.0) / 5.0;
    // Une rangée : (touche, bord gauche, largeur).
    let standard = |rangee: &[Touche; 5]| -> Vec<(Touche, f32, f32)> {
        rangee
            .iter()
            .enumerate()
            .map(|(c, touche)| (*touche, (colonne + ESPACE) * c as f32, colonne))
            .collect()
    };
    let etiree = |rangs: std::ops::Range<usize>| -> Vec<(Touche, f32, f32)> {
        let nombre = rangs.len().max(1) as f32;
        let largeur = (grille.width() - ESPACE * (nombre - 1.0)) / nombre;
        rangs
            .enumerate()
            .map(|(i, rang)| {
                (
                    Touche::Outil(rang as u8),
                    (largeur + ESPACE) * i as f32,
                    largeur,
                )
            })
            .collect()
    };
    let rangees: Vec<Vec<(Touche, f32, f32)>> = match champs {
        None => Touche::GRILLE.iter().map(standard).collect(),
        Some(nombre) => {
            let nombre = nombre.min(CHAMPS_D_OUTIL);
            // Jusqu'à cinq champs : une rangée; au-delà, deux rangées équilibrées.
            let premiere = if nombre <= 5 {
                nombre
            } else {
                nombre.div_ceil(2)
            };
            let mut rangees = vec![etiree(0..premiere)];
            if premiere < nombre {
                rangees.push(etiree(premiere..nombre));
            }
            rangees.push(vec![
                (Touche::Metres, 0.0, colonne),
                (
                    Touche::Outil(TOUCHE_RESULTAT),
                    colonne + ESPACE,
                    3.0 * colonne + 2.0 * ESPACE,
                ),
                (Touche::Pourcent, 4.0 * (colonne + ESPACE), colonne),
            ]);
            rangees.extend(Touche::GRILLE.iter().skip(3).map(standard));
            rangees
        }
    };
    let nombre = rangees.len() as f32;
    let hauteur = (grille.height() - ESPACE * (nombre - 1.0)) / nombre;
    rangees
        .iter()
        .enumerate()
        .flat_map(|(r, rangee)| {
            rangee.iter().map(move |(touche, gauche, largeur)| Case {
                touche: *touche,
                rect: Rect::from_min_size(
                    Pos2::new(
                        grille.left() + gauche,
                        grille.top() + (hauteur + ESPACE) * r as f32,
                    ),
                    Vec2::new(*largeur, hauteur),
                ),
            })
        })
        .collect()
}

/// Le texte, raccourci au besoin par des points de suspension, et la taille de
/// police pour qu'il tienne dans `place` : la taille diminue d'abord, jusqu'à
/// `plancher`; le texte est coupé ensuite.
fn ajuster_ou_couper(
    painter: &Painter,
    texte: &str,
    (taille, plancher): (f32, f32),
    place: f32,
) -> (String, f32) {
    let large = largeur(painter, texte, theme::police_grasse(taille));
    if large <= place {
        return (texte.to_owned(), taille);
    }
    let reduite = taille * place / large;
    if reduite >= plancher {
        return (texte.to_owned(), reduite);
    }
    let mut coupe: Vec<char> = texte.chars().collect();
    while coupe.len() > 1 {
        coupe.pop();
        let essai: String = coupe.iter().collect::<String>().trim_end().to_owned() + "…";
        if largeur(painter, &essai, theme::police_grasse(plancher)) <= place {
            return (essai, plancher);
        }
    }
    (texte.to_owned(), plancher)
}

/// Dessine le clavier dans `rect`. `outil` : les champs de l'outil de métier
/// branché sur le clavier, s'il y en a un (section 23).
pub fn dessiner(
    ui: &mut Ui,
    rect: Rect,
    p: &Palette,
    contexte: Contexte,
    outil: Option<&[ToucheOutil]>,
    etat: &mut EtatClavier,
) -> Vec<ActionClavier> {
    let mut actions = Vec::new();
    let painter = ui.painter_at(rect);
    let grille = rect.shrink(MARGE);
    let cases = disposer(grille, outil.map(<[ToucheOutil]>::len));
    let champ_de = |touche: Touche| match (touche, outil) {
        (Touche::Outil(rang), Some(champs)) if rang != TOUCHE_RESULTAT => {
            champs.get(usize::from(rang))
        }
        _ => None,
    };
    // La case d'une touche ordinaire : toutes ont la même taille.
    let colonne = (grille.width() - ESPACE * 4.0) / 5.0;
    let hauteur_case = cases.first().map_or(0.0, |case| case.rect.height());
    if colonne < 8.0 || hauteur_case < 8.0 {
        return actions;
    }
    let hauteur_etiquette = (hauteur_case * PART_ETIQUETTE).max(11.0);
    let hauteur_bouton = hauteur_case - hauteur_etiquette;

    // Taille de police uniforme par groupe : les mots, puis les chiffres et symboles.
    let ordinaires = || {
        cases
            .iter()
            .map(|case| case.touche)
            .filter(|touche| champ_de(*touche).is_none())
    };
    let libelle_de =
        |touche: Touche| textes::libelle(touche, contexte.trigo, contexte.point_decimal);
    let plafond_des_mots = (hauteur_bouton * 0.36).clamp(9.0, 17.0);
    let libelles: Vec<String> = ordinaires().map(libelle_de).collect();
    let taille_mots = taille_ajustee(
        &painter,
        libelles
            .iter()
            .map(String::as_str)
            .filter(|l| !est_court(l)),
        plafond_des_mots,
        colonne - 2.0 * MARGE_LIBELLE,
        true,
    );
    let taille_courts = (hauteur_bouton * 0.50).clamp(12.0, 26.0);
    let taille_secondaire = taille_ajustee(
        &painter,
        ordinaires().map(|t| textes::libelle_secondaire(t, contexte.trigo)),
        (hauteur_etiquette * 0.72).clamp(8.0, 11.5),
        colonne + ESPACE - 2.0,
        true,
    );
    // Les touches de l'outil : le libellé complet s'il tient sur la touche,
    // sinon le libellé court; une même taille pour toutes.
    let libelles_d_outil: Vec<(Touche, &str)> = cases
        .iter()
        .filter_map(|case| {
            let champ = champ_de(case.touche)?;
            let place = case.rect.width() - 2.0 * MARGE_LIBELLE;
            let complet = largeur(&painter, champ.nom, theme::police_grasse(plafond_des_mots));
            Some((
                case.touche,
                if complet <= place {
                    champ.nom
                } else {
                    champ.libelle
                },
            ))
        })
        .collect();
    let taille_outil = cases
        .iter()
        .filter_map(|case| {
            let (_, libelle) = libelles_d_outil.iter().find(|(t, _)| *t == case.touche)?;
            Some(taille_ajustee(
                &painter,
                std::iter::once(*libelle),
                plafond_des_mots,
                case.rect.width() - 2.0 * MARGE_LIBELLE,
                true,
            ))
        })
        .fold(plafond_des_mots, f32::min);
    let taille_valeur = (hauteur_etiquette * 0.78).clamp(8.5, 12.5);

    let mut une_touche_enfoncee = false;
    for case in &cases {
        let touche = case.touche;
        let zone_etiquette = Rect::from_min_size(
            case.rect.min,
            Vec2::new(case.rect.width(), hauteur_etiquette),
        );
        let zone_bouton = Rect::from_min_max(
            Pos2::new(case.rect.left(), case.rect.top() + hauteur_etiquette),
            case.rect.max,
        );
        let ligne_etiquette = Pos2::new(
            zone_etiquette.center().x,
            zone_etiquette.bottom() - hauteur_etiquette * 0.42,
        );

        // Touche d'un champ de l'outil : son contenu s'écrit au-dessus d'elle.
        if let Some(champ) = champ_de(touche) {
            let place = case.rect.width() + ESPACE - 2.0;
            if champ.valeur.is_empty() {
                // Champ vide : l'unité d'un nombre tapé sans unité, entre parenthèses.
                if !champ.unite.is_empty() {
                    painter.text(
                        ligne_etiquette,
                        Align2::CENTER_CENTER,
                        format!("({})", champ.unite),
                        theme::police(taille_valeur),
                        p.texte_attenue,
                    );
                }
            } else {
                let (texte, taille) =
                    ajuster_ou_couper(&painter, &champ.valeur, (taille_valeur, 8.0), place);
                painter.text(
                    ligne_etiquette,
                    Align2::CENTER_CENTER,
                    texte,
                    theme::police_grasse(taille),
                    p.titre,
                );
            }
            let reponse = ui.interact(zone_bouton, Id::unique(("touche", touche)), Sense::CLICK);
            reponse.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, champ.nom));
            let libelle = libelles_d_outil
                .iter()
                .find(|(t, _)| *t == touche)
                .map_or(champ.libelle, |(_, libelle)| libelle);
            let famille = if champ.choix {
                Famille::Construction
            } else {
                Famille::Fonction
            };
            dessiner_bouton(
                &painter,
                zone_bouton,
                (libelle, taille_outil),
                p,
                famille,
                (reponse.is_pointer_button_down_on(), reponse.hovered()),
            );
            if let Some(action) = interpreter(ui, &reponse, touche, etat, &mut une_touche_enfoncee)
            {
                actions.push(action);
            }
            reponse.on_hover_text(textes::infobulle_d_outil(
                champ.nom,
                &champ.valeur,
                champ.choix,
            ));
            continue;
        }

        // Étiquette secondaire : un clic équivaut à Conv puis la touche.
        let secondaire = textes::libelle_secondaire(touche, contexte.trigo);
        if !secondaire.is_empty() {
            let reponse = ui.interact(
                zone_etiquette,
                Id::unique(("etiquette-secondaire", touche)),
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
                ligne_etiquette,
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
        let reponse = ui.interact(zone_bouton, Id::unique(("touche", touche)), Sense::CLICK);
        let nom = textes::nom(touche, contexte.trigo);
        reponse.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, &nom));
        let libelle = libelle_de(touche);
        let taille = if est_court(&libelle) {
            taille_courts
        } else {
            taille_mots
        };
        // Conv armé : la touche reste visuellement enfoncée.
        let enfoncee =
            reponse.is_pointer_button_down_on() || (touche == Touche::Conv && contexte.conv_arme);
        dessiner_bouton(
            &painter,
            zone_bouton,
            (&libelle, taille),
            p,
            theme::famille(touche),
            (enfoncee, reponse.hovered()),
        );
        if let Some(action) = interpreter(ui, &reponse, touche, etat, &mut une_touche_enfoncee) {
            actions.push(action);
        }
        let raccourci = raccourcis::raccourci(touche);
        reponse.on_hover_text(textes::infobulle(
            touche,
            contexte.trigo,
            raccourci.as_deref(),
        ));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn touches_masquees_par_un_outil() {
        // Sans outil, toutes les touches de la grille sont au clavier.
        for touche in Touche::GRILLE.iter().flatten() {
            assert!(est_au_clavier(*touche, false));
        }
        // Avec un outil : ni charpente ni Longueur, Largeur, Hauteur.
        for touche in [
            Touche::Pente,
            Touche::Escalier,
            Touche::Longueur,
            Touche::Hauteur,
        ] {
            assert!(!est_au_clavier(touche, true));
        }
        for touche in [
            Touche::Metres,
            Touche::Pourcent,
            Touche::Pieds,
            Touche::Chiffre(7),
            Touche::Egal,
            Touche::Outil(0),
        ] {
            assert!(est_au_clavier(touche, true));
        }
    }

    #[test]
    fn disposition_du_clavier_d_un_outil() {
        let grille = Rect::from_min_size(Pos2::ZERO, Vec2::new(404.0, 600.0));
        // Clavier de base : quarante touches.
        assert_eq!(disposer(grille, None).len(), 40);
        // Quatre champs : une rangée de champs, la rangée de Résultat, cinq rangées.
        let cases = disposer(grille, Some(4));
        assert_eq!(cases.len(), 4 + 3 + 25);
        assert_eq!(cases[0].touche, Touche::Outil(0));
        assert_eq!(cases[4].touche, Touche::Metres);
        assert_eq!(cases[5].touche, Touche::Outil(TOUCHE_RESULTAT));
        assert_eq!(cases[6].touche, Touche::Pourcent);
        // Les champs se partagent toute la largeur; Résultat couvre trois colonnes.
        assert!((cases[3].rect.right() - grille.right()).abs() < 0.01);
        assert!(
            (cases[5].rect.width() - (3.0 * cases[4].rect.width() + 2.0 * ESPACE)).abs() < 0.01
        );
        // Neuf champs : deux rangées, cinq puis quatre.
        let cases = disposer(grille, Some(9));
        assert_eq!(cases.len(), 9 + 3 + 25);
        assert!((cases[4].rect.top() - cases[0].rect.top()).abs() < 0.01);
        assert!(cases[5].rect.top() > cases[4].rect.top());
        // La dernière touche touche le bas de la grille, quel que soit le nombre de rangées.
        for champs in [None, Some(2), Some(6), Some(10)] {
            let cases = disposer(grille, champs);
            let bas = cases.iter().map(|c| c.rect.bottom()).fold(0.0f32, f32::max);
            assert!((bas - grille.bottom()).abs() < 0.01);
        }
    }
}
