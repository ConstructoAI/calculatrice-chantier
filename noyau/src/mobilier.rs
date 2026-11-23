//! Les objets d'une pièce (section 25) : l'escalier, le mobilier et les
//! symboles des corps de métier (point 40), placés sur le plan à l'échelle de
//! ses mesures. Chacun a ses dimensions de départ, sa
//! hauteur pour la 3D et un symbole, décrit dans son rectangle (u le long de
//! sa largeur, v le long de sa profondeur, de 0 à 1).

use serde::{Deserialize, Serialize};

/// Ce qu'est un objet : un meuble, un appareil, ou le symbole d'un corps de
/// métier (section 25.1, point 40), posé sur le plan comme un meuble.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GenreDObjet {
    Escalier,
    Lit,
    #[default]
    Table,
    Sofa,
    Comptoir,
    Toilette,
    Bain,
    Douche,
    Laveuse,
    Refrigerateur,
    Cuisiniere,
    Secheuse,
    // La plomberie.
    ChauffeEau,
    FiltreAEau,
    Adoucisseur,
    Coude90,
    Coude45,
    Te,
    RobinetDArret,
    Clapet,
    CompteurDEau,
    DrainDePlancher,
    ColonneDeChute,
    Lavabo,
    Evier,
    // L'électricité.
    Panneau,
    CompteurElectrique,
    Prise,
    PriseDdft,
    Prise240,
    Interrupteur,
    Luminaire,
    Encastre,
    Detecteur,
    BoiteDeJonction,
    Plinthe,
    Thermostat,
    // La ventilation et le chauffage.
    Fournaise,
    ThermopompeMurale,
    ThermopompeExterieure,
    Echangeur,
    BoucheDeSoufflage,
    RetourDAir,
    Ventilateur,
    Hotte,
    // La cuisine et la salle de bain.
    ArmoireDuBas,
    ArmoireDuHaut,
    Ilot,
    LaveVaisselle,
    Vanite,
}

impl GenreDObjet {
    /// Tous les genres; les boutons les rangent par famille (`Famille::genres`).
    pub const TOUS: [GenreDObjet; 50] = [
        GenreDObjet::Escalier,
        GenreDObjet::Lit,
        GenreDObjet::Table,
        GenreDObjet::Sofa,
        GenreDObjet::Comptoir,
        GenreDObjet::Toilette,
        GenreDObjet::Bain,
        GenreDObjet::Douche,
        GenreDObjet::Laveuse,
        GenreDObjet::Refrigerateur,
        GenreDObjet::Cuisiniere,
        GenreDObjet::Secheuse,
        GenreDObjet::ChauffeEau,
        GenreDObjet::FiltreAEau,
        GenreDObjet::Adoucisseur,
        GenreDObjet::Coude90,
        GenreDObjet::Coude45,
        GenreDObjet::Te,
        GenreDObjet::RobinetDArret,
        GenreDObjet::Clapet,
        GenreDObjet::CompteurDEau,
        GenreDObjet::DrainDePlancher,
        GenreDObjet::ColonneDeChute,
        GenreDObjet::Lavabo,
        GenreDObjet::Evier,
        GenreDObjet::Panneau,
        GenreDObjet::CompteurElectrique,
        GenreDObjet::Prise,
        GenreDObjet::PriseDdft,
        GenreDObjet::Prise240,
        GenreDObjet::Interrupteur,
        GenreDObjet::Luminaire,
        GenreDObjet::Encastre,
        GenreDObjet::Detecteur,
        GenreDObjet::BoiteDeJonction,
        GenreDObjet::Plinthe,
        GenreDObjet::Thermostat,
        GenreDObjet::Fournaise,
        GenreDObjet::ThermopompeMurale,
        GenreDObjet::ThermopompeExterieure,
        GenreDObjet::Echangeur,
        GenreDObjet::BoucheDeSoufflage,
        GenreDObjet::RetourDAir,
        GenreDObjet::Ventilateur,
        GenreDObjet::Hotte,
        GenreDObjet::ArmoireDuBas,
        GenreDObjet::ArmoireDuHaut,
        GenreDObjet::Ilot,
        GenreDObjet::LaveVaisselle,
        GenreDObjet::Vanite,
    ];

    /// Largeur, profondeur et hauteur de départ, en pouces : des tailles
    /// courantes, à ajuster ensuite. L'escalier monte jusqu'au haut des murs.
    /// Un symbole (une prise, un coude) prend la taille qu'il a sur un plan au
    /// 1/4 po = 1 pi.
    pub fn dimensions(self) -> (f64, f64, f64) {
        match self {
            GenreDObjet::Escalier => (36.0, 120.0, 0.0),
            GenreDObjet::Lit => (60.0, 80.0, 24.0),
            GenreDObjet::Table => (36.0, 60.0, 30.0),
            GenreDObjet::Sofa => (84.0, 36.0, 34.0),
            GenreDObjet::Comptoir => (96.0, 25.0, 36.0),
            GenreDObjet::Toilette => (20.0, 28.0, 30.0),
            GenreDObjet::Bain => (30.0, 60.0, 20.0),
            GenreDObjet::Douche => (36.0, 36.0, 80.0),
            GenreDObjet::Laveuse => (27.0, 30.0, 38.0),
            GenreDObjet::Refrigerateur => (30.0, 32.0, 68.0),
            GenreDObjet::Cuisiniere => (30.0, 26.0, 36.0),
            GenreDObjet::Secheuse => (27.0, 30.0, 42.0),
            GenreDObjet::ChauffeEau => (22.0, 22.0, 68.0),
            GenreDObjet::FiltreAEau => (10.0, 10.0, 46.0),
            GenreDObjet::Adoucisseur => (30.0, 18.0, 54.0),
            GenreDObjet::Coude90 | GenreDObjet::Coude45 | GenreDObjet::Te => (8.0, 8.0, 14.0),
            GenreDObjet::RobinetDArret => (8.0, 8.0, 20.0),
            GenreDObjet::Clapet => (8.0, 8.0, 8.0),
            GenreDObjet::CompteurDEau => (8.0, 8.0, 18.6),
            GenreDObjet::DrainDePlancher => (8.0, 8.0, 0.3),
            GenreDObjet::ColonneDeChute => (6.0, 6.0, PLAFOND_DE_REFERENCE),
            GenreDObjet::Lavabo => (20.0, 18.0, 40.0),
            GenreDObjet::Evier => (33.0, 22.0, 48.0),
            GenreDObjet::Panneau => (16.0, 6.0, 72.0),
            GenreDObjet::CompteurElectrique => (12.0, 12.0, 66.0),
            GenreDObjet::Prise => (12.0, 12.0, 18.0),
            GenreDObjet::PriseDdft => (12.0, 12.0, 47.0),
            GenreDObjet::Prise240 => (12.0, 12.0, 20.0),
            GenreDObjet::Interrupteur => (12.0, 12.0, 49.0),
            GenreDObjet::Luminaire => (12.0, 12.0, PLAFOND_DE_REFERENCE),
            GenreDObjet::Encastre
            | GenreDObjet::Detecteur
            | GenreDObjet::BoiteDeJonction
            | GenreDObjet::Ventilateur => (10.0, 10.0, PLAFOND_DE_REFERENCE),
            GenreDObjet::Plinthe => (48.0, 5.0, 10.0),
            GenreDObjet::Thermostat => (8.0, 8.0, 62.0),
            GenreDObjet::Fournaise => (22.0, 28.0, PLAFOND_DE_REFERENCE),
            GenreDObjet::ThermopompeMurale => (32.0, 9.0, 92.0),
            GenreDObjet::ThermopompeExterieure => (32.0, 14.0, 28.0),
            GenreDObjet::Echangeur => (30.0, 18.0, 88.0),
            GenreDObjet::BoucheDeSoufflage => (12.0, 5.0, 0.4),
            GenreDObjet::RetourDAir => (24.0, 4.0, 16.0),
            GenreDObjet::Hotte => (30.0, 20.0, PLAFOND_DE_REFERENCE),
            GenreDObjet::ArmoireDuBas => (30.0, 24.0, 36.0),
            GenreDObjet::ArmoireDuHaut => (30.0, 12.0, 84.0),
            GenreDObjet::Ilot => (72.0, 36.0, 36.0),
            GenreDObjet::LaveVaisselle => (24.0, 24.0, 36.0),
            GenreDObjet::Vanite => (30.0, 21.0, 40.0),
        }
    }

