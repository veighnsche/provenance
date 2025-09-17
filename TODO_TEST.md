# TODO_TEST — BDD (Gherkin) Plan and Step Map

Status: In Progress v0.2 (cucumber macros wired across crates). This document enumerates granular Gherkin features, scenarios, steps, and their Rust step-function mappings for everything we own in this repo.

Guiding rules
- Deterministic: no network, no time-based behavior, stable ordering and formats.
- Repo-root paths in tests use `CARGO_MANIFEST_DIR` to resolve `examples/minimal` and `schemas/`.
- Favor golden fixtures (manifest JSON, AST JSON, HTML) to assert byte-for-byte where practical.
- Keep the Proofdown parser integration feature-gated (`external_pml`). We test integration, not the parser internals.

---

## Directory layout (proposed)

- `features/` (already present)
  - `manifest.feature`
  - `signing.feature`
  - `proofdown_front_page.feature`
  - `ssg.feature` (add)
  - `renderers.feature` (add)
  - `badges.feature`
  - `tools.feature` (add)
- `crates/*/tests/bdd/` (step definitions grouped by concern)
  - `crates/manifest_contract/tests/bdd/steps_manifest.rs`
  - `crates/manifest_contract/tests/bdd/steps_signing.rs`
  - `crates/provenance_ssg/tests/bdd/steps_ssg.rs`
  - `crates/renderers/tests/bdd/steps_renderers.rs`
  - `crates/badges/tests/bdd/steps_badges.rs`
  - `crates/tools/tests/bdd/steps_tools.rs`
  - `crates/*/tests/bdd/mod.rs` to register steps
- Per-crate Cucumber harness (in place)
  - Each crate defines `tests/bdd_main.rs` with an async tokio main using cucumber macros and a local `World` (see `crates/*/tests/bdd_world/mod.rs`).
  - Legacy `bdd_harness` remains only for `manifest_contract` behind `#[cfg(not(feature = "bdd"))]` and is not used when running with `--features bdd`.

---

## Common step vocabulary

- Given
  - `Given the repo root is "{path}"` → `common::given_repo_root(Path)`
  - `And the manifest path is "{path}"` → `common::given_manifest_path(Path)`
  - `And the JSON schema path is "{path}"` → `common::given_schema_path(Path)`
  - `And the public key is read from "{path}"` → `common::given_pubkey_b64(String)`
- When
  - `When I load the manifest` → `manifest::when_load_manifest()`
  - `And I validate it against the schema` → `manifest::when_validate_schema()`
  - `And I canonicalize the manifest` → `manifest::when_canonicalize()`
  - `And I verify the signature` → `signing::when_verify_signature()`
  - `When I run the SSG with output dir "{path}" [and features {flags}]` → `ssg::when_run_ssg(out, opts)`
  - `When I render "markdown|json|table:coverage|summary:test|image" from "{path}"` → `renderers::when_render(kind, path)`
  - `When I generate a large JSON file of ~{n} MB at "{path}"` → `tools::when_gen_large_json(n, path)`
- Then
  - `Then validation should pass` / `fail with message containing "{needle}"` → `manifest::then_validation_result(expect)`
  - `Then canonical bytes should be stable across runs` → `manifest::then_canonical_stable()`
  - `Then signature verification should pass` / `fail` → `signing::then_verify_result(expect)`
  - `Then file "{path}" should exist` → `common::then_file_exists(path)`
  - `Then HTML at "{path}" should contain "{needle}"` → `common::then_html_contains(path, needle)`
  - `Then badge JSON at "{path}" has fields (label:"{l}", message:"{m}", color:"{c}")` → `badges::then_badge_json_fields(...)`

---

## Feature: manifest.feature (we own)

Scenarios
- Valid manifest passes schema and semantics
  - Given repo root → examples/minimal
  - Given manifest path `.provenance/manifest.json`
  - Given schema path `schemas/manifest.schema.json`
  - When I load the manifest
  - And I validate it against the schema
  - Then validation should pass
- Missing `artifacts` fails schema
  - Use inline JSON docstring or temp file with missing field
  - Expect failure message contains `schema validation failed`
- Duplicate artifact id fails semantics
  - Edit a cloned manifest in tmp and duplicate `tests-summary`
  - Expect failure `duplicate artifact id`
- Path escapes root fails semantics
  - Set `path` to `../etc/passwd`
  - Expect failure `escapes root`
- Unknown render value fails semantics
  - Set `render` to `bogus`
  - Expect failure `unknown render`
- Invalid sha256 length fails semantics
  - Set `sha256` to short hex
  - Expect failure `invalid sha256`

Step functions
- `steps_manifest.rs`
  - `when_load_manifest()` → wraps `manifest_contract::load_manifest`
  - `when_validate_schema()` → wraps `manifest_contract::validate_schema`
  - `then_validation_result(expect)` → asserts Ok/Err + substring on error
  - `when_canonicalize()` / `then_canonical_stable()` → wraps `manifest_contract::canonicalize`

---

## Feature: signing.feature (we own)

Scenarios
- Canonicalization stable and signature verifies
  - Given manifest & pubkey b64 (from examples)
  - When I canonicalize and verify signature (use `.sig`)
  - Then verification should pass
- Mutated manifest fails verification
  - Change one byte in a temp copy
  - Expect verification fails
- Wrong public key fails verification
  - Use a random key from tools `GenTestKey`
  - Expect verification fails

Step functions
- `steps_signing.rs`
  - `when_verify_signature()` → wraps `ed25519_verify`
  - Helpers to read `.sig` and pubkey files

---

