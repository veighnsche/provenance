use dioxus::prelude::*;
use serde::Deserialize;
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

pub fn page_base(children: Element) -> String {
    let html = dioxus_ssr::render_lazy(rsx! {
        html { lang: "en",
            head { 
                meta { name: "viewport", content: "width=device-width, initial-scale=1.0" }
                style { dangerous_inner_html: "body{font-family:system-ui,-apple-system,Segoe UI,Roboto,Ubuntu,Cantarell,Noto Sans,sans-serif;margin:0;padding:0;color:#111} .container{max-width:1040px;margin:0 auto;padding:24px} header{padding:16px 0;border-bottom:1px solid #eee;margin-bottom:24px} .cards{display:grid;grid-template-columns:repeat(auto-fit,minmax(280px,1fr));gap:16px} .card{border:1px solid #eee;border-radius:8px;padding:16px;background:#fff} table{width:100%;border-collapse:collapse} th,td{padding:8px;border-bottom:1px solid #eee;text-align:left} .badge{display:inline-block;padding:2px 8px;border-radius:6px;font-size:12px;color:#fff} .ok{background:#28a745} .warn{background:#ff9800} .err{background:#d32f2f} code, pre{background:#f7f7f7;border-radius:6px;padding:2px 6px} pre{padding:12px;overflow-x:auto} .muted{color:#777;font-size:14px}" }
            }
            body { 
                div { class: "container", {children} }
            }
        }
    });
    html
}

pub fn index_page(
    title: &str,
    commit: &str,
    kpis: BTreeMap<&str, String>,
    featured: Vec<ArtifactView>,
) -> String {
    let html = dioxus_ssr::render_lazy(rsx! {
        div {
            header { h1 { "{} — {}", title, commit } }
            if !kpis.is_empty() {
                div { class: "cards",
                    for (k, v) in kpis.iter() {
                        div { class: "card", h3 { "{}", *k }, p { strong { "{}", v } } }
                    }
                }
            }
            h2 { "Artifacts" }
            div { class: "cards",
                for a in featured.iter() {
                    div { class: "card",
                        h3 { "{}", a.title }
                        p { class: "muted", "{}", a.id }
                        p { if a.verified { span { class: "badge ok", "verified" } } else { span { class: "badge err", "digest mismatch" } } }
                        p { a { href: format!("/a/{}/", a.id), "View" } " · " a { href: a.download_href.clone(), "Download" } }
                    }
                }
            }
        }
    });
    page_base(html)
}

pub fn artifact_page(a: &ArtifactView, body_html: &str) -> String {
    let html = dioxus_ssr::render_lazy(rsx! {
        div {
            header { h1 { "{}", a.title } p { class: "muted", "{}", a.id } }
            p { if a.verified { span { class: "badge ok", "verified" } } else { span { class: "badge err", "digest mismatch" } } }
            article { dangerous_inner_html: body_html.to_string() }
            p { a { href: a.download_href.clone(), "Download raw" } }
        }
    });
    page_base(html)
}

pub fn render_tests_summary(json_bytes: &[u8]) -> anyhow::Result<String> {
    let s: TestSummary = serde_json::from_slice(json_bytes)?;
    let html = dioxus_ssr::render_lazy(rsx! {
        div { class: "cards",
            div { class: "card", h3 { "Total" } p { strong { "{}", s.total } } }
            div { class: "card", h3 { "Passed" } p { strong { "{}", s.passed } } }
            div { class: "card", h3 { "Failed" } p { strong { "{}", s.failed } } }
            div { class: "card", h3 { "Duration" } p { strong { "{:.2}s", s.duration_seconds } } }
        }
    });
    Ok(html)
}

pub fn render_coverage(json_bytes: &[u8]) -> anyhow::Result<String> {
    let c: Coverage = serde_json::from_slice(json_bytes)?;
    let total = c.total.as_ref().map(|t| t.pct).unwrap_or(0.0);
    let mut rows = String::new();
    if let Some(files) = c.files {
        for f in files {
            rows.push_str(&format!("<tr><td>{}</td><td>{:.1}%</td></tr>", html_escape(&f.path), f.pct));
        }
    }
    let html = format!(
        "<div class=\"card\"><h3>Total</h3><p><strong>{:.1}%</strong></p></div><h3>Files</h3><table><thead><tr><th>Path</th><th>Coverage</th></tr></thead><tbody>{}</tbody></table>",
        total, rows
    );
    Ok(html)
}

pub fn render_markdown(md: &str) -> String {
    use pulldown_cmark::{html, Options, Parser};
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    let parser = Parser::new_ext(md, options);
    let mut out = String::new();
    html::push_html(&mut out, parser);
    out
}

pub fn render_json_pretty(bytes: &[u8]) -> anyhow::Result<String> {
    let val: serde_json::Value = serde_json::from_slice(bytes)?;
    Ok(format!("<pre>{}</pre>", html_escape(&serde_json::to_string_pretty(&val)?)))
}

pub fn render_image(src_href: &str, alt: &str) -> String {
    format!("<img src=\"{}\" alt=\"{}\" style=\"max-width:100%;height:auto\" />", src_href, html_escape(alt))
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}