    /// Les parties du meuble en 3D, chacune de sa matière. L'escalier se bâtit
    /// à part, en marches jusqu'au haut des murs.
    pub fn volumes(self) -> Vec<Volume> {
        use Matiere::*;
        match self {
            GenreDObjet::Escalier => Vec::new(),
            // La tête, la base, le matelas, la couverture et deux oreillers.
            GenreDObjet::Lit => vec![
                volume((0.0, 1.0), (0.0, 0.03), 0.0, 40.0, BoisFonce),
                volume((0.0, 1.0), (0.03, 1.0), 0.0, 12.0, BoisFonce),
                volume((0.02, 0.98), (0.03, 0.99), 12.0, 22.0, Literie),
                volume((0.02, 0.98), (0.32, 0.99), 22.0, 23.0, Tissu),
                volume((0.08, 0.46), (0.05, 0.18), 22.0, 26.0, Coussin),
                volume((0.54, 0.92), (0.05, 0.18), 22.0, 26.0, Coussin),
            ],
            // Le plateau, ses quatre pattes, et les chaises de son symbole.
            GenreDObjet::Table => {
                let mut parties = vec![volume((0.0, 1.0), (0.0, 1.0), 28.5, 30.0, Bois)];
                for u in [(0.03, 0.09), (0.91, 0.97)] {
                    for v in [(0.03, 0.07), (0.93, 0.97)] {
                        parties.push(volume(u, v, 0.0, 28.5, Bois));
                    }
                }
                for (u, v, dossier) in [
                    ((0.25, 0.45), (-0.12, 0.0), (0.0, -1.0)),
                    ((0.55, 0.75), (-0.12, 0.0), (0.0, -1.0)),
                    ((0.25, 0.45), (1.0, 1.12), (0.0, 1.0)),
                    ((0.55, 0.75), (1.0, 1.12), (0.0, 1.0)),
                    ((-0.2, 0.0), (0.4, 0.6), (-1.0, 0.0)),
                    ((1.0, 1.2), (0.4, 0.6), (1.0, 0.0)),
                ] {
                    parties.extend(chaise(u, v, dossier));
                }
                parties
            }
            // Le dossier, les bras, l'assise, et les coussins du siège et du dos.
            GenreDObjet::Sofa => vec![
                volume((0.0, 1.0), (0.0, 0.15), 0.0, 34.0, Tissu),
                volume((0.0, 0.12), (0.15, 1.0), 0.0, 25.0, Tissu),
                volume((0.88, 1.0), (0.15, 1.0), 0.0, 25.0, Tissu),
                volume((0.12, 0.88), (0.15, 1.0), 0.0, 15.0, Tissu),
                volume((0.12, 0.5), (0.25, 0.98), 15.0, 19.0, Coussin),
                volume((0.5, 0.88), (0.25, 0.98), 15.0, 19.0, Coussin),
                volume((0.12, 0.5), (0.15, 0.25), 15.0, 31.0, Coussin),
                volume((0.5, 0.88), (0.15, 0.25), 15.0, 31.0, Coussin),
            ],
            // Le socle, les armoires et leurs portes, le dessus, l'évier et
            // son robinet; le devant est du côté v = 1.
            GenreDObjet::Comptoir => vec![
                volume((0.0, 1.0), (0.0, 0.88), 0.0, 4.0, Noir),
                volume((0.0, 1.0), (0.0, 0.94), 4.0, 35.0, Armoire),
                volume((0.248, 0.252), (0.94, 0.945), 6.0, 33.0, Noir),
                volume((0.498, 0.502), (0.94, 0.945), 6.0, 33.0, Noir),
                volume((0.748, 0.752), (0.94, 0.945), 6.0, 33.0, Noir),
                volume((0.0, 1.0), (0.0, 1.0), 35.0, 36.5, Comptoir),
                volume((0.4, 0.6), (0.15, 0.7), 36.5, 36.7, Metal),
                volume((0.48, 0.52), (0.05, 0.12), 36.5, 44.0, Metal),
            ],
            // Le réservoir, la cuvette et le siège.
            GenreDObjet::Toilette => vec![
                volume((0.1, 0.9), (0.0, 0.26), 0.0, 30.0, Porcelaine),
                volume((0.22, 0.78), (0.26, 0.94), 0.0, 14.5, Porcelaine),
                volume((0.15, 0.85), (0.26, 1.0), 14.5, 16.0, Porcelaine),
            ],
            // Le fond et les quatre rebords (le bain est creux), et le robinet.
            GenreDObjet::Bain => vec![
                volume((0.08, 0.92), (0.06, 0.94), 0.0, 5.0, Porcelaine),
                volume((0.0, 0.08), (0.0, 1.0), 0.0, 20.0, Porcelaine),
                volume((0.92, 1.0), (0.0, 1.0), 0.0, 20.0, Porcelaine),
                volume((0.08, 0.92), (0.0, 0.06), 0.0, 20.0, Porcelaine),
                volume((0.08, 0.92), (0.94, 1.0), 0.0, 20.0, Porcelaine),
                volume((0.45, 0.55), (0.01, 0.05), 20.0, 24.0, Metal),
            ],
            // La base, deux parois de verre et la pomme.
            GenreDObjet::Douche => vec![
                volume((0.0, 1.0), (0.0, 1.0), 0.0, 4.0, Porcelaine),
                volume((0.0, 1.0), (0.98, 1.0), 4.0, 78.0, Verre),
                volume((0.98, 1.0), (0.0, 0.98), 4.0, 78.0, Verre),
                volume((0.45, 0.55), (0.0, 0.05), 70.0, 74.0, Metal),
            ],
            // La caisse, la console et son afficheur, et le couvercle.
            GenreDObjet::Laveuse => vec![
                volume((0.0, 1.0), (0.0, 1.0), 0.0, 36.0, Electromenager),
                volume((0.0, 1.0), (0.0, 0.15), 36.0, 42.0, Electromenager),
                volume((0.4, 0.6), (0.15, 0.16), 38.0, 40.0, Noir),
                volume((0.15, 0.85), (0.25, 0.9), 36.0, 36.4, Metal),
            ],
            // La caisse, le joint des portes et leurs poignées.
            GenreDObjet::Refrigerateur => vec![
                volume((0.0, 1.0), (0.0, 1.0), 0.0, 68.0, Metal),
                volume((0.0, 1.0), (1.0, 1.005), 44.0, 44.6, Noir),
                volume((0.85, 0.9), (1.0, 1.04), 47.0, 62.0, Noir),
                volume((0.85, 0.9), (1.0, 1.04), 20.0, 40.0, Noir),
            ],
            // La caisse, le dessus et ses quatre brûleurs, le dosseret, et le
            // hublot et la poignée du four.
            GenreDObjet::Cuisiniere => {
                let mut parties = vec![
                    volume((0.0, 1.0), (0.0, 1.0), 0.0, 35.6, Electromenager),
                    volume((0.0, 1.0), (0.0, 1.0), 35.6, 36.0, Noir),
                    volume((0.0, 1.0), (0.0, 0.1), 36.0, 42.0, Electromenager),
                    volume((0.15, 0.85), (1.0, 1.01), 8.0, 24.0, Noir),
                    volume((0.1, 0.9), (1.0, 1.04), 28.0, 30.0, Metal),
                ];
                for u in [(0.16, 0.40), (0.60, 0.84)] {
                    for v in [(0.16, 0.44), (0.58, 0.86)] {
                        parties.push(volume(u, v, 36.0, 36.3, Metal));
                    }
                }
                parties
            }
            // La caisse, la console et son afficheur, et le hublot.
            GenreDObjet::Secheuse => vec![
                volume((0.0, 1.0), (0.0, 1.0), 0.0, 36.0, Electromenager),
                volume((0.0, 1.0), (0.0, 0.15), 36.0, 42.0, Electromenager),
                volume((0.4, 0.6), (0.15, 0.16), 38.0, 40.0, Noir),
                volume((0.25, 0.75), (1.0, 1.02), 10.0, 30.0, Noir),
            ],
            // Le réservoir et son dessus, l'eau froide et l'eau chaude, et le
            // robinet de vidange.
            GenreDObjet::ChauffeEau => {
                let mut parties =
                    cylindre((0.0, 1.0), (0.0, 1.0), 0.0, 58.0, Electromenager).to_vec();
                parties.extend(cylindre((0.06, 0.94), (0.06, 0.94), 58.0, 60.0, Metal));
                parties.push(volume((0.25, 0.33), (0.46, 0.54), 60.0, 68.0, Cuivre));
                parties.push(volume((0.67, 0.75), (0.46, 0.54), 60.0, 68.0, Cuivre));
                parties.push(volume((0.46, 0.54), (1.0, 1.05), 6.0, 9.0, Cuivre));
                parties
            }
            // La tête, le bol transparent, et le tuyau de part et d'autre.
            GenreDObjet::FiltreAEau => vec![
                volume((0.25, 0.75), (0.3, 0.7), 42.0, 46.0, Noir),
                volume((0.3, 0.7), (0.3, 0.7), 26.0, 42.0, Verre),
                volume((0.0, 0.25), (0.44, 0.56), 43.0, 45.0, Cuivre),
                volume((0.75, 1.0), (0.44, 0.56), 43.0, 45.0, Cuivre),
            ],
            // Le réservoir de résine et sa tête, le bac à sel et son couvercle.
            GenreDObjet::Adoucisseur => {
                let mut parties = cylindre((0.0, 0.36), (0.2, 0.8), 0.0, 46.0, Noir).to_vec();
                parties.push(volume(
                    (0.04, 0.32),
                    (0.24, 0.76),
                    46.0,
                    54.0,
                    Electromenager,
                ));
                parties.push(volume(
                    (0.42, 0.98),
                    (0.06, 0.94),
                    0.0,
                    36.0,
                    Electromenager,
                ));
                parties.push(volume((0.43, 0.97), (0.07, 0.93), 36.0, 37.0, Noir));
                parties
            }
            // Le tuyau, à 1 pi du plancher, qui tourne d'un quart de tour…
            GenreDObjet::Coude90 => vec![
                volume((0.38, 0.62), (0.38, 1.0), 10.0, 14.0, Cuivre),
                volume((0.62, 1.0), (0.38, 0.62), 10.0, 14.0, Cuivre),
            ],
            // … d'un huitième de tour, en trois pas…
            GenreDObjet::Coude45 => vec![
                volume((0.38, 0.62), (0.5, 1.0), 10.0, 14.0, Cuivre),
                volume((0.5, 0.74), (0.26, 0.5), 10.0, 14.0, Cuivre),
                volume((0.74, 0.98), (0.02, 0.26), 10.0, 14.0, Cuivre),
            ],
            // … ou qui se divise en deux.
            GenreDObjet::Te => vec![
                volume((0.0, 1.0), (0.38, 0.62), 10.0, 14.0, Cuivre),
                volume((0.38, 0.62), (0.62, 1.0), 10.0, 14.0, Cuivre),
            ],
            // Le tuyau, le corps, la tige et la poignée.
            GenreDObjet::RobinetDArret => vec![
                volume((0.0, 0.2), (0.42, 0.58), 10.0, 14.0, Cuivre),
                volume((0.8, 1.0), (0.42, 0.58), 10.0, 14.0, Cuivre),
                volume((0.2, 0.8), (0.3, 0.7), 9.0, 15.0, Cuivre),
                volume((0.45, 0.55), (0.45, 0.55), 15.0, 19.0, Metal),
                volume((0.3, 0.7), (0.44, 0.56), 19.0, 20.0, Noir),
            ],
            // Le clapet antiretour du renvoi, au plancher, et son couvercle.
            GenreDObjet::Clapet => vec![
                volume((0.2, 0.8), (0.28, 0.72), 0.0, 7.0, Noir),
                volume((0.0, 0.2), (0.38, 0.62), 1.0, 6.0, Noir),
                volume((0.8, 1.0), (0.38, 0.62), 1.0, 6.0, Noir),
                volume((0.3, 0.7), (0.36, 0.64), 7.0, 8.0, Metal),
            ],
            // Le tuyau, le corps et son cadran.
            GenreDObjet::CompteurDEau => vec![
                volume((0.0, 0.2), (0.42, 0.58), 12.0, 16.0, Cuivre),
                volume((0.8, 1.0), (0.42, 0.58), 12.0, 16.0, Cuivre),
                volume((0.2, 0.8), (0.2, 0.8), 10.0, 18.0, Cuivre),
                volume((0.3, 0.7), (0.3, 0.7), 18.0, 18.6, Verre),
            ],
            // La grille, au ras du plancher.
            GenreDObjet::DrainDePlancher => {
                vec![volume((0.1, 0.9), (0.1, 0.9), 0.0, 0.3, Metal)]
            }
            // Le tuyau, du plancher au plafond.
            GenreDObjet::ColonneDeChute => {
                vec![jusqu_au_plafond((0.2, 0.8), (0.2, 0.8), 0.0, Noir)]
            }
            // La vasque, son pied, le robinet et son bec.
            GenreDObjet::Lavabo => vec![
                volume((0.0, 1.0), (0.0, 1.0), 30.0, 34.0, Porcelaine),
                volume((0.35, 0.65), (0.25, 0.6), 0.0, 30.0, Porcelaine),
                volume((0.45, 0.55), (0.05, 0.15), 34.0, 40.0, Metal),
                volume((0.47, 0.53), (0.15, 0.3), 38.0, 39.5, Metal),
            ],
            // Le rebord sur le comptoir, les deux cuves, le robinet et son bec.
            GenreDObjet::Evier => vec![
                volume((0.0, 1.0), (0.0, 1.0), 36.5, 36.8, Metal),
                volume((0.06, 0.48), (0.2, 0.9), 36.8, 36.85, Noir),
                volume((0.52, 0.94), (0.2, 0.9), 36.8, 36.85, Noir),
                volume((0.46, 0.54), (0.04, 0.12), 36.8, 48.0, Metal),
                volume((0.47, 0.53), (0.12, 0.4), 45.0, 47.0, Metal),
            ],
            // La boîte et sa porte, le haut à 6 pi.
            GenreDObjet::Panneau => vec![
                volume((0.0, 1.0), (0.0, 1.0), 42.0, 72.0, Metal),
                volume((0.06, 0.94), (1.0, 1.04), 44.0, 70.0, Electromenager),
            ],
            // Le socle et le dôme de verre.
            GenreDObjet::CompteurElectrique => vec![
                volume((0.3, 0.7), (0.0, 0.12), 54.0, 66.0, Electromenager),
                volume((0.33, 0.67), (0.12, 0.36), 56.0, 64.0, Verre),
            ],
            // La plaque au mur et ses deux prises, à 16 po du plancher…
            GenreDObjet::Prise => vec![
                volume((0.38, 0.62), (0.0, 0.04), 13.0, 18.0, Electromenager),
                volume((0.44, 0.56), (0.04, 0.05), 14.0, 15.2, Noir),
                volume((0.44, 0.56), (0.04, 0.05), 15.8, 17.0, Noir),
            ],
            // … au-dessus du comptoir…
            GenreDObjet::PriseDdft => vec![
                volume((0.38, 0.62), (0.0, 0.04), 42.0, 47.0, Electromenager),
                volume((0.44, 0.56), (0.04, 0.05), 43.0, 44.2, Noir),
                volume((0.44, 0.56), (0.04, 0.05), 44.8, 46.0, Noir),
            ],
            // … ou derrière la cuisinière et la sécheuse.
            GenreDObjet::Prise240 => vec![
                volume((0.36, 0.64), (0.0, 0.04), 14.0, 20.0, Electromenager),
                volume((0.43, 0.57), (0.04, 0.05), 15.0, 19.0, Noir),
            ],
            // La plaque et le levier, à 46 po.
            GenreDObjet::Interrupteur => vec![
                volume((0.38, 0.62), (0.0, 0.04), 44.0, 49.0, Electromenager),
                volume((0.47, 0.53), (0.04, 0.08), 46.0, 47.0, Noir),
            ],
            // La base au plafond et le globe.
            GenreDObjet::Luminaire => vec![
                volume((0.4, 0.6), (0.4, 0.6), 94.0, 96.0, Metal),
                volume((0.2, 0.8), (0.2, 0.8), 88.0, 94.0, Literie),
            ],
            // Au ras du plafond.
            GenreDObjet::Encastre => vec![volume((0.1, 0.9), (0.1, 0.9), 95.6, 96.0, Literie)],
            GenreDObjet::Detecteur => {
                vec![volume((0.12, 0.88), (0.12, 0.88), 94.5, 96.0, Literie)]
            }
            GenreDObjet::BoiteDeJonction => {
                vec![volume((0.25, 0.75), (0.25, 0.75), 94.0, 96.0, Metal)]
            }
            // Le corps, au bas du mur, et sa grille.
            GenreDObjet::Plinthe => vec![
                volume((0.0, 1.0), (0.0, 1.0), 3.0, 10.0, Electromenager),
                volume((0.04, 0.96), (1.0, 1.02), 7.0, 9.0, Noir),
            ],
            GenreDObjet::Thermostat => {
                vec![volume((0.3, 0.7), (0.0, 0.12), 58.0, 62.0, Electromenager)]
            }
            // La caisse, son plénum jusqu'au plafond, et sa porte.
            GenreDObjet::Fournaise => vec![
                volume((0.0, 1.0), (0.0, 1.0), 0.0, 48.0, Electromenager),
                jusqu_au_plafond((0.1, 0.9), (0.08, 0.45), 48.0, Metal),
                volume((0.08, 0.92), (1.0, 1.01), 4.0, 44.0, Metal),
            ],
            // L'appareil, haut sur le mur, et son volet.
            GenreDObjet::ThermopompeMurale => vec![
                volume((0.0, 1.0), (0.0, 1.0), 80.0, 92.0, Electromenager),
                volume((0.04, 0.96), (1.0, 1.03), 81.0, 84.0, Metal),
            ],
            // Les pieds, l'appareil et la grille de son ventilateur.
            GenreDObjet::ThermopompeExterieure => vec![
                volume((0.04, 0.96), (0.1, 0.9), 0.0, 4.0, Noir),
                volume((0.0, 1.0), (0.0, 1.0), 4.0, 28.0, Electromenager),
                volume((0.12, 0.6), (1.0, 1.01), 8.0, 26.0, Noir),
            ],
            // L'appareil, accroché sous les solives, et ses quatre conduits.
            GenreDObjet::Echangeur => {
                let mut parties = vec![volume((0.0, 1.0), (0.0, 1.0), 66.0, 84.0, Electromenager)];
                for centre in [0.15, 0.38, 0.62, 0.85] {
                    parties.push(volume(
                        (centre - 0.07, centre + 0.07),
                        (0.22, 0.38),
                        84.0,
                        88.0,
                        Metal,
                    ));
                }
                parties
            }
            // La grille, au plancher…
            GenreDObjet::BoucheDeSoufflage => {
                vec![volume((0.0, 1.0), (0.0, 1.0), 0.0, 0.4, Metal)]
            }
            // … ou au bas du mur.
            GenreDObjet::RetourDAir => vec![volume((0.0, 1.0), (0.0, 0.25), 4.0, 16.0, Metal)],
            GenreDObjet::Ventilateur => vec![volume((0.1, 0.9), (0.1, 0.9), 95.0, 96.0, Literie)],
            // La hotte, à 2 pi au-dessus de la cuisinière, et sa cheminée.
            GenreDObjet::Hotte => vec![
                volume((0.0, 1.0), (0.0, 1.0), 60.0, 66.0, Metal),
                jusqu_au_plafond((0.3, 0.7), (0.0, 0.4), 66.0, Metal),
            ],
            // Le socle, la caisse, ses deux portes et leurs poignées, et le
            // comptoir.
            GenreDObjet::ArmoireDuBas => vec![
                volume((0.0, 1.0), (0.0, 0.86), 0.0, 4.0, Noir),
                volume((0.0, 1.0), (0.0, 0.95), 4.0, 34.5, Armoire),
                volume((0.02, 0.49), (0.95, 0.97), 5.0, 33.0, Armoire),
                volume((0.51, 0.98), (0.95, 0.97), 5.0, 33.0, Armoire),
                volume((0.42, 0.46), (0.97, 0.99), 29.0, 32.0, Metal),
                volume((0.54, 0.58), (0.97, 0.99), 29.0, 32.0, Metal),
                volume((0.0, 1.0), (0.0, 1.0), 34.5, 36.0, Comptoir),
            ],
            // La caisse, à 18 po au-dessus du comptoir, ses portes et leurs
            // poignées.
            GenreDObjet::ArmoireDuHaut => vec![
                volume((0.0, 1.0), (0.0, 0.92), 54.0, 84.0, Armoire),
                volume((0.02, 0.49), (0.92, 0.97), 55.0, 83.0, Armoire),
                volume((0.51, 0.98), (0.92, 0.97), 55.0, 83.0, Armoire),
                volume((0.42, 0.46), (0.97, 1.0), 56.0, 59.0, Metal),
                volume((0.54, 0.58), (0.97, 1.0), 56.0, 59.0, Metal),
            ],
            // Le socle, la caisse et le comptoir, qui déborde du côté des
            // tabourets.
            GenreDObjet::Ilot => vec![
                volume((0.03, 0.97), (0.03, 0.66), 0.0, 4.0, Noir),
                volume((0.02, 0.98), (0.02, 0.7), 4.0, 34.5, Armoire),
                volume((0.0, 1.0), (0.0, 1.0), 34.5, 36.0, Comptoir),
            ],
            // La caisse, la porte et sa poignée, sous le comptoir.
            GenreDObjet::LaveVaisselle => vec![
                volume((0.0, 1.0), (0.0, 0.96), 0.0, 34.5, Metal),
                volume((0.0, 1.0), (0.96, 1.0), 4.0, 34.0, Metal),
                volume((0.2, 0.8), (1.0, 1.03), 31.0, 32.0, Noir),
                volume((0.0, 1.0), (0.0, 1.0), 34.5, 36.0, Comptoir),
            ],
            // Le socle, la caisse et ses portes, le dessus, le robinet et son bec.
            GenreDObjet::Vanite => vec![
                volume((0.0, 1.0), (0.0, 0.86), 0.0, 4.0, Noir),
                volume((0.0, 1.0), (0.0, 0.95), 4.0, 32.5, Armoire),
                volume((0.02, 0.49), (0.95, 0.97), 5.0, 31.0, Armoire),
                volume((0.51, 0.98), (0.95, 0.97), 5.0, 31.0, Armoire),
                volume((0.0, 1.0), (0.0, 1.0), 32.5, 34.0, Porcelaine),
                volume((0.45, 0.55), (0.05, 0.15), 34.0, 40.0, Metal),
                volume((0.47, 0.53), (0.15, 0.3), 38.0, 39.5, Metal),
            ],
        }
    }

