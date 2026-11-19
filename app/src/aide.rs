//! Contenu des fiches d'aide (annexe B du cahier des charges) et recherche.
//!
//! Dans les textes, les accents graves entourent le nom d'une touche : l'écran
//! d'aide les dessine comme de petites touches.

use noyau::Touche;

/// Une fiche d'aide.
pub struct Fiche {
    pub titre: &'static str,
    /// Frappes qui donnent accès à la fonction (fonctions secondaires).
    pub acces: &'static str,
    /// « À quoi ça sert ».
    pub resume: &'static str,
    /// « Comment faire ».
    pub mode_emploi: &'static str,
    /// Exemple chiffré : frappes, puis résultat.
    pub exemple: Option<(&'static str, &'static str)>,
    pub note: Option<&'static str>,
    /// Mots-clés de recherche, non affichés.
    pub mots_cles: &'static str,
    /// Touches dont c'est la fiche.
    pub touches: &'static [Touche],
    /// Fiche d'une fonction secondaire (avec `Conv`).
    pub secondaire: bool,
    /// Fiche propre au mode trigonométrique.
    pub trigo: bool,
}

const fn fiche(
    titre: &'static str,
    resume: &'static str,
    mode_emploi: &'static str,
    exemple: Option<(&'static str, &'static str)>,
    mots_cles: &'static str,
    touches: &'static [Touche],
) -> Fiche {
    Fiche {
        titre,
        acces: "",
        resume,
        mode_emploi,
        exemple,
        note: None,
        mots_cles,
        touches,
        secondaire: false,
        trigo: false,
    }
}

const fn secondaire(
    titre: &'static str,
    acces: &'static str,
    resume: &'static str,
    mode_emploi: &'static str,
    exemple: Option<(&'static str, &'static str)>,
    mots_cles: &'static str,
    touches: &'static [Touche],
) -> Fiche {
    Fiche {
        titre,
        acces,
        resume,
        mode_emploi,
        exemple,
        note: None,
        mots_cles,
        touches,
        secondaire: true,
        trigo: false,
    }
}

const fn avec_note(mut f: Fiche, note: &'static str) -> Fiche {
    f.note = Some(note);
    f
}

const CHIFFRES: &[Touche] = &[
    Touche::Chiffre(0),
    Touche::Chiffre(1),
    Touche::Chiffre(2),
    Touche::Chiffre(3),
    Touche::Chiffre(4),
    Touche::Chiffre(5),
    Touche::Chiffre(6),
    Touche::Chiffre(7),
    Touche::Chiffre(8),
    Touche::Chiffre(9),
    Touche::Virgule,
];

