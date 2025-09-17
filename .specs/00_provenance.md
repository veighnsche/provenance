# Artifact-First Testing Mirror — Spec (v1)

## 0. Terminology

- **MUST/SHOULD/MAY** follow RFC-2119.
- **Index (Manifest)**: the signed config that enumerates all renderable artifacts and the front-page markup. By convention it is published at `.provenance/manifest.json` with signature `.provenance/manifest.json.sig`.
- **Artifact**: any file produced by CI and referenced by the Index.
- **Proofdown**: the safer-than-HTML markup rendered by the Worker (working name). See `.specs/01_proofdown.md`. Recommended extension: `.pml`.
- **Worker**: the Cloudflare Worker serving the site.

---

## 1. Source of Truth & Provenance

- The system **MUST** treat a single **Index** file as the sole source of truth.
- The Index **MUST** be produced by CI and **MUST** be cryptographically signed (Ed25519).
- The Worker **MUST** verify the Index signature before rendering any content.
- The Index **MUST** include SHA-256 digests for every referenced Artifact.
- The Worker **MUST** verify each Artifact’s digest before using or linking it.
- The Index **SHOULD** record CI metadata (repo, commit, run id/url, attempt, generated_at).
- The system **MAY** include additional provenance (e.g., in-toto/SLSA attestations).

---

## 2. Index File

- The Index **MUST** specify:
  - `version` (integer)
  - `repo` (owner/repo)
  - `commit` (full SHA)
  - `workflow_run` (id, url, attempt)
  - `front_page.markup` (path to Proofdown file, e.g., `ci/front_page.pml`)
  - `front_page.title` (string)
  - `artifacts[]` (array)
- Each `artifacts[]` item **MUST** include:
  - `id` (unique, kebab/slug)
  - `title` (human label)
  - `path` (repo-relative)
  - `media_type` (MIME)
  - `render` (renderer hint)
  - `sha256` (hex)
- The Index **SHOULD** be JSON or TOML; YAML **MAY** be used if parser weight is acceptable.
- The Index **MAY** include grouping/section metadata for navigation.
 - Recommended publish path to avoid collisions: `.provenance/manifest.json` (signature: `.provenance/manifest.json.sig`).

---

## 3. Artifacts

- Every renderable file **MUST** be registered in the Index; unregistered files **MUST NOT** be rendered.
- Artifact paths **MUST** be immutable for a given commit (`HEAD`, tag, or fixed branch ref).
- Artifacts **SHOULD** be placed under `ci/` (convention), but this is not required.
- Large Artifacts **MAY** be truncated in views, with a clearly labeled download link.

---

## 4. Front Page Markup (Proofdown)

- The front page **MUST** be authored in Proofdown and referenced by `front_page.markup` in the Index.
- Proofdown **MUST NOT** allow arbitrary HTML/JS/CSS injection.
- Proofdown **MUST** support a whitelisted set of structural components (e.g., `grid`, `tabs`, `card`, `section`, `gallery`).
- Proofdown **MUST** support **artifact components** that reference Artifacts **by `id` only** (e.g., `artifact.markdown`, `artifact.table`, `artifact.json`, `artifact.image`, `artifact.summary`, `artifact.gauge`, `artifact.viewer kind="llm-proof"`).
- Proofdown **MUST** allow data interpolation from verified Index fields via a safe placeholder syntax (e.g., `{{ commit }}`).
- Proofdown **SHOULD** provide a link macro as specified in `.specs/01_proofdown.md` supporting `a:` artifact links and commit-pinned `repo:` links that resolve only to verified resources.
- The renderer **MUST** reject any component referencing an unknown `id`.
- The language **SHOULD** be deterministic and side-effect-free.
- The renderer **MAY** expose a minimal style system (e.g., component attributes like `cols=3`, `gap=16`) but **MUST NOT** accept raw style strings.
- Recommended file extension for Proofdown is `.pml` (Proof Markup Language).

---

## 5. Worker Behavior

- The Worker **MUST** run in the Cloudflare Workers/V8 isolate (no threads, no filesystem).
- On request, the Worker **MUST**:
  1) Fetch Index and signature,
  2) Verify signature,
  3) Verify digests lazily per Artifact as needed,
  4) Render Proofdown → HTML,
  5) Serve HTML.
- Fragment routes (e.g., `/fragment/{artifact_id}`) **SHOULD** be supported to stream/render heavy views on demand (htmx-friendly).
- The Worker **SHOULD** implement caching (ETag/If-None-Match) and **MAY** use Workers Cache with bounded TTL.
- The Worker **MAY** provide deep links `/a/{id}` to render a single Artifact page.

---

## 6. Security

- If Index signature verification fails, the Worker **MUST** refuse to render and **MUST** show a clear error state.
- If an Artifact digest fails verification, the Worker **MUST** refuse to render that Artifact and **MUST** indicate the failure.
- The Worker **MUST NOT** fetch or render any resource not listed in the verified Index.
- The Worker **MUST** sanitize all text output and **MUST NOT** execute client-supplied scripts.
- Download links **MUST** point to the exact verified resource (commit-pinned Raw URL or Worker-proxied verified stream).

---

## 7. CI Responsibilities