    /// Le symbole de l'objet dans son rectangle, en plus de son contour.
    /// `profondeur` (pouces) donne le nombre de marches d'un escalier.
    pub fn symbole(self, profondeur: f64) -> Vec<Trait> {
        use Trait::{Cercle, Ligne, Rectangle, Tirets};
        match self {
            GenreDObjet::Escalier => {
                // Une marche par 10 po de giron, et la flèche qui monte.
                let marches = (profondeur / 10.0).round().clamp(3.0, 20.0) as usize;
                let mut traits: Vec<Trait> = (1..marches)
                    .map(|k| {
                        let v = k as f64 / marches as f64;
                        Ligne((0.0, v), (1.0, v))
                    })
                    .collect();
                traits.push(Ligne((0.5, 0.94), (0.5, 0.06)));
                traits.push(Ligne((0.5, 0.06), (0.38, 0.16)));
                traits.push(Ligne((0.5, 0.06), (0.62, 0.16)));
                traits
            }
            GenreDObjet::Lit => vec![
                Rectangle((0.08, 0.04), (0.46, 0.18)),
                Rectangle((0.54, 0.04), (0.92, 0.18)),
                Ligne((0.0, 0.3), (1.0, 0.3)),
            ],
            GenreDObjet::Table => vec![
                Rectangle((0.25, -0.12), (0.45, 0.0)),
                Rectangle((0.55, -0.12), (0.75, 0.0)),
                Rectangle((0.25, 1.0), (0.45, 1.12)),
                Rectangle((0.55, 1.0), (0.75, 1.12)),
                Rectangle((-0.2, 0.4), (0.0, 0.6)),
                Rectangle((1.0, 0.4), (1.2, 0.6)),
            ],
            GenreDObjet::Sofa => vec![
                Rectangle((0.0, 0.0), (1.0, 0.25)),
                Rectangle((0.0, 0.25), (0.12, 1.0)),
                Rectangle((0.88, 0.25), (1.0, 1.0)),
            ],
            GenreDObjet::Comptoir => vec![
                Ligne((0.0, 0.85), (1.0, 0.85)),
                Rectangle((0.4, 0.15), (0.6, 0.7)),
            ],
            GenreDObjet::Toilette => vec![
                Rectangle((0.1, 0.0), (0.9, 0.28)),
                Cercle((0.5, 0.62), 0.36),
            ],
            GenreDObjet::Bain => vec![
                Rectangle((0.08, 0.06), (0.92, 0.94)),
                Cercle((0.5, 0.14), 0.06),
            ],
            GenreDObjet::Douche => vec![
                Ligne((0.0, 0.0), (1.0, 1.0)),
                Ligne((1.0, 0.0), (0.0, 1.0)),
                Cercle((0.5, 0.5), 0.06),
            ],
            GenreDObjet::Laveuse => vec![Cercle((0.5, 0.55), 0.36)],
            GenreDObjet::Refrigerateur => vec![Ligne((0.0, 0.85), (1.0, 0.85))],
            GenreDObjet::Cuisiniere => vec![
                Cercle((0.28, 0.3), 0.14),
                Cercle((0.72, 0.3), 0.14),
                Cercle((0.28, 0.72), 0.14),
                Cercle((0.72, 0.72), 0.14),
            ],
            GenreDObjet::Secheuse => vec![
                Ligne((0.0, 0.15), (1.0, 0.15)),
                Rectangle((0.15, 0.3), (0.85, 0.9)),
            ],
            // Le symbole des plans : un réservoir rond.
            GenreDObjet::ChauffeEau => vec![Cercle((0.5, 0.5), 0.5), Cercle((0.5, 0.5), 0.4)],
            GenreDObjet::FiltreAEau => vec![
                Ligne((0.0, 0.5), (0.2, 0.5)),
                Ligne((0.8, 0.5), (1.0, 0.5)),
                Cercle((0.5, 0.5), 0.3),
            ],
            GenreDObjet::Adoucisseur => vec![
                Cercle((0.18, 0.5), 0.3),
                Rectangle((0.42, 0.06), (0.98, 0.94)),
            ],
            // Le tuyau en double trait, et un trait à chaque joint.
            GenreDObjet::Coude90 => vec![
                Ligne((0.38, 1.0), (0.38, 0.38)),
                Ligne((0.38, 0.38), (1.0, 0.38)),
                Ligne((0.62, 1.0), (0.62, 0.62)),
                Ligne((0.62, 0.62), (1.0, 0.62)),
                Ligne((0.28, 0.9), (0.72, 0.9)),
                Ligne((0.9, 0.28), (0.9, 0.72)),
            ],
            GenreDObjet::Coude45 => vec![
                Ligne((0.38, 1.0), (0.38, 0.45)),
                Ligne((0.38, 0.45), (0.815, 0.015)),
                Ligne((0.62, 1.0), (0.62, 0.55)),
                Ligne((0.62, 0.55), (0.985, 0.185)),
                Ligne((0.28, 0.9), (0.72, 0.9)),
                Ligne((0.674, 0.015), (0.985, 0.326)),
            ],
            GenreDObjet::Te => vec![
                Ligne((0.0, 0.38), (1.0, 0.38)),
                Ligne((0.0, 0.62), (0.38, 0.62)),
                Ligne((0.62, 0.62), (1.0, 0.62)),
                Ligne((0.38, 0.62), (0.38, 1.0)),
                Ligne((0.62, 0.62), (0.62, 1.0)),
                Ligne((0.1, 0.28), (0.1, 0.72)),
                Ligne((0.9, 0.28), (0.9, 0.72)),
                Ligne((0.28, 0.9), (0.72, 0.9)),
            ],
            // Deux triangles pointe à pointe sur le tuyau, et la poignée.
            GenreDObjet::RobinetDArret => vec![
                Ligne((0.0, 0.5), (0.2, 0.5)),
                Ligne((0.8, 0.5), (1.0, 0.5)),
                Ligne((0.2, 0.25), (0.2, 0.75)),
                Ligne((0.8, 0.25), (0.8, 0.75)),
                Ligne((0.2, 0.25), (0.8, 0.75)),
                Ligne((0.2, 0.75), (0.8, 0.25)),
                Ligne((0.5, 0.5), (0.5, 0.1)),
                Ligne((0.36, 0.1), (0.64, 0.1)),
            ],
            // Le clapet, et le sens de l'écoulement.
            GenreDObjet::Clapet => vec![
                Ligne((0.0, 0.5), (0.2, 0.5)),
                Ligne((0.8, 0.5), (1.0, 0.5)),
                Ligne((0.2, 0.3), (0.2, 0.7)),
                Ligne((0.8, 0.3), (0.8, 0.7)),
                Ligne((0.2, 0.7), (0.8, 0.3)),
                Ligne((0.32, 0.86), (0.68, 0.86)),
                Ligne((0.68, 0.86), (0.6, 0.8)),
                Ligne((0.68, 0.86), (0.6, 0.92)),
            ],
            GenreDObjet::CompteurDEau => vec![
                Cercle((0.5, 0.5), 0.36),
                Ligne((0.0, 0.5), (0.14, 0.5)),
                Ligne((0.86, 0.5), (1.0, 0.5)),
            ],
            // Une grille ronde.
            GenreDObjet::DrainDePlancher => vec![
                Cercle((0.5, 0.5), 0.45),
                Cercle((0.5, 0.5), 0.18),
                Ligne((0.5, 0.1), (0.5, 0.32)),
                Ligne((0.5, 0.68), (0.5, 0.9)),
                Ligne((0.1, 0.5), (0.32, 0.5)),
                Ligne((0.68, 0.5), (0.9, 0.5)),
            ],
            GenreDObjet::ColonneDeChute => vec![Cercle((0.5, 0.5), 0.45), Cercle((0.5, 0.5), 0.3)],
            // La vasque, son drain et le robinet.
            GenreDObjet::Lavabo => {
                let mut traits = ovale((0.5, 0.56), 0.36, 0.32);
                traits.push(Cercle((0.5, 0.56), 0.05));
                traits.push(Ligne((0.5, 0.06), (0.5, 0.2)));
                traits
            }
            // Les deux cuves, leurs drains et le robinet.
            GenreDObjet::Evier => vec![
                Rectangle((0.06, 0.2), (0.48, 0.9)),
                Rectangle((0.52, 0.2), (0.94, 0.9)),
                Cercle((0.27, 0.62), 0.05),
                Cercle((0.73, 0.62), 0.05),
                Ligne((0.5, 0.04), (0.5, 0.17)),
            ],
            // La porte; son sigle le nomme.
            GenreDObjet::Panneau => vec![Ligne((0.04, 0.82), (0.96, 0.82))],
            GenreDObjet::CompteurElectrique => {
                vec![Cercle((0.5, 0.55), 0.36), Ligne((0.5, 0.0), (0.5, 0.19))]
            }
            // Le symbole des plans : un rond traversé de deux traits…
            GenreDObjet::Prise | GenreDObjet::PriseDdft => vec![
                Cercle((0.5, 0.3), 0.22),
                Ligne((0.42, 0.02), (0.42, 0.58)),
                Ligne((0.58, 0.02), (0.58, 0.58)),
            ],
            // … ou de trois.
            GenreDObjet::Prise240 => vec![
                Cercle((0.5, 0.3), 0.24),
                Ligne((0.38, 0.02), (0.38, 0.58)),
                Ligne((0.5, 0.02), (0.5, 0.58)),
                Ligne((0.62, 0.02), (0.62, 0.58)),
            ],
            // Un trait jusqu'au mur; le S est son sigle.
            GenreDObjet::Interrupteur => vec![Ligne((0.5, 0.0), (0.5, 0.3))],
            // Un rond qui rayonne.
            GenreDObjet::Luminaire => vec![
                Cercle((0.5, 0.5), 0.28),
                Ligne((0.5, 0.04), (0.5, 0.2)),
                Ligne((0.5, 0.8), (0.5, 0.96)),
                Ligne((0.04, 0.5), (0.2, 0.5)),
                Ligne((0.8, 0.5), (0.96, 0.5)),
            ],
            GenreDObjet::Encastre => vec![Cercle((0.5, 0.5), 0.4), Cercle((0.5, 0.5), 0.22)],
            GenreDObjet::Detecteur => vec![Cercle((0.5, 0.5), 0.44)],
            GenreDObjet::BoiteDeJonction | GenreDObjet::Thermostat => {
                vec![Cercle((0.5, 0.5), 0.4)]
            }
            // La résistance, en zigzag.
            GenreDObjet::Plinthe => {
                let point = |k: usize| {
                    (
                        0.04 + 0.115 * k as f64,
                        if k.is_multiple_of(2) { 0.25 } else { 0.75 },
                    )
                };
                (0..8).map(|k| Ligne(point(k), point(k + 1))).collect()
            }
            // Le plénum dessus, et la porte devant.
            GenreDObjet::Fournaise => vec![
                Rectangle((0.1, 0.08), (0.9, 0.45)),
                Ligne((0.0, 0.88), (1.0, 0.88)),
            ],
            // Au-dessus de la coupe : en tirets, comme son contour.
            GenreDObjet::ThermopompeMurale => vec![Tirets((0.04, 0.7), (0.96, 0.7))],
            // Le ventilateur et le panneau de service.
            GenreDObjet::ThermopompeExterieure => vec![
                Cercle((0.36, 0.5), 0.4),
                Cercle((0.36, 0.5), 0.08),
                Rectangle((0.74, 0.2), (0.94, 0.8)),
            ],
            // Les quatre conduits, sur le dessus.
            GenreDObjet::Echangeur => [0.15, 0.38, 0.62, 0.85]
                .map(|u| Cercle((u, 0.3), 0.18))
                .to_vec(),
            // Les ailettes.
            GenreDObjet::BoucheDeSoufflage => [0.2, 0.35, 0.5, 0.65, 0.8]
                .map(|u| Ligne((u, 0.2), (u, 0.8)))
                .to_vec(),
            GenreDObjet::RetourDAir => {
                vec![Ligne((0.0, 0.0), (1.0, 1.0)), Ligne((0.0, 1.0), (1.0, 0.0))]
            }
            // Un carré et son ventilateur.
            GenreDObjet::Ventilateur => {
                vec![Rectangle((0.1, 0.1), (0.9, 0.9)), Cercle((0.5, 0.5), 0.28)]
            }
            // La cheminée, en tirets comme la hotte.
            GenreDObjet::Hotte => vec![
                Tirets((0.3, 0.0), (0.3, 0.4)),
                Tirets((0.3, 0.4), (0.7, 0.4)),
                Tirets((0.7, 0.4), (0.7, 0.0)),
            ],
            // La face des portes et leur joint.
            GenreDObjet::ArmoireDuBas => {
                vec![Ligne((0.0, 0.9), (1.0, 0.9)), Ligne((0.5, 0.9), (0.5, 1.0))]
            }
            // Au-dessus de la coupe : en tirets, en croix.
            GenreDObjet::ArmoireDuHaut => vec![
                Tirets((0.0, 0.0), (1.0, 1.0)),
                Tirets((0.0, 1.0), (1.0, 0.0)),
            ],
            // La face des armoires; le comptoir déborde au-delà.
            GenreDObjet::Ilot => vec![Ligne((0.0, 0.7), (1.0, 0.7))],
            GenreDObjet::LaveVaisselle => vec![Ligne((0.0, 0.88), (1.0, 0.88))],
            // La vasque dans le comptoir, son drain et le robinet.
            GenreDObjet::Vanite => {
                let mut traits = ovale((0.5, 0.55), 0.3, 0.3);
                traits.push(Cercle((0.5, 0.58), 0.05));
                traits.push(Ligne((0.5, 0.06), (0.5, 0.2)));
                traits
            }
        }
    }

