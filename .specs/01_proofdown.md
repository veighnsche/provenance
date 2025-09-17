# Proofdown — Markup Spec (v1‑draft)

This document specifies “Proofdown” (working name), a constrained, deterministic markup language for the Provenance artifact‑first testing mirror. Proofdown is designed to be authored by AIs and reviewed by humans; it is less than HTML and more than Markdown, with a small, typed component set and strict security guarantees.

Status: draft. Naming may change; semantics are stable once versioned.

---

## 0. Terminology

- MUST/SHOULD/MAY follow RFC‑2119.
- Index: the signed config enumerating all renderable artifacts and front‑page markup.
- Artifact: any file produced by CI and referenced by the Index.
- Proofdown: the markup language defined by this spec. Recommended extension: `.pml`.
- Worker: the Cloudflare Worker runtime that verifies, renders, and serves content.

---

## 1. Goals & Non‑Goals

- MUST be safe: no raw HTML/JS/CSS; no network fetch beyond verified Index resources.
- MUST be deterministic: fixed grammar; stable AST; no side effects; no time/random/IO.
- MUST be verifiable: every external reference resolves only to resources listed in the verified Index or derived deterministically from those resources.
- MUST be AI‑authorable: concise, unambiguous components/attributes; lintable; templates.
- SHOULD support human review: easy artifact linking and embedding; readable defaults.
- Non‑Goals: general web layout; Turing completeness; arbitrary styling; client scripting.

---

## 2. File & Versioning

- Encoding: UTF‑8, normalized newlines `\n`.
- File extension: `.pml` (Proof Markup Language).
- Top‑level versioning: Proofdown syntax is versioned via the Index `version`. Breaking changes are gated by major `version` and MUST be rejected by the Worker if unknown.

---

## 3. Syntax Overview

Proofdown combines:

- Markdown‑like paragraphs, headings (`#` to `####`), lists, code fences (with explicit language), and inline code.
- A small HTML‑like component syntax for typed blocks and inlines, e.g., `<grid cols=3> ... </grid>`.
- Interpolation of verified Index fields via `{{ field }}`.
- A dedicated link macro `[[...]]` with special protocols for artifact and repository links.

### 3.1 Component grammar (informal)

- Open tag: `<name attr1=value attr2="string value">`
- Close tag: `</name>`; self‑closing: `<name ... />`
- Names are lowercase with dots for namespaces (e.g., `artifact.summary`, `repo.code`).
- Attributes are typed: numbers, booleans (`true|false`), enums, strings (quoted or bare if alnum/`-_/.:`), and validated by schema.
- Unknown components or attributes MUST be rejected with a parser error.

### 3.2 Interpolation

- Allowed placeholders (from verified Index): `{{ version }}`, `{{ repo }}`, `{{ commit }}`, `{{ workflow_run.id }}`, `{{ workflow_run.url }}`, `{{ workflow_run.attempt }}`, `{{ front_page.title }}`.
- Interpolation is text‑only; values are HTML‑escaped; no expression language.

### 3.3 Link macro

- Form: `[[target]]` or `[[target | label]]`.
- Targets may use a protocol:
  - `a:<artifact-id>` — link to artifact page `/a/<id>` (artifact MUST exist in Index).
  - `repo:<path>[#Lstart[-Lend]]` — link to a repository file (commit‑pinned); MAY include line anchor/range.
  - `src:<path>[#Lstart[-Lend]]` — alias for `repo:`.
  - `doc:#anchor` — link to an in‑document anchor.
  - `gh:issue:<id>` / `gh:pr:<id>` — external links to GitHub issue/PR; rendered as anchors only (no fetch).
  - `ci:run` / `ci:job:<name>` — external links to the CI run URL and job anchors (derived from verified Index `workflow_run`).
- Shorthand path: if `target` matches a path‑like pattern `^[A-Za-z0-9._\-/]+(?:#L\d+(?:-L\d+)?)?$` and no protocol is given, interpret it as `repo:<target>`.
- Labels: if a label is provided (`[[... | label]]`), render it; otherwise derive a default label from the target.
- Invalid targets MUST produce a clear validation error.

---

