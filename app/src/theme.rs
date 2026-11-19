//! Identité visuelle « Constructo » (section 16.3) : palettes, polices, dégradés,
//! ombres, fond quadrillé et icône.
//!
//! Les couleurs reprennent celles de l'ERP Constructo AI : bleu primaire, barres
//! bleu marine en dégradé, accent orange, fond façon plan quadrillé.
//! Toutes les couleurs de l'application sont définies ici.

use std::sync::Arc;

use eframe::egui::epaint::{RectShape, Shadow};
use eframe::egui::{
    self, Color32, FontData, FontDefinitions, FontFamily, FontId, Painter, Pos2, Rect, Stroke, Ui,
};
use noyau::Touche;

/// Couleurs d'une touche : dégradé de la face, texte et liseré.
#[derive(Clone, Copy, Debug)]
pub struct CouleursTouche {
    /// Haut du dégradé (couleur de référence pour le contraste).
    pub fond: Color32,
    /// Bas du dégradé.
    pub fond_bas: Color32,
    pub texte: Color32,
    pub bord: Color32,
}

/// Jetons de couleur d'un thème.
#[derive(Clone, Copy, Debug)]
pub struct Palette {
    pub sombre: bool,
    /// Fond de l'application : dégradé en diagonale, du premier au second.
    pub fond_app: Color32,
    pub fond_app_2: Color32,
    /// Quadrillage du fond : lignes fines et lignes majeures.
    pub grille: Color32,
    pub grille_majeure: Color32,
    /// Barre supérieure : dégradé du haut vers le bas.
    pub barre_fond: Color32,
    pub barre_fond_bas: Color32,
    /// Barre inférieure et tiroir du menu.
    pub pied_fond: Color32,
    pub pied_fond_bas: Color32,
    pub barre_texte: Color32,
    /// Repère de l'onglet actif.
    pub barre_accent: Color32,
    /// Afficheur.
    pub ecran_fond: Color32,
    pub ecran_fond_bas: Color32,
    pub ecran_bord: Color32,
    pub ecran_encre: Color32,
    pub ecran_secondaire: Color32,
    pub ecran_etiquette: Color32,
    /// Halo sous l'afficheur.
    pub ecran_halo: Color32,
    pub touche_fonction: CouleursTouche,
    pub touche_construction: CouleursTouche,
    pub touche_chiffre: CouleursTouche,
    pub touche_conv: CouleursTouche,
    pub touche_effacer: CouleursTouche,
    pub etiquette_secondaire: Color32,
    pub avertissement: Color32,
    pub erreur: Color32,
    /// Texte courant sur le fond de l'application.
    pub texte: Color32,
    pub texte_attenue: Color32,
    /// Grands titres des écrans.
    pub entete: Color32,
    /// Titres de section et liens.
    pub titre: Color32,
    pub primaire: Color32,
    /// Fond des cartes, des listes et des dialogues.
    pub carte: Color32,
    pub carte_bord: Color32,
    /// Fond des champs de saisie.
    pub champ: Color32,
    /// Voile derrière un tiroir ou un dialogue.
    pub voile: Color32,
    /// Boutons courants des écrans secondaires.
    pub bouton: Color32,
    pub bouton_bord: Color32,
    /// Élément sélectionné et bouton d'action principal.
    pub selection: Color32,
    pub selection_texte: Color32,
    /// Petites touches dessinées dans les textes d'aide.
    pub puce: Color32,
    pub puce_texte: Color32,
    /// Ombre portée neutre.
    pub ombre: Color32,
}