    /// Sa famille première, celle de son calque au DXF; il peut paraître
    /// dans d'autres rangées (`Famille::genres`).
    pub fn famille(self) -> Famille {
        use GenreDObjet::*;
        match self {
            Escalier | Lit | Table | Sofa | Comptoir | Toilette | Bain | Douche | Laveuse
            | Refrigerateur | Cuisiniere | Secheuse => Famille::Mobilier,
            ChauffeEau | FiltreAEau | Adoucisseur | Coude90 | Coude45 | Te | RobinetDArret
            | Clapet | CompteurDEau | DrainDePlancher | ColonneDeChute => Famille::Plomberie,
            Panneau | CompteurElectrique | Prise | PriseDdft | Prise240 | Interrupteur
            | Luminaire | Encastre | Detecteur | BoiteDeJonction | Plinthe | Thermostat => {
                Famille::Electricite
            }
            Fournaise
            | ThermopompeMurale
            | ThermopompeExterieure
            | Echangeur
            | BoucheDeSoufflage
            | RetourDAir
            | Ventilateur
            | Hotte => Famille::Ventilation,
            Lavabo | Evier | ArmoireDuBas | ArmoireDuHaut | Ilot | LaveVaisselle | Vanite => {
                Famille::CuisineEtBain
            }
        }
    }

    /// Vrai s'il occupe le plancher : il ne chevauche alors ni les murs, ni
    /// les cloisons, ni un autre objet qui l'occupe aussi (R-98). Un symbole
    /// au mur, au plafond ou sur un tuyau, une armoire du haut, un évier dans
    /// son comptoir se posent n'importe où, même hors de la pièce.
    pub fn encombre(self) -> bool {
        use GenreDObjet::*;
        matches!(
            self,
            Escalier
                | Lit
                | Table
                | Sofa
                | Comptoir
                | Toilette
                | Bain
                | Douche
                | Laveuse
                | Refrigerateur
                | Cuisiniere
                | Secheuse
                | ChauffeEau
                | Adoucisseur
                | Lavabo
                | Fournaise
                | ArmoireDuBas
                | Ilot
                | LaveVaisselle
                | Vanite
        )
    }

    /// Vrai s'il se pose contre un mur, le dos (v = 0) contre lui, quand on
    /// touche la pièce près d'un mur ou d'une cloison (`contre_le_mur`).
    pub fn contre_un_mur(self) -> bool {
        use GenreDObjet::*;
        matches!(
            self,
            Comptoir
                | Toilette
                | Laveuse
                | Refrigerateur
                | Cuisiniere
                | Secheuse
                | Lavabo
                | Evier
                | Panneau
                | Prise
                | PriseDdft
                | Prise240
                | Interrupteur
                | Plinthe
                | Thermostat
                | ThermopompeMurale
                | RetourDAir
                | Hotte
                | ArmoireDuBas
                | ArmoireDuHaut
                | LaveVaisselle
                | Vanite
        )
    }

    /// Vrai s'il est fixé au plafond : en 3D, ses parties, décrites sous un
    /// plafond de 8 pi (`PLAFOND_DE_REFERENCE`), montent avec le plafond.
    pub fn au_plafond(self) -> bool {
        use GenreDObjet::*;
        matches!(
            self,
            Luminaire | Encastre | Detecteur | BoiteDeJonction | Ventilateur
        )
    }

    /// Le contour de son rectangle sur le plan.
    pub fn contour(self) -> Contour {
        use GenreDObjet::*;
        match self {
            ThermopompeMurale | Hotte | ArmoireDuHaut => Contour::Tirets,
            ChauffeEau | FiltreAEau | Coude90 | Coude45 | Te | RobinetDArret | Clapet
            | CompteurDEau | DrainDePlancher | ColonneDeChute | CompteurElectrique | Prise
            | PriseDdft | Prise240 | Interrupteur | Luminaire | Encastre | Detecteur
            | BoiteDeJonction | Thermostat | Ventilateur => Contour::Aucun,
            _ => Contour::Plein,
        }
    }

    /// Vrai si son nom s'écrit sur le plan, faute de sigle : un meuble, un
    /// appareil. Un petit symbole (une prise, un coude) se suffit; les
    /// armoires, l'évier et la hotte, en rangée le long d'un mur, aussi : la
    /// légende du PDF les nomme.
    pub fn nomme_sur_le_plan(self) -> bool {
        use GenreDObjet::*;
        let (largeur, profondeur, _) = self.dimensions();
        !matches!(self, ArmoireDuBas | ArmoireDuHaut | Evier | Hotte)
            && self.contour() != Contour::Aucun
            && largeur.min(profondeur) >= 8.0
    }

    /// Vrai si son sigle s'écrit à côté de son symbole, du côté de son devant,
    /// plutôt que dessus : une prise DDFT, une prise 240 V, un interrupteur.
    pub fn sigle_a_cote(self) -> bool {
        matches!(
            self,
            GenreDObjet::PriseDdft | GenreDObjet::Prise240 | GenreDObjet::Interrupteur
        )
    }

    /// Où s'écrit son sigle (DDFT, 240 V…), dans son rectangle (u, v) : au
    /// milieu d'un rond, ou, s'il va à côté du symbole (`sigle_a_cote`), au
    /// bord de celui-ci.
    pub fn place_du_sigle(self) -> (f64, f64) {
        match self {
            GenreDObjet::PriseDdft | GenreDObjet::Prise240 => (0.5, 0.6),
            GenreDObjet::Interrupteur => (0.5, 0.3),
            GenreDObjet::CompteurElectrique => (0.5, 0.55),
            GenreDObjet::Echangeur => (0.5, 0.76),
            _ => (0.5, 0.5),
        }
    }
}

/// Une famille d'objets, comme un corps de métier de la calculatrice : la
/// rangée de pastilles de la section des objets (section 25.1, point 40).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Famille {
    #[default]
    Mobilier,
    Plomberie,
    Electricite,
    Ventilation,
    CuisineEtBain,
}

impl Famille {
    /// Les familles, dans l'ordre de la rangée.
    pub const TOUTES: [Famille; 5] = [
        Famille::Mobilier,
        Famille::Plomberie,
        Famille::Electricite,
        Famille::Ventilation,
        Famille::CuisineEtBain,
    ];

    /// Ses objets, dans l'ordre de leurs boutons. Un objet peut être de
    /// plusieurs familles : une toilette est un meuble de la salle de bain et
    /// un appareil du plombier; un thermostat sert à l'électricien comme au
    /// chauffagiste.
    pub fn genres(self) -> &'static [GenreDObjet] {
        use GenreDObjet::*;
        match self {
            Famille::Mobilier => &[
                Escalier,
                Lit,
                Table,
                Sofa,
                Comptoir,
                Toilette,
                Bain,
                Douche,
                Laveuse,
                Secheuse,
                Refrigerateur,
                Cuisiniere,
            ],
            Famille::Plomberie => &[
                ChauffeEau,
                FiltreAEau,
                Adoucisseur,
                Coude90,
                Coude45,
                Te,
                RobinetDArret,
                Clapet,
                CompteurDEau,
                DrainDePlancher,
                ColonneDeChute,
                Toilette,
                Bain,
                Douche,
                Lavabo,
                Evier,
                Laveuse,
            ],
            Famille::Electricite => &[
                Panneau,
                CompteurElectrique,
                Prise,
                PriseDdft,
                Prise240,
                Interrupteur,
                Luminaire,
                Encastre,
                Detecteur,
                BoiteDeJonction,
                Plinthe,
                Thermostat,
            ],
            Famille::Ventilation => &[
                Fournaise,
                ThermopompeMurale,
                ThermopompeExterieure,
                Echangeur,
                BoucheDeSoufflage,
                RetourDAir,
                Ventilateur,
                Hotte,
                Plinthe,
                Thermostat,
            ],
            Famille::CuisineEtBain => &[
                ArmoireDuBas,
                ArmoireDuHaut,
                Ilot,
                Comptoir,
                Evier,
                LaveVaisselle,
                Refrigerateur,
                Cuisiniere,
                Hotte,
                Vanite,
                Lavabo,
                Toilette,
                Bain,
                Douche,
            ],
        }
    }

    /// Le calque de ses objets au DXF : les meubles, les armoires et les
    /// appareils sanitaires ensemble, chaque métier à part.
    pub fn calque(self) -> &'static str {
        match self {
            Famille::Mobilier | Famille::CuisineEtBain => "MOBILIER",
            Famille::Plomberie => "PLOMBERIE",
            Famille::Electricite => "ELECTRICITE",
            Famille::Ventilation => "VENTILATION",
        }
    }
}

/// Le contour du rectangle d'un objet sur le plan.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Contour {
    /// En trait plein : un meuble, un appareil.
    Plein,
    /// En tirets : ce qui est au-dessus de la coupe du plan (une armoire du
    /// haut, une hotte, une thermopompe murale).
    Tirets,
    /// Aucun : le symbole se suffit (une prise, un coude, un chauffe-eau rond).
    Aucun,
}

/// La hauteur du plafond sous lequel se décrivent les objets du plafond et
/// les parties qui y montent : 8 pi (pouces).
pub const PLAFOND_DE_REFERENCE: f64 = 96.0;

/// Un cylindre debout dans le rectangle (u, v), de `bas` à `haut` : trois
/// blocs qui en abattent les coins.
fn cylindre(u: (f64, f64), v: (f64, f64), bas: f64, haut: f64, matiere: Matiere) -> [Volume; 3] {
    let (du, dv) = ((u.1 - u.0) * 0.15, (v.1 - v.0) * 0.15);
    [
        volume((u.0 + du, u.1 - du), v, bas, haut, matiere),
        volume((u.0, u.0 + du), (v.0 + dv, v.1 - dv), bas, haut, matiere),
        volume((u.1 - du, u.1), (v.0 + dv, v.1 - dv), bas, haut, matiere),
    ]
}

/// Un ovale de centre `centre` et de demi-axes `ru` et `rv` (en part de la
/// largeur et de la profondeur), en vingt traits : il s'étire avec l'objet.
fn ovale(centre: (f64, f64), ru: f64, rv: f64) -> Vec<Trait> {
    let point = |k: usize| {
        let angle = std::f64::consts::TAU * k as f64 / 20.0;
        (centre.0 + ru * angle.cos(), centre.1 + rv * angle.sin())
    };
    (0..20)
        .map(|k| Trait::Ligne(point(k), point(k + 1)))
        .collect()
}

/// La longueur d'un tiret et l'écart entre deux, sur le plan (pouces).
const TIRET: f64 = 3.0;
const ECART_DES_TIRETS: f64 = 2.0;

/// Un trait en tirets de `a` à `b` (sur le plan), qui commence et finit par
/// un tiret.
fn en_tirets(a: (f64, f64), b: (f64, f64), traits: &mut Vec<TraitSurLePlan>) {
    let longueur = (b.0 - a.0).hypot(b.1 - a.1);
    let tirets = ((longueur + ECART_DES_TIRETS) / (TIRET + ECART_DES_TIRETS))
        .round()
        .max(1.0);
    let tiret = (longueur - (tirets - 1.0) * ECART_DES_TIRETS) / tirets;
    if tirets < 2.0 || tiret <= 0.0 {
        traits.push(TraitSurLePlan::Ligne(a, b));
        return;
    }
    let point = |t: f64| {
        (
            a.0 + (b.0 - a.0) * t / longueur,
            a.1 + (b.1 - a.1) * t / longueur,
        )
    };
    for k in 0..tirets as usize {
        let debut = k as f64 * (tiret + ECART_DES_TIRETS);
        traits.push(TraitSurLePlan::Ligne(point(debut), point(debut + tiret)));
    }
}

/// Ce dont une partie d'un meuble est faite, pour sa couleur en 3D.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Matiere {
    Bois,
    BoisFonce,
    Tissu,
    Coussin,
    Literie,
    Porcelaine,
    Metal,
    Electromenager,
    Noir,
    Verre,
    Comptoir,
    Armoire,
    /// Le cuivre des tuyaux, des robinets et des raccords.
    Cuivre,
}

/// Une partie d'un meuble en 3D : un bloc, de `u` et `v` (en part de la
/// largeur et de la profondeur du meuble, comme son symbole) et de `bas` à
/// `haut` (pouces). Deux parties d'un même meuble ne se chevauchent pas.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Volume {
    pub u: (f64, f64),
    pub v: (f64, f64),
    pub bas: f64,
    pub haut: f64,
    pub matiere: Matiere,
    /// Vrai s'il monte jusqu'au plafond de la pièce (une colonne de chute, le
    /// plénum d'une fournaise) : `haut` est alors celui d'un plafond de 8 pi.
    pub jusqu_au_plafond: bool,
}

const fn volume(u: (f64, f64), v: (f64, f64), bas: f64, haut: f64, matiere: Matiere) -> Volume {
    Volume {
        u,
        v,
        bas,
        haut,
        matiere,
        jusqu_au_plafond: false,
    }
}

/// Une partie qui monte de `bas` jusqu'au plafond.
const fn jusqu_au_plafond(u: (f64, f64), v: (f64, f64), bas: f64, matiere: Matiere) -> Volume {
    Volume {
        u,
        v,
        bas,
        haut: PLAFOND_DE_REFERENCE,
        matiere,
        jusqu_au_plafond: true,
    }
}

/// Une chaise de la table : son siège, et son dossier du côté loin de la
/// table (`dossier` : la direction de ce côté, en u ou en v).
fn chaise(u: (f64, f64), v: (f64, f64), dossier: (f64, f64)) -> [Volume; 2] {
    // Le dossier occupe le sixième de la chaise, du côté de `dossier`.
    let couper = |(a, b): (f64, f64), sens: f64| {
        let part = (b - a) / 6.0;
        if sens > 0.0 {
            ((a, b - part), (b - part, b))
        } else if sens < 0.0 {
            ((a + part, b), (a, a + part))
        } else {
            ((a, b), (a, b))
        }
    };
    let (siege_u, dos_u) = couper(u, dossier.0);
    let (siege_v, dos_v) = couper(v, dossier.1);
    [
        volume(siege_u, siege_v, 0.0, 18.0, Matiere::Bois),
        volume(dos_u, dos_v, 0.0, 34.0, Matiere::Bois),
    ]
}

/// Un trait d'un symbole, dans le rectangle de l'objet.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Trait {
    Ligne((f64, f64), (f64, f64)),
    /// Une ligne en tirets : ce qui est au-dessus de la coupe du plan.
    Tirets((f64, f64), (f64, f64)),
    Rectangle((f64, f64), (f64, f64)),
    /// Un cercle : son centre, et son rayon en part du plus petit côté.
    Cercle((f64, f64), f64),
}

/// Un trait d'un symbole, une fois sur le plan (pouces).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TraitSurLePlan {
    Ligne((f64, f64), (f64, f64)),
    Cercle((f64, f64), f64),
}

/// Un objet posé dans une pièce.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Objet {
    pub genre: GenreDObjet,
    /// Son centre, en pouces, dans le repère du plan (le départ du tracé en
    /// (0, 0), y vers le bas).
    pub centre: (f64, f64),
    /// Sa largeur et sa profondeur, telles qu'inscrites; vides : celles de départ.
    #[serde(default)]
    pub largeur: String,
    #[serde(default)]
    pub profondeur: String,
    /// Ses quarts de tour, dans le sens des aiguilles d'une montre (0 à 3).
    #[serde(default)]
    pub rotation: u8,
    /// Ses degrés de rotation en plus de ses quarts de tour, dans le même sens
    /// (par pas de 15°).
    #[serde(default)]
    pub angle: f64,
    /// La forme d'un escalier (point 30); sans objet pour un meuble.
    #[serde(default)]
    pub forme: FormeDEscalier,
    /// La hauteur à monter d'un escalier, telle qu'inscrite; vide : celle des murs.
    #[serde(default)]
    pub hauteur: String,
    /// Le nombre de contremarches d'un escalier, tel qu'inscrit; vide : calculé
    /// pour des contremarches d'au plus 7 1/2 po.
    #[serde(default)]
    pub contremarches: String,
    /// Vrai s'il est retourné comme dans un miroir, sa gauche et sa droite
    /// échangées (point 31) : un escalier tournant tourne de l'autre côté.
    #[serde(default)]
    pub retourne: bool,
}

