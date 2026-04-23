---
name: gitquarry-operator
description: Operate gitquarry correctly for GitHub repository search, inspection, authentication, host selection, and script-safe output. Use when running gitquarry commands, choosing between native and discover search, selecting rank, depth, README, explain, format, progress, host, or config options, troubleshooting auth or flag conflicts, or producing operator-ready gitquarry command lines.
---

# Gitquarry Operator

Use this skill to drive `gitquarry` as a tool, not just to pick one benchmarked search preset.

## Workflow

1. Classify the task as `auth`, `search`, `inspect`, `config`, or scripting.
2. Verify the effective host before assuming credentials or config state.
3. Start with the narrowest command that solves the task.
4. Prefer native `search` first. Turn on discover mode only when the request actually needs broader coverage, reranking, README evidence, or explain output.
5. Prefer structured flags over stuffing GitHub qualifiers into the free-text query.
6. Keep enhanced behavior explicit. Do not imply that `discover`, `readme`, or reranking are the default path.
7. Prefer `json` or `compact` plus `--progress off` for scripts, CI, and agent runs.

## Command Selection

- Use `gitquarry auth login|status|logout` for credential work.
- Use `gitquarry search` for repository discovery and ranking.
- Use `gitquarry inspect <owner/repo>` when the target repository is already known.
- Use `gitquarry config path|show` when the task is about effective config state.

## Search Rules

- Start with `gitquarry search "<query>"` unless the user explicitly needs enhanced discovery behavior.
- Add structured filters such as `--language`, `--topic`, `--org`, `--user`, star ranges, or date windows before escalating to discover mode.
- Use `--mode discover` only for broader candidate collection, local reranking, README-aware reranking, or explain-driven ranking analysis.
- If discover mode is used and the task does not specify a rank, remember that gitquarry itself defaults to `blended`.
- If you are recommending a safer advanced preset to a human, prefer `--mode discover --depth balanced --rank quality --explain`.
- Add `--readme` only as an explicit second pass when evidence matters more than latency.

## Output Rules

- Use `pretty` for human terminal reading.
- Use `json` for structured automation.
- Use `compact` for machine pipelines or logs.
- Use `csv` for flat exports.
- Prefer `--progress off` in non-interactive runs.

## Host And Auth Rules

- Treat credentials as host-scoped.
- Use `--host` when the target is GitHub Enterprise or a non-default API host.
- Prefer host-specific env vars such as `GITQUARRY_TOKEN_GITHUB_COM` when scripting against multiple hosts.
- Use `GITQUARRY_CONFIG_DIR` to isolate state in CI, tests, or agent runs.
- Do not assume insecure credential fallback is allowed unless `GITQUARRY_ALLOW_INSECURE_STORAGE=1` is explicitly set.

## Failure Rules

- If a discover-only flag is used without `--mode discover`, fix the command instead of guessing around the error.
- If a raw query qualifier conflicts with a structured flag, remove one side of the conflict.
- If `inspect` input is not `owner/repo`, correct the repository shape before auth debugging.
- If auth fails, verify token, host, and resolution order before changing search flags.

## References

Read these only when needed:

- `references/benchmark-operator-playbook.md` for the full operator playbook, command patterns, host/auth/scripting rules, and benchmark-backed discover heuristics.
- `../docs/commands/search.mdx` when exact search flag behavior matters.
- `../docs/commands/inspect.mdx` when the task is repository inspection rather than search.
- `../docs/guides/output-and-scripting.mdx` when the task is CI, pipeline, or agent-safe usage.
- `../docs/guides/github-enterprise-hosts.mdx` when the task involves non-default hosts.

When recommending commands, explain whether the operator is buying native fidelity, broader coverage, or stronger evidence, and what latency or complexity that choice adds.
