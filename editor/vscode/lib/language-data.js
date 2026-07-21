const { assertions } = require('./tool-data-assertions');
const { browser } = require('./tool-data-browser');
const { core } = require('./tool-data-core');
const { data } = require('./tool-data-data');
const { files } = require('./tool-data-files');
const { network } = require('./tool-data-network');
const { system } = require('./tool-data-system');
const { terminal } = require('./tool-data-terminal');
const { constants, keywords } = require('./language-words');
const { methods } = require('./method-data');
const { factories } = require('./resource-factory-data');

const builtins = {
  ...assertions, ...browser, ...core, ...data, ...files,
  ...network, ...system, ...terminal,
};
const namespaces = {
  resource: ['resource', 'Create move-only host resources with explicit lifecycle controls.'],
};

module.exports = { builtins, constants, factories, keywords, methods, namespaces };
