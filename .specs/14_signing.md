# Signing Spec — Provenance Manifest (v1‑draft)

This document specifies how the Provenance Manifest is signed and verified.

Related: `./12_canonicalization.md`, `./44_repo_contract.md`, `./16_worker.md`, `schemas/manifest.schema.json`.

---

## 0. Goals

- Ensure the Manifest bytes cannot be tampered with without detection.
- Keep signing simple, cross‑platform, and compatible with the Workers WebCrypto API.
- Support key rotation and multiple trusted keys.

---

## 1. Algorithm and Formats (v1)

- Signature algorithm: Ed25519.
- Public key: 32 raw bytes. Stored as Base64 (or hex) in Worker secret `INDEX_PUBKEY_ED25519`.
- Signature: 64 raw bytes over canonicalized Manifest bytes (see `./03_canonicalization.md`) and encoded as Base64 in `.provenance/manifest.json.sig`.
- Trust model: single public key per deployment in v1. Multi‑key trust MAY be added in v2.

---

## 2. Canonical Bytes to Sign

- Compute canonical bytes of the Manifest via the algorithm in `./03_canonicalization.md` (parse → sort keys → re‑serialize → UTF‑8 bytes).
- The signature MUST be produced over these canonical bytes.

---

## 3. Signing (reference)

Example pseudo‑CLI flow using Node/WebCrypto:

```ts
import { subtle } from 'crypto';

// 1) Load secret key (seed) securely in CI (32 bytes).
// 2) Derive Ed25519 keypair and export public key for the Worker deployment.

// 3) Read and canonicalize manifest
const canonicalBytes = canonicalize(JSON.parse(await fs.readFile('..../manifest.json', 'utf8')));

// 4) Import private key and sign
const privateKey = await subtle.importKey('pkcs8', PRIVATE_KEY_PKCS8_BYTES, { name: 'Ed25519' }, false, ['sign']);
const signature = await subtle.sign('Ed25519', privateKey, canonicalBytes);
await fs.writeFile('..../manifest.json.sig', Buffer.from(signature).toString('base64'));
```

Notes
- Private key handling is CI‑specific; do not embed in repo.
- `@noble/ed25519` may be used if WebCrypto is unavailable in CI.

---

## 4. Verification (Worker)

- Fetch `.provenance/manifest.json` and `.provenance/manifest.json.sig` under `RAW_BASE_URL`.
- Compute canonical bytes of the manifest.
- Import the public key `INDEX_PUBKEY_ED25519` and verify the Base64 signature with WebCrypto (`crypto.subtle.verify('Ed25519', ...)`).
- On failure, return HTTP 409 with a clear error page.

---

## 5. Key Rotation and Multi‑Key (forward‑looking)

- v1: single key per deployment. Rotate by updating the Worker secret and re‑signing manifests.
- v2 (proposed):
  - Trust set of public keys with key ids.
  - Signature file may include `{ key_id: "...", sig: "..." }` JSON to indicate which public key was used.

---

## 6. Test Vectors

- Provide a small Manifest and its canonical bytes and signature in `examples/minimal/` (signature is illustrative only).

---

## 7. Non‑Goals

- Hardware security modules (HSM) integration.
- Multi‑signatures or threshold schemes.
