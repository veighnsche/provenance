# Canonicalization Spec — Provenance Manifest (v1‑draft)

This document defines how to produce a stable byte representation of the Provenance Manifest for signing and verification.

Related: `./11_repo_contract.md`, `./02_worker.md`, `schemas/manifest.schema.json`.

---

## 0. Goals

- Deterministic bytes across platforms and toolchains.
- Human‑readable JSON remains the source; canonicalization only affects signing bytes.
- Easy to implement in common languages.

---

## 1. Canonical JSON Rules

- Encoding: UTF‑8.
- Newlines: `\n` (LF); no BOM.
- Whitespace: minimal — one space after `:` and after `,` as in standard pretty printing (see example), or compact (no extra spaces). The canonicalization step ignores whitespace by re‑serializing from an in‑memory representation.
- Object key order: lexicographic ascending (byte order) for all objects at every depth.
- Number formatting: integers only where specified by schema; no floats in v1.
- Strings: no control characters; escape per JSON spec.

Implementation pattern

1) Parse JSON into an in‑memory structure.
2) Recursively sort object keys ascending.
3) Serialize with UTF‑8, LF newlines, and a stable whitespace policy (either compact or standard two‑space indentation). The exact indent is not significant if both signer and verifier re‑serialize prior to signing/verification.

---

## 2. Canonicalization Pseudocode

```ts
function canonicalize(obj: any): Uint8Array {
  const normalized = normalize(obj);
  const text = JSON.stringify(normalized);
  return new TextEncoder().encode(text);
}

function normalize(x: any): any {
  if (Array.isArray(x)) return x.map(normalize);
  if (x && typeof x === 'object') {
    const out: any = {};
    Object.keys(x).sort().forEach(k => { out[k] = normalize(x[k]); });
    return out;
  }
  return x;
}
```

The Worker SHOULD re‑parse and re‑serialize before verifying signatures to avoid whitespace differences.

---

## 3. Test Vectors

Input (formatted, keys scrambled):

```json
{ "commit": "8c6a9f4e", "version": 1, "repo": "acme/provenance",
  "front_page": { "markup": "ci/front_page.pml", "title": "QA Evidence" },
  "workflow_run": { "attempt": 1, "url": "https://github.com/...", "id": 123 },
  "artifacts": [
    { "sha256": "00...ff", "id": "tests-summary", "title": "Tests", "render": "summary:test", "media_type": "application/json", "path": "ci/tests/summary.json" }
  ]
}
```

Canonical JSON (compact example):

```json
{"artifacts":[{"id":"tests-summary","media_type":"application/json","path":"ci/tests/summary.json","render":"summary:test","sha256":"00...ff","title":"Tests"}],"commit":"8c6a9f4e","front_page":{"markup":"ci/front_page.pml","title":"QA Evidence"},"repo":"acme/provenance","version":1,"workflow_run":{"attempt":1,"id":123,"url":"https://github.com/..."}}
```

The signature MUST be computed over the canonical bytes.

---

## 4. Hashing and Digests

- Artifact digests: `sha256` (hex) required in v1. Hash agility MAY be introduced in v2 via a `{ algo, value }` object.
- Page ETags are separate from manifest signing and derived per Worker policy (see Worker spec).

---

## 5. Non‑Goals

- Canonicalizing across numeric types beyond integers.
- Support for comments or non‑JSON formats.
