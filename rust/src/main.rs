// news-bar : barre fine defilante, ancree en bas/haut de l'ecran.
// Pas de fenetre console (remplace l'ancien lanceur .vbs cote PowerShell).
#![windows_subsystem = "windows"]

mod config;
mod feeds;
mod platform;
mod translate;

fn main() {
    let cfg = config::load();
    check_updates();
    platform::run(cfg);
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
