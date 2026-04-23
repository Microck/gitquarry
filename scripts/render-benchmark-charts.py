#!/usr/bin/env -S uv run --script
# /// script
# dependencies = [
#   "altair==5.5.0",
#   "pandas==2.2.3",
#   "vl-convert-python==1.7.0",
# ]
# ///

from __future__ import annotations

import argparse
from pathlib import Path

import altair as alt
import pandas as pd

ROOT = Path(__file__).resolve().parent.parent
DEFAULT_STUDY_DIR = ROOT / "target" / "benchmark-study"
DEFAULT_OUTPUT_DIR = ROOT / "docs" / "images" / "benchmark-study"
PNG_PPI = 200
PNG_SCALE_FACTOR = 2

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
    "native-best-match": "Native baseline",
    "discover-quick-native": "Quick native",
    "discover-balanced-native": "Balanced native",
    "discover-deep-native": "Deep native",
    "discover-balanced-query-readme": "Balanced query + README",
    "discover-balanced-query": "Balanced query",
    "discover-balanced-activity": "Balanced activity",
    "discover-balanced-quality": "Balanced quality",
    "discover-balanced-blended": "Balanced blended",
    "discover-balanced-blended-query-heavy": "Query-heavy blend",
    "discover-balanced-blended-activity-heavy": "Activity-heavy blend",
    "discover-balanced-blended-quality-heavy": "Quality-heavy blend",
    "discover-balanced-activity-updated-1y": "Activity + updated 1y",
    "discover-balanced-blended-rust": "Blended + Rust",
}

SCATTER_SHORT_LABELS = {
    "discover-balanced-query": "Query",
    "discover-balanced-activity": "Activity",
    "discover-balanced-quality": "Quality",
    "discover-balanced-blended": "Blended",
    "discover-balanced-blended-query-heavy": "Q-heavy",
    "discover-balanced-blended-quality-heavy": "Quality-heavy",
    "discover-balanced-activity-updated-1y": "Updated 1y",
    "discover-balanced-blended-rust": "Rust",
    "discover-balanced-native": "Native",
}

SCENARIO_COLORS = {
    "native-best-match": "#111827",
    "discover-quick-native": "#2563eb",
    "discover-balanced-native": "#0f766e",
    "discover-deep-native": "#c2410c",
    "discover-balanced-query-readme": "#dc2626",
    "discover-balanced-query": "#ea580c",
    "discover-balanced-activity": "#16a34a",
    "discover-balanced-quality": "#2563eb",
    "discover-balanced-blended": "#475569",
    "discover-balanced-blended-query-heavy": "#c2410c",
    "discover-balanced-blended-activity-heavy": "#15803d",
    "discover-balanced-blended-quality-heavy": "#1d4ed8",
    "discover-balanced-activity-updated-1y": "#b91c1c",
    "discover-balanced-blended-rust": "#92400e",
}

SURFACE_COLORS = {
    "name": "#94a3b8",
    "description": "#2563eb",
    "topics": "#0f766e",
    "readme": "#dc2626",
}

QUERY_COLORS = {
    "api gateway": "#2563eb",
    "terminal ui": "#0f766e",
}

TRADEOFF_LABEL_OFFSETS = {
    ("api gateway", "discover-balanced-query"): (0.22, 0.0),
    ("api gateway", "discover-balanced-activity"): (0.22, 0.005),
    ("api gateway", "discover-balanced-quality"): (0.22, 0.005),
    ("api gateway", "discover-balanced-blended"): (0.22, -0.008),
    ("api gateway", "discover-balanced-blended-quality-heavy"): (0.22, 0.0),
    ("terminal ui", "discover-balanced-query"): (0.18, 0.0),
    ("terminal ui", "discover-balanced-activity"): (0.18, -0.01),
    ("terminal ui", "discover-balanced-quality"): (0.18, 0.0),
    ("terminal ui", "discover-balanced-blended"): (0.18, -0.012),
    ("terminal ui", "discover-balanced-blended-quality-heavy"): (0.18, 0.0),
}

