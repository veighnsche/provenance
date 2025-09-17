Feature: Proofdown front page rendering
  The front page is authored in Proofdown and rendered to static RSX/HTML.

  Background:
    Given a front page file at "examples/minimal/ci/front_page.pml"

  Scenario: Structural components are rendered
    Given the front page contains a grid with three cards
    When I build the site with the SSG
    Then "site/index.html" includes the titles "Tests", "Coverage", and "Failures"

  Scenario: Artifact components embed verified data
    Given the front page uses <artifact.summary id="tests-summary" />
    When I build the site
    Then "site/index.html" shows total/passed/failed counts from the tests summary

  Scenario: Unknown components are errors
    Given the front page contains an unknown component <foo.bar/>
    When I build the site
    Then the build fails with an error mentioning "unknown component"

  Scenario: Includes have bounded depth and no cycles
    Given the front page includes another Proofdown file via <include.pml id="..."/>
    And includes are nested deeper than the allowed limit
    When I build the site
    Then the build fails with an error mentioning "include depth"
