mod manifest;
mod render;

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use manifest::{Artifact, Manifest};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use sha2::{Digest, Sha256};
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
}

fn main() -> Result<()> {
    let args = Args::parse();
    fs::create_dir_all(&args.out).context("create output dir")?;

    // Load manifest
    let manifest_path = args.root.join(&args.manifest);
    let manifest_text = fs::read_to_string(&manifest_path)
        .with_context(|| format!("read manifest at {}", manifest_path.display()))?;
    let manifest: Manifest = serde_json::from_str(&manifest_text).context("parse manifest")?;

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
    let mut kpis = std::collections::BTreeMap::new();
    if let Some(ts) = load_artifact_bytes(&manifest.artifacts, &args.root, "summary:test")? {
        if let Ok(s) = serde_json::from_slice::<render::TestSummary>(&ts) {
            kpis.insert("Tests", format!("{} total, {} passed, {} failed", s.total, s.passed, s.failed));
            kpis.insert("Duration", format!("{:.2}s", s.duration_seconds));
        }
    }
    if let Some(cv) = load_artifact_bytes(&manifest.artifacts, &args.root, "table:coverage")? {
        if let Ok(c) = serde_json::from_slice::<render::Coverage>(&cv) {
            if let Some(t) = c.total { kpis.insert("Coverage", format!("{:.1}%", t.pct)); }
        }
    }

    // Render index page (fallback without Proofdown front page for now)
    let index_html = render::index_page(
        &manifest.front_page.title,
        &manifest.commit,
        kpis,
        views.iter().map(|v| v.as_view()).collect(),
    );
    write_html(&args.out.join("index.html"), &index_html)?;

    // Render per-artifact pages
    for v in &views {
        let a = &v.artifact;
        let bytes = fs::read(args.root.join(&a.path)).unwrap_or_default();
        let body = match a.render.as_str() {
            "summary:test" => render::render_tests_summary(&bytes).unwrap_or_else(|e| format!("<pre>parse error: {}</pre>", e)),
            "table:coverage" => render::render_coverage(&bytes).unwrap_or_else(|e| format!("<pre>parse error: {}</pre>", e)),
            "markdown" => render::render_markdown(&String::from_utf8_lossy(&bytes)),
            "json" => render::render_json_pretty(&bytes).unwrap_or_else(|e| format!("<pre>parse error: {}</pre>", e)),
            "image" => render::render_image(&v.download_href, &a.title),
            other => format!("<pre>Unsupported render: {}</pre>", other),
        };
        let page_html = render::artifact_page(&v.as_view(), &body);
        let out_dir = args.out.join("a").join(&a.id);
        fs::create_dir_all(&out_dir).context("create artifact page dir")?;
        write_html(&out_dir.join("index.html"), &page_html)?;
    }

    // robots.txt
    let robots = "User-agent: *\nDisallow: /fragment/\n";
    fs::write(args.out.join("robots.txt"), robots).ok();

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
    artifact: Artifact,
    verified: bool,
    download_href: String,
    digest_hex: Option<String>,
}

impl ArtifactViewExt {
    fn from(artifact: Artifact, verified: bool, download_href: String, digest_hex: Option<String>) -> Self {
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

fn load_artifact_bytes(artifacts: &[Artifact], root: &Path, render_hint: &str) -> Result<Option<Vec<u8>>> {
    if let Some(a) = artifacts.iter().find(|a| a.render == render_hint) {
        let p = root.join(&a.path);
        if p.is_file() {
            let b = fs::read(p)?;
            return Ok(Some(b));
        }
    }
    Ok(None)
}
