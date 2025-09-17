#![cfg(not(feature = "bdd"))]
// Legacy bdd_harness-driven tests. Disabled when running with feature `bdd` (cucumber-based).
mod steps_manifest;
mod steps_signing;

use bdd_harness::{Runner, Step};

fn repo_root() -> std::path::PathBuf {
    // workspace root = manifest_contract/../.. from this file
    let crate_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    crate_dir.parent().and_then(|p| p.parent()).unwrap().to_path_buf()
}

#[test]
fn run_manifest_feature() {
    let mut r = Runner::new(repo_root());
    r.add_steps(steps_manifest::registry());
    r.run_feature_file("features/manifest.feature").expect("manifest.feature passes");
}

#[test]
fn run_signing_feature() {
    let mut r = Runner::new(repo_root());
    r.add_steps(steps_manifest::registry());
    r.add_steps(steps_signing::registry());
    r.run_feature_file("features/signing.feature").expect("signing.feature passes");
}
