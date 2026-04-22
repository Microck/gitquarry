# gitquarry v1 spec

## product summary

`gitquarry` is a Rust terminal CLI for public GitHub repository search.

The core contract is:

- Plain `search "query"` is native GitHub-style repository search.
- Custom filtering and better discovery exist behind explicit flags.
- Nothing heavier happens silently.

## command surface

### root

- `gitquarry search [OPTIONS] [QUERY]`
- `gitquarry inspect [OPTIONS] <OWNER/REPO>`
- `gitquarry auth login [OPTIONS]`
- `gitquarry auth status [OPTIONS]`
- `gitquarry auth logout [OPTIONS]`
- `gitquarry config path`
- `gitquarry config show`
- `gitquarry version`
- `gitquarry --version`
- `gitquarry --generate-completion <bash|zsh|fish|powershell>`

### global options

- `--host <HOST_OR_URL>`
  - Accept bare hostnames or full URLs.
  - Normalize internally.
  - Default host is `github.com` unless overridden by config.

## auth

### scope

- Auth is always required.
- v1 is PAT-only.
- Device flow is out of scope for v1.
- v1 is read-only and public-repos-only.

### commands

- `auth login`
  - Interactive guided PAT setup.
  - Teaches the user how to create a PAT.
  - Recommends fine-grained PATs when possible.
  - Validates the PAT immediately.
  - Stores credentials per host.
- `auth status`
  - Shows whether the current effective host has a saved PAT.
- `auth logout`
  - Deletes the PAT for the current effective host.

### storage

- Secrets are stored in the OS credential store by default.
- Env vars override saved credentials.
- If secure storage is unavailable, insecure file fallback is allowed only with explicit opt-in and warning.
- PATs are stored per host, not globally.

## config

### location

- Store config in a standard per-user config path.
- Do not store config in the repo or current working directory.

### allowed saved defaults

- `host`
- `format`
- `limit`
- `progress`
- color preference

### disallowed saved defaults

These must remain explicit per command:

- `mode`
- `rank`
- `depth`
- `readme`
- `explain`
- blended ranking weights
- discovery-only engine knobs

## search

## synopsis

```bash
gitquarry search [OPTIONS] [QUERY]
```

### base behavior

- If the user passes no enhancement flags, search behaves as close to native GitHub repository search as possible.
- Default output format is `pretty`.
- Default result limit is `10`.
- Empty query is invalid in plain native search.

### output formats

- `pretty` - default
- `json`
- `compact`
- `csv`

### pretty output visual default

- The default accent color for human-facing `pretty` output is electric blue.
- The exact electric-blue shade does not need to be fixed in v1 as long as it stays in that visual family.
- ANSI color choices such as `21` or `27` are acceptable defaults.
- Users may override color preference through config or terminal settings where supported.

### retrieval modes

- `--mode native`
  - One native GitHub repository search only.
- `--mode discover`
  - Explicit enhanced retrieval.
  - Uses staged candidate collection and local reranking.

Default:

- no `--mode` flag -> native behavior

### rank modes

- `--rank native`
- `--rank query`
- `--rank activity`
- `--rank quality`
- `--rank blended`

Rules:

- No enhancement flags -> effective rank is native.
- `--mode discover` with no `--rank` -> default to `blended`.
- `--rank` never changes retrieval mode.
- `--sort` never changes rank mode.

### native ordering

`--sort` is reserved for native GitHub-like retrieval ordering.

Supported values:

- `best-match`
- `stars`
- `updated`

Rules:

- In `--rank native`, final output order follows native ordering and `--sort`.
- In non-native rank modes, `--sort` influences candidate retrieval only. Final displayed order comes from the selected rank mode.

### structured filter flags

#### owner / visibility

- `--user <USER>`
- `--org <ORG>`
- `--archived <true|false>`
- `--template <true|false>`
- `--fork <false|true|only>`

#### metadata / classification

- `--language <LANG>` repeatable
- `--topic <TOPIC>` repeatable
- `--license <LICENSE>` repeatable

#### numeric ranges

- `--min-stars <N>`
- `--max-stars <N>`
- `--min-forks <N>`
- `--max-forks <N>`
- `--min-size <N>`
- `--max-size <N>`

#### absolute date filters

- `--created-after <YYYY-MM-DD>`
- `--created-before <YYYY-MM-DD>`
- `--updated-after <YYYY-MM-DD>`
- `--updated-before <YYYY-MM-DD>`
- `--pushed-after <YYYY-MM-DD>`
- `--pushed-before <YYYY-MM-DD>`

#### relative date filters

- `--created-within <DURATION>`
- `--updated-within <DURATION>`
- `--pushed-within <DURATION>`

Relative duration syntax:

- `12h`
- `30d`
- `8w`
- `6m`
- `1y`

### filter semantics

- Repeated structured flags use AND semantics.
- OR semantics remain available only through raw GitHub query syntax in the query string.
- `--topic` is GitHub-native topics only.
- Custom label/category matching is not exposed as a first-class flag.
- Custom label/category interpretation is query-driven and only available in enhanced/discovery mode.

### native vs post-filter behavior

Compile directly into GitHub-native search when possible:

- user/org
- archived
- template
- fork
- language
- topic
- license
- stars
- forks
- size
- created
- pushed
- sort

Apply post-retrieval using repo metadata:

- `updated-*`

### empty query rules

- Plain native search requires a query.
- Empty query is allowed only in explicit discovery-oriented usage.

Examples:

- valid: `gitquarry search "rust sdk"`
- valid: `gitquarry search --mode discover --topic cli --sort updated`
- invalid: `gitquarry search`

### raw query plus structured flags

The free-text query is the base query.
Structured flags append constraints.

If the user manually writes a raw GitHub qualifier in the query and also passes an overlapping structured flag, fail clearly instead of guessing.

Examples of conflicts:

- `language:rust` plus `--language go`
- `topic:cli` plus `--topic sdk`
- `stars:>100` plus `--min-stars 200`

### discovery depth

Discovery depth is discover-only.

- `--depth quick`
- `--depth balanced`
- `--depth deep`

Do not allow `--depth` outside discover mode.

Default in discover mode:

- `balanced`

### candidate pool targets

Discovery uses a deduped candidate pool before final reranking.

Defaults:

- `quick`: target `max(25, limit * 3)`, cap `100`
- `balanced`: target `max(50, limit * 5)`, cap `200`
- `deep`: target `max(100, limit * 8)`, cap `400`

### discovery retrieval behavior

Discovery retrieval is staged and bounded.

#### quick

- Seed with one native GitHub repository search only.
- No extra fan-out beyond the initial retrieval.
- Rerank only the returned candidate set.

#### balanced

- Seed with one native GitHub repository search.
- If the deduped pool is below target:
  - run the same search with `sort=updated`
  - run one recency-biased shard using a recent `pushed` bucket
- stop as soon as the target pool is reached

#### deep

- Includes balanced behavior.
- If still below target:
  - add older `pushed` bucket searches
  - add star-bucket shard searches if the user did not already constrain stars
- stop as soon as the target pool is reached

### README enrichment

- `--readme` is explicit.
- No README fetching occurs by default.
- `--readme` never changes retrieval mode by itself.
- README is enrichment-only, not candidate retrieval.

README window:

- metadata ranking runs first
- then README enrichment inspects only the top candidate window
- default window = `min(20, max(limit * 2, 10))`

### explainability

- `--explain` is explicit.
- Without `--explain`, no ranking reasons are shown.
- In native mode, `--explain` is minimal or unsupported.
- In enhanced modes, `--explain` can show:
  - matched surfaces
  - query score
  - activity score
  - quality score
  - blended score
  - effective weights
  - key contributing signals

### blended ranking weights

Weight flags:

- `--weight-query <0.0..3.0>`
- `--weight-activity <0.0..3.0>`
- `--weight-quality <0.0..3.0>`

Rules:

- weight flags are valid only with `--rank blended`
- each value must be in the inclusive range `0.0` to `3.0`
- default weight for each component is `1.0`
- all-zero weight sets are invalid

## scoring

### query score

Literal-first only.
No automatic synonym expansion in v1.

Use weighted literal matching across:

- repository name
- description
- topics
- README only if `--readme` is enabled

### activity score

Use only:

- `pushed_at`
- `updated_at`
- release recency
- archived penalty

### quality score

Use only:

- stars
- forks
- contributor count
- license presence
- README presence
- template status

No issue-based signals in v1.
No momentum/trending signals in v1.

### blended score

For `--rank blended`, compute a normalized weighted average:

`(wq * query + wa * activity + wql * quality) / (wq + wa + wql)`

Only include components whose weight is greater than zero.

## inspect

## synopsis

```bash
gitquarry inspect [OPTIONS] <OWNER/REPO>
```

### behavior

- Accept only explicit `owner/repo`.
- No cross-command memory.
- Default is metadata-first.
- README is optional and explicit.

### defaults

- output format defaults to `pretty`
- default content includes:
  - full name
  - URL
  - description
  - stars
  - forks
  - language
  - topics
  - license
  - created / updated / pushed
  - archived / template / fork status
  - open issues count
  - latest release summary when cheaply available

### inspect flags

- `--readme`
- `--format pretty|json|compact|csv`

## progress

- progress output goes to `stderr` only
- default progress mode is `auto`
- support `--progress auto|on|off`
- in `auto`, show progress only when `stderr` is a TTY
- no progress is written to stdout
- prefer phase-based progress and counts over fake ETA
- show ETA only for bounded loops where estimation is defensible

## advanced engine flags

- `--concurrency <N>`
  - discover/enrichment only
  - default `1`
  - error outside explicit discovery/enrichment usage

Other engine internals remain internal in v1.

## error handling

- fail on the first clear error
- print plain text to `stderr`
- include a stable symbolic error code
- exit code `1`

Examples:

- `E_FLAG_REQUIRES_MODE: --concurrency requires --mode discover`
- `E_FLAG_CONFLICT: --created-after cannot be combined with --created-within`
- `E_AUTH_REQUIRED: auth required for host github.example.com; run gitquarry auth login --host github.example.com`

Errors are never emitted as JSON in v1.

## version and completion

- support both `version` and root `--version`
- include shell completion generation in v1
- support `bash|zsh|fish|powershell`
