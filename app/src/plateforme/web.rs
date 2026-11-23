//! Version Web : données dans le stockage local du navigateur, export par
//! téléchargement, plein écran du navigateur.
//!
//! La page qui héberge l'application est `app/index.html` : les identifiants
//! ci-dessous sont ceux de ses éléments.

use std::cell::RefCell;
use std::io;

use noyau::langue::Langue;
use wasm_bindgen::JsCast as _;
use wasm_bindgen::JsValue;
use wasm_bindgen::closure::Closure;

use super::{Arrivee, ChoixDeFichiers, Export, Fichier, FichierChoisi, Installation, Lecture};
use crate::{pieces, textes};

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

    /// Garde une pièce jointe sous sa clé, dans la base du site (le stockage
    /// local, limité à environ 5 Mo, ne suffirait pas). Un échec arrive par
    /// `arrivees`.
    pub fn ecrire_piece(&self, cle: &str, octets: Vec<u8>) {
        let cle = cle.to_string();
        wasm_bindgen_futures::spawn_local(async move {
            let reussi = match base().await {
                Some(base) => ecrire_dans(&base, &cle, &octets).await,
                None => false,
            };
            if !reussi {
                deposer(Arrivee::EchecEcriture);
            }
        });
    }

    /// Lit une pièce jointe; elle arrive par `arrivees`.
    pub fn lire_piece(&self, cle: &str) {
        let cle = cle.to_string();
        wasm_bindgen_futures::spawn_local(async move {
            let octets = match base().await {
                Some(base) => lire_dans(&base, &cle).await,
                None => None,
            };
            deposer(Arrivee::Lue { cle, octets });
        });
    }

    /// Efface une pièce jointe (rien si elle n'existe pas).
    pub fn supprimer_piece(&self, cle: &str) {
        let cle = cle.to_string();
        wasm_bindgen_futures::spawn_local(async move {
            if let Some(base) = base().await
                && let Ok(transaction) = base
                    .transaction_with_str_and_mode(MAGASIN, web_sys::IdbTransactionMode::Readwrite)
                && let Ok(magasin) = transaction.object_store(MAGASIN)
            {
                let _ = magasin.delete(&JsValue::from_str(&cle));
                let _ = terminer(&transaction).await;
            }
        });
    }

    /// Sans objet dans un navigateur : une pièce jointe s'y enregistre
    /// (téléchargement) au lieu de s'ouvrir dans un programme du poste.
    pub fn ouvrir_piece(&self, _cle: &str) -> bool {
        false
    }
}

/// La base IndexedDB du site, et son magasin des pièces jointes (R-95).
const NOM_DE_LA_BASE: &str = textes::APP_ID;
const VERSION_DE_LA_BASE: u32 = 1;
const MAGASIN: &str = "pieces-jointes";
/// Le champ de fichier caché qui ouvre le choix de fichiers du navigateur.
const ID_CHOIX_DE_FICHIERS: &str = "choix-de-fichiers";

