Feature: Badges generation (static JSON/SVG)
  The system produces badges derived only from verified inputs.

  Background:
    Given a valid, signed manifest and verified artifacts for tests and coverage

  Scenario: Provenance badge is "verified" when signature passes
    When I generate badges
    Then the file "site/badge/provenance.json" exists
    And the JSON has schemaVersion 1
    And the JSON message contains "verified"

  Scenario: Tests badge summarizes pass/fail counts
    When I generate badges
    Then the file "site/badge/tests.json" exists
    And the JSON message contains "passed"

  Scenario: Coverage badge reports percentage with thresholds
    When I generate badges
    Then the file "site/badge/coverage.json" exists
    And the JSON message matches "[0-9]+(\.[0-9]+)?%"

  Scenario: Missing artifact yields error badge
    Given the coverage artifact is missing
    When I generate badges
    Then the JSON badge for coverage has message "error"
    And uses color "red"
