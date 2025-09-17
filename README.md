# Provenance — Artifact‑First Testing Mirror

Secure, deterministic publishing of CI artifacts backed by cryptographic provenance. This project defines a minimal but strong pattern to render testing evidence as a website while verifying exactly what CI produced.

Status: Spec v1. Implementation scaffolding and reference Worker are forthcoming.

> IMPORTANT: External Submodule Ownership — `crates/proofdown_parser`
>
> The Proofdown parser lives in a separate git submodule and is maintained by external consultants.
> Do NOT modify parser code in this repository. Propose changes in the submodule repository and
> update the submodule pointer here. In this workspace, only integration wiring/feature gating
> should be changed.
>
> Path: `crates/proofdown_parser/` (nested workspace)
> Feature gating in SSG: `external_pml`

---

## Why this exists

AI‑generated code is now pervasive. Ethos must lead: documentation and testing take precedence over the code itself. Provenance operationalizes that ethos by flipping the model:

- A single, signed Index (the contract) enumerates exactly which artifacts exist for a commit.
- Each artifact is addressed by SHA‑256 and rendered with a constrained set of safe viewers.
- A Cloudflare Worker verifies the Index signature and each artifact’s digest before anything is shown.
- A purpose‑built markup (“Proofdown”, working name) presents both tests and artifacts for human review, optimized for AI to author reliably.

The result is a tamper‑evident, cacheable, and linkable corpus of evidence that centers specs, contracts, tests, and documented proof—then code.

---

## Ethos and priority order

SPECS → CONTRACTS → TESTING ARTIFACTS → DOCUMENTED PROOF → CODE

We prioritize verifiable evidence and human‑readable documentation over implementation. AI‑authored code must ship with artifacts and a reviewable proof page, or it does not ship.

---

## Core concepts

- Index: The sole source of truth. Produced by CI, signed with Ed25519, and lists all renderable artifacts plus front‑page markup.
- Artifact: Any file emitted by CI that is referenced by id in the Index, including specs, contracts, test outputs, generated documentation, and AI‑authored proofs.
- Proofdown (working name; formerly “CMD”): a safe, component‑based markup that is less than HTML and more than Markdown, designed for AI to author deterministically and for humans to review. Strictly whitelisted; no raw HTML/JS/CSS.
- Worker: A Cloudflare Worker (V8 isolate) that fetches the Index and signature, verifies them, lazily verifies artifact digests, invokes the Rust parser via WASM to render Proofdown → HTML, and serves routes.

---

## High‑level flow

1) CI builds artifacts for a commit.
2) CI computes SHA‑256 for each artifact and writes the Index.
3) CI signs the Index (Ed25519) and publishes Index + signature + artifacts (e.g., to a `ci-snapshots` branch or static hosting).
4) Worker fetches the Index and signature on request, verifies signature, lazily verifies artifact digests, calls the Proofdown parser (Rust→WASM) to render, and serves HTML.

If any verification fails, the Worker refuses to render and returns a clear error state.

---

## Index schema (minimum)

- version: integer (gates breaking changes)
- repo: owner/repo
- commit: full SHA
- workflow_run: id, url, attempt
- front_page.markup: path to a Proofdown file
- front_page.title: string
- artifacts[]: array of objects with:
  - id: unique kebab/slug
  - title: human label
  - path: repo‑relative path to the artifact
  - media_type: MIME type
  - render: renderer hint (e.g., `markdown`, `json`, `table:coverage`, `summary:test`, `image`)
  - sha256: hex digest of the exact bytes

The Index should be JSON or TOML (YAML is allowed if parser weight is acceptable). It may include grouping metadata for navigation.

Example (JSON, abbreviated):

```json
{
  "version": 1,
  "repo": "acme/provenance",
  "commit": "8c6a9f4e...",
  "workflow_run": { "id": 12345, "url": "https://github.com/.../runs/12345", "attempt": 1 },
  "front_page": { "title": "QA Evidence for {{ commit }}", "markup": "ci/front_page.pml" },
  "artifacts": [
    {
      "id": "tests-summary",
      "title": "Test Summary",
      "path": "ci/tests/summary.json",
      "media_type": "application/json",
      "render": "summary:test",
      "sha256": "..."
    },
    {
      "id": "coverage",
      "title": "Coverage",
      "path": "ci/coverage/coverage.json",
      "media_type": "application/json",
      "render": "table:coverage",
      "sha256": "..."
    },
    {
      "id": "failures",
      "title": "Failing Specs",
      "path": "ci/tests/failures.md",
      "media_type": "text/markdown",
      "render": "markdown",
      "sha256": "..."
    }
  ]
}
```

The signature is stored out‑of‑band (e.g., `.provenance/manifest.json.sig`) and verified by the Worker against the exact bytes of the canonicalized Index file.

---

## Contracts