CHURN_LABEL_OFFSETS = {
    ("api gateway", "discover-balanced-query"): (0.03, 0.0),
    ("api gateway", "discover-balanced-activity"): (0.03, 0.0),
    ("api gateway", "discover-balanced-quality"): (0.03, 0.05),
    ("api gateway", "discover-balanced-blended"): (0.03, -0.05),
    ("api gateway", "discover-balanced-blended-quality-heavy"): (0.03, 0.0),
    ("api gateway", "discover-balanced-activity-updated-1y"): (0.03, 0.0),
    ("api gateway", "discover-balanced-blended-rust"): (0.025, 0.0),
    ("terminal ui", "discover-balanced-query"): (0.03, 0.0),
    ("terminal ui", "discover-balanced-activity"): (0.03, -0.04),
    ("terminal ui", "discover-balanced-quality"): (0.03, 0.03),
    ("terminal ui", "discover-balanced-blended"): (0.03, 0.0),
    ("terminal ui", "discover-balanced-blended-quality-heavy"): (0.03, 0.0),
    ("terminal ui", "discover-balanced-activity-updated-1y"): (0.03, 0.0),
    ("terminal ui", "discover-balanced-blended-rust"): (0.025, 0.0),
}

FONT_FAMILY = "Mona Sans, DejaVu Sans, Arial, sans-serif"
TITLE_FONT_FAMILY = "Monaspace Neon, Mona Sans, DejaVu Sans Mono, monospace"
BACKGROUND = "#f6f7fb"
PANEL_BACKGROUND = "#ffffff"
GRID_COLOR = "#d7dee8"
AXIS_COLOR = "#475569"
TITLE_COLOR = "#0f172a"
SUBTITLE_COLOR = "#475569"


def load_csv(path: Path) -> pd.DataFrame:
    if not path.exists():
        raise SystemExit(f"Missing benchmark artifact: {path}")
    return pd.read_csv(path)


def scenario_label(scenario: str) -> str:
    return SCENARIO_LABELS.get(scenario, scenario.replace("-", " ").title())


def query_label(query: str) -> str:
    return query.title()


def seconds_label(value: float) -> str:
    return f"{value:.1f}s"


def percent_label(value: float) -> str:
    return f"{value * 100:.0f}%"


def add_offset_columns(
    data: pd.DataFrame,
    *,
    offset_map: dict[tuple[str, str], tuple[float, float]],
    x_field: str,
    y_field: str,
) -> pd.DataFrame:
    adjusted = data.copy()
    offsets = adjusted.apply(
        lambda row: offset_map.get((str(row["query"]), str(row["scenario"])), (0.0, 0.0)),
        axis=1,
    )
    adjusted["label_x"] = adjusted[x_field] + offsets.map(lambda item: item[0])
    adjusted["label_y"] = adjusted[y_field] + offsets.map(lambda item: item[1])
    return adjusted


def base_chart(data: pd.DataFrame, *, width: int, height: int) -> alt.Chart:
    return alt.Chart(data).properties(width=width, height=height)


def save_chart(chart: alt.TopLevelMixin, output_dir: Path, basename: str) -> None:
    svg_path = output_dir / f"{basename}.svg"
    png_path = output_dir / f"{basename}.png"
    chart.save(str(svg_path))
    chart.save(str(png_path), ppi=PNG_PPI, scale_factor=PNG_SCALE_FACTOR)


