from __future__ import annotations

import argparse
import csv
import html
import shutil
import subprocess
from collections import Counter
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
DEFAULT_STUDY_DIR = ROOT / "target" / "benchmark-study"
DEFAULT_OUTPUT_DIR = ROOT / "docs" / "images" / "benchmark-study"
PNG_DENSITY = 288


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
BALANCED_TRADEOFF_SCENARIOS = [
    "discover-balanced-query",
    "discover-balanced-activity",
    "discover-balanced-quality",
    "discover-balanced-blended",
    "discover-balanced-blended-quality-heavy",
]
CORE_RETENTION_SCENARIOS = BALANCED_TRADEOFF_SCENARIOS
SURFACE_MIX_SCENARIOS = BALANCED_TRADEOFF_SCENARIOS
BALANCED_PARETO_SCENARIOS = [
    "discover-balanced-native",
    "discover-balanced-query",
    "discover-balanced-activity",
    "discover-balanced-quality",
    "discover-balanced-blended",
    "discover-balanced-blended-query-heavy",
    "discover-balanced-blended-activity-heavy",
    "discover-balanced-blended-quality-heavy",
]

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
    "discover-balanced-blended-query-heavy": "balanced blended query-heavy",
    "discover-balanced-blended-activity-heavy": "balanced blended activity-heavy",
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
    "discover-balanced-blended-query-heavy": "#c2410c",
    "discover-balanced-blended-activity-heavy": "#15803d",
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


def rasterize_svg(path: Path) -> None:
    convert = shutil.which("convert")
    if not convert:
        return
    subprocess.run(
        [
            convert,
            "-background",
            "white",
            "-density",
            str(PNG_DENSITY),
            str(path),
            "-alpha",
            "remove",
            "-alpha",
            "off",
            str(path.with_suffix(".png")),
        ],
        check=True,
    )

def short_balanced_label(scenario: str) -> str:
    label_map = {
        "discover-balanced-native": "native",
        "discover-balanced-query": "query",
        "discover-balanced-activity": "activity",
        "discover-balanced-quality": "quality",
        "discover-balanced-blended": "blended",
        "discover-balanced-blended-query-heavy": "q-heavy",
        "discover-balanced-blended-activity-heavy": "a-heavy",
        "discover-balanced-blended-quality-heavy": "qual-heavy",
    }
    return label_map.get(scenario, scenario)


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
    target = output_dir / "latency-profile.svg"
    write_svg(target, width, 520, "".join(body))
    rasterize_svg(target)


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
    target = output_dir / "baseline-churn.svg"
    write_svg(target, width, 640, "".join(body))
    rasterize_svg(target)


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
    target = output_dir / "persistence-leaders.svg"
    write_svg(target, width, 580, "".join(body))
    rasterize_svg(target)


def build_depth_overhead_chart(paired_effects: list[dict[str, str]], output_dir: Path) -> None:
    width = 1560
    panel_width = 730
    panels: list[str] = []
    effect_map = {
        (row["query"], row["label"]): row
        for row in paired_effects
        if row["effect_type"] == "depth-over-native"
    }
    colors = {
        "quick-native-over-baseline": "#2563eb",
        "balanced-native-over-baseline": "#0f766e",
        "deep-native-over-baseline": "#b45309",
    }
    labels = {
        "quick-native-over-baseline": "quick native",
        "balanced-native-over-baseline": "balanced native",
        "deep-native-over-baseline": "deep native",
    }
    for index, query in enumerate(PERSISTENCE_QUERIES):
        rows = []
        for label in ("quick-native-over-baseline", "balanced-native-over-baseline", "deep-native-over-baseline"):
            effect = effect_map[(query, label)]
            rows.append(
                {
                    "label": labels[label],
                    "value": float(effect["added_ms"]),
                    "color": colors[label],
                }
            )
        panels.append(
            panel_bar_chart(
                x=40 + (index * 760),
                y=96,
                width=panel_width,
                title=query,
                subtitle="Extra latency added over native best-match",
                rows=rows,
                value_formatter=lambda value: f"+{value / 1000:.1f}s",
                scale_max=60000,
                scale_ticks=[0, 15000, 30000, 45000, 60000],
            )
        )
    body = [
        svg_text(40, 44, "Depth overhead versus the native path", size=34, weight=800),
        svg_text(
            40,
            72,
            "Quick is the first meaningful latency jump, balanced is the practical middle ground, and deep roughly doubles the balanced cost.",
            size=16,
            fill="#334155",
        ),
        *panels,
    ]
    target = output_dir / "depth-overhead.svg"
    write_svg(target, width, 460, "".join(body))
    rasterize_svg(target)


