use crate::bdd_world::World;
use anyhow::anyhow;
use cucumber::{given, then, when};
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};

fn repo_root(world: &World) -> PathBuf {
    world.repo_root()
}

fn temp_out() -> PathBuf {
    let mut p = std::env::temp_dir();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    p.push(format!("prov-bdd-out-{}", nanos));
    p
}

#[given(regex = r#"^a valid, signed manifest and verified artifacts for tests and coverage$"#)]
pub async fn given_valid_signed_manifest(world: &mut World) {
    let base = repo_root(world);
    let p = base.join("examples/minimal");
    for rel in [
        ".provenance/manifest.json",
        ".provenance/manifest.json.sig",
        ".provenance/public_test_ed25519.key.b64",
        "ci/tests/summary.json",
        "ci/coverage/coverage.json",
    ] {
        let fp = p.join(rel);
        assert!(fp.is_file(), "missing required file: {}", fp.display());
    }
    world.set("repo_root", p.to_string_lossy());
    world.set("manifest_path", ".provenance/manifest.json");
}

#[given(regex = r#"^the repo root is \"([^\"]+)\"$"#)]
pub async fn given_repo_root(world: &mut World, rel: String) {
    let p = if Path::new(&rel).is_absolute() { PathBuf::from(rel) } else { repo_root(world).join(rel) };
    world.set("repo_root", p.to_string_lossy());
}

#[given(regex = r#"^the manifest path is \"([^\"]+)\"$"#)]
pub async fn given_manifest_path(world: &mut World, rel: String) {
    world.set("manifest_path", rel);
}

#[given(regex = r#"^the JSON schema path is \"([^\"]+)\"$"#)]
pub async fn given_schema_path(world: &mut World, rel: String) {
    world.set("schema_path", rel);
}

#[given(regex = r#"^file \"([^\"]+)\" exists$"#)]
pub async fn given_file_exists_under_repo(world: &mut World, rel: String) {
    let p = repo_root(world).join(rel);
    assert!(p.is_file(), "missing file: {}", p.display());
}

#[when(regex = r#"^I run the SSG with output dir \"([^\"]+)\"$"#)]
pub async fn when_run_ssg_out(world: &mut World, out_label: String) {
    run_ssg(world, &out_label, None, false);
}

#[when(regex = r#"^I run the SSG with output dir \"([^\"]+)\" and truncate inline bytes to (\d+)$"#)]
pub async fn when_run_ssg_out_trunc(world: &mut World, out_label: String, limit: u64) {
    run_ssg(world, &out_label, Some(limit as usize), false);
}

#[when(regex = r#"^I generate badges$"#)]
pub async fn when_generate_badges(world: &mut World) {
    world.set("verify_manifest", "true");
    let pk_path = repo_root(world).join("examples/minimal/.provenance/public_test_ed25519.key.b64");
    let pk_b64 = fs::read_to_string(&pk_path).expect("read pubkey");
    world.set("pubkey_b64", pk_b64.trim());
    run_ssg(world, "site", None, true);
}

fn run_ssg(world: &mut World, out_label: &str, limit: Option<usize>, verify: bool) {
    let out = if out_label == "TMP" { temp_out() } else {
        let p = PathBuf::from(out_label);
        if p.is_absolute() { p } else { repo_root(world).join(p) }
    };
    let root = repo_root(world);
    let manifest = PathBuf::from(world.get("manifest_path").unwrap_or(".provenance/manifest.json".into()));
    let manifest = if manifest.is_absolute() { manifest } else { root.join(manifest) };
    let schema = world.get("schema_path").map(PathBuf::from).unwrap_or_else(|| repo_root(world).join("schemas/manifest.schema.json"));

    let args = provenance_ssg::Args {
        root: root,
        manifest: manifest.strip_prefix(&repo_root(world)).unwrap_or(&manifest).to_path_buf(),
        out: out.clone(),
        copy_assets: true,
        verify_manifest: verify || world.get("verify_manifest").unwrap_or_default() == "true",
        pubkey: world.get("pubkey_b64"),
        schema_path: Some(schema),
        truncate_inline_bytes: limit.unwrap_or(1_000_000),
    };
    provenance_ssg::run_with_args(args).expect("ssg run ok");
    world.set("out_dir", out.to_string_lossy());
}

#[then(regex = r#"^file \"([^\"]+)\" should exist$"#)]
pub async fn then_file_exists_under_out(world: &mut World, rel: String) {
    let out = PathBuf::from(world.get("out_dir").expect("out_dir not set"));
    let p = out.join(rel);
    assert!(p.is_file(), "missing file: {}", p.display());
}

