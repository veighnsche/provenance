# Repository Contract — Provenance (v1‑draft)

This document defines the repository‑level contract for powering a Provenance mirror: the entry point the service reads, required files and layout, naming, publishing, and validation rules.

Related specs: `./00_provenance.md`, `./01_proofdown.md`, `./02_worker.md`.

---

## 0. Terminology

- MUST/SHOULD/MAY follow RFC‑2119.
- Index: signed config enumerating front page (`front_page.markup`) and all renderable artifacts.
- Proofdown: constrained markup (`.pml`) used for front page and includes.
- Worker: Cloudflare Worker that verifies the Index and artifact digests and renders Proofdown.

---

## 1. Provenance Manifest (entry point)

- The canonical entry point is `.provenance/manifest.json` published at a commit‑pinned location reachable by the Worker via `RAW_BASE_URL`.
- The signature MUST be published alongside as `.provenance/manifest.json.sig` (out‑of‑band).
- The Worker is configured with:
  - `RAW_BASE_URL` — e.g., `https://raw.githubusercontent.com/<owner>/<repo>/<commit>/`
  - `INDEX_PATH` — default `.provenance/manifest.json`
  - `INDEX_SIG_PATH` — default `.provenance/manifest.json.sig`

Notes

- The Worker MUST verify `.provenance/manifest.json` with Ed25519 using its configured `INDEX_PUBKEY_ED25519` before parsing.
- The Worker MUST reject unknown or dynamically constructed base URLs; only descendants of `RAW_BASE_URL` are fetched.

---

## 2. Manifest Requirements

- Format: JSON (preferred) or TOML. JSON is normative for examples.
- Canonicalization (recommended for signing):
  - UTF‑8, `\n` newlines, no BOM.
  - Object keys sorted ascending (lexicographic) before signing.
  - No insignificant whitespace changes between signing and publish.
- Signature file `.provenance/manifest.json.sig` contains raw Ed25519 signature bytes encoded in Base64.
  - Optional: a `minisign` `.minisig` format MAY be supported behind a feature flag.
- Required fields (see `./00_provenance.md#2-index-file`):
  - `version`, `repo`, `commit`, `workflow_run`, `front_page.markup`, `front_page.title`, `artifacts[]`.
- Every artifact in `artifacts[]` MUST include `id`, `title`, `path`, `media_type`, `render`, `sha256`.

---

## 3. Repository Layout (recommended)

```
.
├── .provenance/
│   ├── manifest.json          # Entry point (signed)
│   └── manifest.json.sig      # Ed25519 signature (Base64)
└── ci/
    ├── front_page.pml     # Proofdown front page
    ├── tests/summary.json # summary:test input
    ├── tests/failures.md  # failing specs in Markdown
    ├── coverage/coverage.json
    ├── source/            # optional
    │   └── bundle.tar.gz  # optional: repo:bundle
    └── symbols/           # optional
        └── symbols.json   # optional: repo:symbols (ctags/LSIF)
```

- Paths in `manifest.json` are repo‑relative to `RAW_BASE_URL` (e.g., `ci/tests/summary.json`).
- `front_page.markup` MUST point to the Proofdown front page (e.g., `ci/front_page.pml`).

---

## 4. Naming and IDs

- Artifact `id` MUST match: `^[a-z0-9]([a-z0-9-]*[a-z0-9])?$` (kebab case).
- Reserved (SHOULD use):
  - `tests-summary` — test KPIs (render `summary:test`).
  - `coverage` — coverage table (render `table:coverage`).
  - `failures` — failing specs (render `markdown`).
- Viewer hints (minimum set): `markdown`, `json`, `table:coverage`, `summary:test`, `image`.
- Repo viewers (optional): `repo:file`, `repo:bundle`, `repo:symbols`.

---

## 5. Publishing and Access

- Public repos: publish to a commit‑pinned raw location under GitHub Raw or equivalent static hosting.
- Private repos: do not embed secrets in the Worker. Prefer a public `ci-snapshots` mirror or object storage where the Worker can read without credentials.
- Branches: you MAY publish to a dedicated `ci-snapshots` branch and configure the Worker with a static base (owner/repo/branch) if commit pinning is handled server‑side.

---

## 6. CI Responsibilities (repo contract)

- Produce all referenced artifacts.
- Compute SHA‑256 per artifact and embed in `.provenance/manifest.json`.
- Canonicalize `.provenance/manifest.json` and sign with Ed25519 (publish `.provenance/manifest.json.sig`).
- Fail CI if any required artifact is missing or checks fail.
- Optionally emit:
  - Source bundle (`render: repo:bundle`) for `repo:` snippets/trees.
  - Symbols (`render: repo:symbols`) for symbol links.

---

## 7. Example `manifest.json` (abbreviated)

```json
{
  "version": 1,
  "repo": "acme/provenance",
  "commit": "8c6a9f4e...",
  "workflow_run": { "id": 12345, "url": "https://github.com/.../runs/12345", "attempt": 1 },
  "front_page": { "title": "QA Evidence for {{ commit }}", "markup": "ci/front_page.pml" },
  "artifacts": [
    {
      "id": "tests-summary",
      "title": "Test Summary",
      "path": "ci/tests/summary.json",
      "media_type": "application/json",
      "render": "summary:test",
      "sha256": "..."
    },
    {
      "id": "coverage",
      "title": "Coverage",
      "path": "ci/coverage/coverage.json",
      "media_type": "application/json",
      "render": "table:coverage",
      "sha256": "..."
    },
    {
      "id": "failures",
      "title": "Failing Specs",
      "path": "ci/tests/failures.md",
      "media_type": "text/markdown",
      "render": "markdown",
      "sha256": "..."
    }
  ]
}
```

---

## 8. Validation (static)

- Schema validation: Manifest MUST validate against the provided JSON Schema (TBD path: `schemas/manifest.schema.json`).
- Lints:
  - Paths MUST be repo‑relative and normalized; disallow `..`.
  - `media_type` SHOULD match actual file type.
  - `render` MUST be in the allowed set (or feature‑flagged viewers).
  - `id`s MUST be unique.

---

## 9. Compatibility and Evolution

- Backward‑compatible additions to artifacts and viewers are allowed.
- Breaking changes are gated by `version` and require Worker support.

---

## 10. Migration: `index.json` → `.provenance/manifest.json`

Steps
1) Create a `.provenance/` directory at repo root.
2) Move or regenerate your manifest from `index.json` to `.provenance/manifest.json` (same JSON shape).
3) Produce the signature at `.provenance/manifest.json.sig` (Base64 Ed25519 over canonical bytes).
4) Update CI to publish the `.provenance/` directory alongside `ci/` artifacts.
5) Update Worker config defaults: `INDEX_PATH=.provenance/manifest.json`, `INDEX_SIG_PATH=.provenance/manifest.json.sig`.
6) Update docs/README badges and any links pointing to the old location.

Notes
- The JSON content is unchanged; only the path changes to avoid collisions with other tools using `index.json`.
- Keep branch or commit‑pinned publishing semantics identical to the previous setup.