def build_readme_tax_chart(paired_effects: list[dict[str, str]], output_dir: Path) -> None:
    width = 1560
    panel_width = 730
    panels: list[str] = []
    effect_map = {
        (row["query"], row["label"]): row
        for row in paired_effects
        if row["effect_type"] == "readme-tax"
    }
    colors = {
        "query-readme-tax": "#ea580c",
        "quality-readme-tax": "#2563eb",
        "blended-readme-tax": "#475569",
    }
    labels = {
        "query-readme-tax": "query + README",
        "quality-readme-tax": "quality + README",
        "blended-readme-tax": "blended + README",
    }
    for index, query in enumerate(PERSISTENCE_QUERIES):
        rows = []
        for label in ("query-readme-tax", "quality-readme-tax", "blended-readme-tax"):
            effect = effect_map[(query, label)]
            rows.append(
                {
                    "label": labels[label],
                    "value": float(effect["added_ms"]),
                    "color": colors[label],
                }
            )
        panels.append(
            panel_bar_chart(
                x=40 + (index * 760),
                y=96,
                width=panel_width,
                title=query,
                subtitle="Added latency of README enrichment over the matching balanced run",
                rows=rows,
                value_formatter=lambda value: f"+{value / 1000:.1f}s",
                scale_max=5000,
                scale_ticks=[0, 1000, 2000, 3000, 4000, 5000],
            )
        )
    body = [
        svg_text(40, 44, "README enrichment tax", size=34, weight=800),
        svg_text(
            40,
            72,
            "In this run, README enrichment added roughly three to five seconds while leaving the final top-10 overlap unchanged for the balanced modes tested.",
            size=16,
            fill="#334155",
        ),
        *panels,
    ]
    target = output_dir / "readme-tax.svg"
    write_svg(target, width, 460, "".join(body))
    rasterize_svg(target)


