//! Le code QR (évolution 9 de la section 27) : des octets en mode octet, à
//! correction L ou M, de la version 1 à la version 40, dessinés sans
//! bibliothèque d'après la norme ISO/IEC 18004 — motifs de repérage, de
//! synchronisation et d'alignement, codes de Reed-Solomon dans GF(256),
//! entrelacement des blocs, masque choisi par ses pénalités, information de
//! format et de version.

/// Le niveau de correction des erreurs : L (7 % environ) ou M (15 %).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Correction {
    L,
    M,
}

impl Correction {
    fn rang(self) -> usize {
        match self {
            Correction::L => 0,
            Correction::M => 1,
        }
    }

    /// Ses deux bits dans l'information de format.
    fn bits(self) -> u32 {
        match self {
            Correction::L => 1,
            Correction::M => 0,
        }
    }
}

/// Les mots de correction par bloc, selon le niveau et la version.
const CORRECTION_PAR_BLOC: [[u8; 41]; 2] = [
    [
        0, 7, 10, 15, 20, 26, 18, 20, 24, 30, 18, 20, 24, 26, 30, 22, 24, 28, 30, 28, 28, 28, 28,
        30, 30, 26, 28, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30,
    ],
    [
        0, 10, 16, 26, 18, 24, 16, 18, 22, 22, 26, 30, 22, 22, 24, 24, 28, 28, 26, 26, 26, 26, 28,
        28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28,
    ],
];

/// Le nombre de blocs, selon le niveau et la version.
const BLOCS: [[u8; 41]; 2] = [
    [
        0, 1, 1, 1, 1, 1, 2, 2, 2, 2, 4, 4, 4, 4, 4, 6, 6, 6, 6, 7, 8, 8, 9, 9, 10, 12, 12, 12, 13,
        14, 15, 16, 17, 18, 19, 19, 20, 21, 22, 24, 25,
    ],
    [
        0, 1, 1, 1, 2, 2, 4, 4, 4, 5, 5, 5, 8, 9, 9, 10, 10, 11, 13, 14, 16, 17, 17, 18, 20, 21,
        23, 25, 26, 28, 29, 31, 33, 35, 37, 38, 40, 43, 45, 47, 49,
    ],
];

/// Le nombre de modules qui portent des données (les mots et leur
/// correction), une fois les motifs posés.
fn modules_de_donnees(version: usize) -> usize {
    let mut modules = (16 * version + 128) * version + 64;
    if version >= 2 {
        let alignements = version / 7 + 2;
        modules -= (25 * alignements - 10) * alignements - 55;
        if version >= 7 {
            modules -= 36;
        }
    }
    modules
}

/// Le nombre de mots de données d'une version, à ce niveau.
fn mots_de_donnees(version: usize, correction: Correction) -> usize {
    let niveau = correction.rang();
    modules_de_donnees(version) / 8
        - usize::from(CORRECTION_PAR_BLOC[niveau][version]) * usize::from(BLOCS[niveau][version])
}

/// Le plus d'octets qu'un code QR puisse porter à ce niveau (version 40).
pub fn capacite(correction: Correction) -> usize {
    (mots_de_donnees(40, correction) * 8 - 4 - 16) / 8
}

/// Le produit de deux éléments de GF(256), modulo x⁸ + x⁴ + x³ + x² + 1.
fn multiplier(x: u8, y: u8) -> u8 {
    let mut z: u32 = 0;
    for i in (0..8).rev() {
        z = (z << 1) ^ ((z >> 7) * 0x11D);
        z ^= ((u32::from(y) >> i) & 1) * u32::from(x);
    }
    z as u8
}

/// Le polynôme générateur de Reed-Solomon de ce degré, sans son premier
/// coefficient (1).
fn generateur(degre: usize) -> Vec<u8> {
    let mut resultat = vec![0u8; degre];
    resultat[degre - 1] = 1;
    let mut racine = 1u8;
    for _ in 0..degre {
        for j in 0..degre {
            resultat[j] = multiplier(resultat[j], racine);
            if j + 1 < degre {
                resultat[j] ^= resultat[j + 1];
            }
        }
        racine = multiplier(racine, 0x02);
    }
    resultat
}

