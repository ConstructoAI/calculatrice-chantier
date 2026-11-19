//! Matériaux : blocs, semelle, gypse, montants, coût (sections 12.12 à 12.17).

use super::Sortie;
use crate::erreurs::Erreur;
use crate::etiquettes::Etiquette;
use crate::format::Rendu;
use crate::prefs::Preferences;
use crate::unites::{EPS, Format, Genre, PO_PAR_PI, PO2_PAR_PI2, Systeme};
use crate::valeur::Valeur;

/// Donnée d'un calcul de matériaux : une longueur (calcul linéaire) ou une aire.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DonneeMateriau {
    Lineaire(Valeur),
    Surface(Valeur),
}

impl DonneeMateriau {
    /// Contrôle le genre de la donnée et retourne sa mesure en unité de base.
    fn mesure(&self) -> Result<f64, Erreur> {
        match self {
            DonneeMateriau::Lineaire(l) if l.genre == Genre::Longueur => positive(l),
            DonneeMateriau::Surface(a) if a.genre == Genre::Aire => positive(a),
            _ => Err(Erreur::Unites),
        }
    }
}

fn positive(v: &Valeur) -> Result<f64, Erreur> {
    if v.base.is_finite() && v.base > 0.0 {
        Ok(v.base)
    } else {
        Err(Erreur::Saisie)
    }
}

fn quantite(etiquette: Etiquette, x: f64) -> Result<Sortie, Erreur> {
    if !x.is_finite() {
        return Err(Erreur::Depassement);
    }
    Ok(Sortie::nouvelle(etiquette, Valeur::nombre(x)).rendue(Rendu::Flottant))
}

/// Blocs et briques (12.12) : nombre de blocs, puis la taille mémorisée utilisée.
pub fn calculer_blocs(
    donnee: &DonneeMateriau,
    longueur_bloc: &Valeur,
    aire_bloc: &Valeur,
) -> Result<Vec<Sortie>, Erreur> {
    let mesure = donnee.mesure()?;
    match donnee {
        DonneeMateriau::Lineaire(_) => Ok(vec![
            quantite(Etiquette::Blocs, mesure / positive(longueur_bloc)?)?,
            Sortie::nouvelle(Etiquette::LongueurBloc, *longueur_bloc).memorisee(),
        ]),
        DonneeMateriau::Surface(_) => Ok(vec![
            quantite(Etiquette::Blocs, mesure / positive(aire_bloc)?)?,
            Sortie::nouvelle(Etiquette::AireBloc, *aire_bloc).memorisee(),
        ]),
    }
}

/// Semelle (12.13) : volume de béton, puis la section mémorisée.
pub fn calculer_semelle(
    longueur: &Valeur,
    section: &Valeur,
    prefs: &Preferences,
) -> Result<Vec<Sortie>, Erreur> {
    if longueur.genre != Genre::Longueur {
        return Err(Erreur::Unites);
    }
    let volume = positive(longueur)? * positive(section)?;
    if !volume.is_finite() {
        return Err(Erreur::Depassement);
    }
    let format = prefs.format_volume_impose().unwrap_or({
        if longueur.format.systeme() == Some(Systeme::Metrique) {
            Format::M3
        } else {
            Format::Vg3
        }
    });
    Ok(vec![
        Sortie::nouvelle(Etiquette::Semelle, Valeur::en_base(volume, format)),
        Sortie::nouvelle(Etiquette::SectionSemelle, *section).memorisee(),
    ])
}

/// Gypse (12.14) : feuilles de 4 × 8, 4 × 9, 4 × 10 et 4 × 12, puis la donnée.
pub fn calculer_gypse(donnee: &DonneeMateriau) -> Result<Vec<Sortie>, Erreur> {
    const FORMATS: [(Etiquette, f64); 4] = [
        (Etiquette::Feuilles4x8, 32.0),
        (Etiquette::Feuilles4x9, 36.0),
        (Etiquette::Feuilles4x10, 40.0),
        (Etiquette::Feuilles4x12, 48.0),
    ];
    /// Largeur d'une feuille, en pieds.
    const LARGEUR_FEUILLE: f64 = 4.0;
    let mut sorties = Vec::with_capacity(5);
    let mesure = donnee.mesure()?;
    match donnee {
        DonneeMateriau::Surface(a) => {
            let pi2 = mesure / PO2_PAR_PI2;
            for (etiquette, aire_feuille) in FORMATS {
                sorties.push(quantite(etiquette, pi2 / aire_feuille)?);
            }
            sorties.push(Sortie::nouvelle(Etiquette::Aire, *a));
        }
        DonneeMateriau::Lineaire(l) => {
            // Feuilles posées debout : une feuille par 4 pieds de mur.
            let feuilles = mesure / (LARGEUR_FEUILLE * PO_PAR_PI);
            for (etiquette, _) in FORMATS {
                sorties.push(quantite(etiquette, feuilles)?);
            }
            sorties.push(Sortie::nouvelle(Etiquette::Longueur, *l));
        }
    }
    Ok(sorties)
}

