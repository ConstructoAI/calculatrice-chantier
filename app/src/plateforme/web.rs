//! Version Web : données dans le stockage local du navigateur, export par
//! téléchargement, plein écran du navigateur.
//!
//! La page qui héberge l'application est `app/index.html` : les identifiants
//! ci-dessous sont ceux de ses éléments.

use std::io;

use noyau::langue::Langue;
use wasm_bindgen::JsCast as _;
use wasm_bindgen::closure::Closure;

use super::{Export, Fichier, Installation, Lecture};
use crate::textes;

/// L'application tourne-t-elle dans un navigateur?
pub const NAVIGATEUR: bool = true;
/// Le stockage local est limité (environ 5 Mo par site) : JSON compact.
pub const JSON_LISIBLE: bool = false;
/// Une page peut être fermée sans préavis : chaque changement est enregistré
/// aussitôt (l'écriture dans le stockage local est immédiate et peu coûteuse).
pub const INTERVALLE_SAUVEGARDE: f64 = 0.0;

/// Le canevas où l'application se dessine.
const ID_CANEVAS: &str = "calculatrice";
/// L'écran d'attente, retiré dès que l'application est prête.
const ID_CHARGEMENT: &str = "chargement";
/// Le message d'échec, caché tant que tout va bien.
const ID_ECHEC: &str = "echec";

fn document() -> Option<web_sys::Document> {
    web_sys::window()?.document()
}

fn stockage_local() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

/// Toutes les pages d'un même site partagent le stockage local : les clés
/// portent donc l'identifiant de l'application.
fn cle(nom: &str) -> String {
    format!("{}/{nom}", textes::APP_ID)
}

/// Là où sont conservées les données : le stockage local du navigateur.
pub struct Depot;

impl Depot {
    pub fn ouvrir() -> Self {
        Depot
    }

    /// Pas de dossier à montrer dans « À propos » : les données sont dans le navigateur.
    pub fn emplacement(&self) -> Option<String> {
        None
    }

    pub fn lire(&self, nom: &str) -> Lecture {
        // Stockage refusé (navigation privée, réglage du navigateur) : on
        // repart des valeurs par défaut, comme à la première visite.
        let Some(stockage) = stockage_local() else {
            return Lecture::Absent;
        };
        match stockage.get_item(&cle(nom)) {
            Ok(Some(texte)) => Lecture::Texte(texte),
            Ok(None) => Lecture::Absent,
            Err(_) => Lecture::Illisible,
        }
    }

    pub fn ecrire(&self, nom: &str, contenu: &str) -> io::Result<()> {
        let stockage =
            stockage_local().ok_or_else(|| io::Error::from(io::ErrorKind::Unsupported))?;
        // L'écriture échoue quand le quota du site est atteint.
        stockage
            .set_item(&cle(nom), contenu)
            .map_err(|_| io::Error::from(io::ErrorKind::StorageFull))
    }

    /// Déplace une donnée illisible sous un autre nom pour ne pas l'écraser (R-46).
    pub fn mettre_de_cote(&self, nom: &str, nouveau_nom: &str) {
        let Some(stockage) = stockage_local() else {
            return;
        };
        if let Ok(Some(texte)) = stockage.get_item(&cle(nom)) {
            let _ = stockage.set_item(&cle(nouveau_nom), &texte);
        }
        let _ = stockage.remove_item(&cle(nom));
    }
}

/// Fait télécharger le fichier par le navigateur, sous le nom proposé.
pub fn exporter(fichier: &Fichier<'_>) -> Export {
    match telecharger(fichier.nom, fichier.contenu) {
        Some(()) => Export::Reussi,
        None => Export::Echec,
    }
}

fn telecharger(nom: &str, contenu: &str) -> Option<()> {
    let document = document()?;
    let morceaux = js_sys::Array::of1(&wasm_bindgen::JsValue::from_str(contenu));
    let options = web_sys::BlobPropertyBag::new();
    options.set_type("text/plain;charset=utf-8");
    let blob = web_sys::Blob::new_with_str_sequence_and_options(&morceaux, &options).ok()?;
    // L'adresse temporaire n'est pas révoquée : le navigateur peut encore en
    // avoir besoin après le clic, et elle disparaît avec la page.
    let adresse = web_sys::Url::create_object_url_with_blob(&blob).ok()?;
    let lien = document
        .create_element("a")
        .ok()?
        .dyn_into::<web_sys::HtmlAnchorElement>()
        .ok()?;
    lien.set_href(&adresse);
    lien.set_download(nom);
    // Certains navigateurs ignorent le clic sur un lien hors de la page.
    let corps = document.body()?;
    corps.append_child(&lien).ok()?;
    lien.click();
    lien.remove();
    Some(())
}

/// Adresse du manuel d'utilisation, dans la langue en cours : une page du
/// site, à côté de celle de l'application, gardée hors ligne avec elle
/// (`app/sw.js`).
pub fn adresse_du_manuel(_depot: &Depot) -> String {
    textes::FICHIER_DU_MANUEL.traduit().to_string()
}

/// Langue du navigateur, si c'est le français ou l'anglais : elle sert au
/// premier lancement seulement (section 24). Les langues préférées sont
/// examinées dans l'ordre.
pub fn langue_de_l_appareil() -> Option<Langue> {
    let navigateur = web_sys::window()?.navigator();
    navigateur
        .languages()
        .iter()
        .filter_map(|langue| langue.as_string())
        .chain(navigateur.language())
        .find_map(|langue| Langue::d_apres(&langue))
}

