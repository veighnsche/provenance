use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Default, cucumber::World)]
pub struct World {
    pub strs: BTreeMap<String, String>,
    pub json: BTreeMap<String, serde_json::Value>,
}

impl World {
    pub fn set(&mut self, k: &str, v: impl Into<String>) { self.strs.insert(k.into(), v.into()); }
    pub fn get(&self, k: &str) -> Option<String> { self.strs.get(k).cloned() }
    pub fn set_json(&mut self, k: &str, v: serde_json::Value) { self.json.insert(k.into(), v); }
    pub fn get_json(&self, k: &str) -> Option<serde_json::Value> { self.json.get(k).cloned() }

    pub fn repo_root(&self) -> PathBuf {
        if let Some(rr) = self.get("repo_root") { return PathBuf::from(rr); }
        if let Some(b) = self.get("base_dir") { return PathBuf::from(b); }
        // default to workspace root: crate_dir/../..
        let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        crate_dir.parent().and_then(|p| p.parent()).unwrap().to_path_buf()
    }
}
