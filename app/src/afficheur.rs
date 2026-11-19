//! Dessin de l'afficheur : indicateurs, étiquette, valeur à fractions empilées
//! (section 16.2).

use std::sync::Arc;

use eframe::egui::{
    self, Align2, Color32, Galley, Painter, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, Vec2,
};
use noyau::{Ecran, Prefixe, Segment, TexteValeur};

use crate::textes;
use crate::theme::{self, Palette};

/// Ce que l'utilisateur a demandé en agissant sur l'afficheur.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionAfficheur {
    RetourArriere,
    Copier,
}

/// Taille relative des unités et des fractions par rapport aux chiffres.
const PART_UNITE: f32 = 0.55;
const PART_FRACTION: f32 = 0.50;
/// Espace entre deux morceaux, en proportion de la taille des chiffres.
const PART_ESPACE: f32 = 0.20;
/// Plus petite taille de chiffres admise quand la valeur est trop large.
const TAILLE_MINIMALE: f32 = 13.0;
/// Rayon des coins de la carte de l'afficheur.
const RAYON_ECRAN: u8 = 12;

/// Morceau mesuré, prêt à être dessiné.
enum Morceau {
    Texte(Arc<Galley>),
    Unite(Arc<Galley>),
    Fraction { num: Arc<Galley>, den: Arc<Galley> },
}

/// Repères verticaux d'une police : du haut du galley au sommet des chiffres et
/// à la ligne de base. Mesurés sur le chiffre « 0 », qui repose sur la ligne.
#[derive(Clone, Copy)]
struct Reperes {
    sommet: f32,
    base: f32,
}

fn reperes(painter: &Painter, taille: f32) -> Reperes {
    let zero = painter.layout_no_wrap("0".to_owned(), theme::police_grasse(taille), Color32::WHITE);
    let encre = zero.mesh_bounds;
    if encre.height() > 1.0 {
        Reperes {
            sommet: encre.min.y - zero.rect.min.y,
            base: encre.max.y - zero.rect.min.y,
        }
    } else {
        // Repli : proportions courantes d'une police latine.
        Reperes {
            sommet: zero.size().y * 0.25,
            base: zero.size().y * 0.80,
        }
    }
}

struct Composition {
    morceaux: Vec<(Morceau, f32)>,
    largeur: f32,
}

/// Mesure une valeur à une taille de chiffres donnée.
fn composer(painter: &Painter, valeur: &TexteValeur, taille: f32, encre: Color32) -> Composition {
    let espace = taille * PART_ESPACE;
    let mut morceaux = Vec::with_capacity(valeur.0.len());
    let mut largeur = 0.0;
    let mut colle = true;
    for segment in &valeur.0 {
        let (morceau, l, colle_ensuite) = match segment {
            Segment::Texte(t) => {
                let g = painter.layout_no_wrap(t.clone(), theme::police_grasse(taille), encre);
                let l = g.size().x;
                // Un signe seul se colle à ce qui suit.
                let signe_seul = t.chars().count() == 1 && t.starts_with('\u{2212}');
                (Morceau::Texte(g), l, signe_seul)
            }
            Segment::Unite(u) => {
                let g = painter.layout_no_wrap(
                    u.clone(),
                    theme::police_grasse(taille * PART_UNITE),
                    encre,
                );
                let l = g.size().x;
                (Morceau::Unite(g), l, false)
            }
            Segment::Fraction { num, den } => {
                let police = theme::police_grasse(taille * PART_FRACTION);
                let n = painter.layout_no_wrap(num.to_string(), police.clone(), encre);
                let d = painter.layout_no_wrap(den.to_string(), police, encre);
                let l = n.size().x.max(d.size().x) + taille * 0.10;
                (Morceau::Fraction { num: n, den: d }, l, false)
            }
        };
        let avant = if colle { 0.0 } else { espace };
        largeur += avant + l;
        morceaux.push((morceau, avant));
        colle = colle_ensuite;
    }
    Composition { morceaux, largeur }
}

