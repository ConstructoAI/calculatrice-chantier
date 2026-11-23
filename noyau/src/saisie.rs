//! Machine de saisie des valeurs (section 7).
//!
//! Chaque frappe acceptée empile l'état précédent : le retour arrière (7.7) se
//! contente de le dépiler.

use crate::erreurs::Erreur;
use crate::etiquettes::{DEGRE, MINUTE, MOINS, SECONDE, symbole};
use crate::format::{Segment, TexteValeur, grouper, separateur_decimal};
use crate::prefs::Preferences;
use crate::unites::{Format, Genre};
use crate::valeur::Valeur;

/// Unité qu'une frappe peut apposer à la saisie.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UniteSaisie {
    Pieds,
    Pouces,
    Verges,
    Metres,
    Cm,
    Mm,
    Lb,
    Kg,
    Tc,
    T,
    Acre,
    /// Les heures et les minutes d'une durée : « 7 h 45 » (évolution 16).
    Heures,
    Minutes,
}

impl UniteSaisie {
    /// Unité qui accepte le carré et le cube par pressions répétées (R-19).
    fn accepte_exposant(self) -> bool {
        matches!(
            self,
            UniteSaisie::Pieds
                | UniteSaisie::Pouces
                | UniteSaisie::Verges
                | UniteSaisie::Metres
                | UniteSaisie::Cm
                | UniteSaisie::Mm
        )
    }

    fn format(self, exposant: u8) -> Format {
        use UniteSaisie::*;
        match (self, exposant) {
            (Pieds, 2) => Format::Pi2,
            (Pieds, 3) => Format::Pi3,
            (Pieds, _) => Format::PiPo,
            (Pouces, 2) => Format::Po2,
            (Pouces, 3) => Format::Po3,
            (Pouces, _) => Format::PoFrac,
            (Verges, 2) => Format::Vg2,
            (Verges, 3) => Format::Vg3,
            (Verges, _) => Format::Vg,
            (Metres, 2) => Format::M2,
            (Metres, 3) => Format::M3,
            (Metres, _) => Format::M,
            (Cm, 2) => Format::Cm2,
            (Cm, 3) => Format::Cm3,
            (Cm, _) => Format::Cm,
            (Mm, 2) => Format::Mm2,
            (Mm, 3) => Format::Mm3,
            (Mm, _) => Format::Mm,
            (Lb, _) => Format::Lb,
            (Kg, _) => Format::Kg,
            (Tc, _) => Format::Tc,
            (T, _) => Format::T,
            (Acre, _) => Format::Acre,
            (Heures, _) => Format::HMin,
            (Minutes, _) => Format::Min,
        }
    }
}

/// Une frappe de saisie.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Frappe {
    Chiffre(u8),
    Virgule,
    Unite(UniteSaisie),
    Barre,
    FractionRapide { num: u32, den: u32 },
}

/// Ce qu'il advient d'une frappe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Issue {
    Acceptee,
    /// Frappe sans objet : rien ne change.
    Ignoree,
    /// La saisie en cours est complète : la frappe commence une nouvelle saisie.
    Nouvelle,
    Erreur,
}

/// Nombre maximal de chiffres par nombre (7.1).
const MAX_CHIFFRES: usize = 12;
/// Nombre maximal de chiffres du numérateur et du dénominateur (7.3).
const MAX_CHIFFRES_FRACTION: usize = 4;

#[derive(Clone, Debug, Default, PartialEq)]
struct Etat {
    negatif: bool,
    /// Nombre en cours de frappe : chiffres et virgules.
    tampon: String,
    pieds: Option<String>,
    pouces: Option<String>,
    /// Les pouces ont reçu leur étiquette par la touche Pouces.
    pouces_etiquetes: bool,
    /// Numérateur et chiffres du dénominateur d'une fraction tapée avec « / ».
    fraction: Option<(u32, String)>,
    fraction_rapide: Option<(u32, u32)>,
    /// Unité simple (hors pieds et pouces) et son nombre.
    unite: Option<(UniteSaisie, String)>,
    /// 0 ou 1 : linéaire; 2 : carré; 3 : cube.
    exposant: u8,
    /// Les heures et les minutes d'une durée, comme les pieds et les pouces.
    heures: Option<String>,
    minutes: Option<String>,
}

