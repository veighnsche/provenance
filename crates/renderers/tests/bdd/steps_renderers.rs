use anyhow::{anyhow, Result};
use bdd_harness::{Step, State};
use std::fs;
use std::path::{Path, PathBuf};

pub fn registry() -> Vec<Step> {
    vec![
        step(r#"the repo root is "([^"]+)""#, given_repo_root),
        step(r#"I render "(markdown|json|table:coverage|summary:test)" from "([^"]+)""#, when_render_from_path),
        step(r#"the rendered HTML should contain "([^"]+)""#, then_render_contains),
        step(r#"I render an image with src "([^"]+)" and alt "([^"]+)""#, when_render_image),
    ]
}

fn step(pat: &str, f: fn(&mut State, &regex::Captures) -> Result<()>) -> bdd_harness::Step { bdd_harness::Step::new(pat, f).unwrap() }

fn repo_root(state: &State) -> PathBuf {
    let rr = state.get("repo_root").unwrap_or_else(|| state.get("base_dir").unwrap());
    PathBuf::from(rr)
}

fn given_repo_root(state: &mut State, caps: &regex::Captures) -> Result<()> {
    let rel = caps.get(1).unwrap().as_str();
    let p = if Path::new(rel).is_absolute() { PathBuf::from(rel) } else { repo_root(state).join(rel) };
    state.set("repo_root", p.to_string_lossy().to_string());
    Ok(())
}

fn when_render_from_path(state: &mut State, caps: &regex::Captures) -> Result<()> {
    let kind = caps.get(1).unwrap().as_str();
    let rel = caps.get(2).unwrap().as_str();
    let p = repo_root(state).join(rel);
    match kind {
        "markdown" => {
            let txt = fs::read_to_string(&p).map_err(|e| anyhow!("read {}: {}", p.display(), e))?;
            let html = renderers::render_markdown(&txt);
            state.set("last_html", html);
        }
        "json" => {
            let bytes = fs::read(&p).map_err(|e| anyhow!("read {}: {}", p.display(), e))?;
            let html = renderers::render_json_pretty(&bytes)?;
            state.set("last_html", html);
        }
        "table:coverage" => {
            let bytes = fs::read(&p).map_err(|e| anyhow!("read {}: {}", p.display(), e))?;
            let html = renderers::render_coverage(&bytes)?;
            state.set("last_html", html);
        }
        "summary:test" => {
            let bytes = fs::read(&p).map_err(|e| anyhow!("read {}: {}", p.display(), e))?;
            let html = renderers::render_tests_summary(&bytes)?;
            state.set("last_html", html);
        }
        _ => return Err(anyhow!("unknown kind: {}", kind)),
    }
    Ok(())
}

fn when_render_image(state: &mut State, caps: &regex::Captures) -> Result<()> {
    let src = caps.get(1).unwrap().as_str();
    let alt = caps.get(2).unwrap().as_str();
    let html = renderers::render_image(src, alt);
    state.set("last_html", html);
    Ok(())
}

fn then_render_contains(state: &mut State, caps: &regex::Captures) -> Result<()> {
    let needle = caps.get(1).unwrap().as_str();
    let html = state.get("last_html").unwrap_or_default();
    if html.contains(needle) { Ok(()) } else { Err(anyhow!("rendered HTML did not contain '{}': {}", needle, html)) }
}
