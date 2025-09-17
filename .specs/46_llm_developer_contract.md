# LLM Developer Contract — Provenance (v1‑draft)

This document defines the contract for AI/LLM developers and IDE assistants to qualify for a Provenance badge. It specifies the expected prompts, workflows, artifacts, and acceptance gates enforced by CI and the Worker.

Related specs: `./00_provenance.md`, `./01_proofdown.md`, `./02_worker.md`, `./11_repo_contract.md`.

---

## 0. Principles

- Ship evidence, not claims: SPECS → CONTRACTS → TESTING ARTIFACTS → DOCUMENTED PROOF → CODE.
- Every behavior change updates the spec/contract and regenerates artifacts.
- Human reviewability is a first‑class outcome: front page Proofdown must clearly present what changed and why it’s correct.

---

## 1. What you must produce (minimum)

- Updated specs/contracts when behavior changes (e.g., `.specs/` files and component grammar if relevant).
- Test artifacts:
  - `tests-summary` (render `summary:test`)
  - `coverage` (render `table:coverage`)
  - `failures` (render `markdown`) if failures exist
- Front page Proofdown at `ci/front_page.pml` referencing artifacts by `id`.
- `.provenance/manifest.json` and `.provenance/manifest.json.sig` with SHA‑256 per artifact and signed manifest.

---

## 2. IDE assistant prompt (template)

Use this structured system/user prompt to guide IDE AIs toward provenance compliance.

```
System: You are an AI software engineer working in a provenance‑first repository. Your goal is to produce verifiable evidence for any code you change. Follow the priority: SPECS → CONTRACTS → TESTING ARTIFACTS → DOCUMENTED PROOF → CODE. Do not ship changes without updated artifacts and a Proofdown front page that presents them.

Rules:
- Update `.specs/*` documents for behavior changes.
- Add/maintain tests; emit `tests-summary`, `coverage`, and `failures` artifacts.
- Author `ci/front_page.pml` in Proofdown to present the artifacts (use `artifact.*` components and `repo:` snippets for critical paths).
- Ensure `.provenance/manifest.json` lists all artifacts with `sha256` and is signed; produce `.provenance/manifest.json.sig`.
- Link all repo code snippets/paths commit‑pinned (no unverified fetches).
- Run CI gates locally (schema, canonicalization, signature, digest, Proofdown parse/lint).
- Provide README badges pointing to `/badge/*` endpoints.

Deliverables:
- A PR that passes all provenance gates and links to the Worker mirror URL showing the verified proof page for the commit.
```

---

## 3. Developer workflow

1) Make the change behind a spec/contract update.
2) Add/adjust tests; run them; collect outputs.
3) Update `ci/front_page.pml` to include summaries, coverage, failures, and code snippets for critical paths.
4) Generate `.provenance/manifest.json` with SHA‑256 per artifact and sign it (produce `.provenance/manifest.json.sig`).
5) Publish artifacts to the commit‑pinned location (or ensure CI does); verify locally using the same signature key pair.
6) Open a PR with:
   - Links to `/` and key `/a/{id}` pages.
   - README badges (or confirm they already exist).
7) Address lints and fix any verification errors.

---

## 4. Acceptance gates (CI)

- Index schema validation + canonicalization check.
- Ed25519 signature verification.
- SHA‑256 digest verification for all artifacts.
- Proofdown parse/lint; unknown components = error.
- Renderer coverage: `summary:test`, `table:coverage`, `markdown` at minimum.
- Optional: thresholds (e.g., coverage ≥ 80%).

PR must include

- Spec/contract diffs when behavior changes.
- Updated artifacts and signed index.
- Links to the provenance mirror.

---

## 5. Proofdown checklist for AI authors

- Use `<grid>`, `<card>`, `<section>` for structure.
- Reference artifacts by `id` only; never by path.
- Use `[[a:<id>]]` for artifact deep links and `[[repo:<path>#L..]]` or `<repo.code/>` for code context (must be resolvable via verified artifacts/bundle).
- Prefer `<artifact.summary/>`, `<artifact.table/>`, `<artifact.json/>`, `<artifact.markdown/>` for rendering.
- Keep includes under depth limit; avoid cycles.
- Keep snippet ranges modest; add download links for large evidence.

---

## 6. Examples

- Add a feature with new tests:
  - Update `.specs/` with the new behavior.
  - Update tests; emit new `tests-summary`/`coverage`.
  - Add a new section to `ci/front_page.pml` showing KPIs and a `<repo.code/>` snippet of the changed function.
  - Regenerate and sign `.provenance/manifest.json`.

- Fix a bug:
  - Add a failing test first; capture failure evidence; then fix and show delta in Proofdown with a `repo.diff` (if bundles enabled).

---

## 7. Anti‑patterns (will fail gates)

- Changing behavior without updating specs/contracts.
- Missing or unsigned `.provenance/manifest.json`.
- Unreferenced artifacts or artifacts without `sha256`.
- Proofdown with unknown components/attrs or unresolved `repo:` links.
- README badges pointing to dynamic or unverified routes.

---

## 8. Tooling suggestions

- Provide scripts to:
  - Build artifacts and compute SHA‑256
  - Canonicalize and sign `.provenance/manifest.json`
  - Lint Proofdown
  - Validate schema
- Provide a dev `wrangler.toml` and Miniflare config for local verification.