/// Montants (12.15) : nombre de montants d'un mur, montant de départ compris.
pub fn calculer_montants(longueur: &Valeur, entraxe: &Valeur) -> Result<Sortie, Erreur> {
    if longueur.genre != Genre::Longueur {
        return Err(Erreur::Unites);
    }
    let n = (positive(longueur)? / positive(entraxe)? - EPS).ceil() + 1.0;
    if !n.is_finite() {
        return Err(Erreur::Depassement);
    }
    Ok(Sortie::entier(Etiquette::Montants, n))
}

/// Coût (12.17) : quantité dans son unité d'affichage × prix unitaire.
pub fn calculer_cout(quantite: &Valeur, prix: &Valeur) -> Result<Sortie, Erreur> {
    if !prix.est_sans_unite() {
        return Err(Erreur::Saisie);
    }
    let unites = match quantite.format {
        // Le bois se vend au millier de pmp.
        Format::Pmp => quantite.numerique() / 1000.0,
        _ => quantite.numerique(),
    };
    let cout = unites * prix.base;
    if !cout.is_finite() {
        return Err(Erreur::Depassement);
    }
    Ok(Sortie::nouvelle(
        Etiquette::Cout,
        Valeur {
            base: cout,
            genre: Genre::SansUnite,
            format: Format::Monnaie,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pi_po(po: f64) -> Valeur {
        Valeur::en_base(po, Format::PiPo)
    }

    #[test]
    fn blocs() {
        let lb = Valeur::en_base(16.0, Format::PoFrac);
        let ab = Valeur::en_base(128.0, Format::Po2);
        let s = calculer_blocs(&DonneeMateriau::Lineaire(pi_po(240.0)), &lb, &ab).unwrap();
        assert_eq!(s[0].valeur.base, 15.0);
        assert_eq!(s[1].etiquette, Etiquette::LongueurBloc);
        let aire = Valeur::avec_format(300.0, Format::Pi2);
        let s = calculer_blocs(&DonneeMateriau::Surface(aire), &lb, &ab).unwrap();
        assert_eq!(s[0].valeur.base, 337.5);
        assert!(s[1].memorise && s[1].etiquette == Etiquette::AireBloc);
    }

    #[test]
    fn semelle() {
        let p = Preferences::default();
        let section = Valeur::en_base(264.0, Format::Po2);
        let s = calculer_semelle(&pi_po(1200.0), &section, &p).unwrap();
        assert!((s[0].valeur.numerique() - 6.790_123).abs() < 1e-6);
        assert_eq!(s[0].valeur.format, Format::Vg3);
        let s = calculer_semelle(&Valeur::avec_format(30.0, Format::M), &section, &p).unwrap();
        assert_eq!(s[0].valeur.format, Format::M3);
    }

    #[test]
    fn gypse() {
        let aire = Valeur::avec_format(150.0, Format::Pi2);
        let s = calculer_gypse(&DonneeMateriau::Surface(aire)).unwrap();
        let n: Vec<f64> = s[..4].iter().map(|x| x.valeur.base).collect();
        assert!((n[0] - 4.6875).abs() < 1e-12 && (n[1] - 150.0 / 36.0).abs() < 1e-12);
        assert!((n[2] - 3.75).abs() < 1e-12 && (n[3] - 3.125).abs() < 1e-12);
        assert_eq!(s[4].etiquette, Etiquette::Aire);
        let s = calculer_gypse(&DonneeMateriau::Lineaire(pi_po(480.0))).unwrap();
        assert_eq!(s[0].valeur.base, 10.0);
        assert_eq!(s[4].etiquette, Etiquette::Longueur);
    }

    #[test]
    fn montants() {
        let oc = Valeur::en_base(16.0, Format::PoFrac);
        assert_eq!(
            calculer_montants(&pi_po(223.5), &oc).unwrap().valeur.base,
            15.0
        );
        assert_eq!(
            calculer_montants(&pi_po(192.0), &oc).unwrap().valeur.base,
            13.0
        );
        let oc24 = Valeur::en_base(24.0, Format::PoFrac);
        assert_eq!(
            calculer_montants(&pi_po(144.0), &oc24).unwrap().valeur.base,
            7.0
        );
    }

    #[test]
    fn cout() {
        let volume = Valeur::avec_format(5.146_605, Format::Vg3);
        let c = calculer_cout(&volume, &Valeur::nombre(55.0)).unwrap();
        assert!((c.valeur.base - 283.063).abs() < 1e-3);
        let pmp = Valeur::avec_format(72.0, Format::Pmp);
        assert!(
            (calculer_cout(&pmp, &Valeur::nombre(250.0))
                .unwrap()
                .valeur
                .base
                - 18.0)
                .abs()
                < 1e-9
        );
        let longueur = pi_po(150.0);
        assert!(
            (calculer_cout(&longueur, &Valeur::nombre(3.5))
                .unwrap()
                .valeur
                .base
                - 43.75)
                .abs()
                < 1e-9
        );
        assert_eq!(
            calculer_cout(&longueur, &pi_po(12.0)).map(|_| ()),
            Err(Erreur::Saisie)
        );
    }
}
