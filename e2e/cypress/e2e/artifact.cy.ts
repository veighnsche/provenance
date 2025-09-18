describe('Artifact pages', () => {
  const known = [
    { id: 'tests-summary', title: 'Test Summary' },
    { id: 'coverage', title: 'Coverage' },
    { id: 'failures', title: 'Failing Specs' },
  ] as const;

  known.forEach(({ id, title }) => {
    it(`renders artifact ${id}`, () => {
      cy.visit(`/a/${id}/`);
      cy.get('header h1').should('have.text', title);
      cy.get('header .muted').should('contain.text', id);
      cy.contains('a', /Download raw|Download/i)
        .should('have.attr', 'href')
        .and('match', new RegExp(`/assets/${id}/`));
      cy.get('article').should('exist');
    });
  });

  it('breadcrumb nav exists and links home', () => {
    cy.visit('/a/tests-summary/');
    cy.get('nav[aria-label="Breadcrumb"]').within(() => {
      cy.contains('a', 'Home').should('have.attr', 'href', '/index.html');
    });
  });

  it('coverage artifact shows table rows', () => {
    cy.visit('/a/coverage/');
    cy.get('article table').should('exist');
    cy.get('article table tbody tr').its('length').should('be.gte', 1);
  });

  it('markdown artifact renders content', () => {
    cy.visit('/a/failures/');
    cy.get('article').should('contain.text', 'Failing Specs');
  });
});
