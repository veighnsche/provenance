# Cloudflare Worker Runtime Spec — Provenance (v1‑draft)

This document defines the technical requirements to run the Provenance mirror on Cloudflare Workers (V8 isolate). It complements the core spec (`.specs/00_provenance.md`) and the markup spec (`.specs/10_proofdown.md`).

Status: draft. Behavior described here is normative for the Worker implementation.

---

## 0. Goals

- Run entirely within the Cloudflare Workers V8 isolate.
- Verify provenance before rendering anything: Ed25519 signature for the Index, SHA‑256 for artifacts.
- Render Proofdown → HTML deterministically (via WASM parser) and serve a minimal set of routes.
- Stream large content and enforce strict security boundaries.

---

## 1. Environment and Constraints

- Runtime: Cloudflare Workers (V8 isolate), module syntax (ESM). No Node.js built‑ins, no filesystem, no threads.
- Available APIs (non‑exhaustive):
  - `fetch`, `Request`, `Response`, `URL`, `URLPattern`.
  - `crypto.subtle` (Web Crypto), `crypto.getRandomValues`.
  - Streams: `ReadableStream`, `WritableStream`, `TransformStream`.
  - Cache API: `caches.default` (per‑PoP, best effort).
  - Optional data services (feature‑flagged): KV, Durable Objects, R2 (not required for MVP).
- Resource limits: subject to plan; design with strict CPU/heap budgets. Prefer streaming and lazy verification.

---

## 2. Configuration (Secrets and Env)

Set via `wrangler secret put` or `wrangler.toml` bindings.

- `INDEX_PUBKEY_ED25519` (required): Base64 (or hex) encoded 32‑byte public key used to verify the Index signature.
- `RAW_BASE_URL` (required): Base URL for commit‑pinned raw files (e.g., `https://raw.githubusercontent.com/<owner>/<repo>/<commit>/`). Must end with a trailing `/`.
- `INDEX_PATH` (required): Path (relative to `RAW_BASE_URL`) of the provenance manifest (default: `.provenance/manifest.json`).
- `INDEX_SIG_PATH` (required): Path (relative to `RAW_BASE_URL`) of the manifest signature (default: `.provenance/manifest.json.sig`).
- `FEATURES` (optional): Comma‑separated flags (e.g., `repo_bundle,repo_symbols`).
- `CACHE_TTL_SECONDS` (optional, default 300): TTL for HTTP caching headers.

Notes

- Defaults (recommended): `INDEX_PATH=.provenance/manifest.json`, `INDEX_SIG_PATH=.provenance/manifest.json.sig`.
- All upstream fetches MUST be restricted to descendants of `RAW_BASE_URL`. Reject if the request escapes repo root or includes `..`.
- Do not accept user parameters that could influence `RAW_BASE_URL` without strict validation.

---

## 3. Routes and HTTP Semantics

- `GET /` — Render the front page from Proofdown referenced by the verified Index.
- `GET /fragment/:artifact_id` — Stream/render a single artifact fragment (heavy payloads).
- `GET /a/:artifact_id` — Dedicated artifact page.
- `GET /download/:artifact_id` — Download the verified bytes for the artifact.
- `GET /health` — Liveness/readiness (no upstream fetch). Returns JSON with service version and configured RAW_BASE_URL host.
- `GET /robots.txt` — Disallow crawling of fragment routes; allow root and badges.
- `HEAD` supported for all `GET` routes with appropriate headers.

### 3.1 Badge endpoints

- `GET /badge/:kind.svg` — SVG badge image for GitHub READMEs.
- `GET /badge/:kind.json` — Shields.io‑compatible JSON badge:
  - `{ "schemaVersion": 1, "label": string, "message": string, "color": string }`

Supported kinds (minimum)

- `provenance` — overall verification status: `verified` (Index sig ok) or `error`.
- `tests` — summarize from `summary:test` artifact (e.g., `123 passed`, `2 failed`).
- `coverage` — percentage from coverage artifact (rounded; thresholds colorized).

Rules

- Badges MUST be derived only from verified inputs (signed Index and referenced artifacts).
- Unknown `kind` → 404 or 415; missing artifact or verification failure → 409 with an `error` badge.
- Query params (enumerated only):
  - `style=flat|flat-square` (default `flat`)
  - `label=<alnum-_-safe>` (length‑limited)
  - Any other param MUST be rejected.

Content types

- SVG: `image/svg+xml; charset=utf-8`.
- JSON: `application/json; charset=utf-8` (Shields schema).

Caching

- Set `ETag` and `Cache-Control` (short TTL, e.g., 60–300s).
- Respect `If-None-Match`.

Examples

```md
[![Provenance](https://<worker-domain>/badge/provenance.svg)](https://<worker-domain>/)
[![Tests](https://<worker-domain>/badge/tests.svg)](https://<worker-domain>/a/tests-summary)
[![Coverage](https://<worker-domain>/badge/coverage.svg)](https://<worker-domain>/a/coverage)
```

