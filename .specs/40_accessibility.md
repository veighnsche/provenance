# Accessibility & UX Checklist — Provenance (v1‑draft)

This checklist defines minimum accessibility requirements for the Provenance UI and viewers.

Related: `./01_proofdown.md`, `./10_human_contract.md`.

---

## 0. Goals

- Evidence must be reviewable by everyone. Meet baseline a11y standards without compromising safety.

---

## 1. Structure & Semantics

- Use semantic headings (`h1`..`h4`) matching sections in Proofdown.
- Provide `<nav>` landmarks for global and in‑page navigation (e.g., table of contents).
- Tables (coverage) must include `<thead>`, `<tbody>`, and scope attributes.

---

## 2. Keyboard Navigation

- All interactive elements are keyboard accessible in a logical tab order.
- Focus styles are visible and meet contrast guidelines.
- Expand/collapse controls in JSON/fragment viewers are operable via keyboard.

---

## 3. Color & Contrast

- Text and icons meet WCAG AA contrast.
- Do not use color alone to convey status; include labels or icons (e.g., pass/fail badges with text).

---

## 4. Images and Media

- All images require `alt` text; decorative images use empty `alt`.
- Large media must provide a truncation banner and a verified download link.

---

## 5. Messaging & Errors

- Error banners are descriptive, include cause (signature/digest failure) and next steps.
- Truncation notices specify limits and provide the exact verified download.

---

## 6. Components Guidance

- `artifact.json`: keyboard toggles, aria-expanded, collapsed by default on large structures.
- `artifact.table`: sortable columns announced to screen readers.
- `repo.code`: line numbers exposed to assistive tech; highlight ranges indicated non‑color‑only.
- `tabs`: proper ARIA roles and relationships.

---

## 7. Testing

- Run automated a11y checks (e.g., axe) on `/` and key artifact views.
- Include keyboard‑only navigation verification in test plans.