impl Etat {
    fn nb_virgules(&self) -> usize {
        self.tampon.matches(',').count()
    }

    fn est_dms(&self) -> bool {
        self.nb_virgules() >= 2
    }

    fn a_fraction(&self) -> bool {
        self.fraction.is_some() || self.fraction_rapide.is_some()
    }

    /// La saisie est une durée : des heures ou des minutes.
    fn est_duree(&self) -> bool {
        self.heures.is_some() || self.minutes.is_some()
    }

    /// La saisie est une unité seule, sans rien d'autre.
    fn unite_seule(&self) -> Option<UniteSaisie> {
        if !self.tampon.is_empty() || self.a_fraction() {
            return None;
        }
        if self.est_duree() {
            return match (&self.heures, &self.minutes) {
                (Some(_), None) => Some(UniteSaisie::Heures),
                (None, Some(_)) => Some(UniteSaisie::Minutes),
                _ => None,
            };
        }
        match (&self.unite, &self.pieds, &self.pouces) {
            (Some((u, _)), None, None) => Some(*u),
            (None, Some(_), None) => Some(UniteSaisie::Pieds),
            (None, None, Some(_)) if self.pouces_etiquetes => Some(UniteSaisie::Pouces),
            _ => None,
        }
    }

    fn avancer_exposant(&mut self) {
        self.exposant = match self.exposant {
            0 | 1 => 2,
            2 => 3,
            _ => 1,
        };
    }

    /// La saisie ne peut plus recevoir de chiffres : elle est complète.
    fn est_complete(&self) -> bool {
        self.unite.is_some()
            || self.exposant > 1
            || self.fraction_rapide.is_some()
            || self.minutes.is_some()
    }

    fn appliquer(&mut self, frappe: Frappe) -> Issue {
        match frappe {
            Frappe::Chiffre(c) => {
                if self.est_complete() {
                    return Issue::Nouvelle;
                }
                let chiffre = char::from(b'0' + c.min(9));
                if let Some((_, den)) = &mut self.fraction {
                    if den.len() >= MAX_CHIFFRES_FRACTION {
                        return Issue::Ignoree;
                    }
                    den.push(chiffre);
                    return Issue::Acceptee;
                }
                let nb = self.tampon.chars().filter(char::is_ascii_digit).count();
                if nb >= MAX_CHIFFRES {
                    return Issue::Ignoree;
                }
                self.tampon.push(chiffre);
                Issue::Acceptee
            }
            Frappe::Virgule => {
                if self.est_complete() {
                    return Issue::Nouvelle;
                }
                if self.fraction.is_some() {
                    return Issue::Ignoree;
                }
                match self.nb_virgules() {
                    0 => {
                        if self.tampon.is_empty() {
                            self.tampon.push('0');
                        }
                        self.tampon.push(',');
                        Issue::Acceptee
                    }
                    // Seconde virgule d'un nombre sans unité : saisie D:M:S (7.5).
                    1 if self.pieds.is_none() && self.pouces.is_none() && !self.est_duree() => {
                        self.tampon.push(',');
                        Issue::Acceptee
                    }
                    _ => Issue::Ignoree,
                }
            }
            Frappe::Unite(u) => self.appliquer_unite(u),
            Frappe::Barre => {
                if self.est_complete()
                    || self.fraction.is_some()
                    || self.tampon.is_empty()
                    || self.est_duree()
                {
                    return Issue::Ignoree;
                }
                if self.tampon.contains(',') || self.tampon.len() > MAX_CHIFFRES_FRACTION {
                    return Issue::Erreur;
                }
                match self.tampon.parse::<u32>() {
                    Ok(num) => {
                        self.fraction = Some((num, String::new()));
                        self.tampon.clear();
                        Issue::Acceptee
                    }
                    Err(_) => Issue::Erreur,
                }
            }
            Frappe::FractionRapide { num, den } => {
                if self.est_duree() {
                    return Issue::Ignoree;
                }
                if self.unite.is_some() || self.exposant > 1 {
                    return Issue::Nouvelle;
                }
                if den == 0 {
                    return Issue::Erreur;
                }
                if !self.tampon.is_empty() {
                    if self.tampon.contains(',') || (self.pouces.is_some() && self.pouces_etiquetes)
                    {
                        return Issue::Erreur;
                    }
                    self.pouces = Some(std::mem::take(&mut self.tampon));
                    self.pouces_etiquetes = false;
                }
                // Une fraction déjà présente est remplacée (7.6).
                self.fraction = None;
                self.fraction_rapide = Some((num, den));
                Issue::Acceptee
            }
        }
    }