/// Toutes les fiches : les touches principales, puis les fonctions secondaires.
pub static FICHES: &[Fiche] = &[
    // ------------------------------------------------------------------
    // B.1 Touches principales
    // ------------------------------------------------------------------
    fiche(
        "Pente",
        "La pente d'un toit ou d'un plan incliné.",
        "Elle s'exprime en pouces de montée pour 12 pouces de course (7 po = pente 7/12), en degrés, en pourcentage ou en ratio. Pour la saisir : `7` `Pouces` `Pente` (7/12), `30` `Pente` (30 degrés) ou `75` `%` `Pente` (75 %). Pour la calculer : saisissez deux valeurs parmi la montée, la course et la diagonale, puis appuyez sur `Pente`. Appuyez de nouveau pour la voir sous ses autres formes. La pente saisie reste en mémoire jusqu'à ce que vous en saisissiez une autre.",
        Some((
            "`30,45` `Pente`, puis `Pente` à répétition",
            "30,45° · 58,78702 % · ratio 0,58787 · 7 1/16 po",
        )),
        "toit inclinaison degrés pourcentage pitch",
        &[Touche::Pente],
    ),
    fiche(
        "Montée",
        "Le côté vertical d'un triangle rectangle.",
        "C'est la hauteur d'un toit de la sablière au faîte, la hauteur totale d'un escalier d'un plancher fini à l'autre, ou la flèche d'un arc. Tapez la mesure puis `Montée` pour la saisir. Pour la calculer, saisissez deux autres valeurs (pente, course ou diagonale) et appuyez sur `Montée`.",
        Some((
            "`7` `Pouces` `Pente` `11` `Pieds` `6` `Pouces` `Course` `Montée`",
            "MONTÉE 6 pi 8 1/2 po",
        )),
        "hauteur vertical rise",
        &[Touche::Montee],
    ),
    fiche(
        "Course",
        "Le côté horizontal d'un triangle rectangle.",
        "C'est la demi-portée d'un toit (du mur extérieur au centre du faîte), la longueur au sol d'un escalier, ou la corde d'un arc. Tapez la mesure puis `Course`, ou calculez-la à partir de deux autres valeurs.",
        Some((
            "`6` `Pouces` `Pente` `4` `Pieds` `Diag` `Course`",
            "COURSE 3 pi 6 15/16 po",
        )),
        "horizontal portée run",
        &[Touche::Course],
    ),
    avec_note(
        fiche(
            "Diag",
            "La diagonale (hypoténuse).",
            "C'est la longueur du chevron commun, ou la diagonale qui sert à vérifier l'équerrage d'une fondation. Appuyez de nouveau pour obtenir la coupe d'aplomb et la coupe de niveau.",
            Some((
                "`7` `Pouces` `Pente` `7` `Pieds` `2` `Pouces` `Course` `Diag`, puis `Diag` deux fois",
                "DIAGONALE 8 pi 3 9/16 po · COUPE D'APLOMB 30,26° · COUPE DE NIVEAU 59,74°",
            )),
            "chevron commun hypoténuse équerre",
            &[Touche::Diag],
        ),
        "La longueur est mesurée de pointe à pointe : ajoutez le débord du toit et retranchez la demi-épaisseur de la faîtière au besoin.",
    ),
    avec_note(
        fiche(
            "Arêtier Noue",
            "La longueur du chevron d'arêtier ou de noue et ses coupes.",
            "Saisissez la pente et la course du chevron commun, puis appuyez sur `Arêtier Noue`. Appuyez de nouveau pour la coupe d'aplomb, la coupe de niveau et la coupe latérale (biseau). Pour un toit dont les deux versants ont des pentes différentes, saisissez d'abord la pente irrégulière.",
            Some((
                "`8` `Pieds` `5` `Pouces` `Course` `7` `Pouces` `Pente` `Arêtier Noue`, puis `Arêtier Noue` à répétition",
                "ARÊTIER/NOUE 12 pi 10 1/2 po · 22,42° · 67,58° · 45,00°",
            )),
            "croupe hip valley arête",
            &[Touche::AretierNoue],
        ),
        "Les longueurs sont théoriques, d'axe en axe.",
    ),
    fiche(
        "Onglet composé",
        "Les réglages de la scie à onglets pour une moulure couronne coupée à plat.",
        "Tapez l'angle du coin (90 pour un coin d'équerre), puis `Onglet composé` : vous obtenez l'angle d'onglet (rotation de la table) et l'angle de biseau (inclinaison de la lame). Pour un ouvrage à plusieurs côtés égaux, tapez plutôt le nombre de côtés (toute valeur inférieure à 25 est prise pour un nombre de côtés). Vérifiez d'abord que l'angle de la moulure mémorisé correspond à votre moulure.",
        Some((
            "`90` `Onglet composé`, puis `Onglet composé`",
            "ONGLET 35,26° · BISEAU 30,00°",
        )),
        "corniche moulure biseau scie",
        &[Touche::OngletCompose],
    ),
    avec_note(
        fiche(
            "Escalier",
            "Le calcul complet d'un escalier droit.",
            "Saisissez la hauteur à monter avec `Montée`, la longueur disponible avec `Course`, ou les deux, puis appuyez sur `Escalier` à répétition : hauteur et nombre de contremarches, giron et nombre de marches, écarts d'arrondi, longueur de la trémie, longueur du limon, inclinaison. La hauteur de contremarche et le giron visés, l'échappée et l'épaisseur du plancher se règlent dans les Préférences. Un symbole d'avertissement signale une contremarche plus haute que la valeur visée.",
            Some((
                "`9` `Pieds` `11` `Pouces` `Montée`, puis `Escalier` à répétition",
                "CONTREMARCHE 7 7/16 po · 16 contremarches · GIRON 10 po · 15 marches · TRÉMIE 10 pi 1 po · LIMON 15 pi 6 15/16 po · 36,64°",
            )),
            "marche contremarche giron limon trémie",
            &[Touche::Escalier],
        ),
        "Le giron n'inclut pas le nez de marche. Vérifiez toujours les exigences du Code de construction.",
    ),
    fiche(
        "Arc",
        "Tout ce qui concerne un arc de cercle.",
        "Deux données suffisent : le diamètre et l'angle, le diamètre et la longueur d'arc, ou encore la corde (touche `Course`) et la flèche (touche `Montée`). Appuyez sur `Arc` à répétition : angle, longueur d'arc, corde, aire du segment, aire du secteur, flèche, puis la longueur de chaque montant d'un mur cintré à l'entraxe mémorisé.",
        Some((
            "`5` `Pieds` `Cercle` `85` `Arc`, puis `Arc`",
            "ARC 85,00° · ARC 3 pi 8 1/2 po",
        )),
        "cintre arche fenêtre cintrée corde flèche",
        &[Touche::Arc],
    ),
    fiche(
        "Cercle",
        "Aire et circonférence d'un cercle.",
        "Tapez le diamètre, puis `Cercle`; appuyez de nouveau pour l'aire, puis la circonférence.",
        Some((
            "`11` `Pouces` `Cercle`, puis `Cercle` deux fois",
            "AIRE 95,03318 po² · CIRCONFÉRENCE 34 9/16 po",
        )),
        "diamètre rond",
        &[Touche::Cercle],
    ),
    fiche(
        "Empannon",
        "La longueur de chaque empannon.",
        "Les empannons sont les chevrons raccourcis qui s'appuient sur l'arêtier ou la noue. Saisissez la pente et la course, puis appuyez sur `Empannon` à répétition : entraxe, incrément (différence de longueur d'un empannon au suivant), chaque longueur, puis les coupes. L'ordre (décroissant ou croissant) se choisit dans les Préférences.",
        Some((
            "`8` `Pieds` `5` `Pouces` `Course` `7` `Pouces` `Pente`, puis `Empannon` à répétition",
            "ENTRAXE 16 po · INCRÉMENT 1 pi 6 1/2 po · 8 pi 2 3/8 po · 6 pi 7 7/8 po · 5 pi 1 3/8 po…",
        )),
        "chevron jack",
        &[Touche::Empannon],
    ),
    fiche(
        "m",
        "Mètres.",
        "Tapez un nombre puis `m`. Deux pressions donnent des mètres carrés, trois des mètres cubes. Sur une valeur déjà affichée, `m` la convertit en mètres.",
        Some(("`8` `Pieds` `Conv` `m`", "2,438 m")),
        "métrique mètre",
        &[Touche::Metres],
    ),
    fiche(
        "Longueur",
        "Mémorise une longueur",
        "pour les calculs d'aire, de volume et de matériaux (blocs, semelle, gypse, toiture).",
        Some((
            "`20` `Pieds` `6` `Pouces` `Longueur`",
            "LONGUEUR 20 pi 6 po",
        )),
        "dimension",
        &[Touche::Longueur],
    ),
    fiche(
        "Largeur",
        "Mémorise la largeur,",
        "puis donne, à chaque pression : l'aire, la diagonale d'équerrage et le périmètre du rectangle.",
        Some((
            "`20` `Pieds` `6` `Pouces` `Longueur` `25` `Pieds` `6` `Pouces` `Largeur`, puis `Largeur` à répétition",
            "AIRE 522,75 pi² · ÉQUERRAGE 32 pi 8 5/8 po · PÉRIMÈTRE 92 pi 0 po",
        )),
        "aire équerrage périmètre",
        &[Touche::Largeur],
    ),
    fiche(
        "Hauteur",
        "Mémorise la hauteur,",
        "puis donne, à chaque pression : le volume, l'aire des quatre murs et l'aire de la pièce (murs et plafond), utile pour la peinture et le gypse.",
        Some((
            "`15` `Pieds` `Longueur` `20` `Pieds` `Largeur` `12` `Pieds` `Hauteur`, puis `Hauteur` à répétition",
            "VOLUME 3 600 pi³ · MURS 840 pi² · PIÈCE 1 140 pi²",
        )),
        "volume murs",
        &[Touche::Hauteur],
    ),
    fiche(
        "%",
        "Pourcentages.",
        "`+` `10` `%` ajoute 10 % à la valeur, `−` `20` `%` en retranche 20 %, `×` `18` `%` en calcule 18 %, `÷` `80` `%` la divise par 0,80. Fonctionne aussi avec des mesures. Sert enfin à saisir une pente en pourcentage : `2` `%` `Pente`.",
        Some(("`137` `Pieds` `Pieds` `+` `10` `%`", "150,7 pi²")),
        "pertes rabais majoration pourcent",
        &[Touche::Pourcent],
    ),
    fiche(
        "Verges, Pieds, Pouces",
        "Les unités impériales.",
        "Tapez le nombre, puis l'unité : `5` `Pieds` `7` `Pouces`. Deux pressions de la même unité donnent des unités carrées, trois des unités cubes. Sur une valeur affichée, la touche convertit; `Pieds` alterne entre pieds-pouces et pieds décimaux, `Pouces` entre fraction et décimales.",
        Some((
            "`5` `Pieds` `7` `Pouces` `1` `/` `2` `+` `18` `Pieds` `8` `Pouces` `=`",
            "24 pi 3 1/2 po",
        )),
        "impérial conversion verge pied pouce",
        &[Touche::Verges, Touche::Pieds, Touche::Pouces],
    ),
    fiche(
        "/",
        "La barre de fraction.",
        "`1` `/` `2` donne un demi-pouce. Si vous ne tapez pas le dénominateur, la résolution en vigueur est utilisée : `15` `/` vaut 15/16.",
        Some(("`15` `/` `=`", "15/16 po")),
        "fraction",
        &[Touche::Fraction],
    ),
    fiche(
        "Effacer",
        "Une pression efface l'affichage et l'erreur éventuelle.",
        "Deux pressions effacent aussi les valeurs temporaires (montée, course, longueur, diamètre…). La pente saisie, les mémoires et les valeurs mémorisées sont conservées.",
        Some(("`5` `+` `3` `Effacer` `4` `=`", "9")),
        "remise à zéro",
        &[Touche::Effacer],
    ),
    fiche(
        "Conv",
        "La touche de conversion et de seconde fonction.",
        "Suivie d'une touche, elle exécute la fonction écrite en petit au-dessus de cette touche. Suivie d'une unité, elle convertit la valeur affichée. Vous pouvez aussi cliquer directement sur la petite étiquette.",
        Some(("`14` `Pieds` `Conv` `Verges`", "4,666667 vg")),
        "convertir seconde fonction",
        &[Touche::Conv],
    ),
    fiche(
        "Stocker",
        "Met une valeur en mémoire.",
        "`Stocker` `1`, `2` ou `3` : trois registres libres. `Stocker` `4` à `9` et `0` : les valeurs mémorisées de la calculatrice (bloc, entraxe, section de semelle, contremarche visée, épaisseur du plancher, giron visé, masse volumique).",
        Some(("`18` `Pouces` `Stocker` `5`", "ENTRAXE 18 po MÉM.")),
        "mémoire",
        &[Touche::Stocker],
    ),
    fiche(
        "Rappel",
        "Rappelle une valeur.",
        "Suivie d'une touche, elle affiche ce que cette touche a en mémoire : `Rappel` `5` montre l'entraxe, `Rappel` `Course` la dernière course. La valeur rappelée peut servir dans un calcul.",
        Some(("`Rappel` `7`", "CONTREMARCHE VISÉE 7 1/2 po MÉM.")),
        "mémoire",
        &[Touche::Rappel],
    ),
    fiche(
        "M+",
        "La mémoire cumulative.",
        "Chaque pression ajoute la valeur affichée au total. `Rappel` `M+` montre le total; `M+` ensuite montre la moyenne, puis le nombre de valeurs. `Rappel` `Rappel` montre le total et vide la mémoire.",
        Some((
            "`100` `M+` `200` `M+` `50` `Conv` `M+` `Rappel` `M+`, puis `M+` deux fois",
            "TOTAL M 250 · MOYENNE 83,33333 · NOMBRE 3",
        )),
        "total somme moyenne",
        &[Touche::MPlus],
    ),
    fiche(
        "=",
        "Termine le calcul.",
        "Après une touche de fonction, `=` rend le résultat disponible pour une autre fonction.",
        Some(("`2` `+` `3` `×` `4` `=`", "20")),
        "égal résultat",
        &[Touche::Egal],
    ),
    fiche(
        "⌫",
        "Retour arrière :",
        "annule la dernière frappe de la saisie en cours.",
        Some(("`12` `3` `⌫` `⌫` `7` `Pouces` `=`", "17 po")),
        "corriger effacer",
        &[Touche::RetourArriere],
    ),
    fiche(
        "Chiffres et virgule",
        "La saisie des nombres.",
        "Tapez les chiffres, puis l'unité s'il y a lieu. La virgule sépare les décimales. Un nombre compte au plus 12 chiffres. Dès qu'un résultat est affiché, le premier chiffre tapé commence une nouvelle valeur.",
        Some(("`17,32` `Pieds` `=`", "17,32 pi")),
        "nombre décimale",
        CHIFFRES,
    ),
    fiche(
        "+  −  ×  ÷",
        "Les quatre opérations.",
        "Les calculs se font dans l'ordre de la frappe, sans priorité des opérateurs : `2` `+` `3` `×` `4` `=` donne 20. Les unités suivent : une longueur multipliée par une longueur donne une aire; une aire multipliée par une longueur, un volume.",
        Some((
            "`5` `Pieds` `3` `Pouces` `×` `11` `Pieds` `6` `Pouces` `1` `/` `2` `=`",
            "60,59375 pi²",
        )),
        "addition soustraction multiplication division plus moins fois",
        &[
            Touche::Additionner,
            Touche::Soustraire,
            Touche::Multiplier,
            Touche::Diviser,
        ],
    ),
    // ------------------------------------------------------------------
    // B.2 Fonctions secondaires (avec Conv)
    // ------------------------------------------------------------------
    secondaire(
        "Ratio",
        "`Conv` `Pente`",
        "Saisit la pente comme un ratio",
        "montée ÷ course : `0,65` `Conv` `Pente`.",
        Some((
            "`0,65` `Conv` `Pente`, puis `Pente`",
            "RATIO DE PENTE 0,65 · PENTE 7 13/16 po",
        )),
        "pente rapport",
        &[Touche::Pente],
    ),
    secondaire(
        "Mur en pente",
        "`Conv` `Montée`",
        "La longueur de chaque montant d'un mur dont le dessus suit la pente du toit",
        "(mur pignon, mur rampant). Saisissez la montée et la course (ou la pente et l'une des deux), puis `Conv` `Montée` et `Montée` à répétition : entraxe, chaque montant, base, angle du mur. Si le mur en pente repose sur un mur de hauteur constante, tapez cette hauteur juste avant `Conv` `Montée` : elle s'ajoute à chaque montant.",
        Some((
            "`4` `Pieds` `Montée` `8` `Pieds` `Course` `Conv` `Montée`, puis `Montée` à répétition",
            "ENTRAXE 16 po · 3 pi 4 po · 2 pi 8 po · 2 pi 0 po · 1 pi 4 po · 0 pi 8 po · ANGLE DU MUR 26,57°",
        )),
        "pignon rampant colombage",
        &[Touche::Montee],
    ),
    secondaire(
        "Polygone",
        "`Conv` `Course`",
        "Une figure régulière à plusieurs côtés",
        "(patio hexagonal, gloriette octogonale). Saisissez le rayon (`15` `Pieds` `Conv` `Arc`), tapez le nombre de côtés, puis `Conv` `Course` : angle intérieur, demi-angle (réglage de coupe), longueur d'un côté, périmètre, aire.",
        Some((
            "`15` `Pieds` `Conv` `Arc` `6` `Conv` `Course`, puis `Course` à répétition",
            "ANGLE INTÉRIEUR 120,00° · DEMI-ANGLE 60,00° · CÔTÉ 15 pi 0 po · PÉRIMÈTRE 90 pi 0 po · AIRE 584,5671 pi²",
        )),
        "hexagone octogone",
        &[Touche::Course],
    ),
    avec_note(
        secondaire(
            "Toiture",
            "`Conv` `Diag`",
            "Les matériaux de couverture.",
            "Saisissez la pente, puis la longueur et la largeur du bâtiment en plan (ou tapez directement une aire en plan), puis `Conv` `Diag` : aire réelle du toit, nombre de carrés (100 pi²), nombre de paquets de bardeaux, nombre de feuilles de 4 × 8.",
            Some((
                "`10` `Pouces` `Pente` `14` `Pieds` `Longueur` `11` `Pieds` `Largeur` `Conv` `Diag`, puis `Diag` à répétition",
                "AIRE DU TOIT 200,4631 pi² · CARRÉS 2,00 · PAQUETS 6,01 · FEUILLES 4×8 6,26",
            )),
            "bardeaux contreplaqué carré paquet",
            &[Touche::Diag],
        ),
        "Les quantités ne sont pas arrondies et n'incluent ni les pertes, ni les débords, ni les noues.",
    ),
    secondaire(
        "Pente irrég.",
        "`Conv` `Arêtier Noue`",
        "La seconde pente d'un toit irrégulier,",
        "dont deux versants de pentes différentes se rencontrent sur un arêtier ou une noue : `8` `Pouces` `Conv` `Arêtier Noue`. Elle est effacée par deux pressions d'`Effacer`.",
        Some((
            "`7` `Pouces` `Pente` `4` `Pieds` `Course` `8` `Pouces` `Conv` `Arêtier Noue`, puis `Arêtier Noue`",
            "PENTE IRRÉGULIÈRE 8 po · ARÊTIER/NOUE IRRÉG. 5 pi 9 11/16 po",
        )),
        "pente irrégulière deux pentes",
        &[Touche::AretierNoue],
    ),
    secondaire(
        "Angle moulure",
        "`Conv` `Onglet composé`",
        "L'angle entre le dos de la moulure et le mur.",
        "Les moulures couronnes courantes sont à 38° ou à 45°. `38` `Conv` `Onglet composé` le mémorise; sans nombre, la touche l'affiche.",
        Some((
            "`38` `Conv` `Onglet composé` `60` `Onglet composé`, puis `Onglet composé`",
            "ANGLE MOULURE 38,00° · ONGLET 46,84° · BISEAU 43,03°",
        )),
        "moulure couronne corniche",
        &[Touche::OngletCompose],
    ),
    secondaire(
        "Contrem. max.",
        "`Conv` `Escalier`",
        "Le même calcul d'escalier, avec une hauteur de contremarche plafonnée.",
        "La contremarche ne dépasse jamais la hauteur visée : la calculatrice ajoute une contremarche au besoin. À utiliser quand une hauteur maximale est imposée.",
        Some((
            "`10` `Pieds` `1` `Pouces` `Montée` `15` `Pieds` `5` `Pouces` `Course` `Conv` `Escalier`, puis `Escalier`",
            "CONTREMARCHE 7 1/8 po · 17 contremarches",
        )),
        "contremarche maximale escalier",
        &[Touche::Escalier],
    ),
    secondaire(
        "Rayon",
        "`Conv` `Arc`",
        "Saisit le rayon d'un cercle,",
        "ou le calcule à partir de la corde et de la flèche d'un arc.",
        Some((
            "`3` `Pieds` `6` `Pouces` `Course` `1` `Pieds` `3` `Pouces` `Montée` `Conv` `Arc`",
            "RAYON 1 pi 10 3/16 po",
        )),
        "cercle arc",
        &[Touche::Arc],
    ),
    secondaire(
        "Colonne/Cône",
        "`Conv` `Cercle`",
        "Volume et surface d'une colonne, puis d'un cône.",
        "Saisissez le diamètre (`Cercle`) et la hauteur (`Hauteur`), puis `Conv` `Cercle` et `Cercle` à répétition. Pratique pour le béton des colonnes et des coffrages tubulaires.",
        Some((
            "`2` `Pieds` `4` `Pouces` `Cercle` `4` `Pieds` `6` `Pouces` `Hauteur` `Conv` `Cercle`, puis `Cercle`",
            "COLONNE – VOLUME 19,24226 pi³ · COLONNE – SURFACE 41,53884 pi²",
        )),
        "cylindre pilier béton",
        &[Touche::Cercle],
    ),
    secondaire(
        "Emp. irrég.",
        "`Conv` `Empannon`",
        "Les empannons du versant à pente irrégulière d'abord.",
        "Dans les Préférences, choisissez de garder le même entraxe sur les deux versants ou d'apparier les empannons pour qu'ils se rencontrent sur l'arêtier.",
        Some((
            "`7` `Pouces` `Pente` `4` `Pieds` `Course` `8` `Pouces` `Conv` `Arêtier Noue` `Conv` `Empannon`, puis `Empannon` à répétition",
            "ENTRAXE 16 po · INCRÉMENT 1 pi 4 13/16 po · 2 pi 9 5/8 po · 1 pi 4 13/16 po…",
        )),
        "empannon irrégulier",
        &[Touche::Empannon],
    ),
    avec_note(
        secondaire(
            "Blocs",
            "`Conv` `Longueur`",
            "Le nombre de blocs de béton ou de briques.",
            "À partir de la longueur et de la hauteur du mur saisies, d'une aire affichée, ou d'une longueur seule (une rangée). Bloc par défaut : 8 × 16 po, joint compris. Pour des briques, mémorisez l'aire d'une brique avec `Stocker` `4`.",
            Some((
                "`300` `Pieds` `Pieds` `Conv` `Longueur`, puis `+` `3` `%`",
                "BLOCS 337,5 · 347,625",
            )),
            "maçonnerie brique",
            &[Touche::Longueur],
        ),
        "Le résultat n'est pas arrondi : ajoutez vos pertes avec `+` `5` `%`.",
    ),
    secondaire(
        "Semelle",
        "`Conv` `Largeur`",
        "Le volume de béton d'une semelle.",
        "Tapez la longueur de la semelle, puis `Conv` `Largeur`. La section de la semelle (largeur × épaisseur) est mémorisée : `16` `Pouces` `×` `8` `Pouces` `=` `Stocker` `6`.",
        Some(("`100` `Pieds` `Conv` `Largeur`", "SEMELLE 6,790123 vg³")),
        "fondation béton footing",
        &[Touche::Largeur],
    ),
    avec_note(
        secondaire(
            "Gypse",
            "`Conv` `Hauteur`",
            "Le nombre de feuilles de gypse",
            "de 4 × 8, 4 × 9, 4 × 10 et 4 × 12 pour une aire, ou pour une longueur de mur si les feuilles sont posées debout.",
            Some((
                "`150` `Pieds` `Pieds` `Conv` `Hauteur`, puis `Hauteur` à répétition",
                "FEUILLES 4×8 4,6875 · 4×9 4,166667 · 4×10 3,75 · 4×12 3,125",
            )),
            "placoplâtre panneau feuille",
            &[Touche::Hauteur],
        ),
        "Le résultat n'est pas arrondi et ne tient pas compte des ouvertures.",
    ),
    secondaire(
        "x²",
        "`Conv` `%`",
        "Le carré.",
        "Le carré d'une longueur donne une aire.",
        Some(("`7` `Pieds` `4` `Pouces` `Conv` `%`", "53,77778 pi²")),
        "carré puissance",
        &[Touche::Pourcent],
    ),
    secondaire(
        "√x",
        "`Conv` `Effacer`",
        "La racine carrée.",
        "La racine d'une aire donne une longueur.",
        Some(("`25` `Pieds` `Pieds` `Conv` `Effacer`", "5 pi 0 po")),
        "racine carrée",
        &[Touche::Effacer],
    ),
    secondaire(
        "cm, mm",
        "`Conv` `7`, `Conv` `9`",
        "Centimètres et millimètres :",
        "pour saisir (`250` `Conv` `9`) ou pour convertir la valeur affichée.",
        Some(("`3` `/` `4` `Conv` `9`", "19,05 mm")),
        "centimètre millimètre métrique",
        &[Touche::Chiffre(7), Touche::Chiffre(9)],
    ),
    secondaire(
        "pmp",
        "`Conv` `8`",
        "Le pied mesure de planche",
        "(144 pouces cubes). Tapez les dimensions nominales d'une pièce en pouces, pouces et pieds : `2` `×` `4` `×` `14` `Conv` `8`. Convertit aussi un volume.",
        Some(("`2` `×` `4` `×` `14` `Conv` `8`", "9,333333 pmp")),
        "bois d'œuvre planche pied-planche",
        &[Touche::Chiffre(8)],
    ),
    secondaire(
        "1/x",
        "`Conv` `÷`",
        "L'inverse",
        "d'un nombre.",
        Some(("`8` `Conv` `÷`", "0,125")),
        "inverse",
        &[Touche::Diviser],
    ),
    secondaire(
        "lb, kg, t courte, t métr.",
        "`Conv` `4`, `1`, `6`, `3`",
        "Les masses.",
        "Pour saisir une masse ou convertir la valeur affichée. Un volume est converti en masse au moyen de la masse volumique mémorisée (et l'inverse). La tonne courte vaut 2 000 lb; la tonne métrique, 1 000 kg.",
        Some((
            "`5` `Verges` `Verges` `Verges` `Conv` `4`, puis `Conv` `6`",
            "15 000 lb · 7,5 tc",
        )),
        "poids béton gravier livre kilogramme tonne",
        &[
            Touche::Chiffre(4),
            Touche::Chiffre(1),
            Touche::Chiffre(6),
            Touche::Chiffre(3),
        ],
    ),
    avec_note(
        secondaire(
            "Montants",
            "`Conv` `5`",
            "Le nombre de montants d'un mur",
            "pour une longueur donnée, à l'entraxe mémorisé, montant de départ compris.",
            Some(("`16` `Pieds` `Conv` `5`", "MONTANTS 13")),
            "colombage stud",
            &[Touche::Chiffre(5)],
        ),
        "N'inclut ni les montants doublés, ni les coins, ni les cadres d'ouverture.",
    ),
    secondaire(
        "Acre",
        "`Conv` `2`",
        "L'acre",
        "(43 560 pi²), pour saisir une superficie ou convertir une aire affichée.",
        Some(("`10000` `Pieds` `Pieds` `Conv` `2`", "0,229568 acre")),
        "terrain superficie",
        &[Touche::Chiffre(2)],
    ),
    secondaire(
        "Coût",
        "`Conv` `0`",
        "Le coût total.",
        "La quantité étant affichée : `×`, le prix unitaire, puis `Conv` `0`. Le prix s'applique à l'unité affichée (par pied, par verge cube, par mètre carré…). Pour le bois en pmp, le prix est celui du millier de pmp.",
        Some((
            "`12` `Pieds` `6` `Pouces` `×` `3,50` `Conv` `0`",
            "COÛT 43,75 $",
        )),
        "prix estimation",
        &[Touche::Chiffre(0)],
    ),
    secondaire(
        "Effacer tout",
        "`Conv` `×`",
        "Remet la calculatrice à neuf :",
        "efface les mémoires et rétablit les valeurs mémorisées par défaut. Les préférences sont conservées. Une confirmation est demandée.",
        None,
        "remise à neuf réinitialiser",
        &[Touche::Multiplier],
    ),
    secondaire(
        "+/−",
        "`Conv` `−`",
        "Change le signe",
        "de la valeur.",
        Some(("`5` `Conv` `−` `=`", "−5")),
        "signe négatif",
        &[Touche::Soustraire],
    ),
    secondaire(
        "π",
        "`Conv` `+`",
        "La constante π",
        "(3,141593).",
        Some(("`Conv` `+`", "3,141593")),
        "pi",
        &[Touche::Additionner],
    ),
    secondaire(
        "dms◄►déc",
        "`Conv` `,`",
        "Convertit un angle",
        "entre les degrés décimaux et les degrés-minutes-secondes. Pour saisir 23° 42′ 39″ : `23` `,` `42` `,` `39`.",
        Some(("`44,29` `Conv` `,`", "44°17′24″")),
        "angle arpentage degrés minutes secondes",
        &[Touche::Virgule],
    ),
    secondaire(
        "Ruban",
        "`Conv` `=`",
        "Affiche le ruban de calcul :",
        "la liste de tout ce qui a été saisi et calculé. Vous pouvez l'enregistrer, le copier ou l'exporter.",
        None,
        "historique",
        &[Touche::Egal],
    ),
    secondaire(
        "Préf.",
        "`Conv` `Stocker`",
        "Ouvre les Préférences.",
        "Les changements s'appliquent immédiatement et sont conservés d'une séance à l'autre.",
        None,
        "préférences réglages",
        &[Touche::Stocker],
    ),
    secondaire(
        "MC",
        "`Conv` `Rappel`",
        "Vide la mémoire cumulative",
        "M+.",
        None,
        "mémoire effacer",
        &[Touche::Rappel],
    ),
    secondaire(
        "M−",
        "`Conv` `M+`",
        "Soustrait",
        "la valeur affichée de la mémoire cumulative.",
        Some(("`100` `M+` `50` `Conv` `M+` `Rappel` `M+`", "TOTAL M 50")),
        "mémoire soustraire",
        &[Touche::MPlus],
    ),
    Fiche {
        titre: "sin, cos, tan",
        acces: "mode trigonométrique",
        resume: "Les fonctions trigonométriques,",
        mode_emploi: "en degrés. Avec `Conv` : les fonctions inverses, qui donnent un angle. Dans ce mode, elles remplacent les touches `Longueur`, `Largeur` et `Hauteur`.",
        exemple: Some(("`30` `sin` · `0,5` `Conv` `sin`", "0,5 · 30,00°")),
        note: None,
        mots_cles: "trigonométrie sinus cosinus tangente",
        touches: &[Touche::Longueur, Touche::Largeur, Touche::Hauteur],
        secondaire: false,
        trigo: true,
    },
    Fiche {
        titre: "Fractions rapides",
        acces: "bande de touches",
        resume: "Une bande de touches",
        mode_emploi: "qui saisit une fraction de pouce d'un seul geste : `5` `Pieds` `6` puis `3/8`. S'active dans les Préférences.",
        exemple: Some(("`5` `Pieds` `6` `3/8` `=`", "5 pi 6 3/8 po")),
        note: None,
        mots_cles: "fraction rapide seizième",
        touches: &[],
        secondaire: false,
        trigo: false,
    },
];

