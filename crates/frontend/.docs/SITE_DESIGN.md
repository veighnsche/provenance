# Provenance Site Design and Front-End Plan

This document captures all the missing pieces to turn the Provenance static site into a complete, polished product. It defines the architecture, crate layout, navigation and UX, Proofdown presentation, theming, CSS maintenance, deployment (Cloudflare), badges design, and testing/CI.

## Goals

- Deterministic, secure, accessible static site for CI evidence.
- Simple to generate locally and in CI.
- Clear navigation: overview KPIs, artifacts browsing, proofdown front page, badges.
- Front-end code organized for maintainability (RSX components) and testability.

---

## Architecture Overview

- Crates
  - `provenance_ssg` (existing): reads manifest, validates, builds artifact views, writes site.
  - `renderers` (existing): safe HTML renderers for markdown, JSON, coverage tables, test summaries.
  - `proofdown_parser`/`proofdown_ast` (existing, optional): parse Proofdown front page; feature `external_pml`.
  - `frontend` (NEW, proposed): RSX components (Dioxus) for page templates and layout, compiled to static HTML strings consumed by `provenance_ssg`.

- Data flow
  1. Load `manifest.json` (+ schema + semantics + optional signature).
  2. Build `ArtifactView` models (with verification and download hrefs).
  3. Render pages using RSX components (layout, index, artifact, list, proofdown widgets) + `renderers` for artifact bodies.
  4. Write static files into `site/` (plus `assets/`, `badge/`, `robots.txt`).

- Routing (static URLs)
  - `/index.html` – home page (KPIs, featured artifacts, optional Proofdown front page content).
  - `/a/<artifact-id>/index.html` – artifact detail pages.
  - `/artifacts/index.html` – all artifacts index with filters/search.
  - `/badge/*.json` and `/badge/*.svg` – badges.
  - `/assets/<artifact-id>/<filename>` – raw artifact downloads.
  - `/fragment/*` – small chunks if we split large pages (private, disallowed by robots).

- Determinism
  - Artifacts sorted by `id` at build time.
  - Coverage rows sorted by path in renderer.
  - Stable RSX component output ordering (inputs sorted/predictable).

---

## Front-End (RSX) Plan

- Create `crates/frontend/` (new crate) with Dioxus and server-side rendering to string.
  - Components:
    - `Layout` (skip link, header/top bar, main, footer)
    - `TopNav` (links + theme toggle)
    - `KpiCards`, `Card`, `Table`
    - `Breadcrumbs`
    - `ArtifactList` (search + filter + list)
    - `ArtifactPage` (header, badges, download link, body slot)
    - `Proofdown` widgets: `Grid`, `Card`, `ArtifactEmbed(kind="summary|table|json|markdown|image|link")`
  - Exported render functions:
    - `render_index(props) -> String`
    - `render_artifact(props) -> String`
    - `render_artifacts_index(props) -> String`
    - `render_proofdown(doc, context) -> String`
  - `provenance_ssg` will depend on `frontend` and call these functions to obtain HTML strings before writing to disk.

- Why a separate crate?
  - Clear separation of concerns; smaller `provenance_ssg`.
  - Enables unit testing RSX components.
  - Allows optional dev server in `frontend` for local design iteration (no SSG IO).

---

## Navigation & Site Map

- Top bar links
  - Home (`/`)
  - Artifacts (`/artifacts/`)
  - Proofdown (if enabled) (`/index.html#front-page` or dedicated `/front/` if we split)
  - Badges (`/badge/` index page with explanations)
  - Search (invokes client-side search modal)
  - Theme toggle (Light/Dark/System)
  - Commit info (short SHA, links to repo/commit)

- Breadcrumbs
  - On artifact pages: `Home / <Artifact Title>`

- Footer
  - Project links, schema/spec links, version, license, robots.txt link.

---

## Pages & UX

### Home Page

