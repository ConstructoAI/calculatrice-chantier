//! Calcul d'heures (section 22.20) : les heures et les minutes au clavier,
//! comme des pieds et des pouces, et les heures décimales.

mod commun;
use commun::verifier;

#[test]
fn heures_et_minutes() {
    verifier(&[
        ("7 h 45 min + 8 h 30 min =", "16 h 15"),
        // Un nombre laissé après les heures est en minutes.
        ("7 h 45 + 8 h 30 =", "16 h 15"),
        // La touche des heures passe aux heures décimales, puis revient.
        ("↳ h", "16,25 h"),
        ("↳ h", "16 h 15"),
        ("↳ min", "975 min"),
        ("45 min + 30 min =", "75 min"),
        ("↳ h", "1 h 15"),
        ("8 h 30 × 5 =", "42 h 30"),
        ("37 h 30 ÷ 5 =", "7 h 30"),
        ("40 h ÷ 8 h =", "5"),
        ("16 h 15 − 7 h 45 =", "8 h 30"),
        ("7 h 45 − 8 h 30 =", "−0 h 45"),
        // Des heures décimales et des heures et minutes : en heures et minutes.
        ("7,5 h + 0 h 45 min =", "8 h 15"),
        ("7,5 h + 0,25 h =", "7,75 h"),
        // Un nombre devient une durée; une durée ne s'ajoute pas à une longueur.
        ("7,5 Conv h", "7,5 h"),
        ("5 Pi + 3 h =", "ERREUR – unités incompatibles"),
    ]);
}
