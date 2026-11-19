# Polices de la version Web

Dans un navigateur, le programme n'a pas accès aux polices de Windows. La version Web
embarque donc des polices libres. La version de bureau ne les utilise pas : elle charge
Segoe UI, déjà présente sous Windows.

| Fichier | Rôle | Origine | Licence |
|---|---|---|---|
| `selawk.ttf`, `selawkb.ttf` | texte courant et gras | Selawik 1.01, Microsoft — fichiers d'origine, non modifiés | SIL OFL 1.1 (`Selawik-LICENCE.txt`) |
| `OpenSans-Italic-extrait.ttf` | italique (étiquettes secondaires des touches) | Open Sans Italic — extrait | SIL OFL 1.1 (`OpenSans-LICENCE.txt`) |
| `OpenSans-Regular-signes.ttf`, `OpenSans-Bold-signes.ttf` | signes absents de Selawik : `−` `√` `π` `Ω` `′` `″` et l'espace insécable | Open Sans — extraits | SIL OFL 1.1 (`OpenSans-LICENCE.txt`) |
| `DejaVuSans-symboles.ttf` | symboles absents de Selawik : `⁻` `→` `↔` `◄` `►` `⌫` | DejaVu Sans 2.37 — extrait | licence Bitstream Vera et domaine public (`DejaVu-LICENCE.txt`) |

Pourquoi Selawik : ses caractères ont exactement les largeurs de ceux de Segoe UI. La
mise en page de la version Web est donc la même que celle de la version de bureau.

## Sources

- Selawik : <https://github.com/microsoft/Selawik/releases/tag/1.01> (`Selawik_Release.zip`)
- Open Sans : <https://github.com/googlefonts/opensans> (dossier `fonts/ttf`)
- DejaVu Sans : <https://github.com/dejavu-fonts/dejavu-fonts/releases/tag/version_2_37>

## Refaire les extraits

Les extraits ne gardent que les caractères utiles, pour alléger la page. Ils se refont
avec `pyftsubset` (paquet Python `fonttools`), à partir des fichiers d'origine :

```sh
SIGNES="U+00A0,U+03A9,U+03C0,U+2032,U+2033,U+2212,U+221A"
pyftsubset OpenSans-Regular.ttf --unicodes="$SIGNES" --name-IDs='*' \
    --output-file=OpenSans-Regular-signes.ttf
pyftsubset OpenSans-Bold.ttf --unicodes="$SIGNES" --name-IDs='*' \
    --output-file=OpenSans-Bold-signes.ttf
pyftsubset OpenSans-Italic.ttf --name-IDs='*' \
    --unicodes="U+0020-007E,U+00A0-00FF,U+0152,U+0153,U+2013,U+2014,U+2018,U+2019,U+201C,U+201D,U+2026,U+2032,U+2033,U+03C0,U+2212,U+221A" \
    --output-file=OpenSans-Italic-extrait.ttf
pyftsubset DejaVuSans.ttf --unicodes="U+207B,U+2192,U+2194,U+25BA,U+25C4,U+232B" \
    --name-IDs='*' --output-file=DejaVuSans-symboles.ttf
```

`--name-IDs='*'` conserve dans chaque extrait la mention de droit d'auteur et la licence
de la police d'origine.

## Ajouter un caractère

Le test `polices_web_sans_caractere_manquant` (`app/src/theme.rs`) relève tous les
caractères des textes du programme et échoue si l'un d'eux n'a pas de glyphe dans ces
polices, en courant, en gras ou en italique. S'il échoue après l'ajout d'un texte :
ajouter le caractère à la liste `--unicodes` de l'extrait voulu, refaire l'extrait et
relancer `cargo test`.
