#!/usr/bin/env python3
"""
dlss_ssim_check.py — DLSS output quality regression gate.

Compares a DLSS-upscaled render against a native-resolution reference
using the Structural Similarity Index (SSIM). Fails CI if SSIM < threshold.

Requires: Pillow, scikit-image

Usage:
    python tools/ci/dlss_ssim_check.py \
        --reference renders/native_1080p_ref.png \
        --candidate renders/dlss_quality_1080p.png \
        --threshold 0.92
"""

import argparse
import sys

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--reference',  required=True)
    parser.add_argument('--candidate',  required=True)
    parser.add_argument('--threshold',  type=float, default=0.92)
    args = parser.parse_args()

    try:
        from PIL import Image
        import numpy as np
        from skimage.metrics import structural_similarity as ssim

        ref = np.array(Image.open(args.reference).convert('RGB'))
        cand = np.array(Image.open(args.candidate).convert('RGB'))

        if ref.shape != cand.shape:
            cand_img = Image.fromarray(cand).resize((ref.shape[1], ref.shape[0]))
            cand = np.array(cand_img)

        score = ssim(ref, cand, channel_axis=2, data_range=255)
        print(f"SSIM score: {score:.4f} (threshold: {args.threshold})")

        if score < args.threshold:
            print("FAIL: DLSS output quality below threshold")
            sys.exit(1)
        print("PASS: DLSS quality gate passed")

    except ImportError:
        print("SKIP: scikit-image/Pillow not installed — gate runs on UE5 self-hosted runner only")
        sys.exit(0)

if __name__ == '__main__':
    main()
