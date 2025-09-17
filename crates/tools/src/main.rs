use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use ed25519_dalek::{Signer, SigningKey};
use base64::Engine as _;
use hex::ToHex;
use manifest_contract as mc;
use rand::rngs::OsRng;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Parser)]
#[command(name = "provenance-tools", version, about = "Helper tools for Provenance manifests")] 
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Debug, Subcommand)]
enum Cmd {
    /// Compute SHA-256 for artifacts in manifest and update fields in-place
    UpdateSha {
        /// Project root directory (where artifact paths are relative)
        #[arg(long, default_value = ".")]
        root: PathBuf,
        /// Path to manifest.json
        #[arg(long, default_value = ".provenance/manifest.json")]
        manifest: PathBuf,
    },
    /// Canonicalize and sign manifest (Ed25519, Base64 signature file)
    Sign {
        /// Path to manifest.json
        #[arg(long, default_value = ".provenance/manifest.json")]
        manifest: PathBuf,
        /// Path to private key (32 bytes seed) file
        #[arg(long)]
        privkey: PathBuf,
        /// Output signature file path (defaults to manifest.json.sig next to manifest)
        #[arg(long)]
        sig_out: Option<PathBuf>,
        /// Optional output path for public key (Base64)
        #[arg(long)]
        pubkey_out: Option<PathBuf>,
    },
    /// Generate a test Ed25519 keypair (writes private and public key files)
    GenTestKey {
        /// Output path for private key file (mode 600 recommended)
        #[arg(long)]
        privkey_out: PathBuf,
        /// Output path for public key file (Base64)
        #[arg(long)]
        pubkey_out: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::UpdateSha { root, manifest } => update_sha(&root, &manifest),
        Cmd::Sign { manifest, privkey, sig_out, pubkey_out } => sign_manifest(&manifest, &privkey, sig_out.as_ref(), pubkey_out.as_ref()),
        Cmd::GenTestKey { privkey_out, pubkey_out } => gen_test_key(&privkey_out, &pubkey_out),
    }
}

fn update_sha(root: &Path, manifest_path: &Path) -> Result<()> {
    let (m, mut val) = mc::load_manifest(manifest_path)?;
    let artifacts = val.get_mut("artifacts").and_then(|v| v.as_array_mut()).ok_or_else(|| anyhow!("manifest.artifacts must be array"))?;
    for (i, art) in m.artifacts.iter().enumerate() {
        let src = root.join(&art.path);
        let digest = sha256_file(&src).with_context(|| format!("compute sha256 for {}", src.display()))?;
        if let Some(obj) = artifacts.get_mut(i).and_then(|v| v.as_object_mut()) {
            obj.insert("sha256".to_string(), Value::String(digest.clone()));
        }
        println!("{} => {}", art.id, digest);
    }
    // Write back manifest pretty
    let txt = serde_json::to_string_pretty(&val)? + "\n";
    fs::write(manifest_path, txt).with_context(|| format!("write {}", manifest_path.display()))?;
    Ok(())
}

fn sign_manifest(manifest_path: &Path, privkey_path: &Path, sig_out: Option<&PathBuf>, pubkey_out: Option<&PathBuf>) -> Result<()> {
    let (_m, val) = mc::load_manifest(manifest_path)?;
    let canonical = mc::canonicalize(&val);

    let sk_bytes = fs::read(privkey_path).with_context(|| format!("read private key {}", privkey_path.display()))?;
    if sk_bytes.len() != 32 {
        return Err(anyhow!("private key must be 32 bytes (seed)"));
    }
    let sk = SigningKey::from_bytes(&sk_bytes.try_into().unwrap());
    let sig = sk.sign(&canonical);
    let sig_b64 = base64::engine::general_purpose::STANDARD.encode(sig.to_bytes());

    let sig_path = sig_out
        .map(|p| p.clone())
        .unwrap_or_else(|| manifest_path.with_extension("json.sig"));
    fs::write(&sig_path, format!("{}\n", sig_b64)).with_context(|| format!("write {}", sig_path.display()))?;

    if let Some(pk_path) = pubkey_out {
        let pk_b64 = base64::engine::general_purpose::STANDARD.encode(sk.verifying_key().to_bytes());
        fs::write(pk_path, format!("{}\n", pk_b64)).with_context(|| format!("write {}", pk_path.display()))?;
    }

    println!("Wrote signature to {}", sig_path.display());
    Ok(())
}

fn gen_test_key(priv_out: &Path, pub_out: &Path) -> Result<()> {
    let mut rng = OsRng;
    let sk = SigningKey::generate(&mut rng);
    let pk = sk.verifying_key();
    let sk_bytes = sk.to_bytes();
    let pk_b64 = base64::engine::general_purpose::STANDARD.encode(pk.to_bytes());
    // Write private (32 bytes)
    {
        let mut f = fs::File::create(priv_out).with_context(|| format!("create {}", priv_out.display()))?;
        f.write_all(&sk_bytes)?;
    }
    fs::write(pub_out, format!("{}\n", pk_b64)).with_context(|| format!("write {}", pub_out.display()))?;
    println!("Wrote priv={} ({} bytes) and pub={} (base64)", priv_out.display(), sk_bytes.len(), pub_out.display());
    Ok(())
}

fn sha256_file(path: impl AsRef<Path>) -> Result<String> {
    let mut f = fs::File::open(path.as_ref())?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 { break; }
        hasher.update(&buf[..n]);
    }
    let got = hasher.finalize();
    Ok(got.encode_hex::<String>())
}
