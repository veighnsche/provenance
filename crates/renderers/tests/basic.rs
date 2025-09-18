use renderers::*;

#[test]
fn markdown_renders_basic() {
    let md = "# Title\n\nHello <world> & friends.";
    let html = render_markdown(md);
    assert!(html.contains("<h1>Title</h1>"));
    // Output is sanitized: unknown tags are removed/escaped; ampersand escaped; base text present
    assert!(html.contains("Hello "));
    assert!(html.contains("&amp; friends."));
    assert!(!html.contains("<world>"));
}

#[test]
fn json_pretty_escapes_and_formats() {
    let val = serde_json::json!({"a": "<>&"});
    let bytes = serde_json::to_vec(&val).unwrap();
    let html = render_json_pretty(&bytes).unwrap();
    assert!(html.contains("&lt;&gt;&amp;"));
    assert!(html.starts_with("<pre>{"));
}

#[test]
fn tests_summary_card_has_kpis() {
    let summary = serde_json::json!({
        "total": 10,
        "passed": 9,
        "failed": 1,
        "duration_seconds": 12.34
    });
    let html = render_tests_summary(serde_json::to_string(&summary).unwrap().as_bytes()).unwrap();
    assert!(html.contains("Total"));
    assert!(html.contains("Passed"));
    assert!(html.contains("Failed"));
}

#[test]
fn coverage_table_has_rows() {
    let cov = serde_json::json!({
        "total": {"pct": 80.5},
        "files": [
            {"path":"src/lib.rs","pct": 92.1},
            {"path":"src/main.rs","pct": 70.0}
        ]
    });
    let html = render_coverage(serde_json::to_string(&cov).unwrap().as_bytes()).unwrap();
    assert!(html.contains("Total"));
    assert!(html.contains("src/lib.rs"));
    assert!(html.contains("92.1%"));
}
