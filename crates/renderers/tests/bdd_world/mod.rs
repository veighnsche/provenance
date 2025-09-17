use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Default, cucumber::World)]
pub struct World {
    pub strs: BTreeMap<String, String>,
}

impl World {
    pub fn set(&mut self, k: &str, v: impl Into<String>) { self.strs.insert(k.into(), v.into()); }
    pub fn get(&self, k: &str) -> Option<String> { self.strs.get(k).cloned() }

    pub fn repo_root(&self) -> PathBuf {
        if let Some(rr) = self.get("repo_root") { return PathBuf::from(rr); }
        if let Some(b) = self.get("base_dir") { return PathBuf::from(b); }
        let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        crate_dir.parent().and_then(|p| p.parent()).unwrap().to_path_buf()
    }
}