/// Écrit le titre de la page et sa langue, pour le navigateur et les lecteurs d'écran.
pub fn regler_la_page(langue: Langue, titre: &str) {
    let Some(document) = document() else {
        return;
    };
    document.set_title(titre);
    if let Some(racine) = document.document_element() {
        let code = match langue {
            Langue::Francais => "fr-CA",
            Langue::Anglais => "en-CA",
        };
        let _ = racine.set_attribute("lang", code);
    }
}

/// Le navigateur permet-il le plein écran? (Pas sur tous les téléphones.)
pub fn plein_ecran_possible() -> bool {
    document().is_some_and(|d| d.fullscreen_enabled())
}

/// Entre en plein écran ou en sort. Le navigateur exige un geste de
/// l'utilisateur : le plein écran ne peut donc pas être une préférence
/// rétablie au démarrage, comme sur le bureau.
pub fn basculer_plein_ecran() {
    let Some(document) = document() else {
        return;
    };
    if document.fullscreen_element().is_some() {
        document.exit_fullscreen();
        return;
    }
    let Some(racine) = document.document_element() else {
        return;
    };
    appeler_en_silence(&racine, "requestFullscreen");
}

/// Appelle une méthode du navigateur qui rend une promesse, et attend cette
/// promesse pour qu'un refus reste silencieux au lieu de s'inscrire comme une
/// erreur dans la console.
fn appeler_en_silence(objet: &wasm_bindgen::JsValue, methode: &str) {
    let promesse = js_sys::Reflect::get(objet, &methode.into())
        .ok()
        .and_then(|fonction| fonction.dyn_into::<js_sys::Function>().ok())
        .and_then(|fonction| fonction.call0(objet).ok())
        .and_then(|retour| retour.dyn_into::<js_sys::Promise>().ok());
    if let Some(promesse) = promesse {
        wasm_bindgen_futures::spawn_local(async move {
            let _ = wasm_bindgen_futures::JsFuture::from(promesse).await;
        });
    }
}

/// Nom de la variable où la page range l'invitation à installer que lui
/// envoient Chrome et Edge (voir le script de `app/index.html`).
const INVITATION: &str = "invitationInstallation";

fn invitation() -> Option<wasm_bindgen::JsValue> {
    let fenetre = web_sys::window()?;
    let valeur = js_sys::Reflect::get(&fenetre, &INVITATION.into()).ok()?;
    (!valeur.is_null() && !valeur.is_undefined()).then_some(valeur)
}

/// Ce que le menu peut offrir pour installer l'application sur l'appareil.
pub fn installation() -> Installation {
    let Some(fenetre) = web_sys::window() else {
        return Installation::Aucune;
    };
    // Déjà ouverte comme une application installée : il n'y a rien à offrir.
    let autonome = fenetre
        .match_media("(display-mode: standalone)")
        .ok()
        .flatten()
        .is_some_and(|media| media.matches());
    // Safari sur iPhone et iPad l'indique à sa façon.
    let autonome_ios = js_sys::Reflect::get(&fenetre.navigator(), &"standalone".into())
        .is_ok_and(|valeur| valeur.is_truthy());
    if autonome || autonome_ios {
        Installation::Aucune
    } else if invitation().is_some() {
        Installation::Proposee
    } else {
        Installation::Manuelle
    }
}

/// Montre l'invitation du navigateur à installer l'application.
pub fn installer() {
    let Some(invitation) = invitation() else {
        return;
    };
    appeler_en_silence(&invitation, "prompt");
    // Une invitation ne sert qu'une fois.
    if let Some(fenetre) = web_sys::window() {
        let _ = js_sys::Reflect::set(&fenetre, &INVITATION.into(), &wasm_bindgen::JsValue::NULL);
    }
}

/// `F1` ouvre l'aide de la calculatrice, pas celle du navigateur.
fn reserver_la_touche_d_aide(document: &web_sys::Document) {
    let ecouteur =
        Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(|evenement: web_sys::KeyboardEvent| {
            if evenement.key() == "F1" {
                evenement.prevent_default();
            }
        });
    let _ = document.add_event_listener_with_callback("keydown", ecouteur.as_ref().unchecked_ref());
    // L'écouteur vit aussi longtemps que la page.
    ecouteur.forget();
}

/// Lance l'application dans le canevas de la page.
pub fn demarrer(creer: eframe::AppCreator<'static>) {
    wasm_bindgen_futures::spawn_local(async move {
        let Some(document) = document() else {
            return;
        };
        document.set_title(textes::NOM_APPLICATION.traduit());
        let canevas = document
            .get_element_by_id(ID_CANEVAS)
            .and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok());
        let pret = match canevas {
            Some(canevas) => {
                let demarrage = eframe::WebRunner::new()
                    .start(canevas.clone(), eframe::WebOptions::default(), creer)
                    .await;
                if demarrage.is_ok() {
                    reserver_la_touche_d_aide(&document);
                    // Le clavier d'ordinateur répond sans qu'il faille d'abord cliquer.
                    let _ = canevas.focus();
                }
                demarrage.is_ok()
            }
            None => false,
        };
        if let Some(chargement) = document.get_element_by_id(ID_CHARGEMENT) {
            chargement.remove();
        }
        if !pret && let Some(echec) = document.get_element_by_id(ID_ECHEC) {
            let _ = echec.remove_attribute("hidden");
        }
    });
}