def style_chart(chart: alt.TopLevelMixin, title: str, subtitle: str) -> alt.TopLevelMixin:
    return (
        chart.properties(
            title=alt.TitleParams(
                text=title,
                subtitle=subtitle,
            )
        )
        .configure_view(stroke=None, fill=PANEL_BACKGROUND)
        .configure(background=BACKGROUND)
        .configure_title(
            font=TITLE_FONT_FAMILY,
            fontSize=22,
            color=TITLE_COLOR,
            subtitleColor=SUBTITLE_COLOR,
            subtitleFont=FONT_FAMILY,
            subtitleFontSize=13,
            anchor="start",
            dx=8,
            dy=-4,
        )
        .configure_header(
            titleFont=TITLE_FONT_FAMILY,
            titleFontSize=16,
            labelFont=FONT_FAMILY,
            labelFontSize=13,
            titleColor=TITLE_COLOR,
            labelColor=AXIS_COLOR,
            labelPadding=8,
        )
        .configure_axis(
            labelFont=FONT_FAMILY,
            titleFont=FONT_FAMILY,
            labelColor=AXIS_COLOR,
            titleColor=AXIS_COLOR,
            gridColor=GRID_COLOR,
            gridOpacity=0.75,
            domainColor="#c7d2e0",
            tickColor="#c7d2e0",
            labelFontSize=12,
            titleFontSize=12,
            titlePadding=10,
        )
        .configure_legend(
            labelFont=FONT_FAMILY,
            titleFont=FONT_FAMILY,
            labelColor=AXIS_COLOR,
            titleColor=AXIS_COLOR,
            symbolType="circle",
            symbolSize=120,
            orient="top",
            padding=6,
            offset=8,
        )
    )


def build_latency_chart(run_summaries: pd.DataFrame, output_dir: Path) -> None:
    data = run_summaries[run_summaries["scenario"].isin(LATENCY_SCENARIOS)].copy()
    data["query_label"] = data["query"].map(query_label)
    data["scenario_label"] = data["scenario"].map(scenario_label)
    data["duration_s"] = data["duration_ms"] / 1000
    data["value_label"] = data["duration_s"].map(seconds_label)

    chart = base_chart(data, width=390, height=180)
    bars = chart.mark_bar(cornerRadiusEnd=8, size=28).encode(
        y=alt.Y(
            "scenario_label:N",
            sort=[scenario_label(item) for item in LATENCY_SCENARIOS],
            title=None,
        ),
        x=alt.X("duration_s:Q", title="Wall time (seconds)"),
        color=alt.Color(
            "scenario:N",
            scale=alt.Scale(
                domain=list(SCENARIO_COLORS.keys()),
                range=list(SCENARIO_COLORS.values()),
            ),
            legend=None,
        ),
        tooltip=[
            alt.Tooltip("query_label:N", title="Query"),
            alt.Tooltip("scenario_label:N", title="Scenario"),
            alt.Tooltip("duration_s:Q", title="Duration (s)", format=".2f"),
        ],
    )
    labels = chart.mark_text(
        align="left",
        baseline="middle",
        dx=8,
        color=TITLE_COLOR,
        font=FONT_FAMILY,
        fontSize=12,
        fontWeight=600,
    ).encode(
        y=alt.Y("scenario_label:N", sort=[scenario_label(item) for item in LATENCY_SCENARIOS]),
        x=alt.X("duration_s:Q"),
        text="value_label:N",
    )

    faceted = alt.layer(bars, labels).facet(
        column=alt.Column("query_label:N", title=None, sort=[query_label(item) for item in PERSISTENCE_QUERIES])
    )
    chart_out = style_chart(
        faceted,
        "Latency profile",
        "Native remains the only sub-second path. Balanced is the main analysis tier and deep roughly doubles that tax.",
    )
    save_chart(chart_out, output_dir, "latency-profile")


