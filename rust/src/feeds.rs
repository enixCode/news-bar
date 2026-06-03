// Recuperation et parsing des flux RSS/Atom (multi-plateforme, pas de Win32 ici).
// HTTP via ureq (synchrone, leger) + TLS native-tls (SChannel sur Windows,
// certificats geres par l'OS). Parsing XML via roxmltree (leger), avec une
// extraction maison des titres + liens (RSS <item> et Atom <entry>).

use std::time::Duration;

use ureq::config::Config as UreqConfig;
use ureq::tls::{RootCerts, TlsConfig, TlsProvider};

use crate::config::Feed;

// Titre brut tel que recupere du flux (avant traduction / formatage).
pub struct Headline {
    pub source: String,
    pub title: String,
    pub url: String,
}

// Item d'affichage final (texte formate + lien ouvert au clic).
pub struct Item {
    pub text: String,
    pub url: String,
}

// Recupere tous les flux. Une source qui echoue (reseau, parse) est ignoree :
// jamais de panic.
pub fn fetch_all(feeds: &[Feed], max_items: usize) -> Vec<Headline> {
    let agent = UreqConfig::builder()
        .timeout_global(Some(Duration::from_secs(20)))
        .tls_config(
            TlsConfig::builder()
                .provider(TlsProvider::NativeTls)
                .root_certs(RootCerts::PlatformVerifier)
                .build(),
        )
        .build()
        .new_agent();

    let mut out = Vec::new();
    for f in feeds {
        let body = match agent.get(f.url.as_str()).call() {
            Ok(mut resp) => match resp.body_mut().read_to_string() {
                Ok(s) => s,
                Err(_) => continue,
            },
            Err(_) => continue,
        };
        parse_into(&body, f, max_items, &mut out);
    }
    out
}

// Extrait jusqu'a `max` titres + liens d'un flux RSS (<item>) ou Atom (<entry>).
fn parse_into(xml: &str, feed: &Feed, max: usize, out: &mut Vec<Headline>) {
    let doc = match roxmltree::Document::parse(xml) {
        Ok(d) => d,
        Err(_) => return,
    };
    let mut n = 0;
    for node in doc.descendants() {
        let tag = node.tag_name().name();
        if tag != "item" && tag != "entry" {
            continue;
        }
        // Titre (nom local : insensible au namespace Atom).
        let raw_title = node
            .children()
            .find(|c| c.tag_name().name() == "title")
            .and_then(|t| t.text())
            .unwrap_or("")
            .trim();
        if raw_title.is_empty() {
            continue;
        }
        let title = decode_entities(raw_title);
        // Lien : RSS = <link>texte</link>, Atom = <link href="..."/>.
        let url = node
            .children()
            .filter(|c| c.tag_name().name() == "link")
            .find_map(|l| {
                l.attribute("href")
                    .map(|h| h.to_string())
                    .or_else(|| l.text().map(|s| s.trim().to_string()))
            })
            .unwrap_or_default();

        out.push(Headline {
            source: feed.source.clone(),
            title,
            url,
        });
        n += 1;
        if n >= max {
            break;
        }
    }
}

// Decode les entites HTML/XML courantes (&amp; &#39; &#8217; ...). Necessaire car
// beaucoup de flux mettent les titres en CDATA avec des entites non decodees par
// le parseur XML (ex. "S&amp;P 500" reste brut -> on le ramene a "S&P 500").
fn decode_entities(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(amp) = rest.find('&') {
        out.push_str(&rest[..amp]);
        let after = &rest[amp..];
        if let Some(semi) = after.find(';') {
            if semi <= 11 {
                if let Some(c) = entity_char(&after[1..semi]) {
                    out.push(c);
                    rest = &after[semi + 1..];
                    continue;
                }
            }
        }
        out.push('&');
        rest = &after[1..];
    }
    out.push_str(rest);
    out
}

fn entity_char(ent: &str) -> Option<char> {
    match ent {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" | "ldquo" | "rdquo" => Some('"'),
        "apos" | "lsquo" | "rsquo" => Some('\''),
        "nbsp" => Some(' '),
        "hellip" => Some('\u{2026}'),
        "ndash" => Some('\u{2013}'),
        "mdash" => Some('\u{2014}'),
        _ => {
            if let Some(hex) = ent.strip_prefix("#x").or_else(|| ent.strip_prefix("#X")) {
                u32::from_str_radix(hex, 16).ok().and_then(char::from_u32)
            } else if let Some(dec) = ent.strip_prefix('#') {
                dec.parse::<u32>().ok().and_then(char::from_u32)
            } else {
                None
            }
        }
    }
}