    fn appliquer_unite(&mut self, u: UniteSaisie) -> Issue {
        // Même unité pressée de nouveau, immédiatement : carré, cube, linéaire (R-19).
        if let Some(seule) = self.unite_seule() {
            if seule == u && u.accepte_exposant() {
                self.avancer_exposant();
                return Issue::Acceptee;
            }
            if seule == u {
                return Issue::Ignoree;
            }
        }
        match u {
            UniteSaisie::Heures => {
                if self.unite.is_some()
                    || self.pieds.is_some()
                    || self.pouces.is_some()
                    || self.a_fraction()
                    || self.minutes.is_some()
                {
                    return Issue::Ignoree;
                }
                if self.est_dms() {
                    return Issue::Erreur;
                }
                if self.heures.is_some() {
                    return if self.tampon.is_empty() {
                        Issue::Ignoree
                    } else {
                        Issue::Erreur
                    };
                }
                if self.tampon.is_empty() {
                    return Issue::Ignoree;
                }
                self.heures = Some(std::mem::take(&mut self.tampon));
                Issue::Acceptee
            }
            UniteSaisie::Minutes => {
                if self.unite.is_some()
                    || self.pieds.is_some()
                    || self.pouces.is_some()
                    || self.a_fraction()
                    || self.minutes.is_some()
                {
                    return Issue::Ignoree;
                }
                if self.est_dms() {
                    return Issue::Erreur;
                }
                if self.tampon.is_empty() {
                    return Issue::Ignoree;
                }
                self.minutes = Some(std::mem::take(&mut self.tampon));
                Issue::Acceptee
            }
            UniteSaisie::Pieds => {
                if self.unite.is_some() || self.exposant > 1 || self.est_duree() {
                    return Issue::Ignoree;
                }
                if self.est_dms() {
                    return Issue::Erreur;
                }
                if self.pieds.is_some() {
                    // Deux fois Pieds avec un nombre entre les deux (R-16).
                    return if self.tampon.is_empty() {
                        Issue::Ignoree
                    } else {
                        Issue::Erreur
                    };
                }
                if self.pouces.is_some() || self.a_fraction() {
                    return Issue::Erreur;
                }
                if self.tampon.is_empty() {
                    return Issue::Ignoree;
                }
                self.pieds = Some(std::mem::take(&mut self.tampon));
                self.exposant = 1;
                Issue::Acceptee
            }
            UniteSaisie::Pouces => {
                if self.unite.is_some() || self.exposant > 1 || self.est_duree() {
                    return Issue::Ignoree;
                }
                if self.est_dms() {
                    return Issue::Erreur;
                }
                if self.pouces.is_some() && self.pouces_etiquetes {
                    return if self.tampon.is_empty() {
                        Issue::Ignoree
                    } else {
                        Issue::Erreur
                    };
                }
                if self.a_fraction() || self.tampon.is_empty() {
                    return Issue::Ignoree;
                }
                self.pouces = Some(std::mem::take(&mut self.tampon));
                self.pouces_etiquetes = true;
                self.exposant = 1;
                Issue::Acceptee
            }
            _ => {
                if self.pieds.is_some()
                    || self.pouces.is_some()
                    || self.a_fraction()
                    || self.unite.is_some()
                    || self.est_duree()
                {
                    return Issue::Ignoree;
                }
                if self.est_dms() {
                    return Issue::Erreur;
                }
                if self.tampon.is_empty() {
                    return Issue::Ignoree;
                }
                self.unite = Some((u, std::mem::take(&mut self.tampon)));
                self.exposant = 1;
                Issue::Acceptee
            }
        }
    }
}

/// Lit un nombre tapé (« 17,32 », « 17, », « 5 »).
fn lire_decimal(texte: &str) -> Result<f64, Erreur> {
    if texte.is_empty() {
        return Ok(0.0);
    }
    texte
        .replace(',', ".")
        .parse::<f64>()
        .ok()
        .filter(|x| x.is_finite())
        .ok_or(Erreur::Saisie)
}

