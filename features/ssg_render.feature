Feature: Static site generation (RSX, read-only)
  The SSG builds a deterministic, read-only site from a verified manifest and artifacts.

  Background:
    Given a repository with examples at "examples/minimal/"
    And a manifest at "examples/minimal/.provenance/manifest.json"

  Scenario: Build produces index and per-artifact pages
    When I run the SSG with --root examples/minimal --out site
    Then the file "site/index.html" exists
    And the file "site/a/tests-summary/index.html" exists
    And the file "site/a/coverage/index.html" exists
    And the file "site/a/failures/index.html" exists
    And the file "site/robots.txt" exists

  Scenario: Digest verification banner is shown
    Given an artifact with an incorrect sha256 in the manifest
    When I build the site
    Then the artifact page contains the text "digest mismatch"

  Scenario: Deterministic output across runs
    When I build the site twice with the same inputs
    Then the byte contents of "site/index.html" are identical between runs

  Scenario: Large artifacts are truncated with a notice
    Given a large JSON artifact exceeding the configured size limit
    When I build the site
    Then the artifact page shows a truncation notice
    And provides a verified "Download" link
