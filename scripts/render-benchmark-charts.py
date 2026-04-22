from __future__ import annotations

import argparse
import csv
import html
from collections import Counter
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
DEFAULT_STUDY_DIR = ROOT / "target" / "benchmark-study"
DEFAULT_OUTPUT_DIR = ROOT / "docs" / "images" / "benchmark-study"


LATENCY_SCENARIOS = [
    "native-best-match",
    "discover-quick-native",
    "discover-balanced-native",
    "discover-deep-native",
    "discover-balanced-query-readme",
]

CHURN_SCENARIOS = [
    "discover-balanced-query",
    "discover-balanced-activity",
    "discover-balanced-quality",
    "discover-balanced-blended",
    "discover-balanced-blended-quality-heavy",
    "discover-balanced-activity-updated-1y",
    "discover-balanced-blended-rust",
]

PERSISTENCE_QUERIES = ["api gateway", "terminal ui"]

SCENARIO_LABELS = {
    "native-best-match": "native best-match",
    "discover-quick-native": "discover quick native",
    "discover-balanced-native": "discover balanced native",
    "discover-deep-native": "discover deep native",
    "discover-balanced-query-readme": "balanced query + README",
    "discover-balanced-query": "balanced query",
    "discover-balanced-activity": "balanced activity",
    "discover-balanced-quality": "balanced quality",
    "discover-balanced-blended": "balanced blended",
    "discover-balanced-blended-quality-heavy": "balanced blended quality-heavy",
    "discover-balanced-activity-updated-1y": "balanced activity + updated 1y",
    "discover-balanced-blended-rust": "balanced blended + rust",
}

SCENARIO_COLORS = {
    "native-best-match": "#0f172a",
    "discover-quick-native": "#2563eb",
    "discover-balanced-native": "#0f766e",
    "discover-deep-native": "#b45309",
    "discover-balanced-query-readme": "#dc2626",
    "discover-balanced-query": "#ea580c",
    "discover-balanced-activity": "#16a34a",
    "discover-balanced-quality": "#2563eb",
    "discover-balanced-blended": "#475569",
    "discover-balanced-blended-quality-heavy": "#1d4ed8",
    "discover-balanced-activity-updated-1y": "#b91c1c",
    "discover-balanced-blended-rust": "#9a3412",
}


def load_csv(path: Path) -> list[dict[str, str]]:
    with path.open("r", encoding="utf-8", newline="") as handle:
        return list(csv.DictReader(handle))


def ensure_dir(path: Path) -> None:
    path.mkdir(parents=True, exist_ok=True)


def escape(value: object) -> str:
    return html.escape(str(value), quote=True)


def svg_text(
    x: float,
    y: float,
    text: str,
    *,
    size: int = 16,
    weight: int = 400,
    fill: str = "#0f172a",
    anchor: str = "start",
) -> str:
    return (
        f'<text x="{x:.1f}" y="{y:.1f}" font-family="Mona Sans, Inter, sans-serif" '
        f'font-size="{size}" font-weight="{weight}" fill="{fill}" '
        f'text-anchor="{anchor}">{escape(text)}</text>'
    )


def svg_rect(
    x: float,
    y: float,
    width: float,
    height: float,
    *,
    fill: str,
    rx: float = 0,
    stroke: str | None = None,
    stroke_width: float = 0,
    opacity: float | None = None,
) -> str:
    attrs = [
        f'x="{x:.1f}"',
        f'y="{y:.1f}"',
        f'width="{width:.1f}"',
        f'height="{height:.1f}"',
        f'fill="{fill}"',
    ]
    if rx:
        attrs.append(f'rx="{rx:.1f}"')
    if stroke:
        attrs.append(f'stroke="{stroke}"')
        attrs.append(f'stroke-width="{stroke_width:.1f}"')
    if opacity is not None:
        attrs.append(f'opacity="{opacity:.3f}"')
    return f"<rect {' '.join(attrs)} />"


