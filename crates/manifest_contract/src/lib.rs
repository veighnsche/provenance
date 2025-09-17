use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Manifest {
    pub version: u32,
    pub repo: String,
    pub commit: String,
    pub workflow_run: WorkflowRun,
    pub front_page: FrontPage,
    pub artifacts: Vec<Artifact>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkflowRun {
    pub id: serde_json::Value, // allow string or number
    pub url: String,
    pub attempt: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FrontPage {
    pub title: String,
    pub markup: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Artifact {
    pub id: String,
    pub title: String,
    pub path: String,
    pub media_type: String,
    pub render: String,
    pub sha256: String,
}

/// Load manifest JSON as typed struct and raw JSON value
pub fn load_manifest(path: impl AsRef<Path>) -> Result<(Manifest, Value)> {
    let txt = fs::read_to_string(&path).with_context(|| format!("read manifest at {}", path.as_ref().display()))?;
    let val: Value = serde_json::from_str(&txt).context("parse manifest JSON")?;
    let m: Manifest = serde_json::from_value(val.clone()).context("deserialize manifest")?;
    Ok((m, val))
}

/// Validate manifest JSON against a JSON Schema (provided as text)
pub fn validate_schema(manifest_json: &Value, schema_json_text: &str) -> Result<()> {
    let schema_val: Value = serde_json::from_str(schema_json_text).context("parse schema JSON")?;
    // JSONSchema::compile expects the schema to live at least as long as the compiled validator.
    // We compile and validate within this function; to satisfy the borrow, leak the Value (one-time per call).
    let schema: &'static Value = Box::leak(Box::new(schema_val));
    let compiled = jsonschema::JSONSchema::compile(schema).context("compile schema")?;
    if let Err(errors) = compiled.validate(manifest_json) {
        let mut msgs = Vec::new();
        for err in errors {
            msgs.push(format!("{} at {}", err, err.instance_path));
        }
        return Err(anyhow!("schema validation failed:\n{}", msgs.join("\n")));
    }
    Ok(())
}

/// Canonicalize a JSON value by sorting object keys recursively and returning UTF-8 bytes
pub fn canonicalize(manifest_json: &Value) -> Vec<u8> {
    fn normalize(v: &Value) -> Value {
        match v {
            Value::Object(m) => {
                // Sort keys
                let mut bt: BTreeMap<String, Value> = BTreeMap::new();
                for (k, v) in m.iter() {
                    bt.insert(k.clone(), normalize(v));
                }
                // Rebuild as Map (in insertion order which is sorted)
                let mut out = Map::new();
                for (k, v) in bt.into_iter() {
                    out.insert(k, v);
                }
                Value::Object(out)
            }
            Value::Array(arr) => Value::Array(arr.iter().map(normalize).collect()),
            _ => v.clone(),
        }
    }
    let normalized = normalize(manifest_json);
    serde_json::to_vec(&normalized).expect("serialize canonical json")
}

/// Verify an Ed25519 signature over canonical bytes. Signature is base64, public key is base64 or hex.
pub fn ed25519_verify(canonical_bytes: &[u8], signature_b64: &str, pubkey_b64_or_hex: &str) -> Result<bool> {
    let sig_bytes = B64.decode(signature_b64.trim()).context("base64-decode signature")?;
    let sig = Signature::from_slice(&sig_bytes).context("ed25519 signature bytes")?;

    let s = pubkey_b64_or_hex.trim();
    // Try base64 first
    let pk_bytes = match B64.decode(s) {
        Ok(bytes) => bytes,
        Err(_) => hex::decode(s).context("hex-decode public key")?,
    };
    if pk_bytes.len() != 32 {
        return Err(anyhow!("public key must be 32 bytes"));
    }
    let vk = VerifyingKey::from_bytes(&pk_bytes.try_into().expect("len checked"))?;
    Ok(vk.verify(canonical_bytes, &sig).is_ok())
}

/// Semantic validations not covered by schema.
/// - Unique artifact ids
/// - Path normalization (reject `..`, absolute, or leading slash)
/// - Allowed render values (defensive check; schema already enumerates)
pub fn validate_semantics(m: &Manifest, root: impl AsRef<Path>) -> Result<()> {
    // Unique IDs
    let mut seen = HashSet::new();
    for a in &m.artifacts {
        if !seen.insert(&a.id) {
            return Err(anyhow!("duplicate artifact id: {}", a.id));
        }
    }

    // Paths
    let root = root.as_ref();
    for a in &m.artifacts {
        if a.path.starts_with('/') {
            return Err(anyhow!("artifact path must be repo-relative, got: {}", a.path));
        }
        if a.path.contains("..") {
            return Err(anyhow!("artifact path must not contain '..': {}", a.path));
        }
        let joined = root.join(&a.path);
        let canon_root = dunce::canonicalize(root).unwrap_or_else(|_| PathBuf::from(root));
        if let Ok(canon) = dunce::canonicalize(&joined) {
            if !canon.starts_with(&canon_root) {
                return Err(anyhow!("artifact path escapes root: {}", a.path));
            }
        }
        // Defensive render set (kept in sync with schema)
        match a.render.as_str() {
            "markdown" | "json" | "table:coverage" | "summary:test" | "image" | "repo:file" | "repo:bundle" | "repo:symbols" => {}
            other => return Err(anyhow!("unknown render: {} for id {}", other, a.id)),
        }
        // sha256 format (defensive)
        if a.sha256.len() != 64 || !a.sha256.chars().all(|c| c.is_ascii_hexdigit() && c.is_ascii_lowercase() || c.is_ascii_digit()) {
            return Err(anyhow!("invalid sha256 for id {}: {}", a.id, a.sha256));
        }
    }
    Ok(())
}

