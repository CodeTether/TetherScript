const vscode = require('vscode');
const { registerCommands } = require('./lib/commands');
const { registerCompletions } = require('./lib/completions');
const { registerSqlCompletions } = require('./lib/sql-completions');
const { registerHovers } = require('./lib/hovers');
const { registerSqlHovers } = require('./lib/sql-hovers');
const { createLspController } = require('./lib/lsp');
const { registerNavigation } = require('./lib/navigation');

let lsp;

function activate(context) {
  lsp = createLspController(vscode);
  registerCommands(vscode, context);
  registerCompletions(vscode, context);
  registerSqlCompletions(vscode, context);
  registerHovers(vscode, context);
  registerSqlHovers(vscode, context);
  registerNavigation(vscode, context);
  lsp.register(context);

  void lsp.start(false);
}

function deactivate() {
  return lsp ? lsp.stop(false) : undefined;
}

module.exports = { activate, deactivate };
