//! Liste de coupe optimisée (section 22.19) : des morceaux à couper dans des
//! longueurs offertes en stock, trait de scie compris, pour le bois, les
//! moulures, les conduits et les tuyaux.
//!
//! Le calcul cherche le plan qui achète le moins de longueur : les morceaux,
//! du plus long au plus court, vont d'abord dans la plus longue longueur
//! offerte, et chaque pièce est ensuite ramenée à la plus courte longueur qui
//! contient ses coupes; pour peu de morceaux, une recherche exacte, bornée,
//! améliore ce premier plan quand elle le peut.

use crate::analyse::{analyser_longueur, transposer};
use crate::format::separateur_decimal_en;
use crate::langue::{self, Langue};
use crate::prefs::Preferences;
use crate::unites::Format;

/// Le plus grand nombre de morceaux qu'on range (la somme des quantités).
pub const MAX_MORCEAUX: usize = 2000;

/// Au-delà de ce nombre de morceaux, le rangement décroissant suffit.
const MAX_POUR_LA_RECHERCHE: usize = 14;

/// Le nombre d'essais de la recherche exacte, au plus.
const BUDGET_DE_LA_RECHERCHE: usize = 200_000;

/// Écart toléré sur les longueurs (pouces).
const TOLERANCE: f64 = 1e-6;

/// Une pièce de stock à acheter, et ce qu'on y coupe.
#[derive(Clone, Debug, PartialEq)]
pub struct PieceDeStock {
    /// Sa longueur, en pouces.
    pub longueur: f64,
    /// Les morceaux qu'on y coupe, du plus long au plus court (pouces).
    pub coupes: Vec<f64>,
    /// Ce qui reste après la dernière coupe, traits de scie déduits (pouces).
    pub chute: f64,
}

/// Le plan de coupe : les pièces à acheter, et les morceaux qu'aucune
/// longueur offerte ne peut donner.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PlanDeCoupe {
    pub pieces: Vec<PieceDeStock>,
    /// Les morceaux trop longs, du plus long au plus court (pouces).
    pub trop_longs: Vec<f64>,
}

impl PlanDeCoupe {
    /// La longueur totale à acheter (pouces).
    pub fn longueur_achetee(&self) -> f64 {
        self.pieces.iter().map(|piece| piece.longueur).sum()
    }

    /// La chute totale (pouces).
    pub fn chute(&self) -> f64 {
        self.pieces.iter().map(|piece| piece.chute).sum()
    }

    /// Combien de pièces de chaque longueur acheter, de la plus longue à la
    /// plus courte : (longueur en pouces, nombre).
    pub fn a_acheter(&self) -> Vec<(f64, usize)> {
        let mut compte: Vec<(f64, usize)> = Vec::new();
        for piece in &self.pieces {
            match compte
                .iter_mut()
                .find(|(longueur, _)| (*longueur - piece.longueur).abs() < TOLERANCE)
            {
                Some((_, nombre)) => *nombre += 1,
                None => compte.push((piece.longueur, 1)),
            }
        }
        compte.sort_by(|a, b| b.0.total_cmp(&a.0));
        compte
    }
}

/// Une pièce en cours de remplissage.
#[derive(Clone, Debug)]
struct Barre {
    longueur: f64,
    coupes: Vec<f64>,
    somme: f64,
}

impl Barre {
    /// Vrai si le morceau y entre encore : les morceaux et un trait de scie
    /// entre chacun.
    fn recoit(&self, morceau: f64, trait_de_scie: f64) -> bool {
        self.somme + morceau + trait_de_scie * self.coupes.len() as f64 <= self.longueur + TOLERANCE
    }

    /// La longueur occupée : les morceaux et les traits de scie entre eux.
    fn occupee(&self, trait_de_scie: f64) -> f64 {
        self.somme + trait_de_scie * self.coupes.len().saturating_sub(1) as f64
    }

    fn placer(&mut self, morceau: f64) {
        self.coupes.push(morceau);
        self.somme += morceau;
    }

    fn retirer(&mut self) {
        if let Some(morceau) = self.coupes.pop() {
            self.somme -= morceau;
        }
    }
}

