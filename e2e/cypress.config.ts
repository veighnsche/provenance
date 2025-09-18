import { defineConfig } from 'cypress'

export default defineConfig({
  e2e: {
    baseUrl: 'http://localhost:5173',
    specPattern: 'cypress/e2e/**/*.cy.{ts,tsx,js,jsx}',
    supportFile: 'cypress/support/e2e.ts',
    video: false,
    screenshotsFolder: 'cypress/screenshots',
    downloadsFolder: 'cypress/downloads',
  },
});