- The Index format is a versioned contract with a JSON Schema/TOML schema and canonicalization rules.
- Renderer `render` hints and `media_type` mappings are contracts; unknown values must be rejected.
- The Proofdown component set and AST are versioned; parsers must be backward‑compatible; breaking changes are gated by `index.version`.
- AI authors adhere to templates and linting to ensure deterministic output; PRs failing contract checks are rejected in CI.

---

## Proofdown (working name)

- Front page must be authored in Proofdown and referenced by `front_page.markup`.
- Recommended file extension: `.pml` (Proof Markup Language).
- Only whitelisted structural components are allowed: e.g., `grid`, `tabs`, `card`, `section`, `gallery`.
- Artifact components reference artifacts by id only: e.g., `artifact.markdown`, `artifact.table`, `artifact.json`, `artifact.image`, `artifact.summary`, `artifact.gauge`, `artifact.viewer kind="llm-proof"`.
- Safe data interpolation from verified Index fields is allowed via placeholders like `{{ commit }}`.
- No raw HTML/JS/CSS; no arbitrary CSS. Styling is limited to typed, validated attributes (e.g., `cols=3`, `gap=16`).
- Deterministic grammar with a stable AST; no side effects.
- Designed for AI authoring: component and attribute names are unambiguous; linting and templates guide generation.

Example Proofdown front page:

```md
# {{ front_page.title }}

<grid cols=3 gap=16>
  <card title="Tests">
    <artifact.summary id="tests-summary" />
  </card>
  <card title="Coverage">
    <artifact.table id="coverage" />
  </card>
  <card title="Failures">
    <artifact.markdown id="failures" />
  </card>
</grid>
```

---

## Worker behavior and routes

- Environment: Cloudflare Workers / V8 isolate (no threads, no filesystem).
- Request pipeline:
  1) Fetch Index and signature
  2) Verify signature (Ed25519)
  3) Verify artifact digests lazily on demand
  4) Render Proofdown → HTML
  5) Serve HTML
- Routes:
  - `/` renders the front page.
  - `/fragment/{artifact_id}` streams a specific artifact view (htmx‑friendly; good for heavy payloads).
  - `/a/{id}` renders a dedicated artifact page (deep link).
- Caching: ETag/If‑None‑Match; Workers Cache with bounded TTL may be used.

## Architecture: Cloudflare Worker + Rust monorepo

- Cloudflare Worker (JS/TS, V8 isolate) is the host. Heavy logic (parsing, canonicalization) runs in Rust compiled to WASM.
- Rust crates (planned):
  - `crates/proofdown_parser`: parser + AST; `no_std`‑friendly; `wasm-bindgen` bindings.
  - `crates/index_contract`: Index schema, canonicalization, Ed25519 verification helpers.
  - `crates/renderers`: pure, deterministic render helpers for known viewers.
  - `crates/wasm_adapter`: glue between Workers and WASM (streams, digest, errors).
- Local dev: Miniflare; Deploy: Wrangler; Secrets: Worker env for public key.
- Data flow: Request → fetch Index+sig → verify (WebCrypto or `index_contract`) → fetch artifact → verify SHA‑256 → parse Proofdown (WASM) → render HTML → stream response.

---

## Security model

- If Index signature verification fails → refuse to render; show clear error.
- If an artifact digest fails → refuse to render that artifact; indicate failure inline.
- The Worker never fetches or renders resources not present in the verified Index.
- All text output is sanitized; client‑supplied scripts never execute.
- Download links point to the exact verified resource (commit‑pinned raw URLs or Worker‑proxied verified streams).

---

## Renderers (minimum set)

- `markdown`: safe Markdown → HTML
- `json`: collapsible JSON tree
- `table:coverage`: coverage JSON → table
- `summary:test`: KPIs (total/passed/failed/duration)
- `image`: responsive image

Specialized viewers (e.g., `viewer:llm-proof`) should be supported when the corresponding artifact is present.

Unknown `render` values fall back to a generic safe viewer or are rejected with a clear message.

---

## CI responsibilities

- Produce all artifacts referenced by the Index.
- Compute SHA‑256 for each artifact; embed in Index.
- Sign the Index with the CI private key (Ed25519). The Worker is configured with the matching public key.
- Fail the pipeline if the Index is malformed, unsigned, or references missing files.
- Optionally publish to a dedicated `ci-snapshots` branch consumed by the Worker.

---

## Configuration & secrets

- Store the verification public key as a Worker secret/env var.
- Optionally configure a static GitHub raw base (owner/repo/branch). Any dynamic routing inputs must be validated to prevent SSRF/path traversal.
- Feature flags may enable experimental viewers.

---

## Badges (GitHub README)

Expose read‑only badge endpoints from the Worker and embed them in your repository README to link into the provenance mirror. Badges are derived only from verified inputs (signed Index and required artifacts).

Examples (replace `<worker-domain>` with your deployment):