def build_depth_overhead_chart(paired_effects: pd.DataFrame, output_dir: Path) -> None:
    data = paired_effects[paired_effects["effect_type"] == "depth-over-native"].copy()
    depth_order = ["quick", "balanced", "deep"]
    depth_labels = {"quick": "Quick", "balanced": "Balanced", "deep": "Deep"}
    data["query_label"] = data["query"].map(query_label)
    data["depth"] = data["compare_scenario"].str.extract(r"discover-(quick|balanced|deep)-")[0]
    data["depth_label"] = data["depth"].map(depth_labels)
    data["added_s"] = data["added_ms"] / 1000
    data["value_label"] = data["added_s"].map(lambda value: f"+{value:.1f}s")

    chart = base_chart(data, width=360, height=200)
    line = chart.mark_line(strokeWidth=3, point=alt.OverlayMarkDef(size=140, filled=True)).encode(
        x=alt.X("depth_label:N", sort=[depth_labels[item] for item in depth_order], title=None),
        y=alt.Y("added_s:Q", title="Added latency over native (seconds)"),
        color=alt.Color(
            "query:N",
            scale=alt.Scale(domain=list(QUERY_COLORS.keys()), range=list(QUERY_COLORS.values())),
            legend=alt.Legend(title=None),
        ),
        tooltip=[
            alt.Tooltip("query_label:N", title="Query"),
            alt.Tooltip("depth_label:N", title="Depth"),
            alt.Tooltip("added_s:Q", title="Added seconds", format=".2f"),
        ],
    )
    labels = chart.mark_text(
        dy=-14,
        font=FONT_FAMILY,
        fontSize=12,
        fontWeight=600,
        color=TITLE_COLOR,
    ).encode(
        x=alt.X("depth_label:N", sort=[depth_labels[item] for item in depth_order]),
        y=alt.Y("added_s:Q"),
        text="value_label:N",
        detail="query:N",
    )

    chart_out = style_chart(
        alt.layer(line, labels),
        "Depth overhead versus native",
        "Quick is already expensive. Balanced is the practical tradeoff tier. Deep should stay an explicit heavy-recall choice.",
    )
    save_chart(chart_out, output_dir, "depth-overhead")


def build_readme_tax_chart(paired_effects: pd.DataFrame, output_dir: Path) -> None:
    data = paired_effects[paired_effects["effect_type"] == "readme-tax"].copy()
    data["query_label"] = data["query"].map(query_label)
    data["rank_mode"] = data["label"].str.replace("-readme-tax", "", regex=False).str.replace("-", " ").str.title()
    data["added_s"] = data["added_ms"] / 1000
    data["value_label"] = data["added_s"].map(lambda value: f"+{value:.1f}s")

    chart = base_chart(data, width=320, height=210)
    bars = chart.mark_bar(cornerRadiusTopLeft=8, cornerRadiusTopRight=8, width=48).encode(
        x=alt.X("rank_mode:N", title=None, sort=["Query", "Quality", "Blended"]),
        y=alt.Y("added_s:Q", title="README overhead (seconds)"),
        color=alt.Color(
            "rank_mode:N",
            scale=alt.Scale(
                domain=["Query", "Quality", "Blended"],
                range=["#ea580c", "#2563eb", "#475569"],
            ),
            legend=None,
        ),
        tooltip=[
            alt.Tooltip("query_label:N", title="Query"),
            alt.Tooltip("rank_mode:N", title="Base rank"),
            alt.Tooltip("added_s:Q", title="Added seconds", format=".2f"),
        ],
    )
    labels = chart.mark_text(
        dy=-10,
        font=FONT_FAMILY,
        fontSize=12,
        fontWeight=600,
        color=TITLE_COLOR,
    ).encode(
        x=alt.X("rank_mode:N", sort=["Query", "Quality", "Blended"]),
        y=alt.Y("added_s:Q"),
        text="value_label:N",
    )

    faceted = alt.layer(bars, labels).facet(
        column=alt.Column("query_label:N", title=None, sort=[query_label(item) for item in PERSISTENCE_QUERIES])
    )
    chart_out = style_chart(
        faceted,
        "README enrichment tax",
        "README evidence costs another 3 to 5 seconds in this run and did not improve top-10 overlap.",
    )
    save_chart(chart_out, output_dir, "readme-tax")