/// Fiche de la fonction principale d'une touche.
pub fn fiche_principale(touche: Touche, trigo: bool) -> Option<&'static Fiche> {
    let touche = match touche {
        Touche::FractionRapide { .. } => {
            return FICHES.iter().find(|f| f.titre == "Fractions rapides");
        }
        autre => autre,
    };
    if trigo
        && let Some(f) = FICHES
            .iter()
            .find(|f| f.trigo && f.touches.contains(&touche))
    {
        return Some(f);
    }
    FICHES
        .iter()
        .find(|f| !f.secondaire && !f.trigo && f.touches.contains(&touche))
}

/// Fiche de la fonction secondaire d'une touche (avec `Conv`).
pub fn fiche_secondaire(touche: Touche, trigo: bool) -> Option<&'static Fiche> {
    // En mode trigonométrique, les fonctions inverses sont décrites dans la même fiche.
    if trigo && matches!(touche, Touche::Longueur | Touche::Largeur | Touche::Hauteur) {
        return None;
    }
    FICHES
        .iter()
        .find(|f| f.secondaire && f.touches.contains(&touche))
}

/// Minuscules sans accents, pour une recherche tolérante (section 15.3).
pub fn normaliser(texte: &str) -> String {
    texte
        .to_lowercase()
        .chars()
        .filter(|c| *c != '`')
        .map(|c| match c {
            'à' | 'â' | 'ä' => 'a',
            'ç' => 'c',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'î' | 'ï' => 'i',
            'ô' | 'ö' => 'o',
            'ù' | 'û' | 'ü' => 'u',
            'œ' => 'o',
            '\u{00A0}' => ' ',
            autre => autre,
        })
        .collect()
}

