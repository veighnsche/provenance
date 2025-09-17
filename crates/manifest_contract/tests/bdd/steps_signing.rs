use bdd_harness::{Step, State};
use anyhow::{anyhow, Result};
use std::fs;
use std::path::PathBuf;

pub fn registry() -> Vec<Step> {
    vec![
        step(r#"a canonicalized manifest at "([^"]+)""#, given_canonicalized_manifest_at),
        step(r#"a Base64 signature at "([^"]+)""#, given_signature_at),
        step(r#"the public key INDEX_PUBKEY_ED25519 for the manifest"#, given_index_pubkey),
        step(r#"a different public key"#, given_different_pubkey),
        step(r#"I verify the signature against the canonical manifest bytes"#, when_verify_signature_now),
        step(r#"I verify the signature"#, when_verify_signature_now),
        step(r#"I canonicalize and verify"#, when_verify_signature_now),
        step(r#"the verification result is "(ok|error)""#, then_verification_result),
    ]
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
        Ok(ok) => { state.set("verify_ok", if ok { "true" } else { "false" }); state.set("last_err", ""); }
        Err(e) => { state.set("verify_ok", "false"); state.set("last_err", format!("{}", e)); }
    }
    Ok(())
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
