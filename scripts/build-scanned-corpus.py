#!/usr/bin/env python3
"""Synthesize a scanned-PDF corpus from clean text PDFs.

For each source PDF, render the first page at 300 DPI to a PNG, then wrap
that PNG back into a single-page PDF. The result is an image-only PDF —
no text layer — that mimics what a real scanned document looks like.

Usage: build-scanned-corpus.py
"""
from __future__ import annotations

import subprocess
import sys
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / "test-corpus" / "source"
SCANNED = ROOT / "test-corpus" / "scanned"
DPI = 300

SCANNED.mkdir(parents=True, exist_ok=True)


def render_page_to_png(pdf: Path, out_png_prefix: Path, dpi: int = DPI) -> Path:
    """Render page 1 of a PDF to PNG via pdftoppm. Returns the PNG path."""
    # pdftoppm appends "-1" to the output prefix for page 1
    subprocess.run(
        [
            "pdftoppm",
            "-png",
            "-r",
            str(dpi),
            "-f",
            "1",
            "-l",
            "1",
            str(pdf),
            str(out_png_prefix),
        ],
        check=True,
        capture_output=True,
    )
    # pdftoppm may zero-pad page numbers when total pages > 9 (e.g., -1 vs -01)
    # so glob for any file matching the prefix and page 1
    parent = out_png_prefix.with_suffix("").parent
    stem = out_png_prefix.name
    candidates = sorted(parent.glob(f"{stem}-*.png"))
    if not candidates:
        raise FileNotFoundError(f"pdftoppm produced no output for prefix {stem}")
    # Return the first (lowest-numbered) page
    return candidates[0]


def png_to_image_pdf(png: Path, out_pdf: Path) -> None:
    """Wrap a PNG in a single-page PDF via PIL (no text layer)."""
    image = Image.open(png)
    # Convert to RGB (PIL can't save RGBA as PDF directly)
    if image.mode != "RGB":
        image = image.convert("RGB")
    image.save(out_pdf, "PDF", resolution=float(DPI))


def main() -> int:
    sources = sorted(SOURCE.glob("sample-*.pdf"))
    if not sources:
        print(f"No source PDFs found in {SOURCE}", file=sys.stderr)
        return 1

    for src in sources:
        name = src.stem
        png_prefix = SCANNED / name
        scanned_pdf = SCANNED / f"{name}.pdf"

        print(f"[{name}] rendering page 1 at {DPI} DPI...", end=" ", flush=True)
        try:
            png = render_page_to_png(src, png_prefix)
            png_to_image_pdf(png, scanned_pdf)
            # Keep the PNG too — useful for ocr_image testing later
            size_kb = scanned_pdf.stat().st_size / 1024
            print(f"ok ({size_kb:.0f} KB)")
        except Exception as exc:
            print(f"FAIL: {exc}")
            return 1

    print(f"\nCorpus written to {SCANNED}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