impl Fiche {
    /// Vrai si la fiche contient tous les mots de la requête, sans égard à la
    /// casse ni aux accents, dans son titre, ses mots-clés ou son texte.
    pub fn correspond(&self, requete: &str) -> bool {
        let contenu = normaliser(&format!(
            "{} {} {} {} {} {}",
            self.titre,
            self.mots_cles,
            self.resume,
            self.mode_emploi,
            self.acces,
            self.note.unwrap_or("")
        ));
        normaliser(requete)
            .split_whitespace()
            .all(|mot| contenu.contains(mot))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Fiches correspondant à une recherche; toutes si la requête est vide.
    fn chercher(requete: &str) -> Vec<&'static Fiche> {
        FICHES.iter().filter(|f| f.correspond(requete)).collect()
    }

    #[test]
    fn la_recherche_ignore_la_casse_et_les_accents() {
        let titres = |r: &str| chercher(r).iter().map(|f| f.titre).collect::<Vec<_>>();
        assert!(titres("arete").contains(&"Arêtier Noue"));
        assert!(titres("ARÊTIER").contains(&"Arêtier Noue"));
        assert!(titres("beton").contains(&"Semelle"));
        assert!(titres("stud").contains(&"Montants"));
        assert!(titres("scie biseau").contains(&"Onglet composé"));
        assert!(titres("zzzz").is_empty());
        assert_eq!(chercher("").len(), FICHES.len());
    }