impl Objet {
    /// Un objet neuf, à ses dimensions de départ.
    pub fn nouveau(genre: GenreDObjet, centre: (f64, f64)) -> Objet {
        Objet {
            genre,
            centre,
            largeur: String::new(),
            profondeur: String::new(),
            rotation: 0,
            angle: 0.0,
            forme: FormeDEscalier::Droit,
            hauteur: String::new(),
            contremarches: String::new(),
            retourne: false,
        }
    }

    /// Sa largeur et sa profondeur de départ, en pouces : celles de son genre,
    /// ou, pour un escalier, celles de sa forme.
    pub fn dimensions_de_depart(&self) -> (f64, f64) {
        if self.genre == GenreDObjet::Escalier {
            self.forme.dimensions()
        } else {
            let (largeur, profondeur, _) = self.genre.dimensions();
            (largeur, profondeur)
        }
    }

    /// Son angle, en degrés de 0 à 360, dans le sens des aiguilles d'une montre.
    pub fn angle_total(&self) -> f64 {
        (f64::from(self.rotation) * 90.0 + self.angle).rem_euclid(360.0)
    }

    /// Le fait tourner de `degres` (négatif : dans l'autre sens).
    pub fn pivoter(&mut self, degres: f64) {
        self.angle = (self.angle + degres).rem_euclid(360.0);
    }

    /// Un point de son rectangle (u, v), sur le plan; `dimensions` est sa
    /// largeur et sa profondeur en pouces.
    pub fn vers_le_plan(
        &self,
        (u, v): (f64, f64),
        (largeur, profondeur): (f64, f64),
    ) -> (f64, f64) {
        // Retourné : sa gauche et sa droite échangées.
        let u = if self.retourne { 1.0 - u } else { u };
        let (dx, dy) = ((u - 0.5) * largeur, (v - 0.5) * profondeur);
        let angle = self.angle_total();
        let quarts = angle / 90.0;
        // Un quart de tour exact garde des coins exacts.
        let (dx, dy) = if (quarts - quarts.round()).abs() < 1e-9 {
            match (quarts.round() as i64).rem_euclid(4) {
                0 => (dx, dy),
                1 => (-dy, dx),
                2 => (-dx, -dy),
                _ => (dy, -dx),
            }
        } else {
            let (sinus, cosinus) = angle.to_radians().sin_cos();
            (dx * cosinus - dy * sinus, dx * sinus + dy * cosinus)
        };
        (self.centre.0 + dx, self.centre.1 + dy)
    }

    /// Les quatre coins de son rectangle, sur le plan.
    pub fn coins(&self, dimensions: (f64, f64)) -> [(f64, f64); 4] {
        [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]
            .map(|coin| self.vers_le_plan(coin, dimensions))
    }

    /// Son contour et son symbole, sur le plan. `contremarches` : celles d'un
    /// escalier calculé (sinon, une marche par 10 po de profondeur).
    pub fn traits_sur_le_plan(
        &self,
        dimensions: (f64, f64),
        contremarches: Option<usize>,
    ) -> Vec<TraitSurLePlan> {
        let coins = self.coins(dimensions);
        let escalier = self.genre == GenreDObjet::Escalier;
        // Un colimaçon n'a pas de contour carré : son cercle en tient lieu; un
        // symbole se suffit; ce qui est au-dessus de la coupe est en tirets.
        let contour = if escalier && self.forme == FormeDEscalier::Colimacon {
            Contour::Aucun
        } else {
            self.genre.contour()
        };
        let mut traits: Vec<TraitSurLePlan> = Vec::new();
        for k in 0..4 {
            let (a, b) = (coins[k], coins[(k + 1) % 4]);
            match contour {
                Contour::Plein => traits.push(TraitSurLePlan::Ligne(a, b)),
                Contour::Tirets => en_tirets(a, b, &mut traits),
                Contour::Aucun => {}
            }
        }
        let plus_petit = dimensions.0.min(dimensions.1);
        let symbole = if escalier {
            let marches =
                contremarches.unwrap_or(((dimensions.1 / 10.0).round().clamp(3.0, 20.0)) as usize);
            symbole_d_escalier(self.forme, dimensions, marches)
        } else {
            self.genre.symbole(dimensions.1)
        };
        for trait_ in symbole {
            match trait_ {
                Trait::Ligne(a, b) => traits.push(TraitSurLePlan::Ligne(
                    self.vers_le_plan(a, dimensions),
                    self.vers_le_plan(b, dimensions),
                )),
                Trait::Tirets(a, b) => en_tirets(
                    self.vers_le_plan(a, dimensions),
                    self.vers_le_plan(b, dimensions),
                    &mut traits,
                ),
                Trait::Rectangle((u0, v0), (u1, v1)) => {
                    let r = [(u0, v0), (u1, v0), (u1, v1), (u0, v1)]
                        .map(|c| self.vers_le_plan(c, dimensions));
                    for k in 0..4 {
                        traits.push(TraitSurLePlan::Ligne(r[k], r[(k + 1) % 4]));
                    }
                }
                Trait::Cercle(centre, rayon) => traits.push(TraitSurLePlan::Cercle(
                    self.vers_le_plan(centre, dimensions),
                    rayon * plus_petit,
                )),
            }
        }
        traits
    }

    /// Vrai si le point (sur le plan) est dans son rectangle.
    pub fn contient(&self, point: (f64, f64), dimensions: (f64, f64)) -> bool {
        let coins = self.coins(dimensions);
        let signes: Vec<f64> = (0..4)
            .map(|k| {
                let (a, b) = (coins[k], coins[(k + 1) % 4]);
                (b.0 - a.0) * (point.1 - a.1) - (b.1 - a.1) * (point.0 - a.0)
            })
            .collect();
        signes.iter().all(|s| *s >= 0.0) || signes.iter().all(|s| *s <= 0.0)
    }
}

// ----------------------------------------------------------------------
// Escaliers (point 30, R-101) : forme, calcul des contremarches, symbole et
// marches en 3D, dans le rectangle de l'escalier (u, v de 0 à 1; le bas en
// v = 1, la montée vers v = 0).
// ----------------------------------------------------------------------

/// La forme d'un escalier.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FormeDEscalier {
    #[default]
    Droit,
    /// En L : deux volées et un palier dans le coin.
    EnL,
    /// En U : deux volées côte à côte et un palier qui les relie.
    EnU,
    /// Quart tournant : deux volées et deux marches dansantes dans le coin.
    Tournant,
    /// Colimaçon : des marches en éventail autour d'un poteau.
    Colimacon,
}

impl FormeDEscalier {
    /// Toutes les formes, dans l'ordre du choix.
    pub const TOUTES: [FormeDEscalier; 5] = [
        FormeDEscalier::Droit,
        FormeDEscalier::EnL,
        FormeDEscalier::EnU,
        FormeDEscalier::Tournant,
        FormeDEscalier::Colimacon,
    ];

    /// Largeur et profondeur de départ de son emprise, en pouces.
    pub fn dimensions(self) -> (f64, f64) {
        match self {
            FormeDEscalier::Droit => (36.0, 120.0),
            FormeDEscalier::EnL | FormeDEscalier::Tournant => (72.0, 96.0),
            FormeDEscalier::EnU => (84.0, 84.0),
            FormeDEscalier::Colimacon => (60.0, 60.0),
        }
    }
}

/// Contremarche visée d'un escalier du dessin, en pouces : la valeur de départ
/// du calcul d'escalier (section 13.4). Ce n'est pas une norme.
pub const CONTREMARCHE_VISEE: f64 = 7.5;
/// Largeur d'une volée d'un escalier en L, en U ou tournant, en pouces.
pub const LARGEUR_DE_VOLEE: f64 = 36.0;
/// Nombre de contremarches permis.
const CONTREMARCHES_PERMISES: std::ops::RangeInclusive<usize> = 3..=40;

/// Un escalier calculé.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EscalierCalcule {
    /// La hauteur à monter, en pouces.
    pub hauteur: f64,
    pub contremarches: usize,
    /// La hauteur d'une contremarche, en pouces.
    pub contremarche: f64,
    /// Le giron d'un escalier droit : sa profondeur partagée entre ses marches.
    pub giron: Option<f64>,
}

/// Calcule un escalier : `contremarches` inscrites, ou le plus petit nombre
/// qui donne des contremarches d'au plus 7 1/2 po. Rien si la hauteur n'en
/// est pas une.
pub fn calculer_l_escalier(
    forme: FormeDEscalier,
    (_, profondeur): (f64, f64),
    hauteur: f64,
    contremarches: Option<usize>,
) -> Option<EscalierCalcule> {
    if !(hauteur.is_finite() && hauteur > 0.0) {
        return None;
    }
    let calcule = (hauteur / CONTREMARCHE_VISEE - 1e-9).ceil() as usize;
    let n = contremarches
        .filter(|n| CONTREMARCHES_PERMISES.contains(n))
        .unwrap_or(calcule.clamp(
            *CONTREMARCHES_PERMISES.start(),
            *CONTREMARCHES_PERMISES.end(),
        ));
    Some(EscalierCalcule {
        hauteur,
        contremarches: n,
        contremarche: hauteur / n as f64,
        // Un escalier droit a n − 1 marches sur sa profondeur.
        giron: (forme == FormeDEscalier::Droit).then(|| profondeur / (n - 1) as f64),
    })
}

/// La largeur d'une volée, en part de la largeur et de la profondeur de
/// l'escalier (`volees` côte à côte dans la largeur).
fn parts_de_volee((largeur, profondeur): (f64, f64), volees: f64) -> (f64, f64) {
    let volee = LARGEUR_DE_VOLEE
        .min(largeur / volees)
        .min(profondeur)
        .max(1.0);
    (volee / largeur.max(1.0), volee / profondeur.max(1.0))
}

/// Le nombre de marches de chaque volée : (première, seconde), sur `marches`
/// marches au total.
fn partager(marches: usize) -> (usize, usize) {
    let premiere = marches.div_ceil(2);
    (premiere, marches - premiere)
}

/// Un point d'un colimaçon : à `r` (part du rayon) et `angle` (radians) du centre.
fn point_du_colimacon((largeur, profondeur): (f64, f64), r: f64, angle: f64) -> (f64, f64) {
    let rayon = largeur.min(profondeur) / 2.0;
    (
        0.5 + r * rayon / largeur.max(1.0) * angle.cos(),
        0.5 + r * rayon / profondeur.max(1.0) * angle.sin(),
    )
}

/// L'angle de la k-ième marche d'un colimaçon de `marches` marches : elles
/// font 330° en tout, à partir du bas du cercle, dans le sens horaire.
fn angle_du_colimacon(k: usize, marches: usize) -> f64 {
    (90.0 - 330.0 * k as f64 / marches.max(1) as f64).to_radians()
}

/// Le symbole d'un escalier de `contremarches` contremarches : ses marches,
/// ses paliers et la flèche qui monte.
pub fn symbole_d_escalier(
    forme: FormeDEscalier,
    dimensions: (f64, f64),
    contremarches: usize,
) -> Vec<Trait> {
    use Trait::Ligne;
    let n = contremarches.max(2);
    let fleche = |traits: &mut Vec<Trait>, points: &[(f64, f64)]| {
        for paire in points.windows(2) {
            traits.push(Ligne(paire[0], paire[1]));
        }
        // La pointe, dans le sens du dernier trait.
        if let [.., a, b] = points {
            let (dx, dy) = (b.0 - a.0, b.1 - a.1);
            let longueur = dx.hypot(dy).max(1e-9);
            let (ux, uy) = (dx / longueur * 0.07, dy / longueur * 0.07);
            traits.push(Ligne(*b, (b.0 - ux - uy * 0.8, b.1 - uy + ux * 0.8)));
            traits.push(Ligne(*b, (b.0 - ux + uy * 0.8, b.1 - uy - ux * 0.8)));
        }
    };
    let mut traits = Vec::new();
    match forme {
        FormeDEscalier::Droit => {
            for k in 1..n {
                let v = k as f64 / n as f64;
                traits.push(Ligne((0.0, v), (1.0, v)));
            }
            fleche(&mut traits, &[(0.5, 0.94), (0.5, 0.06)]);
        }
        FormeDEscalier::EnL | FormeDEscalier::Tournant => {
            let (su, sv) = parts_de_volee(dimensions, 1.0);
            let dansantes = if forme == FormeDEscalier::Tournant {
                2
            } else {
                1
            };
            let (t1, t2) = partager(n.saturating_sub(1 + dansantes).max(2));
            for k in 1..t1 {
                let v = 1.0 - (1.0 - sv) * k as f64 / t1 as f64;
                traits.push(Ligne((0.0, v), (su, v)));
            }
            for k in 1..t2 {
                let u = su + (1.0 - su) * k as f64 / t2 as f64;
                traits.push(Ligne((u, 0.0), (u, sv)));
            }
            // Le coin : un palier, ou deux marches dansantes.
            traits.push(Ligne((0.0, sv), (su, sv)));
            traits.push(Ligne((su, 0.0), (su, sv)));
            if forme == FormeDEscalier::Tournant {
                traits.push(Ligne((0.0, 0.0), (su, sv)));
            }
            traits.push(Ligne((su, sv), (su, 1.0)));
            traits.push(Ligne((su, sv), (1.0, sv)));
            fleche(
                &mut traits,
                &[(su / 2.0, 0.95), (su / 2.0, sv / 2.0), (0.96, sv / 2.0)],
            );
        }
        FormeDEscalier::EnU => {
            let (su, sv) = parts_de_volee(dimensions, 2.0);
            let (t1, t2) = partager(n.saturating_sub(2).max(2));
            for k in 1..t1 {
                let v = 1.0 - (1.0 - sv) * k as f64 / t1 as f64;
                traits.push(Ligne((0.0, v), (su, v)));
            }
            for k in 1..t2 {
                let v = sv + (1.0 - sv) * k as f64 / t2 as f64;
                traits.push(Ligne((1.0 - su, v), (1.0, v)));
            }
            traits.push(Ligne((0.0, sv), (su, sv)));
            traits.push(Ligne((1.0 - su, sv), (1.0, sv)));
            traits.push(Ligne((su, sv), (su, 1.0)));
            traits.push(Ligne((1.0 - su, sv), (1.0 - su, 1.0)));
            fleche(
                &mut traits,
                &[
                    (su / 2.0, 0.95),
                    (su / 2.0, sv / 2.0),
                    (1.0 - su / 2.0, sv / 2.0),
                    (1.0 - su / 2.0, 0.95),
                ],
            );
        }
        FormeDEscalier::Colimacon => {
            let marches = n - 1;
            traits.push(Trait::Cercle((0.5, 0.5), 0.5));
            traits.push(Trait::Cercle((0.5, 0.5), 0.08));
            for k in 0..=marches {
                let angle = angle_du_colimacon(k, marches);
                traits.push(Ligne(
                    point_du_colimacon(dimensions, 0.16, angle),
                    point_du_colimacon(dimensions, 1.0, angle),
                ));
            }
            // La flèche suit le cercle, à mi-rayon.
            let arc: Vec<(f64, f64)> = (0..=10)
                .map(|k| {
                    let angle = angle_du_colimacon(k * marches / 10, marches);
                    point_du_colimacon(dimensions, 0.58, angle)
                })
                .collect();
            fleche(&mut traits, &arc);
        }
    }
    traits
}

