# TODO — Provenance v1 Compliance Backlog

This backlog translates the specs in `.specs/` into a granular, actionable plan to reach v1 compliance and be fully tested. Keep this document updated as tasks are completed.

> WARNING: External Submodule — `crates/proofdown_parser`
>
> The Proofdown parser is maintained by external consultants in a nested workspace (git submodule).
> Do NOT modify parser code in this repository. Propose changes in the submodule repository and
> update the submodule pointer here. In this workspace, only integration wiring/feature gating
> should be changed.
>
> Path: `crates/proofdown_parser/` • Feature flag in SSG: `external_pml`

Specs references for this plan:

- `.specs/00_provenance.md`
- `.specs/10_proofdown.md`
- `.specs/12_canonicalization.md`
- `.specs/14_signing.md`
- `.specs/16_worker.md`
- `.specs/18_workspace.md`
- `.specs/20_testing_and_tiers.md`
- `.specs/22_crate_minimums.md`
- `.specs/40_accessibility.md`
- `.specs/42_human_contract.md`
- `.specs/44_repo_contract.md`
- `.specs/46_llm_developer_contract.md`

## Execution Order (Phased)

1) Repo bootstrap and examples
2) `manifest_contract` (schema + canonicalize + verify)
3) `proofdown_parser` and `renderers`
4) `provenance_ssg` integration, truncation, determinism
5) `badges` crate and SSG badge outputs
6) BDD harness and feature step implementations
7) Worker runtime (Cloudflare) and WASM integration
8) Accessibility checks
9) CI gates and determinism
10) Docs/templates polish

---

## Repository & Examples

- [x] Create missing crates declared in `Cargo.toml`
  - [x] `crates/renderers/`
  - [x] `crates/badges/`
- [ ] Ensure canonical crate layout per `.specs/22_crate_minimums.md`
  - [ ] Add `README.md`, `CHANGELOG.md`, `.provenance/`, `ci/`, `.specs/` to each crate
- [x] Prepare example Ed25519 test keys for local tests (non-secret)
  - [x] Provide public test key in repo for verification
  - [x] Private test key used only by local scripts (exclude from VCS)
- [x] Add script: compute SHA-256 for artifacts and update manifest
  - [x] Input: repo root + manifest path
  - [x] Output: updated `sha256` fields
- [x] Add script/CLI: canonicalize and sign `.provenance/manifest.json` (Ed25519 Base64)
  - [x] Inputs: manifest path, private key
  - [x] Output: `.provenance/manifest.json.sig`
- [x] Update `examples/minimal/` with real SHA-256 values in `.provenance/manifest.json`
- [x] Generate real Base64 signature for `examples/minimal/.provenance/manifest.json.sig`
- [ ] Add a large JSON fixture for truncation tests (e.g., 5–10 MB) under `examples/minimal/ci/tests/`
- [x] Ensure `examples/minimal/ci/tests/failures.md` exists and is non-empty

---

## `manifest_contract` crate

- [x] Expose `Manifest`, `Artifact`, `WorkflowRun`, `FrontPage` types (single source of truth)
- [x] Implement `load_manifest(path)` with clear error context
- [x] JSON Schema validation using `schemas/manifest.schema.json`
  - [x] Validate required fields, formats, allowed `render` values
- [x] `canonicalize(manifest) -> Vec<u8>`
  - [x] Sort keys recursively; UTF-8, `\n`; stable JSON serialization
- [x] `ed25519_verify(canonical_bytes, base64_sig, pubkey_b64_or_hex)`
- [x] Detailed validation errors
  - [x] Unique artifact ids
  - [x] Path normalization (reject `..`, enforce repo-relative)
  - [x] Allowed `render` values per `.specs/00_provenance.md#8-renderers` + schema
- [ ] Unit tests + golden test vectors
  - [x] Canonicalization vectors from `.specs/12_canonicalization.md`
  - [ ] Schema validation success/failure cases

---

## `proofdown_parser` crate

- [x] Create `src/lib.rs` with:
  - [x] AST structs (document/blocks/inlines/components)
  - [ ] Error types with line/column
- [ ] Tokenizer + parser for:
  - [ ] Headings `#..####`, paragraphs, lists, code fences
  - [ ] Inline code
- [ ] Component grammar parsing for:
  - [ ] Structural: `grid`, `section`, `card`, `tabs`, `gallery`
  - [ ] Artifact viewers: `artifact.summary`, `artifact.table`, `artifact.json`, `artifact.markdown`, `artifact.image`, `artifact.viewer`, `artifact.link`
- [ ] Unknown components/attributes as hard errors
- [ ] Link macro `[[...]]`:
  - [ ] `a:`, `repo:`, `src:`, `doc:`, `gh:`, `ci:`, `sym:` and path shorthand semantics
- [ ] Includes with depth limit and cycle detection
- [ ] Enforce attribute bounds & types (see `.specs/10_proofdown.md#7-attributes-amp-types`)
- [ ] Optional: WASM bindings via `wasm-bindgen` for Worker
- [ ] Golden AST tests for examples in `.specs/10_proofdown.md`

---

## `renderers` crate

- [x] Pure, deterministic render helpers (no IO):
  - [x] `markdown` (safe)
  - [x] `json` (pretty, escaped)
  - [x] `table:coverage`
  - [x] `summary:test`
  - [x] `image`
