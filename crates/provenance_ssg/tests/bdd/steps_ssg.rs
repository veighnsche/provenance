use anyhow::{anyhow, Result};
use bdd_harness::{Step, State};
use std::fs;
use std::path::{Path, PathBuf};

pub fn registry() -> Vec<Step> {
    vec![
        step(r#"a valid, signed manifest and verified artifacts for tests and coverage"#, given_valid_signed_manifest),
        step(r#"the repo root is "([^"]+)""#, given_repo_root),
        step(r#"the manifest path is "([^"]+)""#, given_manifest_path),
        step(r#"the JSON schema path is "([^"]+)""#, given_schema_path),
        step(r#"file "([^"]+)" exists"#, then_file_exists_under_repo),
        step(r#"I run the SSG with output dir "([^"]+)""#, when_run_ssg_out),
        step(r#"I run the SSG with output dir "([^"]+)" and truncate inline bytes to (\d+)"#, when_run_ssg_out_trunc),
        step(r#"I generate badges"#, when_generate_badges),
        step(r#"file "([^"]+)" should exist"#, then_file_exists_under_out),
        step(r#"HTML at "([^"]+)" should contain "([^"]+)""#, then_html_contains_under_out),
        step(r#"the JSON has schemaVersion (\d+)"#, then_badge_schema_version),
        step(r#"the JSON message contains "([^"]+)""#, then_badge_message_contains),
        step(r#"the JSON message matches "([^"]+)""#, then_badge_message_matches),
    ]
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
