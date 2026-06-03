# Prompt pour agent IA : installer et configurer news-bar

Copie-colle le bloc ci-dessous à un agent de code (Claude Code, etc.) tournant sur ta machine Windows. Il installe news-bar et le configure **à ta place**, en te posant les bonnes questions.

---

```
Tu vas installer et configurer "news-bar" pour moi sur ce PC Windows. news-bar est
une barre fine qui defile en bas de l'ecran avec les titres de flux RSS, cliquables
(un clic ouvre l'article), et traduisibles en francais. Depot : github.com/enixCode/news-bar.

Procede ainsi, en m'interrogeant quand c'est necessaire :

1) INSTALLER
   - Lance dans PowerShell :
       powershell -c "irm https://github.com/enixCode/news-bar/releases/latest/download/news-bar-installer.ps1 | iex"
   - Puis lance la barre une fois :  news-bar
     (au 1er lancement elle s'inscrit au demarrage auto et ecrit une config par
     defaut dans %APPDATA%\news-bar\config.json). Verifie qu'elle tourne
     (Get-Process news-bar).

2) ME DEMANDER MES GOUTS (ne suppose pas, demande)
   - Quels SUJETS d'actualite je veux suivre ? (ex. tech, finance, monde, France,
     gaming, science...). Propose-moi une liste et laisse-moi choisir.
   - Je veux les titres traduits en FRANCAIS, oui ou non ?
   - Position (bas/haut), et est-ce que je veux changer couleurs/vitesse, ou laisser
     les valeurs par defaut ?

3) TROUVER LES FLUX
   - Pour chaque sujet choisi, trouve l'URL du flux RSS/Atom OFFICIEL et VERIFIE
     qu'elle renvoie du XML valide (fetch l'URL, cherche <rss/<feed/<item/<entry).
     N'invente jamais une URL : teste-la. Ecarte celles qui sont mortes ou payantes
     en contenu (les titres seuls suffisent pour la barre, c'est OK).
   - Propose-moi la liste finale des flux avant de l'ecrire.

4) ECRIRE LA CONFIG
   - Edite %APPDATA%\news-bar\config.json : remplis "feeds" avec mes flux
     (chaque entree { "source": "Nom court", "url": "..." }), mets "translate" a
     true/false selon mon choix, ajuste position/couleurs/vitesse si demande.
   - Garde le format JSON valide.

5) TRADUCTION (si je l'ai demandee)
   - Explique-moi de creer une cle gratuite sur console.groq.com.
   - NE me demande JAMAIS de coller la cle dans le chat. Demande-moi de la poser
     moi-meme :  setx GROQ_API_KEY "ma_cle"
   - Une fois faite, relance la barre en injectant la cle persistante :
       $env:GROQ_API_KEY = [Environment]::GetEnvironmentVariable("GROQ_API_KEY","User")
       Stop-Process -Name news-bar -Force
       Start-Process news-bar

6) VERIFIER
   - Confirme que la barre tourne, affiche mes flux, et (si traduction activee) en
     francais. Si la traduction ne se declenche pas, c'est presque toujours que la
     cle n'est pas vue par le process : il faut relancer la barre APRES le setx.

Regles : ne mets jamais de cle API dans config.json ni dans un fichier du depot.
Si un flux echoue, signale-le et propose une alternative, ne plante pas le reste.
```

---

L'agent peut aussi **construire depuis les sources** au lieu d'utiliser le binaire publié :
`cd rust && cargo build --release`, puis lancer `rust/target/release/news-bar.exe` (au 1er
lancement il s'inscrit au démarrage et écrit sa config par défaut dans `%APPDATA%\news-bar`).
