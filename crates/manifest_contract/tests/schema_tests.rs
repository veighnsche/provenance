use anyhow::Result;
use serde_json::Value;

#[test]
fn schema_success_on_example_manifest() -> Result<()> {
    let crate_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let manifest_path = crate_dir.join("../../examples/minimal/.provenance/manifest.json");
    let schema_path = crate_dir.join("../../schemas/manifest.schema.json");

    let txt = std::fs::read_to_string(&manifest_path)?;
    let val: Value = serde_json::from_str(&txt)?;
    let schema_txt = std::fs::read_to_string(&schema_path)?;

    manifest_contract::validate_schema(&val, &schema_txt)?;
    Ok(())
}

#[test]
fn schema_failure_on_missing_artifacts() {
    let bad = serde_json::json!({
        "version": 1,
        "repo": "acme/provenance",
        "commit": "0123457",
        "workflow_run": {"id": 1, "url": "https://example.com/run/1", "attempt": 1},
        "front_page": {"title": "T", "markup": "ci/front_page.pml"}
    });
    let schema_txt = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../schemas/manifest.schema.json")
    ).expect("schema present");
    let err = manifest_contract::validate_schema(&bad, &schema_txt).expect_err("must fail");
    let msg = format!("{}", err);
    assert!(msg.contains("schema validation failed"));
}
