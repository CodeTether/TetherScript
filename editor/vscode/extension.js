const vscode = require('vscode');
const { LanguageClient, TransportKind } = require('vscode-languageclient/node');
const { registerCommands } = require('./lib/commands');
const { registerCompletions } = require('./lib/completions');
const { registerHovers } = require('./lib/hovers');

let client;

function activate(context) {
  const config = vscode.workspace.getConfiguration('tetherscript');
  const command = config.get('serverPath') || 'tetherscript';

  const serverOptions = {
    run: { command, args: ['lsp'], transport: TransportKind.stdio },
    debug: { command, args: ['lsp'], transport: TransportKind.stdio },
  };

  const clientOptions = {
    documentSelector: [{ scheme: 'file', language: 'tetherscript' }],
    synchronize: {
      fileEvents: vscode.workspace.createFileSystemWatcher('**/*.{tether,kl}'),
    },
  };

  registerCommands(vscode, context);
  registerCompletions(vscode, context);
  registerHovers(vscode, context);

  client = new LanguageClient('tetherscript', 'tetherscript', serverOptions, clientOptions);
  client.start();
}

function deactivate() {
  return client ? client.stop() : undefined;
}

module.exports = { activate, deactivate };
