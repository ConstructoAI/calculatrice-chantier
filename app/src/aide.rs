//! Contenu des fiches d'aide (annexe B du cahier des charges) et recherche.
//!
//! Dans les textes, les accents graves entourent le nom d'une touche : l'écran
//! d'aide les dessine comme de petites touches.
//!
//! Chaque fiche existe en français et en anglais (section 24); en anglais, les
//! touches portent leur nom anglais et les nombres s'écrivent à l'anglaise.

use noyau::Touche;
use noyau::langue::{Texte, t, tel};

/// Une fiche d'aide.
pub struct Fiche {
    pub titre: Texte,
    /// Frappes qui donnent accès à la fonction (fonctions secondaires).
    pub acces: Texte,
    /// « À quoi ça sert ».
    pub resume: Texte,
    /// « Comment faire ».
    pub mode_emploi: Texte,
    /// Exemple chiffré : frappes, puis résultat.
    pub exemple: Option<(Texte, Texte)>,
    pub note: Option<Texte>,
    /// Mots-clés de recherche, non affichés : les mots français, puis les
    /// mots anglais, pour que la recherche réponde aux deux langues.
    pub mots_cles: &'static str,
    /// Touches dont c'est la fiche.
    pub touches: &'static [Touche],
    /// Fiche d'une fonction secondaire (avec `Conv`).
    pub secondaire: bool,
    /// Fiche propre au mode trigonométrique.
    pub trigo: bool,
}

