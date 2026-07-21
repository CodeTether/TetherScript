const vscode = require('vscode');
const { registerCommands } = require('./lib/commands');
const { registerCompletions } = require('./lib/completions');
const { registerSqlCompletions } = require('./lib/sql-completions');
const { registerHovers } = require('./lib/hovers');
const { registerModuleCompletions } = require('./lib/module-completions');
const { registerModuleDiagnostics } = require('./lib/module-diagnostics-register');
const { registerModuleHovers } = require('./lib/module-hovers');
const { registerModuleNavigation } = require('./lib/module-navigation-register');
const { registerSqlHovers } = require('./lib/sql-hovers');
const { createLspController } = require('./lib/lsp');
const { registerNavigation } = require('./lib/navigation');

let lsp;

function activate(context) {
  lsp = createLspController(vscode);
  registerCommands(vscode, context);
  registerCompletions(vscode, context);
  registerModuleCompletions(vscode, context);
  registerModuleDiagnostics(vscode, context);
  registerSqlCompletions(vscode, context);
  registerHovers(vscode, context);
  registerModuleHovers(vscode, context);
  registerSqlHovers(vscode, context);
  registerNavigation(vscode, context);
  registerModuleNavigation(vscode, context);
  lsp.register(context);

  void lsp.start(false);
}

function deactivate() {
  return lsp ? lsp.stop(false) : undefined;
}

module.exports = { activate, deactivate };