//! Robustesse (section 19.1) : sous des frappes pseudo-aléatoires, le moteur ne
//! panique jamais et l'écran reste formatable. Propriétés de la section 19.16.

mod commun;
use commun::{jouer, moteur_avec, moteur_neuf, texte};
use noyau::conversions::{Conversion, UniteTouche, convertir};
use noyau::prefs::{EmpannonsIrreguliers, ModeFractions, MurCintre, Ordre, Preferences};
use noyau::unites::{Format, MasseVolumique};
use noyau::{EtatPersistant, Moteur, Touche, Valeur};

/// Générateur pseudo-aléatoire maison (xorshift64*), à graine fixe.
struct Alea(u64);

impl Alea {
    fn suivant(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn sous(&mut self, n: u64) -> u64 {
        self.suivant() % n
    }

    fn unite(&mut self) -> f64 {
        (self.suivant() >> 11) as f64 / (1u64 << 53) as f64
    }
}

fn toutes_les_touches() -> Vec<Touche> {
    let mut touches: Vec<Touche> = Touche::GRILLE.iter().flatten().copied().collect();
    touches.push(Touche::RetourArriere);
    touches.push(Touche::FractionRapide { num: 3, den: 8 });
    touches.push(Touche::FractionRapide { num: 1, den: 2 });
    touches.push(Touche::FractionRapide { num: 0, den: 0 });
    // Les chiffres et les touches de saisie reviennent plus souvent.
    for c in 0..10 {
        touches.push(Touche::Chiffre(c));
        touches.push(Touche::Chiffre(c));
    }
    touches.extend([
        Touche::Pieds,
        Touche::Pouces,
        Touche::Fraction,
        Touche::Virgule,
    ]);
    touches
}

fn marteler(mut moteur: Moteur, graine: u64, frappes: usize) {
    let touches = toutes_les_touches();
    let mut alea = Alea(graine);
    for i in 0..frappes {
        let ecran = moteur.ecran();
        // En état d'erreur, Effacer sort plus vite pour mieux couvrir le reste.
        let touche = if ecran.erreur && alea.sous(3) == 0 {
            Touche::Effacer
        } else {
            touches[alea.sous(touches.len() as u64) as usize]
        };
        moteur.appuyer(touche);
        let ecran = moteur.ecran();
        let affiche = texte(&ecran);
        assert!(
            !affiche.is_empty(),
            "écran vide après {i} frappes ({touche:?})"
        );
        assert!(
            !affiche.contains("NaN") && !affiche.contains("inf"),
            "écran non formatable après {i} frappes ({touche:?}) : {affiche}"
        );
        if i % 5000 == 4999 {
            // L'état persistant se reconstruit sans perte apparente.
            let copie = Moteur::nouveau(moteur.etat_persistant());
            assert_eq!(copie.preferences(), moteur.preferences());
            let _ = moteur.ruban().en_texte("Ruban");
        }
        if i % 20000 == 19999 {
            moteur.effacer_tout();
        }
    }
}

#[test]
fn frappes_aleatoires_preferences_par_defaut() {
    marteler(moteur_neuf(), 0x1234_5678_9ABC_DEF1, 80_000);
}

#[test]
fn frappes_aleatoires_mode_trigonometrique_et_exponentiel() {
    let moteur = moteur_avec(|p| {
        p.mode_trigo = true;
        p.exponentielle = true;
        p.resolution = 64;
        p.mode_fractions = ModeFractions::Constant;
        p.decimales_sans_unite = Some(2);
    });
    marteler(moteur, 0x0BAD_CAFE_F00D_BEEF, 60_000);
}

#[test]
fn frappes_aleatoires_autres_reglages_de_fonctions() {
    let moteur = moteur_avec(|p| {
        p.mur_pente = Ordre::Croissant;
        p.empannons = Ordre::Croissant;
        p.empannons_irreguliers = EmpannonsIrreguliers::Apparies;
        p.mur_cintre = MurCintre::Interieur;
        p.resolution = 2;
        p.metres_fixes = false;
        p.degres_fixes = false;
        p.separateur_milliers = false;
    });
    marteler(moteur, 0x0F1E_2D3C_4B5A_6978, 60_000);
}

#[test]
fn etat_persistant_incoherent() {
    // Un fichier modifié à la main ne doit pas faire paniquer le moteur.
    let mut etat = EtatPersistant::default();
    etat.preferences.resolution = 7;
    etat.memorises.entraxe = Valeur::nombre(f64::NAN);
    etat.memorises.contremarche_visee = Valeur::en_base(-3.0, Format::PoFrac);
    etat.memorises.paquets_par_carre = 0.0;
    etat.memorises.masse_volumique.valeur = f64::INFINITY;
    etat.registres[0] = Some(Valeur::nombre(f64::NAN));
    let mut moteur = Moteur::nouveau(etat);
    assert_eq!(texte(&jouer(&mut moteur, "Rap 5")), "ENTRAXE │ 16 po");
    assert_eq!(texte(&jouer(&mut moteur, "Rap 1")), "AUCUNE VALEUR");
    assert_eq!(
        texte(&jouer(&mut moteur, "Eff 5 Pi 7 Po 1 / 2 =")),
        "5 pi 7 1/2 po"
    );
    marteler(moteur, 0x5EED_5EED_5EED_5EED, 5_000);
}

#[test]
fn changer_de_format_ne_modifie_jamais_la_base() {
    // Propriété 19.16 : conversion aller-retour sur 1 000 longueurs.
    let mut alea = Alea(0xA5A5_5A5A_1234_4321);
    let mv = MasseVolumique::default();
    let unites = [
        UniteTouche::Verges,
        UniteTouche::Pieds,
        UniteTouche::Pouces,
        UniteTouche::Metres,
        UniteTouche::Cm,
        UniteTouche::Mm,
    ];
    for _ in 0..1000 {
        let base = alea.unite() * 20_000.0;
        let mut v = Valeur::en_base(base, Format::PiPo);
        for _ in 0..12 {
            let u = unites[alea.sous(unites.len() as u64) as usize];
            match convertir(&v, u, &mv) {
                Ok(Conversion::Valeur(nouvelle)) => v = nouvelle,
                autre => panic!("conversion inattendue : {autre:?}"),
            }
            assert_eq!(v.base.to_bits(), base.to_bits());
        }
    }
}

/// Appuie `n` fois sur une touche et retourne les écrans obtenus.
fn ecrans(moteur: &mut Moteur, touche: &str, n: usize) -> Vec<String> {
    (0..n)
        .map(|_| {
            let e = jouer(moteur, touche);
            format!("{}|{}|{}", texte(&e), e.memorise, e.avertissement.is_some())
        })
        .collect()
}

#[test]
fn chaque_cycle_reboucle_sur_sa_premiere_sortie() {
    // Propriété 19.16 : après n pressions (n = longueur du cycle), la suivante
    // ramène à la première sortie.
    let preparation = "7 Po Pente 8 Pi 5 Po Course 8 Po Conv Arêtier \
                       20 Pi Longueur 12 Pi Largeur 8 Pi Hauteur 5 Pi Cercle 85 Arc 6 Onglet Eff";
    // (lancement, touche de défilement)
    let fonctions = [
        ("Pente", "Pente"),
        ("Montée", "Montée"),
        ("Course", "Course"),
        ("Diag", "Diag"),
        ("Arêtier", "Arêtier"),
        ("Empannon", "Empannon"),
        ("Conv Empannon", "Empannon"),
        ("Conv Montée", "Montée"),
        ("Conv Diag", "Diag"),
        ("Escalier", "Escalier"),
        ("Conv Escalier", "Escalier"),
        ("Onglet", "Onglet"),
        ("Arc", "Arc"),
        ("Cercle", "Cercle"),
        ("Conv Cercle", "Cercle"),
        ("6 Conv Course", "Course"),
        ("Longueur", "Longueur"),
        ("Largeur", "Largeur"),
        ("Hauteur", "Hauteur"),
        ("Conv Longueur", "Longueur"),
        ("Conv Largeur", "Largeur"),
        ("Conv Hauteur", "Hauteur"),
    ];
    for (lancement, touche) in fonctions {
        let mut moteur = moteur_neuf();
        jouer(&mut moteur, preparation);
        let premier = ecrans(&mut moteur, lancement, 1).remove(0);
        let mut suite = vec![premier];
        suite.extend(ecrans(&mut moteur, touche, 199));
        let periode = (1..=100)
            .find(|p| (0..100).all(|i| suite[i] == suite[i + p]))
            .unwrap_or_else(|| panic!("le cycle de « {lancement} » ne reboucle pas"));
        assert_eq!(suite[periode], suite[0], "cycle de « {lancement} »");
    }
}

#[test]
fn les_preferences_par_defaut_sont_celles_du_cahier() {
    let p = Preferences::default();
    assert_eq!(p.resolution, 16);
    assert!(!p.mode_trigo && !p.fractions_rapides && !p.exponentielle);
    assert!(p.metres_fixes && p.degres_fixes && p.separateur_milliers);
    assert_eq!(p.decimales_sans_unite, None);
}
