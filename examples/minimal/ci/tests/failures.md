# Failing Specs

- api: returns 500 on bad input
  - steps: send malformed JSON
  - expected: 400 with validation errors
  - actual: 500 internal server error
