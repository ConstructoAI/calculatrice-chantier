//! Partager un plan par code QR (évolution 9 de la section 27) : le plan en
//! JSON compact — sans les champs restés à leur valeur de départ, les nombres
//! arrondis au dix-millième —, en base64 dans l'adresse du site (`…/#plan=…`).
//! Le collègue qui scanne le code ouvre la version Web, qui propose d'ajouter
//! le plan à ses pièces. Rien n'est envoyé à un serveur : le plan voyage dans
//! le « # » de l'adresse. Le JSON s'y comprime d'abord, par une compression
//! écrite ici (des reprises de ce qui précède, à la manière de LZ77) : un plan
//! de trois pièces tient alors dans un code bien plus petit, plus facile à
//! scanner. L'adresse commence par « 1. », le numéro de ce format.

use noyau::dessin::{Note, Ouverture, Piece, Plan, Retrait};
use serde_json::Value;

use crate::stockage::{de_base64, en_base64};

/// L'adresse du site, où s'ouvre la version Web.
pub const ADRESSE_DU_SITE: &str = "https://constructoai.github.io/calculatrice-chantier/";

/// L'adresse qui porte tout le plan.
pub fn adresse_du_plan(plan: &Plan) -> String {
    adresse(plan)
}

/// L'adresse qui porte une seule pièce du plan.
pub fn adresse_de_la_piece(plan: &Plan, rang: usize) -> String {
    let piece = plan.pieces.get(rang).cloned().unwrap_or_default();
    adresse(&Plan {
        pieces: vec![Piece {
            place: None,
            ..piece
        }],
        courante: 0,
        mise_en_page: plan.mise_en_page.clone(),
    })
}

fn adresse(plan: &Plan) -> String {
    format!(
        "{ADRESSE_DU_SITE}#plan={FORMAT}{}",
        en_base64_pour_l_adresse(&comprimer(json_compact(plan).as_bytes()))
    )
}

/// Le plan en JSON compact.
fn json_compact(plan: &Plan) -> String {
    let mut valeur = serde_json::to_value(plan).unwrap_or(Value::Null);
    let defaut = serde_json::to_value(Plan::default()).unwrap_or(Value::Null);
    elaguer(&mut valeur, &defaut);
    serde_json::to_string(&valeur).unwrap_or_default()
}

/// Le plan que porte la fin d'une adresse (ce qui suit « #plan= »); `None`
/// s'il est illisible.
pub fn plan_de_l_adresse(texte: &str) -> Option<Plan> {
    let texte = texte
        .trim()
        .split(['&', '#'])
        .next()?
        .strip_prefix(FORMAT)?;
    let mut base64: String = texte
        .chars()
        .map(|c| match c {
            '-' => '+',
            '_' => '/',
            autre => autre,
        })
        .collect();
    while !base64.len().is_multiple_of(4) {
        base64.push('=');
    }
    let json = String::from_utf8(decomprimer(&de_base64(&base64)?)?).ok()?;
    let mut plan: Plan = serde_json::from_str(&json).ok()?;
    if plan.pieces.is_empty() {
        return None;
    }
    plan.normaliser();
    Some(plan)
}

/// Le numéro du format de l'adresse : un JSON compact, comprimé, en base64.
const FORMAT: &str = "1.";

/// La plus longue reprise : 131 octets.
const PLUS_LONGUE_REPRISE: usize = 131;

/// Comprime des octets : une suite de jetons, chacun un octet. De 0 à 127 :
/// autant d'octets tels quels que le jeton plus un, qui suivent. De 128 à
/// 255 : une reprise de 4 à 131 octets (le jeton moins 124), à la distance
/// que donnent les deux octets suivants, de ce qui précède.
fn comprimer(octets: &[u8]) -> Vec<u8> {
    let mut sortie = Vec::with_capacity(octets.len());
    let mut tels_quels: Vec<u8> = Vec::new();
    let vider = |sortie: &mut Vec<u8>, tels_quels: &mut Vec<u8>| {
        for morceau in tels_quels.chunks(128) {
            sortie.push((morceau.len() - 1) as u8);
            sortie.extend_from_slice(morceau);
        }
        tels_quels.clear();
    };
    let mut i = 0;
    while i < octets.len() {
        // La plus longue reprise de ce qui précède, au plus 65 535 octets en
        // arrière; elle peut chevaucher ce qu'elle recopie.
        let mut meilleure = (0, 0);
        for j in i.saturating_sub(usize::from(u16::MAX))..i {
            let mut longueur = 0;
            while i + longueur < octets.len()
                && longueur < PLUS_LONGUE_REPRISE
                && octets[j + longueur] == octets[i + longueur]
            {
                longueur += 1;
            }
            if longueur > meilleure.0 {
                meilleure = (longueur, i - j);
            }
        }
        // Une reprise coûte trois octets : elle vaut la peine dès quatre.
        if meilleure.0 >= 4 {
            vider(&mut sortie, &mut tels_quels);
            sortie.push(0x80 | (meilleure.0 - 4) as u8);
            sortie.extend_from_slice(&(meilleure.1 as u16).to_be_bytes());
            i += meilleure.0;
        } else {
            tels_quels.push(octets[i]);
            i += 1;
        }
    }
    vider(&mut sortie, &mut tels_quels);
    sortie
}

