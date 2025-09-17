Feature: Renderers produce safe, deterministic HTML
  As a viewer of artifacts
  I want renderers to be safe and predictable
  So that outputs are deterministic and readable

  Background:
    Given the repo root is "examples/minimal"

  Scenario: Markdown renders headings and escapes ampersand
    When I render "markdown" from "ci/tests/failures.md"
    Then the rendered HTML should contain "<h1>"
    And the rendered HTML should contain "&amp;"

  Scenario: JSON pretty renderer escapes special characters
    When I render "json" from "ci/tests/summary.json"
    Then the rendered HTML should contain "&lt;&gt;&amp;"

  Scenario: Coverage table contains rows and total
    When I render "table:coverage" from "ci/coverage/coverage.json"
    Then the rendered HTML should contain "Total"
    And the rendered HTML should contain "src/lib.rs"

  Scenario: Image renderer produces <img> with alt text
    When I render an image with src "/assets/tests-summary/summary.json" and alt "Summary"
    Then the rendered HTML should contain "<img"
    And the rendered HTML should contain "alt=\"Summary\""
