# Provenance BDD Wiring Migration (Full Alignment)

Goal: migrate all BDD tests to cucumber-rs macros with an async tokio main and a shared World per `.docs/BDD_WIRING.md`. This replaces the lightweight `bdd_harness` runner.

Conventions:
- Macro imports: `use cucumber::{given, when, then};`
- World: `#[derive(Default, cucumber::World)] pub struct World { ... }`
- Entrypoint: `World::cucumber().fail_on_skipped().run_and_exit(<features_path>).await;`
- Features root: repository `features/` (unless `PROVENANCE_BDD_FEATURE_PATH` is set; relative paths are resolved against repo root).

Tasks

1) manifest_contract
- [x] Add `tests/world/mod.rs` defining `World` (string map + json map) and helpers: `repo_root(&World) -> PathBuf`.
- [x] Add `tests/steps/mod.rs` re-exporting step modules.
- [x] Port steps into macro form:
  - [x] `tests/steps/manifest.rs` (schema validation, semantic checks, canonicalization, uniqueness errors).
  - [x] `tests/steps/signing.rs` (canonicalize, signature verification, mutate-one-char, recanonicalize+verify).
- [x] Replace `tests/bdd_main.rs` with tokio async main that wires modules and resolves features path.
- [x] Add dev-dependencies: `cucumber = { version = "0.20", features = ["macros"] }`, `tokio = { version = "1", features = ["macros", "rt-multi-thread"] }`.
- [x] Run: `cargo test -p manifest_contract --features bdd --test bdd -- --nocapture`.

2) renderers
- [x] Add `tests/world/mod.rs` with the same `World` model.
- [x] Add `tests/steps/mod.rs` and port `renderers` steps to macros: markdown/json/table:coverage/summary:test/image; store HTML in world.
- [x] Replace `tests/bdd_main.rs` with tokio async main.
- [x] Add dev-dependencies as above.
- [x] Run: `cargo test -p renderers --features bdd --test bdd -- --nocapture`.

3) provenance_ssg
- [x] Add `tests/world/mod.rs` as above.
- [x] Add `tests/steps/mod.rs` and port `ssg` steps: repo root, manifest/schema paths, run SSG (with/without truncation), badges, missing coverage (copy to temp and remove), HTML assertions.
- [x] Replace `tests/bdd_main.rs` with tokio async main.
- [x] Add dev-dependencies as above.
- [x] Run: `cargo test -p provenance_ssg --features bdd --test bdd -- --nocapture`.

Notes
- Do not modify `crates/proofdown_parser` per repository guidance.
- Preserve semantics of existing step logic; only migrate wiring to macros/World.
- Use `base64::Engine` where calling `STANDARD.encode()/decode()`.
- Keep features hermetic and repo-relative.

Completion Criteria
- All three crates compile and pass their BDD tests using cucumber macros.
- `PROVENANCE_BDD_FEATURE_PATH` can target an individual feature file.
- No usage of `bdd_harness` remains in test binaries (it may remain in dev-deps temporarily).  
  Note: Legacy `bdd_harness`-based tests remain behind `#[cfg(not(feature = "bdd"))]` for `manifest_contract`. Consider removing in a follow-up.

Status: All target crates are wired to cucumber macros with async tokio mains and per-crate `World` contexts. Remaining optional cleanup is to remove legacy `bdd_harness` tests.
