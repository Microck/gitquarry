# gitquarry architecture

## system overview

The CLI is split into five subsystems:

- command parsing and validation
- auth and host resolution
- native search execution
- enhanced discovery and reranking
- rendering and error reporting

## request pipeline

### native search

1. Resolve effective host.
2. Resolve host-scoped PAT.
3. Parse the query and structured flags.
4. Validate conflicts against raw query qualifiers.
5. Build one GitHub repository-search REST request.
6. Fetch one result set.
7. Apply post-filters that are not natively expressible, such as `updated-*`.
8. Render in the selected output format.

### discovery search

1. Resolve effective host and PAT.
2. Parse and validate flags.
3. Build the seed search.
4. Collect candidates using the staged fan-out plan for the selected depth.
5. Dedupe on `full_name`.
6. Enrich candidates with repo metadata needed for activity/quality scoring.
7. Optionally enrich the top README window if `--readme` is enabled.
8. Compute selected rank mode.
9. Apply limit.
10. Render output, optionally with `--explain`.

### inspect

1. Resolve effective host and PAT.
2. Fetch repo metadata by explicit `owner/repo`.
3. Optionally fetch README if `--readme` is enabled.
4. Render details.

## storage model

### secrets

- PATs are stored per host in the OS credential store.
- Env vars override stored credentials.
- Insecure fallback storage is opt-in only.

### config

- Store non-secret settings in a per-user config file.
- Config never stores behavioral search defaults that would silently enable enhanced behavior.

### cache / index

- v1 uses per-command ephemeral in-memory state only.
- No persistent repo metadata DB survives across commands.
- No persistent discovery index in default behavior.

## scoring model

### normalization

Each score component is normalized to a stable `0..1` range before ranking.

### component use

- `native` rank: preserve retrieval order
- `query` rank: query score only
- `activity` rank: activity score only
- `quality` rank: quality score only
- `blended` rank: weighted average of query, activity, quality

### explainability

Every enhanced score must be explainable from explicit inputs and repo facts.
There is no learned personalization and no history-based bias.

## progress and concurrency

### progress

- Human progress output uses `stderr`.
- Structured stdout remains clean.
- Progress is phase-based:
  - searching
  - collecting
  - enriching metadata
  - enriching README
  - ranking

### concurrency

- Official GitHub guidance favors serial requests to avoid secondary rate limits.
- Default concurrency is `1`.
- `--concurrency` is advanced and explicit.
- On secondary-rate-limit responses, the client must:
  - respect `retry-after`
  - respect `x-ratelimit-reset`
  - otherwise back off exponentially with jitter

## host handling

### normalization

Accept:

- `github.com`
- `github.example.com`
- `https://github.example.com`
- `https://github.example.com/api/v3`

Normalize into:

- canonical web host
- canonical REST API base URL

Defaults:

- GitHub.com web host -> `github.com`
- GitHub.com REST base -> `https://api.github.com`
- GHES REST base -> `https://<host>/api/v3`

### support promise

- GitHub.com is the primary target.
- Custom hosts are best-effort in v1.
- The CLI should always send a stable API version header where supported.

## validation and conflict policy

### first-error policy

- Validate flags before any network work where possible.
- Return the first clear error only.

### examples of enforced conflicts

- raw qualifier plus overlapping structured flag
- `--created-after` plus `--created-within`
- `--weight-query` without `--rank blended`
- `--concurrency` without explicit discovery/enrichment mode

## output contracts

### success

- `pretty` is human-first and concise
- `json` is stable and structured
- `compact` is minified JSON
- `csv` is flat export-friendly output

### errors

- stderr only
- symbolic error code prefix
- plain text message
- shell exit code `1`
