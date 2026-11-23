//! Écran « Liste de coupe » (section 22.19) : les morceaux à couper, les
//! longueurs offertes en stock et le trait de scie; le plan de coupe, calculé
//! par le noyau (`noyau::coupe`), dessiné pièce par pièce.

use eframe::egui::{self, Pos2, Rect, RichText, Sense, Stroke, Ui, Vec2};
use noyau::Preferences;
use noyau::analyse::analyser_longueur;
use noyau::coupe::{self, MAX_MORCEAUX, PieceDeStock, PlanDeCoupe};
use noyau::langue::Langue;
use noyau::metiers::{Article, Quantite};
use noyau::unites::Format;

use super::metiers::{Action, Position};
use super::{bouton_principal, cadre_de_saisie, carte, corps, en_tete, section};
use crate::textes;
use crate::theme::{self, Palette};

/// La clé de la liste de coupe parmi les saisies des métiers (`metiers.json`).
pub const IDENTIFIANT: &str = "liste-de-coupe";

/// Les rangs des saisies : les longueurs offertes, le trait de scie, le
/// matériau, puis, deux par deux, la quantité et la longueur de chaque morceau.
const OFFERTES: usize = 0;
const TRAIT: usize = 1;
const MATERIAU: usize = 2;
const PREMIER_MORCEAU: usize = 3;

/// Le nombre de lignes de morceaux, au plus.
const MAX_LIGNES: usize = 40;

/// Les saisies de départ : 8, 10, 12 et 16 pi, un trait de 1/8 po, un morceau.
pub fn saisies_de_depart() -> Vec<String> {
    vec![
        "8; 10; 12; 16".to_string(),
        "1/8".to_string(),
        String::new(),
        "1".to_string(),
        String::new(),
    ]
}

/// Récrit les longueurs de la liste dans l'autre langue (section 24); les
/// quantités et le matériau ne changent pas.
pub fn transposer(saisies: &mut [String], de: Langue, vers: Langue, prefs: &Preferences) {
    for (rang, saisie) in saisies.iter_mut().enumerate() {
        let longueur = rang == OFFERTES
            || rang == TRAIT
            || (rang >= PREMIER_MORCEAU && (rang - PREMIER_MORCEAU) % 2 == 1);
        if longueur {
            *saisie = coupe::transposer_les_longueurs(saisie, de, vers, prefs);
        }
    }
}

/// Ce que les saisies donnent, une fois lues.
struct Donnees {
    /// (longueur en pouces, quantité).
    morceaux: Vec<(f64, usize)>,
    offertes: Vec<f64>,
    trait_de_scie: f64,
    /// Le format de la première longueur tapée : il donne le ton des résultats.
    modele: Option<Format>,
}

/// Lit les saisies; sinon, ce qui ne va pas.
fn lire(saisies: &[String]) -> Result<Donnees, String> {
    let offertes = coupe::lire_les_longueurs(&saisies[OFFERTES], Format::PiPo)
        .map_err(|rang| textes::coupe_offerte_illisible(rang + 1))?;
    if offertes.is_empty() {
        return Err(textes::COUPE_AUCUNE_OFFERTE.traduit().to_string());
    }
    let trait_de_scie = match saisies[TRAIT].trim() {
        "" => 0.0,
        texte => analyser_longueur(texte, Format::PoFrac)
            .ok()
            .map(|valeur| valeur.base)
            .filter(|pouces| pouces.is_finite() && *pouces >= 0.0)
            .ok_or_else(|| textes::COUPE_TRAIT_ILLISIBLE.traduit().to_string())?,
    };
    let mut morceaux = Vec::new();
    let mut modele = None;
    let mut total = 0;
    for (ligne, paire) in saisies[PREMIER_MORCEAU..].chunks(2).enumerate() {
        let (quantite, longueur) = (paire[0].trim(), paire.get(1).map_or("", |l| l.trim()));
        if longueur.is_empty() {
            continue;
        }
        let quantite = if quantite.is_empty() {
            1
        } else {
            quantite
                .parse::<usize>()
                .ok()
                .filter(|q| (1..=999).contains(q))
                .ok_or_else(|| textes::coupe_quantite_illisible(ligne + 1))?
        };
        let valeur = analyser_longueur(longueur, Format::PiPo)
            .ok()
            .filter(|valeur| valeur.base.is_finite() && valeur.base > 0.0)
            .ok_or_else(|| textes::coupe_longueur_illisible(ligne + 1))?;
        modele = modele.or(Some(valeur.format));
        total += quantite;
        morceaux.push((valeur.base, quantite));
    }
    if total > MAX_MORCEAUX {
        return Err(textes::coupe_trop_de_morceaux(MAX_MORCEAUX));
    }
    Ok(Donnees {
        morceaux,
        offertes,
        trait_de_scie,
        modele,
    })
}