/// Les mots de correction de ces données : le reste de leur division par le
/// générateur.
fn correction_de(donnees: &[u8], generateur: &[u8]) -> Vec<u8> {
    let mut reste = vec![0u8; generateur.len()];
    for &octet in donnees {
        let facteur = octet ^ reste[0];
        reste.remove(0);
        reste.push(0);
        for (r, &g) in reste.iter_mut().zip(generateur) {
            *r ^= multiplier(g, facteur);
        }
    }
    reste
}

/// Un code QR : sa version, sa taille en modules et ses modules (vrai pour un
/// module foncé), rangée par rangée.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodeQr {
    pub version: usize,
    pub taille: usize,
    modules: Vec<bool>,
    /// Vrai pour un module d'un motif, que le masque ne touche pas.
    motifs: Vec<bool>,
}

impl CodeQr {
    /// Vrai si le module de la colonne `x` et de la rangée `y` est foncé.
    pub fn fonce(&self, x: usize, y: usize) -> bool {
        self.modules[y * self.taille + x]
    }

    fn poser(&mut self, x: usize, y: usize, fonce: bool) {
        let k = y * self.taille + x;
        self.modules[k] = fonce;
        self.motifs[k] = true;
    }

    /// Les motifs : synchronisation, repérage, alignement, et la place de
    /// l'information de format et de version.
    fn poser_les_motifs(&mut self) {
        let taille = self.taille;
        for i in 0..taille {
            self.poser(6, i, i % 2 == 0);
            self.poser(i, 6, i % 2 == 0);
        }
        for (x, y) in [(3, 3), (taille - 4, 3), (3, taille - 4)] {
            self.poser_un_reperage(x, y);
        }
        let positions = positions_des_alignements(self.version);
        let n = positions.len();
        for i in 0..n {
            for j in 0..n {
                let coin = (i == 0 && j == 0) || (i == 0 && j == n - 1) || (i == n - 1 && j == 0);
                if !coin {
                    self.poser_un_alignement(positions[i], positions[j]);
                }
            }
        }
        self.poser_le_format(Correction::L, 0);
        self.poser_la_version();
    }

    fn poser_un_reperage(&mut self, x: usize, y: usize) {
        for dy in -4i64..=4 {
            for dx in -4i64..=4 {
                let distance = dx.abs().max(dy.abs());
                let (xx, yy) = (x as i64 + dx, y as i64 + dy);
                if (0..self.taille as i64).contains(&xx) && (0..self.taille as i64).contains(&yy) {
                    self.poser(xx as usize, yy as usize, distance != 2 && distance != 4);
                }
            }
        }
    }

    fn poser_un_alignement(&mut self, x: usize, y: usize) {
        for dy in -2i64..=2 {
            for dx in -2i64..=2 {
                let (xx, yy) = ((x as i64 + dx) as usize, (y as i64 + dy) as usize);
                self.poser(xx, yy, dx.abs().max(dy.abs()) != 1);
            }
        }
    }

    /// L'information de format : le niveau et le masque, avec leur code BCH,
    /// en deux exemplaires; et le module toujours foncé.
    fn poser_le_format(&mut self, correction: Correction, masque: u32) {
        let donnees = (correction.bits() << 3) | masque;
        let mut reste = donnees;
        for _ in 0..10 {
            reste = (reste << 1) ^ ((reste >> 9) * 0x537);
        }
        let bits = ((donnees << 10) | reste) ^ 0x5412;
        let bit = |i: usize| (bits >> i) & 1 == 1;
        for i in 0..=5 {
            self.poser(8, i, bit(i));
        }
        self.poser(8, 7, bit(6));
        self.poser(8, 8, bit(7));
        self.poser(7, 8, bit(8));
        for i in 9..15 {
            self.poser(14 - i, 8, bit(i));
        }
        let taille = self.taille;
        for i in 0..8 {
            self.poser(taille - 1 - i, 8, bit(i));
        }
        for i in 8..15 {
            self.poser(8, taille - 15 + i, bit(i));
        }
        self.poser(8, taille - 8, true);
    }