const fn fiche(
    titre: Texte,
    resume: Texte,
    mode_emploi: Texte,
    exemple: Option<(Texte, Texte)>,
    mots_cles: &'static str,
    touches: &'static [Touche],
) -> Fiche {
    Fiche {
        titre,
        acces: Texte::VIDE,
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
    titre: Texte,
    acces: Texte,
    resume: Texte,
    mode_emploi: Texte,
    exemple: Option<(Texte, Texte)>,
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

const fn avec_note(mut f: Fiche, note: Texte) -> Fiche {
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
        t("Pente", "Pitch"),
        t(
            "La pente d'un toit ou d'un plan incliné.",
            "The pitch of a roof or of an inclined plane.",
        ),
        t(
            "Elle s'exprime en pouces de montée pour 12 pouces de course (7 po = pente 7/12), en degrés, en pourcentage ou en ratio. Pour la saisir : `7` `Pouces` `Pente` (7/12), `30` `Pente` (30 degrés) ou `75` `%` `Pente` (75 %). Pour la calculer : saisissez deux valeurs parmi la montée, la course et la diagonale, puis appuyez sur `Pente`. Appuyez de nouveau pour la voir sous ses autres formes. La pente saisie reste en mémoire jusqu'à ce que vous en saisissiez une autre.",
            "It is expressed in inches of rise per 12 inches of run (7 in = a 7/12 pitch), in degrees, as a percentage or as a ratio. To enter it: `7` `Inch` `Pitch` (7/12), `30` `Pitch` (30 degrees) or `75` `%` `Pitch` (75%). To calculate it: enter two values among the rise, the run and the diagonal, then press `Pitch`. Press again to see it in its other forms. The pitch you enter stays in memory until you enter another one.",
        ),
        Some((
            t(
                "`30,45` `Pente`, puis `Pente` à répétition",
                "`30.45` `Pitch`, then `Pitch` repeatedly",
            ),
            t(
                "30,45° · 58,78702 % · ratio 0,58787 · 7 1/16 po",
                "30.45° · 58.78702% · ratio 0.58787 · 7 1/16 in",
            ),
        )),
        "toit inclinaison degrés pourcentage pitch roof slope degrees percent",
        &[Touche::Pente],
    ),
    fiche(
        t("Montée", "Rise"),
        t(
            "Le côté vertical d'un triangle rectangle.",
            "The vertical side of a right triangle.",
        ),
        t(
            "C'est la hauteur d'un toit de la sablière au faîte, la hauteur totale d'un escalier d'un plancher fini à l'autre, ou la flèche d'un arc. Tapez la mesure puis `Montée` pour la saisir. Pour la calculer, saisissez deux autres valeurs (pente, course ou diagonale) et appuyez sur `Montée`.",
            "It is the height of a roof from the top plate to the ridge, the total height of a stair from one finished floor to the next, or the rise of an arc. Type the measurement then `Rise` to enter it. To calculate it, enter two other values (pitch, run or diagonal) and press `Rise`.",
        ),
        Some((
            t(
                "`7` `Pouces` `Pente` `11` `Pieds` `6` `Pouces` `Course` `Montée`",
                "`7` `Inch` `Pitch` `11` `Feet` `6` `Inch` `Run` `Rise`",
            ),
            t("MONTÉE 6 pi 8 1/2 po", "RISE 6 ft 8 1/2 in"),
        )),
        "hauteur vertical rise height",
        &[Touche::Montee],
    ),
    fiche(
        t("Course", "Run"),
        t(
            "Le côté horizontal d'un triangle rectangle.",
            "The horizontal side of a right triangle.",
        ),
        t(
            "C'est la demi-portée d'un toit (du mur extérieur au centre du faîte), la longueur au sol d'un escalier, ou la corde d'un arc. Tapez la mesure puis `Course`, ou calculez-la à partir de deux autres valeurs.",
            "It is half the span of a roof (from the outside wall to the centre of the ridge), the length of a stair along the floor, or the chord of an arc. Type the measurement then `Run`, or calculate it from two other values.",
        ),
        Some((
            t(
                "`6` `Pouces` `Pente` `4` `Pieds` `Diag` `Course`",
                "`6` `Inch` `Pitch` `4` `Feet` `Diag` `Run`",
            ),
            t("COURSE 3 pi 6 15/16 po", "RUN 3 ft 6 15/16 in"),
        )),
        "horizontal portée run span",
        &[Touche::Course],
    ),
    avec_note(
        fiche(
            tel("Diag"),
            t("La diagonale (hypoténuse).", "The diagonal (hypotenuse)."),
            t(
                "C'est la longueur du chevron commun, ou la diagonale qui sert à vérifier l'équerrage d'une fondation. Appuyez de nouveau pour obtenir la coupe d'aplomb et la coupe de niveau.",
                "It is the length of the common rafter, or the diagonal used to check that a foundation is square. Press again for the plumb cut and the level cut.",
            ),
            Some((
                t(
                    "`7` `Pouces` `Pente` `7` `Pieds` `2` `Pouces` `Course` `Diag`, puis `Diag` deux fois",
                    "`7` `Inch` `Pitch` `7` `Feet` `2` `Inch` `Run` `Diag`, then `Diag` twice",
                ),
                t(
                    "DIAGONALE 8 pi 3 9/16 po · COUPE D'APLOMB 30,26° · COUPE DE NIVEAU 59,74°",
                    "DIAGONAL 8 ft 3 9/16 in · PLUMB CUT 30.26° · LEVEL CUT 59.74°",
                ),
            )),
            "chevron commun hypoténuse équerre common rafter hypotenuse diagonal square",
            &[Touche::Diag],
        ),
        t(
            "La longueur est mesurée de pointe à pointe : ajoutez le débord du toit et retranchez la demi-épaisseur de la faîtière au besoin.",
            "The length is measured point to point: add the roof overhang and subtract half the thickness of the ridge board as needed.",
        ),
    ),
    avec_note(
        fiche(
            t("Arêtier Noue", "Hip Valley"),
            t(
                "La longueur du chevron d'arêtier ou de noue et ses coupes.",
                "The length of the hip or valley rafter and its cuts.",
            ),
            t(
                "Saisissez la pente et la course du chevron commun, puis appuyez sur `Arêtier Noue`. Appuyez de nouveau pour la coupe d'aplomb, la coupe de niveau et la coupe latérale (biseau). Pour un toit dont les deux versants ont des pentes différentes, saisissez d'abord la pente irrégulière.",
                "Enter the pitch and the run of the common rafter, then press `Hip Valley`. Press again for the plumb cut, the level cut and the cheek cut (bevel). For a roof whose two sides have different pitches, enter the irregular pitch first.",
            ),
            Some((
                t(
                    "`8` `Pieds` `5` `Pouces` `Course` `7` `Pouces` `Pente` `Arêtier Noue`, puis `Arêtier Noue` à répétition",
                    "`8` `Feet` `5` `Inch` `Run` `7` `Inch` `Pitch` `Hip Valley`, then `Hip Valley` repeatedly",
                ),
                t(
                    "ARÊTIER/NOUE 12 pi 10 1/2 po · 22,42° · 67,58° · 45,00°",
                    "HIP/VALLEY 12 ft 10 1/2 in · 22.42° · 67.58° · 45.00°",
                ),
            )),
            "croupe hip valley arête",
            &[Touche::AretierNoue],
        ),
        t(
            "Les longueurs sont théoriques, d'axe en axe.",
            "Lengths are theoretical, centreline to centreline.",
        ),
    ),
    fiche(
        t("Onglet composé", "Compound Mitre"),
        t(
            "Les réglages de la scie à onglets pour une moulure couronne coupée à plat.",
            "The mitre saw settings for a crown moulding cut flat.",
        ),
        t(
            "Tapez l'angle du coin (90 pour un coin d'équerre), puis `Onglet composé` : vous obtenez l'angle d'onglet (rotation de la table) et l'angle de biseau (inclinaison de la lame). Pour un ouvrage à plusieurs côtés égaux, tapez plutôt le nombre de côtés (toute valeur inférieure à 25 est prise pour un nombre de côtés). Vérifiez d'abord que l'angle de la moulure mémorisé correspond à votre moulure.",
            "Type the corner angle (90 for a square corner), then `Compound Mitre`: you get the mitre angle (table rotation) and the bevel angle (blade tilt). For a piece with several equal sides, type the number of sides instead (any value under 25 is taken as a number of sides). First check that the stored crown angle matches your moulding.",
        ),
        Some((
            t(
                "`90` `Onglet composé`, puis `Onglet composé`",
                "`90` `Compound Mitre`, then `Compound Mitre`",
            ),
            t(
                "ONGLET 35,26° · BISEAU 30,00°",
                "MITRE 35.26° · BEVEL 30.00°",
            ),
        )),
        "corniche moulure biseau scie crown moulding molding bevel saw mitre miter",
        &[Touche::OngletCompose],
    ),
    avec_note(
        fiche(
            t("Escalier", "Stair"),
            t(
                "Le calcul complet d'un escalier droit.",
                "The complete layout of a straight stair.",
            ),
            t(
                "Saisissez la hauteur à monter avec `Montée`, la longueur disponible avec `Course`, ou les deux, puis appuyez sur `Escalier` à répétition : hauteur et nombre de contremarches, giron et nombre de marches, écarts d'arrondi, longueur de la trémie, longueur du limon, inclinaison. La hauteur de contremarche et le giron visés, l'échappée et l'épaisseur du plancher se règlent dans les Préférences. Un symbole d'avertissement signale une contremarche plus haute que la valeur visée.",
                "Enter the height to climb with `Rise`, the length available with `Run`, or both, then press `Stair` repeatedly: riser height and number of risers, tread depth and number of treads, rounding differences, stairwell opening length, stringer length, incline. The target riser height and tread depth, the headroom and the floor thickness are set in Preferences. A warning symbol flags a riser higher than the target value.",
            ),
            Some((
                t(
                    "`9` `Pieds` `11` `Pouces` `Montée`, puis `Escalier` à répétition",
                    "`9` `Feet` `11` `Inch` `Rise`, then `Stair` repeatedly",
                ),
                t(
                    "CONTREMARCHE 7 7/16 po · 16 contremarches · GIRON 10 po · 15 marches · TRÉMIE 10 pi 1 po · LIMON 15 pi 6 15/16 po · 36,64°",
                    "RISER HEIGHT 7 7/16 in · 16 risers · TREAD DEPTH 10 in · 15 treads · STAIRWELL OPENING 10 ft 1 in · STRINGER 15 ft 6 15/16 in · 36.64°",
                ),
            )),
            "marche contremarche giron limon trémie stair step riser tread stringer stairwell",
            &[Touche::Escalier],
        ),
        t(
            "Le giron n'inclut pas le nez de marche. Vérifiez toujours les exigences du Code de construction.",
            "The tread depth does not include the nosing. Always check the requirements of the building code.",
        ),
    ),
    fiche(
        tel("Arc"),
        t(
            "Tout ce qui concerne un arc de cercle.",
            "Everything about a circular arc.",
        ),
        t(
            "Deux données suffisent : le diamètre et l'angle, le diamètre et la longueur d'arc, ou encore la corde (touche `Course`) et la flèche (touche `Montée`). Appuyez sur `Arc` à répétition : angle, longueur d'arc, corde, aire du segment, aire du secteur, flèche, puis la longueur de chaque montant d'un mur cintré à l'entraxe mémorisé.",
            "Two values are enough: the diameter and the angle, the diameter and the arc length, or the chord (`Run` key) and the arc rise (`Rise` key). Press `Arc` repeatedly: angle, arc length, chord, segment area, sector area, arc rise, then the length of each stud of an arched wall at the stored on-centre spacing.",
        ),
        Some((
            t(
                "`5` `Pieds` `Cercle` `85` `Arc`, puis `Arc`",
                "`5` `Feet` `Circle` `85` `Arc`, then `Arc`",
            ),
            t(
                "ARC 85,00° · ARC 3 pi 8 1/2 po",
                "ARC 85.00° · ARC 3 ft 8 1/2 in",
            ),
        )),
        "cintre arche fenêtre cintrée corde flèche arch arched window chord",
        &[Touche::Arc],
    ),
    fiche(
        t("Cercle", "Circle"),
        t(
            "Aire et circonférence d'un cercle.",
            "Area and circumference of a circle.",
        ),
        t(
            "Tapez le diamètre, puis `Cercle`; appuyez de nouveau pour l'aire, puis la circonférence.",
            "Type the diameter, then `Circle`; press again for the area, then the circumference.",
        ),
        Some((
            t(
                "`11` `Pouces` `Cercle`, puis `Cercle` deux fois",
                "`11` `Inch` `Circle`, then `Circle` twice",
            ),
            t(
                "AIRE 95,03318 po² · CIRCONFÉRENCE 34 9/16 po",
                "AREA 95.03318 in² · CIRCUMFERENCE 34 9/16 in",
            ),
        )),
        "diamètre rond diameter round circle",
        &[Touche::Cercle],
    ),
    fiche(
        t("Empannon", "Jack"),
        t(
            "La longueur de chaque empannon.",
            "The length of each jack rafter.",
        ),
        t(
            "Les empannons sont les chevrons raccourcis qui s'appuient sur l'arêtier ou la noue. Saisissez la pente et la course, puis appuyez sur `Empannon` à répétition : entraxe, incrément (différence de longueur d'un empannon au suivant), chaque longueur, puis les coupes. L'ordre (décroissant ou croissant) se choisit dans les Préférences.",
            "Jack rafters are the shortened rafters that land on the hip or the valley. Enter the pitch and the run, then press `Jack` repeatedly: on-centre spacing, increment (the difference in length from one jack to the next), each length, then the cuts. The order (descending or ascending) is chosen in Preferences.",
        ),
        Some((
            t(
                "`8` `Pieds` `5` `Pouces` `Course` `7` `Pouces` `Pente`, puis `Empannon` à répétition",
                "`8` `Feet` `5` `Inch` `Run` `7` `Inch` `Pitch`, then `Jack` repeatedly",
            ),
            t(
                "ENTRAXE 16 po · INCRÉMENT 1 pi 6 1/2 po · 8 pi 2 3/8 po · 6 pi 7 7/8 po · 5 pi 1 3/8 po…",
                "ON CENTRE 16 in · INCREMENT 1 ft 6 1/2 in · 8 ft 2 3/8 in · 6 ft 7 7/8 in · 5 ft 1 3/8 in…",
            ),
        )),
        "chevron jack rafter",
        &[Touche::Empannon],
    ),
    fiche(
        tel("m"),
        t("Mètres.", "Metres."),
        t(
            "Tapez un nombre puis `m`. Deux pressions donnent des mètres carrés, trois des mètres cubes. Sur une valeur déjà affichée, `m` la convertit en mètres.",
            "Type a number then `m`. Two presses give square metres, three give cubic metres. With a value already displayed, `m` converts it to metres.",
        ),
        Some((
            t("`8` `Pieds` `Conv` `m`", "`8` `Feet` `Conv` `m`"),
            t("2,438 m", "2.438 m"),
        )),
        "métrique mètre metric metre meter",
        &[Touche::Metres],
    ),
    fiche(
        t("Longueur", "Length"),
        t("Mémorise une longueur", "Stores a length"),
        t(
            "pour les calculs d'aire, de volume et de matériaux (blocs, semelle, gypse, toiture).",
            "for area, volume and material calculations (blocks, footing, drywall, roofing).",
        ),
        Some((
            t(
                "`20` `Pieds` `6` `Pouces` `Longueur`",
                "`20` `Feet` `6` `Inch` `Length`",
            ),
            t("LONGUEUR 20 pi 6 po", "LENGTH 20 ft 6 in"),
        )),
        "dimension length",
        &[Touche::Longueur],
    ),
    fiche(
        t("Largeur", "Width"),
        t("Mémorise la largeur,", "Stores the width,"),
        t(
            "puis donne, à chaque pression : l'aire, la diagonale d'équerrage et le périmètre du rectangle.",
            "then gives, with each press: the area, the square-up diagonal and the perimeter of the rectangle.",
        ),
        Some((
            t(
                "`20` `Pieds` `6` `Pouces` `Longueur` `25` `Pieds` `6` `Pouces` `Largeur`, puis `Largeur` à répétition",
                "`20` `Feet` `6` `Inch` `Length` `25` `Feet` `6` `Inch` `Width`, then `Width` repeatedly",
            ),
            t(
                "AIRE 522,75 pi² · ÉQUERRAGE 32 pi 8 5/8 po · PÉRIMÈTRE 92 pi 0 po",
                "AREA 522.75 ft² · SQUARE-UP 32 ft 8 5/8 in · PERIMETER 92 ft 0 in",
            ),
        )),
        "aire équerrage périmètre area square perimeter width",
        &[Touche::Largeur],
    ),
    fiche(
        t("Hauteur", "Height"),
        t("Mémorise la hauteur,", "Stores the height,"),
        t(
            "puis donne, à chaque pression : le volume, l'aire des quatre murs et l'aire de la pièce (murs et plafond), utile pour la peinture et le gypse.",
            "then gives, with each press: the volume, the area of the four walls and the room area (walls and ceiling), handy for paint and drywall.",
        ),
        Some((
            t(
                "`15` `Pieds` `Longueur` `20` `Pieds` `Largeur` `12` `Pieds` `Hauteur`, puis `Hauteur` à répétition",
                "`15` `Feet` `Length` `20` `Feet` `Width` `12` `Feet` `Height`, then `Height` repeatedly",
            ),
            t(
                "VOLUME 3 600 pi³ · MURS 840 pi² · PIÈCE 1 140 pi²",
                "VOLUME 3,600 ft³ · WALLS 840 ft² · ROOM 1,140 ft²",
            ),
        )),
        "volume murs walls height room",
        &[Touche::Hauteur],
    ),
    fiche(
        tel("%"),
        t("Pourcentages.", "Percentages."),
        t(
            "`+` `10` `%` ajoute 10 % à la valeur, `−` `20` `%` en retranche 20 %, `×` `18` `%` en calcule 18 %, `÷` `80` `%` la divise par 0,80. Fonctionne aussi avec des mesures. Sert enfin à saisir une pente en pourcentage : `2` `%` `Pente`.",
            "`+` `10` `%` adds 10% to the value, `−` `20` `%` takes 20% off, `×` `18` `%` calculates 18% of it, `÷` `80` `%` divides it by 0.80. It also works with measurements. Finally, it is used to enter a pitch as a percentage: `2` `%` `Pitch`.",
        ),
        Some((
            t(
                "`137` `Pieds` `Pieds` `+` `10` `%`",
                "`137` `Feet` `Feet` `+` `10` `%`",
            ),
            t("150,7 pi²", "150.7 ft²"),
        )),
        "pertes rabais majoration pourcent waste discount markup percent",
        &[Touche::Pourcent],
    ),
    fiche(
        t("Verges, Pieds, Pouces", "Yards, Feet, Inch"),
        t("Les unités impériales.", "The imperial units."),
        t(
            "Tapez le nombre, puis l'unité : `5` `Pieds` `7` `Pouces`. Deux pressions de la même unité donnent des unités carrées, trois des unités cubes. Sur une valeur affichée, la touche convertit; `Pieds` alterne entre pieds-pouces et pieds décimaux, `Pouces` entre fraction et décimales.",
            "Type the number, then the unit: `5` `Feet` `7` `Inch`. Two presses of the same unit give square units, three give cubic units. With a value displayed, the key converts; `Feet` toggles between feet-inches and decimal feet, `Inch` between fraction and decimals.",
        ),
        Some((
            t(
                "`5` `Pieds` `7` `Pouces` `1` `/` `2` `+` `18` `Pieds` `8` `Pouces` `=`",
                "`5` `Feet` `7` `Inch` `1` `/` `2` `+` `18` `Feet` `8` `Inch` `=`",
            ),
            t("24 pi 3 1/2 po", "24 ft 3 1/2 in"),
        )),
        "impérial conversion verge pied pouce imperial yard foot feet inch",
        &[Touche::Verges, Touche::Pieds, Touche::Pouces],
    ),
    fiche(
        tel("/"),
        t("La barre de fraction.", "The fraction bar."),
        t(
            "`1` `/` `2` donne un demi-pouce. Si vous ne tapez pas le dénominateur, la résolution en vigueur est utilisée : `15` `/` vaut 15/16.",
            "`1` `/` `2` gives half an inch. If you do not type the denominator, the current resolution is used: `15` `/` means 15/16.",
        ),
        Some((tel("`15` `/` `=`"), t("15/16 po", "15/16 in"))),
        "fraction",
        &[Touche::Fraction],
    ),
    fiche(
        t("Effacer", "Clear"),
        t(
            "Une pression efface l'affichage et l'erreur éventuelle.",
            "One press clears the display and any error.",
        ),
        t(
            "Deux pressions effacent aussi les valeurs temporaires (montée, course, longueur, diamètre…). La pente saisie, les mémoires et les valeurs mémorisées sont conservées.",
            "Two presses also clear the temporary values (rise, run, length, diameter…). The entered pitch, the memories and the stored values are kept.",
        ),
        Some((
            t(
                "`5` `+` `3` `Effacer` `4` `=`",
                "`5` `+` `3` `Clear` `4` `=`",
            ),
            tel("9"),
        )),
        "remise à zéro clear reset",
        &[Touche::Effacer],
    ),
    fiche(
        tel("Conv"),
        t(
            "La touche de conversion et de seconde fonction.",
            "The conversion and second-function key.",
        ),
        t(
            "Suivie d'une touche, elle exécute la fonction écrite en petit au-dessus de cette touche. Suivie d'une unité, elle convertit la valeur affichée. Vous pouvez aussi cliquer directement sur la petite étiquette.",
            "Followed by a key, it runs the function written in small type above that key. Followed by a unit, it converts the displayed value. You can also click the small label directly.",
        ),
        Some((
            t("`14` `Pieds` `Conv` `Verges`", "`14` `Feet` `Conv` `Yards`"),
            t("4,666667 vg", "4.666667 yd"),
        )),
        "convertir seconde fonction convert second function",
        &[Touche::Conv],
    ),
    fiche(
        t("Stocker", "Store"),
        t("Met une valeur en mémoire.", "Stores a value in memory."),
        t(
            "`Stocker` `1`, `2` ou `3` : trois registres libres. `Stocker` `4` à `9` et `0` : les valeurs mémorisées de la calculatrice (bloc, entraxe, section de semelle, contremarche visée, épaisseur du plancher, giron visé, masse volumique).",
            "`Store` `1`, `2` or `3`: three free registers. `Store` `4` to `9` and `0`: the calculator's stored values (block, on-centre spacing, footing section, target riser, floor thickness, target tread, density).",
        ),
        Some((
            t("`18` `Pouces` `Stocker` `5`", "`18` `Inch` `Store` `5`"),
            t("ENTRAXE 18 po MÉM.", "ON CENTRE 18 in MEM"),
        )),
        "mémoire memory store recall",
        &[Touche::Stocker],
    ),
    fiche(
        t("Rappel", "Recall"),
        t("Rappelle une valeur.", "Recalls a value."),
        t(
            "Suivie d'une touche, elle affiche ce que cette touche a en mémoire : `Rappel` `5` montre l'entraxe, `Rappel` `Course` la dernière course. La valeur rappelée peut servir dans un calcul.",
            "Followed by a key, it shows what that key holds in memory: `Recall` `5` shows the on-centre spacing, `Recall` `Run` the last run. The recalled value can be used in a calculation.",
        ),
        Some((
            t("`Rappel` `7`", "`Recall` `7`"),
            t(
                "CONTREMARCHE VISÉE 7 1/2 po MÉM.",
                "TARGET RISER 7 1/2 in MEM",
            ),
        )),
        "mémoire memory store recall",
        &[Touche::Rappel],
    ),
    fiche(
        tel("M+"),
        t("La mémoire cumulative.", "The cumulative memory."),
        t(
            "Chaque pression ajoute la valeur affichée au total. `Rappel` `M+` montre le total; `M+` ensuite montre la moyenne, puis le nombre de valeurs. `Rappel` `Rappel` montre le total et vide la mémoire.",
            "Each press adds the displayed value to the total. `Recall` `M+` shows the total; `M+` after that shows the average, then the number of values. `Recall` `Recall` shows the total and clears the memory.",
        ),
        Some((
            t(
                "`100` `M+` `200` `M+` `50` `Conv` `M+` `Rappel` `M+`, puis `M+` deux fois",
                "`100` `M+` `200` `M+` `50` `Conv` `M+` `Recall` `M+`, then `M+` twice",
            ),
            t(
                "TOTAL M 250 · MOYENNE 83,33333 · NOMBRE 3",
                "M TOTAL 250 · AVERAGE 83.33333 · COUNT 3",
            ),
        )),
        "total somme moyenne sum average memory",
        &[Touche::MPlus],
    ),
    fiche(
        tel("="),
        t("Termine le calcul.", "Completes the calculation."),
        t(
            "Après une touche de fonction, `=` rend le résultat disponible pour une autre fonction.",
            "After a function key, `=` makes the result available to another function.",
        ),
        Some((tel("`2` `+` `3` `×` `4` `=`"), tel("20"))),
        "égal résultat equals result",
        &[Touche::Egal],
    ),
    fiche(
        tel("⌫"),
        t("Retour arrière :", "Backspace:"),
        t(
            "annule la dernière frappe de la saisie en cours.",
            "undoes the last keystroke of the current entry.",
        ),
        Some((
            t(
                "`12` `3` `⌫` `⌫` `7` `Pouces` `=`",
                "`12` `3` `⌫` `⌫` `7` `Inch` `=`",
            ),
            t("17 po", "17 in"),
        )),
        "corriger effacer backspace correct delete",
        &[Touche::RetourArriere],
    ),
    fiche(
        t("Chiffres et virgule", "Digits and decimal point"),
        t("La saisie des nombres.", "Entering numbers."),
        t(
            "Tapez les chiffres, puis l'unité s'il y a lieu. La virgule sépare les décimales. Un nombre compte au plus 12 chiffres. Dès qu'un résultat est affiché, le premier chiffre tapé commence une nouvelle valeur.",
            "Type the digits, then the unit if there is one. The point separates the decimals. A number has at most 12 digits. Once a result is displayed, the first digit you type starts a new value.",
        ),
        Some((
            t("`17,32` `Pieds` `=`", "`17.32` `Feet` `=`"),
            t("17,32 pi", "17.32 ft"),
        )),
        "nombre décimale number decimal digit",
        CHIFFRES,
    ),
    fiche(
        tel("+  −  ×  ÷"),
        t("Les quatre opérations.", "The four operations."),
        t(
            "Les calculs se font dans l'ordre de la frappe, sans priorité des opérateurs : `2` `+` `3` `×` `4` `=` donne 20. Les unités suivent : une longueur multipliée par une longueur donne une aire; une aire multipliée par une longueur, un volume.",
            "Calculations are done in the order you key them, with no operator precedence: `2` `+` `3` `×` `4` `=` gives 20. Units follow along: a length times a length gives an area; an area times a length, a volume.",
        ),
        Some((
            t(
                "`5` `Pieds` `3` `Pouces` `×` `11` `Pieds` `6` `Pouces` `1` `/` `2` `=`",
                "`5` `Feet` `3` `Inch` `×` `11` `Feet` `6` `Inch` `1` `/` `2` `=`",
            ),
            t("60,59375 pi²", "60.59375 ft²"),
        )),
        "addition soustraction multiplication division plus moins fois add subtract multiply divide minus times",
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
        tel("Ratio"),
        t("`Conv` `Pente`", "`Conv` `Pitch`"),
        t(
            "Saisit la pente comme un ratio",
            "Enters the pitch as a ratio",
        ),
        t(
            "montée ÷ course : `0,65` `Conv` `Pente`.",
            "of rise ÷ run: `0.65` `Conv` `Pitch`.",
        ),
        Some((
            t(
                "`0,65` `Conv` `Pente`, puis `Pente`",
                "`0.65` `Conv` `Pitch`, then `Pitch`",
            ),
            t(
                "RATIO DE PENTE 0,65 · PENTE 7 13/16 po",
                "PITCH RATIO 0.65 · PITCH 7 13/16 in",
            ),
        )),
        "pente rapport pitch ratio slope",
        &[Touche::Pente],
    ),
    secondaire(
        t("Mur en pente", "Rake wall"),
        t("`Conv` `Montée`", "`Conv` `Rise`"),
        t(
            "La longueur de chaque montant d'un mur dont le dessus suit la pente du toit",
            "The length of each stud in a wall whose top follows the roof pitch",
        ),
        t(
            "(mur pignon, mur rampant). Saisissez la montée et la course (ou la pente et l'une des deux), puis `Conv` `Montée` et `Montée` à répétition : entraxe, chaque montant, base, angle du mur. Si le mur en pente repose sur un mur de hauteur constante, tapez cette hauteur juste avant `Conv` `Montée` : elle s'ajoute à chaque montant.",
            "(gable wall, rake wall). Enter the rise and the run (or the pitch and one of the two), then `Conv` `Rise` and `Rise` repeatedly: on-centre spacing, each stud, base, wall angle. If the rake wall sits on a wall of constant height, type that height just before `Conv` `Rise`: it is added to each stud.",
        ),
        Some((
            t(
                "`4` `Pieds` `Montée` `8` `Pieds` `Course` `Conv` `Montée`, puis `Montée` à répétition",
                "`4` `Feet` `Rise` `8` `Feet` `Run` `Conv` `Rise`, then `Rise` repeatedly",
            ),
            t(
                "ENTRAXE 16 po · 3 pi 4 po · 2 pi 8 po · 2 pi 0 po · 1 pi 4 po · 0 pi 8 po · ANGLE DU MUR 26,57°",
                "ON CENTRE 16 in · 3 ft 4 in · 2 ft 8 in · 2 ft 0 in · 1 ft 4 in · 0 ft 8 in · WALL ANGLE 26.57°",
            ),
        )),
        "pignon rampant colombage gable rake wall studs",
        &[Touche::Montee],
    ),
    secondaire(
        t("Polygone", "Polygon"),
        t("`Conv` `Course`", "`Conv` `Run`"),
        t(
            "Une figure régulière à plusieurs côtés",
            "A regular shape with several sides",
        ),
        t(
            "(patio hexagonal, gloriette octogonale). Saisissez le rayon (`15` `Pieds` `Conv` `Arc`), tapez le nombre de côtés, puis `Conv` `Course` : angle intérieur, demi-angle (réglage de coupe), longueur d'un côté, périmètre, aire.",
            "(hexagonal patio, octagonal gazebo). Enter the radius (`15` `Feet` `Conv` `Arc`), type the number of sides, then `Conv` `Run`: interior angle, half angle (saw setting), length of one side, perimeter, area.",
        ),
        Some((
            t(
                "`15` `Pieds` `Conv` `Arc` `6` `Conv` `Course`, puis `Course` à répétition",
                "`15` `Feet` `Conv` `Arc` `6` `Conv` `Run`, then `Run` repeatedly",
            ),
            t(
                "ANGLE INTÉRIEUR 120,00° · DEMI-ANGLE 60,00° · CÔTÉ 15 pi 0 po · PÉRIMÈTRE 90 pi 0 po · AIRE 584,5671 pi²",
                "INTERIOR ANGLE 120.00° · HALF ANGLE 60.00° · SIDE 15 ft 0 in · PERIMETER 90 ft 0 in · AREA 584.5671 ft²",
            ),
        )),
        "hexagone octogone hexagon octagon polygon",
        &[Touche::Course],
    ),
    avec_note(
        secondaire(
            t("Toiture", "Roof"),
            tel("`Conv` `Diag`"),
            t("Les matériaux de couverture.", "Roofing materials."),
            t(
                "Saisissez la pente, puis la longueur et la largeur du bâtiment en plan (ou tapez directement une aire en plan), puis `Conv` `Diag` : aire réelle du toit, nombre de carrés (100 pi²), nombre de paquets de bardeaux, nombre de feuilles de 4 × 8.",
                "Enter the pitch, then the length and width of the building in plan (or type a plan area directly), then `Conv` `Diag`: actual roof area, number of squares (100 ft²), number of shingle bundles, number of 4 × 8 sheets.",
            ),
            Some((
                t(
                    "`10` `Pouces` `Pente` `14` `Pieds` `Longueur` `11` `Pieds` `Largeur` `Conv` `Diag`, puis `Diag` à répétition",
                    "`10` `Inch` `Pitch` `14` `Feet` `Length` `11` `Feet` `Width` `Conv` `Diag`, then `Diag` repeatedly",
                ),
                t(
                    "AIRE DU TOIT 200,4631 pi² · CARRÉS 2,00 · PAQUETS 6,01 · FEUILLES 4×8 6,26",
                    "ROOF AREA 200.4631 ft² · SQUARES 2.00 · BUNDLES 6.01 · 4×8 SHEETS 6.26",
                ),
            )),
            "bardeaux contreplaqué carré paquet shingles plywood square bundle roofing",
            &[Touche::Diag],
        ),
        t(
            "Les quantités ne sont pas arrondies et n'incluent ni les pertes, ni les débords, ni les noues.",
            "Quantities are not rounded and include neither waste, overhangs nor valleys.",
        ),
    ),
    secondaire(
        t("Pente irrég.", "Irreg. pitch"),
        t("`Conv` `Arêtier Noue`", "`Conv` `Hip Valley`"),
        t(
            "La seconde pente d'un toit irrégulier,",
            "The second pitch of an irregular roof,",
        ),
        t(
            "dont deux versants de pentes différentes se rencontrent sur un arêtier ou une noue : `8` `Pouces` `Conv` `Arêtier Noue`. Elle est effacée par deux pressions d'`Effacer`.",
            "where two sides with different pitches meet at a hip or a valley: `8` `Inch` `Conv` `Hip Valley`. It is cleared by two presses of `Clear`.",
        ),
        Some((
            t(
                "`7` `Pouces` `Pente` `4` `Pieds` `Course` `8` `Pouces` `Conv` `Arêtier Noue`, puis `Arêtier Noue`",
                "`7` `Inch` `Pitch` `4` `Feet` `Run` `8` `Inch` `Conv` `Hip Valley`, then `Hip Valley`",
            ),
            t(
                "PENTE IRRÉGULIÈRE 8 po · ARÊTIER/NOUE IRRÉG. 5 pi 9 11/16 po",
                "IRREGULAR PITCH 8 in · IRREG. HIP/VALLEY 5 ft 9 11/16 in",
            ),
        )),
        "pente irrégulière deux pentes irregular pitch two pitches",
        &[Touche::AretierNoue],
    ),
    secondaire(
        t("Angle moulure", "Crown angle"),
        t("`Conv` `Onglet composé`", "`Conv` `Compound Mitre`"),
        t(
            "L'angle entre le dos de la moulure et le mur.",
            "The angle between the back of the moulding and the wall.",
        ),
        t(
            "Les moulures couronnes courantes sont à 38° ou à 45°. `38` `Conv` `Onglet composé` le mémorise; sans nombre, la touche l'affiche.",
            "Common crown mouldings are 38° or 45°. `38` `Conv` `Compound Mitre` stores it; with no number, the key displays it.",
        ),
        Some((
            t(
                "`38` `Conv` `Onglet composé` `60` `Onglet composé`, puis `Onglet composé`",
                "`38` `Conv` `Compound Mitre` `60` `Compound Mitre`, then `Compound Mitre`",
            ),
            t(
                "ANGLE MOULURE 38,00° · ONGLET 46,84° · BISEAU 43,03°",
                "CROWN ANGLE 38.00° · MITRE 46.84° · BEVEL 43.03°",
            ),
        )),
        "moulure couronne corniche crown moulding molding spring angle",
        &[Touche::OngletCompose],
    ),
    secondaire(
        t("Contrem. max.", "Riser limit"),
        t("`Conv` `Escalier`", "`Conv` `Stair`"),
        t(
            "Le même calcul d'escalier, avec une hauteur de contremarche plafonnée.",
            "The same stair calculation, with a capped riser height.",
        ),
        t(
            "La contremarche ne dépasse jamais la hauteur visée : la calculatrice ajoute une contremarche au besoin. À utiliser quand une hauteur maximale est imposée.",
            "The riser never exceeds the target height: the calculator adds a riser when needed. Use it when a maximum height is required.",
        ),
        Some((
            t(
                "`10` `Pieds` `1` `Pouces` `Montée` `15` `Pieds` `5` `Pouces` `Course` `Conv` `Escalier`, puis `Escalier`",
                "`10` `Feet` `1` `Inch` `Rise` `15` `Feet` `5` `Inch` `Run` `Conv` `Stair`, then `Stair`",
            ),
            t(
                "CONTREMARCHE 7 1/8 po · 17 contremarches",
                "RISER HEIGHT 7 1/8 in · 17 risers",
            ),
        )),
        "contremarche maximale escalier maximum riser stair",
        &[Touche::Escalier],
    ),
    secondaire(
        t("Rayon", "Radius"),
        tel("`Conv` `Arc`"),
        t(
            "Saisit le rayon d'un cercle,",
            "Enters the radius of a circle,",
        ),
        t(
            "ou le calcule à partir de la corde et de la flèche d'un arc.",
            "or calculates it from the chord and the rise of an arc.",
        ),
        Some((
            t(
                "`3` `Pieds` `6` `Pouces` `Course` `1` `Pieds` `3` `Pouces` `Montée` `Conv` `Arc`",
                "`3` `Feet` `6` `Inch` `Run` `1` `Feet` `3` `Inch` `Rise` `Conv` `Arc`",
            ),
            t("RAYON 1 pi 10 3/16 po", "RADIUS 1 ft 10 3/16 in"),
        )),
        "cercle arc circle radius",
        &[Touche::Arc],
    ),
    secondaire(
        t("Colonne/Cône", "Column/Cone"),
        t("`Conv` `Cercle`", "`Conv` `Circle`"),
        t(
            "Volume et surface d'une colonne, puis d'un cône.",
            "Volume and surface area of a column, then of a cone.",
        ),
        t(
            "Saisissez le diamètre (`Cercle`) et la hauteur (`Hauteur`), puis `Conv` `Cercle` et `Cercle` à répétition. Pratique pour le béton des colonnes et des coffrages tubulaires.",
            "Enter the diameter (`Circle`) and the height (`Height`), then `Conv` `Circle` and `Circle` repeatedly. Handy for the concrete in columns and form tubes.",
        ),
        Some((
            t(
                "`2` `Pieds` `4` `Pouces` `Cercle` `4` `Pieds` `6` `Pouces` `Hauteur` `Conv` `Cercle`, puis `Cercle`",
                "`2` `Feet` `4` `Inch` `Circle` `4` `Feet` `6` `Inch` `Height` `Conv` `Circle`, then `Circle`",
            ),
            t(
                "COLONNE – VOLUME 19,24226 pi³ · COLONNE – SURFACE 41,53884 pi²",
                "COLUMN – VOLUME 19.24226 ft³ · COLUMN – SURFACE 41.53884 ft²",
            ),
        )),
        "cylindre pilier béton cylinder column cone concrete pier",
        &[Touche::Cercle],
    ),
    secondaire(
        t("Emp. irrég.", "Irreg. jack"),
        t("`Conv` `Empannon`", "`Conv` `Jack`"),
        t(
            "Les empannons du versant à pente irrégulière d'abord.",
            "The jack rafters of the irregular-pitch side first.",
        ),
        t(
            "Dans les Préférences, choisissez de garder le même entraxe sur les deux versants ou d'apparier les empannons pour qu'ils se rencontrent sur l'arêtier.",
            "In Preferences, choose to keep the same on-centre spacing on both sides, or to mate the jacks so they meet at the hip.",
        ),
        Some((
            t(
                "`7` `Pouces` `Pente` `4` `Pieds` `Course` `8` `Pouces` `Conv` `Arêtier Noue` `Conv` `Empannon`, puis `Empannon` à répétition",
                "`7` `Inch` `Pitch` `4` `Feet` `Run` `8` `Inch` `Conv` `Hip Valley` `Conv` `Jack`, then `Jack` repeatedly",
            ),
            t(
                "ENTRAXE 16 po · INCRÉMENT 1 pi 4 13/16 po · 2 pi 9 5/8 po · 1 pi 4 13/16 po…",
                "ON CENTRE 16 in · INCREMENT 1 ft 4 13/16 in · 2 ft 9 5/8 in · 1 ft 4 13/16 in…",
            ),
        )),
        "empannon irrégulier irregular jack rafter",
        &[Touche::Empannon],
    ),
    avec_note(
        secondaire(
            t("Blocs", "Blocks"),
            t("`Conv` `Longueur`", "`Conv` `Length`"),
            t(
                "Le nombre de blocs de béton ou de briques.",
                "The number of concrete blocks or bricks.",
            ),
            t(
                "À partir de la longueur et de la hauteur du mur saisies, d'une aire affichée, ou d'une longueur seule (une rangée). Bloc par défaut : 8 × 16 po, joint compris. Pour des briques, mémorisez l'aire d'une brique avec `Stocker` `4`.",
                "From the wall length and height you entered, a displayed area, or a length alone (one course). Default block: 8 × 16 in, joint included. For bricks, store the area of one brick with `Store` `4`.",
            ),
            Some((
                t(
                    "`300` `Pieds` `Pieds` `Conv` `Longueur`, puis `+` `3` `%`",
                    "`300` `Feet` `Feet` `Conv` `Length`, then `+` `3` `%`",
                ),
                t("BLOCS 337,5 · 347,625", "BLOCKS 337.5 · 347.625"),
            )),
            "maçonnerie brique masonry brick block",
            &[Touche::Longueur],
        ),
        t(
            "Le résultat n'est pas arrondi : ajoutez vos pertes avec `+` `5` `%`.",
            "The result is not rounded: add your waste with `+` `5` `%`.",
        ),
    ),
    secondaire(
        t("Semelle", "Footing"),
        t("`Conv` `Largeur`", "`Conv` `Width`"),
        t(
            "Le volume de béton d'une semelle.",
            "The volume of concrete in a footing.",
        ),
        t(
            "Tapez la longueur de la semelle, puis `Conv` `Largeur`. La section de la semelle (largeur × épaisseur) est mémorisée : `16` `Pouces` `×` `8` `Pouces` `=` `Stocker` `6`.",
            "Type the length of the footing, then `Conv` `Width`. The footing section (width × thickness) is stored: `16` `Inch` `×` `8` `Inch` `=` `Store` `6`.",
        ),
        Some((
            t(
                "`100` `Pieds` `Conv` `Largeur`",
                "`100` `Feet` `Conv` `Width`",
            ),
            t("SEMELLE 6,790123 vg³", "FOOTING 6.790123 yd³"),
        )),
        "fondation béton footing foundation concrete",
        &[Touche::Largeur],
    ),
    avec_note(
        secondaire(
            t("Gypse", "Drywall"),
            t("`Conv` `Hauteur`", "`Conv` `Height`"),
            t(
                "Le nombre de feuilles de gypse",
                "The number of drywall sheets",
            ),
            t(
                "de 4 × 8, 4 × 9, 4 × 10 et 4 × 12 pour une aire, ou pour une longueur de mur si les feuilles sont posées debout.",
                "of 4 × 8, 4 × 9, 4 × 10 and 4 × 12 for an area, or for a length of wall if the sheets are hung vertically.",
            ),
            Some((
                t(
                    "`150` `Pieds` `Pieds` `Conv` `Hauteur`, puis `Hauteur` à répétition",
                    "`150` `Feet` `Feet` `Conv` `Height`, then `Height` repeatedly",
                ),
                t(
                    "FEUILLES 4×8 4,6875 · 4×9 4,166667 · 4×10 3,75 · 4×12 3,125",
                    "4×8 SHEETS 4.6875 · 4×9 4.166667 · 4×10 3.75 · 4×12 3.125",
                ),
            )),
            "placoplâtre panneau feuille drywall gypsum sheet",
            &[Touche::Hauteur],
        ),
        t(
            "Le résultat n'est pas arrondi et ne tient pas compte des ouvertures.",
            "The result is not rounded and does not account for openings.",
        ),
    ),
    secondaire(
        tel("x²"),
        tel("`Conv` `%`"),
        t("Le carré.", "The square."),
        t(
            "Le carré d'une longueur donne une aire.",
            "The square of a length gives an area.",
        ),
        Some((
            t(
                "`7` `Pieds` `4` `Pouces` `Conv` `%`",
                "`7` `Feet` `4` `Inch` `Conv` `%`",
            ),
            t("53,77778 pi²", "53.77778 ft²"),
        )),
        "carré puissance square power",
        &[Touche::Pourcent],
    ),
    secondaire(
        tel("√x"),
        t("`Conv` `Effacer`", "`Conv` `Clear`"),
        t("La racine carrée.", "The square root."),
        t(
            "La racine d'une aire donne une longueur.",
            "The root of an area gives a length.",
        ),
        Some((
            t(
                "`25` `Pieds` `Pieds` `Conv` `Effacer`",
                "`25` `Feet` `Feet` `Conv` `Clear`",
            ),
            t("5 pi 0 po", "5 ft 0 in"),
        )),
        "racine carrée square root",
        &[Touche::Effacer],
    ),
    secondaire(
        tel("cm, mm"),
        tel("`Conv` `7`, `Conv` `9`"),
        t(
            "Centimètres et millimètres :",
            "Centimetres and millimetres:",
        ),
        t(
            "pour saisir (`250` `Conv` `9`) ou pour convertir la valeur affichée.",
            "to enter a value (`250` `Conv` `9`) or to convert the displayed value.",
        ),
        Some((tel("`3` `/` `4` `Conv` `9`"), t("19,05 mm", "19.05 mm"))),
        "centimètre millimètre métrique centimetre millimetre centimeter millimeter metric",
        &[Touche::Chiffre(7), Touche::Chiffre(9)],
    ),
    secondaire(
        t("pmp", "bd ft"),
        tel("`Conv` `8`"),
        t("Le pied mesure de planche", "The board foot"),
        t(
            "(144 pouces cubes). Tapez les dimensions nominales d'une pièce en pouces, pouces et pieds : `2` `×` `4` `×` `14` `Conv` `8`. Convertit aussi un volume.",
            "(144 cubic inches). Type the nominal dimensions of a piece in inches, inches and feet: `2` `×` `4` `×` `14` `Conv` `8`. It also converts a volume.",
        ),
        Some((
            tel("`2` `×` `4` `×` `14` `Conv` `8`"),
            t("9,333333 pmp", "9.333333 bd ft"),
        )),
        "bois d'œuvre planche pied-planche lumber board foot feet",
        &[Touche::Chiffre(8)],
    ),
    secondaire(
        tel("1/x"),
        tel("`Conv` `÷`"),
        t("L'inverse", "The reciprocal"),
        t("d'un nombre.", "of a number."),
        Some((tel("`8` `Conv` `÷`"), t("0,125", "0.125"))),
        "inverse reciprocal",
        &[Touche::Diviser],
    ),
    secondaire(
        t("lb, kg, t courte, t métr.", "lb, kg, ton, tonne"),
        tel("`Conv` `4`, `1`, `6`, `3`"),
        t("Les masses.", "Weights."),
        t(
            "Pour saisir une masse ou convertir la valeur affichée. Un volume est converti en masse au moyen de la masse volumique mémorisée (et l'inverse). La tonne courte vaut 2 000 lb; la tonne métrique, 1 000 kg.",
            "To enter a weight or convert the displayed value. A volume is converted to a weight using the stored density (and the reverse). The short ton is 2,000 lb; the metric tonne, 1,000 kg.",
        ),
        Some((
            t(
                "`5` `Verges` `Verges` `Verges` `Conv` `4`, puis `Conv` `6`",
                "`5` `Yards` `Yards` `Yards` `Conv` `4`, then `Conv` `6`",
            ),
            t("15 000 lb · 7,5 tc", "15,000 lb · 7.5 ton"),
        )),
        "poids béton gravier livre kilogramme tonne weight concrete gravel pound kilogram ton",
        &[
            Touche::Chiffre(4),
            Touche::Chiffre(1),
            Touche::Chiffre(6),
            Touche::Chiffre(3),
        ],
    ),
    avec_note(
        secondaire(
            t("Montants", "Studs"),
            tel("`Conv` `5`"),
            t(
                "Le nombre de montants d'un mur",
                "The number of studs in a wall",
            ),
            t(
                "pour une longueur donnée, à l'entraxe mémorisé, montant de départ compris.",
                "for a given length, at the stored on-centre spacing, starting stud included.",
            ),
            Some((
                t("`16` `Pieds` `Conv` `5`", "`16` `Feet` `Conv` `5`"),
                t("MONTANTS 13", "STUDS 13"),
            )),
            "colombage stud studs wall",
            &[Touche::Chiffre(5)],
        ),
        t(
            "N'inclut ni les montants doublés, ni les coins, ni les cadres d'ouverture.",
            "Does not include doubled studs, corners or opening frames.",
        ),
    ),
    secondaire(
        tel("Acre"),
        tel("`Conv` `2`"),
        t("L'acre", "The acre"),
        t(
            "(43 560 pi²), pour saisir une superficie ou convertir une aire affichée.",
            "(43,560 ft²), to enter a land area or convert a displayed area.",
        ),
        Some((
            t(
                "`10000` `Pieds` `Pieds` `Conv` `2`",
                "`10000` `Feet` `Feet` `Conv` `2`",
            ),
            t("0,229568 acre", "0.229568 acre"),
        )),
        "terrain superficie land lot area",
        &[Touche::Chiffre(2)],
    ),
    secondaire(
        t("Coût", "Cost"),
        tel("`Conv` `0`"),
        t("Le coût total.", "The total cost."),
        t(
            "La quantité étant affichée : `×`, le prix unitaire, puis `Conv` `0`. Le prix s'applique à l'unité affichée (par pied, par verge cube, par mètre carré…). Pour le bois en pmp, le prix est celui du millier de pmp.",
            "With the quantity displayed: `×`, the unit price, then `Conv` `0`. The price applies to the displayed unit (per foot, per cubic yard, per square metre…). For lumber in board feet, the price is per thousand board feet.",
        ),
        Some((
            t(
                "`12` `Pieds` `6` `Pouces` `×` `3,50` `Conv` `0`",
                "`12` `Feet` `6` `Inch` `×` `3.50` `Conv` `0`",
            ),
            t("COÛT 43,75 $", "COST $43.75"),
        )),
        "prix estimation price cost estimate",
        &[Touche::Chiffre(0)],
    ),
    secondaire(
        t("Effacer tout", "Clear all"),
        tel("`Conv` `×`"),
        t(
            "Remet la calculatrice à neuf :",
            "Restores the calculator to new:",
        ),
        t(
            "efface les mémoires et rétablit les valeurs mémorisées par défaut. Les préférences sont conservées. Une confirmation est demandée.",
            "clears the memories and restores the default stored values. Preferences are kept. You are asked to confirm.",
        ),
        None,
        "remise à neuf réinitialiser reset clear all",
        &[Touche::Multiplier],
    ),
    secondaire(
        tel("+/−"),
        tel("`Conv` `−`"),
        t("Change le signe", "Changes the sign"),
        t("de la valeur.", "of the value."),
        Some((tel("`5` `Conv` `−` `=`"), tel("−5"))),
        "signe négatif sign negative",
        &[Touche::Soustraire],
    ),
    secondaire(
        tel("π"),
        tel("`Conv` `+`"),
        t("La constante π", "The constant π"),
        t("(3,141593).", "(3.141593)."),
        Some((tel("`Conv` `+`"), t("3,141593", "3.141593"))),
        "pi",
        &[Touche::Additionner],
    ),
    secondaire(
        t("dms◄►déc", "dms◄►dec"),
        t("`Conv` `,`", "`Conv` `.`"),
        t("Convertit un angle", "Converts an angle"),
        t(
            "entre les degrés décimaux et les degrés-minutes-secondes. Pour saisir 23° 42′ 39″ : `23` `,` `42` `,` `39`.",
            "between decimal degrees and degrees-minutes-seconds. To enter 23° 42′ 39″: `23` `.` `42` `.` `39`.",
        ),
        Some((
            t("`44,29` `Conv` `,`", "`44.29` `Conv` `.`"),
            tel("44°17′24″"),
        )),
        "angle arpentage degrés minutes secondes surveying degrees minutes seconds",
        &[Touche::Virgule],
    ),
    secondaire(
        t("Ruban", "Tape"),
        tel("`Conv` `=`"),
        t(
            "Affiche le ruban de calcul :",
            "Shows the calculation tape:",
        ),
        t(
            "la liste de tout ce qui a été saisi et calculé. Vous pouvez l'enregistrer, le copier ou l'exporter.",
            "the list of everything that was entered and calculated. You can save it, copy it or export it.",
        ),
        None,
        "historique history tape",
        &[Touche::Egal],
    ),
    secondaire(
        t("Préf.", "Prefs"),
        t("`Conv` `Stocker`", "`Conv` `Store`"),
        t("Ouvre les Préférences.", "Opens Preferences."),
        t(
            "Les changements s'appliquent immédiatement et sont conservés d'une séance à l'autre.",
            "Changes apply immediately and are kept from one session to the next.",
        ),
        None,
        "préférences réglages preferences settings",
        &[Touche::Stocker],
    ),
    secondaire(
        tel("MC"),
        t("`Conv` `Rappel`", "`Conv` `Recall`"),
        t("Vide la mémoire cumulative", "Clears the cumulative memory"),
        tel("M+."),
        None,
        "mémoire effacer memory clear",
        &[Touche::Rappel],
    ),
    secondaire(
        tel("M−"),
        tel("`Conv` `M+`"),
        t("Soustrait", "Subtracts"),
        t(
            "la valeur affichée de la mémoire cumulative.",
            "the displayed value from the cumulative memory.",
        ),
        Some((
            t(
                "`100` `M+` `50` `Conv` `M+` `Rappel` `M+`",
                "`100` `M+` `50` `Conv` `M+` `Recall` `M+`",
            ),
            t("TOTAL M 50", "M TOTAL 50"),
        )),
        "mémoire soustraire memory subtract",
        &[Touche::MPlus],
    ),
    Fiche {
        titre: tel("sin, cos, tan"),
        acces: t("mode trigonométrique", "trigonometric mode"),
        resume: t(
            "Les fonctions trigonométriques,",
            "The trigonometric functions,",
        ),
        mode_emploi: t(
            "en degrés. Avec `Conv` : les fonctions inverses, qui donnent un angle. Dans ce mode, elles remplacent les touches `Longueur`, `Largeur` et `Hauteur`.",
            "in degrees. With `Conv`: the inverse functions, which give an angle. In this mode, they replace the `Length`, `Width` and `Height` keys.",
        ),
        exemple: Some((
            t(
                "`30` `sin` · `0,5` `Conv` `sin`",
                "`30` `sin` · `0.5` `Conv` `sin`",
            ),
            t("0,5 · 30,00°", "0.5 · 30.00°"),
        )),
        note: None,
        mots_cles: "trigonométrie sinus cosinus tangente trigonometry sine cosine tangent",
        touches: &[Touche::Longueur, Touche::Largeur, Touche::Hauteur],
        secondaire: false,
        trigo: true,
    },
    Fiche {
        titre: t("Fractions rapides", "Quick fractions"),
        acces: t("bande de touches", "strip of keys"),
        resume: t("Une bande de touches", "A strip of keys"),
        mode_emploi: t(
            "qui saisit une fraction de pouce d'un seul geste : `5` `Pieds` `6` puis `3/8`. S'active dans les Préférences.",
            "that enters a fraction of an inch in one move: `5` `Feet` `6` then `3/8`. Turn it on in Preferences.",
        ),
        exemple: Some((
            t("`5` `Pieds` `6` `3/8` `=`", "`5` `Feet` `6` `3/8` `=`"),
            t("5 pi 6 3/8 po", "5 ft 6 3/8 in"),
        )),
        note: None,
        mots_cles: "fraction rapide seizième quick fraction sixteenth",
        touches: &[],
        secondaire: false,
        trigo: false,
    },
    Fiche {
        titre: t("Dessin d'une pièce", "Room sketch"),
        acces: t("onglet Dessin", "Sketch tab"),
        resume: t(
            "Tracez le contour d'une pièce, puis inscrivez la mesure de chaque côté :",
            "Draw the outline of a room, then enter the measurement of each side:",
        ),
        mode_emploi: t(
            "la calculatrice en donne la surface et le périmètre. Tracez avec le doigt ou la souris, d'un seul geste ou un côté à la fois, ou avec les flèches du bas de la planche, qui ajoutent chacune un côté dans leur direction; le croquis n'a pas besoin d'être à l'échelle, les angles sont mis d'équerre. Touchez ensuite un côté et inscrivez sa mesure avec le pavé : `Pieds`, `Pouces` et les fractions de pouce. Dès qu'il ne manque qu'un côté dans un sens, il est déduit des autres. Ajoutez des retraits (îlot, trémie) et d'autres pièces avec `+ Pièce` (`Renommer`, ou un toucher sur la pièce choisie, change son nom); pour les murs, inscrivez leur hauteur (et le `Plafond` : plat, `Cathédrale` ou `En pente`, avec sa hauteur haute; les murs en pignon montent au faîte, et l'écran donne la surface exacte des murs, celle du plafond et le volume; `+ Retombée` ajoute un caisson sous le plafond) et ajoutez les portes et les fenêtres, puis touchez leur mur pour les placer, et faites-les glisser le long du mur pour les déplacer — leur liste choisit leur type (battante, double, coulissante, de grange, pliante, porte-fenêtre, de garage, baie libre, fenêtre, fenêtre en baie), un format courant et, au besoin, `Inverser` leur sens; le bordereau, sous les pièces, les numérote (P1, F1…), comme le plan et le PDF; les finis de la pièce (plancher, peinture et sa couleur, plafond, plinthes) se choisissent dans leur section, le plancher paraît en trame sur le plan, le bilan donne la quantité de chacun, et le tableau des finis les réunit pour tout le plan; le bouton `3D` montre la pièce en 3D. Ajoutez l'escalier, le mobilier et les symboles des métiers avec leurs boutons, puis touchez la pièce pour les poser : la rangée `Mobilier`, `Plomberie`, `Électricité`, `Ventilation`, `Cuisine et SdB` change les boutons, comme la barre des métiers de la calculatrice (chauffe-eau, coudes, robinets, panneau, prises, interrupteurs, luminaires, fournaise, thermopompe, armoires…), et un objet du mur, une prise ou une armoire, se colle au mur ou à la cloison le plus proche, le dos contre lui; faites-les glisser, et `+15°` ou `+90°` les tournent, aussi sur le plan quand un meuble est choisi. L'escalier a sa forme (droit, en L, en U, quart tournant, colimaçon) et sa hauteur à monter (celle des murs au départ) : ses contremarches se calculent pour 7 1/2 po au plus (leur nombre se change au besoin), et le giron d'un escalier droit en découle. Les meubles ne se superposent pas : déplacé, un meuble s'arrête contre les autres, contre les cloisons et contre les murs, et glisse le long. Les symboles, eux, se chevauchent et se posent n'importe où, même hors de la pièce (le compteur, la thermopompe extérieure); le bilan compte chacun, et le PDF ajoute une page de légende. Un meuble qu'on déplace ou qu'on choisit montre ses dégagements, jusqu'aux murs, aux cloisons et aux autres meubles; le battant d'une porte qui frappe un meuble, ou un meuble qui traverse un mur, paraît en rouge, et le bilan le signale. `Mesurer`, au bas de la planche, donne la distance entre deux points touchés, accrochés aux coins et aux murs proches. `Crayon` trace au doigt par-dessus le plan — flèches, cercles, surlignage —, en rouge, en bleu, en vert, en noir ou au surligneur; les traits restent avec le plan et vont au PDF et au DXF. `+ Note` écrit une note sur le plan : touchez l'endroit, puis écrivez-la. `+ Cloison` trace un mur intérieur : glissez d'un bout à l'autre, en tournant les coins au besoin (une cloison en L ou en U); ses bouts vont jusqu'aux murs voisins, et `+ Porte` ou `+ Ouverture` (un passage sans porte), dans sa ligne, lui en donnent autant qu'il en faut. Les espaces que ferment les cloisons (une chambre et sa garde-robe) se trouvent d'eux-mêmes : leur nom, qu'on peut changer, et leur aire s'écrivent au milieu de chacun, au bilan et au PDF. Touchez une cloison sur le plan : le pavé s'ouvre sur la longueur du segment touché, comme pour un côté de mur, et `Suivant` passe au segment suivant. Les murs extérieurs ont 9 po d'épaisseur et les cloisons 5 po : les deux se règlent; le bouton `Intérieur` ou `Extérieur` dit de quel côté des murs sont vos mesures. Un mur s'étire par la poignée de son milieu : glissez-la, et ses voisins s'allongent; un segment de cloison aussi, par la sienne. `Annuler` défait le dernier changement, et `Rétablir` le refait. `Dupliquer` ajoute une copie de la pièce, et `Miroir` la retourne, la gauche et la droite échangées; `Copier`, dans la ligne d'un meuble ou d'une cloison, en pose une copie à côté, et le `Miroir` d'un escalier tournant le fait tourner de l'autre côté. Inscrivez ensuite le tout au ruban ou envoyez-le vers un outil. `Mes dessins`, en haut de l'écran, garde plusieurs dessins : `+ Nouveau dessin`, `Ouvrir`, `Renommer`, `Dupliquer` (pour comparer deux variantes), `Supprimer`; `Enregistrer le fichier` en fait un fichier, que `Ouvrir un fichier…` rouvre sur un autre appareil, ordinateur ou téléphone.",
            "the calculator gives its area and perimeter. Draw with your finger or the mouse, in one stroke or one side at a time, or with the arrows at the bottom of the board, each adding a side in its direction; the sketch does not need to be to scale, corners are squared up. Then tap a side and enter its measurement with the keypad: `Feet`, `Inches` and fractions of an inch. As soon as only one side is missing in a direction, it is worked out from the others. Add deductions (island, stairwell) and more rooms with `+ Room` (`Rename`, or a tap on the chosen room, changes its name); for the walls, enter their height (and the `Ceiling`: flat, `Cathedral` or `Sloped`, with its high height; gable walls rise to the ridge, and the screen gives the exact wall area, the ceiling area and the volume; `+ Bulkhead` adds a box below the ceiling) and add the doors and windows, then tap their wall to place them, and drag them along the wall to move them — their list picks their type (hinged, double, sliding, barn, bifold, patio, garage, open passage, window, bay window), a common size and, if needed, `Flip` their swing; the schedule, below the rooms, numbers them (P1, F1…), as the plan and the PDF do; the room's finishes (floor, paint and its colour, ceiling, baseboards) are picked in their section, the floor shows as a pattern on the plan, the summary gives each one's quantity, and the finish schedule gathers them for the whole plan; the `3D` button shows the room in 3D. Add stairs, furniture and trade symbols with their buttons, then tap the room to set them down: the `Furniture`, `Plumbing`, `Electrical`, `HVAC`, `Kitchen & bath` row changes the buttons, like the calculator's trade bar (water heater, elbows, valves, panel, receptacles, switches, lights, furnace, heat pump, cabinets…), and a wall item, a receptacle or a cabinet, sticks to the nearest wall or partition, its back against it; drag them, and `+15°` or `+90°` turns them, also on the plan when an item is chosen. Stairs have a shape (straight, L-shaped, U-shaped, quarter-turn, spiral) and a rise (the wall height to start with): their risers are worked out at 7 1/2 in at most (their number can be changed), and the tread of straight stairs follows. Furniture does not overlap: when moved, an item stops against the others, against partitions and against walls, and slides along them. Symbols, for their part, overlap and go anywhere, even outside the room (the meter, the outdoor heat pump); the summary counts each one, and the PDF adds a legend page. An item you move or choose shows its clearances to the walls, partitions and other items; a door swing that hits an item, or an item that crosses a wall, shows in red, and the summary flags it. `Measure`, at the bottom of the board, gives the distance between two tapped points, which snap to nearby corners and walls. `Pen` draws freehand over the plan — arrows, circles, highlighting —, in red, blue, green, black or with the highlighter; the strokes stay with the plan and go to the PDF and the DXF. `+ Note` writes a note on the plan: tap the spot, then type it. `+ Partition` draws an interior wall: drag from one end to the other, turning corners as needed (an L- or U-shaped partition); its ends reach the nearby walls, and `+ Door` or `+ Opening` (a doorless passage), on its row, give it as many as it needs. The spaces the partitions close off (a bedroom and its closet) are found on their own: their name, which you can change, and their area are written in the middle of each, in the summary and in the PDF. Tap a partition on the plan: the keypad opens on the length of the touched segment, as for a side of a wall, and `Next` moves to the next segment. Exterior walls are 9 in thick and partitions 5 in: both can be changed; the `Interior` or `Exterior` button tells which side of the walls your measurements are on. A wall stretches by the handle at its middle: drag it, and its neighbours lengthen; a partition segment does too, by its own. `Undo` reverses the last change, and `Redo` brings it back. `Duplicate` adds a copy of the room, and `Mirror` flips it, left and right swapped; `Copy`, on the row of an item or a partition, sets a copy beside it, and the `Mirror` of a turning staircase makes it turn the other way. Then record everything on the tape or send it to a tool. `My drawings`, at the top of the screen, keeps several drawings: `+ New drawing`, `Open`, `Rename`, `Duplicate` (to compare two variants), `Delete`; `Save as a file` makes a file of one, which `Open a file…` reopens on another device, computer or phone.",
        ),
        exemple: Some((
            t(
                "Un rectangle : `12` `Pieds` en haut, `10` `Pieds` `6` `Pouces` à droite",
                "A rectangle: `12` `Feet` on top, `10` `Feet` `6` `Inches` on the right",
            ),
            t(
                "Surface 126 pi² (11,7 m²) · Périmètre 45 pi 0 po · Diagonale 15 pi 11 5/16 po",
                "Area 126 ft² (11.7 m²) · Perimeter 45 ft 0 in · Diagonal 15 ft 11 5/16 in",
            ),
        )),
        note: Some(t(
            "Un nombre inscrit sans unité est en pieds. `Agrandir`, en haut de la planche, lui donne presque tout l'écran, et `Réduire` ramène les listes; chaque section sous le plan se plie et se déplie d'un toucher sur son titre. Les boutons `+` et `−` de la planche zooment, `Ajuster` recadre; deux doigts zooment et déplacent la vue, et une forme fermée se déplace au doigt. `Cotes`, au bas du plan, montre les lignes de cote : le long de chaque mur, les distances entre ses coins, ses portes et fenêtres et les cloisons qui le touchent, et l'écart entre deux cloisons parallèles. `Exporter en PDF` ou `Exporter en DXF` enregistre le plan, avec ses cotes si elles se montrent; au-dessus, la `Mise en page du PDF` choisit l'échelle (ajustée, 1/4 po = 1 pi, 1/8 po = 1 pi, 1:50, 1:100) et remplit le cartouche (projet, adresse, dessiné par). Un mur en biais se trace à 45°, ou avec les flèches après le bouton `45°`; pour un autre angle, touchez le côté : sous le pavé, `Angle par rapport au côté précédent` offre 22,5°, 30°, 45°, 60° et 90°, ou l'angle tapé. Une pièce à quatre côtés, tous mesurés, reçoit ses diagonales dans la section `Diagonales` : elle se redessine telle qu'elle est, avec l'angle de chaque coin et son écart à l'équerre.",
            "A number entered without a unit is in feet. `Enlarge`, at the top of the board, gives it almost the whole screen, and `Shrink` brings the lists back; each section below the plan folds and unfolds with a tap on its title. The `+` and `−` buttons on the board zoom, `Fit` reframes; two fingers zoom and pan, and a closed shape pans with one finger. `Dims`, at the bottom of the plan, shows the dimension lines: along each wall, the distances between its corners, its doors and windows and the partitions that meet it, and the gap between two parallel partitions. `Export as PDF` or `Export as DXF` saves the plan, with its dimensions if they are shown; above them, the `PDF layout` picks the scale (fit, 1/4 in = 1 ft, 1/8 in = 1 ft, 1:50, 1:100) and fills in the title block (project, address, drawn by). An angled wall is drawn at 45°, or with the arrows after the `45°` button; for another angle, tap the side: below the keypad, `Angle from the previous side` offers 22.5°, 30°, 45°, 60° and 90°, or the angle you type. A four-sided room, all sides measured, takes its diagonals in the `Diagonals` section: it is redrawn as it really is, with each corner's angle and how far it is out of square.",
        )),
        mots_cles: "dessin dessiner croquis pièce forme plan surface aire périmètre contour flèche flèches biais angle angles libre trapèze 22,5 30 60 diagonale diagonales équerrage hors d'équerre coin coins plafond cathédrale pente pignon faîte retombée caisson volume cote cotes ligne de cote chaîne échelle cartouche lettre 11 x 17 3d maquette escalier mobilier meuble lit table sofa comptoir toilette bain douche laveuse note notes texte cloison cloisons polyligne division épaisseur renommer nom étirer poignée sketch draw drawing room shape floor plan area perimeter outline arrow arrows angled 45 free angle trapezoid diagonal diagonals out of square squareness corner corners ceiling cathedral sloped gable ridge bulkhead soffit volume dimension dimensions dimension line string scale title block letter tabloid stairs furniture bed table sofa counter toilet bathtub shower washer fridge range note notes text partition partitions polyline thickness rename name stretch handle annuler rétablir refaire dupliquer copier copie miroir retourner jumelé variante undo redo duplicate copy mirror flip semi-detached variant",
        touches: &[],
        secondaire: false,
        trigo: false,
    },
    Fiche {
        titre: t("Bloc-notes", "Notepad"),
        acces: t("onglet Notes", "Notes tab"),
        resume: t(
            "Gardez vos notes de chantier dans la calculatrice :",
            "Keep your job-site notes in the calculator:",
        ),
        mode_emploi: t(
            "`+ Nouvelle note` en ouvre une : écrivez son titre et son texte, ils s'enregistrent d'eux-mêmes. Chaque note garde la date de sa dernière modification; `Copier` la met dans le presse-papiers, `Exporter` en fait un fichier texte, `Supprimer` la retire avec ses pièces jointes. Une note laissée vide disparaît en revenant à la liste. Une ligne qui commence par [ ] s'affiche en case à cocher, au-dessus du texte : un toucher la coche ou la décoche, et `+ Case` en ajoute une. Le champ de recherche, en tête de la liste, ne garde que les notes qui contiennent les mots cherchés. Sous le texte, `+ Photo` joint des photos (sur un téléphone : l'appareil photo ou la galerie) et `+ Fichier` joint un fichier (PDF, plan, soumission). Touchez une vignette pour voir la photo en grand; `Enregistrer` en garde une copie ailleurs, `Retirer` l'efface. Sur la photo en grand, `+ Cote` puis un glissé trace une cote : inscrivez la mesure prise au ruban; une cote laissée vide est estimée d'après la cote de référence (marquée ~). `Exporter en PDF` fait un PDF de la photo cotée.",
            "`+ New note` opens one: type its title and text, they save on their own. Each note keeps the date it was last changed; `Copy` puts it on the clipboard, `Export` makes a text file of it, `Delete` removes it with its attachments. A note left empty disappears when you go back to the list. A line that starts with [ ] shows as a checkbox, above the text: a tap ticks or unticks it, and `+ Checkbox` adds one. The search field, at the head of the list, keeps only the notes that contain the words you look for. Below the text, `+ Photo` attaches photos (on a phone: the camera or the gallery) and `+ File` attaches a file (PDF, plan, quote). Tap a thumbnail to see the photo full size; `Save` keeps a copy elsewhere, `Remove` erases it. On the full-size photo, `+ Dimension` then a drag draws a dimension: enter the measurement taken with the tape; a dimension left empty is estimated from the reference dimension (marked ~). `Export as PDF` makes a PDF of the dimensioned photo.",
        ),
        exemple: None,
        note: Some(t(
            "Le bloc-notes garde jusqu'à 500 notes, sur l'appareil, et chaque note jusqu'à 50 pièces jointes. Une photo de plus de 2048 pixels est réduite à 2048 pixels; un fichier joint ne dépasse pas 25 Mo.",
            "The notepad keeps up to 500 notes, on the device, and each note up to 50 attachments. A photo larger than 2048 pixels is reduced to 2048 pixels; an attached file cannot exceed 25 MB.",
        )),
        mots_cles: "case cases cocher cochée liste à cocher tâches recherche rechercher chercher checkbox checklist tick search find notes note bloc-notes texte chantier liste photo photos pièce jointe pièces jointes fichier image notepad text list attachment attachments file picture",
        touches: &[],
        secondaire: false,
        trigo: false,
    },
    Fiche {
        titre: t("Niveau", "Level"),
        acces: t("onglet Métiers (téléphone)", "Trades tab (phone)"),
        resume: t(
            "Le téléphone devient un niveau et un inclinomètre :",
            "The phone becomes a level and an inclinometer:",
        ),
        mode_emploi: t(
            "Dans l'onglet Métiers, la carte Niveau — sur un téléphone ou une tablette qui a un capteur d'inclinaison; ailleurs, elle est cachée. Posez l'appareil à plat sur un chevron, un tuyau ou une rampe, l'écran vers le haut : la bulle montre le côté haut, et l'écran donne la pente en degrés, sur 12 et en pour cent. `Envoyer` met une valeur à l'afficheur de la calculatrice. `Figer` garde la lecture pour la lire à l'aise; `Étalonner`, sur une surface de référence de niveau, en fait le zéro. Sur un iPhone, `Autoriser le capteur` demande d'abord la permission.",
            "In the Trades tab, the Level card — on a phone or tablet with a tilt sensor; elsewhere, it is hidden. Lay the device flat on a rafter, a pipe or a ramp, screen up: the bubble shows the high side, and the screen gives the slope in degrees, in 12 and in percent. `Send` puts a value on the calculator's display. `Hold` keeps the reading so you can read it at ease; `Calibrate`, on a level reference surface, makes it zero. On an iPhone, `Allow the sensor` first asks for permission.",
        ),
        exemple: Some((
            t(
                "Le téléphone sur un chevron à 6/12, puis `Envoyer`",
                "The phone on a 6/12 rafter, then `Send`",
            ),
            t(
                "26,57° · 6,00 sur 12 · 50,0 %",
                "26.57° · 6.00 in 12 · 50.0%",
            ),
        )),
        note: Some(t(
            "Un étui inégal fausse la lecture : étalonnez avec l'étui. Le niveau n'émet ni son ni vibration.",
            "An uneven case skews the reading: calibrate with the case on. The level makes no sound or vibration.",
        )),
        mots_cles: "niveau inclinomètre bulle pente degrés pourcentage chevron tuyau rampe capteur étalonner level inclinometer bubble slope degrees pitch percent rafter pipe ramp sensor calibrate",
        touches: &[],
        secondaire: false,
        trigo: false,
    },
    Fiche {
        titre: t("Conversions d'unités", "Unit conversions"),
        acces: t(
            "onglet Métiers, ou barre des métiers : Conversions",
            "Trades tab, or trade bar: Conversions",
        ),
        resume: t(
            "Toutes les unités du chantier, d'un coup :",
            "Every unit on the job site, at once:",
        ),
        mode_emploi: t(
            "Dans l'onglet Métiers, la carte Conversions d'unités : choisissez la grandeur — longueurs, aires, volumes, liquides, masses, masse volumique, pressions et charges, forces, charges linéaires, couples de serrage, températures, puissances, énergies, débits, vitesses, pentes, isolation thermique, éclairement ou bois d'œuvre —, tapez la valeur et choisissez son unité de départ : la calculatrice la donne dans toutes les autres unités de sa grandeur, à quatre chiffres significatifs. Au clavier, touchez Conversions dans la liste des métiers, puis la grandeur : la touche Unité fait défiler les unités de départ. Le bois d'œuvre donne les dimensions réelles d'un 2 × 4 ou d'un 2 × 6 (1 1/2 po × 3 1/2 po, 38 mm × 89 mm) et ses pieds-mesure de planche.",
            "In the Trades tab, the Unit conversions card: pick the quantity — lengths, areas, volumes, liquids, weights, density, pressure and loads, forces, line loads, torque, temperatures, power, energy, flow rates, speeds, slopes, thermal insulation, illuminance or dimension lumber —, type the value and pick its unit: the calculator gives it in every other unit of its kind, to four significant digits. On the keypad, tap Conversions in the trade list, then the quantity: the Unit key cycles through the units. Dimension lumber gives the actual size of a 2 × 4 or a 2 × 6 (1 1/2 in × 3 1/2 in, 38 mm × 89 mm) and its board feet.",
        ),
        exemple: Some((
            t(
                "`Conversions` · `Pressions et charges` · 60 · lb/po²",
                "`Conversions` · `Pressure and loads` · 60 · psi",
            ),
            t(
                "413,7 kPa · 4,137 bar · 8 640 lb/pi²",
                "413.7 kPa · 4.137 bar · 8,640 psf",
            ),
        )),
        note: Some(t(
            "Les longueurs, les aires, les volumes et les masses se convertissent aussi au clavier, par les touches d'unités.",
            "Lengths, areas, volumes and weights also convert on the keypad, with the unit keys.",
        )),
        mots_cles: "conversion conversions convertir unité unités pression psi kPa bar charge neige température celsius fahrenheit kelvin puissance BTU tonne réfrigération kW HP cheval-vapeur énergie kWh therm débit gpm cfm gallon litre vitesse pente pourcentage degrés valeur R RSI U isolation éclairement lux pied-bougie bois œuvre 2x4 nominal réel pmp convert unit pressure load snow temperature power ton energy flow speed slope insulation illuminance lumber board feet",
        touches: &[],
        secondaire: false,
        trigo: false,
    },
    Fiche {
        titre: t("Carnet de nivellement", "Levelling book"),
        acces: t("onglet Métiers", "Trades tab"),
        resume: t(
            "Au niveau laser, les élévations et ce qu'il faut creuser ou remblayer :",
            "With a laser level, the elevations and what to cut or fill:",
        ),
        mode_emploi: t(
            "Dans l'onglet Métiers, la carte Carnet de nivellement : inscrivez l'élévation du repère (100, en pieds, par exemple) et la lecture faite sur lui; la calculatrice en tire la hauteur de l'instrument. Pour chaque point, inscrivez son nom et sa lecture (`+ Point` en ajoute un) : elle donne son élévation et, par rapport à l'élévation visée — celle du carnet ou celle de la ligne —, ce qu'il faut creuser ou remblayer. Le carnet se garde sur l'appareil; `Exporter (CSV)` en fait un fichier pour un tableur.",
            "In the Trades tab, the Levelling book card: enter the benchmark elevation (100, in feet, for example) and the reading taken on it; the calculator works out the height of instrument. For each point, enter its name and its reading (`+ Point` adds one): it gives the point's elevation and, against the target elevation — the book's or the line's own —, what to cut or fill. The book is kept on the device; `Export (CSV)` makes a file of it for a spreadsheet.",
        ),
        exemple: Some((
            t(
                "Repère `100`, lu à `5`; un point lu à `6` pi `3` po, visée `99`",
                "Benchmark `100`, read at `5`; a point read at `6` ft `3` in, target `99`",
            ),
            t(
                "Élévation 98 pi 9 po · Remblayer 3 po",
                "Elevation 98 ft 9 in · Fill 3 in",
            ),
        )),
        note: Some(t(
            "Un nombre seul est en pieds; le mètre s'écrit 1,25 m. Une élévation peut être négative.",
            "A number alone is in feet; metres are written 1.25 m. An elevation can be negative.",
        )),
        mots_cles: "nivellement niveau laser repère lecture élévation visée creuser remblayer déblai remblai terrassement carnet arpentage levelling level laser benchmark reading elevation target cut fill grading book survey",
        touches: &[],
        secondaire: false,
        trigo: false,
    },
    Fiche {
        titre: t("Calcul d'heures", "Hours calculation"),
        acces: t("barre des métiers : Heures", "trade bar: Hours"),
        resume: t(
            "Additionnez des heures et des minutes comme des pieds et des pouces :",
            "Add hours and minutes like feet and inches:",
        ),
        mode_emploi: t(
            "Touchez Heures dans la barre des métiers, ou dans la liste qu'ouvre le métier : les touches `h` et `min` s'ajoutent en haut du clavier. Un nombre laissé après les heures compte en minutes. `h` sur un résultat le passe en heures décimales, et inversement; `min`, en minutes. Une durée se multiplie et se divise par un nombre; deux durées se divisent l'une par l'autre. La Feuille de temps, dans la barre ou dans l'onglet Métiers, compte les heures de la semaine : l'arrivée, le départ et le dîner de chaque jour, sur 24 heures.",
            "Tap Hours in the trade bar, or in the list the trade opens: the `h` and `min` keys are added at the top of the keypad. A number left after the hours counts as minutes. `h` on a result switches it to decimal hours, and back; `min`, to minutes. A duration is multiplied and divided by a number; two durations divide one by the other. The Timesheet, in the bar or in the Trades tab, counts the week's hours: arrival, departure and lunch each day, on the 24-hour clock.",
        ),
        exemple: Some((
            t(
                "`7` `h` `45` `min` `+` `8` `h` `30` `min` `=`, puis `h`",
                "`7` `h` `45` `min` `+` `8` `h` `30` `min` `=`, then `h`",
            ),
            t("16 h 15, puis 16,25 h", "16 h 15 min, then 16.25 h"),
        )),
        note: Some(t(
            "Aucun taux de salaire : la feuille de temps compte les heures, pas la paie.",
            "No pay rate: the timesheet counts hours, not pay.",
        )),
        mots_cles: "heures minutes heure durée temps feuille de temps semaine arrivée départ dîner pause horaire décimales hours minutes hour duration time timesheet week arrival departure lunch break decimal",
        touches: &[Touche::Heures, Touche::Minutes],
        secondaire: false,
        trigo: false,
    },
    Fiche {
        titre: t("Liste de coupe", "Cut list"),
        acces: t("onglet Métiers", "Trades tab"),
        resume: t(
            "Combien de pièces acheter, et comment les couper :",
            "How many lengths to buy, and how to cut them:",
        ),
        mode_emploi: t(
            "Dans l'onglet Métiers, la carte Liste de coupe : inscrivez chaque morceau à couper, sa quantité et sa longueur (`+ Morceau` en ajoute une ligne), puis les longueurs offertes en magasin (8; 10; 12; 16 pi au départ) et le trait de scie (1/8 po). La calculatrice donne les pièces à acheter, la longueur achetée et la chute, puis le plan de coupe de chaque pièce, dessiné à l'échelle : le plan qui achète le moins de longueur. `Ajouter à la liste d'achat` y inscrit les pièces, sous le nom du matériau s'il est donné.",
            "In the Trades tab, the Cut list card: enter each piece to cut, its quantity and its length (`+ Piece` adds a line), then the stock lengths sold in stores (8; 10; 12; 16 ft to start with) and the saw kerf (1/8 in). The calculator gives the lengths to buy, the length bought and the offcut, then the cutting plan of each length, drawn to scale: the plan that buys the least length. `Add to shopping list` writes the lengths there, under the material's name if one is given.",
        ),
        exemple: Some((
            t(
                "4 morceaux de `4` pi et 2 de `6` pi `6` po, en 8, 10, 12 ou 16 pi",
                "4 pieces of `4` ft and 2 of `6` ft `6` in, from 8, 10, 12 or 16 ft",
            ),
            t(
                "2 × 16 pi : 6 pi 6 po + 4 pi + 4 pi chacune, chute 1 pi 5 5/8 po",
                "2 × 16 ft: 6 ft 6 in + 4 ft + 4 ft each, offcut 1 ft 5 5/8 in",
            ),
        )),
        note: Some(t(
            "Un nombre seul est en pieds. Un morceau plus long que toutes les longueurs offertes est signalé et laissé hors du plan.",
            "A number alone is in feet. A piece longer than every stock length is flagged and left out of the plan.",
        )),
        mots_cles: "coupe couper liste optimisée optimiser morceaux longueurs stock chute retailles trait scie bois moulures conduits tuyaux madriers cut list optimize pieces lengths stock offcut waste kerf saw lumber mouldings conduit pipe",
        touches: &[],
        secondaire: false,
        trigo: false,
    },
    Fiche {
        titre: t("Rapport de calcul en PDF", "Calculation report as PDF"),
        acces: t("onglet Métiers", "Trades tab"),
        resume: t(
            "Gardez la trace d'un calcul, ou joignez-la à une soumission :",
            "Keep a record of a calculation, or attach it to a quote:",
        ),
        mode_emploi: t(
            "`PDF`, sous les résultats d'un calculateur, en fait un rapport d'une page : le nom du calculateur, la date, le projet, les données entrées, les résultats et la note du calculateur, avec Constructo AI · constructoai.ca au bas, comme les plans. Le projet, facultatif, se tape au moment d'exporter; il reste proposé pour le rapport suivant. Sur un ordinateur, une boîte demande où enregistrer le fichier; dans un navigateur, il se télécharge.",
            "`PDF`, below a calculator's results, makes a one-page report of it: the calculator's name, the date, the project, the inputs, the results and the calculator's note, with Constructo AI · constructoai.ca at the bottom, like the plans. The project, optional, is typed when exporting; it stays offered for the next report. On a computer, a box asks where to save the file; in a browser, it downloads.",
        ),
        exemple: None,
        note: None,
        mots_cles: "rapport pdf imprimer imprimé soumission dossier projet client document calcul report print printed quote file project client document calculation",
        touches: &[],
        secondaire: false,
        trigo: false,
    },
    Fiche {
        titre: t("Liste d'achat", "Shopping list"),
        acces: t("onglet Métiers", "Trades tab"),
        resume: t(
            "Ce qu'il faut acheter, au fil des calculs :",
            "What to buy, calculation after calculation:",
        ),
        mode_emploi: t(
            "`Ajouter à la liste d'achat`, sous les résultats d'un calculateur qui donne des sacs, des panneaux, des boîtes ou des rouleaux, les inscrit dans la note Liste d'achat du bloc-notes, créée au besoin : une ligne à cocher par article, avec son format (le rendement d'un sac, les dimensions d'une tuile). Un article déjà sur la liste s'y additionne, tant que sa ligne n'est pas cochée : cochez ce qui est acheté. `Copier` ou `Exporter`, dans la note, la passent au commis ou au fournisseur.",
            "`Add to shopping list`, below the results of a calculator that gives bags, boards, boxes or rolls, writes them into the Shopping list note of the notepad, created when needed: one line to tick per item, with its format (the yield of a bag, the size of a tile). An item already on the list adds up there as long as its line is not ticked: tick what is bought. `Copy` or `Export`, in the note, hand it to the clerk or the supplier.",
        ),
        exemple: None,
        note: Some(t(
            "La liste ne porte aucun prix. Les rangs, les voyages de camion et les autres comptes qui ne s'achètent pas n'y vont pas.",
            "The list carries no prices. Courses, truckloads and other counts that are not bought do not go on it.",
        )),
        mots_cles: "liste achat acheter achats magasin quincaillerie fournisseur commande sacs panneaux boîtes rouleaux matériaux shopping list buy purchase store hardware supplier order bags boards boxes rolls materials",
        touches: &[],
        secondaire: false,
        trigo: false,
    },
    Fiche {
        titre: t(
            "Calculateurs épinglés et récents",
            "Pinned and recent calculators",
        ),
        acces: t("onglet Métiers", "Trades tab"),
        resume: t(
            "Gardez sous la main les calculateurs dont vous vous servez le plus :",
            "Keep the calculators you use most within reach:",
        ),
        mode_emploi: t(
            "`Épingler`, au bas du formulaire d'un calculateur, le met en tête de l'onglet Métiers et de la barre des métiers de l'accueil, marqué d'une étoile : un toucher sur sa pastille le branche sur le clavier. `Désépingler` l'en retire. Les derniers calculateurs ouverts se retrouvent sous Récents, en tête de l'onglet Métiers. Là où le système le permet, l'application installée offre aussi des raccourcis sur son icône : les métiers, le dessin, le bloc-notes et les rubans.",
            "`Pin`, at the bottom of a calculator's form, puts it at the head of the Trades tab and of the home screen's trade bar, marked with a star: a tap on its chip plugs it into the keypad. `Unpin` removes it. The last calculators you opened are found under Recent, at the head of the Trades tab. Where the system allows it, the installed app also offers shortcuts on its icon: the trades, the drawing, the notepad and the tapes.",
        ),
        exemple: None,
        note: Some(t(
            "Douze calculateurs au plus peuvent être épinglés.",
            "Up to twelve calculators can be pinned.",
        )),
        mots_cles: "épingler épinglé épinglés désépingler favori favoris étoile récent récents derniers raccourci raccourcis icône pin pinned unpin favourite favorite favorites star recent last shortcut shortcuts icon",
        touches: &[],
        secondaire: false,
        trigo: false,
    },
    Fiche {
        titre: t("Mode chantier", "Job-site mode"),
        acces: t("menu, ou Préférences", "menu, or Preferences"),
        resume: t(
            "Pour travailler dehors, en plein soleil ou avec des gants :",
            "For working outdoors, in full sun or with gloves:",
        ),
        mode_emploi: t(
            "`Mode chantier`, dans le menu ou dans les Préférences, passe la calculatrice en noir, blanc et jaune de chantier, à plat et très contrastée. Sur l'écran de la calculatrice, les cinq rangées du bas — unités, chiffres et opérations — sont plus hautes, et le clavier descend jusqu'au bas de l'écran, sous le pouce, pour taper d'une main; le menu mène alors aux autres écrans, qui gardent leur barre d'onglets. Une touche enfoncée s'inverse, bien visible au soleil. `Quitter le mode chantier`, dans le menu, rend les couleurs du thème choisi.",
            "`Job-site mode`, in the menu or in Preferences, switches the calculator to black, white and safety yellow, flat and high-contrast. On the calculator screen, the bottom five rows — units, digits and operations — are taller, and the keypad goes down to the bottom of the screen, under your thumb, for one-handed typing; the menu then leads to the other screens, which keep their tab bar. A pressed key turns inverse, easy to see in the sun. `Leave job-site mode`, in the menu, brings back the colours of the chosen theme.",
        ),
        exemple: None,
        note: Some(t(
            "Le mode chantier passe avant le thème (Automatique, Clair ou Sombre), qui revient quand on le quitte.",
            "Job-site mode takes precedence over the theme (Automatic, Light or Dark), which comes back when you leave it.",
        )),
        mots_cles: "chantier soleil contraste lisible gants gant grosses touches une main pouce extérieur dehors jaune noir job site sun sunlight contrast readable gloves glove big keys one hand thumb outdoors yellow black",
        touches: &[],
        secondaire: false,
        trigo: false,
    },
    Fiche {
        titre: t("Sauvegarde complète", "Full backup"),
        acces: t("menu", "menu"),
        resume: t(
            "Gardez toutes vos données dans un seul fichier, ou passez-les à un autre appareil :",
            "Keep all your data in one file, or move it to another device:",
        ),
        mode_emploi: t(
            "`Exporter mes données`, dans le menu, réunit tout ce que la calculatrice garde — réglages, mémoires, rubans, dessins, notes et leurs photos — dans un fichier. Gardez-le à l'abri, ou envoyez-le à un autre appareil (courriel, nuage, câble). Sur l'autre appareil, `Importer des données` le lit et, après confirmation, remplace ses données par celles du fichier.",
            "`Export my data`, in the menu, gathers everything the calculator keeps — settings, memories, tapes, drawings, notes and their photos — into one file. Keep it safe, or send it to another device (email, cloud, cable). On the other device, `Import data` reads it and, after confirmation, replaces its data with the file's.",
        ),
        exemple: None,
        note: Some(t(
            "L'importation remplace les données de l'appareil : sauvegardez-les d'abord si vous voulez les garder.",
            "Importing replaces the device's data: back it up first if you want to keep it.",
        )),
        mots_cles: "sauvegarde sauvegarder copie exporter importer transférer transfert téléphone ordinateur appareil données backup export import transfer phone computer device data",
        touches: &[],
        secondaire: false,
        trigo: false,
    },
    Fiche {
        titre: t("Clavier adapté au métier", "Keypad adapted to your trade"),
        acces: t("barre des métiers", "trade bar"),
        resume: t(
            "Le clavier prend les touches de votre métier :",
            "The keypad takes on the keys of your trade:",
        ),
        mode_emploi: t(
            "touchez le nom du métier, au-dessus du clavier, puis un de ses outils. Les champs de l'outil deviennent les touches du haut : tapez une valeur, puis appuyez sur la touche du champ où la ranger. `Résultat` calcule et inscrit le calcul au ruban; appuyez de nouveau pour voir le résultat suivant. `Conv` puis la touche d'un champ le remet à sa valeur de départ. Maintenez une touche de l'outil enfoncée pour lire son mode d'emploi.",
            "tap the trade name above the keypad, then one of its tools. The tool's fields become the top keys: type a value, then press the key of the field where you want to store it. `Result` calculates and records the calculation on the tape; press again to see the next result. `Conv` then a field's key resets it to its starting value. Press and hold one of the tool's keys to read how to use it.",
        ),
        exemple: Some((
            t(
                "Béton, Dalle de béton : `24` `Pieds` `Longueur` `20` `Pieds` `Largeur` `4` `Pouces` `Épaisseur` `Résultat` `Résultat`",
                "Concrete, Concrete slab: `24` `Feet` `Length` `20` `Feet` `Width` `4` `Inch` `Thickness` `Result` `Result`",
            ),
            t(
                "VOLUME DE BÉTON 5,93 vg³ (4,53 m³)",
                "CONCRETE VOLUME 5.93 yd³ (4.53 m³)",
            ),
        )),
        note: Some(t(
            "L'onglet Métiers offre les mêmes calculateurs en formulaires à remplir.",
            "The Trades tab offers the same calculators as fill-in forms.",
        )),
        mots_cles: "métier outil plombier électricien béton maçon couvreur gypse acier soudure terrassement estimation soumission trade tool plumber electrician concrete mason roofer drywall steel welding earthwork estimating bid",
        touches: &[],
        secondaire: false,
        trigo: false,
    },
];

/// Fiche de la fonction principale d'une touche.
pub fn fiche_principale(touche: Touche, trigo: bool) -> Option<&'static Fiche> {
    let touche = match touche {
        Touche::FractionRapide { .. } => {
            return FICHES.iter().find(|f| f.titre.fr == "Fractions rapides");
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
    /// casse ni aux accents, dans son titre, ses mots-clés ou son texte (celui
    /// de la langue en cours).
    pub fn correspond(&self, requete: &str) -> bool {
        let contenu = normaliser(&format!(
            "{} {} {} {} {} {}",
            self.titre,
            self.mots_cles,
            self.resume,
            self.mode_emploi,
            self.acces,
            self.note.unwrap_or(Texte::VIDE)
        ));
        normaliser(requete)
            .split_whitespace()
            .all(|mot| contenu.contains(mot))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use noyau::langue::{self, Langue};

    /// Fiches correspondant à une recherche; toutes si la requête est vide.
    fn chercher(requete: &str) -> Vec<&'static Fiche> {
        FICHES.iter().filter(|f| f.correspond(requete)).collect()
    }

    #[test]
    fn la_recherche_ignore_la_casse_et_les_accents() {
        let titres = |r: &str| chercher(r).iter().map(|f| f.titre.fr).collect::<Vec<_>>();
        assert!(titres("arete").contains(&"Arêtier Noue"));
        assert!(titres("ARÊTIER").contains(&"Arêtier Noue"));
        assert!(titres("beton").contains(&"Semelle"));
        assert!(titres("stud").contains(&"Montants"));
        assert!(titres("scie biseau").contains(&"Onglet composé"));
        assert!(titres("zzzz").is_empty());
        assert_eq!(chercher("").len(), FICHES.len());
        // Les mots anglais se cherchent aussi, quelle que soit la langue.
        assert!(titres("hip").contains(&"Arêtier Noue"));
        assert!(titres("drywall").contains(&"Gypse"));
        let _anglais = langue::pour_la_portee(Langue::Anglais);
        let titres = |r: &str| chercher(r).iter().map(|f| f.titre.en).collect::<Vec<_>>();
        assert!(titres("hip").contains(&"Hip Valley"));
        assert!(titres("plumb cut").contains(&"Diag"));
        assert!(titres("stringer").contains(&"Stair"));
        assert!(titres("SAW bevel").contains(&"Compound Mitre"));
        assert!(titres("arete").contains(&"Hip Valley"));
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
            fiche_principale(Touche::Longueur, true).map(|f| f.titre.fr),
            Some("sin, cos, tan")
        );
        assert_eq!(
            fiche_principale(Touche::Longueur, false).map(|f| f.titre.fr),
            Some("Longueur")
        );
    }

    #[test]
    fn les_exemples_citent_des_frappes_et_un_resultat() {
        for langue in Langue::TOUTES {
            let _langue = langue::pour_la_portee(langue);
            for f in FICHES {
                let titre = f.titre.traduit();
                assert!(!titre.is_empty(), "{langue:?}");
                assert!(!f.resume.traduit().is_empty(), "{titre}");
                assert!(!f.mode_emploi.traduit().is_empty(), "{titre}");
                if let Some((frappes, resultat)) = f.exemple {
                    assert!(frappes.traduit().contains('`'), "exemple de « {titre} »");
                    assert!(!resultat.traduit().is_empty());
                }
                // Les accents graves vont par paires.
                for texte in [f.mode_emploi, f.acces, f.note.unwrap_or(Texte::VIDE)] {
                    assert_eq!(
                        texte.traduit().matches('`').count() % 2,
                        0,
                        "accents graves de « {titre} »"
                    );
                }
            }
        }
    }

    /// Les textes d'une fiche, pour les comparer d'une langue à l'autre.
    fn textes(f: &Fiche) -> Vec<Texte> {
        let mut textes = vec![f.titre, f.acces, f.resume, f.mode_emploi];
        textes.extend(f.exemple.into_iter().flat_map(|(a, b)| [a, b]));
        textes.extend(f.note);
        textes
    }

    #[test]
    fn les_deux_langues_citent_les_memes_nombres_et_les_memes_touches() {
        let chiffres =
            |texte: &str| -> String { texte.chars().filter(char::is_ascii_digit).collect() };
        for f in FICHES {
            for texte in textes(f) {
                // Seule l'écriture des nombres change : les chiffres restent.
                assert_eq!(
                    chiffres(texte.fr),
                    chiffres(texte.en),
                    "« {} » : {}",
                    f.titre.fr,
                    texte.en
                );
                assert_eq!(
                    texte.fr.matches('`').count(),
                    texte.en.matches('`').count(),
                    "« {} » : {}",
                    f.titre.fr,
                    texte.en
                );
                assert_eq!(texte.fr.is_empty(), texte.en.is_empty(), "{}", f.titre.fr);
            }
        }
    }

    #[test]
    fn aucune_fiche_anglaise_n_est_restee_en_francais() {
        for f in FICHES {
            for texte in textes(f) {
                assert!(
                    !texte
                        .en
                        .chars()
                        .any(|c| "àâäçèéêëîïôöùûüœÀÂÇÈÉÊÎÔÙŒ".contains(c)),
                    "« {} » : {}",
                    f.titre.fr,
                    texte.en
                );
                // Ni virgule décimale ni symbole français d'unité.
                for mot in texte
                    .en
                    .split(|c: char| c.is_whitespace() || "()".contains(c))
                {
                    assert!(
                        !["pi", "po", "vg", "tc", "pi²", "po²", "pi³", "vg³", "pmp"].contains(&mot),
                        "« {} » : {}",
                        f.titre.fr,
                        texte.en
                    );
                }
            }
        }
    }
}