/// Dessine une valeur alignée à droite dans `zone`, sa ligne de base au bas de
/// la zone. La taille diminue jusqu'à ce que la valeur tienne.
pub fn dessiner_valeur(
    painter: &Painter,
    zone: Rect,
    valeur: &TexteValeur,
    taille_maximale: f32,
    encre: Color32,
) {
    let mut taille = taille_maximale;
    let mut composition = composer(painter, valeur, taille, encre);
    for _ in 0..3 {
        if composition.largeur <= zone.width() || taille <= TAILLE_MINIMALE {
            break;
        }
        taille = (taille * zone.width() / composition.largeur * 0.98).max(TAILLE_MINIMALE);
        composition = composer(painter, valeur, taille, encre);
    }
    let grands = reperes(painter, taille);
    let unites = reperes(painter, taille * PART_UNITE);
    let petits = reperes(painter, taille * PART_FRACTION);
    // Ligne de base commune.
    let base = zone.bottom() - taille * 0.12;
    let hauteur_chiffres = grands.base - grands.sommet;
    let mut x = zone.right() - composition.largeur;
    for (morceau, avant) in &composition.morceaux {
        x += avant;
        match morceau {
            Morceau::Texte(g) => {
                painter.galley(Pos2::new(x, base - grands.base), Arc::clone(g), encre);
                x += g.size().x;
            }
            Morceau::Unite(g) => {
                painter.galley(Pos2::new(x, base - unites.base), Arc::clone(g), encre);
                x += g.size().x;
            }
            Morceau::Fraction { num, den } => {
                // Numérateur au-dessus de la barre, dénominateur en dessous; la
                // barre passe à mi-hauteur des chiffres.
                let largeur = num.size().x.max(den.size().x) + taille * 0.10;
                let barre_y = base - hauteur_chiffres * 0.5;
                let jeu = (taille * 0.045).max(1.0);
                let num_x = x + (largeur - num.size().x) / 2.0;
                let den_x = x + (largeur - den.size().x) / 2.0;
                painter.galley(
                    Pos2::new(num_x, barre_y - jeu - petits.base),
                    Arc::clone(num),
                    encre,
                );
                painter.galley(
                    Pos2::new(den_x, barre_y + jeu - petits.sommet),
                    Arc::clone(den),
                    encre,
                );
                painter.line_segment(
                    [Pos2::new(x, barre_y), Pos2::new(x + largeur, barre_y)],
                    Stroke::new((taille * 0.045).max(1.0), encre),
                );
                x += largeur;
            }
        }
    }
}

/// Texte sur une ligne, réduit au besoin pour tenir dans la largeur.
fn texte_ajuste(
    painter: &Painter,
    pos: Pos2,
    ancre: Align2,
    texte: &str,
    police: egui::FontId,
    couleur: Color32,
    largeur: f32,
) {
    let g = painter.layout_no_wrap(texte.to_owned(), police.clone(), couleur);
    let police = if g.size().x > largeur && g.size().x > 0.0 {
        egui::FontId::new((police.size * largeur / g.size().x).max(8.0), police.family)
    } else {
        police
    };
    painter.text(pos, ancre, texte, police, couleur);
}

/// Pictogramme « retour arrière », dessiné en traits.
fn dessiner_retour_arriere(painter: &Painter, rect: Rect, couleur: Color32) {
    let c = rect.center();
    let (l, h) = (rect.width() * 0.52, rect.height() * 0.50);
    let gauche = c.x - l / 2.0;
    let droite = c.x + l / 2.0;
    let pointe = h * 0.5;
    let contour = vec![
        Pos2::new(gauche, c.y),
        Pos2::new(gauche + pointe, c.y - h / 2.0),
        Pos2::new(droite, c.y - h / 2.0),
        Pos2::new(droite, c.y + h / 2.0),
        Pos2::new(gauche + pointe, c.y + h / 2.0),
        Pos2::new(gauche, c.y),
    ];
    let trait_ = Stroke::new(1.6, couleur);
    painter.line(contour, trait_);
    let cx = (gauche + pointe + droite) / 2.0;
    let d = h * 0.20;
    painter.line_segment(
        [Pos2::new(cx - d, c.y - d), Pos2::new(cx + d, c.y + d)],
        trait_,
    );
    painter.line_segment(
        [Pos2::new(cx - d, c.y + d), Pos2::new(cx + d, c.y - d)],
        trait_,
    );
}

