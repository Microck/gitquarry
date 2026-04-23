#!/usr/bin/env python3

from __future__ import annotations

import argparse
import csv
import json
import os
import shutil
import statistics
import subprocess
import sys
import time
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parent.parent
DEFAULT_OUTPUT_DIR = ROOT / "target" / "benchmark-study"
DEFAULT_LIMIT = 10
DEFAULT_QUERIES = ["api gateway", "terminal ui"]
README_BASE_PAIRS = [
    ("discover-balanced-query", "discover-balanced-query-readme", "query-readme-tax"),
    ("discover-balanced-quality", "discover-balanced-quality-readme", "quality-readme-tax"),
    ("discover-balanced-blended", "discover-balanced-blended-readme", "blended-readme-tax"),
]
WEIGHT_BASE_PAIRS = [
    ("discover-balanced-blended", "discover-balanced-blended-query-heavy", "query-heavy-shift"),
    ("discover-balanced-blended", "discover-balanced-blended-activity-heavy", "activity-heavy-shift"),
    ("discover-balanced-blended", "discover-balanced-blended-quality-heavy", "quality-heavy-shift"),
]
SLICE_BASE_PAIRS = [
    ("native-rust", "discover-balanced-blended-rust", "rust-discover-tax"),
    ("native-updated-1y", "discover-balanced-activity-updated-1y", "updated-1y-discover-tax"),
]


@dataclass(frozen=True)
class Scenario:
    name: str
    label: str
    group: str
    args: tuple[str, ...]
    expect_explain: bool


def discover(
    *,
    depth: str,
    rank: str,
    readme: bool = False,
    extra: tuple[str, ...] = (),
    label_suffix: str = "",
) -> Scenario:
    base_name = f"discover-{depth}-{rank}"
    base_label = f"discover/{depth}/{rank}"
    if readme:
        base_name += "-readme"
        base_label += "+readme"
    if label_suffix:
        base_name += f"-{label_suffix}"
        base_label += f"+{label_suffix}"
    args = ["--mode", "discover", "--depth", depth, "--rank", rank, "--explain"]
    if readme:
        args.append("--readme")
    args.extend(extra)
    return Scenario(
        name=base_name,
        label=base_label,
        group="discover",
        args=tuple(args),
        expect_explain=True,
    )


def native(*, name: str, label: str, extra: tuple[str, ...] = ()) -> Scenario:
    return Scenario(
        name=name,
        label=label,
        group="native",
        args=tuple(extra),
        expect_explain=False,
    )


def build_scenarios() -> list[Scenario]:
    scenarios: list[Scenario] = [
        native(name="native-best-match", label="native/best-match"),
        native(
            name="native-stars",
            label="native/stars",
            extra=("--sort", "stars"),
        ),
        native(
            name="native-updated",
            label="native/updated",
            extra=("--sort", "updated"),
        ),
    ]

    for depth in ("quick", "balanced", "deep"):
        for rank in ("native", "query", "activity", "quality", "blended"):
            scenarios.append(discover(depth=depth, rank=rank))

    scenarios.extend(
        [
            discover(depth="balanced", rank="query", readme=True),
            discover(depth="balanced", rank="quality", readme=True),
            discover(depth="balanced", rank="blended", readme=True),
            discover(
                depth="balanced",
                rank="blended",
                extra=(
                    "--weight-query",
                    "2.0",
                    "--weight-activity",
                    "0.5",
                    "--weight-quality",
                    "0.5",
                ),
                label_suffix="query-heavy",
            ),
            discover(
                depth="balanced",
                rank="blended",
                extra=(
                    "--weight-query",
                    "0.5",
                    "--weight-activity",
                    "2.0",
                    "--weight-quality",
                    "0.5",
                ),
                label_suffix="activity-heavy",
            ),
            discover(
                depth="balanced",
                rank="blended",
                extra=(
                    "--weight-query",
                    "0.5",
                    "--weight-activity",
                    "0.5",
                    "--weight-quality",
                    "2.0",
                ),
                label_suffix="quality-heavy",
            ),
            native(
                name="native-rust",
                label="native/rust",
                extra=("--language", "Rust"),
            ),
            discover(
                depth="balanced",
                rank="blended",
                extra=("--language", "Rust"),
                label_suffix="rust",
            ),
            native(
                name="native-updated-1y",
                label="native/updated-1y",
                extra=("--updated-within", "1y"),
            ),
            discover(
                depth="balanced",
                rank="activity",
                extra=("--updated-within", "1y"),
                label_suffix="updated-1y",
            ),
        ]
    )
    return scenarios


def now_iso() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat()


