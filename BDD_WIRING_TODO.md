# Provenance BDD Wiring Migration (Full Alignment)

Goal: migrate all BDD tests to cucumber-rs macros with an async tokio main and a shared World per `.docs/BDD_WIRING.md`. This replaces the lightweight `bdd_harness` runner.

Conventions:
- Macro imports: `use cucumber::{given, when, then};`
- World: `#[derive(Default, cucumber::World)] pub struct World { ... }`
- Entrypoint: `World::cucumber().fail_on_skipped().run_and_exit(<features_path>).await;`
- Features root: repository `features/` (unless `PROVENANCE_BDD_FEATURE_PATH` is set; relative paths are resolved against repo root).

Tasks

1) manifest_contract
- [ ] Add `tests/world/mod.rs` defining `World` (string map + json map) and helpers: `repo_root(&World) -> PathBuf`.
- [ ] Add `tests/steps/mod.rs` re-exporting step modules.
- [ ] Port steps into macro form:
  - [ ] `tests/steps/manifest.rs` (schema validation, semantic checks, canonicalization, uniqueness errors).
  - [ ] `tests/steps/signing.rs` (canonicalize, signature verification, mutate-one-char, recanonicalize+verify).
- [ ] Replace `tests/bdd_main.rs` with tokio async main that wires modules and resolves features path.
- [ ] Add dev-dependencies: `cucumber = { version = "0.20", features = ["macros"] }`, `tokio = { version = "1", features = ["macros", "rt-multi-thread"] }`.
- [ ] Run: `cargo test -p manifest_contract --features bdd --test bdd -- --nocapture`.

2) renderers
- [ ] Add `tests/world/mod.rs` with the same `World` model.
- [ ] Add `tests/steps/mod.rs` and port `renderers` steps to macros: markdown/json/table:coverage/summary:test/image; store HTML in world.
- [ ] Replace `tests/bdd_main.rs` with tokio async main.
- [ ] Add dev-dependencies as above.
- [ ] Run: `cargo test -p renderers --features bdd --test bdd -- --nocapture`.

3) provenance_ssg
- [ ] Add `tests/world/mod.rs` as above.
- [ ] Add `tests/steps/mod.rs` and port `ssg` steps: repo root, manifest/schema paths, run SSG (with/without truncation), badges, missing coverage (copy to temp and remove), HTML assertions.
- [ ] Replace `tests/bdd_main.rs` with tokio async main.
- [ ] Add dev-dependencies as above.
- [ ] Run: `cargo test -p provenance_ssg --features bdd --test bdd -- --nocapture`.

Notes
- Do not modify `crates/proofdown_parser` per repository guidance.
- Preserve semantics of existing step logic; only migrate wiring to macros/World.
- Use `base64::Engine` where calling `STANDARD.encode()/decode()`.
- Keep features hermetic and repo-relative.

Completion Criteria
- All three crates compile and pass their BDD tests using cucumber macros.
- `PROVENANCE_BDD_FEATURE_PATH` can target an individual feature file.
- No usage of `bdd_harness` remains in test binaries (it may remain in dev-deps temporarily).
