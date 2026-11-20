//! Estimation et soumission (section 22.3).
//!
//! La méthode est celle du « coût majoré » : au coût de base des travaux
//! s'ajoutent l'administration, les imprévus et le profit, tous trois calculés
//! sur ce coût de base; les taxes s'appliquent ensuite au prix avant taxes.
//! Chaque montant est arrondi au cent, pour que les lignes affichées
//! s'additionnent exactement.

use super::{Calcul, Calculateur, Champ, Lues, Metier, Quantite, Resultat};
use crate::langue::{Texte, t, tel};

const ADMINISTRATION: Champ = Champ::nombre(tel("Administration"), tel("%"))
    .court(tel("Admin."))
    .defaut("3")
    .nul_permis();
const IMPREVUS: Champ = Champ::nombre(t("Imprévus (contingences)", "Contingency"), tel("%"))
    .court(t("Imprévus", "Contingency"))
    .defaut("12")
    .nul_permis();
const PROFIT: Champ = Champ::nombre(tel("Profit"), tel("%"))
    .defaut("15")
    .nul_permis();
/// Taxe sur les produits et services, et taxe de vente du Québec : les deux
/// se calculent sur le montant avant taxes.
const TPS: Champ = Champ::nombre(t("TPS", "GST"), tel("%"))
    .defaut("5")
    .nul_permis();
const TVQ: Champ = Champ::nombre(t("TVQ", "QST"), tel("%"))
    .defaut("9,975")
    .nul_permis();

const SENS_DES_TAXES: &[Texte] = &[
    t("avant taxes", "before taxes"),
    t("taxes comprises", "taxes included"),
];

const NOTE_DES_MAJORATIONS: Texte = t(
    "Les trois pourcentages s'appliquent au coût de base; les taxes, au prix avant taxes. Repères pour l'administration et les imprévus : 3 % et 12 % en résidentiel neuf, 3 % et 15 % en rénovation résidentielle, 3 % et 10 % en commercial neuf.",
    "The three percentages apply to the base cost; taxes apply to the price before taxes. Typical administration and contingency rates: 3% and 12% for new residential work, 3% and 15% for residential renovation, 3% and 10% for new commercial work.",
);

