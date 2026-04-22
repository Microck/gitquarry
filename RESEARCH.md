# gitquarry research notes

## goal

This document records the external constraints that informed the v1 spec.

## GitHub search and API constraints

### search result cap

GitHub REST search returns up to `1,000` results per search query.

Why it matters:

- broad discovery cannot rely on one giant query
- explicit discovery mode needs staged fan-out and dedupe

Source:

- https://docs.github.com/en/rest/search/search

### search rate limits

For authenticated users, REST search endpoints are more restrictive than the normal core API limit:

- search endpoints: `30 requests/minute`
- unauthenticated search: `10 requests/minute`

Why it matters:

- discovery mode must stay bounded
- README and metadata enrichment cannot be open-ended
- PAT auth is required for a credible user experience

Source:

- https://docs.github.com/en/rest/search/search

### primary api limits

For authenticated users:

- REST: `5,000 requests/hour`
- GraphQL: `5,000 points/hour`

Why it matters:

- “higher API cost” in this project means user rate budget, latency, and support complexity, not direct GitHub billing

Sources:

- https://docs.github.com/en/rest/using-the-rest-api/rate-limits-for-the-rest-api
- https://docs.github.com/en/graphql/overview/rate-limits-and-query-limits-for-the-graphql-api

### secondary rate limits and concurrency

GitHub documents:

- no more than `100` concurrent requests across REST and GraphQL
- REST endpoint secondary limit around `900` points/minute
- official best-practice guidance says to make requests serially instead of concurrently to avoid secondary rate limits

Why it matters:

- default concurrency should be conservative
- v1 uses `--concurrency` as an advanced knob with default `1`

Sources:

- https://docs.github.com/en/rest/using-the-rest-api/best-practices-for-using-the-rest-api
- https://docs.github.com/en/rest/using-the-rest-api/rate-limits-for-the-rest-api

### query limits

GitHub documents:

- query length limit of `256` characters
- at most five `AND/OR/NOT` operators

Why it matters:

- the CLI must validate or clearly surface failures for overcomplicated raw query input
- structured flags should reduce the need for giant raw qualifier strings

Source:

- https://docs.github.com/en/search-github/getting-started-with-searching-on-github/troubleshooting-search-queries

### repository search surfaces

GitHub repository search can match:

- name
- description
- topics
- README via `in:readme`

Why it matters:

- metadata-only matching is enough for native search
- README is the cheapest next enrichment layer for better scaffold discovery

Source:

- https://docs.github.com/en/search-github/searching-on-github/searching-for-repositories

## rest vs graphql

Decision:

- v1 stays REST-only

Reason:

- simpler request model
- fewer rate-limit dimensions
- enough capability for native search, metadata enrichment, and README fetches

Deferred:

- selective GraphQL enrichment can be reconsidered later if REST turns out too inefficient for per-repo detail collection

## host support

### GitHub.com vs GHES

GitHub.com and GHES differ primarily in:

- base URL shape
- supported API versions
- feature parity by server version

Examples:

- GitHub.com REST base: `https://api.github.com`
- GHES REST base: `https://<host>/api/v3`

Why it matters:

- custom `--host` support is feasible in v1
- but it must be documented as best-effort rather than parity-guaranteed

Sources:

- https://docs.github.com/en/rest
- https://docs.github.com/en/enterprise-server@latest/rest

## auth model

### device flow

GitHub device flow for CLIs is possible, but the tool author must register and maintain the app/client identity.

Why it matters:

- v1 chose PAT-only auth to avoid app-registration overhead
- the CLI still offers a guided auth wizard so setup stays friendly

Sources:

- https://docs.github.com/en/apps/oauth-apps/building-oauth-apps/authorizing-oauth-apps
- https://docs.github.com/en/apps/creating-github-apps/writing-code-for-a-github-app/building-a-cli-with-a-github-app

## output reference inspiration

`kagi-cli` informed these v1 output/documentation choices:

- one main search command
- multiple explicit output formats
- structured output is supported directly
- shell completion generation is worth including

But `gitquarry` intentionally diverges in two ways:

- default `search` output is `pretty`, not JSON
- PATs should be stored securely by default, not only in env/config plaintext

Reference repo:

- https://github.com/Microck/kagi-cli
