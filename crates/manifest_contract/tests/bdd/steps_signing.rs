use bdd_harness::{Step, State};
use base64::Engine; // for STANDARD.encode()/decode()
use anyhow::{anyhow, Result};
use std::fs;
use std::path::PathBuf;

pub fn registry() -> Vec<Step> {
    vec![
        step(r#"a canonicalized manifest at \"([^\"]+)\""#, given_canonicalized_manifest_at),
        step(r#"a Base64 signature at \"([^\"]+)\""#, given_signature_at),
        step(r#"the public key INDEX_PUBKEY_ED25519 for the manifest"#, given_index_pubkey),
        step(r#"a different public key"#, given_different_pubkey),
        step(r#"I change one character in the manifest"#, given_change_one_char_in_manifest),
        step(r#"I verify the signature against the canonical manifest bytes"#, when_verify_signature_now),
        step(r#"I verify the signature"#, when_verify_signature_now),
        step(r#"I canonicalize and verify"#, when_canonicalize_and_verify),
        step(r#"the verification result is \"(ok|error)\""#, then_verification_result),
    ]
}

fn given_change_one_char_in_manifest(state: &mut State, _caps: &regex::Captures) -> Result<()> {
    // Load current manifest, mutate a single character in a stable field, write to temp, update state and canonical
    let mp = state.get("manifest_path").ok_or_else(|| anyhow!("manifest_path not set"))?;
    let txt = fs::read_to_string(&PathBuf::from(&mp))?;
    let mut val: serde_json::Value = serde_json::from_str(&txt)?;
    if !mutate_one_char(&mut val) {
        return Err(anyhow!("could not find a string field to mutate in manifest"));
    }
    let tmp = temp_path("manifest-onechar.json");
    fs::write(&tmp, serde_json::to_string_pretty(&val)? + "\n")?;
    state.set("manifest_path", tmp.to_string_lossy());
    // Recompute canonical immediately for subsequent verify steps that rely on canonical_b64
    let canonical = manifest_contract::canonicalize(&val);
    state.set("canonical_b64", base64::engine::general_purpose::STANDARD.encode(&canonical));
    Ok(())
}

fn mutate_one_char(v: &mut serde_json::Value) -> bool {
    // Prefer the top-level "commit" string if present; else mutate the first string we can find.
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
            serde_json::Value::String(s) => {
                let new = flip_first_char(s);
                *s = new;
                true
            }
            serde_json::Value::Array(arr) => {
                for el in arr.iter_mut() { if recurse(el) { return true; } }
                false
            }
            serde_json::Value::Object(map) => {
                for (_k, v) in map.iter_mut() { if recurse(v) { return true; } }
                false
            }
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

fn step(pat: &str, f: fn(&mut State, &regex::Captures) -> Result<()>) -> Step { Step::new(pat, f).unwrap() }

fn repo_root(state: &State) -> PathBuf {
    let base = state.get("repo_root").unwrap_or_else(|| state.get("base_dir").unwrap());
    PathBuf::from(base)
}

fn given_canonicalized_manifest_at(state: &mut State, caps: &regex::Captures) -> Result<()> {
    let rel = caps.get(1).unwrap().as_str();
    let path = repo_root(state).join(rel);
    let txt = fs::read_to_string(&path)?;
    let val: serde_json::Value = serde_json::from_str(&txt)?;
    let canonical = manifest_contract::canonicalize(&val);
    state.set("canonical_b64", base64::engine::general_purpose::STANDARD.encode(&canonical));
    state.set("manifest_path", path.to_string_lossy());
    Ok(())
}

fn given_signature_at(state: &mut State, caps: &regex::Captures) -> Result<()> {
    let rel = caps.get(1).unwrap().as_str();
    let path = repo_root(state).join(rel);
    let sig_b64 = fs::read_to_string(&path)?;
    state.set("sig_b64", sig_b64.trim());
    Ok(())
}

fn given_index_pubkey(state: &mut State, _caps: &regex::Captures) -> Result<()> {
    let pk_path = repo_root(state).join("examples/minimal/.provenance/public_test_ed25519.key.b64");
    let pk_b64 = fs::read_to_string(&pk_path)?;
    state.set("pubkey", pk_b64.trim());
    Ok(())
}

fn given_different_pubkey(state: &mut State, _caps: &regex::Captures) -> Result<()> {
    let pk_path = repo_root(state).join("examples/minimal/.provenance/public_test_ed25519.key.b64");
    let mut pk_b64 = fs::read_to_string(&pk_path)?;
    let mut chars: Vec<char> = pk_b64.trim().chars().collect();
    if let Some(first) = chars.get_mut(0) { *first = if *first == 'A' { 'B' } else { 'A' }; }
    let wrong = chars.into_iter().collect::<String>();
    state.set("pubkey", wrong);
    Ok(())
}

fn when_verify_signature_now(state: &mut State, _caps: &regex::Captures) -> Result<()> {
    let can_b64 = state.get("canonical_b64").ok_or_else(|| anyhow!("canonical not set"))?;
    let sig_b64 = state.get("sig_b64").ok_or_else(|| anyhow!("sig not set"))?;
    let pubkey = state.get("pubkey").ok_or_else(|| anyhow!("pubkey not set"))?;
    let can = base64::engine::general_purpose::STANDARD.decode(can_b64).unwrap();
    match manifest_contract::ed25519_verify(&can, &sig_b64, &pubkey) {
        Ok(ok) => {
            state.set("verify_ok", if ok { "true" } else { "false" });
            if ok {
                state.set("last_err", "");
            } else {
                // Provide a stable error string for feature assertions
                state.set("last_err", "signature mismatch");
            }
        }
        Err(e) => { state.set("verify_ok", "false"); state.set("last_err", format!("{}", e)); }
    }
    Ok(())
}

fn when_canonicalize_and_verify(state: &mut State, _caps: &regex::Captures) -> Result<()> {
    // Recompute canonical from the (possibly modified) manifest_path, then verify
    let mp = state.get("manifest_path").ok_or_else(|| anyhow!("manifest_path not set"))?;
    let txt = fs::read_to_string(&PathBuf::from(mp))?;
    let val: serde_json::Value = serde_json::from_str(&txt)?;
    let canonical = manifest_contract::canonicalize(&val);
    state.set("canonical_b64", base64::engine::general_purpose::STANDARD.encode(&canonical));
    when_verify_signature_now(state, &regex::Regex::new("").unwrap().captures("").unwrap())
}

fn then_verification_result(state: &mut State, caps: &regex::Captures) -> Result<()> {
    let expect = caps.get(1).unwrap().as_str();
    let ok = state.get("verify_ok").unwrap_or_else(|| "false".into());
    match expect {
        "ok" => if ok == "true" { Ok(()) } else { Err(anyhow!("expected ok, got false ({} )", state.get("last_err").unwrap_or_default())) },
        "error" => if ok == "false" { Ok(()) } else { Err(anyhow!("expected error, got ok")) },
        _ => Err(anyhow!("unknown expectation"))
    }
}
