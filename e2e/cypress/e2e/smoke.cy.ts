describe('Provenance site smoke tests', () => {
  it('loads the home page', () => {
    cy.visit('/index.html');
    cy.get('h1').first().should('be.visible');
    cy.get('a.skip-link').should('have.attr', 'href', '#main');
    cy.get('main#main').should('exist');
  });

  it('navigates to an artifact page and shows badges', () => {
    cy.visit('/a/tests-summary/');
    cy.get('h1').should('exist');
    cy.get('span.badge').invoke('text').then((txt) => {
      expect(txt.trim().toLowerCase()).to.match(/verified|digest mismatch/);
    });
    cy.contains(/Download( raw)?/).should('exist');
  });

  it('artifacts index lists entries', () => {
    cy.visit('/artifacts/');
    cy.get('table').should('exist');
    cy.get('table tbody tr').its('length').should('be.greaterThan', 0);
  });
});