/// Range les morceaux — (longueur en pouces, quantité) — dans les longueurs
/// offertes (pouces), avec ce trait de scie (pouces) : le plan qui achète le
/// moins de longueur, puis le moins de pièces. Les morceaux de plus de
/// `MAX_MORCEAUX` au total ne sont pas rangés : l'appelant les refuse avant.
pub fn planifier(morceaux: &[(f64, usize)], stock: &[f64], trait_de_scie: f64) -> PlanDeCoupe {
    let trait_de_scie = if trait_de_scie.is_finite() {
        trait_de_scie.max(0.0)
    } else {
        0.0
    };
    let mut longueurs: Vec<f64> = stock
        .iter()
        .copied()
        .filter(|l| l.is_finite() && *l > TOLERANCE)
        .collect();
    longueurs.sort_by(f64::total_cmp);
    longueurs.dedup_by(|a, b| (*a - *b).abs() < TOLERANCE);
    let mut a_couper: Vec<f64> = morceaux
        .iter()
        .filter(|(longueur, _)| longueur.is_finite() && *longueur > TOLERANCE)
        .flat_map(|&(longueur, quantite)| std::iter::repeat_n(longueur, quantite))
        .take(MAX_MORCEAUX)
        .collect();
    a_couper.sort_by(|a, b| b.total_cmp(a));
    let Some(&plus_longue) = longueurs.last() else {
        return PlanDeCoupe {
            pieces: Vec::new(),
            trop_longs: a_couper,
        };
    };
    let (trop_longs, a_couper): (Vec<f64>, Vec<f64>) = a_couper
        .into_iter()
        .partition(|morceau| *morceau > plus_longue + TOLERANCE);

    let mut meilleur = ranger(&a_couper, &longueurs, trait_de_scie);
    if a_couper.len() <= MAX_POUR_LA_RECHERCHE {
        let mut recherche = Recherche {
            morceaux: &a_couper,
            longueurs: &longueurs,
            trait_de_scie,
            barres: Vec::new(),
            meilleur_cout: cout(&meilleur),
            meilleur_nombre: meilleur.len(),
            meilleures: None,
            essais: 0,
        };
        recherche.explorer(0, 0.0);
        if let Some(barres) = recherche.meilleures {
            meilleur = barres;
        }
    }

    let mut pieces: Vec<PieceDeStock> = meilleur
        .into_iter()
        .map(|mut barre| {
            barre.coupes.sort_by(|a, b| b.total_cmp(a));
            let chute = barre.longueur - barre.somme - trait_de_scie * barre.coupes.len() as f64;
            PieceDeStock {
                longueur: barre.longueur,
                coupes: barre.coupes,
                chute: if chute > TOLERANCE { chute } else { 0.0 },
            }
        })
        .collect();
    pieces.sort_by(|a, b| {
        b.longueur
            .total_cmp(&a.longueur)
            .then(a.chute.total_cmp(&b.chute))
    });
    PlanDeCoupe { pieces, trop_longs }
}

fn cout(barres: &[Barre]) -> f64 {
    barres.iter().map(|barre| barre.longueur).sum()
}

/// Le rangement décroissant : chaque morceau, du plus long au plus court, va
/// dans la pièce où il laisse le moins de place (la plus longue longueur
/// offerte), ou dans une pièce neuve; puis chaque pièce est ramenée à la plus
/// courte longueur qui contient ses coupes.
fn ranger(morceaux: &[f64], longueurs: &[f64], trait_de_scie: f64) -> Vec<Barre> {
    let Some(&plus_longue) = longueurs.last() else {
        return Vec::new();
    };
    let mut barres: Vec<Barre> = Vec::new();
    for &morceau in morceaux {
        let choisie = barres
            .iter()
            .enumerate()
            .filter(|(_, barre)| barre.recoit(morceau, trait_de_scie))
            .min_by(|(_, a), (_, b)| {
                let reste = |barre: &Barre| {
                    barre.longueur
                        - barre.somme
                        - morceau
                        - trait_de_scie * barre.coupes.len() as f64
                };
                reste(a).total_cmp(&reste(b))
            })
            .map(|(rang, _)| rang);
        match choisie {
            Some(rang) => barres[rang].placer(morceau),
            None => {
                let mut barre = Barre {
                    longueur: plus_longue,
                    coupes: Vec::new(),
                    somme: 0.0,
                };
                barre.placer(morceau);
                barres.push(barre);
            }
        }
    }
    for barre in &mut barres {
        let occupee = barre.occupee(trait_de_scie);
        if let Some(&courte) = longueurs.iter().find(|l| **l + TOLERANCE >= occupee) {
            barre.longueur = courte;
        }
    }
    barres
}