const fn rvb(hex: u32) -> Color32 {
    Color32::from_rgb((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
}

const fn touche(haut: u32, bas: u32, texte: u32, bord: u32) -> CouleursTouche {
    CouleursTouche {
        fond: rvb(haut),
        fond_bas: rvb(bas),
        texte: rvb(texte),
        bord: rvb(bord),
    }
}

pub const CLAIR: Palette = Palette {
    sombre: false,
    fond_app: rvb(0xFFFFFF),
    fond_app_2: rvb(0xCFE2FC),
    // Bleu primaire, très transparent (couleurs prémultipliées).
    grille: Color32::from_rgba_premultiplied(0, 6, 11, 13),
    grille_majeure: Color32::from_rgba_premultiplied(0, 11, 20, 24),
    barre_fond: rvb(0x002B6B),
    barre_fond_bas: rvb(0x002050),
    pied_fond: rvb(0x002050),
    pied_fond_bas: rvb(0x001833),
    barre_texte: rvb(0xFFFFFF),
    barre_accent: rvb(0xF97316),
    ecran_fond: rvb(0xFFFFFF),
    ecran_fond_bas: rvb(0xEAF2FF),
    ecran_bord: rvb(0xB4D0F0),
    ecran_encre: rvb(0x002050),
    ecran_secondaire: rvb(0x605E5C),
    ecran_etiquette: rvb(0x005EA2),
    ecran_halo: Color32::from_rgba_premultiplied(5, 14, 33, 38),
    touche_fonction: touche(0x0078D4, 0x005EA2, 0xFFFFFF, 0x004E8C),
    touche_construction: touche(0x002B6B, 0x002050, 0xFFFFFF, 0x001833),
    touche_chiffre: touche(0xFFFFFF, 0xEEF3F9, 0x002050, 0xC3CFDD),
    touche_conv: touche(0xFB923C, 0xF97316, 0x001833, 0xEA580C),
    touche_effacer: touche(0xDC2626, 0xB91C1C, 0xFFFFFF, 0x991B1B),
    etiquette_secondaire: rvb(0x9A3412),
    avertissement: rvb(0x92400E),
    erreur: rvb(0xB91C1C),
    texte: rvb(0x323130),
    texte_attenue: rvb(0x605E5C),
    entete: rvb(0x002050),
    titre: rvb(0x005EA2),
    primaire: rvb(0x0078D4),
    carte: rvb(0xFFFFFF),
    carte_bord: rvb(0xDCE4EE),
    champ: rvb(0xFFFFFF),
    voile: Color32::from_rgba_premultiplied(0, 7, 18, 115),
    bouton: rvb(0xFFFFFF),
    bouton_bord: rvb(0x8A8886),
    selection: rvb(0x0078D4),
    selection_texte: rvb(0xFFFFFF),
    puce: rvb(0xDEECF9),
    puce_texte: rvb(0x004578),
    ombre: Color32::from_rgba_premultiplied(0, 6, 14, 46),
};

pub const SOMBRE: Palette = Palette {
    sombre: true,
    fond_app: rvb(0x0F1923),
    fond_app_2: rvb(0x162340),
    grille: Color32::from_rgba_premultiplied(10, 10, 10, 10),
    grille_majeure: Color32::from_rgba_premultiplied(19, 19, 19, 19),
    barre_fond: rvb(0x002B6B),
    barre_fond_bas: rvb(0x002050),
    pied_fond: rvb(0x002050),
    pied_fond_bas: rvb(0x001833),
    barre_texte: rvb(0xFFFFFF),
    barre_accent: rvb(0xF97316),
    ecran_fond: rvb(0x1E293B),
    ecran_fond_bas: rvb(0x0F172A),
    ecran_bord: rvb(0x334155),
    ecran_encre: rvb(0xF8FAFC),
    ecran_secondaire: rvb(0xA8B3C2),
    ecran_etiquette: rvb(0x6CB8F6),
    ecran_halo: Color32::from_rgba_premultiplied(0, 26, 46, 56),
    touche_fonction: touche(0x0078D4, 0x005EA2, 0xFFFFFF, 0x2B88D8),
    touche_construction: touche(0x003380, 0x002359, 0xFFFFFF, 0x1F4E96),
    touche_chiffre: touche(0x334155, 0x1E293B, 0xF8FAFC, 0x475569),
    touche_conv: touche(0xFB923C, 0xF97316, 0x001833, 0xFDBA74),
    touche_effacer: touche(0xDC2626, 0xB91C1C, 0xFFFFFF, 0xEF4444),
    etiquette_secondaire: rvb(0xFDBA74),
    avertissement: rvb(0xFCD34D),
    erreur: rvb(0xF87171),
    texte: rvb(0xF3F2F1),
    texte_attenue: rvb(0xA8B3C2),
    entete: rvb(0xFFFFFF),
    titre: rvb(0x6CB8F6),
    primaire: rvb(0x2B88D8),
    carte: rvb(0x1F2937),
    carte_bord: rvb(0x374151),
    champ: rvb(0x111827),
    voile: Color32::from_rgba_premultiplied(0, 0, 0, 153),
    bouton: rvb(0x374151),
    bouton_bord: rvb(0x4B5563),
    selection: rvb(0x0078D4),
    selection_texte: rvb(0xFFFFFF),
    puce: rvb(0x17324D),
    puce_texte: rvb(0xA8CCFF),
    ombre: Color32::from_rgba_premultiplied(0, 0, 0, 115),
};

pub fn palette(sombre: bool) -> &'static Palette {
    if sombre { &SOMBRE } else { &CLAIR }
}

/// Famille de couleur d'une touche (section 16.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Famille {
    Fonction,
    Construction,
    Chiffre,
    Conv,
    Effacer,
}

