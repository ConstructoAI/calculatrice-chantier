//! Inscrit dans le programme la révision du code source dont il est issu :
//! l'identifiant court du commit git. Elle s'affiche au pied du menu, à côté
//! de la version, pour savoir exactement quelle mouture est installée.

use std::process::Command;

fn main() {
    // Sans dépôt git (archive du code source, par exemple), la révision reste vide.
    let revision = Command::new("git")
        .args(["rev-parse", "--short=7", "HEAD"])
        .output()
        .ok()
        .filter(|sortie| sortie.status.success())
        .and_then(|sortie| String::from_utf8(sortie.stdout).ok())
        .map(|texte| texte.trim().to_string())
        .filter(|texte| !texte.is_empty() && texte.chars().all(|c| c.is_ascii_hexdigit()))
        .unwrap_or_default();
    println!("cargo:rustc-env=REVISION={revision}");
    // À refaire quand le commit courant change.
    println!("cargo:rerun-if-changed=../.git/HEAD");
    println!("cargo:rerun-if-changed=../.git/logs/HEAD");
    println!("cargo:rerun-if-changed=build.rs");
}