/// Pictogramme d'avertissement : un triangle et son point d'exclamation.
pub fn dessiner_avertissement(painter: &Painter, centre: Pos2, taille: f32, couleur: Color32) {
    let h = taille * 0.9;
    let points = vec![
        Pos2::new(centre.x, centre.y - h / 2.0),
        Pos2::new(centre.x + taille / 2.0, centre.y + h / 2.0),
        Pos2::new(centre.x - taille / 2.0, centre.y + h / 2.0),
        Pos2::new(centre.x, centre.y - h / 2.0),
    ];
    painter.line(points, Stroke::new(1.5, couleur));
    painter.line_segment(
        [
            Pos2::new(centre.x, centre.y - h * 0.12),
            Pos2::new(centre.x, centre.y + h * 0.18),
        ],
        Stroke::new(1.5, couleur),
    );
    painter.circle_filled(Pos2::new(centre.x, centre.y + h * 0.33), 0.9, couleur);
}

/// Dessine l'afficheur dans `rect` et retourne l'action demandée, s'il y en a une.
pub fn dessiner(
    ui: &mut Ui,
    rect: Rect,
    p: &Palette,
    ecran: &Ecran,
    copie_visible: bool,
) -> Option<ActionAfficheur> {
    let mut action = None;
    let cadre = rect.shrink2(Vec2::new(10.0, 8.0));
    // Carte de l'afficheur : halo, dégradé, liseré et filet d'accent. Le halo
    // déborde de la zone : il est peint sans découpe.
    theme::ombre_portee(ui.painter(), cadre, RAYON_ECRAN, p.ecran_halo, (18, 6));
    let painter = ui.painter_at(rect);
    theme::rect_degrade(&painter, cadre, RAYON_ECRAN, p.ecran_fond, p.ecran_fond_bas);
    painter.rect_stroke(
        cadre,
        RAYON_ECRAN,
        Stroke::new(1.0, p.ecran_bord),
        StrokeKind::Inside,
    );
    let accent = Rect::from_min_max(
        Pos2::new(cadre.left() + 16.0, cadre.top() + 1.0),
        Pos2::new(cadre.right() - 16.0, cadre.top() + 3.0),
    );
    painter.rect_filled(accent, 2, p.primaire);
    let interieur = cadre.shrink2(Vec2::new(14.0, 10.0));

    // Double-clic sur la valeur : copie dans le presse-papiers.
    let zone_valeur = Rect::from_min_max(
        Pos2::new(interieur.left() + 44.0, interieur.top() + 40.0),
        interieur.max,
    );
    let reponse_valeur = ui
        .interact(
            zone_valeur,
            egui::Id::unique("afficheur-valeur"),
            Sense::CLICK,
        )
        .on_hover_text(textes::INFOBULLE_VALEUR);
    if reponse_valeur.double_clicked() {
        action = Some(ActionAfficheur::Copier);
    }

    // Ligne 1 : indicateurs.
    let ligne1 = interieur.top() + 8.0;
    let police_indicateur = theme::police_grasse(11.5);
    let mut x = interieur.left();
    let prefixe = match ecran.prefixe {
        Prefixe::Conv => Some(textes::INDICATEUR_CONV),
        Prefixe::Stocker => Some(textes::INDICATEUR_STOCKER),
        Prefixe::Rappel => Some(textes::INDICATEUR_RAPPEL),
        Prefixe::Aucun => None,
    };
    // Chaque indicateur est une pastille; `vers_la_droite` dit de quel côté
    // elle s'ancre. Retourne le bord libre de la pastille.
    let pastille =
        |texte: &str, fond: Color32, encre: Color32, ancre: f32, vers_la_droite: bool| {
            let g = painter.layout_no_wrap(texte.to_owned(), police_indicateur.clone(), encre);
            let largeur = g.size().x + 14.0;
            let gauche = if vers_la_droite {
                ancre
            } else {
                ancre - largeur
            };
            let zone =
                Rect::from_min_size(Pos2::new(gauche, ligne1 - 9.5), Vec2::new(largeur, 19.0));
            painter.rect_filled(zone, 9, fond);
            painter.galley(
                Pos2::new(zone.left() + 7.0, zone.center().y - g.size().y / 2.0),
                g,
                encre,
            );
            if vers_la_droite {
                zone.right() + 6.0
            } else {
                zone.left() - 6.0
            }
        };
    let fond_discret = theme::melanger(p.ecran_fond_bas, p.primaire, 0.16);
    if let Some(texte) = prefixe {
        x = pastille(texte, p.touche_conv.fond, p.touche_conv.texte, x, true);
    }
    if ecran.memoire_active {
        x = pastille(
            textes::INDICATEUR_MEMOIRE,
            p.selection,
            p.selection_texte,
            x,
            true,
        );
    }
    if let Some(op) = ecran.operateur {
        pastille(&op.to_string(), fond_discret, p.ecran_etiquette, x, true);
    }
    // À droite : MÉM., avertissement, confirmation de copie.
    let mut droite = interieur.right();
    if ecran.memorise {
        droite = pastille(
            textes::INDICATEUR_MEMORISE,
            fond_discret,
            p.ecran_etiquette,
            droite,
            false,
        );
    }
    if let Some(avertissement) = &ecran.avertissement {
        let centre = Pos2::new(droite - 8.0, ligne1);
        dessiner_avertissement(&painter, centre, 15.0, p.avertissement);
        // Le texte complet apparaît aussi en infobulle.
        let zone = Rect::from_center_size(centre, Vec2::splat(22.0));
        ui.interact(
            zone,
            egui::Id::unique("afficheur-avertissement"),
            Sense::hover(),
        )
        .on_hover_text(avertissement);
        droite -= 26.0;
    }
    if copie_visible {
        painter.text(
            Pos2::new(droite, ligne1),
            Align2::RIGHT_CENTER,
            textes::COPIE,
            theme::police(11.5),
            p.ecran_secondaire,
        );
    }

    // Ligne 2 : étiquette (et note), ou avertissement.
    let ligne2 = interieur.top() + 31.0;
    let largeur = interieur.width();
    if let Some(etiquette) = &ecran.etiquette {
        let texte = match &ecran.note {
            Some(note) => format!("{etiquette}  ·  {note}"),
            None => etiquette.clone(),
        };
        texte_ajuste(
            &painter,
            Pos2::new(interieur.left(), ligne2),
            Align2::LEFT_CENTER,
            &texte,
            theme::police_grasse(15.0),
            p.ecran_etiquette,
            largeur,
        );
    }
    if let Some(avertissement) = &ecran.avertissement {
        texte_ajuste(
            &painter,
            Pos2::new(interieur.left(), ligne2 + 17.0),
            Align2::LEFT_CENTER,
            avertissement,
            theme::police(11.5),
            p.avertissement,
            largeur,
        );
    }

    // Ligne 3 : la valeur, ou le message.
    let bas = Rect::from_min_max(
        Pos2::new(interieur.left() + 44.0, ligne2 + 12.0),
        interieur.max,
    );
    match &ecran.message {
        Some(message) => {
            let couleur = if ecran.erreur {
                p.erreur
            } else {
                p.ecran_encre
            };
            texte_ajuste(
                &painter,
                Pos2::new(interieur.right(), bas.bottom() - 6.0),
                Align2::RIGHT_BOTTOM,
                message,
                theme::police_grasse(23.0),
                couleur,
                interieur.width(),
            );
        }
        None => {
            let taille = (bas.height() * 0.62).clamp(22.0, 54.0);
            dessiner_valeur(&painter, bas, &ecran.valeur, taille, p.ecran_encre);
        }
    }

    // Coin inférieur gauche : la touche de retour arrière.
    let taille_bouton = Vec2::new(40.0, 30.0);
    let bouton = Rect::from_min_size(
        Pos2::new(interieur.left(), interieur.bottom() - taille_bouton.y),
        taille_bouton,
    );
    let reponse = ui
        .interact(
            bouton,
            egui::Id::unique("afficheur-retour-arriere"),
            Sense::CLICK,
        )
        .on_hover_text(textes::INFOBULLE_RETOUR_ARRIERE);
    reponse.widget_info(|| {
        egui::WidgetInfo::labeled(
            egui::Role::Button,
            true,
            textes::nom(noyau::Touche::RetourArriere, false),
        )
    });
    // Même traitement que les touches claires : dégradé discret et liseré.
    let part = if reponse.is_pointer_button_down_on() {
        0.22
    } else if reponse.hovered() {
        0.12
    } else {
        0.0
    };
    let nuance = |c: Color32| theme::melanger(c, p.primaire, part);
    theme::rect_degrade(
        &painter,
        bouton,
        6,
        nuance(p.touche_chiffre.fond),
        nuance(p.touche_chiffre.fond_bas),
    );
    painter.rect_stroke(
        bouton,
        6,
        Stroke::new(1.0, p.touche_chiffre.bord),
        StrokeKind::Inside,
    );
    dessiner_retour_arriere(&painter, bouton, p.touche_chiffre.texte);
    if reponse.clicked() {
        action = Some(ActionAfficheur::RetourArriere);
    }
    action
}
