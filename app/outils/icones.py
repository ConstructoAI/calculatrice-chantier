"""Produit les icônes de l'application à partir du logo de Constructo AI.

Le logo (`app/icones/logo-constructo-ai.png`, fond transparent) est posé sur un
fond blanc, pour rester lisible sur une barre des tâches ou un écran d'accueil
sombres. Si le logo change, remplacer ce fichier, puis relancer :

    python app/outils/icones.py

Les fichiers sont écrits dans `app/icones/` :

- les icônes PNG de l'application Web installable, que trunk copie dans le site;
- `icone-96.rvba`, les pixels bruts (rouge, vert, bleu, alpha) de l'icône de la
  fenêtre et de la marque de la barre supérieure, lus par `app/src/theme.rs`.

Dépendance : Pillow (`pip install pillow`).
"""
import os

from PIL import Image, ImageDraw

BLANC = (255, 255, 255, 255)
RAYON = 0.18
SUR_ECHANTILLONS = 4

# (nom du fichier, côté en pixels, coins arrondis et transparents, part du côté prise par le logo)
ICONES = [
    ("icone-192.png", 192, True, 0.78),
    ("icone-512.png", 512, True, 0.78),
    # Icône « masquable » d'Android : fond plein, logo dans la zone toujours visible.
    ("icone-masquable-512.png", 512, False, 0.58),
    # iOS arrondit lui-même les coins et ne gère pas la transparence.
    ("icone-apple-180.png", 180, False, 0.72),
    # Fenêtre de bureau et barre supérieure : pixels bruts, sans en-tête.
    ("icone-96.rvba", 96, True, 0.78),
]


def icone(logo, cote, arrondi, part):
    grand = cote * SUR_ECHANTILLONS
    fond = Image.new("RGBA", (grand, grand), (255, 255, 255, 0))
    dessin = ImageDraw.Draw(fond)
    if arrondi:
        dessin.rounded_rectangle((0, 0, grand - 1, grand - 1), radius=round(grand * RAYON), fill=BLANC)
    else:
        dessin.rectangle((0, 0, grand, grand), fill=BLANC)
    # Le logo, réduit pour tenir dans la part voulue, puis centré.
    echelle = grand * part / max(logo.size)
    taille = (round(logo.width * echelle), round(logo.height * echelle))
    reduit = logo.resize(taille, Image.LANCZOS)
    fond.alpha_composite(reduit, ((grand - taille[0]) // 2, (grand - taille[1]) // 2))
    return fond.resize((cote, cote), Image.LANCZOS)


def main():
    dossier = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "icones")
    logo = Image.open(os.path.join(dossier, "logo-constructo-ai.png")).convert("RGBA")
    logo = logo.crop(logo.getbbox())
    for nom, cote, arrondi, part in ICONES:
        image = icone(logo, cote, arrondi, part)
        chemin = os.path.join(dossier, nom)
        if nom.endswith(".rvba"):
            with open(chemin, "wb") as fichier:
                fichier.write(image.tobytes())
        else:
            image.save(chemin, optimize=True)
        print(nom, cote)


if __name__ == "__main__":
    main()
