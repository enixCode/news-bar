// news-bar : barre fine defilante, ancree en bas/haut de l'ecran.
// Pas de fenetre console (remplace l'ancien lanceur .vbs cote PowerShell).
#![windows_subsystem = "windows"]

mod config;
mod feeds;
mod platform;
mod translate;

fn main() {
    // Mode dev : `news-bar --dev`. Cohabite avec la version de prod (pas
    // d'autostart, pas d'auto-update), signale par une couleur orange + un texte.
    let dev = std::env::args().any(|a| a == "--dev");
    let mut cfg = config::load();
    if dev {
        cfg.fg = [255, 165, 0];
        cfg.segments.insert(0, "[ MODE DEV ]".into());
    } else {
        check_updates();
    }
    platform::run(cfg, dev);
}

// Lance l'updater (installe a cote du binaire par l'installeur dist) en tache de
// fond : la mise a jour est ainsi automatique a chaque demarrage. Ignore en
// silence s'il est absent (build de dev / binaire portable).
fn check_updates() {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let updater = dir.join("news-bar-update.exe");
            if updater.exists() {
                let _ = std::process::Command::new(updater).spawn();
            }
        }
    }
}