/// Une marche (ou un palier) en 3D : ses quatre coins sur le plan, son bas et
/// son haut (pouces), et vrai si sa face avant est cachée par la suivante.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MarcheEn3d {
    pub coins: [(f64, f64); 4],
    pub bas: f64,
    pub haut: f64,
    pub sans_l_avant: bool,
}

impl Objet {
    /// Les marches et les paliers d'un escalier, en blocs, du bas vers le haut.
    pub fn marches_en_3d(
        &self,
        dimensions: (f64, f64),
        calcul: &EscalierCalcule,
    ) -> Vec<MarcheEn3d> {
        let n = calcul.contremarches.max(2);
        let r = calcul.hauteur / n as f64;
        let rectangle = |(u0, u1): (f64, f64), (v0, v1): (f64, f64)| {
            [(u0, v0), (u1, v0), (u1, v1), (u0, v1)].map(|c| self.vers_le_plan(c, dimensions))
        };
        let quad = |points: [(f64, f64); 4]| points.map(|c| self.vers_le_plan(c, dimensions));
        let marche = |coins, niveau: usize, sans_l_avant| MarcheEn3d {
            coins,
            bas: 0.0,
            haut: r * niveau as f64,
            sans_l_avant,
        };
        let mut marches = Vec::new();
        match self.forme {
            FormeDEscalier::Droit => {
                for k in 0..n {
                    let v0 = 1.0 - (k + 1) as f64 / n as f64;
                    let v1 = 1.0 - k as f64 / n as f64;
                    marches.push(marche(rectangle((0.0, 1.0), (v0, v1)), k + 1, k + 1 < n));
                }
            }
            FormeDEscalier::EnL | FormeDEscalier::Tournant => {
                let (su, sv) = parts_de_volee(dimensions, 1.0);
                let tournant = self.forme == FormeDEscalier::Tournant;
                let dansantes = if tournant { 2 } else { 1 };
                let (t1, t2) = partager(n.saturating_sub(1 + dansantes).max(2));
                for k in 0..t1 {
                    let v0 = 1.0 - (1.0 - sv) * (k + 1) as f64 / t1 as f64;
                    let v1 = 1.0 - (1.0 - sv) * k as f64 / t1 as f64;
                    marches.push(marche(rectangle((0.0, su), (v0, v1)), k + 1, false));
                }
                if tournant {
                    // Deux marches dansantes, de part et d'autre de la diagonale du
                    // coin (un triangle, avec un quatrième point sur la diagonale).
                    let milieu = (su / 2.0, sv / 2.0);
                    marches.push(marche(
                        quad([(0.0, sv), (0.0, 0.0), milieu, (su, sv)]),
                        t1 + 1,
                        false,
                    ));
                    marches.push(marche(
                        quad([(0.0, 0.0), (su, 0.0), (su, sv), milieu]),
                        t1 + 2,
                        false,
                    ));
                } else {
                    marches.push(marche(rectangle((0.0, su), (0.0, sv)), t1 + 1, false));
                }
                for k in 0..t2 {
                    let u0 = su + (1.0 - su) * k as f64 / t2 as f64;
                    let u1 = su + (1.0 - su) * (k + 1) as f64 / t2 as f64;
                    marches.push(marche(
                        rectangle((u0, u1), (0.0, sv)),
                        t1 + dansantes + k + 1,
                        false,
                    ));
                }
            }
            FormeDEscalier::EnU => {
                let (su, sv) = parts_de_volee(dimensions, 2.0);
                let (t1, t2) = partager(n.saturating_sub(2).max(2));
                for k in 0..t1 {
                    let v0 = 1.0 - (1.0 - sv) * (k + 1) as f64 / t1 as f64;
                    let v1 = 1.0 - (1.0 - sv) * k as f64 / t1 as f64;
                    marches.push(marche(rectangle((0.0, su), (v0, v1)), k + 1, false));
                }
                marches.push(marche(rectangle((0.0, 1.0), (0.0, sv)), t1 + 1, false));
                for k in 0..t2 {
                    let v0 = sv + (1.0 - sv) * k as f64 / t2 as f64;
                    let v1 = sv + (1.0 - sv) * (k + 1) as f64 / t2 as f64;
                    marches.push(marche(
                        rectangle((1.0 - su, 1.0), (v0, v1)),
                        t1 + 2 + k,
                        false,
                    ));
                }
            }
            FormeDEscalier::Colimacon => {
                let tours = n - 1;
                for k in 0..tours {
                    let (a0, a1) = (
                        angle_du_colimacon(k, tours),
                        angle_du_colimacon(k + 1, tours),
                    );
                    let coins = quad([
                        point_du_colimacon(dimensions, 0.08, a0),
                        point_du_colimacon(dimensions, 1.0, a0),
                        point_du_colimacon(dimensions, 1.0, a1),
                        point_du_colimacon(dimensions, 0.08, a1),
                    ]);
                    marches.push(marche(coins, k + 1, false));
                }
                // Le poteau, jusqu'en haut.
                let (pu, pv) = parts_de_volee(dimensions, 1.0);
                let poteau = (pu.min(pv) * 0.12).max(0.01);
                marches.push(MarcheEn3d {
                    coins: rectangle((0.5 - poteau, 0.5 + poteau), (0.5 - poteau, 0.5 + poteau)),
                    bas: 0.0,
                    haut: calcul.hauteur,
                    sans_l_avant: false,
                });
            }
        }
        marches
    }
}

// ----------------------------------------------------------------------
// Collisions (R-98) : un objet ne chevauche ni un autre objet, ni une cloison,
// ni les murs de sa pièce.
// ----------------------------------------------------------------------

/// Tolérance des collisions, en pouces : deux formes qui se touchent ne se
/// chevauchent pas.
const TOLERANCE: f64 = 1e-6;

/// Vrai si deux polygones convexes (en pouces, sur le plan) se chevauchent,
/// au sens strict : se toucher par un côté ou un coin n'est pas chevaucher.
/// Un segment (deux points) compte comme un polygone aplati. Méthode des axes
/// séparateurs.
pub fn se_chevauchent(a: &[(f64, f64)], b: &[(f64, f64)]) -> bool {
    let axes = |forme: &[(f64, f64)]| -> Vec<(f64, f64)> {
        (0..forme.len())
            .filter_map(|k| {
                let (p, q) = (forme[k], forme[(k + 1) % forme.len()]);
                let (dx, dy) = (q.0 - p.0, q.1 - p.1);
                let longueur = dx.hypot(dy);
                (longueur > TOLERANCE).then(|| (-dy / longueur, dx / longueur))
            })
            .collect()
    };
    let projeter = |forme: &[(f64, f64)], axe: (f64, f64)| {
        forme.iter().fold((f64::MAX, f64::MIN), |(bas, haut), p| {
            let x = p.0 * axe.0 + p.1 * axe.1;
            (bas.min(x), haut.max(x))
        })
    };
    axes(a).into_iter().chain(axes(b)).all(|axe| {
        let (a0, a1) = projeter(a, axe);
        let (b0, b1) = projeter(b, axe);
        // Un segment n'a pas d'épaisseur sur sa normale : il traverse l'autre
        // forme s'il tombe strictement dedans.
        if a1 - a0 <= TOLERANCE {
            b0 < a0 - TOLERANCE && a1 + TOLERANCE < b1
        } else if b1 - b0 <= TOLERANCE {
            a0 < b0 - TOLERANCE && b1 + TOLERANCE < a1
        } else {
            a1.min(b1) - a0.max(b0) > TOLERANCE
        }
    })
}

/// Vrai si le point est dans le polygone (règle des croisements).
fn dans_le_polygone((x, y): (f64, f64), polygone: &[(f64, f64)]) -> bool {
    let mut dedans = false;
    for k in 0..polygone.len() {
        let (a, b) = (polygone[k], polygone[(k + 1) % polygone.len()]);
        if (a.1 > y) != (b.1 > y) && x < a.0 + (y - a.1) * (b.0 - a.0) / (b.1 - a.1) {
            dedans = !dedans;
        }
    }
    dedans
}

/// Ce qui encombre une pièce pour un objet qu'on déplace : les autres objets
/// et les morceaux de cloison (des quadrilatères convexes), et le contour
/// intérieur des murs, s'il est connu.
pub struct Encombrement<'a> {
    pub obstacles: Vec<[(f64, f64); 4]>,
    pub piece: Option<&'a [(f64, f64)]>,
}

impl Encombrement<'_> {
    /// Vrai si un objet de ces quatre coins est à sa place : dans la pièce, et
    /// sans chevaucher d'obstacle (il peut les toucher).
    pub fn libre(&self, coins: &[(f64, f64); 4]) -> bool {
        if let Some(piece) = self.piece {
            // Aucun mur ne traverse l'objet, et l'objet est du bon côté.
            let traverse = (0..piece.len())
                .any(|k| se_chevauchent(&[piece[k], piece[(k + 1) % piece.len()]], coins));
            let centre = (
                coins.iter().map(|c| c.0).sum::<f64>() / 4.0,
                coins.iter().map(|c| c.1).sum::<f64>() / 4.0,
            );
            if traverse || !dans_le_polygone(centre, piece) {
                return false;
            }
        }
        !self.obstacles.iter().any(|o| se_chevauchent(o, coins))
    }
}

/// Les coins de l'objet s'il avait ce centre.
fn coins_au_centre(objet: &Objet, dimensions: (f64, f64), centre: (f64, f64)) -> [(f64, f64); 4] {
    let mut deplace = objet.clone();
    deplace.centre = centre;
    deplace.coins(dimensions)
}

/// Où l'objet peut aller, au plus près du centre `voulu` : il avance pas à pas
/// (`pas`, en pouces) et s'arrête contre ce qui l'encombre, puis glisse le long
/// (le mouvement se fait en x puis en y, ou en y puis en x : le plus proche du
/// but l'emporte). Un objet qui chevauche déjà quelque chose (un plan d'avant
/// les collisions) va où l'on veut, pour qu'on puisse le dégager.
pub fn deplacer_sans_chevaucher(
    objet: &Objet,
    dimensions: (f64, f64),
    voulu: (f64, f64),
    encombrement: &Encombrement<'_>,
    pas: f64,
) -> (f64, f64) {
    let depart = objet.centre;
    let libre =
        |centre: (f64, f64)| encombrement.libre(&coins_au_centre(objet, dimensions, centre));
    // Un objet qui chevauche déjà quelque chose va où l'on veut; un chemin
    // direct libre se prend tel quel.
    if !libre(depart) || trajet_libre(depart, voulu, pas, &libre) {
        return voulu;
    }
    // Avancer en ligne droite jusqu'au dernier pas libre.
    let avancer = |de: (f64, f64), vers: (f64, f64)| -> (f64, f64) {
        let (dx, dy) = (vers.0 - de.0, vers.1 - de.1);
        let n = nombre_de_pas(dx, dy, pas);
        let mut dernier = de;
        for k in 1..=n {
            let t = k as f64 / n as f64;
            let point = (de.0 + dx * t, de.1 + dy * t);
            if !libre(point) {
                break;
            }
            dernier = point;
        }
        dernier
    };
    // En x puis en y, ou en y puis en x : l'objet glisse le long de l'obstacle.
    let x_d_abord = avancer(depart, (voulu.0, depart.1));
    let par_x = avancer(x_d_abord, (x_d_abord.0, voulu.1));
    let y_d_abord = avancer(depart, (depart.0, voulu.1));
    let par_y = avancer(y_d_abord, (voulu.0, y_d_abord.1));
    let distance = |p: (f64, f64)| (p.0 - voulu.0).hypot(p.1 - voulu.1);
    if distance(par_x) <= distance(par_y) {
        par_x
    } else {
        par_y
    }
}

/// Le nombre de pas d'au plus `pas` pouces pour aller de (0, 0) à (dx, dy)
/// (sur la grille, un déplacement d'un nombre entier de pas en garde le compte).
fn nombre_de_pas(dx: f64, dy: f64, pas: f64) -> usize {
    ((dx.abs().max(dy.abs()) / pas) - 1e-9).ceil().max(1.0) as usize
}

/// Vrai si l'objet passe de `de` à `vers` sans rien traverser (pas à pas).
fn trajet_libre(
    de: (f64, f64),
    vers: (f64, f64),
    pas: f64,
    libre: &dyn Fn((f64, f64)) -> bool,
) -> bool {
    let (dx, dy) = (vers.0 - de.0, vers.1 - de.1);
    let n = nombre_de_pas(dx, dy, pas);
    (1..=n).all(|k| {
        let t = k as f64 / n as f64;
        libre((de.0 + dx * t, de.1 + dy * t))
    })
}

/// Le centre libre le plus proche de celui de l'objet, à `rayon` pouces au
/// plus, sur la grille de `pas` pouces; rien s'il n'y en a pas. L'objet déjà
/// libre reste où il est.
pub fn position_libre_proche(
    objet: &Objet,
    dimensions: (f64, f64),
    encombrement: &Encombrement<'_>,
    pas: f64,
    rayon: f64,
) -> Option<(f64, f64)> {
    let (cx, cy) = objet.centre;
    let libre =
        |centre: (f64, f64)| encombrement.libre(&coins_au_centre(objet, dimensions, centre));
    if libre((cx, cy)) {
        return Some((cx, cy));
    }
    let anneaux = (rayon / pas).ceil() as i64;
    for anneau in 1..=anneaux {
        // Les points de l'anneau, du plus proche au plus loin du centre.
        let mut points: Vec<(f64, f64)> = Vec::new();
        for i in -anneau..=anneau {
            for j in -anneau..=anneau {
                if i.abs().max(j.abs()) == anneau {
                    points.push((cx + i as f64 * pas, cy + j as f64 * pas));
                }
            }
        }
        points.sort_by(|a, b| {
            let da = (a.0 - cx).hypot(a.1 - cy);
            let db = (b.0 - cx).hypot(b.1 - cy);
            da.total_cmp(&db)
        });
        if let Some(point) = points.into_iter().find(|p| libre(*p)) {
            return Some(point);
        }
    }
    None
}

/// Le dégagement d'un côté d'un objet (évolution 23) : la distance libre
/// devant ce côté, jusqu'au premier mur, cloison ou objet qu'il frapperait en
/// avançant tout droit; `de` est sur le côté, `a` sur l'obstacle (pouces).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Degagement {
    pub de: (f64, f64),
    pub a: (f64, f64),
    pub distance: f64,
}