## Feature: ssg.feature (we own)

Scenarios
- Generate minimal site (feature off)
  - Given repo root examples/minimal
  - When I run the SSG with output dir TMP
  - Then `index.html` exists
  - And `a/tests-summary/index.html` exists
- Truncation for large files
  - Given large JSON at `examples/minimal/ci/tests/large.json`
  - When I run the SSG with `--truncate-inline-bytes 1`
  - Then `a/failures/index.html` contains `Truncated`
- Badges emitted
  - After site generation, badge JSON and SVG exist for `provenance`, `tests`, `coverage`
  - And tests badge values match summary

Step functions
- `steps_ssg.rs`
  - `when_run_ssg(out, opts)` → uses `provenance_ssg::run_with_args`
  - `then_file_exists` & `then_html_contains` from `common`

---

## Feature: renderers.feature (we own)

Scenarios
- Markdown renders headings and escapes ampersand
  - Render `ci/tests/failures.md` → contains `<h1>` and `&amp;`
- JSON pretty view escapes `<>&`
  - Render `ci/tests/summary.json` with kind `json` → contains `&lt;&gt;&amp;`
- Coverage table contains rows and total
  - Render `ci/coverage/coverage.json` with kind `table:coverage` → contains `Total` and file rows
- Image renderer produces `<img>` tag
  - `renderers::render_image` with a path and alt text → contains proper attrs

Step functions
- `steps_renderers.rs` call into `renderers::{render_markdown, render_json_pretty, render_coverage, render_image}`

---

## Feature: badges.feature (we own)

Scenarios
- Provenance badge toggles by verification
  - With all artifacts verified → label `provenance`, message `verified`, color `brightgreen`
  - With mismatch → message `unverified`, color `red`
- Tests badge reflects totals
  - For summary `{total:10, passed:9, failed:1}` → message `9/10 pass`, color `orange`
- Coverage badge reflects thresholds
  - At 92.0% → `brightgreen`; at 0% → `lightgrey`

Step functions
- `steps_badges.rs` calls into `badges::{badge_provenance, badge_tests, badge_coverage}` and inspects JSON

---

## Feature: tools.feature (we own)

Scenarios
- UpdateSha updates manifest `sha256` fields
  - Create temp copies of small fixture files and manifest
  - Run CLI `tools UpdateSha --root ... --manifest ...`
  - Assert manifest changes match computed digests
- Sign produces `.sig`
  - Use a test private key and `Sign` subcommand to produce signature file
  - Assert `.sig` exists and signature verifies
- GenLargeJson produces target size
  - Run `GenLargeJson --out ... --size-mb 6`
  - Assert file size ≥ requested size and JSON parses

Step functions
- `steps_tools.rs` invoke the `tools` binary (spawn) or factor helpers to call functions directly

---

## Skeletons (illustrative)

Example: step registration (inline test module)
```rust
// crates/manifest_contract/tests/bdd/mod.rs
mod steps_manifest;
mod steps_signing;

#[test]
fn run_manifest_feature() {
    bdd::run_feature_file("features/manifest.feature", &[
        steps_manifest::registry(),
        steps_signing::registry(),
    ]);
}
```

Example: step signature (one of many)
```rust
// crates/provenance_ssg/tests/bdd/steps_ssg.rs
pub fn registry() -> Vec<bdd::Step> { vec![
    bdd::step!(Given r#"the repo root is "(.+)""#, |ctx, caps| {
        ctx.repo_root = Some(resolve_repo_root(&caps[1]));
    }),
    bdd::step!(When r#"I run the SSG with output dir "(.+)""#, |ctx, caps| {
        let out = temp_out(&caps[1]);
        let args = build_args(ctx, &out);
        provenance_ssg::run_with_args(args).expect("ssg ok");
        ctx.last_out = Some(out);
    }),
]}
```

Note: The actual harness (`bdd::...`) can be a tiny utility we write in Week 4, but we can start implementing step functions as plain test helpers now and bind them to a parser later.

---

## Status summary

- Implemented via cucumber macros (per-crate): `manifest.feature`, `signing.feature`, `renderers.feature`, `ssg.feature`, `badges.feature`.
- Pending: `tools.feature` and Proofdown front page integration scenarios (feature-gated via `external_pml`).

## Phased implementation plan

- Week 1: Complete — manifest and signing steps implemented; common world/context utilities landed.
- Week 2: Mostly complete — renderers and SSG steps implemented incl. truncation and badges; golden HTML snapshots TBD for determinism.
- Week 3: In progress — badges scenarios implemented; tools CLI steps pending; add negative scenarios as needed.
- Week 4: Harness in place via cucumber; CI gates run BDD through `cargo xtask test-all`.

---

## Traceability map

- `features/manifest.feature` ↔ `crates/manifest_contract/tests/bdd/steps_manifest.rs`
- `features/signing.feature` ↔ `crates/manifest_contract/tests/bdd/steps_signing.rs`
- `features/ssg.feature` ↔ `crates/provenance_ssg/tests/bdd/steps_ssg.rs`
- `features/renderers.feature` ↔ `crates/renderers/tests/bdd/steps_renderers.rs`
- `features/badges.feature` ↔ `crates/badges/tests/bdd/steps_badges.rs`
- `features/tools.feature` ↔ `crates/tools/tests/bdd/steps_tools.rs`

---

## Notes

- Proofdown parser is externally maintained. For integration scenarios with `external_pml`, we only assert the SSG’s behavior given a parsed AST or through the feature-gated path, not parser internals.
- All file writes in tests go to temp dirs; source fixtures live under `examples/minimal/`.
