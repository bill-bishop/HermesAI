module.exports = function (config) {
  config.set({
    browsers: ['ChromeHeadlessNoSandbox'],
    customLaunchers: {
      ChromeHeadlessNoSandbox: {
        base: 'ChromeHeadless',
        flags: [
          '--no-sandbox',
          '--disable-gpu',
          '--disable-dev-shm-usage',
          '--disable-setuid-sandbox'
        ]
      }
    },
    singleRun: true, // run once then exit (non-interactive mode)
    // ... keep your other settings here (frameworks, reporters, etc.)
  });
};
