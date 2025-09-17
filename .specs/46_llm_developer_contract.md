# LLM Developer Contract — Provenance (v1‑draft)

This document defines the contract for AI/LLM developers and IDE assistants to qualify for a Provenance badge. It specifies the expected prompts, workflows, artifacts, and acceptance gates enforced by CI and the Worker.

Related specs: `./00_provenance.md`, `./10_proofdown.md`, `./16_worker.md`, `./44_repo_contract.md`.

---

## 0. Purpose & Scope (framework‑agnostic)

- Use Provenance without changing your language, test framework, or CI system.
- Your goal is to emit verifiable testing evidence and a signed manifest so a Provenance mirror can safely render it.
- Principle: ship evidence, not claims → SPECS → CONTRACTS → TEST ARTIFACTS → PROOF PAGE → CODE.
- Human reviewability: the front page (Proofdown) summarizes KPIs and links to details; all links resolve only to verified artifacts.

---

## 1. Required outputs (minimum, framework‑agnostic)

- Test artifacts (recommended paths — adjust for your repo):
  - `tests-summary` (render `summary:test`) → `ci/tests/summary.json`
    - Shape: `{ "total": <int>, "passed": <int>, "failed": <int>, "duration_seconds": <number> }`
  - `coverage` (render `table:coverage`) → `ci/coverage/coverage.json`
    - Shape: `{ "total": { "pct": <number> }, "files": [ { "path": "<file>", "pct": <number> } ] }`
  - `failures` (optional; render `markdown`) → `ci/tests/failures.md`
- Front page Proofdown → `ci/front_page.pml`
  - Use only whitelisted components; reference artifacts by `id` (not by path).
- Signed manifest → `.provenance/manifest.json` and `.provenance/manifest.json.sig`
  - Every artifact listed with `id`, `title`, `path`, `media_type`, `render`, `sha256` (hex, lowercase).
  - Signature: Ed25519 over canonical JSON bytes (sorted keys, UTF‑8, `\n` newlines), signature stored as Base64 in `.sig`.

---

## 2. IDE assistant prompt (copy/paste)

Use the following add‑on prompt with your IDE AI. It does not change frameworks; it only instructs how to emit Provenance outputs.

```
You are integrating this repository with Provenance (artifact‑first mirror). Do not change our language, test framework, or CI.

Your job is to ensure the repo emits and publishes:
1) Testing artifacts produced by our existing tests:
   - tests summary JSON (id: tests-summary, render: summary:test)
   - coverage JSON (id: coverage, render: table:coverage)
   - failures.md (optional; id: failures, render: markdown)
2) A front page at ci/front_page.pml using only whitelisted components and referencing artifacts by id.
3) A signed manifest at .provenance/manifest.json with sha256 for each artifact, plus .provenance/manifest.json.sig.

Rules:
- Manifest is canonicalized JSON (sorted keys, UTF‑8, \n newlines) signed with Ed25519; .sig stores Base64 signature.
- Only resources listed in the manifest may be rendered/linked. Verify artifact SHA‑256 before use.
- Outputs must be deterministic (no timestamps/random order). No unverified external fetches.
- Publish manifest + signature + artifacts at a commit‑pinned URL.

Outcome: a PR that includes the new/updated artifacts, Proofdown front page, signed manifest, and instructions/CI to publish them.
```

---

## 3. Publishing & CI (non‑prescriptive)

Use your existing test/coverage jobs. Add the following steps:
1) Persist test artifacts (summary.json, coverage.json, optional failures.md) at stable paths.
2) Compute SHA‑256 for each artifact and update `.provenance/manifest.json` accordingly.
3) Canonicalize the manifest and sign with Ed25519; write Base64 signature to `.provenance/manifest.json.sig`.
4) Publish manifest, signature, and artifacts to a commit‑pinned location (e.g., GitHub Raw for the commit SHA, or an equivalent static snapshot).
5) Treat verification failures (manifest sig or artifact digest) as CI errors.

---

## 4. Acceptance gates (CI)

- Canonicalization + Ed25519 signature verification for the manifest.
- SHA‑256 digest verification for every listed artifact before rendering/linking.
- Proofdown parse/lint; unknown components/attributes are hard errors.
- Supported viewers (Provenance minimum): `markdown`, `json`, `table:coverage`, `summary:test`, `image`.
- Deterministic outputs (repeatable builds yield identical bytes for equivalent inputs).
- Optional: coverage thresholds (e.g., ≥ 80%).

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
  - Emit/refresh `tests-summary` and `coverage` artifacts.
  - Update `ci/front_page.pml` to present new KPIs and link to details.
  - Update `.provenance/manifest.json` sha256 values, re‑canonicalize, and re‑sign.

- Fix a bug:
  - Capture failing evidence (e.g., failures.md); after fix, show delta via updated artifacts.

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
  - Validate manifest/inputs as appropriate for your stack
- Provide a dev `wrangler.toml` and Miniflare config for local verification.