def build_balanced_tradeoff_chart(scenario_analysis: list[dict[str, str]], output_dir: Path) -> None:
    width = 1560
    height = 620
    panel_width = 730
    panel_height = 420
    parts: list[str] = [
        svg_text(40, 44, "Balanced-mode tradeoff map", size=34, weight=800),
        svg_text(
            40,
            72,
            "Up and left is better for baseline preservation at lower cost. Rightward drift buys more semantic movement or specialized filtering.",
            size=16,
            fill="#334155",
        ),
    ]

    label_map = {
        "discover-balanced-query": "query",
        "discover-balanced-activity": "activity",
        "discover-balanced-quality": "quality",
        "discover-balanced-blended": "blended",
        "discover-balanced-blended-quality-heavy": "quality-heavy",
    }

    for panel_index, query in enumerate(PERSISTENCE_QUERIES):
        x = 40 + (panel_index * 760)
        y = 96
        parts.append(svg_rect(x, y, panel_width, panel_height, fill="#f8fafc", stroke="#cbd5e1", stroke_width=1, rx=24))
        parts.append(svg_text(x + 28, y + 40, query, size=26, weight=700))
        parts.append(svg_text(x + 28, y + 68, "Balanced scenarios only: duration vs native-baseline fidelity", size=14, fill="#475569"))

        chart_x = x + 84
        chart_y = y + 110
        chart_width = 560
        chart_height = 240
        min_x = 25.0
        max_x = 31.0
        min_y = 0.2
        max_y = 0.7

        for tick in (25.0, 27.0, 29.0, 31.0):
            tx = chart_x + ((tick - min_x) / (max_x - min_x)) * chart_width
            parts.append(svg_line(tx, chart_y, tx, chart_y + chart_height, stroke="#cbd5e1", opacity=0.55))
            parts.append(svg_text(tx, chart_y + chart_height + 26, f"{tick:.0f}s", size=12, fill="#64748b", anchor="middle"))
        for tick in (0.2, 0.4, 0.6):
            ty = chart_y + chart_height - ((tick - min_y) / (max_y - min_y)) * chart_height
            parts.append(svg_line(chart_x, ty, chart_x + chart_width, ty, stroke="#cbd5e1", opacity=0.55))
            parts.append(svg_text(chart_x - 16, ty + 4, f"{tick:.2f}", size=12, fill="#64748b", anchor="end"))

        parts.append(svg_text(chart_x + chart_width / 2, chart_y + chart_height + 52, "duration", size=13, fill="#475569", anchor="middle"))
        parts.append(svg_text(chart_x - 46, chart_y + chart_height / 2, "Jaccard", size=13, fill="#475569", anchor="middle"))

        for scenario in BALANCED_TRADEOFF_SCENARIOS:
            row = next(item for item in scenario_analysis if item["query"] == query and item["scenario"] == scenario)
            duration_s = float(row["duration_ms"]) / 1000.0
            jaccard = float(row["top_k_jaccard"])
            px = chart_x + ((duration_s - min_x) / (max_x - min_x)) * chart_width
            py = chart_y + chart_height - ((jaccard - min_y) / (max_y - min_y)) * chart_height
            color = SCENARIO_COLORS[scenario]
            parts.append(f'<circle cx="{px:.1f}" cy="{py:.1f}" r="9" fill="{color}" stroke="#ffffff" stroke-width="2" />')
            parts.append(svg_text(px + 14, py + 5, label_map[scenario], size=13, fill="#0f172a"))

    target = output_dir / "balanced-tradeoff.svg"
    write_svg(target, width, height, "".join(parts))
    rasterize_svg(target)

def build_core_retention_chart(scenario_analysis: list[dict[str, str]], output_dir: Path) -> None:
    width = 1560
    panel_width = 730
    panels: list[str] = []
    for index, query in enumerate(PERSISTENCE_QUERIES):
        rows = []
        by_scenario = {
            row["scenario"]: row
            for row in scenario_analysis
            if row["query"] == query and row["scenario"] in CORE_RETENTION_SCENARIOS
        }
        for scenario in CORE_RETENTION_SCENARIOS:
            row = by_scenario[scenario]
            rows.append(
                {
                    "label": short_balanced_label(scenario),
                    "value": float(row["core_top5_retained"]),
                    "color": SCENARIO_COLORS[scenario],
                }
            )
        panels.append(
            panel_bar_chart(
                x=40 + (index * 760),
                y=96,
                width=panel_width,
                title=query,
                subtitle="How many of the native top five still survive in the balanced result set",
                rows=rows,
                value_formatter=lambda value: f"{int(value)}/5",
                scale_max=5,
                scale_ticks=[0, 1, 2, 3, 4, 5],
            )
        )

    body = [
        svg_text(40, 44, "Baseline core retention", size=34, weight=800),
        svg_text(
            40,
            72,
            "Quality-oriented modes keep more of the native core. Query-heavy modes trade away core retention to maximize novel results.",
            size=16,
            fill="#334155",
        ),
        *panels,
    ]
    target = output_dir / "core-retention.svg"
    write_svg(target, width, 580, "".join(body))
    rasterize_svg(target)

