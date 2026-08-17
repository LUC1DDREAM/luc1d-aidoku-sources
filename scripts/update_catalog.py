#!/usr/bin/env python3
"""Simple catalog builder for LUC1D's custom Aidoku sources."""

import json
import hashlib
from pathlib import Path
from datetime import datetime, timezone

ROOT = Path(__file__).resolve().parents[1]
SOURCES_DIR = ROOT / "sources"
ICONS_DIR = ROOT / "icons"

# Source metadata mapping
SOURCE_INFO = {
    "en.asurascans": {
        "name": "Asura Scans",
        "languages": ["en"],
        "contentRating": 0,
        "baseURL": "https://asurascans.com"
    },
    "en.mangadex": {
        "name": "MangaDex",
        "languages": ["en"],
        "contentRating": 2,
        "baseURL": "https://mangadex.org"
    },
    "en.webtoons": {
        "name": "WEBTOON",
        "languages": ["en"],
        "contentRating": 1,
        "baseURL": "https://www.webtoons.com"
    }
}

def main():
    sources = []
    
    # Scan sources directory
    for aix_file in sorted(SOURCES_DIR.glob("*.aix")):
        # Parse filename: en.asurascans-v1.aix
        name_parts = aix_file.stem.rsplit("-v", 1)
        if len(name_parts) != 2:
            print(f"WARNING: Skipping {aix_file.name} - invalid filename format")
            continue
        
        source_id = name_parts[0]
        version = int(name_parts[1])
        
        # Get metadata
        if source_id not in SOURCE_INFO:
            print(f"WARNING: No metadata for {source_id}, skipping")
            continue
        
        info = SOURCE_INFO[source_id]
        
        # Calculate SHA256
        sha256 = hashlib.sha256(aix_file.read_bytes()).hexdigest()
        
        # Find icon
        icon_name = f"{source_id}-v{version}.png"
        icon_path = ICONS_DIR / icon_name
        
        if not icon_path.exists():
            print(f"WARNING: Icon missing for {source_id} v{version}")
            icon_url = f"icons/{icon_name}"  # Will be placeholder
        else:
            icon_url = f"icons/{icon_name}"
        
        sources.append({
            "id": source_id,
            "name": info["name"],
            "version": version,
            "iconURL": icon_url,
            "downloadURL": f"sources/{aix_file.name}",
            "languages": info["languages"],
            "contentRating": info["contentRating"],
            "baseURL": info["baseURL"],
            "sha256": sha256
        })
    
    # Build catalog
    catalog = {
        "name": "LUC1D's Custom Aidoku Sources",
        "generatedAt": datetime.now(timezone.utc).isoformat(),
        "sources": sources
    }
    
    # Write index files
    (ROOT / "index.json").write_text(
        json.dumps(catalog, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8"
    )
    
    (ROOT / "index.min.json").write_text(
        json.dumps(catalog, separators=(",", ":"), ensure_ascii=False),
        encoding="utf-8"
    )
    
    print(f"✓ Catalog updated: {len(sources)} source(s)")
    for source in sources:
        print(f"  - {source['name']} v{source['version']}")

if __name__ == "__main__":
    main()
