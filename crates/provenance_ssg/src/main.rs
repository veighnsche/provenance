mod render;

use anyhow::{anyhow, Context, Result};
use badges as badges_lib;
use clap::Parser;
use manifest_contract as mc;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
// IMPORTANT: The Proofdown parser is an external submodule workspace maintained by external consultants.
// Do NOT modify parser code in this repository. Integrate via feature-gated optional dependency only.
#[cfg(feature = "external_pml")]
use proofdown_parser as pml;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(name = "provenance-ssg", version, about = "Static site generator for Provenance (read-only)")]
struct Args {
    /// Project root directory (where CI artifacts and .provenance live)
    #[arg(long, default_value = ".")]
    root: PathBuf,

    /// Path to the manifest JSON relative to --root
    #[arg(long, default_value = ".provenance/manifest.json")]
    manifest: PathBuf,

    /// Output directory for generated static site
    #[arg(long, default_value = "site")] 
    out: PathBuf,

    /// Copy raw artifact files alongside the site (for downloads & images)
    #[arg(long, default_value_t = true)]
    copy_assets: bool,

    /// Verify manifest: schema + canonicalize + Ed25519 signature
    #[arg(long, default_value_t = false)]
    verify_manifest: bool,

    /// Public key for signature verification (Base64 or hex)
    #[arg(long)]
    pubkey: Option<String>,

    /// Path to JSON Schema (default schemas/manifest.schema.json)
    #[arg(long)]
    schema_path: Option<PathBuf>,

    /// Maximum inline bytes before truncation notice (for JSON/markdown)
    #[arg(long, default_value_t = 1_000_000usize)]
    truncate_inline_bytes: usize,
}

