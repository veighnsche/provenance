use manifest_contract as mc;
use serde_json::json;

#[test]
fn canonicalization_sorts_keys_recursively() {
    let input = json!({
        "commit": "8c6a9f4e",
        "version": 1,
        "repo": "acme/provenance",
        "front_page": { "markup": "ci/front_page.pml", "title": "QA Evidence" },
        "workflow_run": { "attempt": 1, "url": "https://github.com/...", "id": 123 },
        "artifacts": [
            { "sha256": "00...ff", "id": "tests-summary", "title": "Tests", "render": "summary:test", "media_type": "application/json", "path": "ci/tests/summary.json" }
        ]
    });
    let canon = mc::canonicalize(&input);
    let canon_text = String::from_utf8(canon).unwrap();
    let expected = "{\"artifacts\":[{\"id\":\"tests-summary\",\"media_type\":\"application/json\",\"path\":\"ci/tests/summary.json\",\"render\":\"summary:test\",\"sha256\":\"00...ff\",\"title\":\"Tests\"}],\"commit\":\"8c6a9f4e\",\"front_page\":{\"markup\":\"ci/front_page.pml\",\"title\":\"QA Evidence\"},\"repo\":\"acme/provenance\",\"version\":1,\"workflow_run\":{\"attempt\":1,\"id\":123,\"url\":\"https://github.com/...\"}}";
    assert_eq!(canon_text, expected);
}