def obtain_token() -> str:
    for key in sorted(os.environ):
        if key.startswith("GITQUARRY_TOKEN") and os.environ[key]:
            return os.environ[key]

    gh = shutil.which("gh")
    if gh:
        token_result = subprocess.run(
            [gh, "auth", "token"],
            capture_output=True,
            text=True,
            check=False,
        )
        if token_result.returncode == 0:
            token = token_result.stdout.strip()
            if token:
                return token

        status_result = subprocess.run(
            [gh, "auth", "status", "--show-token"],
            capture_output=True,
            text=True,
            check=False,
        )
        if status_result.returncode == 0:
            status_output = "\n".join(
                part for part in (status_result.stdout, status_result.stderr) if part
            )
            for line in status_output.splitlines():
                if "Token:" in line:
                    token = line.split("Token:", 1)[1].strip()
                    if token and not set(token) <= {"*"}:
                        return token

    raise SystemExit(
        "No GitHub token found. Set GITQUARRY_TOKEN or authenticate gh before running the study."
    )


def ensure_binary(binary: Path) -> None:
    if binary.exists():
        return
    subprocess.run(["cargo", "build"], cwd=ROOT, check=True)


def slugify_query(query: str) -> str:
    return "-".join(query.lower().split())


def parse_dt(value: str | None) -> datetime | None:
    if not value:
        return None
    return datetime.fromisoformat(value.replace("Z", "+00:00"))


def median_or_zero(values: list[float]) -> float:
    return statistics.median(values) if values else 0.0

def to_float(value: Any, default: float = 0.0) -> float:
    if value in ("", None):
        return default
    return float(value)

def format_duration_ms(value: float) -> str:
    if value >= 1000:
        return f"{value / 1000:.1f}s"
    return f"{int(value)}ms"

def parse_scenario_metadata(scenario_name: str) -> dict[str, Any]:
    depth = "native"
    rank_family = "native"
    readme_enabled = False
    weight_profile = "default"
    slice_profile = "default"

    if scenario_name.startswith("discover-"):
        parts = scenario_name.split("-")
        if len(parts) >= 3:
            depth = parts[1]
            rank_family = parts[2]
        readme_enabled = "readme" in parts
        if "query-heavy" in scenario_name:
            weight_profile = "query-heavy"
        elif "activity-heavy" in scenario_name:
            weight_profile = "activity-heavy"
        elif "quality-heavy" in scenario_name:
            weight_profile = "quality-heavy"
        if scenario_name.endswith("-rust"):
            slice_profile = "rust"
        elif scenario_name.endswith("-updated-1y"):
            slice_profile = "updated-1y"
    else:
        if scenario_name.endswith("-rust"):
            slice_profile = "rust"
        elif scenario_name.endswith("-updated-1y"):
            slice_profile = "updated-1y"

    return {
        "depth": depth,
        "rank_family": rank_family,
        "readme_enabled": readme_enabled,
        "weight_profile": weight_profile,
        "slice_profile": slice_profile,
    }

def build_baseline_core_sets(
    repo_rows: list[dict[str, Any]],
    queries: list[str],
) -> dict[str, dict[int, set[str]]]:
    baseline_core: dict[str, dict[int, set[str]]] = {}
    for query in queries:
        baseline_rows = sorted(
            [
                row
                for row in repo_rows
                if row["query"] == query and row["scenario"] == "native-best-match"
            ],
            key=lambda item: item["rank_position"],
        )
        baseline_core[query] = {
            3: {row["full_name"] for row in baseline_rows[:3]},
            5: {row["full_name"] for row in baseline_rows[:5]},
        }
    return baseline_core

def surface_share(surface_counts: dict[str, int], surface: str) -> float:
    total = sum(surface_counts.values())
    if total <= 0:
        return 0.0
    return round(surface_counts.get(surface, 0) / total, 4)


def repo_row(
    *,
    query: str,
    scenario: Scenario,
    run_index: int,
    duration_ms: int,
    repo: dict[str, Any],
    rank_position: int,
    started_at: str,
) -> dict[str, Any]:
    explain = repo.get("explain") or {}
    updated_at = parse_dt(repo.get("updated_at"))
    age_days = (
        (datetime.now(timezone.utc) - updated_at).days if updated_at is not None else None
    )
    return {
        "query": query,
        "scenario": scenario.name,
        "scenario_label": scenario.label,
        "group": scenario.group,
        "run_index": run_index,
        "started_at": started_at,
        "duration_ms": duration_ms,
        "rank_position": rank_position,
        "full_name": repo.get("full_name", ""),
        "stars": repo.get("stargazers_count", 0),
        "forks": repo.get("forks_count", 0),
        "language": repo.get("language") or "",
        "archived": bool(repo.get("archived", False)),
        "is_template": bool(repo.get("is_template", False)),
        "topics_count": len(repo.get("topics") or []),
        "has_readme": bool((repo.get("readme") or "").strip()),
        "has_explain": bool(repo.get("explain")),
        "updated_age_days": age_days if age_days is not None else "",
        "query_score": explain.get("query", ""),
        "activity_score": explain.get("activity", ""),
        "quality_score": explain.get("quality", ""),
        "blended_score": explain.get("blended", ""),
        "matched_surfaces": "|".join(explain.get("matched_surfaces") or []),
    }


