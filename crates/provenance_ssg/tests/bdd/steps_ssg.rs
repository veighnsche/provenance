use anyhow::{anyhow, Result};
use bdd_harness::{Step, State};
use std::fs;
use std::path::{Path, PathBuf};

pub fn registry() -> Vec<Step> {
    vec![
        step(r#"a valid, signed manifest and verified artifacts for tests and coverage"#, given_valid_signed_manifest),
        step(r#"the repo root is \"([^\"]+)\""#, given_repo_root),
        step(r#"the manifest path is \"([^\"]+)\""#, given_manifest_path),
        step(r#"the JSON schema path is \"([^\"]+)\""#, given_schema_path),
        step(r#"file \"([^\"]+)\" exists"#, then_file_exists_under_repo),
        step(r#"I run the SSG with output dir \"([^\"]+)\""#, when_run_ssg_out),
        step(r#"I run the SSG with output dir \"([^\"]+)\" and truncate inline bytes to (\d+)"#, when_run_ssg_out_trunc),
        step(r#"I generate badges"#, when_generate_badges),
        step(r#"file \"([^\"]+)\" should exist"#, then_file_exists_under_out),
        step(r#"HTML at \"([^\"]+)\" should contain \"([^\"]+)\""#, then_html_contains_under_out),
        step(r#"the JSON has schemaVersion (\d+)"#, then_badge_schema_version),
        step(r#"the JSON message contains \"([^\"]+)\""#, then_badge_message_contains),
        step(r#"the JSON message matches \"([^\"]+)\""#, then_badge_message_matches),
        step(r#"the coverage artifact is missing"#, given_coverage_artifact_missing),
        step(r#"the JSON badge for coverage has message \"([^\"]+)\""#, then_coverage_badge_message_is),
        step(r#"the JSON badge for coverage uses color \"([^\"]+)\""#, then_coverage_badge_color_is),
    ]
}

fn given_valid_signed_manifest(state: &mut State, _caps: &regex::Captures) -> Result<()> {
    // Point repo_root to the provided minimal example which contains a signed manifest
    // and verified artifacts for tests and coverage.
    let base = repo_root(state);
    let p = base.join("examples/minimal");
    // Sanity checks
    for rel in [
        ".provenance/manifest.json",
        ".provenance/manifest.json.sig",
        ".provenance/public_test_ed25519.key.b64",
        "ci/tests/summary.json",
        "ci/coverage/coverage.json",
    ] {
        let fp = p.join(rel);
        if !fp.is_file() { return Err(anyhow!("missing required file: {}", fp.display())); }
    }
    state.set("repo_root", p.to_string_lossy());
    // Default manifest path relative to repo_root
    state.set("manifest_path", ".provenance/manifest.json");
    Ok(())
}

fn given_coverage_artifact_missing(state: &mut State, _caps: &regex::Captures) -> Result<()> {
    // Create a temporary copy of examples/minimal and remove the coverage artifact
    let src = repo_root(state).join("examples/minimal");
    if !src.is_dir() { return Err(anyhow!("source example not found: {}", src.display())); }
    let dst = temp_out();
    // Shallow recursive copy
    copy_tree(&src, &dst)?;
    let cov = dst.join("ci/coverage/coverage.json");
    if cov.exists() { std::fs::remove_file(&cov).map_err(|e| anyhow!("rm {}: {}", cov.display(), e))?; }
    state.set("repo_root", dst.to_string_lossy());
    state.set("manifest_path", ".provenance/manifest.json");
    Ok(())
}

fn then_coverage_badge_message_is(state: &mut State, caps: &regex::Captures) -> Result<()> {
    let want = caps.get(1).unwrap().as_str();
    let out = PathBuf::from(state.get("out_dir").ok_or_else(|| anyhow!("out_dir not set"))?);
    let p = out.join("badge/coverage.json");
    let v: serde_json::Value = serde_json::from_str(&fs::read_to_string(&p)?)?;
    let msg = v.get("message").and_then(|v| v.as_str()).unwrap_or("");
    if msg == want { Ok(()) } else { Err(anyhow!("coverage badge message '{}' != '{}' in {}", msg, want, p.display())) }
}

fn then_coverage_badge_color_is(state: &mut State, caps: &regex::Captures) -> Result<()> {
    let want = caps.get(1).unwrap().as_str();
    let out = PathBuf::from(state.get("out_dir").ok_or_else(|| anyhow!("out_dir not set"))?);
    let p = out.join("badge/coverage.json");
    let v: serde_json::Value = serde_json::from_str(&fs::read_to_string(&p)?)?;
    let color = v.get("color").and_then(|v| v.as_str()).unwrap_or("");
    if color == want { Ok(()) } else { Err(anyhow!("coverage badge color '{}' != '{}' in {}", color, want, p.display())) }
}

fn copy_tree(src: &Path, dst: &Path) -> Result<()> {
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

fn step(pat: &str, f: fn(&mut State, &regex::Captures) -> Result<()>) -> Step { Step::new(pat, f).unwrap() }

fn repo_root(state: &State) -> PathBuf {
    let rr = state.get("repo_root").unwrap_or_else(|| state.get("base_dir").unwrap());
    PathBuf::from(rr)
}

fn given_repo_root(state: &mut State, caps: &regex::Captures) -> Result<()> {
    let rel = caps.get(1).unwrap().as_str();
    let p = if Path::new(rel).is_absolute() { PathBuf::from(rel) } else { repo_root(state).join(rel) };
    state.set("repo_root", p.to_string_lossy().to_string());
    Ok(())
}

fn given_manifest_path(state: &mut State, caps: &regex::Captures) -> Result<()> {
    let rel = caps.get(1).unwrap().as_str();
    let p = if Path::new(rel).is_absolute() { PathBuf::from(rel) } else { repo_root(state).join(rel) };
    state.set("manifest_path", p.to_string_lossy().to_string());
    Ok(())
}

fn given_schema_path(state: &mut State, caps: &regex::Captures) -> Result<()> {
    let rel = caps.get(1).unwrap().as_str();
    let p = if Path::new(rel).is_absolute() { PathBuf::from(rel) } else { repo_root(state).join(rel) };
    state.set("schema_path", p.to_string_lossy().to_string());
    Ok(())
}

fn then_file_exists_under_repo(state: &mut State, caps: &regex::Captures) -> Result<()> {
    let rel = caps.get(1).unwrap().as_str();
    let p = repo_root(state).join(rel);
    if p.is_file() { Ok(()) } else { Err(anyhow!("missing file: {}", p.display())) }
}

fn when_run_ssg_out(state: &mut State, caps: &regex::Captures) -> Result<()> {
    run_ssg(state, caps.get(1).unwrap().as_str(), None, false)
}

fn when_run_ssg_out_trunc(state: &mut State, caps: &regex::Captures) -> Result<()> {
    let out = caps.get(1).unwrap().as_str();
    let limit: usize = caps.get(2).unwrap().as_str().parse().unwrap_or(1);
    run_ssg(state, out, Some(limit), false)
}

fn when_generate_badges(state: &mut State, _caps: &regex::Captures) -> Result<()> {
    // Ensure verify=true with repo pubkey and write to repo_root/site
    state.set("verify_manifest", "true");
    let pk_path = repo_root(state).join("examples/minimal/.provenance/public_test_ed25519.key.b64");
    let pk_b64 = std::fs::read_to_string(&pk_path)?;
    state.set("pubkey_b64", pk_b64.trim());
    run_ssg(state, "site", None, true)
}

fn run_ssg(state: &mut State, out_label: &str, limit: Option<usize>, verify: bool) -> Result<()> {
    let out = if out_label == "TMP" { temp_out() } else {
        let p = PathBuf::from(out_label);
        if p.is_absolute() { p } else { repo_root(state).join(p) }
    };
    let root = repo_root(state);
    let manifest = PathBuf::from(state.get("manifest_path").unwrap_or(".provenance/manifest.json".into()));
    let manifest = if manifest.is_absolute() { manifest } else { root.join(manifest) };
    let schema = state.get("schema_path").map(PathBuf::from).unwrap_or_else(|| repo_root(state).join("schemas/manifest.schema.json"));

    let args = provenance_ssg::Args {
        root: root,
        manifest: manifest.strip_prefix(&repo_root(state)).unwrap_or(&manifest).to_path_buf(),
        out: out.clone(),
        copy_assets: true,
        verify_manifest: verify || state.get("verify_manifest").unwrap_or_default() == "true",
        pubkey: state.get("pubkey_b64"),
        schema_path: Some(schema),
        truncate_inline_bytes: limit.unwrap_or(1_000_000),
    };
    provenance_ssg::run_with_args(args)?;
    state.set("out_dir", out.to_string_lossy().to_string());
    Ok(())
}

fn then_file_exists_under_out(state: &mut State, caps: &regex::Captures) -> Result<()> {
    let rel = caps.get(1).unwrap().as_str();
    let out = PathBuf::from(state.get("out_dir").ok_or_else(|| anyhow!("out_dir not set"))?);
    let p = out.join(rel);
    if p.is_file() { Ok(()) } else { Err(anyhow!("missing file: {}", p.display())) }
}

fn then_html_contains_under_out(state: &mut State, caps: &regex::Captures) -> Result<()> {
    let rel = caps.get(1).unwrap().as_str();
    let needle = caps.get(2).unwrap().as_str();
    let out = PathBuf::from(state.get("out_dir").ok_or_else(|| anyhow!("out_dir not set"))?);
    let p = out.join(rel);
    let txt = fs::read_to_string(&p).map_err(|e| anyhow!("read {}: {}", p.display(), e))?;
    if txt.contains(needle) { Ok(()) } else { Err(anyhow!("HTML did not contain '{}': {}", needle, p.display())) }
}

fn then_badge_schema_version(state: &mut State, caps: &regex::Captures) -> Result<()> {
    let want: u64 = caps.get(1).unwrap().as_str().parse().unwrap_or(1);
    let out = PathBuf::from(state.get("out_dir").ok_or_else(|| anyhow!("out_dir not set"))?);
    let p = out.join("badge/provenance.json");
    let v: serde_json::Value = serde_json::from_str(&fs::read_to_string(&p)?)?;
    let got = v.get("schemaVersion").and_then(|v| v.as_u64()).unwrap_or(0);
    if got == want { Ok(()) } else { Err(anyhow!("schemaVersion {}, want {} in {}", got, want, p.display())) }
}

fn then_badge_message_contains(state: &mut State, caps: &regex::Captures) -> Result<()> {
    let needle = caps.get(1).unwrap().as_str();
    let out = PathBuf::from(state.get("out_dir").ok_or_else(|| anyhow!("out_dir not set"))?);
    // Probe all badge JSONs
    for name in ["provenance", "tests", "coverage"] {
        let p = out.join(format!("badge/{}.json", name));
        if p.is_file() {
            let v: serde_json::Value = serde_json::from_str(&fs::read_to_string(&p)?)?;
            if let Some(msg) = v.get("message").and_then(|v| v.as_str()) {
                if msg.contains(needle) { return Ok(()); }
            }
        }
    }
    Err(anyhow!("no badge message contained '{}'", needle))
}

fn then_badge_message_matches(state: &mut State, caps: &regex::Captures) -> Result<()> {
    let patt = regex::Regex::new(caps.get(1).unwrap().as_str())?;
    let out = PathBuf::from(state.get("out_dir").ok_or_else(|| anyhow!("out_dir not set"))?);
    let p = out.join("badge/coverage.json");
    let v: serde_json::Value = serde_json::from_str(&fs::read_to_string(&p)?)?;
    let msg = v.get("message").and_then(|v| v.as_str()).unwrap_or("");
    if patt.is_match(msg) { Ok(()) } else { Err(anyhow!("badge message '{}' did not match {}", msg, patt)) }
}

fn temp_out() -> PathBuf {
    let mut p = std::env::temp_dir();
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    p.push(format!("prov-bdd-out-{}", nanos));
    p
}
