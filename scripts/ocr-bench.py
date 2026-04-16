#!/usr/bin/env python3
"""Benchmark OCR engines on the synthetic scanned corpus.

Runs each engine on the PNG scans in test-corpus/scanned/,
compares output to ground truth (first-page markdown from source PDFs),
and writes results to test-corpus/results/.

Metrics:
- Wall-clock (seconds)
- Extracted character count
- Character-level accuracy vs. first-page ground truth (rough Levenshtein ratio)
- Word-level recall on a 50-word keyword set
"""
from __future__ import annotations

import difflib
import json
import re
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SCANNED = ROOT / "test-corpus" / "scanned"
GROUND_TRUTH = ROOT / "test-corpus" / "ground-truth"
RESULTS = ROOT / "test-corpus" / "results"
RESULTS.mkdir(parents=True, exist_ok=True)


def normalize(text: str) -> str:
    """Normalize for fair comparison: lowercase, collapse whitespace, strip punct."""
    text = text.lower()
    # Strip most punctuation except apostrophes (for contractions)
    text = re.sub(r"[^\w\s']", " ", text)
    text = re.sub(r"\s+", " ", text).strip()
    return text


def first_page_ground_truth(gt_path: Path) -> str:
    """Extract roughly the first-page worth of ground truth text.

    Since our scans are only page 1, we compare against the first ~2000 chars
    of the full-document ground truth. This is approximate but good enough
    for accuracy-trend comparison across engines.
    """
    text = gt_path.read_text(errors="ignore")
    # Take first 3000 chars — roughly one page of dense text
    return text[:3000]


def run_ocrs(png: Path) -> tuple[str, float]:
    """Run ocrs on a PNG. Returns (output_text, elapsed_seconds)."""
    start = time.time()
    result = subprocess.run(
        ["ocrs", str(png)],
        capture_output=True,
        text=True,
        check=False,
    )
    elapsed = time.time() - start
    return result.stdout, elapsed


def accuracy_ratio(ocr_text: str, ground_truth: str) -> float:
    """Compute a rough character-level similarity ratio (0.0 - 1.0)."""
    a = normalize(ocr_text)
    b = normalize(ground_truth)
    if not b:
        return 0.0
    return difflib.SequenceMatcher(None, a, b).ratio()


def keyword_recall(ocr_text: str, ground_truth: str, top_n: int = 50) -> float:
    """Pick top-N frequent content words from ground truth, check OCR recall.

    Filters common stopwords and returns fraction found in OCR output.
    """
    stopwords = {
        "the", "a", "an", "and", "or", "of", "to", "in", "for", "on", "at",
        "is", "are", "was", "were", "be", "been", "being", "have", "has",
        "had", "do", "does", "did", "will", "would", "should", "could",
        "may", "might", "must", "can", "this", "that", "these", "those",
        "i", "you", "he", "she", "it", "we", "they", "me", "him", "her",
        "us", "them", "my", "your", "his", "its", "our", "their",
        "with", "by", "from", "as", "but", "not", "if", "then", "so",
        "all", "any", "some", "no", "one", "two", "three", "new",
    }

    gt_words = [
        w for w in normalize(ground_truth).split()
        if w not in stopwords and len(w) > 2
    ]
    if not gt_words:
        return 0.0

    # Take top-N by frequency
    from collections import Counter
    top = [w for w, _ in Counter(gt_words).most_common(top_n)]

    ocr_words = set(normalize(ocr_text).split())
    found = sum(1 for w in top if w in ocr_words)
    return found / len(top) if top else 0.0


def main() -> int:
    scans = sorted(SCANNED.glob("sample-*-*.png"))
    if not scans:
        print(f"No PNG scans in {SCANNED}", file=sys.stderr)
        return 1

    print(f"{'Sample':<10} {'Engine':<10} {'Time(s)':>8} {'OCR chars':>10} {'Accuracy':>9} {'Keyword':>8}")
    print("-" * 60)

    results = []
    for png in scans:
        # Sample name: "sample-N" from "sample-N-1.png" or "sample-N-01.png"
        stem = png.stem  # e.g., "sample-4-1" or "sample-2-01"
        sample_id = "-".join(stem.split("-")[:2])  # "sample-4"

        gt_path = GROUND_TRUTH / f"{sample_id}.txt"
        if not gt_path.exists():
            print(f"  [{sample_id}] missing ground truth, skipping")
            continue

        ground_truth = first_page_ground_truth(gt_path)

        # ocrs
        ocr_text, elapsed = run_ocrs(png)
        accuracy = accuracy_ratio(ocr_text, ground_truth)
        recall = keyword_recall(ocr_text, ground_truth)

        results.append({
            "sample": sample_id,
            "engine": "ocrs",
            "elapsed_s": round(elapsed, 3),
            "chars": len(ocr_text),
            "accuracy_ratio": round(accuracy, 3),
            "keyword_recall": round(recall, 3),
        })

        print(f"{sample_id:<10} {'ocrs':<10} {elapsed:>8.2f} {len(ocr_text):>10} {accuracy:>9.3f} {recall:>8.3f}")

        # Save raw output for manual inspection
        out_path = RESULTS / f"{sample_id}.ocrs.txt"
        out_path.write_text(ocr_text)

    # Save JSON results
    results_path = RESULTS / "summary.json"
    results_path.write_text(json.dumps(results, indent=2))
    print(f"\nResults written to {RESULTS}")
    print(f"Summary: {results_path}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
