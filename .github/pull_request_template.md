# Provenance PR Checklist

Use this checklist to qualify for the provenance badge.

## Required

- [ ] Specs/contracts updated for behavior changes (`.specs/*`).
- [ ] Tests updated/added; artifacts emitted (`tests-summary`, `coverage`, `failures` if any).
- [ ] Proofdown front page updated (`ci/front_page.pml`) and references artifacts by `id`.
- [ ] Manifest updated: `.provenance/manifest.json` lists all artifacts with `sha256`.
- [ ] Manifest signed: `.provenance/manifest.json.sig` (Ed25519 over canonical bytes).
- [ ] README badges present or updated (`/badge/provenance.svg`, `/badge/tests.svg`, `/badge/coverage.svg`).

## Links

- [ ] Worker root: <https://YOUR_WORKER_DOMAIN/>
- [ ] Artifacts: `/a/tests-summary`, `/a/coverage`, `/a/failures` (if present)
- [ ] Download links verified (`/download/{id}`)

## Self‑review

- [ ] Unknown components/attrs rejected; Proofdown parses.
- [ ] Repo links/snippets resolve (per‑file artifacts or bundle present).
- [ ] a11y basics: headings, keyboard nav, contrasts, alt texts.

---

For IDE AIs: follow the LLM Developer Contract `.specs/12_llm_developer_contract.md`.
