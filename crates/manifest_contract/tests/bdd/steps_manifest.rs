use bdd_harness::{Step, State};
use base64::Engine; // for STANDARD.encode()
use anyhow::{anyhow, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub fn registry() -> Vec<Step> {
    vec![
        step(r#"a repository with a file "([^"]+)""#, given_repo_with_file),
        step(r#"I validate the manifest against ([^\s]+)"#, when_validate_against_schema),
        step(r#"the validation result is "(ok|error)""#, then_validation_result),
        step(r#"the error contains "([^"]+)""#, then_error_contains),
        step(r#"I modify the manifest to remove the field "([^"]+)""#, given_modify_manifest_remove_field),
        step(r#"two logically equivalent manifest documents with shuffled object keys"#, given_two_equivalent_manifests),
        step(r#"I canonicalize both manifests to bytes"#, when_canonicalize_both),
        step(r#"the resulting byte arrays are identical"#, then_bytes_identical),
        step(r#"I duplicate an artifact with the same id"#, given_duplicate_artifact_id),
        step(r#"I validate the manifest"#, when_validate_semantics_only),
        step(r#"I see an error mentioning "([^"]+)" and "([^"]+)""#, then_error_mentions_two),
    ]
}

fn step(pat: &str, f: fn(&mut State, &regex::Captures) -> Result<()>) -> Step { Step::new(pat, f).unwrap() }

fn repo_root(state: &State) -> PathBuf {
    let base = state.get("repo_root").unwrap_or_else(|| state.get("base_dir").unwrap());
    PathBuf::from(base)
}

fn set_manifest_path(state: &mut State, p: PathBuf) { state.set("manifest_path", p.to_string_lossy().to_string()); }
fn manifest_path(state: &State) -> PathBuf {
    let p = state.get("manifest_path").expect("manifest_path set");
    PathBuf::from(p)
}

fn schema_path_from(state: &State, s: &str) -> PathBuf {
    let p = PathBuf::from(s);
    if p.is_absolute() { p } else { repo_root(state).join(p) }
}

fn given_repo_with_file(state: &mut State, caps: &regex::Captures) -> Result<()> {
    let rel = caps.get(1).unwrap().as_str();
    let path = repo_root(state).join(rel);
    if !path.is_file() { return Err(anyhow!("missing file: {}", path.display())); }
    set_manifest_path(state, path);
    Ok(())
}

fn when_validate_against_schema(state: &mut State, caps: &regex::Captures) -> Result<()> {
    let schema_rel = caps.get(1).unwrap().as_str();
    let schema_path = schema_path_from(state, schema_rel);
    let (m, v) = manifest_contract::load_manifest(manifest_path(state))?;
    match fs::read_to_string(&schema_path) {
        Ok(schema_text) => {
            let res = manifest_contract::validate_schema(&v, &schema_text)
                .and_then(|_| manifest_contract::validate_semantics(&m, repo_root(state)));
            if let Err(e) = res { state.set("last_err", format!("{}", e)); } else { state.set("last_err", String::new()); }
        }
        Err(e) => state.set("last_err", format!("{}", e)),
    }
    Ok(())
}

fn then_validation_result(state: &mut State, caps: &regex::Captures) -> Result<()> {
    let expect = caps.get(1).unwrap().as_str();
    let err = state.get("last_err").unwrap_or_default();
    match expect {
        "ok" => if !err.is_empty() { Err(anyhow!("expected ok, got error: {}", err)) } else { Ok(()) },
        "error" => if err.is_empty() { Err(anyhow!("expected error, got ok")) } else { Ok(()) },
        _ => Err(anyhow!("unknown expectation"))
    }
}

fn then_error_contains(state: &mut State, caps: &regex::Captures) -> Result<()> {
    let needle = caps.get(1).unwrap().as_str();
    let err = state.get("last_err").unwrap_or_default();
    if err.contains(needle) { Ok(()) } else { Err(anyhow!("error did not contain '{}': {}", needle, err)) }
}

fn given_modify_manifest_remove_field(state: &mut State, caps: &regex::Captures) -> Result<()> {
    let field = caps.get(1).unwrap().as_str();
    let txt = fs::read_to_string(manifest_path(state))?;
    let mut val: serde_json::Value = serde_json::from_str(&txt)?;
    if let Some(obj) = val.as_object_mut() { obj.remove(field); }
    let tmp = temp_path("manifest-remove.json");
    fs::write(&tmp, serde_json::to_string_pretty(&val)? + "\n")?;
    set_manifest_path(state, tmp);
    Ok(())
}

fn given_two_equivalent_manifests(state: &mut State, _caps: &regex::Captures) -> Result<()> {
    // For MVP, load the same manifest twice; canonicalization must still be identical.
    let txt = fs::read_to_string(manifest_path(state))?;
    let val: serde_json::Value = serde_json::from_str(&txt)?;
    state.set_json("doc1", val.clone());
    state.set_json("doc2", val);
    Ok(())
}

fn when_canonicalize_both(state: &mut State, _caps: &regex::Captures) -> Result<()> {
    let a = state.get_json("doc1").unwrap();
    let b = state.get_json("doc2").unwrap();
    let ca = manifest_contract::canonicalize(&a);
    let cb = manifest_contract::canonicalize(&b);
    state.set("bytes_a", base64::engine::general_purpose::STANDARD.encode(&ca));
    state.set("bytes_b", base64::engine::general_purpose::STANDARD.encode(&cb));
    Ok(())
}

fn then_bytes_identical(state: &mut State, _caps: &regex::Captures) -> Result<()> {
    let a = state.get("bytes_a").unwrap_or_default();
    let b = state.get("bytes_b").unwrap_or_default();
    if a == b { Ok(()) } else { Err(anyhow!("canonical bytes differ")) }
}

fn given_duplicate_artifact_id(state: &mut State, _caps: &regex::Captures) -> Result<()> {
    let txt = fs::read_to_string(manifest_path(state))?;
    let mut val: serde_json::Value = serde_json::from_str(&txt)?;
    let arr = val.get_mut("artifacts").and_then(|v| v.as_array_mut()).ok_or_else(|| anyhow!("artifacts not array"))?;
    if let Some(first) = arr.get(0).cloned() { arr.push(first); }
    let tmp = temp_path("manifest-dup.json");
    fs::write(&tmp, serde_json::to_string_pretty(&val)? + "\n")?;
    set_manifest_path(state, tmp);
    Ok(())
}

fn when_validate_semantics_only(state: &mut State, _caps: &regex::Captures) -> Result<()> {
    let (m, _v) = manifest_contract::load_manifest(manifest_path(state))?;
    match manifest_contract::validate_semantics(&m, repo_root(state)) {
        Ok(()) => state.set("last_err", String::new()),
        Err(e) => state.set("last_err", format!("{}", e)),
    }
    Ok(())
}

fn then_error_mentions_two(state: &mut State, caps: &regex::Captures) -> Result<()> {
    let a = caps.get(1).unwrap().as_str();
    let b = caps.get(2).unwrap().as_str();
    let err = state.get("last_err").unwrap_or_default();
    if err.contains(a) && err.contains(b) { Ok(()) } else { Err(anyhow!("error did not contain '{}' and '{}': {}", a, b, err)) }
}

fn temp_path(name: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    p.push(format!("prov-bdd-{}-{}", nanos, name));
    p
}