pub const CALCULATEURS: &[Calculateur] = &[
    Calculateur {
        id: "soumission",
        metier: Metier::Estimation,
        nom: t("Prix d'une soumission", "Bid price"),
        resume: t(
            "Du coût des travaux au prix à soumettre : administration, imprévus, profit et taxes.",
            "From the cost of the work to the price to submit: administration, contingency, profit and taxes.",
        ),
        champs: &[
            Champ::nombre(
                t("Coût de base des travaux", "Base cost of the work"),
                tel("$"),
            )
            .court(t("Coût\nde base", "Base\ncost")),
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
        nom: t("Budget au pied carré", "Budget per square foot"),
        resume: t(
            "L'ordre de grandeur du prix d'un projet, d'après sa superficie et un coût au pied carré.",
            "A ballpark price for a project, from its floor area and a cost per square foot.",
        ),
        champs: &[
            Champ::nombre(t("Superficie habitable", "Living area"), t("pi²", "ft²"))
                .court(t("Superficie", "Area")),
            Champ::nombre(t("Coût de base", "Base cost"), t("$/pi²", "$/ft²"))
                .court(t("Coût\nde base", "Base\ncost")),
            Champ::nombre(t("Superficie du garage", "Garage area"), t("pi²", "ft²"))
                .court(tel("Garage"))
                .facultatif(),
            Champ::nombre(
                t("Coût de base du garage", "Garage base cost"),
                t("$/pi²", "$/ft²"),
            )
            .court(t("Coût\ngarage", "Garage\ncost"))
            .facultatif(),
            ADMINISTRATION,
            IMPREVUS,
            PROFIT,
            TPS,
            TVQ,
        ],
        note: t(
            "Estimation budgétaire seulement : le coût au pied carré dépend de la région, de la qualité des finis et de la complexité du projet.",
            "Budget estimate only: the cost per square foot depends on the region, the quality of the finishes and the complexity of the project.",
        ),
        calcul: budget_au_pied_carre,
    },
    Calculateur {
        id: "main-d-oeuvre",
        metier: Metier::Estimation,
        nom: t("Coût de la main-d'œuvre", "Labour cost"),
        resume: t(
            "Le coût d'une équipe, d'après ses heures, son taux horaire et les charges de l'employeur.",
            "The cost of a crew, from its hours, its hourly rate and the employer's payroll burden.",
        ),
        champs: &[
            Champ::entier(t("Nombre d'ouvriers", "Number of workers"), Texte::VIDE)
                .court(t("Ouvriers", "Workers"))
                .defaut("1"),
            Champ::nombre(t("Heures par ouvrier", "Hours per worker"), tel("h"))
                .court(t("Heures", "Hours")),
            Champ::nombre(t("Taux horaire", "Hourly rate"), tel("$/h"))
                .court(t("Taux\nhoraire", "Hourly\nrate")),
            Champ::nombre(
                t(
                    "Charges et avantages sociaux",
                    "Payroll burden and benefits",
                ),
                tel("%"),
            )
            .court(t("Charges", "Burden"))
            .defaut("0")
            .nul_permis(),
        ],
        note: t(
            "Les charges regroupent les vacances, les avantages sociaux et les cotisations de l'employeur; elles s'ajoutent au taux horaire.",
            "The burden covers vacation pay, benefits and employer contributions; it is added to the hourly rate.",
        ),
        calcul: main_d_oeuvre,
    },
    Calculateur {
        id: "taxes",
        metier: Metier::Estimation,
        nom: t("Taxes du Québec", "Québec taxes"),
        resume: t(
            "La TPS et la TVQ d'un montant, ou le montant avant taxes d'un total.",
            "The GST and QST on an amount, or the amount before taxes from a total.",
        ),
        champs: &[
            Champ::nombre(t("Montant", "Amount"), tel("$")),
            Champ::choix(t("Ce montant est", "This amount is"), SENS_DES_TAXES)
                .court(t("Montant\ndonné", "Amount\ngiven")),
            TPS,
            TVQ,
        ],
        note: t(
            "Les deux taxes se calculent sur le montant avant taxes. Changez les taux s'ils ont changé.",
            "Both taxes are calculated on the amount before taxes. Change the rates if they have changed.",
        ),
        calcul: taxes,
    },
    Calculateur {
        id: "majoration-et-marge",
        metier: Metier::Estimation,
        nom: t("Majoration et marge", "Markup and margin"),
        resume: t(
            "Le prix de vente d'après le coût et la majoration, et la marge qui en résulte.",
            "The selling price from the cost and the markup, and the resulting margin.",
        ),
        champs: &[
            Champ::nombre(t("Coût", "Cost"), tel("$")),
            Champ::nombre(t("Majoration sur le coût", "Markup on cost"), tel("%"))
                .court(t("Majoration", "Markup"))
                .nul_permis(),
        ],
        note: t(
            "La majoration se mesure sur le coût, la marge sur le prix de vente : 25 % de majoration donnent 20 % de marge.",
            "Markup is measured on the cost, margin on the selling price: a 25% markup gives a 20% margin.",
        ),
        calcul: majoration_et_marge,
    },
    Calculateur {
        id: "facturation-progressive",
        metier: Metier::Estimation,
        nom: t("Facture d'étape", "Progress invoice"),
        resume: t(
            "Le montant d'une facture d'étape, d'après l'avancement des travaux et la retenue.",
            "The amount of a progress invoice, from the work completed and the holdback.",
        ),
        champs: &[
            Champ::nombre(
                t(
                    "Montant du contrat, avant taxes",
                    "Contract amount, before taxes",
                ),
                tel("$"),
            )
            .court(t("Contrat", "Contract")),
            Champ::nombre(t("Avancement des travaux", "Work completed"), tel("%"))
                .court(t("Avancement", "Completed")),
            Champ::nombre(t("Retenue", "Holdback"), tel("%"))
                .defaut("10")
                .nul_permis(),
            Champ::nombre(
                t(
                    "Déjà facturé, avant taxes",
                    "Previously invoiced, before taxes",
                ),
                tel("$"),
            )
            .defaut("0")
            .nul_permis()
            .court(t("Déjà\nfacturé", "Already\ninvoiced")),
            TPS,
            TVQ,
        ],
        note: t(
            "Le montant déjà facturé est celui des factures précédentes, avant taxes et retenue déduite.",
            "The amount previously invoiced is the total of the earlier invoices, before taxes and net of holdback.",
        ),
        calcul: facturation_progressive,
    },
    Calculateur {
        id: "duree-des-travaux",
        metier: Metier::Estimation,
        nom: t("Durée des travaux", "Job duration"),
        resume: t(
            "Les heures et les jours de travail, d'après la quantité à réaliser et le rendement de l'équipe.",
            "The hours and days of work, from the quantity to complete and the crew's production rate.",
        ),
        champs: &[
            Champ::nombre(
                t("Quantité à réaliser", "Quantity to complete"),
                Texte::VIDE,
            )
            .court(t("Quantité", "Quantity")),
            Champ::nombre(
                t(
                    "Rendement d'un ouvrier, à l'heure",
                    "Production per worker, per hour",
                ),
                Texte::VIDE,
            )
            .court(t("Rendement", "Production")),
            Champ::entier(t("Nombre d'ouvriers", "Number of workers"), Texte::VIDE)
                .defaut("1")
                .court(t("Ouvriers", "Workers")),
            Champ::nombre(t("Heures par jour", "Hours per day"), tel("h"))
                .defaut("8")
                .court(t("Heures\npar jour", "Hours\nper day")),
        ],
        note: t(
            "La quantité et le rendement s'expriment dans la même unité : des pieds carrés et des pieds carrés à l'heure, par exemple.",
            "The quantity and the production rate use the same unit: square feet and square feet per hour, for example.",
        ),
        calcul: duree_des_travaux,
    },
];

/// Arrondit un montant au cent.
fn au_cent(montant: f64) -> f64 {
    (montant * 100.0).round() / 100.0
}

fn dollars(libelle: Texte, montant: f64) -> Resultat {
    Resultat::nouveau(libelle, Quantite::Fixe(montant, 2, tel("$")))
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
        dollars(tel("Administration"), prix.administration),
        dollars(t("Imprévus", "Contingency"), prix.imprevus),
        dollars(tel("Profit"), prix.profit),
        dollars(
            t("Prix avant taxes", "Price before taxes"),
            prix.avant_taxes,
        ),
        dollars(t("TPS", "GST"), prix.tps),
        dollars(t("TVQ", "QST"), prix.tvq),
        dollars(
            t("Prix, taxes comprises", "Price, taxes included"),
            prix.total,
        ),
    ])
}