fn main() -> Result<()> {
    let args = Args::parse();
    fs::create_dir_all(&args.out).context("create output dir")?;

    // Load manifest (typed + raw JSON)
    let manifest_path = args.root.join(&args.manifest);
    let (manifest, manifest_json) = mc::load_manifest(&manifest_path)?;

    // Optional schema validation
    let schema_path = args
        .schema_path
        .unwrap_or_else(|| PathBuf::from("schemas/manifest.schema.json"));
    let schema_text = fs::read_to_string(&schema_path)
        .with_context(|| format!("read schema at {}", schema_path.display()))?;
    mc::validate_schema(&manifest_json, &schema_text)?;
    mc::validate_semantics(&manifest, &args.root)?;

    // Optional signature verification
    let mut provenance_verified = false;
    if args.verify_manifest {
        let sig_path = manifest_path.with_extension("json.sig");
        let sig_b64 = fs::read_to_string(&sig_path)
            .with_context(|| format!("read signature at {}", sig_path.display()))?;
        let canonical = mc::canonicalize(&manifest_json);
        let pubkey = args
            .pubkey
            .as_deref()
            .ok_or_else(|| anyhow!("--pubkey is required with --verify-manifest"))?;
        provenance_verified = mc::ed25519_verify(&canonical, &sig_b64, pubkey)
            .context("verify Ed25519 signature")?;
        if !provenance_verified {
            return Err(anyhow!("manifest signature verification failed"));
        }
    }

    // Prepare assets dir
    let assets_dir = args.out.join("assets");
    if args.copy_assets {
        fs::create_dir_all(&assets_dir).context("create assets dir")?;
    }

    // Pre-compute verification status and build artifact views
    let mut views = Vec::new();
    for a in &manifest.artifacts {
        let src = args.root.join(&a.path);
        let (verified, digest_hex) = verify_sha256(&src, &a.sha256).unwrap_or((false, None));

        // Copy asset if requested
        let download_href = if args.copy_assets && src.is_file() {
            let base = src
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("artifact");
            let safe_name = sanitize_file_name(base);
            let dest_sub = PathBuf::from("assets").join(&a.id);
            let dest_dir = args.out.join(&dest_sub);
            fs::create_dir_all(&dest_dir).ok();
            let dest_path = dest_dir.join(&safe_name);
            let _ = fs::copy(&src, &dest_path);
            format!("/{}/{}", dest_sub.to_string_lossy(), safe_name)
        } else {
            // Fallback: relative path into repo (may not work when hosted separately)
            format!("/{}", a.path)
        };

        views.push(ArtifactViewExt::from(a.clone(), verified, download_href, digest_hex));
    }

    // Build index KPIs from known artifacts
    let mut kpis: BTreeMap<&str, String> = BTreeMap::new();
    let mut tests_summary: Option<render::TestSummary> = None;
    let mut coverage: Option<render::Coverage> = None;
    if let Some(ts) = load_artifact_bytes(&manifest.artifacts, &args.root, "summary:test")? {
        if let Ok(s) = serde_json::from_slice::<render::TestSummary>(&ts) {
            kpis.insert("Tests", format!("{} total, {} passed, {} failed", s.total, s.passed, s.failed));
            kpis.insert("Duration", format!("{:.2}s", s.duration_seconds));
            tests_summary = Some(s);
        }
    }
    if let Some(cv) = load_artifact_bytes(&manifest.artifacts, &args.root, "table:coverage")? {
        if let Ok(c) = serde_json::from_slice::<render::Coverage>(&cv) {
            if let Some(ref t) = c.total { kpis.insert("Coverage", format!("{:.1}%", t.pct)); }
            coverage = Some(c);
        }
    }

    // Build index page
    #[cfg(feature = "external_pml")]
    {
        // Parse and render Proofdown front page
        let front_path = args.root.join(&manifest.front_page.markup);
        let front_text = fs::read_to_string(&front_path).with_context(|| format!("read front page {}", front_path.display()))?;
        let doc = pml::parse(&front_text).context("parse Proofdown front page")?;
        let index_inner = render_front_page(&doc, &manifest, &views, args.truncate_inline_bytes, &args.root)?;
        let index_html = render::page_base(index_inner);
        write_html(args.out.join("index.html"), &index_html)?;
    }
    #[cfg(not(feature = "external_pml"))]
    {
        // Fallback: simple index using KPIs and featured artifacts without Proofdown parsing
        let index_html = render::index_page(
            &manifest.front_page.title,
            &manifest.commit,
            kpis,
            views.iter().map(|v| v.as_view()).collect(),
        );
        write_html(args.out.join("index.html"), &index_html)?;
    }

    // Render per-artifact pages
    for v in &views {
        let a = &v.artifact;
        let src = args.root.join(&a.path);
        let bytes = fs::read(&src).unwrap_or_default();
        let body = match a.render.as_str() {
            "summary:test" => render::render_tests_summary(&bytes).unwrap_or_else(|e| format!("<pre>parse error: {}</pre>", e)),
            "table:coverage" => render::render_coverage(&bytes).unwrap_or_else(|e| format!("<pre>parse error: {}</pre>", e)),
            "markdown" => {
                if file_too_large(&src, args.truncate_inline_bytes) {
                    format!("<div class=\"card\"><strong>Truncated</strong>: file too large. <a href=\"{}\">Download</a></div>", v.download_href)
                } else {
                    render::render_markdown(&String::from_utf8_lossy(&bytes))
                }
            }
            "json" => {
                if file_too_large(&src, args.truncate_inline_bytes) {
                    format!("<div class=\"card\"><strong>Truncated</strong>: file too large. <a href=\"{}\">Download</a></div>", v.download_href)
                } else {
                    render::render_json_pretty(&bytes).unwrap_or_else(|e| format!("<pre>parse error: {}</pre>", e))
                }
            }
            "image" => render::render_image(&v.download_href, &a.title),
            other => return Err(anyhow!("Unsupported render: {} for id {}", other, a.id)),
        };
        let page_html = render::artifact_page(&v.as_view(), &body);
        let out_dir = args.out.join("a").join(&a.id);
        fs::create_dir_all(&out_dir).context("create artifact page dir")?;
        write_html(out_dir.join("index.html"), &page_html)?;
    }

    // robots.txt
    let robots = "User-agent: *\nDisallow: /fragment/\n";
    fs::write(args.out.join("robots.txt"), robots).ok();

    // Write badges
    let badge_dir = args.out.join("badge");
    fs::create_dir_all(&badge_dir).ok();
    // provenance badge
    let all_artifacts_verified = views.iter().all(|v| v.verified);
    let prov_badge = badges_lib::badge_provenance(provenance_verified && all_artifacts_verified);
    write_badge(&badge_dir, "provenance", &prov_badge)?;
    // tests badge
    if let Some(s) = &tests_summary {
        let s_badge = badges_lib::TestSummary { total: s.total, passed: s.passed, failed: s.failed, duration_seconds: s.duration_seconds };
        let b = badges_lib::badge_tests(&s_badge);
        write_badge(&badge_dir, "tests", &b)?;
    }
    // coverage badge
    if let Some(c) = &coverage {
        let c_badge = badges_lib::Coverage { total: c.total.as_ref().map(|t| badges_lib::CoverageTotal { pct: t.pct }) };
        let b = badges_lib::badge_coverage(&c_badge);
        write_badge(&badge_dir, "coverage", &b)?;
    }

    println!("Site generated at {}", args.out.display());
    Ok(())
}

