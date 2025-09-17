use crate::bdd_world::World;
use cucumber::{given, then, when};
use std::fs;
use std::path::{Path, PathBuf};

fn repo_root(world: &World) -> PathBuf { world.repo_root() }

#[given(regex = r#"^the repo root is \"([^\"]+)\"$"#)]
pub async fn given_repo_root(world: &mut World, rel: String) {
    let p = if Path::new(&rel).is_absolute() { PathBuf::from(rel) } else { repo_root(world).join(rel) };
    world.set("repo_root", p.to_string_lossy());
}

#[when(regex = r#"^I render \"(markdown|json|table:coverage|summary:test)\" from \"([^\"]+)\"$"#)]
pub async fn when_render_from_path(world: &mut World, kind: String, rel: String) {
    let p = repo_root(world).join(rel);
    match kind.as_str() {
        "markdown" => {
            let txt = fs::read_to_string(&p).expect("read markdown");
            let html = renderers::render_markdown(&txt);
            world.set("last_html", html);
        }
        "json" => {
            let bytes = fs::read(&p).expect("read json");
            let html = renderers::render_json_pretty(&bytes).expect("render json");
            world.set("last_html", html);
        }
        "table:coverage" => {
            let bytes = fs::read(&p).expect("read coverage");
            let html = renderers::render_coverage(&bytes).expect("render coverage");
            world.set("last_html", html);
        }
        "summary:test" => {
            let bytes = fs::read(&p).expect("read tests summary");
            let html = renderers::render_tests_summary(&bytes).expect("render tests summary");
            world.set("last_html", html);
        }
        _ => panic!("unknown kind: {}", kind),
    }
}

#[when(regex = r#"^I render an image with src \"([^\"]+)\" and alt \"([^\"]+)\"$"#)]
pub async fn when_render_image(world: &mut World, src: String, alt: String) {
    let html = renderers::render_image(&src, &alt);
    world.set("last_html", html);
}

#[then(regex = r#"^the rendered HTML should contain \"([^\"]+)\"$"#)]
pub async fn then_render_contains(world: &mut World, needle: String) {
    let html = world.get("last_html").unwrap_or_default();
    assert!(html.contains(&needle), "rendered HTML did not contain '{}': {}", needle, html);
}

#[then(regex = r#"^the rendered HTML should contain \"alt=\\\"([^\\\"]+)\\\"\"$"#)]
pub async fn then_render_contains_alt(world: &mut World, alt: String) {
    let html = world.get("last_html").unwrap_or_default();
    let target = format!("alt=\"{}\"", alt);
    assert!(html.contains(&target), "rendered HTML did not contain '{}': {}", target, html);
}