def build_surface_mix_chart(scenario_analysis: list[dict[str, str]], output_dir: Path) -> None:
    width = 1560
    height = 640
    panel_width = 730
    panel_height = 470
    surface_colors = {
        "name": "#ea580c",
        "description": "#2563eb",
        "topics": "#0f766e",
        "readme": "#64748b",
    }
    parts: list[str] = [
        svg_text(40, 44, "Balanced-mode surface attribution mix", size=34, weight=800),
        svg_text(
            40,
            72,
            "These shares show what each ranking mode is actually leaning on inside the explain payload. Quality shifts weight away from repo names and toward richer descriptive evidence.",
            size=16,
            fill="#334155",
        ),
    ]

    legend_x = 40
    for surface in ("name", "description", "topics", "readme"):
        parts.append(svg_rect(legend_x, 92, 18, 18, fill=surface_colors[surface], rx=4))
        parts.append(svg_text(legend_x + 28, 106, surface, size=13, fill="#334155"))
        legend_x += 140

    for panel_index, query in enumerate(PERSISTENCE_QUERIES):
        x = 40 + (panel_index * 760)
        y = 140
        parts.append(svg_rect(x, y, panel_width, panel_height, fill="#f8fafc", stroke="#cbd5e1", stroke_width=1, rx=24))
        parts.append(svg_text(x + 28, y + 40, query, size=26, weight=700))
        parts.append(svg_text(x + 28, y + 68, "Share of matched surfaces across the balanced focus set", size=14, fill="#475569"))

        label_x = x + 28
        chart_x = x + 200
        chart_width = 470
        top = y + 120

        for tick in (0.0, 0.25, 0.5, 0.75, 1.0):
            tx = chart_x + (tick * chart_width)
            parts.append(svg_line(tx, top - 10, tx, top + (len(SURFACE_MIX_SCENARIOS) * 58) - 18, stroke="#cbd5e1", opacity=0.55))
            parts.append(svg_text(tx, top - 18, f"{int(tick * 100)}%", size=12, fill="#64748b", anchor="middle"))

        by_scenario = {
            row["scenario"]: row
            for row in scenario_analysis
            if row["query"] == query and row["scenario"] in SURFACE_MIX_SCENARIOS
        }
        for row_index, scenario in enumerate(SURFACE_MIX_SCENARIOS):
            row = by_scenario[scenario]
            row_y = top + (row_index * 58)
            bar_y = row_y - 18
            parts.append(svg_text(label_x, row_y, short_balanced_label(scenario), size=14, fill="#0f172a"))
            parts.append(svg_rect(chart_x, bar_y, chart_width, 24, fill="#e2e8f0", rx=12))

            shares = [
                ("name", float(row["surface_name_share"])),
                ("description", float(row["surface_description_share"])),
                ("topics", float(row["surface_topics_share"])),
                ("readme", float(row["surface_readme_share"])),
            ]
            segment_x = chart_x
            for surface, share in shares:
                if share <= 0:
                    continue
                segment_width = chart_width * share
                parts.append(svg_rect(segment_x, bar_y, segment_width, 24, fill=surface_colors[surface], rx=12 if segment_x == chart_x else 0))
                segment_x += segment_width
            parts.append(svg_text(chart_x + chart_width + 18, row_y, f"{int(float(row['surface_breadth']))} surfaces", size=13, fill="#334155"))

    target = output_dir / "surface-mix.svg"
    write_svg(target, width, height, "".join(parts))
    rasterize_svg(target)