/// Défait `comprimer`; `None` si les jetons ne se suivent pas bien (ou si le
/// résultat dépasse un million d'octets).
fn decomprimer(jetons: &[u8]) -> Option<Vec<u8>> {
    let mut sortie: Vec<u8> = Vec::with_capacity(jetons.len() * 3);
    let mut i = 0;
    while i < jetons.len() {
        let jeton = jetons[i];
        i += 1;
        if jeton < 0x80 {
            let nombre = usize::from(jeton) + 1;
            sortie.extend_from_slice(jetons.get(i..i + nombre)?);
            i += nombre;
        } else {
            let longueur = usize::from(jeton & 0x7F) + 4;
            let distance = usize::from(u16::from_be_bytes([*jetons.get(i)?, *jetons.get(i + 1)?]));
            i += 2;
            if distance == 0 || distance > sortie.len() {
                return None;
            }
            let debut = sortie.len() - distance;
            for k in 0..longueur {
                let octet = sortie[debut + k];
                sortie.push(octet);
            }
        }
        if sortie.len() > 1 << 20 {
            return None;
        }
    }
    Some(sortie)
}

/// Du base64 qui va tel quel dans une adresse : « - » et « _ » au lieu de
/// « + » et « / », sans « = » à la fin.
fn en_base64_pour_l_adresse(octets: &[u8]) -> String {
    en_base64(octets)
        .chars()
        .filter(|&c| c != '=')
        .map(|c| match c {
            '+' => '-',
            '/' => '_',
            autre => autre,
        })
        .collect()
}

/// Le gabarit d'un élément d'une liste, d'après le nom de la liste : une
/// structure qui reprend sa valeur de départ pour chaque champ absent.
fn gabarit(cle: &str) -> Option<Value> {
    match cle {
        "pieces" => serde_json::to_value(Piece::default()).ok(),
        "ouvertures" => serde_json::to_value(Ouverture::default()).ok(),
        "retraits" => serde_json::to_value(Retrait::default()).ok(),
        "notes" => serde_json::to_value(Note::default()).ok(),
        "cloisons" => serde_json::to_value(noyau::cloison::Cloison::default()).ok(),
        // Un objet garde toujours son genre et son centre, qui n'ont pas de
        // valeur de départ.
        "objets" => {
            let mut gabarit = serde_json::to_value(noyau::mobilier::Objet::nouveau(
                noyau::mobilier::GenreDObjet::Table,
                (0.0, 0.0),
            ))
            .ok()?;
            if let Value::Object(champs) = &mut gabarit {
                champs.remove("genre");
                champs.remove("centre");
            }
            Some(gabarit)
        }
        _ => None,
    }
}

/// Retire de `valeur` ce qui vaut ce que porte `defaut` au même endroit, et
/// arrondit les nombres au dix-millième.
fn elaguer(valeur: &mut Value, defaut: &Value) {
    let Value::Object(champs) = valeur else {
        arrondir(valeur);
        return;
    };
    let defauts = match defaut {
        Value::Object(defauts) => defauts.clone(),
        _ => serde_json::Map::new(),
    };
    champs.retain(|cle, v| defauts.get(cle) != Some(v));
    for (cle, v) in champs.iter_mut() {
        match (v, defauts.get(cle)) {
            (v @ Value::Object(_), Some(d @ Value::Object(_))) => elaguer(v, d),
            (Value::Array(elements), _) => match gabarit(cle) {
                Some(gabarit) => elements.iter_mut().for_each(|e| elaguer(e, &gabarit)),
                None => elements.iter_mut().for_each(arrondir),
            },
            (v, _) => arrondir(v),
        }
    }
}