thread_local! {
    /// Ce qui attend d'être relevé par l'application (`arrivees`).
    static ARRIVEES: RefCell<Vec<Arrivee>> = const { RefCell::new(Vec::new()) };
    /// Le contexte d'egui, pour redessiner quand quelque chose arrive.
    static CONTEXTE: RefCell<Option<eframe::egui::Context>> = const { RefCell::new(None) };
    /// La base, ouverte une fois pour toutes.
    static BASE: RefCell<Option<web_sys::IdbDatabase>> = const { RefCell::new(None) };
    /// Les angles du capteur d'inclinaison, lissés (section 22.22) : rien
    /// tant qu'il n'a rien envoyé.
    static INCLINAISON: RefCell<Option<[f64; 2]>> = const { RefCell::new(None) };
    /// Un événement aux angles nuls est arrivé : le navigateur n'a pas de
    /// capteur à lire (Chrome et Edge, sur un ordinateur).
    static INCLINAISON_NULLE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

fn deposer(arrivee: Arrivee) {
    ARRIVEES.with(|a| a.borrow_mut().push(arrivee));
    CONTEXTE.with(|c| {
        if let Some(ctx) = c.borrow().as_ref() {
            ctx.request_repaint();
        }
    });
}

/// Relève ce qui est arrivé depuis la dernière fois : pièces jointes lues,
/// fichiers choisis, échecs d'écriture.
pub fn arrivees() -> Vec<Arrivee> {
    ARRIVEES.with(|a| std::mem::take(&mut *a.borrow_mut()))
}

/// Retient le contexte d'egui : ce qui arrive plus tard fait redessiner.
pub fn retenir_le_contexte(ctx: &eframe::egui::Context) {
    CONTEXTE.with(|c| *c.borrow_mut() = Some(ctx.clone()));
}

/// Attend la réponse d'une demande à la base : son résultat, ou rien en cas d'échec.
async fn attendre(demande: &web_sys::IdbRequest) -> Option<JsValue> {
    let suivie = demande.clone();
    let promesse = js_sys::Promise::new(&mut |resoudre, rejeter| {
        let lue = suivie.clone();
        let succes = Closure::once_into_js(move || {
            let resultat = lue.result().unwrap_or(JsValue::UNDEFINED);
            let _ = resoudre.call1(&JsValue::NULL, &resultat);
        });
        let echec = Closure::once_into_js(move || {
            let _ = rejeter.call0(&JsValue::NULL);
        });
        suivie.set_onsuccess(Some(succes.unchecked_ref()));
        suivie.set_onerror(Some(echec.unchecked_ref()));
    });
    wasm_bindgen_futures::JsFuture::from(promesse).await.ok()
}

/// Attend la fin d'une transaction : vrai si elle est enregistrée (un quota
/// dépassé l'annule).
async fn terminer(transaction: &web_sys::IdbTransaction) -> bool {
    let suivie = transaction.clone();
    let promesse = js_sys::Promise::new(&mut |resoudre, rejeter| {
        let rejeter_aussi = rejeter.clone();
        let fin = Closure::once_into_js(move || {
            let _ = resoudre.call0(&JsValue::NULL);
        });
        let echec = Closure::once_into_js(move || {
            let _ = rejeter.call0(&JsValue::NULL);
        });
        let abandon = Closure::once_into_js(move || {
            let _ = rejeter_aussi.call0(&JsValue::NULL);
        });
        suivie.set_oncomplete(Some(fin.unchecked_ref()));
        suivie.set_onerror(Some(echec.unchecked_ref()));
        suivie.set_onabort(Some(abandon.unchecked_ref()));
    });
    wasm_bindgen_futures::JsFuture::from(promesse).await.is_ok()
}

/// Ouvre la base du site, et crée son magasin à la première ouverture.
async fn base() -> Option<web_sys::IdbDatabase> {
    if let Some(base) = BASE.with(|b| b.borrow().clone()) {
        return Some(base);
    }
    let fabrique = web_sys::window()?.indexed_db().ok()??;
    let demande = fabrique
        .open_with_u32(NOM_DE_LA_BASE, VERSION_DE_LA_BASE)
        .ok()?;
    let creer = Closure::once_into_js(move |evenement: web_sys::Event| {
        let base = evenement
            .target()
            .and_then(|cible| cible.dyn_into::<web_sys::IdbOpenDbRequest>().ok())
            .and_then(|demande| demande.result().ok())
            .and_then(|resultat| resultat.dyn_into::<web_sys::IdbDatabase>().ok());
        if let Some(base) = base {
            let _ = base.create_object_store(MAGASIN);
        }
    });
    demande.set_onupgradeneeded(Some(creer.unchecked_ref()));
    let base = attendre(&demande)
        .await?
        .dyn_into::<web_sys::IdbDatabase>()
        .ok()?;
    BASE.with(|b| *b.borrow_mut() = Some(base.clone()));
    Some(base)
}

async fn ecrire_dans(base: &web_sys::IdbDatabase, cle: &str, octets: &[u8]) -> bool {
    let Ok(transaction) =
        base.transaction_with_str_and_mode(MAGASIN, web_sys::IdbTransactionMode::Readwrite)
    else {
        return false;
    };
    let Ok(magasin) = transaction.object_store(MAGASIN) else {
        return false;
    };
    let tableau = js_sys::Uint8Array::from(octets);
    if magasin
        .put_with_key(&tableau, &JsValue::from_str(cle))
        .is_err()
    {
        return false;
    }
    terminer(&transaction).await
}

async fn lire_dans(base: &web_sys::IdbDatabase, cle: &str) -> Option<Vec<u8>> {
    let transaction = base.transaction_with_str(MAGASIN).ok()?;
    let magasin = transaction.object_store(MAGASIN).ok()?;
    let demande = magasin.get(&JsValue::from_str(cle)).ok()?;
    let valeur = attendre(&demande).await?;
    valeur
        .dyn_into::<js_sys::Uint8Array>()
        .ok()
        .map(|tableau| tableau.to_vec())
}

/// Ouvre le choix de fichiers du navigateur : des photos (sur un téléphone,
/// l'appareil photo ou la galerie), tout fichier, ou des dessins (JSON). Le
/// résultat arrive par `arrivees`.
pub fn choisir_des_fichiers(genre: ChoixDeFichiers) {
    let Some(document) = document() else {
        return;
    };
    // Un champ resté d'un choix annulé est retiré d'abord.
    if let Some(ancien) = document.get_element_by_id(ID_CHOIX_DE_FICHIERS) {
        ancien.remove();
    }
    let Some(champ) = document
        .create_element("input")
        .ok()
        .and_then(|e| e.dyn_into::<web_sys::HtmlInputElement>().ok())
    else {
        return;
    };
    champ.set_type("file");
    champ.set_multiple(true);
    match genre {
        ChoixDeFichiers::Photos => champ.set_accept("image/*"),
        ChoixDeFichiers::Dessins | ChoixDeFichiers::Sauvegarde => {
            champ.set_accept(".json,application/json")
        }
        ChoixDeFichiers::Tous => {}
    }
    champ.set_id(ID_CHOIX_DE_FICHIERS);
    champ.set_hidden(true);
    let lu = champ.clone();
    let choisis = Closure::once_into_js(move || {
        let liste = lu.files();
        lu.remove();
        let fichiers: Vec<web_sys::File> = liste
            .map(|l| (0..l.length()).filter_map(|i| l.get(i)).collect())
            .unwrap_or_default();
        if fichiers.is_empty() {
            return;
        }
        wasm_bindgen_futures::spawn_local(async move {
            let mut recus = Vec::with_capacity(fichiers.len());
            for fichier in fichiers {
                let octets = if fichier.size() > pieces::TAILLE_MAX_LECTURE as f64 {
                    None
                } else {
                    wasm_bindgen_futures::JsFuture::from(fichier.array_buffer())
                        .await
                        .ok()
                        .map(|tampon| js_sys::Uint8Array::new(&tampon).to_vec())
                };
                recus.push(FichierChoisi {
                    nom: fichier.name(),
                    type_mime: fichier.type_(),
                    octets,
                });
            }
            deposer(Arrivee::Choisis(recus));
        });
    });
    let _ = champ.add_event_listener_with_callback("change", choisis.unchecked_ref());
    let retire = champ.clone();
    let annule = Closure::once_into_js(move || retire.remove());
    let _ = champ.add_event_listener_with_callback("cancel", annule.unchecked_ref());
    if let Some(corps) = document.body() {
        let _ = corps.append_child(&champ);
    }
    champ.click();
}

/// Fait télécharger le fichier par le navigateur, sous le nom proposé.
pub fn exporter(fichier: &Fichier<'_>) -> Export {
    match telecharger(fichier.nom, fichier.contenu, fichier.type_mime) {
        Some(()) => Export::Reussi,
        None => Export::Echec,
    }
}

fn telecharger(nom: &str, contenu: &[u8], type_mime: &str) -> Option<()> {
    let document = document()?;
    let tableau = js_sys::Uint8Array::from(contenu);
    let morceaux = js_sys::Array::of1(&tableau);
    let options = web_sys::BlobPropertyBag::new();
    options.set_type(type_mime);
    let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(&morceaux, &options).ok()?;
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

/// Écoute le capteur d'inclinaison du téléphone (section 22.22), s'il en a
/// un : ses angles, lissés, attendent `inclinaison_lue`. Sur un appareil sans
/// capteur, rien n'arrive, et le niveau reste caché.
pub fn ecouter_l_inclinaison() {
    let Some(fenetre) = web_sys::window() else {
        return;
    };
    let rappel = Closure::<dyn FnMut(web_sys::DeviceOrientationEvent)>::new(
        |evenement: web_sys::DeviceOrientationEvent| {
            let (Some(beta), Some(gamma)) = (evenement.beta(), evenement.gamma()) else {
                INCLINAISON_NULLE.with(|nulle| nulle.set(true));
                return;
            };
            INCLINAISON.with(|inclinaison| {
                let mut inclinaison = inclinaison.borrow_mut();
                *inclinaison = Some(match *inclinaison {
                    // Un lissage léger : la lecture ne tremble pas.
                    Some([b, g]) => [b + 0.25 * (beta - b), g + 0.25 * (gamma - g)],
                    None => [beta, gamma],
                });
            });
        },
    );
    let _ = fenetre
        .add_event_listener_with_callback("deviceorientation", rappel.as_ref().unchecked_ref());
    rappel.forget();
}

/// Les derniers angles du capteur d'inclinaison : (avant-arrière,
/// gauche-droite), en degrés.
pub fn inclinaison_lue() -> Option<[f64; 2]> {
    INCLINAISON.with(|inclinaison| *inclinaison.borrow())
}

/// La fonction `requestPermission` du capteur, là où le navigateur en a une
/// (Safari, sur iPhone et iPad).
fn demande_de_permission() -> Option<js_sys::Function> {
    let fenetre = web_sys::window()?;
    let classe =
        js_sys::Reflect::get(&fenetre, &JsValue::from_str("DeviceOrientationEvent")).ok()?;
    js_sys::Reflect::get(&classe, &JsValue::from_str("requestPermission"))
        .ok()?
        .dyn_into::<js_sys::Function>()
        .ok()
}

/// Vrai si le capteur attend la permission de l'utilisateur (iPhone, iPad) :
/// rien n'est arrivé, pas même un événement aux angles nuls, et le navigateur
/// sait demander la permission.
pub fn inclinaison_a_permettre() -> bool {
    inclinaison_lue().is_none()
        && !INCLINAISON_NULLE.with(std::cell::Cell::get)
        && demande_de_permission().is_some()
}

/// Demande la permission de lire le capteur (au toucher d'un bouton).
pub fn demander_l_inclinaison() {
    let Some(demande) = demande_de_permission() else {
        return;
    };
    let Ok(classe) = web_sys::window()
        .map(|f| js_sys::Reflect::get(&f, &JsValue::from_str("DeviceOrientationEvent")))
        .transpose()
    else {
        return;
    };
    let _ = classe.map(|classe| demande.call0(&classe));
}

/// Vrai si le niveau a de quoi lire : un capteur qui a parlé, ou qui attend
/// la permission de l'utilisateur.
pub fn inclinaison_disponible() -> bool {
    inclinaison_lue().is_some() || inclinaison_a_permettre()
}

/// L'écran qu'un raccourci de l'application installée demande d'ouvrir
/// (section 22.16) : le paramètre `ecran` de l'adresse de la page.
pub fn ecran_demande() -> Option<String> {
    let adresse = document()?.url().ok()?;
    let requete = adresse.split_once('?')?.1.split('#').next()?;
    requete
        .split('&')
        .find_map(|paire| paire.strip_prefix("ecran="))
        .map(str::to_string)
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
