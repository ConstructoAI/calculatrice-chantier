"""Dessine les icônes de l'application Web installable (fichiers PNG).

Le dessin est celui de l'icône de fenêtre (`icone_rvba` dans `app/src/theme.rs`) :
une équerre de charpentier orange sur un fond bleu marine en dégradé. Si le
dessin change là-bas, le changer ici aussi, puis relancer :

    python app/outils/icones.py

Les fichiers sont écrits dans `app/icones/`, que trunk copie dans le site.

Aucune dépendance : seulement la bibliothèque standard de Python.
"""
import os
import struct
import zlib

MARINE = (0.0, 43.0, 107.0)
MARINE_BAS = (0.0, 24.0, 51.0)
ORANGE = (249.0, 115.0, 22.0)
RAYON = 0.18
SUR_ECHANTILLONS = 4

# (nom du fichier, côté en pixels, coins arrondis et transparents, échelle du dessin)
ICONES = [
    ("icone-192.png", 192, True, 1.0),
    ("icone-512.png", 512, True, 1.0),
    # Icône « masquable » d'Android : fond plein, dessin dans la zone toujours visible.
    ("icone-masquable-512.png", 512, False, 0.70),
    # iOS arrondit lui-même les coins et ne gère pas la transparence.
    ("icone-apple-180.png", 180, False, 0.86),
]


def dans(u, v, x0, x1, y0, y1):
    return x0 <= u <= x1 and y0 <= v <= y1


def classe(u, v, arrondi, echelle):
    """0 : hors de l'icône; 1 : fond; 2 : équerre."""
    if arrondi:
        dx = abs(u - 0.5) - (0.5 - RAYON)
        dy = abs(v - 0.5) - (0.5 - RAYON)
        if dx > 0.0 and dy > 0.0 and dx * dx + dy * dy > RAYON * RAYON:
            return 0
    # Le dessin est réduit autour du centre quand l'échelle est inférieure à 1.
    u = (u - 0.5) / echelle + 0.5
    v = (v - 0.5) / echelle + 0.5
    verticale = dans(u, v, 0.24, 0.42, 0.18, 0.82)
    horizontale = dans(u, v, 0.24, 0.82, 0.64, 0.82)
    if verticale or horizontale:
        graduation_v = any(dans(u, v, 0.34, 0.42, y - 0.012, y + 0.012) for y in (0.28, 0.38, 0.48))
        graduation_h = any(dans(u, v, x - 0.012, x + 0.012, 0.64, 0.72) for x in (0.52, 0.62, 0.72))
        return 1 if graduation_v or graduation_h else 2
    return 1


def couleur(c, v):
    if c == 2:
        return ORANGE
    # Le fond s'assombrit vers le bas, comme les barres de l'application.
    return tuple(MARINE[i] + (MARINE_BAS[i] - MARINE[i]) * v for i in range(3))


def pixels(taille, arrondi, echelle):
    lignes = []
    n = SUR_ECHANTILLONS
    for y in range(taille):
        ligne = bytearray()
        for x in range(taille):
            # Pixel uniforme (cas le plus fréquent) : une seule évaluation suffit.
            coins = {classe((x + a) / taille, (y + b) / taille, arrondi, echelle)
                     for a in (0.02, 0.5, 0.98) for b in (0.02, 0.5, 0.98)}
            if len(coins) == 1:
                c = coins.pop()
                if c == 0:
                    ligne += b"\0\0\0\0"
                else:
                    r, g, b = couleur(c, (y + 0.5) / taille)
                    ligne += bytes((round(r), round(g), round(b), 255))
                continue
            somme = [0.0, 0.0, 0.0]
            couverts = 0
            for sy in range(n):
                for sx in range(n):
                    u = (x + (sx + 0.5) / n) / taille
                    v = (y + (sy + 0.5) / n) / taille
                    c = classe(u, v, arrondi, echelle)
                    if c:
                        r, g, b = couleur(c, v)
                        somme[0] += r
                        somme[1] += g
                        somme[2] += b
                        couverts += 1
            if couverts == 0:
                ligne += b"\0\0\0\0"
            else:
                ligne += bytes((round(somme[0] / couverts), round(somme[1] / couverts),
                                round(somme[2] / couverts), round(255 * couverts / (n * n))))
        lignes.append(bytes(ligne))
    return lignes


def png(lignes, taille):
    def bloc(nom, donnees):
        corps = nom + donnees
        return struct.pack(">I", len(donnees)) + corps + struct.pack(">I", zlib.crc32(corps) & 0xFFFFFFFF)

    brut = b"".join(b"\0" + ligne for ligne in lignes)  # filtre 0 sur chaque ligne
    entete = struct.pack(">IIBBBBB", taille, taille, 8, 6, 0, 0, 0)  # 8 bits, RVBA
    return b"\x89PNG\r\n\x1a\n" + bloc(b"IHDR", entete) + bloc(b"IDAT", zlib.compress(brut, 9)) + bloc(b"IEND", b"")


if __name__ == "__main__":
    dossier = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "icones")
    for nom, taille, arrondi, echelle in ICONES:
        donnees = png(pixels(taille, arrondi, echelle), taille)
        with open(os.path.join(dossier, nom), "wb") as fichier:
            fichier.write(donnees)
        print("%s : %d x %d, %d octets" % (nom, taille, taille, len(donnees)))