- Header: title (`front_page.title`) and commit.
- KPI cards: Tests total/passed/failed, Duration, Coverage.
- Featured artifacts grid (configurable by manifest or chosen heuristics).
- Proofdown section (if `external_pml`): rendered components (`grid`, `card`, `artifact.*`) with text using a CommonMark/GFM subset (including tables); raw HTML is not supported and is escaped as text.

### Artifacts Index (`/artifacts/`)

- List of all artifacts with search, filters, and sort.
  - Filters: by render kind (`summary:test`, `table:coverage`, `markdown`, `json`, `image`), verification status, media type.
  - Sort: by id (default), title, type.
- Each row/card shows id, title, render type, verification badge, link to detail.

### Artifact Detail (`/a/<id>/`)

- Header: title, id (subtext), breadcrumbs.
- Verification badge: `verified` or `digest mismatch`.
- Body: rendered by `renderers` according to type, with truncation notice for large files.
- Download link: `/assets/<artifact-id>/<filename>`.
- Optional metadata: media type, size (if available), sha256 (with copy button).

### Badges Overview (`/badge/`)

- Explains JSON and SVG badges.
- Shows current `provenance`, `tests`, `coverage` badges.
- Links to schema (`schemas/badge.schema.json`).

### Proofdown Presentation

- Markdown: CommonMark/GFM subset (including tables) for text; raw HTML is not supported and is escaped as text.
- Supported components (initial): `grid`, `card`, `artifact.summary`, `artifact.table`, `artifact.json`, `artifact.markdown`, `artifact.image`, `artifact.link`.
- Unknown components: render an error card with context (line/col) and a link to docs.
- Include depth and node limits: render an error banner when exceeded.
- Sanitization: any text/markdown from artifacts is sanitized; Proofdown renderer itself emits safe HTML.

---

## GitHub Mirror Views (Optional, v1.1+)

- Mirror pages per repository/commit/workflow run for multi-repo deployments.
  - `/gh/<org>/<repo>/<commit>/index.html` – overview of that commit’s evidence.
  - Aggregation pages by branch or workflow run id.
- These are aliases (symlinks or generated pages) mapping to the same artifact set; useful when a site aggregates many repos.
- Requires multi-manifest aggregation step (out of scope for v1 single-manifest SSG).

---

## Theming (Light/Dark/System)

- CSS variables with two theme palettes.
- Toggle in top bar.
- Persistence:
  - On toggle: store `localStorage.setItem('theme', 'light'|'dark'|'system')`.
  - On load: read localStorage; if `system`, use `prefers-color-scheme` media query.
  - Add `data-theme="light|dark"` on `<html>`; CSS variables switch by attribute.
- No cookies.

---

## CSS Maintenance

- Approach
  - v1: central CSS variables and small CSS in `Layout` (RSX) to keep SSG minimal.
  - v1.1: extract to a standalone CSS asset and include `<link rel="stylesheet">`.
- Tooling (no Node required):
  - Use `lightningcss` (Rust) in an `xtask` to minify and autoprefix.
  - Style linting with `stylelint` optional (can be run in CI if Node is available).
- Conventions
  - BEM-like utility classes for components: `.container, .cards, .card, .badge, .breadcrumb`.
  - Tokenized CSS variables: `--bg`, `--fg`, `--muted`, `--card-bg`, `--ok`, `--warn`, `--err`.
  - Accessible focus styles and minimum contrast.

---

## Accessibility

- Landmarks: skip link, `<main>`, `<nav aria-label="Breadcrumb">`.
- Headings: single `<h1>` per page; semantic nesting.
- Tables: `<th scope="col">` and `<caption>` where applicable.
- Images: `alt` text always present.
- Keyboard navigation and visible focus styles.
- Color contrast meets WCAG AA.

---

## Badges Design

- JSON (schema): `schemaVersion`, `label`, `message`, `color`.
- SVG: compact flat style (`flat`), accessible `aria-label`.
- Variants
  - `provenance`: `verified` or `unverified` (`red`) depending on signature + artifact verification.
  - `tests`: `X/Y passed` with color thresholds.
  - `coverage`: `NN.N%` with thresholds.