/// Les dégagements des quatre côtés d'un objet, jusqu'aux `obstacles` (des
/// segments : les faces des murs et des cloisons, les côtés des autres objets)
/// à `portee` pouces au plus; un côté collé à un obstacle, ou sans obstacle à
/// portée, n'en a pas.
pub fn degagements(
    objet: &Objet,
    dimensions: (f64, f64),
    obstacles: &[Face],
    portee: f64,
) -> Vec<Degagement> {
    let coins = objet.coins(dimensions);
    let centre = objet.centre;
    let mut resultat = Vec::new();
    for k in 0..4 {
        let (a, b) = (coins[k], coins[(k + 1) % 4]);
        let longueur = (b.0 - a.0).hypot(b.1 - a.1);
        if longueur < 1e-9 {
            continue;
        }
        let s = ((b.0 - a.0) / longueur, (b.1 - a.1) / longueur);
        // La normale qui s'éloigne du centre de l'objet.
        let mut n = (-s.1, s.0);
        if (a.0 - centre.0) * n.0 + (a.1 - centre.1) * n.1 < 0.0 {
            n = (-n.0, -n.1);
        }
        // Chaque obstacle, ramené au repère du côté (le long, devant), et
        // coupé à la bande que le côté balaie en avançant.
        let local = |p: (f64, f64)| {
            let (dx, dy) = (p.0 - a.0, p.1 - a.1);
            (dx * s.0 + dy * s.1, dx * n.0 + dy * n.1)
        };
        let mut plus_pres: Option<(f64, f64)> = None;
        for &(p, q) in obstacles {
            let (p, q) = (local(p), local(q));
            // La bande est un peu plus étroite que le côté : un mur qui ne
            // touche que son coin (celui contre lequel l'objet est collé) ne
            // compte pas.
            let (bord, fin_de_bande) = (
                0.01_f64.min(longueur / 4.0),
                longueur - 0.01_f64.min(longueur / 4.0),
            );
            let (mut t0, mut t1) = (0.0_f64, 1.0_f64);
            let d = (q.0 - p.0, q.1 - p.1);
            if d.0.abs() < 1e-12 {
                if p.0 < bord || p.0 > fin_de_bande {
                    continue;
                }
            } else {
                let (ta, tb) = ((bord - p.0) / d.0, (fin_de_bande - p.0) / d.0);
                t0 = t0.max(ta.min(tb));
                t1 = t1.min(ta.max(tb));
                if t0 > t1 {
                    continue;
                }
            }
            let (e0, e1) = (
                (p.0 + d.0 * t0, p.1 + d.1 * t0),
                (p.0 + d.0 * t1, p.1 + d.1 * t1),
            );
            // Le point de l'obstacle le plus près, devant le côté; un obstacle
            // qui traverse le côté n'en laisse aucun.
            let candidat = if e0.1 < -1e-6 && e1.1 < -1e-6 {
                continue;
            } else if e0.1 < -1e-6 || e1.1 < -1e-6 {
                (0.0, 0.0)
            } else if e0.1 <= e1.1 {
                (e0.0, e0.1)
            } else {
                (e1.0, e1.1)
            };
            if plus_pres.is_none_or(|(_, distance)| candidat.1 < distance) {
                plus_pres = Some(candidat);
            }
        }
        if let Some((le_long, distance)) = plus_pres
            && distance > 0.25
            && distance <= portee
        {
            let de = (a.0 + s.0 * le_long, a.1 + s.1 * le_long);
            resultat.push(Degagement {
                de,
                a: (de.0 + n.0 * distance, de.1 + n.1 * distance),
                distance,
            });
        }
    }
    resultat
}

/// Une face de mur ou de cloison : de `a` vers `b`, son côté libre à sa droite.
pub type Face = ((f64, f64), (f64, f64));

/// Les faces des murs et des cloisons contre lesquelles un objet s'appuie :
/// chacune de `a` vers `b`, son côté libre à sa droite à l'écran (la normale
/// (−y, x), y vers le bas). Le côté libre d'une pièce est son dedans; celui
/// d'un morceau de cloison, son dehors.
pub fn faces_d_appui(piece: Option<&[(f64, f64)]>, cloisons: &[[(f64, f64); 4]]) -> Vec<Face> {
    let mut faces = Vec::new();
    let mut ajouter = |sommets: &[(f64, f64)], dedans_libre: bool| {
        let aire = aire_signee(sommets);
        if aire.abs() < 1e-9 {
            return;
        }
        // Une aire positive (le sens des aiguilles d'une montre à l'écran)
        // met le dedans à droite de chaque côté.
        let dans_l_ordre = (aire > 0.0) == dedans_libre;
        for k in 0..sommets.len() {
            let (a, b) = (sommets[k], sommets[(k + 1) % sommets.len()]);
            faces.push(if dans_l_ordre { (a, b) } else { (b, a) });
        }
    };
    if let Some(piece) = piece {
        ajouter(piece, true);
    }
    for morceau in cloisons {
        ajouter(morceau, false);
    }
    faces
}

/// L'aire signée d'un polygone (la formule du lacet) : positive quand il
/// tourne dans le sens des aiguilles d'une montre à l'écran (y vers le bas).
fn aire_signee(sommets: &[(f64, f64)]) -> f64 {
    let n = sommets.len();
    (0..n)
        .map(|k| {
            let (a, b) = (sommets[k], sommets[(k + 1) % n]);
            a.0 * b.1 - b.0 * a.1
        })
        .sum::<f64>()
        / 2.0
}

