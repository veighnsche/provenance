/// THIS FILE IS GETTING TOO LARGE
/// PLEASE MODULARIZE AT YOUR NEXT REFACTORING

use crate::bdd_world::World;
use anyhow::anyhow;
use cucumber::{given, then, when};
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};

fn repo_root(world: &World) -> PathBuf {
    world.repo_root()
}

#[given(regex = r#"^an artifact with an incorrect sha256 in the manifest$"#)]
pub async fn given_artifact_with_incorrect_sha(world: &mut World) {
    // Read the current manifest, replace first artifact sha256 with a wrong value, write to temp, and point manifest_path to it
    let root = repo_root(world);
    let manifest_path = PathBuf::from(world.get("manifest_path").unwrap_or(".provenance/manifest.json".into()));
    let mpath = if manifest_path.is_absolute() { manifest_path } else { root.join(manifest_path) };
    let txt = fs::read_to_string(&mpath).expect("read manifest");
    let mut val: serde_json::Value = serde_json::from_str(&txt).expect("parse manifest");
    if let Some(arr) = val.get_mut("artifacts").and_then(|v| v.as_array_mut()) {
        if let Some(first) = arr.get_mut(0) {
            if let Some(obj) = first.as_object_mut() {
                let bad = "0".repeat(64);
                obj.insert("sha256".into(), serde_json::Value::String(bad));
            }
        }
    }
    let tmp = {
        let mut p = std::env::temp_dir();
        let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        p.push(format!("manifest-badsha-{}.json", nanos));
        p
    };
    fs::write(&tmp, serde_json::to_string_pretty(&val).unwrap() + "\n").expect("write temp manifest");
    world.set("manifest_path", tmp.to_string_lossy());
}

fn workspace_root() -> PathBuf {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    crate_dir.parent().and_then(|p| p.parent()).unwrap().to_path_buf()
}

// Additional steps for ssg_render.feature

#[given(regex = r#"^a repository with examples at \"([^\"]+)\"$"#)]
pub async fn given_repository_with_examples(world: &mut World, rel: String) {
    // Set base_dir to the repo root; actual --root will be computed from this + rel
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let ws_root = crate_dir.parent().and_then(|p| p.parent()).unwrap().to_path_buf();
    world.set("base_dir", ws_root.to_string_lossy());
    // Also set repo_root to the examples dir and manifest path default under this dir
    let root = repo_root(world).join(rel.trim_end_matches('/'));
    world.set("repo_root", root.to_string_lossy());
    let manifest = root.join(".provenance/manifest.json");
    world.set("manifest_path", manifest.to_string_lossy());
}

#[given(regex = r#"^a manifest at \"([^\"]+)\"$"#)]
pub async fn given_a_manifest_at(world: &mut World, rel: String) {
    // If a repo-relative path with leading examples/... is provided, absolutize it from workspace root
    let p = PathBuf::from(&rel);
    if p.is_absolute() || (!rel.starts_with("examples/") && !rel.starts_with("./examples/")) {
        world.set("manifest_path", rel);
    } else {
        let abs = workspace_root().join(p);
        world.set("manifest_path", abs.to_string_lossy());
    }
}

#[when(regex = r#"^I run the SSG with --root ([^\s]+) --out ([^\s]+)$"#)]
pub async fn when_run_ssg_with_root_out(world: &mut World, root_rel: String, out_label: String) {
    // Compose explicit root for this run
    let base = workspace_root();
    let root = base.join(&root_rel);
    world.set("repo_root", root.to_string_lossy());
    // Make manifest path relative to root for this run
    world.set("manifest_path", ".provenance/manifest.json");
    run_ssg(world, &out_label, None, false);
    // For assertions in ssg_render.feature which reference paths like "site/index.html" relative to repo base,
    // set out_dir to the repo base so the Then step joins it with e.g. "site/index.html".
    world.set("out_dir", PathBuf::from(base).to_string_lossy());
}

#[when(regex = r#"^I build the site$"#)]
pub async fn when_build_the_site(world: &mut World) {
    let limit = world.get("truncate_limit").and_then(|s| s.parse::<usize>().ok());
    run_ssg(world, "site", limit, false);
    // 'out_dir' remains the repo root for assertions like "site/index.html"
    let base = world.repo_root();
    world.set("out_dir", PathBuf::from(base).to_string_lossy());
}

#[given(regex = r#"^a large JSON artifact exceeding the configured size limit$"#)]
pub async fn given_large_json_exceeding_limit(world: &mut World) {
    // Force truncation for inline renderers
    world.set("truncate_limit", "1");
}