## 4. Security & Verification

- Worker MUST refuse to render on any parser error or unknown component/attribute.
- Worker MUST NOT fetch or render any resource not listed in the verified Index.
- All text is sanitized; no client scripts are emitted; styles are limited to component attrs.
- Download links MUST point only to verified bytes (commit‑pinned raw URLs or Worker‑proxied verified streams by SHA‑256).

---

## 5. Link Semantics (Artifact & Repo‑aware)

### 5.1 Artifact links

- `[[a:<id>]]` produces a link to the artifact page `/a/<id>` and is valid only if there is an Index `artifacts[]` entry with `id=<id>`.
- `[[a:<id> | Title]]` overrides the label.
- The component `<artifact.link id="..." />` is equivalent to `[[a:...]]` but supports extra attrs like `download=true` to point to the verified bytes directly.

### 5.2 Artifact embeds

- `<artifact.markdown id="..." />`, `<artifact.json id="..." />`, `<artifact.table id="..." />`, `<artifact.summary id="..." />`, `<artifact.image id="..." />`, and `<artifact.viewer id="..." kind="..." />` embed the verified artifact using the mapped viewer from the Index `render` hint + `media_type`.
- The renderer MUST reject unknown `id`s.

### 5.3 Repo links (commit‑pinned)

Proofdown “repo” links are commit‑pinned to avoid drift and MUST resolve to verified bytes.

Two supported resolutions:

1) Per‑file artifacts in Index (recommended)
   - For each file you intend to link, CI emits an `artifacts[]` entry with:
     - `id`: unique (e.g., `src-lib-rs`)
     - `title`: human label
     - `path`: repo‑relative (e.g., `src/lib.rs`)
     - `media_type`: inferred (e.g., `text/plain`)
     - `render`: `repo:file`
     - `sha256`: digest of the exact file bytes at `commit`
   - The link `[[repo:src/lib.rs#L10-L20]]` becomes a deep link to the verified file view; `<repo.code path="src/lib.rs" range="10-20" lang="rust" />` embeds a snippet.
   - The Worker lazily fetches and verifies the bytes against `sha256` (or serves from cache by SHA) before rendering.

2) Source bundle artifact (optional)
   - CI emits a single `artifacts[]` entry representing a source bundle (e.g., zip/tar) with `render: repo:bundle` and `sha256`.
   - The Worker extracts the requested file bytes (subject to size/time limits) to resolve `repo:` links/snippets.
   - This path is heavier and MAY be disabled by deployments.

If neither resolution is present in the Index, any `repo:` link MUST be rejected at render time.

### 5.4 Repo snippet component

- `<repo.code path="<repo-path>" range="start-end" lang="auto|rust|ts|..." highlight="L12,L17-L19" / >`
  - `path`: repo‑relative file path
  - `range`: `start-end` line numbers (1‑based); `start` only shows from line
  - `lang`: explicit or `auto` (language detection MAY be implemented)
  - `highlight`: optional comma‑separated single lines or ranges
- The component resolves only if the file is represented as a verified artifact or bundled as above.

### 5.5 Shorthand repo path links

- `[[src/lib.rs#L10-L20]]` is equivalent to `[[repo:src/lib.rs#L10-L20]]`.
- The shorthand MUST be rejected if the path cannot be resolved via the verified Index strategies in §5.3.

### 5.6 Repository tree

- `<repo.tree path="<dir/>" depth=1..5 include="**/*" exclude="" />` renders a directory listing.
  - `path`: directory prefix to display (must be within repository)
  - `depth`: recursion depth limit
  - `include`/`exclude`: optional glob patterns
- Each entry links to a verified viewer of the file (per‑file artifact) or a snippet view if a bundle is present.
- If the directory cannot be resolved (no per‑file artifacts and no bundle), the component MUST error.

### 5.7 Repository diff (commit‑pinned)

- `<repo.diff base_id="<artifact-id>" head_id="<artifact-id>" path="<path>" context=3 />` renders a unified diff for a single file.
  - `base_id`/`head_id`: artifacts representing source bundles or per‑file artifacts at two points (both MUST be listed in Index with digests; typical `render: repo:bundle`).
  - `path`: repo‑relative file path inside the bundles.
  - `context`: lines of context to show around hunks.