def build_balanced_tradeoff_chart(scenario_analysis: pd.DataFrame, output_dir: Path) -> None:
    data = scenario_analysis[scenario_analysis["scenario"].isin(BALANCED_TRADEOFF_SCENARIOS)].copy()
    data["query_label"] = data["query"].map(query_label)
    data["scenario_label"] = data["scenario"].map(scenario_label)
    data["duration_s"] = data["duration_ms"] / 1000
    data["label_text"] = data["scenario"].map(lambda value: SCATTER_SHORT_LABELS.get(value, scenario_label(value)))
    data = add_offset_columns(
        data,
        offset_map=TRADEOFF_LABEL_OFFSETS,
        x_field="duration_s",
        y_field="top_k_jaccard",
    )

    chart = base_chart(data, width=400, height=245)
    points = chart.mark_circle(opacity=0.92, stroke="#ffffff", strokeWidth=1.5).encode(
        x=alt.X("duration_s:Q", title="Wall time (seconds)"),
        y=alt.Y("top_k_jaccard:Q", title="Top-10 Jaccard versus native", scale=alt.Scale(domain=[0, 1])),
        size=alt.Size("median_stars:Q", title="Median stars", scale=alt.Scale(range=[120, 1600])),
        color=alt.Color(
            "scenario:N",
            scale=alt.Scale(domain=list(SCENARIO_COLORS.keys()), range=list(SCENARIO_COLORS.values())),
            legend=None,
        ),
        tooltip=[
            alt.Tooltip("query_label:N", title="Query"),
            alt.Tooltip("scenario_label:N", title="Scenario"),
            alt.Tooltip("duration_s:Q", title="Duration (s)", format=".2f"),
            alt.Tooltip("top_k_jaccard:Q", title="Jaccard", format=".4f"),
            alt.Tooltip("novel_results:Q", title="Novel repos"),
            alt.Tooltip("median_stars:Q", title="Median stars", format=".0f"),
        ],
    )
    labels = chart.mark_text(
        baseline="middle",
        font=FONT_FAMILY,
        fontSize=10,
        color=TITLE_COLOR,
    ).encode(
        x="label_x:Q",
        y="label_y:Q",
        text="label_text:N",
    )
    faceted = alt.layer(points, labels).facet(
        column=alt.Column("query_label:N", title=None, sort=[query_label(item) for item in PERSISTENCE_QUERIES])
    )
    chart_out = style_chart(
        faceted,
        "Balanced-mode tradeoff map",
        "Balanced quality is the safest non-native default. Query buys novelty by giving up much more of the baseline core.",
    )
    save_chart(chart_out, output_dir, "balanced-tradeoff")


def build_balanced_pareto_chart(scenario_analysis: pd.DataFrame, output_dir: Path) -> None:
    data = scenario_analysis[scenario_analysis["scenario"].isin(BALANCED_PARETO_SCENARIOS)].copy()
    data["query_label"] = data["query"].map(query_label)
    data["scenario_label"] = data["scenario"].map(scenario_label)
    data["duration_s"] = data["duration_ms"] / 1000
    data["frontier_label"] = data["balanced_general_frontier"].map(lambda value: "Frontier" if value else "Off frontier")
    data["label_text"] = data.apply(
        lambda row: SCATTER_SHORT_LABELS.get(str(row["scenario"]), row["scenario_label"])
        if bool(row["balanced_general_frontier"])
        else "",
        axis=1,
    )

    chart = base_chart(data, width=400, height=245)
    points = chart.mark_circle(stroke="#ffffff", strokeWidth=1.4).encode(
        x=alt.X("duration_s:Q", title="Wall time (seconds)"),
        y=alt.Y("top_k_jaccard:Q", title="Top-10 Jaccard versus native", scale=alt.Scale(domain=[0, 1])),
        size=alt.Size("novel_results:Q", title="Novel repos", scale=alt.Scale(range=[130, 1200])),
        color=alt.Color(
            "frontier_label:N",
            scale=alt.Scale(domain=["Frontier", "Off frontier"], range=["#0f766e", "#cbd5e1"]),
            legend=alt.Legend(title=None),
        ),
        opacity=alt.condition(alt.datum.balanced_general_frontier, alt.value(0.96), alt.value(0.42)),
        tooltip=[
            alt.Tooltip("query_label:N", title="Query"),
            alt.Tooltip("scenario_label:N", title="Scenario"),
            alt.Tooltip("duration_s:Q", title="Duration (s)", format=".2f"),
            alt.Tooltip("top_k_jaccard:Q", title="Jaccard", format=".4f"),
            alt.Tooltip("novel_results:Q", title="Novel repos"),
            alt.Tooltip("frontier_label:N", title="Status"),
        ],
    )
    labels = chart.mark_text(
        align="left",
        baseline="middle",
        dx=10,
        font=FONT_FAMILY,
        fontSize=11,
        color=TITLE_COLOR,
    ).encode(
        x="duration_s:Q",
        y="top_k_jaccard:Q",
        text="label_text:N",
    )
    faceted = alt.layer(points, labels).facet(
        column=alt.Column("query_label:N", title=None, sort=[query_label(item) for item in PERSISTENCE_QUERIES])
    )
    chart_out = style_chart(
        faceted,
        "Balanced frontier map",
        "Frontier scenarios are the only balanced options worth keeping as tradeoff leaders for their query family.",
    )
    save_chart(chart_out, output_dir, "balanced-pareto")