def write_csv(path: Path, rows: list[dict[str, Any]]) -> None:
    if not rows:
        path.write_text("", encoding="utf-8")
        return
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(rows[0].keys()))
        writer.writeheader()
        writer.writerows(rows)


def compare_lists(left: list[str], right: list[str]) -> dict[str, Any]:
    left_set = set(left)
    right_set = set(right)
    common = left_set & right_set
    union = left_set | right_set
    overlap = len(common)
    jaccard = overlap / len(union) if union else 1.0
    novelty = len(right_set - left_set)
    rank_shift_values: list[int] = []
    for item in common:
        left_rank = left.index(item) + 1
        right_rank = right.index(item) + 1
        rank_shift_values.append(abs(left_rank - right_rank))
    avg_rank_shift = (
        round(sum(rank_shift_values) / len(rank_shift_values), 3)
        if rank_shift_values
        else None
    )
    return {
        "top_k_overlap": overlap,
        "top_k_jaccard": round(jaccard, 4),
        "novel_results": novelty,
        "avg_abs_rank_shift_common": avg_rank_shift,
    }


def summarize_run(
    *,
    query: str,
    scenario: Scenario,
    duration_ms: int,
    payload: dict[str, Any],
    started_at: str,
) -> tuple[dict[str, Any], list[dict[str, Any]]]:
    items = payload.get("items", [])
    repo_rows = [
        repo_row(
            query=query,
            scenario=scenario,
            run_index=index,
            duration_ms=duration_ms,
            repo=repo,
            rank_position=index + 1,
            started_at=started_at,
        )
        for index, repo in enumerate(items)
    ]

    stars = [row["stars"] for row in repo_rows]
    forks = [row["forks"] for row in repo_rows]
    updated_age_days = [
        row["updated_age_days"]
        for row in repo_rows
        if isinstance(row["updated_age_days"], int)
    ]
    matched_surface_counter: dict[str, int] = {}
    for row in repo_rows:
        for surface in filter(None, str(row["matched_surfaces"]).split("|")):
            matched_surface_counter[surface] = matched_surface_counter.get(surface, 0) + 1

    summary = {
        "query": query,
        "scenario": scenario.name,
        "scenario_label": scenario.label,
        "group": scenario.group,
        "started_at": started_at,
        "duration_ms": duration_ms,
        "compiled_query": payload.get("compiled_query", ""),
        "mode": payload.get("mode", ""),
        "rank": payload.get("rank", ""),
        "result_count": len(items),
        "total_count": payload.get("total_count", len(items)),
        "median_stars": round(median_or_zero(stars), 3),
        "median_forks": round(median_or_zero(forks), 3),
        "median_updated_age_days": round(median_or_zero(updated_age_days), 3),
        "language_diversity": len({row["language"] for row in repo_rows if row["language"]}),
        "readme_coverage": round(
            sum(1 for row in repo_rows if row["has_readme"]) / len(repo_rows), 4
        )
        if repo_rows
        else 0.0,
        "explain_coverage": round(
            sum(1 for row in repo_rows if row["has_explain"]) / len(repo_rows), 4
        )
        if repo_rows
        else 0.0,
        "matched_surface_counts": matched_surface_counter,
    }
    return summary, repo_rows