- Large diffs MAY be truncated; a verified download link MUST be provided.
- If either artifact is missing or unverified, the component MUST error.

### 5.8 Symbol links

- Symbol links require a symbols artifact (e.g., ctags/LSIF JSON) listed in the Index with `render: repo:symbols`.
- `[[sym:<path>::<symbol-name>]]` links to the code location for a symbol resolved via the symbols artifact.
- `<repo.symbol path="<path>" name="<symbol-name>" />` renders a link (or snippet, implementation‑defined) to the symbol definition.
- If the symbol cannot be resolved deterministically from the symbols artifact, the link MUST error.

### 5.9 Includes (commit‑pinned partials)

- `<include.pml id="<artifact-id>" />` inlines another Proofdown document.
  - The target MUST be an artifact in the Index with `media_type: text/pml` or an explicit `render: proofdown`.
  - Includes are expanded with a depth limit (e.g., 3) to prevent cycles; cyclic includes MUST error.

### 5.10 External anchors (CI/GitHub)

- `[[ci:run]]` links to `workflow_run.url` from the verified Index.
- `[[ci:job:<name>]]` links to an anchor within the CI UI for the named job (best‑effort; no fetch).
- `[[gh:issue:<id>]]` and `[[gh:pr:<id>]]` produce external anchors; these MUST NOT trigger any network fetch during render.

---

## 6. Components (minimum set)

### 6.1 Structural

- `<grid cols=1..6 gap=0..64> ... </grid>` — responsive grid
- `<section title="..."> ... </section>` — titled block
- `<card title="..."> ... </card>` — card container
- `<tabs>
    <tab title="..."> ... </tab>
    <tab title="..."> ... </tab>
  </tabs>`
- `<gallery cols=2..6> <image .../> ... </gallery>`

### 6.2 Artifact viewers

- `<artifact.summary id="..." />` — test KPIs
- `<artifact.table id="..." />` — coverage table
- `<artifact.json id="..." collapsed=true depth=2 />`
- `<artifact.markdown id="..." />`
- `<artifact.image id="..." alt="..." max_height="480" />`
- `<artifact.viewer id="..." kind="llm-proof" />` — specialized viewer
- `<artifact.link id="..." download=false title="..." />`

### 6.3 Repo viewers

- `<repo.code path="..." range="10-40" lang="rust" highlight="L12,L17-L19" />`
- `<repo.link path="..." label="..." lines="10-40" />` — link to file with optional line range
- `<repo.tree path="src/" depth=2 include="**/*.rs" />` — directory listing
- `<repo.diff base_id="src-bundle-prev" head_id="src-bundle-cur" path="src/lib.rs" context=3 />` — unified diff for a single file
- `<repo.symbol path="src/lib.rs" name="function_name" />` — link/snippet for a symbol (requires symbols artifact)

### 6.4 Presentation & utilities

- `<metric label="Coverage" value="85%" trend="+2%" intent="good|warn|bad" />`
- `<badge kind="pass|fail|warn|info" text="All tests passed" />`
- `<kv key="commit" value="{{ commit }}" />`
- `<toc />` — renders table of contents derived from headings in the current document
- `<artifact.list group="<name>" sort="title|id" />` — list artifacts by Index grouping metadata
- `<include.pml id="partials-release-notes" />` — inline another Proofdown doc from an artifact

---

## 7. Attributes & Types

- Integers: decimal, bounded per component (e.g., `cols=3`, `gap=16`).
- Booleans: `true|false`.
- Enums: validated against a fixed set (e.g., `kind="llm-proof"`).
- Strings: quoted; bare allowed if matches `[A-Za-z0-9._\-/:]+`.
- Unknown/malformed attributes MUST be rejected.
- Glob patterns: components that accept `include`/`exclude` MUST validate patterns against a safe subset (no `..` path traversal; anchored under repo root).
- Enforced bounds: components MUST enforce reasonable bounds (e.g., `repo.tree.depth <= 5`, `repo.code.range` line count <= configured limit).

---

## 8. Styling

