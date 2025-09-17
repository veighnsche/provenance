use anyhow::{anyhow, Result};
use regex::{Captures, Regex};
use std::fs;
use std::path::{Path, PathBuf};

// Minimal BDD harness: regex-based steps and a simple line-oriented Gherkin runner.

pub type StepFn = fn(&mut State, &Captures) -> Result<()>;

pub struct Step {
    pub pattern: Regex,
    pub func: StepFn,
}

impl Step {
    pub fn new(pattern: &str, func: StepFn) -> Result<Self> {
        Ok(Self { pattern: Regex::new(pattern)?, func })
    }
}

#[derive(Default)]
pub struct State {
    // Loose map for cross-step exchange
    pub strs: std::collections::BTreeMap<String, String>,
    pub json: std::collections::BTreeMap<String, serde_json::Value>,
}

impl State {
    pub fn set(&mut self, k: &str, v: impl Into<String>) { self.strs.insert(k.into(), v.into()); }
    pub fn get(&self, k: &str) -> Option<String> { self.strs.get(k).cloned() }
    pub fn set_json(&mut self, k: &str, v: serde_json::Value) { self.json.insert(k.into(), v); }
    pub fn get_json(&self, k: &str) -> Option<serde_json::Value> { self.json.get(k).cloned() }
}

pub struct Runner {
    steps: Vec<Step>,
    pub base_dir: PathBuf,
}

impl Runner {
    pub fn new(base_dir: impl AsRef<Path>) -> Self { Self { steps: Vec::new(), base_dir: base_dir.as_ref().to_path_buf() } }
    pub fn add_step(&mut self, step: Step) { self.steps.push(step); }
    pub fn add_steps(&mut self, steps: Vec<Step>) { self.steps.extend(steps); }

    pub fn run_feature_file(&self, rel_path: &str) -> Result<()> {
        let path = self.base_dir.join(rel_path);
        let text = fs::read_to_string(&path).map_err(|e| anyhow!("read {}: {}", path.display(), e))?;
        self.run_text(&text)
    }

    pub fn run_text(&self, feature_text: &str) -> Result<()> {
        // Extract Background lines and Scenarios blocks; then execute steps.
        let mut background: Vec<String> = Vec::new();
        let mut scenarios: Vec<Vec<String>> = Vec::new();
        let mut cur: Option<Vec<String>> = None;
        let mut in_background = false;
        for raw in feature_text.lines() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') { continue; }
            if line.starts_with("Feature:") { continue; }
            if line.starts_with("Background:") { in_background = true; continue; }
            if line.starts_with("Scenario:") {
                in_background = false;
                if let Some(block) = cur.take() { scenarios.push(block); }
                cur = Some(Vec::new());
                continue;
            }
            if in_background {
                if let Some(step) = normalize_step(line) { background.push(step); }
                continue;
            }
            if let Some(ref mut block) = cur {
                if let Some(step) = normalize_step(line) { block.push(step); }
            }
        }
        if let Some(block) = cur.take() { scenarios.push(block); }

        for scn in scenarios {
            let mut state = State::default();
            // Provide base_dir and repo_root defaults for convenience
            let base = self.base_dir.to_string_lossy().to_string();
            state.set("base_dir", &base);
            state.set("repo_root", &base);
            // Run background
            for s in &background { self.exec_line(&mut state, s)?; }
            // Run scenario
            for s in &scn { self.exec_line(&mut state, s)?; }
        }
        Ok(())
    }

    fn exec_line(&self, state: &mut State, line: &str) -> Result<()> {
        for st in &self.steps {
            if let Some(caps) = st.pattern.captures(line) {
                return (st.func)(state, &caps);
            }
        }
        Err(anyhow!("No step matched: {}", line))
    }
}

fn normalize_step(line: &str) -> Option<String> {
    for kw in ["Given ", "When ", "Then ", "And "] {
        if let Some(rest) = line.strip_prefix(kw) { return Some(rest.trim().to_string()); }
    }
    None
}
