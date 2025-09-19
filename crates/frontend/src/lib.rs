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
    out.push_str(&format!("<header class=\"page\"><nav aria-label=\"Breadcrumb\"><a href=\"/index.html\">Home</a> / <a href=\"/artifacts/\">Artifacts</a> / <span>{}</span></nav><h1>{}</h1><p class=\"muted\">{}</p></header>", esc(a.title), esc(a.title), esc(a.id)));
    // Content grid with left nav and right metadata
    out.push_str("<div class=\"content-grid\">");
    // Left navigation
    out.push_str("<aside class=\"left-nav\" aria-label=\"Page navigation\">");
    out.push_str("<nav><ul>");
    out.push_str("<li><a href=\"/artifacts/\">← All artifacts</a></li>");
    out.push_str("</ul></nav>");
    out.push_str("</aside>");
    // Main article
    out.push_str("<section>");
    out.push_str(&format!("<p>{}</p>", badge(a.verified)));
    out.push_str(&format!("<article>{}</article>", body_html));
    out.push_str(&format!("<p><a href=\"{}\">Download raw</a></p>", esc(a.download_href)));
    out.push_str("</section>");
    // Right metadata
    out.push_str("<aside class=\"right-meta\" aria-label=\"Metadata\">");
    out.push_str("<div class=\"card\"><h3>Metadata</h3>");
    out.push_str("<dl class=\"meta\">");
    out.push_str(&format!("<dt>ID</dt><dd>{}</dd>", esc(a.id)));
    out.push_str(&format!("<dt>Title</dt><dd>{}</dd>", esc(a.title)));
    out.push_str(&format!("<dt>Media</dt><dd>{}</dd>", esc(a.media_type)));
    out.push_str(&format!("<dt>Status</dt><dd>{}</dd>", if a.verified { "verified" } else { "digest mismatch" }));
    out.push_str(&format!("<dt>Download</dt><dd><a href=\"{}\">file</a></dd>", esc(a.download_href)));
    out.push_str("</dl></div>");
    out.push_str("</aside>");
    // Close grid
    out.push_str("</div>");
    out
}

pub fn render_artifacts_index<'a>(items: &[Artifact<'a>]) -> String {
    let mut out = String::new();
    out.push_str("<header class=\"page\"><h1>All Artifacts</h1></header>");
    // Layout with left sidebar filters and main results
    out.push_str("<div class=\"content-grid\">");
    // Left filters sidebar with GET form and #search anchor
    out.push_str("<aside class=\"left-nav\" aria-label=\"Filters\">");
    out.push_str("<a id=\"search\"></a>");
    out.push_str("<form class=\"filters\" method=\"get\" action=\"/artifacts/index.html\">");
    out.push_str("<div class=\"row\"><label for=\"q\">Search</label><input id=\"q\" name=\"q\" type=\"search\" placeholder=\"id, title, kind\" autofocus></div>");
    out.push_str("<div class=\"row\"><label for=\"kind\">Kind</label><select id=\"kind\" name=\"kind\"><option value=\"\">Any</option><option>summary:test</option><option>table:coverage</option><option>markdown</option><option>json</option><option>image</option></select></div>");
    out.push_str("<div class=\"row\"><label for=\"verified\">Verified</label><select id=\"verified\" name=\"verified\"><option value=\"\">Any</option><option value=\"true\">Verified</option><option value=\"false\">Error</option></select></div>");
    out.push_str("<div class=\"row\"><label for=\"media\">Media</label><select id=\"media\" name=\"media\"><option value=\"\">Any</option><option>application/json</option><option>text/markdown</option><option>image/*</option><option>text/*</option></select></div>");
    out.push_str("<div class=\"row\"><label for=\"sort\">Sort by</label><select id=\"sort\" name=\"sort\"><option value=\"id\">ID</option><option value=\"title\">Title</option><option value=\"render\">Kind</option></select></div>");
    out.push_str("<div class=\"row\"><span></span><button type=\"submit\">Apply</button></div>");
    out.push_str("</form>");
    out.push_str("</aside>");
    // Main results table
    out.push_str("<section>");
    out.push_str("<table class=\"table\"><thead><tr><th scope=\"col\">ID</th><th scope=\"col\">Title</th><th scope=\"col\">Render</th><th scope=\"col\">Media</th><th scope=\"col\">Verified</th></tr></thead><tbody>");
    for a in items.iter() {
        out.push_str(&format!(
            "<tr><th scope=\"row\"><a href=\"/a/{}/\">{}</a></th><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            esc(a.id), esc(a.id), esc(a.title), esc(a.render), esc(a.media_type), badge(a.verified)
        ));
    }
    out.push_str("</tbody></table>");
    out.push_str("</section>");
    // No right aside on index page
    out.push_str("</div>");
    out
}