/// Le plan de la dernière lecture, gardé d'une image à l'autre : il ne se
/// recalcule que si les saisies changent.
#[derive(Clone, Default)]
struct Cache {
    cle: Vec<String>,
    plan: PlanDeCoupe,
}

fn plan_en_cache(ui: &Ui, saisies: &[String], donnees: &Donnees) -> PlanDeCoupe {
    let id = egui::Id::unique("plan-de-coupe");
    ui.ctx().memory_mut(|memoire| {
        let cache = memoire.data.get_temp_mut_or_default::<Cache>(id);
        if cache.cle != saisies {
            cache.cle = saisies.to_vec();
            cache.plan =
                coupe::planifier(&donnees.morceaux, &donnees.offertes, donnees.trait_de_scie);
        }
        cache.plan.clone()
    })
}

/// Un champ d'une ligne, avec son indice.
fn champ(ui: &mut Ui, p: &Palette, texte: &mut String, largeur: f32, indice: &str) -> bool {
    let reponse = ui.add(
        egui::TextEdit::singleline(texte)
            .desired_width(largeur)
            .char_limit(60)
            .hint_text(indice),
    );
    cadre_de_saisie(ui, p, &reponse);
    reponse.changed()
}

/// Affiche la liste de coupe. `saisies` : celles de `IDENTIFIANT`.
pub fn afficher(
    ui: &mut Ui,
    p: &Palette,
    position: &mut Position,
    saisies: &mut Vec<String>,
    prefs: &Preferences,
) -> Option<Action> {
    let mut action = None;
    if en_tete(
        ui,
        p,
        textes::LISTE_DE_COUPE.traduit(),
        textes::RETOUR.traduit(),
    ) {
        position.liste_de_coupe = false;
    }
    if saisies.len() < PREMIER_MORCEAU + 2 || !(saisies.len() - PREMIER_MORCEAU).is_multiple_of(2) {
        *saisies = saisies_de_depart();
    }
    let mut modifie = false;
    corps(ui, "liste-de-coupe", |ui| {
        ui.add_space(6.0);
        ui.label(RichText::new(textes::LISTE_DE_COUPE_RESUME.traduit()).color(p.texte_attenue));

        // Les morceaux à couper : quantité × longueur.
        section(ui, p, textes::COUPE_MORCEAUX.traduit());
        let lignes = (saisies.len() - PREMIER_MORCEAU) / 2;
        let mut a_retirer = None;
        carte(ui, p, |ui| {
            for ligne in 0..lignes {
                let rang = PREMIER_MORCEAU + 2 * ligne;
                ui.horizontal(|ui| {
                    modifie |= champ(ui, p, &mut saisies[rang], 40.0, "1");
                    ui.label("×");
                    let largeur = (ui.available_width() - 86.0).max(90.0);
                    modifie |= champ(
                        ui,
                        p,
                        &mut saisies[rang + 1],
                        largeur,
                        textes::COUPE_INDICE_LONGUEUR.traduit(),
                    );
                    if lignes > 1 && ui.small_button(textes::DESSIN_ENLEVER.traduit()).clicked() {
                        a_retirer = Some(rang);
                    }
                });
            }
            if lignes < MAX_LIGNES && ui.button(textes::COUPE_PLUS_MORCEAU.traduit()).clicked() {
                saisies.push("1".to_string());
                saisies.push(String::new());
                modifie = true;
            }
        });
        if let Some(rang) = a_retirer {
            saisies.drain(rang..rang + 2);
            modifie = true;
        }

        // Le stock : les longueurs offertes, le trait de scie, le matériau.
        section(ui, p, textes::COUPE_STOCK.traduit());
        carte(ui, p, |ui| {
            for (rang, libelle, indice, aide) in [
                (
                    OFFERTES,
                    textes::COUPE_OFFERTES,
                    textes::COUPE_OFFERTES_INDICE,
                    Some(textes::COUPE_OFFERTES_AIDE),
                ),
                (TRAIT, textes::COUPE_TRAIT, textes::COUPE_TRAIT_INDICE, None),
                (
                    MATERIAU,
                    textes::COUPE_MATERIAU,
                    textes::COUPE_MATERIAU_INDICE,
                    None,
                ),
            ] {
                ui.label(RichText::new(libelle.traduit()).font(theme::police_grasse(13.0)));
                let largeur = ui.available_width();
                modifie |= champ(ui, p, &mut saisies[rang], largeur, indice.traduit());
                if let Some(aide) = aide {
                    ui.label(
                        RichText::new(aide.traduit())
                            .size(12.0)
                            .color(p.texte_attenue),
                    );
                }
                ui.add_space(4.0);
            }
        });

        // Le plan de coupe.
        let donnees = match lire(saisies) {
            Ok(donnees) => donnees,
            Err(message) => {
                ui.add_space(8.0);
                ui.label(RichText::new(message).color(p.avertissement));
                return;
            }
        };
        if donnees.morceaux.is_empty() {
            return;
        }
        let plan = plan_en_cache(ui, saisies, &donnees);
        let en_clair = |pouces: f64| Quantite::Longueur(pouces).en_clair(donnees.modele, prefs);
        section(ui, p, textes::COUPE_PLAN.traduit());
        if !plan.trop_longs.is_empty() {
            let liste: Vec<String> = plan.trop_longs.iter().map(|m| en_clair(*m)).collect();
            ui.label(
                RichText::new(textes::coupe_trop_longs(&liste.join(", "))).color(p.avertissement),
            );
            ui.add_space(4.0);
        }
        if plan.pieces.is_empty() {
            return;
        }
        let a_acheter: Vec<String> = plan
            .a_acheter()
            .iter()
            .map(|(longueur, nombre)| format!("{nombre} × {}", en_clair(*longueur)))
            .collect();
        let achetee = plan.longueur_achetee();
        let chute = textes::coupe_chute(
            &en_clair(plan.chute()),
            (100.0 * plan.chute() / achetee).round(),
        );
        let resultats = vec![
            (textes::COUPE_A_ACHETER.traduit(), a_acheter.join("; ")),
            (textes::COUPE_LONGUEUR_ACHETEE.traduit(), en_clair(achetee)),
            (textes::COUPE_CHUTE.traduit(), chute),
        ];
        carte(ui, p, |ui| {
            for (libelle, valeur) in &resultats {
                ui.label(RichText::new(*libelle).size(12.5).color(p.texte_attenue));
                ui.label(
                    RichText::new(valeur)
                        .font(theme::police_grasse(16.0))
                        .color(p.entete),
                );
            }
        });
        ui.add_space(6.0);
        // Chaque pièce : ses coupes dessinées à l'échelle de la plus longue.
        let plus_longue = plan
            .pieces
            .iter()
            .map(|piece| piece.longueur)
            .fold(0.0, f64::max);
        let mut details = Vec::with_capacity(plan.pieces.len());
        carte(ui, p, |ui| {
            for piece in &plan.pieces {
                let coupes: Vec<String> = piece.coupes.iter().map(|c| en_clair(*c)).collect();
                let detail = format!(
                    "{} : {}",
                    en_clair(piece.longueur),
                    textes::coupe_detail(
                        &coupes.join(" + "),
                        &en_clair(piece.chute),
                        piece.chute > 0.0
                    )
                );
                ui.label(RichText::new(&detail).size(13.0));
                dessiner_la_piece(ui, p, piece, plus_longue, donnees.trait_de_scie, &en_clair);
                ui.add_space(6.0);
                details.push(detail);
            }
        });

        ui.add_space(10.0);
        ui.horizontal_wrapped(|ui| {
            if ui
                .add(bouton_principal(
                    p,
                    textes::AJOUTER_AU_RUBAN.traduit(),
                    14.5,
                ))
                .clicked()
            {
                let morceaux: Vec<String> = donnees
                    .morceaux
                    .iter()
                    .map(|(longueur, nombre)| format!("{nombre} × {}", en_clair(*longueur)))
                    .collect();
                let mut entrees = vec![
                    (textes::COUPE_MORCEAUX.traduit(), morceaux.join("; ")),
                    (
                        textes::COUPE_OFFERTES.traduit(),
                        saisies[OFFERTES].trim().to_string(),
                    ),
                    (
                        textes::COUPE_TRAIT.traduit(),
                        Quantite::Longueur(donnees.trait_de_scie)
                            .en_clair(Some(Format::PoFrac), prefs),
                    ),
                ];
                if !saisies[MATERIAU].trim().is_empty() {
                    entrees.push((
                        textes::COUPE_MATERIAU.traduit(),
                        saisies[MATERIAU].trim().to_string(),
                    ));
                }
                let mut tout = resultats.clone();
                tout.extend(
                    details
                        .iter()
                        .map(|d| (textes::COUPE_PIECE.traduit(), d.clone())),
                );
                action = Some(Action::Consigner {
                    titre: textes::LISTE_DE_COUPE.traduit(),
                    entrees,
                    resultats: tout,
                });
            }
            if ui
                .button(textes::AJOUTER_A_LA_LISTE_D_ACHAT.traduit())
                .on_hover_text(textes::INFOBULLE_LISTE_D_ACHAT.traduit())
                .clicked()
            {
                let materiau = match saisies[MATERIAU].trim() {
                    "" => textes::COUPE_PIECES_DE_STOCK.traduit().to_string(),
                    materiau => materiau.to_string(),
                };
                let articles = plan
                    .a_acheter()
                    .iter()
                    .map(|(longueur, nombre)| Article {
                        description: format!("{materiau} · {}", en_clair(*longueur)),
                        nombre: *nombre as u64,
                    })
                    .collect();
                action = Some(Action::ListeDAchat(articles));
            }
            if ui.button(textes::RECOMMENCER.traduit()).clicked() {
                *saisies = saisies_de_depart();
                modifie = true;
            }
        });
        ui.add_space(14.0);
    });
    if modifie && action.is_none() {
        action = Some(Action::Modifie);
    }
    action
}

