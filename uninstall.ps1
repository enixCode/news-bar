# news-bar : desinstallation.
# Arrete la barre, retire le demarrage automatique, supprime la config et les binaires.
$ErrorActionPreference = 'SilentlyContinue'

Get-Process -Name 'news-bar' | Stop-Process -Force

# Demarrage automatique.
Remove-ItemProperty -Path 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run' -Name 'news-bar'

# Config utilisateur.
Remove-Item -Recurse -Force (Join-Path $env:APPDATA 'news-bar')

# Binaires installes par l'installeur dist (~/.cargo/bin).
$bin = Join-Path $env:USERPROFILE '.cargo\bin'
Remove-Item -Force (Join-Path $bin 'news-bar.exe')
Remove-Item -Force (Join-Path $bin 'news-bar-update.exe')

Write-Host "news-bar desinstalle. (Si tu avais pose GROQ_API_KEY : setx GROQ_API_KEY `"`" pour l'effacer.)"
