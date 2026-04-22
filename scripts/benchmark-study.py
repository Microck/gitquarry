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


def render_report(
    *,
    run_summaries: list[dict[str, Any]],
    repo_rows: list[dict[str, Any]],
    comparisons: list[dict[str, Any]],
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
        lines.extend([f"## Query: `{query}`", ""])

        fastest = sorted(query_runs, key=lambda row: row["duration_ms"])[:5]
        lines.append("### Fastest Scenarios")
        lines.append("")
        for row in fastest:
            lines.append(
                f"- `{row['scenario']}`: {row['duration_ms']} ms, median stars {row['median_stars']}, language diversity {row['language_diversity']}"
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

    write_csv(output_dir / "run-summaries.csv", run_summaries)
    write_csv(output_dir / "repo-rows.csv", repo_rows)
    write_csv(output_dir / "comparisons.csv", comparisons)
    (output_dir / "run-summaries.json").write_text(
        json.dumps(run_summaries, indent=2) + "\n",
        encoding="utf-8",
    )
    (output_dir / "comparisons.json").write_text(
        json.dumps(comparisons, indent=2) + "\n",
        encoding="utf-8",
    )
    (output_dir / "report.md").write_text(
        render_report(
            run_summaries=run_summaries,
            repo_rows=repo_rows,
            comparisons=comparisons,
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
