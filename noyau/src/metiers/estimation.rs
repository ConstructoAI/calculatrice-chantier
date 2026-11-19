//! Estimation et soumission (section 22.3).
//!
//! La méthode est celle du « coût majoré » : au coût de base des travaux
//! s'ajoutent l'administration, les imprévus et le profit, tous trois calculés
//! sur ce coût de base; les taxes s'appliquent ensuite au prix avant taxes.
//! Chaque montant est arrondi au cent, pour que les lignes affichées
//! s'additionnent exactement.

use super::{Calcul, Calculateur, Champ, Lues, Metier, Quantite, Resultat};

const ADMINISTRATION: Champ = Champ::nombre("Administration", "%")
    .defaut("3")
    .nul_permis();
const IMPREVUS: Champ = Champ::nombre("Imprévus (contingences)", "%")
    .defaut("12")
    .nul_permis();
const PROFIT: Champ = Champ::nombre("Profit", "%").defaut("15").nul_permis();
/// Taxe sur les produits et services, et taxe de vente du Québec : les deux
/// se calculent sur le montant avant taxes.
const TPS: Champ = Champ::nombre("TPS", "%").defaut("5").nul_permis();
const TVQ: Champ = Champ::nombre("TVQ", "%").defaut("9,975").nul_permis();

const SENS_DES_TAXES: &[&str] = &["avant taxes", "taxes comprises"];

const NOTE_DES_MAJORATIONS: &str = "Les trois pourcentages s'appliquent au coût de base; les taxes, au prix avant taxes. Repères pour l'administration et les imprévus : 3 % et 12 % en résidentiel neuf, 3 % et 15 % en rénovation résidentielle, 3 % et 10 % en commercial neuf.";

pub const CALCULATEURS: &[Calculateur] = &[
    Calculateur {
        id: "soumission",
        metier: Metier::Estimation,
        nom: "Prix d'une soumission",
        resume: "Du coût des travaux au prix à soumettre : administration, imprévus, profit et taxes.",
        champs: &[
            Champ::nombre("Coût de base des travaux", "$"),
            ADMINISTRATION,
            IMPREVUS,
            PROFIT,
            TPS,
            TVQ,
        ],
        note: NOTE_DES_MAJORATIONS,
        calcul: soumission,
    },
    Calculateur {
        id: "budget-au-pied-carre",
        metier: Metier::Estimation,
        nom: "Budget au pied carré",
        resume: "L'ordre de grandeur du prix d'un projet, d'après sa superficie et un coût au pied carré.",
        champs: &[
            Champ::nombre("Superficie habitable", "pi²"),
            Champ::nombre("Coût de base", "$/pi²"),
            Champ::nombre("Superficie du garage", "pi²").facultatif(),
            Champ::nombre("Coût de base du garage", "$/pi²").facultatif(),
            ADMINISTRATION,
            IMPREVUS,
            PROFIT,
            TPS,
            TVQ,
        ],
        note: "Estimation budgétaire seulement : le coût au pied carré dépend de la région, de la qualité des finis et de la complexité du projet.",
        calcul: budget_au_pied_carre,
    },
    Calculateur {
        id: "main-d-oeuvre",
        metier: Metier::Estimation,
        nom: "Coût de la main-d'œuvre",
        resume: "Le coût d'une équipe, d'après ses heures, son taux horaire et les charges de l'employeur.",
        champs: &[
            Champ::entier("Nombre d'ouvriers", "").defaut("1"),
            Champ::nombre("Heures par ouvrier", "h"),
            Champ::nombre("Taux horaire", "$/h"),
            Champ::nombre("Charges et avantages sociaux", "%")
                .defaut("0")
                .nul_permis(),
        ],
        note: "Les charges regroupent les vacances, les avantages sociaux et les cotisations de l'employeur; elles s'ajoutent au taux horaire.",
        calcul: main_d_oeuvre,
    },
    Calculateur {
        id: "taxes",
        metier: Metier::Estimation,
        nom: "Taxes du Québec",
        resume: "La TPS et la TVQ d'un montant, ou le montant avant taxes d'un total.",
        champs: &[
            Champ::nombre("Montant", "$"),
            Champ::choix("Ce montant est", SENS_DES_TAXES),
            TPS,
            TVQ,
        ],
        note: "Les deux taxes se calculent sur le montant avant taxes. Changez les taux s'ils ont changé.",
        calcul: taxes,
    },
    Calculateur {
        id: "majoration-et-marge",
        metier: Metier::Estimation,
        nom: "Majoration et marge",
        resume: "Le prix de vente d'après le coût et la majoration, et la marge qui en résulte.",
        champs: &[
            Champ::nombre("Coût", "$"),
            Champ::nombre("Majoration sur le coût", "%").nul_permis(),
        ],
        note: "La majoration se mesure sur le coût, la marge sur le prix de vente : 25 % de majoration donnent 20 % de marge.",
        calcul: majoration_et_marge,
    },
];

/// Arrondit un montant au cent.
fn au_cent(montant: f64) -> f64 {
    (montant * 100.0).round() / 100.0
}