def build_balanced_pareto_chart(scenario_analysis: list[dict[str, str]], output_dir: Path) -> None:
    width = 1560
    height = 640
    panel_width = 730
    panel_height = 420
    parts: list[str] = [
        svg_text(40, 44, "Balanced frontier map", size=34, weight=800),
        svg_text(
            40,
            72,
            "Each point is a balanced discover mode without README or slice filters. X is latency, Y is novel results, bubble size is native top-five retention, and the white ring marks the nondominated frontier.",
            size=16,
            fill="#334155",
        ),
    ]
    label_offsets = {
        "discover-balanced-query": (16, 6),
        "discover-balanced-activity": (16, 6),
        "discover-balanced-quality": (16, 6),
        "discover-balanced-blended": (16, 6),
        "discover-balanced-native": (16, 6),
        "discover-balanced-blended-query-heavy": (16, -10),
        "discover-balanced-blended-activity-heavy": (16, -10),
        "discover-balanced-blended-quality-heavy": (16, 6),
    }

    for panel_index, query in enumerate(PERSISTENCE_QUERIES):
        x = 40 + (panel_index * 760)
        y = 116
        parts.append(svg_rect(x, y, panel_width, panel_height, fill="#f8fafc", stroke="#cbd5e1", stroke_width=1, rx=24))
        parts.append(svg_text(x + 28, y + 40, query, size=26, weight=700))
        parts.append(svg_text(x + 28, y + 68, "Lower latency with more novel results is better, but bubble size shows how much native core survives", size=14, fill="#475569"))

        candidates = [
            row
            for row in scenario_analysis
            if row["query"] == query and row["scenario"] in BALANCED_PARETO_SCENARIOS
        ]
        min_duration = min(float(row["duration_ms"]) / 1000.0 for row in candidates) - 0.3
        max_duration = max(float(row["duration_ms"]) / 1000.0 for row in candidates) + 0.3
        chart_x = x + 84
        chart_y = y + 110
        chart_width = 560
        chart_height = 240

        for tick in range(0, 7):
            ty = chart_y + chart_height - ((tick / 6) * chart_height)
            parts.append(svg_line(chart_x, ty, chart_x + chart_width, ty, stroke="#cbd5e1", opacity=0.55))
            parts.append(svg_text(chart_x - 14, ty + 4, str(tick), size=12, fill="#64748b", anchor="end"))
        x_ticks = 4
        for tick_index in range(x_ticks + 1):
            tick_value = min_duration + ((max_duration - min_duration) * tick_index / x_ticks)
            tx = chart_x + ((tick_value - min_duration) / (max_duration - min_duration)) * chart_width
            parts.append(svg_line(tx, chart_y, tx, chart_y + chart_height, stroke="#cbd5e1", opacity=0.55))
            parts.append(svg_text(tx, chart_y + chart_height + 26, f"{tick_value:.1f}s", size=12, fill="#64748b", anchor="middle"))

        parts.append(svg_text(chart_x + chart_width / 2, chart_y + chart_height + 54, "duration", size=13, fill="#475569", anchor="middle"))
        parts.append(svg_text(chart_x - 48, chart_y + chart_height / 2, "novel", size=13, fill="#475569", anchor="middle"))

        for row in candidates:
            duration_s = float(row["duration_ms"]) / 1000.0
            novel_results = float(row["novel_results"])
            retained = float(row["core_top5_retained"])
            px = chart_x + ((duration_s - min_duration) / (max_duration - min_duration)) * chart_width
            py = chart_y + chart_height - ((novel_results / 6.0) * chart_height)
            radius = 7 + (retained * 1.8)
            color = SCENARIO_COLORS[row["scenario"]]
            if row["balanced_general_frontier"] == "True":
                parts.append(f'<circle cx="{px:.1f}" cy="{py:.1f}" r="{radius + 4:.1f}" fill="none" stroke="#ffffff" stroke-width="3" />')
                parts.append(f'<circle cx="{px:.1f}" cy="{py:.1f}" r="{radius + 4:.1f}" fill="none" stroke="#0f172a" stroke-width="1.5" opacity="0.65" />')
            parts.append(f'<circle cx="{px:.1f}" cy="{py:.1f}" r="{radius:.1f}" fill="{color}" opacity="0.92" stroke="#ffffff" stroke-width="2" />')
            label_dx, label_dy = label_offsets[row["scenario"]]
            parts.append(svg_text(px + radius + label_dx, py + label_dy, short_balanced_label(row["scenario"]), size=13, fill="#0f172a"))

    target = output_dir / "balanced-pareto.svg"
    write_svg(target, width, height, "".join(parts))
    rasterize_svg(target)


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
    scenario_analysis = load_csv(study_dir / "scenario-analysis.csv")
    paired_effects = load_csv(study_dir / "paired-effects.csv")

    build_latency_chart(run_summaries, output_dir)
    build_churn_chart(comparisons, output_dir)
    build_persistence_chart(repo_rows, output_dir)
    build_depth_overhead_chart(paired_effects, output_dir)
    build_readme_tax_chart(paired_effects, output_dir)
    build_balanced_tradeoff_chart(scenario_analysis, output_dir)
    build_core_retention_chart(scenario_analysis, output_dir)
    build_surface_mix_chart(scenario_analysis, output_dir)
    build_balanced_pareto_chart(scenario_analysis, output_dir)

    print(f"Wrote chart assets to {output_dir}")


if __name__ == "__main__":
    main()
