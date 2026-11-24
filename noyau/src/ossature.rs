//! L'ossature dessinée (évolution 28 de la section 27) : les montants aux 16
//! ou 24 po dans les murs et les cloisons, doublés aux coins et autour des
//! ouvertures, leur compte par mur, et les linteaux à leur longueur. Aucune
//! table de portée : ni la section des linteaux, ni celle des montants.

use serde::{Deserialize, Serialize};

use crate::dessin::{Dessin, Solution};

/// L'épaisseur d'un montant (un 2 × 4 ou un 2 × 6 vu de côté), en pouces.
pub const EPAISSEUR_D_UN_MONTANT: f64 = 1.5;

/// L'entraxe des montants.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Entraxe {
    #[default]
    Seize,
    VingtQuatre,
}

impl Entraxe {
    /// L'entraxe, en pouces.
    pub fn pouces(self) -> f64 {
        match self {
            Entraxe::Seize => 16.0,
            Entraxe::VingtQuatre => 24.0,
        }
    }
}

/// D'où vient un mur d'ossature : un mur de la pièce (son côté), ou un
/// segment d'une cloison (la cloison, et son segment).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MurDOssature {
    Mur(usize),
    Cloison(usize, usize),
}

/// L'ossature d'un mur : sa ligne (de `a` à `b`), sa bande (de `de` à `a_`
/// pouces le long de la normale (−y, x) de la ligne), la position du milieu
/// de chacun de ses montants et les linteaux de ses ouvertures (depuis `a`).
#[derive(Clone, Debug, PartialEq)]
pub struct OssatureDUnMur {
    pub mur: MurDOssature,
    pub a: (f64, f64),
    pub b: (f64, f64),
    pub bande: (f64, f64),
    pub montants: Vec<f64>,
    pub linteaux: Vec<(f64, f64)>,
}

impl OssatureDUnMur {
    /// La longueur du mur.
    pub fn longueur(&self) -> f64 {
        (self.b.0 - self.a.0).hypot(self.b.1 - self.a.1)
    }

    /// Le rectangle de chaque montant, sur le plan (pouces).
    pub fn rectangles_des_montants(&self) -> Vec<[(f64, f64); 4]> {
        let longueur = self.longueur();
        if longueur < 1e-9 {
            return Vec::new();
        }
        let u = (
            (self.b.0 - self.a.0) / longueur,
            (self.b.1 - self.a.1) / longueur,
        );
        let n = (-u.1, u.0);
        let point = |t: f64, k: f64| (self.a.0 + u.0 * t + n.0 * k, self.a.1 + u.1 * t + n.1 * k);
        let demi = EPAISSEUR_D_UN_MONTANT / 2.0;
        self.montants
            .iter()
            .map(|&t| {
                [
                    point(t - demi, self.bande.0),
                    point(t + demi, self.bande.0),
                    point(t + demi, self.bande.1),
                    point(t - demi, self.bande.1),
                ]
            })
            .collect()
    }

    /// Chaque linteau, sur le plan : un trait au milieu de la bande.
    pub fn traits_des_linteaux(&self) -> Vec<((f64, f64), (f64, f64))> {
        let longueur = self.longueur();
        if longueur < 1e-9 {
            return Vec::new();
        }
        let u = (
            (self.b.0 - self.a.0) / longueur,
            (self.b.1 - self.a.1) / longueur,
        );
        let n = (-u.1, u.0);
        let k = (self.bande.0 + self.bande.1) / 2.0;
        self.linteaux
            .iter()
            .map(|&(de, a)| {
                (
                    (self.a.0 + u.0 * de + n.0 * k, self.a.1 + u.1 * de + n.1 * k),
                    (self.a.0 + u.0 * a + n.0 * k, self.a.1 + u.1 * a + n.1 * k),
                )
            })
            .collect()
    }
}

/// Les montants d'un mur de `longueur` pouces, à l'entraxe donné, et ses
/// linteaux : deux montants à chaque bout (le coin), un à chaque entraxe
/// depuis le début, deux de chaque côté d'une ouverture (le montant de rive et
/// le montant nain du linteau), aucun dans l'ouverture; un linteau dépasse
/// l'ouverture d'un montant de chaque côté.
pub fn montants(
    longueur: f64,
    entraxe: f64,
    ouvertures: &[(f64, f64)],
) -> (Vec<f64>, Vec<(f64, f64)>) {
    let e = EPAISSEUR_D_UN_MONTANT;
    if longueur < e {
        return (Vec::new(), Vec::new());
    }
    let mut montants = vec![e / 2.0, longueur - e / 2.0];
    if longueur >= 4.0 * e {
        montants.push(e * 1.5);
        montants.push(longueur - e * 1.5);
    }
    let mut t = entraxe;
    while t < longueur - e {
        let dans_une_ouverture = ouvertures.iter().any(|&(de, a)| t > de - e && t < a + e);
        if !dans_une_ouverture {
            montants.push(t);
        }
        t += entraxe;
    }
    let mut linteaux = Vec::new();
    for &(de, a) in ouvertures {
        let (de, a) = (de.max(0.0), a.min(longueur));
        montants.extend([de - e / 2.0, de - e * 1.5, a + e / 2.0, a + e * 1.5]);
        linteaux.push(((de - e).max(0.0), (a + e).min(longueur)));
    }
    // Dans le mur, et un seul montant là où deux se chevauchent.
    let mut montants: Vec<f64> = montants
        .into_iter()
        .map(|t| t.clamp(e / 2.0, longueur - e / 2.0))
        .collect();
    montants.sort_by(f64::total_cmp);
    montants.dedup_by(|a, b| (*a - *b).abs() < e - 1e-6);
    (montants, linteaux)
}

