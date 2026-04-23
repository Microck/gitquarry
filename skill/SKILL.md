---
name: gitquarry-search-operator
description: Choose and run effective gitquarry search strategies using benchmark-backed heuristics. Use when deciding between native and discover mode, selecting rank, depth, README, recency, language, or blend-weight flags, or producing operator-ready commands with clear latency versus result-quality tradeoffs.
---

# Gitquarry Search Operator

Use this skill when the task is not just "run `gitquarry search`", but "pick the right search shape" or explain why one shape is better than another.

## Workflow

1. Classify the operator intent before picking flags.
2. Start from native search unless the task explicitly calls for broader discovery, reranking, or explanation-heavy analysis.
3. Use balanced `quality` as the default advanced preset.
4. Switch to balanced `query` only when novelty is the goal and lower baseline retention is acceptable.
5. Treat README enrichment as a second pass, not a default toggle.
6. Treat recency and language constraints as separate intents, not as small tweaks to the default path.

## Default Presets

Use these presets unless the request clearly points elsewhere:

- Fastest baseline: `gitquarry search "<query>"`
- Safest advanced default: `gitquarry search "<query>" --mode discover --depth balanced --rank quality --explain`
- Broader semantic exploration: `gitquarry search "<query>" --mode discover --depth balanced --rank query --explain`
- Noisy infrastructure-style query with safer curation: `gitquarry search "<query>" --mode discover --depth balanced --rank blended --weight-query 0.5 --weight-activity 0.5 --weight-quality 2.0 --explain`
- Freshness-driven slice: `gitquarry search "<query>" --mode discover --depth balanced --rank activity --updated-within 1y --explain`
- Language-first slice: `gitquarry search "<query>" --language Rust`
- README inspection pass: `gitquarry search "<query>" --mode discover --depth balanced --rank quality --readme --explain`

## Selection Rules

- Prefer native when latency matters most or when the user wants the GitHub baseline preserved.
- Prefer balanced `quality` when the user wants better curation without throwing away the core native set.
- Prefer balanced `query` when the user explicitly wants semantic expansion, alternative repos, or a wider exploration set.
- Prefer blended `quality`-heavy for noisy query families similar to `api gateway`.
- Add `--readme` only when match evidence or explain output matters more than latency.
- Start with native language filtering before adding discover inside that slice.
- Use recency flags only when freshness is a hard requirement.

## References

Read these only when needed:

- `references/benchmark-operator-playbook.md` for the benchmark-derived command matrix, latency ladder, query-specific guidance, and anti-patterns.
- `../docs/project/benchmark-study.mdx` when you need the full published benchmark narrative.
- `../target/benchmark-study/report.md` when you need the raw summary numbers and scenario names.

When producing recommendations, cite the benchmark logic, not vague preference. The useful question is always: what extra latency is being bought, and is that trade worth it for this query intent?
