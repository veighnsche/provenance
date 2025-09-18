# Human Review Contract — Provenance (v1‑draft)

This document defines the human‑facing contract: which URLs are available, how to navigate and review testing artifacts and documentation, and how the UI ensures AI‑generated code is human‑reviewable.

Related specs: `./00_provenance.md`, `./10_proofdown.md`, `./16_worker.md`.

---

## 0. Objectives

- Make AI‑generated code reviewable by prioritizing specs, contracts, testing artifacts, and documented proof over implementation.
- Provide predictable, stable URLs for deep linking and audit trails.
- Present verified evidence in a safe, readable UI with explicit trust indicators.

---

## 1. Public Routes (stable)

- `/` — Front page (Proofdown). Overview KPIs and curated links to artifacts and documentation.
- `/a/{artifact_id}` — Dedicated artifact page using the appropriate verified viewer.
- `/fragment/{artifact_id}` — Streams a single heavy fragment (e.g., large JSON/coverage) on demand.
- `/download/{artifact_id}` — Direct download of the verified bytes (ETag = `sha256:<hex>`).
- `/badge/{kind}.svg|json` — Badges for GitHub READMEs (see `./00_provenance.md#15-badges-github-readme-integration`).

Notes
- All routes are commit‑pinned via the verified Index and MUST NOT serve unlisted resources.
- Deep links remain stable for the lifetime of a commit.

---

## 2. How to review (human workflow)

1) Open `/` and scan the KPIs and summary cards.
2) Follow links to `/a/tests-summary`, `/a/coverage`, and `/a/failures`.
3) For details, expand fragments (or visit `/fragment/{id}`) to stream heavy payloads.
4) Use verified download buttons for raw evidence.
5) Follow repository links/snippets embedded in Proofdown to view code in context (commit‑pinned).
6) Confirm trust indicators: signature verified, digest verified. Any failure must be explicit.

Acceptance checklist (minimum)
- Specs and contracts updated and referenced.
- Tests present; test summary shows expected totals and durations.
- Coverage table present and above threshold as required by the project.
- Failures (if any) are transparent, with remediation notes.
- Key code paths are linked/snippet‑embedded with line ranges and, where available, symbol links.
- All links point to verified resources; downloads show exact bytes.

---

## 3. Reviewability features (UI)

- Verified markers
  - Index signature status displayed on the front page.
  - Per‑artifact digest status displayed adjacent to viewers and downloads.
- Readability
  - Collapsible JSON trees; CommonMark/GFM subset (including tables); raw HTML not supported (escaped as text); responsive images.
  - Structured layout components (`grid`, `section`, `card`, `tabs`).
- Deep linking
  - Artifact pages at `/a/{id}`; fragments at `/fragment/{id}`; anchors within Proofdown using `[[doc:#anchor]]`.
- Code context (commit‑pinned)
  - `repo:` links and `<repo.code/>` snippets for file/line ranges; `<repo.tree/>` for directory views (when configured).
  - Optional symbol links/snippets via a verified symbols artifact.
- Evidence composition
  - `<include.pml id="..."/>` to compose larger documents from smaller, commit‑pinned partials.
- Accessibility (SHOULD)
  - Sufficient color contrast; keyboard navigable; semantic headings; `alt` text for images.

---

## 4. Error and degraded states

- Signature fail → Front page shows a clear error and halts rendering.
- Digest fail → Specific artifact panels show an error banner and avoid rendering unsafe bytes.
- Missing artifact → Placeholder with remediation hint (e.g., not emitted by CI for this commit).
- Oversize artifacts/snippets → Truncated with a notice and a verified download link.

---

## 5. Evidence taxonomy (recommended)

- Specs & contracts — human authored; Proofdown sections for scope and acceptance criteria.
- Tests — unit/integration/e2e summaries with durations and flake notes.
- Coverage — total and per‑module tables; highlight critical deltas.
- Failures — filtered list with reproduction steps.
- Code links — snippets for critical paths with ranges and optional symbol targets.
- Extra provenance — SPDX/in‑toto, dependency SBOMs, security scan results (optional viewers).

---

## 6. Examples (deep links)

- Overview: `/` — curated KPIs and links.
- Test summary: `/a/tests-summary` — KPIs and breakdown.
- Coverage: `/a/coverage` — sortable table; download link.
- Failures: `/a/failures` — Markdown list with anchors.
- Verified download: `/download/tests-summary` — exact JSON bytes with ETag = `sha256:<hex>`.

---

## 7. Non‑goals

- Arbitrary code browsing (defer to GitHub except for commit‑pinned snippets/trees per the Index).
- Rendering or fetching resources not present in the verified Index.