pub fn famille(t: Touche) -> Famille {
    match t {
        Touche::Pente
        | Touche::Montee
        | Touche::Course
        | Touche::Diag
        | Touche::AretierNoue
        | Touche::Verges
        | Touche::Pieds
        | Touche::Pouces
        | Touche::Fraction
        | Touche::Stocker
        | Touche::Rappel => Famille::Fonction,
        Touche::Chiffre(_) | Touche::Virgule | Touche::FractionRapide { .. } => Famille::Chiffre,
        Touche::Conv => Famille::Conv,
        Touche::Effacer => Famille::Effacer,
        _ => Famille::Construction,
    }
}

impl Palette {
    pub fn couleurs(&self, f: Famille) -> CouleursTouche {
        match f {
            Famille::Fonction => self.touche_fonction,
            Famille::Construction => self.touche_construction,
            Famille::Chiffre => self.touche_chiffre,
            Famille::Conv => self.touche_conv,
            Famille::Effacer => self.touche_effacer,
        }
    }
}

/// Mélange deux couleurs opaques : `t` = 0 donne `a`, 1 donne `b`.
pub fn melanger(a: Color32, b: Color32, t: f32) -> Color32 {
    let m = |x: u8, y: u8| -> u8 {
        (f32::from(x) + (f32::from(y) - f32::from(x)) * t)
            .round()
            .clamp(0.0, 255.0) as u8
    };
    Color32::from_rgb(m(a.r(), b.r()), m(a.g(), b.g()), m(a.b(), b.b()))
}

/// Assombrit une couleur d'une fraction donnée (0,12 = 12 %).
pub fn assombrir(c: Color32, fraction: f32) -> Color32 {
    melanger(c, Color32::BLACK, fraction)
}

/// Éclaircit une couleur d'une fraction donnée.
pub fn eclaircir(c: Color32, fraction: f32) -> Color32 {
    melanger(c, Color32::WHITE, fraction)
}

// ----------------------------------------------------------------------
// Dégradés, ombres et fond
// ----------------------------------------------------------------------

fn id_de_la_rampe() -> egui::Id {
    egui::Id::unique("rampe-de-degrade")
}

/// Crée la texture qui sert à tous les dégradés : une rampe d'opacité, de
/// transparente en haut à opaque en bas. La poignée doit rester en vie tant que
/// l'application dessine.
pub fn creer_la_rampe(ctx: &egui::Context) -> egui::TextureHandle {
    const HAUTEUR: usize = 64;
    let pixels = (0..HAUTEUR)
        .map(|i| {
            let a = (i as f32 / (HAUTEUR - 1) as f32 * 255.0).round() as u8;
            Color32::from_rgba_premultiplied(a, a, a, a)
        })
        .collect();
    let image = egui::ColorImage::new([1, HAUTEUR], pixels);
    let poignee = ctx.load_texture("rampe-de-degrade", image, egui::TextureOptions::LINEAR);
    ctx.data_mut(|d| d.insert_temp(id_de_la_rampe(), poignee.id()));
    poignee
}

