use jsonschema::JSONSchema;
use serde_json::json;

#[test]
fn provenance_badge_conforms_to_schema() {
    let schema = load_schema();
    let b = badges::badge_provenance(true);
    let v = serde_json::to_value(&b).unwrap();
    validate(&schema, &v);

    let b2 = badges::badge_provenance(false);
    let v2 = serde_json::to_value(&b2).unwrap();
    validate(&schema, &v2);
}

#[test]
fn tests_badge_conforms_to_schema() {
    let schema = load_schema();
    let s = badges::TestSummary { total: 10, passed: 9, failed: 1, duration_seconds: 1.23 };
    let b = badges::badge_tests(&s);
    let v = serde_json::to_value(&b).unwrap();
    validate(&schema, &v);
}

#[test]
fn coverage_badge_conforms_to_schema() {
    let schema = load_schema();
    let c = badges::Coverage { total: Some(badges::CoverageTotal { pct: 92.0 }) };
    let b = badges::badge_coverage(&c);
    let v = serde_json::to_value(&b).unwrap();
    validate(&schema, &v);
}

fn load_schema() -> JSONSchema {
    let crate_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = crate_dir.parent().and_then(|p| p.parent()).unwrap().to_path_buf();
    let schema_path = repo_root.join("schemas/badge.schema.json");
    let schema_txt = std::fs::read_to_string(&schema_path).expect("read schema");
    let schema_val: serde_json::Value = serde_json::from_str(&schema_txt).expect("parse schema");
    JSONSchema::compile(&schema_val).expect("compile schema")
}

fn validate(schema: &JSONSchema, v: &serde_json::Value) {
    if let Err(errors) = schema.validate(v) {
        let msgs: Vec<String> = errors.map(|e| format!("{} at {}", e, e.instance_path)).collect();
        panic!("schema validation failed for badge: {}\n{}", v, msgs.join("\n"));
    }
}