#[then(regex = r#"^HTML at \"([^\"]+)\" should contain \"([^\"]+)\"$"#)]
pub async fn then_html_contains_under_out(world: &mut World, rel: String, needle: String) {
    let out = PathBuf::from(world.get("out_dir").expect("out_dir not set"));
    let p = out.join(rel);
    let txt = fs::read_to_string(&p).expect("read HTML");
    assert!(txt.contains(&needle), "HTML did not contain '{}': {}", needle, p.display());
}

#[then(regex = r#"^the JSON has schemaVersion (\d+)$"#)]
pub async fn then_badge_schema_version(world: &mut World, want: u64) {
    let out = PathBuf::from(world.get("out_dir").expect("out_dir not set"));
    let p = out.join("badge/provenance.json");
    let v: serde_json::Value = serde_json::from_str(&fs::read_to_string(&p).expect("read json")).expect("parse json");
    let got = v.get("schemaVersion").and_then(|v| v.as_u64()).unwrap_or(0);
    assert_eq!(got, want, "schemaVersion {}, want {} in {}", got, want, p.display());
}

#[then(regex = r#"^the JSON message contains \"([^\"]+)\"$"#)]
pub async fn then_badge_message_contains(world: &mut World, needle: String) {
    let out = PathBuf::from(world.get("out_dir").expect("out_dir not set"));
    for name in ["provenance", "tests", "coverage"] {
        let p = out.join(format!("badge/{}.json", name));
        if p.is_file() {
            let v: serde_json::Value = serde_json::from_str(&fs::read_to_string(&p).expect("read badge")).expect("parse badge");
            if let Some(msg) = v.get("message").and_then(|v| v.as_str()) {
                if msg.contains(&needle) { return; }
            }
        }
    }
    panic!("no badge message contained '{}'", needle);
}

#[then(regex = r#"^the JSON message matches \"([^\"]+)\"$"#)]
pub async fn then_badge_message_matches(world: &mut World, patt: String) {
    let re = Regex::new(&patt).expect("regex");
    let out = PathBuf::from(world.get("out_dir").expect("out_dir not set"));
    let p = out.join("badge/coverage.json");
    let v: serde_json::Value = serde_json::from_str(&fs::read_to_string(&p).expect("read json")).expect("parse json");
    let msg = v.get("message").and_then(|v| v.as_str()).unwrap_or("");
    assert!(re.is_match(msg), "badge message '{}' did not match {}", msg, patt);
}

#[given(regex = r#"^the coverage artifact is missing$"#)]
pub async fn given_coverage_artifact_missing(world: &mut World) {
    let src = repo_root(world).join("examples/minimal");
    assert!(src.is_dir(), "source example not found: {}", src.display());
    let dst = temp_out();
    copy_tree(&src, &dst).expect("copy tree");
    let cov = dst.join("ci/coverage/coverage.json");
    if cov.exists() { std::fs::remove_file(&cov).expect("rm coverage.json"); }
    world.set("repo_root", dst.to_string_lossy());
    world.set("manifest_path", ".provenance/manifest.json");
}

#[then(regex = r#"^the JSON badge for coverage has message \"([^\"]+)\"$"#)]
pub async fn then_coverage_badge_message_is(world: &mut World, want: String) {
    let out = PathBuf::from(world.get("out_dir").expect("out_dir not set"));
    let p = out.join("badge/coverage.json");
    let v: serde_json::Value = serde_json::from_str(&fs::read_to_string(&p).expect("read json")).expect("parse json");
    let msg = v.get("message").and_then(|v| v.as_str()).unwrap_or("");
    assert_eq!(msg, want, "coverage badge message '{}' != '{}' in {}", msg, want, p.display());
}

#[then(regex = r#"^uses color \"([^\"]+)\"$"#)]
pub async fn then_coverage_badge_color_is(world: &mut World, want: String) {
    let out = PathBuf::from(world.get("out_dir").expect("out_dir not set"));
    let p = out.join("badge/coverage.json");
    let v: serde_json::Value = serde_json::from_str(&fs::read_to_string(&p).expect("read json")).expect("parse json");
    let color = v.get("color").and_then(|v| v.as_str()).unwrap_or("");
    assert_eq!(color, want, "coverage badge color '{}' != '{}' in {}", color, want, p.display());
}

fn copy_tree(src: &Path, dst: &Path) -> Result<(), anyhow::Error> {
    for entry in walkdir::WalkDir::new(src) {
        let ent = entry.map_err(|e| anyhow!("walk {}: {}", src.display(), e))?;
        let rel = ent.path().strip_prefix(src).unwrap();
        let out_path = dst.join(rel);
        if ent.file_type().is_dir() {
            std::fs::create_dir_all(&out_path)?;
        } else if ent.file_type().is_file() {
            if let Some(parent) = out_path.parent() { std::fs::create_dir_all(parent)?; }
            std::fs::copy(ent.path(), &out_path).map_err(|e| anyhow!("copy {} -> {}: {}", ent.path().display(), out_path.display(), e))?;
        }
    }
    Ok(())
}