- No raw CSS classes or styles.
- Limited styling only via typed attributes documented per component.
- The Worker MAY theme components consistently; components MUST NOT accept arbitrary style strings.

---

## 9. Error Handling & Lints

- Parser errors MUST include line/column and a helpful message.
- Unknown components/attributes MUST be errors (not warnings).
- Invalid links (unknown artifact id; missing repo resolution) MUST be errors.
- Optional lints (SHOULD):
  - Unused components or empty containers.
  - Broken anchors.
  - Overly long code snippets (recommend using fragments route).

---

## 10. Performance & Limits

- Large artifacts/snippets SHOULD be rendered via fragment routes (e.g., `/fragment/{artifact_id}`) to stream on demand.
- The Worker SHOULD cap snippet size and render time; show a truncation notice with a verified download link.

---

## 11. Examples

### 11.1 Front page proof

```pml
# QA Evidence for {{ commit }}

<grid cols=3 gap=16>
  <card title="Tests">
    <artifact.summary id="tests-summary" />
    [[a:coverage | Full Coverage Report]]
  </card>
  <card title="Coverage">
    <artifact.table id="coverage" />
  </card>
  <card title="Key Failures">
    <artifact.markdown id="failures" />
  </card>
</grid>
```

### 11.2 Repo links and snippets

```pml
<section title="Critical Code Path">
  [[repo:src/lib.rs#L10-L42 | src/lib.rs (L10–L42)]]
  <repo.code path="src/lib.rs" range="10-42" lang="rust" highlight="L12,L17-L19" />
</section>
```

### 11.3 Artifact download link

```pml
<card title="Raw Evidence">
  <artifact.link id="tests-summary" download=true title="Download tests-summary.json" />
</card>
```

### 11.4 Shorthand repo link and tree

```pml
[[src/main.rs#L1-L30 | main.rs (L1–L30)]]
<repo.tree path="src/" depth=2 include="**/*.rs" />
```

### 11.5 Repo diff (commit‑pinned)

```pml
<repo.diff base_id="src-bundle-prev" head_id="src-bundle-cur" path="src/lib.rs" context=3 />
```

### 11.6 Symbol link

```pml
[[sym:src/lib.rs::init]]
<repo.symbol path="src/lib.rs" name="init" />
```

### 11.7 Includes and TOC

```pml
<toc />
<include.pml id="partials-release-notes" />
```

### 11.8 Presentation widgets

```pml
<metric label="Coverage" value="85%" trend="+2%" intent="good" />
<badge kind="pass" text="All tests passed" />
<kv key="commit" value="{{ commit }}" />
```

---

## 12. Implementation Notes (reference)

- Parser: Rust crate `proofdown_parser` produces a stable AST; bindings via `wasm-bindgen`.
- Validation: performed against the verified Index in the Worker (or WASM helper from `index_contract`).
- Rendering: pure functions turn AST + verified context → HTML; no side effects.
- Optional viewers (repo.tree, repo.diff, repo.symbol) depend on presence of corresponding artifacts (`repo:file`, `repo:bundle`, `repo:symbols`) and MAY be feature‑flagged.

---

## 13. Compatibility & Evolution

- New components/viewers MAY be added in a backward‑compatible manner.
- Breaking grammar/semantics changes MUST be gated by Index `version` and rejected if unknown.

---

## 14. Grammar (informal EBNF)

```
document      = { block } ;

block         = heading | paragraph | list | code_fence | component ;

heading       = ( "#" | "##" | "###" | "####" ), space, text, newline ;
paragraph     = textline, { textline }, blankline ;
list          = listitem, { listitem } ;
listitem      = ( "-" | "*" | digit, "." ), space, text, newline ;
code_fence    = "```", lang, newline, { anyline }, "```", newline ;

component     = open_tag, { block | text | inline_code }, ( close_tag | self_close ) ;
open_tag      = "<", name, { space, attribute }, ">" ;
self_close    = space?, "/" , ">" ;
close_tag     = "</", name, ">" ;

attribute     = key, "=", value ;
name          = lc_alpha, { lc_alpha | digit | "." | "-" } ;
key           = lc_alpha, { lc_alpha | digit | "-" | "_" } ;
value         = quoted | bare ;
bare          = 1*( ALNUM | "_" | "-" | "/" | ":" | "." ) ;
quoted        = '"', { anychar - '"' }, '"' ;

inline_code   = "`", { anychar - "`" }, "`" ;
textline      = { text | link_macro | inline_code }, newline ;
link_macro    = "[[", target, ( "|", label )?, "]]" ;
target        = see Link Macro ABNF ;

