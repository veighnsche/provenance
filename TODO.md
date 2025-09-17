# TODO — Next 4 Weeks Execution Plan (Spec-First, Artifact-First)

> IMPORTANT: External Submodule — `crates/proofdown_parser`
>
> The Proofdown parser is maintained by external consultants in a nested workspace (git submodule).
> Do NOT modify parser code in this repository. Propose changes in the submodule repo and update the
> submodule pointer here. In this workspace, only integration wiring/feature gating should be changed.
>
> Path: `crates/proofdown_parser/` • SSG feature flag: `external_pml`

---

## Guiding Principles

- Specs → Contracts → Tests/Artifacts → Documented Proof → Code (in that order)
- Determinism: stable ordering, canonicalized formats, no time/IO randomness
- Security: verify manifest signature, verify artifact digests, strict viewer mapping
- CI one‑button: `cargo xtask check-all`, `cargo xtask test-all`, feature-gated integration build

---

## Week 1 — Contracts, Canonicalization, Test Fixtures

- Manifest contract hardening (crate: `manifest_contract`)
  - [x] Add JSON Schema success/failure test cases (good/bad fixtures)
  - [x] Canonicalization golden test (bytes match across runs)
  - [x] Ed25519 verify path test (public test key in repo)
- SSG test fixtures
  - [x] Add large JSON fixture (5–10 MB) under `examples/minimal/ci/tests/` for truncation
  - [ ] Expand `failures.md` to include representative edge cases (UTF‑8, long lines)
- CI & tooling
  - [x] Ensure GitHub CI checks out submodules, runs `cargo xtask check-all` and `test-all`
  - [ ] Add `cargo tree -d` step (optional) to detect duplicate deps
- Docs
  - [ ] Cross‑link contracts and specs in READMEs; finalize external‑ownership warnings

Exit criteria

- `cargo xtask check-all` and `test-all` green
- New fixtures present; schema and canonicalization tests pass

---

## Week 2 — Proofdown Integration Surface + Renderers Golden Tests

- Proofdown integration (without editing submodule code)
  - [x] Keep `external_pml` off by default; add an integration job that enables it
  - [ ] Add golden AST JSON fixtures (exported from submodule examples) for a few front pages
  - [ ] Add SSG-side parser integration test that loads AST JSON and exercises renderer mapping
- Renderers (crate: `renderers`)
  - [ ] Golden tests: `markdown`, `json` (pretty/escaped), `table:coverage`, `summary:test`, `image`
  - [x] Truncation policy tests using the large JSON fixture
- Badges (crate: `badges`)
  - [ ] Schema-based validation of Shields JSON shapes; tests for `provenance`, `tests`, `coverage`

Exit criteria

- Renderer golden tests in place and stable
- SSG can render index using precomputed AST JSON (when `external_pml` is off)

---

## Week 3 — Deterministic SSG Output, Accessibility, Fragments

- SSG determinism (crate: `provenance_ssg`)
  - [ ] Stable ordering for artifacts/nav, pinned float formatting, stable HTML
  - [ ] Golden HTML tests for `/index.html` and a couple of `/a/{id}` pages
- Accessibility
  - [ ] Add headings/landmarks/table semantics; alt text checks in renderers
- Fragments & downloads
  - [ ] Implement `/fragment/{artifact_id}` for heavy payloads; verified download links

Exit criteria

- Reproducible builds: golden HTML snapshots pass
- Basic accessibility checks pass in CI

---

## Week 4 — BDD Harness, Feature Gates, Release Prep

- BDD (features/)
  - [x] Add a minimal Rust test harness that executes `features/*.feature` scenarios (smoke subset)
  - [ ] Scenarios: signature failure, digest mismatch, missing artifact, basic Proofdown front page
- Feature gates & docs
  - [ ] Document `external_pml` usage and integration workflow with the submodule
  - [ ] Document how to promote `proofdown_ast` to a published crate (if/when ready)
- Release prep
  - [ ] Version tagging plan; changelog scaffolding in each crate
  - [ ] CI: release job templates (draft)

Exit criteria

- BDD smoke tests run in CI
- Clear documented path to enable `external_pml` and to update submodule pointer

---

## Stretch (As Time Allows)

- Worker PoC (Cloudflare): skeleton routes `/`, `/fragment/{id}`, `/a/{id}` using static inputs
- Repo viewers planning (spec refresh only): `repo.code`, `repo.tree`, `repo.diff` constraints
- WASM binder exploration notes for `proofdown_parser`

---

## Risks & Mitigations

- External submodule velocity misalignment
  - Mitigation: keep SSG able to run with `external_pml` disabled using AST JSON fixtures
- Non-determinism sneaks in
  - Mitigation: golden tests for AST/HTML, formatter pins, explicit ordering
- CI flakiness with large fixtures
  - Mitigation: fragments route, truncation banners, Worker cache notes

---

## Handy Commands

- Outer + nested checks: `cargo xtask check-all`
- Outer + nested tests: `cargo xtask test-all`
- Integration build (SSG with parser): `cargo xtask ci-integration-build`
- Build example site: `cargo run -p provenance_ssg -- --root examples/minimal --out site`

---

## References

- Specs: `/.specs/00_provenance.md`, `/.specs/18_workspace.md`
- Parser contract: `crates/proofdown_parser/.specs/10_contract_with_provenance_ssg.md`
- Nested workspace plan: `crates/proofdown_parser/.plans/00_workspace_plan.md`