/// Rectangle arrondi rempli d'un dégradé vertical, de `haut` à `bas`.
pub fn rect_degrade(painter: &Painter, rect: Rect, rayon: u8, haut: Color32, bas: Color32) {
    painter.rect_filled(rect, rayon, haut);
    if haut == bas {
        return;
    }
    let rampe = painter
        .ctx()
        .data(|d| d.get_temp::<egui::TextureId>(id_de_la_rampe()));
    if let Some(texture) = rampe {
        // La couleur du bas, modulée par la rampe d'opacité, recouvre celle du haut.
        let uv = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0));
        painter.add(RectShape::filled(rect, rayon, bas).with_texture(texture, uv));
    }
}

/// Ombre portée douce sous un rectangle arrondi.
pub fn ombre_portee(
    painter: &Painter,
    rect: Rect,
    rayon: u8,
    couleur: Color32,
    (flou, decalage): (u8, i8),
) {
    let ombre = Shadow {
        offset: [0, decalage],
        blur: flou,
        spread: 0,
        color: couleur,
    };
    painter.add(ombre.as_shape(rect, rayon));
}

/// Pas du quadrillage du fond et fréquence des lignes majeures.
const PAS_DE_GRILLE: f32 = 24.0;
const LIGNES_PAR_MAJEURE: i64 = 5;