#[then(regex = r#"^the artifact page contains the text \"([^\"]+)\"$"#)]
pub async fn then_any_artifact_page_contains(world: &mut World, needle: String) {
    let out = PathBuf::from(world.get("out_dir").expect("out_dir not set"));
    let a_dir = out.join("site").join("a");
    let mut found = false;
    if a_dir.is_dir() {
        for entry in walkdir::WalkDir::new(&a_dir).min_depth(2).max_depth(2) {
            let ent = entry.expect("walk");
            if ent.file_type().is_file() && ent.path().ends_with("index.html") {
                let txt = std::fs::read_to_string(ent.path()).unwrap_or_default();
                if txt.contains(&needle) { found = true; break; }
            }
        }
    }
    assert!(found, "no artifact page under '{}' contained '{}'", a_dir.display(), needle);
}

#[when(regex = r#"^I build the site twice with the same inputs$"#)]
pub async fn when_build_twice(world: &mut World) {
    // First run
    let out1 = temp_out();
    let root = repo_root(world);
    let manifest_rel = PathBuf::from(world.get("manifest_path").unwrap_or(".provenance/manifest.json".into()));
    let manifest1 = if manifest_rel.is_absolute() { manifest_rel.clone() } else { root.join(&manifest_rel) };
    let schema1 = world.get("schema_path").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("schemas/manifest.schema.json"));
    let schema1 = if schema1.is_absolute() { schema1 } else { workspace_root().join(schema1) };
    let args1 = provenance_ssg::Args {
        root: root.clone(),
        manifest: manifest1.strip_prefix(&root).unwrap_or(&manifest1).to_path_buf(),
        out: out1.clone(),
        copy_assets: true,
        verify_manifest: false,
        pubkey: None,
        schema_path: Some(schema1),
        truncate_inline_bytes: 1_000_000,
    };
    provenance_ssg::run_with_args(args1).expect("first run ok");

    // Second run
    let out2 = temp_out();
    let args2 = provenance_ssg::Args {
        root: root.clone(),
        manifest: manifest_rel,
        out: out2.clone(),
        copy_assets: true,
        verify_manifest: false,
        pubkey: None,
        schema_path: Some(workspace_root().join("schemas/manifest.schema.json")),
        truncate_inline_bytes: 1_000_000,
    };
    provenance_ssg::run_with_args(args2).expect("second run ok");

    world.set("out_dir_1", out1.to_string_lossy());
    world.set("out_dir_2", out2.to_string_lossy());
}

#[then(regex = r#"^the byte contents of \"([^\"]+)\" are identical between runs$"#)]
pub async fn then_bytes_identical_between_runs(world: &mut World, rel: String) {
    let mut rel_path = rel;
    // Accept either "site/..." or plain relative; strip leading "site/" to map into out dirs
    if let Some(stripped) = rel_path.strip_prefix("site/") { rel_path = stripped.to_string(); }
    let out1 = PathBuf::from(world.get("out_dir_1").expect("out_dir_1 not set"));
    let out2 = PathBuf::from(world.get("out_dir_2").expect("out_dir_2 not set"));
    let p1 = out1.join(&rel_path);
    let p2 = out2.join(&rel_path);
    let b1 = std::fs::read(&p1).expect("read run1 file");
    let b2 = std::fs::read(&p2).expect("read run2 file");
    assert_eq!(b1, b2, "bytes differ for {} and {}", p1.display(), p2.display());
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
    let pk_path = repo_root(world).join(".provenance/public_test_ed25519.key.b64");
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
    let schema_rel = world.get("schema_path").map(PathBuf::from);
    let schema = match schema_rel {
        Some(p) if p.is_absolute() => p,
        Some(p) => workspace_root().join(p),
        None => workspace_root().join("schemas/manifest.schema.json"),
    };

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
    let p = if out.ends_with("site") && rel.starts_with("site/") {
        out.join(rel.trim_start_matches("site/"))
    } else {
        out.join(rel)
    };
    assert!(p.is_file(), "missing file: {}", p.display());
}

#[then(regex = r#"^the file \"([^\"]+)\" exists$"#)]
pub async fn then_the_file_exists(world: &mut World, rel: String) {
    let out = PathBuf::from(world.get("out_dir").expect("out_dir not set"));
    let p = if out.ends_with("site") && rel.starts_with("site/") {
        out.join(rel.trim_start_matches("site/"))
    } else {
        out.join(rel)
    };
    assert!(p.is_file(), "missing file: {}", p.display());
}

#[then(regex = r#"^the artifact page shows a truncation notice$"#)]
pub async fn then_truncation_notice(world: &mut World) {
    then_any_artifact_page_contains(world, "Truncated".to_string()).await;
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
    let src = repo_root(world);
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
