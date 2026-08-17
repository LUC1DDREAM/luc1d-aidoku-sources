# LUC1D's Custom Aidoku Sources

Custom manga sources for the Aidoku iOS app.

## 📚 Available Sources

- **Asura Scans** (v1) - Full-featured source with:
  - ✅ API-based chapter listing
  - ✅ Premium/locked chapter detection
  - ✅ Unlock time badges
  - ✅ High-quality image loading
  - ✅ Search & filters

## 🚀 Installation

### Option 1: Direct URL (Recommended)

Add this repository URL to Aidoku:

```
https://luc1ddream.github.io/luc1d-aidoku-sources/index.json
```

### Option 2: Manual Installation

1. Download the `.aix` file from [sources/](sources/)
2. Open in Aidoku via Files app

## 🛠️ Building from Source

### Prerequisites

- Rust toolchain with `wasm32-unknown-unknown` target
- Python 3.7+

### Build Steps

```bash
cd src/en.asurascans
bash scripts/build.sh
```

The compiled `.aix` package will be in `target/wasm32-unknown-unknown/release/`.

## 📝 Source Information

### Asura Scans

- **ID**: `en.asurascans`
- **Language**: English
- **Base URL**: https://asurascans.com
- **Features**:
  - Direct API integration
  - Premium chapter detection
  - Unlock time countdown
  - Advanced filtering

## 🔧 Development

This repository uses GitHub Actions for automated builds and catalog updates.

### Adding a New Source

1. Create source directory: `src/<lang>.<source-name>/`
2. Add required files:
   - `res/source.json` - Source metadata
   - `res/icon.png` - Source icon
   - `res/filters.json` - Optional filters
   - `src/lib.rs` - Source implementation
   - `Cargo.toml` - Build configuration
3. Update `.github/workflows/` with build workflow
4. Push to `main` branch

### Version Scheme

Each source maintains its own version number starting from v1.

## 📄 License

Sources in this repository are custom implementations for personal use.

## 🙏 Credits

- **Aidoku Team** - iOS manga reader framework
- **Asura Scans** - Manga hosting

---

**Note**: This is a personal source repository. For official community sources, see [Aidoku Community Sources](https://github.com/Aidoku/aidoku-community-sources).
