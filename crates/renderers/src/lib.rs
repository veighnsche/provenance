use serde::Deserialize;
use ammonia::Builder as HtmlSanitizer;

#[derive(Debug, Deserialize)]
pub struct TestSummary {
    pub total: u64,
    pub passed: u64,
    pub failed: u64,
    pub duration_seconds: f64,
}

#[derive(Debug, Deserialize)]
pub struct CoverageTotal { pub pct: f64 }
#[derive(Debug, Deserialize)]
pub struct CoverageFile { pub path: String, pub pct: f64 }
#[derive(Debug, Deserialize)]
pub struct Coverage {
    pub total: Option<CoverageTotal>,
    pub files: Option<Vec<CoverageFile>>,
}

pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

pub fn render_markdown(md: &str) -> String {
    use pulldown_cmark::{html, Options, Parser};
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    let parser = Parser::new_ext(md, options);
    let mut out = String::new();
    html::push_html(&mut out, parser);
    // Sanitize potentially dangerous HTML (e.g., raw <script> in markdown)
    let cleaner = HtmlSanitizer::default();
    cleaner.clean(&out).to_string()
}

pub fn render_json_pretty(bytes: &[u8]) -> anyhow::Result<String> {
    let val: serde_json::Value = serde_json::from_slice(bytes)?;
    Ok(format!("<pre>{}</pre>", html_escape(&serde_json::to_string_pretty(&val)?)))
}

pub fn render_image(src_href: &str, alt: &str) -> String {
    format!("<img src=\"{}\" alt=\"{}\" style=\"max-width:100%;height:auto\" />", src_href, html_escape(alt))
}

pub fn render_tests_summary(json_bytes: &[u8]) -> anyhow::Result<String> {
    let s: TestSummary = serde_json::from_slice(json_bytes)?;
    let html = format!(
        "<div class=\"cards\">\
            <div class=\"card\"><h3>Total</h3><p><strong>{}</strong></p></div>\
            <div class=\"card\"><h3>Passed</h3><p><strong>{}</strong></p></div>\
            <div class=\"card\"><h3>Failed</h3><p><strong>{}</strong></p></div>\
            <div class=\"card\"><h3>Duration</h3><p><strong>{:.2}s</strong></p></div>\
        </div>",
        s.total, s.passed, s.failed, s.duration_seconds
    );
    Ok(html)
}

pub fn render_coverage(json_bytes: &[u8]) -> anyhow::Result<String> {
    let c: Coverage = serde_json::from_slice(json_bytes)?;
    let total = c.total.as_ref().map(|t| t.pct).unwrap_or(0.0);
    let mut rows = String::new();
    if let Some(mut files) = c.files {
        files.sort_by(|a, b| a.path.cmp(&b.path));
        for f in files {
            rows.push_str(&format!("<tr><td>{}</td><td>{:.1}%</td></tr>", html_escape(&f.path), f.pct));
        }
    }
    let html = format!(
        "<div class=\"card\"><h3>Total</h3><p><strong>{:.1}%</strong></p></div><h3>Files</h3><table><thead><tr><th scope=\"col\">Path</th><th scope=\"col\">Coverage</th></tr></thead><tbody>{}</tbody></table>",
        total, rows
    );
    Ok(html)
}
