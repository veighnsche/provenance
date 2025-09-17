use anyhow::Result;
use serde_json::Value;

#[test]
fn canonicalization_is_stable() -> Result<()> {
    let crate_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let manifest_path = crate_dir.join("../../examples/minimal/.provenance/manifest.json");
    let txt = std::fs::read_to_string(&manifest_path)?;
    let val: Value = serde_json::from_str(&txt)?;

    let a = manifest_contract::canonicalize(&val);
    let b = manifest_contract::canonicalize(&val);
    assert_eq!(a, b, "canonical bytes must be byte-for-byte identical across calls");

    let reparsed: Value = serde_json::from_slice(&a)?;
    let c = manifest_contract::canonicalize(&reparsed);
    assert_eq!(a, c, "canonicalization must be idempotent under parse->canonicalize");

    Ok(())
}