/// Peint le fond de l'application dans `rect` : un dégradé en diagonale et un
/// quadrillage façon plan. Le motif est calé sur la fenêtre entière, pour se
/// raccorder d'un panneau à l'autre.
pub fn peindre_le_fond(ui: &Ui, rect: Rect, p: &Palette) {
    let fenetre = ui.ctx().content_rect();
    let couleur = |pos: Pos2| -> Color32 {
        let tx = ((pos.x - fenetre.left()) / fenetre.width().max(1.0)).clamp(0.0, 1.0);
        let ty = ((pos.y - fenetre.top()) / fenetre.height().max(1.0)).clamp(0.0, 1.0);
        melanger(p.fond_app, p.fond_app_2, (tx + ty) / 2.0)
    };
    let painter = ui.painter_at(rect);
    let mut maillage = egui::Mesh::default();
    for coin in [
        rect.left_top(),
        rect.right_top(),
        rect.right_bottom(),
        rect.left_bottom(),
    ] {
        maillage.colored_vertex(coin, couleur(coin));
    }
    maillage.add_triangle(0, 1, 2);
    maillage.add_triangle(0, 2, 3);
    painter.add(egui::Shape::mesh(maillage));

    let trait_de = |k: i64| -> Stroke {
        let couleur = if k % LIGNES_PAR_MAJEURE == 0 {
            p.grille_majeure
        } else {
            p.grille
        };
        Stroke::new(1.0, couleur)
    };
    let mut k = ((rect.left() - fenetre.left()) / PAS_DE_GRILLE).ceil() as i64;
    loop {
        let x = fenetre.left() + k as f32 * PAS_DE_GRILLE;
        if x > rect.right() {
            break;
        }
        painter.line_segment(
            [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
            trait_de(k),
        );
        k += 1;
    }
    let mut k = ((rect.top() - fenetre.top()) / PAS_DE_GRILLE).ceil() as i64;
    loop {
        let y = fenetre.top() + k as f32 * PAS_DE_GRILLE;
        if y > rect.bottom() {
            break;
        }
        painter.line_segment(
            [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
            trait_de(k),
        );
        k += 1;
    }
}

// ----------------------------------------------------------------------
// Polices
// ----------------------------------------------------------------------

/// Nom de la famille de police grasse.
const FAMILLE_GRAS: &str = "gras";
/// Nom de la famille de police italique.
const FAMILLE_ITALIQUE: &str = "italique";

pub fn police(taille: f32) -> FontId {
    FontId::proportional(taille)
}

pub fn police_grasse(taille: f32) -> FontId {
    FontId::new(taille, FontFamily::Name(FAMILLE_GRAS.into()))
}

pub fn police_italique(taille: f32) -> FontId {
    FontId::new(taille, FontFamily::Name(FAMILLE_ITALIQUE.into()))
}

/// Dossier des polices du système, s'il existe.
fn dossier_des_polices() -> Option<std::path::PathBuf> {
    #[cfg(windows)]
    {
        let windows = std::env::var_os("WINDIR").or_else(|| std::env::var_os("SystemRoot"))?;
        Some(std::path::PathBuf::from(windows).join("Fonts"))
    }
    #[cfg(not(windows))]
    {
        None
    }
}

/// Installe les polices : Segoe UI (courant, gras, italique) sous Windows, avec
/// repli sur les polices intégrées d'egui si un fichier est absent.
pub fn installer_les_polices(ctx: &egui::Context) {
    let mut definitions = FontDefinitions::default();
    let defaut: Vec<String> = definitions
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    let dossier = dossier_des_polices();
    let mut charger = |nom: &str, fichiers: &[&str]| -> bool {
        let Some(dossier) = &dossier else {
            return false;
        };
        for fichier in fichiers {
            if let Ok(octets) = std::fs::read(dossier.join(fichier)) {
                definitions
                    .font_data
                    .insert(nom.to_owned(), Arc::new(FontData::from_owned(octets)));
                return true;
            }
        }
        false
    };
    let a_courant = charger("segoe-ui", &["segoeui.ttf"]);
    let a_gras = charger("segoe-ui-gras", &["segoeuib.ttf"]);
    let a_italique = charger("segoe-ui-italique", &["segoeuii.ttf"]);
    // Polices de symboles du système, en renfort pour ⌫ ⚠ ☰ ⚙ ★ ◄ ►.
    let a_symboles = charger("symboles", &["seguisym.ttf"]);

    let chaine = |principale: Option<&str>| -> Vec<String> {
        let mut noms: Vec<String> = principale.iter().map(|s| (*s).to_owned()).collect();
        noms.extend(defaut.iter().cloned());
        if a_symboles {
            noms.push("symboles".to_owned());
        }
        noms
    };
    definitions.families.insert(
        FontFamily::Proportional,
        chaine(a_courant.then_some("segoe-ui")),
    );
    let gras = if a_gras {
        Some("segoe-ui-gras")
    } else {
        a_courant.then_some("segoe-ui")
    };
    definitions
        .families
        .insert(FontFamily::Name(FAMILLE_GRAS.into()), chaine(gras));
    let italique = if a_italique {
        Some("segoe-ui-italique")
    } else {
        a_courant.then_some("segoe-ui")
    };
    definitions
        .families
        .insert(FontFamily::Name(FAMILLE_ITALIQUE.into()), chaine(italique));
    ctx.set_fonts(definitions);
}

/// Règle le style des écrans secondaires (boutons, listes, champs) selon la palette.
pub fn installer_le_style(ctx: &egui::Context, p: &Palette) {
    use egui::{CornerRadius, TextStyle, Visuals};
    ctx.set_theme(if p.sombre {
        egui::Theme::Dark
    } else {
        egui::Theme::Light
    });
    // Les deux styles (clair et sombre) d'egui reçoivent la palette en vigueur.
    ctx.all_styles_mut(|style| {
        let mut v = if p.sombre {
            Visuals::dark()
        } else {
            Visuals::light()
        };
        v.panel_fill = p.fond_app;
        v.window_fill = p.carte;
        v.window_stroke = Stroke::new(1.0, p.carte_bord);
        v.window_corner_radius = CornerRadius::same(8);
        v.menu_corner_radius = CornerRadius::same(6);
        v.window_shadow = Shadow {
            offset: [0, 8],
            blur: 24,
            spread: 0,
            color: p.ombre,
        };
        v.popup_shadow = Shadow {
            offset: [0, 4],
            blur: 12,
            spread: 0,
            color: p.ombre,
        };
        v.override_text_color = Some(p.texte);
        v.weak_text_color = Some(p.texte_attenue);
        v.extreme_bg_color = p.champ;
        v.text_edit_bg_color = Some(p.champ);
        v.faint_bg_color = melanger(p.carte, p.primaire, 0.05);
        v.hyperlink_color = p.titre;
        v.selection.bg_fill = p.selection;
        v.selection.stroke = Stroke::new(1.0, p.selection_texte);
        let rayon = CornerRadius::same(4);
        v.widgets.noninteractive.bg_fill = p.carte;
        v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, p.carte_bord);
        v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, p.texte);
        v.widgets.noninteractive.corner_radius = rayon;
        for (etat, fraction) in [
            (&mut v.widgets.inactive, 0.0),
            (&mut v.widgets.hovered, 0.07),
            (&mut v.widgets.active, 0.14),
            (&mut v.widgets.open, 0.09),
        ] {
            let fond = melanger(p.bouton, p.texte, fraction);
            etat.bg_fill = fond;
            etat.weak_bg_fill = fond;
            etat.bg_stroke = Stroke::new(1.0, p.bouton_bord);
            etat.fg_stroke = Stroke::new(1.0, p.texte);
            etat.corner_radius = rayon;
            etat.expansion = 0.0;
        }
        // Survol : liseré de la couleur primaire, comme dans l'ERP.
        v.widgets.hovered.bg_stroke = Stroke::new(1.0, p.primaire);
        style.visuals = v;
        style.interaction.selectable_labels = false;
        style.spacing.item_spacing = egui::vec2(8.0, 7.0);
        style.spacing.button_padding = egui::vec2(12.0, 6.0);
        style.spacing.interact_size.y = 30.0;
        style.spacing.combo_width = 150.0;
        for (texte, taille) in [
            (TextStyle::Small, 12.0),
            (TextStyle::Body, 14.5),
            (TextStyle::Button, 14.5),
            (TextStyle::Heading, 20.0),
            (TextStyle::Monospace, 13.5),
        ] {
            style
                .text_styles
                .insert(texte, FontId::proportional(taille));
        }
    });
}

