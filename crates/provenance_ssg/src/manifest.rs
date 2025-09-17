use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Manifest {
    pub version: u32,
    pub repo: String,
    pub commit: String,
    pub workflow_run: WorkflowRun,
    pub front_page: FrontPage,
    pub artifacts: Vec<Artifact>,
}

#[derive(Debug, Deserialize)]
pub struct WorkflowRun {
    pub id: serde_json::Value, // allow string or number
    pub url: String,
    pub attempt: u32,
}

#[derive(Debug, Deserialize)]
pub struct FrontPage {
    pub title: String,
    pub markup: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Artifact {
    pub id: String,
    pub title: String,
    pub path: String,
    pub media_type: String,
    pub render: String,
    pub sha256: String,
}