    #[test]
    fn chaque_touche_de_la_grille_a_sa_fiche() {
        for touche in Touche::GRILLE.iter().flatten().copied() {
            assert!(
                fiche_principale(touche, false).is_some(),
                "fiche principale de {touche:?}"
            );
            if touche.a_une_fonction_secondaire() {
                assert!(
                    fiche_secondaire(touche, false).is_some(),
                    "fiche secondaire de {touche:?}"
                );
            }
        }
        assert!(fiche_principale(Touche::RetourArriere, false).is_some());
        assert!(fiche_principale(Touche::FractionRapide { num: 3, den: 8 }, false).is_some());
        assert_eq!(
            fiche_principale(Touche::Longueur, true).map(|f| f.titre),
            Some("sin, cos, tan")
        );
        assert_eq!(
            fiche_principale(Touche::Longueur, false).map(|f| f.titre),
            Some("Longueur")
        );
    }

    #[test]
    fn les_exemples_citent_des_frappes_et_un_resultat() {
        for f in FICHES {
            assert!(!f.titre.is_empty() && !f.resume.is_empty() && !f.mode_emploi.is_empty());
            if let Some((frappes, resultat)) = f.exemple {
                assert!(frappes.contains('`'), "exemple de « {} »", f.titre);
                assert!(!resultat.is_empty());
            }
            // Les accents graves vont par paires.
            for texte in [f.mode_emploi, f.acces, f.note.unwrap_or("")] {
                assert_eq!(
                    texte.matches('`').count() % 2,
                    0,
                    "accents graves de « {} »",
                    f.titre
                );
            }
        }
    }
}
