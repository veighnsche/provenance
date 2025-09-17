Feature: Provenance Manifest validity and canonicalization
  The manifest describes a commit’s artifacts and front page. It must validate
  against schema, canonicalize deterministically, and be safe to sign.

  Background:
    Given a repository with a file ".provenance/manifest.json"

  Scenario: Valid manifest passes schema
    When I validate the manifest against schemas/manifest.schema.json
    Then the validation result is "ok"

  Scenario: Invalid manifest is rejected
    Given I modify the manifest to remove the field "commit"
    When I validate the manifest against schemas/manifest.schema.json
    Then the validation result is "error"
    And the error contains "required property"

  Scenario: Canonicalization is stable
    Given two logically equivalent manifest documents with shuffled object keys
    When I canonicalize both manifests to bytes
    Then the resulting byte arrays are identical

  Scenario: Artifact ids must be unique
    Given I duplicate an artifact with the same id
    When I validate the manifest
    Then I see an error mentioning "id" and "unique"
