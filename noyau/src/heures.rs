//! Calcul d'heures (section 22.20) : lire une heure du jour et une durée, et
//! compter les heures d'une journée de travail — arrivée, départ, dîner —
//! pour la feuille de temps de la semaine. Aucun taux de salaire.

/// Une journée, en minutes.
const MINUTES_PAR_JOUR: f64 = 24.0 * 60.0;

/// Un nombre positif, la virgule ou le point comme séparateur décimal.
fn nombre(texte: &str) -> Option<f64> {
    texte
        .trim()
        .replace(',', ".")
        .parse::<f64>()
        .ok()
        .filter(|x| x.is_finite() && *x >= 0.0)
}

/// Des heures et des minutes : « 7:30 », « 7 h 30 », « 7h30 », « 7 h »,
/// « 45 min », ou un nombre seul — en minutes s'il est entier et que
/// `entier_en_minutes`, en heures sinon.
fn lire_heures_minutes(texte: &str, entier_en_minutes: bool) -> Option<f64> {
    let texte = texte.trim().to_lowercase();
    if texte.is_empty() {
        return None;
    }
    if let Some(minutes) = texte.strip_suffix("min") {
        return nombre(minutes);
    }
    for separateur in [':', 'h'] {
        if let Some((heures, minutes)) = texte.split_once(separateur) {
            let heures = nombre(heures)?;
            let minutes = minutes.trim().trim_end_matches("min").trim();
            let minutes = if minutes.is_empty() {
                0.0
            } else {
                nombre(minutes)?
            };
            return (minutes < 60.0).then_some(heures * 60.0 + minutes);
        }
    }
    let x = nombre(&texte)?;
    let entier = !texte.contains([',', '.']);
    Some(if entier_en_minutes && entier {
        x
    } else {
        x * 60.0
    })
}

/// Lit une heure du jour, sur 24 heures : « 7:30 », « 7 h 30 », « 7h30 »,
/// « 16 » ou « 16,5 » — en minutes depuis minuit.
pub fn lire_une_heure(texte: &str) -> Option<f64> {
    lire_heures_minutes(texte, false).filter(|minutes| *minutes <= MINUTES_PAR_JOUR)
}

/// Lit une durée, celle d'un dîner : « 0 h 30 », « 1:15 », « 1 h »,
/// « 0,5 h », « 45 min », ou un nombre entier seul, en minutes (« 30 »).
pub fn lire_une_duree(texte: &str) -> Option<f64> {
    lire_heures_minutes(texte, true)
}

/// Les minutes travaillées d'une journée : du départ moins l'arrivée — le
/// lendemain si le départ vient avant l'arrivée —, moins le dîner; `None` si
/// le dîner dépasse la journée.
pub fn minutes_du_jour(arrivee: f64, depart: f64, diner: f64) -> Option<f64> {
    let mut presence = depart - arrivee;
    if presence < 0.0 {
        presence += MINUTES_PAR_JOUR;
    }
    let travail = presence - diner;
    (travail >= 0.0).then_some(travail)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lire_les_heures_du_jour() {
        for texte in ["7:30", "7 h 30", "7h30", "7 H 30", "7,5"] {
            assert_eq!(lire_une_heure(texte), Some(450.0), "{texte}");
        }
        assert_eq!(lire_une_heure("16"), Some(960.0));
        assert_eq!(lire_une_heure("16 h"), Some(960.0));
        assert_eq!(lire_une_heure("24:00"), Some(1440.0));
        for texte in ["", "25", "7:75", "sept", "-1"] {
            assert_eq!(lire_une_heure(texte), None, "{texte}");
        }
    }

    #[test]
    fn lire_les_durees() {
        assert_eq!(lire_une_duree("30"), Some(30.0));
        assert_eq!(lire_une_duree("0 h 30"), Some(30.0));
        assert_eq!(lire_une_duree("1:15"), Some(75.0));
        assert_eq!(lire_une_duree("1 h"), Some(60.0));
        assert_eq!(lire_une_duree("0,5 h"), Some(30.0));
        assert_eq!(lire_une_duree("0,5"), Some(30.0));
        assert_eq!(lire_une_duree("45 min"), Some(45.0));
        assert_eq!(lire_une_duree("quinze"), None);
    }

    #[test]
    fn compter_une_journee() {
        // De 7 h 30 à 16 h 30, une demi-heure de dîner : 8 h 30.
        assert_eq!(minutes_du_jour(450.0, 990.0, 30.0), Some(510.0));
        // De nuit : de 22 h à 6 h, sans dîner.
        assert_eq!(minutes_du_jour(1320.0, 360.0, 0.0), Some(480.0));
        // Un dîner plus long que la journée.
        assert_eq!(minutes_du_jour(450.0, 480.0, 60.0), None);
    }
}
