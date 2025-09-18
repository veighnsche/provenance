use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct Artifact<'a> {
    pub id: &'a str,
    pub title: &'a str,
    pub render: &'a str,
    pub media_type: &'a str,
    pub verified: bool,
    pub download_href: &'a str,
}

fn badge(verified: bool) -> &'static str {
    if verified { "<span class=\"badge ok\">verified</span>" } else { "<span class=\"badge err\">digest mismatch</span>" }
}

fn esc(s: &str) -> String { s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;") }

pub fn render_index<'a>(title: &str, commit: &str, kpis: &BTreeMap<&str, String>, featured: &[Artifact<'a>]) -> String {
    let mut out = String::new();
    out.push_str(&format!("<header><h1>{} — {}</h1></header>", esc(title), esc(commit)));
    if !kpis.is_empty() {
        out.push_str("<div class=\"cards\">");
        for (k, v) in kpis.iter() {
            out.push_str(&format!("<div class=\"card\"><h3>{}</h3><p><strong>{}</strong></p></div>", esc(k), esc(v)));
        }
        out.push_str("</div>");
    }
    out.push_str("<h2>Artifacts</h2><div class=\"cards\">");
    for a in featured.iter() {
        out.push_str(&format!(
            "<div class=\"card\"><h3>{}</h3><p class=\"muted\">{}</p><p>{}</p><p><a href=\"/a/{}/\">View</a> \u{00b7} <a href=\"{}\">Download</a></p></div>",
            esc(a.title), esc(a.id), badge(a.verified), esc(a.id), esc(a.download_href)
        ));
    }
    out.push_str("</div>");
    out
}

pub fn render_artifact<'a>(a: &Artifact<'a>, body_html: &str) -> String {
    let mut out = String::new();
    out.push_str(&format!("<header><nav aria-label=\"Breadcrumb\"><a href=\"/index.html\">Home</a> / <span>{}</span></nav><h1>{}</h1><p class=\"muted\">{}</p></header>", esc(a.title), esc(a.title), esc(a.id)));
    out.push_str(&format!("<p>{}</p>", badge(a.verified)));
    out.push_str(&format!("<article>{}</article>", body_html));
    out.push_str(&format!("<p><a href=\"{}\">Download raw</a></p>", esc(a.download_href)));
    out
}

pub fn render_artifacts_index<'a>(items: &[Artifact<'a>]) -> String {
    let mut out = String::new();
    out.push_str("<header><h1>All Artifacts</h1></header>");
    out.push_str("<table class=\"table\"><thead><tr><th scope=\"col\">ID</th><th scope=\"col\">Title</th><th scope=\"col\">Render</th><th scope=\"col\">Media</th><th scope=\"col\">Verified</th></tr></thead><tbody>");
    for a in items.iter() {
        out.push_str(&format!(
            "<tr><th scope=\"row\"><a href=\"/a/{}/\">{}</a></th><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            esc(a.id), esc(a.id), esc(a.title), esc(a.render), esc(a.media_type), badge(a.verified)
        ));
    }
    out.push_str("</tbody></table>");
    out
}