- [ ] Structural layout RSX/HTML (`grid/section/card/tabs/gallery`)
- [ ] HTML sanitization for all text output
- [ ] Accessibility semantics: headings `h1..h4`, `<nav>`, tables with `<thead>`, `<tbody>`, `scope`
- [ ] Golden RSX/HTML tests for example inputs

---

## `badges` crate

- [x] Compute Shields JSON for `provenance`, `tests`, `coverage`
  - [ ] Schema: `schemas/badge.schema.json`
- [x] Error badges for missing/invalid artifacts (`message = error`, `color = red`)
- [x] Minimal SVG generation with thresholds/colors (e.g., coverage thresholds)
- [ ] Schema tests for JSON badges

---

## `provenance_ssg` crate

- [x] Replace local manifest structs/usages with `manifest_contract` types
- [x] Add `--verify-manifest` flag:
  - [x] Schema validation
  - [x] Canonicalize + Ed25519 verify (fail build on mismatch)
- [x] Parse front page `ci/front_page.pml` via `proofdown_parser`
- [x] Render Proofdown AST via `renderers` with verified data context
  - [x] Resolve `<artifact.*>` by `id` only; unknown `id` -> error
- [x] Truncation policy for large artifacts (configurable limits)
  - [x] Show truncation banner + verified `Download` link
- [ ] Deterministic output guarantees
  - [ ] Stable ordering; fixed float formatting
- [x] Write static badges:
  - [x] `site/badge/provenance.json` and `.svg`
  - [x] `site/badge/tests.json` and `.svg`
  - [x] `site/badge/coverage.json` and `.svg`
- [x] Strict viewer mapping for per-artifact pages (reject unknown `render`)
- [ ] Accessibility markup in generated HTML
- [ ] Golden tests (`index.html`, `a/*/index.html`)

---

## BDD Harness & Steps (`features/*.feature`)

- [ ] Add Rust cucumber test runner for repo-level features
- [ ] Steps: manifest schema validation + unique id check
- [ ] Steps: canonicalization byte equality (shuffled keys)
- [ ] Steps: signature verification success/failure
- [ ] Steps: run SSG build and assert files exist and contain expected strings
- [ ] Steps: deterministic build (build twice and byte-compare)
- [ ] Steps: truncation notice for large artifacts
- [ ] Steps: Proofdown unknown component -> error
- [ ] Steps: badges JSON shape and values

---

## Worker (Cloudflare)

- [ ] Create `worker/` with `wrangler.toml` and `src/index.ts`
- [ ] Env bindings:
  - [ ] `INDEX_PUBKEY_ED25519`, `RAW_BASE_URL`, `INDEX_PATH`, `INDEX_SIG_PATH`, `CACHE_TTL_SECONDS`, `FEATURES`
- [ ] Implement routes:
  - [ ] `/`, `/fragment/:id`, `/a/:id`, `/download/:id`, `/health`, `/robots.txt`
- [ ] Verify manifest signature (WebCrypto); lazily verify artifact SHA-256 on stream
- [ ] Integrate Proofdown WASM parser; render via `renderers` (WASM or TS glue)
- [ ] Badges: `/badge/:kind.json` and `/badge/:kind.svg` with ETag/Cache-Control
- [ ] Strict fetch policy under `RAW_BASE_URL`; path normalization; strong CSP headers
- [ ] ETag formulas and conditional requests
- [ ] Error pages with HTTP 409 for verification failures

---

## Accessibility & UX (`.specs/40_accessibility.md`)

- [ ] Automated a11y checks (axe/pa11y) for `site/index.html` and representative `a/*` pages
- [ ] Verify keyboard navigation and focus styles in generated HTML
- [ ] Tables use `<thead>`, `<tbody>`, and `scope`
- [ ] JSON/fragment viewers operable via keyboard (expand/collapse)
- [ ] Images require `alt` text; decorative images have empty `alt`

---

## CI & Determinism

- [ ] GitHub Actions workflow
  - [ ] `cargo fmt` / `clippy` / `build` for all crates
  - [ ] Unit & integration tests
  - [ ] BDD features
  - [ ] Manifest schema + canonicalization + signature verify for `examples/minimal/` and crate manifests
- [ ] Determinism gate: build twice and diff checksums of key outputs
- [ ] Provenance checks for examples and crate manifests

---

## Documentation & Templates

- [ ] Top-level `README.md`: project overview, how to run SSG & Worker locally, how to generate/sign manifests
- [ ] Per-crate `README.md` and `.specs` stubs:
  - [ ] `00_goals.md`, `01_plan.md`, `02_api.md`, `03_testing.md`
- [ ] Add `templates/crate/*` starter files referenced by `.specs/22_crate_minimums.md`

---

## Definition of Done (v1)

- [ ] Signed manifest verified; per-artifact SHA-256 verified
- [ ] Front page Proofdown parsed/rendered with whitelisted components; unknowns error
- [ ] Minimum viewers implemented: `markdown`, `json`, `table:coverage`, `summary:test`, `image`
- [ ] Deterministic outputs; golden tests for pages and badges
- [ ] Badge outputs/logic validated (JSON + SVG)
- [ ] Clear error states (signature/digest failures) with appropriate HTTP codes (Worker)
- [ ] Baseline accessibility checks pass

Notes:

- Artifact ids, paths, media types, and `render` hints must conform to schema in `schemas/manifest.schema.json` and rules in `.specs/44_repo_contract.md`.
- Only resources listed in the verified manifest may be fetched or rendered.
