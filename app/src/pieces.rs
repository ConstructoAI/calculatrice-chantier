//! Pièces jointes du bloc-notes (section 26) : photos et fichiers.
//!
//! Une photo est lue, tournée selon l'orientation que le téléphone a notée,
//! réduite si elle dépasse 2 048 pixels, et gardée avec une vignette de
//! 320 pixels (R-96). Un autre fichier est gardé tel quel, jusqu'à 25 Mo.
//! Les octets vivent hors de `notes.json` (R-95) : c'est l'affaire du `Depot`
//! de la plateforme, sous les clés données ici.

use std::io::Cursor;

use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;
use image::{DynamicImage, ImageDecoder as _, ImageFormat, ImageReader, Limits, RgbImage};

use crate::stockage::PieceJointe;

/// Côté le plus long d'une photo gardée, en pixels.
pub const COTE_MAX_PHOTO: u32 = 2048;
/// Côté le plus long d'une vignette, en pixels.
pub const COTE_VIGNETTE: u32 = 320;
/// Qualité JPEG d'une photo réduite, et d'une vignette.
const QUALITE_PHOTO: u8 = 85;
const QUALITE_VIGNETTE: u8 = 80;
/// Taille maximale d'un fichier joint autre qu'une photo, en octets (25 Mo).
pub const TAILLE_MAX_FICHIER: usize = 25 * 1024 * 1024;
/// Au-delà de cette taille (64 Mo), un fichier choisi n'est même pas lu.
pub const TAILLE_MAX_LECTURE: u64 = 64 * 1024 * 1024;
/// Nombre maximal de pièces jointes dans une note.
pub const MAX_PIECES_PAR_NOTE: usize = 50;
/// Mémoire permise pour lire une photo : 400 Mo, soit environ 100 mégapixels.
const MEMOIRE_MAX: u64 = 400 * 1024 * 1024;

/// Une photo prête à être gardée.
pub struct PhotoPreparee {
    pub octets: Vec<u8>,
    /// « jpg » ou « png ».
    pub extension: &'static str,
    pub type_mime: &'static str,
    /// Dimensions de la photo gardée, une fois tournée.
    pub largeur: u32,
    pub hauteur: u32,
    /// Vignette, en JPEG.
    pub vignette: Vec<u8>,
}

/// Le format de ces octets, si c'est une image que la calculatrice sait lire.
fn format_lisible(octets: &[u8]) -> Option<ImageFormat> {
    match image::guess_format(octets).ok()? {
        format @ (ImageFormat::Jpeg | ImageFormat::Png) => Some(format),
        _ => None,
    }
}

/// Vrai si ces octets sont une photo (JPEG ou PNG).
pub fn est_une_photo(octets: &[u8]) -> bool {
    format_lisible(octets).is_some()
}

/// Lit une image et la tourne selon son orientation.
fn lire(octets: &[u8]) -> Option<DynamicImage> {
    let format = format_lisible(octets)?;
    let mut lecteur = ImageReader::with_format(Cursor::new(octets), format);
    let mut limites = Limits::default();
    limites.max_alloc = Some(MEMOIRE_MAX);
    lecteur.limits(limites);
    let mut decodeur = lecteur.into_decoder().ok()?;
    let orientation = decodeur.orientation().ok();
    let mut image = DynamicImage::from_decoder(decodeur).ok()?;
    if let Some(orientation) = orientation {
        image.apply_orientation(orientation);
    }
    Some(image)
}

/// L'image sur fond blanc, sans transparence (pour le JPEG).
fn sur_fond_blanc(image: &DynamicImage) -> RgbImage {
    if !image.color().has_alpha() {
        return image.to_rgb8();
    }
    let rvba = image.to_rgba8();
    RgbImage::from_fn(rvba.width(), rvba.height(), |x, y| {
        let [r, v, b, a] = rvba.get_pixel(x, y).0;
        let melanger = |c: u8| {
            let opacite = u16::from(a);
            ((u16::from(c) * opacite + 255 * (255 - opacite) + 127) / 255) as u8
        };
        image::Rgb([melanger(r), melanger(v), melanger(b)])
    })
}