Response headers (minimum)

- `Content-Type` — `text/html; charset=utf-8` for pages; `application/json` for JSON; media‑specific for others.
- `ETag` — Stable across identical verified content; recommended `"sha256:<hex>"` for artifacts, and a composite for pages (e.g., commit + index version + artifact ids).
- `Cache-Control` — `public, max-age=<TTL>, stale-while-revalidate=30`.
- `Content-Security-Policy` — Strong defaults (example): `default-src 'none'; img-src 'self' data:; style-src 'self' 'unsafe-inline'; script-src 'none'; connect-src 'self'; font-src 'self'; base-uri 'none'; frame-ancestors 'none'`.

Error codes

- 400 — Bad request (invalid artifact id, malformed query).
- 404 — Not found (unknown artifact id or path).
- 409 — Verification conflict (digest/signature mismatch).
- 415 — Unsupported media/viewer.
- 502 — Upstream fetch failed.

---

## 4. Fetch and Verification Pipeline

On every request needing data:

1) Fetch Index and signature

- Fetch `INDEX_PATH` and `INDEX_SIG_PATH` from `RAW_BASE_URL`.
- Do not parse before verification. Collect raw bytes for verification.

2) Verify Index signature (Ed25519)

- Import public key (Base64/hex → raw bytes → `crypto.subtle.importKey('raw', key, 'Ed25519', false, ['verify'])`).
- Verify: `crypto.subtle.verify('Ed25519', key, sigBytes, indexBytes)`.
- Fallback (optional): `@noble/ed25519` if Ed25519 is unavailable in the runtime.
- On failure: return `409` with a clear error page.

3) Parse Index and perform lightweight validation

- Ensure required fields are present; validate artifact ids, paths, media types, render hints.
- Do not trust any path that attempts traversal; enforce repo‑relative normalization.

4) Resolve and verify artifacts lazily

- For an artifact needed by the page:
  - Construct a commit‑pinned raw URL under `RAW_BASE_URL` using the artifact `path`.
  - Fetch the bytes; stream and compute `SHA-256` via `crypto.subtle.digest` while teeing to the renderer or response stream.
  - Compare against `artifact.sha256` from the Index. If mismatch, fail with `409` and mark artifact as unrenderable.
- Cache verified bytes by their `sha256` (Cache API key suggestion: `/sha/<hex>`).

5) Render Proofdown → HTML

- Load the Proofdown parser WASM (Rust→WASM) at startup; reuse instance across requests when possible.
- Feed AST into deterministic renderers mapped by `render` + `media_type`.
- All text output must be sanitized before HTML emission; strictly avoid client scripts.

---

## 5. Proofdown Integration

- The Worker hosts the Proofdown parser (WASM) and calls it for:
  - Parsing `.pml` files (front page and includes) into a stable AST.
  - Validating components, attributes, and link macros (`a:`, `repo:`/`src:`, `doc:`, optional `ci:` and `gh:` anchors).
- The renderer rejects unknown artifact ids and unresolved repo links (unless verified via per‑file artifacts or a bundle).
- Includes are depth‑limited (e.g., 3) to prevent cycles.

---

## 6. Caching Strategy

- HTTP layer
  - Set `ETag` and `Cache-Control` for all responses.
  - Support conditional requests via `If-None-Match` and `If-Modified-Since` (optional).
- Workers Cache
  - Cache verified artifact bytes by `sha256` (content‑addressed) with conservative TTL.
  - Cache rendered fragments keyed by `(commit, artifact_id, viewer_version)`.
- In‑memory (isolate‑local) caches MAY be used (ephemeral per instance) for the Index and WASM module.

ETag formulas (reference)
- Artifacts: `"sha256:<hex>"` where `<hex>` is from the manifest.
- Pages: stable hash of `(commit, manifest.version, front_page.markup, referenced_artifact_ids[], viewer_versions[])`.
- Fragments: stable hash of `(commit, artifact_id, viewer_version)`.

---

## 7. Security Model (Worker‑specific)

- Upstream fetch policy: only under `RAW_BASE_URL`; no dynamic hosts; block `..` and absolute URL injection.
- No rendering of unverified resources; any attempt to reference unknown ids must be rejected.
- Sanitize all text; remove/escape any unsafe HTML.
- Supply strict `Content-Security-Policy` headers.
- Do not include secrets in responses or logs. Minimize logs; avoid dumping inputs.

Threat considerations
- SSRF: `RAW_BASE_URL` is deployment‑fixed; all user inputs MUST NOT influence upstream URLs.
- Path traversal: normalize and reject any path containing `..` or absolute prefixes; enforce repo‑relative paths.
- Zip‑slip: for `repo:bundle`, reject entries with `..`, absolute paths, or symlinks; cap file count, depth, and total uncompressed size.
- Resource limits: enforce max response size and render time; stream to minimize heap.

---

## 8. Performance and Limits