fn budget_au_pied_carre(e: &Lues) -> Calcul {
    if e.presente(2) != e.presente(3) {
        return Err(t(
            "Pour le garage, donnez la superficie et le coût au pied carré.",
            "For the garage, enter both the area and the cost per square foot.",
        ));
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
        dollars(t("Coût de base", "Base cost"), base),
        dollars(
            t(
                "Administration, imprévus et profit",
                "Administration, contingency and profit",
            ),
            au_cent(prix.administration + prix.imprevus + prix.profit),
        ),
        dollars(
            t("Prix avant taxes", "Price before taxes"),
            prix.avant_taxes,
        ),
        dollars(t("TPS et TVQ", "GST and QST"), au_cent(prix.tps + prix.tvq)),
        dollars(
            t("Prix, taxes comprises", "Price, taxes included"),
            prix.total,
        ),
        Resultat::nouveau(
            t(
                "Prix au pied carré, taxes comprises",
                "Price per square foot, taxes included",
            ),
            Quantite::Fixe(au_cent(prix.total / superficie), 2, t("$/pi²", "$/ft²")),
        ),
    ])
}

fn main_d_oeuvre(e: &Lues) -> Calcul {
    let heures = e.nombre(0) * e.nombre(1);
    let taux = e.nombre(2) * (1.0 + e.nombre(3) / 100.0);
    Ok(vec![
        Resultat::nouveau(
            t("Heures totales", "Total hours"),
            Quantite::Fixe(heures, 1, tel("h")),
        ),
        Resultat::nouveau(
            t(
                "Coût horaire, charges comprises",
                "Hourly cost, burden included",
            ),
            Quantite::Fixe(au_cent(taux), 2, tel("$/h")),
        ),
        dollars(
            t("Coût de la main-d'œuvre", "Labour cost"),
            au_cent(heures * taux),
        ),
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
        dollars(t("Montant avant taxes", "Amount before taxes"), avant_taxes),
        dollars(t("TPS", "GST"), tps),
        dollars(t("TVQ", "QST"), tvq),
        dollars(t("Total, taxes comprises", "Total, taxes included"), total),
    ])
}

