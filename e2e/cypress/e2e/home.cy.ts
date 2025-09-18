describe('Home page', () => {
  it('shows header with commit and KPI cards', () => {
    cy.visit('/');

    // Header contains commit from manifest (generic h1 to support both renderers)
    cy.get('h1').first().should('contain.text', 'deadbeef');

    // There should be at least one card grid (KPI or Proofdown grid)
    cy.get('.cards .card').its('length').should('be.gte', 1);
  });

  it('lists featured artifacts with links', () => {
    cy.visit('/');
    // If the "Artifacts" section exists (frontend fallback), validate its links
    cy.get('body').then(($body) => {
      const artifactsH2 = [...$body.find('h2')].find((el) => (el.textContent || '').includes('Artifacts'));
      if (artifactsH2) {
        const $h2 = Cypress.$(artifactsH2);
        cy.wrap($h2).next('.cards').as('grid');
        cy.get('@grid').find('.card').its('length').should('be.gte', 3);
        cy.get('@grid').find('a[href^="/a/"]').should('exist');
        cy.get('@grid').find('a').contains(/Download/i).should('exist');
      } else {
        // Otherwise (external_pml front page), just ensure the page presents cards
        cy.get('.cards .card').its('length').should('be.gte', 1);
      }
    });
  });

  it('has a skip link and main landmark', () => {
    cy.visit('/');
    cy.get('a.skip-link').should('have.attr', 'href', '#main');
    cy.get('main#main').should('exist');
  });
});
