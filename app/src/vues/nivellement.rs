//! Écran « Carnet de nivellement » (section 22.21) : le repère, la lecture
//! faite sur lui et l'élévation visée, puis une ligne par point — son nom, sa
//! lecture au niveau laser et, s'il le faut, son élévation visée. Chaque
//! point donne son élévation et ce qu'il faut creuser ou remblayer.

use eframe::egui::{self, RichText, Ui};
use noyau::Preferences;
use noyau::analyse::analyser_longueur;
use noyau::langue::{self, Langue};
use noyau::metiers::Quantite;
use noyau::nivellement::{PointNivele, hauteur_de_l_instrument, niveler};
use noyau::unites::Format;

use super::metiers::{Action, Position};
use super::{cadre_de_saisie, carte, corps, en_tete, section};
use crate::stockage::{Carnet, MAX_POINTS, PointDuCarnet};
use crate::textes;
use crate::theme::{self, Palette};

/// Un écart plus petit qu'un trente-deuxième de pouce est « au niveau ».
const AU_NIVEAU: f64 = 1.0 / 32.0;

/// Une longueur tapée : rien si le champ est vide; une erreur s'il ne se lit
/// pas. Un nombre seul est en pieds; une élévation peut être négative.
fn longueur(texte: &str) -> Result<Option<(f64, Format)>, ()> {
    let texte = texte.trim();
    if texte.is_empty() {
        return Ok(None);
    }
    analyser_longueur(texte, Format::PiPo)
        .ok()
        .filter(|valeur| valeur.base.is_finite())
        .map(|valeur| Some((valeur.base, valeur.format)))
        .ok_or(())
}

/// Ce que le carnet donne, une fois lu.
struct Calcul {
    /// La hauteur de l'instrument, si le repère et sa lecture se lisent.
    hauteur: Option<f64>,
    /// Pour chaque point : son résultat, ou rien (incomplet), ou une erreur.
    points: Vec<Result<Option<PointNivele>, ()>>,
    /// Le format de la première longueur tapée : il donne le ton.
    modele: Option<Format>,
    /// Ce qui ne se lit pas dans le repère.
    erreur: Option<&'static str>,
}

fn calculer(carnet: &Carnet) -> Calcul {
    let mut modele = None;
    let mut retenir = |lu: &Option<(f64, Format)>| {
        if modele.is_none() {
            modele = lu.map(|(_, format)| format);
        }
    };
    let repere = longueur(&carnet.repere);
    let lecture = longueur(&carnet.lecture_repere);
    let visee = longueur(&carnet.visee);
    let mut erreur = None;
    for (lu, message) in [
        (&repere, textes::NIVELLEMENT_REPERE_ILLISIBLE),
        (&lecture, textes::NIVELLEMENT_LECTURE_ILLISIBLE),
        (&visee, textes::NIVELLEMENT_VISEE_ILLISIBLE),
    ] {
        match lu {
            Ok(valeur) => retenir(valeur),
            Err(()) => erreur = erreur.or(Some(message.traduit())),
        }
    }
    let hauteur = match (&repere, &lecture) {
        (Ok(Some((r, _))), Ok(Some((l, _)))) => Some(hauteur_de_l_instrument(*r, *l)),
        _ => None,
    };
    let visee_du_carnet = visee.ok().flatten().map(|(v, _)| v);
    let points = carnet
        .points
        .iter()
        .map(|point| {
            let lue = longueur(&point.lecture)?;
            let propre = longueur(&point.visee)?;
            retenir(&lue);
            let Some(((lecture, _), hauteur)) = lue.zip(hauteur) else {
                return Ok(None);
            };
            let visee = propre.map(|(v, _)| v).or(visee_du_carnet);
            Ok(Some(niveler(hauteur, lecture, visee)))
        })
        .collect();
    Calcul {
        hauteur,
        points,
        modele,
        erreur,
    }
}

/// « Creuser 6 po », « Remblayer 3 1/4 po » ou « Au niveau ».
fn ecart_en_clair(ecart: f64, en_clair: &dyn Fn(f64) -> String) -> String {
    if ecart.abs() < AU_NIVEAU {
        textes::NIVELLEMENT_AU_NIVEAU.traduit().to_string()
    } else if ecart > 0.0 {
        textes::nivellement_creuser(&en_clair(ecart))
    } else {
        textes::nivellement_remblayer(&en_clair(-ecart))
    }
}