def build_core_retention_chart(scenario_analysis: pd.DataFrame, output_dir: Path) -> None:
    data = scenario_analysis[scenario_analysis["scenario"].isin(CORE_RETENTION_SCENARIOS)].copy()
    data["query_label"] = data["query"].map(query_label)
    data["scenario_label"] = data["scenario"].map(scenario_label)
    rates = pd.concat(
        [
            data[["query_label", "scenario_label", "core_top3_rate"]].rename(columns={"core_top3_rate": "retention"}),
            data[["query_label", "scenario_label", "core_top5_rate"]].rename(columns={"core_top5_rate": "retention"}),
        ],
        ignore_index=True,
    )
    rates["cohort"] = ["Native top 3"] * len(data) + ["Native top 5"] * len(data)
    rates["retention_label"] = rates["retention"].map(percent_label)

    chart = base_chart(rates, width=250, height=210)
    heatmap = chart.mark_rect(cornerRadius=6).encode(
        x=alt.X("cohort:N", title=None, sort=["Native top 3", "Native top 5"]),
        y=alt.Y(
            "scenario_label:N",
            title=None,
            sort=[scenario_label(item) for item in CORE_RETENTION_SCENARIOS],
        ),
        color=alt.Color(
            "retention:Q",
            title="Retention",
            scale=alt.Scale(domain=[0, 1], range=["#eff6ff", "#2563eb"]),
        ),
        tooltip=[
            alt.Tooltip("query_label:N", title="Query"),
            alt.Tooltip("scenario_label:N", title="Scenario"),
            alt.Tooltip("cohort:N", title="Baseline slice"),
            alt.Tooltip("retention:Q", title="Retention", format=".2%"),
        ],
    )
    labels = chart.mark_text(font=FONT_FAMILY, fontSize=12, fontWeight=600).encode(
        x=alt.X("cohort:N", sort=["Native top 3", "Native top 5"]),
        y=alt.Y("scenario_label:N", sort=[scenario_label(item) for item in CORE_RETENTION_SCENARIOS]),
        text="retention_label:N",
        color=alt.condition(alt.datum.retention > 0.55, alt.value("white"), alt.value(TITLE_COLOR)),
    )
    faceted = alt.layer(heatmap, labels).facet(
        column=alt.Column("query_label:N", title=None, sort=[query_label(item) for item in PERSISTENCE_QUERIES])
    )
    chart_out = style_chart(
        faceted,
        "Baseline core retention",
        "Quality retains the native core far better than query, which is why it is the default advanced recommendation.",
    )
    save_chart(chart_out, output_dir, "core-retention")


