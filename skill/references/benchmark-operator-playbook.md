# Benchmark Operator Playbook

## Contents

1. Default recommendation
2. Latency ladder
3. Intent-to-command mapping
4. Query-family guidance
5. Option-level guidance
6. Failure modes to avoid

## Default Recommendation

Use this when an operator wants a stronger-than-native result set without a long discussion:

```bash
gitquarry search "<query>" --mode discover --depth balanced --rank quality --explain
```

Why:

- It was the best default non-native choice across the live study.
- It preserved the native core much better than `query`.
- It stayed in the practical latency band of the benchmark rather than jumping to deep-mode cost.

## Latency Ladder

Current benchmark ranges:

| Path | Typical cost from study | What it buys |
| --- | --- | --- |
| Native | `~0.5s` to `~1.1s` | Pure GitHub baseline, lowest latency |
| Discover quick | `~16s` to `~18s` | Cheapest discover proof point, often little change if rank stays conservative |
| Discover balanced | `~27s` to `~30s` | Main tradeoff zone for curation, explain output, and controlled novelty |
| Discover deep | `~53s` to `~60s` | High recall tax, reserve for deliberate heavy investigations |
| README enrichment tax | `+3s` to `+5s` | Stronger match evidence, not better top-10 overlap in this run |

Operator rule:

- Native first for interactive speed.
- Balanced for analysis and decision-grade search.
- Deep only when heavy recall is worth doubling the balanced tax.

## Intent-To-Command Mapping

| Intent | Command | Why |
| --- | --- | --- |
| Fastest baseline | `gitquarry search "<query>"` | Preserves native GitHub behavior and stays sub-second. |
| Safer advanced default | `gitquarry search "<query>" --mode discover --depth balanced --rank quality --explain` | Best general upgrade from native without major drift. |
| Broader semantic exploration | `gitquarry search "<query>" --mode discover --depth balanced --rank query --explain` | Highest novelty in the balanced family. |
| Noisy infra-style query | `gitquarry search "<query>" --mode discover --depth balanced --rank blended --weight-query 0.5 --weight-activity 0.5 --weight-quality 2.0 --explain` | Best upgrade for the `api gateway` benchmark shape. |
| Fresh repositories | `gitquarry search "<query>" --mode discover --depth balanced --rank activity --updated-within 1y --explain` | Freshness is the intent, so churn is acceptable. |
| Language slice | `gitquarry search "<query>" --language Rust` | Cheap first pass. Add discover only if the slice still needs expansion. |
| README evidence pass | `gitquarry search "<query>" --mode discover --depth balanced --rank quality --readme --explain` | Use after a promising result set exists and stronger evidence is needed. |

## Query-Family Guidance

### `api gateway`-style queries

These are noisy, crowded, and semantically broad. Safer curation matters more than raw novelty.

Best benchmarked upgrade:

```bash
gitquarry search "api gateway" --mode discover --depth balanced --rank blended --weight-query 0.5 --weight-activity 0.5 --weight-quality 2.0 --explain
```

Why:

- It kept `8/10` of the native top 10.
- It retained `5/5` of the native top five.
- It improved quality signals without paying deep-mode cost.

Avoid balanced `query` here unless the operator explicitly wants semantic drift.

### `terminal ui`-style queries

These are lexically cleaner. Plain `quality` already performs strongly without needing heavier weighting.

Best benchmarked upgrade:

```bash
gitquarry search "terminal ui" --mode discover --depth balanced --rank quality --explain
```

Why:

- It matched the best non-native Jaccard in the study.
- It retained `4/5` of the native top five.
- It was cheaper than the quality-heavy alternative.

If the user wants wider exploration, move to balanced `query` or query-heavy blended, but call out the lower retention.

## Option-Level Guidance

### Rank

| Rank | Use it when | Avoid it when |
| --- | --- | --- |
| `native` | Baseline preservation or validation is the priority | You expect discover to materially change the result set |
| `quality` | You want a curated upgrade that still resembles native | You need maximum novelty |
| `query` | You want semantic expansion and more new repos | You need strong native-core retention |
| `activity` | Freshness is part of the search intent | You want a stable default search |
| `blended` | You want a compromise mode or weighted tuning | You need the strongest default baseline preservation |

### Depth

| Depth | Use it when | Avoid it when |
| --- | --- | --- |
| `quick` | You only need the cheapest discover smoke test | You expect large quality gains |
| `balanced` | You want the main analysis tier | You need native latency |
| `deep` | You are doing explicit high-recall exploration | Time budget matters |

### Optional flags

| Flag | Good use | What the benchmark says |
| --- | --- | --- |
| `--readme` | Evidence gathering, explanation pass, result inspection | Adds `~3s` to `~5s`; no top-10 gain in this run |
| `--updated-within 1y` | Freshness-only searches | Produces high churn; treat as a different intent |
| `--language <lang>` | Narrowing before expansion | Native language filtering is cheap and should come first |

## Failure Modes To Avoid

- Do not recommend discover mode as if it were near-native. Even quick discover is a major latency jump.
- Do not turn on `--readme` by default. The study does not justify it as a baseline toggle.
- Do not recommend balanced `query` as the safest advanced preset. It buys novelty by sacrificing native-core retention.
- Do not mix recency intent into ordinary discovery guidance. Freshness changes the product you are returning.
- Do not skip stating the cost. The operator should know whether the chosen path is a sub-second baseline or a 30-second analysis run.

## Escalation Pattern

Use this sequence when the user wants a progressive search workflow:

1. Start with `gitquarry search "<query>"`.
2. Upgrade to balanced `quality` if the baseline is too literal or too noisy.
3. Upgrade to balanced `query` only if the user still wants more alternatives.
4. Add `--readme`, recency, or language constraints only when the task explicitly requires them.
