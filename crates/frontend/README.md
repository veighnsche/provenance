# Frontend (RSX) for Provenance SSG

This crate provides reusable RSX (Dioxus) components and server-side rendering helpers for the Provenance static site generator.

> IMPORTANT: External Submodule Ownership — `crates/proofdown_parser`
>
> The Proofdown parser lives in a nested workspace maintained externally. Do NOT modify parser code here.
> Integrate via the feature-gated optional dependency `external_pml` in `provenance_ssg` and propose parser
> changes upstream in the submodule repo.

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

## References

- Specs and contracts live under `/.specs/` at the repo root. Start with:
  - `/.specs/00_provenance.md` (overall contract)
  - `/.specs/18_workspace.md` (workspace wiring and feature gates)
  - `/.specs/40_accessibility.md` (accessibility expectations)


## Roadmap

- Extract the layout into RSX and minimize inline CSS in SSG.
- Add a small search UI consuming `/search_index.json`.
- Expand Proofdown widgets as components.