fn majoration_et_marge(e: &Lues) -> Calcul {
    let cout = au_cent(e.nombre(0));
    let prix = au_cent(cout * (1.0 + e.nombre(1) / 100.0));
    let profit = au_cent(prix - cout);
    Ok(vec![
        dollars(t("Prix de vente", "Selling price"), prix),
        dollars(tel("Profit"), profit),
        Resultat::nouveau(
            t("Marge sur le prix de vente", "Margin on selling price"),
            Quantite::Fixe(profit / prix * 100.0, 2, tel("%")),
        ),
    ])
}

fn facturation_progressive(e: &Lues) -> Calcul {
    let avancement = e.nombre(1);
    if avancement > 100.0 {
        return Err(t(
            "L'avancement ne dépasse pas 100 %.",
            "Work completed cannot exceed 100%.",
        ));
    }
    if e.nombre(2) >= 100.0 {
        return Err(t(
            "La retenue doit être inférieure à 100 %.",
            "The holdback must be less than 100%.",
        ));
    }
    let executes = au_cent(e.nombre(0) * avancement / 100.0);
    let retenue = au_cent(executes * e.nombre(2) / 100.0);
    let facture = au_cent(executes - retenue - au_cent(e.nombre(3)));
    if facture < 0.0 {
        return Err(t(
            "Le montant déjà facturé dépasse les travaux exécutés, retenue déduite.",
            "The amount already invoiced exceeds the work completed, net of holdback.",
        ));
    }
    let tps = au_cent(facture * e.nombre(4) / 100.0);
    let tvq = au_cent(facture * e.nombre(5) / 100.0);
    Ok(vec![
        dollars(
            t("Travaux exécutés à ce jour", "Work completed to date"),
            executes,
        ),
        dollars(t("Retenue", "Holdback"), retenue),
        dollars(
            t("Cette facture, avant taxes", "This invoice, before taxes"),
            facture,
        ),
        dollars(t("TPS", "GST"), tps),
        dollars(t("TVQ", "QST"), tvq),
        dollars(
            t(
                "Cette facture, taxes comprises",
                "This invoice, taxes included",
            ),
            au_cent(facture + tps + tvq),
        ),
    ])
}

fn duree_des_travaux(e: &Lues) -> Calcul {
    if e.nombre(3) > 24.0 {
        return Err(t("Une journée compte 24 heures.", "A day has 24 hours."));
    }
    let heures_de_travail = e.nombre(0) / e.nombre(1);
    let duree = heures_de_travail / e.nombre(2);
    Ok(vec![
        Resultat::nouveau(
            t("Heures de travail, en tout", "Total labour hours"),
            Quantite::Fixe(heures_de_travail, 1, tel("h")),
        ),
        Resultat::nouveau(
            t("Durée, à cette équipe", "Duration with this crew"),
            Quantite::Fixe(duree, 1, tel("h")),
        ),
        Resultat::nouveau(
            t("Jours de travail", "Working days"),
            Quantite::Fixe(duree / e.nombre(3), 1, t("jours", "days")),
        ),
    ])
}
