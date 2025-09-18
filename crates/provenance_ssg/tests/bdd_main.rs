// BDD entrypoint with cucumber macros and async tokio main (harness = false)
#[cfg(not(feature = "bdd"))]
fn main() {}

#[cfg(feature = "bdd")]
mod bdd_world;
#[cfg(feature = "bdd")]
mod steps;
#[cfg(feature = "bdd")]
use cucumber::World as _; // bring trait into scope for World::cucumber()

#[cfg(feature = "bdd")]
#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let crate_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = crate_dir.parent().and_then(|p| p.parent()).unwrap().to_path_buf();

    if let Ok(fp) = std::env::var("PROVENANCE_BDD_FEATURE_PATH") {
        let p = std::path::PathBuf::from(fp);
        let features = if p.is_absolute() { p } else { repo_root.join(p) };
        bdd_world::World::cucumber()
            .fail_on_skipped()
            .run_and_exit(features)
            .await;
    } else {
        let list = [
            repo_root.join("features/ssg.feature"),
            repo_root.join("features/ssg_render.feature"),
            repo_root.join("features/badges.feature"),
        ];
        for f in list {
            bdd_world::World::cucumber()
                .fail_on_skipped()
                .run(f)
                .await;
        }
    }
}
