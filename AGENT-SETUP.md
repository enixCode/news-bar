# news-bar : installation et configuration par un agent IA

Ce fichier contient **tout** ce qu'un agent de code (Claude Code, Cursor...) doit savoir pour installer, configurer, mettre à jour et désinstaller news-bar sur une machine Windows.

**Pour l'utilisateur :** copie le bloc « PROMPT » ci-dessous et colle-le à ton agent. Il fera le reste, de façon interactive (il te demandera tes sujets d'actu, tes préférences, etc.).

---

## PROMPT (à copier-coller à l'agent)

```
Tu vas installer et configurer "news-bar" pour moi sur ce PC Windows, de facon
interactive (demande-moi mes preferences, ne suppose pas). news-bar est une barre
fine qui defile en bas de l'ecran avec les titres de flux RSS, cliquables (clic =
ouvre l'article), traduisibles en francais. Depot : github.com/enixCode/news-bar.
Toutes les references dont tu as besoin sont dans le fichier AGENT-SETUP.md (sections
plus bas) : lis-les.

Etapes :

1) INSTALLER (voir section "Installer / mettre a jour / desinstaller")
   - Lance l'installeur one-liner, puis demarre la barre une fois (news-bar).
   - Verifie qu'elle tourne : Get-Process news-bar.

2) ME DEMANDER MES GOUTS (interactif)
   - Quels SUJETS je veux suivre ? (tech, finance, monde, France, gaming, science,
     sport...). Propose une liste, laisse-moi choisir.
   - Je veux les titres en FRANCAIS ? (oui/non)
   - Position bas/haut, couleurs/vitesse par defaut ou personnalisees ?

3) TROUVER LES FLUX (voir section "Trouver des flux RSS")
   - Pour chaque sujet, trouve l'URL du flux RSS/Atom officiel et VERIFIE qu'elle
     renvoie du XML valide (fetch + cherche <rss/<feed/<item/<entry). N'invente
     jamais une URL. Propose-moi la liste finale avant de l'ecrire.

4) ECRIRE LA CONFIG (voir section "Reference de config.json")
   - Edite %APPDATA%\news-bar\config.json : remplis "feeds", mets "translate" selon
     mon choix, ajuste position/couleurs/vitesse si demande. Garde un JSON valide.
   - Relance la barre pour appliquer.

5) TRADUCTION si demandee (voir section "Traduction")
   - Guide-moi pour creer une cle Groq gratuite. NE me demande JAMAIS de coller la
     cle dans le chat : fais-moi la poser moi-meme avec setx. Puis relance la barre
     en injectant la cle (commande dans la section Traduction).

6) VERIFIER
   - Confirme que la barre affiche mes flux (et en francais si active). Si la
     traduction ne part pas, c'est presque toujours que la cle n'est pas vue : il
     faut relancer la barre APRES le setx.

Regles : ne mets jamais de cle API dans config.json ni dans un fichier. Si un flux
echoue, signale-le et propose une alternative, ne bloque pas le reste.
```

---

## Comment news-bar fonctionne (pour l'agent)

- C'est un binaire natif Windows unique (`news-bar.exe`), sans runtime.
- Au lancement, il : s'inscrit au **démarrage automatique** (clé Run `HKCU`), écrit une **config par défaut** dans `%APPDATA%\news-bar\config.json` si absente, lance l'**updater** en tâche de fond, puis affiche la barre.
- Un **thread de fond** récupère les flux toutes les `refresh_minutes`, les traduit (si activé), et reconstruit le bandeau sans saccade.
- La config est lue dans cet ordre : à côté de l'exe, puis `%APPDATA%\news-bar\config.json`, puis le dossier courant.
- La clé de traduction Groq vient **uniquement** de la variable d'environnement `GROQ_API_KEY`, jamais d'un fichier.

## Installer / mettre à jour / désinstaller

**Installer** (PowerShell) :

```powershell
powershell -c "irm https://github.com/enixCode/news-bar/releases/latest/download/news-bar-installer.ps1 | iex"
news-bar
```

