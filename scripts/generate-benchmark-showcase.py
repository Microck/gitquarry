from __future__ import annotations

import argparse
import shutil
import subprocess
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
DEFAULT_SOURCE_DIR = ROOT / "docs" / "images" / "benchmark-study"
DEFAULT_OUTPUT_DIR = ROOT / "docs" / "images" / "benchmark-study"
DEFAULT_CHARTS = [
    "latency-profile",
    "depth-overhead",
    "readme-tax",
    "balanced-tradeoff",
    "balanced-pareto",
    "core-retention",
    "surface-mix",
    "baseline-churn",
    "persistence-leaders",
]
DEFAULT_MODEL = "gpt-image-2"
REFERENCE_DENSITY = 192
SHOWCASE_PROMPT = """Use case: infographic-diagram
Asset type: benchmark study infographic for docs site
Primary request: redesign this benchmark chart into a polished premium editorial infographic while preserving the exact chart structure, visible labels, plotted geometry, scenario ordering, and all numeric values from the source image.
Input images: Image 1: edit target reference chart
Style/medium: crisp modern data journalism infographic, premium SaaS benchmark report, clean typography, subtle depth, luminous card surfaces, high contrast, meticulous spacing
Composition/framing: same layout, same aspect ratio, same amount of information, no cropping
Lighting/mood: bright, precise, trustworthy, technical
Color palette: slate, cobalt, teal, amber, restrained red accents on a clean light background
Text (verbatim): preserve every visible word, number, label, axis tick, legend, and caption exactly as shown in the source image
Constraints: keep benchmark meaning intact; preserve bars, points, bubbles, labels, values, and relative positions; polish only; no extra metrics; no fictional text; no logo; no watermark; no people
Avoid: distorted letters, merged labels, missing values, decorative clutter, perspective distortion, 3D charts"""


def rasterize_reference(svg_path: Path, png_path: Path) -> None:
    convert = shutil.which("convert")
    if not convert:
        raise SystemExit("ImageMagick `convert` is required to rasterize SVG source charts.")

    subprocess.run(
        [
            convert,
            "-background",
            "white",
            "-density",
            str(REFERENCE_DENSITY),
            str(svg_path),
            "-alpha",
            "remove",
            "-alpha",
            "off",
            str(png_path),
        ],
        check=True,
    )


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Generate published benchmark showcase PNGs with egaki from deterministic SVG source charts."
    )
    parser.add_argument(
        "--source-dir",
        default=str(DEFAULT_SOURCE_DIR),
        help="Directory containing deterministic SVG source charts",
    )
    parser.add_argument(
        "--output-dir",
        default=str(DEFAULT_OUTPUT_DIR),
        help="Directory to write published PNG showcase images into",
    )
    parser.add_argument(
        "--charts",
        nargs="*",
        default=DEFAULT_CHARTS,
        help="Chart basenames to render",
    )
    parser.add_argument(
        "--model",
        default=DEFAULT_MODEL,
        help="Egaki image model to use",
    )
    args = parser.parse_args()

    source_dir = Path(args.source_dir)
    output_dir = Path(args.output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    for chart_name in args.charts:
        svg_path = source_dir / f"{chart_name}.svg"
        if not svg_path.exists():
            raise SystemExit(f"Missing source SVG: {svg_path}")

        output_path = output_dir / f"{chart_name}.png"
        with tempfile.TemporaryDirectory(prefix=f"gitquarry-{chart_name}-") as temp_dir_str:
            temp_dir = Path(temp_dir_str)
            reference_png = temp_dir / f"{chart_name}-reference.png"
            rasterize_reference(svg_path, reference_png)

            subprocess.run(
                [
                    "egaki",
                    "image",
                    SHOWCASE_PROMPT,
                    "-m",
                    args.model,
                    "-i",
                    str(reference_png),
                    "-o",
                    str(output_path),
                ],
                check=True,
            )

            print(f"Generated {output_path}")


if __name__ == "__main__":
    main()
