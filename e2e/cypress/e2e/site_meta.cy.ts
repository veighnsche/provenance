describe('Site metadata and indices', () => {
  it('exposes robots.txt with fragment disallow', () => {
    cy.request('/robots.txt').then((res) => {
      expect(res.status).to.eq(200);
      expect(res.headers['content-type']).to.match(/text\/plain/);
      expect(res.body).to.contain('Disallow: /fragment/');
    });
  });

  it('exposes search_index.json with known items', () => {
    cy.request('/search_index.json').then((res) => {
      expect(res.status).to.eq(200);
      expect(res.headers['content-type']).to.match(/application\/json/);
      const data = res.body as Array<any>;
      expect(Array.isArray(data)).to.be.true;
      const ids = new Set(data.map((x) => x.id));
      expect(ids).to.include('tests-summary');
      expect(ids).to.include('coverage');
      expect(ids).to.include('failures');
    });
  });
});
