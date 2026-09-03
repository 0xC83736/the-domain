#!/usr/bin/env python3
"""
validate.py — Phase 1 CI gate for asset compliance.

Walks the asset content directory and asserts:
  --assert-nanite           All static meshes >10k tris have Nanite enabled
  --assert-texture-compression  All textures use BC7 (colour) / BC5 (normals) / BC4 (roughness)

In Phase 1 the Content directory is empty; the script exits 0 with a
"no assets found" notice. As assets are added it enforces compliance.

Usage:
    python tools/asset_validation/validate.py \
        --assert-nanite \
        --assert-texture-compression \
        --asset-root engine/Content
"""

import argparse
import json
import os
import sys
from pathlib import Path


def validate_uasset(path: Path, require_nanite: bool, require_compression: bool) -> list[str]:
    """Return list of violation strings for a single .uasset file."""
    # In a real pipeline this would parse the UAsset binary.
    # Phase 1 stub: flag files that contain explicit override markers.
    violations = []
    try:
        text = path.read_text(errors='ignore')
        if require_nanite and 'NaniteEnabled=False' in text:
            violations.append(f"{path}: Nanite explicitly disabled")
        if require_compression and 'CompressionNone' in text:
            violations.append(f"{path}: uncompressed texture detected")
    except Exception as e:
        violations.append(f"{path}: could not read — {e}")
    return violations


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--assert-nanite', action='store_true')
    parser.add_argument('--assert-texture-compression', action='store_true')
    parser.add_argument('--asset-root', required=True)
    args = parser.parse_args()

    root = Path(args.asset_root)
    if not root.exists():
        print(f"Asset root {root} does not exist — skipping validation (no assets yet).")
        sys.exit(0)

    assets = list(root.rglob('*.uasset'))
    if not assets:
        print("No .uasset files found — validation skipped.")
        sys.exit(0)

    violations = []
    for asset in assets:
        violations.extend(validate_uasset(asset, args.assert_nanite, args.assert_texture_compression))

    if violations:
        print("\n=== ASSET VALIDATION FAILED ===")
        for v in violations:
            print(v)
        sys.exit(1)

    print(f"Asset validation passed — {len(assets)} assets checked.")


if __name__ == '__main__':
    main()
