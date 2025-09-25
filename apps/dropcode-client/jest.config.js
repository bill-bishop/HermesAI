module.exports = {
  transform: {
    '^.+\\.(ts|tsx)$': 'ts-jest',
    '^.+\\.(mjs|js)$': 'babel-jest',
    '^.+\\.(html)$': 'ts-jest',
  },
  testEnvironment: 'jsdom',
  roots: ['<rootDir>/src/'],
  moduleFileExtensions: ['ts', 'tsx', 'js', 'jsx', 'json', 'node', 'mjs'],
  setupFilesAfterEnv: ['<rootDir>/src/setup-jest.ts'],
  verbose: true,
  transformIgnorePatterns: [
    'node_modules/(?!@angular|rxjs)'
  ],
  moduleNameMapper: {
    '\\.(html|scss)$': 'jest-transform-stub',
  },
};