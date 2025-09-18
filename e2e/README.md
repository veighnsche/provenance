# E2E (Cypress) for Provenance Site

This directory contains Cypress end-to-end tests against the generated static site.

## Prerequisites

- Node.js 18+
- Rust toolchain (for building the site)

## How it works

- We generate the example site for `examples/minimal/` via a helper script.
- We serve the generated static site with a local HTTP server.
- Cypress runs headless tests against `http://localhost:5173`.

## Commands

Install dependencies:

```bash
cd e2e
npm install
```

Run E2E tests headless:

```bash
npm run e2e
```

Open Cypress in interactive mode:

```bash
npm run e2e:open
```

## What’s tested (initial)

- Home page renders and shows the Artifacts section.
- First artifact link navigates to its detail page with a badge and a download link.
- Artifacts index shows at least one row.

## Notes

- The site is built from `examples/minimal`. For other test fixtures, adjust `scripts/build-site.js` to point to different roots or add flags.
- CI integration can run `npm ci && npm run e2e` on Ubuntu with Xvfb or `cypress run --headless` (Chrome).