// ----------------------------------------------------------------------
// Icône
// ----------------------------------------------------------------------

/// Icône de la fenêtre : une équerre de charpentier orange sur fond bleu
/// marine, dessinée par le code (aucun fichier image). Retourne des octets RVBA.
pub fn icone_rvba(taille: u32) -> Vec<u8> {
    const MARINE: [f32; 3] = [0.0, 43.0, 107.0];
    const MARINE_BAS: [f32; 3] = [0.0, 24.0, 51.0];
    const ORANGE: [f32; 3] = [249.0, 115.0, 22.0];
    const SUR_ECHANTILLONS: u32 = 4;
    let n = taille.max(1);
    let mut pixels = Vec::with_capacity((n * n * 4) as usize);
    for y in 0..n {
        for x in 0..n {
            let mut somme = [0.0f32; 4];
            for sy in 0..SUR_ECHANTILLONS {
                for sx in 0..SUR_ECHANTILLONS {
                    let u = (x as f32 + (sx as f32 + 0.5) / SUR_ECHANTILLONS as f32) / n as f32;
                    let v = (y as f32 + (sy as f32 + 0.5) / SUR_ECHANTILLONS as f32) / n as f32;
                    // Le fond s'assombrit vers le bas, comme les barres.
                    let fond = [
                        MARINE[0] + (MARINE_BAS[0] - MARINE[0]) * v,
                        MARINE[1] + (MARINE_BAS[1] - MARINE[1]) * v,
                        MARINE[2] + (MARINE_BAS[2] - MARINE[2]) * v,
                    ];
                    if let Some(c) = couleur_icone(u, v, fond, ORANGE) {
                        somme[0] += c[0];
                        somme[1] += c[1];
                        somme[2] += c[2];
                        somme[3] += 1.0;
                    }
                }
            }
            let total = (SUR_ECHANTILLONS * SUR_ECHANTILLONS) as f32;
            let couverture = somme[3].max(1.0);
            pixels.push((somme[0] / couverture).round() as u8);
            pixels.push((somme[1] / couverture).round() as u8);
            pixels.push((somme[2] / couverture).round() as u8);
            pixels.push((somme[3] / total * 255.0).round() as u8);
        }
    }
    pixels
}