- Stream heavy content and avoid buffering whole payloads in memory.
- Enforce per‑artifact limits: max bytes, max parse/render time. Truncate with a clear notice and verified download link.
- Avoid quadratic rendering patterns; prefer iterators/streams.
- WASM parser: initialize once; reuse; avoid per‑request compilation.

---

## 9. Observability

- Structured logs (minimal): `{ route, artifact_id?, verify_index_ok, verify_artifact_ok?, duration_ms, cf_ray? }`.
- Correlate using `cf-ray` header when present.
- Response headers:
  - `X-Provenance-Commit: <commit-sha>`
  - `X-Provenance-Manifest-Version: <version>`
  - `X-Provenance-Viewer-Version: <name>@<semver>` (when applicable)

---

## 10. Example Wrangler Config (reference)

```toml
# wrangler.toml (reference)
name = "provenance-worker"
main = "src/index.ts"
compatibility_date = "2024-01-01"

[vars]
RAW_BASE_URL = "https://raw.githubusercontent.com/owner/repo/8c6a9f4e.../"
INDEX_PATH = ".provenance/manifest.json"
INDEX_SIG_PATH = ".provenance/manifest.json.sig"
CACHE_TTL_SECONDS = 300
FEATURES = "repo_bundle,repo_symbols"

# Add with: wrangler secret put INDEX_PUBKEY_ED25519
```

---

## 11. Pseudocode (verification core)

```ts
async function verifyIndex(env: Env) {
  const indexResp = await fetch(new URL(env.INDEX_PATH, env.RAW_BASE_URL));
  const sigResp = await fetch(new URL(env.INDEX_SIG_PATH, env.RAW_BASE_URL));
  if (!indexResp.ok || !sigResp.ok) throw new Error("Upstream error");

  const indexBytes = new Uint8Array(await indexResp.arrayBuffer());
  const sigBytes = new Uint8Array(await sigResp.arrayBuffer());

  const pubKeyBytes = decodeBase64(env.INDEX_PUBKEY_ED25519);
  const key = await crypto.subtle.importKey("raw", pubKeyBytes, "Ed25519", false, ["verify"]);
  const ok = await crypto.subtle.verify("Ed25519", key, sigBytes, indexBytes);
  if (!ok) throw new VerificationError("Index signature mismatch");

  return JSON.parse(new TextDecoder().decode(indexBytes));
}

async function verifyArtifact(env: Env, path: string, expectedSha256Hex: string): Promise<Response> {
  const url = new URL(path, env.RAW_BASE_URL);
  const resp = await fetch(url);
  if (!resp.ok || !resp.body) throw new Error("Upstream artifact fetch failed");

  const [forHash, forReturn] = resp.body.tee();
  const digestPromise = (async () => {
    const reader = forHash.getReader();
    const chunks: Uint8Array[] = [];
    for (;;) {
      const { value, done } = await reader.read();
      if (done) break;
      chunks.push(value);
    }
    const all = concat(chunks);
    const hash = await crypto.subtle.digest("SHA-256", all);
    const hex = toHex(new Uint8Array(hash));
    if (hex !== expectedSha256Hex) throw new VerificationError("Artifact digest mismatch");
  })();

  // Start hashing but return stream immediately
  const returned = new Response(forReturn, resp);
  returned.headers.set("ETag", `"sha256:${expectedSha256Hex}"`);
  returned.headers.set("Cache-Control", `public, max-age=${env.CACHE_TTL_SECONDS || 300}, stale-while-revalidate=30`);
  event.waitUntil(digestPromise);
  return returned;
}
```

---

## 12. Feature Flags

- `repo_bundle` — enable source bundle resolution for `repo:` links/snippets.
- `repo_symbols` — enable symbol links/snippets (requires symbols artifact present in Index).
- `experimental_viewers` — enable non‑core viewers.

---

## 13. Compliance (Worker‑specific)

- Index signature verified with `INDEX_PUBKEY_ED25519`.
- Artifact `sha256` verified before embedding or linking to raw bytes.
- Proofdown parser (WASM) used for all `.pml` documents; unknown components rejected.
- Only `RAW_BASE_URL` descendants fetched; no dynamic host fetches.
- Strong CSP headers set on all HTML responses.

---

## 14. Future Considerations

- Pre‑warm WASM and public key on isolate startup for lower latency.
- Use Durable Objects for cross‑PoP cache coordination (optional).
- Add support for signed bundles (e.g., minisign) with merkle proofs for selective file verification.

---

## 15. Error Model (APIs)

- JSON endpoints (e.g., `GET /badge/:kind.json`) SHOULD return RFC 7807 Problem Details on error with `application/problem+json` content type:
  - `{ "type": "about:blank", "title": "Bad Request", "status": 400, "detail": "unknown kind" }`
- HTML endpoints SHOULD render human‑readable error pages with stable HTTP codes per "Error codes" and include a link back to `/`.
- Downloads SHOULD use `409` for verification mismatch and include a small RFC 7807 JSON in the body if `Accept: application/problem+json` is sent.