fn write_html(path: PathBuf, html: &str) -> Result<()> {
    fs::write(&path, html).with_context(|| format!("write {}", path.display()))
}

fn verify_sha256(path: impl AsRef<Path>, hex_expected: &str) -> Result<(bool, Option<String>)> {
    let mut f = fs::File::open(&path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 { break; }
        hasher.update(&buf[..n]);
    }
    let got = hasher.finalize();
    let got_hex = format!("{:x}", got);
    Ok((got_hex == hex_expected, Some(got_hex)))
}

fn sanitize_file_name(name: &str) -> String {
    utf8_percent_encode(name, NON_ALPHANUMERIC).to_string()
}

struct ArtifactViewExt {
    artifact: mc::Artifact,
    verified: bool,
    download_href: String,
    digest_hex: Option<String>,
}

impl ArtifactViewExt {
    fn from(artifact: mc::Artifact, verified: bool, download_href: String, digest_hex: Option<String>) -> Self {
        Self { artifact, verified, download_href, digest_hex }
    }
    fn as_view(&self) -> render::ArtifactView {
        render::ArtifactView {
            id: &self.artifact.id,
            title: &self.artifact.title,
            render: &self.artifact.render,
            media_type: &self.artifact.media_type,
            path_rel: &self.artifact.path,
            verified: self.verified,
            download_href: self.download_href.clone(),
        }
    }
}

fn load_artifact_bytes(artifacts: &[mc::Artifact], root: &Path, render_hint: &str) -> Result<Option<Vec<u8>>> {
    if let Some(a) = artifacts.iter().find(|a| a.render == render_hint) {
        let p = root.join(&a.path);
        if p.is_file() {
            let b = fs::read(p)?;
            return Ok(Some(b));
        }
    }
    Ok(None)
}

fn file_too_large(path: &Path, limit: usize) -> bool {
    match fs::metadata(path) { Ok(m) => m.len() as usize > limit, Err(_) => false }
}

fn write_badge(dir: &Path, kind: &str, b: &badges_lib::ShieldsBadge) -> Result<()> {
    let json_path = dir.join(format!("{}.json", kind));
    let svg_path = dir.join(format!("{}.svg", kind));
    let txt = serde_json::to_string_pretty(b)? + "\n";
    fs::write(json_path, txt)?;
    let svg = badges_lib::to_svg(b, Some("flat"))?;
    fs::write(svg_path, svg)?;
    Ok(())
}

