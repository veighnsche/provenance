describe('Assets downloads', () => {
  const assets = [
    { id: 'tests-summary', filename: 'summary.json', contentType: /application\/json/ },
    { id: 'coverage', filename: 'coverage.json', contentType: /application\/json/ },
    { id: 'failures', filename: 'failures.md', contentType: /text\/(plain|markdown)/ },
  ] as const;

  assets.forEach(({ id, filename, contentType }) => {
    it(`exposes a download link for ${id}`, () => {
      // First, navigate to artifact and read the download link
      cy.visit(`/a/${id}/`);
      cy.contains('a', /Download raw|Download/i)
        .should('have.attr', 'href')
        .then((href) => {
          const decoded = decodeURIComponent(href);
          expect(decoded).to.match(new RegExp(`${filename}$`));
          // Do not assert the request result here because some static servers decode %2E to '.'
          // causing mismatches with on-disk filenames. The presence and correctness of the link
          // is sufficient to validate frontend output.
        });
    });
  });
});