    /// L'information de version (à partir de la version 7), en deux
    /// exemplaires.
    fn poser_la_version(&mut self) {
        if self.version < 7 {
            return;
        }
        let version = self.version as u32;
        let mut reste = version;
        for _ in 0..12 {
            reste = (reste << 1) ^ ((reste >> 11) * 0x1F25);
        }
        let bits = (version << 12) | reste;
        for i in 0..18 {
            let fonce = (bits >> i) & 1 == 1;
            let (a, b) = (self.taille - 11 + i % 3, i / 3);
            self.poser(a, b, fonce);
            self.poser(b, a, fonce);
        }
    }

    /// Les mots, en zigzag depuis le coin du bas à droite, deux colonnes à la
    /// fois, la colonne 6 sautée.
    fn placer(&mut self, mots: &[u8]) {
        let taille = self.taille;
        let mut i = 0;
        let mut droite = taille as i64 - 1;
        while droite >= 1 {
            if droite == 6 {
                droite = 5;
            }
            for vertical in 0..taille {
                for j in 0..2 {
                    let x = (droite - j) as usize;
                    let monte = ((droite + 1) & 2) == 0;
                    let y = if monte {
                        taille - 1 - vertical
                    } else {
                        vertical
                    };
                    let k = y * taille + x;
                    if !self.motifs[k] && i < mots.len() * 8 {
                        self.modules[k] = (mots[i >> 3] >> (7 - (i & 7))) & 1 == 1;
                        i += 1;
                    }
                }
            }
            droite -= 2;
        }
    }

    /// Applique (ou retire : c'est le même ou exclusif) un des huit masques.
    fn masquer(&mut self, masque: u32) {
        let taille = self.taille;
        for y in 0..taille {
            for x in 0..taille {
                let inverser = match masque {
                    0 => (x + y) % 2 == 0,
                    1 => y % 2 == 0,
                    2 => x % 3 == 0,
                    3 => (x + y) % 3 == 0,
                    4 => (x / 3 + y / 2) % 2 == 0,
                    5 => x * y % 2 + x * y % 3 == 0,
                    6 => (x * y % 2 + x * y % 3) % 2 == 0,
                    _ => ((x + y) % 2 + x * y % 3) % 2 == 0,
                };
                let k = y * taille + x;
                if inverser && !self.motifs[k] {
                    self.modules[k] = !self.modules[k];
                }
            }
        }
    }

    /// Les pénalités d'un masque : les longues suites d'une même couleur, les
    /// carrés de 2 × 2, ce qui ressemble à un motif de repérage, et l'écart à
    /// la moitié de modules foncés.
    fn penalites(&self) -> usize {
        let taille = self.taille;
        let mut penalites = 0;
        for sens in 0..2 {
            for a in 0..taille {
                let suite: Vec<bool> = (0..taille)
                    .map(|b| {
                        if sens == 0 {
                            self.fonce(b, a)
                        } else {
                            self.fonce(a, b)
                        }
                    })
                    .collect();
                let mut longueur = 1;
                for k in 1..=taille {
                    if k < taille && suite[k] == suite[k - 1] {
                        longueur += 1;
                    } else {
                        if longueur >= 5 {
                            penalites += 3 + longueur - 5;
                        }
                        longueur = 1;
                    }
                }
                // Un faux motif de repérage, 1:1:3:1:1, quatre modules clairs
                // d'un côté (le dehors du code est clair).
                let mut bordee = vec![false; 4];
                bordee.extend(&suite);
                bordee.extend([false; 4]);
                const MOTIF: [bool; 7] = [true, false, true, true, true, false, true];
                for k in 0..bordee.len().saturating_sub(10) {
                    let clairs = |debut: usize| bordee[debut..debut + 4].iter().all(|&m| !m);
                    if bordee[k..k + 7] == MOTIF && clairs(k + 7)
                        || bordee[k + 4..k + 11] == MOTIF && clairs(k)
                    {
                        penalites += 40;
                    }
                }
            }
        }
        for y in 0..taille - 1 {
            for x in 0..taille - 1 {
                let m = self.fonce(x, y);
                if m == self.fonce(x + 1, y)
                    && m == self.fonce(x, y + 1)
                    && m == self.fonce(x + 1, y + 1)
                {
                    penalites += 3;
                }
            }
        }
        let fonces = self.modules.iter().filter(|&&m| m).count();
        let total = taille * taille;
        let k = (fonces * 20).abs_diff(total * 10).div_ceil(total) - 1;
        penalites + k * 10
    }
}

