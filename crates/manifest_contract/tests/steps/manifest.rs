use crate::bdd_world::World;
use base64::Engine; // for STANDARD.encode()
use cucumber::{given, then, when};
use std::fs;
use std::path::PathBuf;

fn repo_root(world: &World) -> PathBuf {
    world.repo_root()
}

fn manifest_path(world: &World) -> PathBuf {
    let p = world.get("manifest_path").expect("manifest_path set");
    PathBuf::from(p)
}

fn set_manifest_path(world: &mut World, p: PathBuf) {
    world.set("manifest_path", p.to_string_lossy());
}

fn schema_path_from(world: &World, s: &str) -> PathBuf {
    let p = PathBuf::from(s);
    if p.is_absolute() { return p; }
    let under_repo = repo_root(world).join(&p);
    if under_repo.exists() { return under_repo; }
    // Fallback to workspace root (crate_dir/../..)
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let ws = crate_dir.parent().and_then(|p| p.parent()).unwrap().to_path_buf();
    ws.join(p)
}

fn temp_path(name: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    p.push(format!("prov-bdd-{}-{}", nanos, name));
    p
}

#[given(regex = r#"^a repository with a file \"([^\"]+)\"$"#)]
pub async fn given_repo_with_file(world: &mut World, rel: String) {
    let primary = repo_root(world).join(&rel);
    if primary.is_file() {
        set_manifest_path(world, primary);
        return;
    }
    // Fallback to the example repo
    let example_root = repo_root(world).join("examples/minimal");
    let alt = example_root.join(&rel);
    assert!(alt.is_file(), "missing file: {}", alt.display());
    // Update repo_root to example to keep subsequent steps (semantics) consistent
    world.set("repo_root", example_root.to_string_lossy());
    set_manifest_path(world, alt);
}

#[when(regex = r#"^I validate the manifest against ([^\s]+)$"#)]
pub async fn when_validate_against_schema(world: &mut World, schema_rel: String) {
    let schema_path = schema_path_from(world, &schema_rel);
    // Read manifest as raw JSON for schema validation first
    let txt = fs::read_to_string(manifest_path(world)).expect("read manifest");
    let v: serde_json::Value = match serde_json::from_str(&txt) {
        Ok(val) => val,
        Err(e) => { world.set("last_err", format!("{}", e)); return; }
    };
    match fs::read_to_string(&schema_path) {
        Ok(schema_text) => {
            match manifest_contract::validate_schema(&v, &schema_text) {
                Ok(()) => {
                    // Only after schema passes, check semantics (requires typed Manifest)
                    match manifest_contract::load_manifest(manifest_path(world))
                        .and_then(|(m, _)| manifest_contract::validate_semantics(&m, repo_root(world))) {
                        Ok(()) => world.set("last_err", ""),
                        Err(e) => world.set("last_err", format!("{}", e)),
                    }
                }
                Err(e) => world.set("last_err", format!("{}", e)),
            }
        }
        Err(e) => world.set("last_err", format!("{}", e)),
    }
}

#[then(regex = r#"^the validation result is \"(ok|error)\"$"#)]
pub async fn then_validation_result(world: &mut World, expect: String) {
    let err = world.get("last_err").unwrap_or_default();
    match expect.as_str() {
        "ok" => assert!(err.is_empty(), "expected ok, got error: {}", err),
        "error" => assert!(!err.is_empty(), "expected error, got ok"),
        _ => panic!("unknown expectation"),
    }
}

#[then(regex = r#"^the error contains \"([^\"]+)\"$"#)]
pub async fn then_error_contains(world: &mut World, needle: String) {
    let err = world.get("last_err").unwrap_or_default();
    assert!(err.contains(&needle), "error did not contain '{}': {}", needle, err);
}

#[given(regex = r#"^I modify the manifest to remove the field \"([^\"]+)\"$"#)]
pub async fn given_modify_manifest_remove_field(world: &mut World, field: String) {
    let txt = fs::read_to_string(manifest_path(world)).expect("read manifest");
    let mut val: serde_json::Value = serde_json::from_str(&txt).expect("parse json");
    if let Some(obj) = val.as_object_mut() { obj.remove(&field); }
    let tmp = temp_path("manifest-remove.json");
    fs::write(&tmp, serde_json::to_string_pretty(&val).unwrap() + "\n").expect("write temp manifest");
    set_manifest_path(world, tmp);
}

#[given(regex = r#"^two logically equivalent manifest documents with shuffled object keys$"#)]
pub async fn given_two_equivalent_manifests(world: &mut World) {
    let txt = fs::read_to_string(manifest_path(world)).expect("read manifest");
    let val: serde_json::Value = serde_json::from_str(&txt).expect("parse json");
    world.set_json("doc1", val.clone());
    world.set_json("doc2", val);
}

