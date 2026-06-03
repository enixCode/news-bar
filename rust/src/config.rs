// Lecture de config.json. Toute valeur absente/invalide retombe sur un defaut ;
// un fichier introuvable/illisible donne la config par defaut.

use serde::Deserialize;
use std::path::PathBuf;

#[derive(Clone)]
pub struct Feed {
    pub source: String,
    pub url: String,
}

#[derive(Clone)]
pub struct Config {
    pub position: String, // "bottom" | "top"
    pub speed: f64,
    pub font_name: String,
    pub font_size: f64,
    pub bold: bool,
    pub padding: i32,
    pub bg: [u8; 3], // R, G, B
    pub fg: [u8; 3],
    pub separator: String,
    pub feeds: Vec<Feed>,
    pub refresh_minutes: u64,
    pub max_items_per_feed: usize,
    pub item_format: String, // gabarit avec {source} et {title}
    pub translate: bool,     // traduire les titres en francais (cle via GROQ_API_KEY)
    pub groq_model: String,
}

impl Config {
    fn fallback() -> Self {
        Config {
            position: "bottom".into(),
            speed: 1.5,
            font_name: "Consolas".into(),
            font_size: 11.0,
            bold: true,
            padding: 12,
            bg: [18, 18, 22],
            fg: [96, 230, 150],
            separator: "          ".into(),
            feeds: Vec::new(),
            refresh_minutes: 15,
            max_items_per_feed: 5,
            item_format: "[{source}] {title}".into(),
            translate: true,
            groq_model: "llama-3.1-8b-instant".into(),
        }
    }
}

#[derive(Deserialize)]
struct RawFont {
    name: Option<String>,
    size: Option<f64>,
    bold: Option<bool>,
}

#[derive(Deserialize)]
struct RawColors {
    background: Option<[u8; 3]>,
    foreground: Option<[u8; 3]>,
}

#[derive(Deserialize)]
struct RawFeed {
    source: Option<String>,
    url: Option<String>,
}

#[derive(Deserialize)]
struct Raw {
    position: Option<String>,
    speed: Option<f64>,
    font: Option<RawFont>,
    padding: Option<i32>,
    colors: Option<RawColors>,
    separator: Option<String>,
    feeds: Option<Vec<RawFeed>>,
    refresh_minutes: Option<u64>,
    max_items_per_feed: Option<usize>,
    item_format: Option<String>,
    translate: Option<bool>,
    groq_model: Option<String>,
}

// Config par defaut embarquee (le config.json du depot), ecrite au 1er lancement
// si aucune config n'existe encore.
const DEFAULT_CONFIG: &str = include_str!("../../config.json");

fn appdata_config() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|a| PathBuf::from(a).join("news-bar").join("config.json"))
}

// Cherche config.json : a cote de l'exe (portable), puis %APPDATA%\news-bar, puis
// le dossier courant (dev).
fn find_config() -> Option<PathBuf> {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let p = dir.join("config.json");
            if p.is_file() {
                return Some(p);
            }
        }
    }
    if let Some(p) = appdata_config() {
        if p.is_file() {
            return Some(p);
        }
    }
    let cwd = PathBuf::from("config.json");
    if cwd.is_file() {
        return Some(cwd);
    }
    None
}

// Retourne le chemin de config a lire ; si aucune n'existe, en ecrit une par
// defaut dans %APPDATA%\news-bar et renvoie ce chemin.
fn ensure_config() -> Option<PathBuf> {
    if let Some(p) = find_config() {
        return Some(p);
    }
    let p = appdata_config()?;
    if let Some(dir) = p.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    std::fs::write(&p, DEFAULT_CONFIG).ok().map(|_| p)
}

pub fn load() -> Config {
    let path = match ensure_config() {
        Some(p) => p,
        None => return Config::fallback(),
    };
    let bytes = match std::fs::read(&path) {
        Ok(b) => b,
        Err(_) => return Config::fallback(),
    };
    // Decode UTF-8 tolerant + suppression d'un eventuel BOM.
    let raw = String::from_utf8_lossy(&bytes);
    let raw = raw.trim_start_matches('\u{feff}');
    let j: Raw = match serde_json::from_str(raw) {
        Ok(j) => j,
        Err(_) => return Config::fallback(),
    };

    let d = Config::fallback();
    let font = j.font.unwrap_or(RawFont {
        name: None,
        size: None,
        bold: None,
    });
    let colors = j.colors.unwrap_or(RawColors {
        background: None,
        foreground: None,
    });
    let feeds = j
        .feeds
        .unwrap_or_default()
        .into_iter()
        .filter_map(|f| {
            f.url.map(|url| Feed {
                source: f.source.unwrap_or_default(),
                url,
            })
        })
        .collect();

    Config {
        position: j.position.unwrap_or(d.position),
        speed: j.speed.unwrap_or(d.speed),
        font_name: font.name.unwrap_or(d.font_name),
        font_size: font.size.unwrap_or(d.font_size),
        bold: font.bold.unwrap_or(d.bold),
        padding: j.padding.unwrap_or(d.padding),
        bg: colors.background.unwrap_or(d.bg),
        fg: colors.foreground.unwrap_or(d.fg),
        separator: j.separator.unwrap_or(d.separator),
        feeds,
        refresh_minutes: j.refresh_minutes.unwrap_or(d.refresh_minutes),
        max_items_per_feed: j.max_items_per_feed.unwrap_or(d.max_items_per_feed),
        item_format: j.item_format.unwrap_or(d.item_format),
        translate: j.translate.unwrap_or(d.translate),
        groq_model: j.groq_model.unwrap_or(d.groq_model),
    }
}
