# Changelog

Toutes les modifications notables de news-bar. Format inspiré de [Keep a Changelog](https://keepachangelog.com/fr/1.0.0/), versions selon [SemVer](https://semver.org/lang/fr/).

## [1.0.1] - 2026-05-30

### Corrigé
- Défilement continu : le texte se répète côte à côte, plus aucun moment où la barre est vide (avant, il y avait un trou entre la fin d'un passage et le retour du texte).

## [1.0.0] - 2026-05-30

### Ajouté
- Barre défilante ancrée en bas (ou en haut), façon bandeau de news.
- Réservation d'espace écran via l'API AppBar (les fenêtres ne la recouvrent pas).
- Configuration complète via `config.json` (texte, vitesse, couleurs, police, position).
- Auto-update au démarrage depuis GitHub, avec préservation du `config.json`.
- Installation en une ligne (`install.ps1`), mise à jour forcée (`update.ps1`), désinstallation (`uninstall.ps1`).
- Support DPI / écrans haute résolution.
