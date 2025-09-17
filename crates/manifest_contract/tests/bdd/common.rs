// Common BDD context for manifest-related steps (skeleton)
use std::path::PathBuf;

pub struct Ctx {
    pub repo_root: Option<PathBuf>,
    pub manifest_path: Option<PathBuf>,
    pub schema_path: Option<PathBuf>,
    // Loaded state
    pub last_err: Option<String>,
}

impl Default for Ctx {
    fn default() -> Self {
        Self { repo_root: None, manifest_path: None, schema_path: None, last_err: None }
    }
}