def svg_line(
    x1: float,
    y1: float,
    x2: float,
    y2: float,
    *,
    stroke: str,
    stroke_width: float = 1,
    opacity: float = 1,
) -> str:
    return (
        f'<line x1="{x1:.1f}" y1="{y1:.1f}" x2="{x2:.1f}" y2="{y2:.1f}" '
        f'stroke="{stroke}" stroke-width="{stroke_width:.1f}" opacity="{opacity:.3f}" />'
    )


def panel_bar_chart(
    *,
    x: float,
    y: float,
    width: float,
    title: str,
    subtitle: str,
    rows: list[dict[str, object]],
    value_formatter,
    scale_max: float,
    scale_ticks: list[float],
) -> str:
    parts: list[str] = []
    panel_height = 114 + (len(rows) * 56)
    parts.append(
        svg_rect(
            x,
            y,
            width,
            panel_height,
            fill="#f8fafc",
            stroke="#cbd5e1",
            stroke_width=1,
            rx=24,
        )
    )
    parts.append(svg_text(x + 28, y + 40, title, size=26, weight=700))
    parts.append(svg_text(x + 28, y + 68, subtitle, size=14, fill="#475569"))

    label_x = x + 28
    chart_x = x + 260
    value_x = x + width - 26
    bar_width = width - 332
    top = y + 104

    for tick in scale_ticks:
        tx = chart_x + ((tick / scale_max) * bar_width)
        parts.append(svg_line(tx, top - 8, tx, top + (len(rows) * 56) - 20, stroke="#cbd5e1", opacity=0.55))
        parts.append(svg_text(tx, top - 18, value_formatter(tick), size=12, fill="#64748b", anchor="middle"))

    for index, row in enumerate(rows):
        row_y = top + (index * 56)
        bar_y = row_y - 18
        value = float(row["value"])
        color = str(row["color"])
        label = str(row["label"])

        parts.append(svg_text(label_x, row_y, label, size=14, fill="#0f172a"))
        parts.append(svg_rect(chart_x, bar_y, bar_width, 22, fill="#e2e8f0", rx=11))
        parts.append(svg_rect(chart_x, bar_y, (value / scale_max) * bar_width, 22, fill=color, rx=11))
        parts.append(svg_text(value_x, row_y, value_formatter(value), size=14, fill="#0f172a", anchor="end"))

    return "".join(parts)


def write_svg(path: Path, width: int, height: int, body: str) -> None:
    path.write_text(
        "\n".join(
            [
                f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}" width="{width}" height="{height}" role="img">',
                svg_rect(0, 0, width, height, fill="#ffffff"),
                body,
                "</svg>",
            ]
        )
        + "\n",
        encoding="utf-8",
    )


def build_latency_chart(run_summaries: list[dict[str, str]], output_dir: Path) -> None:
    width = 1560
    panel_width = 730
    panels: list[str] = []
    for index, query in enumerate(PERSISTENCE_QUERIES):
        rows = []
        by_scenario = {
            row["scenario"]: row
            for row in run_summaries
            if row["query"] == query and row["scenario"] in LATENCY_SCENARIOS
        }
        for scenario in LATENCY_SCENARIOS:
            row = by_scenario[scenario]
            rows.append(
                {
                    "label": SCENARIO_LABELS[scenario],
                    "value": float(row["duration_ms"]),
                    "color": SCENARIO_COLORS[scenario],
                }
            )
        panels.append(
            panel_bar_chart(
                x=40 + (index * 760),
                y=96,
                width=panel_width,
                title=query,
                subtitle="Latency ladder for representative scenarios",
                rows=rows,
                value_formatter=lambda value: f"{int(value / 1000)}s" if value >= 1000 else f"{int(value)}ms",
                scale_max=62000,
                scale_ticks=[0, 15000, 30000, 45000, 60000],
            )
        )

    body = [
        svg_text(40, 44, "Benchmark latency profile", size=34, weight=800),
        svg_text(
            40,
            72,
            "Native stays sub-second. Depth drives the biggest cost jump, and README enrichment adds another few seconds on top of balanced discover.",
            size=16,
            fill="#334155",
        ),
        *panels,
    ]
    write_svg(output_dir / "latency-profile.svg", width, 520, "".join(body))


