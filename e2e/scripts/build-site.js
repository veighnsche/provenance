#!/usr/bin/env node
const { spawnSync } = require('node:child_process');
const { resolve } = require('node:path');

// Build the minimal example site into examples/minimal/site
const repoRoot = resolve(__dirname, '..', '..');
const exampleRoot = resolve(repoRoot, 'examples', 'minimal');

function run(cmd, args, cwd) {
  const res = spawnSync(cmd, args, { cwd, stdio: 'inherit' });
  if (res.status !== 0) {
    process.exit(res.status || 1);
  }
}

// Ensure Rust workspace builds the SSG and runs the generation for the example
console.log('Building the site for examples/minimal ...');
run('cargo', ['run', '-p', 'provenance_ssg', '--quiet', '--', '--root', exampleRoot, '--out', 'site', '--schema-path', resolve(repoRoot, 'schemas', 'manifest.schema.json')], repoRoot);
console.log('Done.');
