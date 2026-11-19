# [Documentation](README.md)
"""Crop the FPAS gallery's scale-one captures using their logical bounds."""

import argparse
from pathlib import Path

from PIL import Image


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("captures", type=Path, help="Folder written by GuiDocCaptures.")
    parser.add_argument("output", type=Path, help="Folder for the PNG crops.")
    args = parser.parse_args()

    regions = sorted(args.captures.glob("*.crop"))
    if not regions:
        parser.error(f"no crop files in {args.captures}")

    args.output.mkdir(parents=True, exist_ok=True)
    for region in regions:
        left, top, right, bottom = map(int, region.read_text(encoding="utf-8").split(","))
        with Image.open(region.with_suffix(".png")) as frame:
            if not (0 <= left < right <= frame.width and 0 <= top < bottom <= frame.height):
                parser.error(f"crop outside the frame: {region}")
            image = frame.crop((left, top, right, bottom)).convert("RGB")
            if image.getbbox() is None or image.getcolors(1) is not None:
                parser.error(f"blank widget capture: {region}")
            image.save(args.output / f"{region.stem}.png", optimize=True)


if __name__ == "__main__":
    main()
