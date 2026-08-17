# Setup Instructions

## 📦 Repository erstellt!

Deine neue Aidoku Sources Repo ist bereit: **luc1d-aidoku-sources**

## 🚀 Nächste Schritte

### 1. GitHub Repository erstellen

```bash
# Gehe zu GitHub und erstelle ein neues Repository:
# Name: luc1d-aidoku-sources
# Public
# OHNE README, .gitignore, oder License (haben wir schon)
```

### 2. Repository pushen

```bash
cd /root/luc1d-aidoku-sources
git remote add origin https://github.com/LUC1DDREAM/luc1d-aidoku-sources.git
git push -u origin main
```

### 3. GitHub Pages aktivieren

1. Gehe zu **Settings** → **Pages**
2. Source: **GitHub Actions**
3. Speichern

### 4. Ersten Build triggern

Der Build startet automatisch nach dem ersten Push, oder manuell:

1. Gehe zu **Actions** Tab
2. Wähle "Build Asura Scans v1"
3. Klicke "Run workflow"

### 5. In Aidoku nutzen

Nach dem Build (ca. 2-3 Minuten):

**Catalog URL:**
```
https://luc1ddream.github.io/luc1d-aidoku-sources/index.json
```

## 📁 Struktur

```
luc1d-aidoku-sources/
├── .github/workflows/
│   ├── build-asura.yml      # Baut Asura Scans bei Änderungen
│   └── pages.yml            # Deployed zu GitHub Pages
├── config/
│   └── repository.json      # Repository Metadata
├── scripts/
│   └── update_catalog.py    # Generiert index.json
├── sources/                 # Compiled .aix files (nach Build)
├── icons/                   # Source icons (nach Build)
├── src/
│   └── en.asurascans/       # Asura Scans v1 Source Code
│       ├── src/             # Rust source
│       ├── res/             # Metadata & assets
│       └── scripts/         # Build scripts
└── README.md
```

## ✨ Features

### Asura Scans v1

- ✅ API-basierte Chapters
- ✅ Premium/Locked Detection
- ✅ Unlock Time Badges
- ✅ Search & Filters
- ✅ High-Quality Images

## 🔧 Neue Source hinzufügen

1. Erstelle `src/<lang>.<name>/` mit:
   - `Cargo.toml`
   - `src/lib.rs`
   - `res/source.json` (version: 1)
   - `res/icon.png`
   - `scripts/build.sh`

2. Workflow erstellen:
   `.github/workflows/build-<name>.yml`

3. Push → automatischer Build

## 📝 Version Updates

Um eine neue Version zu releasen:

```bash
cd src/en.asurascans
# Ändere Code...
```

Dann in `res/source.json` und `Cargo.toml`:
```json
"version": 2  // Increment
```

Push → Build → Automatisches Catalog Update

## 🎯 Aktueller Stand

- ✅ Repo Structure erstellt
- ✅ Asura Scans v1 Source Code
- ✅ GitHub Actions Workflows
- ✅ Catalog Builder Script
- ⏳ Warte auf GitHub Push
- ⏳ Warte auf ersten Build
