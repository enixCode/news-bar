// Frontiere par plateforme. La logique partagee (lecture de config, assemblage
// du texte, calcul du defilement) vit hors d'ici ; chaque OS fournit son propre
// `run(cfg)` qui cree la fenetre, dessine et reserve l'espace ecran.
//
// Backend present : Windows (Win32 : SHAppBarMessage + GDI).
// Pour ajouter Linux : creer `platform/linux.rs` exposant `pub fn run(cfg)`
// (X11 : _NET_WM_STRUT_PARTIAL + fenetre _NET_WM_WINDOW_TYPE_DOCK), puis
// l'aiguiller ci-dessous sous `#[cfg(target_os = "linux")]`.

use crate::config::Config;

#[cfg(windows)]
mod windows;

#[cfg(windows)]
pub fn run(cfg: Config) {
    windows::run(cfg);
}

#[cfg(not(windows))]
pub fn run(_cfg: Config) {
    eprintln!("news-bar : plateforme non encore supportee (Windows uniquement pour l'instant).");
}