/// La recherche exacte, pour peu de morceaux : chaque morceau, du plus long au
/// plus court, va dans une pièce déjà ouverte ou dans une pièce neuve de
/// chaque longueur offerte; une branche qui ne peut pas faire mieux que le
/// meilleur plan connu est abandonnée.
struct Recherche<'a> {
    morceaux: &'a [f64],
    longueurs: &'a [f64],
    trait_de_scie: f64,
    barres: Vec<Barre>,
    meilleur_cout: f64,
    meilleur_nombre: usize,
    meilleures: Option<Vec<Barre>>,
    essais: usize,
}

impl Recherche<'_> {
    fn explorer(&mut self, rang: usize, cout_actuel: f64) {
        self.essais += 1;
        if self.essais > BUDGET_DE_LA_RECHERCHE {
            return;
        }
        if rang == self.morceaux.len() {
            let mieux = cout_actuel < self.meilleur_cout - TOLERANCE
                || (cout_actuel < self.meilleur_cout + TOLERANCE
                    && self.barres.len() < self.meilleur_nombre);
            if mieux {
                self.meilleur_cout = cout_actuel;
                self.meilleur_nombre = self.barres.len();
                self.meilleures = Some(self.barres.clone());
            }
            return;
        }
        // Une borne : la place qui manque encore, au mieux payée au prix de la
        // plus courte longueur offerte (trait de scie compris).
        let k = self.trait_de_scie;
        let restant: f64 = self.morceaux[rang..].iter().map(|m| m + k).sum();
        let libre: f64 = self
            .barres
            .iter()
            .map(|barre| barre.longueur + k - barre.somme - k * barre.coupes.len() as f64)
            .sum();
        let plus_courte = self.longueurs[0];
        let borne = (restant - libre).max(0.0) * plus_courte / (plus_courte + k);
        if cout_actuel + borne >= self.meilleur_cout - TOLERANCE
            && !(cout_actuel + borne < self.meilleur_cout + TOLERANCE
                && self.barres.len() + 1 < self.meilleur_nombre)
        {
            return;
        }
        let morceau = self.morceaux[rang];
        // Dans une pièce déjà ouverte; deux pièces pareilles ne s'essaient qu'une fois.
        let mut essayees: Vec<(f64, f64, usize)> = Vec::new();
        for i in 0..self.barres.len() {
            let barre = &self.barres[i];
            let etat = (barre.longueur, barre.somme, barre.coupes.len());
            if !barre.recoit(morceau, k)
                || essayees.iter().any(|e| {
                    (e.0 - etat.0).abs() < TOLERANCE
                        && (e.1 - etat.1).abs() < TOLERANCE
                        && e.2 == etat.2
                })
            {
                continue;
            }
            essayees.push(etat);
            self.barres[i].placer(morceau);
            self.explorer(rang + 1, cout_actuel);
            self.barres[i].retirer();
        }
        // Dans une pièce neuve, de chaque longueur qui le reçoit.
        for &longueur in self.longueurs {
            if morceau > longueur + TOLERANCE {
                continue;
            }
            let mut barre = Barre {
                longueur,
                coupes: Vec::new(),
                somme: 0.0,
            };
            barre.placer(morceau);
            self.barres.push(barre);
            self.explorer(rang + 1, cout_actuel + longueur);
            self.barres.pop();
        }
    }
}

/// Les longueurs d'une liste, telles qu'écrites : séparées par des
/// points-virgules, des virgules suivies d'une espace, ou, en anglais, des
/// virgules (en français, une virgule sans espace est décimale).
fn decouper(texte: &str, langue: Langue) -> Vec<String> {
    let anglais = langue == Langue::Anglais;
    texte
        .replace(", ", ";")
        .split(|c| c == ';' || (anglais && c == ','))
        .map(str::trim)
        .filter(|partie| !partie.is_empty())
        .map(str::to_string)
        .collect()
}

