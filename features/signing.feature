Feature: Manifest signing and verification
  To ensure integrity, the manifest is signed over canonical bytes with Ed25519.

  Background:
    Given a canonicalized manifest at ".provenance/manifest.json"
    And a Base64 signature at ".provenance/manifest.json.sig"

  Scenario: Signature verification succeeds with correct key
    Given the public key INDEX_PUBKEY_ED25519 for the manifest
    When I verify the signature against the canonical manifest bytes
    Then the verification result is "ok"

  Scenario: Signature verification fails with wrong key
    Given a different public key
    When I verify the signature
    Then the verification result is "error"
    And the error contains "signature mismatch"

  Scenario: Signature fails if manifest bytes change
    Given I change one character in the manifest
    When I canonicalize and verify
    Then the verification result is "error"
