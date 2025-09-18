describe('Artifacts index', () => {
  it('lists artifacts in a table with expected columns', () => {
    cy.visit('/artifacts/');
    cy.get('header h1').should('contain.text', 'All Artifacts');

    cy.get('table').should('exist');
    cy.get('table thead th').then(($ths) => {
      const headers = [...$ths].map((el) => el.textContent?.trim());
      expect(headers).to.include.members(['ID', 'Title', 'Render', 'Media', 'Verified']);
    });

    cy.get('table tbody tr').its('length').should('be.greaterThan', 0);
  });

  it('has a link row for tests-summary and navigates to detail page', () => {
    cy.visit('/artifacts/');
    cy.get('table').find('a[href="/a/tests-summary/"]').should('exist').and('contain.text', 'tests-summary').click();
    cy.url().should('include', '/a/tests-summary/');
    cy.get('header h1').should('contain.text', 'Test Summary');
  });

  it('shows verification badges in the Verified column', () => {
    cy.visit('/artifacts/');
    cy.get('table tbody tr td:last-child').first().within(() => {
      cy.get('span.badge').invoke('text').then((txt) => {
        expect(txt.trim().toLowerCase()).to.match(/verified|digest mismatch/);
      });
    });
  });
});