space         = 1*( SP ) ;
newline       = ( CR? LF ) ;
blankline     = newline, { newline } ;
```

Notes

- Parser MUST reject unknown components/attrs; attributes are validated post-parse by schema.
- Whitespace normalization and HTML escaping are performed at render time.

---

## 15. Link Macro (ABNF)

```
link      = "[[" SP* target SP* ( "|" SP* label SP* )? "]]"
label     = 1*( %x21-7E ) ;  ; printable ASCII (trimmed)

target    = a_target / repo_target / src_target / doc_target / gh_target / ci_target / sym_target / path_shorthand

a_target      = "a:" id
repo_target   = "repo:" path [ linefrag ]
src_target    = "src:" path [ linefrag ]
doc_target    = "doc:#" anchor
gh_target     = "gh:" ( "issue:" 1*DIGIT / "pr:" 1*DIGIT )
ci_target     = "ci:" ( "run" / "job:" jobname )
sym_target    = "sym:" path "::" symbol

path_shorthand = path [ linefrag ] ; interpreted as repo:<path>

id         = 1*( ALPHA / DIGIT / "-" / "_" )
path       = 1*( ALPHA / DIGIT / "/" / "." / "-" / "_" )
anchor     = 1*( ALPHA / DIGIT / "-" / "_" )
jobname    = 1*( ALPHA / DIGIT / "-" / "_" )
symbol     = 1*( VCHAR )
linefrag   = "#L" 1*DIGIT [ "-L" 1*DIGIT ]
```

Semantics

- `path_shorthand` MUST validate under §5.3 resolution rules.
- Invalid targets are hard errors.

---

## 16. Component Registry and Bounds (v1)

Structural

- `grid(cols=1..6, gap=0..64)`
- `section(title: string)`
- `card(title: string)`
- `tabs` with nested `tab(title: string)`
- `gallery(cols=2..6)` with `image` children

Artifact viewers

- `artifact.summary(id)` — expects `summary:test` JSON; errors on missing fields
- `artifact.table(id)` — expects coverage JSON; sortable
- `artifact.json(id, collapsed=true|false, depth=0..8)`
- `artifact.markdown(id)`
- `artifact.image(id, alt, max_height=128..2048)`
- `artifact.viewer(id, kind: enum)`
- `artifact.link(id, download=true|false, title)`

Repo viewers

- `repo.code(path, range, lang, highlight)` — range line count SHOULD be ≤ 400
- `repo.link(path, label, lines)`
- `repo.tree(path, depth=1..5, include, exclude)`
- `repo.diff(base_id, head_id, path, context=0..10)`
- `repo.symbol(path, name)`

Utilities

- `metric(label, value, trend, intent=good|warn|bad)`
- `badge(kind=pass|fail|warn|info, text)`
- `kv(key, value)`
- `toc()`
- `include.pml(id)` — include depth ≤ 3

All unknown attributes MUST error. Bounds are enforced at validation time.

---

## 17. Viewer Input Shapes (reference)

`summary:test` JSON (minimum)

```
{
  "total": 120,
  "passed": 118,
  "failed": 2,
  "duration_seconds": 45.6
}
```

`table:coverage` JSON (minimum)

```
{
  "total": { "pct": 85.2 },
  "files": [
    { "path": "src/lib.rs", "pct": 92.1 },
    { "path": "src/main.rs", "pct": 78.0 }
  ]
}
```

---

## 18. Include Rules

- Includes expand within the same verification context; no network fetches.
- Maximum include depth is 3; cycles MUST error with a clear message showing the path.
- Variables from the manifest (e.g., `{{ commit }}`) are available in included documents; no user-defined variables.