def build_surface_mix_chart(scenario_analysis: pd.DataFrame, output_dir: Path) -> None:
    data = scenario_analysis[scenario_analysis["scenario"].isin(SURFACE_MIX_SCENARIOS)].copy()
    data["query_label"] = data["query"].map(query_label)
    data["scenario_label"] = data["scenario"].map(scenario_label)
    long = data.melt(
        id_vars=["query_label", "scenario_label"],
        value_vars=[
            "surface_name_share",
            "surface_description_share",
            "surface_topics_share",
            "surface_readme_share",
        ],
        var_name="surface_key",
        value_name="share",
    )
    long["surface"] = long["surface_key"].map(
        {
            "surface_name_share": "name",
            "surface_description_share": "description",
            "surface_topics_share": "topics",
            "surface_readme_share": "readme",
        }
    )

    chart = base_chart(long, width=390, height=210)
    bars = chart.mark_bar(cornerRadiusEnd=6, cornerRadiusTopRight=6).encode(
        y=alt.Y(
            "scenario_label:N",
            sort=[scenario_label(item) for item in SURFACE_MIX_SCENARIOS],
            title=None,
        ),
        x=alt.X("share:Q", title="Share of matched surfaces", stack="normalize", axis=alt.Axis(format="%")),
        color=alt.Color(
            "surface:N",
            scale=alt.Scale(domain=list(SURFACE_COLORS.keys()), range=list(SURFACE_COLORS.values())),
            legend=alt.Legend(title=None),
        ),
        order=alt.Order("surface:N"),
        tooltip=[
            alt.Tooltip("query_label:N", title="Query"),
            alt.Tooltip("scenario_label:N", title="Scenario"),
            alt.Tooltip("surface:N", title="Surface"),
            alt.Tooltip("share:Q", title="Share", format=".2%"),
        ],
    )

    faceted = bars.facet(
        column=alt.Column("query_label:N", title=None, sort=[query_label(item) for item in PERSISTENCE_QUERIES])
    )
    chart_out = style_chart(
        faceted,
        "Surface attribution mix",
        "Quality leans less on repository names and more on descriptions and topics, which helps explain its safer curation profile.",
    )
    save_chart(chart_out, output_dir, "surface-mix")


def build_churn_chart(comparisons: pd.DataFrame, output_dir: Path) -> None:
    data = comparisons[comparisons["scenario"].isin(CHURN_SCENARIOS)].copy()
    data["query_label"] = data["query"].map(query_label)
    data["scenario_label"] = data["scenario"].map(scenario_label)
    data["label_text"] = data["scenario"].map(lambda value: SCATTER_SHORT_LABELS.get(value, scenario_label(value)))
    data = add_offset_columns(
        data,
        offset_map=CHURN_LABEL_OFFSETS,
        x_field="top_k_jaccard",
        y_field="novel_results",
    )

    chart = base_chart(data, width=400, height=240)
    points = chart.mark_circle(size=260, opacity=0.9, stroke="#ffffff", strokeWidth=1.5).encode(
        x=alt.X("top_k_jaccard:Q", title="Top-10 Jaccard versus native", scale=alt.Scale(domain=[0, 1])),
        y=alt.Y("novel_results:Q", title="Novel repositories in top 10", scale=alt.Scale(domain=[0, 10])),
        color=alt.Color(
            "scenario:N",
            scale=alt.Scale(domain=list(SCENARIO_COLORS.keys()), range=list(SCENARIO_COLORS.values())),
            legend=None,
        ),
        tooltip=[
            alt.Tooltip("query_label:N", title="Query"),
            alt.Tooltip("scenario_label:N", title="Scenario"),
            alt.Tooltip("top_k_jaccard:Q", title="Jaccard", format=".4f"),
            alt.Tooltip("novel_results:Q", title="Novel repos"),
            alt.Tooltip("avg_abs_rank_shift_common:Q", title="Avg. rank shift", format=".2f"),
        ],
    )
    labels = chart.mark_text(
        baseline="middle",
        font=FONT_FAMILY,
        fontSize=10,
        color=TITLE_COLOR,
    ).encode(
        x="label_x:Q",
        y="label_y:Q",
        text="label_text:N",
    )
    faceted = alt.layer(points, labels).facet(
        column=alt.Column("query_label:N", title=None, sort=[query_label(item) for item in PERSISTENCE_QUERIES])
    )
    chart_out = style_chart(
        faceted,
        "Scenario churn versus native",
        "Recency and language slices behave like different intents, not like small tweaks to the default search path.",
    )
    save_chart(chart_out, output_dir, "baseline-churn")