/// Le centre des motifs d'alignement d'une version, sur chaque axe.
fn positions_des_alignements(version: usize) -> Vec<usize> {
    if version == 1 {
        return Vec::new();
    }
    let alignements = version / 7 + 2;
    let taille = version * 4 + 17;
    let pas = (version * 8 + alignements * 3 + 5) / (alignements * 4 - 4) * 2;
    let mut positions: Vec<usize> = (0..alignements - 1).map(|i| taille - 7 - i * pas).collect();
    positions.push(6);
    positions.reverse();
    positions
}

/// Les mots de données, coupés en blocs, chacun suivi de sa correction, puis
/// entrelacés.
fn entrelacer(donnees: &[u8], version: usize, correction: Correction) -> Vec<u8> {
    let niveau = correction.rang();
    let blocs = usize::from(BLOCS[niveau][version]);
    let par_bloc = usize::from(CORRECTION_PAR_BLOC[niveau][version]);
    let mots = modules_de_donnees(version) / 8;
    let courts = blocs - mots % blocs;
    let longueur_courte = mots / blocs;
    let generateur = generateur(par_bloc);
    let mut morceaux: Vec<(Vec<u8>, Vec<u8>)> = Vec::with_capacity(blocs);
    let mut debut = 0;
    for i in 0..blocs {
        let longueur = longueur_courte - par_bloc + usize::from(i >= courts);
        let bloc = donnees[debut..debut + longueur].to_vec();
        debut += longueur;
        let correction = correction_de(&bloc, &generateur);
        morceaux.push((bloc, correction));
    }
    let mut resultat = Vec::with_capacity(mots);
    for i in 0..=longueur_courte - par_bloc {
        for (bloc, _) in &morceaux {
            if let Some(&mot) = bloc.get(i) {
                resultat.push(mot);
            }
        }
    }
    for i in 0..par_bloc {
        for (_, correction) in &morceaux {
            resultat.push(correction[i]);
        }
    }
    resultat
}

