# news-bar : flux RSS (suivi)

Objectif : le bandeau affiche des titres RSS defilants, **cliquables** (un clic ouvre l'article dans le navigateur). Plus de texte fixe.

## Decisions
- RSS uniquement (plus de segments custom).
- Reseau : `ureq` + `native-tls` (SChannel sur Windows, certifs OS, zero crate crypto tierce).
- Parsing : `feed-rs` (RSS 0.9->2.0, Atom, JSON Feed).
- Rafraichissement : toutes les 15 min (configurable).
- Multi-bureaux Windows : inchange (la barre reste sur le bureau de lancement).

## Etapes
- [x] 1. Config etendue : `feeds`, `refresh_minutes`, `max_items_per_feed`, `item_format` ; segments retires.
- [x] 2. `feeds.rs` : `fetch_all()` (ureq agent native-tls + feed-rs) -> `Vec<Item { text, url }>`. Source en echec ignoree, jamais de panic.
- [x] 3. `render.rs` : `Strip` construit a partir d'items + hit-boxes (titre -> url).
- [x] 4. `windows/mod.rs` : thread worker (fetch periodique) + `WM_APP_REFRESH` -> reconstruction du `Strip` sans bloquer l'UI.
- [x] 5. Clic gauche sur un titre -> ouvre l'URL (`ShellExecuteW`). Fermeture = clic droit > Fermer (le double-clic ne ferme plus).

## Flux configures (config.json)
| Source | Etat |
|---|---|
| Bloomberg | flux officiel public |
| Stratechery | officiel (articles gratuits ~1/sem) |
| Rest of World | officiel, complet |
| The Information | officiel, titres seuls (paywall) |
| FT | officiel (peut bloquer selon IP/bot) |
| Reuters | pas de flux officiel -> Google News RSS (non-officiel) |

## Notes
- La barre n'affiche que des TITRES : les paywalls (FT, The Information) ne genent pas l'affichage.
- Au demarrage : placeholder "chargement des flux..." jusqu'au 1er fetch.
- Reseau coupe : le dernier contenu valide reste affiche.

## Reste a faire (separe)
- Distribution : publier l'.exe via GitHub Releases, migrer install/update/uninstall + autostart vers le natif.
- Backend Linux (X11 : `_NET_WM_STRUT_PARTIAL`) si besoin, sur une vraie session Linux.