/// Lit les longueurs offertes : « 8; 10; 12; 16 » (un nombre seul prend le
/// format `defaut`). En cas d'erreur, le rang de la longueur illisible.
pub fn lire_les_longueurs(texte: &str, defaut: Format) -> Result<Vec<f64>, usize> {
    decouper(texte, langue::en_cours())
        .iter()
        .enumerate()
        .map(|(rang, partie)| {
            analyser_longueur(partie, defaut)
                .ok()
                .map(|valeur| valeur.base)
                .filter(|pouces| pouces.is_finite() && *pouces > 0.0)
                .ok_or(rang)
        })
        .collect()
}

/// Récrit une longueur, ou une liste de longueurs, comme elle se tape dans
/// l'autre langue (section 24) : « 8; 10,5 » devient « 8; 10.5 ». Ce qui ne
/// se lit pas reste tel quel.
pub fn transposer_les_longueurs(
    texte: &str,
    de: Langue,
    vers: Langue,
    prefs: &Preferences,
) -> String {
    if de == vers {
        return texte.to_string();
    }
    let decimal = separateur_decimal_en(prefs, vers);
    let parties: Vec<String> = decouper(texte, de)
        .iter()
        .map(|partie| transposer(partie, de, vers, decimal))
        .collect();
    if parties.len() <= 1 {
        parties.into_iter().next().unwrap_or_default()
    } else {
        parties.join("; ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Le meilleur coût, par énumération de toutes les façons de grouper les
    /// morceaux (pour peu de morceaux) : chaque groupe prend la plus courte
    /// longueur qui le contient.
    fn meilleur_cout_par_enumeration(morceaux: &[f64], longueurs: &[f64], k: f64) -> f64 {
        fn groupes(reste: &[f64], courant: &mut Vec<Vec<f64>>, tous: &mut Vec<Vec<Vec<f64>>>) {
            let Some((&premier, suite)) = reste.split_first() else {
                tous.push(courant.clone());
                return;
            };
            for i in 0..courant.len() {
                courant[i].push(premier);
                groupes(suite, courant, tous);
                courant[i].pop();
            }
            courant.push(vec![premier]);
            groupes(suite, courant, tous);
            courant.pop();
        }
        let mut tous = Vec::new();
        groupes(morceaux, &mut Vec::new(), &mut tous);
        tous.iter()
            .filter_map(|partition| {
                partition
                    .iter()
                    .map(|groupe| {
                        let occupee: f64 =
                            groupe.iter().sum::<f64>() + k * (groupe.len() - 1) as f64;
                        longueurs
                            .iter()
                            .copied()
                            .find(|l| *l + TOLERANCE >= occupee)
                    })
                    .sum::<Option<f64>>()
            })
            .fold(f64::INFINITY, f64::min)
    }

    /// Chaque pièce du plan reçoit ses coupes, traits de scie compris, et
    /// tous les morceaux y sont.
    fn verifier(plan: &PlanDeCoupe, morceaux: &[f64], k: f64) {
        let mut coupes: Vec<f64> = plan.pieces.iter().flat_map(|p| p.coupes.clone()).collect();
        coupes.extend(plan.trop_longs.iter().copied());
        coupes.sort_by(f64::total_cmp);
        let mut attendus = morceaux.to_vec();
        attendus.sort_by(f64::total_cmp);
        assert_eq!(coupes, attendus);
        for piece in &plan.pieces {
            let occupee: f64 =
                piece.coupes.iter().sum::<f64>() + k * (piece.coupes.len() - 1) as f64;
            assert!(occupee <= piece.longueur + TOLERANCE, "{piece:?}");
            assert!(piece.chute >= 0.0);
        }
    }

    #[test]
    fn deux_morceaux_de_quatre_pieds_ne_sortent_pas_d_un_huit_pieds() {
        // Quatre morceaux de 4 pi, trait de 1/8 po : deux par 10 pi.
        let plan = planifier(&[(48.0, 4)], &[96.0, 120.0, 144.0, 192.0], 0.125);
        assert_eq!(plan.a_acheter(), [(120.0, 2)]);
        assert!((plan.longueur_achetee() - 240.0).abs() < 1e-9);
        assert!((plan.pieces[0].chute - (120.0 - 96.0 - 0.25)).abs() < 1e-9);
        // Sans trait de scie, deux par 8 pi, sans chute.
        let plan = planifier(&[(48.0, 4)], &[96.0, 120.0], 0.0);
        assert_eq!(plan.a_acheter(), [(96.0, 2)]);
        assert_eq!(plan.chute(), 0.0);
    }

    #[test]
    fn le_plan_est_le_meilleur_pour_peu_de_morceaux() {
        // Des cas tirés d'un générateur simple, toujours le même.
        let mut graine: u64 = 7;
        let mut tirer = |n: u64| {
            graine = graine
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            (graine >> 33) % n
        };
        let longueurs = [96.0, 120.0, 144.0, 192.0];
        for _ in 0..40 {
            let nombre = 2 + tirer(6) as usize;
            let morceaux: Vec<f64> = (0..nombre).map(|_| 12.0 + 6.0 * tirer(25) as f64).collect();
            let entree: Vec<(f64, usize)> = morceaux.iter().map(|m| (*m, 1)).collect();
            let plan = planifier(&entree, &longueurs, 0.125);
            verifier(&plan, &morceaux, 0.125);
            let attendu = meilleur_cout_par_enumeration(&morceaux, &longueurs, 0.125);
            assert!(
                (plan.longueur_achetee() - attendu).abs() < 1e-6,
                "{morceaux:?} : {} au lieu de {attendu}",
                plan.longueur_achetee()
            );
        }
    }

    #[test]
    fn beaucoup_de_morceaux_et_morceaux_trop_longs() {
        let morceaux = [(30.0, 40), (54.5, 25), (90.0, 10), (250.0, 2)];
        let plan = planifier(&morceaux, &[96.0, 144.0, 192.0], 0.125);
        let tous: Vec<f64> = morceaux
            .iter()
            .flat_map(|&(l, q)| std::iter::repeat_n(l, q))
            .collect();
        verifier(&plan, &tous, 0.125);
        assert_eq!(plan.trop_longs, [250.0, 250.0]);
        // Rien de mieux que la longueur des morceaux, ni de pire que le double.
        let somme: f64 = tous.iter().filter(|m| **m < 200.0).sum();
        assert!(plan.longueur_achetee() >= somme);
        assert!(plan.longueur_achetee() < 2.0 * somme);
        // Sans longueur offerte, tout est trop long.
        let vide = planifier(&[(30.0, 2)], &[], 0.125);
        assert!(vide.pieces.is_empty());
        assert_eq!(vide.trop_longs.len(), 2);
    }

    #[test]
    fn lire_les_longueurs_offertes() {
        let lues = lire_les_longueurs("8; 10, 12 pi;16 pi 6 po", Format::PiPo).unwrap();
        assert_eq!(lues, [96.0, 120.0, 144.0, 198.0]);
        assert_eq!(lire_les_longueurs("8; dix", Format::PiPo), Err(1));
        assert_eq!(lire_les_longueurs("", Format::PiPo), Ok(Vec::new()));
        // En français, la virgule sans espace est décimale.
        assert_eq!(lire_les_longueurs("8,5", Format::PiPo), Ok(vec![102.0]));
        // En anglais, la virgule sépare.
        {
            let _anglais = langue::pour_la_portee(Langue::Anglais);
            assert_eq!(
                lire_les_longueurs("8,10", Format::PiPo),
                Ok(vec![96.0, 120.0])
            );
        }
        // D'une langue à l'autre, les valeurs ne changent pas.
        let prefs = Preferences::default();
        let anglais = transposer_les_longueurs(
            "8; 10,5; 12 pi 6 po",
            Langue::Francais,
            Langue::Anglais,
            &prefs,
        );
        assert_eq!(anglais, "8; 10.5; 12 ft 6 in");
        assert_eq!(
            transposer_les_longueurs(&anglais, Langue::Anglais, Langue::Francais, &prefs),
            "8; 10,5; 12 pi 6 po"
        );
        assert_eq!(
            transposer_les_longueurs("1/8", Langue::Francais, Langue::Anglais, &prefs),
            "1/8"
        );
    }
}
