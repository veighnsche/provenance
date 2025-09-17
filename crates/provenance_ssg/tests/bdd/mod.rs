mod steps_ssg;

use bdd_harness::Runner;

fn repo_root() -> std::path::PathBuf {
    // provenance_ssg/../.. is workspace root
    let crate_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    crate_dir.parent().and_then(|p| p.parent()).unwrap().to_path_buf()
}

#[test]
fn run_ssg_feature() {
    let mut r = Runner::new(repo_root());
    r.add_steps(steps_ssg::registry());
    r.run_feature_file("features/ssg.feature").expect("ssg.feature passes");
}

#[test]
fn run_badges_feature() {
    let mut r = Runner::new(repo_root());
    r.add_steps(steps_ssg::registry());
    r.run_feature_file("features/badges.feature").expect("badges.feature passes");
}