```md
[![Provenance](https://<worker-domain>/badge/provenance.svg)](https://<worker-domain>/)
[![Tests](https://<worker-domain>/badge/tests.svg)](https://<worker-domain>/a/tests-summary)
[![Coverage](https://<worker-domain>/badge/coverage.svg)](https://<worker-domain>/a/coverage)
```

Notes
- SVG endpoint: `/badge/<kind>.svg`
- Shields.io JSON endpoint: `/badge/<kind>.json` returns `{ schemaVersion, label, message, color }`
- Supported kinds: `provenance`, `tests`, `coverage`
- Optional params: `style=flat|flat-square`, `label=<safe>`

Specs: see `./.specs/00_provenance.md#15-badges-github-readme-integration` and `./.specs/02_worker.md#31-badge-endpoints`.

---

## Error handling

- Verification failures produce human‑friendly error pages with stable HTTP status (e.g., 502/409).
- Missing artifacts are indicated inline with remediation hints (e.g., “not emitted by CI for commit X”).
- Proofdown parser errors surface with line/column pointers.

---

## Performance & limits

- Stream responses where practical (fragments).
- Enforce per‑artifact size/time limits and degrade gracefully.
- Cache verified bytes (by SHA) to avoid refetching.

---

## Extensibility

- New components/viewers can be added without breaking existing Proofdown (backward‑compatible grammar).
- Index `version` gates incompatible changes; the Worker rejects unknown major versions.
- Additional provenance formats (e.g., SPDX, in‑toto) can be attached and surfaced by dedicated viewers.

---

## Quick start (reference implementation plan)

This repository currently contains the Spec. A reference implementation will follow roughly this plan:

1) Define the Index schema and canonicalization (stable JSON stringification).
2) Add a CI step to generate artifacts under `ci/`, compute SHA‑256, and emit `.provenance/manifest.json` and `.provenance/manifest.json.sig` (Ed25519 via `minisign`, WebCrypto, or Rust signer).
3) Publish `.provenance/manifest.json`, its signature, and artifacts to a static location (e.g., `ci-snapshots` branch or object storage).
4) Scaffold a Rust workspace:
   - `crates/proofdown_parser` (Rust→WASM via `wasm-bindgen`)
   - `crates/index_contract` (schema, canonicalization, Ed25519 verification)
   - `crates/renderers` (deterministic helpers)
   - `crates/wasm_adapter` (Workers bindings)
5) Implement a Cloudflare Worker (TypeScript) that:
   - Fetches `.provenance/manifest.json` and `.provenance/manifest.json.sig`
   - Verifies the signature using WebCrypto (`crypto.subtle`) or calls into `index_contract` WASM
   - Lazily fetches and verifies artifact bytes by SHA‑256 before rendering
   - Uses `proofdown_parser` WASM to render Proofdown → HTML
6) Expose routes: `/`, `/fragment/{artifact_id}`, and `/a/{id}`; wire up ETag and bounded TTL caching.
7) Ship the minimum renderer set and add specialized viewers behind flags.

Example repository layout:

```
.
├── .specs/
│   └── 00_provenance.md                # This spec
├── Cargo.toml                          # Rust workspace
├── crates/
│   ├── proofdown_parser/               # Parser + AST (Rust→WASM)
│   ├── index_contract/                 # Index schema/canonicalization/signature
│   ├── renderers/                      # Deterministic viewer helpers
│   └── wasm_adapter/                   # Workers/WASM glue
├── .provenance/
│   ├── manifest.json                   # Produced by CI (signed)
│   └── manifest.json.sig               # Ed25519 signature of manifest.json
├── ci/
│   ├── front_page.pml                  # Front page Proofdown (PML)
│   ├── tests/summary.json              # Test summary (for summary:test)
│   ├── tests/failures.md               # Failing specs rendered as markdown
│   └── coverage/coverage.json          # Coverage table input
└── worker/                             # Reference Worker (to be added)
```

---

## Compliance checklist (minimum)

- Signed Index verified
- SHA‑256 per artifact verified
- Front page Proofdown rendered with whitelisted components
- Test summary + Coverage + Failures renderers present
- Deep links + Downloads present
- Clear failure states for signature/digest errors

---

## AI authoring workflow expectations

- Every PR (especially AI‑authored) includes updated SPECS/CONTRACTS when behavior changes.
- Tests run in CI, artifacts are emitted, `.provenance/manifest.json` + signature are updated, and a Proofdown front page references artifacts by id.
- CI gates: Index schema validation + canonicalization, Ed25519 signature verification, SHA‑256 digest verification, Proofdown parse/lint, renderer coverage.
- PRs link to the provenance mirror (Worker URL) showing the verified proof page for the commit.

---

## Contributing

This is a spec‑first project. Please propose changes via PRs that update `.specs/00_provenance.md` and, when applicable, include a migration note gated by `version`.

---

## License

TBD.
