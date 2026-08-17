#!/usr/bin/env python3
"""Simple catalog builder for LUC1D's custom Aidoku sources."""

import json
import hashlib
from pathlib import Path
from datetime import datetime, timezone

ROOT = Path(__file__).resolve().parents[1]
SOURCES_DIR = ROOT / "sources"
ICONS_DIR = ROOT / "icons"
CONFIG_DIR = ROOT / "config"

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
        
        # Calculate SHA256
        sha256 = hashlib.sha256(aix_file.read_bytes()).hexdigest()
        
        # Find icon
        icon_name = f"{source_id}-v{version}.png"
        icon_path = ICONS_DIR / icon_name
        
        if not icon_path.exists():
            print(f"WARNING: Icon missing for {source_id} v{version}")
            continue
        
        # Read source.json from AIX (simple approach - we know the structure)
        # For now, use hardcoded metadata
        sources.append({
            "id": source_id,
            "name": "Asura Scans",
            "version": version,
            "iconURL": f"icons/{icon_name}",
            "downloadURL": f"sources/{aix_file.name}",
            "languages": ["en"],
            "contentRating": 0,
            "baseURL": "https://asurascans.com",
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