fn en_jpeg(image: &DynamicImage, qualite: u8) -> Option<Vec<u8>> {
    let mut octets = Vec::new();
    JpegEncoder::new_with_quality(&mut octets, qualite)
        .encode_image(&sur_fond_blanc(image))
        .ok()?;
    Some(octets)
}

fn en_png(image: &DynamicImage) -> Option<Vec<u8>> {
    let mut octets = Cursor::new(Vec::new());
    image.write_to(&mut octets, ImageFormat::Png).ok()?;
    Some(octets.into_inner())
}

/// Prépare une photo choisie par l'utilisateur : réduite à 2 048 pixels si
/// elle est plus grande (et réenregistrée dans son format), sinon gardée
/// telle quelle; et sa vignette. Rien si elle est illisible.
pub fn preparer_photo(octets: &[u8]) -> Option<PhotoPreparee> {
    let format = format_lisible(octets)?;
    let image = lire(octets)?;
    let trop_grande = image.width().max(image.height()) > COTE_MAX_PHOTO;
    let image = if trop_grande {
        image.resize(COTE_MAX_PHOTO, COTE_MAX_PHOTO, FilterType::Triangle)
    } else {
        image
    };
    let vignette = en_jpeg(
        &image.thumbnail(COTE_VIGNETTE, COTE_VIGNETTE),
        QUALITE_VIGNETTE,
    )?;
    let (octets, extension, type_mime) = match (format, trop_grande) {
        (ImageFormat::Png, true) => (en_png(&image)?, "png", "image/png"),
        (ImageFormat::Png, false) => (octets.to_vec(), "png", "image/png"),
        (_, true) => (en_jpeg(&image, QUALITE_PHOTO)?, "jpg", "image/jpeg"),
        // Une photo assez petite garde ses octets d'origine : son orientation
        // s'applique à chaque lecture.
        (_, false) => (octets.to_vec(), "jpg", "image/jpeg"),
    };
    Some(PhotoPreparee {
        octets,
        extension,
        type_mime,
        largeur: image.width(),
        hauteur: image.height(),
        vignette,
    })
}

/// Les pixels d'une photo gardée ou d'une vignette, tournée, prêts à afficher :
/// (largeur, hauteur, RVBA).
pub fn pixels(octets: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    let image = lire(octets)?.to_rgba8();
    Some((image.width(), image.height(), image.into_raw()))
}

/// L'extension d'un nom de fichier, en minuscules, si elle est raisonnable
/// (lettres et chiffres, 8 au plus); « bin » sinon.
pub fn extension_de(nom: &str) -> String {
    match nom.rsplit_once('.') {
        Some((base, extension))
            if !base.is_empty()
                && !extension.is_empty()
                && extension.len() <= 8
                && extension.chars().all(|c| c.is_ascii_alphanumeric()) =>
        {
            extension.to_ascii_lowercase()
        }
        _ => "bin".to_string(),
    }
}

/// Le type d'un fichier d'après son extension, pour le navigateur.
pub fn type_mime_de(extension: &str) -> &'static str {
    match extension {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "pdf" => "application/pdf",
        "txt" => "text/plain",
        "csv" => "text/csv",
        "dxf" => "image/vnd.dxf",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xls" => "application/vnd.ms-excel",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        _ => "application/octet-stream",
    }
}

/// Clé des octets d'une pièce jointe : son numéro et son extension (le
/// programme qui l'ouvre sur le bureau se choisit d'après l'extension).
pub fn cle_des_octets(piece: &PieceJointe) -> String {
    format!("{}.{}", piece.id, extension_de(&piece.nom))
}

