use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct ArtifactView<'a> {
    pub id: &'a str,
    pub title: &'a str,
    pub render: &'a str,
    pub media_type: &'a str,
    pub path_rel: &'a str,
    pub verified: bool,
    pub download_href: String,
}

pub use renderers::{Coverage, TestSummary};

pub fn page_base(inner_html: String) -> String {
    format!(
        "<!doctype html><html lang=\"en\"><head><meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\"><link rel=\"stylesheet\" href=\"/assets/site.css\"></head><body><a href=\"#main\" class=\"skip-link\">Skip to content</a><div class=\"container\"><header class=\"topbar\" role=\"banner\"><nav aria-label=\"Global\" class=\"global-nav\"><a href=\"/index.html\">Home</a><a href=\"/artifacts/\">Artifacts</a><a href=\"/badge/\">Badges</a><a href=\"/artifacts/#search\">Search</a></nav></header><main id=\"main\" role=\"main\">{}</main></div></body></html>",
        inner_html
    )
}

pub fn index_page(
    title: &str,
    commit: &str,
    kpis: BTreeMap<&str, String>,
    featured: Vec<ArtifactView>,
) -> String {
    let mut inner = String::new();
    inner.push_str(&format!("<header><h1>{} — {}</h1></header>", html_escape(title), html_escape(commit)));
    if !kpis.is_empty() {
        inner.push_str("<div class=\"cards\">");
        for (k, v) in kpis.iter() {
            inner.push_str(&format!("<div class=\"card\"><h3>{}</h3><p><strong>{}</strong></p></div>", html_escape(k), html_escape(v)));
        }
        inner.push_str("</div>");
    }
    inner.push_str("<h2>Artifacts</h2><div class=\"cards\">");
    for a in featured.iter() {
        inner.push_str(&format!(
            "<div class=\"card\"><h3>{}</h3><p class=\"muted\">{}</p><p>{}</p><p><a href=\"/a/{}/\">View</a> · <a href=\"{}\">Download</a></p></div>",
            html_escape(a.title),
            html_escape(a.id),
            if a.verified { "<span class=\"badge ok\">verified</span>".to_string() } else { "<span class=\"badge err\">digest mismatch</span>".to_string() },
            html_escape(a.id),
            html_escape(&a.download_href)
        ));
    }
    inner.push_str("</div>");
    page_base(inner)
}

pub fn artifact_page(a: &ArtifactView, body_html: &str) -> String {
    let mut inner = String::new();
    inner.push_str(&format!("<header><nav aria-label=\"Breadcrumb\"><a href=\"/index.html\">Home</a> / <span>{}</span></nav><h1>{}</h1><p class=\"muted\">{}</p></header>", html_escape(a.title), html_escape(a.title), html_escape(a.id)));
    inner.push_str(&format!("<p>{}</p>", if a.verified { "<span class=\"badge ok\">verified</span>" } else { "<span class=\"badge err\">digest mismatch</span>" }));
    inner.push_str(&format!("<article>{}</article>", body_html));
    inner.push_str(&format!("<p><a href=\"{}\">Download raw</a></p>", html_escape(&a.download_href)));
    page_base(inner)
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

pub use renderers::{render_coverage, render_image, render_json_pretty, render_markdown, render_tests_summary};

/// Site CSS (extracted from previous inline style), plus minimal layout for top bar and sidebars.
pub fn site_css() -> &'static str {
    r#"body{font-family:system-ui,-apple-system,Segoe UI,Roboto,Ubuntu,Cantarell,Noto Sans,sans-serif;margin:0;padding:0;color:#111;background:#fafafa}
    a{color:inherit;text-decoration:underline}
    .container{max-width:1040px;margin:0 auto;padding:24px}
    .topbar{padding:8px 0 16px 0;border-bottom:1px solid #eee;margin-bottom:16px}
    .global-nav{display:flex;gap:16px;flex-wrap:wrap}
    .cards{display:grid;grid-template-columns:repeat(auto-fit,minmax(280px,1fr));gap:16px}
    .card{border:1px solid #eee;border-radius:8px;padding:16px;background:#fff}
    header.page{padding:16px 0;border-bottom:1px solid #eee;margin-bottom:24px}
    table{width:100%;border-collapse:collapse}
    thead th{scope:col}
    th,td{padding:8px;border-bottom:1px solid #eee;text-align:left}
    .badge{display:inline-block;padding:2px 8px;border-radius:6px;font-size:12px;color:#fff}
    .ok{background:#28a745}
    .warn{background:#ff9800}
    .err{background:#d32f2f}
    code, pre{background:#f7f7f7;border-radius:6px;padding:2px 6px}
    pre{padding:12px;overflow-x:auto}
    .muted{color:#777;font-size:14px}
    .skip-link{position:absolute;left:-10000px;top:auto;width:1px;height:1px;overflow:hidden}
    .skip-link:focus{position:static;width:auto;height:auto}
    /* Content layout with optional left/right sidebars */
    .content-grid{display:grid;gap:16px;grid-template-columns:220px 1fr 260px;align-items:start}
    .left-nav{position:sticky;top:12px}
    .right-meta{position:sticky;top:12px}
    @media (max-width: 1024px){.content-grid{grid-template-columns:1fr}.left-nav,.right-meta{position:relative;top:auto}}
    /* Filters/search form */
    form.filters{display:grid;gap:12px;margin:16px 0}
    form.filters .row{display:grid;grid-template-columns:160px 1fr;gap:8px;align-items:center}
    input[type=search],select{padding:8px;border:1px solid #ddd;border-radius:6px;background:#fff}
    button{padding:8px 12px;border:1px solid #ddd;border-radius:6px;background:#f7f7f7;cursor:pointer}
    button:hover{background:#eee}
    dl.meta{display:grid;grid-template-columns:120px 1fr;gap:8px}
    dl.meta dt{color:#555}
    dl.meta dd{margin:0}
    "#
}