/// Une pièce de stock, à l'échelle de la plus longue : ses coupes en couleurs
/// alternées, leur longueur écrite si elle y tient, puis la chute.
fn dessiner_la_piece(
    ui: &mut Ui,
    p: &Palette,
    piece: &PieceDeStock,
    plus_longue: f64,
    trait_de_scie: f64,
    en_clair: &dyn Fn(f64) -> String,
) {
    let largeur = ui.available_width();
    let (zone, _) = ui.allocate_exact_size(Vec2::new(largeur, 24.0), Sense::hover());
    if !ui.is_rect_visible(zone) || plus_longue <= 0.0 {
        return;
    }
    let echelle = largeur as f64 / plus_longue;
    let painter = ui.painter();
    let barre = Rect::from_min_size(
        zone.min,
        Vec2::new((piece.longueur * echelle) as f32, zone.height()),
    );
    // La chute : le fond de la pièce.
    painter.rect_filled(barre, 3, theme::melanger(p.carte, p.texte_attenue, 0.22));
    let mut x = 0.0;
    for (rang, coupe) in piece.coupes.iter().enumerate() {
        let couleurs = if rang % 2 == 0 {
            p.touche_fonction
        } else {
            p.touche_construction
        };
        let morceau = Rect::from_min_size(
            Pos2::new(zone.left() + (x * echelle) as f32, zone.top()),
            Vec2::new(((coupe * echelle) as f32).max(1.0), zone.height()),
        );
        painter.rect_filled(morceau, 2, couleurs.fond);
        let texte = en_clair(*coupe);
        let police = theme::police_grasse(11.0);
        let ecrit = painter.layout_no_wrap(texte, police, couleurs.texte);
        if ecrit.size().x + 6.0 <= morceau.width() {
            painter.galley(morceau.center() - ecrit.size() / 2.0, ecrit, couleurs.texte);
        }
        x += coupe + trait_de_scie;
    }
    painter.rect_stroke(
        barre,
        3,
        Stroke::new(1.0, p.carte_bord),
        egui::StrokeKind::Inside,
    );
}