/// Résultat de la validation d'une saisie.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Validee {
    pub valeur: Valeur,
    /// Résolution temporaire demandée par une fraction plus fine (R-18).
    pub resolution_temp: Option<u32>,
}

/// Saisie en cours.
#[derive(Clone, Debug, Default)]
pub struct Saisie {
    etat: Etat,
    historique: Vec<Etat>,
}

impl Saisie {
    pub fn nouvelle() -> Saisie {
        Saisie::default()
    }

    /// Vrai dès qu'une frappe a été acceptée.
    pub fn est_active(&self) -> bool {
        !self.historique.is_empty()
    }

    pub fn vider(&mut self) {
        self.etat = Etat::default();
        self.historique.clear();
    }

    /// Soumet une frappe à la saisie.
    pub fn frapper(&mut self, frappe: Frappe) -> Issue {
        let mut suivant = self.etat.clone();
        let issue = suivant.appliquer(frappe);
        if issue == Issue::Acceptee {
            let precedent = std::mem::replace(&mut self.etat, suivant);
            self.historique.push(precedent);
        }
        issue
    }

    /// Annule la dernière frappe (7.7). Retourne faux s'il n'y avait rien à annuler.
    pub fn retour_arriere(&mut self) -> bool {
        match self.historique.pop() {
            Some(precedent) => {
                let negatif = self.etat.negatif;
                self.etat = precedent;
                self.etat.negatif = negatif && self.est_active();
                true
            }
            None => false,
        }
    }

    /// Change le signe de la saisie (+/−).
    pub fn changer_signe(&mut self) {
        self.etat.negatif = !self.etat.negatif;
    }

    /// Vrai si la saisie est un nombre nu, sans unité ni fraction.
    pub fn est_nombre_simple(&self) -> bool {
        let e = &self.etat;
        e.pieds.is_none()
            && e.pouces.is_none()
            && !e.a_fraction()
            && e.unite.is_none()
            && !e.est_duree()
    }

    /// Unité que porte seule la saisie (« 5 cm », « 100 pi »), s'il y a lieu.
    pub fn unite_seule(&self) -> Option<UniteSaisie> {
        self.etat.unite_seule()
    }