/// Les nombres arrondis au dix-millième (un joint de 1/16 po reste exact);
/// un nombre rond s'écrit sans « .0 ».
fn arrondir(valeur: &mut Value) {
    match valeur {
        Value::Number(nombre) if nombre.is_f64() => {
            let x = nombre.as_f64().unwrap_or(0.0);
            let arrondi = (x * 10_000.0).round() / 10_000.0;
            *valeur = if arrondi.fract() == 0.0 && arrondi.abs() < 1e15 {
                Value::from(arrondi as i64)
            } else {
                serde_json::Number::from_f64(arrondi).map_or(Value::Null, Value::Number)
            };
        }
        Value::Array(elements) => elements.iter_mut().for_each(arrondir),
        Value::Object(champs) => champs.values_mut().for_each(arrondir),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use noyau::dessin::{Cote, Dessin, Sens};

    fn piece(nom: &str, longueur: &str, largeur: &str) -> Piece {
        let cote = |sens, saisie: &str| Cote {
            sens,
            croquis: 150.0,
            saisie: saisie.to_string(),
        };
        Piece {
            nom: nom.into(),
            dessin: Dessin {
                cotes: vec![
                    cote(Sens::Est, longueur),
                    cote(Sens::Sud, largeur),
                    cote(Sens::Ouest, ""),
                    cote(Sens::Nord, ""),
                ],
                fermee: true,
                hauteur_des_murs: "8".into(),
                ouvertures: vec![Ouverture {
                    largeur: "32 po".into(),
                    hauteur: "80 po".into(),
                    cote: Some(0),
                    position: "2".into(),
                    ..Ouverture::default()
                }],
                objets: vec![noyau::mobilier::Objet::nouveau(
                    noyau::mobilier::GenreDObjet::Lit,
                    (40.5, 60.25),
                )],
                ..Dessin::default()
            },
            ..Piece::default()
        }
    }

    #[test]
    fn un_plan_fait_l_aller_retour_par_l_adresse() {
        let plan = Plan {
            pieces: vec![piece("Salon", "12", "10"), piece("Chambre", "10", "8")],
            ..Plan::default()
        };
        let adresse = adresse_du_plan(&plan);
        let fin = adresse.strip_prefix(ADRESSE_DU_SITE).unwrap();
        let texte = fin.strip_prefix("#plan=").unwrap();
        let (format, base64) = texte.split_at(2);
        assert_eq!(format, FORMAT);
        // Rien que des caractères qui vont tels quels dans une adresse.
        assert!(
            base64
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        );
        // Comprimé, le plan prend bien moins de place que son JSON compact.
        let compact = json_compact(&plan);
        assert!(comprimer(compact.as_bytes()).len() * 2 < compact.len());
        assert_eq!(plan_de_l_adresse(texte), Some(plan.clone()));
        // Le JSON compact est bien plus court que le JSON complet.
        let complet = serde_json::to_string(&plan).unwrap();
        assert!(
            json_compact(&plan).len() * 3 < complet.len(),
            "{}",
            json_compact(&plan)
        );
        // Une pièce seule.
        let seule = plan_de_l_adresse(
            adresse_de_la_piece(&plan, 1)
                .split("#plan=")
                .nth(1)
                .unwrap(),
        )
        .unwrap();
        assert_eq!(seule.pieces.len(), 1);
        assert_eq!(seule.pieces[0].nom, "Chambre");
        // Une adresse illisible ne donne rien.
        assert!(plan_de_l_adresse("pas du base64 !").is_none());
        assert!(plan_de_l_adresse("").is_none());
    }

    #[test]
    fn la_compression_fait_l_aller_retour() {
        // Du texte qui se répète (avec des reprises qui chevauchent ce
        // qu'elles recopient), des octets quelconques, du vide.
        let repete =
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".repeat(9);
        let quelconques: Vec<u8> = (0..3000u32).map(|k| (k * 7919 % 251) as u8).collect();
        for octets in [repete.as_bytes(), &quelconques, b"", b"abc"] {
            assert_eq!(decomprimer(&comprimer(octets)).as_deref(), Some(octets));
        }
        assert!(comprimer(repete.as_bytes()).len() < 30);
        // Des jetons qui ne se suivent pas bien ne donnent rien.
        assert!(decomprimer(&[0x05, b'a']).is_none());
        assert!(decomprimer(&[0x80, 0, 9]).is_none());
    }

    #[test]
    fn les_nombres_s_arrondissent_au_dix_millieme() {
        let mut valeur = serde_json::json!({"a": 12.345678, "b": [200.0, 1.005], "c": 3});
        arrondir(&mut valeur);
        assert_eq!(
            valeur,
            serde_json::json!({"a": 12.3457, "b": [200, 1.005], "c": 3})
        );
    }
}