/// Le code QR de ces octets, en mode octet, à ce niveau de correction, dans la
/// plus petite version qui les tient. `None` s'ils sont trop nombreux pour la
/// version 40 : rien n'est jamais tronqué.
pub fn encoder(octets: &[u8], correction: Correction) -> Option<CodeQr> {
    let (version, capacite) = (1..=40).find_map(|version| {
        let capacite = mots_de_donnees(version, correction) * 8;
        let compte = if version <= 9 { 8 } else { 16 };
        (4 + compte + octets.len() * 8 <= capacite).then_some((version, capacite))
    })?;
    let mut bits: Vec<bool> = Vec::with_capacity(capacite);
    let mut ajouter = |valeur: u32, longueur: usize| {
        for i in (0..longueur).rev() {
            bits.push((valeur >> i) & 1 == 1);
        }
    };
    ajouter(0b0100, 4);
    ajouter(octets.len() as u32, if version <= 9 { 8 } else { 16 });
    for &octet in octets {
        ajouter(u32::from(octet), 8);
    }
    // La fin : quatre zéros au plus, puis jusqu'au bout de l'octet.
    let fin = (capacite - bits.len()).min(4);
    bits.extend(std::iter::repeat_n(false, fin));
    let bourrage = (8 - bits.len() % 8) % 8;
    bits.extend(std::iter::repeat_n(false, bourrage));
    let mut donnees: Vec<u8> = bits
        .chunks(8)
        .map(|octet| octet.iter().fold(0u8, |a, &b| (a << 1) | u8::from(b)))
        .collect();
    let mut remplissage = 0xEC;
    while donnees.len() * 8 < capacite {
        donnees.push(remplissage);
        remplissage ^= 0xEC ^ 0x11;
    }
    let taille = version * 4 + 17;
    let mut code = CodeQr {
        version,
        taille,
        modules: vec![false; taille * taille],
        motifs: vec![false; taille * taille],
    };
    code.poser_les_motifs();
    code.placer(&entrelacer(&donnees, version, correction));
    // Le masque qui donne le moins de pénalités.
    let mut meilleur = (usize::MAX, 0);
    for masque in 0..8 {
        code.masquer(masque);
        code.poser_le_format(correction, masque);
        let penalites = code.penalites();
        if penalites < meilleur.0 {
            meilleur = (penalites, masque);
        }
        code.masquer(masque);
    }
    code.masquer(meilleur.1);
    code.poser_le_format(correction, meilleur.1);
    Some(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_capacite_des_versions() {
        // Les mots de données de la norme : 1-L 19, 1-M 16, 5-M 86, 10-L 274,
        // 40-L 2956, 40-M 2334.
        assert_eq!(mots_de_donnees(1, Correction::L), 19);
        assert_eq!(mots_de_donnees(1, Correction::M), 16);
        assert_eq!(mots_de_donnees(5, Correction::M), 86);
        assert_eq!(mots_de_donnees(10, Correction::L), 274);
        assert_eq!(mots_de_donnees(40, Correction::L), 2956);
        assert_eq!(mots_de_donnees(40, Correction::M), 2334);
        assert_eq!(capacite(Correction::L), 2953);
        // La plus petite version qui tient : 17 octets en 1-L, 18 en 2-L.
        assert_eq!(encoder(&[b'a'; 17], Correction::L).unwrap().version, 1);
        assert_eq!(encoder(&[b'a'; 18], Correction::L).unwrap().version, 2);
        // Trop gros : rien, jamais tronqué.
        assert!(encoder(&vec![0; 2953], Correction::L).is_some());
        assert!(encoder(&vec![0; 2954], Correction::L).is_none());
    }

    #[test]
    fn les_motifs_d_un_code() {
        let code = encoder(b"https://constructoai.ca/", Correction::M).unwrap();
        let n = code.taille;
        assert_eq!(n, code.version * 4 + 17);
        // Les trois motifs de repérage : un anneau foncé, un anneau clair et
        // un carré foncé au milieu.
        for (x, y) in [(3, 3), (n - 4, 3), (3, n - 4)] {
            assert!(code.fonce(x, y) && code.fonce(x - 3, y) && !code.fonce(x - 2, y));
            assert!(!code.fonce(x - 2, y - 2) && code.fonce(x - 1, y - 1));
        }
        // La synchronisation alterne; le module foncé du format est là.
        for i in 8..n - 8 {
            assert_eq!(code.fonce(i, 6), i % 2 == 0);
            assert_eq!(code.fonce(6, i), i % 2 == 0);
        }
        assert!(code.fonce(8, n - 8));
        // Les alignements de la version 7 : 6, 22 et 38.
        assert_eq!(positions_des_alignements(7), vec![6, 22, 38]);
        assert_eq!(positions_des_alignements(32), vec![6, 34, 60, 86, 112, 138]);
    }

    #[test]
    fn reed_solomon_d_un_exemple_de_la_norme() {
        // L'exemple de l'annexe de la norme : « 01234567 » en version 1-M,
        // ses 16 mots de données et leurs 10 mots de correction.
        let donnees = [
            0x10, 0x20, 0x0C, 0x56, 0x61, 0x80, 0xEC, 0x11, 0xEC, 0x11, 0xEC, 0x11, 0xEC, 0x11,
            0xEC, 0x11,
        ];
        let correction = correction_de(&donnees, &generateur(10));
        assert_eq!(
            correction,
            vec![0xA5, 0x24, 0xD4, 0xC1, 0xED, 0x36, 0xC7, 0x87, 0x2C, 0x55]
        );
    }
}
