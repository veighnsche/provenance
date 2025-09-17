use anyhow::Result;
use serde_json::Value;

#[test]
fn signature_verifies_for_example_manifest() -> Result<()> {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let manifest_path = root.join("../../examples/minimal/.provenance/manifest.json");
    let sig_path = root.join("../../examples/minimal/.provenance/manifest.json.sig");
    let pubkey_path = root.join("../../examples/minimal/.provenance/public_test_ed25519.key.b64");

    let txt = std::fs::read_to_string(&manifest_path)?;
    let val: Value = serde_json::from_str(&txt)?;
    let canonical = manifest_contract::canonicalize(&val);

    let sig_b64 = std::fs::read_to_string(&sig_path)?;
    let pk_b64 = std::fs::read_to_string(&pubkey_path)?;

    let ok = manifest_contract::ed25519_verify(&canonical, sig_b64.trim(), pk_b64.trim())?;
    assert!(ok, "signature must verify for example manifest");
    Ok(())
}
