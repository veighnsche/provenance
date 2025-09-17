use std::path::PathBuf;

#[test]
fn deterministic_output_for_index_and_pages() {
    // Arrange paths relative to repo root
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = crate_dir.parent().and_then(|p| p.parent()).unwrap().to_path_buf();

    // Two different temp output dirs
    let out1 = tempdir_path("prov-ssg-det-1");
    let out2 = tempdir_path("prov-ssg-det-2");

    // Build args
    let args = provenance_ssg::Args {
        root: repo_root.join("examples/minimal"),
        manifest: PathBuf::from(".provenance/manifest.json"),
        out: out1.clone(),
        copy_assets: true,
        verify_manifest: false,
        pubkey: None,
        schema_path: Some(repo_root.join("schemas/manifest.schema.json")),
        truncate_inline_bytes: 1_000_000,
    };

    // Generate first
    provenance_ssg::run_with_args(args.clone()).expect("first run ok");
    // Generate second
    let mut args2 = args.clone();
    args2.out = out2.clone();
    provenance_ssg::run_with_args(args2).expect("second run ok");

    // Compare important files
    assert_eq!(
        std::fs::read(out1.join("index.html")).unwrap(),
        std::fs::read(out2.join("index.html")).unwrap(),
        "index.html differs across runs"
    );
    assert_eq!(
        std::fs::read(out1.join("a").join("tests-summary").join("index.html")).unwrap(),
        std::fs::read(out2.join("a").join("tests-summary").join("index.html")).unwrap(),
        "tests-summary page differs across runs"
    );
}

fn tempdir_path(prefix: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    p.push(format!("{}-{}", prefix, nanos));
    p
}