    /// Valide la saisie et retourne sa valeur (R-13 à R-18, R-21).
    pub fn valider(&self, resolution: u32) -> Result<Validee, Erreur> {
        let e = &self.etat;
        let signe = if e.negatif { -1.0 } else { 1.0 };
        let simple = |valeur: Valeur| Validee {
            valeur: Valeur {
                base: signe * valeur.base,
                ..valeur
            },
            resolution_temp: None,
        };

        // Une durée : les heures, puis les minutes (un nombre laissé en suspens
        // après les heures est en minutes, comme des pouces après des pieds).
        if e.est_duree() {
            let mut minutes = 0.0;
            if let Some(h) = &e.heures {
                minutes += lire_decimal(h)? * 60.0;
            }
            let suspendues = e.minutes.is_none() && !e.tampon.is_empty();
            match &e.minutes {
                Some(m) => minutes += lire_decimal(m)?,
                None if suspendues => minutes += lire_decimal(&e.tampon)?,
                None => {}
            }
            let format = match (&e.heures, &e.minutes) {
                (Some(h), None) if !suspendues && h.contains(',') => Format::HDec,
                (None, Some(_)) => Format::Min,
                _ => Format::HMin,
            };
            return Ok(simple(Valeur::en_base(minutes, format)));
        }

        // Unité simple, avec exposant éventuel.
        if let Some((u, nombre)) = &e.unite {
            let x = lire_decimal(nombre)?;
            return Ok(simple(Valeur::avec_format(x, u.format(e.exposant))));
        }

        if e.pieds.is_some() || e.pouces.is_some() || e.a_fraction() {
            // Unité seule au carré ou au cube (R-19).
            if e.exposant > 1 {
                let (unite, nombre) = match (&e.pieds, &e.pouces) {
                    (Some(p), _) => (UniteSaisie::Pieds, p),
                    (None, Some(p)) => (UniteSaisie::Pouces, p),
                    _ => return Err(Erreur::Saisie),
                };
                return Ok(simple(Valeur::avec_format(
                    lire_decimal(nombre)?,
                    unite.format(e.exposant),
                )));
            }
            let mut pouces = 0.0;
            let mut pieds_decimaux = false;
            let mut pouces_decimaux = false;
            let mut a_des_pouces = false;
            if let Some(p) = &e.pieds {
                pouces += lire_decimal(p)? * 12.0;
                pieds_decimaux = p.contains(',');
            }
            if let Some(p) = &e.pouces {
                pouces += lire_decimal(p)?;
                pouces_decimaux = p.contains(',');
                a_des_pouces = true;
            }
            let mut resolution_temp = None;
            if let Some((num, den)) = &e.fraction {
                let d = if den.is_empty() {
                    // Pas de dénominateur : la résolution en vigueur (R-17).
                    resolution
                } else {
                    den.parse::<u32>().map_err(|_| Erreur::Saisie)?
                };
                if d == 0 {
                    return Err(Erreur::Saisie);
                }
                pouces += f64::from(*num) / f64::from(d);
                if !den.is_empty() && d.is_power_of_two() && d > resolution && d <= 64 {
                    resolution_temp = Some(d);
                }
            } else if let Some((num, den)) = e.fraction_rapide {
                if den == 0 {
                    return Err(Erreur::Saisie);
                }
                pouces += f64::from(num) / f64::from(den);
            }
            if !e.tampon.is_empty() {
                // Nombre laissé en suspens (R-15).
                if e.pouces.is_some() || e.pieds.is_none() {
                    return Err(Erreur::Saisie);
                }
                pouces += lire_decimal(&e.tampon)?;
                pouces_decimaux = e.tampon.contains(',');
                a_des_pouces = true;
            }
            let format = if e.pieds.is_some() {
                if pieds_decimaux && !a_des_pouces && !e.a_fraction() {
                    Format::PiDec
                } else {
                    Format::PiPo
                }
            } else if a_des_pouces && pouces_decimaux && !e.a_fraction() {
                Format::PoDec
            } else {
                Format::PoFrac
            };
            return Ok(Validee {
                valeur: Valeur::en_base(signe * pouces, format),
                resolution_temp,
            });
        }

        // Angle en degrés-minutes-secondes (R-21).
        if e.est_dms() {
            let mut parties = e.tampon.split(',');
            let mut lire_partie = || lire_decimal(parties.next().unwrap_or(""));
            let degres = lire_partie()?;
            let minutes = lire_partie()?;
            let secondes = lire_partie()?;
            return Ok(simple(Valeur {
                base: degres + minutes / 60.0 + secondes / 3600.0,
                genre: Genre::SansUnite,
                format: Format::Dms,
            }));
        }

        Ok(simple(Valeur::nombre(lire_decimal(&e.tampon)?)))
    }