/// L'ossature de la pièce complète : ses murs, puis chaque segment de ses
/// cloisons.
pub fn ossature(dessin: &Dessin, solution: &Solution) -> Vec<OssatureDUnMur> {
    let entraxe = dessin.entraxe.pouces();
    let mut murs = Vec::new();
    if let Some(sommets) = dessin.sommets_mesures(solution)
        && sommets.len() > 2
        && matches!(solution.etat, crate::dessin::Etat::Complete { .. })
    {
        let aire: f64 = sommets
            .windows(2)
            .map(|s| s[0].0 * s[1].1 - s[1].0 * s[0].1)
            .sum();
        let e = solution.epaisseur_des_murs;
        // La normale (−y, x) pointe vers le dedans si l'aire est positive. Le
        // mur est au dehors de mesures intérieures, au dedans d'extérieures.
        let vers_dedans = if aire > 0.0 { 1.0 } else { -1.0 };
        let bande = if dessin.mesures_exterieures {
            if vers_dedans > 0.0 {
                (0.0, e)
            } else {
                (-e, 0.0)
            }
        } else if vers_dedans > 0.0 {
            (-e, 0.0)
        } else {
            (0.0, e)
        };
        for cote in 0..dessin.cotes.len() {
            let (Some(&a), Some(&b)) =
                (sommets.get(cote), sommets.get(cote + 1).or(sommets.first()))
            else {
                continue;
            };
            let longueur = (b.0 - a.0).hypot(b.1 - a.1);
            let ouvertures: Vec<(f64, f64)> = solution
                .placements
                .iter()
                .flatten()
                .filter(|placement| placement.cote == cote)
                .map(|placement| (placement.debut, placement.fin))
                .collect();
            let (montants, linteaux) = montants(longueur, entraxe, &ouvertures);
            murs.push(OssatureDUnMur {
                mur: MurDOssature::Mur(cote),
                a,
                b,
                bande,
                montants,
                linteaux,
            });
        }
    }
    for (rang, cloison) in solution.cloisons.iter().enumerate() {
        let Some(cloison) = cloison else {
            continue;
        };
        let demi = cloison.epaisseur / 2.0;
        for (segment, paire) in cloison.sommets.windows(2).enumerate() {
            let longueur = (paire[1].0 - paire[0].0).hypot(paire[1].1 - paire[0].1);
            let ouvertures: Vec<(f64, f64)> = cloison
                .ouvertures
                .iter()
                .filter(|ouverture| ouverture.segment == segment)
                .map(|ouverture| (ouverture.debut, ouverture.fin))
                .collect();
            let (montants, linteaux) = montants(longueur, entraxe, &ouvertures);
            murs.push(OssatureDUnMur {
                mur: MurDOssature::Cloison(rang, segment),
                a: paire[0],
                b: paire[1],
                bande: (-demi, demi),
                montants,
                linteaux,
            });
        }
    }
    murs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn les_montants_d_un_mur() {
        // Un mur de 12 pi aux 16 po : deux montants à chaque coin, et huit
        // entre les deux (16, 32… 128 po).
        let (liste, linteaux) = montants(144.0, 16.0, &[]);
        assert_eq!(liste.len(), 2 + 8 + 2);
        assert!(linteaux.is_empty());
        assert!(liste.contains(&16.0) && liste.contains(&128.0));
        // Aux 24 po : cinq entre les coins.
        assert_eq!(montants_seuls(144.0, 24.0), 2 + 5 + 2);
        // Une porte de 32 po à 1 pi du coin : les montants de 16 et 32 po
        // tombent, deux montants de chaque côté s'ajoutent; le linteau la
        // dépasse d'un montant de chaque côté.
        let (liste, linteaux) = montants(144.0, 16.0, &[(12.0, 44.0)]);
        assert!(!liste.iter().any(|&t| t > 12.0 && t < 44.0));
        assert!(liste.contains(&11.25) && liste.contains(&9.75));
        assert!(liste.contains(&44.75) && liste.contains(&46.25));
        assert_eq!(linteaux, vec![(10.5, 45.5)]);
        // Aucun montant ne se chevauche.
        for paire in liste.windows(2) {
            assert!(paire[1] - paire[0] >= EPAISSEUR_D_UN_MONTANT - 1e-6);
        }
    }

    fn montants_seuls(longueur: f64, entraxe: f64) -> usize {
        montants(longueur, entraxe, &[]).0.len()
    }
}
