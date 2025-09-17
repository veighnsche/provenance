use crate::bdd_world::World;
use base64::Engine; // for STANDARD.encode()/decode()
use cucumber::{given, then, when};
use std::fs;
use std::path::PathBuf;

fn repo_root(world: &World) -> PathBuf { world.repo_root() }

fn temp_path(name: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    p.push(format!("prov-bdd-{}-{}", nanos, name));
    p
}

#[given(regex = r#"^a canonicalized manifest at \"([^\"]+)\"$"#)]
pub async fn given_canonicalized_manifest_at(world: &mut World, rel: String) {
    let mut path = repo_root(world).join(&rel);
    if !path.is_file() {
        let example_root = repo_root(world).join("examples/minimal");
        let alt = example_root.join(&rel);
        assert!(alt.is_file(), "missing manifest: {}", alt.display());
        world.set("repo_root", example_root.to_string_lossy());
        path = alt;
    }
    let txt = fs::read_to_string(&path).expect("read manifest");
    let val: serde_json::Value = serde_json::from_str(&txt).expect("parse json");
    let canonical = manifest_contract::canonicalize(&val);
    world.set("canonical_b64", base64::engine::general_purpose::STANDARD.encode(&canonical));
    world.set("manifest_path", path.to_string_lossy());
}

#[given(regex = r#"^a Base64 signature at \"([^\"]+)\"$"#)]
pub async fn given_signature_at(world: &mut World, rel: String) {
    let mut path = repo_root(world).join(&rel);
    if !path.is_file() {
        let example_root = repo_root(world).join("examples/minimal");
        let alt = example_root.join(&rel);
        assert!(alt.is_file(), "missing signature: {}", alt.display());
        world.set("repo_root", example_root.to_string_lossy());
        path = alt;
    }
    let sig_b64 = fs::read_to_string(&path).expect("read sig");
    world.set("sig_b64", sig_b64.trim());
}

#[given(regex = r#"^the public key INDEX_PUBKEY_ED25519 for the manifest$"#)]
pub async fn given_index_pubkey(world: &mut World) {
    let pk_path = resolve_pubkey_path(world);
    let pk_b64 = fs::read_to_string(&pk_path).expect("read pubkey");
    world.set("pubkey", pk_b64.trim());
}

#[given(regex = r#"^a different public key$"#)]
pub async fn given_different_pubkey(world: &mut World) {
    let pk_path = resolve_pubkey_path(world);
    let pk_b64 = fs::read_to_string(&pk_path).expect("read pubkey");
    // Decode, flip one byte, re-encode to Base64 to keep format valid
    let mut bytes = base64::engine::general_purpose::STANDARD
        .decode(pk_b64.trim())
        .expect("decode pubkey b64");
    if let Some(b0) = bytes.get_mut(0) { *b0 ^= 0x01; }
    let wrong_b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    world.set("pubkey", wrong_b64);
}

#[given(regex = r#"^I change one character in the manifest$"#)]
pub async fn given_change_one_char_in_manifest(world: &mut World) {
    let mp = world.get("manifest_path").expect("manifest_path not set");
    let txt = fs::read_to_string(&PathBuf::from(&mp)).expect("read manifest");
    let mut val: serde_json::Value = serde_json::from_str(&txt).expect("parse json");
    if !mutate_one_char(&mut val) {
        panic!("could not find a string field to mutate in manifest");
    }
    let tmp = temp_path("manifest-onechar.json");
    fs::write(&tmp, serde_json::to_string_pretty(&val).unwrap() + "\n").expect("write temp manifest");
    world.set("manifest_path", tmp.to_string_lossy());
    // update canonical for subsequent verify
    let canonical = manifest_contract::canonicalize(&val);
    world.set("canonical_b64", base64::engine::general_purpose::STANDARD.encode(&canonical));
}

