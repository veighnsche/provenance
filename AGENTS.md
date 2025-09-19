# AGENTS.md — Internal IDE AI Operating Handbook (Provenance Monorepo)

This document is for our internal IDE AI agents working on this repository only. It defines how to operate inside this monorepo to achieve Provenance v1 goals. This is not for external adopters; external projects should follow `./.specs/46_llm_developer_contract.md`.

> IMPORTANT: External Submodule Ownership — `crates/proofdown_parser`, `crates/proofdown_renderer`
>
> The Proofdown parser and the Proofdown HTML renderer are maintained by external consultants in nested workspaces (git submodules).
> Do NOT modify code in these submodules in this repository. Propose changes in the respective submodule repos and update the
> submodule pointers here. In this workspace, only integration wiring/feature gating should be changed.

---

## Quick Add‑On Prompt (copy/paste for the internal IDE AI)

```
System: You are the internal IDE AI for the Provenance monorepo. Your job is to make this repo comply with our specs and produce verifiable, deterministic outputs. Follow the order:
SPECS → CONTRACTS → TESTING ARTIFACTS → PROOF PAGE → CODE.

Rules (internal):
- Use repository specs in .specs/ as the source of truth.
- Prefer adding/updating tests and artifacts before changing production code.
- Do not fetch from the network during builds/tests; operate read‑only against local inputs.
- Ensure deterministic outputs: stable ordering, pinned formatting, no timestamps.
- Only render/link resources listed in .provenance/manifest.json; verify sha256 before rendering.
- Keep changes small, documented, and runnable (cargo build/test green). 

Non‑goals: Do NOT mandate frameworks for external repos. That’s handled in .specs/46_*; this prompt is internal only.
```

---

## Repository Map (internal paths you will touch)

- Specs: `./.specs/`
- Features (BDD scenarios): `./features/` (scenarios exist; harness to be added)
- Schemas: `./schemas/manifest.schema.json`, `./schemas/badge.schema.json`
- Examples: `./examples/minimal/ci/*`, `./examples/minimal/.provenance/*`
- Crates:
  - `./crates/manifest_contract/` — manifest types, load/validate, canonicalize, verify
  - `./crates/proofdown_parser/` — Proofdown AST + parser (EXTERNAL SUBMODULE; DO NOT MODIFY HERE)
  - `./crates/proofdown_renderer/` — Proofdown AST → safe HTML (EXTERNAL SUBMODULE; DO NOT MODIFY HERE)
  - `./crates/renderers/` — deterministic render helpers for non-Proofdown viewers (markdown/json/table:coverage/summary:test/image)
  - `./crates/badges/` — JSON+SVG badges (to be created)
  - `./crates/provenance_ssg/` — static site generator (bin)

---

## Guardrails (do these always)

- Run `cargo build --workspace` and keep it green.
- Run format/lints: `cargo fmt --all`, `cargo clippy --all -- -D warnings`.
- Keep imports at the top of files; avoid introducing non‑determinism.
- Never introduce network calls in library/SSG code or tests.
- Validate file paths are repo‑relative; reject `..` (path traversal).

---

## Core Internal Goals (short list)

- Wire `provenance_ssg` to use `manifest_contract` types and manifest loading.
- Add `--verify-manifest` to SSG (schema, canonicalization, Ed25519 signature).
- Implement viewers in `renderers`: `markdown`, `json`, `table:coverage`, `summary:test`, `image`.
- Use `proofdown_parser` to parse and `proofdown_renderer` to render `ci/front_page.pml`.
- Implement badges in `badges` and have SSG output `site/badge/*.json` and `.svg`.
- Ensure deterministic outputs; add golden tests as appropriate.

For the full backlog, see `./TODO.md`.

---

## Internal Workflows

### 1) When changing behavior

- Update the relevant spec(s) in `./.specs/`.
- Add/adjust tests:
  - Unit/integration tests in each crate.
  - BDD steps (once the harness exists) to exercise `features/*.feature`.
- Regenerate or update example artifacts in `./examples/minimal/ci/*`.
- Update `.provenance/manifest.json` with correct `sha256` values; canonicalize and sign to produce `.sig`.
- Build the site: `cargo run -p provenance_ssg -- --root examples/minimal --out site`.
- Verify deterministic outputs and correctness.

### 2) Manifest & signing (dev)

- Canonicalization (internal helper to implement in `manifest_contract`): JSON parse → sort keys → serialize UTF‑8 with `\n` newlines.
- Ed25519 signing helper (internal): sign canonical bytes → write Base64 signature to `.provenance/manifest.json.sig`.
- Do not commit private keys. Use a test key locally and commit only the public key when needed in tests.

### 3) Proofdown front page

- Author `examples/minimal/ci/front_page.pml` using only whitelisted components.
- Use artifact `id` (not path) in components like `<artifact.summary id="tests-summary"/>`.
- Parser must reject unknown components/attributes with clear errors.

### 4) Viewers and truncation

- Viewers must be pure and deterministic.
- Large artifacts: implement truncation policy with a banner and a verified Download link.

---

## Commands (reference)

```bash
# Build everything
cargo build --workspace

# Lints
cargo fmt --all
cargo clippy --all -- -D warnings

# Build example site deterministically
cargo run -p provenance_ssg -- --root examples/minimal --out site

# (Once added) run BDD harness
# cargo run -p bdd_harness -- --features ./features
```

---

## Quality Gates (internal)

- Manifest validates against schema; canonicalization is stable; signature verifies.
- SHA‑256 digest verified for every artifact before rendering.
- Proofdown parse/lint passes; unknown components/attrs are errors.
- Minimum viewers implemented and used: `markdown`, `json`, `table:coverage`, `summary:test`, `image`.
- Proofdown front page rendered via `proofdown_renderer` with whitelisted components; unknown components/attrs cause clear errors.
- Deterministic outputs (repeatable, byte‑identical for same inputs).
- Badges (JSON+SVG) derived only from verified inputs.
- Accessibility baseline for generated pages (headings, landmarks, table semantics).

---

## Do / Don’t

- Do: keep tasks small and incremental; update `./TODO.md` as you complete items.
- Do: add descriptive errors and logs where helpful (without leaking sensitive data).
- Do: ensure imports at file tops; keep modules tidy.
- Don’t: fetch external resources in builds/tests.
- Don’t: render/link resources not in the manifest.
- Don’t: introduce nondeterminism (timestamps, random ordering, unstable maps).

---

## Cross‑refs

- External integration prompt: `./.specs/46_llm_developer_contract.md`
- Workspace plan: `./.specs/18_workspace.md`
- Testing & tiers: `./.specs/20_testing_and_tiers.md`
- Repo contract: `./.specs/44_repo_contract.md`
- Canonicalization: `./.specs/12_canonicalization.md`
- Signing: `./.specs/14_signing.md`
- Accessibility: `./.specs/40_accessibility.md`
