# Frontend TODO

This list tracks the immediate integration tasks for the new `frontend` (RSX) crate. These should be completed right away.

- [x] Scaffold crate with initial RSX helpers: `render_index`, `render_artifact`, `render_artifacts_index`.
- [x] Add README with scope and roadmap.
- [ ] Wire `provenance_ssg` to use `frontend::render_index` for the home page (non-Proofdown path) and wrap with `page_base`.
- [ ] Wire `provenance_ssg` to use `frontend::render_artifact` for artifact pages and wrap with `page_base`.
- [ ] Generate `/artifacts/index.html` using `frontend::render_artifacts_index`.
- [ ] Generate `/search_index.json` for client-side search: id, title, render, media_type, verified.
- [ ] Build, run tests, and ensure no regressions.