fn mutate_one_char(v: &mut serde_json::Value) -> bool {
    if let Some(obj) = v.as_object_mut() {
        if let Some(c) = obj.get_mut("commit") {
            if let Some(s) = c.as_str() {
                *c = serde_json::Value::String(flip_first_char(s));
                return true;
            }
        }
    }
    fn recurse(val: &mut serde_json::Value) -> bool {
        match val {
            serde_json::Value::String(s) => { let new = flip_first_char(s); *s = new; true }
            serde_json::Value::Array(arr) => { for el in arr.iter_mut() { if recurse(el) { return true; } } false }
            serde_json::Value::Object(map) => { for (_k, v) in map.iter_mut() { if recurse(v) { return true; } } false }
            _ => false,
        }
    }
    recurse(v)
}

fn flip_first_char(s: &str) -> String {
    let mut chars: Vec<char> = s.chars().collect();
    if let Some(first) = chars.get_mut(0) {
        *first = match *first {
            'A' => 'B', 'B' => 'A',
            'a' => 'b', 'b' => 'a',
            '0' => '1', '1' => '0',
            other => if other.is_ascii_alphanumeric() { 'X' } else { other },
        };
    }
    chars.into_iter().collect()
}

#[when(regex = r#"^I verify the signature against the canonical manifest bytes$"#)]
pub async fn when_verify_signature_against_canonical(world: &mut World) {
    verify_now(world).await;
}

#[when(regex = r#"^I verify the signature$"#)]
pub async fn when_verify_signature(world: &mut World) {
    verify_now(world).await;
}

#[when(regex = r#"^I canonicalize and verify$"#)]
pub async fn when_canonicalize_and_verify(world: &mut World) {
    let mp = world.get("manifest_path").expect("manifest_path not set");
    let txt = fs::read_to_string(&PathBuf::from(mp)).expect("read manifest");
    let val: serde_json::Value = serde_json::from_str(&txt).expect("parse json");
    let canonical = manifest_contract::canonicalize(&val);
    world.set("canonical_b64", base64::engine::general_purpose::STANDARD.encode(&canonical));
    verify_now(world).await;
}

async fn verify_now(world: &mut World) {
    let can_b64 = world.get("canonical_b64").expect("canonical not set");
    let sig_b64 = world.get("sig_b64").expect("sig not set");
    let pubkey = match world.get("pubkey") {
        Some(pk) => pk,
        None => {
            // Try to auto-load the default index pubkey
            let pk_path = resolve_pubkey_path(world);
            let pk_b64 = fs::read_to_string(&pk_path).expect("read pubkey");
            let pk = pk_b64.trim().to_string();
            world.set("pubkey", &pk);
            pk
        }
    };
    let can = base64::engine::general_purpose::STANDARD.decode(can_b64).unwrap();
    match manifest_contract::ed25519_verify(&can, &sig_b64, &pubkey) {
        Ok(ok) => {
            world.set("verify_ok", if ok { "true" } else { "false" });
            if ok { world.set("last_err", ""); } else { world.set("last_err", "signature mismatch"); }
        }
        Err(e) => { world.set("verify_ok", "false"); world.set("last_err", format!("{}", e)); }
    }
}

fn resolve_pubkey_path(world: &World) -> PathBuf {
    let rr = repo_root(world);
    let p1 = rr.join(".provenance/public_test_ed25519.key.b64");
    if p1.is_file() { return p1; }
    let p2 = rr.join("examples/minimal/.provenance/public_test_ed25519.key.b64");
    if p2.is_file() { return p2; }
    // Final fallback: workspace example
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let ws = crate_dir.parent().and_then(|p| p.parent()).unwrap().to_path_buf();
    ws.join("examples/minimal/.provenance/public_test_ed25519.key.b64")
}

#[then(regex = r#"^the verification result is \"(ok|error)\"$"#)]
pub async fn then_verification_result(world: &mut World, expect: String) {
    let ok = world.get("verify_ok").unwrap_or_else(|| "false".into());
    match expect.as_str() {
        "ok" => assert_eq!(ok, "true", "expected ok, got false ({} )", world.get("last_err").unwrap_or_default()),
        "error" => assert_eq!(ok, "false", "expected error, got ok"),
        _ => panic!("unknown expectation"),
    }
}
