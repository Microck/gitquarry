---
name: gitquarry-operator
description: Operate gitquarry correctly for GitHub repository search, inspection, comparison, remote tree/code lookup, recipes, authentication, host selection, and script-safe output. Use when running gitquarry commands, choosing between native and discover search, selecting rank, depth, README, explain, plan/probe flags, tree/code path filters, format, progress, host, or config options, troubleshooting auth or flag conflicts, or producing operator-ready gitquarry command lines.
allowed-tools: Bash(gitquarry:*)
---

# Gitquarry Operator

Use this skill to drive `gitquarry` as a tool, not just to pick one benchmarked search preset.

## Start Here

Load the version-matched copy from the installed CLI before choosing commands:

```bash
gitquarry skills get gitquarry
```

Use `gitquarry agent` as a shorthand for the same core guide.

## Workflow

1. Classify the task as `auth`, `search`, `inspect`, `compare`, `tree`, `code`, `recipe`, `mcp`, `config`, or scripting.
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
- Use `gitquarry compare <owner/repo>...` when several known repositories need side-by-side evidence.
- Use `gitquarry tree <owner/repo>` when the task needs repository paths without cloning.
- Use `gitquarry code <owner/repo> <pattern>` when the task needs remote code search without cloning.
- Use `gitquarry recipe run <file>` when a checked-in TOML search recipe should be executed reproducibly.
- Use `gitquarry mcp` only when an MCP client will launch gitquarry as a stdio server.
- Use `gitquarry config path|show` when the task is about effective config state.

## Tree And Code Rules

- Prefer `tree` over `source path` when path inspection is enough and no local checkout is needed.
- Prefer `code` over `source path` for bounded literal or regex code search inside one known repository.
- Add `--path` filters, `--depth`, `--limit`, or `--max-file-bytes` when a broad remote scan could be noisy or API-heavy.
- Use `--reference` when a branch, tag, or commit matters.
- Use `--mode regex` only when regex semantics are required; literal search is the default.

## Search Rules

- Start with `gitquarry search "<query>"` unless the user explicitly needs enhanced discovery behavior.
- Add structured filters such as `--language`, `--topic`, `--org`, `--user`, star ranges, or date windows before escalating to discover mode.
- Use `--mode discover` only for broader candidate collection, local reranking, README-aware reranking, or explain-driven ranking analysis.
- If discover mode is used and the task does not specify a rank, remember that gitquarry itself defaults to `blended`.
- If you are recommending a safer advanced preset to a human, prefer `--mode discover --depth balanced --rank quality --explain`.
- Add `--readme` only as an explicit second pass when evidence matters more than latency.
- Use `--plan` before network execution when the task is to debug compiled query, effective mode, rank, sort, limit, or estimated request count.
- Use `--probe-path` and `--probe-code` when search results need explicit no-clone evidence without changing result order.
- Keep probes bounded with `--probe-limit`, `--probe-match-limit`, path filters, and `--probe-max-file-bytes` when scanning broad repositories.

## Compare And Recipe Rules

- Use `compare` only for explicit known repositories. It does not discover candidates or compute a trust score.
- Add `--tree-summary` only when file-layout counts matter.
- Add `--readme` only when README text is needed in the comparison output.
- Prefer `recipe run` for shared, reviewed workflows in repos, docs, CI, or incident runbooks.
- Do not put credentials in recipes. Use normal host-scoped auth or token environment variables.
- Treat CLI `--host` as higher precedence than recipe `host`.

## MCP Rules

- `gitquarry mcp` is a stdio JSON-RPC server for MCP clients; do not run it as an interactive terminal command unless you are testing JSON-RPC manually.
- Register it with clients as `gitquarry mcp`, for example `codex mcp add gitquarry -- "$(command -v gitquarry)" mcp`.
- Use `--host` before `mcp` for GitHub Enterprise, for example `gitquarry --host https://ghe.example.com mcp`.
- MCP tools return JSON command output as text content. Parse that text as JSON when structured fields matter.
- Available tools are `gitquarry_search`, `gitquarry_inspect`, `gitquarry_tree`, `gitquarry_code`, `gitquarry_compare`, and `gitquarry_skill`.
- Credentials come from the same environment, config, and host-scoped auth paths as normal CLI commands.

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
- If `compare` input is not `owner/repo`, correct the repository shape before auth debugging.
- If `recipe run` fails with `E_RECIPE_INVALID`, fix unknown keys, malformed TOML, or invalid mapped search flags.
- If `tree` or `code` is too broad, narrow with `--path`, `--depth`, or `--limit` before escalating to `source path`.
- If auth fails, verify token, host, and resolution order before changing search flags.

## References

Read these only when needed:

- `references/benchmark-operator-playbook.md` for the full operator playbook, command patterns, host/auth/scripting rules, and benchmark-backed discover heuristics.
- `../docs/commands/search.mdx` when exact search flag behavior matters.
- `../docs/commands/inspect.mdx` when the task is repository inspection rather than search.
- `../docs/commands/tree.mdx` when the task is remote repository tree inspection.
- `../docs/commands/code.mdx` when the task is no-clone code search.
- `../docs/commands/compare.mdx` when the task is side-by-side explicit repository comparison.
- `../docs/commands/recipe.mdx` when the task is checked-in search recipe execution.
- `../docs/commands/mcp.mdx` when the task is MCP server setup.
- `../docs/guides/output-and-scripting.mdx` when the task is CI, pipeline, or agent-safe usage.
- `../docs/guides/github-enterprise-hosts.mdx` when the task involves non-default hosts.

When recommending commands, explain whether the operator is buying native fidelity, broader coverage, or stronger evidence, and what latency or complexity that choice adds.
