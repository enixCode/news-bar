# news-bar

Une barre fine qui défile en bas de l'écran, façon bandeau d'actualités : les titres de tes flux RSS, **cliquables**, et (en option) **traduits en français**.

Binaire natif unique (~800 Ko), zéro runtime, DPI-aware.

- Ancrée en bas (ou en haut), elle **réserve son espace** : les fenêtres maximisées s'arrêtent juste au-dessus, comme la barre des tâches.
- Titres **RSS / Atom** rafraîchis automatiquement en arrière-plan.
- **Clic gauche** sur un titre : ouvre l'article dans le navigateur.
- **Traduction française** optionnelle (via Groq, gratuit).

## Installation

Une ligne dans PowerShell :

```powershell
powershell -c "irm https://github.com/enixCode/news-bar/releases/latest/download/news-bar-installer.ps1 | iex"
```

L'installeur télécharge le binaire (compilé par la CI), l'installe, et met en place l'updater. Lance ensuite la barre une fois :

```powershell
news-bar
```

Elle démarre, **s'ajoute au démarrage de Windows** (elle se lancera seule ensuite), **se met à jour toute seule**, et crée une config par défaut. C'est tout.

## Utilisation

- **Clic gauche** sur un titre : ouvre l'article.
- **Clic droit** : « Fermer la barre ».

## Configuration

Tes réglages sont dans `%APPDATA%\news-bar\config.json`. Édite le fichier, sauvegarde, relance la barre.

| Clé | Rôle |
|---|---|
| `feeds` | tes flux : liste de `{ "source": "Nom", "url": "..." }` |
| `position` | `"bottom"` ou `"top"` |
| `speed` | vitesse de défilement (décimales OK) |
| `colors.background` / `colors.foreground` | couleurs `[R, G, B]` |
| `font.name` / `font.size` / `font.bold` | police |
| `refresh_minutes` | rafraîchissement des flux (15 min par défaut) |
| `max_items_per_feed` | titres gardés par flux |
| `translate` | traduire les titres en français (`true` / `false`) |

### Traduction en français (option)

Crée une clé gratuite sur [console.groq.com](https://console.groq.com), pose-la une fois, relance la barre :

```powershell
setx GROQ_API_KEY "ta_cle"
```

Sans clé, les titres restent en version originale. Ne mets jamais ta clé dans `config.json` : la variable d'environnement la garde hors du fichier.

## Mise à jour / Désinstallation

La barre **se met à jour seule** au démarrage (via l'updater installé). Pour désinstaller :

```powershell
irm https://raw.githubusercontent.com/enixCode/news-bar/main/uninstall.ps1 | iex
```

## Pour les développeurs

Construire localement :

```powershell
cd rust
cargo build --release
```

Le binaire est dans `rust/target/release/news-bar.exe` (il lit `config.json` à côté de lui, ou dans `%APPDATA%\news-bar`).

La distribution est gérée par [dist](https://opensource.axo.dev/cargo-dist/) (`dist-workspace.toml`) et un workflow GitHub Actions (`.github/workflows/release.yml`) : un `git tag vX.Y.Z` poussé déclenche le build dans le cloud et publie la Release (binaire + installeur + updater).

```
news-bar/
├─ dist-workspace.toml            distribution (dist)
├─ config.json                    modele de config par defaut
├─ uninstall.ps1
└─ rust/src/
   ├─ main.rs       point d'entree (+ auto-update)
   ├─ config.rs     lecture de config.json
   ├─ feeds.rs      fetch + parsing RSS/Atom (ureq + roxmltree)
   ├─ translate.rs  traduction FR via Groq
   └─ platform/windows/  fenetre, rendu GDI, AppBar, clic, autostart
```

Réseau : `ureq` + native-tls (certificats du magasin Windows). Architecture découpée par OS, prête pour un futur backend Linux (X11).

## Prérequis

Windows 10 ou 11.

## Licence

MIT. Voir [LICENSE](LICENSE).