/// Un champ, avec son libellé au-dessus.
fn champ_libelle(
    ui: &mut Ui,
    p: &Palette,
    libelle: &str,
    texte: &mut String,
    indice: &str,
) -> bool {
    ui.label(RichText::new(libelle).font(theme::police_grasse(13.0)));
    let reponse = ui.add(
        egui::TextEdit::singleline(texte)
            .desired_width(ui.available_width())
            .char_limit(40)
            .hint_text(indice),
    );
    cadre_de_saisie(ui, p, &reponse);
    ui.add_space(4.0);
    reponse.changed()
}

/// Un champ d'une ligne de point, de largeur fixe.
fn champ_de_point(
    ui: &mut Ui,
    p: &Palette,
    texte: &mut String,
    largeur: f32,
    indice: &str,
) -> bool {
    let reponse = ui.add_sized(
        egui::vec2(largeur, 26.0),
        egui::TextEdit::singleline(texte)
            .char_limit(30)
            .hint_text(indice),
    );
    cadre_de_saisie(ui, p, &reponse);
    reponse.changed()
}

/// Affiche le carnet de nivellement.
pub fn afficher(
    ui: &mut Ui,
    p: &Palette,
    position: &mut Position,
    carnet: &mut Carnet,
    prefs: &Preferences,
) -> Option<Action> {
    let mut action = None;
    if en_tete(
        ui,
        p,
        textes::CARNET_DE_NIVELLEMENT.traduit(),
        textes::RETOUR.traduit(),
    ) {
        position.carnet = false;
    }
    if carnet.points.is_empty() {
        carnet.points.push(PointDuCarnet::default());
    }
    let mut modifie = false;
    corps(ui, "carnet-de-nivellement", |ui| {
        ui.add_space(6.0);
        ui.label(RichText::new(textes::NIVELLEMENT_RESUME.traduit()).color(p.texte_attenue));
        ui.add_space(6.0);
        carte(ui, p, |ui| {
            modifie |= champ_libelle(
                ui,
                p,
                textes::NIVELLEMENT_TITRE.traduit(),
                &mut carnet.titre,
                textes::NIVELLEMENT_TITRE_INDICE.traduit(),
            );
        });

        // Le repère, la lecture faite sur lui, et l'élévation visée.
        section(ui, p, textes::NIVELLEMENT_REPERE.traduit());
        carte(ui, p, |ui| {
            modifie |= champ_libelle(
                ui,
                p,
                textes::NIVELLEMENT_ELEVATION_DU_REPERE.traduit(),
                &mut carnet.repere,
                "100",
            );
            modifie |= champ_libelle(
                ui,
                p,
                textes::NIVELLEMENT_LECTURE_SUR_LE_REPERE.traduit(),
                &mut carnet.lecture_repere,
                textes::NIVELLEMENT_INDICE_LECTURE.traduit(),
            );
            modifie |= champ_libelle(
                ui,
                p,
                textes::NIVELLEMENT_VISEE.traduit(),
                &mut carnet.visee,
                textes::NIVELLEMENT_INDICE_VISEE.traduit(),
            );
            ui.label(
                RichText::new(textes::NIVELLEMENT_AIDE.traduit())
                    .size(12.0)
                    .color(p.texte_attenue),
            );
        });

        let calcul = calculer(carnet);
        let en_clair = |pouces: f64| Quantite::Longueur(pouces).en_clair(calcul.modele, prefs);
        if let Some(message) = calcul.erreur {
            ui.label(RichText::new(message).color(p.avertissement));
        }
        if let Some(hauteur) = calcul.hauteur {
            ui.add_space(4.0);
            ui.label(
                RichText::new(textes::nivellement_hauteur(&en_clair(hauteur)))
                    .font(theme::police_grasse(14.0))
                    .color(p.entete),
            );
        }

        // Les points : nom, lecture, visée; puis l'élévation et l'écart.
        section(ui, p, textes::NIVELLEMENT_POINTS.traduit());
        let mut a_retirer = None;
        let nombre = carnet.points.len();
        carte(ui, p, |ui| {
            for (rang, point) in carnet.points.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    let reste = (ui.available_width() - 72.0 - 3.0 * 6.0).max(150.0);
                    modifie |= champ_de_point(
                        ui,
                        p,
                        &mut point.nom,
                        reste * 0.30,
                        &format!("P{}", rang + 1),
                    );
                    modifie |= champ_de_point(
                        ui,
                        p,
                        &mut point.lecture,
                        reste * 0.35,
                        textes::NIVELLEMENT_COLONNE_LECTURE.traduit(),
                    );
                    modifie |= champ_de_point(
                        ui,
                        p,
                        &mut point.visee,
                        reste * 0.35,
                        textes::NIVELLEMENT_COLONNE_VISEE.traduit(),
                    );
                    if nombre > 1 && ui.small_button(textes::DESSIN_ENLEVER.traduit()).clicked() {
                        a_retirer = Some(rang);
                    }
                });
                let ligne = match calcul.points.get(rang) {
                    Some(Ok(Some(resultat))) => {
                        let mut texte =
                            textes::nivellement_elevation(&en_clair(resultat.elevation));
                        if let Some(ecart) = resultat.ecart {
                            texte.push_str(" · ");
                            texte.push_str(&ecart_en_clair(ecart, &en_clair));
                        }
                        Some((texte, p.texte))
                    }
                    Some(Err(())) => Some((
                        textes::NIVELLEMENT_POINT_ILLISIBLE.traduit().to_string(),
                        p.avertissement,
                    )),
                    _ => None,
                };
                if let Some((texte, couleur)) = ligne {
                    ui.label(RichText::new(texte).size(13.0).color(couleur));
                }
                ui.add_space(4.0);
            }
            if nombre < MAX_POINTS
                && ui
                    .button(textes::NIVELLEMENT_PLUS_POINT.traduit())
                    .clicked()
            {
                carnet.points.push(PointDuCarnet::default());
                modifie = true;
            }
        });
        if let Some(rang) = a_retirer {
            carnet.points.remove(rang);
            modifie = true;
        }

        ui.add_space(10.0);
        ui.horizontal_wrapped(|ui| {
            if ui
                .button(textes::NIVELLEMENT_EXPORTER.traduit())
                .on_hover_text(textes::INFOBULLE_NIVELLEMENT_EXPORTER.traduit())
                .clicked()
            {
                action = Some(Action::ExporterCarnet);
            }
            if ui.button(textes::NIVELLEMENT_VIDER.traduit()).clicked() {
                *carnet = Carnet::default();
                carnet.points.push(PointDuCarnet::default());
                modifie = true;
            }
        });
        ui.add_space(14.0);
    });
    if modifie && action.is_none() {
        action = Some(Action::CarnetModifie);
    }
    action
}