/// Clé de la vignette d'une photo.
pub fn cle_de_la_vignette(id: u64) -> String {
    format!("{id}.vignette.jpg")
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{GenericImageView as _, Rgb, Rgba, RgbaImage};

    fn jpeg(largeur: u32, hauteur: u32) -> Vec<u8> {
        let image = RgbImage::from_fn(largeur, hauteur, |x, y| {
            Rgb([(x % 256) as u8, (y % 256) as u8, 128])
        });
        let mut octets = Vec::new();
        JpegEncoder::new_with_quality(&mut octets, 90)
            .encode_image(&image)
            .unwrap();
        octets
    }

    fn png(largeur: u32, hauteur: u32) -> Vec<u8> {
        let image = DynamicImage::ImageRgba8(RgbaImage::from_fn(largeur, hauteur, |x, _| {
            Rgba([200, 30, 30, if x < largeur / 2 { 255 } else { 0 }])
        }));
        en_png(&image).unwrap()
    }

    #[test]
    fn une_grande_photo_est_reduite_avec_sa_vignette() {
        let photo = preparer_photo(&jpeg(3000, 1500)).unwrap();
        assert_eq!((photo.largeur, photo.hauteur), (2048, 1024));
        assert_eq!((photo.extension, photo.type_mime), ("jpg", "image/jpeg"));
        let gardee = image::load_from_memory(&photo.octets).unwrap();
        assert_eq!(gardee.dimensions(), (2048, 1024));
        let vignette = image::load_from_memory(&photo.vignette).unwrap();
        assert_eq!(vignette.dimensions(), (320, 160));
    }

    #[test]
    fn une_petite_photo_garde_ses_octets() {
        let origine = jpeg(800, 600);
        let photo = preparer_photo(&origine).unwrap();
        assert_eq!(photo.octets, origine);
        assert_eq!((photo.largeur, photo.hauteur), (800, 600));
        let (largeur, hauteur, rvba) = pixels(&photo.octets).unwrap();
        assert_eq!((largeur, hauteur), (800, 600));
        assert_eq!(rvba.len(), 800 * 600 * 4);
    }

    #[test]
    fn un_png_reste_un_png_et_sa_vignette_est_sur_fond_blanc() {
        let photo = preparer_photo(&png(4096, 100)).unwrap();
        assert_eq!((photo.extension, photo.type_mime), ("png", "image/png"));
        assert_eq!((photo.largeur, photo.hauteur), (2048, 50));
        assert!(est_une_photo(&photo.octets));
        let vignette = image::load_from_memory(&photo.vignette).unwrap().to_rgb8();
        // La moitié transparente devient blanche.
        let [r, v, b] = vignette.get_pixel(vignette.width() - 2, 2).0;
        assert!(r > 240 && v > 240 && b > 240, "{r} {v} {b}");
        // La moitié opaque garde sa couleur.
        let [r, v, _] = vignette.get_pixel(1, 2).0;
        assert!(r > 150 && v < 90, "{r} {v}");
    }

    #[test]
    fn ce_qui_n_est_pas_une_photo_est_refuse() {
        assert!(!est_une_photo(b"%PDF-1.4 un devis"));
        assert!(preparer_photo(b"%PDF-1.4 un devis").is_none());
        assert!(pixels(b"pas une image").is_none());
        // Un JPEG tronqué est illisible.
        let entier = jpeg(64, 64);
        assert!(preparer_photo(&entier[..entier.len() / 3]).is_none());
    }

    #[test]
    fn extensions_et_types() {
        assert_eq!(extension_de("Devis final.PDF"), "pdf");
        assert_eq!(extension_de("IMG_1234.jpeg"), "jpeg");
        assert_eq!(extension_de("sans-extension"), "bin");
        assert_eq!(extension_de(".cache"), "bin");
        assert_eq!(extension_de("plan.tres-longue-extension"), "bin");
        assert_eq!(extension_de("drole.p d f"), "bin");
        assert_eq!(type_mime_de("pdf"), "application/pdf");
        assert_eq!(type_mime_de("inconnu"), "application/octet-stream");
        let piece = PieceJointe {
            id: 42,
            nom: "Soumission.PDF".into(),
            ..PieceJointe::default()
        };
        assert_eq!(cle_des_octets(&piece), "42.pdf");
        assert_eq!(cle_de_la_vignette(42), "42.vignette.jpg");
    }
}
