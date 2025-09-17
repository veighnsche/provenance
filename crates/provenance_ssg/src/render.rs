use dioxus::prelude::*;
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

pub fn page_base(children: Element) -> String {
    let html = dioxus_ssr::render(rsx! {
        html { lang: "en",
            head { 
                meta { name: "viewport", content: "width=device-width, initial-scale=1.0" }
                style { dangerous_inner_html: "body{font-family:system-ui,-apple-system,Segoe UI,Roboto,Ubuntu,Cantarell,Noto Sans,sans-serif;margin:0;padding:0;color:#111} .container{max-width:1040px;margin:0 auto;padding:24px} header{padding:16px 0;border-bottom:1px solid #eee;margin-bottom:24px} .cards{display:grid;grid-template-columns:repeat(auto-fit,minmax(280px,1fr));gap:16px} .card{border:1px solid #eee;border-radius:8px;padding:16px;background:#fff} table{width:100%;border-collapse:collapse} thead th{scope:col} th,td{padding:8px;border-bottom:1px solid #eee;text-align:left} .badge{display:inline-block;padding:2px 8px;border-radius:6px;font-size:12px;color:#fff} .ok{background:#28a745} .warn{background:#ff9800} .err{background:#d32f2f} code, pre{background:#f7f7f7;border-radius:6px;padding:2px 6px} pre{padding:12px;overflow-x:auto} .muted{color:#777;font-size:14px}" }
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
    let html = dioxus_ssr::render(rsx! {
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
    let html = dioxus_ssr::render(rsx! {
        div {
            header { h1 { "{}", a.title } p { class: "muted", "{}", a.id } }
            p { if a.verified { span { class: "badge ok", "verified" } } else { span { class: "badge err", "digest mismatch" } } }
            article { dangerous_inner_html: body_html.to_string() }
            p { a { href: a.download_href.clone(), "Download raw" } }
        }
    });
    page_base(html)
}

pub use renderers::{render_coverage, render_image, render_json_pretty, render_markdown, render_tests_summary};