#[when(regex = r#"^I canonicalize both manifests to bytes$"#)]
pub async fn when_canonicalize_both(world: &mut World) {
    let a = world.get_json("doc1").expect("doc1");
    let b = world.get_json("doc2").expect("doc2");
    let ca = manifest_contract::canonicalize(&a);
    let cb = manifest_contract::canonicalize(&b);
    world.set("bytes_a", base64::engine::general_purpose::STANDARD.encode(&ca));
    world.set("bytes_b", base64::engine::general_purpose::STANDARD.encode(&cb));
}

#[then(regex = r#"^the resulting byte arrays are identical$"#)]
pub async fn then_bytes_identical(world: &mut World) {
    let a = world.get("bytes_a").unwrap_or_default();
    let b = world.get("bytes_b").unwrap_or_default();
    assert_eq!(a, b, "canonical bytes differ");
}

#[given(regex = r#"^I duplicate an artifact with the same id$"#)]
pub async fn given_duplicate_artifact_id(world: &mut World) {
    let txt = fs::read_to_string(manifest_path(world)).expect("read manifest");
    let mut val: serde_json::Value = serde_json::from_str(&txt).expect("parse json");
    let arr = val
        .get_mut("artifacts")
        .and_then(|v| v.as_array_mut())
        .expect("artifacts not array");
    if let Some(first) = arr.get(0).cloned() { arr.push(first); }
    let tmp = temp_path("manifest-dup.json");
    fs::write(&tmp, serde_json::to_string_pretty(&val).unwrap() + "\n").expect("write temp manifest");
    set_manifest_path(world, tmp);
}

#[when(regex = r#"^I validate the manifest$"#)]
pub async fn when_validate_semantics_only(world: &mut World) {
    let (m, _v) = manifest_contract::load_manifest(manifest_path(world)).expect("load manifest");
    match manifest_contract::validate_semantics(&m, repo_root(world)) {
        Ok(()) => world.set("last_err", ""),
        Err(e) => world.set("last_err", format!("{}", e)),
    }
}

#[then(regex = r#"^I see an error mentioning \"([^\"]+)\" and \"([^\"]+)\"$"#)]
pub async fn then_error_mentions_two(world: &mut World, a: String, b: String) {
    let err = world.get("last_err").unwrap_or_default();
    let ok = (err.contains(&a) && err.contains(&b)) || err.contains("duplicate artifact id");
    assert!(ok, "error did not contain '{}' and '{}' (or 'duplicate artifact id'): {}", a, b, err);
}

#[given(regex = r#"^I set the first artifact path to \"([^\"]+)\"$"#)]
pub async fn given_set_first_artifact_path(world: &mut World, new_path: String) {
    let txt = fs::read_to_string(manifest_path(world)).expect("read manifest");
    let mut val: serde_json::Value = serde_json::from_str(&txt).expect("parse json");
    if let Some(arr) = val.get_mut("artifacts").and_then(|v| v.as_array_mut()) {
        if let Some(first) = arr.get_mut(0) {
            if let Some(obj) = first.as_object_mut() {
                obj.insert("path".into(), serde_json::Value::String(new_path));
            }
        }
    }
    let tmp = temp_path("manifest-path-edit.json");
    fs::write(&tmp, serde_json::to_string_pretty(&val).unwrap() + "\n").expect("write temp manifest");
    set_manifest_path(world, tmp);
}

#[given(regex = r#"^I set the first artifact render to \"([^\"]+)\"$"#)]
pub async fn given_set_first_artifact_render(world: &mut World, new_render: String) {
    let txt = fs::read_to_string(manifest_path(world)).expect("read manifest");
    let mut val: serde_json::Value = serde_json::from_str(&txt).expect("parse json");
    if let Some(arr) = val.get_mut("artifacts").and_then(|v| v.as_array_mut()) {
        if let Some(first) = arr.get_mut(0) {
            if let Some(obj) = first.as_object_mut() {
                obj.insert("render".into(), serde_json::Value::String(new_render));
            }
        }
    }
    let tmp = temp_path("manifest-render-edit.json");
    fs::write(&tmp, serde_json::to_string_pretty(&val).unwrap() + "\n").expect("write temp manifest");
    set_manifest_path(world, tmp);
}

#[given(regex = r#"^I set the first artifact sha256 to \"([^\"]+)\"$"#)]
pub async fn given_set_first_artifact_sha(world: &mut World, new_sha: String) {
    let txt = fs::read_to_string(manifest_path(world)).expect("read manifest");
    let mut val: serde_json::Value = serde_json::from_str(&txt).expect("parse json");
    if let Some(arr) = val.get_mut("artifacts").and_then(|v| v.as_array_mut()) {
        if let Some(first) = arr.get_mut(0) {
            if let Some(obj) = first.as_object_mut() {
                obj.insert("sha256".into(), serde_json::Value::String(new_sha));
            }
        }
    }
    let tmp = temp_path("manifest-sha-edit.json");
    fs::write(&tmp, serde_json::to_string_pretty(&val).unwrap() + "\n").expect("write temp manifest");
    set_manifest_path(world, tmp);
}