def build_scenario_analysis(
    *,
    run_summaries: list[dict[str, Any]],
    comparisons: list[dict[str, Any]],
    repo_rows: list[dict[str, Any]],
    queries: list[str],
) -> list[dict[str, Any]]:
    run_index = {(row["query"], row["scenario"]): row for row in run_summaries}
    comparison_index = {(row["query"], row["scenario"]): row for row in comparisons}
    repo_index: dict[tuple[str, str], list[dict[str, Any]]] = {}
    for row in repo_rows:
        repo_index.setdefault((row["query"], row["scenario"]), []).append(row)
    baseline_core_sets = build_baseline_core_sets(repo_rows, queries)
    analysis_rows: list[dict[str, Any]] = []

    for summary in run_summaries:
        baseline = run_index[(summary["query"], "native-best-match")]
        comparison = comparison_index[(summary["query"], summary["scenario"])]
        metadata = parse_scenario_metadata(summary["scenario"])
        duration_ms = to_float(summary["duration_ms"])
        baseline_duration_ms = to_float(baseline["duration_ms"])
        added_vs_native_ms = duration_ms - baseline_duration_ms
        scenario_repos = sorted(
            repo_index[(summary["query"], summary["scenario"])],
            key=lambda item: item["rank_position"],
        )
        scenario_repo_names = {row["full_name"] for row in scenario_repos}
        surface_counts = summary["matched_surface_counts"]
        if not isinstance(surface_counts, dict):
            surface_counts = {}
        analysis_rows.append(
            {
                **summary,
                **comparison,
                **metadata,
                "baseline_duration_ms": int(baseline_duration_ms),
                "added_vs_native_ms": int(added_vs_native_ms),
                "added_vs_native_pct": round(
                    (added_vs_native_ms / baseline_duration_ms) if baseline_duration_ms else 0.0,
                    4,
                ),
                "novel_per_added_second": round(
                    to_float(comparison["novel_results"]) / max(added_vs_native_ms / 1000.0, 0.001),
                    4,
                )
                if added_vs_native_ms > 0
                else "",
                "jaccard_per_added_second": round(
                    to_float(comparison["top_k_jaccard"]) / max(added_vs_native_ms / 1000.0, 0.001),
                    6,
                )
                if added_vs_native_ms > 0
                else "",
                "core_top3_retained": len(
                    scenario_repo_names & baseline_core_sets[summary["query"]][3]
                ),
                "core_top5_retained": len(
                    scenario_repo_names & baseline_core_sets[summary["query"]][5]
                ),
                "core_top3_rate": round(
                    len(scenario_repo_names & baseline_core_sets[summary["query"]][3]) / 3,
                    4,
                ),
                "core_top5_rate": round(
                    len(scenario_repo_names & baseline_core_sets[summary["query"]][5]) / 5,
                    4,
                ),
                "surface_breadth": len(surface_counts),
                "surface_name_share": surface_share(surface_counts, "name"),
                "surface_description_share": surface_share(surface_counts, "description"),
                "surface_topics_share": surface_share(surface_counts, "topics"),
                "surface_readme_share": surface_share(surface_counts, "readme"),
            }
        )

    for row in analysis_rows:
        row["balanced_general_frontier"] = False

    for query in queries:
        frontier_candidates = [
            row
            for row in analysis_rows
            if row["query"] == query
            and row["group"] == "discover"
            and row["depth"] == "balanced"
            and not row["readme_enabled"]
            and row["slice_profile"] == "default"
        ]
        for candidate in frontier_candidates:
            candidate_duration = to_float(candidate["duration_ms"])
            candidate_jaccard = to_float(candidate["top_k_jaccard"])
            candidate_novel = to_float(candidate["novel_results"])
            dominated = False
            for other in frontier_candidates:
                if other is candidate:
                    continue
                other_duration = to_float(other["duration_ms"])
                other_jaccard = to_float(other["top_k_jaccard"])
                other_novel = to_float(other["novel_results"])
                if (
                    other_duration <= candidate_duration
                    and other_jaccard >= candidate_jaccard
                    and other_novel >= candidate_novel
                    and (
                        other_duration < candidate_duration
                        or other_jaccard > candidate_jaccard
                        or other_novel > candidate_novel
                    )
                ):
                    dominated = True
                    break
            candidate["balanced_general_frontier"] = not dominated

    return analysis_rows