/// Une cellule d'un fichier CSV, entre guillemets si elle contient le
/// séparateur, un guillemet ou un saut de ligne.
fn cellule(texte: &str, separateur: char) -> String {
    if texte.contains([separateur, '"', '\n', '\r']) {
        format!("\"{}\"", texte.replace('"', "\"\""))
    } else {
        texte.to_string()
    }
}

/// Le carnet en CSV (UTF-8 avec indicateur d'ordre des octets, fins de ligne
/// Windows), pour un tableur : le point-virgule sépare en français, la
/// virgule en anglais.
pub fn en_csv(carnet: &Carnet, prefs: &Preferences) -> String {
    let separateur = match langue::en_cours() {
        Langue::Francais => ';',
        Langue::Anglais => ',',
    };
    let calcul = calculer(carnet);
    let en_clair = |pouces: f64| {
        Quantite::Longueur(pouces)
            .en_clair(calcul.modele, prefs)
            .replace('\u{a0}', " ")
    };
    let ligne = |cellules: &[String]| -> String {
        cellules
            .iter()
            .map(|c| cellule(c, separateur))
            .collect::<Vec<_>>()
            .join(&separateur.to_string())
            + "\r\n"
    };
    let mut csv = String::from("\u{feff}");
    csv.push_str(&ligne(&[
        textes::CARNET_DE_NIVELLEMENT.traduit().to_string(),
        carnet.titre.trim().to_string(),
    ]));
    let mut repere = vec![
        textes::NIVELLEMENT_ELEVATION_DU_REPERE
            .traduit()
            .to_string(),
        carnet.repere.trim().to_string(),
        textes::NIVELLEMENT_LECTURE_SUR_LE_REPERE
            .traduit()
            .to_string(),
        carnet.lecture_repere.trim().to_string(),
    ];
    if let Some(hauteur) = calcul.hauteur {
        repere.push(textes::NIVELLEMENT_HAUTEUR.traduit().to_string());
        repere.push(en_clair(hauteur));
    }
    csv.push_str(&ligne(&repere));
    csv.push_str("\r\n");
    csv.push_str(&ligne(&[
        textes::NIVELLEMENT_COLONNE_POINT.traduit().to_string(),
        textes::NIVELLEMENT_COLONNE_LECTURE.traduit().to_string(),
        textes::NIVELLEMENT_COLONNE_ELEVATION.traduit().to_string(),
        textes::NIVELLEMENT_COLONNE_VISEE.traduit().to_string(),
        textes::NIVELLEMENT_COLONNE_CREUSER.traduit().to_string(),
        textes::NIVELLEMENT_COLONNE_REMBLAYER.traduit().to_string(),
    ]));
    for (point, resultat) in carnet.points.iter().zip(&calcul.points) {
        if point.nom.trim().is_empty() && point.lecture.trim().is_empty() {
            continue;
        }
        let (elevation, creuser, remblayer) = match resultat {
            Ok(Some(r)) => {
                let (creuser, remblayer) = match r.ecart {
                    Some(e) if e >= AU_NIVEAU => (en_clair(e), String::new()),
                    Some(e) if e <= -AU_NIVEAU => (String::new(), en_clair(-e)),
                    _ => (String::new(), String::new()),
                };
                (en_clair(r.elevation), creuser, remblayer)
            }
            _ => (String::new(), String::new(), String::new()),
        };
        let visee = if point.visee.trim().is_empty() {
            carnet.visee.trim()
        } else {
            point.visee.trim()
        };
        csv.push_str(&ligne(&[
            point.nom.trim().to_string(),
            point.lecture.trim().to_string(),
            elevation,
            visee.to_string(),
            creuser,
            remblayer,
        ]));
    }
    csv
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_carnet_en_csv() {
        let carnet = Carnet {
            titre: "Entrée; garage".to_string(),
            repere: "100".to_string(),
            lecture_repere: "5".to_string(),
            visee: "99".to_string(),
            points: vec![
                PointDuCarnet {
                    nom: "P1".to_string(),
                    lecture: "6 pi 3 po".to_string(),
                    visee: String::new(),
                },
                PointDuCarnet {
                    nom: "P2".to_string(),
                    lecture: "4".to_string(),
                    visee: String::new(),
                },
                PointDuCarnet::default(),
            ],
        };
        let calcul = calculer(&carnet);
        assert_eq!(calcul.hauteur, Some(1260.0));
        assert_eq!(
            calcul.points[0],
            Ok(Some(niveler(1260.0, 75.0, Some(1188.0))))
        );
        assert_eq!(calcul.points[2], Ok(None));
        let csv = en_csv(&carnet, &Preferences::default());
        assert!(csv.starts_with('\u{feff}'));
        // Un titre qui contient le séparateur est mis entre guillemets.
        assert!(csv.contains("\"Entrée; garage\""));
        let lignes: Vec<&str> = csv.split("\r\n").collect();
        assert!(
            lignes[4].starts_with("P1;6 pi 3 po;98 pi 9 po;99;;3 po"),
            "{}",
            lignes[4]
        );
        assert!(
            lignes[5].starts_with("P2;4;101 pi 0 po;99;2 pi 0 po;"),
            "{}",
            lignes[5]
        );
        // La ligne vide n'y est pas.
        assert_eq!(lignes.len(), 7);
    }

    #[test]
    fn une_lecture_illisible() {
        let carnet = Carnet {
            repere: "100".to_string(),
            lecture_repere: "cinq".to_string(),
            points: vec![PointDuCarnet {
                nom: "P1".to_string(),
                lecture: "4".to_string(),
                visee: String::new(),
            }],
            ..Carnet::default()
        };
        let calcul = calculer(&carnet);
        assert!(calcul.erreur.is_some());
        assert_eq!(calcul.hauteur, None);
        assert_eq!(calcul.points[0], Ok(None));
    }
}