#[cfg(feature = "external_pml")]
fn render_front_page(doc: &pml::Document, manifest: &mc::Manifest, views: &[ArtifactViewExt], truncate_limit: usize, root: &Path) -> Result<String> {
    // Minimal renderer for the example components: grid/card + artifact.summary/table/markdown
    fn render_blocks(blocks: &[pml::Block], manifest: &mc::Manifest, views: &[ArtifactViewExt], truncate_limit: usize, root: &Path) -> Result<String> {
        let mut out = String::new();
        for b in blocks {
            match b {
                pml::Block::Heading { level, text } => {
                    out.push_str(&format!("<h{}>{}</h{}>", level, html_escape(&interpolate(text, manifest)), level));
                }
                pml::Block::Paragraph(t) => {
                    out.push_str(&format!("<p>{}</p>", html_escape(&interpolate(t, manifest))));
                }
                pml::Block::Component(c) => {
                    out.push_str(&render_component(c, manifest, views, truncate_limit, root)?);
                }
            }
        }
        Ok(out)
    }

    fn render_component(c: &pml::Component, manifest: &mc::Manifest, views: &[ArtifactViewExt], truncate_limit: usize, root: &Path) -> Result<String> {
        match c.name.as_str() {
            "grid" => {
                let cols = pml::find_attr(&c.attrs, "cols").unwrap_or("3");
                let gap = pml::find_attr(&c.attrs, "gap").unwrap_or("16");
                let mut inner = String::new();
                inner.push_str(&render_blocks(&c.children, manifest, views, truncate_limit, root)?);
                Ok(format!("<div class=\"cards\" style=\"grid-template-columns:repeat({},{})\">{}</div>", cols, "minmax(280px,1fr)", inner))
            }
            "card" => {
                let title = pml::find_attr(&c.attrs, "title").unwrap_or("");
                let mut inner = String::new();
                inner.push_str(&render_blocks(&c.children, manifest, views, truncate_limit, root)?);
                Ok(format!("<div class=\"card\"><h3>{}</h3>{}</div>", html_escape(&interpolate(title, manifest)), inner))
            }
            n if n.starts_with("artifact.") => render_artifact_component(&n[9..], c, manifest, views, truncate_limit, root),
            other => Err(anyhow!("unknown component: {}", other)),
        }
    }

    fn render_artifact_component(kind: &str, c: &pml::Component, _manifest: &mc::Manifest, views: &[ArtifactViewExt], truncate_limit: usize, root: &Path) -> Result<String> {
        let id = pml::find_attr(&c.attrs, "id").ok_or_else(|| anyhow!("artifact.* requires id attribute"))?;
        let v = views.iter().find(|v| v.artifact.id == id).ok_or_else(|| anyhow!("unknown artifact id: {}", id))?;
        let a = &v.artifact;
        let src = root.join(&a.path);
        let bytes = fs::read(&src).unwrap_or_default();
        match kind {
            "summary" => render::render_tests_summary(&bytes).map_err(|e| anyhow!("{}", e)),
            "table" => render::render_coverage(&bytes).map_err(|e| anyhow!("{}", e)),
            "json" => {
                if file_too_large(&src, truncate_limit) { Ok(trunc(&v.download_href)) } else { render::render_json_pretty(&bytes).map_err(|e| anyhow!("{}", e)) }
            }
            "markdown" => {
                if file_too_large(&src, truncate_limit) { Ok(trunc(&v.download_href)) } else { Ok(render::render_markdown(&String::from_utf8_lossy(&bytes))) }
            }
            "image" => Ok(render::render_image(&v.download_href, &a.title)),
            "link" => Ok(format!("<a href=\"/a/{}/\">{}</a>", html_escape(&a.id), html_escape(&a.title))),
            other => Err(anyhow!("unknown artifact component: {}", other)),
        }
    }

    fn trunc(href: &str) -> String { format!("<div class=\"card\"><strong>Truncated</strong>: file too large. <a href=\"{}\">Download</a></div>", href) }
    fn html_escape(s: &str) -> String { s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;") }
    fn interpolate(t: &str, m: &mc::Manifest) -> String {
        let mut out = t.to_string();
        out = out.replace("{{ commit }}", &m.commit);
        out = out.replace("{{ front_page.title }}", &m.front_page.title);
        out
    }

    render_blocks(&doc.blocks, manifest, views, truncate_limit, root)
}
