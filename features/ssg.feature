Feature: Static Site Generation (SSG)
  As a user of provenance-ssg
  I want a deterministic site generator
  So that verified artifacts render to stable HTML

  Background:
    Given the repo root is "examples/minimal"
    And the manifest path is ".provenance/manifest.json"
    And the JSON schema path is "schemas/manifest.schema.json"

  Scenario: Generate minimal site (Proofdown disabled)
    When I run the SSG with output dir "TMP"
    Then file "index.html" should exist
    And file "a/tests-summary/index.html" should exist

  Scenario: Truncate inline rendering for large inputs
    Given file "ci/tests/large.json" exists
    When I run the SSG with output dir "TMP" and truncate inline bytes to 1
    Then HTML at "a/failures/index.html" should contain "Truncated"

  Scenario: Badges are written
    When I run the SSG with output dir "TMP"
    Then file "badge/provenance.json" should exist
    And file "badge/tests.json" should exist
    And file "badge/coverage.json" should exist
