# Frontend (RSX) for Provenance SSG

This crate provides reusable RSX (Dioxus) components and server-side rendering helpers for the Provenance static site generator.

## What it does

- Renders the primary page sections as HTML strings:
  - Home/Index (KPIs + featured artifacts)
  - Artifact detail pages
  - Artifacts index (table)
- Produces accessible, deterministic markup ready to be wrapped by the SSG layout.

## Components (initial)

- `render_index(title, commit, kpis, featured)`
- `render_artifact(artifact, body_html)`
- `render_artifacts_index(items)`

A higher-level Layout (top bar, theme toggle, skip link, container) is currently provided by the SSG `page_base()` wrapper. In a future refactor we can migrate that into a `Layout` component here.

## Development

- This is a pure library crate targeting server-side rendering to string.
- Run workspace tests to validate end-to-end integration:

```bash
cargo test --all-features
```

## Roadmap

- Extract the layout into RSX and minimize inline CSS in SSG.
- Add a small search UI consuming `/search_index.json`.
- Expand Proofdown widgets as components.