def build_churn_chart(comparisons: list[dict[str, str]], output_dir: Path) -> None:
    width = 1560
    panel_width = 730
    panels: list[str] = []
    for index, query in enumerate(PERSISTENCE_QUERIES):
        by_scenario = {
            row["scenario"]: row
            for row in comparisons
            if row["query"] == query and row["scenario"] in CHURN_SCENARIOS
        }
        rows = []
        for scenario in CHURN_SCENARIOS:
            row = by_scenario[scenario]
            rows.append(
                {
                    "label": SCENARIO_LABELS[scenario],
                    "value": float(row["top_k_jaccard"]),
                    "color": SCENARIO_COLORS[scenario],
                }
            )
        panels.append(
            panel_bar_chart(
                x=40 + (index * 760),
                y=96,
                width=panel_width,
                title=query,
                subtitle="Top-10 Jaccard similarity versus native best-match",
                rows=rows,
                value_formatter=lambda value: f"{value:.2f}",
                scale_max=0.70,
                scale_ticks=[0.0, 0.2, 0.4, 0.6],
            )
        )

    body = [
        svg_text(40, 44, "Scenario churn versus the native baseline", size=34, weight=800),
        svg_text(
            40,
            72,
            "Quality-oriented ranking stays closer to the native baseline. Rust and recency slices deliberately pull the result set away from the default top 10.",
            size=16,
            fill="#334155",
        ),
        *panels,
    ]
    write_svg(output_dir / "baseline-churn.svg", width, 640, "".join(body))


def build_persistence_chart(repo_rows: list[dict[str, str]], output_dir: Path) -> None:
    width = 1560
    panel_width = 730
    panels: list[str] = []
    for index, query in enumerate(PERSISTENCE_QUERIES):
        counts = Counter(
            row["full_name"]
            for row in repo_rows
            if row["query"] == query and row["full_name"]
        )
        top_rows = counts.most_common(6)
        rows = [
            {"label": label, "value": float(value), "color": "#2563eb" if idx < 3 else "#0f766e"}
            for idx, (label, value) in enumerate(top_rows)
        ]
        panels.append(
            panel_bar_chart(
                x=40 + (index * 760),
                y=96,
                width=panel_width,
                title=query,
                subtitle="Repositories that persist across the most scenarios",
                rows=rows,
                value_formatter=lambda value: f"{int(value)}",
                scale_max=max(28, max((row["value"] for row in rows), default=1)),
                scale_ticks=[0, 5, 10, 15, 20, 25],
            )
        )

    body = [
        svg_text(40, 44, "Most persistent repositories across the study", size=34, weight=800),
        svg_text(
            40,
            72,
            "These repositories keep surfacing even as ranking strategy, depth, README enrichment, and filter slices change.",
            size=16,
            fill="#334155",
        ),
        *panels,
    ]
    write_svg(output_dir / "persistence-leaders.svg", width, 580, "".join(body))


def main() -> None:
    parser = argparse.ArgumentParser(description="Render static SVG charts for the gitquarry benchmark study.")
    parser.add_argument(
        "--study-dir",
        default=str(DEFAULT_STUDY_DIR),
        help="Directory containing benchmark-study CSV outputs",
    )
    parser.add_argument(
        "--output-dir",
        default=str(DEFAULT_OUTPUT_DIR),
        help="Directory to write SVG chart assets into",
    )
    args = parser.parse_args()

    study_dir = Path(args.study_dir)
    output_dir = Path(args.output_dir)
    ensure_dir(output_dir)

    run_summaries = load_csv(study_dir / "run-summaries.csv")
    comparisons = load_csv(study_dir / "comparisons.csv")
    repo_rows = load_csv(study_dir / "repo-rows.csv")

    build_latency_chart(run_summaries, output_dir)
    build_churn_chart(comparisons, output_dir)
    build_persistence_chart(repo_rows, output_dir)

    print(f"Wrote chart assets to {output_dir}")


if __name__ == "__main__":
    main()
