# Rust Workspace Plan (v1-draft)

This document outlines the crates in the Provenance monorepo and how they wire together for a read-only, deterministic static site build.

---

## Crates and responsibilities

- `provenance_ssg` (bin)
  - Static site generator (read-only RSX/HTML + inline CSS)
  - Loads `.provenance/manifest.json`, verifies SHA-256 per artifact
  - Renders `/index.html`, `/a/{id}/index.html`, `/robots.txt`
  - Will parse and render the Proofdown front page into RSX
  - Will generate static badges JSON/SVG files under `/badge/`
  - Depends on: `manifest_contract`, `proofdown_parser`, `renderers`, `badges`

- `manifest_contract` (lib)
  - Manifest types, canonicalization helper, basic validation
  - JSON Schema validation (future); Ed25519 verify helper (future)
  - Provides stable API to load/validate manifests

- `proofdown_parser` (lib)
  - Parses `.pml` Proofdown files into a stable AST (no network/IO)
  - Validates components/attributes and link macro forms syntactically
  - No rendering; pure parse + AST datatypes

- `renderers` (lib)
  - Pure, deterministic render helpers for known viewers (markdown/json/table:coverage/summary:test/image)
  - Maps AST + verified data → RSX/HTML

- `badges` (lib)
  - Derives badge values from verified inputs (provenance/tests/coverage)
  - Produces Shields-compatible JSON and lightweight SVG

---

## Data and control flow (static build)

1) `provenance_ssg` loads and validates the manifest via `manifest_contract`.
2) It computes SHA-256 of artifacts and marks verified status for display.
3) It parses the Proofdown front page via `proofdown_parser`.
4) It renders AST and viewers via `renderers` into RSX/HTML.
5) It writes static files and badges via `badges`.

---

## Testing strategy (Gherkin + Rust)

- Spec features in `features/*.feature` (Gherkin) describe behavior:
  - Manifest validity, canonicalization, and signing
  - Static rendering (index, per-artifact pages)
  - Proofdown front page parsing (structural components + artifact embeds)
  - Badges outputs (JSON/SVG)
  - Security constraints (no external fetch; path traversal blocked)
- Rust test harness will exercise SSG and libraries against example fixtures.

---

## Non-goals (v1)

- Dynamic runtime or client-side interactivity
- Network fetching beyond local repo inputs during build
- Repo bundle extraction (may be added later with strict limits)