def build_persistence_chart(repo_rows: pd.DataFrame, output_dir: Path) -> None:
    data = repo_rows[repo_rows["query"].isin(PERSISTENCE_QUERIES)].copy()
    counts = (
        data.groupby(["query", "full_name"], as_index=False)
        .size()
        .rename(columns={"size": "appearances"})
        .sort_values(["query", "appearances", "full_name"], ascending=[True, False, True])
    )
    top = counts.groupby("query", group_keys=False).head(10).copy()
    top["query_label"] = top["query"].map(query_label)
    top["label_text"] = top["appearances"].astype(int).astype(str)

    chart = base_chart(top, width=400, height=280)
    bars = chart.mark_bar(cornerRadiusEnd=8, size=22).encode(
        y=alt.Y("full_name:N", title=None, sort="-x"),
        x=alt.X("appearances:Q", title="Number of scenarios the repo survived"),
        color=alt.Color(
            "query:N",
            scale=alt.Scale(domain=list(QUERY_COLORS.keys()), range=list(QUERY_COLORS.values())),
            legend=None,
        ),
        tooltip=[
            alt.Tooltip("query_label:N", title="Query"),
            alt.Tooltip("full_name:N", title="Repository"),
            alt.Tooltip("appearances:Q", title="Appearances"),
        ],
    )
    labels = chart.mark_text(
        align="left",
        baseline="middle",
        dx=8,
        font=FONT_FAMILY,
        fontSize=12,
        fontWeight=600,
        color=TITLE_COLOR,
    ).encode(
        y=alt.Y("full_name:N", sort="-x"),
        x=alt.X("appearances:Q"),
        text="label_text:N",
    )
    faceted = alt.layer(bars, labels).facet(
        column=alt.Column("query_label:N", title=None, sort=[query_label(item) for item in PERSISTENCE_QUERIES])
    )
    chart_out = style_chart(
        faceted,
        "Persistence leaders across the study",
        "These repositories survive mode, ranking, and slice changes, which makes them useful anchors for demos and explanation examples.",
    )
    save_chart(chart_out, output_dir, "persistence-leaders")


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Render high-resolution benchmark charts directly from benchmark study CSV artifacts."
    )
    parser.add_argument(
        "--study-dir",
        default=str(DEFAULT_STUDY_DIR),
        help="Directory containing benchmark CSV artifacts",
    )
    parser.add_argument(
        "--output-dir",
        default=str(DEFAULT_OUTPUT_DIR),
        help="Directory to write SVG and PNG chart assets into",
    )
    args = parser.parse_args()

    alt.data_transformers.disable_max_rows()

    study_dir = Path(args.study_dir)
    output_dir = Path(args.output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    run_summaries = load_csv(study_dir / "run-summaries.csv")
    comparisons = load_csv(study_dir / "comparisons.csv")
    paired_effects = load_csv(study_dir / "paired-effects.csv")
    scenario_analysis = load_csv(study_dir / "scenario-analysis.csv")
    repo_rows = load_csv(study_dir / "repo-rows.csv")

    build_latency_chart(run_summaries, output_dir)
    build_depth_overhead_chart(paired_effects, output_dir)
    build_readme_tax_chart(paired_effects, output_dir)
    build_balanced_tradeoff_chart(scenario_analysis, output_dir)
    build_balanced_pareto_chart(scenario_analysis, output_dir)
    build_core_retention_chart(scenario_analysis, output_dir)
    build_surface_mix_chart(scenario_analysis, output_dir)
    build_churn_chart(comparisons, output_dir)
    build_persistence_chart(repo_rows, output_dir)

    print(f"Rendered benchmark charts to {output_dir}")


if __name__ == "__main__":
    main()
