module.exports = {
  transform: {
    '^.+\\.(ts|tsx)$': 'babel-jest',
    '^.+\\.(js|jsx)$': 'babel-jest',
    '^.+\\.(mjs)$': 'babel-jest',
  },
  testEnvironment: 'jsdom',
  roots: ['<rootDir>/src/'],
  moduleFileExtensions: ['ts', 'tsx', 'js', 'jsx', 'json', 'node', 'mjs'],
  setupFilesAfterEnv: ['<rootDir>/src/setup-jest.ts'],
  verbose: true,
  transformIgnorePatterns: [
    'node_modules/(?!@angular|@testing-library)'
  ],
};