fn dollars(libelle: &'static str, montant: f64) -> Resultat {
    Resultat::nouveau(libelle, Quantite::Fixe(montant, 2, "$"))
}

/// Un prix bâti sur un coût de base : majorations, puis taxes.
struct Prix {
    administration: f64,
    imprevus: f64,
    profit: f64,
    avant_taxes: f64,
    tps: f64,
    tvq: f64,
    total: f64,
}

/// `taux` : administration, imprévus, profit, TPS et TVQ, en pourcentage.
fn majorer(base: f64, taux: [f64; 5]) -> Prix {
    let part = |pourcentage: f64, sur: f64| au_cent(sur * pourcentage / 100.0);
    let administration = part(taux[0], base);
    let imprevus = part(taux[1], base);
    let profit = part(taux[2], base);
    let avant_taxes = au_cent(base + administration + imprevus + profit);
    let tps = part(taux[3], avant_taxes);
    let tvq = part(taux[4], avant_taxes);
    Prix {
        administration,
        imprevus,
        profit,
        avant_taxes,
        tps,
        tvq,
        total: au_cent(avant_taxes + tps + tvq),
    }
}

fn soumission(e: &Lues) -> Calcul {
    let prix = majorer(
        au_cent(e.nombre(0)),
        [
            e.nombre(1),
            e.nombre(2),
            e.nombre(3),
            e.nombre(4),
            e.nombre(5),
        ],
    );
    Ok(vec![
        dollars("Administration", prix.administration),
        dollars("Imprévus", prix.imprevus),
        dollars("Profit", prix.profit),
        dollars("Prix avant taxes", prix.avant_taxes),
        dollars("TPS", prix.tps),
        dollars("TVQ", prix.tvq),
        dollars("Prix, taxes comprises", prix.total),
    ])
}

fn budget_au_pied_carre(e: &Lues) -> Calcul {
    if e.presente(2) != e.presente(3) {
        return Err("Pour le garage, donnez la superficie et le coût au pied carré.");
    }
    let superficie = e.nombre(0) + e.nombre(2);
    let base = au_cent(e.nombre(0) * e.nombre(1) + e.nombre(2) * e.nombre(3));
    let prix = majorer(
        base,
        [
            e.nombre(4),
            e.nombre(5),
            e.nombre(6),
            e.nombre(7),
            e.nombre(8),
        ],
    );
    Ok(vec![
        dollars("Coût de base", base),
        dollars(
            "Administration, imprévus et profit",
            au_cent(prix.administration + prix.imprevus + prix.profit),
        ),
        dollars("Prix avant taxes", prix.avant_taxes),
        dollars("TPS et TVQ", au_cent(prix.tps + prix.tvq)),
        dollars("Prix, taxes comprises", prix.total),
        Resultat::nouveau(
            "Prix au pied carré, taxes comprises",
            Quantite::Fixe(au_cent(prix.total / superficie), 2, "$/pi²"),
        ),
    ])
}

fn main_d_oeuvre(e: &Lues) -> Calcul {
    let heures = e.nombre(0) * e.nombre(1);
    let taux = e.nombre(2) * (1.0 + e.nombre(3) / 100.0);
    Ok(vec![
        Resultat::nouveau("Heures totales", Quantite::Fixe(heures, 1, "h")),
        Resultat::nouveau(
            "Coût horaire, charges comprises",
            Quantite::Fixe(au_cent(taux), 2, "$/h"),
        ),
        dollars("Coût de la main-d'œuvre", au_cent(heures * taux)),
    ])
}

fn taxes(e: &Lues) -> Calcul {
    let montant = au_cent(e.nombre(0));
    let (taux_tps, taux_tvq) = (e.nombre(2), e.nombre(3));
    let (avant_taxes, tps, tvq, total) = if e.choix(1) == 1 {
        // Du total vers le montant avant taxes : la TVQ prend le reste, pour
        // que les trois lignes redonnent exactement le total saisi.
        let avant_taxes = au_cent(montant / (1.0 + (taux_tps + taux_tvq) / 100.0));
        let tps = au_cent(avant_taxes * taux_tps / 100.0);
        (
            avant_taxes,
            tps,
            au_cent(montant - avant_taxes - tps),
            montant,
        )
    } else {
        let tps = au_cent(montant * taux_tps / 100.0);
        let tvq = au_cent(montant * taux_tvq / 100.0);
        (montant, tps, tvq, au_cent(montant + tps + tvq))
    };
    Ok(vec![
        dollars("Montant avant taxes", avant_taxes),
        dollars("TPS", tps),
        dollars("TVQ", tvq),
        dollars("Total, taxes comprises", total),
    ])
}

fn majoration_et_marge(e: &Lues) -> Calcul {
    let cout = au_cent(e.nombre(0));
    let prix = au_cent(cout * (1.0 + e.nombre(1) / 100.0));
    let profit = au_cent(prix - cout);
    Ok(vec![
        dollars("Prix de vente", prix),
        dollars("Profit", profit),
        Resultat::nouveau(
            "Marge sur le prix de vente",
            Quantite::Fixe(profit / prix * 100.0, 2, "%"),
        ),
    ])
}