L'installeur (généré par `dist` / cargo-dist) télécharge le binaire compilé par la CI, l'installe dans `~/.cargo/bin` (sur le PATH) avec son updater. Le 1er `news-bar` déclenche l'auto-inscription au démarrage et la config par défaut.

**Mettre à jour** : automatique (l'updater `news-bar-update` tourne au démarrage de la barre). Pour forcer : lancer `news-bar-update`.

**Désinstaller** :

```powershell
irm https://raw.githubusercontent.com/enixCode/news-bar/main/uninstall.ps1 | iex
```

(Arrête la barre, retire la clé Run de démarrage, supprime `%APPDATA%\news-bar` et les binaires.)

## Référence de config.json

Fichier : `%APPDATA%\news-bar\config.json`. Toute clé absente prend une valeur par défaut.

| Clé | Type | Rôle | Défaut |
|---|---|---|---|
| `feeds` | liste | sources RSS : `{ "source": "Nom", "url": "..." }` | (vide) |
| `position` | texte | `"bottom"` ou `"top"` | `"bottom"` |
| `speed` | nombre | pixels/image (plus grand = plus rapide) | `1.5` |
| `font.name` | texte | police | `"Consolas"` |
| `font.size` | nombre | taille en points | `11` |
| `font.bold` | booléen | gras | `true` |
| `padding` | entier | marge verticale (hauteur de barre) | `12` |
| `colors.background` | `[R,G,B]` | couleur de fond (0-255) | `[18,18,22]` |
| `colors.foreground` | `[R,G,B]` | couleur du texte | `[96,230,150]` |
| `separator` | texte | espaces entre deux titres | dix espaces |
| `segments` | liste | textes fixes affichés (rappels, raccourcis) ; non cliquables | (vide) |
| `refresh_minutes` | entier | fréquence de rafraîchissement | `15` |
| `max_items_per_feed` | entier | titres gardés par flux | `5` |
| `item_format` | texte | gabarit d'un titre, avec `{source}` et `{title}` | `"[{source}] {title}"` |
| `translate` | booléen | traduire les titres | `true` |
| `language` | texte | langue cible de traduction | `"français"` |
| `groq_model` | texte | modèle Groq pour la traduction | `"llama-3.1-8b-instant"` |

Exemple minimal :

```json
{
  "position": "bottom",
  "translate": true,
  "feeds": [
    { "source": "HN", "url": "https://news.ycombinator.com/rss" },
    { "source": "Le Monde", "url": "https://www.lemonde.fr/rss/une.xml" }
  ]
}
```

## Trouver des flux RSS

- Beaucoup de sites ont un flux : tester `https://SITE/rss`, `/feed`, `/feed/latest`, `/rss.xml`, ou chercher « *site* RSS feed ».
- **Toujours vérifier** : fetch l'URL et confirme la présence de `<rss`, `<feed`, `<channel`, `<item` ou `<entry`. Écarter les pages HTML / 404.
- Les flux derrière paywall conviennent quand même (la barre n'affiche que les **titres**).
- Pour une source sans flux officiel (ex. Reuters), un fallback non-officiel : `https://news.google.com/rss/search?q=site:SITE.com&hl=fr&gl=FR&ceid=FR:fr` (le signaler comme non-officiel).

## Traduction

1. Clé gratuite sur console.groq.com (API Keys → Create).
2. La poser (l'utilisateur le fait lui-même, jamais collée dans le chat) :
   ```powershell
   setx GROQ_API_KEY "la_cle"
   ```
3. Relancer la barre en lui injectant la clé persistante (un `setx` ne touche pas les process déjà lancés) :
   ```powershell
   $env:GROQ_API_KEY = [Environment]::GetEnvironmentVariable("GROQ_API_KEY","User")
   Stop-Process -Name news-bar -Force
   Start-Process news-bar
   ```

Si l'API échoue (mauvaise clé, quota/429, réseau), la barre garde simplement les titres en version originale : ce n'est pas grave.