- Error state badges emitted when required artifacts are missing.

---

## Cloudflare Deployment Plan

- Platform: Cloudflare Pages (static hosting) for `site/` output.
- Optional Cloudflare Worker for:
  - Trailing slash normalization and 301 redirects.
  - Caching headers: `Cache-Control: public, max-age=600, stale-while-revalidate=86400` for HTML, longer for assets.
  - Security headers: CSP (script-src 'none'; frame-ancestors 'none'), HSTS, X-Content-Type-Options, Referrer-Policy.
- Optional bindings (only if needed):
  - `R2_BUCKET` – if hosting large artifacts outside Pages; SSG would write assets to R2 and emit signed links.
  - `KV_SEARCH` – if we want to serve a search index centrally (v2; v1 will embed static index JSON).
  - `CF_ANALYTICS_TOKEN` – Cloudflare Web Analytics.
- Purge strategy
  - On each deploy, purge by prefix `/` or use Pages’ atomic deploys (recommended; no purge needed as URLs are content-hashed for assets).

---

## Search

- v1: Client-side search using a small Lunr-like index (id, title, render, media_type) generated by SSG to `/search_index.json`.
- UI: keyboard `/` focuses search; results link to artifacts.

---

## Performance

- Minified CSS, no blocking scripts.
- Brotli/Gzip enabled via Cloudflare.
- Cache: long TTL for `/assets/` and `/badge/*.svg`; shorter TTL for HTML.
- Avoid large inline JSON; truncate with download link.

---

## Security

- Sanitize all rendered HTML (markdown/JSON display), no raw HTML injection.
- CSP with `script-src 'none'` by default.
- No external fonts; use system fonts for determinism.
- All links with `rel="noopener"` when `target="_blank"` (rare in static outputs).

---

## RSX Component Sketch (non-binding)

- `Layout { title, commit, children }`
- `TopNav { items: Vec<NavItem>, theme }`
- `KpiCards { tests, duration, coverage }`
- `ArtifactCard { id, title, verified, href }`
- `ArtifactList { filters, items }`
- `ArtifactPage { artifact, verified_badge, download_href, body_html }`
- `Breadcrumbs { trail }`
- `Proofdown::{Grid, Card, ArtifactEmbed}`

SSG calls `frontend::*::render_*()` and writes returned strings through `page_base()` wrapper (or moves that wrapper into the `Layout` component).

---

## Testing and CI

- Golden snapshot tests for HTML of index and one artifact page.
- Determinism checks (already present): build twice, compare bytes.
- Accessibility checks with `pa11y-ci` or axe (can run in CI on generated site).
- Lighthouse runs in CI for performance, a11y, SEO (informational gate).
- Schema tests for badges (present) and manifest (present).

---

## Implementation Plan (Phased)

1) Front-end crate

- Create `crates/frontend` with Dioxus, implement `Layout`, `TopNav`, `ArtifactPage`, and `KpiCards`.
- Integrate `provenance_ssg` to use `frontend` for page rendering.

2) Artifacts index + search

- Build `/artifacts/index.html` with filters and local search index.

3) Proofdown widgets

- Map AST to RSX components; error states for unknown components/limits.

4) Theming

- CSS variables + toggle + persistence.

5) Cloudflare

- Pages deploy config; optional Worker for headers and redirects.

6) Testing

- Golden snapshots, pa11y CI, Lighthouse CI.

---

## Open Questions

- Do we keep `page_base()` CSS inline or extract to an asset now?
- Should we aggregate multiple manifests (multi-repo) in v1 or defer to v2?
- Do we need internationalization in v1?

---

## Appendix: Top Bar Items (v1)

- Home
- Artifacts
- Proofdown (if enabled)
- Badges
- Search
- Theme
- Commit link