- CI **MUST** produce all Artifacts referenced by the Index.
- CI **MUST** compute and embed SHA-256 digests for each Artifact in the Index.
- CI **MUST** sign the Index with the CI private key; the corresponding public key **MUST** be configured in the Worker.
- CI **SHOULD** fail the pipeline if the Index is malformed, unsigned, or references missing files.
- CI **MAY** publish to a dedicated `ci-snapshots` branch; the Worker **MAY** be configured to read from that branch.

---

## 8. Renderers

- The renderer **MUST** map `render` hints + `media_type` to a fixed set of viewers.
- Minimum viewers that **MUST** be implemented:
  - `markdown` → safe Markdown to HTML
  - `json` → collapsible JSON tree
  - `table:coverage` → coverage JSON to table
  - `summary:test` → test summary JSON to KPIs (total/passed/failed/duration)
  - `image` → responsive image
- Specialized viewers (e.g., `viewer:llm-proof`) **SHOULD** be supported if `llm_proof.json` is provided.
- Unknown `render` values **MUST** fall back to a generic safe viewer or be rejected with a clear message.

---

## 9. UX & Navigation

- The site **MUST** prioritize browsing testing evidence over code (code browsing is deferred to GitHub).
- A navigation menu **SHOULD** be auto-generated from `artifacts[]` (grouping if provided).
- The front page **SHOULD** present a concise human overview (KPIs, badges, links to details).
- Each Artifact view **SHOULD** include a verified **Download** action.
- Long content **MAY** be paginated or collapsible.

---

## 10. Performance & Limits

- The Worker **SHOULD** stream responses where practical (fragments).
- The Worker **SHOULD** enforce per-Artifact size/time limits and degrade gracefully.
- The system **MAY** cache verified bytes (by SHA) to avoid refetching.

---

## 11. Configuration & Secrets

- The Worker **MUST** store the verification public key securely (Worker secret/env).
- The Worker **MAY** accept a static GitHub raw base (owner/repo/branch) per deployment; dynamic routing **SHOULD** validate inputs to prevent SSRF/path traversal.
- Feature flags **MAY** be provided to enable/disable experimental viewers.

---

## 12. Error Handling

- All verification failures **MUST** produce human-friendly error pages with stable HTTP codes (e.g., 502/409).
- Missing Artifacts **SHOULD** be indicated inline with remediation hints (e.g., “not emitted by CI for commit X”).
- Parser errors in Proofdown **MUST** surface with a line/column pointer.

---

## 13. Extensibility

- New components/viewers **MAY** be added without breaking existing Proofdown (backward compatible grammar).
- Index `version` **MUST** gate incompatible changes; the Worker **MUST** reject unknown major versions.
- Additional provenance formats **MAY** be attached (e.g., SPDX, in-toto) and surfaced by dedicated viewers.

---

## 14. Compliance Checklist (MINIMUM)

- ✅ Signed Index verified
- ✅ SHA-256 per Artifact verified
- ✅ Front page Proofdown rendered with whitelisted components
- ✅ Test summary + Coverage + Failures renderers present
- ✅ Deep links + Downloads present
- ✅ Clear failure states for signature/digest errors
- ✅ Badge endpoints exposed (`/badge/provenance.svg|json`, `/badge/tests.svg|json`, `/badge/coverage.svg|json`)

---

## 15. Badges (GitHub README integration)

- The Worker **SHOULD** expose read‑only badge endpoints for GitHub READMEs linking to the provenance mirror.
- Badges **MUST** be derived only from verified inputs (signed Index + required artifacts) and **MUST NOT** fetch unlisted resources.
- Supported endpoints (minimum):
  - `GET /badge/provenance.svg` — displays overall provenance status (e.g., `verified` if Index signature verifies; `error` otherwise).
  - `GET /badge/tests.svg` — summarizes test results from the `summary:test` artifact (e.g., `123 passed`, `2 failed`).
  - `GET /badge/coverage.svg` — displays coverage percentage from the coverage artifact.
  - JSON variants compatible with Shields.io: `GET /badge/<kind>.json` returning `{ schemaVersion: 1, label, message, color }`.
- Optional query parameters (enumerated; any other parameter **MUST** be rejected):
  - `style=flat|flat-square` (default: `flat`)
  - `label=<safe-label>` (alphanumeric + `-_` only; length limit)
- Caching: badges **SHOULD** set `ETag` and `Cache-Control` (short TTL; e.g., 60–300 seconds) and respect `If-None-Match`.
- Security: badge routes **MUST** operate against `RAW_BASE_URL` configured for the deployment (owner/repo/commit or branch). They **MUST NOT** accept dynamic origins.
- Example README usage:

```md
[![Provenance](https://<worker-domain>/badge/provenance.svg)](https://<worker-domain>/)
[![Tests](https://<worker-domain>/badge/tests.svg)](https://<worker-domain>/a/tests-summary)
[![Coverage](https://<worker-domain>/badge/coverage.svg)](https://<worker-domain>/a/coverage)
```

- The Worker **MAY** color badges by thresholds (e.g., coverage ≥ 90% = `brightgreen`, ≥ 80% = `green`, ≥ 70% = `yellow`, else `red`).
- If a required artifact is missing or fails verification, the badge **MUST** render as `error` with color `red` (HTTP 409 recommended).
