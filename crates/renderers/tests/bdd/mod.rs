mod steps_renderers;

use bdd_harness::Runner;

fn repo_root() -> std::path::PathBuf {
    // renderers/../.. is workspace root
    let crate_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    crate_dir.parent().and_then(|p| p.parent()).unwrap().to_path_buf()
}

#[test]
fn run_renderers_feature() {
    let mut r = Runner::new(repo_root());
    r.add_steps(steps_renderers::registry());
    r.run_feature_file("features/renderers.feature").expect("renderers.feature passes");
}
