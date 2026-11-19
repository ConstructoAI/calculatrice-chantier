// Service de la calculatrice : il garde une copie du site dans l'appareil pour
// que la calculatrice s'ouvre hors ligne, sur un chantier sans réseau.
//
// - La page est demandée au réseau d'abord (pour recevoir les mises à jour),
//   et prise dans la copie locale si le réseau ne répond pas.
// - Le programme (fichiers .js et .wasm, dont le nom change à chaque version)
//   et les fichiers fixes sont pris dans la copie locale d'abord.
//
// Ce fichier ne contient aucun numéro de version : la liste des fichiers du
// programme est lue dans la page elle-même.

const COPIE = "calculatrice-chantier";
const PAGE = "./";
const FICHIERS_FIXES = [
    "manifest.webmanifest",
    "manuel.html",
    "icones/icone-192.png",
    "icones/icone-512.png",
    "icones/icone-masquable-512.png",
    "icones/icone-apple-180.png",
];
// Les fichiers du programme, tels que la page les nomme.
const MOTIF_PROGRAMME = /calculatrice-chantier-[0-9a-f]+(?:_bg\.wasm|\.js)/g;
// La même forme, pour reconnaître une adresse.
const EST_DU_PROGRAMME = /calculatrice-chantier-[0-9a-f]+(?:_bg\.wasm|\.js)$/;
// Délai accordé au réseau avant de servir la copie locale de la page.
const DELAI_RESEAU_MS = 4000;

function fichiersDuProgramme(html) {
    return [...new Set(html.match(MOTIF_PROGRAMME) || [])];
}

// Range la page dans la copie locale et retourne la liste des fichiers du programme.
async function rangerLaPage(copie, reponse) {
    const html = await reponse.clone().text();
    await copie.put(PAGE, reponse);
    return fichiersDuProgramme(html);
}

// Télécharge ce qui manque dans la copie locale.
async function completer(copie, fichiers) {
    await Promise.all(
        fichiers.map(async (fichier) => {
            if (await copie.match(fichier)) {
                return;
            }
            const reponse = await fetch(fichier);
            if (reponse.ok) {
                await copie.put(fichier, reponse);
            }
        })
    );
}

// Retire les anciennes versions du programme, une fois la nouvelle au complet.
async function faireLeMenage(copie, programme) {
    for (const fichier of programme) {
        if (!(await copie.match(fichier))) {
            return;
        }
    }
    for (const requete of await copie.keys()) {
        const nom = new URL(requete.url).pathname.split("/").pop();
        if (EST_DU_PROGRAMME.test(nom) && !programme.includes(nom)) {
            await copie.delete(requete);
        }
    }
}

self.addEventListener("install", (evenement) => {
    evenement.waitUntil(
        (async () => {
            const copie = await caches.open(COPIE);
            const reponse = await fetch(PAGE, { cache: "no-cache" });
            if (!reponse.ok) {
                throw new Error("page indisponible");
            }
            const programme = await rangerLaPage(copie, reponse);
            await completer(copie, [...programme, ...FICHIERS_FIXES]);
            await faireLeMenage(copie, programme);
            await self.skipWaiting();
        })()
    );
});

self.addEventListener("activate", (evenement) => {
    evenement.waitUntil(self.clients.claim());
});

function reseauAvecDelai(requete) {
    return new Promise((resoudre, rejeter) => {
        const minuterie = setTimeout(() => rejeter(new Error("réseau trop lent")), DELAI_RESEAU_MS);
        fetch(requete, { cache: "no-cache" }).then(
            (reponse) => {
                clearTimeout(minuterie);
                resoudre(reponse);
            },
            (erreur) => {
                clearTimeout(minuterie);
                rejeter(erreur);
            }
        );
    });
}

// La page : le réseau d'abord, la copie locale sinon.
async function servirLaPage(evenement) {
    const copie = await caches.open(COPIE);
    try {
        const reponse = await reseauAvecDelai(PAGE);
        if (reponse.ok) {
            const pourLaCopie = reponse.clone();
            evenement.waitUntil(
                (async () => {
                    const programme = await rangerLaPage(copie, pourLaCopie);
                    // Le programme est mis en copie au moment où la page le charge
                    // (voir `servirUnFichier`); on attend avant de retirer l'ancien.
                    await new Promise((suite) => setTimeout(suite, 20000));
                    await faireLeMenage(copie, programme);
                })()
            );
            return reponse;
        }
    } catch (_) {
        // Hors ligne ou réseau trop lent : on sert la copie locale.
    }
    return (await copie.match(PAGE)) || Response.error();
}

async function chercherEtGarder(copie, requete) {
    const reponse = await fetch(requete);
    if (reponse.ok) {
        await copie.put(requete, reponse.clone());
    }
    return reponse;
}

// Les autres fichiers : la copie locale d'abord, le réseau sinon.
async function servirUnFichier(evenement) {
    const requete = evenement.request;
    const copie = await caches.open(COPIE);
    const garde = await copie.match(requete);
    if (!garde) {
        return chercherEtGarder(copie, requete);
    }
    // Un fichier du programme ne change jamais (son nom porte son empreinte).
    // Les fichiers fixes (icônes, manifeste) sont servis tout de suite, puis
    // rafraîchis en arrière-plan pour la prochaine fois.
    if (!EST_DU_PROGRAMME.test(new URL(requete.url).pathname)) {
        evenement.waitUntil(chercherEtGarder(copie, requete).catch(() => {}));
    }
    return garde;
}

// La requête vise-t-elle la page de l'application (la racine du site, ou son
// `index.html`)?
function estLaPage(requete) {
    const chemin = new URL(requete.url).pathname;
    const racine = new URL(PAGE, self.location.href).pathname;
    return chemin === racine || chemin === racine + "index.html";
}

self.addEventListener("fetch", (evenement) => {
    const requete = evenement.request;
    if (requete.method !== "GET" || new URL(requete.url).origin !== self.location.origin) {
        return;
    }
    // Seule la page de l'application suit la règle « le réseau d'abord ». Les
    // autres pages du site — le manuel d'utilisation — sont des fichiers comme
    // les autres : sans cette distinction, ouvrir le manuel rouvrirait la
    // calculatrice.
    if (requete.mode === "navigate" && estLaPage(requete)) {
        evenement.respondWith(servirLaPage(evenement));
    } else {
        evenement.respondWith(servirUnFichier(evenement));
    }
});
