// BDD entrypoint aligned with .docs/BDD_WIRING.md (harness = false)
use bdd_harness::Runner;

#[path = "bdd/steps_manifest.rs"] mod steps_manifest;
#[path = "bdd/steps_signing.rs"] mod steps_signing;

fn repo_root() -> std::path::PathBuf {
    // manifest_contract/../.. is the workspace root
    let crate_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    crate_dir.parent().and_then(|p| p.parent()).unwrap().to_path_buf()
}

fn main() {
    let repo = repo_root();
    let mut r = Runner::new(&repo);
    r.add_steps(steps_manifest::registry());
    r.add_steps(steps_signing::registry());

    // Allow targeting a single feature path via env var, else run both defaults
    if let Ok(fp) = std::env::var("PROVENANCE_BDD_FEATURE_PATH") {
        // Absolute or repo-root-relative
        r.run_feature_file(&fp).expect("feature run ok");
    } else {
        r.run_feature_file("features/manifest.feature").expect("manifest.feature passes");
        r.run_feature_file("features/signing.feature").expect("signing.feature passes");
    }
}