def build_paired_effects(
    *,
    run_summaries: list[dict[str, Any]],
    comparisons: list[dict[str, Any]],
) -> list[dict[str, Any]]:
    run_index = {(row["query"], row["scenario"]): row for row in run_summaries}
    comparison_index = {(row["query"], row["scenario"]): row for row in comparisons}
    effect_rows: list[dict[str, Any]] = []

    def append_effect(
        *,
        query: str,
        effect_type: str,
        base_scenario: str,
        compare_scenario: str,
        label: str,
    ) -> None:
        base_run = run_index[(query, base_scenario)]
        compare_run = run_index[(query, compare_scenario)]
        base_comparison = comparison_index[(query, base_scenario)]
        compare_comparison = comparison_index[(query, compare_scenario)]
        base_duration_ms = to_float(base_run["duration_ms"])
        compare_duration_ms = to_float(compare_run["duration_ms"])
        effect_rows.append(
            {
                "query": query,
                "effect_type": effect_type,
                "label": label,
                "base_scenario": base_scenario,
                "compare_scenario": compare_scenario,
                "base_duration_ms": int(base_duration_ms),
                "compare_duration_ms": int(compare_duration_ms),
                "added_ms": int(compare_duration_ms - base_duration_ms),
                "added_pct": round(
                    ((compare_duration_ms - base_duration_ms) / base_duration_ms) if base_duration_ms else 0.0,
                    4,
                ),
                "base_jaccard": base_comparison["top_k_jaccard"],
                "compare_jaccard": compare_comparison["top_k_jaccard"],
                "jaccard_delta": round(
                    to_float(compare_comparison["top_k_jaccard"]) - to_float(base_comparison["top_k_jaccard"]),
                    4,
                ),
                "base_novel_results": base_comparison["novel_results"],
                "compare_novel_results": compare_comparison["novel_results"],
                "novel_results_delta": int(to_float(compare_comparison["novel_results"]) - to_float(base_comparison["novel_results"])),
                "base_median_stars": base_run["median_stars"],
                "compare_median_stars": compare_run["median_stars"],
                "median_stars_delta": round(
                    to_float(compare_run["median_stars"]) - to_float(base_run["median_stars"]),
                    3,
                ),
            }
        )

    for query in DEFAULT_QUERIES:
        for depth in ("quick", "balanced", "deep"):
            append_effect(
                query=query,
                effect_type="depth-over-native",
                base_scenario="native-best-match",
                compare_scenario=f"discover-{depth}-native",
                label=f"{depth}-native-over-baseline",
            )
        for base_scenario, compare_scenario, label in README_BASE_PAIRS:
            append_effect(
                query=query,
                effect_type="readme-tax",
                base_scenario=base_scenario,
                compare_scenario=compare_scenario,
                label=label,
            )
        for base_scenario, compare_scenario, label in WEIGHT_BASE_PAIRS:
            append_effect(
                query=query,
                effect_type="weighting-shift",
                base_scenario=base_scenario,
                compare_scenario=compare_scenario,
                label=label,
            )
        for base_scenario, compare_scenario, label in SLICE_BASE_PAIRS:
            append_effect(
                query=query,
                effect_type="slice-tax",
                base_scenario=base_scenario,
                compare_scenario=compare_scenario,
                label=label,
            )

    return effect_rows