    /// Écho de la saisie, dans l'ordre de la frappe (7.8).
    pub fn echo(&self, prefs: &Preferences) -> TexteValeur {
        let e = &self.etat;
        let habiller = |nombre: &str| -> String {
            match nombre.split_once(',') {
                Some((entier, decimales)) => {
                    format!(
                        "{}{}{}",
                        grouper(entier, prefs),
                        separateur_decimal(prefs),
                        decimales
                    )
                }
                None => grouper(nombre, prefs),
            }
        };
        let avec_exposant = |format: Format| symbole(format).to_string();
        let mut segments: Vec<Segment> = Vec::new();
        if e.est_duree() {
            if let Some(h) = &e.heures {
                segments.push(Segment::Texte(habiller(h)));
                segments.push(Segment::Unite(symbole(Format::HMin).to_string()));
            }
            if let Some(m) = &e.minutes {
                segments.push(Segment::Texte(habiller(m)));
                segments.push(Segment::Unite(symbole(Format::Min).to_string()));
            } else if !e.tampon.is_empty() {
                segments.push(Segment::Texte(habiller(&e.tampon)));
            }
        } else if let Some((u, nombre)) = &e.unite {
            segments.push(Segment::Texte(habiller(nombre)));
            segments.push(Segment::Unite(avec_exposant(u.format(e.exposant))));
        } else {
            if let Some(p) = &e.pieds {
                segments.push(Segment::Texte(habiller(p)));
                let format = match e.exposant {
                    2 => Format::Pi2,
                    3 => Format::Pi3,
                    _ => Format::PiPo,
                };
                segments.push(Segment::Unite(avec_exposant(format)));
            }
            if let Some(p) = &e.pouces {
                segments.push(Segment::Texte(habiller(p)));
                if e.pouces_etiquetes {
                    let format = match e.exposant {
                        2 if e.pieds.is_none() => Format::Po2,
                        3 if e.pieds.is_none() => Format::Po3,
                        _ => Format::PoFrac,
                    };
                    segments.push(Segment::Unite(avec_exposant(format)));
                }
            }
            if let Some((num, den)) = &e.fraction {
                match den.parse::<u32>() {
                    Ok(d) => segments.push(Segment::Fraction { num: *num, den: d }),
                    Err(_) => segments.push(Segment::Texte(format!("{num}/"))),
                }
            }
            if let Some((num, den)) = e.fraction_rapide {
                segments.push(Segment::Fraction { num, den });
            }
            if !e.tampon.is_empty() {
                if e.est_dms() {
                    let mut parties = e.tampon.split(',');
                    let mut texte = String::new();
                    texte.push_str(parties.next().unwrap_or(""));
                    texte.push(DEGRE);
                    let minutes = parties.next().unwrap_or("");
                    if !minutes.is_empty() {
                        texte.push_str(minutes);
                        texte.push(MINUTE);
                    }
                    let secondes = parties.next().unwrap_or("");
                    if !secondes.is_empty() {
                        texte.push_str(secondes);
                        texte.push(SECONDE);
                    }
                    segments.push(Segment::Texte(texte));
                } else {
                    segments.push(Segment::Texte(habiller(&e.tampon)));
                }
            }
        }
        if segments.is_empty() {
            segments.push(Segment::Texte("0".to_string()));
        }
        if e.negatif {
            match segments.first_mut() {
                Some(Segment::Texte(t)) => t.insert(0, MOINS),
                _ => segments.insert(0, Segment::Texte(MOINS.to_string())),
            }
        }
        TexteValeur(segments)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::etiquettes::INSECABLE;

    fn taper(saisie: &mut Saisie, frappes: &[Frappe]) {
        for f in frappes {
            assert_eq!(
                saisie.frapper(*f),
                Issue::Acceptee,
                "frappe refusée : {f:?}"
            );
        }
    }

    fn c(n: u8) -> Frappe {
        Frappe::Chiffre(n)
    }

    const PI: Frappe = Frappe::Unite(UniteSaisie::Pieds);
    const PO: Frappe = Frappe::Unite(UniteSaisie::Pouces);

    fn echo(s: &Saisie) -> String {
        s.echo(&Preferences::default())
            .en_texte()
            .replace(INSECABLE, " ")
    }

    #[test]
    fn pieds_pouces_fraction() {
        let mut s = Saisie::nouvelle();
        taper(&mut s, &[c(5), PI, c(1), PO, c(1), Frappe::Barre]);
        assert_eq!(echo(&s), "5 pi 1 po 1/");
        taper(&mut s, &[c(2)]);
        assert_eq!(echo(&s), "5 pi 1 po 1/2");
        let v = s.valider(16).unwrap().valeur;
        assert_eq!((v.base, v.format), (61.5, Format::PiPo));
    }

    #[test]
    fn formats_selon_la_frappe() {
        let cas: [(&[Frappe], f64, Format); 6] = [
            (&[c(5), PI, c(6)], 66.0, Format::PiPo),
            (&[c(5), PI, c(1), Frappe::Barre, c(2)], 60.5, Format::PiPo),
            (
                &[c(8), PO, c(1), Frappe::Barre, c(8)],
                8.125,
                Format::PoFrac,
            ),
            (&[c(1), Frappe::Barre, c(2)], 0.5, Format::PoFrac),
            (
                &[c(1), c(7), Frappe::Virgule, c(3), c(2), PI],
                17.32 * 12.0,
                Format::PiDec,
            ),
            (&[c(9), Frappe::Virgule, c(5), PO], 9.5, Format::PoDec),
        ];
        for (frappes, base, format) in cas {
            let mut s = Saisie::nouvelle();
            taper(&mut s, frappes);
            let v = s.valider(16).unwrap().valeur;
            assert!((v.base - base).abs() < 1e-9, "{frappes:?}");
            assert_eq!(v.format, format, "{frappes:?}");
        }
    }

    #[test]
    fn exposants_par_repetition() {
        let mut s = Saisie::nouvelle();
        taper(&mut s, &[c(1), c(0), c(0), PI, PI]);
        assert_eq!(echo(&s), "100 pi²");
        assert_eq!(
            s.valider(16).unwrap().valeur,
            Valeur::avec_format(100.0, Format::Pi2)
        );
        taper(&mut s, &[PI]);
        assert_eq!(
            s.valider(16).unwrap().valeur,
            Valeur::avec_format(100.0, Format::Pi3)
        );
        taper(&mut s, &[PI]);
        assert_eq!(s.valider(16).unwrap().valeur.format, Format::PiPo);
    }

    #[test]
    fn fraction_sans_denominateur_et_resolution_temporaire() {
        let mut s = Saisie::nouvelle();
        taper(&mut s, &[c(1), c(5), Frappe::Barre]);
        assert_eq!(s.valider(16).unwrap().valeur.base, 15.0 / 16.0);
        let mut s = Saisie::nouvelle();
        taper(&mut s, &[c(4), c(4), Frappe::Barre, c(6), c(4)]);
        let v = s.valider(16).unwrap();
        assert_eq!(v.resolution_temp, Some(64));
        let mut s = Saisie::nouvelle();
        taper(&mut s, &[c(1), Frappe::Barre, c(0)]);
        assert_eq!(s.valider(16), Err(Erreur::Saisie));
    }

    #[test]
    fn erreurs_de_saisie() {
        let mut s = Saisie::nouvelle();
        taper(&mut s, &[c(5), PI, c(6)]);
        assert_eq!(s.frapper(PI), Issue::Erreur);
        let mut s = Saisie::nouvelle();
        taper(&mut s, &[c(5), PO, c(6)]);
        assert_eq!(s.valider(16), Err(Erreur::Saisie));
        let mut s = Saisie::nouvelle();
        taper(&mut s, &[c(1), Frappe::Virgule, c(5)]);
        assert_eq!(s.frapper(Frappe::Barre), Issue::Erreur);
    }

    #[test]
    fn degres_minutes_secondes() {
        let mut s = Saisie::nouvelle();
        taper(
            &mut s,
            &[c(2), c(3), Frappe::Virgule, c(4), c(2), Frappe::Virgule],
        );
        assert_eq!(echo(&s), "23°42′");
        taper(&mut s, &[c(3), c(9)]);
        assert_eq!(echo(&s), "23°42′39″");
        let v = s.valider(16).unwrap().valeur;
        assert_eq!(v.format, Format::Dms);
        assert!((v.base - (23.0 + 42.0 / 60.0 + 39.0 / 3600.0)).abs() < 1e-12);
    }

    #[test]
    fn fractions_rapides_et_retour_arriere() {
        let mut s = Saisie::nouvelle();
        taper(
            &mut s,
            &[c(5), PI, c(6), Frappe::FractionRapide { num: 3, den: 8 }],
        );
        assert_eq!(s.valider(16).unwrap().valeur.base, 66.375);
        assert_eq!(echo(&s), "5 pi 6 3/8");
        assert!(s.retour_arriere());
        assert_eq!(echo(&s), "5 pi 6");
        assert!(s.retour_arriere());
        assert!(s.retour_arriere());
        assert!(s.retour_arriere());
        assert!(!s.est_active());
        assert!(!s.retour_arriere());
    }

    #[test]
    fn douze_chiffres_au_plus() {
        let mut s = Saisie::nouvelle();
        for _ in 0..12 {
            assert_eq!(s.frapper(c(9)), Issue::Acceptee);
        }
        assert_eq!(s.frapper(c(9)), Issue::Ignoree);
        assert_eq!(echo(&s), "999 999 999 999");
    }

    #[test]
    fn unite_simple_puis_chiffre_commence_une_nouvelle_saisie() {
        let mut s = Saisie::nouvelle();
        taper(&mut s, &[c(5), Frappe::Unite(UniteSaisie::Metres)]);
        assert_eq!(s.frapper(c(6)), Issue::Nouvelle);
        taper(&mut s, &[Frappe::Unite(UniteSaisie::Metres)]);
        assert_eq!(
            s.valider(16).unwrap().valeur,
            Valeur::avec_format(5.0, Format::M2)
        );
    }
}
