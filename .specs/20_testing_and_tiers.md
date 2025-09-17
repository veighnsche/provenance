# Testing Methodology and Readiness Tiers (v1‑draft)

This document defines how we test in the Provenance workspace and what each crate must provide at each readiness tier (pre‑alpha → production). It also specifies crate‑level provenance conventions so this repository functions as a provenance example.

Related: `./00_provenance.md`, `./01_proofdown.md`, `./02_worker.md`, `./03_canonicalization.md`, `./04_signing.md`, `./06_workspace.md`.

---

## 0. Principles

- Evidence first: tests, artifacts, and Proofdown pages precede feature claims.
- Determinism: identical inputs → identical outputs (byte‑stable RSX HTML and badges).
- Safety: no unverified or external fetches in tests; crate builds are read‑only.
- Human reviewability: artifacts are summarized on crate front pages with linkable details.

---

## 1. Testing Methodology (workspace)

- Behavior‑Driven Specs (Gherkin)
  - `.feature` files live in `features/` at the repo root and specify end‑to‑end behaviors: manifest validity, signing, SSG rendering, Proofdown front page, badges, and security.
  - Executed by a Rust BDD harness (e.g., `cucumber` crate) that drives the SSG and validators against fixtures under `examples/`.

- Crate Unit Tests
  - Each crate provides `src/` unit tests and `tests/` integration tests for its domain (e.g., parser grammar, schema validation helpers, renderer helpers).

- Golden Tests (determinism)
  - HTML and badge JSON outputs are compared to golden files stored under `tests/golden/` or generated under a temp dir and hashed.
  - Acceptable differences MUST be zero; outputs are byte‑identical between runs.

- Security Tests
  - Path normalization rejects `..` and absolute paths.
  - If/when repo bundles are introduced, zip‑slip/symlink defenses are validated.
  - Oversize artifacts trigger truncation banners with verified download links.

- Accessibility Smoke
  - Generated pages SHOULD pass basic automated checks (e.g., headings, landmarks, color contrast); manual checks recommended for RC.

- CI Gates
  - `cargo fmt` / `clippy` / `build` for all crates
  - Run BDD features and crate tests
  - Validate manifest schema + canonicalization + signature for `examples/minimal/`

---

## 2. Crate‑level Provenance Conventions

Every crate is itself a provenance example. Each crate directory MUST follow:

```
crates/<name>/
├── src/
├── README.md                         # crate docs and usage
├── .provenance/
│   ├── manifest.json                 # crate‑scoped manifest
│   └── manifest.json.sig             # Ed25519 signature (Base64) over canonical bytes
└── ci/
    ├── front_page.pml                # Proofdown (crate front page)
    ├── tests/summary.json            # if tests exist
    ├── coverage/coverage.json        # if coverage is measured
    └── tests/failures.md             # optional
```

Notes
- The root SSG can be pointed at a crate via `--root crates/<name>` to build a crate‑scoped static site into `crates/<name>/site/`.
- The top‑level `examples/minimal/` remains a canonical cross‑crate example; crates provide additional, crate‑specific minimal manifests.

---

## 3. Readiness Tiers and Criteria

- Pre‑Alpha
  - Crate compiles; `cargo fmt`/`clippy` pass with no errors.
  - Crate README.md exists outlining purpose and scope.
  - Crate `.provenance/manifest.json` exists (may be partial); front_page.pml stub.
  - Unit test skeleton in place (at least one test).

- Alpha
  - Minimal features implemented per crate (see §4).
  - Crate manifest validates against `schemas/manifest.schema.json` (repo schema).
  - Canonicalization produces stable bytes; crate manifest is signed (sig file present).
  - Crate front page renders with SSG (index + at least one artifact page).
  - Golden test for at least one output.

- Beta
  - All required features (per crate) implemented for v1 spec.
  - Deterministic build verified (byte‑identical outputs across runs for crate site).
  - Error handling: unknown inputs produce explicit errors; truncation banners present.
  - Basic a11y smoke checks OK on crate front page and one artifact page.

- Release Candidate (RC)
  - Full spec compliance for the crate; BDD features covering the crate pass.
  - Security tests: path normalization, no external fetches; if bundle present, zip‑slip defenses tested.
  - Docs complete (crate README includes usage, provenance instructions).

- Release Ready
  - Version pinned; CHANGELOG updated; license headers audited.
  - Reproducible builds on CI (Linux; macOS optional).
  - All artifacts referenced by crate manifest are produced by CI.

- Publish Ready
  - `cargo publish --dry-run` succeeds for libraries; binaries (SSG) have clear install docs.
  - README badges link to crate site badges and artifact pages.

- Production Ready
  - Security review recorded; threat model updated.
  - SLOs/docs for maintenance; no P0/P1 issues open.
  - Provenance artifacts routinely updated in CI (manifests + signatures).

---

## 4. Minimal Requirements by Crate (v1)

- `provenance_ssg` (bin)
  - Alpha: render index/per‑artifact pages from root manifest; verify sha256; markdown/json/image/coverage/test summary viewers; robots.txt.
  - Beta: parse and render front_page.pml (structural + artifact.* + links); truncation banners; deterministic output; static badges JSON.
  - RC: static badges SVG; `--verify-manifest` flag (canonicalize + Ed25519); golden tests; a11y smoke on index and at least one artifact page.

- `manifest_contract` (lib)
  - Alpha: manifest types + load; basic validation; helpers for common fields.
  - Beta: JSON Schema validation; canonicalization helper; Ed25519 verification helper.
  - RC: golden canonicalization bytes for test vectors; comprehensive validation errors.

- `proofdown_parser` (lib)
  - Alpha: parse structural components (grid, section, card) and artifact.* components; link macro parsing; unknowns error.
  - Beta: includes with depth/cycle checks; repo viewers (optional) as stubs; attribute bounds enforced.
  - RC: full minimal grammar per spec; golden AST tests for front‑page examples.

- `renderers` (lib)
  - Alpha: markdown/json/table:coverage/summary:test/image render helpers.
  - Beta: RSX components for structural layout; HTML sanitization; deterministic rendering.
  - RC: golden RSX/HTML outputs for example inputs; a11y checks for table structure.

- `badges` (lib)
  - Alpha: compute `provenance/tests/coverage` values from verified inputs; output Shields JSON.
  - Beta: minimal SVG generation; thresholds and colors; error badge on missing/invalid artifacts.
  - RC: schema tests for badge JSON; golden SVG tests.

---

## 5. How We Test (crate‑scoped)

- Unit tests under `src/` and `tests/` with focused assertions.
- Optional BDD scenarios under `crates/<name>/features/` for crate‑local behaviors (executed by the same harness).
- Golden outputs stored under `crates/<name>/tests/golden/` or generated and hashed.
- Crate build docs include a command to build a crate‑scoped site:
  - `cargo run -p provenance_ssg -- --root crates/<name> --out crates/<name>/site --verify-manifest`

---

## 6. CI Gating (summary)

- Formatting/lints: `cargo fmt --all -- --check`, `cargo clippy --all -- -D warnings`.
- Build: `cargo build --workspace`.
- Tests: BDD features + unit/integration tests.
- Provenance checks: schema + canonicalization + signature verify for example manifests and crate manifests.
- Determinism: build twice, compare checksums of key outputs (index.html, badge JSON).

---

## 7. Governance

- Tiers are documented in PRs upon graduation (e.g., “provenance_ssg → Beta”).
- Breaking changes: require a manifest version bump and a migration note; tie to release tags.
- New components/viewers: added with backward compatibility; deprecations documented.
