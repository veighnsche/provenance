use renderers::*;

#[test]
fn markdown_is_sanitized_and_renders_basic() {
    let md = "<script>alert(1)</script>**bold**";
    let html = render_markdown(md);
    assert!(html.contains("bold"));
    assert!(!html.contains("<script>"));
}

#[test]
fn json_pretty_escapes_and_wraps_in_pre() {
    let v = serde_json::json!({"a":"<b>"});
    let html = render_json_pretty(serde_json::to_string(&v).unwrap().as_bytes()).unwrap();
    assert!(html.starts_with("<pre>"));
    assert!(html.contains("&lt;b&gt;"));
}

#[test]
fn coverage_is_sorted_and_structured() {
    let cov = serde_json::json!({
        "total": {"pct": 83.3},
        "files": [
            {"path":"z.rs","pct":50.0},
            {"path":"a.rs","pct":100.0}
        ]
    });
    let html = render_coverage(serde_json::to_string(&cov).unwrap().as_bytes()).unwrap();
    let a_idx = html.find("a.rs").unwrap();
    let z_idx = html.find("z.rs").unwrap();
    assert!(a_idx < z_idx, "files not sorted by path: {}", html);
    assert!(html.contains("<thead><tr><th scope=\"col\">Path</th><th scope=\"col\">Coverage</th></tr></thead>"));
}

#[test]
fn tests_summary_cards_have_expected_counts() {
    let s = serde_json::json!({"total":10,"passed":9,"failed":1,"duration_seconds":1.2});
    let html = render_tests_summary(serde_json::to_string(&s).unwrap().as_bytes()).unwrap();
    assert!(html.contains("<strong>10</strong>"));
    assert!(html.contains("<strong>9</strong>"));
    assert!(html.contains("<strong>1</strong>"));
}
