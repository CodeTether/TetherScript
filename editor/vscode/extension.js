const vscode = require('vscode');
const { registerCommands } = require('./lib/commands');
const { registerCompletions } = require('./lib/completions');
const { registerHovers } = require('./lib/hovers');
const { createLspController } = require('./lib/lsp');
const { registerNavigation } = require('./lib/navigation');

let lsp;

function activate(context) {
  lsp = createLspController(vscode);
  registerCommands(vscode, context);
  registerCompletions(vscode, context);
  registerHovers(vscode, context);
  registerNavigation(vscode, context);
  lsp.register(context);

  void lsp.start(false);
}

function deactivate() {
  return lsp ? lsp.stop(false) : undefined;
}

module.exports = { activate, deactivate };