/// Pose `objet` contre la face la plus proche de son centre, à `portee`
/// pouces au plus (`faces_d_appui`) : il pivote pour lui tourner le dos
/// (v = 0) et s'y appuie, le long de la face au pas `pas`, sans en dépasser
/// les bouts s'il y tient. Retourne vrai s'il s'est posé contre un mur.
pub fn contre_le_mur(
    objet: &mut Objet,
    dimensions: (f64, f64),
    faces: &[Face],
    portee: f64,
    pas: f64,
) -> bool {
    let p = objet.centre;
    let mut plus_proche: Option<(f64, usize)> = None;
    for (rang, &(a, b)) in faces.iter().enumerate() {
        let longueur = (b.0 - a.0).hypot(b.1 - a.1);
        if longueur < 1e-9 {
            continue;
        }
        let d = ((b.0 - a.0) / longueur, (b.1 - a.1) / longueur);
        let t = ((p.0 - a.0) * d.0 + (p.1 - a.1) * d.1).clamp(0.0, longueur);
        let distance = (p.0 - a.0 - d.0 * t).hypot(p.1 - a.1 - d.1 * t);
        if distance <= portee && plus_proche.is_none_or(|(meilleure, _)| distance < meilleure) {
            plus_proche = Some((distance, rang));
        }
    }
    let Some((_, rang)) = plus_proche else {
        return false;
    };
    let (a, b) = faces[rang];
    let longueur = (b.0 - a.0).hypot(b.1 - a.1);
    let d = ((b.0 - a.0) / longueur, (b.1 - a.1) / longueur);
    let libre = (-d.1, d.0);
    let (largeur, profondeur) = dimensions;
    let t = (p.0 - a.0) * d.0 + (p.1 - a.1) * d.1;
    let t = if longueur >= largeur {
        ((t / pas).round() * pas).clamp(largeur / 2.0, longueur - largeur / 2.0)
    } else {
        longueur / 2.0
    };
    // Son dos (0, −1), tourné de θ, regarde (sin θ, −cos θ) : vers le mur, à
    // l'opposé du côté libre.
    let angle = (-libre.0).atan2(libre.1).to_degrees();
    let angle = ((angle * 1e6).round() / 1e6).rem_euclid(360.0);
    objet.rotation = 0;
    objet.angle = if angle.abs() < 1e-9 { 0.0 } else { angle };
    objet.centre = (
        a.0 + d.0 * t + libre.0 * profondeur / 2.0,
        a.1 + d.1 * t + libre.1 * profondeur / 2.0,
    );
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn un_objet_pivote_d_un_quart_de_tour() {
        let mut lit = Objet::nouveau(GenreDObjet::Lit, (100.0, 50.0));
        let (largeur, profondeur, _) = GenreDObjet::Lit.dimensions();
        let dimensions = (largeur, profondeur);
        // Sans rotation : 60 po de large (x), 80 po de profond (y).
        assert_eq!(lit.coins(dimensions)[0], (70.0, 10.0));
        assert_eq!(lit.coins(dimensions)[2], (130.0, 90.0));
        assert!(lit.contient((100.0, 85.0), dimensions));
        assert!(!lit.contient((100.0, 95.0), dimensions));
        // Un quart de tour : 80 po en x, 60 po en y.
        lit.rotation = 1;
        let xs: Vec<f64> = lit.coins(dimensions).iter().map(|c| c.0).collect();
        let ys: Vec<f64> = lit.coins(dimensions).iter().map(|c| c.1).collect();
        let etendue = |v: &[f64]| {
            v.iter().cloned().fold(f64::MIN, f64::max) - v.iter().cloned().fold(f64::MAX, f64::min)
        };
        assert!((etendue(&xs) - 80.0).abs() < 1e-9 && (etendue(&ys) - 60.0).abs() < 1e-9);
        assert!(lit.contient((135.0, 50.0), dimensions));
        assert!(!lit.contient((100.0, 85.0), dimensions));
    }

    #[test]
    fn les_parties_d_un_meuble_ne_se_chevauchent_pas() {
        // Deux parties se séparent en u, en v ou en hauteur : la 3D les peint
        // dans le bon ordre.
        let disjoints = |a: (f64, f64), b: (f64, f64)| a.1 <= b.0 + 1e-9 || b.1 <= a.0 + 1e-9;
        for genre in GenreDObjet::TOUS {
            let parties = genre.volumes();
            assert_eq!(
                parties.is_empty(),
                genre == GenreDObjet::Escalier,
                "{genre:?}"
            );
            for (i, a) in parties.iter().enumerate() {
                assert!(
                    a.u.0 < a.u.1 && a.v.0 < a.v.1 && a.bas < a.haut,
                    "{genre:?} {i}"
                );
                for b in &parties[i + 1..] {
                    assert!(
                        disjoints(a.u, b.u)
                            || disjoints(a.v, b.v)
                            || disjoints((a.bas, a.haut), (b.bas, b.haut)),
                        "{genre:?} : {a:?} et {b:?}"
                    );
                }
            }
        }
        // Une table : son plateau, quatre pattes, et six chaises de deux parties.
        assert_eq!(GenreDObjet::Table.volumes().len(), 1 + 4 + 12);
    }

    #[test]
    fn un_objet_pivote_au_15_degres() {
        let mut table = Objet::nouveau(GenreDObjet::Table, (100.0, 100.0));
        let dimensions = (36.0, 60.0);
        table.pivoter(15.0);
        table.pivoter(15.0);
        assert!((table.angle_total() - 30.0).abs() < 1e-9);
        // Le coin (0, 0) tourne de 30° autour du centre.
        let (dx, dy) = (-18.0_f64, -30.0_f64);
        let (sinus, cosinus) = 30.0_f64.to_radians().sin_cos();
        let attendu = (
            100.0 + dx * cosinus - dy * sinus,
            100.0 + dx * sinus + dy * cosinus,
        );
        let coin = table.coins(dimensions)[0];
        assert!((coin.0 - attendu.0).abs() < 1e-9 && (coin.1 - attendu.1).abs() < 1e-9);
        assert!(table.contient((100.0, 100.0), dimensions));
        // Six pas de 15° font un quart de tour exact.
        let mut lit = Objet::nouveau(GenreDObjet::Lit, (0.0, 0.0));
        for _ in 0..6 {
            lit.pivoter(15.0);
        }
        assert_eq!(lit.coins((60.0, 80.0))[0], (40.0, -30.0));
        // −15° revient en arrière.
        lit.pivoter(-15.0);
        assert!((lit.angle_total() - 75.0).abs() < 1e-9);
    }

    #[test]
    fn le_symbole_d_un_escalier() {
        // 10 pi de long : 12 marches, donc 11 lignes de marche et la flèche.
        let traits = GenreDObjet::Escalier.symbole(120.0);
        assert_eq!(traits.len(), 11 + 3);
        let escalier = Objet::nouveau(GenreDObjet::Escalier, (0.0, 0.0));
        // Le contour (4 traits) et le symbole.
        assert_eq!(
            escalier.traits_sur_le_plan((36.0, 120.0), None).len(),
            4 + 14
        );
        // Chaque genre a des dimensions et un symbole.
        for genre in GenreDObjet::TOUS {
            let (largeur, profondeur, hauteur) = genre.dimensions();
            assert!(largeur > 0.0 && profondeur > 0.0 && hauteur >= 0.0);
            assert!(!genre.symbole(profondeur).is_empty());
        }
    }

    /// Une pièce de 20 pi sur 12 pi (en pouces, y vers le bas).
    const PIECE: [(f64, f64); 4] = [(0.0, 0.0), (240.0, 0.0), (240.0, 144.0), (0.0, 144.0)];

    fn table_en(centre: (f64, f64)) -> (Objet, (f64, f64)) {
        (Objet::nouveau(GenreDObjet::Table, centre), (36.0, 60.0))
    }

    #[test]
    fn se_toucher_n_est_pas_chevaucher() {
        let a = [(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)];
        let colle = [(10.0, 0.0), (20.0, 0.0), (20.0, 10.0), (10.0, 10.0)];
        let mord = [(9.5, 2.0), (20.0, 2.0), (20.0, 8.0), (9.5, 8.0)];
        assert!(!se_chevauchent(&a, &colle));
        assert!(se_chevauchent(&a, &mord));
        // Un carré tourné de 45° qui ne touche que la pointe… ou qui mord.
        let losange = |dx: f64| {
            [
                (15.0 + dx, 5.0),
                (20.0 + dx, 0.0),
                (25.0 + dx, 5.0),
                (20.0 + dx, 10.0),
            ]
        };
        assert!(!se_chevauchent(&a, &losange(-5.0)));
        assert!(se_chevauchent(&a, &losange(-6.0)));
        // Un segment le long d'un côté ne traverse pas; en travers, il traverse.
        assert!(!se_chevauchent(&[(0.0, 10.0), (30.0, 10.0)], &a));
        assert!(se_chevauchent(&[(5.0, -5.0), (5.0, 15.0)], &a));
    }

    #[test]
    fn un_meuble_s_arrete_contre_un_autre_et_glisse_le_long() {
        // Une table de 36 po sur 60 po contre un lit posé à droite.
        let lit = Objet::nouveau(GenreDObjet::Lit, (150.0, 72.0));
        let encombrement = Encombrement {
            obstacles: vec![lit.coins((60.0, 80.0))],
            piece: Some(&PIECE),
        };
        let (table, dimensions) = table_en((40.0, 72.0));
        // Le lit occupe x de 120 à 180 : la table (demi-largeur 18) s'arrête à x = 102.
        let arrivee =
            deplacer_sans_chevaucher(&table, dimensions, (150.0, 72.0), &encombrement, 1.0);
        assert_eq!(arrivee, (102.0, 72.0));
        // En visant en biais, elle avance en y le long du lit.
        let arrivee =
            deplacer_sans_chevaucher(&table, dimensions, (130.0, 90.0), &encombrement, 1.0);
        assert_eq!(arrivee, (102.0, 90.0));
        // Un chemin libre se prend tel quel.
        let arrivee =
            deplacer_sans_chevaucher(&table, dimensions, (60.0, 80.0), &encombrement, 1.0);
        assert_eq!(arrivee, (60.0, 80.0));
    }

    #[test]
    fn un_meuble_reste_dans_la_piece_et_ne_traverse_pas_une_cloison() {
        // Une cloison de 5 po d'épaisseur, de haut en bas, à x de 118 à 123.
        let cloison = [(118.0, 0.0), (123.0, 0.0), (123.0, 144.0), (118.0, 144.0)];
        let encombrement = Encombrement {
            obstacles: vec![cloison],
            piece: Some(&PIECE),
        };
        let (table, dimensions) = table_en((40.0, 72.0));
        // Un geste rapide vers l'autre côté s'arrête contre la cloison, sans la sauter.
        let arrivee =
            deplacer_sans_chevaucher(&table, dimensions, (200.0, 72.0), &encombrement, 1.0);
        assert_eq!(arrivee, (100.0, 72.0));
        // Vers le mur du haut : la table (demi-profondeur 30) s'arrête à y = 30.
        let arrivee =
            deplacer_sans_chevaucher(&table, dimensions, (40.0, -50.0), &encombrement, 1.0);
        assert_eq!(arrivee, (40.0, 30.0));
        // Dehors, elle n'est pas libre.
        assert!(!encombrement.libre(&table_en((-100.0, 72.0)).0.coins(dimensions)));
    }

    #[test]
    fn une_piece_en_l_retient_un_meuble_dans_son_coin_rentrant() {
        // Une pièce en L : le coin de (120, 72) à (240, 144) est dehors.
        let en_l = [
            (0.0, 0.0),
            (240.0, 0.0),
            (240.0, 72.0),
            (120.0, 72.0),
            (120.0, 144.0),
            (0.0, 144.0),
        ];
        let encombrement = Encombrement {
            obstacles: Vec::new(),
            piece: Some(&en_l),
        };
        let (table, dimensions) = table_en((60.0, 100.0));
        // Vers la droite, le mur du coin rentrant l'arrête (x de 120 moins 18).
        let arrivee =
            deplacer_sans_chevaucher(&table, dimensions, (200.0, 100.0), &encombrement, 1.0);
        assert_eq!(arrivee, (102.0, 100.0));
        // À cheval sur le coin rentrant : pas libre, même si ses coins sont dedans.
        let travers = Objet::nouveau(GenreDObjet::Sofa, (130.0, 60.0));
        assert!(!encombrement.libre(&travers.coins((84.0, 36.0))));
    }

    #[test]
    fn un_meuble_qui_chevauche_deja_se_degage_librement() {
        let lit = Objet::nouveau(GenreDObjet::Lit, (60.0, 72.0));
        let encombrement = Encombrement {
            obstacles: vec![lit.coins((60.0, 80.0))],
            piece: Some(&PIECE),
        };
        // Une table posée sur le lit (un plan d'avant les collisions).
        let (table, dimensions) = table_en((60.0, 72.0));
        assert!(!encombrement.libre(&table.coins(dimensions)));
        let arrivee =
            deplacer_sans_chevaucher(&table, dimensions, (65.0, 72.0), &encombrement, 1.0);
        assert_eq!(arrivee, (65.0, 72.0));
        // La place libre la plus proche est juste à côté du lit.
        let proche = position_libre_proche(&table, dimensions, &encombrement, 1.0, 120.0).unwrap();
        assert!(encombrement.libre(&table_en(proche).0.coins(dimensions)));
        assert!(
            (proche.0 - 60.0).abs() + (proche.1 - 72.0).abs() <= 50.0,
            "{proche:?}"
        );
        // Sans place assez proche : rien.
        assert!(position_libre_proche(&table, dimensions, &encombrement, 1.0, 3.0).is_none());
    }

    #[test]
    fn le_calcul_d_un_escalier() {
        // 8 pi 10 po à monter : 15 contremarches de 7 1/15 po (106 / 15).
        let droit = calculer_l_escalier(FormeDEscalier::Droit, (36.0, 140.0), 106.0, None).unwrap();
        assert_eq!(droit.contremarches, 15);
        assert!((droit.contremarche - 106.0 / 15.0).abs() < 1e-9);
        // 14 marches sur 140 po : un giron de 10 po.
        assert!((droit.giron.unwrap() - 10.0).abs() < 1e-9);
        // Une hauteur juste divisible ne prend pas de contremarche de trop.
        let juste = calculer_l_escalier(FormeDEscalier::Droit, (36.0, 120.0), 90.0, None).unwrap();
        assert_eq!(juste.contremarches, 12);
        // Le nombre inscrit l'emporte; un nombre déraisonnable est ignoré.
        let inscrit =
            calculer_l_escalier(FormeDEscalier::EnL, (72.0, 96.0), 96.0, Some(14)).unwrap();
        assert_eq!((inscrit.contremarches, inscrit.giron), (14, None));
        let ignore =
            calculer_l_escalier(FormeDEscalier::Droit, (36.0, 120.0), 96.0, Some(500)).unwrap();
        assert_eq!(ignore.contremarches, 13);
        assert!(calculer_l_escalier(FormeDEscalier::Droit, (36.0, 120.0), 0.0, None).is_none());
    }

    #[test]
    fn chaque_forme_d_escalier_a_son_symbole_et_ses_marches() {
        for forme in FormeDEscalier::TOUTES {
            let mut escalier = Objet::nouveau(GenreDObjet::Escalier, (100.0, 100.0));
            escalier.forme = forme;
            let dimensions = escalier.dimensions_de_depart();
            assert_eq!(dimensions, forme.dimensions());
            let calcul = calculer_l_escalier(forme, dimensions, 96.0, None).unwrap();
            let traits = escalier.traits_sur_le_plan(dimensions, Some(calcul.contremarches));
            assert!(traits.len() >= 8, "{forme:?} : {} traits", traits.len());
            let marches = escalier.marches_en_3d(dimensions, &calcul);
            // Chaque bloc a des côtés de longueur non nulle, et le plus haut
            // atteint presque la hauteur à monter.
            for marche in &marches {
                for k in 0..4 {
                    let (a, b) = (marche.coins[k], marche.coins[(k + 1) % 4]);
                    assert!((a.0 - b.0).hypot(a.1 - b.1) > 1e-6, "{forme:?}");
                }
            }
            let plus_haut = marches.iter().map(|m| m.haut).fold(0.0, f64::max);
            assert!(
                plus_haut >= 96.0 * 12.0 / 13.0 - 1e-9,
                "{forme:?} : {plus_haut}"
            );
            // Les marches restent dans l'emprise de l'escalier.
            for marche in &marches {
                for (x, y) in marche.coins {
                    assert!((x - 100.0).abs() <= dimensions.0 / 2.0 + 1e-6, "{forme:?}");
                    assert!((y - 100.0).abs() <= dimensions.1 / 2.0 + 1e-6, "{forme:?}");
                }
            }
        }
        // Le droit : une marche par contremarche; le colimaçon : son poteau en plus.
        let droit = Objet::nouveau(GenreDObjet::Escalier, (0.0, 0.0));
        let calcul = calculer_l_escalier(FormeDEscalier::Droit, (36.0, 120.0), 96.0, None).unwrap();
        assert_eq!(droit.marches_en_3d((36.0, 120.0), &calcul).len(), 13);
        let mut colimacon = droit.clone();
        colimacon.forme = FormeDEscalier::Colimacon;
        assert_eq!(colimacon.marches_en_3d((60.0, 60.0), &calcul).len(), 12 + 1);
    }

    #[test]
    fn les_familles_d_objets() {
        // Chaque genre est d'une famille, et paraît dans sa rangée.
        let mut vus = std::collections::HashSet::new();
        for genre in GenreDObjet::TOUS {
            assert!(vus.insert(genre), "{genre:?} en double");
            assert!(genre.famille().genres().contains(&genre), "{genre:?}");
        }
        for famille in Famille::TOUTES {
            let genres = famille.genres();
            for (i, genre) in genres.iter().enumerate() {
                assert!(!genres[i + 1..].contains(genre), "{famille:?} : {genre:?}");
            }
        }
        // Les meubles d'avant restent au calque MOBILIER.
        assert_eq!(GenreDObjet::Toilette.famille().calque(), "MOBILIER");
        assert_eq!(GenreDObjet::Prise.famille().calque(), "ELECTRICITE");
        assert_eq!(GenreDObjet::Coude90.famille().calque(), "PLOMBERIE");
        assert_eq!(GenreDObjet::Fournaise.famille().calque(), "VENTILATION");
        // Un symbole ne fait obstacle à rien; une armoire du bas, oui.
        assert!(!GenreDObjet::Prise.encombre() && GenreDObjet::ArmoireDuBas.encombre());
        // Le nom d'un meuble s'écrit sur le plan; un coude se suffit.
        assert!(GenreDObjet::Lit.nomme_sur_le_plan() && !GenreDObjet::Coude90.nomme_sur_le_plan());
    }

    #[test]
    fn les_symboles_des_metiers_sur_le_plan() {
        // Une prise : son symbole seul, sans contour.
        let prise = Objet::nouveau(GenreDObjet::Prise, (0.0, 0.0));
        assert_eq!(prise.traits_sur_le_plan((12.0, 12.0), None).len(), 3);
        // Un lavabo : son contour, sa vasque en vingt traits, son drain et
        // son robinet.
        let lavabo = Objet::nouveau(GenreDObjet::Lavabo, (0.0, 0.0));
        assert_eq!(
            lavabo.traits_sur_le_plan((20.0, 18.0), None).len(),
            4 + 20 + 1 + 1
        );
        // Une armoire du haut, au-dessus de la coupe : tout en tirets courts.
        let armoire = Objet::nouveau(GenreDObjet::ArmoireDuHaut, (0.0, 0.0));
        let traits = armoire.traits_sur_le_plan((30.0, 12.0), None);
        assert!(traits.len() > 8);
        for t in &traits {
            let TraitSurLePlan::Ligne(a, b) = *t else {
                panic!("un cercle dans une armoire");
            };
            assert!((b.0 - a.0).hypot(b.1 - a.1) <= TIRET + 1.0);
        }
        // Un côté de 30 po : six tirets de 3 1/3 po et cinq écarts de 2 po.
        let mut cote = Vec::new();
        en_tirets((0.0, 0.0), (30.0, 0.0), &mut cote);
        assert_eq!(cote.len(), 6);
        let TraitSurLePlan::Ligne(debut, fin) = cote[5] else {
            panic!("un cercle");
        };
        assert!((debut.0 - 80.0 / 3.0).abs() < 1e-9 && (fin.0 - 30.0).abs() < 1e-9);
    }

    /// Une pièce de 12 pi sur 10 pi (en pouces, y vers le bas).
    const SALLE: [(f64, f64); 4] = [(0.0, 0.0), (144.0, 0.0), (144.0, 120.0), (0.0, 120.0)];

    #[test]
    fn une_prise_se_pose_contre_le_mur_le_plus_proche() {
        let faces = faces_d_appui(Some(&SALLE), &[]);
        let poser = |centre: (f64, f64)| {
            let mut prise = Objet::nouveau(GenreDObjet::Prise, centre);
            let pose = contre_le_mur(&mut prise, (12.0, 12.0), &faces, 24.0, 1.0);
            (pose, prise.angle_total(), prise.centre)
        };
        // Chaque mur : le dos de la prise contre lui, son centre à 6 po.
        assert_eq!(poser((50.2, 10.0)), (true, 0.0, (50.0, 6.0)));
        assert_eq!(poser((140.0, 60.0)), (true, 90.0, (138.0, 60.0)));
        assert_eq!(poser((60.0, 118.0)), (true, 180.0, (60.0, 114.0)));
        assert_eq!(poser((3.0, 50.0)), (true, 270.0, (6.0, 50.0)));
        // Trop loin de tout mur : rien ne change.
        assert_eq!(poser((72.0, 60.0)), (false, 0.0, (72.0, 60.0)));
        // Près d'un coin, une armoire reste le long de son mur.
        let mut armoire = Objet::nouveau(GenreDObjet::ArmoireDuBas, (2.0, 3.0));
        assert!(contre_le_mur(&mut armoire, (30.0, 24.0), &faces, 36.0, 1.0));
        assert_eq!(
            (armoire.angle_total(), armoire.centre),
            (270.0, (12.0, 15.0))
        );
        // Contre une cloison de 5 po : la face du côté où l'on touche.
        let cloison = [(57.5, 0.0), (62.5, 0.0), (62.5, 60.0), (57.5, 60.0)];
        let faces = faces_d_appui(Some(&SALLE), &[cloison]);
        let mut prise = Objet::nouveau(GenreDObjet::Prise, (65.0, 30.0));
        assert!(contre_le_mur(&mut prise, (12.0, 12.0), &faces, 24.0, 1.0));
        assert_eq!((prise.angle_total(), prise.centre), (270.0, (68.5, 30.0)));
        let mut prise = Objet::nouveau(GenreDObjet::Prise, (55.0, 30.0));
        assert!(contre_le_mur(&mut prise, (12.0, 12.0), &faces, 24.0, 1.0));
        assert_eq!((prise.angle_total(), prise.centre), (90.0, (51.5, 30.0)));
    }

    #[test]
    fn les_degagements_d_une_table() {
        // Une table de 36 × 60 au milieu de la salle de 144 × 120.
        let faces = faces_d_appui(Some(&SALLE), &[]);
        let table = Objet::nouveau(GenreDObjet::Table, (72.0, 60.0));
        let mut distances: Vec<f64> = degagements(&table, (36.0, 60.0), &faces, 600.0)
            .iter()
            .map(|d| d.distance)
            .collect();
        distances.sort_by(f64::total_cmp);
        // 30 po en haut et en bas, 54 po à gauche et à droite.
        assert_eq!(distances, vec![30.0, 30.0, 54.0, 54.0]);
        // Un lit à sa droite : le dégagement de ce côté s'arrête au lit.
        let lit = Objet::nouveau(GenreDObjet::Lit, (120.0, 60.0));
        let mut obstacles = faces.clone();
        let coins = lit.coins((30.0, 80.0));
        obstacles.extend((0..4).map(|k| (coins[k], coins[(k + 1) % 4])));
        let a_droite = degagements(&table, (36.0, 60.0), &obstacles, 600.0)
            .into_iter()
            .find(|d| d.a.0 > d.de.0 + 1.0)
            .unwrap();
        assert!((a_droite.distance - (105.0 - 90.0)).abs() < 1e-9);
        assert!((a_droite.de.0 - 90.0).abs() < 1e-9 && (a_droite.a.0 - 105.0).abs() < 1e-9);
        // Collée au mur du haut : pas de dégagement de ce côté.
        let collee = Objet::nouveau(GenreDObjet::Table, (72.0, 30.0));
        assert_eq!(degagements(&collee, (36.0, 60.0), &faces, 600.0).len(), 3);
    }
}