/// Couleur de l'icône au point (u, v) du carré unité; `None` hors du fond arrondi.
fn couleur_icone(u: f32, v: f32, fond: [f32; 3], equerre: [f32; 3]) -> Option<[f32; 3]> {
    const RAYON: f32 = 0.18;
    // Fond : carré aux coins arrondis.
    let dx = (u - 0.5).abs() - (0.5 - RAYON);
    let dy = (v - 0.5).abs() - (0.5 - RAYON);
    if dx > 0.0 && dy > 0.0 && dx * dx + dy * dy > RAYON * RAYON {
        return None;
    }
    let dans = |x0: f32, x1: f32, y0: f32, y1: f32| u >= x0 && u <= x1 && v >= y0 && v <= y1;
    // Équerre : une branche verticale et une branche horizontale.
    let verticale = dans(0.24, 0.42, 0.18, 0.82);
    let horizontale = dans(0.24, 0.82, 0.64, 0.82);
    if verticale || horizontale {
        // Graduations, de la couleur du fond.
        let graduation_v = [0.28, 0.38, 0.48]
            .iter()
            .any(|y| dans(0.34, 0.42, *y - 0.012, *y + 0.012));
        let graduation_h = [0.52, 0.62, 0.72]
            .iter()
            .any(|x| dans(*x - 0.012, *x + 0.012, 0.64, 0.72));
        if graduation_v || graduation_h {
            return Some(fond);
        }
        return Some(equerre);
    }
    Some(fond)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icone_de_la_bonne_taille() {
        let pixels = icone_rvba(32);
        assert_eq!(pixels.len(), 32 * 32 * 4);
        // Le coin est transparent, le centre est opaque.
        assert_eq!(pixels[3], 0);
        let centre = ((16 * 32 + 16) * 4 + 3) as usize;
        assert_eq!(pixels[centre], 255);
    }

    fn luminance(c: Color32) -> f32 {
        let canal = |x: u8| {
            let s = f32::from(x) / 255.0;
            if s <= 0.03928 {
                s / 12.92
            } else {
                ((s + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * canal(c.r()) + 0.7152 * canal(c.g()) + 0.0722 * canal(c.b())
    }

    fn contraste(a: Color32, b: Color32) -> f32 {
        let (la, lb) = (luminance(a), luminance(b));
        (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
    }

    #[test]
    fn contraste_des_libelles_de_touches() {
        // Section 16.6 : contraste texte/fond d'au moins 4,5:1, sur tout le dégradé.
        for p in [&CLAIR, &SOMBRE] {
            let theme = if p.sombre { "sombre" } else { "clair" };
            for f in [
                Famille::Fonction,
                Famille::Construction,
                Famille::Chiffre,
                Famille::Conv,
                Famille::Effacer,
            ] {
                let c = p.couleurs(f);
                assert!(contraste(c.fond, c.texte) >= 4.5, "{f:?} haut ({theme})");
                assert!(contraste(c.fond_bas, c.texte) >= 4.5, "{f:?} bas ({theme})");
            }
        }
    }

    #[test]
    fn contraste_des_textes_courants() {
        for p in [&CLAIR, &SOMBRE] {
            let theme = if p.sombre { "sombre" } else { "clair" };
            for fond in [p.ecran_fond, p.ecran_fond_bas] {
                assert!(contraste(fond, p.ecran_encre) >= 4.5, "afficheur ({theme})");
                assert!(
                    contraste(fond, p.ecran_etiquette) >= 4.5,
                    "étiquette ({theme})"
                );
                assert!(contraste(fond, p.erreur) >= 4.5, "erreur ({theme})");
                assert!(
                    contraste(fond, p.avertissement) >= 4.5,
                    "avertissement ({theme})"
                );
            }
            for fond in [p.fond_app, p.fond_app_2] {
                assert!(contraste(fond, p.texte) >= 4.5, "texte ({theme})");
                assert!(contraste(fond, p.titre) >= 4.5, "titre ({theme})");
                assert!(
                    contraste(fond, p.etiquette_secondaire) >= 4.5,
                    "étiquette secondaire ({theme})"
                );
            }
            assert!(contraste(p.carte, p.texte) >= 4.5, "carte ({theme})");
            assert!(contraste(p.puce, p.puce_texte) >= 4.5, "puce ({theme})");
            assert!(
                contraste(p.selection, p.selection_texte) >= 4.5,
                "sélection ({theme})"
            );
            for fond in [p.barre_fond, p.barre_fond_bas, p.pied_fond, p.pied_fond_bas] {
                assert!(contraste(fond, p.barre_texte) >= 4.5, "barre ({theme})");
            }
        }
    }

    #[test]
    fn melange_de_couleurs() {
        assert_eq!(
            assombrir(Color32::from_rgb(100, 100, 100), 0.5),
            Color32::from_rgb(50, 50, 50)
        );
        assert_eq!(
            eclaircir(Color32::from_rgb(55, 55, 55), 1.0),
            Color32::WHITE
        );
    }
}
