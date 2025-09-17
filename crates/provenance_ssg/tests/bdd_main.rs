// BDD entrypoint aligned with .docs/BDD_WIRING.md (harness = false)
use bdd_harness::Runner;

#[path = "bdd/steps_ssg.rs"] mod steps_ssg;

fn repo_root() -> std::path::PathBuf {
    // provenance_ssg/../.. is the workspace root
    let crate_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    crate_dir.parent().and_then(|p| p.parent()).unwrap().to_path_buf()
}

fn main() {
    let repo = repo_root();
    let mut r = Runner::new(&repo);
    r.add_steps(steps_ssg::registry());

    if let Ok(fp) = std::env::var("PROVENANCE_BDD_FEATURE_PATH") {
        r.run_feature_file(&fp).expect("feature run ok");
    } else {
        r.run_feature_file("features/ssg.feature").expect("ssg.feature passes");
        r.run_feature_file("features/badges.feature").expect("badges.feature passes");
    }
}