def render_report(
    *,
    run_summaries: list[dict[str, Any]],
    repo_rows: list[dict[str, Any]],
    comparisons: list[dict[str, Any]],
    scenario_analysis: list[dict[str, Any]],
    paired_effects: list[dict[str, Any]],
    queries: list[str],
    missing_artifacts: list[str],
    scenario_count: int,
    limit: int,
) -> str:
    lines = [
        "# Gitquarry Benchmark Study",
        "",
        f"- Generated: {now_iso()}",
        f"- Queries: {', '.join(queries)}",
        f"- Scenarios per query: {scenario_count}",
        f"- Result limit per run: {limit}",
        "",
        "## Method",
        "",
        "- Native baseline runs compare `best-match`, `stars`, and `updated` sorting.",
        "- Discover runs sweep `quick`, `balanced`, and `deep` depths across `native`, `query`, `activity`, `quality`, and `blended` ranks.",
        "- Focused slices add README enrichment, weighted blended variants, Rust filtering, and recency filtering.",
        "- All discover runs use `--explain` so score components and matched surfaces can be analyzed.",
        "",
    ]

    if missing_artifacts:
        lines.extend(
            [
                "## Exclusions",
                "",
                "The following query/scenario pairs were excluded from the final report because the live run timed out or hit the GitHub rate limit before producing artifacts:",
                "",
            ]
        )
        for item in missing_artifacts:
            lines.append(f"- `{item}`")
        lines.append("")

    for query in queries:
        query_runs = [row for row in run_summaries if row["query"] == query]
        query_results = [row for row in repo_rows if row["query"] == query]
        query_comparisons = [row for row in comparisons if row["query"] == query]
        query_analysis = [row for row in scenario_analysis if row["query"] == query]
        analysis_index = {row["scenario"]: row for row in query_analysis}
        effect_index = {
            (row["effect_type"], row["label"]): row
            for row in paired_effects
            if row["query"] == query
        }
        lines.extend([f"## Query: `{query}`", ""])

        fastest = sorted(query_runs, key=lambda row: row["duration_ms"])[:5]
        lines.append("### Fastest Scenarios")
        lines.append("")
        for row in fastest:
            lines.append(
                f"- `{row['scenario']}`: {row['duration_ms']} ms, median stars {row['median_stars']}, language diversity {row['language_diversity']}"
            )
        lines.append("")

        lines.append("### Added Latency vs `native-best-match`")
        lines.append("")
        for depth in ("quick", "balanced", "deep"):
            effect = effect_index[("depth-over-native", f"{depth}-native-over-baseline")]
            lines.append(
                f"- `{effect['compare_scenario']}` adds {format_duration_ms(to_float(effect['added_ms']))} over native best-match"
            )
        lines.append("")

        highest_churn = sorted(
            query_comparisons,
            key=lambda row: (
                row["novel_results"],
                row["top_k_jaccard"] * -1,
            ),
            reverse=True,
        )[:5]
        lines.append("### Highest Churn vs `native-best-match`")
        lines.append("")
        for row in highest_churn:
            lines.append(
                f"- `{row['scenario']}`: overlap {row['top_k_overlap']}/{limit}, jaccard {row['top_k_jaccard']}, novel {row['novel_results']}, avg rank shift {row['avg_abs_rank_shift_common']}"
            )
        lines.append("")

        readme_runs = [
            row for row in query_runs if "readme" in row["scenario"] or row["readme_coverage"] > 0
        ]
        if readme_runs:
            lines.append("### README-Enriched Scenarios")
            lines.append("")
            for row in sorted(readme_runs, key=lambda item: item["scenario"]):
                lines.append(
                    f"- `{row['scenario']}`: readme coverage {row['readme_coverage']:.2%}, median stars {row['median_stars']}, duration {row['duration_ms']} ms"
                )
            lines.append("")

        lines.append("### README Tax")
        lines.append("")
        for _, _, label in README_BASE_PAIRS:
            effect = effect_index[("readme-tax", label)]
            lines.append(
                f"- `{effect['compare_scenario']}` adds {format_duration_ms(to_float(effect['added_ms']))} over `{effect['base_scenario']}` with Jaccard delta {effect['jaccard_delta']:+.4f}"
            )
        lines.append("")

        frontier_rows = [
            row
            for row in query_analysis
            if row.get("balanced_general_frontier")
        ]
        lines.append("### Balanced Frontier")
        lines.append("")
        for row in sorted(
            frontier_rows,
            key=lambda item: (
                to_float(item["duration_ms"]),
                -to_float(item["top_k_jaccard"]),
                -to_float(item["novel_results"]),
            ),
        ):
            lines.append(
                f"- `{row['scenario']}`: {format_duration_ms(to_float(row['duration_ms']))}, Jaccard {to_float(row['top_k_jaccard']):.4f}, novel {int(to_float(row['novel_results']))}, retains {int(to_float(row['core_top5_retained']))}/5 of the native top five"
            )
        lines.append("")

        balanced_focus = [
            analysis_index[name]
            for name in (
                "discover-balanced-query",
                "discover-balanced-activity",
                "discover-balanced-quality",
                "discover-balanced-blended",
                "discover-balanced-blended-quality-heavy",
            )
            if name in analysis_index
        ]
        lines.append("### Balanced Tradeoff Snapshot")
        lines.append("")
        for row in balanced_focus:
            lines.append(
                f"- `{row['scenario']}`: {format_duration_ms(to_float(row['duration_ms']))}, Jaccard {to_float(row['top_k_jaccard']):.4f}, novel {int(to_float(row['novel_results']))}, median stars {to_float(row['median_stars']):.1f}"
            )
        lines.append("")

        lines.append("### Core Retention Snapshot")
        lines.append("")
        for row in balanced_focus:
            lines.append(
                f"- `{row['scenario']}`: retains {int(to_float(row['core_top3_retained']))}/3 of the native top three and {int(to_float(row['core_top5_retained']))}/5 of the native top five"
            )
        lines.append("")

        lines.append("### Surface Attribution Snapshot")
        lines.append("")
        for row in balanced_focus:
            lines.append(
                f"- `{row['scenario']}`: name {to_float(row['surface_name_share']) * 100:.0f}%, description {to_float(row['surface_description_share']) * 100:.0f}%, topics {to_float(row['surface_topics_share']) * 100:.0f}%, README {to_float(row['surface_readme_share']) * 100:.0f}%"
            )
        lines.append("")

        best_preserver = max(
            balanced_focus,
            key=lambda row: (to_float(row["top_k_jaccard"]), -to_float(row["duration_ms"])),
        )
        best_explorer = max(
            balanced_focus,
            key=lambda row: (to_float(row["novel_results"]), -to_float(row["top_k_jaccard"])),
        )
        lines.append("### Recommendation Snapshot")
        lines.append("")
        lines.append(
            f"- Preserve the native baseline: `{best_preserver['scenario']}` with Jaccard {to_float(best_preserver['top_k_jaccard']):.4f} at {format_duration_ms(to_float(best_preserver['duration_ms']))}"
        )
        lines.append(
            f"- Maximize novel results within balanced discover: `{best_explorer['scenario']}` with {int(to_float(best_explorer['novel_results']))} novel results at {format_duration_ms(to_float(best_explorer['duration_ms']))}"
        )
        lines.append("")

        top_repo_counts: dict[str, int] = {}
        for row in query_results:
            top_repo_counts[row["full_name"]] = top_repo_counts.get(row["full_name"], 0) + 1
        most_common = sorted(top_repo_counts.items(), key=lambda item: (-item[1], item[0]))[:10]
        lines.append("### Most Persistent Repositories")
        lines.append("")
        for repo_name, count in most_common:
            lines.append(f"- `{repo_name}` appeared in {count} scenarios")
        lines.append("")

    return "\n".join(lines).rstrip() + "\n"


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Run a reproducible gitquarry benchmark study for two live queries."
    )
    parser.add_argument(
        "--output-dir",
        default=str(DEFAULT_OUTPUT_DIR),
        help="Directory for raw outputs, CSVs, and the markdown report",
    )
    parser.add_argument(
        "--limit",
        type=int,
        default=DEFAULT_LIMIT,
        help="Result limit to request from each search run",
    )
    parser.add_argument(
        "--sleep-seconds",
        type=float,
        default=0.75,
        help="Delay between live API runs to avoid bursty search traffic",
    )
    parser.add_argument(
        "--timeout-seconds",
        type=float,
        default=300.0,
        help="Per-scenario command timeout in seconds",
    )
    parser.add_argument(
        "--binary",
        default=str(ROOT / "target" / "debug" / "gitquarry"),
        help="Path to the gitquarry binary to execute",
    )
    parser.add_argument(
        "--queries",
        nargs="*",
        default=DEFAULT_QUERIES,
        help="Queries to run; defaults to the built-in benchmark pair",
    )
    parser.add_argument(
        "--scenarios",
        nargs="*",
        default=None,
        help="Optional scenario name filter for partial or resumed runs",
    )
    parser.add_argument(
        "--resume",
        action="store_true",
        help="Skip scenarios that already have both raw JSON and metadata files",
    )
    parser.add_argument(
        "--analyze-only",
        action="store_true",
        help="Skip live runs and rebuild the report from existing raw outputs and metadata",
    )
    parser.add_argument(
        "--allow-missing",
        action="store_true",
        help="Skip missing query/scenario artifacts during analysis and note them in the report",
    )
    args = parser.parse_args()

    output_dir = Path(args.output_dir)
    raw_dir = output_dir / "raw"
    output_dir.mkdir(parents=True, exist_ok=True)
    raw_dir.mkdir(parents=True, exist_ok=True)

    binary = Path(args.binary)
    scenario_map = {scenario.name: scenario for scenario in build_scenarios()}
    selected_scenarios = list(scenario_map.values())
    if args.scenarios:
        missing = [name for name in args.scenarios if name not in scenario_map]
        if missing:
            raise SystemExit(f"Unknown scenarios requested: {', '.join(missing)}")
        selected_scenarios = [scenario_map[name] for name in args.scenarios]

    queries = args.queries or DEFAULT_QUERIES

    if not args.analyze_only:
        ensure_binary(binary)
        token = obtain_token()

        for query in queries:
            query_slug = slugify_query(query)
            query_dir = raw_dir / query_slug
            query_dir.mkdir(parents=True, exist_ok=True)

            for scenario in selected_scenarios:
                json_path = query_dir / f"{scenario.name}.json"
                stderr_path = query_dir / f"{scenario.name}.stderr.txt"
                meta_path = query_dir / f"{scenario.name}.meta.json"
                if args.resume and json_path.exists() and meta_path.exists():
                    continue

                started_at = now_iso()
                command = [
                    str(binary),
                    "search",
                    query,
                    "--format",
                    "json",
                    "--limit",
                    str(args.limit),
                    "--progress",
                    "off",
                    *scenario.args,
                ]
                env = os.environ.copy()
                env["GITQUARRY_TOKEN"] = token

                start = time.perf_counter()
                try:
                    result = subprocess.run(
                        command,
                        cwd=ROOT,
                        capture_output=True,
                        text=True,
                        check=False,
                        env=env,
                        timeout=args.timeout_seconds,
                    )
                except subprocess.TimeoutExpired as err:
                    raise SystemExit(
                        f"Scenario {scenario.name!r} timed out for query {query!r} after {args.timeout_seconds} seconds"
                    ) from err

                duration_ms = int((time.perf_counter() - start) * 1000)
                if result.returncode != 0:
                    raise SystemExit(
                        f"Scenario {scenario.name!r} failed for query {query!r}: {result.stderr.strip()}"
                    )

                payload = json.loads(result.stdout)
                json_path.write_text(
                    json.dumps(payload, indent=2) + "\n",
                    encoding="utf-8",
                )
                stderr_path.write_text(result.stderr, encoding="utf-8")
                meta_path.write_text(
                    json.dumps(
                        {
                            "query": query,
                            "scenario": scenario.name,
                            "scenario_label": scenario.label,
                            "group": scenario.group,
                            "started_at": started_at,
                            "duration_ms": duration_ms,
                            "command": command,
                            "limit": args.limit,
                        },
                        indent=2,
                    )
                    + "\n",
                    encoding="utf-8",
                )

                time.sleep(args.sleep_seconds)

    run_summaries: list[dict[str, Any]] = []
    repo_rows: list[dict[str, Any]] = []
    missing_artifacts: list[str] = []

    for query in queries:
        query_dir = raw_dir / slugify_query(query)
        for scenario in selected_scenarios:
            json_path = query_dir / f"{scenario.name}.json"
            meta_path = query_dir / f"{scenario.name}.meta.json"
            if not json_path.exists() or not meta_path.exists():
                if args.allow_missing:
                    missing_artifacts.append(f"{query} :: {scenario.name}")
                    continue
                raise SystemExit(
                    f"Missing study artifacts for query {query!r}, scenario {scenario.name!r}"
                )

            payload = json.loads(json_path.read_text(encoding="utf-8"))
            meta = json.loads(meta_path.read_text(encoding="utf-8"))
            summary, rows = summarize_run(
                query=query,
                scenario=scenario,
                duration_ms=meta["duration_ms"],
                payload=payload,
                started_at=meta["started_at"],
            )
            run_summaries.append(summary)
            repo_rows.extend(rows)

    baseline_by_query = {
        query: [
            row["full_name"]
            for row in sorted(
                [
                    item
                    for item in repo_rows
                    if item["query"] == query and item["scenario"] == "native-best-match"
                ],
                key=lambda item: item["rank_position"],
            )
        ]
        for query in queries
    }

    comparisons: list[dict[str, Any]] = []
    for query in queries:
        baseline = baseline_by_query[query]
        for scenario in selected_scenarios:
            scenario_rows = sorted(
                [
                    row
                    for row in repo_rows
                    if row["query"] == query and row["scenario"] == scenario.name
                ],
                key=lambda item: item["rank_position"],
            )
            ranked = [row["full_name"] for row in scenario_rows]
            metrics = compare_lists(baseline, ranked)
            comparisons.append(
                {
                    "query": query,
                    "scenario": scenario.name,
                    "scenario_label": scenario.label,
                    **metrics,
                }
            )

    scenario_analysis = build_scenario_analysis(
        run_summaries=run_summaries,
        comparisons=comparisons,
        repo_rows=repo_rows,
        queries=queries,
    )
    paired_effects = build_paired_effects(
        run_summaries=run_summaries,
        comparisons=comparisons,
    )
    balanced_frontier = [
        row for row in scenario_analysis if row.get("balanced_general_frontier")
    ]

    write_csv(output_dir / "run-summaries.csv", run_summaries)
    write_csv(output_dir / "repo-rows.csv", repo_rows)
    write_csv(output_dir / "comparisons.csv", comparisons)
    write_csv(output_dir / "scenario-analysis.csv", scenario_analysis)
    write_csv(output_dir / "paired-effects.csv", paired_effects)
    write_csv(output_dir / "balanced-frontier.csv", balanced_frontier)
    (output_dir / "run-summaries.json").write_text(
        json.dumps(run_summaries, indent=2) + "\n",
        encoding="utf-8",
    )
    (output_dir / "comparisons.json").write_text(
        json.dumps(comparisons, indent=2) + "\n",
        encoding="utf-8",
    )
    (output_dir / "scenario-analysis.json").write_text(
        json.dumps(scenario_analysis, indent=2) + "\n",
        encoding="utf-8",
    )
    (output_dir / "paired-effects.json").write_text(
        json.dumps(paired_effects, indent=2) + "\n",
        encoding="utf-8",
    )
    (output_dir / "balanced-frontier.json").write_text(
        json.dumps(balanced_frontier, indent=2) + "\n",
        encoding="utf-8",
    )
    (output_dir / "report.md").write_text(
        render_report(
            run_summaries=run_summaries,
            repo_rows=repo_rows,
            comparisons=comparisons,
            scenario_analysis=scenario_analysis,
            paired_effects=paired_effects,
            queries=queries,
            missing_artifacts=missing_artifacts,
            scenario_count=len(selected_scenarios),
            limit=args.limit,
        ),
        encoding="utf-8",
    )

    print(f"Wrote study outputs to {output_dir}")


if __name__ == "__main__":
    main()
