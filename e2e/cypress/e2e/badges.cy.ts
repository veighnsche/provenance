describe('Badges endpoints', () => {
  const kinds = ['provenance', 'tests', 'coverage'] as const;

  kinds.forEach((k) => {
    it(`badge ${k} JSON exists and has schema keys`, () => {
      cy.request(`/badge/${k}.json`).then((res) => {
        expect(res.status).to.eq(200);
        expect(res.headers['content-type']).to.match(/application\/json/);
        expect(res.body).to.have.property('schemaVersion');
        expect(res.body).to.have.property('label');
        expect(res.body).to.have.property('message');
      });
    });

    it(`badge ${k} SVG exists`, () => {
      cy.request(`/badge/${k}.svg`).then((res) => {
        expect(res.status).to.eq(200);
        expect(res.headers['content-type']).to.match(/image\/svg\+xml/);
        expect(res.body).to.contain('<svg');
      });
    });
  });
});
