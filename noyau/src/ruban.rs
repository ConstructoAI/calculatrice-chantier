//! Ruban de calcul (section 14).

use serde::{Deserialize, Serialize};

/// Nombre maximal de lignes du ruban courant (annexe A.2).
pub const MAX_LIGNES: usize = 500;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum GenreLigne {
    #[default]
    Operande,
    SousTotal,
    Total,
    Sortie,
    Conversion,
    Memoire,
    Separateur,
}

/// Une ligne du ruban.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct LigneRuban {
    pub genre: GenreLigne,
    /// Signe devant l'opérande : + − × ÷ = →.
    pub operateur: Option<char>,
    /// Étiquette d'une sortie de fonction.
    pub etiquette: Option<String>,
    /// Valeur formatée.
    pub texte: String,
}

impl LigneRuban {
    /// Signe montré en tête de ligne.
    pub fn signe(&self) -> Option<char> {
        match self.genre {
            GenreLigne::Total | GenreLigne::SousTotal => Some('='),
            GenreLigne::Conversion => Some('→'),
            _ => self.operateur,
        }
    }
}

/// Ruban de calcul : la trace des saisies et des résultats.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Ruban {
    pub lignes: Vec<LigneRuban>,
}

impl Ruban {
    pub fn est_vide(&self) -> bool {
        self.lignes.is_empty()
    }

    pub fn vider(&mut self) {
        self.lignes.clear();
    }

    pub fn ajouter(&mut self, ligne: LigneRuban) {
        self.lignes.push(ligne);
        if self.lignes.len() > MAX_LIGNES {
            let surplus = self.lignes.len() - MAX_LIGNES;
            self.lignes.drain(..surplus);
        }
    }

    pub fn operande(&mut self, operateur: Option<char>, texte: String) {
        self.ajouter(LigneRuban {
            genre: GenreLigne::Operande,
            operateur,
            etiquette: None,
            texte,
        });
    }

    pub fn total(&mut self, texte: String) {
        self.ajouter(LigneRuban {
            genre: GenreLigne::Total,
            operateur: Some('='),
            etiquette: None,
            texte,
        });
    }

    pub fn sortie(&mut self, etiquette: String, texte: String) {
        self.ajouter(LigneRuban {
            genre: GenreLigne::Sortie,
            operateur: None,
            etiquette: Some(etiquette),
            texte,
        });
    }

    pub fn conversion(&mut self, texte: String) {
        self.ajouter(LigneRuban {
            genre: GenreLigne::Conversion,
            operateur: Some('→'),
            etiquette: None,
            texte,
        });
    }

    pub fn memoire(&mut self, etiquette: String, texte: String) {
        self.ajouter(LigneRuban {
            genre: GenreLigne::Memoire,
            operateur: None,
            etiquette: Some(etiquette),
            texte,
        });
    }

    /// Ajoute un filet de séparation, sauf en tête ou à la suite d'un autre filet.
    pub fn separateur(&mut self) {
        match self.lignes.last() {
            None => {}
            Some(l) if l.genre == GenreLigne::Separateur => {}
            Some(_) => self.ajouter(LigneRuban {
                genre: GenreLigne::Separateur,
                ..LigneRuban::default()
            }),
        }
    }

    /// Texte à copier ou à exporter (section 14.3) : un titre, un filet, puis
    /// les lignes, valeurs alignées à droite.
    pub fn en_texte(&self, titre: &str) -> String {
        exporter(titre, &self.lignes)
    }
}

/// Met en forme des lignes de ruban en texte brut.
pub fn exporter(titre: &str, lignes: &[LigneRuban]) -> String {
    let gauche = |l: &LigneRuban| -> String {
        let mut g = String::new();
        if let Some(s) = l.signe() {
            g.push(s);
        }
        if let Some(e) = &l.etiquette {
            if !g.is_empty() {
                g.push(' ');
            }
            g.push_str(e);
        }
        g
    };
    let largeur_gauche = lignes
        .iter()
        .map(|l| gauche(l).chars().count())
        .max()
        .unwrap_or(0);
    let largeur_droite = lignes
        .iter()
        .map(|l| l.texte.chars().count())
        .max()
        .unwrap_or(0);
    let largeur = (largeur_gauche + 2 + largeur_droite).max(titre.chars().count());
    let filet: String = "-".repeat(largeur);
    let mut sortie = String::new();
    sortie.push_str(titre);
    sortie.push('\n');
    sortie.push_str(&filet);
    sortie.push('\n');
    for l in lignes {
        if l.genre == GenreLigne::Separateur {
            sortie.push_str(&filet);
            sortie.push('\n');
            continue;
        }
        let g = gauche(l);
        let remplissage = largeur.saturating_sub(g.chars().count() + l.texte.chars().count());
        sortie.push_str(&g);
        sortie.push_str(&" ".repeat(remplissage));
        sortie.push_str(&l.texte);
        sortie.push('\n');
    }
    sortie
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_ruban_garde_les_dernieres_lignes() {
        let mut r = Ruban::default();
        for i in 0..(MAX_LIGNES + 25) {
            r.operande(None, i.to_string());
        }
        assert_eq!(r.lignes.len(), MAX_LIGNES);
        assert_eq!(r.lignes[0].texte, "25");
    }

    #[test]
    fn export_aligne_a_droite() {
        let mut r = Ruban::default();
        r.operande(None, "5 pi 4 1/2 po".into());
        r.operande(Some('+'), "8 po".into());
        r.total("16 pi 6 1/2 po".into());
        r.separateur();
        r.separateur();
        r.sortie("COURSE".into(), "7 pi 2 po".into());
        let texte = r.en_texte("Ruban");
        let lignes: Vec<&str> = texte.lines().collect();
        assert_eq!(lignes[0], "Ruban");
        assert!(lignes[1].chars().all(|c| c == '-'));
        assert!(lignes[2].ends_with("5 pi 4 1/2 po"));
        assert!(lignes[3].starts_with('+') && lignes[3].ends_with("8 po"));
        assert!(lignes[4].starts_with('=') && lignes[4].ends_with("16 pi 6 1/2 po"));
        assert_eq!(lignes[5], lignes[1]);
        assert!(lignes[6].starts_with("COURSE") && lignes[6].ends_with("7 pi 2 po"));
        let largeur = lignes[2].chars().count();
        assert!(lignes[2..].iter().all(|l| l.chars().count() == largeur));
    }
}
